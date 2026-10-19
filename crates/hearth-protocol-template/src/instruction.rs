//! Flat template instructions (protocol/format.md, 7.2; domain/render.md, 8.3).
use crate::Expression;
use alloc::boxed::Box;
/// A compiled template, with the first byte of each line.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Template {
    pub instructions: Box<[Instruction]>,
    pub lines: Box<[u32]>,
}
/// One instruction at its opening tag or first text byte.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Instruction {
    pub at: u32,
    pub does: Does,
}
/// The work an instruction performs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Does {
    Text(Box<[u8]>),
    Output(Expression),
    Branch { condition: Expression, over: u32 },
    Jump { over: u32 },
    Enter { names: Names, iterable: Expression, over: u32 },
    Advance { back: u32 },
    Set { name: Box<[u8]>, value: Expression },
}

/// Names bound by one loop (protocol/format.md, 7.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Names {
    One { name: Box<[u8]> },
    Pair { first: Box<[u8]>, second: Box<[u8]> },
}
