//! Publish present tasks before rendering their files (format.md, 8.1).
use crate::{
    errors::{Error, Place},
    evaluate::context,
    parts::{exports::Exports, meta::Meta},
    schema::{self, Validated},
};
use skein_lib::List;
pub(super) fn prepare(
    meta: &Meta,
    checked: &Validated,
    containers: &[crate::compose::Container],
    revision: &schema::Revision<'_>,
    targets: &[crate::compose::Target],
    limits: &crate::limits::Limits,
    errors: &mut crate::errors::Collector,
) -> Option<(Exports, u64)> {
    let mut exported = List::with_capacity(u32::try_from(checked.tasks.len()).expect("task count"));
    let mut left = limits.expanded;
    let names = super::validation::targets(targets, &meta.environment);
    let check = schema::Check::Target { environment: &meta.environment, targets: &names };
    let target = context::target(meta, containers);
    for task in &checked.tasks {
        let Some((document, Some(declaration))) = revision.documents.declaration(&task.name) else { continue };
        let against = schema::Against {
            declaration,
            defaults: revision.defaults.of(&task.name).expect("validated task"),
            listing: revision.listing,
        };
        match crate::exports::evaluate(&against, &check, &task.settings, &target, limits, &mut left) {
            Ok(mut values) => {
                values.task.clone_from(&task.name);
                exported.push(values).expect("task count");
            }
            Err(fault) => {
                errors.push(Error {
                    side: None,
                    kind: fault.kind,
                    class: crate::errors::Class::Configuration,
                    at: Some(Place {
                        path: skein_lib::bytes::copy_of(revision.documents.path(document)),
                        position: Some(fault.at),
                    }),
                    also: None,
                    target: Some(meta.target.clone()),
                    task: Some(task.name.clone()),
                });
                return None;
            }
        }
    }
    Some((crate::parts::exports::build(exported.into_boxed()), left))
}
