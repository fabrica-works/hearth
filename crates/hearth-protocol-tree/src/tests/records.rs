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
    (Kind::Layer, 2, 64, Refused::Limit(b"depth")),
    (Kind::Layer, 8, 10, Refused::Limit(b"key")),
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
            Kind::Layer => layer::write(&a_layer(LayerKind::Packages), &limits),
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

use super::fixtures::{AT, LINE_AT, a_layer, digest_item, edit, layer_items};
use crate::{
    decode::Malformed,
    layer::{self, How, Layer, LayerKind, Line, Step, Tool},
    limits::LayerLimits,
};

const LAYER_KINDS: [LayerKind; 4] = [LayerKind::Empty, LayerKind::Packages, LayerKind::Software, LayerKind::Config];

#[test]
fn an_empty_layer_record_encodes_its_fields_in_key_order_byte_for_byte() {
    let expected = hex(concat!(
        "6865617274680072 0002 0001 00000000 000000000000028a ",
        "07 000000000000000c ",
        "04 0000000000000007 6368616e676573 05 0303030303030303030303030303030303030303030303030303030303030303 ",
        "04 0000000000000005 636f737473 07 0000000000000000 ",
        "04 0000000000000006 636f756e7473 07 0000000000000006 ",
        "04 0000000000000005 6279746573 03 00000000000003e8 ",
        "04 0000000000000007 64656c65746573 03 0000000000000002 ",
        "04 0000000000000007 656e7472696573 03 000000000000001e ",
        "04 0000000000000005 66696c6573 03 0000000000000014 ",
        "04 0000000000000006 686173686564 03 0000000000000384 ",
        "04 0000000000000004 70757473 03 0000000000000005 ",
        "04 0000000000000006 696e70757473 07 0000000000000000 ",
        "04 0000000000000003 6b6579 05 0101010101010101010101010101010101010101010101010101010101010101 ",
        "04 0000000000000004 6b696e64 04 0000000000000005 656d707479 ",
        "04 0000000000000004 6d616465 07 0000000000000002 ",
        "04 0000000000000002 6174 03 186cc6acd4b00000 ",
        "04 0000000000000002 6279 04 0000000000000005 6275696c74 ",
        "04 0000000000000006 706172656e74 00 ",
        "04 0000000000000009 737562766f6c756d65 07 0000000000000002 ",
        "04 000000000000000b 706172656e745f75756964 00 ",
        "04 0000000000000004 75756964 04 0000000000000010 11111111111111111111111111111111 ",
        "04 0000000000000005 746f6f6c73 06 0000000000000000 ",
        "04 0000000000000004 74726565 05 0202020202020202020202020202020202020202020202020202020202020202 ",
        "04 0000000000000009 747265655f66696c65 06 0000000000000002 ",
        "05 0404040404040404040404040404040404040404040404040404040404040404 ",
        "03 0000000000001000",
    ));
    assert_eq!(
        layer::write(&a_layer(LayerKind::Empty), &SHIPPED),
        Ok(expected),
        "store.md, 6.2's empty layer: parent none, no scan, no placed"
    );
}

