//! Cache files' provenance records, kind 5 at version 1 (domain/store.md, 6.5).

use crate::{
    fields::Fields,
    frame::{self, Kind, Refused, Unreadable},
    limits::Limits,
};
use alloc::boxed::Box;
use hearth_lib::encode::{Digest, Encoder};
use skein_lib::{Duration, Wall};

/// Where a cache file first came from, with its time and cost.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CacheFile {
    pub at: Wall,
    pub origin: Origin,
    pub sha256: Digest,
    pub size: u64,
    pub took: Duration,
}
/// A map of the variant's fields and `kind`, its name (domain/store.md, 3.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Origin {
    Fetch { uri: Box<[u8]> },
    Index { resolution: Digest },
    Blob { oid: Box<[u8]> },
    Rendered { path: Box<[u8]> },
}
pub fn read(bytes: &[u8], limits: &Limits) -> Result<CacheFile, Unreadable> {
    let mut fields = Fields::new(frame::body(bytes, Kind::CacheFile, limits)?, limits);
    let (mut at, mut origin, mut sha256, mut size, mut took) = (None, None, None, None, None);
    for _ in 0..fields.map(b"")? {
        match fields.key()?.as_ref() {
            b"at" => at = Some(fields.wall(b"at")?),
            b"origin" => origin = Some(read_origin(&mut fields, limits)?),
            b"sha256" => sha256 = Some(fields.digest(b"sha256")?),
            b"size" => size = Some(fields.uint(b"size")?),
            b"took" => took = Some(fields.duration(b"took")?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    fields.finish()?;
    Ok(CacheFile {
        at: at.ok_or(Unreadable::Missing(b"at"))?,
        origin: origin.ok_or(Unreadable::Missing(b"origin"))?,
        sha256: sha256.ok_or(Unreadable::Missing(b"sha256"))?,
        size: size.ok_or(Unreadable::Missing(b"size"))?,
        took: took.ok_or(Unreadable::Missing(b"took"))?,
    })
}
fn read_origin(fields: &mut Fields<'_>, limits: &Limits) -> Result<Origin, Unreadable> {
    let count = fields.map(b"origin")?;
    if count == 0 {
        return Err(Unreadable::Missing(b"kind"));
    }
    if fields.key()?.as_ref() != b"kind" {
        return Err(Unreadable::Unknown);
    }
    let kind = fields.bytes(b"origin", u32::MAX)?;
    let (key, name, limit) = match kind.as_ref() {
        b"fetch" => (b"uri".as_slice(), b"uri".as_slice(), limits.cache_file.path),
        b"index" => (b"resolution".as_slice(), b"resolution".as_slice(), 0),
        b"blob" => (b"oid".as_slice(), b"oid".as_slice(), limits.cache_file.oid),
        b"rendered" => (b"path".as_slice(), b"path".as_slice(), limits.cache_file.path),
        _ => return Err(Unreadable::Shape(b"origin")),
    };
    if count == 1 {
        return Err(Unreadable::Missing(name));
    }
    if fields.key()?.as_ref() != key {
        return Err(Unreadable::Unknown);
    }
    let origin = match kind.as_ref() {
        b"fetch" => Origin::Fetch { uri: fields.bytes(name, limit)? },
        b"index" => Origin::Index { resolution: fields.digest(name)? },
        b"blob" => Origin::Blob { oid: fields.bytes(name, limit)? },
        b"rendered" => Origin::Rendered { path: fields.bytes(name, limit)? },
        _ => unreachable!("the kind was checked before its field"),
    };
    if count != 2 {
        return Err(Unreadable::Unknown);
    }
    Ok(origin)
}
pub fn write(record: &CacheFile, limits: &Limits) -> Result<Box<[u8]>, Refused> {
    if limits.depth < 2 {
        return Err(Refused::Limit(b"depth"));
    }
    let key = match &record.origin {
        Origin::Index { .. } => 10,
        Origin::Fetch { .. } | Origin::Blob { .. } | Origin::Rendered { .. } => 6,
    };
    if limits.key < key {
        return Err(Refused::Limit(b"key"));
    }
    let (value, limit, name) = match &record.origin {
        Origin::Fetch { uri } => (uri.as_ref(), limits.cache_file.path, b"uri".as_slice()),
        Origin::Index { .. } => (b"".as_slice(), 0, b"resolution".as_slice()),
        Origin::Blob { oid } => (oid.as_ref(), limits.cache_file.oid, b"oid".as_slice()),
        Origin::Rendered { path } => (path.as_ref(), limits.cache_file.path, b"path".as_slice()),
    };
    if value.len() > usize::try_from(limit).expect("a u32 fits in memory") {
        return Err(Refused::Limit(name));
    }
    let mut measure = Encoder::measuring();
    encode(record, &mut measure);
    let mut writer = Encoder::writing(measure.measured());
    encode(record, &mut writer);
    frame::write(Kind::CacheFile, &writer.written(), limits)
}
fn encode(record: &CacheFile, out: &mut Encoder) {
    out.map(5);
    out.key(b"at");
    out.uint(record.at.as_nanos());
    out.key(b"origin");
    out.map(2);
    out.key(b"kind");
    match &record.origin {
        Origin::Fetch { uri } => {
            out.bytes(b"fetch");
            out.key(b"uri");
            out.bytes(uri);
        }
        Origin::Index { resolution } => {
            out.bytes(b"index");
            out.key(b"resolution");
            out.digest(resolution);
        }
        Origin::Blob { oid } => {
            out.bytes(b"blob");
            out.key(b"oid");
            out.bytes(oid);
        }
        Origin::Rendered { path } => {
            out.bytes(b"rendered");
            out.key(b"path");
            out.bytes(path);
        }
    }
    out.key(b"sha256");
    out.digest(&record.sha256);
    out.key(b"size");
    out.uint(record.size);
    out.key(b"took");
    out.uint(record.took.as_nanos());
}
