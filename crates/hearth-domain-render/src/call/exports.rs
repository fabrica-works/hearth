//! Publish cached pairs and assemble each reader task's environment root.
use crate::{
    compose::Target,
    errors::Error,
    evaluate::{Entry, Value},
    exports::{Cache, Fleet},
    parts::exports::Exports,
    schema::Validated,
};
use alloc::boxed::Box;
use skein_lib::List;
pub(super) fn prepare(
    target: &Target,
    checked: &Validated,
    fleet: &Fleet<'_>,
    cache: &Cache,
) -> Result<(Exports, Value), Box<Error>> {
    let capacity = u32::try_from(checked.tasks.len()).expect("tasks");
    let mut exported = List::with_capacity(capacity);
    let mut roots = List::with_capacity(capacity);
    for task in &checked.tasks {
        if let Some(values) = cache.published(target, &task.name)? {
            exported.push(values).expect("tasks");
        }
        let Some((_, Some(declaration))) = fleet.revision.documents.declaration(&task.name) else { continue };
        let root = match cache.root(target, declaration, fleet) {
            Ok(root) => root,
            Err(mut error) => {
                error.target = Some(target.name.clone());
                error.task = Some(task.name.clone());
                return Err(error);
            }
        };
        roots.push(Entry { key: task.name.clone(), value: Some(root) }).expect("tasks");
    }
    Ok((crate::parts::exports::build(exported.into_boxed()), Value::Map(roots.into_boxed())))
}

impl super::Side {
    pub(super) fn prime_exports(&mut self, target: &Target) -> bool {
        let revision =
            crate::schema::Revision { documents: &self.documents, defaults: &self.defaults, listing: &self.entries };
        let fleet =
            Fleet { revision, targets: &self.targets, tasks: &self.tasks, faults: &self.errors, limits: &self.limits };
        self.exports.prime(target, &fleet)
    }
    pub(super) fn read_exports(&self, target: &Target, checked: &Validated) -> Result<(Exports, Value), Box<Error>> {
        let revision =
            crate::schema::Revision { documents: &self.documents, defaults: &self.defaults, listing: &self.entries };
        let fleet =
            Fleet { revision, targets: &self.targets, tasks: &self.tasks, faults: &self.errors, limits: &self.limits };
        prepare(target, checked, &fleet, &self.exports)
    }
}
