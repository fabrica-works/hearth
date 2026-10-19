use super::{bytes, composition};
use crate::{limits, revision as r};

// Rulings 194 and 204: 131,065 zeros, plus two 32-byte map entries and two key bytes.
const NO_ZEROES: u32 = 0; // The empty a/b map has no list elements.
pub const DOCUMENT_ZEROES: u32 = 131_065;
pub const DOCUMENT_HELD_BYTES: u64 = 4_194_146;
pub const ADMITTED_DOCUMENTS: u32 = 4;
pub const OVER_BUDGET_DOCUMENTS: u32 = ADMITTED_DOCUMENTS + 1;
// The source fits one document: two bytes per zero, then the enclosing syntax.
pub const DOCUMENT_SOURCE_BYTES: u64 = 262_143;
/// The shipped template input allowance (domain/render.md, 2.3).
pub const TEMPLATE_BYTES: u64 = 262_144;
pub const DOCUMENT_PATHS: &[&[u8]] = &[b"envs/a.json", b"envs/b.json", b"envs/c.json", b"envs/d.json", b"envs/e.json"];

pub fn empty_document() -> r::Document {
    document_with_zeroes(NO_ZEROES)
}
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

pub const DECODED_TOO_LARGE: crate::errors::Decode =
    crate::errors::Decode::TooLarge { limit: crate::errors::Limit::Decoded, max: limits::SHIPPED.decoded };

// A 256-KiB comment has four delimiter bytes and one more line than newlines.
pub const COMMENT_NEWLINES: u32 = limits::SHIPPED.template.saturating_sub(4);
pub const COMMENT_LINES: u32 = COMMENT_NEWLINES.saturating_add(1);
pub const COMMENT_HELD_BYTES: u64 = 1_048_564;
// Sixteen such line tables fit in 16 MiB; q is the seventeenth distinct blob.
const COMMENT_NAMES: &[u8] = b"abcdefghijklmnopq";
pub fn comment_template() -> crate::evaluate::template::Template {
    let mut lines = skein_lib::List::with_capacity(COMMENT_LINES);
    lines.push(0).expect("first line");
    for offset in 3..=COMMENT_NEWLINES.saturating_add(2) {
        lines.push(offset).expect("comment lines");
    }
    crate::evaluate::template::Template { lines: lines.into_boxed(), instructions: Box::new([]) }
}
pub fn decoded_comment_templates() -> crate::call::Side {
    let mut call = super::template_call(
        b"tasks/motd/tree/a.j2",
        r::Mode::File,
        u64::from(limits::SHIPPED.template),
        limits::SHIPPED,
    );
    let count = u32::try_from(COMMENT_NAMES.len()).expect("comment count");
    let mut entries = skein_lib::List::with_capacity(count);
    let mut invalid = skein_lib::List::with_capacity(count);
    for name in COMMENT_NAMES {
        let mut path = skein_lib::Writer::new(b"tasks/motd/tree/a.j2".len());
        path.put(b"tasks/motd/tree/").expect("prefix");
        path.put(core::slice::from_ref(name)).expect("name");
        path.put(b".j2").expect("extension");
        let path = path.finish();
        entries
            .push(r::Entry {
                oid: r::Oid(path.clone()),
                path,
                mode: r::Mode::File,
                size: u64::from(limits::SHIPPED.template),
            })
            .expect("comment count");
        invalid.push(false).expect("comment count");
    }
    call.entries = entries.into_boxed();
    call.invalid = invalid.into_boxed();
    call.files = false;
    let mut output = skein_lib::Queue::with_capacity(super::SINGLE_REQUEST);
    call.files(limits::SHIPPED, &mut output);
    let template = comment_template();
    for blob in call.pending.clone() {
        call.decoded(&blob.oid, crate::vocabulary::Decoded::Template(template.clone()));
    }
    call
}
