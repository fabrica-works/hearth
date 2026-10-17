//! Declaration maps (format.md, 5.1).
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

use super::{Position, Value};
use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Text {
    pub at: Position,
    pub text: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Number {
    pub at: Position,
    pub value: i64,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Declaration {
    pub at: Position,
    pub types: Box<[Type]>,
    pub settings: Box<[Spec]>,
    pub reads: Box<[Read]>,
    pub requires: Box<[Guard]>,
    pub scripts: Box<[Script]>,
    pub exports: Box<[Export]>,
    pub entries: Box<[Entries]>,
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
impl Fragment {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Customize => b"customize.sh",
            Self::Software => b"software.sh",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Read {
    pub task: Text,
    pub at: Position,
    pub settings: Box<[Text]>,
    pub exports: bool,
    pub optional: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Type {
    pub at: Position,
    pub form: Form,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Form {
    Bool,
    Int,
    String,
    Path,
    Mode,
    Duration,
    Secret,
    Target,
    File,
    Enum(Box<[Text]>),
    List(u32),
    Map(u32),
    Object(Box<[Spec]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Spec {
    pub name: Text,
    pub at: Position,
    pub ty: u32,
    pub default: Option<Value>,
    pub optional: bool,
    pub min: Option<Number>,
    pub max: Option<Number>,
    pub max_length: Option<Number>,
}

/// A declaration expression at its JSON string (format.md, 5.9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Condition {
    pub at: Position,
    pub expression: crate::evaluate::Expression,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Guard {
    pub name: Text,
    pub at: Position,
    pub when: Condition,
    pub message: Text,
}

/// A compiled entry key at its opening quote (format.md, 5.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Interpolated {
    Plain { at: Position, text: Box<[u8]> },
    Parts { at: Position, parts: Box<[crate::evaluate::template::Instruction]> },
}
impl Interpolated {
    #[must_use]
    pub const fn at(&self) -> Position {
        match self {
            Self::Plain { at, text: _ } | Self::Parts { at, parts: _ } => *at,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entries {
    pub map: Map,
    pub at: Position,
    pub entries: Box<[Entry]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub key: Interpolated,
    pub at: Position,
    pub when: Option<Condition>,
    pub for_each: Option<Condition>,
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
    Text(Interpolated),
    Plain { at: Position, text: Box<[u8]> },
    Typed(Interpolated),
    Int(Number),
    Bool { at: Position, value: bool },
    Strings { at: Position, each: Box<[Interpolated]> },
    Range { at: Position, fields: Box<[Field]> },
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
impl Name {
    #[must_use]
    pub const fn takes(self) -> crate::errors::Takes {
        match self {
            Self::Action
            | Self::Age
            | Self::Algorithm
            | Self::Argument
            | Self::Class
            | Self::Component
            | Self::Copy
            | Self::From
            | Self::Gecos
            | Self::Group
            | Self::Home
            | Self::Kind
            | Self::Link
            | Self::Mode
            | Self::Owner
            | Self::Param
            | Self::Password
            | Self::Path
            | Self::PublicPath
            | Self::Sha256
            | Self::Shell
            | Self::Template
            | Self::Type
            | Self::Unit
            | Self::Unpack
            | Self::Url
            | Self::Value => crate::errors::Takes::Text,
            Self::Base | Self::Count | Self::Gid | Self::Order | Self::Strip | Self::Uid => crate::errors::Takes::Int,
            Self::Directory => crate::errors::Takes::Bool,
            Self::Groups | Self::Paths => crate::errors::Takes::Strings,
            Self::Subordinate => crate::errors::Takes::Range,
        }
    }
    #[must_use]
    pub const fn key(self) -> &'static [u8] {
        match self {
            Self::Action => b"action",
            Self::Age => b"age",
            Self::Algorithm => b"algorithm",
            Self::Argument => b"argument",
            Self::Base => b"base",
            Self::Class => b"class",
            Self::Component => b"component",
            Self::Copy => b"copy",
            Self::Count => b"count",
            Self::Directory => b"directory",
            Self::From => b"from",
            Self::Gecos => b"gecos",
            Self::Gid => b"gid",
            Self::Group => b"group",
            Self::Groups => b"groups",
            Self::Home => b"home",
            Self::Kind => b"kind",
            Self::Link => b"link",
            Self::Mode => b"mode",
            Self::Order => b"order",
            Self::Owner => b"owner",
            Self::Param => b"param",
            Self::Password => b"password",
            Self::Path => b"path",
            Self::Paths => b"paths",
            Self::PublicPath => b"public_path",
            Self::Sha256 => b"sha256",
            Self::Shell => b"shell",
            Self::Strip => b"strip",
            Self::Subordinate => b"subordinate",
            Self::Template => b"template",
            Self::Type => b"type",
            Self::Uid => b"uid",
            Self::Unit => b"unit",
            Self::Unpack => b"unpack",
            Self::Url => b"url",
            Self::Value => b"value",
        }
    }
}

/// One typed public value (format.md, 8.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Export {
    pub name: Text,
    pub at: Position,
    pub ty: u32,
    pub optional: bool,
    pub value: Interpolated,
}
