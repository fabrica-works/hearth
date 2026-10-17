use super::fixtures::{SHIPPED, a_cache_file, cache_items, frame_of, hex, text};
use crate::{
    cache_file::{self, Origin},
    frame::{Kind, Refused, Unreadable},
    store::{self, Range, Store},
};
use hearth_lib::encode::{Digest, Item};
use skein_lib::{Wall, bytes};

#[test]
fn a_store_record_encodes_its_fields_in_key_order_byte_for_byte() {
    let record = Store { layout: 1, made: Wall::from_nanos(1_760_000_000_000_000_000), range: None, uid: 1000 };
    let expected = hex(concat!(
        "6865617274680072 0001 0001 00000000 000000000000004c 07 0000000000000003 ",
        "04 0000000000000006 6c61796f7574 03 0000000000000001 ",
        "04 0000000000000004 6d616465 03 186cc6acd4b00000 ",
        "04 0000000000000003 756964 03 00000000000003e8",
    ));
    assert_eq!(store::write(&record, &SHIPPED), Ok(expected), "store.md, 6.6's fields in byte order");
}

#[test]
fn a_cache_file_record_encodes_its_fields_in_key_order_byte_for_byte() {
    let expected = hex(concat!(
        "6865617274680072 0005 0001 00000000 00000000000000e0 ",
        "07 0000000000000005 ",
        "04 0000000000000002 6174 03 186cc6acd4b00000 ",
        "04 0000000000000006 6f726967696e 07 0000000000000002 ",
        "04 0000000000000004 6b696e64 04 0000000000000008 72656e6465726564 ",
        "04 0000000000000004 70617468 04 000000000000001b 7461736b732f636f72652f747265652f6574632f6d6f74642e6a32 ",
        "04 0000000000000006 736861323536 05 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 ",
        "04 0000000000000004 73697a65 03 0000000000000000 ",
        "04 0000000000000004 746f6f6b 03 000000000016e360",
    ));
    assert_eq!(cache_file::write(&a_cache_file(), &SHIPPED), Ok(expected), "store.md, 6.5's body, worked out from 3.1");
}

#[test]
fn a_store_record_with_a_range_reads_back_with_it() {
    let record =
        Store { layout: 7, made: Wall::from_nanos(42), range: Some(Range { base: 100_000, count: 65_536 }), uid: 1000 };
    let bytes = store::write(&record, &SHIPPED).expect("bounded record");
    assert_eq!(store::read(&bytes, &SHIPPED), Ok(record), "M1 preserves a future store's range and layout");
}

#[test]
fn each_origin_of_a_cache_file_reads_back_as_its_variant() {
    let origins = [
        Origin::Fetch { uri: bytes::copy_of(b"https://example.invalid/p") },
        Origin::Index { resolution: Digest([7; 32]) },
        Origin::Blob { oid: bytes::copy_of(b"abcdef") },
        a_cache_file().origin,
    ];
    for origin in origins {
        let mut record = a_cache_file();
        record.origin = origin;
        let bytes = cache_file::write(&record, &SHIPPED).expect("bounded record");
        assert_eq!(cache_file::read(&bytes, &SHIPPED), Ok(record.clone()), "origin {:?}", record.origin);
    }
}

#[test]
fn a_body_that_is_one_value_but_no_map_is_no_record() {
    for item in [Item::List(0), Item::Uint(1)] {
        let body = [item];
        assert_eq!(store::read(&frame_of(Kind::Store, &body), &SHIPPED), Err(Unreadable::Record), "store root");
        assert_eq!(
            cache_file::read(&frame_of(Kind::CacheFile, &body), &SHIPPED),
            Err(Unreadable::Record),
            "cache root"
        );
    }
}

#[test]
fn an_origin_of_another_kind_is_unreadable() {
    let items = cache_items(&[Item::Map(2), text(b"kind"), text(b"copied"), text(b"path"), text(b"a")], true);
    assert_eq!(
        cache_file::read(&frame_of(Kind::CacheFile, &items), &SHIPPED),
        Err(Unreadable::Shape(b"origin")),
        "unknown origin"
    );
}

#[test]
fn a_record_without_a_required_field_names_it_missing() {
    let items = cache_items(&[Item::Map(2), text(b"kind"), text(b"blob"), text(b"oid"), text(b"abc")], false);
    assert_eq!(
        cache_file::read(&frame_of(Kind::CacheFile, &items), &SHIPPED),
        Err(Unreadable::Missing(b"took")),
        "missing cost"
    );
}

#[test]
fn a_field_its_version_does_not_name_makes_it_unreadable() {
    let items = [Item::Map(1), text(b"unknown"), Item::Uint(1)];
    assert_eq!(
        store::read(&frame_of(Kind::Store, &items), &SHIPPED),
        Err(Unreadable::Unknown),
        "version 1 knows no field of this name"
    );
}

#[test]
fn a_uri_past_4096_bytes_is_unreadable() {
    let items = cache_items(
        &[Item::Map(2), text(b"kind"), text(b"fetch"), text(b"uri"), Item::Bytes(bytes::zeroed(4_097))],
        true,
    );
    assert_eq!(
        cache_file::read(&frame_of(Kind::CacheFile, &items), &SHIPPED),
        Err(Unreadable::Limit(b"uri")),
        "URI checked before copying"
    );
}

#[test]
fn writing_a_cache_file_past_a_limit_is_refused() {
    let mut record = a_cache_file();
    record.origin = Origin::Rendered { path: bytes::zeroed(4_097) };
    assert_eq!(cache_file::write(&record, &SHIPPED), Err(Refused::Limit(b"path")), "writer checks its path limit");
}
