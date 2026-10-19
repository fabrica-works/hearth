//! The items of the tree file, batched into writes (domain/store.md, 5.4).
use crate::{
    After, Tree, Wait,
    limits::{self, Limits},
    vocabulary::{Item, Output, Payload, Request, Xattr},
    walk,
};
use alloc::boxed::Box;
use skein_lib::{List, Queue};

/// Items waiting for a write, at most `limits.batch` bytes' worth, except an entry with its put
/// that is heavier by itself.
#[derive(Debug, Hash)]
pub(crate) struct Batch {
    items: List<Item>,
    bytes: u64,
    /// The path of the first entry in the batch.
    first: Option<Box<[u8]>>,
    /// The same, of the batch being written.
    pub(crate) flight: Option<Box<[u8]>>,
}
impl Batch {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self { items: List::with_capacity(limits::batch_items(limits)), bytes: 0, first: None, flight: None }
    }
    /// Nothing waiting, nothing in flight, within the same bounds.
    pub(crate) fn emptied(&self) -> Self {
        Self { items: List::with_capacity(self.items.capacity()), bytes: 0, first: None, flight: None }
    }
}

/// What an item weighs: its own size, and the bytes it owns.
fn weigh(item: &Item) -> u64 {
    let own = u64::try_from(size_of::<Item>()).expect("an item fits in a u64");
    let xattr = u64::try_from(size_of::<Xattr>()).expect("an attribute fits in a u64");
    let owned = match item {
        Item::Entry(entry, _) => {
            let mut sum = entry.path.len();
            for attribute in &entry.xattrs {
                sum = sum.saturating_add(attribute.name.len()).saturating_add(attribute.value.len());
            }
            let payload = match &entry.payload {
                Payload::Symlink { target } => target.len(),
                Payload::Hardlink { path } => path.len(),
                Payload::Directory
                | Payload::File { .. }
                | Payload::Char { .. }
                | Payload::Block { .. }
                | Payload::Fifo
                | Payload::Socket => 0,
            };
            let attributes = u64::try_from(entry.xattrs.len()).expect("a length fits in a u64").saturating_mul(xattr);
            u64::try_from(sum.saturating_add(payload)).expect("a length fits in a u64").saturating_add(attributes)
        }
        Item::Put { path } | Item::Delete { path } => u64::try_from(path.len()).expect("a length fits in a u64"),
    };
    own.saturating_add(owned)
}

/// An item and the one that goes with it, if any (an entry and its put), are kept for the next
/// write, or written with the batch before them when they would pass its bound, and the walk goes
/// on `after` the write.
pub(crate) fn queue(
    tree: &mut Tree,
    limits: &Limits,
    first: Item,
    second: Option<Item>,
    after: After,
    out: &mut Queue<Output>,
) {
    let weight = match &second {
        Some(second) => weigh(&first).saturating_add(weigh(second)),
        None => weigh(&first),
    };
    let full = tree.batch.bytes.saturating_add(weight) > u64::from(limits.batch);
    let flushing = full && !tree.batch.items.is_empty();
    if flushing {
        send(tree, limits, out);
    }
    if tree.batch.items.is_empty() {
        let path = match &first {
            Item::Entry(entry, _) => skein_lib::bytes::copy_of(&entry.path),
            Item::Put { path } | Item::Delete { path } => skein_lib::bytes::copy_of(path),
        };
        tree.batch.first = Some(path);
    }
    tree.batch.bytes = tree.batch.bytes.saturating_add(weight);
    assert!(tree.batch.items.push(first).is_ok(), "an item and its put fit the batch");
    if let Some(second) = second {
        assert!(tree.batch.items.push(second).is_ok(), "an item and its put fit the batch");
    }
    if flushing {
        tree.wait = Wait::Written(after);
    } else {
        walk::proceed(tree, limits, after, out);
    }
}

/// The batch is written, however light, and the walk goes on `after` it.
pub(crate) fn flush(tree: &mut Tree, limits: &Limits, after: After, out: &mut Queue<Output>) {
    if tree.batch.items.is_empty() {
        walk::proceed(tree, limits, after, out);
    } else {
        send(tree, limits, out);
        tree.wait = Wait::Written(after);
    }
}
fn send(tree: &mut Tree, limits: &Limits, out: &mut Queue<Output>) {
    let items = core::mem::replace(&mut tree.batch.items, List::with_capacity(limits::batch_items(limits)));
    tree.batch.flight = tree.batch.first.take();
    tree.batch.bytes = 0;
    out.push(Output::Request(Request::Write { items: items.into_boxed() }));
}
