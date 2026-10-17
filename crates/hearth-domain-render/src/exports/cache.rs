//! Revision-local pairs and the environment roots which share them.
use super::{Exported, evaluation};
use crate::{
    compose::{self, Target},
    errors::{self, Cause, Error, Kind, Place},
    evaluate::{Entry, FaultKind, Value, context},
    limits::Limits,
    revision::{self, declaration::Declaration},
    schema::{self, Against, Check},
};
use alloc::boxed::Box;
use skein_lib::{List, Map, bytes};
pub(crate) struct Fleet<'a> {
    pub revision: &'a schema::Revision<'a>,
    pub targets: &'a [Target],
    pub settings: &'a [compose::Settings],
    pub faults: &'a errors::Collector,
    pub limits: &'a Limits,
}
type Pair = (Box<[u8]>, Box<[u8]>);
#[derive(Debug)]
pub(crate) struct Cache {
    pairs: Map<Pair, Result<Exported, Box<Error>>>,
    left: u64,
    limited: bool,
}
impl Cache {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self { pairs: Map::with_capacity(0), left: limits.exported, limited: false }
    }
    /// Advance one missing pair per resume, including the reader's own published part.
    pub(crate) fn prime(&mut self, reader: &Target, fleet: &Fleet<'_>) -> bool {
        if self.limited {
            return false;
        }
        for (target, settings) in fleet.targets.iter().zip(fleet.settings) {
            if target.environment != reader.environment {
                continue;
            }
            for task in &settings.tasks {
                let Some((_, Some(declaration))) = fleet.revision.documents.declaration(&task.name) else { continue };
                if !needed(reader, target, &task.name, !declaration.exports.is_empty(), fleet) {
                    continue;
                }
                let key = (target.name.clone(), task.name.clone());
                if self.pairs.contains_key(&key) {
                    continue;
                }
                let values = self.evaluate(target, task, fleet);
                if self.pairs.len() == self.pairs.capacity() {
                    let mut larger = Map::with_capacity(self.pairs.capacity().max(1).saturating_mul(2));
                    for (key, value) in &self.pairs {
                        larger.insert(key.clone(), value.clone()).expect("doubled pairs");
                    }
                    self.pairs = larger;
                }
                self.pairs.insert(key, values).expect("reserved pair");
                return true;
            }
        }
        false
    }
    fn evaluate(
        &mut self,
        target: &Target,
        task: &compose::Present,
        fleet: &Fleet<'_>,
    ) -> Result<Exported, Box<Error>> {
        let charge = crate::evaluate::ELEMENT
            .saturating_add(u64::try_from(target.name.len().saturating_add(task.name.len())).expect("names"));
        let Some(left) = self.left.checked_sub(charge) else {
            self.limited = true;
            return Err(Box::new(limit(target, Some(&task.name))));
        };
        self.left = left;
        let revision = fleet.revision;
        if let Some(error) = document_fault(target, &task.name, fleet) {
            return Err(Box::new(error));
        }
        let (document, Some(declaration)) = revision.documents.declaration(&task.name).expect("declared task") else {
            unreachable!("prime checks declaration")
        };
        let Some(defaults) = revision.defaults.of(&task.name) else {
            return Err(Box::new(limit(target, Some(&task.name))));
        };
        let against = Against { declaration, defaults, listing: revision.listing };
        let mut names = List::with_capacity(u32::try_from(fleet.targets.len()).expect("target count"));
        for other in fleet.targets {
            if other.environment == target.environment {
                names.push(other.name.clone()).expect("target count");
            }
        }
        let names = names.into_boxed();
        let check = Check::Target { environment: &target.environment, targets: &names };
        let mut validator = schema::Validator::new(fleet.limits);
        let mut faults = List::with_capacity(16);
        let mut settings = task.settings.clone();
        settings.from = task.from;
        let settings = validator.settings(&settings, &against, &check, &mut self.left, &mut faults);
        if validator.limited() {
            self.limited = true;
            return Err(Box::new(limit(target, Some(&task.name))));
        }
        if let Some(fault) = faults.as_slice().first() {
            return Err(Box::new(placed(
                target,
                &task.name,
                Kind::Schema(fault.schema.clone()),
                revision.documents.path(fault.document),
                fault.at,
            )));
        }
        let placement = compose::placed(target, revision.documents, fleet.targets)?;
        let containers = compose::containers(target, fleet.targets, revision.documents)?;
        let target_root = context::target(&compose::metadata(target, placement), &containers);
        match evaluation(
            &against,
            &check,
            &settings.expect("settings validated"),
            &target_root,
            fleet.limits,
            &mut self.left,
            &mut self.limited,
        ) {
            Ok(mut values) => {
                values.task.clone_from(&task.name);
                Ok(values)
            }
            Err(fault) => {
                Err(Box::new(placed(target, &task.name, fault.kind, revision.documents.path(document), fault.at)))
            }
        }
    }
    pub(crate) fn published(&self, target: &Target, task: &[u8]) -> Result<Option<Exported>, Box<Error>> {
        if self.limited {
            return Err(Box::new(limit(target, Some(task))));
        }
        match self.pairs.get(&(target.name.clone(), bytes::copy_of(task))) {
            Some(Ok(values)) => Ok(Some(values.clone())),
            Some(Err(error)) => Err(error.clone()),
            None => Ok(None),
        }
    }
    pub(crate) fn root(
        &self,
        reader: &Target,
        declaration: &Declaration,
        fleet: &Fleet<'_>,
    ) -> Result<Value, Box<Error>> {
        let mut reads = List::with_capacity(u32::try_from(declaration.reads.len()).expect("reads"));
        for read in &declaration.reads {
            if !read.exports {
                continue;
            }
            if self.limited {
                return Err(Box::new(limit(reader, None)));
            }
            for index in 0..fleet.limits.errors {
                let Some(error) = fleet.faults.get(index) else { break };
                let Some(at) = &error.at else { continue };
                if broken(&error.kind)
                    && hidden(&at.path, fleet)
                    && let revision::Class::Host { base, name } = revision::class(&at.path)
                    && environment(name, base, &reader.environment)
                    && let Some(error) = exported(error, base, &read.task.text)
                {
                    return Err(Box::new(error));
                }
            }
            let mut targets = List::with_capacity(u32::try_from(fleet.targets.len()).expect("target count"));
            for (target, settings) in fleet.targets.iter().zip(fleet.settings) {
                if target.environment != reader.environment || !has(settings, &read.task.text) {
                    continue;
                }
                let values = match self.published(target, &read.task.text) {
                    Ok(Some(values)) => values.values,
                    Ok(None) => Box::new([]),
                    Err(error) => {
                        return Err(Box::new(exported(&error, &target.name, &read.task.text).expect("cached cause")));
                    }
                };
                targets.push(Entry { key: target.name.clone(), value: Some(Value::Map(values)) }).expect("targets");
            }
            reads
                .push(Entry { key: read.task.text.clone(), value: Some(Value::Map(targets.into_boxed())) })
                .expect("reads");
        }
        Ok(Value::Map(reads.into_boxed()))
    }
}
fn needed(reader: &Target, target: &Target, task: &[u8], publishes: bool, fleet: &Fleet<'_>) -> bool {
    if reader.name == target.name && publishes {
        return true;
    }
    for (target, settings) in fleet.targets.iter().zip(fleet.settings) {
        if target.name != reader.name {
            continue;
        }
        for present in &settings.tasks {
            if let Some((_, Some(declaration))) = fleet.revision.documents.declaration(&present.name) {
                for read in &declaration.reads {
                    if read.exports && read.task.text.as_ref() == task {
                        return true;
                    }
                }
            }
        }
    }
    false
}
fn has(settings: &compose::Settings, task: &[u8]) -> bool {
    for present in &settings.tasks {
        if present.name.as_ref() == task {
            return true;
        }
    }
    false
}
fn broken(kind: &Kind) -> bool {
    match kind {
        Kind::Decode(_) | Kind::Repository(_) | Kind::Composition(_) => true,
        Kind::Exported(_)
        | Kind::Tool(_)
        | Kind::Guard(_)
        | Kind::Collision(_)
        | Kind::Store(_)
        | Kind::Schema(_)
        | Kind::Evaluation(_) => false,
    }
}
fn hidden(path: &[u8], fleet: &Fleet<'_>) -> bool {
    match fleet.revision.documents.find(path) {
        Some(index) => fleet.revision.documents.get(index).is_none(),
        None => true,
    }
}
fn environment(name: &[u8], base: &[u8], environment: &[u8]) -> bool {
    if name == base {
        return true;
    }
    let Some(suffix) = name.strip_prefix(base) else { return false };
    suffix.strip_prefix(b"-") == Some(environment)
}
fn document_fault(target: &Target, task: &[u8], fleet: &Fleet<'_>) -> Option<Error> {
    for index in 0..fleet.limits.errors {
        let Some(error) = fleet.faults.get(index) else { break };
        let Some(at) = &error.at else { continue };
        if let Kind::Composition(_) = error.kind
            && let Some(name) = error.task.as_deref()
            && name != task
        {
            continue;
        }
        for document in [target.fleet, target.base_document, Some(target.environment_document), Some(target.base_env)]
            .into_iter()
            .flatten()
        {
            if fleet.revision.documents.path(document) == at.path.as_ref() && broken(&error.kind) {
                return Some(error.clone());
            }
        }
    }
    None
}
fn exported(error: &Error, exporter: &[u8], task: &[u8]) -> Option<Error> {
    let cause = match &error.kind {
        Kind::Decode(value) => Cause::Decode(value.clone()),
        Kind::Repository(value) => Cause::Repository(value.clone()),
        Kind::Composition(value) => Cause::Composition(value.clone()),
        Kind::Schema(value) => Cause::Schema(value.clone()),
        Kind::Evaluation(value) => Cause::Evaluation(*value),
        Kind::Exported(_) | Kind::Tool(_) | Kind::Guard(_) | Kind::Collision(_) | Kind::Store(_) => return None,
    };
    let mut error = error.clone();
    error.kind =
        Kind::Exported(errors::Exported { exporter: bytes::copy_of(exporter), task: bytes::copy_of(task), cause });
    error.task = Some(bytes::copy_of(task));
    Some(error)
}
fn placed(target: &Target, task: &[u8], kind: Kind, path: &[u8], at: revision::Position) -> Error {
    Error {
        side: None,
        kind,
        class: errors::Class::Configuration,
        at: Some(Place { path: bytes::copy_of(path), position: Some(at) }),
        also: None,
        target: Some(target.name.clone()),
        task: Some(bytes::copy_of(task)),
    }
}
fn limit(target: &Target, task: Option<&[u8]>) -> Error {
    let mut scope = None;
    if let Some(task) = task {
        scope = Some(bytes::copy_of(task));
    }
    Error {
        side: None,
        kind: Kind::Evaluation(FaultKind::Limit),
        class: errors::Class::Configuration,
        at: None,
        also: None,
        target: Some(target.name.clone()),
        task: scope,
    }
}
