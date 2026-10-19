//! The walk's events, requests and answers, typed (domain/store.md, 5.3 and 5.4).
//!
//! The domain parses nothing: the protocol layer turns these into the pipes' messages, the
//! tree file's items and the layer's syscalls, and back. Paths are relative to the layer's
//! root, which is the empty path (3.4).
use alloc::boxed::Box;
use hearth_lib::encode::Digest;

/// What the walk hears: the helper's `walk` or `hash` request, and the one answer to each request
/// it made.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    /// Walks the layer of this key against its parent's tree file; the end item names the key
    /// (6.3). With no parent it is `hash`: every file is read, every entry a put.
    Walk { key: Digest, parent: Option<Parent> },
    /// The answer to `Stat`.
    Stat(Stat),
    /// The answer to `List`, `Next` or `Attributes`: at most the page asked for, in any order,
    /// without `.` and `..`. `last` ends a directory's listing.
    Names { names: Box<[Box<[u8]>]>, last: bool },
    /// The answer to `Attribute`.
    Value { bytes: Box<[u8]> },
    /// The answer to `ReadLink`.
    Target { bytes: Box<[u8]> },
    /// The answer to `HashFile`: the file's SHA-256 and the bytes it was read in.
    FileHashed { sha256: Digest, size: u64 },
    /// The answer to `Create` and `Write`.
    Done,
    /// The answer to `Digests`.
    Digested { tree: Digest, changes: Digest },
    /// The answer to `Finish`: the tree file's SHA-256 and size.
    Finished { sha256: Digest, size: u64 },
    /// The answer to `ParentNext`: the parent's next entry, with its local index.
    ParentEntry { entry: Entry, local: Local },
    /// The other answer to `ParentNext`: the parent's tree file has no entry left. `tree` and
    /// `boot` are its end item's; `sha256` and `size` are those of the bytes streamed.
    ParentEnd { tree: Digest, boot: Box<[Kernel]>, sha256: Digest, size: u64 },
    /// The answer to any request that failed.
    Failed { fault: Fault },
}

/// The parent's record (6.2): its tree digest and its tree file's SHA-256 and size, which the
/// stream read must come to.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Parent {
    pub tree: Digest,
    pub tree_file: TreeFile,
}

/// Why a request failed below.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fault {
    NotFound,
    Permission,
    NoSpace,
    /// An item of a `Write` passed its bound.
    Bound,
    Other,
}

/// What the walk asks, one request at a time.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    /// The entry at `path`, not following a link.
    Stat {
        path: Box<[u8]>,
    },
    /// Opens the directory and answers its first `max` names; `Next` goes on.
    List {
        path: Box<[u8]>,
        max: u32,
    },
    Next {
        max: u32,
    },
    /// The names of the attributes of `path`, all of them, covered or not.
    Attributes {
        path: Box<[u8]>,
    },
    Attribute {
        path: Box<[u8]>,
        name: Box<[u8]>,
    },
    ReadLink {
        path: Box<[u8]>,
    },
    /// The file's contents, streamed and hashed; `size` is its stat's.
    HashFile {
        path: Box<[u8]>,
        size: u64,
    },
    /// The tree file, with a placeholder header (6.3).
    Create,
    /// Appends the items, in order.
    Write {
        items: Box<[Item]>,
    },
    /// The tree digest and the change set's, from the tree file: `entries` and `changes` are the
    /// counts they name first (3.4, 3.5).
    Digests {
        parent_tree: Option<Digest>,
        entries: u64,
        changes: u64,
    },
    /// The parent's next item of the tree file, change items skipped.
    ParentNext,
    /// Writes the end item and the real header, then syncs.
    Finish {
        end: End,
    },
}

/// What the walk answers, once for a `Walk`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    /// The shape of the store's `Walk` (6.2): a layer's digests, its tree file and its counts.
    Walked { tree: Digest, changes: Digest, tree_file: TreeFile, counts: Counts },
    /// Names the path the walk was at.
    Failed { path: Box<[u8]>, cause: Cause },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Output {
    Request(Request),
    Answer(Answer),
}

