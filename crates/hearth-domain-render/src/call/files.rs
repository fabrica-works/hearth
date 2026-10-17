//! Concrete files plans, with each fault placed at its task declaration.
mod sources;
use super::Side;
use crate::{
    entries::{self, Expansion},
    errors::{Error, Kind, Place, Repository},
    kinds::files::{self, Plan, Source, TreeFile},
    parts::meta::Meta,
    revision::{self, Mode, declaration::Map},
    schema::Validated,
    vocabulary::Class,
};
use alloc::boxed::Box;
use skein_lib::{List, Map as Ordered, Writer, bytes::copy_of};
impl Side {
    pub(crate) fn plans(
        &self,
        meta: &Meta,
        checked: &Validated,
        expansion: &Expansion,
        errors: &mut List<Error>,
    ) -> Option<Box<[Plan]>> {
        let rows = expansion.of(Map::Files);
        let capacity = u32::try_from(self.entries.len()).expect("listing bound");
        let mut plans = List::with_capacity(u32::try_from(checked.tasks.len()).expect("task bound"));
        let mut destinations =
            Ordered::with_capacity(capacity.saturating_add(u32::try_from(rows.len()).expect("expanded count")));
        let mut valid = true;
        for task in &checked.tasks {
            let Some((document, Some(declaration))) = self.documents.declaration(&task.name) else {
                return None;
            };
            let mut tree = List::with_capacity(capacity);
            for entry in &self.entries {
                let revision::Class::Tree { task: name, path } = revision::class(&entry.path) else {
                    continue;
                };
                if name != task.name.as_ref() {
                    continue;
                }
                let source = if entry.mode == Mode::Link {
                    let target = self.links.get(&entry.oid)?;
                    if !files::link(target) {
                        errors
                            .push(Error {
                                side: None,
                                kind: Kind::Repository(Repository::LinkTarget),
                                class: Class::Configuration,
                                at: Some(Place { path: entry.path.clone(), position: None }),
                                also: None,
                                target: Some(meta.target.clone()),
                                task: Some(task.name.clone()),
                            })
                            .expect("fault capacity");
                        valid = false;
                        continue;
                    }
                    Source::Link { target: target.clone() }
                } else if path.ends_with(b".j2") {
                    self.templates.get(&entry.oid)?;
                    Source::Template { oid: entry.oid.clone(), path: entry.path.clone() }
                } else {
                    Source::Static { sha256: *self.cached.get(&entry.oid)? }
                };
                let path = path.strip_suffix(b".j2").unwrap_or(path);
                let mut absolute = Writer::new(path.len().saturating_add(1));
                absolute.put(b"/").expect("prefix room");
                absolute.put(path).expect("path room");
                tree.push(TreeFile {
                    path: absolute.finish(),
                    source,
                    mode: if entry.mode == Mode::Executable { 0o755 } else { 0o644 },
                })
                .expect("listing bound");
            }
            let mut faults = List::with_capacity(self.limits.expansion);
            let mut selected = List::with_capacity(self.limits.expansion);
            for row in &rows {
                if row.task == task.name.as_ref() {
                    selected.push(entries::Located { task: row.task, entry: row.entry }).expect("expansion bound");
                }
            }
            let dropped = files::dropped(declaration, selected.as_slice());
            valid &= self.unused(&task.name, declaration, meta, errors);
            let mut plan = files::plan(&task.name, tree.as_slice(), selected.as_slice(), &dropped, &mut faults);
            self.sources(&mut plan, &mut faults);
            for fault in faults.into_boxed() {
                valid = false;
                let also = self.destination(&task.name, selected.as_slice(), &fault);
                errors
                    .push(Error {
                        side: None,
                        kind: fault.kind,
                        class: Class::Configuration,
                        at: Some(Place { path: copy_of(self.documents.path(document)), position: Some(fault.at) }),
                        also,
                        target: Some(meta.target.clone()),
                        task: Some(task.name.clone()),
                    })
                    .expect("fault capacity");
            }
            let mut installs = List::with_capacity(u32::try_from(plan.installs.len()).expect("plan bound"));
            for install in plan.installs {
                if !destinations.contains_key(&install.path) {
                    destinations.insert(install.path.clone(), ()).expect("destination bound");
                    installs.push(install).expect("plan bound");
                }
            } // transitional: hearth 22
            plans.push(Plan { task: plan.task, installs: installs.into_boxed() }).expect("task bound");
        }
        if valid { Some(plans.into_boxed()) } else { None }
    }
}

