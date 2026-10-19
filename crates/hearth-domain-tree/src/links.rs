//! Hard links: an inode's first path, and what its later paths must agree with (domain/store.md,
//! 3.4 and 5.4).
use crate::{
    limits::Limits,
    read::MODE,
    vocabulary::{Bound, Cause, Refusal, Stat, Xattr},
};
use alloc::boxed::Box;
use hearth_lib::{encode::Digest, hash::H};
use skein_lib::{Map, bytes};

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
pub(crate) fn agreement(mode: u32, uid: u32, gid: u32, xattrs: &[Xattr]) -> Digest {
    let mut hash = H::new(b"link", 4);
    let values = hash.values();
    values.uint(u64::from(mode));
    values.uint(u64::from(uid));
    values.uint(u64::from(gid));
    values.map(u64::try_from(xattrs.len()).expect("a length fits in a u64"));
    for xattr in xattrs {
        values.key(&xattr.name);
        values.bytes(&xattr.value);
    }
    hash.finish()
}

/// The entry at `path`, a non-directory with more than one link, read whole, is the first of its
/// inode (`None`: it is kept), or a later one (the first path), which must agree with it.
pub(crate) fn seen(
    linked: &mut Linked,
    limits: &Limits,
    stat: &Stat,
    path: &[u8],
    xattrs: &[Xattr],
) -> Result<Option<Box<[u8]>>, Cause> {
    let agree = agreement(stat.mode & MODE, stat.uid, stat.gid, xattrs);
    if let Some(link) = linked.inodes.get(&stat.inode) {
        return if link.agree == agree {
            Ok(Some(bytes::copy_of(&link.path)))
        } else {
            Err(Cause::Refused(Refusal::Link))
        };
    }
    register(linked, limits, stat.inode, path, agree)?;
    Ok(None)
}

/// An unchanged entry's inode, with more than one link, is met: its first path is the entry's
/// own, or the one it points at. Nothing is checked, since its change time is its inode's, and
/// that did not move (domain/store.md, 4.3).
pub(crate) fn met(linked: &mut Linked, limits: &Limits, inode: u64, first: &[u8], agree: Digest) -> Result<(), Cause> {
    if linked.inodes.contains_key(&inode) {
        return Ok(());
    }
    register(linked, limits, inode, first, agree)
}

fn register(linked: &mut Linked, limits: &Limits, inode: u64, first: &[u8], agree: Digest) -> Result<(), Cause> {
    let length = u64::try_from(first.len()).expect("a path fits in a u64");
    let paths = linked.paths.saturating_add(length);
    if paths > u64::from(limits.linked_paths) {
        return Err(Cause::Bound(Bound::LinkedPaths));
    }
    match linked.inodes.insert(inode, Link { path: bytes::copy_of(first), agree }) {
        Ok(_) => {
            linked.paths = paths;
            Ok(())
        }
        Err(_) => Err(Cause::Bound(Bound::Linked)),
    }
}
