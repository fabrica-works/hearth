use super::{bytes, composition};
use crate::{limits, revision as r};

// Ruling 194: 524,281 zeros, plus two 32-byte map entries and two key bytes.
const NO_ZEROES: u32 = 0; // The empty a/b map has no list elements.
pub const DOCUMENT_ZEROES: u32 = 524_281;
pub const DOCUMENT_HELD_BYTES: u64 = 16_777_058;
pub const ADMITTED_DOCUMENTS: u32 = 4;
pub const OVER_BUDGET_DOCUMENTS: u32 = ADMITTED_DOCUMENTS + 1;
// The source fits one document: two bytes per zero, then the enclosing syntax.
pub const DOCUMENT_SOURCE_BYTES: u64 = 1_048_575;
pub const DOCUMENT_PATHS: &[&[u8]] = &[b"envs/a.json", b"envs/b.json", b"envs/c.json", b"envs/d.json", b"envs/e.json"];

pub fn list_document() -> r::Document {
    document_with_zeroes(DOCUMENT_ZEROES)
}
fn document_with_zeroes(zeroes: u32) -> r::Document {
    let at = composition::at(1, 1);
    let mut values = skein_lib::List::with_capacity(zeroes);
    for _ in 0..zeroes {
        values.push(composition::int(at, 0)).expect("zero count");
    }
    r::Document {
        at,
        members: None,
        layer: r::Layer {
            debian: None,
            placement: None,
            tasks: Box::new([r::Task {
                at,
                name: bytes::copy_of(b"a"),
                value: r::Value {
                    at,
                    kind: r::Kind::Map(Box::new([r::Pair {
                        at,
                        key: bytes::copy_of(b"b"),
                        value: r::Value { at, kind: r::Kind::List(values.into_boxed()) },
                    }])),
                },
            }]),
        },
    }
}
pub fn decoded_documents(count: u32) -> crate::call::Side {
    let mut entries = skein_lib::List::with_capacity(count);
    for path in DOCUMENT_PATHS.iter().take(usize::try_from(count).expect("fixture count")) {
        entries
            .push(r::Entry {
                path: bytes::copy_of(path),
                oid: r::Oid(bytes::copy_of(path)),
                mode: r::Mode::File,
                size: DOCUMENT_SOURCE_BYTES,
            })
            .expect("fixture count");
    }
    let mut call = crate::call::Side::new(bytes::copy_of(b"/repo"), Box::new([]), &limits::SHIPPED);
    let mut output = skein_lib::Queue::with_capacity(super::SINGLE_REQUEST);
    call.listed(limits::SHIPPED, r::Oid(bytes::copy_of(b"commit")), entries.as_slice(), &mut output);
    for entry in entries.as_slice() {
        call.decoded(&entry.oid, crate::vocabulary::Decoded::Document(list_document()));
    }
    call
}

pub fn decode_empty_documents(call: &mut crate::call::Side) {
    for blob in call.pending.clone() {
        call.decoded(&blob.oid, crate::vocabulary::Decoded::Document(document_with_zeroes(NO_ZEROES)));
    }
}
