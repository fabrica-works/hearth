//! Hard links: an inode's first path, and what its later paths must agree with (domain/store.md,
//! 3.4 and 5.4).
use crate::{
    limits::Limits,
    vocabulary::{Bound, Cause, Refusal, Stat, Xattr},
};
use alloc::boxed::Box;
use hearth_lib::{encode::Digest, hash::H};
use skein_lib::{List, Map, bytes};

/// An inode's first path in tree order, and a digest of the mode, owners and covered attributes
/// its later paths must have.
#[derive(Debug, Hash)]
pub(crate) struct Link {
    path: Box<[u8]>,
    agree: Digest,
}
/// The inodes with more than one link seen so far, and their first paths' bytes.
#[derive(Debug, Hash)]
pub(crate) struct Linked {
    inodes: Map<u64, Link>,
    paths: u64,
}
impl Linked {
    pub(crate) const fn new(limits: &Limits) -> Self {
        Self { inodes: Map::with_capacity(limits.linked), paths: 0 }
    }
    /// None seen, within the same bounds.
    pub(crate) const fn emptied(&self) -> Self {
        Self { inodes: Map::with_capacity(self.inodes.capacity()), paths: 0 }
    }
}

/// The digest later paths agree on: `H("link", mode, uid, gid, xattrs)`, an internal check that
/// no record holds.
fn agreement(stat: &Stat, xattrs: &List<Xattr>) -> Digest {
    let mut hash = H::new(b"link", 4);
    let values = hash.values();
    values.uint(u64::from(stat.mode));
    values.uint(u64::from(stat.uid));
    values.uint(u64::from(stat.gid));
    values.map(u64::from(xattrs.len()));
    for xattr in xattrs {
        values.key(&xattr.name);
        values.bytes(&xattr.value);
    }
    hash.finish()
}

/// The entry at `path`, a non-directory with more than one link, is the first of its inode
/// (`None`: it is kept), or a later one (the first path), which must agree with it.
pub(crate) fn seen(
    linked: &mut Linked,
    limits: &Limits,
    stat: &Stat,
    path: &[u8],
    xattrs: &List<Xattr>,
) -> Result<Option<Box<[u8]>>, Cause> {
    let agree = agreement(stat, xattrs);
    if let Some(link) = linked.inodes.get(&stat.inode) {
        return if link.agree == agree {
            Ok(Some(bytes::copy_of(&link.path)))
        } else {
            Err(Cause::Refused(Refusal::Link))
        };
    }
    let length = u64::try_from(path.len()).expect("a path fits in a u64");
    let paths = linked.paths.saturating_add(length);
    if paths > u64::from(limits.linked_paths) {
        return Err(Cause::Bound(Bound::LinkedPaths));
    }
    match linked.inodes.insert(stat.inode, Link { path: bytes::copy_of(path), agree }) {
        Ok(_) => {
            linked.paths = paths;
            Ok(None)
        }
        Err(_) => Err(Cause::Bound(Bound::Linked)),
    }
}
