use super::*;
use crate::config;
#[must_use]
pub fn read_declaration(body: &[u8]) -> render::revision::declaration::Declaration {
    let mut layer = Layer::new();
    layer.read(Box::new([render::vocabulary::Blob { kind: render::vocabulary::As::Declaration, ..blob(body) }]));
    let builder::vocabulary::Event::Render(render::vocabulary::Event::Read {
        read: render::vocabulary::Decoded::Declaration(declaration),
        ..
    }) = layer.run(&batch(body), Exit::Code(0))
    else {
        panic!("decoded declaration")
    };
    declaration
}

pub const GUARDED: &[u8] = br#"{"requires":{"ssh":{"if":"true","message":"keep as written"}}}"#;
pub fn declaration_event(body: &[u8]) -> builder::vocabulary::Event {
    let mut layer = Layer::new();
    layer.read(Box::new([render::vocabulary::Blob { kind: render::vocabulary::As::Declaration, ..blob(body) }]));
    layer.run(&batch(body), Exit::Code(0))
}

pub fn guard_conditions() -> [(config::declaration::Guard, hearth_protocol_template::Expression); 2] {
    [(b"true".as_slice(), true), (b"false".as_slice(), false)].map(|(condition, value)| {
        let at = config::Position { offset: 0, line: 3, column: 5 };
        let text = |text: &[u8]| config::declaration::Text { at, text: bytes::copy_of(text) };
        (
            config::declaration::Guard { name: text(b"ssh"), at, when: text(condition), message: text(b"plain") },
            hearth_protocol_template::Expression {
                values: 1,
                operations: Box::new([hearth_protocol_template::Operation {
                    at: 0,
                    op: hearth_protocol_template::Op::Boolean(value),
                }]),
            },
        )
    })
}

pub fn guard_faults() -> [render::errors::Guard; 2] {
    [
        render::errors::Guard { name: bytes::copy_of(b"ssh"), message: bytes::copy_of(b"port needed") },
        render::errors::Guard { name: bytes::copy_of(b"web"), message: bytes::copy_of(b"web needed") },
    ]
}

#[must_use]
pub fn every_entry_condition() -> [(config::entries::Entry, hearth_protocol_template::Expression); 2] {
    guard_conditions().map(|(guard, expression)| {
        (
            config::entries::Entry {
                key: guard.name,
                at: guard.at,
                when: Some(guard.when.clone()),
                for_each: Some(guard.when),
                fields: Box::new([]),
            },
            expression,
        )
    })
}
#[must_use]
pub fn collision_faults() -> [render::errors::Collision; 2] {
    [
        render::errors::Collision::Key { key: bytes::copy_of(b"ops") },
        render::errors::Collision::Key { key: bytes::copy_of(b"dev") },
    ]
}

pub const FIELD_NAMES: &[config::entries::Name] = &[
    config::entries::Name::Action,
    config::entries::Name::Age,
    config::entries::Name::Algorithm,
    config::entries::Name::Argument,
    config::entries::Name::Base,
    config::entries::Name::Class,
    config::entries::Name::Component,
    config::entries::Name::Copy,
    config::entries::Name::Count,
    config::entries::Name::Directory,
    config::entries::Name::From,
    config::entries::Name::Gecos,
    config::entries::Name::Gid,
    config::entries::Name::Group,
    config::entries::Name::Groups,
    config::entries::Name::Home,
    config::entries::Name::Kind,
    config::entries::Name::Link,
    config::entries::Name::Mode,
    config::entries::Name::Order,
    config::entries::Name::Owner,
    config::entries::Name::Param,
    config::entries::Name::Password,
    config::entries::Name::Path,
    config::entries::Name::Paths,
    config::entries::Name::PublicPath,
    config::entries::Name::Sha256,
    config::entries::Name::Shell,
    config::entries::Name::Strip,
    config::entries::Name::Subordinate,
    config::entries::Name::Template,
    config::entries::Name::Type,
    config::entries::Name::Uid,
    config::entries::Name::Unit,
    config::entries::Name::Unpack,
    config::entries::Name::Url,
    config::entries::Name::Value,
];
pub fn declaration_text(text: &[u8]) -> config::declaration::Text {
    config::declaration::Text { at: config::Position { offset: 0, line: 3, column: 5 }, text: bytes::copy_of(text) }
}
#[must_use]
pub fn every_literal() -> [config::entries::Literal; 7] {
    use config::{
        declaration::Number,
        entries::{Field, Literal, Name},
    };
    let text = declaration_text(b"plain");
    let at = text.at;
    [
        Literal::Text(text.clone()),
        Literal::Plain(declaration_text(b"{{ key }}.j2")),
        Literal::Typed(text.clone()),
        Literal::Int(Number { at, value: 42 }),
        Literal::Bool { at, value: true },
        Literal::Strings { at, each: Box::new([text]) },
        Literal::Range {
            at,
            fields: Box::new([
                Field { name: Name::Base, at, value: Literal::Int(Number { at, value: 42 }) },
                Field { name: Name::Count, at, value: Literal::Int(Number { at, value: 2 }) },
            ]),
        },
    ]
}
