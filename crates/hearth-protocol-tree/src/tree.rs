//! A layer's tree file, kind 3 at version 1 (domain/store.md, 3.4, 3.5 and 6.3):
//! the frame's header, then items. The walk writes the header last, with the
//! body's length, at offset 0 of the placeholder it began with.

pub mod digest;
pub mod read;

pub use self::{
    digest::{ChangesDigest, TreeDigest},
    read::{Next, Reader},
};

use crate::{
    frame::{self, Kind, Refused},
    layer::{self, Counts},
    limits::Limits,
};
use alloc::boxed::Box;
use hearth_lib::{
    encode::{Digest, Encoder},
    path::path_valid,
};
use skein_lib::Writer;

/// An item's tag (domain/store.md, 6.3).
pub const ENTRY: u8 = 1;
/// A change item's tag.
pub const CHANGE: u8 = 2;
/// The end item's tag.
pub const END: u8 = 3;
/// The most attributes an entry holds (domain/store.md, 5.6).
pub const XATTRS: u32 = 64;
/// The most bytes in an attribute's value, `XATTR_SIZE_MAX` (domain/store.md, 5.6).
pub const VALUE: u32 = 65_536;
/// The most bytes in a link's target (domain/store.md, 5.6).
pub const TARGET: u32 = 4_096;
/// The most bytes in a kernel's version: a path component.
pub const VERSION: u32 = 255;
/// The largest owner id (domain/store.md, 3.4).
pub const OWNER: u32 = 65_535;
/// The largest mode, with setuid, setgid and sticky.
pub const MODE: u32 = 0o7777;
/// The nested containers an entry's or the end item's value opens at most.
const DEPTH: u32 = 3;

/// One path's entry as the tree helper read it (domain/store.md, 3.4).
/// `xattrs` are in strictly increasing order of their names.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub path: Box<[u8]>,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub xattrs: Box<[Xattr]>,
    pub payload: Payload,
}
/// An extended attribute, by name and value.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Xattr {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}
/// What an entry holds, one variant for each of the eight kinds.
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
impl Payload {
    /// The kind's number in the digest (domain/store.md, 3.4).
    #[must_use]
    pub const fn kind(&self) -> u8 {
        match self {
            Self::Directory => 1,
            Self::File { .. } => 2,
            Self::Symlink { .. } => 3,
            Self::Hardlink { .. } => 4,
            Self::Char { .. } => 5,
            Self::Block { .. } => 6,
            Self::Fifo => 7,
            Self::Socket => 8,
        }
    }
}
/// What the digests leave out, beside an entry in the tree file: the inode and
/// the times 4.3 compares, as seconds and nanoseconds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Local {
    pub inode: u64,
    pub ctime: (u64, u32),
    pub mtime: (u64, u32),
}
/// What a change does at its path (domain/store.md, 3.5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Op {
    Put,
    Delete,
}
impl Op {
    /// The operation's number in the digest.
    #[must_use]
    pub const fn number(self) -> u8 {
        match self {
            Self::Put => 1,
            Self::Delete => 2,
        }
    }
}
/// The kernel a layer holds, from its walk (domain/store.md, 6.3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Kernel {
    pub version: Box<[u8]>,
    pub kernel: Digest,
    pub modules: Digest,
    pub initramfs: Digest,
}
/// The last item: what the digests came to, and the layer's kernels, in
/// strictly increasing order of their versions.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct End {
    pub key: Digest,
    pub parent_tree: Option<Digest>,
    pub tree: Digest,
    pub changes: Digest,
    pub counts: Counts,
    pub boot: Box<[Kernel]>,
}
/// An item of a tree file: an entry, a change at a path (a put follows its
/// own path's entry), or the end.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Item {
    Entry(Entry, Local),
    Change { op: Op, path: Box<[u8]> },
    End(End),
}

/// The header of a tree file with a body of `body_len` bytes.
#[must_use]
pub fn header(body_len: u64) -> [u8; 24] {
    frame::header(Kind::Tree, body_len)
}

/// The bytes of one item: its tag, the length of its value as a `u32` and the
/// canonical value, refused when it passes a bound or is not what the reader
/// takes.
pub fn item(item: &Item, limits: &Limits) -> Result<Box<[u8]>, Refused> {
    validate(item, limits)?;
    let mut measure = Encoder::measuring();
    encode(item, &mut measure);
    let length = measure.measured();
    if length > u64::from(limits.tree.item) {
        return Err(Refused::Limit(b"item"));
    }
    let mut writer = Encoder::writing(length);
    encode(item, &mut writer);
    let value = writer.written();
    let length = u32::try_from(length).expect("an item is within its u32 limit");
    let size = value.len().checked_add(5).expect("a bounded item fits in memory");
    let mut out = Writer::new(size);
    out.put(&[tag(item)]).expect("the item has room");
    out.put(&length.to_be_bytes()).expect("the item has room");
    out.put(&value).expect("the item has room");
    Ok(out.finish())
}
const fn tag(item: &Item) -> u8 {
    match item {
        Item::Entry(..) => ENTRY,
        Item::Change { .. } => CHANGE,
        Item::End(_) => END,
    }
}