#[test]
fn a_packages_layer_record_encodes_its_fields_in_key_order_byte_for_byte() {
    let expected = hex(concat!(
        "6865617274680072 0002 0001 00000000 0000000000000540 ",
        "07 000000000000000d ",
        "04 0000000000000007 6368616e676573 05 0303030303030303030303030303030303030303030303030303030303030303 ",
        "04 0000000000000005 636f737473 07 0000000000000002 ",
        "04 0000000000000005 6665746368 07 0000000000000006 ",
        "04 0000000000000003 637075 03 00000000000003e8 ",
        "04 0000000000000007 66657463686564 03 0000000000001388 ",
        "04 0000000000000006 686173686564 03 0000000000000000 ",
        "04 0000000000000006 6d656d6f7279 03 0000000000100000 ",
        "04 0000000000000004 77616c6c 03 00000000000007d0 ",
        "04 0000000000000007 7772697474656e 03 0000000000000000 ",
        "04 0000000000000004 77616c6b 07 0000000000000006 ",
        "04 0000000000000003 637075 03 0000000000000bb8 ",
        "04 0000000000000007 66657463686564 03 0000000000000000 ",
        "04 0000000000000006 686173686564 03 0000000000000384 ",
        "04 0000000000000006 6d656d6f7279 03 0000000000200000 ",
        "04 0000000000000004 77616c6c 03 0000000000000fa0 ",
        "04 0000000000000007 7772697474656e 03 0000000000000000 ",
        "04 0000000000000006 636f756e7473 07 0000000000000006 ",
        "04 0000000000000005 6279746573 03 00000000000003e8 ",
        "04 0000000000000007 64656c65746573 03 0000000000000002 ",
        "04 0000000000000007 656e7472696573 03 000000000000001e ",
        "04 0000000000000005 66696c6573 03 0000000000000014 ",
        "04 0000000000000006 686173686564 03 0000000000000384 ",
        "04 0000000000000004 70757473 03 0000000000000005 ",
        "04 0000000000000006 696e70757473 07 0000000000000002 ",
        "04 0000000000000004 70617274 05 0505050505050505050505050505050505050505050505050505050505050505 ",
        "04 000000000000000a 7265736f6c7574696f6e 05 0606060606060606060606060606060606060606060606060606060606060606 ",
        "04 0000000000000003 6b6579 05 0101010101010101010101010101010101010101010101010101010101010101 ",
        "04 0000000000000004 6b696e64 04 0000000000000008 7061636b61676573 ",
        "04 0000000000000004 6d616465 07 0000000000000006 ",
        "04 0000000000000002 6174 03 186cc6acd4b00000 ",
        "04 0000000000000004 62617365 00 ",
        "04 0000000000000002 6279 04 0000000000000005 6275696c74 ",
        "04 0000000000000005 6465707468 03 0000000000000000 ",
        "04 0000000000000003 686f77 04 0000000000000007 73637261746368 ",
        "04 0000000000000004 6c696e65 03 18493fba64ef0000 ",
        "04 0000000000000006 706172656e74 05 0808080808080808080808080808080808080808080808080808080808080808 ",
        "04 0000000000000004 7363616e 05 0707070707070707070707070707070707070707070707070707070707070707 ",
        "04 0000000000000009 737562766f6c756d65 07 0000000000000002 ",
        "04 000000000000000b 706172656e745f75756964 04 0000000000000010 22222222222222222222222222222222 ",
        "04 0000000000000004 75756964 04 0000000000000010 11111111111111111111111111111111 ",
        "04 0000000000000005 746f6f6c73 06 0000000000000002 ",
        "06 0000000000000002 ",
        "04 0000000000000003 617074 ",
        "04 0000000000000005 332e302e33 ",
        "06 0000000000000002 ",
        "04 000000000000000a 6d6d6465627374726170 ",
        "04 0000000000000005 312e352e37 ",
        "04 0000000000000004 74726565 05 0202020202020202020202020202020202020202020202020202020202020202 ",
        "04 0000000000000009 747265655f66696c65 06 0000000000000002 ",
        "05 0404040404040404040404040404040404040404040404040404040404040404 ",
        "03 0000000000001000",
    ));
    assert_eq!(
        layer::write(&a_layer(LayerKind::Packages), &SHIPPED),
        Ok(expected),
        "store.md, 6.2's packages layer: base none, steps and tools by name"
    );
}

