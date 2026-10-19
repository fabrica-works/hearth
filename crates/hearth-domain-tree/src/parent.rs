//! The parent's tree file, read as a stream of typed entries and merged with the walk
//! (domain/store.md, 4.3 and 5.4).
//!
//! The walk holds one parent entry at a time. Before it visits a path, every parent path that
//! comes before it in tree order is a delete; after it ends, every parent path left is. At a
//! path both have, an entry whose inode and change time are the parent's is kept unread (4.3).
use crate::{
    After, Tree, Wait, batch,
    limits::Limits,
    vocabulary::{Cause, Entry, Item, Kernel, Kind, Local, Output, Parent, Payload, Request, Stat},
    walk,
};
use alloc::boxed::Box;
use core::cmp::Ordering;
use hearth_lib::{encode::Digest, path::tree_order};
use skein_lib::{Queue, bytes};

/// Where the parent's stream is.
#[derive(Debug, Hash)]
enum Feed {
    /// The next entry is not asked for yet.
    Need,
    /// The entry in hand, not yet merged.
    Have(Entry, Local),
    /// The stream ended, and was the record's.
    Ended,
}

/// The parent, as the walk merges with it.
#[derive(Debug, Hash)]
pub(crate) struct Merge {
    pub(crate) record: Parent,
    feed: Feed,
    /// The last path received, which the next must come after in tree order.
    last: Option<Box<[u8]>>,
    /// The parent's end item's kernels, once its stream ends.
    boot: Box<[Kernel]>,
}
impl Merge {
    /// The merge a walk with this parent, if any, starts with.
    pub(crate) fn start(record: Option<Parent>) -> Option<Self> {
        let record = record?;
        Some(Self { record, feed: Feed::Need, last: None, boot: Box::new([]) })
    }
    /// The parent's tree digest.
    pub(crate) fn tree_of(merge: Option<&Self>) -> Option<Digest> {
        let merge = merge?;
        Some(merge.record.tree)
    }
    /// The kernels to copy into the end item, which the merge no longer holds.
    pub(crate) fn boot(&mut self) -> Box<[Kernel]> {
        core::mem::replace(&mut self.boot, Box::new([]))
    }
    /// The entry in hand, if it is the one at `path`.
    fn at(&self, path: &[u8]) -> Option<(&Entry, &Local)> {
        match &self.feed {
            Feed::Have(entry, local) if *entry.path == *path => Some((entry, local)),
            Feed::Have(..) | Feed::Need | Feed::Ended => None,
        }
    }
    /// The entry in hand, merged: the next is to be asked for.
    fn consume(&mut self) -> Option<Entry> {
        match core::mem::replace(&mut self.feed, Feed::Need) {
            Feed::Have(entry, _) => Some(entry),
            Feed::Need => None,
            Feed::Ended => {
                self.feed = Feed::Ended;
                None
            }
        }
    }
}

/// Whether the stat's kind is the payload's: a hard link is any kind but a directory.
fn agrees(payload: &Payload, kind: Kind) -> bool {
    match payload {
        Payload::Directory => kind == Kind::Directory,
        Payload::File { .. } => kind == Kind::File,
        Payload::Symlink { .. } => kind == Kind::Symlink,
        Payload::Hardlink { .. } => kind != Kind::Directory,
        Payload::Char { .. } => kind == Kind::Char,
        Payload::Block { .. } => kind == Kind::Block,
        Payload::Fifo => kind == Kind::Fifo,
        Payload::Socket => kind == Kind::Socket,
    }
}

/// Whether the entry the walk is at is unchanged (4.3): the parent has its path, and the stat's
/// inode and change time are the parent's. A kind that disagrees is not the same inode.
pub(crate) fn unchanged(merge: Option<&Merge>, path: &[u8], stat: &Stat) -> bool {
    let Some(merge) = merge else { return false };
    match merge.at(path) {
        Some((entry, local)) => {
            local.inode == stat.inode && local.ctime == stat.ctime && agrees(&entry.payload, stat.kind)
        }
        None => false,
    }
}

