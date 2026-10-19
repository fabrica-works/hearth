use super::fixtures::{SHIPPED, hex, items};
use crate::{
    decode::{Decoder, Malformed},
    frame::{self, Kind, Refused, Unreadable},
};
use alloc::boxed::Box;
use hearth_lib::encode::Item;
use skein_lib::{List, bytes};

#[test]
fn a_framed_body_has_the_header_store_md_gives_byte_for_byte() {
    let body = hex("07 0000000000000001 04 0000000000000001 61 03 0000000000000001");
    let framed = frame::write(Kind::Store, &body, &SHIPPED).expect("28 bytes are within 4 KiB");
    let expected = hex(concat!(
        "6865617274680072 0001 0001 00000000 000000000000001c ",
        "07 0000000000000001 04 0000000000000001 61 03 0000000000000001"
    ));
    assert_eq!(framed, expected, "domain/store.md, 6.1's header around 3.2's map");
    assert_eq!(frame::body(&framed, Kind::Store, &SHIPPED), Ok(body.as_ref()), "the frame returns its body");
}

#[test]
fn store_md_3_2s_vectors_decode_to_their_items() {
    let rows = [
        (
            "06 0000000000000002 04 0000000000000008 6865617274682f31 04 0000000000000005 656d707479",
            Box::from([Item::List(2), Item::Bytes(bytes::copy_of(b"hearth/1")), Item::Bytes(bytes::copy_of(b"empty"))]),
        ),
        (
            "07 0000000000000001 04 0000000000000001 61 03 0000000000000001",
            Box::from([Item::Map(1), Item::Bytes(bytes::copy_of(b"a")), Item::Uint(1)]),
        ),
    ];
    for (body, expected) in rows {
        assert_eq!(items(&hex(body), &SHIPPED), Ok(expected), "vector {body}");
    }
}

#[test]
fn a_length_past_the_kinds_bound_is_refused_before_the_body() {
    let header = hex("6865617274680072 0001 0001 00000000 0000000000001001");
    assert_eq!(
        frame::body(&header, Kind::Store, &SHIPPED),
        Err(Unreadable::Bound { length: 4_097 }),
        "bound precedes body length"
    );
}

const BAD_HEADERS: &[(&str, Unreadable)] = &[
    ("6865617274680072 0001 0001 00000000 00000000000000", Unreadable::Short),
    ("0065617274680072 0001 0001 00000000 0000000000000000", Unreadable::Magic),
    ("6865617274680072 0005 0001 00000000 0000000000000000", Unreadable::Kind { found: 5 }),
    ("6865617274680072 0001 0002 00000000 0000000000000000", Unreadable::Version { found: 2 }),
    ("6865617274680072 0001 0001 00000001 0000000000000000", Unreadable::Reserved),
    ("6865617274680072 0001 0001 00000000 0000000000000002 00", Unreadable::Length { length: 2, after: 1 }),
    ("6865617274680072 0001 0001 00000000 0000000000000000 00", Unreadable::Length { length: 0, after: 1 }),
];
#[test]
fn a_header_unlike_store_md_6_1s_makes_the_frame_unreadable() {
    for (header, error) in BAD_HEADERS {
        assert_eq!(frame::body(&hex(header), Kind::Store, &SHIPPED), Err(*error), "header {header}");
    }
}

#[test]
fn a_body_past_the_kinds_bound_is_refused_when_framed() {
    let body = bytes::zeroed(4_097);
    assert_eq!(
        frame::write(Kind::Store, &body, &SHIPPED),
        Err(Refused::Bound { length: 4_097 }),
        "writer checks the bound"
    );
    assert_eq!(frame::largest(Kind::Resolution, &SHIPPED), 4_194_328, "load includes the header");
}

const MALFORMED: &[(&str, Malformed)] = &[
    ("08", Malformed::Tag(8)),
    ("07 0000000000000001 03 0000000000000001 00", Malformed::Key),
    ("07 0000000000000002 04 0000000000000001 61 00 04 0000000000000001 61 00", Malformed::Order),
    ("07 0000000000000002 04 0000000000000001 62 00 04 0000000000000001 61 00", Malformed::Order),
    ("00 00", Malformed::Trailing),
    ("06 ffffffffffffffff", Malformed::Truncated),
    ("03 00000000000000", Malformed::Truncated),
];
#[test]
fn bytes_that_are_not_one_value_of_store_md_3_1_are_malformed() {
    for (body, error) in MALFORMED {
        assert_eq!(items(&hex(body), &SHIPPED), Err(*error), "body {body}");
    }
}

#[test]
fn nesting_one_past_the_depth_limit_is_too_deep() {
    let limits = crate::limits::Limits { depth: 2, ..SHIPPED }; // Three nested lists prove the configured bound.
    assert_eq!(
        items(&hex("06 0000000000000001 06 0000000000000001 06 0000000000000000"), &limits),
        Err(Malformed::Deep),
        "a third container passes depth two"
    );
}

#[test]
fn a_bytes_item_past_its_longest_is_long() {
    let body = hex("04 0000000000000002 6162");
    assert_eq!(Decoder::new(&body, &SHIPPED).item(1), Err(Malformed::Long), "value length is checked before copying");
    let limits = crate::limits::Limits { key: 1, ..SHIPPED }; // A two-byte key proves keys have their own bound.
    assert_eq!(
        items(&hex("07 0000000000000001 04 0000000000000002 6162 00"), &limits),
        Err(Malformed::Long),
        "keys use their configured bound"
    );
}

#[test]
fn scalars_and_empty_containers_decode_as_their_own_kinds() {
    let rows = [
        ("00", Item::None),
        ("01", Item::False),
        ("02", Item::True),
        ("03 ffffffffffffffff", Item::Uint(u64::MAX)),
        ("04 0000000000000000", Item::Bytes(bytes::copy_of(b""))),
        (
            "05 0000000000000000000000000000000000000000000000000000000000000000",
            Item::Digest(hearth_lib::encode::Digest([0; 32])),
        ),
        ("06 0000000000000000", Item::List(0)),
        ("07 0000000000000000", Item::Map(0)),
    ];
    for (body, expected) in rows {
        let mut list = List::with_capacity(1);
        list.push(expected).expect("one item");
        assert_eq!(items(&hex(body), &SHIPPED), Ok(list.into_boxed()), "body {body}");
    }
}
