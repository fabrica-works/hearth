//! A bounded postfix value stack (render.md, 8.3; format.md, 7.4 to 7.7).
pub mod context;
mod lookup;
pub mod operation;
pub mod value;
use crate::limits::Limits;
pub use context::{Binding, Bindings, Context};
pub use operation::{Expression, Missing, Op, Operation};
use skein_lib::{List, Stack, bytes};
pub use value::{Entry, Operand, Value};
#[derive(Debug)]
pub struct Evaluator<'c> {
    values: Stack<Operand<'c>>,
    nesting: u32,
}
impl Evaluator<'_> {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self { values: Stack::with_capacity(limits.expression), nesting: limits.nesting }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Budget {
    left: u64,
}
impl Budget {
    #[must_use]
    pub const fn new(limits: &Limits) -> Self {
        Self { left: limits.made }
    }
    pub(super) fn charge(&mut self, bytes: u64) -> Result<(), FaultKind> {
        self.left = self.left.checked_sub(bytes).ok_or(FaultKind::Limit)?;
        Ok(())
    }
}
pub const ELEMENT: u64 = 32;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Fault {
    pub at: u32,
    pub kind: FaultKind,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FaultKind {
    Unknown,
    NoKey,
    NoIndex,
    Undefined,
    Type,
    Overflow,
    Empty,
    Path,
    Output,
    Limit,
}
pub fn expression<'c>(
    evaluator: &mut Evaluator<'c>,
    expression: &Expression,
    context: &Context<'c>,
    bindings: &Bindings<'c>,
    budget: &mut Budget,
) -> Result<Operand<'c>, Fault> {
    for _ in 0..evaluator.values.len() {
        drop(evaluator.values.pop());
    }
    for operation in &expression.operations {
        let result = evaluator.apply(&operation.op, context, bindings, budget);
        match result {
            Ok(value) => {
                if evaluator.values.push(value).is_err() {
                    return Err(Fault { at: operation.at, kind: FaultKind::Limit });
                }
            }
            Err(kind) => return Err(Fault { at: operation.at, kind }),
        }
    }
    let value = evaluator.values.pop().expect("compiled expression leaves a value");
    assert!(evaluator.values.is_empty(), "compiled expression leaves exactly one value");
    Ok(value)
}
impl<'c> Evaluator<'c> {
    fn pop(&mut self) -> Operand<'c> {
        self.values.pop().expect("compiled operation has its operands")
    }
    fn apply(
        &mut self,
        op: &Op,
        context: &Context<'c>,
        bindings: &Bindings<'c>,
        budget: &mut Budget,
    ) -> Result<Operand<'c>, FaultKind> {
        Ok(match op {
            Op::String(v) => {
                budget.charge(u64::try_from(v.len()).expect("bytes fit"))?;
                Operand::String(bytes::copy_of(v))
            }
            Op::Integer(v) => Operand::Int(*v),
            Op::Boolean(v) => Operand::Bool(*v),
            Op::Name { name, missing: _ } => match name.as_ref() {
                b"target" => Operand::Value(context.target),
                b"debian" => Operand::Value(context.debian),
                b"settings" => Operand::Value(context.settings),
                b"tasks" => Operand::Value(context.tasks),
                b"exports" => Operand::Value(context.exports),
                _ => bindings.find(name, budget, self.nesting)?,
            },
            Op::Key { name, missing } => lookup::key(self.pop(), name, *missing)?,
            Op::Item { missing } => {
                let index = self.pop();
                lookup::item(self.pop(), index, *missing)?
            }
            Op::List { length } => self.list(*length, budget)?,
            Op::Defined => Operand::Bool(!self.pop().undefined()),
            Op::NotDefined => Operand::Bool(self.pop().undefined()),
        })
    }
    fn list(&mut self, length: u32, budget: &mut Budget) -> Result<Operand<'c>, FaultKind> {
        budget.charge(u64::from(length).saturating_mul(ELEMENT))?;
        let mut reverse = Stack::with_capacity(length);
        let mut depth = 1_u32;
        for _ in 0..length {
            let value = self.pop();
            match &value {
                Operand::Undefined => return Err(FaultKind::Undefined),
                Operand::List { depth: inner, items: _ } => depth = depth.max(inner.saturating_add(1)),
                Operand::Value(_) | Operand::Bool(_) | Operand::Int(_) | Operand::String(_) => {}
            }
            reverse.push(value).expect("length reserved");
        }
        if depth > self.nesting {
            return Err(FaultKind::Limit);
        }
        let mut items = List::with_capacity(length);
        for _ in 0..length {
            items.push(reverse.pop().expect("length pushed")).expect("length reserved");
        }
        Ok(Operand::List { depth, items: items.into_boxed() })
    }
}
