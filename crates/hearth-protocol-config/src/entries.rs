//! Uniform declaration entries and their JSON forms (protocol/format.md, 5.2 to 5.7, 5.14, 9.1, 9.3, 10.1).
use crate::{
    Declared, Error, Expected, Object, Position,
    declaration::{Number, Text, fault, object, text, texts, unknown, wrong},
    json::{Kind, Value},
};
use alloc::boxed::Box;
use skein_lib::List;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Map {
    Packages,
    Preseeds,
    Initramfs,
    Cmdline,
    Files,
    Users,
    Groups,
    Presets,
    Tmpfiles,
    Data,
    Secrets,
    Identity,
    Software,
    Live,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entries {
    pub map: Map,
    pub at: Position,
    pub entries: Box<[Entry]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub key: Text,
    pub at: Position,
    pub when: Option<Text>,
    pub for_each: Option<Text>,
    pub fields: Box<[Field]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Field {
    pub name: Name,
    pub at: Position,
    pub value: Literal,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Literal {
    Text(Text),
    Plain(Text),
    Typed(Text),
    Int(Number),
    Bool { at: Position, value: bool },
    Strings { at: Position, each: Box<[Text]> },
    Range { at: Position, fields: Box<[Field]> },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Takes {
    Text,
    Int,
    Bool,
    Strings,
    Range,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Name {
    Action,
    Age,
    Algorithm,
    Argument,
    Base,
    Class,
    Component,
    Copy,
    Count,
    Directory,
    From,
    Gecos,
    Gid,
    Group,
    Groups,
    Home,
    Kind,
    Link,
    Mode,
    Order,
    Owner,
    Param,
    Password,
    Path,
    Paths,
    PublicPath,
    Sha256,
    Shell,
    Strip,
    Subordinate,
    Template,
    Type,
    Uid,
    Unit,
    Unpack,
    Url,
    Value,
}
type FieldShape = (&'static [u8], Name, Takes);
const FIELDS: &[(Map, &[FieldShape])] = &[
    (Map::Packages, &[(b"component", Name::Component, Takes::Text)]),
    (
        Map::Preseeds,
        &[
            (b"type", Name::Type, Takes::Text),
            (b"value", Name::Value, Takes::Text),
            (b"owner", Name::Owner, Takes::Text),
        ],
    ),
    (Map::Initramfs, &[]),
    (Map::Cmdline, &[(b"param", Name::Param, Takes::Text), (b"order", Name::Order, Takes::Int)]),
    (
        Map::Files,
        &[
            (b"template", Name::Template, Takes::Text),
            (b"copy", Name::Copy, Takes::Text),
            (b"link", Name::Link, Takes::Text),
            (b"mode", Name::Mode, Takes::Text),
            (b"owner", Name::Owner, Takes::Text),
            (b"group", Name::Group, Takes::Text),
            (b"directory", Name::Directory, Takes::Bool),
        ],
    ),
    (
        Map::Users,
        &[
            (b"group", Name::Group, Takes::Text),
            (b"gecos", Name::Gecos, Takes::Text),
            (b"home", Name::Home, Takes::Text),
            (b"shell", Name::Shell, Takes::Text),
            (b"password", Name::Password, Takes::Text),
            (b"from", Name::From, Takes::Text),
            (b"uid", Name::Uid, Takes::Int),
            (b"gid", Name::Gid, Takes::Int),
            (b"groups", Name::Groups, Takes::Strings),
            (b"subordinate", Name::Subordinate, Takes::Range),
        ],
    ),
    (Map::Groups, &[(b"from", Name::From, Takes::Text), (b"gid", Name::Gid, Takes::Int)]),
    (Map::Presets, &[(b"action", Name::Action, Takes::Text)]),
    (
        Map::Tmpfiles,
        &[
            (b"type", Name::Type, Takes::Text),
            (b"mode", Name::Mode, Takes::Text),
            (b"owner", Name::Owner, Takes::Text),
            (b"group", Name::Group, Takes::Text),
            (b"age", Name::Age, Takes::Text),
            (b"argument", Name::Argument, Takes::Text),
        ],
    ),
    (
        Map::Data,
        &[
            (b"path", Name::Path, Takes::Text),
            (b"mode", Name::Mode, Takes::Text),
            (b"owner", Name::Owner, Takes::Text),
            (b"group", Name::Group, Takes::Text),
            (b"class", Name::Class, Takes::Text),
        ],
    ),
    (
        Map::Secrets,
        &[
            (b"path", Name::Path, Takes::Text),
            (b"mode", Name::Mode, Takes::Text),
            (b"owner", Name::Owner, Takes::Text),
            (b"group", Name::Group, Takes::Text),
        ],
    ),
    (
        Map::Identity,
        &[
            (b"kind", Name::Kind, Takes::Text),
            (b"algorithm", Name::Algorithm, Takes::Text),
            (b"path", Name::Path, Takes::Text),
            (b"public_path", Name::PublicPath, Takes::Text),
            (b"mode", Name::Mode, Takes::Text),
            (b"owner", Name::Owner, Takes::Text),
            (b"group", Name::Group, Takes::Text),
        ],
    ),
    (
        Map::Software,
        &[
            (b"url", Name::Url, Takes::Text),
            (b"sha256", Name::Sha256, Takes::Text),
            (b"unpack", Name::Unpack, Takes::Text),
            (b"path", Name::Path, Takes::Text),
            (b"mode", Name::Mode, Takes::Text),
            (b"owner", Name::Owner, Takes::Text),
            (b"group", Name::Group, Takes::Text),
            (b"strip", Name::Strip, Takes::Int),
        ],
    ),
    (
        Map::Live,
        &[
            (b"unit", Name::Unit, Takes::Text),
            (b"action", Name::Action, Takes::Text),
            (b"paths", Name::Paths, Takes::Strings),
        ],
    ),
];
pub(crate) fn read(map: Map, at: Position, value: Value) -> Result<Entries, Error> {
    let entries = object(value)?;
    let mut each = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
    for entry in entries {
        let mut item = Entry {
            key: Text { at: entry.at, text: entry.key },
            at: entry.value.at,
            when: None,
            for_each: None,
            fields: Box::new([]),
        };
        let fields = object(entry.value)?;
        let mut each_field = List::with_capacity(u32::try_from(fields.len()).expect("document bounded by u32"));
        for field in fields {
            match &*field.key {
                b"if" => item.when = Some(text(field.value)?),
                b"for_each" => item.for_each = Some(text(field.value)?),
                b"artifact" if map == Map::Software => return Err(fault(field.at, Declared::Later)),
                _ => {
                    let (name, takes) = shape(map, &field.key).ok_or(unknown(field.at, Object::Entry(map)))?;
                    let value = literal(field.value, takes, map == Map::Files && name == Name::Template)?;
                    each_field.push(Field { name, at: field.at, value }).expect("one field per source entry");
                }
            }
        }
        item.fields = each_field.into_boxed();
        each.push(item).expect("one entry per source entry");
    }
    Ok(Entries { map, at, entries: each.into_boxed() })
}
fn shape(map: Map, key: &[u8]) -> Option<(Name, Takes)> {
    for &(which, fields) in FIELDS {
        if which != map {
            continue;
        }
        for &(field, name, takes) in fields {
            if field == key {
                return Some((name, takes));
            }
        }
    }
    None
}
fn literal(value: Value, takes: Takes, plain: bool) -> Result<Literal, Error> {
    if takes != Takes::Range {
        return scalar(value, takes, plain);
    }
    match value.kind {
        Kind::String(text) => Ok(Literal::Typed(Text { at: value.at, text })),
        Kind::Map(entries) => {
            let mut fields = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
            for entry in entries {
                let name = match &*entry.key {
                    b"base" => Name::Base,
                    b"count" => Name::Count,
                    _ => return Err(unknown(entry.at, Object::Range)),
                };
                let literal = scalar(entry.value, Takes::Int, false)?;
                fields.push(Field { name, at: entry.at, value: literal }).expect("one field per source entry");
            }
            Ok(Literal::Range { at: value.at, fields: fields.into_boxed() })
        }
        Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::List(_) => Err(wrong(value.at, Expected::Field(takes))),
    }
}
fn scalar(value: Value, takes: Takes, plain: bool) -> Result<Literal, Error> {
    let at = value.at;
    match value.kind {
        Kind::String(text) => {
            let text = Text { at, text };
            Ok(if plain {
                Literal::Plain(text)
            } else if takes == Takes::Text {
                Literal::Text(text)
            } else {
                Literal::Typed(text)
            })
        }
        Kind::Int(value) if takes == Takes::Int => Ok(Literal::Int(Number { at, value })),
        Kind::Bool(value) if takes == Takes::Bool => Ok(Literal::Bool { at, value }),
        Kind::List(each) if takes == Takes::Strings => {
            let each = texts(Value { at, kind: Kind::List(each) }, false)?;
            Ok(Literal::Strings { at, each })
        }
        Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::Map(_) => {
            Err(wrong(at, Expected::Field(takes)))
        }
    }
}
