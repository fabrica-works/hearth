//! One render call: distinct reads, ordered answers (render.md, 2.2, 8.2).
use crate::{
    compose::{self, Target},
    limits::Limits,
    parts::{self, Spec, meta},
    revision::{Document, Entry, Oid},
    vocabulary::{Answer, Blob, Decoded, Outcome, Output, Request},
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
        let capacity = limits.targets.saturating_mul(2);
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
            Decoded::Refused { at: _ } => {
                self.outcome = self.outcome.worst(Outcome::Failed);
            }
        }
        self.read = self.read.saturating_add(1);
        self.ready = self.read == self.pending.len();
    }
    pub fn fail(&mut self, outcome: Outcome) {
        self.targets = Box::new([]);
        self.outcome = self.outcome.worst(outcome);
        self.ready = true;
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
            return Answer::Spec(Spec {
                digest: parts::spec_digest(&meta_digest, &[]),
                meta,
                meta_digest,
                commit: self.commit.clone().expect("listed commit"),
                parts: Box::new([]),
            });
        }
        Answer::Rendered { outcome: self.outcome }
    }
}
