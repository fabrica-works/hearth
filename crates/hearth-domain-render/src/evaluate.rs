//! A bounded postfix value stack (domain/render.md, 8.3; protocol/format.md, 7.4 to 7.7).
pub mod context;
mod escape;
pub mod filter;
mod frame;
pub mod template;
pub use filter::Filter;
mod lookup;
mod operators;
mod walk;
pub use operators::{Written, output, truthy};
pub mod operation;
pub mod value;
use crate::limits::Limits;
pub use context::{Binding, Bindings, Context};
pub use operation::{Expression, Missing, Op, Operation};
use skein_lib::{List, Stack, bytes};
pub use value::{Entry, Operand, Value};
/// The most heap an evaluator, its temporary storage and a spent budget hold.
/// Returns `None` when the bound does not fit a `u64`.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let depth = limits.depth.saturating_add(limits.nesting);
    // Walks and binding copies run one at a time, above the persistent stacks.
    let scratch = Stack::<walk::Frame<'_>>::worst_case(depth)?
        .max(Stack::<escape::Frame<'_>>::worst_case(depth)?)
        .max(Stack::<(core::slice::Iter<'_, Operand<'_>>, List<Operand<'_>>, u32)>::worst_case(limits.nesting)?);
    Stack::<Operand<'_>>::worst_case(limits.expression)?
        .checked_add(Stack::<(usize, u32)>::worst_case(limits.nesting)?)?
        .checked_add(scratch)?
        // List construction can hold the reverse stack and its output together;
        // shrinking a List can likewise retain its old allocation briefly.
        .checked_add(limits.made.checked_mul(2)?)
}
#[derive(Debug)]
pub struct Evaluator<'c> {
    values: Stack<Operand<'c>>,
    nesting: u32,
    walk_depth: u32,
    branches: Stack<(usize, u32)>,
    walk_steps: u64,
}
impl Evaluator<'_> {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self {
            values: Stack::with_capacity(limits.expression),
            nesting: limits.nesting,
            walk_depth: limits.depth.saturating_add(limits.nesting),
            branches: Stack::with_capacity(limits.nesting),
            // Five roots, each within the 64 MiB validated-input bound (domain/render.md, 2.3).
            walk_steps: limits
                .made
                .saturating_add(limits.validated.saturating_mul(5))
                .saturating_mul(2)
                .saturating_add(u64::from(limits.expression)),
        }
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
    evaluator.clear();
    let result = run(evaluator, expression, context, bindings, budget);
    evaluator.clear();
    result
}
fn run<'c>(
    evaluator: &mut Evaluator<'c>,
    expression: &Expression,
    context: &Context<'c>,
    bindings: &Bindings<'c>,
    budget: &mut Budget,
) -> Result<Operand<'c>, Fault> {
    let mut skip = 0_u32;
    for _ in 0..evaluator.branches.len() {
        let _ended = evaluator.branches.pop().expect("counted");
    }
    for (index, operation) in expression.operations.iter().enumerate() {
        if skip != 0 {
            skip = skip.saturating_sub(1);
            continue;
        }
        evaluator.end_branches(index)?;
        let jump = match evaluator.jump(operation, expression, index) {
            Ok(jump) => jump,
            Err(kind) => return Err(Fault { at: operation.at, kind }),
        };
        if let Some(over) = jump {
            skip = over;
            continue;
        }
        match evaluator.apply(&operation.op, context, bindings, budget) {
            Ok(value) => {
                if evaluator.values.push(value).is_err() {
                    return Err(Fault { at: operation.at, kind: FaultKind::Limit });
                }
            }
            Err(kind) => return Err(Fault { at: operation.at, kind }),
        }
    }
    evaluator.end_branches(expression.operations.len())?;
    let value = evaluator.values.pop().expect("compiled expression leaves a value");
    assert!(evaluator.values.is_empty(), "compiled expression leaves exactly one value");
    Ok(value)
}
impl<'c> Evaluator<'c> {
    fn clear(&mut self) {
        for _ in 0..self.values.len() {
            drop(self.values.pop());
        }
        for _ in 0..self.branches.len() {
            let _ended = self.branches.pop().expect("counted");
        }
    }

