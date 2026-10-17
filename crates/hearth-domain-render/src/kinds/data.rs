//! Persistent directory declarations (format.md, 5.7).
use super::secrets::{fault, formed, mode, required, text};
use crate::{
    entries::Located,
    errors::{Error, Format, Schema},
    parts::data::DataClass,
    revision::{Position, declaration::Name},
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub name: Box<[u8]>,
    pub task: Box<[u8]>,
    pub at: Position,
    pub class: DataClass,
    pub path: Box<[u8]>,
    pub mode: u16,
    pub owner: Box<[u8]>,
    pub group: Box<[u8]>,
}
pub(crate) fn entry(located: &Located<'_>) -> Result<Entry, Box<Error>> {
    let entry = located.entry;
    if let Err(schema) = formed(&entry.key, Format::DataName) {
        return Err(Box::new(fault(located, None, schema)));
    }
    let path = match required(entry, Name::Path) {
        Ok(path) => path,
        Err(schema) => return Err(Box::new(fault(located, None, schema))),
    };
    if let Err(schema) = formed(path, Format::Path) {
        return Err(Box::new(fault(located, Some(Name::Path), schema)));
    }
    let class = match entry.text(Name::Class).unwrap_or(b"data") {
        b"core" => DataClass::Core,
        b"data" => DataClass::Data,
        b"cache" => DataClass::Cache,
        value => {
            return Err(Box::new(fault(
                located,
                Some(Name::Class),
                Schema::Enum {
                    value: copy_of(value),
                    listed: Box::new([copy_of(b"core"), copy_of(b"data"), copy_of(b"cache")]),
                },
            )));
        }
    };
    let mode = if entry.text(Name::Mode).is_some() {
        match mode(entry) {
            Ok(mode) => mode,
            Err(schema) => return Err(Box::new(fault(located, Some(Name::Mode), schema))),
        }
    } else {
        0o755
    };
    Ok(Entry {
        name: entry.key.clone(),
        task: copy_of(located.task),
        at: entry.at,
        class,
        path: copy_of(path),
        mode,
        owner: text(entry, Name::Owner, b"root"),
        group: text(entry, Name::Group, b"root"),
    })
}
pub(crate) fn name(value: &[u8]) -> bool {
    let Some(first) = value.first() else { return false };
    if !first.is_ascii_lowercase() || value.len() > 32 {
        return false;
    }
    for byte in value {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return false;
        }
    }
    true
}
