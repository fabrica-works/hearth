//! Schema diagnostics placed at their source documents.
use crate::{errors, revision::Documents};
use alloc::boxed::Box;
use skein_lib::List;

pub(super) fn limit(target: &[u8]) -> errors::Error {
    errors::Error {
        side: None,
        kind: errors::Kind::Evaluation(crate::evaluate::FaultKind::Limit),
        class: crate::vocabulary::Class::Configuration,
        at: None,
        also: None,
        target: Some(skein_lib::bytes::copy_of(target)),
        task: None,
    }
}

pub(super) fn schema_errors(
    errors: &mut errors::Collector,
    documents: &Documents,
    target: Option<&[u8]>,
    faults: &List<(Box<[u8]>, crate::schema::Fault)>,
) {
    let mut scope = None;
    if let Some(name) = target {
        scope = Some(skein_lib::bytes::copy_of(name));
    }
    for (task, fault) in faults {
        errors::push(
            errors,
            errors::Error {
                side: None,
                kind: errors::Kind::Schema(fault.schema.clone()),
                class: crate::vocabulary::Class::Configuration,
                at: Some(errors::Place {
                    path: skein_lib::bytes::copy_of(documents.path(fault.document)),
                    position: Some(fault.at),
                }),
                also: None,
                target: scope.clone(),
                task: Some(task.clone()),
            },
        );
    }
}
