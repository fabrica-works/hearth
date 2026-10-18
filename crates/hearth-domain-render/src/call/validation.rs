//! Schema diagnostics placed at their source documents.
use super::{Side, Target, compose};
use crate::{
    errors,
    revision::{self, Documents},
};
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
    pub(super) fn validated_target(&mut self, target: &Target) -> Option<crate::schema::Validated> {
        let revision =
            crate::schema::Revision { documents: &self.documents, defaults: &self.defaults, listing: &self.entries };
        let settings = self
            .settings
            .get(usize::try_from(self.next.saturating_sub(1)).expect("bounded cursor"))
            .expect("target settings");
        checked(settings, &revision, target, &self.targets, &mut self.validator, self.validated, &mut self.errors)
    }

    pub(super) fn repository_valid(&self, target: &Target, index: usize) -> bool {
        for (entry, invalid) in self.entries.iter().zip(&self.invalid) {
            if !invalid {
                continue;
            }
            if let Some(task) = revision::task(&entry.path) {
                if let Some(settings) = self.settings.get(index) {
                    for present in &settings.tasks {
                        if present.name.as_ref() == task {
                            return false;
                        }
                    }
                }
            } else {
                if revision::base(&entry.path) == Some(target.base.as_ref()) {
                    return false;
                }
                for index in
                    [target.fleet, target.base_document, Some(target.environment_document), Some(target.base_env)]
                        .into_iter()
                        .flatten()
                {
                    if self.documents.path(index) == entry.path.as_ref() {
                        return false;
                    }
                }
            }
        }
        true
    }

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
        self.defaults = crate::schema::declarations(
            &self.documents,
            &self.entries,
            &mut self.validator,
            &mut self.merge,
            &mut faults,
        );
        let revision =
            crate::schema::Revision { documents: &self.documents, defaults: &self.defaults, listing: &self.entries };
        crate::schema::unclaimed(targets, &revision, &mut self.validator, &mut self.merge, &mut faults);
        schema_errors(&mut self.errors, &self.documents, None, &faults);
    }

    pub(super) fn placement_error(&mut self, error: errors::Error) {
        if let errors::Kind::Composition(errors::Composition::On { on, host: errors::Host::Unplaced }) = &error.kind
            && !self.host_valid(on)
        {
            self.outcome = self.outcome.worst(crate::vocabulary::Outcome::Failed);
        } else {
            self.errors.push(error);
        }
    }

    pub(super) fn needed(&self, target: &Target) -> bool {
        self.names.is_empty() || self.names.contains(&target.name)
    }

    pub(super) fn host_valid(&self, name: &[u8]) -> bool {
        for target in &self.targets {
            if target.name.as_ref() != name {
                continue;
            }
            for index in [target.fleet, target.base_document, Some(target.environment_document), Some(target.base_env)]
                .into_iter()
                .flatten()
            {
                if self.documents.get(index).is_none() {
                    return false;
                }
                for (entry, invalid) in self.entries.iter().zip(&self.invalid) {
                    if *invalid && self.documents.path(index) == entry.path.as_ref() {
                        return false;
                    }
                }
            }
            return true;
        }
        false
    }
}

pub(super) fn targets(targets: &[Target], environment: &[u8]) -> Box<[Box<[u8]>]> {
    let mut names = List::with_capacity(u32::try_from(targets.len()).expect("bounded targets"));
    for target in targets {
        if target.environment.as_ref() == environment {
            names.push(target.name.clone()).expect("target count");
        }
    }
    names.into_boxed()
}

pub(super) fn checked(
    settings: &compose::Settings,
    revision: &crate::schema::Revision<'_>,
    target: &Target,
    targets: &[Target],
    validator: &mut crate::schema::Validator,
    mut left: u64,
    errors: &mut errors::Collector,
) -> Option<crate::schema::Validated> {
    let names = self::targets(targets, &target.environment);
    let check = crate::schema::Check::Target { environment: &target.environment, targets: &names };
    let mut faults = List::with_capacity(16);
    let checked = crate::schema::validate(settings, revision, &check, validator, &mut left, &mut faults);
    schema_errors(errors, revision.documents, Some(&target.name), &faults);
    if validator.limited() {
        errors::push(errors, limit(&target.name));
    }
    checked
}
