//! Errors in the answer (render.md, 7.1 to 7.3).
use crate::form::{json::field, text::Lines};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Error {
    pub kind: Kind,
    pub class: Class,
    pub at: Option<Place>,
    pub also: Option<Place>,
    pub target: Option<Box<[u8]>>,
    pub task: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Guard(Guard),
    Decode(Decode),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Guard {
    pub name: Box<[u8]>,
    pub message: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class {
    Configuration,
    Operational,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Place {
    pub path: Box<[u8]>,
    pub position: Option<Position>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}
impl Error {
    pub(crate) fn text(&self, lines: &mut Lines) {
        if let Some(at) = &self.at {
            at.text(lines);
            lines.put(b": ");
        }
        if let Some(target) = &self.target {
            lines.put(target);
            lines.put(b": ");
        }
        if let Some(task) = &self.task {
            lines.put(task);
            lines.put(b": ");
        }
        match &self.kind {
            Kind::Decode(decode) => decode.text(lines),
            Kind::Guard(guard) => {
                lines.put(b"requires ");
                lines.put(&guard.name);
                lines.put(b": ");
                lines.put(&guard.message);
            }
        }
        if let Some(also) = &self.also {
            lines.put(b" (");
            also.text(lines);
            lines.put(b")");
        }
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"record", b"error");
        match &self.kind {
            Kind::Decode(decode) => {
                field(json, b"kind", b"decode");
                json.key(b"decode");
                decode.json(json);
            }
            Kind::Guard(guard) => {
                field(json, b"kind", b"guard");
                json.key(b"guard");
                json.object_start();
                field(json, b"message", &guard.message);
                field(json, b"name", &guard.name);
                json.object_end();
            }
        }
        field(
            json,
            b"class",
            match self.class {
                Class::Configuration => b"configuration",
                Class::Operational => b"operational",
            },
        );
        if let Some(at) = &self.at {
            json.key(b"at");
            at.json(json);
        }
        if let Some(also) = &self.also {
            json.key(b"also");
            also.json(json);
        }
        if let Some(target) = &self.target {
            field(json, b"target", target);
        }
        if let Some(task) = &self.task {
            field(json, b"task", task);
        }
    }
}
impl Place {
    fn text(&self, lines: &mut Lines) {
        lines.put(&self.path);
        if let Some(position) = self.position {
            lines.put(b":");
            lines.decimal(u64::from(position.line));
            lines.put(b":");
            lines.decimal(u64::from(position.column));
        }
    }
    fn json(&self, json: &mut Encoder) {
        json.object_start();
        if let Some(position) = self.position {
            json.key(b"column");
            json.unsigned(u64::from(position.column));
            json.key(b"line");
            json.unsigned(u64::from(position.line));
        }
        field(json, b"path", &self.path);
        json.object_end();
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Decode {
    ListNull,
    Unexpected,
    Trailing,
    TooDeep,
    StringTooLong,
    NumberTooLong,
    Number,
    Escape,
    Surrogate,
    Utf8,
    Control,
    Truncated,
    NotObject,
    NotInteger,
    Range,
    Nul,
    Keys,
    Note,
    Name,
    Placement,
    Members,
    Member,
    Duplicate { key: Box<[u8]> },
    Settings { task: Box<[u8]> },
    Unknown { field: Box<[u8]> },
    Wrong { field: Field },
    Missing { field: Field },
    Misplaced { field: Field },
    TooLarge { limit: Limit, max: u64 },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Field {
    Kind,
    Firmware,
    Hardware,
    On,
    Mutable,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Limit {
    Document,
    Decoded,
    Static,
}

impl Decode {
    fn text(&self, lines: &mut Lines) {
        match self {
            Self::Unexpected => lines.put(b"not JSON: unexpected byte"),
            Self::Trailing => lines.put(b"not JSON: trailing bytes"),
            Self::TooDeep => lines.put(b"JSON nested too deeply"),
            Self::StringTooLong => lines.put(b"JSON string too long"),
            Self::NumberTooLong => lines.put(b"JSON number too long"),
            Self::Number => lines.put(b"not JSON: invalid number"),
            Self::Escape => lines.put(b"not JSON: invalid escape"),
            Self::Surrogate => lines.put(b"not JSON: invalid surrogate"),
            Self::Utf8 => lines.put(b"not JSON: invalid UTF-8"),
            Self::Control => lines.put(b"not JSON: control byte"),
            Self::Truncated => lines.put(b"not JSON: truncated document"),
            Self::NotObject => lines.put(b"a document must be an object"),
            Self::NotInteger => lines.put(b"a number must be an integer"),
            Self::Range => lines.put(b"an integer outside i64"),
            Self::Nul => lines.put(b"U+0000 is not allowed"),
            Self::Keys => lines.put(b"too many object keys"),
            Self::Note => lines.put(b"_note must be a string"),
            Self::ListNull => lines.put(b"null inside a list"),
            Self::Name => lines.put(b"invalid task name"),
            Self::Placement => lines.put(b"placement must be an object"),
            Self::Members => lines.put(b"members must be an object or null"),
            Self::Member => lines.put(b"invalid member"),
            Self::Duplicate { key } => {
                lines.put(b"duplicate key ");
                lines.quote(key);
            }
            Self::Settings { task } => {
                lines.put(b"settings for ");
                lines.quote(task);
                lines.put(b" must be an object or null");
            }
            Self::Unknown { field } => {
                lines.put(b"unknown field ");
                lines.quote(field);
            }
            Self::Wrong { field } => {
                lines.put(b"wrong value for ");
                lines.put(field.name());
            }
            Self::Missing { field } => {
                lines.put(b"missing ");
                lines.put(field.name());
            }
            Self::Misplaced { field } => {
                lines.put(b"misplaced ");
                lines.put(field.name());
            }
            Self::TooLarge { limit, max } => {
                lines.put(match limit {
                    Limit::Document => b"a document past ",
                    Limit::Decoded => b"decoded documents past ",
                    Limit::Static => b"a static file past ",
                });
                lines.decimal(*max);
                lines.put(b" bytes");
            }
        }
    }
    fn json(&self, json: &mut Encoder) {
        match self {
            Self::Unexpected => json.string(b"unexpected"),
            Self::Trailing => json.string(b"trailing"),
            Self::TooDeep => json.string(b"too_deep"),
            Self::StringTooLong => json.string(b"string_too_long"),
            Self::NumberTooLong => json.string(b"number_too_long"),
            Self::Number => json.string(b"number"),
            Self::Escape => json.string(b"escape"),
            Self::Surrogate => json.string(b"surrogate"),
            Self::Utf8 => json.string(b"utf8"),
            Self::Control => json.string(b"control"),
            Self::Truncated => json.string(b"truncated"),
            Self::NotObject => json.string(b"not_object"),
            Self::NotInteger => json.string(b"not_integer"),
            Self::Range => json.string(b"range"),
            Self::Nul => json.string(b"nul"),
            Self::Keys => json.string(b"keys"),
            Self::Note => json.string(b"note"),
            Self::ListNull => json.string(b"list_null"),
            Self::Name => json.string(b"name"),
            Self::Placement => json.string(b"placement"),
            Self::Members => json.string(b"members"),
            Self::Member => json.string(b"member"),
            Self::Duplicate { key } => {
                json.object_start();
                field(json, b"key", key);
                field(json, b"kind", b"duplicate");
                json.object_end();
            }
            Self::Settings { task } => {
                json.object_start();
                field(json, b"task", task);
                field(json, b"kind", b"settings");
                json.object_end();
            }
            Self::Unknown { field: name } => {
                json.object_start();
                field(json, b"field", name);
                field(json, b"kind", b"unknown");
                json.object_end();
            }
            Self::Wrong { field: name } => {
                json.object_start();
                field(json, b"field", name.name());
                field(json, b"kind", b"wrong");
                json.object_end();
            }
            Self::Missing { field: name } => {
                json.object_start();
                field(json, b"field", name.name());
                field(json, b"kind", b"missing");
                json.object_end();
            }
            Self::Misplaced { field: name } => {
                json.object_start();
                field(json, b"field", name.name());
                field(json, b"kind", b"misplaced");
                json.object_end();
            }
            Self::TooLarge { limit, max } => {
                json.object_start();
                field(json, b"kind", b"too_large");
                field(
                    json,
                    b"limit",
                    match limit {
                        Limit::Document => b"document",
                        Limit::Decoded => b"decoded",
                        Limit::Static => b"static",
                    },
                );
                json.key(b"max");
                json.unsigned(*max);
                json.object_end();
            }
        }
    }
}
impl Field {
    fn name(self) -> &'static [u8] {
        match self {
            Self::Kind => b"kind",
            Self::Firmware => b"firmware",
            Self::Hardware => b"hardware",
            Self::On => b"on",
            Self::Mutable => b"mutable",
        }
    }
}