#[test]
fn each_kind_of_layer_reads_back_as_it_was_written() {
    for kind in LAYER_KINDS {
        let record = a_layer(kind);
        let framed = layer::write(&record, &SHIPPED).expect("bounded record");
        assert_eq!(layer::read(&framed, &SHIPPED), Ok(record), "{kind:?}");
    }
    let mut record = a_layer(LayerKind::Packages);
    record.parent = None;
    record.subvolume.parent_uuid = None;
    record.made.packages =
        Some(Line { base: Some(Digest([11; 32])), depth: 3, how: How::Reconcile, line: Wall::from_nanos(7) });
    let framed = layer::write(&record, &SHIPPED).expect("bounded record");
    assert_eq!(layer::read(&framed, &SHIPPED), Ok(record), "a reconcile with a base and no parent");
}

#[test]
fn a_layer_frame_starts_with_kind_2s_header() {
    let framed = layer::write(&a_layer(LayerKind::Config), &SHIPPED).expect("bounded record");
    assert_eq!(framed.get(..16).expect("header"), hex("6865617274680072 0002 0001 00000000").as_ref(), "kind 2");
    assert_eq!(crate::frame::largest(Kind::Layer, &SHIPPED), 1_048_600, "load includes the header");
}

/// Reads a layer fixture after replacing each run `from` with `to`, in turn.
fn read_edited(kind: LayerKind, edits: &[(&[Item], &[Item])]) -> Result<Layer, Unreadable> {
    let mut items = layer_items(&a_layer(kind));
    for (from, to) in edits {
        items = edit(&items, from, to);
    }
    layer::read(&frame_of(Kind::Layer, &items), &SHIPPED)
}
fn uuid(length: usize, byte: u8) -> Item {
    let mut value = bytes::zeroed(length);
    value.fill(byte);
    Item::Bytes(value)
}

#[test]
fn a_layer_with_a_field_its_version_does_not_name_is_unreadable() {
    let edits: &[(&[Item], &[Item])] = &[
        (&[Item::Map(12)], &[Item::Map(13)]),
        (&[Item::Uint(4_096)], &[Item::Uint(4_096), text(b"zzz"), Item::Uint(1)]),
    ];
    assert_eq!(read_edited(LayerKind::Empty, edits), Err(Unreadable::Unknown), "a field of no name");
}

#[test]
fn a_layer_without_a_required_field_names_it_missing() {
    let cases: [(&[u8], Item, &[u8]); 4] = [
        (b"changes", digest_item(3), b"changes"),
        (b"key", digest_item(1), b"key"),
        (b"tree", digest_item(2), b"tree"),
        (b"parent", Item::None, b"parent"),
    ];
    for (name, value, missing) in cases {
        let edits: &[(&[Item], &[Item])] = &[(&[Item::Map(12)], &[Item::Map(11)]), (&[text(name), value], &[])];
        assert_eq!(read_edited(LayerKind::Empty, edits), Err(Unreadable::Missing(missing)), "{name:?}");
    }
    let edits: &[(&[Item], &[Item])] = &[(&[Item::Map(2), text(b"at"), Item::Uint(AT)], &[Item::Map(1)])];
    assert_eq!(read_edited(LayerKind::Empty, edits), Err(Unreadable::Missing(b"at")), "made.at");
    let edits: &[(&[Item], &[Item])] =
        &[(&[Item::Map(2), text(b"at")], &[Item::Map(1), text(b"at")]), (&[text(b"by"), text(b"built")], &[])];
    assert_eq!(read_edited(LayerKind::Empty, edits), Err(Unreadable::Missing(b"by")), "made.by");
    let edits: &[(&[Item], &[Item])] = &[(&[Item::Map(2), text(b"parent_uuid"), Item::None], &[Item::Map(1)])];
    assert_eq!(read_edited(LayerKind::Empty, edits), Err(Unreadable::Missing(b"parent_uuid")), "subvolume.parent_uuid");
}

