//! The repository's implicit hearth task (protocol/format.md, 5.12).
use super::{Layer, Target, layers, placement};
use crate::errors::{self, Rule};
use crate::revision::{self, Documents, Kind, Position};
use alloc::boxed::Box;
use skein_lib::bytes;
pub(crate) fn reserved(target: &Target, documents: &Documents, errors: &mut errors::Collector) -> bool {
    let container = is_container(target, documents);
    let mut refused = false;
    for (index, source, layer) in layers(target, documents).into_iter().flatten() {
        if [Layer::Fleet, Layer::Environment].contains(&layer) {
            continue;
        }
        for task in &source.tasks {
            if task.name.as_ref() != b"hearth" {
                continue;
            }
            let rule = if container {
                Some(Rule::HearthContainer)
            } else if task.value.kind == Kind::Null {
                Some(Rule::HearthNull)
            } else {
                None
            };
            if let Some(rule) = rule {
                errors.push(error(documents.path(index), task.at, Some(target.name.clone()), rule));
                refused = true;
            }
        }
    }
    if !container && documents.declaration(b"hearth").is_none() {
        errors.push(super::error(
            b"tasks/hearth/task.json",
            None,
            errors::Composition::Task,
            Some(target.name.clone()),
            Some(bytes::copy_of(b"hearth")),
        ));
        refused = true;
    }
    refused
}
pub(super) fn error(path: &[u8], at: Position, target: Option<Box<[u8]>>, rule: Rule) -> errors::Error {
    errors::Error {
        side: None,
        kind: errors::Kind::Reserved(errors::Reserved { rule, what: bytes::copy_of(b"hearth") }),
        class: errors::Class::Configuration,
        at: Some(errors::Place { path: bytes::copy_of(path), position: Some(at) }),
        also: None,
        target,
        task: Some(bytes::copy_of(b"hearth")),
    }
}

fn is_container(target: &Target, documents: &Documents) -> bool {
    let Some(placement) = placement(target, documents) else { return false };
    placement.kind == revision::Runs::Container
}
