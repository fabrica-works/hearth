//! Whole record frames, version 1 (store.md, 6.1 and 6.8).

use crate::{decode::Malformed, limits::Limits};
use alloc::boxed::Box;
use skein_lib::{Reader, Writer};

/// The frame's first eight bytes (store.md, 6.1).
pub const MAGIC: [u8; 8] = *b"hearth\0r";
/// The kinds of record M1 reads and writes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Store,
    Resolution,
    CacheFile,
}
impl Kind {
    #[must_use]
    pub const fn number(self) -> u16 {
        match self {
            Self::Store => 1,
            Self::Resolution => 4,
            Self::CacheFile => 5,
        }
    }
    /// The version this release reads and writes. Owner: hearth 09.1.
    #[must_use]
    pub const fn version(self) -> u16 {
        1
    }
    #[must_use]
    pub const fn bound(self, limits: &Limits) -> u32 {
        match self {
            Self::Store => limits.store.body,
            Self::Resolution => limits.resolution.body,
            Self::CacheFile => limits.cache_file.body,
        }
    }
}
/// Why a record is of a version the builder cannot read (store.md, 6.8).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unreadable {
    Short,
    Magic,
    Kind { found: u16 },
    Version { found: u16 },
    Reserved,
    Bound { length: u64 },
    Length { length: u64, after: u64 },
    Body(Malformed),
}
/// Why a body cannot be framed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refused {
    Bound { length: u64 },
}

/// Checks a frame in header order, checking the kind's bound before its body.
pub fn body<'a>(bytes: &'a [u8], kind: Kind, limits: &Limits) -> Result<&'a [u8], Unreadable> {
    if bytes.len() < 24 {
        return Err(Unreadable::Short);
    }
    let mut reader = Reader::new(bytes);
    if reader.bytes(8) != Some(MAGIC.as_slice()) {
        return Err(Unreadable::Magic);
    }
    let found = reader.u16().expect("the header has 24 bytes");
    if found != kind.number() {
        return Err(Unreadable::Kind { found });
    }
    let found = reader.u16().expect("the header has 24 bytes");
    if found != kind.version() {
        return Err(Unreadable::Version { found });
    }
    if reader.u32() != Some(0) {
        return Err(Unreadable::Reserved);
    }
    let length = reader.u64().expect("the header has 24 bytes");
    if length > u64::from(kind.bound(limits)) {
        return Err(Unreadable::Bound { length });
    }
    let after = u64::from(reader.remaining());
    if length != after {
        return Err(Unreadable::Length { length, after });
    }
    Ok(reader.bytes(reader.remaining()).expect("the remaining body fits"))
}
/// Frames a body at its kind's version.
pub fn write(kind: Kind, body: &[u8], limits: &Limits) -> Result<Box<[u8]>, Refused> {
    let length = u64::try_from(body.len()).expect("a body's size fits in u64");
    if length > u64::from(kind.bound(limits)) {
        return Err(Refused::Bound { length });
    }
    let size = body.len().checked_add(24).expect("a bounded frame fits in memory");
    let mut writer = Writer::new(size);
    writer.put(&MAGIC).expect("the header has room");
    writer.put(&kind.number().to_be_bytes()).expect("the header has room");
    writer.put(&kind.version().to_be_bytes()).expect("the header has room");
    writer.put(&0_u32.to_be_bytes()).expect("the header has room");
    writer.put(&length.to_be_bytes()).expect("the header has room");
    writer.put(body).expect("the body has room");
    Ok(writer.finish())
}
/// The most bytes a file of this kind may hold.
#[must_use]
pub fn largest(kind: Kind, limits: &Limits) -> u64 {
    u64::from(kind.bound(limits)).checked_add(24).expect("a u32 bound plus the header fits")
}