#[test]
fn a_layer_value_of_another_kind_than_its_fields_is_unreadable() {
    let cases: [(&[u8], Item, Item); 4] = [
        (b"key", digest_item(1), Item::Uint(1)),
        (b"parent", Item::None, Item::Uint(1)),
        (b"tree", digest_item(2), Item::Bytes(bytes::zeroed(32))),
        (b"changes", digest_item(3), Item::None),
    ];
    for (name, value, other) in cases {
        let edits: &[(&[Item], &[Item])] = &[(&[text(name), value], &[text(name), other])];
        assert_eq!(read_edited(LayerKind::Empty, edits), Err(Unreadable::Shape(name)), "{name:?}");
    }
    let edits: &[(&[Item], &[Item])] = &[(&[text(b"empty")], &[text(b"other")])];
    assert_eq!(read_edited(LayerKind::Empty, edits), Err(Unreadable::Shape(b"kind")), "a kind of no name");
}

#[test]
fn a_layer_whose_optional_fields_do_not_agree_with_its_kind_is_unreadable() {
    let scan = || digest_item(7);
    let placed = || [digest_item(9), digest_item(10)];
    let [first, second] = placed();
    // placed in a config record
    let edits: &[(&[Item], &[Item])] =
        &[(&[Item::Map(13)], &[Item::Map(14)]), (&[text(b"scan")], &[text(b"placed"), Item::List(0), text(b"scan")])];
    assert_eq!(read_edited(LayerKind::Config, edits), Err(Unreadable::Shape(b"placed")), "placed in a config record");
    // no placed in a software record
    let edits: &[(&[Item], &[Item])] =
        &[(&[Item::Map(14)], &[Item::Map(13)]), (&[text(b"placed"), Item::List(2), first, second], &[])];
    assert_eq!(read_edited(LayerKind::Software, edits), Err(Unreadable::Missing(b"placed")), "software without placed");
    // scan in an empty record
    let edits: &[(&[Item], &[Item])] =
        &[(&[Item::Map(12)], &[Item::Map(13)]), (&[text(b"subvolume")], &[text(b"scan"), scan(), text(b"subvolume")])];
    assert_eq!(read_edited(LayerKind::Empty, edits), Err(Unreadable::Shape(b"scan")), "scan in an empty record");
    // no scan in a config record
    let edits: &[(&[Item], &[Item])] = &[(&[Item::Map(13)], &[Item::Map(12)]), (&[text(b"scan"), scan()], &[])];
    assert_eq!(read_edited(LayerKind::Config, edits), Err(Unreadable::Missing(b"scan")), "config without scan");
}

#[test]
fn a_layer_whose_inputs_do_not_agree_with_its_kind_is_unreadable() {
    let part = || [text(b"part"), digest_item(5)];
    let resolution = || [text(b"resolution"), digest_item(6)];
    let edits: &[(&[Item], &[Item])] =
        &[(&[Item::Map(0), text(b"key")], &[Item::Map(1), part()[0].clone(), part()[1].clone(), text(b"key")])];
    assert_eq!(read_edited(LayerKind::Empty, edits), Err(Unreadable::Shape(b"part")), "part in an empty layer");
    let edits: &[(&[Item], &[Item])] = &[(
        &[Item::Map(1), part()[0].clone(), part()[1].clone()],
        &[Item::Map(2), part()[0].clone(), part()[1].clone(), resolution()[0].clone(), resolution()[1].clone()],
    )];
    assert_eq!(
        read_edited(LayerKind::Config, edits),
        Err(Unreadable::Shape(b"resolution")),
        "resolution in a config layer"
    );
    assert_eq!(
        read_edited(LayerKind::Software, edits),
        Err(Unreadable::Shape(b"resolution")),
        "and in a software layer"
    );
    let edits: &[(&[Item], &[Item])] = &[(
        &[Item::Map(2), part()[0].clone(), part()[1].clone(), resolution()[0].clone(), resolution()[1].clone()],
        &[Item::Map(1), part()[0].clone(), part()[1].clone()],
    )];
    assert_eq!(
        read_edited(LayerKind::Packages, edits),
        Err(Unreadable::Missing(b"resolution")),
        "packages without resolution"
    );
    let edits: &[(&[Item], &[Item])] = &[(
        &[Item::Map(2), part()[0].clone(), part()[1].clone(), text(b"resolution")],
        &[Item::Map(1), text(b"resolution")],
    )];
    assert_eq!(read_edited(LayerKind::Packages, edits), Err(Unreadable::Missing(b"part")), "packages without part");
}