impl Side {
    pub(super) fn files_valid(&self, settings: &crate::compose::Settings) -> bool {
        for task in &settings.tasks {
            let Some((_, Some(_))) = self.documents.declaration(&task.name) else {
                return false;
            };
            if !self.defaults.sound(&task.name) {
                return false;
            }
            for (index, entry) in self.entries.iter().enumerate() {
                let (name, path) = match revision::class(&entry.path) {
                    revision::Class::Tree { task, path } | revision::Class::Template { task, path } => (task, path),
                    revision::Class::Fleet
                    | revision::Class::Environment { .. }
                    | revision::Class::Host { .. }
                    | revision::Class::Declaration { .. }
                    | revision::Class::Script { .. }
                    | revision::Class::Readme { .. }
                    | revision::Class::Refused { .. }
                    | revision::Class::Other => continue,
                };
                if name != task.name.as_ref() {
                    continue;
                }
                if self.invalid.get(index) == Some(&true) {
                    return false;
                }
                if self.refused.contains_key(&entry.oid) {
                    return false;
                }
                let present = match entry.mode {
                    Mode::Link => self.links.contains_key(&entry.oid),
                    Mode::Submodule => false,
                    Mode::File | Mode::Executable => {
                        if path.ends_with(b".j2") {
                            self.templates.contains_key(&entry.oid)
                        } else {
                            self.cached.contains_key(&entry.oid)
                        }
                    }
                };
                if !present {
                    return false;
                }
            }
        }
        true
    }
}

impl Side {
    pub(super) fn compose_files(&mut self) {
        let invalid = crate::compose::rules(&self.documents, &mut self.errors);
        for (document, invalid) in self.documents.entries().iter().zip(invalid) {
            if invalid {
                for (entry, invalid) in self.entries.iter().zip(&mut self.invalid) {
                    if entry.path == document.path {
                        *invalid = true;
                    }
                }
            }
        }
        self.compose();
    }
}

pub(super) fn capacity(plans: &[Plan]) -> u32 {
    let mut capacity = 0_u32;
    for plan in plans {
        capacity = capacity.saturating_add(u32::try_from(plan.installs.len()).expect("plan bound"));
    }
    capacity
}

impl Side {
    fn destination(&self, task: &[u8], rows: &[entries::Located<'_>], fault: &entries::Fault) -> Option<Place> {
        if fault.kind != Kind::Repository(Repository::Destination) {
            return None;
        }
        for row in rows {
            if row.entry.at != fault.at {
                continue;
            }
            for entry in &self.entries {
                let revision::Class::Tree { task: name, path } = revision::class(&entry.path) else {
                    continue;
                };
                if name == task && row.entry.key.strip_prefix(b"/") == Some(path.strip_suffix(b".j2").unwrap_or(path)) {
                    return Some(Place { path: entry.path.clone(), position: None });
                }
            }
        }
        None
    }
}

pub(super) fn fault_capacity(expansion: &Expansion, listed: usize) -> u32 {
    let mut capacity = u32::try_from(listed).expect("listing bound");
    for task in &expansion.tasks {
        for group in &task.groups {
            if group.map == Map::Files {
                capacity = capacity.saturating_add(u32::try_from(group.entries.len()).expect("expanded count"));
            }
        }
    }
    capacity
}
