//! Shapes known by name and kind alone (store.md, 2.1 and 4.4).
use crate::builder::store::vocabulary::{Directory, Removal, Shape};
use crate::render::vocabulary::Digest;
use alloc::boxed::Box;
use skein_io::kernel::Kind;
use skein_lib::Writer;
#[must_use]
pub fn shape(directory: Directory, name: &[u8], kind: Kind) -> Shape {
    if directory == Directory::Top {
        if let b"store.record" | b"cache" | b"resolutions" | b"staging" | b"layers" | b"targets" | b"state"
        | b"artifacts" | b"sources" | b"caches" | b"repo" | b"credentials" = name
        {
            return Shape::Fixed;
        }
        return if kind == Kind::File && temporary(name, b"store.record") { Shape::Temporary } else { Shape::Stray };
    }
    if kind != Kind::File {
        return Shape::Stray;
    }
    let Some(prefix) = name.get(..64) else {
        return Shape::Stray;
    };
    let Some(digest) = digest(prefix) else {
        return Shape::Stray;
    };
    match name.get(64..) {
        Some(b"") if directory == Directory::Cache => Shape::File(digest),
        Some(b".record") => Shape::Record(digest),
        Some(b".tmp") if directory == Directory::Cache => Shape::Temporary,
        Some(suffix) if temporary(suffix, b".record") => Shape::Temporary,
        Some(_) | None => Shape::Stray,
    }
}
fn temporary(name: &[u8], prefix: &[u8]) -> bool {
    let Some(tail) = name.strip_prefix(prefix) else {
        return false;
    };
    let Some(tail) = tail.strip_prefix(b".skein-") else {
        return false;
    };
    if tail.len() != 16 {
        return false;
    }
    for byte in tail {
        if digit(*byte).is_none() {
            return false;
        }
    }
    true
}
fn digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte.saturating_sub(b'0')),
        b'a'..=b'f' => Some(byte.saturating_sub(b'a').saturating_add(10)),
        _ => None,
    }
}
fn digest(name: &[u8]) -> Option<Digest> {
    if name.len() != 64 {
        return None;
    }
    let mut bytes = [0; 32];
    for (byte, pair) in bytes.iter_mut().zip(name.chunks_exact(2)) {
        let [high, low] = pair else {
            return None;
        };
        *byte = digit(*high)?.checked_mul(16)?.checked_add(digit(*low)?)?;
    }
    Some(Digest(bytes))
}
pub(super) fn removal(name: Removal) -> Box<[u8]> {
    let (digest, suffix): (Digest, &[u8]) = match name {
        Removal::Named(name) => return name,
        Removal::File(digest) => (digest, b""),
        Removal::Record(digest) => (digest, b".record"),
    };
    let mut writer = Writer::new(64_usize.checked_add(suffix.len()).expect("digest and suffix"));
    for byte in digest.0 {
        for digit in [byte >> 4_u8, byte & 15] {
            writer
                .put(&[if digit < 10 {
                    b'0'.saturating_add(digit)
                } else {
                    b'a'.saturating_add(digit.saturating_sub(10))
                }])
                .expect("hex room");
        }
    }
    writer.put(suffix).expect("suffix room");
    writer.finish()
}
