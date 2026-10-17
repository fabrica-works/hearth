//! Named templates and repository copies (format.md, 2.1, 5.5, 6.2).
use super::{Box, Class, Error, Kind, List, Map, Meta, Mode, Place, Plan, Repository, Side, Writer, copy_of, revision};
use crate::{
    entries::Fault,
    errors::{Format, Schema},
    kinds::files::Make,
    revision::{
        Entry,
        declaration::{Declaration, Literal, Name},
    },
    schema::forms,
    vocabulary::{As, Blob, Request},
};
use skein_lib::Token;
impl Side {
    pub(in crate::call) fn listed_entry(&self, path: &[u8]) -> Option<&Entry> {
        let mut listed = None;
        for entry in &self.entries {
            if entry.path.as_ref() == path {
                listed = Some(entry);
                break;
            }
        }
        listed
    }
    pub(super) fn sources(&self, plan: &mut Plan, faults: &mut List<Fault>) {
        for install in &mut plan.installs {
            let at = install.at;
            let fault = match &mut install.make {
                Make::Rendered { template, via: Some(_), .. } => {
                    let path = template_path(&plan.task, template);
                    let mut listed = false;
                    if let Some(entry) = self.listed_entry(&path)
                        && let revision::Class::Template { .. } = revision::class(&entry.path)
                    {
                        listed = entry.mode == Mode::File || entry.mode == Mode::Executable;
                    }
                    if listed {
                        *template = path;
                        None
                    } else {
                        Some(Kind::Repository(Repository::NoTemplate))
                    }
                }
                Make::Copied { source, .. } => self.copy_fault(source),
                Make::Rendered { via: None, .. } | Make::Static { .. } | Make::Link { .. } | Make::Directory { .. } => {
                    None
                }
            };
            if let Some(kind) = fault {
                faults.push(Fault { kind, at: at.expect("entry source"), also: None }).expect("fault capacity");
            }
        }
    }
    fn copy_fault(&self, source: &[u8]) -> Option<Kind> {
        if !forms::check(Format::File, source) {
            return Some(Kind::Schema(Schema::Malformed { format: Format::File }));
        }
        match self.listed_entry(source) {
            Some(Entry { mode: Mode::File | Mode::Executable, size, .. }) => {
                if *size > u64::from(self.limits.static_file) {
                    Some(Kind::Schema(Schema::Large { path: copy_of(source), max: u64::from(self.limits.static_file) }))
                } else {
                    None
                }
            }
            Some(Entry { mode: Mode::Link | Mode::Submodule, .. }) | None => {
                Some(Kind::Schema(Schema::File { path: copy_of(source) }))
            }
        }
    }
    pub(super) fn unused(&self, task: &[u8], declaration: &Declaration, meta: &Meta, errors: &mut List<Error>) -> bool {
        let mut valid = true;
        for entry in &self.entries {
            let revision::Class::Template { task: name, .. } = revision::class(&entry.path) else { continue };
            if name != task || named(declaration, task, &entry.path) {
                continue;
            }
            errors
                .push(Error {
                    side: None,
                    kind: Kind::Repository(Repository::Unused),
                    class: Class::Configuration,
                    at: Some(Place { path: entry.path.clone(), position: None }),
                    also: None,
                    target: Some(meta.target.clone()),
                    task: Some(copy_of(task)),
                })
                .expect("fault capacity");
            valid = false;
        }
        valid
    }
    pub(in crate::call) fn copies(&mut self, plans: &[Plan]) -> Option<Request> {
        let capacity = u32::try_from(self.entries.len()).expect("listing bound");
        let mut blobs = List::with_capacity(capacity);
        let mut paths = List::with_capacity(capacity);
        for plan in plans {
            for install in &plan.installs {
                let Make::Copied { source, .. } = &install.make else { continue };
                let entry = self.listed_entry(source).expect("checked copy");
                if self.cached.contains_key(&entry.oid)
                    || super::super::contains(blobs.as_slice(), &entry.oid, As::Cached)
                {
                    continue;
                }
                blobs.push(Blob { oid: entry.oid.clone(), size: entry.size, kind: As::Cached }).expect("listing bound");
                paths.push(source.clone()).expect("listing bound");
            }
        }
        if blobs.is_empty() {
            return None;
        }
        self.pending = blobs.into_boxed();
        self.paths = paths.into_boxed();
        self.read = 0;
        self.ready = false;
        Some(Request::Read {
            request: Token::new(2),
            repository: self.repository.clone(),
            commit: self.commit.clone().expect("listed commit"),
            blobs: self.pending.clone(),
        })
    }
}
fn named(declaration: &Declaration, task: &[u8], path: &[u8]) -> bool {
    for group in &declaration.entries {
        if group.map != Map::Files {
            continue;
        }
        for entry in &group.entries {
            for field in &entry.fields {
                if field.name == Name::Template
                    && let Literal::Plain { text, .. } = &field.value
                    && template_path(task, text).as_ref() == path
                {
                    return true;
                }
            }
        }
    }
    false
}
fn template_path(task: &[u8], name: &[u8]) -> Box<[u8]> {
    let mut path = Writer::new(task.len().saturating_add(name.len()).saturating_add(7));
    path.put(b"tasks/").expect("prefix room");
    path.put(task).expect("task room");
    path.put(b"/").expect("slash room");
    path.put(name).expect("name room");
    path.finish()
}
