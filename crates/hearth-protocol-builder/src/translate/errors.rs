//! Exhaustive config faults and client error records (render.md, 7.1).
use crate::{
    config, records,
    render::{self, errors as target},
};
use skein_json::tokenizer;
#[must_use]
pub fn fault(
    fault: config::Fault,
    limits: &config::limits::Limits,
) -> Option<(target::Decode, Option<render::revision::Position>)> {
    let fault = match fault {
        config::Fault::Json(error) => match error {
            tokenizer::Error::Unexpected => target::Decode::Unexpected,
            tokenizer::Error::Trailing => target::Decode::Trailing,
            tokenizer::Error::TooDeep => target::Decode::TooDeep,
            tokenizer::Error::StringTooLong => target::Decode::StringTooLong,
            tokenizer::Error::NumberTooLong => target::Decode::NumberTooLong,
            tokenizer::Error::Number => target::Decode::Number,
            tokenizer::Error::Escape => target::Decode::Escape,
            tokenizer::Error::Surrogate => target::Decode::Surrogate,
            tokenizer::Error::Utf8 => target::Decode::Utf8,
            tokenizer::Error::Control => target::Decode::Control,
            tokenizer::Error::Truncated => target::Decode::Truncated,
            tokenizer::Error::TooLong => {
                target::Decode::TooLarge { limit: target::Limit::Document, max: u64::from(limits.document) }
            }
            tokenizer::Error::Stream(_) => return None,
        },
        config::Fault::Stream(_) | config::Fault::Declaration(_) => return None,
        config::Fault::Duplicate { key, first } => {
            return Some((target::Decode::Duplicate { key }, Some(super::documents::position(first))));
        }
        config::Fault::NotObject => target::Decode::NotObject,
        config::Fault::NotInteger => target::Decode::NotInteger,
        config::Fault::Range => target::Decode::Range,
        config::Fault::Nul => target::Decode::Nul,
        config::Fault::Keys => target::Decode::Keys,
        config::Fault::Note => target::Decode::Note,
        config::Fault::Shape(shape) => match shape {
            config::Shape::Name => target::Decode::Name,
            config::Shape::Placement => target::Decode::Placement,
            config::Shape::Members => target::Decode::Members,
            config::Shape::Member => target::Decode::Member,
            config::Shape::Settings { task } => target::Decode::Settings { task },
            config::Shape::Unknown { field } => target::Decode::Unknown { field },
            config::Shape::Wrong { field } => target::Decode::Wrong { field: config_field(field) },
            config::Shape::Missing { field } => target::Decode::Missing { field: config_field(field) },
            config::Shape::Misplaced { field } => target::Decode::Misplaced { field: config_field(field) },
        },
    };
    Some((fault, None))
}
fn config_field(field: config::Field) -> target::Field {
    match field {
        config::Field::Kind => target::Field::Kind,
        config::Field::Firmware => target::Field::Firmware,
        config::Field::Hardware => target::Field::Hardware,
        config::Field::On => target::Field::On,
        config::Field::Mutable => target::Field::Mutable,
    }
}
#[must_use]
pub fn error(error: target::Error) -> records::errors::Error {
    let target::Error { kind, class, at, also, target, task } = error;
    let kind = match kind {
        target::Kind::Decode(fault) => records::errors::Kind::Decode(decode(fault)),
    };
    let class = match class {
        render::vocabulary::Class::Configuration => records::errors::Class::Configuration,
        render::vocabulary::Class::Operational => records::errors::Class::Operational,
    };
    records::errors::Error { kind, class, at: place(at), also: place(also), target, task }
}
fn place(place: Option<target::Place>) -> Option<records::errors::Place> {
    let target::Place { path, position } = place?;
    Some(records::errors::Place { path, position: position_to_records(position) })
}
fn position_to_records(position: Option<render::revision::Position>) -> Option<records::errors::Position> {
    let render::revision::Position { line, column } = position?;
    Some(records::errors::Position { line, column })
}
#[must_use]
pub fn decode(fault: target::Decode) -> records::errors::Decode {
    match fault {
        target::Decode::Unexpected => records::errors::Decode::Unexpected,
        target::Decode::Trailing => records::errors::Decode::Trailing,
        target::Decode::TooDeep => records::errors::Decode::TooDeep,
        target::Decode::StringTooLong => records::errors::Decode::StringTooLong,
        target::Decode::NumberTooLong => records::errors::Decode::NumberTooLong,
        target::Decode::Number => records::errors::Decode::Number,
        target::Decode::Escape => records::errors::Decode::Escape,
        target::Decode::Surrogate => records::errors::Decode::Surrogate,
        target::Decode::Utf8 => records::errors::Decode::Utf8,
        target::Decode::Control => records::errors::Decode::Control,
        target::Decode::Truncated => records::errors::Decode::Truncated,
        target::Decode::NotObject => records::errors::Decode::NotObject,
        target::Decode::NotInteger => records::errors::Decode::NotInteger,
        target::Decode::Range => records::errors::Decode::Range,
        target::Decode::Nul => records::errors::Decode::Nul,
        target::Decode::Keys => records::errors::Decode::Keys,
        target::Decode::Note => records::errors::Decode::Note,
        target::Decode::Name => records::errors::Decode::Name,
        target::Decode::Placement => records::errors::Decode::Placement,
        target::Decode::Members => records::errors::Decode::Members,
        target::Decode::Member => records::errors::Decode::Member,
        target::Decode::Duplicate { key } => records::errors::Decode::Duplicate { key },
        target::Decode::Settings { task } => records::errors::Decode::Settings { task },
        target::Decode::Unknown { field } => records::errors::Decode::Unknown { field },
        target::Decode::Wrong { field } => records::errors::Decode::Wrong { field: self::field(field) },
        target::Decode::Missing { field } => records::errors::Decode::Missing { field: self::field(field) },
        target::Decode::Misplaced { field } => records::errors::Decode::Misplaced { field: self::field(field) },
        target::Decode::TooLarge { limit, max } => records::errors::Decode::TooLarge { limit: self::limit(limit), max },
    }
}
#[must_use]
pub fn field(field: target::Field) -> records::errors::Field {
    match field {
        target::Field::Kind => records::errors::Field::Kind,
        target::Field::Firmware => records::errors::Field::Firmware,
        target::Field::Hardware => records::errors::Field::Hardware,
        target::Field::On => records::errors::Field::On,
        target::Field::Mutable => records::errors::Field::Mutable,
    }
}
#[must_use]
pub fn limit(limit: target::Limit) -> records::errors::Limit {
    match limit {
        target::Limit::Document => records::errors::Limit::Document,
        target::Limit::Decoded => records::errors::Limit::Decoded,
        target::Limit::Static => records::errors::Limit::Static,
    }
}
