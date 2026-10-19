//! The tree helper's `hash`: the walk of a layer with no parent (domain/store.md, 5.3 and 5.4).
//!
//! A step machine over its own vocabulary. It reads every entry of the layer depth-first, with
//! an explicit stack and each directory's names sorted, hashes every file, and hands the tree
//! file's items down in tree order. The protocol layer encodes them, reads the file back for the
//! digests, and ends it. The walk answers `Hashed` with the digests, the tree file and the
//! counts, or `Failed` with a path and a cause. It parses no bytes.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod batch;
pub mod limits;
mod links;
mod read;
pub mod vocabulary;
mod walk;
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use limits::Limits;
use skein_lib::{Env, Map, Queue, Stack};
use vocabulary::{Counts, Event, Output};

/// A request, or the answer that ends the walk.
pub const MAX_OUT: u32 = 1;

/// A directory on the stack: its path, and the names not yet visited, sorted bytewise.
#[derive(Debug, Hash)]
struct Level {
    path: Box<[u8]>,
    names: Map<Box<[u8]>, ()>,
}
/// Where a walk goes once the items of the entry in hand are written.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum After {
    /// To the next name.
    Advance,
    /// Into the directory in hand.
    List,
    /// To the tree file's digests.
    Digests,
}
/// What the request in flight is waiting for.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Wait {
    Idle,
    Create,
    Stat,
    Attributes,
    Attribute,
    ReadLink,
    Hash,
    Listing,
    Written(After),
    Digests,
    Finish,
}

#[derive(Debug, Hash)]
pub struct Tree {
    wait: Wait,
    key: Digest,
    /// The root's device, which every entry must share.
    device: u64,
    stack: Stack<Level>,
    /// The directory being listed, until its last page.
    listing: Option<Level>,
    /// The names held on the stack and in the listing.
    held: u32,
    entry: Option<read::Reading>,
    counts: Counts,
    linked: links::Linked,
    batch: batch::Batch,
    /// The tree digest and the change set's, once the file is read back.
    digests: Option<(Digest, Digest)>,
}
impl Tree {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self {
            wait: Wait::Idle,
            key: Digest([0; 32]),
            device: 0,
            stack: Stack::with_capacity(limits.depth.saturating_add(1)),
            listing: None,
            held: 0,
            entry: None,
            counts: walk::no_counts(),
            linked: links::Linked::new(limits),
            batch: batch::Batch::new(limits),
            digests: None,
        }
    }
}

pub fn step(tree: &mut Tree, env: &Env<Limits>, ev: Event, out: &mut Queue<Output>) {
    let limits = &env.limits;
    match ev {
        Event::Hash { key } => walk::begin(tree, key, out),
        Event::Stat(stat) => {
            if tree.wait == Wait::Stat {
                read::stat(tree, limits, stat, out);
            } else {
                walk::unexpected(tree, out);
            }
        }
        Event::Names { names, last } => match tree.wait {
            Wait::Attributes => read::attributes(tree, limits, names, out),
            Wait::Listing => walk::names(tree, limits, names, last, out),
            Wait::Idle
            | Wait::Create
            | Wait::Stat
            | Wait::Attribute
            | Wait::ReadLink
            | Wait::Hash
            | Wait::Written(_)
            | Wait::Digests
            | Wait::Finish => walk::unexpected(tree, out),
        },
        Event::Value { bytes } => {
            if tree.wait == Wait::Attribute {
                read::value(tree, limits, bytes, out);
            } else {
                walk::unexpected(tree, out);
            }
        }
        Event::Target { bytes } => {
            if tree.wait == Wait::ReadLink {
                read::target(tree, limits, bytes, out);
            } else {
                walk::unexpected(tree, out);
            }
        }
        Event::FileHashed { sha256, size } => {
            if tree.wait == Wait::Hash {
                read::hashed(tree, limits, sha256, size, out);
            } else {
                walk::unexpected(tree, out);
            }
        }
        Event::Done => match tree.wait {
            Wait::Create => walk::created(tree, limits, out),
            Wait::Written(after) => walk::proceed(tree, limits, after, out),
            Wait::Idle
            | Wait::Stat
            | Wait::Attributes
            | Wait::Attribute
            | Wait::ReadLink
            | Wait::Hash
            | Wait::Listing
            | Wait::Digests
            | Wait::Finish => walk::unexpected(tree, out),
        },
        Event::Digested { tree: digest, changes } => {
            if tree.wait == Wait::Digests {
                walk::digested(tree, digest, changes, out);
            } else {
                walk::unexpected(tree, out);
            }
        }
        Event::Finished { sha256, size } => {
            if tree.wait == Wait::Finish {
                walk::finished(tree, sha256, size, out);
            } else {
                walk::unexpected(tree, out);
            }
        }
        Event::Failed { fault } => walk::failed(tree, fault, out),
    }
}

#[cfg(test)]
mod tests;
