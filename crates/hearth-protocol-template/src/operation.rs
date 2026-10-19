use crate::Filter;
use alloc::boxed::Box;
/// Postfix operations and their maximum value stack (domain/render.md, 8.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Expression {
    pub operations: Box<[Operation]>,
    pub values: u32,
}
/// An operation and its token's byte offset.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Operation {
    pub at: u32,
    pub op: Op,
}
/// The expression vocabulary (protocol/format.md, 7.4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Op {
    String(Box<[u8]>),
    Integer(i64),
    Boolean(bool),
    Name { name: Box<[u8]>, missing: Missing },
    Key { name: Box<[u8]>, missing: Missing },
    Item { missing: Missing },
    List { length: u32 },
    Filter(Filter),
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
    Defined,
    NotDefined,
    JumpIfFalseKeeping { over: u32 },
    JumpIfTrueKeeping { over: u32 },
    JumpIfFalse { over: u32 },
    Jump { over: u32 },
}

/// A missing lookup's result (protocol/format.md, 7.6); the root name is no link.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Missing {
    Error,
    Undefined,
}
