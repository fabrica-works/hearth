//! What a walk of a fake filesystem must produce, listed without the machine: the entries in
//! tree order by recursion over the nodes, sorted names, the first path of each inode, the covered
//! attributes in order, and the digests by the oracle (domain/store.md, 3.4 and 3.5).
use crate::{fs::Fs, oracle};
use hearth_domain_tree::vocabulary::Kind;
use hearth_lib::encode::Digest;
use hearth_protocol_tree::{
    layer::Counts,
    tree::{End, Entry, Item, Local, Op, Payload, Xattr},
};
use skein_lib::sha256;
use std::collections::BTreeMap;

/// Whether the walk keeps an attribute (3.4).
#[must_use]
pub fn covered(name: &[u8]) -> bool {
    matches!(name, b"security.capability" | b"system.posix_acl_access" | b"system.posix_acl_default")
        || name.starts_with(b"user.")
}

/// A layer's entries, in tree order, and the counts they come to.
#[must_use]
pub fn entries(fs: &Fs) -> (Vec<(Entry, Local)>, Counts) {
    let mut found = Vec::new();
    let mut first = BTreeMap::new();
    visit(fs, 0, &[], &mut first, &mut found);
    let mut counts = Counts { bytes: 0, deletes: 0, entries: 0, files: 0, hashed: 0, puts: 0 };
    for (entry, _) in &found {
        counts.entries += 1;
        counts.puts += 1;
        if let Payload::File { size, .. } = entry.payload {
            counts.files += 1;
            counts.bytes += size;
            counts.hashed += size;
        }
    }
    (found, counts)
}
fn visit(fs: &Fs, at: usize, path: &[u8], first: &mut BTreeMap<u64, Vec<u8>>, found: &mut Vec<(Entry, Local)>) {
    let node = fs.node(at);
    let links = fs.links(node.inode);
    let later = if node.kind != Kind::Directory && links > 1 { first.get(&node.inode).cloned() } else { None };
    if node.kind != Kind::Directory && links > 1 {
        first.entry(node.inode).or_insert_with(|| path.to_vec());
    }
    let payload = match (&later, node.kind) {
        (Some(path), _) => Payload::Hardlink { path: path.as_slice().into() },
        (None, Kind::Directory) => Payload::Directory,
        (None, Kind::File) => {
            Payload::File { size: node.content.len() as u64, content: Digest(sha256::digest(&node.content)) }
        }
        (None, Kind::Symlink) => Payload::Symlink { target: node.target.as_slice().into() },
        (None, Kind::Char) => Payload::Char { major: node.major, minor: node.minor },
        (None, Kind::Block) => Payload::Block { major: node.major, minor: node.minor },
        (None, Kind::Fifo) => Payload::Fifo,
        (None, Kind::Socket) => Payload::Socket,
    };
    let sorted: BTreeMap<&[u8], &[u8]> = node
        .xattrs
        .iter()
        .filter(|(name, _)| covered(name))
        .map(|(name, value)| (name.as_slice(), value.as_slice()))
        .collect();
    let entry = Entry {
        path: path.into(),
        mode: node.mode & 0o7777,
        uid: node.uid,
        gid: node.gid,
        xattrs: sorted.into_iter().map(|(name, value)| Xattr { name: name.into(), value: value.into() }).collect(),
        payload,
    };
    found.push((entry, Local { inode: node.inode, ctime: node.ctime, mtime: node.mtime }));
    let mut names: Vec<&(Vec<u8>, usize)> = node.names.iter().collect();
    names.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, child) in names {
        let mut below = path.to_vec();
        if !below.is_empty() {
            below.push(b'/');
        }
        below.extend_from_slice(name);
        visit(fs, *child, &below, first, found);
    }
}

/// The items of a layer's tree file, the end item last: entries, each with its put, and the
/// digests by the oracle.
#[must_use]
pub fn items(fs: &Fs, key: Digest) -> Vec<Item> {
    let (found, counts) = entries(fs);
    let list: Vec<&Entry> = found.iter().map(|(entry, _)| entry).collect();
    let tree = oracle::tree_digest(&list);
    let changes: Vec<(Op, &[u8], Option<&Entry>)> =
        list.iter().map(|entry| (Op::Put, &*entry.path, Some(*entry))).collect();
    let changes = oracle::changes_digest(None, tree, &changes);
    let mut items = Vec::new();
    for (entry, local) in found {
        let path = entry.path.clone();
        items.push(Item::Entry(entry, local));
        items.push(Item::Change { op: Op::Put, path });
    }
    items.push(Item::End(End { key, parent_tree: None, tree, changes, counts, boot: Box::new([]) }));
    items
}
