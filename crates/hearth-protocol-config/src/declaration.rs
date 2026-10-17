//! A task's fixed declaration shape (format.md, 5.1, 5.8, 5.9, 5.13, 6.4).
use crate::{
    Declared, Error, Expected, Fault, Object, Position, Required,
    json::{Entry, Kind, Value},
    limits::Limits,
    names,
};
use alloc::boxed::Box;
use skein_lib::List;

/// An unescaped string at its opening quote, never read as an expression here.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Text {
    pub at: Position,
    pub text: Box<[u8]>,
}
/// A task.json at its opening brace; an absent map is an empty list.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Declaration {
    pub at: Position,
    pub reads: Box<[Read]>,
    pub requires: Box<[Guard]>,
    pub scripts: Box<[Script]>,
    pub bless: Option<Bless>,
    pub entries: Box<[crate::entries::Entries]>,
    pub types: Box<[crate::schema::Type]>,
    pub settings: Box<[crate::schema::Spec]>,
    pub exports: Box<[crate::schema::Export]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Read {
    pub task: Text,
    pub at: Position,
    pub settings: Box<[Text]>,
    pub exports: bool,
    pub optional: bool,
}
/// A guard's name is an identifier (2.4), at its key; its condition and message are strings.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Guard {
    pub name: Text,
    pub at: Position,
    pub when: Text,
    pub message: Text,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Script {
    pub fragment: Fragment,
    pub at: Position,
    pub order: u16,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fragment {
    Customize,
    Software,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Bless {
    pub at: Position,
    pub ignore_failed: Box<[Text]>,
}
#[derive(Clone, Copy)]
enum Key {
    Doc,
    Settings,
    Reads,
    Requires,
    Scripts,
    Bless,
    Later,
    Exports,
    Map(crate::entries::Map),
}
const KEYS: &[(&[u8], Key)] = &[
    (b"doc", Key::Doc),
    (b"reads", Key::Reads),
    (b"requires", Key::Requires),
    (b"scripts", Key::Scripts),
    (b"bless", Key::Bless),
    (b"checks", Key::Later),
    (b"settings", Key::Settings),
    (b"exports", Key::Exports),
    (b"packages", Key::Map(crate::entries::Map::Packages)),
    (b"preseeds", Key::Map(crate::entries::Map::Preseeds)),
    (b"initramfs", Key::Map(crate::entries::Map::Initramfs)),
    (b"cmdline", Key::Map(crate::entries::Map::Cmdline)),
    (b"files", Key::Map(crate::entries::Map::Files)),
    (b"users", Key::Map(crate::entries::Map::Users)),
    (b"groups", Key::Map(crate::entries::Map::Groups)),
    (b"presets", Key::Map(crate::entries::Map::Presets)),
    (b"tmpfiles", Key::Map(crate::entries::Map::Tmpfiles)),
    (b"data", Key::Map(crate::entries::Map::Data)),
    (b"secrets", Key::Map(crate::entries::Map::Secrets)),
    (b"identity", Key::Map(crate::entries::Map::Identity)),
    (b"software", Key::Map(crate::entries::Map::Software)),
    (b"live", Key::Map(crate::entries::Map::Live)),
];
pub fn declaration(value: Value, limits: &Limits) -> Result<Declaration, Error> {
    let Kind::Map(entries) = value.kind else {
        return Err(Error { at: value.at, fault: Fault::NotObject });
    };
    let mut declaration = Declaration {
        at: value.at,
        reads: Box::new([]),
        requires: Box::new([]),
        scripts: Box::new([]),
        bless: None,
        entries: Box::new([]),
        types: Box::new([]),
        settings: Box::new([]),
        exports: Box::new([]),
    };
    let mut types = List::with_capacity(limits.document.saturating_div(2).saturating_add(1));
    let mut maps = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
    for entry in entries {
        let mut key = None;
        for &(name, candidate) in KEYS {
            if &*entry.key == name {
                key = Some(candidate);
                break;
            }
        }
        match key.ok_or(unknown(entry.at, Object::Declaration))? {
            Key::Doc => {
                let doc = text(entry.value)?;
                if doc.text.contains(&b'\n') {
                    return Err(fault(doc.at, Declared::Doc));
                }
            }
            Key::Settings => {
                let (settings, _) = crate::schema::read(entry.value, limits, &mut types, false)?;
                declaration.settings = settings;
            }
            Key::Exports => {
                let (_, exports) = crate::schema::read(entry.value, limits, &mut types, true)?;
                declaration.exports = exports;
            }
            Key::Reads => declaration.reads = reads(entry.value)?,
            Key::Requires => declaration.requires = guards(entry.value)?,
            Key::Scripts => declaration.scripts = scripts(entry.value)?,
            Key::Bless => declaration.bless = Some(bless(entry.value)?),
            Key::Later => return Err(fault(entry.at, Declared::Later)),
            Key::Map(map) => {
                let entries = crate::entries::read(map, entry.at, entry.value)?;
                maps.push(entries).expect("one map per source entry");
            }
        }
    }
    declaration.entries = maps.into_boxed();
    declaration.types = types.into_boxed();
    Ok(declaration)
}
pub(crate) fn fault(at: Position, declared: Declared) -> Error {
    Error { at, fault: Fault::Declaration(declared) }
}
pub(crate) fn wrong(at: Position, expected: Expected) -> Error {
    fault(at, Declared::Wrong { expected })
}
pub(crate) fn unknown(at: Position, of: Object) -> Error {
    fault(at, Declared::Unknown { of })
}
pub(crate) fn object(value: Value) -> Result<Box<[Entry]>, Error> {
    let Kind::Map(entries) = value.kind else {
        return Err(wrong(value.at, Expected::Object));
    };
    Ok(entries)
}
pub(crate) fn text(value: Value) -> Result<Text, Error> {
    let Kind::String(text) = value.kind else {
        return Err(wrong(value.at, Expected::String));
    };
    Ok(Text { at: value.at, text })
}
fn identifier(text: Text) -> Result<Text, Error> {
    if !names::identifier(&text.text) {
        return Err(fault(text.at, Declared::Name));
    }
    Ok(text)
}
fn boolean(value: Value) -> Result<bool, Error> {
    let Kind::Bool(value) = value.kind else {
        return Err(wrong(value.at, Expected::Bool));
    };
    Ok(value)
}
pub(crate) fn texts(value: Value, identifiers: bool) -> Result<Box<[Text]>, Error> {
    let Kind::List(values) = value.kind else {
        return Err(wrong(value.at, Expected::Strings));
    };
    let mut each = List::with_capacity(u32::try_from(values.len()).expect("document bounded by u32"));
    for value in values {
        let text = text(value)?;
        let text = if identifiers { identifier(text)? } else { text };
        each.push(text).expect("one text per source element");
    }
    Ok(each.into_boxed())
}
fn reads(value: Value) -> Result<Box<[Read]>, Error> {
    let entries = object(value)?;
    let mut reads = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
    for entry in entries {
        let task = identifier(Text { at: entry.at, text: entry.key })?;
        let mut read = Read { task, at: entry.value.at, settings: Box::new([]), exports: false, optional: false };
        for field in object(entry.value)? {
            match &*field.key {
                b"settings" => read.settings = texts(field.value, true)?,
                b"exports" => read.exports = boolean(field.value)?,
                b"optional" => read.optional = boolean(field.value)?,
                _ => return Err(unknown(field.at, Object::Read)),
            }
        }
        reads.push(read).expect("one read per source entry");
    }
    Ok(reads.into_boxed())
}
fn guards(value: Value) -> Result<Box<[Guard]>, Error> {
    let entries = object(value)?;
    let mut guards = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
    for entry in entries {
        let name = identifier(Text { at: entry.at, text: entry.key })?;
        let at = entry.value.at;
        let mut when = None;
        let mut message = None;
        for field in object(entry.value)? {
            match &*field.key {
                b"if" => when = Some(text(field.value)?),
                b"message" => message = Some(text(field.value)?),
                _ => return Err(unknown(field.at, Object::Guard)),
            }
        }
        let when = when.ok_or(fault(at, Declared::Missing { key: Required::If }))?;
        let message = message.ok_or(fault(at, Declared::Missing { key: Required::Message }))?;
        guards.push(Guard { name, at, when, message }).expect("one guard per source entry");
    }
    Ok(guards.into_boxed())
}
fn scripts(value: Value) -> Result<Box<[Script]>, Error> {
    let entries = object(value)?;
    let mut scripts = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
    for entry in entries {
        let fragment = match &*entry.key {
            b"customize.sh" => Fragment::Customize,
            b"software.sh" => Fragment::Software,
            _ => return Err(fault(entry.at, Declared::Name)),
        };
        let mut order = 500;
        for field in object(entry.value)? {
            if &*field.key != b"order" {
                return Err(unknown(field.at, Object::Script));
            }
            let Kind::Int(number) = field.value.kind else {
                return Err(fault(field.value.at, Declared::Order));
            };
            if !(0..=999).contains(&number) {
                return Err(fault(field.value.at, Declared::Order));
            }
            order = u16::try_from(number).expect("order is from 0 to 999");
        }
        scripts.push(Script { fragment, at: entry.at, order }).expect("one script per source entry");
    }
    Ok(scripts.into_boxed())
}
fn bless(value: Value) -> Result<Bless, Error> {
    let mut bless = Bless { at: value.at, ignore_failed: Box::new([]) };
    for field in object(value)? {
        if &*field.key != b"ignore_failed" {
            return Err(unknown(field.at, Object::Bless));
        }
        bless.ignore_failed = texts(field.value, false)?;
    }
    Ok(bless)
}

/// An integer at its first byte in task.json.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Number {
    pub at: Position,
    pub value: i64,
}

/// Peak heap for declaration decoding, including the JSON value still held.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    crate::json::worst_case(limits)?.checked_add(crate::schema::worst_case(limits)?)
}
