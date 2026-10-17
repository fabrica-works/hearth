use crate::{Error, Expression, Missing, Op, Operation, expression, limits::SHIPPED};
use alloc::boxed::Box;
pub fn compile(text: &[u8]) -> Result<Expression, Error> {
    expression::compile(text, &SHIPPED)
}
pub fn bytes(text: &[u8]) -> Box<[u8]> {
    skein_lib::bytes::copy_of(text)
}
pub fn op(at: u32, op: Op) -> Operation {
    Operation { at, op }
}
pub fn name(at: u32, text: &[u8]) -> Operation {
    op(at, Op::Name { name: bytes(text), missing: Missing::Error })
}

pub fn key(at: u32, text: &[u8]) -> Operation {
    op(at, Op::Key { name: bytes(text), missing: Missing::Error })
}
pub fn item(at: u32) -> Operation {
    op(at, Op::Item { missing: Missing::Error })
}
pub fn template(text: &[u8]) -> Result<crate::Template, Error> {
    crate::template::compile(text, &SHIPPED)
}
pub fn text(at: u32, value: &[u8]) -> crate::Instruction {
    crate::Instruction { at, does: crate::Does::Text(bytes(value)) }
}
pub fn expr<const N: usize>(values: u32, operations: [Operation; N]) -> Expression {
    Expression { values, operations: operations.into() }
}
pub fn output(at: u32, value: Expression) -> crate::Instruction {
    crate::Instruction { at, does: crate::Does::Output(value) }
}
