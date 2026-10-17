//! Postfix expressions (render.md, 8.3), mirrored from the template machine.
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
    Defined,
    NotDefined,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Missing {
    Error,
    Undefined,
}
