//! Find distinct resolution keys after assembling every target (render.md, 5.7).
use super::Side;
use crate::{
    diff::Made,
    keys, parts,
    vocabulary::{Answer, Outcome, Output, Request},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{Map, Queue, Token};
#[derive(Debug)]
pub(crate) struct Finding {
    assembled: bool,
    next: usize,
    asked: Map<Digest, Option<Digest>>,
    pending: Option<Digest>,
}
impl Finding {
    pub fn new(capacity: u32) -> Self {
        Self { assembled: false, next: 0, asked: Map::with_capacity(capacity), pending: None }
    }
}
impl Side {
    pub(crate) fn resolved_answer(&mut self, out: &mut Queue<Output>) -> Option<Answer> {
        if !self.finding.assembled {
            match self.answer_for(out) {
                Some(Answer::Spec(mut spec)) => {
                    for part in &spec.parts {
                        if let parts::PartRecord::Packages(packages) = &part.record {
                            let key = keys::resolution(&parts::packages::inputs(packages));
                            spec.resolution = Some(Box::new(keys::Resolution::Needed { key }));
                        }
                    }
                    self.made.insert(spec.meta.target.clone(), Made::Spec(Box::new(spec))).expect("target capacity");
                    return None;
                }
                Some(Answer::Rendered { .. }) => self.finding.assembled = true,
                Some(Answer::Error(_)) => {
                    // finished advanced the cursor; keep the error until after the specs.
                    self.error_next = self.error_next.checked_sub(1).expect("one collected error");
                    self.finding.assembled = true;
                }
                answer @ Some(Answer::Change(_)) => return answer,
                None => return None,
            }
        }
        for (_, made) in &self.made {
            let Made::Spec(spec) = made else { continue };
            if let Some(resolution) = &spec.resolution {
                let key = resolution.key();
                if !self.finding.asked.contains_key(&key) {
                    self.finding.asked.insert(key, None).expect("target capacity");
                    self.finding.pending = Some(key);
                    self.ready = false;
                    out.push(Output::Request(Request::Find { request: Token::new(5), key }));
                    return None;
                }
            }
        }
        for (_, made) in self.made.into_iter().skip(self.finding.next) {
            self.finding.next = self.finding.next.saturating_add(1);
            let Made::Spec(spec) = made else { continue };
            let mut spec = spec.as_ref().clone();
            if let Some(resolution) = &spec.resolution {
                let key = resolution.key();
                if let Some(Some(digest)) = self.finding.asked.get(&key) {
                    let keys = keys::layers(
                        digest,
                        &keys::part(&spec, b"packages"),
                        &keys::part(&spec, b"software"),
                        &keys::part(&spec, b"config"),
                    );
                    spec.resolution = Some(Box::new(keys::Resolution::Known { key, keys }));
                }
            }
            return Some(Answer::Spec(spec));
        }
        Some(self.finished())
    }
    pub(crate) fn found(&mut self, digest: Option<Digest>, failed: bool) {
        let key = self.finding.pending.take().expect("one find in flight");
        *self.finding.asked.get_mut(&key).expect("asked key") = digest;
        if failed {
            self.outcome = self.outcome.worst(Outcome::TryAgain);
        }
        self.ready = true;
    }
}
