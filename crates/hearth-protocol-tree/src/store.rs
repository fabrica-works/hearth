//! Store records, kind 1 at version 1 (domain/store.md, 6.6).

use crate::{
    fields::Fields,
    frame::{self, Kind, Refused, Unreadable},
    limits::Limits,
};
use alloc::boxed::Box;
use hearth_lib::encode::Encoder;
use skein_lib::Wall;

/// A store's identity and creation time, with its optional subordinate range.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Store {
    pub layout: u64,
    pub made: Wall,
    pub range: Option<Range>,
    pub uid: u32,
}
/// The subordinate range; absent in a store without layers (domain/store.md, 2.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Range {
    pub base: u32,
    pub count: u32,
}

pub fn read(bytes: &[u8], limits: &Limits) -> Result<Store, Unreadable> {
    let mut fields = Fields::new(frame::body(bytes, Kind::Store, limits)?, limits);
    let (mut layout, mut made, mut range, mut uid) = (None, None, None, None);
    for _ in 0..fields.map(b"")? {
        match fields.key()?.as_ref() {
            b"layout" => layout = Some(fields.uint(b"layout")?),
            b"made" => made = Some(fields.wall(b"made")?),
            b"range" => range = Some(read_range(&mut fields)?),
            b"uid" => uid = Some(fields.id(b"uid")?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    fields.finish()?;
    Ok(Store {
        layout: layout.ok_or(Unreadable::Missing(b"layout"))?,
        made: made.ok_or(Unreadable::Missing(b"made"))?,
        range,
        uid: uid.ok_or(Unreadable::Missing(b"uid"))?,
    })
}
fn read_range(fields: &mut Fields<'_>) -> Result<Range, Unreadable> {
    let (mut base, mut count) = (None, None);
    for _ in 0..fields.map(b"range")? {
        match fields.key()?.as_ref() {
            b"base" => base = Some(fields.id(b"base")?),
            b"count" => count = Some(fields.id(b"count")?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    Ok(Range { base: base.ok_or(Unreadable::Missing(b"base"))?, count: count.ok_or(Unreadable::Missing(b"count"))? })
}
pub fn write(record: &Store, limits: &Limits) -> Result<Box<[u8]>, Refused> {
    if limits.key < 6 {
        return Err(Refused::Limit(b"key"));
    }
    if limits.depth < if record.range.is_some() { 2 } else { 1 } {
        return Err(Refused::Limit(b"depth"));
    }
    let mut measure = Encoder::measuring();
    encode(record, &mut measure);
    let mut writer = Encoder::writing(measure.measured());
    encode(record, &mut writer);
    frame::write(Kind::Store, &writer.written(), limits)
}
fn encode(record: &Store, out: &mut Encoder) {
    out.map(if record.range.is_some() { 4 } else { 3 });
    out.key(b"layout");
    out.uint(record.layout);
    out.key(b"made");
    out.uint(record.made.as_nanos());
    if let Some(range) = record.range {
        out.key(b"range");
        out.map(2);
        out.key(b"base");
        out.uint(u64::from(range.base));
        out.key(b"count");
        out.uint(u64::from(range.count));
    }
    out.key(b"uid");
    out.uint(u64::from(record.uid));
}
