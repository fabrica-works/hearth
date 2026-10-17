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
