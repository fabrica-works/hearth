//! One render call: distinct reads, ordered answers (render.md, 2.2, 8.2).
use crate::{
    compose::{self, Target},
    limits::Limits,
    parts::{self, Spec, meta},
    revision::{self, Document, Entry, Mode, Oid},
    vocabulary::{Answer, As, Blob, Decoded, Outcome, Output, Request},
};
use alloc::boxed::Box;
use skein_lib::{List, Map, Queue, Token};
#[derive(Debug)]
pub(crate) struct Call {
    pub repository: Box<[u8]>,
    pub names: Box<[Box<[u8]>]>,
    pub commit: Option<Oid>,
    pub targets: Box<[Target]>,
    pub documents: Map<Oid, Option<Document>>,
    pub pending: Box<[Blob]>,
    pub entries: Box<[Entry]>,
    pub cached: Map<Oid, hearth_lib::encode::Digest>,
    pub declarations: Map<Oid, bool>,
    pub files: bool,
    pub read: usize,
    pub next: usize,
    pub ready: bool,
    pub outcome: Outcome,
}
impl Call {
    pub fn new(repository: Box<[u8]>, names: Box<[Box<[u8]>]>) -> Call {
        Call {
            repository,
            names,
            commit: None,
            targets: Box::new([]),
            documents: Map::with_capacity(0),
            pending: Box::new([]),
            entries: Box::new([]),
            cached: Map::with_capacity(0),
            declarations: Map::with_capacity(0),
            files: false,
            read: 0,
            next: 0,
            ready: false,
            outcome: Outcome::Done,
        }
    }
    pub fn listed(&mut self, limits: Limits, commit: Oid, entries: &[Entry], out: &mut Queue<Output>) {
        self.commit = Some(commit);
        let targets = match compose::targets(entries) {
            Ok(targets) => targets,
            Err(compose::TooMany) => {
                self.fail(Outcome::Failed);
                return;
            }
        };
        if targets.len() > usize::try_from(limits.targets).expect("u32 fits usize") {
            self.fail(Outcome::Failed);
            return;
        }
        let mut selected = List::with_capacity(limits.targets);
        for target in &targets {
            if self.names.is_empty() || self.names.contains(&target.name) {
                selected.push(target.clone()).expect("target limit checked");
            }
        }
        for name in &self.names {
            let mut known = false;
            for target in &targets {
                if target.name == *name {
                    known = true;
                }
            }
            if !known {
                self.outcome = self.outcome.worst(Outcome::Usage);
            }
        }
        self.targets = selected.into_boxed();
        self.entries = entries.into();
        let capacity = limits.targets.saturating_mul(3);
        self.documents = Map::with_capacity(capacity);
        let mut blobs = List::with_capacity(capacity);
        for target in &self.targets {
            for blob in &target.documents {
                if self.documents.contains_key(&blob.oid) {
                    continue;
                }
                self.documents.insert(blob.oid.clone(), None).expect("two documents per target");
                if blob.size <= u64::from(limits.document) {
                    blobs.push(blob.clone()).expect("two documents per target");
                } else {
                    self.outcome = self.outcome.worst(Outcome::Failed);
                }
            }
        }
        for entry in entries {
            if let revision::Class::Host { base, name } = revision::class(&entry.path)
                && base == name
                && self.has_base(base)
                && !self.documents.contains_key(&entry.oid)
            {
                self.documents.insert(entry.oid.clone(), None).expect("three documents per target");
                if entry.size <= u64::from(limits.document) {
                    blobs
                        .push(Blob { oid: entry.oid.clone(), size: entry.size, kind: As::Document })
                        .expect("base document room");
                }
            }
        }
        self.pending = blobs.into_boxed();
        if self.pending.is_empty() {
            self.ready = true;
        } else {
            out.push(Output::Request(Request::Read {
                request: Token::new(2),
                repository: self.repository.clone(),
                commit: self.commit.clone().expect("listed commit"),
                blobs: self.pending.clone(),
            }));
        }
    }
    pub fn decoded(&mut self, oid: &Oid, read: Decoded) {
        let expected = self.pending.get(self.read).expect("one event per requested blob");
        assert_eq!(&expected.oid, oid, "blobs answer in request order");
        match read {
            Decoded::Document(document) => {
                *self.documents.get_mut(oid).expect("requested blob") = Some(document);
            }
            Decoded::Declaration => {
                self.declarations.insert(oid.clone(), true).expect("listed declarations");
            }
            Decoded::Refused { at: _ } => {
                self.outcome = self.outcome.worst(Outcome::Failed);
            }
        }
        self.read = self.read.saturating_add(1);
        self.ready = self.read == self.pending.len();
    }
    pub fn cached(&mut self, oid: &Oid, size: u64, digest: hearth_lib::encode::Digest) {
        let expected = self.pending.get(self.read).expect("one event per blob");
        assert_eq!((&expected.oid, expected.size, expected.kind), (oid, size, As::Cached), "cached blob");
        self.cached.insert(oid.clone(), digest).expect("listing capacity");
        self.read = self.read.saturating_add(1);
        self.ready = self.read == self.pending.len();
    }
    fn present(&self, target: &Target) -> Map<Box<[u8]>, bool> {
        let mut present = Map::with_capacity(u32::try_from(self.entries.len()).expect("listing capacity"));
        for base in [true, false] {
            for entry in &self.entries {
                if let revision::Class::Host { base: name, name: host } = revision::class(&entry.path)
                    && name == target.base.as_ref()
                    && ((base && host == name) || (!base && host == target.name.as_ref()))
                    && let Some(Some(document)) = self.documents.get(&entry.oid)
                {
                    for task in &document.layer.tasks {
                        present.insert(task.name.clone(), !task.removed).expect("task capacity");
                    }
                }
            }
        }
        present
    }
    fn has_base(&self, base: &[u8]) -> bool {
        for target in &self.targets {
            if target.base.as_ref() == base {
                return true;
            }
        }
        false
    }
    fn needs(&self, task: &[u8]) -> bool {
        for target in &self.targets {
            if self.present(target).get(task) == Some(&true) {
                return true;
            }
        }
        false
    }
    pub fn files(&mut self, limits: Limits, out: &mut Queue<Output>) -> bool {
        if self.files {
            return false;
        }
        self.files = true;
        let capacity = u32::try_from(self.entries.len()).expect("listing capacity");
        self.cached = Map::with_capacity(capacity);
        self.declarations = Map::with_capacity(capacity);
        let mut blobs = List::with_capacity(capacity);
        for entry in &self.entries {
            let kind = match revision::class(&entry.path) {
                revision::Class::Declaration { task } => Some((task, As::Declaration, limits.document)),
                revision::Class::Tree { task, path }
                    if !path.ends_with(b".j2") && entry.mode != Mode::Link && entry.mode != Mode::Submodule =>
                {
                    Some((task, As::Cached, limits.static_file))
                }
                revision::Class::Fleet
                | revision::Class::Environment { .. }
                | revision::Class::Host { .. }
                | revision::Class::Tree { .. }
                | revision::Class::Other => None,
            };
            if let Some((task, kind, max)) = kind
                && self.needs(task)
                && entry.size <= u64::from(max)
                && !contains(&blobs, &entry.oid, kind)
            {
                blobs.push(Blob { oid: entry.oid.clone(), size: entry.size, kind }).expect("listing capacity");
            }
        }
        self.pending = blobs.into_boxed();
        self.read = 0;
        if self.pending.is_empty() {
            return false;
        }
        self.ready = false;
        out.push(Output::Request(Request::Read {
            request: Token::new(2),
            repository: self.repository.clone(),
            commit: self.commit.clone().expect("commit"),
            blobs: self.pending.clone(),
        }));
        true
    }
    pub fn fail(&mut self, outcome: Outcome) {
        self.targets = Box::new([]);
        self.outcome = self.outcome.worst(outcome);
        self.ready = true;
    }
    fn config(&self, target: &Target) -> Option<parts::config::Config> {
        let present = self.present(target);
        let mut valid = true;
        let mut tree = Map::with_capacity(u32::try_from(self.entries.len()).expect("listing capacity"));
        for (task, is_present) in &present {
            if !is_present {
                continue;
            }
            let mut declared = false;
            for entry in &self.entries {
                match revision::class(&entry.path) {
                    revision::Class::Declaration { task: name } if name == task.as_ref() => {
                        declared = self.declarations.get(&entry.oid) == Some(&true);
                    }
                    revision::Class::Tree { task: name, path } if name == task.as_ref() => {
                        if entry.mode == Mode::Submodule {
                            valid = false;
                        } else if entry.mode != Mode::Link && !path.ends_with(b".j2") {
                            if let Some(digest) = self.cached.get(&entry.oid) {
                                let file = parts::config::file(path, entry.mode, *digest);
                                tree.insert(file.path.clone(), file).expect("listing capacity");
                            } else {
                                valid = false;
                            }
                        }
                    }
                    revision::Class::Fleet
                    | revision::Class::Environment { .. }
                    | revision::Class::Host { .. }
                    | revision::Class::Declaration { .. }
                    | revision::Class::Tree { .. }
                    | revision::Class::Other => {}
                }
            }
            valid &= declared;
        }
        if !valid {
            return None;
        }
        let mut files = List::with_capacity(tree.len());
        for (_, file) in &tree {
            files.push(file.clone()).expect("tree capacity");
        }
        Some(parts::config::Config { tree: files.into_boxed() })
    }
    pub fn answer(&mut self) -> Answer {
        for target in self.targets.get(self.next..).expect("answer cursor within targets") {
            self.next = self.next.saturating_add(1);
            let mut placement = None;
            let mut valid = true;
            for blob in &target.documents {
                match self.documents.get(&blob.oid).expect("target document registered") {
                    Some(document) => {
                        if let Some(p) = &document.layer.placement {
                            placement = Some(p.clone());
                        }
                    }
                    None => {
                        valid = false;
                    }
                }
            }
            if !valid {
                continue;
            }
            for entry in &self.entries {
                if let revision::Class::Host { base, name } = revision::class(&entry.path)
                    && base == name
                    && base == target.base.as_ref()
                    && self.documents.get(&entry.oid) == Some(&None)
                {
                    valid = false;
                }
            }
            if !valid {
                continue;
            }
            let Some(p) = placement else {
                self.outcome = self.outcome.worst(Outcome::Failed);
                continue;
            };
            let on = match p.on {
                Some(on) => Some(on.target),
                None => None,
            };
            let meta = meta::Meta {
                target: target.name.clone(),
                base: target.base.clone(),
                member: None,
                environment: target.environment.clone(),
                placement: meta::Placement { kind: p.kind, firmware: p.firmware, hardware: p.hardware, on },
            };
            let meta_digest = meta::digest(&meta);
            let Some(config) = self.config(target) else {
                self.outcome = self.outcome.worst(Outcome::Failed);
                continue;
            };
            let parts = Box::new([parts::Part {
                digest: parts::config::digest(&config),
                record: parts::PartRecord::Config(config),
            }]);
            return Answer::Spec(Spec {
                digest: parts::spec_digest(&meta_digest, parts.as_ref()),
                meta,
                meta_digest,
                commit: self.commit.clone().expect("listed commit"),
                parts,
            });
        }
        Answer::Rendered { outcome: self.outcome }
    }
}

fn contains(blobs: &List<Blob>, oid: &Oid, kind: As) -> bool {
    for blob in blobs {
        if &blob.oid == oid && blob.kind == kind {
            return true;
        }
    }
    false
}
