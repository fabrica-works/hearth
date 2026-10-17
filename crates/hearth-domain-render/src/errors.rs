//! Decode errors retain their revision places (render.md, 7.1).
use crate::revision::{self, Position};
pub use crate::vocabulary::Class;
use alloc::boxed::Box;
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
pub struct Place {
    pub path: Box<[u8]>,
    pub position: Option<Position>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Decode(Decode),
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

pub(crate) fn refused(path: &[u8], position: Option<Position>, fault: Decode, also: Option<Position>) -> Error {
    let task = match revision::class(path) {
        revision::Class::Declaration { task } | revision::Class::Tree { task, .. } => {
            Some(skein_lib::bytes::copy_of(task))
        }
        revision::Class::Fleet
        | revision::Class::Environment { .. }
        | revision::Class::Host { .. }
        | revision::Class::Other => None,
    };
    Error {
        kind: Kind::Decode(fault),
        class: Class::Configuration,
        at: Some(Place { path: skein_lib::bytes::copy_of(path), position }),
        also: second(path, also),
        target: None,
        task,
    }
}
fn second(path: &[u8], position: Option<Position>) -> Option<Place> {
    Some(Place { path: skein_lib::bytes::copy_of(path), position: Some(position?) })
}
pub(crate) fn push(errors: &mut skein_lib::List<Error>, error: Error) {
    if errors.room() == 0 {
        let mut larger = skein_lib::List::with_capacity(errors.capacity().saturating_mul(2));
        for error in &*errors {
            larger.push(error.clone()).expect("doubled error capacity");
        }
        *errors = larger;
    }
    errors.push(error).expect("error capacity reserved");
}
