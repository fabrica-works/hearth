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
        config::Fault::Stream(_) => return None,
        config::Fault::Declaration(value) => target::Decode::Declaration(declared(value)),
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
            config::Shape::ListNull => target::Decode::ListNull,
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
    let target::Error { kind, class, at, also, target, task, side } = error;
    let kind = match kind {
        target::Kind::Store(fault) => records::errors::Kind::Store(store_error(fault)),
        target::Kind::Evaluation(kind) => records::errors::Kind::Evaluation(evaluation(kind)),
        target::Kind::Schema(fault) => records::errors::Kind::Schema(schema(fault)),
        target::Kind::Decode(fault) => records::errors::Kind::Decode(decode(fault)),
        target::Kind::Repository(why) => records::errors::Kind::Repository(repository(why)),
        target::Kind::Tool(value) => records::errors::Kind::Tool(tool(value)),
        target::Kind::Composition(value) => records::errors::Kind::Composition(composition(value)),
    };
    let class = match class {
        render::vocabulary::Class::Configuration => records::errors::Class::Configuration,
        render::vocabulary::Class::Operational => records::errors::Class::Operational,
    };
    records::errors::Error { side: optional_side(side), kind, class, at: place(at), also: place(also), target, task }
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
        target::Decode::Syntax(value) => records::errors::Decode::Syntax(syntax_to_records(value)),
        target::Decode::Declaration(value) => records::errors::Decode::Declaration(declared_to_records(value)),
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
        target::Decode::ListNull => records::errors::Decode::ListNull,
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
        target::Limit::Template => records::errors::Limit::Template,
        target::Limit::Document => records::errors::Limit::Document,
        target::Limit::Decoded => records::errors::Limit::Decoded,
        target::Limit::Static => records::errors::Limit::Static,
        target::Limit::Tasks => records::errors::Limit::Tasks,
        target::Limit::Targets => records::errors::Limit::Targets,
        target::Limit::Paths => records::errors::Limit::Paths,
        target::Limit::Path => records::errors::Limit::Path,
        target::Limit::Entries => records::errors::Limit::Entries,
    }
}

#[must_use]
pub fn declared(value: config::Declared) -> target::Declared {
    match value {
        config::Declared::Unknown { of } => target::Declared::Unknown { of: object(of) },
        config::Declared::Later => target::Declared::Later,
        config::Declared::Wrong { expected } => target::Declared::Wrong { expected: config_expected(expected) },
        config::Declared::Missing { key } => target::Declared::Missing { key: required(key) },
        config::Declared::Name => target::Declared::Name,
        config::Declared::Doc => target::Declared::Doc,
        config::Declared::Order => target::Declared::Order,
        config::Declared::Optional => target::Declared::Optional,
        config::Declared::Bound => target::Declared::Bound,
        config::Declared::Above => target::Declared::Above,
        config::Declared::Enum => target::Declared::Enum,
        config::Declared::Settings => target::Declared::Settings,
        config::Declared::Secret => target::Declared::Secret,
    }
}

#[must_use]
pub fn object(value: config::Object) -> target::Object {
    match value {
        config::Object::Declaration => target::Object::Declaration,
        config::Object::Read => target::Object::Read,
        config::Object::Guard => target::Object::Guard,
        config::Object::Script => target::Object::Script,
        config::Object::Bless => target::Object::Bless,
        config::Object::Entry(map) => target::Object::Entry(declaration_map(map)),
        config::Object::Range => target::Object::Range,
        config::Object::Spec => target::Object::Spec,
        config::Object::Export => target::Object::Export,
        config::Object::Type => target::Object::Type,
    }
}

#[must_use]
pub fn config_expected(value: config::Expected) -> target::Expected {
    match value {
        config::Expected::Object => target::Expected::Object,
        config::Expected::String => target::Expected::String,
        config::Expected::Strings => target::Expected::Strings,
        config::Expected::Bool => target::Expected::Bool,
        config::Expected::Int => target::Expected::Int,
        config::Expected::Field(takes) => target::Expected::Field(config_takes(takes)),
        config::Expected::True => target::Expected::True,
        config::Expected::Type => target::Expected::Type,
    }
}

#[must_use]
pub fn required(value: config::Required) -> target::Required {
    match value {
        config::Required::If => target::Required::If,
        config::Required::Message => target::Required::Message,
        config::Required::Type => target::Required::Type,
        config::Required::Doc => target::Required::Doc,
        config::Required::Value => target::Required::Value,
    }
}

