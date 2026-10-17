//! Scripted repositories: the domain receives typed documents.
use hearth_domain_render::{
    parts::meta::{Meta, Placement as MetaPlacement},
    revision::{Document, Entry, Firmware, Layer, Mode, Oid, Placement, Position, Runs},
    vocabulary::Decoded,
};
use hearth_lib::encode::Digest;
#[derive(Clone, Debug)]
pub struct Repository {
    pub entries: Vec<Entry>,
    pub documents: Vec<(Oid, Decoded)>,
}
#[must_use]
pub fn oid(byte: u8) -> Oid {
    Oid(vec![byte; 40].into_boxed_slice())
}
#[must_use]
pub fn document(placement: Option<Placement>) -> Decoded {
    Decoded::Document(Document { at: Position { line: 1, column: 1 }, layer: Layer { placement } })
}
#[must_use]
pub fn placement() -> Placement {
    Placement {
        at: Position { line: 1, column: 15 },
        kind: Runs::Metal,
        firmware: Some(Firmware::Efi),
        hardware: Some(Box::from(b"7th-gen".as_slice())),
        on: None,
        mutable: false,
    }
}
#[must_use]
pub fn x1_prod() -> Repository {
    Repository {
        entries: vec![
            entry(b"envs/prod.json", b'1'),
            entry(b"hosts/x1/x1.json", b'2'),
            entry(b"hosts/x1/x1-prod.json", b'3'),
        ],
        documents: vec![
            (oid(b'1'), document(None)),
            (oid(b'2'), document(None)),
            (oid(b'3'), document(Some(placement()))),
        ],
    }
}
#[must_use]
pub fn entry(path: &[u8], byte: u8) -> Entry {
    Entry { path: Box::from(path), mode: Mode::File, oid: oid(byte), size: 2 }
}
#[must_use]
pub fn x1_prod_meta() -> Meta {
    Meta {
        target: Box::from(b"x1-prod".as_slice()),
        base: Box::from(b"x1".as_slice()),
        member: None,
        environment: Box::from(b"prod".as_slice()),
        placement: MetaPlacement {
            kind: Runs::Metal,
            firmware: Some(Firmware::Efi),
            hardware: Some(Box::from(b"7th-gen".as_slice())),
            on: None,
        },
    }
}
#[must_use]
pub fn digest(text: &str) -> Digest {
    Digest(core::array::from_fn(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).expect("digest hex")))
}
