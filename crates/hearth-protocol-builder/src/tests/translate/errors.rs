use crate::records::errors as record_errors;
use crate::render::errors as render_errors;
use crate::render::{self, revision as target};
fn decode_back(fault: record_errors::Decode) -> render_errors::Decode {
    match fault {
        record_errors::Decode::Syntax(value) => {
            render_errors::Decode::Syntax(super::instructions::record_syntax_back(value))
        }
        record_errors::Decode::Declaration(value) => render_errors::Decode::Declaration(declared_back(value)),
        record_errors::Decode::Unexpected => render_errors::Decode::Unexpected,
        record_errors::Decode::Trailing => render_errors::Decode::Trailing,
        record_errors::Decode::TooDeep => render_errors::Decode::TooDeep,
        record_errors::Decode::StringTooLong => render_errors::Decode::StringTooLong,
        record_errors::Decode::NumberTooLong => render_errors::Decode::NumberTooLong,
        record_errors::Decode::Number => render_errors::Decode::Number,
        record_errors::Decode::Escape => render_errors::Decode::Escape,
        record_errors::Decode::Surrogate => render_errors::Decode::Surrogate,
        record_errors::Decode::Utf8 => render_errors::Decode::Utf8,
        record_errors::Decode::Control => render_errors::Decode::Control,
        record_errors::Decode::Truncated => render_errors::Decode::Truncated,
        record_errors::Decode::NotObject => render_errors::Decode::NotObject,
        record_errors::Decode::NotInteger => render_errors::Decode::NotInteger,
        record_errors::Decode::Range => render_errors::Decode::Range,
        record_errors::Decode::Nul => render_errors::Decode::Nul,
        record_errors::Decode::Keys => render_errors::Decode::Keys,
        record_errors::Decode::Note => render_errors::Decode::Note,
        record_errors::Decode::ListNull => render_errors::Decode::ListNull,
        record_errors::Decode::Name => render_errors::Decode::Name,
        record_errors::Decode::Placement => render_errors::Decode::Placement,
        record_errors::Decode::Members => render_errors::Decode::Members,
        record_errors::Decode::Member => render_errors::Decode::Member,
        record_errors::Decode::Duplicate { key } => render_errors::Decode::Duplicate { key },
        record_errors::Decode::Settings { task } => render_errors::Decode::Settings { task },
        record_errors::Decode::Unknown { field } => render_errors::Decode::Unknown { field },
        record_errors::Decode::Wrong { field } => render_errors::Decode::Wrong { field: decode_field_back(field) },
        record_errors::Decode::Missing { field } => render_errors::Decode::Missing { field: decode_field_back(field) },
        record_errors::Decode::Misplaced { field } => {
            render_errors::Decode::Misplaced { field: decode_field_back(field) }
        }
        record_errors::Decode::TooLarge { limit, max } => {
            render_errors::Decode::TooLarge { limit: decode_limit_back(limit), max }
        }
    }
}
fn decode_field_back(field: record_errors::Field) -> render_errors::Field {
    match field {
        record_errors::Field::Kind => render_errors::Field::Kind,
        record_errors::Field::Firmware => render_errors::Field::Firmware,
        record_errors::Field::Hardware => render_errors::Field::Hardware,
        record_errors::Field::On => render_errors::Field::On,
        record_errors::Field::Mutable => render_errors::Field::Mutable,
    }
}
fn decode_limit_back(limit: record_errors::Limit) -> render_errors::Limit {
    match limit {
        record_errors::Limit::Template => render_errors::Limit::Template,
        record_errors::Limit::Document => render_errors::Limit::Document,
        record_errors::Limit::Decoded => render_errors::Limit::Decoded,
        record_errors::Limit::Static => render_errors::Limit::Static,
        record_errors::Limit::Tasks => render_errors::Limit::Tasks,
        record_errors::Limit::Targets => render_errors::Limit::Targets,
        record_errors::Limit::Paths => render_errors::Limit::Paths,
        record_errors::Limit::Path => render_errors::Limit::Path,
        record_errors::Limit::Entries => render_errors::Limit::Entries,
    }
}
const CONFIG_FAULTS: &[(crate::config::Fault, render_errors::Decode, record_errors::Decode)] = &[
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::Unexpected),
        render_errors::Decode::Unexpected,
        record_errors::Decode::Unexpected,
    ),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::Trailing),
        render_errors::Decode::Trailing,
        record_errors::Decode::Trailing,
    ),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::TooDeep),
        render_errors::Decode::TooDeep,
        record_errors::Decode::TooDeep,
    ),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::StringTooLong),
        render_errors::Decode::StringTooLong,
        record_errors::Decode::StringTooLong,
    ),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::NumberTooLong),
        render_errors::Decode::NumberTooLong,
        record_errors::Decode::NumberTooLong,
    ),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::Number),
        render_errors::Decode::Number,
        record_errors::Decode::Number,
    ),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::Escape),
        render_errors::Decode::Escape,
        record_errors::Decode::Escape,
    ),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::Surrogate),
        render_errors::Decode::Surrogate,
        record_errors::Decode::Surrogate,
    ),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::Utf8),
        render_errors::Decode::Utf8,
        record_errors::Decode::Utf8,
    ),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::Control),
        render_errors::Decode::Control,
        record_errors::Decode::Control,
    ),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::Truncated),
        render_errors::Decode::Truncated,
        record_errors::Decode::Truncated,
    ),
    (
        crate::config::Fault::Shape(crate::config::Shape::ListNull),
        render_errors::Decode::ListNull,
        record_errors::Decode::ListNull,
    ),
    (crate::config::Fault::NotObject, render_errors::Decode::NotObject, record_errors::Decode::NotObject),
    (crate::config::Fault::NotInteger, render_errors::Decode::NotInteger, record_errors::Decode::NotInteger),
    (crate::config::Fault::Range, render_errors::Decode::Range, record_errors::Decode::Range),
    (crate::config::Fault::Nul, render_errors::Decode::Nul, record_errors::Decode::Nul),
    (crate::config::Fault::Keys, render_errors::Decode::Keys, record_errors::Decode::Keys),
    (crate::config::Fault::Note, render_errors::Decode::Note, record_errors::Decode::Note),
    (crate::config::Fault::Shape(crate::config::Shape::Name), render_errors::Decode::Name, record_errors::Decode::Name),
    (
        crate::config::Fault::Shape(crate::config::Shape::Placement),
        render_errors::Decode::Placement,
        record_errors::Decode::Placement,
    ),
    (
        crate::config::Fault::Shape(crate::config::Shape::Members),
        render_errors::Decode::Members,
        record_errors::Decode::Members,
    ),
    (
        crate::config::Fault::Shape(crate::config::Shape::Member),
        render_errors::Decode::Member,
        record_errors::Decode::Member,
    ),
    (
        crate::config::Fault::Json(skein_json::tokenizer::Error::TooLong),
        render_errors::Decode::TooLarge { limit: render_errors::Limit::Document, max: 1_048_576 },
        record_errors::Decode::TooLarge { limit: record_errors::Limit::Document, max: 1_048_576 },
    ),
];
#[test]
fn each_config_fault_translates_to_its_decode_and_back() {
    for (row, (fault, input, expected)) in CONFIG_FAULTS.iter().enumerate() {
        assert_eq!(
            crate::translate::errors::fault(fault.clone(), &crate::config::limits::SHIPPED),
            Some((input.clone(), None))
        );
        let record = crate::translate::errors::decode(input.clone());
        assert_eq!(record, *expected, "row {row}: record");
        let back = decode_back(record);
        assert_eq!(back, *input, "row {row}: decode");
    }
}
const DECODE_FIELDS: &[(crate::config::Field, render_errors::Field, record_errors::Field)] = &[
    (crate::config::Field::Kind, render_errors::Field::Kind, record_errors::Field::Kind),
    (crate::config::Field::Firmware, render_errors::Field::Firmware, record_errors::Field::Firmware),
    (crate::config::Field::Hardware, render_errors::Field::Hardware, record_errors::Field::Hardware),
    (crate::config::Field::On, render_errors::Field::On, record_errors::Field::On),
    (crate::config::Field::Mutable, render_errors::Field::Mutable, record_errors::Field::Mutable),
];
#[test]
fn shape_field_faults_translate_each_field_and_back() {
    for (row, (source, input, expected)) in DECODE_FIELDS.iter().enumerate() {
        assert_eq!(
            crate::translate::errors::fault(
                crate::config::Fault::Shape(crate::config::Shape::Wrong { field: *source }),
                &crate::config::limits::SHIPPED
            ),
            Some((render_errors::Decode::Wrong { field: *input }, None))
        );
        let record = crate::translate::errors::field(*input);
        assert_eq!(record, *expected, "row {row}: record");
        let back = decode_field_back(record);
        assert_eq!(back, *input, "row {row}: field");
    }
}
#[test]
fn a_duplicate_fault_translates_the_key_and_first_position() {
    let fault = crate::config::Fault::Duplicate {
        key: skein_lib::bytes::copy_of(b"motd"),
        first: crate::config::Position { offset: 1, line: 1, column: 2 },
    };
    assert_eq!(
        crate::translate::errors::fault(fault, &crate::config::limits::SHIPPED),
        Some((
            render_errors::Decode::Duplicate { key: skein_lib::bytes::copy_of(b"motd") },
            Some(target::Position { line: 1, column: 2 })
        ))
    );
}

