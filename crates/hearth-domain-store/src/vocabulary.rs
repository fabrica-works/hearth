//! Typed store requests and terminals (store.md, 6.6, 8.2).
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{Duration, Token, Wall};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Arrived { arrival: Token, sha256: Digest, size: u64, origin: Origin, took: Duration },
    Open,
    Locked,
    Busy,
    Loaded { record: Option<StoreRecord> },
    Page { entries: Box<[Entry]>, last: bool },
    Done,
    Failed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    Keep { arrival: Token, record: CacheFile },
    Discard { arrival: Token },
    Lock,
    LoadRecord,
    WriteRecord { record: StoreRecord },
    List { directory: Directory, max: u32 },
    Next { max: u32 },
    Close,
    Remove { directory: Directory, name: Removal },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    Cached { arrival: Token },
    Refused { arrival: Token },
    Ready,
    Full { limit: Limit, most: u32 },
    Unusable { step: Step },
    Unopened { why: Why },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Output {
    Request(Request),
    Answer(Answer),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    Lock,
    Record,
    Owner { recorded: u32, builder: u32 },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StoreRecord {
    pub layout: u64,
    pub made: Wall,
    pub range: Option<Range>,
    pub uid: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Range {
    pub base: u32,
    pub count: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Directory {
    Top,
    Cache,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Limit {
    Resolutions,
    CacheFiles,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    Keep,
    List,
    Remove,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub name: Box<[u8]>,
    pub shape: Shape,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Shape {
    File(Digest),
    Record(Digest),
    Temporary,
    Fixed,
    Stray,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Removal {
    File(Digest),
    Record(Digest),
    Named(Box<[u8]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Stray {
    pub directory: Directory,
    pub name: Box<[u8]>,
}

/// A cache file's provenance and cost (store.md, 6.5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CacheFile {
    pub at: Wall,
    pub origin: Origin,
    pub sha256: Digest,
    pub size: u64,
    pub took: Duration,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Origin {
    Fetch { uri: Box<[u8]> },
    Index { resolution: Digest },
    Blob { oid: Box<[u8]> },
    Rendered { path: Box<[u8]> },
}
