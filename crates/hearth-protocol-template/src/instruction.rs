//! Flat template instructions (format.md, 7.2; render.md, 8.3).
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
}