const SHAPE_FAULTS: &[(crate::config::Shape, render_errors::Decode, record_errors::Decode)] = &[
    (
        crate::config::Shape::Wrong { field: crate::config::Field::Kind },
        render_errors::Decode::Wrong { field: render_errors::Field::Kind },
        record_errors::Decode::Wrong { field: record_errors::Field::Kind },
    ),
    (
        crate::config::Shape::Missing { field: crate::config::Field::Kind },
        render_errors::Decode::Missing { field: render_errors::Field::Kind },
        record_errors::Decode::Missing { field: record_errors::Field::Kind },
    ),
    (
        crate::config::Shape::Misplaced { field: crate::config::Field::Kind },
        render_errors::Decode::Misplaced { field: render_errors::Field::Kind },
        record_errors::Decode::Misplaced { field: record_errors::Field::Kind },
    ),
];
#[test]
fn each_shape_field_fault_translates_its_kind_and_back() {
    for (row, (shape, input, expected)) in SHAPE_FAULTS.iter().enumerate() {
        assert_eq!(
            crate::translate::errors::fault(
                crate::config::Fault::Shape(shape.clone()),
                &crate::config::limits::SHIPPED
            ),
            Some((input.clone(), None))
        );
        let record = crate::translate::errors::decode(input.clone());
        assert_eq!(record, *expected, "row {row}: record");
        let back = decode_back(record);
        assert_eq!(back, *input, "row {row}: decode");
    }
}
const DECODE_LIMITS: &[(render_errors::Limit, record_errors::Limit)] = &[
    (render_errors::Limit::Document, record_errors::Limit::Document),
    (render_errors::Limit::Decoded, record_errors::Limit::Decoded),
    (render_errors::Limit::Static, record_errors::Limit::Static),
    (render_errors::Limit::Tasks, record_errors::Limit::Tasks),
    (render_errors::Limit::Targets, record_errors::Limit::Targets),
    (render_errors::Limit::Paths, record_errors::Limit::Paths),
    (render_errors::Limit::Path, record_errors::Limit::Path),
    (render_errors::Limit::Entries, record_errors::Limit::Entries),
];
#[test]
fn each_decode_limit_translates_and_back() {
    for (row, (source, _record)) in DECODE_LIMITS.iter().enumerate() {
        let back = decode_limit_back(crate::translate::errors::limit(*source));
        assert_eq!(back, *source, "row {row}: render limit through records and back");
    }
}
#[test]
fn a_decode_error_translates_each_place_and_scope() {
    let error = crate::tests::fixtures::decode_error();
    assert_eq!(
        crate::translate::errors::error(error),
        record_errors::Error {
            side: None,
            kind: record_errors::Kind::Decode(record_errors::Decode::Duplicate {
                key: skein_lib::bytes::copy_of(b"a")
            }),
            class: record_errors::Class::Configuration,
            at: Some(record_errors::Place {
                path: skein_lib::bytes::copy_of(b"tasks/motd/task.json"),
                position: Some(record_errors::Position { line: 1, column: 10 })
            }),
            also: Some(record_errors::Place {
                path: skein_lib::bytes::copy_of(b"tasks/motd/task.json"),
                position: Some(record_errors::Position { line: 1, column: 2 })
            }),
            target: None,
            task: Some(skein_lib::bytes::copy_of(b"motd"))
        }
    );
}

