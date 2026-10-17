//! Check the composed Debian root before task conditions (render.md, 3.3).
use super::{Target, compose};
use crate::{errors, evaluate::Value, schema};
use skein_lib::{List, bytes};
pub(super) fn check(
    documents: &crate::revision::Documents,
    merge: &mut compose::Merge,
    errors: &mut errors::Collector,
    target: &Target,
) -> Option<Value> {
    let composed = compose::debian(target, documents, merge);
    let mut faults = List::with_capacity(16);
    let value = schema::debian(composed.as_ref(), target.base_document.unwrap_or(target.base_env), &mut faults);
    for fault in &faults {
        errors.push(errors::Error {
            side: None,
            kind: errors::Kind::Schema(fault.schema.clone()),
            class: crate::vocabulary::Class::Configuration,
            at: Some(errors::Place { path: bytes::copy_of(documents.path(fault.document)), position: Some(fault.at) }),
            also: None,
            target: Some(target.name.clone()),
            task: None,
        });
    }
    value
}
