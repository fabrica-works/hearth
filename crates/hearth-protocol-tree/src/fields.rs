//! A record reader's fields in canonical key order (store.md, 6.1).

use crate::{
    decode::{Decoder, Malformed},
    frame::Unreadable,
    limits::Limits,
};
use alloc::boxed::Box;
use hearth_lib::encode::{Digest, Item};
use skein_lib::{Duration, List, Wall};

pub(crate) struct Fields<'a> {
    decoder: Decoder<'a>,
}
impl<'a> Fields<'a> {
    pub(crate) fn new(body: &'a [u8], limits: &Limits) -> Fields<'a> {
        Fields { decoder: Decoder::new(body, limits) }
    }
    fn next(&mut self, field: &'static [u8], longest: u32) -> Result<Item, Unreadable> {
        match self.decoder.item(longest) {
            Ok(Some(item)) => Ok(item),
            Ok(None) => Err(Unreadable::Missing(field)),
            Err(Malformed::Long) => Err(Unreadable::Limit(field)),
            Err(error) => Err(Unreadable::Body(error)),
        }
    }
    pub(crate) fn map(&mut self, field: &'static [u8]) -> Result<u64, Unreadable> {
        match self.next(field, u32::MAX)? {
            Item::Map(count) => Ok(count),
            Item::None
            | Item::False
            | Item::True
            | Item::Uint(_)
            | Item::Bytes(_)
            | Item::Digest(_)
            | Item::List(_) => {
                if field.is_empty() {
                    Err(Unreadable::Record)
                } else {
                    Err(Unreadable::Shape(field))
                }
            }
        }
    }
    pub(crate) fn key(&mut self) -> Result<Box<[u8]>, Unreadable> {
        self.bytes(b"key", u32::MAX)
    }
    pub(crate) fn uint(&mut self, field: &'static [u8]) -> Result<u64, Unreadable> {
        match self.next(field, u32::MAX)? {
            Item::Uint(value) => Ok(value),
            Item::None | Item::False | Item::True | Item::Map(_) | Item::Bytes(_) | Item::Digest(_) | Item::List(_) => {
                Err(Unreadable::Shape(field))
            }
        }
    }
    pub(crate) fn id(&mut self, field: &'static [u8]) -> Result<u32, Unreadable> {
        match u32::try_from(self.uint(field)?) {
            Ok(value) => Ok(value),
            Err(_) => Err(Unreadable::Limit(field)),
        }
    }
    pub(crate) fn bytes(&mut self, field: &'static [u8], limit: u32) -> Result<Box<[u8]>, Unreadable> {
        match self.next(field, limit)? {
            Item::Bytes(value) => Ok(value),
            Item::None | Item::False | Item::True | Item::Uint(_) | Item::Map(_) | Item::Digest(_) | Item::List(_) => {
                Err(Unreadable::Shape(field))
            }
        }
    }
    pub(crate) fn digest(&mut self, field: &'static [u8]) -> Result<Digest, Unreadable> {
        match self.next(field, u32::MAX)? {
            Item::Digest(value) => Ok(value),
            Item::None | Item::False | Item::True | Item::Uint(_) | Item::Bytes(_) | Item::Map(_) | Item::List(_) => {
                Err(Unreadable::Shape(field))
            }
        }
    }
    pub(crate) fn list(&mut self, field: &'static [u8], limit: u32) -> Result<u32, Unreadable> {
        match self.next(field, u32::MAX)? {
            Item::List(count) => {
                if count > u64::from(limit) {
                    return Err(Unreadable::Limit(field));
                }
                Ok(u32::try_from(count).expect("the count is within a u32 limit"))
            }
            Item::None | Item::False | Item::True | Item::Uint(_) | Item::Bytes(_) | Item::Digest(_) | Item::Map(_) => {
                Err(Unreadable::Shape(field))
            }
        }
    }
    pub(crate) fn tuple(&mut self, field: &'static [u8], count: u32) -> Result<(), Unreadable> {
        if self.list(field, count)? != count {
            return Err(Unreadable::Shape(field));
        }
        Ok(())
    }
    pub(crate) fn value(&mut self, field: &'static [u8], limit: u32, longest: u32) -> Result<Box<[Item]>, Unreadable> {
        let mut values = List::with_capacity(limit);
        let mut owed = 1_u64;
        for _ in 0..limit {
            let item = self.next(field, longest)?;
            owed = owed.checked_sub(1).expect("one item is owed");
            let children = match &item {
                Item::List(count) => *count,
                Item::Map(count) => count.checked_mul(2).ok_or(Unreadable::Limit(field))?,
                Item::None | Item::False | Item::True | Item::Uint(_) | Item::Bytes(_) | Item::Digest(_) => 0,
            };
            owed = owed.checked_add(children).ok_or(Unreadable::Limit(field))?;
            if owed > u64::from(values.room().saturating_sub(1)) {
                return Err(Unreadable::Limit(field));
            }
            values.push(item).expect("the checked item has room");
            if owed == 0 {
                return Ok(values.into_boxed());
            }
        }
        Err(Unreadable::Limit(field))
    }
    pub(crate) fn wall(&mut self, field: &'static [u8]) -> Result<Wall, Unreadable> {
        Ok(Wall::from_nanos(self.uint(field)?))
    }
    pub(crate) fn duration(&mut self, field: &'static [u8]) -> Result<Duration, Unreadable> {
        Ok(Duration::from_nanos(self.uint(field)?))
    }
    pub(crate) fn finish(&mut self) -> Result<(), Unreadable> {
        match self.decoder.item(u32::MAX) {
            Ok(None) => Ok(()),
            Ok(Some(_)) => Err(Unreadable::Record),
            Err(error) => Err(Unreadable::Body(error)),
        }
    }
}
