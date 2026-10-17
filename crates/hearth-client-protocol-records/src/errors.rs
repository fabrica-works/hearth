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
    Schema(Schema),
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
            Kind::Schema(schema) => schema.text(lines),
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
            Kind::Schema(schema) => {
                field(json, b"kind", b"schema");
                json.key(b"schema");
                schema.json(json);
            }
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
    Declaration(Declared),
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
            Self::Declaration(declared) => declared.text(lines),
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
            Self::Declaration(declared) => {
                json.object_start();
                json.key(b"declared");
                declared.json(json);
                field(json, b"kind", b"declaration");
                json.object_end();
            }
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

/// Why a decoded task.json is no declaration (format.md, 5.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Declared {
    Unknown { of: Object },
    Later,
    Wrong { expected: Expected },
    Missing { key: Required },
    Name,
    Doc,
    Order,
    Optional,
    Bound,
    Above,
    Enum,
    Settings,
    Secret,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Object {
    Declaration,
    Read,
    Guard,
    Script,
    Bless,
    Entry(Map),
    Range,
    Spec,
    Export,
    Type,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Expected {
    Object,
    String,
    Strings,
    Bool,
    Int,
    Field(Takes),
    True,
    Type,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Required {
    If,
    Message,
    Type,
    Doc,
    Value,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Map {
    Packages,
    Preseeds,
    Initramfs,
    Software,
    Files,
    Users,
    Groups,
    Presets,
    Tmpfiles,
    Data,
    Secrets,
    Identity,
    Cmdline,
    Live,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Takes {
    Text,
    Int,
    Bool,
    Strings,
    Range,
}

impl Declared {
    fn text(self, lines: &mut Lines) {
        match self {
            Self::Unknown { of } => {
                lines.put(b"no such key in a ");
                lines.put(of.name());
            }
            Self::Wrong { expected } => {
                lines.put(b"not ");
                lines.put(expected.name());
            }
            Self::Missing { key } => {
                lines.put(b"missing ");
                lines.put(key.name());
            }
            Self::Later => lines.put(b"a key for a later milestone"),
            Self::Name => lines.put(b"invalid declaration name"),
            Self::Doc => lines.put(b"doc must be one line within its length limit"),
            Self::Order => lines.put(b"order must be from 0 to 999"),
            Self::Optional => lines.put(b"optional beside a default"),
            Self::Bound => lines.put(b"a bound on a type it does not bound"),
            Self::Above => lines.put(b"min above max"),
            Self::Enum => lines.put(b"an enum must list distinct values"),
            Self::Settings => lines.put(b"too many settings"),
            Self::Secret => lines.put(b"an export type holds secret"),
        }
    }
    fn json(self, json: &mut Encoder) {
        json.object_start();
        match self {
            Self::Unknown { of } => {
                field(json, b"kind", b"unknown");
                field(json, b"of", of.name());
            }
            Self::Wrong { expected } => {
                field(json, b"kind", b"wrong");
                field(json, b"expected", expected.name());
            }
            Self::Missing { key } => {
                field(json, b"kind", b"missing");
                field(json, b"key", key.name());
            }
            Self::Later => field(json, b"kind", b"later"),
            Self::Name => field(json, b"kind", b"name"),
            Self::Doc => field(json, b"kind", b"doc"),
            Self::Order => field(json, b"kind", b"order"),
            Self::Optional => field(json, b"kind", b"optional"),
            Self::Bound => field(json, b"kind", b"bound"),
            Self::Above => field(json, b"kind", b"above"),
            Self::Enum => field(json, b"kind", b"enum"),
            Self::Settings => field(json, b"kind", b"settings"),
            Self::Secret => field(json, b"kind", b"secret"),
        }
        json.object_end();
    }
}
impl Object {
    fn name(self) -> &'static [u8] {
        match self {
            Self::Declaration => b"declaration",
            Self::Read => b"read",
            Self::Guard => b"guard",
            Self::Script => b"script",
            Self::Bless => b"bless",
            Self::Range => b"range",
            Self::Spec => b"spec",
            Self::Export => b"export",
            Self::Type => b"type",
            Self::Entry(map) => map.name(),
        }
    }
}
impl Map {
    fn name(self) -> &'static [u8] {
        match self {
            Self::Packages => b"packages entry",
            Self::Preseeds => b"preseeds entry",
            Self::Initramfs => b"initramfs entry",
            Self::Software => b"software entry",
            Self::Files => b"files entry",
            Self::Users => b"users entry",
            Self::Groups => b"groups entry",
            Self::Presets => b"presets entry",
            Self::Tmpfiles => b"tmpfiles entry",
            Self::Data => b"data entry",
            Self::Secrets => b"secrets entry",
            Self::Identity => b"identity entry",
            Self::Cmdline => b"cmdline entry",
            Self::Live => b"live entry",
        }
    }
}
impl Expected {
    fn name(self) -> &'static [u8] {
        match self {
            Self::Object => b"an object",
            Self::String => b"a string",
            Self::Strings => b"a list of strings",
            Self::Bool => b"a bool",
            Self::Int => b"an int",
            Self::True => b"true",
            Self::Type => b"a type",
            Self::Field(takes) => takes.name(),
        }
    }
}
impl Takes {
    fn name(self) -> &'static [u8] {
        match self {
            Self::Text => b"a string",
            Self::Int => b"an int",
            Self::Bool => b"a bool",
            Self::Strings => b"a list of strings",
            Self::Range => b"a range",
        }
    }
}
impl Required {
    fn name(self) -> &'static [u8] {
        match self {
            Self::If => b"if",
            Self::Message => b"message",
            Self::Type => b"type",
            Self::Doc => b"doc",
            Self::Value => b"value",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Schema {
    Task { task: Box<[u8]> },
    Unread { task: Box<[u8]>, setting: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    Wrong { expected: TypeName },
    Enum { value: Box<[u8]>, listed: Box<[Box<[u8]>]> },
    Below { min: i64 },
    Above { max: i64 },
    Length { max_length: i64 },
    Null,
    UndeclaredField { field: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    MissingField { field: Box<[u8]> },
}

impl Schema {
    fn text(&self, lines: &mut Lines) {
        match self {
            Self::Wrong { expected } => {
                lines.put(b"not ");
                lines.put(expected.article());
                lines.put(expected.name());
            }
            Self::Enum { value, listed } => {
                lines.quote(value);
                lines.put(b" is not one of ");
                names(lines, listed);
            }
            Self::Below { min } => {
                lines.put(b"below its min ");
                signed(lines, *min);
            }
            Self::Above { max } => {
                lines.put(b"past its max ");
                signed(lines, *max);
            }
            Self::Length { max_length } => {
                lines.put(b"past its max_length ");
                signed(lines, *max_length);
            }
            Self::Null => lines.put(b"null in a default"),
            Self::UndeclaredField { field, declared } => {
                lines.put(b"undeclared field ");
                lines.quote(field);
                lines.put(b"; the object declares ");
                names(lines, declared);
            }
            Self::MissingField { field } => {
                lines.put(b"missing required field ");
                lines.put(field);
            }

            Self::Task { task } => {
                lines.put(b"reads ");
                lines.put(task);
                lines.put(b", which tasks/ lacks");
            }
            Self::Unread { task, setting, declared } => {
                lines.put(b"reads ");
                lines.put(task);
                lines.put(b".");
                lines.put(setting);
                lines.put(b", which ");
                lines.put(task);
                lines.put(b" does not declare; ");
                lines.put(task);
                if declared.is_empty() {
                    lines.put(b" declares no settings");
                } else {
                    lines.put(b" declares ");
                    for (index, name) in declared.iter().enumerate() {
                        if index != 0 {
                            lines.put(b", ");
                        }
                        lines.put(name);
                    }
                }
            }
        }
    }
    fn json(&self, json: &mut Encoder) {
        json.object_start();
        match self {
            Self::Wrong { expected } => {
                field(json, b"expected", expected.name());
                field(json, b"kind", b"wrong");
            }
            Self::Enum { value, listed } => {
                field(json, b"kind", b"enum");
                json.key(b"listed");
                json_names(json, listed);
                field(json, b"value", value);
            }
            Self::Below { min } => {
                field(json, b"kind", b"below");
                json.key(b"min");
                json.signed(*min);
            }
            Self::Above { max } => {
                field(json, b"kind", b"above");
                json.key(b"max");
                json.signed(*max);
            }
            Self::Length { max_length } => {
                field(json, b"kind", b"length");
                json.key(b"max_length");
                json.signed(*max_length);
            }
            Self::Null => field(json, b"kind", b"null"),
            Self::UndeclaredField { field: name, declared } => {
                json.key(b"declared");
                json_names(json, declared);
                field(json, b"field", name);
                field(json, b"kind", b"undeclared_field");
            }
            Self::MissingField { field: name } => {
                field(json, b"field", name);
                field(json, b"kind", b"missing_field");
            }

            Self::Task { task } => {
                field(json, b"kind", b"task");
                field(json, b"task", task);
            }
            Self::Unread { task, setting, declared } => {
                json.key(b"declared");
                json.array_start();
                for name in declared {
                    json.string(name);
                }
                json.array_end();
                field(json, b"kind", b"unread");
                field(json, b"setting", setting);
                field(json, b"task", task);
            }
        }
        json.object_end();
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TypeName {
    Bool,
    Int,
    String,
    Enum,
    List,
    Map,
    Object,
    Path,
    Mode,
    Duration,
    Secret,
    Target,
    File,
}

fn signed(lines: &mut Lines, value: i64) {
    if value < 0 {
        lines.put(b"-");
    }
    lines.decimal(value.unsigned_abs());
}
fn names(lines: &mut Lines, names: &[Box<[u8]>]) {
    for (index, name) in names.iter().enumerate() {
        if index != 0 {
            lines.put(b", ");
        }
        lines.put(name);
    }
}
fn json_names(json: &mut Encoder, names: &[Box<[u8]>]) {
    json.array_start();
    for name in names {
        json.string(name);
    }
    json.array_end();
}
impl TypeName {
    fn article(&self) -> &'static [u8] {
        match self {
            Self::Int | Self::Enum | Self::Object => b"an ",
            Self::Bool
            | Self::String
            | Self::List
            | Self::Map
            | Self::Path
            | Self::Mode
            | Self::Duration
            | Self::Secret
            | Self::Target
            | Self::File => b"a ",
        }
    }
    fn name(&self) -> &'static [u8] {
        match self {
            Self::Bool => b"bool",
            Self::Int => b"int",
            Self::String => b"string",
            Self::Enum => b"enum",
            Self::List => b"list",
            Self::Map => b"map",
            Self::Object => b"object",
            Self::Path => b"path",
            Self::Mode => b"mode",
            Self::Duration => b"duration",
            Self::Secret => b"secret",
            Self::Target => b"target",
            Self::File => b"file",
        }
    }
}
