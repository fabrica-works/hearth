//! Postfix expressions (domain/render.md, 8.3), mirrored from the template machine.
use super::Filter;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Expression {
    pub operations: Box<[Operation]>,
    pub values: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Operation {
    pub at: u32,
    pub op: Op,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Op {
    String(Box<[u8]>),
    Integer(i64),
    Boolean(bool),
    Name { name: Box<[u8]>, missing: Missing },
    Key { name: Box<[u8]>, missing: Missing },
    Item { missing: Missing },
    List { length: u32 },
    Filter(Filter),
    Defined,
    NotDefined,
    Not,
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
    JumpIfFalseKeeping { over: u32 },
    JumpIfTrueKeeping { over: u32 },
    JumpIfFalse { over: u32 },
    Jump { over: u32 },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Missing {
    Error,
    Undefined,
}
