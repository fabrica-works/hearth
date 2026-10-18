use super::{
    composition::{at, map},
    containers, declaration, declarations, with_document,
};
use crate::{call::Side, compose, limits::SHIPPED, revision, schema};
use skein_lib::{List, bytes::copy_of};

// One string alone passes the shipped 64 MiB composed-settings allowance.
const TOO_MANY_SETTINGS_BYTES: u64 = SHIPPED.composed + 1;
pub fn oversized() -> (Side, Box<[compose::Target]>) {
    let mut documents = declarations(&[(b"hearth", Some(declaration(&[], &[])))]);
    documents = with_document(&documents, b"envs/prod.json", containers::document(None));
    for (path, large) in [(b"hosts/large/large-prod.json".as_slice(), true), (b"hosts/small/small-prod.json", false)] {
        let mut document = containers::document(Some(containers::placement(None)));
        if large {
            document.layer.tasks = Box::new([revision::Task {
                name: copy_of(b"hearth"),
                at: at(1, 1),
                value: map(
                    at(1, 1),
                    &[(
                        b"large",
                        at(1, 1),
                        revision::Value {
                            at: at(1, 1),
                            kind: revision::Kind::String(skein_lib::bytes::zeroed(
                                usize::try_from(TOO_MANY_SETTINGS_BYTES).expect("settings bound"),
                            )),
                        },
                    )],
                ),
            }]);
        }
        documents = with_document(&documents, path, document);
    }
    let targets = super::discover(&documents).targets;
    let mut side = Side::new(copy_of(b"/repo"), Box::new([]), &SHIPPED);
    side.entries = documents.entries().into();
    side.documents = documents;
    side.defaults = schema::declarations(
        &side.documents,
        &side.entries,
        &mut side.validator,
        &mut side.merge,
        &mut List::with_capacity(super::SCHEMA_FAULTS),
    );
    side.tasks = super::target_tasks(&targets, &side.documents);
    side.targets = targets.clone();
    (side, targets)
}
pub fn validate(side: &mut Side, targets: &[compose::Target], name: &[u8]) -> Option<schema::Validated> {
    for (index, target) in targets.iter().enumerate() {
        if target.name.as_ref() == name {
            side.next = u32::try_from(index).expect("target index").saturating_add(1);
            return side.validated_target(target);
        }
    }
    panic!("fixture target")
}