fn length(found: usize, limit: u32, name: &'static [u8]) -> Result<(), Refused> {
    if found > usize::try_from(limit).expect("a u32 fits in memory") { Err(Refused::Limit(name)) } else { Ok(()) }
}
fn validate(item: &Item, limits: &Limits) -> Result<(), Refused> {
    match item {
        Item::Entry(entry, _) => {
            if limits.depth < DEPTH {
                return Err(Refused::Limit(b"depth"));
            }
            validate_entry(entry, limits)
        }
        Item::Change { path, .. } => {
            if limits.depth < 1 {
                return Err(Refused::Limit(b"depth"));
            }
            if path_valid(path) { Ok(()) } else { Err(Refused::Limit(b"path")) }
        }
        Item::End(end) => {
            if limits.tree.key < 11 {
                return Err(Refused::Limit(b"key"));
            }
            if limits.depth < if end.boot.is_empty() { 2 } else { DEPTH } {
                return Err(Refused::Limit(b"depth"));
            }
            length(end.boot.len(), limits.tree.kernels, b"kernels")?;
            let mut previous: Option<&Kernel> = None;
            for kernel in &end.boot {
                length(kernel.version.len(), VERSION, b"version")?;
                if let Some(last) = previous
                    && last.version >= kernel.version
                {
                    return Err(Refused::Order(b"boot"));
                }
                previous = Some(kernel);
            }
            Ok(())
        }
    }
}
fn validate_entry(entry: &Entry, limits: &Limits) -> Result<(), Refused> {
    if !path_valid(&entry.path) {
        return Err(Refused::Limit(b"path"));
    }
    if entry.mode > MODE {
        return Err(Refused::Limit(b"mode"));
    }
    if entry.uid > OWNER {
        return Err(Refused::Limit(b"uid"));
    }
    if entry.gid > OWNER {
        return Err(Refused::Limit(b"gid"));
    }
    length(entry.xattrs.len(), XATTRS, b"xattrs")?;
    let mut previous: Option<&Xattr> = None;
    for xattr in &entry.xattrs {
        length(xattr.name.len(), limits.tree.key, b"key")?;
        length(xattr.value.len(), VALUE, b"value")?;
        if let Some(last) = previous
            && last.name >= xattr.name
        {
            return Err(Refused::Order(b"xattrs"));
        }
        previous = Some(xattr);
    }
    match &entry.payload {
        Payload::Symlink { target } => length(target.len(), TARGET, b"target"),
        Payload::Hardlink { path } => {
            if path_valid(path) {
                Ok(())
            } else {
                Err(Refused::Limit(b"target"))
            }
        }
        Payload::Directory
        | Payload::File { .. }
        | Payload::Char { .. }
        | Payload::Block { .. }
        | Payload::Fifo
        | Payload::Socket => Ok(()),
    }
}

fn encode(item: &Item, out: &mut Encoder) {
    match item {
        Item::Entry(entry, local) => {
            out.list(2);
            encode_entry(entry, out);
            out.list(5);
            out.uint(local.inode);
            out.uint(local.ctime.0);
            out.uint(u64::from(local.ctime.1));
            out.uint(local.mtime.0);
            out.uint(u64::from(local.mtime.1));
        }
        Item::Change { op, path } => {
            out.list(2);
            out.uint(u64::from(op.number()));
            out.bytes(path);
        }
        Item::End(end) => {
            out.map(6);
            out.key(b"boot");
            out.list(u64::try_from(end.boot.len()).expect("bounded kernels"));
            for kernel in &end.boot {
                out.map(4);
                out.key(b"initramfs");
                out.digest(&kernel.initramfs);
                out.key(b"kernel");
                out.digest(&kernel.kernel);
                out.key(b"modules");
                out.digest(&kernel.modules);
                out.key(b"version");
                out.bytes(&kernel.version);
            }
            out.key(b"changes");
            out.digest(&end.changes);
            out.key(b"counts");
            layer::encode_counts(&end.counts, out);
            out.key(b"key");
            out.digest(&end.key);
            out.key(b"parent_tree");
            match &end.parent_tree {
                Some(parent) => out.digest(parent),
                None => out.none(),
            }
            out.key(b"tree");
            out.digest(&end.tree);
        }
    }
}
/// An entry as the tree digest takes it (domain/store.md, 3.4).
pub(crate) fn encode_entry(entry: &Entry, out: &mut Encoder) {
    out.list(7);
    out.bytes(&entry.path);
    out.uint(u64::from(entry.payload.kind()));
    out.uint(u64::from(entry.mode));
    out.uint(u64::from(entry.uid));
    out.uint(u64::from(entry.gid));
    out.map(u64::try_from(entry.xattrs.len()).expect("bounded attributes"));
    for xattr in &entry.xattrs {
        out.key(&xattr.name);
        out.bytes(&xattr.value);
    }
    match &entry.payload {
        Payload::Directory | Payload::Fifo | Payload::Socket => out.none(),
        Payload::File { size, content } => {
            out.list(2);
            out.uint(*size);
            out.digest(content);
        }
        Payload::Symlink { target } => out.bytes(target),
        Payload::Hardlink { path } => out.bytes(path),
        Payload::Char { major, minor } | Payload::Block { major, minor } => {
            out.list(2);
            out.uint(u64::from(*major));
            out.uint(u64::from(*minor));
        }
    }
}
