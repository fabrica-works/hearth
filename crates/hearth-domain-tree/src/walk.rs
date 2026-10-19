//! The walk's course: the stack, the listings, and the ends (domain/store.md, 5.4).
use crate::{
    After, Level, Tree, Wait,
    batch::{self, Batch},
    limits::Limits,
    links::Linked,
    read::Reading,
    vocabulary::{Answer, Bound, Cause, Counts, End, Fault, Output, Refusal, Request, TreeFile},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{Map, Queue, Writer, bytes};

pub(crate) const fn no_counts() -> Counts {
    Counts { bytes: 0, deletes: 0, entries: 0, files: 0, hashed: 0, puts: 0 }
}
/// The root's path.
pub(crate) fn root() -> Box<[u8]> {
    bytes::copy_of(&[])
}

/// A `Hash`: the tree file first. A walk running is not disturbed by a second.
pub(crate) fn begin(tree: &mut Tree, key: Digest, out: &mut Queue<Output>) {
    if tree.wait != Wait::Idle {
        out.push(Output::Answer(Answer::Failed { path: root(), cause: Cause::Busy }));
        return;
    }
    tree.key = key;
    tree.wait = Wait::Create;
    out.push(Output::Request(Request::Create));
}

/// The tree file is there: the root's entry is the first to read.
pub(crate) fn created(tree: &mut Tree, limits: &Limits, out: &mut Queue<Output>) {
    tree.wait = Wait::Stat;
    tree.entry = Some(Reading::new(root(), limits));
    out.push(Output::Request(Request::Stat { path: root() }));
}

/// What a walk does once the items of the entry in hand are written.
pub(crate) fn proceed(tree: &mut Tree, limits: &Limits, after: After, out: &mut Queue<Output>) {
    match after {
        After::Advance => advance(tree, limits, out),
        After::List => list(tree, limits, out),
        After::Digests => {
            tree.wait = Wait::Digests;
            out.push(Output::Request(Request::Digests {
                parent_tree: None,
                entries: tree.counts.entries,
                changes: tree.counts.puts,
            }));
        }
    }
}

/// The next name of the innermost directory with any, as the entry in hand; or, with the stack
/// empty, the end of the walk.
pub(crate) fn advance(tree: &mut Tree, limits: &Limits, out: &mut Queue<Output>) {
    // Each pass takes a name and returns, or pops a level: the stack is empty after its depth.
    for _ in 0..=tree.stack.capacity() {
        let Some(level) = tree.stack.top_mut() else { break };
        match level.names.first() {
            Some((name, ())) => {
                let name = bytes::copy_of(name);
                level.names.remove(&*name);
                let path = join(&level.path, &name);
                tree.held = tree.held.checked_sub(1).expect("a name taken was held");
                if path.len() > usize::try_from(limits.path).expect("a u32 fits in memory") {
                    reset(tree);
                    out.push(Output::Answer(Answer::Failed { path, cause: Cause::Bound(Bound::Path) }));
                } else {
                    tree.wait = Wait::Stat;
                    tree.entry = Some(Reading::new(bytes::copy_of(&path), limits));
                    out.push(Output::Request(Request::Stat { path }));
                }
                return;
            }
            None => drop(tree.stack.pop()),
        }
    }
    assert!(tree.stack.is_empty(), "every level was popped");
    batch::flush(tree, limits, After::Digests, out);
}
/// `dir/name`, or `name` in the root.
fn join(dir: &[u8], name: &[u8]) -> Box<[u8]> {
    if dir.is_empty() {
        return bytes::copy_of(name);
    }
    let length = dir.len().saturating_add(1).saturating_add(name.len());
    let mut path = Writer::new(length);
    path.put(dir).expect("the path has room");
    path.put(b"/").expect("the path has room");
    path.put(name).expect("the path has room");
    path.finish()
}

/// The directory in hand is listed: its first page.
fn list(tree: &mut Tree, limits: &Limits, out: &mut Queue<Output>) {
    let reading = tree.entry.take().expect("the directory is in hand");
    if tree.stack.len() >= tree.stack.capacity() {
        reset(tree);
        out.push(Output::Answer(Answer::Failed { path: reading.path, cause: Cause::Bound(Bound::Depth) }));
        return;
    }
    out.push(Output::Request(Request::List { path: bytes::copy_of(&reading.path), max: limits.page }));
    tree.listing = Some(Level { path: reading.path, names: Map::with_capacity(limits.names) });
    tree.wait = Wait::Listing;
}

/// A page of the listing: its names are kept, sorted, within the bounds.
pub(crate) fn names(tree: &mut Tree, limits: &Limits, names: Box<[Box<[u8]>]>, last: bool, out: &mut Queue<Output>) {
    let listing = tree.listing.as_mut().expect("a directory is being listed");
    let mut failure = if names.len() > usize::try_from(limits.page).expect("a u32 fits in memory") {
        Some(Cause::Unexpected)
    } else {
        None
    };
    for name in names {
        if failure.is_some() {
            break;
        }
        failure = refuse(&name, limits);
        if failure.is_none() {
            if listing.names.len() >= limits.names {
                failure = Some(Cause::Bound(Bound::Names));
            } else if tree.held >= limits.held {
                failure = Some(Cause::Bound(Bound::Held));
            } else {
                match listing.names.insert(name, ()) {
                    Ok(None) => tree.held = tree.held.checked_add(1).expect("held names are within their bound"),
                    Ok(Some(())) => failure = Some(Cause::Refused(Refusal::Name)),
                    Err(_) => failure = Some(Cause::Bound(Bound::Names)),
                }
            }
        }
    }
    if let Some(cause) = failure {
        fail(tree, cause, out);
    } else if last {
        let level = tree.listing.take().expect("a directory is being listed");
        assert!(tree.stack.push(level).is_ok(), "the depth was checked before the listing");
        advance(tree, limits, out);
    } else {
        out.push(Output::Request(Request::Next { max: limits.page }));
    }
}
/// Why a name is not one: it is not a component, or it is too long.
fn refuse(name: &[u8], limits: &Limits) -> Option<Cause> {
    if name.is_empty() || name == b"." || name == b".." || name.contains(&b'/') || name.contains(&0) {
        Some(Cause::Refused(Refusal::Name))
    } else if name.len() > usize::try_from(limits.name).expect("a u32 fits in memory") {
        Some(Cause::Bound(Bound::Name))
    } else {
        None
    }
}

/// The digests came: the end item and the header close the file.
pub(crate) fn digested(tree: &mut Tree, digest: Digest, changes: Digest, out: &mut Queue<Output>) {
    tree.digests = Some((digest, changes));
    tree.wait = Wait::Finish;
    out.push(Output::Request(Request::Finish {
        end: End { key: tree.key, parent_tree: None, tree: digest, changes, counts: tree.counts },
    }));
}
/// The file is whole: the walk is done.
pub(crate) fn finished(tree: &mut Tree, sha256: Digest, size: u64, out: &mut Queue<Output>) {
    let (digest, changes) = tree.digests.expect("the digests precede the end");
    let counts = tree.counts;
    reset(tree);
    out.push(Output::Answer(Answer::Hashed { tree: digest, changes, tree_file: TreeFile { sha256, size }, counts }));
}

/// The request in flight failed below.
pub(crate) fn failed(tree: &mut Tree, fault: Fault, out: &mut Queue<Output>) {
    let cause = match fault {
        Fault::NotFound => Cause::NotFound,
        Fault::Permission => Cause::Permission,
        Fault::NoSpace => Cause::NoSpace,
        Fault::Bound => Cause::Bound(Bound::Item),
        Fault::Other => Cause::Io,
    };
    if tree.wait == Wait::Idle { unexpected(tree, out) } else { fail(tree, cause, out) }
}
/// An event that the request in flight cannot be answered by, or that none was waiting for.
pub(crate) fn unexpected(tree: &mut Tree, out: &mut Queue<Output>) {
    fail(tree, Cause::Unexpected, out);
}
/// The walk ends at the path it was at.
pub(crate) fn fail(tree: &mut Tree, cause: Cause, out: &mut Queue<Output>) {
    let path = match tree.wait {
        Wait::Idle | Wait::Create | Wait::Digests | Wait::Finish => root(),
        Wait::Stat | Wait::Attributes | Wait::Attribute | Wait::ReadLink | Wait::Hash => match &tree.entry {
            Some(reading) => bytes::copy_of(&reading.path),
            None => root(),
        },
        Wait::Listing => match &tree.listing {
            Some(level) => bytes::copy_of(&level.path),
            None => root(),
        },
        Wait::Written(_) => match &tree.batch.flight {
            Some(path) => bytes::copy_of(path),
            None => root(),
        },
    };
    reset(tree);
    out.push(Output::Answer(Answer::Failed { path, cause }));
}
/// A walk ends, however: nothing of it is kept.
fn reset(tree: &mut Tree) {
    for _ in 0..tree.stack.len() {
        drop(tree.stack.pop());
    }
    tree.wait = Wait::Idle;
    tree.device = 0;
    tree.listing = None;
    tree.held = 0;
    tree.entry = None;
    tree.counts = no_counts();
    tree.linked = Linked::emptied(&tree.linked);
    tree.batch = Batch::emptied(&tree.batch);
    tree.digests = None;
}