#[test]
fn a_layer_whose_made_does_not_agree_with_its_kind_is_unreadable() {
    let packages_fields = [
        Item::Map(6),
        text(b"at"),
        Item::Uint(AT),
        text(b"base"),
        Item::None,
        text(b"by"),
        text(b"built"),
        text(b"depth"),
        Item::Uint(0),
        text(b"how"),
        text(b"scratch"),
        text(b"line"),
        Item::Uint(LINE_AT),
    ];
    let edits: &[(&[Item], &[Item])] =
        &[(&packages_fields, &[Item::Map(2), text(b"at"), Item::Uint(AT), text(b"by"), text(b"built")])];
    assert_eq!(read_edited(LayerKind::Packages, edits), Err(Unreadable::Missing(b"base")), "packages without its line");
    let edits: &[(&[Item], &[Item])] = &[
        (&[Item::Map(2), text(b"at")], &[Item::Map(3), text(b"at")]),
        (&[text(b"built")], &[text(b"built"), text(b"how"), text(b"scratch")]),
    ];
    assert_eq!(read_edited(LayerKind::Config, edits), Err(Unreadable::Shape(b"how")), "how without packages");
    let edits: &[(&[Item], &[Item])] = &[
        (&[Item::Map(2), text(b"at")], &[Item::Map(3), text(b"at")]),
        (&[text(b"built")], &[text(b"built"), text(b"line"), Item::Uint(1)]),
    ];
    assert_eq!(read_edited(LayerKind::Software, edits), Err(Unreadable::Shape(b"line")), "line without packages");
    let edits: &[(&[Item], &[Item])] =
        &[(&[Item::Map(6), text(b"at")], &[Item::Map(5), text(b"at")]), (&[text(b"how"), text(b"scratch")], &[])];
    assert_eq!(read_edited(LayerKind::Packages, edits), Err(Unreadable::Missing(b"how")), "packages without how");
}

#[test]
fn a_layer_made_by_other_than_built_or_how_other_than_its_two_names_is_unreadable() {
    let edits: &[(&[Item], &[Item])] = &[(&[text(b"built")], &[text(b"received")])];
    assert_eq!(read_edited(LayerKind::Config, edits), Err(Unreadable::Shape(b"by")), "by received is M6's");
    let edits: &[(&[Item], &[Item])] = &[(&[text(b"scratch")], &[text(b"rebase")])];
    assert_eq!(read_edited(LayerKind::Packages, edits), Err(Unreadable::Shape(b"how")), "a how of no name");
    let edits: &[(&[Item], &[Item])] = &[(&[text(b"how"), text(b"scratch")], &[text(b"how"), Item::Uint(1)])];
    assert_eq!(read_edited(LayerKind::Packages, edits), Err(Unreadable::Shape(b"how")), "a how that is no bytes");
}

#[test]
fn a_uuid_not_of_16_bytes_is_unreadable() {
    let own = [uuid(16, 0x11)];
    for length in [0, 15, 17, 32] {
        let edits: &[(&[Item], &[Item])] = &[(&[text(b"uuid"), own[0].clone()], &[text(b"uuid"), uuid(length, 0x11)])];
        assert_eq!(read_edited(LayerKind::Empty, edits), Err(Unreadable::Shape(b"uuid")), "uuid of {length}");
        let edits: &[(&[Item], &[Item])] =
            &[(&[text(b"parent_uuid"), uuid(16, 0x22)], &[text(b"parent_uuid"), uuid(length, 0x22)])];
        assert_eq!(
            read_edited(LayerKind::Packages, edits),
            Err(Unreadable::Shape(b"parent_uuid")),
            "parent uuid of {length}"
        );
    }
    let edits: &[(&[Item], &[Item])] = &[(&[text(b"uuid"), own[0].clone()], &[text(b"uuid"), Item::None])];
    assert_eq!(read_edited(LayerKind::Empty, edits), Err(Unreadable::Shape(b"uuid")), "a uuid is never none");
}

