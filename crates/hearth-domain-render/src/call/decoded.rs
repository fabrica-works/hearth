//! Charge decoded ownership before retaining an arrival (render.md, 2.3).
use super::Side;
use crate::{
    errors::{self, Decode, Limit},
    revision::{self, Oid},
    vocabulary::{As, Decoded, Outcome},
};
impl Side {
    pub fn decoded(&mut self, oid: &Oid, read: Decoded) {
        let expected = self
            .pending
            .get(usize::try_from(self.read).expect("bounded cursor"))
            .expect("one event per requested blob");
        assert_eq!(&expected.oid, oid, "blobs answer in request order");
        if !self.admit_decoded(oid, &read) {
            self.read = self.read.saturating_add(1);
            self.ready =
                self.reuse.is_some() || usize::try_from(self.read).expect("bounded cursor") == self.pending.len();
            return;
        }
        let expected = self.pending.get(usize::try_from(self.read).expect("bounded cursor")).expect("pending blob");
        match read {
            Decoded::Link(target) => {
                self.links.insert(oid.clone(), target).expect("listing capacity");
            }
            Decoded::Template(template) => {
                self.check_template(oid, &template);
                self.templates.insert(oid.clone(), template).expect("listing capacity");
            }
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
                            revision::Class::Tree { task, .. }
                            | revision::Class::Template { task, .. }
                            | revision::Class::Script { task, .. } => self.needs(task),
                            revision::Class::Fleet
                            | revision::Class::Environment { .. }
                            | revision::Class::Host { .. }
                            | revision::Class::Declaration { .. }
                            | revision::Class::Readme { .. }
                            | revision::Class::Refused { .. }
                            | revision::Class::Other => false,
                        };
                        if asked && &entry.oid == oid && entry.path.ends_with(b".j2") {
                            errors::push(&mut self.errors, errors::refused(&entry.path, Some(at), fault.clone(), also));
                        }
                    }
                } else {
                    for entry in self.documents.entries() {
                        let declared = revision::task(&entry.path).is_some();
                        if &entry.oid == oid && declared == (expected.kind == As::Declaration) {
                            self.errors.push(errors::refused(&entry.path, Some(at), fault.clone(), also));
                        }
                    }
                }
                self.outcome = self.outcome.worst(Outcome::Failed);
            }
        }
        self.read = self.read.saturating_add(1);
        self.ready = self.reuse.is_some() || usize::try_from(self.read).expect("bounded cursor") == self.pending.len();
    }
    fn admit_decoded(&mut self, oid: &Oid, read: &Decoded) -> bool {
        if self.budget.exceeded() {
            return false;
        }
        let bytes = revision::decoded_size(read, &self.limits);
        let mut failed = None;
        match read {
            Decoded::Document(_) | Decoded::Declaration(_) => {
                for entry in self.documents.entries() {
                    if &entry.oid != oid {
                        continue;
                    }
                    let declared = revision::task(&entry.path).is_some();
                    let applies = match read {
                        Decoded::Document(_) => !declared,
                        Decoded::Declaration(_) => declared,
                        Decoded::Template(_) | Decoded::Link(_) | Decoded::Refused { .. } => false,
                    };
                    if applies && !self.budget.take(bytes) {
                        failed = Some(entry.path.clone());
                        break;
                    }
                }
            }
            Decoded::Template(_) => {
                if !self.budget.take(bytes) {
                    failed = Some(
                        self.paths.get(usize::try_from(self.read).expect("bounded cursor")).expect("read path").clone(),
                    );
                }
            }
            Decoded::Link(_) | Decoded::Refused { .. } => {}
        }
        if let Some(reuse) = &mut self.reuse {
            reuse.fit(self.budget.remaining());
        }
        if let Some(path) = failed {
            self.errors.push(errors::refused(
                &path,
                None,
                Decode::TooLarge { limit: Limit::Decoded, max: self.limits.decoded },
                None,
            ));
            self.fail(Outcome::Failed);
            return false;
        }
        true
    }
}
