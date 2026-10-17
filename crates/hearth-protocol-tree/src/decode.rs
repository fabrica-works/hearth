//! A pull decoder of the canonical value's pre-order items (store.md, 3.1).

use crate::limits::Limits;
use alloc::boxed::Box;
use hearth_lib::encode::{self, Digest, Item};
use skein_lib::{Reader, Stack, bytes};

/// A shape the encoder asserts, or a cut in the input.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Malformed {
    Truncated,
    Trailing,
    Tag(u8),
    Deep,
    Key,
    Order,
    Long,
}
/// One body's value, its open containers on a bounded stack.
#[derive(Debug)]
pub struct Decoder<'a> {
    reader: Reader<'a>,
    open: Stack<Open>,
    key: u32,
    whole: bool,
}
#[derive(Debug)]
enum Open {
    List { left: u64 },
    Map { left: u64, last: Option<Box<[u8]>>, awaiting: bool },
}
impl<'a> Decoder<'a> {
    #[must_use]
    pub fn new(body: &'a [u8], limits: &Limits) -> Decoder<'a> {
        Decoder { reader: Reader::new(body), open: Stack::with_capacity(limits.depth), key: limits.key, whole: false }
    }
    /// Returns one item, or `None` once the whole value ends with the body.
    pub fn item(&mut self, longest: u32) -> Result<Option<Item>, Malformed> {
        if self.whole && self.open.is_empty() {
            return if self.reader.is_empty() { Ok(None) } else { Err(Malformed::Trailing) };
        }
        let tag = self.reader.u8().ok_or(Malformed::Truncated)?;
        let key = match self.open.top() {
            Some(Open::Map { awaiting: false, .. }) => true,
            Some(Open::Map { awaiting: true, .. } | Open::List { .. }) | None => false,
        };
        if key && tag != encode::BYTES {
            return Err(Malformed::Key);
        }
        let item = match tag {
            encode::NONE => Item::None,
            encode::FALSE => Item::False,
            encode::TRUE => Item::True,
            encode::UINT => Item::Uint(self.reader.u64().ok_or(Malformed::Truncated)?),
            encode::BYTES => {
                let len = self.reader.u64().ok_or(Malformed::Truncated)?;
                if len > u64::from(if key { self.key } else { longest }) {
                    return Err(Malformed::Long);
                }
                let len = u32::try_from(len).expect("a byte string is within its u32 limit");
                let value = self.reader.bytes(len).ok_or(Malformed::Truncated)?;
                if key {
                    match self.open.top_mut() {
                        Some(Open::Map { last, awaiting, .. }) => {
                            if let Some(previous) = last
                                && previous.as_ref() >= value
                            {
                                return Err(Malformed::Order);
                            }
                            *last = Some(bytes::copy_of(value));
                            *awaiting = true;
                        }
                        Some(Open::List { .. }) | None => unreachable!("the key belongs to the top map"),
                    }
                }
                Item::Bytes(bytes::copy_of(value))
            }
            encode::DIGEST => {
                let value = self.reader.bytes(32).ok_or(Malformed::Truncated)?;
                Item::Digest(Digest(value.try_into().expect("a digest has 32 bytes")))
            }
            encode::LIST | encode::MAP => {
                let count = self.reader.u64().ok_or(Malformed::Truncated)?;
                let slots = if tag == encode::MAP { count.checked_mul(2).ok_or(Malformed::Truncated)? } else { count };
                if slots > u64::from(self.reader.remaining()) {
                    return Err(Malformed::Truncated);
                }
                if tag == encode::MAP { Item::Map(count) } else { Item::List(count) }
            }
            other => return Err(Malformed::Tag(other)),
        };
        if !key {
            self.place();
        }
        let open = match &item {
            Item::List(count) => Some(Open::List { left: *count }),
            Item::Map(count) => Some(Open::Map { left: *count, last: None, awaiting: false }),
            Item::None | Item::False | Item::True | Item::Uint(_) | Item::Bytes(_) | Item::Digest(_) => None,
        };
        if let Some(open) = open
            && self.open.push(open).is_err()
        {
            return Err(Malformed::Deep);
        }
        self.settle();
        Ok(Some(item))
    }
    fn place(&mut self) {
        match self.open.top_mut() {
            Some(Open::List { left }) => *left = left.checked_sub(1).expect("a finished list closes"),
            Some(Open::Map { left, awaiting, .. }) => {
                *left = left.checked_sub(1).expect("a finished map closes");
                *awaiting = false;
            }
            None => self.whole = true,
        }
    }
    fn settle(&mut self) {
        for _ in 0..self.open.capacity() {
            match self.open.top() {
                Some(Open::List { left: 0 } | Open::Map { left: 0, awaiting: false, .. }) => drop(self.open.pop()),
                Some(Open::List { .. } | Open::Map { .. }) | None => return,
            }
        }
    }
}
