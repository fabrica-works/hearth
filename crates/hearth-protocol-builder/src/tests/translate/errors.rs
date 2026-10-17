use crate::records::errors as record_errors;
use crate::render::errors as render_errors;
use crate::render::{self, revision as target};
fn decode_back(fault: record_errors::Decode) -> render_errors::Decode {
    match fault {
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
        record_errors::Limit::Document => render_errors::Limit::Document,
        record_errors::Limit::Decoded => render_errors::Limit::Decoded,
        record_errors::Limit::Static => render_errors::Limit::Static,
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
    for (fault, decoded, record) in CONFIG_FAULTS {
        assert_eq!(
            crate::translate::errors::fault(fault.clone(), &crate::config::limits::SHIPPED),
            Some((decoded.clone(), None))
        );
        assert_eq!(crate::translate::errors::decode(decoded.clone()), *record);
        assert_eq!(decode_back(record.clone()), *decoded);
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
    for (source, field, record) in DECODE_FIELDS {
        assert_eq!(
            crate::translate::errors::fault(
                crate::config::Fault::Shape(crate::config::Shape::Wrong { field: *source }),
                &crate::config::limits::SHIPPED
            ),
            Some((render_errors::Decode::Wrong { field: *field }, None))
        );
        assert_eq!(crate::translate::errors::field(*field), *record);
        assert_eq!(decode_field_back(*record), *field);
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
    for (shape, decoded, record) in SHAPE_FAULTS {
        assert_eq!(
            crate::translate::errors::fault(
                crate::config::Fault::Shape(shape.clone()),
                &crate::config::limits::SHIPPED
            ),
            Some((decoded.clone(), None))
        );
        assert_eq!(crate::translate::errors::decode(decoded.clone()), *record);
        assert_eq!(decode_back(record.clone()), *decoded);
    }
}
const DECODE_LIMITS: &[(render_errors::Limit, record_errors::Limit)] = &[
    (render_errors::Limit::Document, record_errors::Limit::Document),
    (render_errors::Limit::Decoded, record_errors::Limit::Decoded),
    (render_errors::Limit::Static, record_errors::Limit::Static),
];
#[test]
fn each_decode_limit_translates_and_back() {
    for (source, record) in DECODE_LIMITS {
        assert_eq!(crate::translate::errors::limit(*source), *record);
        assert_eq!(decode_limit_back(*record), *source);
    }
}
#[test]
fn a_decode_error_translates_each_place_and_scope() {
    let error = render_errors::Error {
        kind: render_errors::Kind::Decode(render_errors::Decode::Duplicate { key: skein_lib::bytes::copy_of(b"a") }),
        class: render::vocabulary::Class::Configuration,
        at: Some(render_errors::Place {
            path: skein_lib::bytes::copy_of(b"tasks/motd/task.json"),
            position: Some(target::Position { line: 1, column: 10 }),
        }),
        also: Some(render_errors::Place {
            path: skein_lib::bytes::copy_of(b"tasks/motd/task.json"),
            position: Some(target::Position { line: 1, column: 2 }),
        }),
        target: None,
        task: Some(skein_lib::bytes::copy_of(b"motd")),
    };
    assert_eq!(
        crate::translate::errors::error(error),
        record_errors::Error {
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
