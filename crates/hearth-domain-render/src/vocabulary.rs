//! Render's boundary records (render.md, 8.2).
use crate::{
    parts::Spec,
    revision::{Document, Entry, Oid, Position},
};
use alloc::boxed::Box;
pub use hearth_lib::encode::Digest;
use skein_lib::Token;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Diff { repository: Box<[u8]>, from: Box<[u8]>, to: Box<[u8]>, targets: Box<[Box<[u8]>]> },
    Render { repository: Box<[u8]>, revision: Box<[u8]>, targets: Box<[Box<[u8]>]> },
    Listed { request: Token, commit: Oid, entries: Box<[Entry]> },
    Unresolved { request: Token },
    Read { request: Token, oid: Oid, read: Decoded },
    Kept { request: Token, sha256: Digest },
    Cached { request: Token, oid: Oid, size: u64, sha256: Digest },
    Hunks { request: Token, hunks: crate::diff::Hunks },
    Failed { request: Token, class: Class, at: Option<Box<[u8]>>, failure: Failure },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    Keep { request: Token, path: Box<[u8]>, sha256: Digest, bytes: Box<[u8]> },
    Hunks { request: Token, before: Digest, after: Digest },
    List { request: Token, repository: Box<[u8]>, revision: Box<[u8]> },
    Read { request: Token, repository: Box<[u8]>, commit: Oid, blobs: Box<[Blob]> },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    Spec(Spec),
    Error(crate::errors::Error),
    Change(crate::diff::Change),
    Rendered { outcome: Outcome, diff: Option<crate::diff::Tally>, left_out: u32 },
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
    Declaration,
    Template,
    Cached,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Decoded {
    Template(crate::evaluate::template::Template),
    Document(Document),
    Declaration(crate::revision::declaration::Declaration),
    Refused { at: Position, fault: crate::errors::Decode, also: Option<Position> },
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
    Negative,
    Done,
}
impl Outcome {
    #[must_use]
    pub fn worst(self, other: Outcome) -> Outcome {
        core::cmp::min(self, other)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Failure {
    Tool(crate::errors::Tool),
    TooLarge { limit: crate::errors::Limit, max: u64 },
    Store,
}