#[test]
fn a_task_settings_fault_translates_its_task_name() {
    assert_eq!(
        crate::translate::errors::fault(
            crate::config::Fault::Shape(crate::config::Shape::Settings { task: skein_lib::bytes::copy_of(b"motd") }),
            &crate::config::limits::SHIPPED
        ),
        Some((render_errors::Decode::Settings { task: skein_lib::bytes::copy_of(b"motd") }, None))
    );
    assert_eq!(
        decode_back(crate::translate::errors::decode(render_errors::Decode::Settings {
            task: skein_lib::bytes::copy_of(b"motd")
        })),
        render_errors::Decode::Settings { task: skein_lib::bytes::copy_of(b"motd") }
    );
}
#[test]
fn an_unknown_field_fault_translates_its_field_name() {
    assert_eq!(
        crate::translate::errors::fault(
            crate::config::Fault::Shape(crate::config::Shape::Unknown { field: skein_lib::bytes::copy_of(b"ids") }),
            &crate::config::limits::SHIPPED
        ),
        Some((render_errors::Decode::Unknown { field: skein_lib::bytes::copy_of(b"ids") }, None))
    );
    assert_eq!(
        decode_back(crate::translate::errors::decode(render_errors::Decode::Unknown {
            field: skein_lib::bytes::copy_of(b"ids")
        })),
        render_errors::Decode::Unknown { field: skein_lib::bytes::copy_of(b"ids") }
    );
}

