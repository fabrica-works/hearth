use crate::Filter;
use alloc::boxed::Box;
/// Postfix operations and their maximum value stack (render.md, 8.3).
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
/// The expression vocabulary (format.md, 7.4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Op {
    String(Box<[u8]>),
    Integer(i64),
    Boolean(bool),
    Name { name: Box<[u8]> },
    Key { name: Box<[u8]> },
    Item,
    List { length: u32 },
    Filter(Filter),
}