/// An entry read whole is merged: whether it is a put. The parent's entry at its path, if any,
/// is used up; an entry equal to it is no change (3.5).
pub(crate) fn settle(merge: Option<&mut Merge>, entry: &Entry) -> bool {
    let Some(merge) = merge else { return true };
    let same = match merge.at(&entry.path) {
        Some((parent, _)) => *parent == *entry,
        None => return true,
    };
    let used = merge.consume();
    assert!(used.is_some(), "the parent's entry was in hand");
    !same
}

/// The parent's entry at the walk's path, which is the child's, unread.
pub(crate) fn take(merge: Option<&mut Merge>) -> Option<Entry> {
    match merge {
        Some(merge) => merge.consume(),
        None => None,
    }
}

/// Brings the parent up to the walk's path: each parent path before it is a delete, written
/// before the path's own items. With the walk's path in hand it goes on to read it; with none,
/// the walk is over and every parent path left is a delete.
pub(crate) fn catch_up(tree: &mut Tree, limits: &Limits, out: &mut Queue<Output>) {
    enum Step {
        Ask,
        Delete,
        Ready,
    }
    let step = match &tree.parent {
        None => Step::Ready,
        Some(merge) => match &merge.feed {
            Feed::Need => Step::Ask,
            Feed::Ended => Step::Ready,
            Feed::Have(held, _) => match &tree.entry {
                Some(reading) => match tree_order(&held.path, &reading.path) {
                    Ordering::Less => Step::Delete,
                    Ordering::Equal | Ordering::Greater => Step::Ready,
                },
                None => Step::Delete,
            },
        },
    };
    match step {
        Step::Ask => {
            tree.wait = Wait::Parent;
            out.push(Output::Request(Request::ParentNext));
        }
        Step::Delete => delete(tree, limits, out),
        Step::Ready => match &tree.entry {
            Some(reading) => {
                tree.wait = Wait::Stat;
                out.push(Output::Request(Request::Stat { path: bytes::copy_of(&reading.path) }));
            }
            None => batch::flush(tree, limits, After::Digests, out),
        },
    }
}
/// The entry in hand is only in the parent: a delete, standing alone.
fn delete(tree: &mut Tree, limits: &Limits, out: &mut Queue<Output>) {
    let merge = tree.parent.as_mut().expect("a parent entry is in hand");
    let gone = merge.consume().expect("an entry is in hand");
    tree.counts.deletes = tree.counts.deletes.saturating_add(1);
    batch::queue(tree, limits, Item::Delete { path: gone.path }, None, After::Merge, out);
}

/// The parent's next entry: it comes after the last, or the stream is not the parent's.
pub(crate) fn received(tree: &mut Tree, limits: &Limits, entry: Entry, local: Local, out: &mut Queue<Output>) {
    let merge = tree.parent.as_mut().expect("a parent is read while its entry is awaited");
    let ordered = match &merge.last {
        Some(last) => tree_order(last, &entry.path) == Ordering::Less,
        None => true,
    };
    if !ordered {
        walk::fail(tree, Cause::Parent, out);
        return;
    }
    merge.last = Some(bytes::copy_of(&entry.path));
    merge.feed = Feed::Have(entry, local);
    catch_up(tree, limits, out);
}

/// The parent's end: its tree digest, and the bytes streamed, are the record's, or the stream
/// is not the parent's.
pub(crate) fn ended(
    tree: &mut Tree,
    limits: &Limits,
    parent: Digest,
    boot: Box<[Kernel]>,
    sha256: Digest,
    size: u64,
    out: &mut Queue<Output>,
) {
    let merge = tree.parent.as_mut().expect("a parent is read while its entry is awaited");
    if parent != merge.record.tree || sha256 != merge.record.tree_file.sha256 || size != merge.record.tree_file.size {
        walk::fail(tree, Cause::Parent, out);
        return;
    }
    merge.feed = Feed::Ended;
    merge.boot = boot;
    catch_up(tree, limits, out);
}
