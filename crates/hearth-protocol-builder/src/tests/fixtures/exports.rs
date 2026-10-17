use crate::render;
use skein_lib::bytes::copy_of;
pub const EXPORT_PRINT_DEPTH: u32 = 32;
#[must_use]
pub fn export_values() -> [render::evaluate::Value; 5] {
    use render::evaluate::{Entry, Value};
    [
        Value::Bool(true),
        Value::Int(-7),
        Value::String(copy_of(b"vpn")),
        Value::List(Box::new([Value::Int(1)])),
        Value::Map(Box::new([Entry { key: copy_of(b"a"), value: Some(Value::Int(2)) }])),
    ]
}
pub fn nested_export(depth: u32) -> render::evaluate::Value {
    let mut value = render::evaluate::Value::Int(1);
    for _ in 0..depth {
        value = render::evaluate::Value::List(Box::new([value]));
    }
    value
}

#[must_use]
pub fn export_specs() -> [render::parts::Spec; 2] {
    let plain = super::spec();
    let mut member = super::spec();
    member.meta.member = Some(copy_of(b"1"));
    [plain, member]
}
pub const EXPORTED_CAUSES: &[render::errors::Cause] = &[
    render::errors::Cause::Decode(render::errors::Decode::Unexpected),
    render::errors::Cause::Repository(render::errors::Repository::Stray),
    render::errors::Cause::Composition(render::errors::Composition::Null),
    render::errors::Cause::Schema(render::errors::Schema::Main),
    render::errors::Cause::Evaluation(render::evaluate::FaultKind::Type),
];
#[must_use]
pub fn exported_error(cause: render::errors::Cause) -> render::errors::Error {
    render::errors::Error {
        side: None,
        kind: render::errors::Kind::Exported(render::errors::Exported {
            exporter: copy_of(b"ion-prod"),
            task: copy_of(b"vpn"),
            cause,
        }),
        class: render::vocabulary::Class::Configuration,
        at: None,
        also: None,
        target: Some(copy_of(b"carbon-prod")),
        task: Some(copy_of(b"reader")),
    }
}

#[must_use]
pub fn export_changes() -> [render::parts::exports::Change; 3] {
    use render::{
        evaluate::Value,
        parts::exports::{Change, ExportChange},
    };
    [
        Change {
            entries: Box::new([ExportChange {
                task: copy_of(b"vpn"),
                name: copy_of(b"endpoint"),
                before: Some(Value::String(copy_of(b"a"))),
                after: Some(Value::String(copy_of(b"b"))),
            }]),
        },
        Change {
            entries: Box::new([ExportChange {
                task: copy_of(b"vpn"),
                name: copy_of(b"port"),
                before: None,
                after: Some(Value::Int(51820)),
            }]),
        },
        Change {
            entries: Box::new([ExportChange {
                task: copy_of(b"core"),
                name: copy_of(b"nest"),
                before: Some(nested_export(EXPORT_PRINT_DEPTH)),
                after: None,
            }]),
        },
    ]
}
