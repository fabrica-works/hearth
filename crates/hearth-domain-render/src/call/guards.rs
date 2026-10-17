//! Validate all task guards before any target entry or file (render.md, 7.1).
use crate::{
    entries,
    errors::{Error, Place},
    evaluate::{Context, Value, context},
    parts::meta::Meta,
    schema::Validated,
    vocabulary::Class,
};
use alloc::boxed::Box;
pub(crate) fn contained(
    documents: &crate::revision::Documents,
    errors: &mut crate::errors::Collector,
    limits: &crate::limits::Limits,
    meta: &Meta,
    checked: &Validated,
    containers: &[crate::compose::Container],
) -> bool {
    let target = context::target(meta, containers);
    let empty = Value::Map(Box::new([]));
    let mut valid = true;
    for task in &checked.tasks {
        let Some((document, Some(declaration))) = documents.declaration(&task.name) else { continue };
        let context =
            Context { target: &target, debian: &empty, settings: &task.settings, tasks: &task.tasks, exports: &empty };
        for fault in entries::guards(declaration, &context, limits) {
            valid = false;
            errors.push(Error {
                side: None,
                kind: fault.kind,
                class: Class::Configuration,
                at: Some(Place { path: skein_lib::bytes::copy_of(documents.path(document)), position: Some(fault.at) }),
                also: None,
                target: Some(meta.target.clone()),
                task: Some(task.name.clone()),
            });
        }
    }
    if !valid {
        return false;
    }
    let mut left = limits.expanded;
    for task in &checked.tasks {
        let Some((document, Some(declaration))) = documents.declaration(&task.name) else { continue };
        let context =
            Context { target: &target, debian: &empty, settings: &task.settings, tasks: &task.tasks, exports: &empty };
        if let Err(fault) = entries::expand(declaration, &context, limits, &mut left) {
            valid = false;
            let path = documents.path(document);
            let mut also = None;
            if let Some(position) = fault.also {
                also = Some(Place { path: skein_lib::bytes::copy_of(path), position: Some(position) });
            }
            errors.push(Error {
                side: None,
                kind: fault.kind,
                class: Class::Configuration,
                at: Some(Place { path: skein_lib::bytes::copy_of(path), position: Some(fault.at) }),
                also,
                target: Some(meta.target.clone()),
                task: Some(task.name.clone()),
            });
        }
    }
    valid
}

#[cfg(test)]
pub(crate) fn check(
    documents: &crate::revision::Documents,
    errors: &mut crate::errors::Collector,
    limits: &crate::limits::Limits,
    meta: &Meta,
    checked: &Validated,
) -> bool {
    contained(documents, errors, limits, meta, checked, &[])
}