fn limited(layer: LayerLimits) -> Limits {
    Limits { layer, ..SHIPPED }
}
const LAYER_READ_LIMITS: &[(LayerKind, &[u8], u32, Unreadable)] = &[
    (LayerKind::Software, b"placed", 1, Unreadable::Limit(b"placed")),
    (LayerKind::Packages, b"tools", 1, Unreadable::Limit(b"tools")),
    (LayerKind::Packages, b"steps", 1, Unreadable::Limit(b"costs")),
    (LayerKind::Packages, b"name", 9, Unreadable::Limit(b"name")),
    (LayerKind::Packages, b"name", 4, Unreadable::Limit(b"version")),
];
fn tighter(field: &[u8], to: u32) -> Limits {
    let layer = SHIPPED.layer;
    limited(match field {
        b"placed" => LayerLimits { placed: to, ..layer },
        b"tools" => LayerLimits { tools: to, ..layer },
        b"steps" => LayerLimits { steps: to, ..layer },
        b"name" => LayerLimits { name: to, ..layer },
        _ => unreachable!("a known limit"),
    })
}

#[test]
fn a_layer_list_or_name_past_its_limit_is_unreadable() {
    for (kind, field, to, expected) in LAYER_READ_LIMITS {
        let framed = layer::write(&a_layer(*kind), &SHIPPED).expect("bounded record");
        assert_eq!(layer::read(&framed, &tighter(field, *to)), Err(*expected), "{field:?} past {to}");
        let at_limit = layer::read(&framed, &tighter(field, to.saturating_add(10)));
        assert!(at_limit.is_ok(), "{field:?} within {to} plus ten reads");
    }
    let framed = layer::write(&a_layer(LayerKind::Empty), &SHIPPED).expect("bounded record");
    let small = limited(LayerLimits { body: 649, ..SHIPPED.layer });
    assert_eq!(layer::read(&framed, &small), Err(Unreadable::Bound { length: 650 }), "a body past kind 2's bound");
}

#[test]
fn a_step_name_past_the_decoders_key_bound_is_unreadable() {
    let edits: &[(&[Item], &[Item])] = &[(&[text(b"fetch")], &[Item::Bytes(bytes::zeroed(65))])];
    assert_eq!(read_edited(LayerKind::Packages, edits), Err(Unreadable::Limit(b"key")), "a map key of 65 bytes");
}

#[test]
fn tools_not_strictly_increasing_by_name_are_unreadable() {
    let edits: &[(&[Item], &[Item])] = &[(&[text(b"apt")], &[text(b"zzz")])];
    assert_eq!(read_edited(LayerKind::Packages, edits), Err(Unreadable::Order(b"tools")), "out of order");
    let edits: &[(&[Item], &[Item])] = &[(&[text(b"mmdebstrap")], &[text(b"apt")])];
    assert_eq!(read_edited(LayerKind::Packages, edits), Err(Unreadable::Order(b"tools")), "duplicated");
}

#[test]
fn steps_not_strictly_increasing_by_name_are_unreadable_as_map_keys() {
    let mut framed = layer::write(&a_layer(LayerKind::Packages), &SHIPPED).expect("bounded record");
    let mut at = None;
    for (offset, window) in framed.windows(4).enumerate() {
        if window == b"walk" {
            at = Some(offset);
        }
    }
    let at = at.expect("the step's name is in the frame");
    framed.get_mut(at..at.checked_add(4).expect("small")).expect("the name").fill(b'a');
    assert_eq!(layer::read(&framed, &SHIPPED), Err(Unreadable::Body(Malformed::Order)), "aaaa after fetch");
}

