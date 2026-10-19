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
    /// Walk, freeze, record and publish the staged layer (4.2, steps 3 to 6): `parent` and
    /// `inputs` are what its key and record are made of (6.2).
    Seal {
        key: Digest,
        parent: Digest,
        inputs: LayerInputs,
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
    /// The builder made the empty layer's subvolume, whose uuid it read.
    Created {
        uuid: [u8; 16],
    },
    /// The tree helper's answer to its one request in flight (5.3).
    Helped {
        answer: Helped,
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
    Write {
        resolution: Resolution,
    },
    Load {
        key: Digest,
    },
    LoadLayer {
        key: Digest,
    },
    Empty,
    Keep {
        arrival: Token,
        record: CacheFile,
    },
    Discard {
        arrival: Token,
    },
    Lock,
    LoadRecord,
    WriteRecord {
        record: StoreRecord,
    },
    List {
        directory: Directory,
        max: u32,
    },
    Next {
        max: u32,
    },
    Close,
    Remove {
        directory: Directory,
        name: Removal,
    },
    /// The empty layer's subvolume, as `layers/<key>.staging` (4.1); `Created` or `Failed`.
    Create {
        key: Digest,
    },
    /// One of a layer's renames within `layers/` (4.2, steps 5 and 6); `Done` or `Failed`.
    Rename {
        key: Digest,
        rename: Renaming,
    },
    /// `layers/<key>.record`, written by rename (4.2, step 5); `Done` or `Failed`.
    WriteLayer {
        record: Box<Record>,
    },
    /// A request for the tree helper, one at a time (5.3); `Helped` or `Failed`.
    Helper(Helper),
}
/// A rename within `layers/` (4.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Renaming {
    /// `<key>.tree.tmp` to `<key>.tree`.
    Tree,
    /// `<key>.staging` to `<key>`.
    Publish,
}
/// What the tree helper is asked, a closed set (5.3); `layer` and `staging` are keys.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Helper {
    /// `parent`, writable, into `layers/<staging>.staging`.
    Snapshot {
        parent: Digest,
        staging: Digest,
    },
    /// `layer` against its parent's tree file, writing `<layer>.tree.tmp`.
    Walk {
        layer: Digest,
        parent: Option<Digest>,
    },
    /// `layer`, which has no parent, every file read.
    Hash {
        layer: Digest,
    },
    /// The read-only flag, then `syncfs`.
    Freeze {
        layer: Digest,
    },
    Thaw {
        layer: Digest,
    },
    Apply {
        layer: Digest,
    },
    Empty {
        layer: Digest,
    },
    Delete {
        layer: Digest,
    },
}
/// The tree helper's one answer to a request (5.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Helped {
    Snapshotted { uuid: [u8; 16], parent_uuid: [u8; 16] },
    Walked(Walk),
    Hashed(Walk),
    Frozen,
    Thawed,
    Applied,
    Emptied { removed: u64 },
    Deleted,
    Failed { cause: Cause },
}
/// A walk's or hash's result: the digests, the tree file and the counts of a layer's record (6.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Walk {
    pub tree: Digest,
    pub changes: Digest,
    pub tree_file: TreeFile,
    pub counts: Counts,
}
/// `<key>.tree`'s SHA-256 and size.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TreeFile {
    pub sha256: Digest,
    pub size: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Counts {
    pub bytes: u64,
    pub deletes: u64,
    pub entries: u64,
    pub files: u64,
    pub hashed: u64,
    pub puts: u64,
}
/// Why the tree helper failed a request (5.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Cause {
    NotFound,
    Permission,
    Bound,
    Refused,
    NoSpace,
    BadInput,
}
/// What `layers/<key>.record` holds of a sealed layer (6.2): the scan's layer, and what the
/// walk and the snapshot gave. `tools` and `costs` are empty and a packages layer is a
/// scratch build, until their tasks.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Record {
    pub layer: Layer,
    pub walk: Walk,
    pub parent_uuid: Option<[u8; 16]>,
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
    /// `layers/<key>.staging`, once emptied: an `rmdir` (7.3, step 5).
    Staging(Digest),
    /// `layers/<key>.tree.tmp`, done also when there is none: a walk may fail before it writes it.
    TreeTmp(Digest),
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
