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

use super::fixtures::{a_resolution, resolution_items};
use crate::{
    limits::{Limits, ResolutionLimits},
    resolution,
};

const SMALL: Limits = Limits { resolution: ResolutionLimits { packages: 4, ..SHIPPED.resolution }, ..SHIPPED };

#[test]
fn a_resolution_reads_back_as_it_was_written_inputs_included() {
    let record = a_resolution(2);
    let framed = resolution::write(&record, &SHIPPED).expect("bounded record");
    assert_eq!(resolution::read(&framed, &SHIPPED), Ok(record), "the key's inputs return item for item");
}

const PACKAGE_LIMIT: &[(&[u8], u32, Unreadable)] = &[(b"closure", 5, Unreadable::Limit(b"closure"))];
#[test]
fn a_closure_past_its_package_limit_is_unreadable() {
    for (name, packages, expected) in PACKAGE_LIMIT {
        let record = a_resolution(*packages);
        assert_eq!(
            resolution::read(&frame_of(Kind::Resolution, &resolution_items(&record)), &SMALL),
            Err(*expected),
            "{name:?}"
        );
    }
}
const WRITE_PACKAGE_LIMIT: &[(u32, Refused)] = &[(5, Refused::Limit(b"closure"))];
#[test]
fn writing_a_closure_past_its_package_limit_is_refused() {
    for (packages, expected) in WRITE_PACKAGE_LIMIT {
        assert_eq!(resolution::write(&a_resolution(*packages), &SMALL), Err(*expected), "closure of {packages}");
    }
}

type ClosureOrder = (&'static [u8], &'static [u8], &'static [u8], &'static [u8], Unreadable);
const CLOSURE_ORDER: &[ClosureOrder] = &[
    (b"b", b"amd64", b"a", b"amd64", Unreadable::Order(b"closure")),
    (b"a", b"arm64", b"a", b"amd64", Unreadable::Order(b"closure")),
    (b"a", b"amd64", b"a", b"amd64", Unreadable::Order(b"closure")),
];
#[test]
fn a_closure_out_of_name_then_architecture_order_is_unreadable() {
    for (first, architecture, second, next_architecture, expected) in CLOSURE_ORDER {
        let mut record = a_resolution(2);
        let first_package = record.closure.get_mut(0).expect("first");
        first_package.name = bytes::copy_of(first);
        first_package.architecture = bytes::copy_of(architecture);
        let second_package = record.closure.get_mut(1).expect("second");
        second_package.name = bytes::copy_of(second);
        second_package.architecture = bytes::copy_of(next_architecture);
        let framed = frame_of(Kind::Resolution, &resolution_items(&record));
        assert_eq!(
            resolution::read(&framed, &SHIPPED),
            Err(*expected),
            "{first:?}/{architecture:?} then {second:?}/{next_architecture:?}"
        );
    }
}

const BAD_SOURCE: &[(u32, Unreadable)] = &[(1, Unreadable::Limit(b"source")), (u32::MAX, Unreadable::Limit(b"source"))];
#[test]
fn a_package_whose_source_names_no_line_is_unreadable() {
    for (source, expected) in BAD_SOURCE {
        let mut record = a_resolution(2);
        record.closure.get_mut(0).expect("first").source = *source;
        assert_eq!(
            resolution::read(&frame_of(Kind::Resolution, &resolution_items(&record)), &SHIPPED),
            Err(*expected),
            "source {source}"
        );
    }
}

const FIELD_LIMITS: &[(&[u8], Unreadable, Refused)] = &[
    (b"sources", Unreadable::Limit(b"sources"), Refused::Limit(b"sources")),
    (b"indices", Unreadable::Limit(b"indices"), Refused::Limit(b"indices")),
    (b"path", Unreadable::Limit(b"path"), Refused::Limit(b"path")),
    (b"filename", Unreadable::Limit(b"filename"), Refused::Limit(b"filename")),
    (b"uri", Unreadable::Limit(b"uri"), Refused::Limit(b"uri")),
    (b"name", Unreadable::Limit(b"name"), Refused::Limit(b"name")),
    (b"version", Unreadable::Limit(b"version"), Refused::Limit(b"version")),
    (b"architecture", Unreadable::Limit(b"architecture"), Refused::Limit(b"architecture")),
    (b"suite", Unreadable::Limit(b"suite"), Refused::Limit(b"suite")),
    (b"component", Unreadable::Limit(b"component"), Refused::Limit(b"component")),
];
#[test]
fn a_resolution_field_one_past_its_limit_is_unreadable() {
    for (field, expected, _) in FIELD_LIMITS {
        let mut record = a_resolution(2);
        super::fixtures::past_limit(&mut record, field);
        assert_eq!(
            resolution::read(&frame_of(Kind::Resolution, &resolution_items(&record)), &SHIPPED),
            Err(*expected),
            "field {field:?}"
        );
    }
}
#[test]
fn writing_a_resolution_field_one_past_its_limit_is_refused() {
    for (field, _, expected) in FIELD_LIMITS {
        let mut record = a_resolution(2);
        super::fixtures::past_limit(&mut record, field);
        assert_eq!(resolution::write(&record, &SHIPPED), Err(*expected), "field {field:?}");
    }
}

