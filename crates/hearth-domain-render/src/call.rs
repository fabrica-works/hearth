//! One render call: distinct reads, ordered answers (render.md, 2.2, 8.2).
use crate::{
    compose::{self, Target},
    errors::{self, Decode, Limit},
    limits::Limits,
    parts::{self, Spec, meta},
    revision::{self, Budget, Documents, Entry, Mode, Oid},
    vocabulary::{Answer, As, Blob, Decoded, Outcome, Output, Request},
};
use alloc::boxed::Box;
use skein_lib::{List, Map, Queue, Token};
pub(crate) type Sizes = Map<hearth_lib::encode::Digest, u64>;
#[derive(Debug)]
pub(crate) struct Side {
    pub repository: Box<[u8]>,
    pub names: Box<[Box<[u8]>]>,
    pub commit: Option<Oid>,
    pub targets: Box<[Target]>,
    pub documents: Documents,
    pub errors: errors::Collector,
    pub error_next: u32,
    pub paths: Box<[Box<[u8]>]>,
    pub settings: Box<[compose::Settings]>,
    pub merge: compose::Merge,
    pub defaults: crate::schema::Defaults,
    pub budget: Budget,
    pub pending: Box<[Blob]>,
    pub entries: Box<[Entry]>,
    pub invalid: Box<[bool]>,
    pub refused: Map<Oid, ()>,
    pub cached: Map<Oid, hearth_lib::encode::Digest>,
    pub sizes: Sizes,
    pub files: bool,
    pub read: usize,
    pub next: usize,
    pub ready: bool,
    pub outcome: Outcome,
    pub made: Map<Box<[u8]>, crate::diff::Made>,
    pub diff: bool,
}
impl Side {
    pub fn new(repository: Box<[u8]>, names: Box<[Box<[u8]>]>, limits: &Limits) -> Side {
        Side {
            repository,
            names,
            commit: None,
            targets: Box::new([]),
            documents: Documents::new(&[]),
            errors: errors::Collector::new(limits),
            error_next: 0,
            paths: Box::new([]),
            settings: Box::new([]),
            merge: compose::Merge::new(limits),
            defaults: crate::schema::Defaults::new(),
            budget: Budget::new(limits),
            pending: Box::new([]),
            entries: Box::new([]),
            invalid: Box::new([]),
            refused: Map::with_capacity(0),
            cached: Map::with_capacity(0),
            sizes: Map::with_capacity(0),
            files: false,
            read: 0,
            next: 0,
            ready: false,
            outcome: Outcome::Done,
            made: Map::with_capacity(0),
            diff: false,
        }
    }
    pub fn listed(&mut self, limits: Limits, commit: Oid, entries: &[Entry], out: &mut Queue<Output>) {
        self.commit = Some(commit);
        self.invalid = match compose::listing(entries, &limits, &mut self.errors) {
            Ok(invalid) => invalid,
            Err(()) => {
                self.fail(Outcome::Failed);
                return;
            }
        };
        if self.errors.outcome() != Outcome::Done {
            self.outcome = self.outcome.worst(Outcome::Failed);
        }
        self.entries = entries.into();
        self.documents = Documents::new(entries);
        self.read_documents(limits, out);
    }
    fn discover(&mut self, limits: Limits) {
        let discovery = match compose::targets(&self.documents, &limits, &mut self.errors) {
            Ok(discovery) => discovery,
            Err(compose::TooMany) => {
                self.fail(Outcome::Failed);
                return;
            }
        };
        let mut selected = List::with_capacity(limits.targets);
        for target in &discovery.targets {
            if self.names.is_empty() || self.names.contains(&target.name) {
                selected.push(target.clone()).expect("target limit checked");
            }
        }
        for name in &self.names {
            let mut known = false;
            for target in &discovery.names {
                if *target == *name {
                    known = true;
                }
            }
            if !known && !self.diff && !self.hosts_refused() {
                self.outcome = self.outcome.worst(Outcome::Usage);
            }
        }
        self.targets = selected.into_boxed();
        self.made = Map::with_capacity(limits.targets);
        for name in &discovery.names {
            if !self.names.is_empty() && !self.names.contains(name) {
                continue;
            }
            self.made
                .insert(name.clone(), crate::diff::Made::Failed(crate::vocabulary::Class::Configuration))
                .expect("target capacity");
        }
    }
    fn prepare(&mut self, limits: Limits, out: &mut Queue<Output>) -> bool {
        if !self.files {
            self.discover(limits);
        }
        self.files(limits, out)
    }
    fn hosts_refused(&self) -> bool {
        for (entry, invalid) in self.entries.iter().zip(&self.invalid) {
            if *invalid && revision::base(&entry.path).is_some() {
                return true;
            }
            if let revision::Class::Host { .. } = revision::class(&entry.path)
                && let Some(index) = self.documents.find(&entry.path)
                && self.documents.get(index).is_none()
            {
                return true;
            }
        }
        false
    }
    fn read_documents(&mut self, limits: Limits, out: &mut Queue<Output>) {
        let mut blobs = List::with_capacity(u32::try_from(self.entries.len()).expect("listing capacity"));
        let mut paths = List::with_capacity(u32::try_from(self.entries.len()).expect("listing capacity"));
        for entry in self.documents.entries() {
            if entry.size > u64::from(limits.document) {
                errors::push(
                    &mut self.errors,
                    errors::refused(
                        &entry.path,
                        None,
                        Decode::TooLarge { limit: Limit::Document, max: u64::from(limits.document) },
                        None,
                    ),
                );
                self.outcome = self.outcome.worst(Outcome::Failed);
            } else if !self.budget.take(entry.size) {
                errors::push(
                    &mut self.errors,
                    errors::refused(
                        &entry.path,
                        None,
                        Decode::TooLarge { limit: Limit::Decoded, max: limits.decoded },
                        None,
                    ),
                );
                self.fail(Outcome::Failed);
                return;
            } else {
                blobs
                    .push(Blob {
                        oid: entry.oid.clone(),
                        size: entry.size,
                        kind: match revision::class(&entry.path) {
                            revision::Class::Declaration { .. } => As::Declaration,
                            revision::Class::Fleet
                            | revision::Class::Environment { .. }
                            | revision::Class::Host { .. }
                            | revision::Class::Tree { .. }
                            | revision::Class::Template { .. }
                            | revision::Class::Script { .. }
                            | revision::Class::Readme { .. }
                            | revision::Class::Refused { .. }
                            | revision::Class::Other => As::Document,
                        },
                    })
                    .expect("document count");
                paths.push(entry.path.clone()).expect("document count");
            }
        }
        self.paths = paths.into_boxed();
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
            Decoded::Template(_) => {}
            Decoded::Document(document) => {
                self.documents.set(oid, document);
            }
            Decoded::Declaration(declaration) => {
                self.documents.set_declaration(oid, declaration);
            }
            Decoded::Refused { at, fault, also } => {
                if expected.kind == As::Template {
                    self.refused.insert(oid.clone(), ()).expect("listing capacity");
                    for entry in &self.entries {
                        let asked = match revision::class(&entry.path) {
                            revision::Class::Tree { task, .. } => self.needs(task),
                            revision::Class::Fleet
                            | revision::Class::Environment { .. }
                            | revision::Class::Host { .. }
                            | revision::Class::Declaration { .. }
                            | revision::Class::Template { .. }
                            | revision::Class::Script { .. }
                            | revision::Class::Readme { .. }
                            | revision::Class::Refused { .. }
                            | revision::Class::Other => false,
                        };
                        if asked && &entry.oid == oid && entry.path.ends_with(b".j2") {
                            errors::push(&mut self.errors, errors::refused(&entry.path, Some(at), fault.clone(), also));
                        }
                    }
                } else {
                    let path = self.paths.get(self.read).expect("requested path");
                    errors::push(&mut self.errors, errors::refused(path, Some(at), fault, also));
                }
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
        self.sizes.insert(digest, size).expect("listing capacity");
        self.read = self.read.saturating_add(1);
        self.ready = self.read == self.pending.len();
    }
    fn needs(&self, name: &[u8]) -> bool {
        for settings in &self.settings {
            for task in &settings.tasks {
                if task.name.as_ref() == name {
                    return true;
                }
            }
        }
        false
    }
    fn compose(&mut self, limits: Limits) {
        let mut faults = List::with_capacity(16);
        self.defaults = crate::schema::declarations(
            &self.documents,
            &mut crate::schema::Validator::new(&limits),
            &mut self.merge,
            &mut faults,
        );
        for (task, fault) in &faults {
            errors::push(
                &mut self.errors,
                errors::Error {
                    side: None,
                    kind: errors::Kind::Schema(fault.schema.clone()),
                    class: crate::vocabulary::Class::Configuration,
                    at: Some(errors::Place {
                        path: skein_lib::bytes::copy_of(self.documents.path(fault.document)),
                        position: Some(fault.at),
                    }),
                    also: None,
                    target: None,
                    task: Some(task.clone()),
                },
            );
        }
        let mut settings = List::with_capacity(u32::try_from(self.targets.len()).expect("target count"));
        for target in &self.targets {
            settings.push(compose::compose(target, &self.documents, &mut self.merge)).expect("target count");
        }
        self.settings = settings.into_boxed();
    }
    pub fn files(&mut self, limits: Limits, out: &mut Queue<Output>) -> bool {
        if self.files {
            return false;
        }
        self.files = true;
        self.compose(limits);
        let capacity = u32::try_from(self.entries.len()).expect("listing capacity");
        self.cached = Map::with_capacity(capacity);
        self.refused = Map::with_capacity(capacity);
        self.sizes = Map::with_capacity(capacity);
        let mut blobs = List::with_capacity(capacity);
        let mut paths = List::with_capacity(capacity);
        for entry in &self.entries {
            let kind = match revision::class(&entry.path) {
                revision::Class::Tree { task, path }
                    if !path.ends_with(b".j2") && entry.mode != Mode::Link && entry.mode != Mode::Submodule =>
                {
                    Some((task, As::Cached, limits.static_file))
                }
                revision::Class::Tree { task, path }
                    if path.ends_with(b".j2")
                        && entry.mode != Mode::Link
                        && entry.mode != Mode::Submodule
                        && path != b".j2"
                        && !path.ends_with(b"/.j2") =>
                {
                    Some((task, As::Template, limits.template))
                }
                revision::Class::Declaration { .. }
                | revision::Class::Fleet
                | revision::Class::Environment { .. }
                | revision::Class::Host { .. }
                | revision::Class::Tree { .. }
                | revision::Class::Template { .. }
                | revision::Class::Script { .. }
                | revision::Class::Readme { .. }
                | revision::Class::Refused { .. }
                | revision::Class::Other => None,
            };
            if let Some((task, kind, max)) = kind
                && self.needs(task)
            {
                if entry.size > u64::from(max) {
                    let limit = match kind {
                        As::Declaration | As::Document => Limit::Document,
                        As::Cached => Limit::Static,
                        As::Template => Limit::Template,
                    };
                    errors::push(
                        &mut self.errors,
                        errors::refused(&entry.path, None, Decode::TooLarge { limit, max: u64::from(max) }, None),
                    );
                    self.outcome = self.outcome.worst(Outcome::Failed);
                } else if kind == As::Template && !self.budget.take(entry.size) {
                    errors::push(
                        &mut self.errors,
                        errors::refused(
                            &entry.path,
                            None,
                            Decode::TooLarge { limit: Limit::Decoded, max: limits.decoded },
                            None,
                        ),
                    );
                    self.fail(Outcome::Failed);
                    return false;
                } else if !contains(blobs.as_slice(), &entry.oid, kind) {
                    blobs.push(Blob { oid: entry.oid.clone(), size: entry.size, kind }).expect("listing capacity");
                    paths.push(entry.path.clone()).expect("listing capacity");
                }
            }
        }
        self.paths = paths.into_boxed();
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
        self.files = true;
        let mut failed = Map::with_capacity(self.made.capacity());
        for (name, _) in &self.made {
            let class = if outcome == Outcome::TryAgain {
                crate::vocabulary::Class::Operational
            } else {
                crate::vocabulary::Class::Configuration
            };
            failed.insert(name.clone(), crate::diff::Made::Failed(class)).expect("target capacity");
        }
        self.made = failed;
        self.outcome = self.outcome.worst(outcome);
        self.ready = true;
    }
    fn config(&self, settings: &compose::Settings) -> Option<parts::config::Config> {
        let mut valid = true;
        let mut tree = Map::with_capacity(u32::try_from(self.entries.len()).expect("listing capacity"));
        for task in &settings.tasks {
            let task = &task.name;
            let mut declared = false;
            for entry in &self.entries {
                match revision::class(&entry.path) {
                    revision::Class::Declaration { task: name } if name == task.as_ref() => {
                        declared = match self.documents.declaration(name) {
                            Some((_, declaration)) => declaration.is_some() && self.defaults.sound(name),
                            None => false,
                        };
                    }
                    revision::Class::Tree { task: name, path } if name == task.as_ref() => {
                        if entry.mode == Mode::Submodule
                            || self.refused.contains_key(&entry.oid)
                            || (path.ends_with(b".j2") && !contains(&self.pending, &entry.oid, As::Template))
                        {
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
                    | revision::Class::Template { .. }
                    | revision::Class::Script { .. }
                    | revision::Class::Readme { .. }
                    | revision::Class::Refused { .. }
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
    fn repository_valid(&self, target: &Target) -> bool {
        for (entry, invalid) in self.entries.iter().zip(&self.invalid) {
            if !invalid {
                continue;
            }
            if let Some(task) = revision::task(&entry.path) {
                if let Some(settings) = self.settings.get(self.next.saturating_sub(1)) {
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
    pub fn answer(&mut self) -> Answer {
        for target in self.targets.get(self.next..).expect("answer cursor within targets") {
            self.next = self.next.saturating_add(1);
            let mut valid = self.repository_valid(target);
            for index in [target.fleet, target.base_document, Some(target.environment_document), Some(target.base_env)]
                .into_iter()
                .flatten()
            {
                match self.documents.get(index) {
                    Some(_) => {}
                    None => {
                        valid = false;
                    }
                }
            }
            if !valid {
                continue;
            }
            let Some(p) = compose::placement(target, &self.documents) else {
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
                member: target.member.clone(),
                environment: target.environment.clone(),
                placement: meta::Placement { kind: p.kind, firmware: p.firmware, hardware: p.hardware, on },
            };
            let meta_digest = meta::digest(&meta);
            let Some(config) = self.config(self.settings.get(self.next.saturating_sub(1)).expect("target settings"))
            else {
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
        if !self.diff
            && let Some(error) = self.errors.get(self.error_next)
        {
            self.error_next = self.error_next.saturating_add(1);
            return Answer::Error(error.clone());
        }
        Answer::Rendered {
            outcome: self.outcome.worst(self.errors.outcome()),
            diff: None,
            left_out: self.errors.left_out(),
        }
    }
}

fn contains(blobs: &[Blob], oid: &Oid, kind: As) -> bool {
    for blob in blobs {
        if &blob.oid == oid && blob.kind == kind {
            return true;
        }
    }
    false
}

/// A render streams one side; a diff renders from, then to, before comparing.
#[derive(Debug)]
pub(crate) struct Call {
    pub side: Side,
    pub diff: Option<Comparison>,
}
#[derive(Debug)]
pub(crate) struct Comparison {
    revision: Option<Box<[u8]>>,
    from: Option<RenderedSide>,
    names: Box<[Box<[u8]>]>,
    next: usize,
    tally: Option<crate::diff::Tally>,
    outcome: Outcome,
    pending: Option<crate::diff::Change>,
    entry: usize,
    left_out: u32,
}
impl Call {
    pub fn new(repository: Box<[u8]>, names: Box<[Box<[u8]>]>, limits: &Limits) -> Self {
        Self { side: Side::new(repository, names, limits), diff: None }
    }
    pub fn diff(repository: Box<[u8]>, names: Box<[Box<[u8]>]>, to: Box<[u8]>, limits: &Limits) -> Self {
        let mut call = Self::new(repository, names, limits);
        call.side.diff = true;
        call.side.errors.side(crate::diff::Side::From);
        call.diff = Some(Comparison {
            revision: Some(to),
            from: None,
            names: Box::new([]),
            next: 0,
            tally: None,
            outcome: Outcome::Done,
            pending: None,
            entry: 0,
            left_out: call.side.errors.left_out(),
        });
        call
    }
    pub fn hunk_failure(&mut self, class: crate::vocabulary::Class, failure: crate::vocabulary::Failure) {
        let target = self.diff.as_ref().expect("diff").pending.as_ref().expect("change").target.clone();
        if let crate::vocabulary::Failure::Tool(tool) = failure {
            self.side.errors.push(errors::Error {
                side: None,
                kind: errors::Kind::Tool(tool),
                class,
                at: None,
                also: None,
                target: Some(target),
                task: None,
            });
        }
        self.hunks(None);
    }
    pub fn hunks(&mut self, hunks: Option<crate::diff::Hunks>) {
        let diff = self.diff.as_mut().expect("diff hunks");
        let change = diff.pending.as_mut().expect("pending change");
        let entry =
            crate::diff::tree(&mut change.change).get_mut(diff.entry.saturating_sub(1)).expect("requested file");
        if hunks.is_none() {
            diff.outcome = diff.outcome.worst(Outcome::TryAgain);
        }
        match entry {
            parts::config::EntryChange::Changed { hunks: kept, .. } => *kept = hunks,
            parts::config::EntryChange::Added { .. } | parts::config::EntryChange::Removed { .. } => {
                unreachable!("changed file")
            }
        }
        self.side.ready = true;
    }
    pub fn resume(&mut self, limits: Limits, out: &mut Queue<Output>) -> bool {
        if self.side.prepare(limits, out) {
            return false;
        }
        let Some(diff) = &mut self.diff else {
            let answer = self.side.answer();
            let done = match &answer {
                Answer::Rendered { .. } => true,
                Answer::Spec(_) | Answer::Change(_) | Answer::Error(_) => false,
            };
            out.push(Output::Answer(answer));
            return done;
        };
        if diff.tally.is_none() {
            match self.side.answer() {
                Answer::Spec(spec) => {
                    self.side
                        .made
                        .insert(spec.meta.target.clone(), crate::diff::Made::Spec(Box::new(spec)))
                        .expect("target capacity");
                    return false;
                }
                Answer::Rendered { outcome, diff: _, left_out } => {
                    diff.outcome = diff.outcome.worst(outcome);
                    diff.left_out = left_out;
                }
                Answer::Error(error) => {
                    out.push(Output::Answer(Answer::Error(error)));
                    return false;
                }
                Answer::Change(_) => unreachable!("a side renders specs"),
            }
            let Some(commit) = self.side.commit.clone() else {
                return diff.finish(&mut self.side, out);
            };
            if let Some(revision) = diff.revision.take() {
                let made = core::mem::replace(&mut self.side.made, Map::with_capacity(0));
                let sizes = core::mem::replace(&mut self.side.sizes, Map::with_capacity(0));
                diff.from = Some(RenderedSide { commit, made, sizes });
                let errors = core::mem::replace(&mut self.side.errors, errors::Collector::new(&limits));
                self.side = Side::new(self.side.repository.clone(), self.side.names.clone(), &limits);
                self.side.errors = errors;
                self.side.errors.side(crate::diff::Side::To);
                self.side.diff = true;
                out.push(Output::Request(Request::List {
                    request: Token::new(1),
                    repository: self.side.repository.clone(),
                    revision,
                }));
                return false;
            }
            let RenderedSide { commit: from, made, .. } = diff.from.as_ref().expect("from rendered first");
            let mut names = Map::with_capacity(limits.targets.saturating_mul(2));
            for (name, _) in made.into_iter().chain(&self.side.made) {
                names.insert(name.clone(), ()).expect("two sides' targets");
            }
            for name in &self.side.names {
                if !names.contains_key(name) {
                    diff.outcome = diff.outcome.worst(Outcome::Usage);
                }
            }
            let mut ordered = List::with_capacity(names.len());
            for (name, ()) in &names {
                ordered.push(name.clone()).expect("union capacity");
            }
            diff.names = ordered.into_boxed();
            diff.tally = Some(crate::diff::Tally {
                from: from.clone(),
                to: commit,
                targets: names.len(),
                changed: 0,
                unchanged: 0,
            });
        }
        diff.answer(&mut self.side, limits, out)
    }
}

#[derive(Debug)]
struct RenderedSide {
    commit: Oid,
    made: Map<Box<[u8]>, crate::diff::Made>,
    sizes: Sizes,
}

impl Comparison {
    fn answer(&mut self, side: &mut Side, limits: Limits, out: &mut Queue<Output>) -> bool {
        if self.pending.is_some() {
            let (output, ready) = self.resume_hunks(&side.sizes, limits.hunk_file);
            side.ready = ready;
            out.push(output);
            return false;
        }
        if let Some(target) = self.names.get(self.next) {
            self.next = self.next.saturating_add(1);
            let RenderedSide { made: from, .. } = self.from.as_ref().expect("from side");
            let change = crate::diff::compare(from.get(target), side.made.get(target), crate::diff::Needs::Needed);
            let tally = self.tally.as_mut().expect("both sides listed");
            match &change {
                crate::diff::TargetChange::Same => tally.unchanged = tally.unchanged.saturating_add(1),
                crate::diff::TargetChange::Failed { .. } => {}
                crate::diff::TargetChange::Added { .. }
                | crate::diff::TargetChange::Removed
                | crate::diff::TargetChange::Changed { .. } => {
                    tally.changed = tally.changed.saturating_add(1);
                    self.outcome = self.outcome.worst(Outcome::Negative);
                }
            }
            self.pending = Some(crate::diff::Change { target: target.clone(), change });
            self.entry = 0;
            return false;
        }
        self.finish(side, out)
    }
    fn finish(&mut self, side: &mut Side, out: &mut Queue<Output>) -> bool {
        if let Some(error) = side.errors.get(side.error_next) {
            side.error_next = side.error_next.saturating_add(1);
            out.push(Output::Answer(Answer::Error(error.clone())));
            return false;
        }
        self.left_out = side.errors.left_out();
        self.outcome = self.outcome.worst(side.errors.outcome());
        out.push(Output::Answer(self.terminal()));
        true
    }

    fn terminal(&mut self) -> Answer {
        Answer::Rendered { outcome: self.outcome, diff: self.tally.take(), left_out: self.left_out }
    }
    fn resume_hunks(&mut self, sizes: &Sizes, max: u32) -> (Output, bool) {
        let change = self.pending.as_mut().expect("pending change");
        let from = self.from.as_ref().expect("from rendered");
        for entry in crate::diff::tree(&mut change.change).get_mut(self.entry..).expect("entry cursor") {
            self.entry = self.entry.saturating_add(1);
            if let Some((before, after)) = entry.pair(&from.sizes, sizes, max) {
                return (Output::Request(Request::Hunks { request: Token::new(3), before, after }), false);
            }
        }
        (Output::Answer(Answer::Change(self.pending.take().expect("pending change"))), true)
    }
}
