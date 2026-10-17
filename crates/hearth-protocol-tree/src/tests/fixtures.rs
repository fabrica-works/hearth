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

pub fn a_resolution(packages: u32) -> crate::resolution::Resolution {
    use crate::resolution::{Costs, Index, Inputs, Line, Made, Package, Resolution};
    use hearth_lib::encode::Digest;
    use skein_lib::{Decimal, Duration, Wall, Writer};
    let mut closure = List::with_capacity(packages);
    for number in 0..packages {
        let decimal = Decimal::of(u64::from(number));
        let mut name = Writer::new(6);
        name.put(b"p").expect("prefix");
        for _ in decimal.as_bytes().len()..5 {
            name.put(b"0").expect("padding");
        }
        name.put(decimal.as_bytes()).expect("five digits");
        closure
            .push(Package {
                name: name.finish(),
                architecture: bytes::copy_of(b"amd64"),
                version: bytes::copy_of(b"1.0"),
                source: 0,
                filename: bytes::copy_of(b"pool/p.deb"),
                size: 42,
                sha256: Digest([3; 32]),
            })
            .expect("package room");
    }
    Resolution {
        closure: closure.into_boxed(),
        digest: Digest([4; 32]),
        key: Digest([5; 32]),
        inputs: Inputs {
            apt: Box::from([Item::Map(1), text(b"install_recommends"), Item::False]),
            request: Box::from([Item::List(1), text(b"p00000")]),
            sources: Box::from([
                Item::Map(2),
                text(b"components"),
                Item::List(1),
                text(b"main"),
                text(b"suite"),
                text(b"forky"),
            ]),
            variant: Box::from([text(b"apt")]),
        },
        made: Made {
            apt: bytes::copy_of(b""),
            at: Wall::from_nanos(42),
            costs: Costs { fetched: 1_234, took: Duration::from_nanos(1_500_000) },
            mmdebstrap: bytes::copy_of(b""),
        },
        sources: Box::from([Line {
            uri: bytes::copy_of(b"https://snapshot.debian.org/archive/debian/20261001T030405Z/"),
            suite: bytes::copy_of(b"forky"),
            components: Box::from([bytes::copy_of(b"main")]),
            indices: Box::from([
                Index { path: bytes::copy_of(b"InRelease"), size: 100, sha256: Digest([1; 32]) },
                Index { path: bytes::copy_of(b"main/binary-amd64/Packages"), size: 1_000, sha256: Digest([2; 32]) },
            ]),
        }]),
    }
}
pub fn resolution_items(record: &crate::resolution::Resolution) -> Box<[Item]> {
    let mut measure = hearth_lib::encode::Encoder::measuring();
    crate::resolution::encode(record, &mut measure);
    let mut writer = hearth_lib::encode::Encoder::writing(measure.measured());
    crate::resolution::encode(record, &mut writer);
    items(&writer.written(), &SHIPPED).expect("a fixture is a canonical value")
}

pub fn past_limit(record: &mut crate::resolution::Resolution, field: &[u8]) {
    let line = record.sources.get_mut(0).expect("one line");
    match field {
        b"sources" => {
            let mut lines = List::with_capacity(9);
            for _ in 0..9_u32 {
                lines.push(line.clone()).expect("nine lines");
            }
            record.sources = lines.into_boxed();
        }
        b"indices" => {
            let mut indices = List::with_capacity(17);
            for _ in 0..17_u32 {
                indices.push(line.indices.first().expect("one index").clone()).expect("17 indices");
            }
            line.indices = indices.into_boxed();
        }
        b"path" => line.indices.get_mut(0).expect("one index").path = bytes::zeroed(4_097),
        b"filename" => record.closure.get_mut(0).expect("one package").filename = bytes::zeroed(4_097),
        b"uri" => line.uri = bytes::zeroed(4_097),
        b"name" => record.closure.get_mut(1).expect("second package").name = bytes::zeroed(257),
        b"version" => record.closure.get_mut(0).expect("one package").version = bytes::zeroed(257),
        b"architecture" => record.closure.get_mut(0).expect("one package").architecture = bytes::zeroed(257),
        b"suite" => line.suite = bytes::zeroed(257),
        b"component" => *line.components.get_mut(0).expect("one component") = bytes::zeroed(257),
        _ => unreachable!("a known field"),
    }
}
