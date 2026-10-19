//! One entry, read whole: its stat, its covered attributes, then a link's target or a file's
//! hash (domain/store.md, 3.4 and 5.4).
use crate::{
    After, Tree, Wait, batch,
    limits::Limits,
    links,
    vocabulary::{Bound, Cause, Entry, Item, Kind, Local, Output, Payload, Refusal, Request, Stat, Xattr},
    walk,
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{List, Map, Queue, bytes};

/// The largest owner id (3.4).
const OWNER: u32 = 65_535;
/// The permission bits with setuid, setgid and sticky.
const MODE: u32 = 0o7777;
/// The root's mode (3.4).
const ROOT_MODE: u32 = 0o755;

/// The entry in hand, from its name to its items.
#[derive(Debug, Hash)]
pub(crate) struct Reading {
    pub(crate) path: Box<[u8]>,
    stat: Option<Stat>,
    /// The covered attributes not yet read, by name.
    pending: Map<Box<[u8]>, ()>,
    /// The attribute being read.
    current: Option<Box<[u8]>>,
    xattrs: List<Xattr>,
}
impl Reading {
    pub(crate) fn new(path: Box<[u8]>, limits: &Limits) -> Self {
        Self {
            path,
            stat: None,
            pending: Map::with_capacity(limits.attributes),
            current: None,
            xattrs: List::with_capacity(0),
        }
    }
}

/// Why the root is refused, if it is.
const fn root_refusal(stat: &Stat) -> Option<Refusal> {
    match stat.kind {
        Kind::Directory => {}
        Kind::File | Kind::Symlink | Kind::Char | Kind::Block | Kind::Fifo | Kind::Socket => {
            return Some(Refusal::RootKind);
        }
    }
    if stat.mode & MODE != ROOT_MODE {
        Some(Refusal::RootMode)
    } else if stat.uid != 0 || stat.gid != 0 {
        Some(Refusal::RootOwner)
    } else {
        None
    }
}

/// The entry's stat: what the walk refuses is refused before anything else is read.
pub(crate) fn stat(tree: &mut Tree, limits: &Limits, stat: Stat, out: &mut Queue<Output>) {
    let reading = tree.entry.as_mut().expect("an entry is in hand while its stat is awaited");
    let root = reading.path.is_empty();
    let refusal = if root {
        root_refusal(&stat)
    } else if stat.device != tree.device {
        Some(Refusal::Device)
    } else if stat.uid > OWNER || stat.gid > OWNER {
        Some(Refusal::Owner)
    } else {
        None
    };
    if let Some(refusal) = refusal {
        walk::fail(tree, Cause::Refused(refusal), out);
    } else if tree.counts.entries >= u64::from(limits.entries) {
        walk::fail(tree, Cause::Bound(Bound::Entries), out);
    } else {
        if root {
            tree.device = stat.device;
        }
        tree.counts.entries = tree.counts.entries.checked_add(1).expect("entries are within their bound");
        reading.stat = Some(stat);
        tree.wait = Wait::Attributes;
        out.push(Output::Request(Request::Attributes { path: bytes::copy_of(&reading.path) }));
    }
}

/// Whether the walk keeps an attribute: the four families of 3.4.
fn covered(name: &[u8]) -> bool {
    name == b"security.capability"
        || name == b"system.posix_acl_access"
        || name == b"system.posix_acl_default"
        || name.starts_with(b"user.")
}

/// The attributes' names: the covered ones are read, in order, and the rest are dropped.
pub(crate) fn attributes(tree: &mut Tree, limits: &Limits, names: Box<[Box<[u8]>]>, out: &mut Queue<Output>) {
    let reading = tree.entry.as_mut().expect("an entry is in hand while its attributes are awaited");
    let mut failure = if names.len() > usize::try_from(limits.listed).expect("a u32 fits in memory") {
        Some(Cause::Bound(Bound::Attributes))
    } else {
        None
    };
    for name in names {
        if failure.is_some() {
            break;
        }
        if !covered(&name) {
            continue;
        }
        if name.len() > usize::try_from(limits.attribute_name).expect("a u32 fits in memory") {
            failure = Some(Cause::Bound(Bound::AttributeName));
        } else if reading.pending.len() >= limits.attributes {
            failure = Some(Cause::Bound(Bound::Attributes));
        } else if let Ok(Some(())) = reading.pending.insert(name, ()) {
            failure = Some(Cause::Refused(Refusal::Attribute));
        }
    }
    if let Some(cause) = failure {
        walk::fail(tree, cause, out);
    } else {
        reading.xattrs = List::with_capacity(reading.pending.len());
        next_attribute(tree, limits, out);
    }
}

/// The next covered attribute to read, or the entry's payload.
fn next_attribute(tree: &mut Tree, limits: &Limits, out: &mut Queue<Output>) {
    let reading = tree.entry.as_mut().expect("an entry is in hand while its attributes are read");
    match reading.pending.first() {
        Some((name, ())) => {
            let name = bytes::copy_of(name);
            reading.pending.remove(&*name);
            reading.current = Some(bytes::copy_of(&name));
            tree.wait = Wait::Attribute;
            out.push(Output::Request(Request::Attribute { path: bytes::copy_of(&reading.path), name }));
        }
        None => payload(tree, limits, out),
    }
}

/// An attribute's value.
pub(crate) fn value(tree: &mut Tree, limits: &Limits, value: Box<[u8]>, out: &mut Queue<Output>) {
    let reading = tree.entry.as_mut().expect("an entry is in hand while an attribute is awaited");
    if value.len() > usize::try_from(limits.attribute_value).expect("a u32 fits in memory") {
        walk::fail(tree, Cause::Bound(Bound::AttributeValue), out);
        return;
    }
    let name = reading.current.take().expect("an attribute is being read");
    assert!(reading.xattrs.push(Xattr { name, value }).is_ok(), "every covered attribute has room");
    next_attribute(tree, limits, out);
}

/// What remains to read of an entry whose attributes are in: nothing, a link's target, a file's
/// contents, or nothing for a later hard link, which is never read (3.4).
fn payload(tree: &mut Tree, limits: &Limits, out: &mut Queue<Output>) {
    let reading = tree.entry.as_mut().expect("an entry is in hand while its payload is read");
    let stat = reading.stat.expect("the stat came first");
    let mut first: Option<Box<[u8]>> = None;
    if stat.kind != Kind::Directory && stat.links > 1 {
        match links::seen(&mut tree.linked, limits, &stat, &reading.path, &reading.xattrs) {
            Ok(path) => first = path,
            Err(cause) => {
                walk::fail(tree, cause, out);
                return;
            }
        }
    }
    if let Some(path) = first {
        finish(tree, limits, Payload::Hardlink { path }, After::Advance, out);
        return;
    }
    match stat.kind {
        Kind::Directory => finish(tree, limits, Payload::Directory, After::List, out),
        Kind::Symlink => {
            if stat.size > u64::from(limits.target) {
                walk::fail(tree, Cause::Bound(Bound::Target), out);
            } else {
                tree.wait = Wait::ReadLink;
                out.push(Output::Request(Request::ReadLink { path: bytes::copy_of(&reading.path) }));
            }
        }
        Kind::File => {
            tree.wait = Wait::Hash;
            out.push(Output::Request(Request::HashFile { path: bytes::copy_of(&reading.path), size: stat.size }));
        }
        Kind::Char => finish(tree, limits, Payload::Char { major: stat.major, minor: stat.minor }, After::Advance, out),
        Kind::Block => {
            finish(tree, limits, Payload::Block { major: stat.major, minor: stat.minor }, After::Advance, out);
        }
        Kind::Fifo => finish(tree, limits, Payload::Fifo, After::Advance, out),
        Kind::Socket => finish(tree, limits, Payload::Socket, After::Advance, out),
    }
}

/// A link's target.
pub(crate) fn target(tree: &mut Tree, limits: &Limits, target: Box<[u8]>, out: &mut Queue<Output>) {
    if target.len() > usize::try_from(limits.target).expect("a u32 fits in memory") {
        walk::fail(tree, Cause::Bound(Bound::Target), out);
    } else {
        finish(tree, limits, Payload::Symlink { target }, After::Advance, out);
    }
}

/// A file's digest, and the bytes it was read in: the stat's size, or the file changed.
pub(crate) fn hashed(tree: &mut Tree, limits: &Limits, sha256: Digest, size: u64, out: &mut Queue<Output>) {
    let reading = tree.entry.as_ref().expect("an entry is in hand while its file is hashed");
    let stat = reading.stat.expect("the stat came first");
    if size != stat.size {
        walk::fail(tree, Cause::Refused(Refusal::Size), out);
        return;
    }
    tree.counts.files = tree.counts.files.saturating_add(1);
    tree.counts.bytes = tree.counts.bytes.saturating_add(size);
    tree.counts.hashed = tree.counts.hashed.saturating_add(size);
    finish(tree, limits, Payload::File { size, content: sha256 }, After::Advance, out);
}

/// The entry is read: it and its put go to the batch. A directory's stays in hand, to be listed.
fn finish(tree: &mut Tree, limits: &Limits, payload: Payload, after: After, out: &mut Queue<Output>) {
    let reading = tree.entry.as_mut().expect("an entry is in hand while its items are made");
    let stat = reading.stat.expect("the stat came first");
    let xattrs = if reading.xattrs.is_empty() {
        Box::default()
    } else {
        core::mem::replace(&mut reading.xattrs, List::with_capacity(0)).into_boxed()
    };
    let entry = Entry {
        path: bytes::copy_of(&reading.path),
        mode: stat.mode & MODE,
        uid: stat.uid,
        gid: stat.gid,
        xattrs,
        payload,
    };
    let local = Local { inode: stat.inode, ctime: stat.ctime, mtime: stat.mtime };
    let put = Item::Put { path: bytes::copy_of(&reading.path) };
    if after != After::List {
        tree.entry = None;
    }
    tree.counts.puts = tree.counts.puts.saturating_add(1);
    batch::queue(tree, limits, Item::Entry(entry, local), put, after, out);
}
