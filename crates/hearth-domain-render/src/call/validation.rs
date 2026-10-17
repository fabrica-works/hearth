//! Schema diagnostics placed at their source documents.
use super::{Side, Target, compose};
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

impl Side {
    pub(super) fn dependencies_valid(&self, target: &Target, index: usize) -> bool {
        if !self.repository_valid(target, index) {
            return false;
        }
        for index in [target.fleet, target.base_document, Some(target.environment_document), Some(target.base_env)]
            .into_iter()
            .flatten()
        {
            if self.documents.get(index).is_none() {
                return false;
            }
        }
        true
    }

    pub(super) fn defaults(&mut self, targets: &[Target]) {
        let mut faults = List::with_capacity(16);
        self.defaults = crate::schema::declarations(&self.documents, &mut self.validator, &mut self.merge, &mut faults);
        let revision =
            crate::schema::Revision { documents: &self.documents, defaults: &self.defaults, listing: &self.entries };
        crate::schema::unclaimed(targets, &revision, &mut self.validator, &mut self.merge, &mut faults);
        schema_errors(&mut self.errors, &self.documents, None, &faults);
    }

    pub(super) fn needed(&self, target: &Target) -> bool {
        if self.names.is_empty() || self.names.contains(&target.name) {
            return true;
        }
        for selected in &self.targets {
            if self.names.contains(&selected.name)
                && let Some(placement) = compose::placement(selected, &self.documents)
                && let Some(on) = placement.on
                && on.target == target.name
            {
                return true;
            }
        }
        false
    }

    pub(super) fn host_valid(&self, name: &[u8]) -> bool {
        for (index, target) in self.targets.iter().enumerate() {
            if target.name.as_ref() != name {
                continue;
            }
            if !self.dependencies_valid(target, index) || compose::placement(target, &self.documents).is_none() {
                return false;
            }
            return self.config(self.settings.get(index).expect("host settings")).is_some();
        }
        false
    }
}

pub(super) fn host_schema(
    name: &[u8],
    targets: &[Target],
    settings: &[compose::Settings],
    revision: &crate::schema::Revision<'_>,
    validator: &mut crate::schema::Validator,
    mut left: u64,
) -> bool {
    for (target, settings) in targets.iter().zip(settings) {
        if target.name.as_ref() == name {
            let check = crate::schema::Check::Target { environment: &target.environment, targets: &[] };
            return crate::schema::validate(
                settings,
                revision,
                &check,
                validator,
                &mut left,
                &mut List::with_capacity(16),
            )
            .is_some();
        }
    }
    false
}