fn declared_back(value: record_errors::Declared) -> render_errors::Declared {
    match value {
        record_errors::Declared::Unknown { of } => render_errors::Declared::Unknown { of: object_back(of) },
        record_errors::Declared::Later => render_errors::Declared::Later,
        record_errors::Declared::Wrong { expected } => {
            render_errors::Declared::Wrong { expected: expected_back(expected) }
        }
        record_errors::Declared::Missing { key } => render_errors::Declared::Missing { key: required_back(key) },
        record_errors::Declared::Name => render_errors::Declared::Name,
        record_errors::Declared::Doc => render_errors::Declared::Doc,
        record_errors::Declared::Order => render_errors::Declared::Order,
        record_errors::Declared::Optional => render_errors::Declared::Optional,
        record_errors::Declared::Bound => render_errors::Declared::Bound,
        record_errors::Declared::Above => render_errors::Declared::Above,
        record_errors::Declared::Enum => render_errors::Declared::Enum,
        record_errors::Declared::Settings => render_errors::Declared::Settings,
        record_errors::Declared::Secret => render_errors::Declared::Secret,
    }
}

fn object_back(value: record_errors::Object) -> render_errors::Object {
    match value {
        record_errors::Object::Declaration => render_errors::Object::Declaration,
        record_errors::Object::Read => render_errors::Object::Read,
        record_errors::Object::Guard => render_errors::Object::Guard,
        record_errors::Object::Script => render_errors::Object::Script,
        record_errors::Object::Bless => render_errors::Object::Bless,
        record_errors::Object::Entry(map) => render_errors::Object::Entry(declaration_map_back(map)),
        record_errors::Object::Range => render_errors::Object::Range,
        record_errors::Object::Spec => render_errors::Object::Spec,
        record_errors::Object::Export => render_errors::Object::Export,
        record_errors::Object::Type => render_errors::Object::Type,
    }
}

fn expected_back(value: record_errors::Expected) -> render_errors::Expected {
    match value {
        record_errors::Expected::Object => render_errors::Expected::Object,
        record_errors::Expected::String => render_errors::Expected::String,
        record_errors::Expected::Strings => render_errors::Expected::Strings,
        record_errors::Expected::Bool => render_errors::Expected::Bool,
        record_errors::Expected::Int => render_errors::Expected::Int,
        record_errors::Expected::Field(takes) => render_errors::Expected::Field(takes_back(takes)),
        record_errors::Expected::True => render_errors::Expected::True,
        record_errors::Expected::Type => render_errors::Expected::Type,
    }
}

fn required_back(value: record_errors::Required) -> render_errors::Required {
    match value {
        record_errors::Required::If => render_errors::Required::If,
        record_errors::Required::Message => render_errors::Required::Message,
        record_errors::Required::Type => render_errors::Required::Type,
        record_errors::Required::Doc => render_errors::Required::Doc,
        record_errors::Required::Value => render_errors::Required::Value,
    }
}

fn takes_back(value: record_errors::Takes) -> render_errors::Takes {
    match value {
        record_errors::Takes::Text => render_errors::Takes::Text,
        record_errors::Takes::Int => render_errors::Takes::Int,
        record_errors::Takes::Bool => render_errors::Takes::Bool,
        record_errors::Takes::Strings => render_errors::Takes::Strings,
        record_errors::Takes::Range => render_errors::Takes::Range,
    }
}

