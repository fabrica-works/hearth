//! The tree digest and the change set's digest, hashed as their values are
//! fed, without a copy (domain/store.md, 3.4 and 3.5). Each names its count
//! first, so a hasher is told it when made; a wrong count panics, as `H` does.

use super::{Entry, Op, encode_entry};
use hearth_lib::{encode::Digest, hash::H};

/// `H("tree", list [ entry… ])`, the entries fed in tree order.
#[derive(Debug)]
pub struct TreeDigest {
    hash: H,
}
impl TreeDigest {
    /// A digest of `entries` entries.
    #[must_use]
    pub fn new(entries: u64) -> TreeDigest {
        let mut hash = H::new(b"tree", 1);
        hash.values().list(entries);
        TreeDigest { hash }
    }
    /// The next entry.
    pub fn entry(&mut self, entry: &Entry) {
        encode_entry(entry, self.hash.values());
    }
    /// The digest, once every entry is fed.
    #[must_use]
    pub fn finish(self) -> Digest {
        self.hash.finish()
    }
}

/// `H("changes", parent tree digest, child tree digest, list [ change… ])`,
/// the changes fed in tree order. A tree without a parent hashes `none` for
/// the parent's digest, as the layer record's `parent` does.
#[derive(Debug)]
pub struct ChangesDigest {
    hash: H,
}
impl ChangesDigest {
    /// A digest of `changes` changes from `parent_tree` to `tree`.
    #[must_use]
    pub fn new(parent_tree: Option<Digest>, tree: Digest, changes: u64) -> ChangesDigest {
        let mut hash = H::new(b"changes", 3);
        match parent_tree {
            Some(parent) => hash.values().digest(&parent),
            None => hash.values().none(),
        }
        hash.values().digest(&tree);
        hash.values().list(changes);
        ChangesDigest { hash }
    }
    /// The next change: a put carries the child's entry at its path, a
    /// delete none.
    pub fn change(&mut self, op: Op, path: &[u8], entry: Option<&Entry>) {
        let out = self.hash.values();
        out.list(3);
        out.uint(u64::from(op.number()));
        out.bytes(path);
        assert!(entry.is_some() == (op == Op::Put), "a put carries the child's entry, and a delete none");
        match entry {
            Some(entry) => {
                assert!(*entry.path == *path, "a put carries the entry at its path");
                encode_entry(entry, out);
            }
            None => out.none(),
        }
    }
    /// The digest, once every change is fed.
    #[must_use]
    pub fn finish(self) -> Digest {
        self.hash.finish()
    }
}
