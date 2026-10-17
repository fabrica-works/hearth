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
pub fn layout(template: &crate::Template) -> Box<[(u32, &'static str, u32)]> {
    let mut result = skein_lib::List::with_capacity(u32::try_from(template.instructions.len()).expect("test fixture"));
    for instruction in &template.instructions {
        let (kind, count) = match &instruction.does {
            crate::Does::Text(_) => ("text", 0),
            crate::Does::Output(_) => ("output", 0),
            crate::Does::Branch { over, .. } => ("branch", *over),
            crate::Does::Jump { over } => ("jump", *over),
            crate::Does::Enter { over, .. } => ("enter", *over),
            crate::Does::Advance { back } => ("advance", *back),
            crate::Does::Set { .. } => ("set", 0),
        };
        result.push((instruction.at, kind, count)).expect("one observation per instruction");
    }
    result.into_boxed()
}

pub fn binding(head: &[u8], name: &[u8], tail: &[u8]) -> Box<[u8]> {
    let length =
        u32::try_from(head.len().saturating_add(name.len()).saturating_add(tail.len())).expect("fixture length");
    let mut bytes = skein_lib::List::with_capacity(length);
    for part in [head, name, tail] {
        for &byte in part {
            bytes.push(byte).expect("fixture capacity");
        }
    }
    bytes.into_boxed()
}