#[must_use]
pub fn config_takes(value: config::entries::Takes) -> target::Takes {
    match value {
        config::entries::Takes::Text => target::Takes::Text,
        config::entries::Takes::Int => target::Takes::Int,
        config::entries::Takes::Bool => target::Takes::Bool,
        config::entries::Takes::Strings => target::Takes::Strings,
        config::entries::Takes::Range => target::Takes::Range,
    }
}

#[must_use]
pub fn declaration_map(value: config::entries::Map) -> render::revision::declaration::Map {
    match value {
        config::entries::Map::Packages => render::revision::declaration::Map::Packages,
        config::entries::Map::Preseeds => render::revision::declaration::Map::Preseeds,
        config::entries::Map::Initramfs => render::revision::declaration::Map::Initramfs,
        config::entries::Map::Software => render::revision::declaration::Map::Software,
        config::entries::Map::Files => render::revision::declaration::Map::Files,
        config::entries::Map::Users => render::revision::declaration::Map::Users,
        config::entries::Map::Groups => render::revision::declaration::Map::Groups,
        config::entries::Map::Presets => render::revision::declaration::Map::Presets,
        config::entries::Map::Tmpfiles => render::revision::declaration::Map::Tmpfiles,
        config::entries::Map::Data => render::revision::declaration::Map::Data,
        config::entries::Map::Secrets => render::revision::declaration::Map::Secrets,
        config::entries::Map::Identity => render::revision::declaration::Map::Identity,
        config::entries::Map::Cmdline => render::revision::declaration::Map::Cmdline,
        config::entries::Map::Live => render::revision::declaration::Map::Live,
    }
}

#[must_use]
pub fn declared_to_records(value: target::Declared) -> records::errors::Declared {
    match value {
        target::Declared::Unknown { of } => records::errors::Declared::Unknown { of: object_to_records(of) },
        target::Declared::Later => records::errors::Declared::Later,
        target::Declared::Wrong { expected } => {
            records::errors::Declared::Wrong { expected: expected_to_records(expected) }
        }
        target::Declared::Missing { key } => records::errors::Declared::Missing { key: required_to_records(key) },
        target::Declared::Name => records::errors::Declared::Name,
        target::Declared::Doc => records::errors::Declared::Doc,
        target::Declared::Order => records::errors::Declared::Order,
        target::Declared::Optional => records::errors::Declared::Optional,
        target::Declared::Bound => records::errors::Declared::Bound,
        target::Declared::Above => records::errors::Declared::Above,
        target::Declared::Enum => records::errors::Declared::Enum,
        target::Declared::Settings => records::errors::Declared::Settings,
        target::Declared::Secret => records::errors::Declared::Secret,
    }
}

#[must_use]
pub fn object_to_records(value: target::Object) -> records::errors::Object {
    match value {
        target::Object::Declaration => records::errors::Object::Declaration,
        target::Object::Read => records::errors::Object::Read,
        target::Object::Guard => records::errors::Object::Guard,
        target::Object::Script => records::errors::Object::Script,
        target::Object::Bless => records::errors::Object::Bless,
        target::Object::Entry(map) => records::errors::Object::Entry(declaration_map_to_records(map)),
        target::Object::Range => records::errors::Object::Range,
        target::Object::Spec => records::errors::Object::Spec,
        target::Object::Export => records::errors::Object::Export,
        target::Object::Type => records::errors::Object::Type,
    }
}

#[must_use]
pub fn expected_to_records(value: target::Expected) -> records::errors::Expected {
    match value {
        target::Expected::Object => records::errors::Expected::Object,
        target::Expected::String => records::errors::Expected::String,
        target::Expected::Strings => records::errors::Expected::Strings,
        target::Expected::Bool => records::errors::Expected::Bool,
        target::Expected::Int => records::errors::Expected::Int,
        target::Expected::Field(takes) => records::errors::Expected::Field(takes_to_records(takes)),
        target::Expected::True => records::errors::Expected::True,
        target::Expected::Type => records::errors::Expected::Type,
    }
}

#[must_use]
pub fn required_to_records(value: target::Required) -> records::errors::Required {
    match value {
        target::Required::If => records::errors::Required::If,
        target::Required::Message => records::errors::Required::Message,
        target::Required::Type => records::errors::Required::Type,
        target::Required::Doc => records::errors::Required::Doc,
        target::Required::Value => records::errors::Required::Value,
    }
}

