use alloc::boxed::Box;
use skein_lib::bytes;

pub use crate::limits::SHIPPED;
use crate::{
    decode::{Decoder, Malformed},
    limits::Limits,
};
use hearth_lib::encode::Item;
use skein_lib::List;

pub fn hex(text: &str) -> Box<[u8]> {
    let mut digits = 0_usize;
    for byte in text.bytes() {
        if byte != b' ' {
            digits = digits.checked_add(1).expect("the fixture fits in memory");
        }
    }
    assert_eq!(digits % 2, 0, "hex pairs are whole");
    let mut result = bytes::zeroed(digits / 2);
    let mut offset = 0_usize;
    let mut high = None;
    for byte in text.bytes() {
        if byte == b' ' {
            continue;
        }
        let digit =
            u8::try_from(char::from(byte).to_digit(16).expect("a hex digit")).expect("a hex digit fits in a byte");
        if let Some(first) = high {
            let value: u8 = first;
            *result.get_mut(offset).expect("a pair has room") =
                value.checked_mul(16).expect("a high digit fits").checked_add(digit).expect("a byte fits");
            offset = offset.checked_add(1).expect("the fixture fits in memory");
            high = None;
        } else {
            high = Some(digit);
        }
    }
    result
}

pub fn items(body: &[u8], limits: &Limits) -> Result<Box<[Item]>, Malformed> {
    let mut decoder = Decoder::new(body, limits);
    let capacity = u32::try_from(body.len()).expect("a fixture fits");
    let mut result = List::with_capacity(capacity);
    for _ in 0..=capacity {
        match decoder.item(u32::MAX)? {
            Some(item) => result.push(item).expect("each item uses at least one byte"),
            None => return Ok(result.into_boxed()),
        }
    }
    unreachable!("an item uses at least one byte")
}

pub fn frame_of(kind: crate::frame::Kind, items: &[Item]) -> Box<[u8]> {
    let mut measure = hearth_lib::encode::Encoder::measuring();
    for item in items {
        measure.item(item);
    }
    let mut writer = hearth_lib::encode::Encoder::writing(measure.measured());
    for item in items {
        writer.item(item);
    }
    crate::frame::write(kind, &writer.written(), &SHIPPED).expect("fixture fits its frame")
}
pub fn text(value: &[u8]) -> Item {
    Item::Bytes(bytes::copy_of(value))
}
pub fn a_cache_file() -> crate::cache_file::CacheFile {
    crate::cache_file::CacheFile {
        at: skein_lib::Wall::from_nanos(1_760_000_000_000_000_000),
        origin: crate::cache_file::Origin::Rendered { path: bytes::copy_of(b"tasks/core/tree/etc/motd.j2") },
        sha256: hearth_lib::encode::Digest(
            hex("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
                .as_ref()
                .try_into()
                .expect("a digest"),
        ),
        size: 0,
        took: skein_lib::Duration::from_nanos(1_500_000),
    }
}
pub fn cache_items(origin: &[Item], took: bool) -> Box<[Item]> {
    let mut result = List::with_capacity(32);
    let record = a_cache_file();
    for item in [Item::Map(if took { 5 } else { 4 }), text(b"at"), Item::Uint(record.at.as_nanos()), text(b"origin")] {
        result.push(item).expect("fixture room");
    }
    for item in origin {
        result.push(item.clone()).expect("fixture room");
    }
    for item in [text(b"sha256"), Item::Digest(record.sha256), text(b"size"), Item::Uint(0)] {
        result.push(item).expect("fixture room");
    }
    if took {
        for item in [text(b"took"), Item::Uint(record.took.as_nanos())] {
            result.push(item).expect("fixture room");
        }
    }
    result.into_boxed()
}