    fn jump(&mut self, operation: &Operation, expression: &Expression, index: usize) -> Result<Option<u32>, FaultKind> {
        Ok(match &operation.op {
            Op::JumpIfFalseKeeping { over } => {
                if truthy(self.values.top().expect("operand")) {
                    drop(self.pop());
                    Some(0)
                } else {
                    Some(*over)
                }
            }
            Op::JumpIfTrueKeeping { over } => {
                if truthy(self.values.top().expect("operand")) {
                    Some(*over)
                } else {
                    drop(self.pop());
                    Some(0)
                }
            }
            Op::JumpIfFalse { over } => {
                if truthy(&self.pop()) {
                    Some(0)
                } else {
                    let jump_index = index.saturating_add(usize::try_from(*over).expect("offset fits"));
                    let jump = expression.operations.get(jump_index).expect("conditional jump");
                    let Op::Jump { over: tail } = jump.op else {
                        unreachable!("conditional layout");
                    };
                    let end = jump_index.saturating_add(1).saturating_add(usize::try_from(tail).expect("offset fits"));
                    if self.branches.push((end, jump.at)).is_err() {
                        return Err(FaultKind::Limit);
                    }
                    Some(*over)
                }
            }
            Op::Jump { over } => {
                if self.values.top().expect("branch value").undefined() {
                    return Err(FaultKind::Undefined);
                }
                Some(*over)
            }
            Op::String(_)
            | Op::Integer(_)
            | Op::Boolean(_)
            | Op::Name { .. }
            | Op::Key { .. }
            | Op::Item { .. }
            | Op::List { .. }
            | Op::Filter(_)
            | Op::Defined
            | Op::NotDefined
            | Op::Not
            | Op::Equal
            | Op::NotEqual
            | Op::Less
            | Op::LessOrEqual
            | Op::Greater
            | Op::GreaterOrEqual
            | Op::In
            | Op::NotIn
            | Op::Concat
            | Op::Add
            | Op::Subtract => None,
        })
    }
    fn end_branches(&mut self, index: usize) -> Result<(), Fault> {
        for _ in 0..=self.nesting {
            let Some(&(end, at)) = self.branches.top() else {
                break;
            };
            if end != index {
                break;
            }
            if self.values.top().expect("branch value").undefined() {
                return Err(Fault { at, kind: FaultKind::Undefined });
            }
            let _ended = self.branches.pop().expect("top existed");
        }
        Ok(())
    }

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
                name => bindings.find(name, budget, self.nesting)?,
            },
            Op::Key { name, missing } => lookup::key(self.pop(), name, *missing)?,
            Op::Item { missing } => {
                let index = self.pop();
                lookup::item(self.pop(), index, *missing)?
            }
            Op::List { length } => self.list(*length, budget)?,
            Op::Filter(filter) => self.filter(*filter, budget)?,
            Op::Not => Operand::Bool(!truthy(&self.pop())),
            Op::Equal
            | Op::NotEqual
            | Op::Less
            | Op::LessOrEqual
            | Op::Greater
            | Op::GreaterOrEqual
            | Op::In
            | Op::NotIn
            | Op::Concat
            | Op::Add
            | Op::Subtract => {
                let right = self.pop();
                let left = self.pop();
                operators::binary(op, left, right, budget, self.walk_depth, self.walk_steps)?
            }
            Op::JumpIfFalseKeeping { .. } | Op::JumpIfTrueKeeping { .. } | Op::JumpIfFalse { .. } | Op::Jump { .. } => {
                unreachable!("jumps handled by expression")
            }
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
                Operand::Loop { .. } => return Err(FaultKind::Type),
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