#[must_use]
pub fn takes_to_records(value: target::Takes) -> records::errors::Takes {
    match value {
        target::Takes::Text => records::errors::Takes::Text,
        target::Takes::Int => records::errors::Takes::Int,
        target::Takes::Bool => records::errors::Takes::Bool,
        target::Takes::Strings => records::errors::Takes::Strings,
        target::Takes::Range => records::errors::Takes::Range,
    }
}

#[must_use]
pub fn declaration_map_to_records(value: render::revision::declaration::Map) -> records::errors::Map {
    match value {
        render::revision::declaration::Map::Packages => records::errors::Map::Packages,
        render::revision::declaration::Map::Preseeds => records::errors::Map::Preseeds,
        render::revision::declaration::Map::Initramfs => records::errors::Map::Initramfs,
        render::revision::declaration::Map::Software => records::errors::Map::Software,
        render::revision::declaration::Map::Files => records::errors::Map::Files,
        render::revision::declaration::Map::Users => records::errors::Map::Users,
        render::revision::declaration::Map::Groups => records::errors::Map::Groups,
        render::revision::declaration::Map::Presets => records::errors::Map::Presets,
        render::revision::declaration::Map::Tmpfiles => records::errors::Map::Tmpfiles,
        render::revision::declaration::Map::Data => records::errors::Map::Data,
        render::revision::declaration::Map::Secrets => records::errors::Map::Secrets,
        render::revision::declaration::Map::Identity => records::errors::Map::Identity,
        render::revision::declaration::Map::Cmdline => records::errors::Map::Cmdline,
        render::revision::declaration::Map::Live => records::errors::Map::Live,
    }
}

#[must_use]
pub fn schema(schema: target::Schema) -> records::errors::Schema {
    match schema {
        target::Schema::Undeclared { setting, declared } => records::errors::Schema::Undeclared { setting, declared },
        target::Schema::Missing { setting } => records::errors::Schema::Missing { setting },
        target::Schema::Absent { task } => records::errors::Schema::Absent { task },
        target::Schema::Wrong { expected } => records::errors::Schema::Wrong { expected: type_name(expected) },
        target::Schema::Enum { value, listed } => records::errors::Schema::Enum { value, listed },
        target::Schema::Below { min } => records::errors::Schema::Below { min },
        target::Schema::Above { max } => records::errors::Schema::Above { max },
        target::Schema::Length { max_length } => records::errors::Schema::Length { max_length },
        target::Schema::Null => records::errors::Schema::Null,
        target::Schema::UndeclaredField { field, declared } => {
            records::errors::Schema::UndeclaredField { field, declared }
        }
        target::Schema::MissingField { field } => records::errors::Schema::MissingField { field },
        target::Schema::Task { task } => records::errors::Schema::Task { task },
        target::Schema::Unread { task, setting, declared } => {
            records::errors::Schema::Unread { task, setting, declared }
        }
    }
}

#[must_use]
pub fn type_name(name: target::TypeName) -> records::errors::TypeName {
    match name {
        target::TypeName::Bool => records::errors::TypeName::Bool,
        target::TypeName::Int => records::errors::TypeName::Int,
        target::TypeName::String => records::errors::TypeName::String,
        target::TypeName::Enum => records::errors::TypeName::Enum,
        target::TypeName::List => records::errors::TypeName::List,
        target::TypeName::Map => records::errors::TypeName::Map,
        target::TypeName::Object => records::errors::TypeName::Object,
        target::TypeName::Path => records::errors::TypeName::Path,
        target::TypeName::Mode => records::errors::TypeName::Mode,
        target::TypeName::Duration => records::errors::TypeName::Duration,
        target::TypeName::Secret => records::errors::TypeName::Secret,
        target::TypeName::Target => records::errors::TypeName::Target,
        target::TypeName::File => records::errors::TypeName::File,
    }
}