#[test]
fn writing_a_layer_past_a_limit_is_refused() {
    for (kind, field, to, _) in LAYER_READ_LIMITS {
        let expected = match *field {
            b"placed" => Refused::Limit(b"placed"),
            b"tools" => Refused::Limit(b"tools"),
            b"steps" => Refused::Limit(b"costs"),
            _ if *to == 9 => Refused::Limit(b"name"),
            _ => Refused::Limit(b"version"),
        };
        assert_eq!(layer::write(&a_layer(*kind), &tighter(field, *to)), Err(expected), "{field:?} past {to}");
    }
    let mut record = a_layer(LayerKind::Packages);
    record.costs = Box::from([Step { name: bytes::zeroed(65), cost: a_layer(LayerKind::Packages).costs[0].cost }]);
    assert_eq!(layer::write(&record, &SHIPPED), Err(Refused::Limit(b"key")), "a step name past the key bound");
    let small = limited(LayerLimits { body: 649, ..SHIPPED.layer });
    assert_eq!(
        layer::write(&a_layer(LayerKind::Empty), &small),
        Err(Refused::Bound { length: 650 }),
        "a body past kind 2's bound"
    );
}

#[test]
fn writing_tools_or_steps_out_of_order_or_twice_is_refused() {
    let tool = |name: &[u8]| Tool { name: bytes::copy_of(name), version: bytes::copy_of(b"1") };
    for names in [[b"b".as_slice(), b"a"], [b"a", b"a"]] {
        let mut record = a_layer(LayerKind::Packages);
        record.tools = Box::from([tool(names[0]), tool(names[1])]);
        assert_eq!(layer::write(&record, &SHIPPED), Err(Refused::Order(b"tools")), "tools {names:?}");
        let cost = a_layer(LayerKind::Packages).costs[0].cost;
        record = a_layer(LayerKind::Packages);
        record.costs =
            Box::from([Step { name: bytes::copy_of(names[0]), cost }, Step { name: bytes::copy_of(names[1]), cost }]);
        assert_eq!(layer::write(&record, &SHIPPED), Err(Refused::Order(b"costs")), "steps {names:?}");
    }
}

#[test]
fn writing_a_layer_whose_kind_and_optional_fields_disagree_is_refused() {
    let mut config = a_layer(LayerKind::Config);
    config.inputs = a_layer(LayerKind::Software).inputs;
    assert_eq!(layer::write(&config, &SHIPPED), Err(Refused::Limit(b"inputs")), "software inputs, config kind");
    let mut config = a_layer(LayerKind::Config);
    config.placed = Some(Box::from([]));
    assert_eq!(layer::write(&config, &SHIPPED), Err(Refused::Limit(b"placed")), "placed beside config");
    let mut software = a_layer(LayerKind::Software);
    software.placed = None;
    assert_eq!(layer::write(&software, &SHIPPED), Err(Refused::Limit(b"placed")), "software without placed");
    let mut empty = a_layer(LayerKind::Empty);
    empty.scan = Some(Digest([7; 32]));
    assert_eq!(layer::write(&empty, &SHIPPED), Err(Refused::Limit(b"scan")), "scan beside empty");
    let mut config = a_layer(LayerKind::Config);
    config.scan = None;
    assert_eq!(layer::write(&config, &SHIPPED), Err(Refused::Limit(b"scan")), "config without scan");
    let mut packages = a_layer(LayerKind::Packages);
    packages.made.packages = None;
    assert_eq!(layer::write(&packages, &SHIPPED), Err(Refused::Limit(b"made")), "packages without its line");
    let mut config = a_layer(LayerKind::Config);
    config.made.packages = a_layer(LayerKind::Packages).made.packages;
    assert_eq!(layer::write(&config, &SHIPPED), Err(Refused::Limit(b"made")), "a line beside config");
}