fn declaration_map_back(value: record_errors::Map) -> render::revision::declaration::Map {
    match value {
        record_errors::Map::Packages => render::revision::declaration::Map::Packages,
        record_errors::Map::Preseeds => render::revision::declaration::Map::Preseeds,
        record_errors::Map::Initramfs => render::revision::declaration::Map::Initramfs,
        record_errors::Map::Software => render::revision::declaration::Map::Software,
        record_errors::Map::Files => render::revision::declaration::Map::Files,
        record_errors::Map::Users => render::revision::declaration::Map::Users,
        record_errors::Map::Groups => render::revision::declaration::Map::Groups,
        record_errors::Map::Presets => render::revision::declaration::Map::Presets,
        record_errors::Map::Tmpfiles => render::revision::declaration::Map::Tmpfiles,
        record_errors::Map::Data => render::revision::declaration::Map::Data,
        record_errors::Map::Secrets => render::revision::declaration::Map::Secrets,
        record_errors::Map::Identity => render::revision::declaration::Map::Identity,
        record_errors::Map::Cmdline => render::revision::declaration::Map::Cmdline,
        record_errors::Map::Live => render::revision::declaration::Map::Live,
    }
}

fn declared_config(value: render_errors::Declared) -> crate::config::Declared {
    match value {
        render_errors::Declared::Unknown { of } => crate::config::Declared::Unknown { of: object_config(of) },
        render_errors::Declared::Later => crate::config::Declared::Later,
        render_errors::Declared::Wrong { expected } => {
            crate::config::Declared::Wrong { expected: expected_config(expected) }
        }
        render_errors::Declared::Missing { key } => crate::config::Declared::Missing { key: required_config(key) },
        render_errors::Declared::Name => crate::config::Declared::Name,
        render_errors::Declared::Doc => crate::config::Declared::Doc,
        render_errors::Declared::Order => crate::config::Declared::Order,
        render_errors::Declared::Optional => crate::config::Declared::Optional,
        render_errors::Declared::Bound => crate::config::Declared::Bound,
        render_errors::Declared::Above => crate::config::Declared::Above,
        render_errors::Declared::Enum => crate::config::Declared::Enum,
        render_errors::Declared::Settings => crate::config::Declared::Settings,
        render_errors::Declared::Secret => crate::config::Declared::Secret,
    }
}

fn object_config(value: render_errors::Object) -> crate::config::Object {
    match value {
        render_errors::Object::Declaration => crate::config::Object::Declaration,
        render_errors::Object::Read => crate::config::Object::Read,
        render_errors::Object::Guard => crate::config::Object::Guard,
        render_errors::Object::Script => crate::config::Object::Script,
        render_errors::Object::Bless => crate::config::Object::Bless,
        render_errors::Object::Entry(map) => crate::config::Object::Entry(declaration_map_config(map)),
        render_errors::Object::Range => crate::config::Object::Range,
        render_errors::Object::Spec => crate::config::Object::Spec,
        render_errors::Object::Export => crate::config::Object::Export,
        render_errors::Object::Type => crate::config::Object::Type,
    }
}

fn expected_config(value: render_errors::Expected) -> crate::config::Expected {
    match value {
        render_errors::Expected::Object => crate::config::Expected::Object,
        render_errors::Expected::String => crate::config::Expected::String,
        render_errors::Expected::Strings => crate::config::Expected::Strings,
        render_errors::Expected::Bool => crate::config::Expected::Bool,
        render_errors::Expected::Int => crate::config::Expected::Int,
        render_errors::Expected::Field(takes) => crate::config::Expected::Field(takes_config(takes)),
        render_errors::Expected::True => crate::config::Expected::True,
        render_errors::Expected::Type => crate::config::Expected::Type,
    }
}

fn required_config(value: render_errors::Required) -> crate::config::Required {
    match value {
        render_errors::Required::If => crate::config::Required::If,
        render_errors::Required::Message => crate::config::Required::Message,
        render_errors::Required::Type => crate::config::Required::Type,
        render_errors::Required::Doc => crate::config::Required::Doc,
        render_errors::Required::Value => crate::config::Required::Value,
    }
}