#[must_use]
pub fn repository(why: target::Repository) -> records::errors::Repository {
    match why {
        target::Repository::Stray => records::errors::Repository::Stray,
        target::Repository::Name => records::errors::Repository::Name,
        target::Repository::Declaration => records::errors::Repository::Declaration,
        target::Repository::Static => records::errors::Repository::Static,
        target::Repository::Destination => records::errors::Repository::Destination,
        target::Repository::Submodule => records::errors::Repository::Submodule,
        target::Repository::Link => records::errors::Repository::Link,
        target::Repository::Environment { env } => records::errors::Repository::Environment { env },
    }
}
#[must_use]
pub fn tool(tool: target::Tool) -> records::errors::Tool {
    use records::errors as out;
    let command = match tool.command {
        target::Command::RevParse => out::Command::RevParse,
        target::Command::LsTree => out::Command::LsTree,
        target::Command::CatFile => out::Command::CatFile,
        target::Command::Diff => out::Command::Diff,
    };
    let ended = match tool.ended {
        target::Ended::Start => out::Ended::Start,
        target::Ended::Exit(code) => out::Ended::Exit(code),
        target::Ended::Signal(signal) => out::Ended::Signal(signal),
        target::Ended::Output => out::Ended::Output,
        target::Ended::Missing { oid } => out::Ended::Missing { oid: oid.0 },
        target::Ended::Stream => out::Ended::Stream,
    };
    out::Tool { command, ended, tail: tool.tail }
}

fn store_error(error: target::StoreError) -> records::errors::StoreError {
    use records::errors::{Step, StoreError, StoreFault, StoreLimit};
    let fault = match error.fault {
        target::StoreFault::Full { limit, most } => StoreFault::Full {
            limit: match limit {
                target::StoreLimit::Resolutions => StoreLimit::Resolutions,
                target::StoreLimit::CacheFiles => StoreLimit::CacheFiles,
            },
            most,
        },
        target::StoreFault::Failed { step } => StoreFault::Failed {
            step: match step {
                target::Step::List => Step::List,
                target::Step::Remove => Step::Remove,
                target::Step::Load => Step::Load,
                target::Step::Empty => Step::Empty,
                target::Step::Keep => Step::Keep,
            },
        },
    };
    StoreError { store: error.store, fault }
}

