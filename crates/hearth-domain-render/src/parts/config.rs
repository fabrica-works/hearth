//! The static file tree's canonical record (render.md, 4.2).
use crate::revision::Mode;
use alloc::boxed::Box;
use hearth_lib::{
    encode::{Digest, Encoder},
    hash::H,
};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Config {
    pub tree: Box<[TreeEntry]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TreeEntry {
    pub path: Box<[u8]>,
    pub node: Node,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Node {
    Link { target: Box<[u8]>, owner: Box<[u8]>, group: Box<[u8]> },
    Directory { mode: u16, owner: Box<[u8]>, group: Box<[u8]> },
    File { content_digest: Digest, mode: u16, owner: Box<[u8]>, group: Box<[u8]> },
}
#[must_use]
pub fn file(path: &[u8], mode: Mode, content_digest: Digest) -> TreeEntry {
    TreeEntry {
        path: absolute(path),
        node: Node::File {
            content_digest,
            mode: if mode == Mode::Executable { 0o755 } else { 0o644 },
            owner: skein_lib::bytes::copy_of(b"root"),
            group: skein_lib::bytes::copy_of(b"root"),
        },
    }
}
#[must_use]
pub fn digest(config: &Config) -> Digest {
    let mut hash = H::new(b"part:config", 1);
    encode(config, hash.values());
    hash.finish()
}
pub fn encode(config: &Config, record: &mut Encoder) {
    record.map(1);
    record.key(b"tree");
    record.map(u64::try_from(config.tree.len()).expect("bounded tree"));
    for entry in &config.tree {
        record.key(&entry.path);
        match &entry.node {
            Node::Link { target, owner, group } => {
                record.map(4);
                record.key(b"group");
                record.bytes(group);
                record.key(b"kind");
                record.bytes(b"link");
                record.key(b"owner");
                record.bytes(owner);
                record.key(b"target");
                record.bytes(target);
            }
            Node::Directory { mode, owner, group } => {
                record.map(4);
                record.key(b"group");
                record.bytes(group);
                record.key(b"kind");
                record.bytes(b"directory");
                record.key(b"mode");
                record.uint(u64::from(*mode));
                record.key(b"owner");
                record.bytes(owner);
            }
            Node::File { content_digest, mode, owner, group } => {
                record.map(5);
                record.key(b"content_digest");
                record.digest(content_digest);
                record.key(b"group");
                record.bytes(group);
                record.key(b"kind");
                record.bytes(b"file");
                record.key(b"mode");
                record.uint(u64::from(*mode));
                record.key(b"owner");
                record.bytes(owner);
            }
        }
    }
}

fn absolute(path: &[u8]) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(path.len().saturating_add(1));
    writer.put(b"/").expect("prefix room");
    writer.put(path).expect("path room");
    writer.finish()
}

/// Paths changed in byte order (render.md, 6.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub tree: Box<[EntryChange]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EntryChange {
    Added { path: Box<[u8]>, node: Node },
    Removed { path: Box<[u8]>, node: Node },
    Changed { path: Box<[u8]>, before: Node, after: Node, sizes: Option<Sizes>, hunks: Option<crate::diff::Hunks> },
}
#[must_use]
pub fn change(before: Option<&Config>, after: Option<&Config>) -> Change {
    let before = match before {
        Some(config) => config.tree.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.tree.as_ref(),
        None => &[],
    };
    let capacity = u32::try_from(before.len().saturating_add(after.len())).expect("bounded trees");
    let mut paths = skein_lib::Map::with_capacity(capacity);
    for entry in before {
        paths.insert(entry.path.clone(), (Some(&entry.node), None)).expect("tree capacity");
    }
    for entry in after {
        if let Some((_, node)) = paths.get_mut(&entry.path) {
            *node = Some(&entry.node);
        } else {
            paths.insert(entry.path.clone(), (None, Some(&entry.node))).expect("tree capacity");
        }
    }
    let mut tree = skein_lib::List::with_capacity(capacity);
    for (path, (old, new)) in &paths {
        let change = match (old, new) {
            (None, Some(node)) => EntryChange::Added { path: path.clone(), node: (*node).clone() },
            (Some(node), None) => EntryChange::Removed { path: path.clone(), node: (*node).clone() },
            (Some(old), Some(new)) if old != new => EntryChange::Changed {
                path: path.clone(),
                before: (*old).clone(),
                after: (*new).clone(),
                sizes: None,
                hunks: None,
            },
            (Some(_), Some(_)) => continue,
            (None, None) => unreachable!("union path"),
        };
        tree.push(change).expect("tree capacity");
    }
    Change { tree: tree.into_boxed() }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Sizes {
    pub before: u64,
    pub after: u64,
}
impl EntryChange {
    pub(crate) fn pair(
        &mut self,
        before_sizes: &crate::call::Sizes,
        after_sizes: &crate::call::Sizes,
        max: u32,
    ) -> Option<(Digest, Digest)> {
        match self {
            Self::Changed {
                before: Node::File { content_digest: before, .. },
                after: Node::File { content_digest: after, .. },
                sizes,
                ..
            } => {
                let pair = Sizes { before: *before_sizes.get(before)?, after: *after_sizes.get(after)? };
                *sizes = Some(pair);
                if before != after && pair.before < u64::from(max) && pair.after < u64::from(max) {
                    Some((*before, *after))
                } else {
                    None
                }
            }
            Self::Added { .. } | Self::Removed { .. } | Self::Changed { .. } => None,
        }
    }
}

/// A rendered template installs with exactly its last suffix removed.
#[must_use]
pub fn rendered(path: &[u8], mode: Mode, content_digest: Digest) -> TreeEntry {
    file(path.strip_suffix(b".j2").expect("template suffix"), mode, content_digest)
}
