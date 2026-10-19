//! Layers to walk, generated from injected randomness: devices, symlinks, hard links (a symlink
//! with two links among them), attributes of every family, and directories deep and wide.
use crate::fs::{Fs, Node};
use hearth_domain_tree::vocabulary::Kind;
use skein_lib::Rng;

const FAMILIES: &[&[u8]] = &[
    b"security.capability",
    b"system.posix_acl_access",
    b"system.posix_acl_default",
    b"user.",
    b"user.comment",
    b"security.selinux",
    b"trusted.overlay.opaque",
    b"system.other",
    b"btrfs.compression",
];

/// A name that sorts among its siblings in ways that trip a naive order: shared prefixes, `-`
/// and `.` beside `/`, bytes past ASCII.
fn name(rng: &mut Rng, taken: &[Vec<u8>]) -> Vec<u8> {
    const PARTS: &[&[u8]] = &[b"a", b"b", b"ab", b"-", b".", b"_", b"z", b"\xff", b"\x01", b" ", b"0", b"B"];
    loop {
        let mut name = Vec::new();
        for _ in 0..rng.between(1, 4) {
            name.extend_from_slice(PARTS[usize::try_from(rng.below(PARTS.len() as u64)).expect("an index")]);
        }
        if name != b"." && name != b".." && !taken.contains(&name) {
            return name;
        }
    }
}
fn bytes(rng: &mut Rng, most: u64) -> Vec<u8> {
    (0..rng.between(0, most)).map(|_| u8::try_from(rng.below(256)).expect("a byte")).collect()
}
fn attributes(rng: &mut Rng, node: Node) -> Node {
    let mut node = node;
    for _ in 0..rng.between(0, 4) {
        let family = FAMILIES[usize::try_from(rng.below(FAMILIES.len() as u64)).expect("an index")];
        let mut attribute = family.to_vec();
        if family.ends_with(b".") || rng.chance(300) {
            attribute.extend_from_slice(&bytes(rng, 6));
            attribute.retain(|&byte| byte != 0);
        }
        if node.xattrs.iter().all(|(name, _)| *name != attribute) {
            let value = bytes(rng, 40);
            node = node.attr(&attribute, &value);
        }
    }
    node
}
fn mode(rng: &mut Rng) -> u32 {
    const MODES: &[u32] = &[0o644, 0o755, 0o600, 0o4755, 0o2755, 0o1777, 0o000, 0o7777];
    MODES[usize::try_from(rng.below(MODES.len() as u64)).expect("an index")]
}
fn owner(rng: &mut Rng) -> u32 {
    [0, 1, 1000, 65_535, 33][usize::try_from(rng.below(5)).expect("an index")]
}

/// A random layer of about `size` entries below the root.
#[must_use]
pub fn layer(rng: &mut Rng, size: u64) -> Fs {
    let mut fs = Fs::new();
    let mut directories = vec![(Vec::new(), 0_usize)];
    let mut linkable: Vec<usize> = Vec::new();
    let mut inode = 100;
    for _ in 0..size {
        inode += rng.between(1, 5);
        let (parent, _) = directories[usize::try_from(rng.below(directories.len() as u64)).expect("an index")].clone();
        let taken: Vec<Vec<u8>> = fs.list(&parent).expect("a directory").into_iter().collect();
        let leaf = name(rng, &taken);
        let mut path = parent.clone();
        if !path.is_empty() {
            path.push(b'/');
        }
        path.extend_from_slice(&leaf);
        let node = match rng.below(10) {
            0 | 1 => Node::new(Kind::Directory, inode),
            2..=4 => Node::file(inode, &bytes(rng, 200)),
            5 => Node::symlink(inode, &bytes(rng, 30).into_iter().map(|byte| byte.max(1)).collect::<Vec<_>>()),
            6 => Node::device(
                Kind::Char,
                inode,
                u32::try_from(rng.below(300)).expect("small"),
                u32::try_from(rng.below(300)).expect("small"),
            ),
            7 => Node::device(
                Kind::Block,
                inode,
                u32::try_from(rng.below(300)).expect("small"),
                u32::try_from(rng.below(300)).expect("small"),
            ),
            8 => Node::new(Kind::Fifo, inode),
            _ => Node::new(Kind::Socket, inode),
        };
        let node = if node.kind == Kind::Symlink { node } else { Node { mode: mode(rng), ..node } };
        let (uid, gid) = (owner(rng), owner(rng));
        let node = attributes(rng, node.owned(uid, gid));
        let directory = node.kind == Kind::Directory;
        let at = fs.add(&path, node);
        if directory {
            directories.push((path, at));
        } else {
            linkable.push(at);
        }
    }
    // Hard links: another name for some of the non-directories, in any directory.
    for _ in 0..size / 6 {
        if linkable.is_empty() {
            break;
        }
        let at = linkable[usize::try_from(rng.below(linkable.len() as u64)).expect("an index")];
        let (parent, _) = directories[usize::try_from(rng.below(directories.len() as u64)).expect("an index")].clone();
        let taken: Vec<Vec<u8>> = fs.list(&parent).expect("a directory");
        let leaf = name(rng, &taken);
        let mut path = parent;
        if !path.is_empty() {
            path.push(b'/');
        }
        path.extend_from_slice(&leaf);
        fs.link(&path, at);
    }
    fs.shuffle(rng);
    fs
}

/// A layer of one directory below the root with `width` files, and a chain of `depth`
/// directories below it: wide and deep.
#[must_use]
pub fn wide_and_deep(rng: &mut Rng, width: u64, depth: u64) -> Fs {
    let mut fs = Fs::new();
    fs.add(b"wide", Node::new(Kind::Directory, 2));
    for number in 0..width {
        fs.add(format!("wide/{number:06}").as_bytes(), Node::file(10 + number, format!("{number}").as_bytes()));
    }
    let mut path = b"deep".to_vec();
    fs.add(&path, Node::new(Kind::Directory, 3));
    for level in 0..depth {
        path.extend_from_slice(b"/d");
        fs.add(&path, Node::new(Kind::Directory, 4 + level + 1_000_000));
    }
    path.extend_from_slice(b"/leaf");
    fs.add(&path, Node::file(5, b"bottom"));
    fs.shuffle(rng);
    fs
}
