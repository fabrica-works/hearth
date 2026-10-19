//! Edits to a copy of a layer, as the things that happen to a layer between a parent and its
//! child: each moves the change times the kernel moves (domain/store.md, 4.3), and no more.
use crate::{
    fs::{Fs, Node},
    layers,
};
use hearth_domain_tree::vocabulary::Kind;
use skein_lib::Rng;

/// What can happen to a path.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Edit {
    /// New contents: the change and modification times move.
    Write,
    Chmod,
    Chown,
    /// An attribute set, replaced or removed, covered or not.
    Attribute,
    /// Another name for an inode: its change time moves.
    Link,
    /// A name of an inode with two or more removed.
    Unlink,
    DeleteFile,
    /// A directory with everything beneath it.
    DeleteDirectory,
    /// A new inode of another kind at a path.
    KindChange,
    /// The inode number of a deleted file given to a new file, with a new change time.
    ReuseInode,
    /// The times move, the contents do not.
    Touch,
    /// A new path.
    Add,
    /// A name moves, to another directory or under another name.
    Rename,
}
pub const ALL: [Edit; 13] = [
    Edit::Write,
    Edit::Chmod,
    Edit::Chown,
    Edit::Attribute,
    Edit::Link,
    Edit::Unlink,
    Edit::DeleteFile,
    Edit::DeleteDirectory,
    Edit::KindChange,
    Edit::ReuseInode,
    Edit::Touch,
    Edit::Add,
    Edit::Rename,
];

fn pick<T: Clone>(rng: &mut Rng, items: &[T]) -> Option<T> {
    if items.is_empty() {
        None
    } else {
        Some(items[usize::try_from(rng.below(items.len() as u64)).expect("an index")].clone())
    }
}
fn join(directory: &[u8], leaf: &[u8]) -> Vec<u8> {
    let mut path = directory.to_vec();
    if !path.is_empty() {
        path.push(b'/');
    }
    path.extend_from_slice(leaf);
    path
}
/// A name nobody in the directory has.
fn fresh(rng: &mut Rng, fs: &Fs, directory: &[u8]) -> Vec<u8> {
    let taken = fs.list(directory).expect("a directory");
    join(directory, &layers::name(rng, &taken))
}

type Paths = Vec<(Vec<u8>, usize)>;

/// The names of the layer whose node is of a kind `wanted` accepts.
fn named(fs: &Fs, wanted: fn(Kind) -> bool) -> Paths {
    fs.paths().into_iter().filter(|(_, at)| wanted(fs.node(*at).kind)).collect()
}
fn leaves(fs: &Fs) -> Paths {
    named(fs, |kind| kind != Kind::Directory)
}
fn directories(fs: &Fs) -> Paths {
    named(fs, |kind| kind == Kind::Directory)
}
/// Every name but the root's, which the walk refuses to see changed.
fn below_root(fs: &Fs) -> Paths {
    fs.paths().into_iter().filter(|(path, _)| !path.is_empty()).collect()
}
/// The leaves with `count` names.
fn shared(fs: &Fs, count: fn(usize) -> bool) -> Paths {
    leaves(fs).into_iter().filter(|(_, at)| count(fs.names_of(*at))).collect()
}
/// A node as it is made now: its change and modification times are the clock's.
fn born(rng: &mut Rng, fs: &mut Fs, inode: u64) -> Node {
    let mut node = layers::node(rng, inode);
    node.ctime = fs.tick();
    node.mtime = node.ctime;
    node
}