#[must_use]
pub fn syntax(value: hearth_protocol_template::error::ErrorKind) -> target::Syntax {
    use hearth_protocol_template::error::ErrorKind as Source;
    match value {
        Source::TooLong => target::Syntax::TooLong,
        Source::Character => target::Syntax::Character,
        Source::Unterminated => target::Syntax::Unterminated,
        Source::Escape => target::Syntax::Escape,
        Source::Integer => target::Syntax::Integer,
        Source::TooDeep => target::Syntax::TooDeep,
        Source::Misplaced => target::Syntax::Misplaced,
        Source::UnknownFilter => target::Syntax::UnknownFilter,
        Source::UnknownTest => target::Syntax::UnknownTest,
        Source::Utf8 => target::Syntax::Utf8,
        Source::Return => target::Syntax::Return,
        Source::Unclosed => target::Syntax::Unclosed,
        Source::UnknownStatement => target::Syntax::UnknownStatement,
        Source::Unmatched => target::Syntax::Unmatched,
        Source::Unended => target::Syntax::Unended,
        Source::Shadows => target::Syntax::Shadows,
        Source::Interpolation => target::Syntax::Interpolation,
        Source::Expected(value) => target::Syntax::Expected(syntax_expected(value)),
        Source::Arity { filter, given } => target::Syntax::Arity { filter: super::operations::filter(filter), given },
    }
}
#[must_use]
pub fn syntax_expected(value: hearth_protocol_template::error::Expected) -> target::syntax::Expected {
    use hearth_protocol_template::error::Expected as Source;
    match value {
        Source::Operand => target::syntax::Expected::Operand,
        Source::Operator => target::syntax::Expected::Operator,
        Source::Name => target::syntax::Expected::Name,
        Source::CloseParen => target::syntax::Expected::CloseParen,
        Source::CloseBracket => target::syntax::Expected::CloseBracket,
        Source::In => target::syntax::Expected::In,
        Source::Else => target::syntax::Expected::Else,
        Source::Statement => target::syntax::Expected::Statement,
        Source::Assign => target::syntax::Expected::Assign,
        Source::TagEnd => target::syntax::Expected::TagEnd,
    }
}
#[must_use]
pub fn syntax_to_records(value: target::Syntax) -> records::errors::Syntax {
    match value {
        target::Syntax::TooLong => records::errors::Syntax::TooLong,
        target::Syntax::Character => records::errors::Syntax::Character,
        target::Syntax::Unterminated => records::errors::Syntax::Unterminated,
        target::Syntax::Escape => records::errors::Syntax::Escape,
        target::Syntax::Integer => records::errors::Syntax::Integer,
        target::Syntax::TooDeep => records::errors::Syntax::TooDeep,
        target::Syntax::Misplaced => records::errors::Syntax::Misplaced,
        target::Syntax::UnknownFilter => records::errors::Syntax::UnknownFilter,
        target::Syntax::UnknownTest => records::errors::Syntax::UnknownTest,
        target::Syntax::Utf8 => records::errors::Syntax::Utf8,
        target::Syntax::Return => records::errors::Syntax::Return,
        target::Syntax::Unclosed => records::errors::Syntax::Unclosed,
        target::Syntax::UnknownStatement => records::errors::Syntax::UnknownStatement,
        target::Syntax::Unmatched => records::errors::Syntax::Unmatched,
        target::Syntax::Unended => records::errors::Syntax::Unended,
        target::Syntax::Shadows => records::errors::Syntax::Shadows,
        target::Syntax::Interpolation => records::errors::Syntax::Interpolation,
        target::Syntax::Expected(value) => records::errors::Syntax::Expected(expected_to_syntax_record(value)),
        target::Syntax::Arity { filter, given } => records::errors::Syntax::Arity {
            filter: skein_lib::bytes::copy_of(filter_name(filter)),
            expected: filter.arity(),
            given,
        },
    }
}
#[must_use]
pub fn expected_to_syntax_record(value: target::syntax::Expected) -> records::errors::syntax::Expected {
    match value {
        target::syntax::Expected::Operand => records::errors::syntax::Expected::Operand,
        target::syntax::Expected::Operator => records::errors::syntax::Expected::Operator,
        target::syntax::Expected::Name => records::errors::syntax::Expected::Name,
        target::syntax::Expected::CloseParen => records::errors::syntax::Expected::CloseParen,
        target::syntax::Expected::CloseBracket => records::errors::syntax::Expected::CloseBracket,
        target::syntax::Expected::In => records::errors::syntax::Expected::In,
        target::syntax::Expected::Else => records::errors::syntax::Expected::Else,
        target::syntax::Expected::Statement => records::errors::syntax::Expected::Statement,
        target::syntax::Expected::Assign => records::errors::syntax::Expected::Assign,
        target::syntax::Expected::TagEnd => records::errors::syntax::Expected::TagEnd,
    }
}
fn filter_name(value: render::evaluate::Filter) -> &'static [u8] {
    use render::evaluate::Filter;
    match value {
        Filter::Default => b"default",
        Filter::Join => b"join",
        Filter::Split => b"split",
        Filter::First => b"first",
        Filter::Last => b"last",
        Filter::Length => b"length",
        Filter::Replace => b"replace",
        Filter::Items => b"items",
        Filter::Keys => b"keys",
        Filter::Values => b"values",
        Filter::Sort => b"sort",
        Filter::Tojson => b"tojson",
        Filter::ShellQuote => b"shell_quote",
        Filter::SystemdEscape => b"systemd_escape",
        Filter::SystemdEscapePath => b"systemd_escape_path",
        Filter::Basename => b"basename",
        Filter::Dirname => b"dirname",
    }
}

fn optional_side(side: Option<render::diff::Side>) -> Option<records::diff::Side> {
    Some(super::changes::side(side?))
}

#[must_use]
pub fn composition(why: target::Composition) -> records::errors::Composition {
    match why {
        target::Composition::Duplicate => records::errors::Composition::Duplicate,
        target::Composition::Long => records::errors::Composition::Long,
    }
}

#[must_use]
pub fn evaluation(kind: render::evaluate::FaultKind) -> records::errors::Evaluation {
    use crate::render::evaluate::FaultKind;
    match kind {
        FaultKind::Unknown => records::errors::Evaluation::Unknown,
        FaultKind::NoKey => records::errors::Evaluation::NoKey,
        FaultKind::NoIndex => records::errors::Evaluation::NoIndex,
        FaultKind::Undefined => records::errors::Evaluation::Undefined,
        FaultKind::Type => records::errors::Evaluation::Type,
        FaultKind::Overflow => records::errors::Evaluation::Overflow,
        FaultKind::Empty => records::errors::Evaluation::Empty,
        FaultKind::Path => records::errors::Evaluation::Path,
        FaultKind::Output => records::errors::Evaluation::Output,
        FaultKind::Limit => records::errors::Evaluation::Limit,
    }
}
