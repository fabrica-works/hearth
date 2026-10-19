//! A tree entry's path and tree order (domain/store.md, 3.4), shared by the tree file's
//! codec and the walk, which merges by it.

use core::cmp::Ordering;

/// The most bytes in a path (domain/store.md, 3.4).
pub const PATH: u32 = 4_096;

/// Whether `path` is canonical: the root's is empty; otherwise components of
/// at least a byte, none `.` or `..`, joined by `/`, with no NUL, in at most
/// 4096 bytes.
#[must_use]
pub fn path_valid(path: &[u8]) -> bool {
    if path.len() > usize::try_from(PATH).expect("a u32 fits in memory") {
        return false;
    }
    if path.is_empty() {
        return true;
    }
    let mut start = 0_usize;
    for (at, &byte) in path.iter().enumerate() {
        if byte == 0 {
            return false;
        }
        if byte == b'/' {
            if !component_valid(path.get(start..at)) {
                return false;
            }
            start = at.saturating_add(1);
        }
    }
    component_valid(path.get(start..))
}
fn component_valid(component: Option<&[u8]>) -> bool {
    match component {
        Some(name) => !name.is_empty() && name != b"." && name != b"..",
        None => false,
    }
}

/// Compares paths component by component, each bytewise: a directory comes
/// before everything beneath it, and its subtree before its next sibling
/// (`a`, `a/b`, `a-b`). The root's empty path has no component and comes
/// first. It allocates nothing, and totally orders any bytes, valid paths or
/// not, by splitting them at `/`.
#[must_use]
pub fn tree_order(a: &[u8], b: &[u8]) -> Ordering {
    let steps = a.len().saturating_add(b.len());
    let (mut a, mut b) = (rest(a), rest(b));
    // Each step takes a byte of each path, its `/` or its end.
    for _ in 0..=steps {
        match (a, b) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(left), Some(right)) => {
                let (first, after) = component(left);
                let (second, behind) = component(right);
                match first.cmp(second) {
                    Ordering::Equal => (a, b) = (after, behind),
                    other @ (Ordering::Less | Ordering::Greater) => return other,
                }
            }
        }
    }
    unreachable!("each step takes a byte of each path")
}
/// What remains to compare: nothing for the root's empty path.
const fn rest(path: &[u8]) -> Option<&[u8]> {
    if path.is_empty() { None } else { Some(path) }
}
/// The first component, and what follows its `/`, if anything does.
fn component(path: &[u8]) -> (&[u8], Option<&[u8]>) {
    for (at, &byte) in path.iter().enumerate() {
        if byte == b'/' {
            let (name, slash) = path.split_at_checked(at).expect("a position within the path");
            return (name, slash.get(1..));
        }
    }
    (path, None)
}
