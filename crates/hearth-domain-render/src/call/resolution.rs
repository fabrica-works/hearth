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
#[derive(Debug, Hash)]
pub(crate) struct Finding {
    assembled: bool,
    next: u32,
    asked: Map<Digest, Option<crate::resolution::Record>>,
    pending: Option<Digest>,
    deadline: Option<skein_lib::Time>,
    attempted: Map<Digest, Option<(crate::vocabulary::Class, crate::errors::Kind)>>,
    closure: Option<crate::resolution::Closure>,
    fault: Option<crate::errors::Resolution>,
    recording: Option<crate::resolution::Record>,
}
impl Finding {
    pub fn resolve(&mut self, deadline: Option<skein_lib::Time>) {
        self.deadline = deadline;
    }
    pub fn inst(&mut self, inst: crate::resolution::Inst) {
        if self.fault.is_none() {
            self.fault = self.closure.as_mut().expect("active closure").inst(inst).err();
        }
    }
    pub fn stanza(&mut self, line: u32, stanza: crate::resolution::Stanza) {
        if self.fault.is_none() {
            self.fault = self.closure.as_mut().expect("active closure").stanza(line, stanza).err();
        }
    }
    pub fn new(capacity: u32) -> Self {
        Self {
            assembled: false,
            next: 0,
            asked: Map::with_capacity(capacity),
            pending: None,
            deadline: None,
            attempted: Map::with_capacity(capacity),
            closure: None,
            fault: None,
            recording: None,
        }
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
            if let Some(packages) = packages(spec)
                && packages.variant == parts::packages::Variant::Container
            {
                continue;
            }
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
        if let Some(deadline) = self.finding.deadline {
            for (_, made) in &self.made {
                let Made::Spec(spec) = made else { continue };
                let Some(packages) = packages(spec) else { continue };
                if packages.variant == parts::packages::Variant::Container {
                    continue;
                }
                let Some(resolution) = &spec.resolution else { continue };
                let key = resolution.key();
                if self.finding.asked.get(&key) == Some(&None) && !self.finding.attempted.contains_key(&key) {
                    self.ask_resolve(key, packages.clone(), deadline, out);
                    return None;
                }
            }
        }
        for (_, made) in self.made.into_iter().skip(usize::try_from(self.finding.next).expect("bounded cursor")) {
            self.finding.next = self.finding.next.saturating_add(1);
            let Made::Spec(spec) = made else { continue };
            let mut spec = spec.as_ref().clone();
            if let Some(resolution) = &spec.resolution {
                let key = resolution.key();
                if let Some(Some(record)) = self.finding.asked.get(&key) {
                    let keys = keys::layers(
                        &record.digest,
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
    pub(crate) fn record(
        &mut self,
        lines: Box<[crate::resolution::Line]>,
        tools: crate::resolution::Tools,
        costs: crate::resolution::Costs,
        at: skein_lib::Wall,
        out: &mut Queue<Output>,
    ) {
        let key = self.finding.pending.expect("active resolve");
        let closure = self.finding.closure.take().expect("active closure");
        let closed = match self.finding.fault.take() {
            Some(fault) => Err(fault),
            None => closure.close(requested_names(&self.made, &key)),
        };
        match closed {
            Ok(closure) => {
                let digest = keys::closure(&closure);
                let record = crate::resolution::Record {
                    key,
                    inputs: inputs(&self.made, &key),
                    sources: lines,
                    closure,
                    digest,
                    made: crate::resolution::Made { at, tools, costs },
                };
                self.finding.recording = Some(record.clone());
                out.push(Output::Request(Request::Record { request: Token::new(7), resolution: record }));
            }
            Err(fault) => self.resolution_fault(key, Some(fault), None),
        }
    }
    pub(crate) fn recorded(&mut self, failure: Option<(crate::vocabulary::Class, crate::vocabulary::Failure)>) {
        let key = self.finding.pending.take().expect("record in flight");
        let record = self.finding.recording.take().expect("record in flight");
        if failure.is_none() {
            *self.finding.asked.get_mut(&key).expect("asked key") = Some(record);
            self.ready = true;
        } else {
            self.resolution_fault(key, None, failure);
        }
    }
    pub(crate) fn resolve_failed(&mut self, failure: (crate::vocabulary::Class, crate::vocabulary::Failure)) {
        let key = self.finding.pending.take().expect("resolve in flight");
        self.finding.closure = None;
        self.finding.fault = None;
        self.resolution_fault(key, None, Some(failure));
    }
    fn resolution_fault(
        &mut self,
        key: Digest,
        fault: Option<crate::errors::Resolution>,
        failure: Option<(crate::vocabulary::Class, crate::vocabulary::Failure)>,
    ) {
        self.finding.pending = None;
        if let Some((_, crate::vocabulary::Failure::Store)) = &failure {
            self.outcome = self.outcome.worst(Outcome::TryAgain);
        }
        let fault = match fault {
            Some(fault) => {
                let class = match &fault {
                    crate::errors::Resolution::Unmatched { .. } | crate::errors::Resolution::Disagree { .. } => {
                        crate::vocabulary::Class::Operational
                    }
                    crate::errors::Resolution::Unknown { .. }
                    | crate::errors::Resolution::Unsatisfiable { .. }
                    | crate::errors::Resolution::Absent { .. }
                    | crate::errors::Resolution::Missing { .. }
                    | crate::errors::Resolution::TooLarge { .. }
                    | crate::errors::Resolution::TooMany { .. } => crate::vocabulary::Class::Configuration,
                };
                Some((class, crate::errors::Kind::Resolution(fault)))
            }
            None => classified_failure(failure),
        };
        if let Some((class, kind)) = fault {
            self.finding.attempted.insert(key, Some((class, kind.clone()))).expect("attempted key");
            self.resolution_error(key, class, &kind);
        }
        self.ready = true;
    }
    pub(crate) fn found(&mut self, record: Option<crate::resolution::Record>, failed: bool) {
        let key = self.finding.pending.take().expect("one find in flight");
        *self.finding.asked.get_mut(&key).expect("asked key") = record;
        if failed {
            self.outcome = self.outcome.worst(Outcome::TryAgain);
        }
        self.ready = true;
    }
}

fn packages(spec: &parts::Spec) -> Option<&parts::packages::Packages> {
    for part in &spec.parts {
        if let parts::PartRecord::Packages(packages) = &part.record {
            return Some(packages);
        }
    }
    None
}

fn failure_kind(failure: crate::vocabulary::Failure) -> Option<crate::errors::Kind> {
    use crate::{
        errors::{Decode, Kind, Limit, Resolution},
        resolution::Cause,
        vocabulary::Failure,
    };
    Some(match failure {
        Failure::Run { tool: _, cause: Some(Cause::Unknown(package)) } => {
            Kind::Resolution(Resolution::Unknown { package })
        }
        Failure::Run { tool: _, cause: Some(Cause::Unsatisfiable(package)) } => {
            Kind::Resolution(Resolution::Unsatisfiable { package })
        }
        Failure::Run { tool: _, cause: Some(Cause::Absent { uri, suite }) } => {
            Kind::Resolution(Resolution::Absent { uri, suite })
        }
        Failure::Run { tool, cause: None | Some(Cause::Fetch | Cause::Resolve | Cause::Space | Cause::Other) }
        | Failure::Tool(tool) => Kind::Tool(tool),
        Failure::TooLarge { limit: Limit::Closure, max } => {
            Kind::Resolution(Resolution::TooMany { max: u32::try_from(max).expect("closure limit is u32") })
        }
        Failure::TooLarge { limit: Limit::Record, max } => Kind::Resolution(Resolution::TooLarge { max }),
        Failure::TooLarge { limit, max } => Kind::Decode(Decode::TooLarge { limit, max }),
        Failure::Store => return None,
    })
}

fn requested_names<'a>(made: &'a Map<Box<[u8]>, Made>, key: &Digest) -> &'a [Box<[u8]>] {
    for (_, made) in made {
        if let Made::Spec(spec) = made
            && let Some(resolution) = &spec.resolution
            && resolution.key() == *key
            && let Some(packages) = packages(spec)
        {
            return &packages.request;
        }
    }
    unreachable!("resolution holder")
}
fn classified_failure(
    failure: Option<(crate::vocabulary::Class, crate::vocabulary::Failure)>,
) -> Option<(crate::vocabulary::Class, crate::errors::Kind)> {
    let (class, failure) = failure?;
    Some((class, failure_kind(failure)?))
}

fn inputs(made: &Map<Box<[u8]>, Made>, key: &Digest) -> parts::packages::Inputs {
    for (_, made) in made {
        if let Made::Spec(spec) = made
            && let Some(resolution) = &spec.resolution
            && resolution.key() == *key
            && let Some(packages) = packages(spec)
        {
            return parts::packages::inputs(packages);
        }
    }
    unreachable!("key has a holder")
}

impl Finding {
    pub(super) fn record(&self, key: &Digest) -> Option<&crate::resolution::Record> {
        self.asked.get(key)?.as_ref()
    }
    pub(super) fn inherit(&mut self, other: &Self) {
        for (key, record) in &other.asked {
            if record.is_some() && self.asked.get(key) == Some(&None) {
                self.asked.insert(*key, record.clone()).expect("existing key");
            }
        }
    }
}
impl Side {
    pub(super) fn resolve_target(&mut self, target: &[u8], deadline: skein_lib::Time, out: &mut Queue<Output>) -> bool {
        let Some(Made::Spec(spec)) = self.made.get(target) else { return false };
        let Some(resolution) = &spec.resolution else { return false };
        let key = resolution.key();
        let Some(packages) = packages(spec) else { return false };
        if packages.variant == parts::packages::Variant::Container
            || self.finding.record(&key).is_some()
            || self.finding.attempted.contains_key(&key)
        {
            return false;
        }
        self.ask_resolve(key, packages.clone(), deadline, out);
        true
    }
    fn ask_resolve(
        &mut self,
        key: Digest,
        packages: parts::packages::Packages,
        deadline: skein_lib::Time,
        out: &mut Queue<Output>,
    ) {
        self.finding.attempted.insert(key, None).expect("target capacity");
        self.finding.pending = Some(key);
        self.finding.closure = Some(crate::resolution::Closure::new(&self.limits));
        self.finding.fault = None;
        self.ready = false;
        out.push(Output::Request(Request::Resolve {
            request: Token::new(6),
            key,
            sources: packages.sources,
            names: packages.request,
            variant: packages.variant,
            apt: packages.apt,
            deadline,
        }));
    }
}

impl Side {
    pub(super) fn inherit_finding(&mut self, other: &Finding) {
        self.finding.inherit(other);
        for (key, failure) in &other.attempted {
            if self.finding.asked.contains_key(key) && !self.finding.attempted.contains_key(key) {
                if let Some((class, kind)) = failure {
                    self.resolution_error(*key, *class, kind);
                }
                self.finding.attempted.insert(*key, failure.clone()).expect("target capacity");
            }
        }
    }
    fn resolution_error(&mut self, key: Digest, class: crate::vocabulary::Class, kind: &crate::errors::Kind) {
        for (target, made) in &self.made {
            if let Made::Spec(spec) = made
                && let Some(resolution) = &spec.resolution
                && resolution.key() == key
            {
                self.errors.push(crate::errors::Error {
                    side: None,
                    kind: kind.clone(),
                    class,
                    at: None,
                    also: None,
                    target: Some(target.clone()),
                    task: None,
                });
            }
        }
    }
}
