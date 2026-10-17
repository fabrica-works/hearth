//! A document's first fault, where the faulty thing starts (render.md, 7.1).
use crate::Position;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Error {
    pub at: Position,
    pub fault: Fault,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Fault {
    Json(skein_json::tokenizer::Error),
    Stream(skein_lib::stream::Fault),
    NotObject,
    Duplicate { key: Box<[u8]>, first: Position },
    NotInteger,
    Range,
    Nul,
    Keys,
    Note,
    Shape(Shape),
    Declaration(Declared),
}
/// Why a decoded task.json is no declaration (format.md, 5.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Declared {
    Unknown { of: Object },
    Later,
    Undecoded,
    Wrong { expected: Expected },
    Missing { key: Required },
    Name,
    Doc,
    Order,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Object {
    Declaration,
    Read,
    Guard,
    Script,
    Bless,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Expected {
    Object,
    String,
    Strings,
    Bool,
    Int,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Required {
    If,
    Message,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Shape {
    Name,
    Settings { task: Box<[u8]> },
    Placement,
    Members,
    Member,
    Unknown { field: Box<[u8]> },
    Wrong { field: Field },
    Missing { field: Field },
    Misplaced { field: Field },
}

/// Placement's M1 fields (format.md, 3.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Field {
    Kind,
    Firmware,
    Hardware,
    On,
    Mutable,
}
