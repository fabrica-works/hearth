//! Flat compiled text and names, with 32 bytes per instruction/operation (2.3).
use crate::evaluate::{
    ELEMENT, Expression, Op,
    template::{Does, Instruction, Names, Template},
};

pub(crate) fn template(template: &Template) -> u64 {
    instructions(&template.instructions)
}
pub(crate) fn instructions(instructions: &[Instruction]) -> u64 {
    let mut bytes = 0_u64;
    for instruction in instructions {
        bytes = bytes.saturating_add(ELEMENT);
        let owned = match &instruction.does {
            Does::Text(text) => size(text),
            Does::Output(code) | Does::Branch { condition: code, .. } => expression(code),
            Does::Jump { .. } | Does::Advance { .. } => 0,
            Does::Enter { names, iterable, .. } => names_size(names).saturating_add(expression(iterable)),
            Does::Set { name, value } => size(name).saturating_add(expression(value)),
        };
        bytes = bytes.saturating_add(owned);
    }
    bytes
}
fn names_size(names: &Names) -> u64 {
    match names {
        Names::One { name } => size(name),
        Names::Pair { first, second } => size(first).saturating_add(size(second)),
    }
}
pub(crate) fn expression(expression: &Expression) -> u64 {
    let mut bytes = 0_u64;
    for operation in &expression.operations {
        let owned = match &operation.op {
            Op::String(value) | Op::Name { name: value, .. } | Op::Key { name: value, .. } => size(value),
            Op::Integer(_)
            | Op::Boolean(_)
            | Op::Item { .. }
            | Op::List { .. }
            | Op::Filter(_)
            | Op::Defined
            | Op::NotDefined
            | Op::Not
            | Op::Equal
            | Op::NotEqual
            | Op::Less
            | Op::LessOrEqual
            | Op::Greater
            | Op::GreaterOrEqual
            | Op::In
            | Op::NotIn
            | Op::Concat
            | Op::Add
            | Op::Subtract
            | Op::JumpIfFalseKeeping { .. }
            | Op::JumpIfTrueKeeping { .. }
            | Op::JumpIfFalse { .. }
            | Op::Jump { .. } => 0,
        };
        bytes = bytes.saturating_add(ELEMENT).saturating_add(owned);
    }
    bytes
}
fn size(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).expect("compiled bytes fit u64")
}
