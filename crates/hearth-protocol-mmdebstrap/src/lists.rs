//! Apt's list names for the source lines (render.md, 5.3; store.md, 6.4).
use crate::vector::{Component, Line, Name};
use alloc::boxed::Box;
use skein_lib::{List, Writer};
/// Apt's list directory relative to the scratch.
pub const LISTS: &[u8] = b"root/var/lib/apt/lists";
/// One index, with its archive path relative to dists/suite and apt's name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Index {
    pub line: u8,
    pub component: Option<Component>,
    pub path: Box<[u8]>,
    pub name: Box<[u8]>,
}
/// An expected index, a refused compressed copy, or a skipped directory entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Listed {
    Index(u32),
    Compressed(u32),
    Other,
}
fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded index length");
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).expect("summed index length");
    }
    out.finish()
}
/// `InRelease` first, then each component's Packages, for every source line.
#[must_use]
pub fn indices(lines: &[Line], architecture: &Name) -> Box<[Index]> {
    let mut capacity = 0_u32;
    for line in lines {
        capacity = capacity.checked_add(1).expect("bounded index count");
        capacity = capacity
            .checked_add(u32::try_from(line.components.len()).expect("component count fits u32"))
            .expect("bounded index count");
    }
    let mut out = List::with_capacity(capacity);
    for (at, line) in lines.iter().enumerate() {
        let line_number = u8::try_from(at).expect("source line count fits u8");
        let prefix =
            join(&[line.uri.strip_prefix(b"http://").expect("source URI is HTTP"), b"dists/", &line.suite, b"/"]);
        for entry in 0..=line.components.len() {
            let component = if entry == 0 {
                None
            } else {
                line.components.get(entry.checked_sub(1).expect("nonzero entry")).copied()
            };
            let path = match component {
                None => join(&[b"InRelease"]),
                Some(component) => join(&[component.bytes(), b"/binary-", architecture.as_bytes(), b"/Packages"]),
            };
            let mut name = join(&[&prefix, &path]);
            for byte in &mut name {
                if *byte == b'/' {
                    *byte = b'_';
                }
            }
            out.push(Index { line: line_number, component, path, name }).expect("summed index count");
        }
    }
    out.into_boxed()
}
/// Recognize compressed copies without opening or decompressing them.
#[must_use]
pub fn listed(name: &[u8], indices: &[Index]) -> Listed {
    for (at, index) in indices.iter().enumerate() {
        let at = u32::try_from(at).expect("index count fits u32");
        if name == &*index.name {
            return Listed::Index(at);
        }
        if index.component.is_some()
            && let Some(suffix) = name.strip_prefix(&*index.name)
        {
            match suffix {
                b".xz" | b".gz" | b".bz2" | b".lzma" | b".lz4" | b".zst" => return Listed::Compressed(at),
                _ => {}
            }
        }
    }
    Listed::Other
}