fn write(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((_, at)) = pick(rng, &named(fs, |kind| kind == Kind::File)) else { return false };
    fs.node_mut(at).content = layers::bytes(rng, 200);
    fs.touched(at);
    true
}
fn chmod(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((_, at)) = pick(rng, &below_root(fs)) else { return false };
    let modes = [0o644, 0o755, 0o600, 0o4755, 0o1777, 0o000, 0o7777];
    fs.node_mut(at).mode = modes[usize::try_from(rng.below(7)).expect("small")];
    fs.changed(at);
    true
}
fn chown(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((_, at)) = pick(rng, &below_root(fs)) else { return false };
    let owner = [0, 1, 1000, 65_535, 33][usize::try_from(rng.below(5)).expect("small")];
    fs.node_mut(at).uid = owner;
    fs.node_mut(at).gid = owner ^ 1;
    fs.changed(at);
    true
}
/// An attribute removed, replaced, or set, of a family the walk keeps or one it drops.
fn attribute(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((_, at)) = pick(rng, &fs.paths()) else { return false };
    let node = fs.node_mut(at);
    let names = ["user.edit", "trusted.edit", "security.capability", "system.posix_acl_access", "user.other"];
    let name = names[usize::try_from(rng.below(5)).expect("small")].as_bytes().to_vec();
    if rng.chance(300) && !node.xattrs.is_empty() {
        let removed = usize::try_from(rng.below(node.xattrs.len() as u64)).expect("an index");
        node.xattrs.remove(removed);
    } else {
        let value = layers::bytes(rng, 20);
        match node.xattrs.iter_mut().find(|(found, _)| *found == name) {
            Some((_, old)) => *old = value,
            None => node.xattrs.push((name, value)),
        }
    }
    fs.changed(at);
    true
}
fn link(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((_, at)) = pick(rng, &leaves(fs)) else { return false };
    let Some((directory, parent)) = pick(rng, &directories(fs)) else { return false };
    let path = fresh(rng, fs, &directory);
    fs.link(&path, at);
    fs.changed(at);
    fs.touched(parent);
    true
}
fn unlink(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((path, _)) = pick(rng, &shared(fs, |names| names > 1)) else { return false };
    fs.remove(&path);
    true
}
fn delete_file(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((path, _)) = pick(rng, &leaves(fs)) else { return false };
    fs.remove(&path);
    true
}
fn delete_directory(rng: &mut Rng, fs: &mut Fs) -> bool {
    let below: Paths = directories(fs).into_iter().filter(|(path, _)| !path.is_empty()).collect();
    let Some((path, _)) = pick(rng, &below) else { return false };
    fs.remove(&path);
    true
}
fn kind_change(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((path, at)) = pick(rng, &below_root(fs)) else { return false };
    let old = fs.node(at).kind;
    fs.remove(&path);
    let inode = fs.last_inode() + rng.between(1, 4);
    let mut node = born(rng, fs, inode);
    while node.kind == old {
        node = born(rng, fs, inode);
    }
    fs.add(&path, node);
    true
}
fn reuse_inode(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((path, at)) = pick(rng, &shared(fs, |names| names == 1)) else { return false };
    let inode = fs.node(at).inode;
    fs.remove(&path);
    let mut node = Node::file(inode, &layers::bytes(rng, 100));
    node.ctime = fs.tick();
    node.mtime = node.ctime;
    fs.add(&path, node);
    true
}
fn touch(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((_, at)) = pick(rng, &fs.paths()) else { return false };
    fs.touched(at);
    true
}
fn add(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((directory, parent)) = pick(rng, &directories(fs)) else { return false };
    let path = fresh(rng, fs, &directory);
    let inode = fs.last_inode() + rng.between(1, 4);
    let node = born(rng, fs, inode);
    fs.add(&path, node);
    fs.touched(parent);
    true
}
fn rename(rng: &mut Rng, fs: &mut Fs) -> bool {
    let Some((from, at)) = pick(rng, &leaves(fs)) else { return false };
    let Some((directory, parent)) = pick(rng, &directories(fs)) else { return false };
    let to = fresh(rng, fs, &directory);
    fs.link(&to, at);
    fs.touched(parent);
    fs.remove(&from);
    fs.changed(at);
    true
}

/// Applies one edit at a path the layer has; false if it has none to edit that way.
pub fn apply(rng: &mut Rng, fs: &mut Fs, edit: Edit) -> bool {
    match edit {
        Edit::Write => write(rng, fs),
        Edit::Chmod => chmod(rng, fs),
        Edit::Chown => chown(rng, fs),
        Edit::Attribute => attribute(rng, fs),
        Edit::Link => link(rng, fs),
        Edit::Unlink => unlink(rng, fs),
        Edit::DeleteFile => delete_file(rng, fs),
        Edit::DeleteDirectory => delete_directory(rng, fs),
        Edit::KindChange => kind_change(rng, fs),
        Edit::ReuseInode => reuse_inode(rng, fs),
        Edit::Touch => touch(rng, fs),
        Edit::Add => add(rng, fs),
        Edit::Rename => rename(rng, fs),
    }
}

/// Between `least` and `most` edits of kinds drawn at random, and the kinds that were applied.
pub fn edit(rng: &mut Rng, fs: &mut Fs, least: u64, most: u64) -> Vec<Edit> {
    let mut done = Vec::new();
    for _ in 0..rng.between(least, most) {
        let edit = ALL[usize::try_from(rng.below(ALL.len() as u64)).expect("an index")];
        if apply(rng, fs, edit) {
            done.push(edit);
        }
    }
    done
}
