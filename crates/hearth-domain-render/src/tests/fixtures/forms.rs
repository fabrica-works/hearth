use super::{
    composition::{at, map, string},
    schemas::{composed, typed_declaration},
};
use crate::{
    compose::{Composed, Layer, Merge},
    evaluate::Value,
    limits::{self, Limits},
    revision::{self, Entry, Mode, Oid},
    schema::{self, Check, Fault},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
pub fn setting(value: &[u8]) -> Composed {
    composed(&map(at(2, 1), &[(b"x", at(2, 2), string(at(2, 7), value))]))
}
pub fn listing(mode: Mode, size: u64) -> [Entry; 1] {
    [Entry { path: copy_of(b"lab-ca/lab.crt"), mode, oid: Oid(copy_of(b"file")), size }]
}
pub fn validate(
    form: revision::declaration::Form,
    value: &Composed,
    check: &Check<'_>,
    listing: &[Entry],
    limits: &Limits,
) -> Result<Value, Box<[Fault]>> {
    super::schemas::validate_with(&typed_declaration(form), value, check, listing, limits)
}
/// Compose the fixed defaults before the patch, so an explicit deletion
/// of main differs from a patch that never mentions it.
pub fn debian_input(patch: &[(&[u8], revision::Position, revision::Value)]) -> Composed {
    let bottom = map(
        at(1, 1),
        &[(
            b"components",
            at(1, 2),
            map(
                at(1, 15),
                &[(b"main", at(1, 16), revision::Value { at: at(1, 24), kind: revision::Kind::Bool(true) })],
            ),
        )],
    );
    let mut merge = Merge::new(&limits::SHIPPED);
    let bottom = merge.merge(None, &bottom, Layer::Defaults, 0);
    merge.merge(bottom, &map(at(2, 1), patch), Layer::Base, 2).expect("object patch")
}
pub fn debian_fields() -> [(&'static [u8], revision::Position, revision::Value); 3] {
    [
        (b"architecture", at(2, 2), string(at(2, 18), b"amd64")),
        (
            b"snapshot",
            at(2, 26),
            map(
                at(2, 38),
                &[
                    (b"archive", at(2, 39), string(at(2, 50), b"20261001T030405Z")),
                    (b"security", at(2, 70), string(at(2, 82), b"20261001T030405Z")),
                ],
            ),
        ),
        (b"suite", at(2, 100), string(at(2, 109), b"trixie")),
    ]
}
pub fn debian(patch: &[(&[u8], revision::Position, revision::Value)]) -> (Option<Value>, Box<[Fault]>) {
    let mut faults = List::with_capacity(16);
    let mut fields = skein_lib::Map::with_capacity(16);
    for (name, at, value) in debian_fields() {
        fields.insert(name, (at, value)).expect("fixture capacity");
    }
    for (name, at, value) in patch {
        fields.insert(*name, (*at, value.clone())).expect("fixture capacity");
    }
    let mut pairs = List::with_capacity(fields.len());
    for (name, (at, value)) in &fields {
        pairs.push((*name, *at, value.clone())).expect("field count");
    }
    let value = schema::debian(Some(&debian_input(pairs.as_slice())), 2, &mut faults);
    (value, faults.into_boxed())
}
pub fn environment_targets() -> [Box<[u8]>; 2] {
    [copy_of(b"x1-prod"), copy_of(b"x2-prod")]
}

pub const NAME_BYTES: &[u8; 256] = &[b'a'; 256];
