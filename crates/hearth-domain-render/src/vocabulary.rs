//! Render's boundary records (render.md, 8.2).
use crate::{
    parts::Spec,
    revision::{Document, Entry, Oid, Position},
};
use alloc::boxed::Box;
use skein_lib::Token;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Render { repository: Box<[u8]>, revision: Box<[u8]>, targets: Box<[Box<[u8]>]> },
    Listed { request: Token, commit: Oid, entries: Box<[Entry]> },
    Unresolved { request: Token },
    Read { request: Token, oid: Oid, read: Decoded },
    Failed { request: Token, class: Class },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    List { request: Token, repository: Box<[u8]>, revision: Box<[u8]> },
    Read { request: Token, repository: Box<[u8]>, commit: Oid, blobs: Box<[Blob]> },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    Spec(Spec),
    Rendered { outcome: Outcome },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Output {
    Request(Request),
    Answer(Answer),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Blob {
    pub oid: Oid,
    pub size: u64,
    pub kind: As,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum As {
    Document,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Decoded {
    Document(Document),
    Refused { at: Position },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class {
    Configuration,
    Operational,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Outcome {
    Usage,
    Failed,
    TryAgain,
    Done,
}
impl Outcome {
    #[must_use]
    pub fn worst(self, other: Outcome) -> Outcome {
        core::cmp::min(self, other)
    }
}