const REQUEST_ORDER: &[(&[u8], &[u8], Unreadable)] =
    &[(b"b", b"a", Unreadable::Order(b"request")), (b"a", b"a", Unreadable::Order(b"request"))];
#[test]
fn a_request_that_is_not_a_set_is_unreadable() {
    for (first, second, expected) in REQUEST_ORDER {
        let mut record = a_resolution(2);
        record.inputs.request = Box::from([Item::List(2), text(first), text(second)]);
        assert_eq!(
            resolution::read(&frame_of(Kind::Resolution, &resolution_items(&record)), &SHIPPED),
            Err(*expected),
            "request {first:?}, {second:?}"
        );
    }
}
const WRONG_INPUT: &[(&[u8], Unreadable)] = &[(b"variant", Unreadable::Shape(b"variant"))];
#[test]
fn an_input_of_another_kind_than_its_own_is_unreadable() {
    for (name, expected) in WRONG_INPUT {
        let mut record = a_resolution(2);
        record.inputs.variant = Box::from([Item::Uint(1)]);
        assert_eq!(
            resolution::read(&frame_of(Kind::Resolution, &resolution_items(&record)), &SHIPPED),
            Err(*expected),
            "input {name:?}"
        );
    }
}
const RESOLUTION_HEADER: &str = "6865617274680072 0004 0001 00000000";
#[test]
fn a_resolution_frame_starts_with_kind_4s_header() {
    let framed = resolution::write(&a_resolution(2), &SHIPPED).expect("bounded record");
    assert_eq!(framed.get(..16).expect("header"), hex(RESOLUTION_HEADER).as_ref(), "kind 4 version 1");
    let mut reader = skein_lib::Reader::new(framed.get(16..).expect("length and body"));
    let length = reader.u64().expect("body length");
    assert_eq!(length, u64::from(reader.remaining()), "the length names the bytes after the header");
}

const WRITE_BOUNDS: &[(Kind, u32, u32, Refused)] = &[
    (Kind::Store, 0, 64, Refused::Limit(b"depth")),
    (Kind::Store, 8, 5, Refused::Limit(b"key")),
    (Kind::CacheFile, 1, 64, Refused::Limit(b"depth")),
    (Kind::CacheFile, 8, 5, Refused::Limit(b"key")),
    (Kind::Resolution, 4, 64, Refused::Limit(b"depth")),
    (Kind::Resolution, 8, 9, Refused::Limit(b"key")),
];
#[test]
fn writing_a_record_past_the_decoders_bounds_is_refused() {
    for (kind, depth, key, expected) in WRITE_BOUNDS {
        let limits = Limits { depth: *depth, key: *key, ..SHIPPED };
        let result = match kind {
            Kind::Store => {
                store::write(&Store { layout: 1, made: Wall::from_nanos(42), range: None, uid: 1000 }, &limits)
            }
            Kind::CacheFile => cache_file::write(&a_cache_file(), &limits),
            Kind::Resolution => resolution::write(&a_resolution(2), &limits),
        };
        assert_eq!(result, Err(*expected), "{kind:?}, depth {depth}, key {key}");
    }
}
const BAD_INPUTS: &[(&[Item], Refused)] = &[
    (&[Item::Map(1)], Refused::Limit(b"apt")),
    (&[Item::Map(1), Item::Uint(1), Item::None], Refused::Limit(b"apt")),
    (&[Item::Map(0), Item::Uint(1)], Refused::Limit(b"apt")),
];
#[test]
fn writing_an_input_that_is_not_one_value_is_refused() {
    for (items, expected) in BAD_INPUTS {
        let mut record = a_resolution(2);
        let mut apt = skein_lib::List::with_capacity(3);
        for item in *items {
            apt.push(item.clone()).expect("fixture room");
        }
        record.inputs.apt = apt.into_boxed();
        assert_eq!(resolution::write(&record, &SHIPPED), Err(*expected), "items {items:?}");
    }
}
