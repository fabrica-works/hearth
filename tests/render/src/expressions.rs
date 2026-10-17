//! Generated trees and contexts, independent of the postfix evaluator's implementation.
use hearth_domain_render::{
    evaluate::{
        self, Binding, Bindings, Budget, Context, Entry, Evaluator, Expression, Missing, Op, Operand, Operation, Value,
    },
    limits::Limits,
};
use skein_lib::Rng;
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub depth: u32,
    pub width: u32,
}
pub const SMALL: Shape = Shape { depth: 3, width: 3 };
pub const WIDE: Shape = Shape { depth: 6, width: 6 };
pub const KINDS: &[&str] = &["unknown", "key", "index", "undefined", "type", "limit", "overflow", "output"];
#[derive(Clone, Debug)]
pub struct Roots {
    pub values: [Value; 5],
    pub bindings: Vec<(&'static [u8], Value)>,
}
impl Roots {
    #[must_use]
    pub const fn context(&self) -> Context<'_> {
        Context {
            target: &self.values[0],
            debian: &self.values[1],
            settings: &self.values[2],
            tasks: &self.values[3],
            exports: &self.values[4],
        }
    }
}
#[derive(Clone, Debug)]
pub enum Tree {
    String(Vec<u8>),
    Int(i64),
    Bool(bool),
    Name(&'static [u8]),
    Key(Box<Tree>, &'static [u8], Missing),
    Item(Box<Tree>, Box<Tree>, Missing),
    List(Vec<Tree>),
    Defined(Box<Tree>, bool),
    Not(Box<Tree>),
    Binary(Box<Tree>, Binary, Box<Tree>),
    And(Box<Tree>, Box<Tree>),
    Or(Box<Tree>, Box<Tree>),
    Conditional { condition: Box<Tree>, yes: Box<Tree>, no: Box<Tree> },
    Output(Box<Tree>),
}
#[derive(Clone, Copy, Debug)]
pub enum Binary {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    In,
    NotIn,
    Concat,
    Add,
    Subtract,
}
const BINARIES: &[Binary] = &[
    Binary::Equal,
    Binary::NotEqual,
    Binary::Less,
    Binary::LessOrEqual,
    Binary::Greater,
    Binary::GreaterOrEqual,
    Binary::In,
    Binary::NotIn,
    Binary::Concat,
    Binary::Add,
    Binary::Subtract,
];
impl Binary {
    const fn op(self) -> Op {
        match self {
            Self::Equal => Op::Equal,
            Self::NotEqual => Op::NotEqual,
            Self::Less => Op::Less,
            Self::LessOrEqual => Op::LessOrEqual,
            Self::Greater => Op::Greater,
            Self::GreaterOrEqual => Op::GreaterOrEqual,
            Self::In => Op::In,
            Self::NotIn => Op::NotIn,
            Self::Concat => Op::Concat,
            Self::Add => Op::Add,
            Self::Subtract => Op::Subtract,
        }
    }
}

pub type Plain = Value;
#[must_use]
pub fn roots(rng: &mut Rng) -> Roots {
    let values = core::array::from_fn(|_| {
        Value::Map(
            [b"a".as_slice(), b"b", b"c"]
                .into_iter()
                .map(|key| Entry {
                    key: Box::from(key),
                    value: if rng.chance(250) { None } else { Some(context_value(rng, 3)) },
                })
                .collect(),
        )
    });
    let bindings = if rng.chance(500) {
        vec![(b"key".as_slice(), Value::Int(2)), (b"value".as_slice(), context_value(rng, 3))]
    } else {
        vec![]
    };
    Roots { values, bindings }
}
fn context_value(rng: &mut Rng, depth: u32) -> Value {
    if depth == 0 {
        return Value::Int(i64::try_from(rng.below(5)).expect("small"));
    }
    match rng.below(5) {
        0 => Value::Bool(rng.chance(500)),
        1 => Value::String(vec![b'x'; usize::try_from(rng.below(50)).expect("small")].into_boxed_slice()),
        2 => Value::List((0..rng.below(4)).map(|_| context_value(rng, depth - 1)).collect()),
        3 => Value::Int(i64::try_from(rng.below(20)).expect("small") - 10),
        _ => Value::Map(
            [b"a".as_slice(), b"b", b"c"]
                .into_iter()
                .map(|key| Entry {
                    key: Box::from(key),
                    value: if rng.chance(250) { None } else { Some(context_value(rng, depth - 1)) },
                })
                .collect(),
        ),
    }
}
#[must_use]
pub fn tree(rng: &mut Rng, roots: &Roots, shape: Shape) -> Tree {
    let tree = grow(rng, roots, shape);
    if rng.chance(500) { Tree::Output(Box::new(tree)) } else { tree }
}
fn grow(rng: &mut Rng, roots: &Roots, shape: Shape) -> Tree {
    if shape.depth == 0 {
        return leaf(rng);
    }
    let sub = Shape { depth: shape.depth - 1, ..shape };
    match rng.below(16) {
        0 => leaf(rng),
        1 => {
            if roots.bindings.is_empty() {
                Tree::Name(b"settings")
            } else {
                Tree::Name(b"value")
            }
        }
        2 => Tree::Key(Box::new(grow(rng, roots, sub)), b"a", Missing::Error),
        3 => Tree::Item(
            Box::new(grow(rng, roots, sub)),
            Box::new(Tree::Int(i64::try_from(rng.below(6)).expect("small") - 1)),
            Missing::Error,
        ),
        4 => Tree::List((0..rng.below(u64::from(shape.width) + 1)).map(|_| grow(rng, roots, sub)).collect()),
        5 => Tree::Defined(
            Box::new(Tree::Key(
                Box::new(Tree::Key(Box::new(Tree::Name(b"settings")), b"a", Missing::Undefined)),
                b"b",
                Missing::Undefined,
            )),
            rng.chance(500),
        ),
        6 => Tree::Key(Box::new(Tree::Name(b"settings")), b"missing", Missing::Error),
        7 => Tree::Defined(Box::new(grow(rng, roots, sub)), rng.chance(500)),
        8 => Tree::List(vec![
            Tree::String(vec![b'x'; usize::try_from(rng.between(16, 4096)).expect("small")]);
            usize::try_from(shape.width).expect("small")
        ]),
        9 => Tree::Not(Box::new(grow(rng, roots, sub))),
        10 => Tree::Binary(
            Box::new(grow(rng, roots, sub)),
            BINARIES[usize::try_from(rng.below(11)).expect("small")],
            Box::new(grow(rng, roots, sub)),
        ),
        11 => Tree::And(Box::new(grow(rng, roots, sub)), Box::new(grow(rng, roots, sub))),
        12 => Tree::Or(Box::new(grow(rng, roots, sub)), Box::new(grow(rng, roots, sub))),
        13 => Tree::Conditional {
            condition: Box::new(grow(rng, roots, sub)),
            yes: Box::new(grow(rng, roots, sub)),
            no: Box::new(grow(rng, roots, sub)),
        },
        14 => Tree::Binary(Box::new(Tree::Int(i64::MAX)), Binary::Add, Box::new(Tree::Int(1))),
        _ => Tree::Binary(Box::new(Tree::String(b"x".to_vec())), Binary::Concat, Box::new(Tree::Int(-12))),
    }
}
fn leaf(rng: &mut Rng) -> Tree {
    match rng.below(5) {
        0 => Tree::String(vec![b'x'; usize::try_from(rng.below(128)).expect("small")]),
        1 => Tree::Int(i64::try_from(rng.below(5)).expect("small")),
        2 => Tree::Bool(rng.chance(500)),
        _ => Tree::Name(
            [b"target".as_slice(), b"debian", b"settings", b"tasks", b"exports", b"key", b"value", b"nope"]
                [usize::try_from(rng.below(8)).expect("small")],
        ),
    }
}
pub fn flatten(tree: &Tree, out: &mut Vec<Operation>) {
    let op = match tree {
        Tree::String(v) => Op::String(v.clone().into_boxed_slice()),
        Tree::Int(v) => Op::Integer(*v),
        Tree::Bool(v) => Op::Boolean(*v),
        Tree::Name(v) => Op::Name { name: Box::from(*v), missing: Missing::Error },
        Tree::Key(v, k, missing) => {
            flatten(v, out);
            Op::Key { name: Box::from(*k), missing: *missing }
        }
        Tree::Item(v, i, missing) => {
            flatten(v, out);
            flatten(i, out);
            Op::Item { missing: *missing }
        }
        Tree::List(items) => {
            for item in items {
                flatten(item, out);
            }
            Op::List { length: u32::try_from(items.len()).expect("small") }
        }
        Tree::Defined(v, negated) => {
            flatten(v, out);
            if *negated { Op::NotDefined } else { Op::Defined }
        }
        Tree::Not(v) => {
            flatten(v, out);
            Op::Not
        }
        Tree::Binary(a, op, b) => {
            flatten(a, out);
            flatten(b, out);
            op.op()
        }
        Tree::And(a, b) | Tree::Or(a, b) => {
            flatten(a, out);
            let at = out.len();
            emit(out, Op::Jump { over: 0 });
            flatten(b, out);
            let over = u32::try_from(out.len() - at - 1).expect("small");
            out[at].op = if matches!(tree, Tree::And(..)) {
                Op::JumpIfFalseKeeping { over }
            } else {
                Op::JumpIfTrueKeeping { over }
            };
            return;
        }
        Tree::Conditional { condition, yes, no } => {
            flatten(condition, out);
            let branch = out.len();
            emit(out, Op::Jump { over: 0 });
            flatten(yes, out);
            let jump = out.len();
            emit(out, Op::Jump { over: 0 });
            out[branch].op = Op::JumpIfFalse { over: u32::try_from(out.len() - branch - 1).expect("small") };
            flatten(no, out);
            out[jump].op = Op::Jump { over: u32::try_from(out.len() - jump - 1).expect("small") };
            return;
        }
        Tree::Output(v) => {
            flatten(v, out);
            return;
        }
    };
    emit(out, op);
}
fn emit(out: &mut Vec<Operation>, op: Op) {
    out.push(Operation { at: u32::try_from(out.len()).expect("small"), op });
}
/// # Errors
/// Returns the first evaluation fault kind.
pub fn evaluate(tree: &Tree, roots: &Roots, limits: &Limits) -> Result<Option<Plain>, &'static str> {
    let mut operations = vec![];
    flatten(tree, &mut operations);
    let expression = Expression {
        values: u32::try_from(operations.len()).expect("small"),
        operations: operations.into_boxed_slice(),
    };
    let mut bindings = Bindings::with_capacity(u32::try_from(roots.bindings.len()).expect("small"));
    for (name, value) in &roots.bindings {
        bindings.push(Binding { name, value: Operand::Value(value) }).expect("room");
    }
    let result = evaluate::expression(
        &mut Evaluator::new(limits),
        &expression,
        &roots.context(),
        &bindings,
        &mut Budget::new(limits),
    )
    .map_err(|fault| super::fixtures::kind(fault.kind))?;
    if matches!(tree, Tree::Output(_)) {
        let written = evaluate::output(&result).map_err(super::fixtures::kind)?;
        let text = match written {
            evaluate::Written::Bytes(v) => v.to_vec(),
            evaluate::Written::Integer { negative, digits } => {
                let mut v = vec![];
                if negative {
                    v.push(b'-');
                }
                v.extend_from_slice(digits.as_bytes());
                v
            }
        };
        Ok(Some(Value::String(text.into_boxed_slice())))
    } else {
        Ok(owned(result))
    }
}
fn owned(operand: Operand<'_>) -> Option<Plain> {
    Some(match operand {
        Operand::Undefined => return None,
        Operand::Value(v) => v.clone(),
        Operand::Bool(v) => Value::Bool(v),
        Operand::Int(v) => Value::Int(v),
        Operand::String(v) => Value::String(v),
        Operand::List { items, depth: _ } => {
            Value::List(items.into_iter().map(|v| owned(v).expect("defined list")).collect())
        }
    })
}