/// `<key>.tree`'s SHA-256 and size.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TreeFile {
    pub sha256: Digest,
    pub size: u64,
}
/// A layer's counts (6.2): `entries` and `files` in the tree, `bytes` the files' sizes summed,
/// `puts` and `deletes` the change set's, `hashed` the bytes read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Counts {
    pub bytes: u64,
    pub deletes: u64,
    pub entries: u64,
    pub files: u64,
    pub hashed: u64,
    pub puts: u64,
}

/// Why the walk failed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Cause {
    /// A `Walk` while a walk runs.
    Busy,
    /// An event the request in flight cannot have, or none is in flight.
    Unexpected,
    NotFound,
    Permission,
    NoSpace,
    /// An io failure of no other kind.
    Io,
    Bound(Bound),
    Refused(Refusal),
    /// The parent's stream is not the parent: its entries are out of order, or its end is not
    /// the record's.
    Parent,
}

/// The bound a walk passed (5.6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Bound {
    /// Entries in the layer.
    Entries,
    /// Names in one directory.
    Names,
    /// Directories nested.
    Depth,
    /// Names held across the stack, in all.
    Held,
    /// A name's bytes.
    Name,
    Path,
    /// A link's target.
    Target,
    /// Covered attributes of an entry, or names in an attribute list.
    Attributes,
    AttributeName,
    AttributeValue,
    /// Inodes with more than one link.
    Linked,
    /// Their first paths, in all.
    LinkedPaths,
    /// An item of the tree file, refused below.
    Item,
}

/// An entry the layer must not hold (3.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    /// The root is not a directory.
    RootKind,
    /// The root's mode is not 0755.
    RootMode,
    /// The root's owner or group is not 0.
    RootOwner,
    /// An owner past 65535.
    Owner,
    /// An entry on another device than the root's: a nested subvolume.
    Device,
    /// A file whose size changed between its stat and its hash.
    Size,
    /// A later hard link that disagrees with its inode's first path.
    Link,
    /// A name that is empty, `.`, `..`, or holds `/` or NUL, or comes twice.
    Name,
    /// An attribute listed twice.
    Attribute,
}

/// What `Stat` answers, as `statx` without following a link gives it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Stat {
    pub kind: Kind,
    /// `mode & 0o7777`.
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub inode: u64,
    pub links: u32,
    pub device: u64,
    pub major: u32,
    pub minor: u32,
    /// Seconds and nanoseconds.
    pub ctime: (u64, u32),
    pub mtime: (u64, u32),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Directory,
    File,
    Symlink,
    Char,
    Block,
    Fifo,
    Socket,
}

/// An item of the tree file to append (6.3), in tree order: an entry, then its put if it is one;
/// a delete stands alone.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Item {
    Entry(Entry, Local),
    /// The put of the entry just before it.
    Put {
        path: Box<[u8]>,
    },
    /// The parent had `path`; the child has nothing there.
    Delete {
        path: Box<[u8]>,
    },
}
/// An entry as the walk read it (3.4); `xattrs` are in strictly increasing order of their names.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub path: Box<[u8]>,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub xattrs: Box<[Xattr]>,
    pub payload: Payload,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Xattr {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Payload {
    Directory,
    File {
        size: u64,
        content: Digest,
    },
    Symlink {
        target: Box<[u8]>,
    },
    /// The path of the first entry with the inode.
    Hardlink {
        path: Box<[u8]>,
    },
    Char {
        major: u32,
        minor: u32,
    },
    Block {
        major: u32,
        minor: u32,
    },
    Fifo,
    Socket,
}
/// What the digests leave out, beside an entry: the inode and the times 4.3 compares.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Local {
    pub inode: u64,
    pub ctime: (u64, u32),
    pub mtime: (u64, u32),
}
/// A kernel a layer holds (6.3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Kernel {
    pub version: Box<[u8]>,
    pub kernel: Digest,
    pub modules: Digest,
    pub initramfs: Digest,
}
/// The end item (6.3). Its `boot` is the parent's, copied unchanged; the packages layer computes
/// its own later, and with no parent it is empty.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct End {
    pub key: Digest,
    pub parent_tree: Option<Digest>,
    pub tree: Digest,
    pub changes: Digest,
    pub counts: Counts,
    pub boot: Box<[Kernel]>,
}