fn takes_config(value: render_errors::Takes) -> crate::config::entries::Takes {
    match value {
        render_errors::Takes::Text => crate::config::entries::Takes::Text,
        render_errors::Takes::Int => crate::config::entries::Takes::Int,
        render_errors::Takes::Bool => crate::config::entries::Takes::Bool,
        render_errors::Takes::Strings => crate::config::entries::Takes::Strings,
        render_errors::Takes::Range => crate::config::entries::Takes::Range,
    }
}

fn declaration_map_config(value: render::revision::declaration::Map) -> crate::config::entries::Map {
    match value {
        render::revision::declaration::Map::Packages => crate::config::entries::Map::Packages,
        render::revision::declaration::Map::Preseeds => crate::config::entries::Map::Preseeds,
        render::revision::declaration::Map::Initramfs => crate::config::entries::Map::Initramfs,
        render::revision::declaration::Map::Software => crate::config::entries::Map::Software,
        render::revision::declaration::Map::Files => crate::config::entries::Map::Files,
        render::revision::declaration::Map::Users => crate::config::entries::Map::Users,
        render::revision::declaration::Map::Groups => crate::config::entries::Map::Groups,
        render::revision::declaration::Map::Presets => crate::config::entries::Map::Presets,
        render::revision::declaration::Map::Tmpfiles => crate::config::entries::Map::Tmpfiles,
        render::revision::declaration::Map::Data => crate::config::entries::Map::Data,
        render::revision::declaration::Map::Secrets => crate::config::entries::Map::Secrets,
        render::revision::declaration::Map::Identity => crate::config::entries::Map::Identity,
        render::revision::declaration::Map::Cmdline => crate::config::entries::Map::Cmdline,
        render::revision::declaration::Map::Live => crate::config::entries::Map::Live,
    }
}

const DECLARED_FAULTS: &[crate::config::Declared] = &[
    crate::config::Declared::Later,
    crate::config::Declared::Name,
    crate::config::Declared::Doc,
    crate::config::Declared::Order,
    crate::config::Declared::Optional,
    crate::config::Declared::Bound,
    crate::config::Declared::Above,
    crate::config::Declared::Enum,
    crate::config::Declared::Settings,
    crate::config::Declared::Secret,
    crate::config::Declared::Unknown { of: crate::config::Object::Declaration },
    crate::config::Declared::Unknown { of: crate::config::Object::Read },
    crate::config::Declared::Unknown { of: crate::config::Object::Guard },
    crate::config::Declared::Unknown { of: crate::config::Object::Script },
    crate::config::Declared::Unknown { of: crate::config::Object::Bless },
    crate::config::Declared::Unknown { of: crate::config::Object::Range },
    crate::config::Declared::Unknown { of: crate::config::Object::Spec },
    crate::config::Declared::Unknown { of: crate::config::Object::Export },
    crate::config::Declared::Unknown { of: crate::config::Object::Type },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Packages) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Preseeds) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Initramfs) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Software) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Files) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Users) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Groups) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Presets) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Tmpfiles) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Data) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Secrets) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Identity) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Cmdline) },
    crate::config::Declared::Unknown { of: crate::config::Object::Entry(crate::config::entries::Map::Live) },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Object },
    crate::config::Declared::Wrong { expected: crate::config::Expected::String },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Strings },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Bool },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Int },
    crate::config::Declared::Wrong { expected: crate::config::Expected::True },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Type },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Field(crate::config::entries::Takes::Text) },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Field(crate::config::entries::Takes::Int) },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Field(crate::config::entries::Takes::Bool) },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Field(crate::config::entries::Takes::Strings) },
    crate::config::Declared::Wrong { expected: crate::config::Expected::Field(crate::config::entries::Takes::Range) },
    crate::config::Declared::Missing { key: crate::config::Required::If },
    crate::config::Declared::Missing { key: crate::config::Required::Message },
    crate::config::Declared::Missing { key: crate::config::Required::Type },
    crate::config::Declared::Missing { key: crate::config::Required::Doc },
    crate::config::Declared::Missing { key: crate::config::Required::Value },
];
#[test]
fn each_declared_fault_translates_to_render_and_back() {
    for (row, &input) in DECLARED_FAULTS.iter().enumerate() {
        let rendered = crate::translate::errors::declared(input);
        let back = declared_config(declared_back(crate::translate::errors::declared_to_records(rendered)));
        assert_eq!(back, input, "row {row}: declaration fault");
    }
}
