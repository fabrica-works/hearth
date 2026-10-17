//! Input builders for flat template programs.
use crate::{
    evaluate::{
        Context, Expression, Fault, Operation,
        template::{self, Does, Instruction, Names, Template},
    },
    limits::{self, Limits},
};
use alloc::boxed::Box;
use skein_lib::bytes;
pub fn template<const N: usize>(lines: &[u32], instructions: [Instruction; N]) -> Template {
    Template { lines: lines.into(), instructions: Box::new(instructions) }
}
pub fn text(at: u32, bytes: &[u8]) -> Instruction {
    Instruction { at, does: Does::Text(bytes::copy_of(bytes)) }
}
pub fn output(at: u32, expression: Expression) -> Instruction {
    Instruction { at, does: Does::Output(expression) }
}
pub fn branch(at: u32, condition: Expression, over: u32) -> Instruction {
    Instruction { at, does: Does::Branch { condition, over } }
}
pub fn jump(at: u32, over: u32) -> Instruction {
    Instruction { at, does: Does::Jump { over } }
}
pub fn enter(at: u32, names: Names, iterable: Expression, over: u32) -> Instruction {
    Instruction { at, does: Does::Enter { names, iterable, over } }
}
pub fn advance(at: u32, back: u32) -> Instruction {
    Instruction { at, does: Does::Advance { back } }
}
pub fn set(at: u32, name: &[u8], value: Expression) -> Instruction {
    Instruction { at, does: Does::Set { name: bytes::copy_of(name), value } }
}
pub fn one(name: &[u8]) -> Names {
    Names::One { name: bytes::copy_of(name) }
}
pub fn pair(first: &[u8], second: &[u8]) -> Names {
    Names::Pair { first: bytes::copy_of(first), second: bytes::copy_of(second) }
}
pub fn expr<const N: usize>(values: u32, operations: [Operation; N]) -> Expression {
    Expression { values, operations: Box::new(operations) }
}
pub fn run(roots: &super::Roots, template: &Template) -> Result<Box<[u8]>, Fault> {
    run_at(roots, template, &limits::SHIPPED)
}
pub fn run_at(roots: &super::Roots, template: &Template, limits: &Limits) -> Result<Box<[u8]>, Fault> {
    let context: Context<'_> = roots.context();
    template::run(template, &context, limits, u64::from(limits.rendered_file))
}
