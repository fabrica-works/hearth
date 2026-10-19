//! Typed store requests and terminals (domain/store.md, 6.6, 8.2).
use alloc::boxed::Box;
use hearth_lib::encode::{Digest, Item};
use skein_lib::{Duration, Token, Wall};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Record {
        resolution: Resolution,
    },
    Room,
    Unwritable {
        max: u64,
    },
    Find {
        key: Digest,
    },
    Resolution {
        record: Resolution,
    },
    /// Whether a layer of this key is published (domain/build.md, 4.1).
    Want {
        key: Digest,
    },
    /// A writable copy of the parent layer, to become the layer of this key (4.2, step 1).
    Stage {
        key: Digest,
        parent: Digest,
    },
    /// Walk, freeze, record and publish the staged layer (4.2, steps 3 to 6).
    Seal {
        key: Digest,
    },
    /// Drop the staged layer of this key (4.2, last paragraph).
    Discard {
        key: Digest,
    },
    Layer {
        record: Layer,
    },
    Unreadable,
    Arrived {
        arrival: Token,
        sha256: Digest,
        size: u64,
        origin: Origin,
        took: Duration,
    },
    Open,
    Locked,
    Busy,
    Loaded {
        record: Option<StoreRecord>,
    },
    Page {
        entries: Box<[Entry]>,
        last: bool,
    },
    Done,
    Failed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    Write { resolution: Resolution },
    Load { key: Digest },
    LoadLayer { key: Digest },
    Empty,
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
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    Recorded,
    HasRoom,
    Found { resolution: Resolution },
    Absent,
    Have { key: Digest },
    Free { key: Digest },
    Staged { key: Digest },
    Sealed { key: Digest },
    Discarded { key: Digest },
    Declined { why: Decline },
    Waiting,
    Strays { count: u64, first: Stray },
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
    Resolutions,
    Layers,
    Cache,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Limit {
    Resolutions,
    Layers,
    CacheFiles,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    Write,
    Load,
    Empty,
    Keep,
    List,
    Remove,
    Stage,
    Seal,
    Discard,
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

/// A cache file's provenance and cost (domain/store.md, 6.5).
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

/// The four canonical values of a resolution key (domain/render.md, 5.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Inputs {
    pub apt: Box<[Item]>,
    pub request: Box<[Item]>,
    pub sources: Box<[Item]>,
    pub variant: Box<[Item]>,
}
/// A closure package with its source and archive metadata (domain/store.md, 6.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Closed {
    pub source: u32,
    pub filename: Box<[u8]>,
    pub size: u64,
    pub name: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub version: Box<[u8]>,
    pub sha256: Digest,
}

/// A complete resolution record, loaded only when requested (domain/store.md, 6.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Resolution {
    pub key: Digest,
    pub inputs: Inputs,
    pub closure: Box<[Closed]>,
    pub digest: Digest,
    pub sources: Box<[Line]>,
    pub made: Made,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line {
    pub uri: Box<[u8]>,
    pub suite: Box<[u8]>,
    pub components: Box<[Box<[u8]>]>,
    pub indices: Box<[Index]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Index {
    pub path: Box<[u8]>,
    pub size: u64,
    pub sha256: Digest,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Made {
    pub at: Wall,
    pub apt: Box<[u8]>,
    pub mmdebstrap: Box<[u8]>,
    pub costs: Costs,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Costs {
    pub fetched: u64,
    pub took: Duration,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Decline {
    Bound { max: u64 },
    Full { limit: Limit, most: u32 },
    Unusable { step: Step },
}

/// A published layer's record, as the startup scan reads it (domain/store.md, 6.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Layer {
    pub key: Digest,
    pub parent: Option<Digest>,
    pub inputs: LayerInputs,
    pub tree: Digest,
    pub uuid: [u8; 16],
    pub at: Wall,
}
/// A layer's key inputs, which name its kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LayerInputs {
    Empty,
    Packages { part: Digest, resolution: Digest },
    Software { part: Digest },
    Config { part: Digest },
}
