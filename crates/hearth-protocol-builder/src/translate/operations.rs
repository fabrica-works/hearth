//! Total expression translations from the template compiler (protocol/format.md, 7.4 to 7.7).
use crate::render::evaluate as render;
use hearth_protocol_template as template;
use skein_lib::List;
#[must_use]
pub fn expression(expression: template::Expression) -> render::Expression {
    let template::Expression { operations, values } = expression;
    let mut translated = List::with_capacity(u32::try_from(operations.len()).expect("bounded operations"));
    for item in operations {
        translated.push(operation(item)).expect("operation room");
    }
    render::Expression { operations: translated.into_boxed(), values }
}
#[must_use]
pub fn operation(operation: template::Operation) -> render::Operation {
    let template::Operation { at, op: value } = operation;
    render::Operation { at, op: op(value) }
}
#[must_use]
pub fn op(op: template::Op) -> render::Op {
    match op {
        template::Op::String(v) => render::Op::String(v),
        template::Op::Integer(v) => render::Op::Integer(v),
        template::Op::Boolean(v) => render::Op::Boolean(v),
        template::Op::Name { name, missing: mark } => render::Op::Name { name, missing: missing(mark) },
        template::Op::Key { name, missing: mark } => render::Op::Key { name, missing: missing(mark) },
        template::Op::Item { missing: mark } => render::Op::Item { missing: missing(mark) },
        template::Op::List { length } => render::Op::List { length },
        template::Op::Filter(value) => render::Op::Filter(filter(value)),
        template::Op::Not => render::Op::Not,
        template::Op::Equal => render::Op::Equal,
        template::Op::NotEqual => render::Op::NotEqual,
        template::Op::Less => render::Op::Less,
        template::Op::LessOrEqual => render::Op::LessOrEqual,
        template::Op::Greater => render::Op::Greater,
        template::Op::GreaterOrEqual => render::Op::GreaterOrEqual,
        template::Op::In => render::Op::In,
        template::Op::NotIn => render::Op::NotIn,
        template::Op::Concat => render::Op::Concat,
        template::Op::Add => render::Op::Add,
        template::Op::Subtract => render::Op::Subtract,
        template::Op::Defined => render::Op::Defined,
        template::Op::NotDefined => render::Op::NotDefined,
        template::Op::JumpIfFalseKeeping { over } => render::Op::JumpIfFalseKeeping { over },
        template::Op::JumpIfTrueKeeping { over } => render::Op::JumpIfTrueKeeping { over },
        template::Op::JumpIfFalse { over } => render::Op::JumpIfFalse { over },
        template::Op::Jump { over } => render::Op::Jump { over },
    }
}
#[must_use]
pub fn missing(missing: template::Missing) -> render::Missing {
    match missing {
        template::Missing::Error => render::Missing::Error,
        template::Missing::Undefined => render::Missing::Undefined,
    }
}
#[must_use]
pub fn filter(filter: template::Filter) -> render::Filter {
    match filter {
        template::Filter::Default => render::Filter::Default,
        template::Filter::Join => render::Filter::Join,
        template::Filter::Split => render::Filter::Split,
        template::Filter::First => render::Filter::First,
        template::Filter::Last => render::Filter::Last,
        template::Filter::Length => render::Filter::Length,
        template::Filter::Replace => render::Filter::Replace,
        template::Filter::Items => render::Filter::Items,
        template::Filter::Keys => render::Filter::Keys,
        template::Filter::Values => render::Filter::Values,
        template::Filter::Sort => render::Filter::Sort,
        template::Filter::Tojson => render::Filter::Tojson,
        template::Filter::ShellQuote => render::Filter::ShellQuote,
        template::Filter::SystemdEscape => render::Filter::SystemdEscape,
        template::Filter::SystemdEscapePath => render::Filter::SystemdEscapePath,
        template::Filter::Basename => render::Filter::Basename,
        template::Filter::Dirname => render::Filter::Dirname,
    }
}
