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
