//! The scripted parent delays answers and checks request terminals.
use crate::fixtures::{self, Repository};
use hearth_domain_render::{
    self as render, Render, limits,
    vocabulary::{Answer, Event, Outcome, Output, Request},
};
use skein_lib::{Env, Queue, Rng, Time, Wall};
use skein_world::domain::{Ledger, Schedule, Span};
#[derive(Debug)]
pub struct Run {
    pub answers: Vec<Answer>,
    pub outcome: Outcome,
    pub left_out: u32,
    pub requests: Vec<Request>,
}
#[must_use]
pub fn run(repository: &Repository, targets: &[&[u8]], seed: u64) -> Run {
    run_with(repository, targets, seed, limits::SHIPPED)
}
#[must_use]
pub fn run_with(repository: &Repository, targets: &[&[u8]], seed: u64, limits: limits::Limits) -> Run {
    drive(Revisions { from: repository, to: None }, targets, seed, limits, skein_world::Memory::Unchecked, 0)
}
#[must_use]
pub fn checked(
    repository: &Repository,
    seed: u64,
    limits: limits::Limits,
    bound: u64,
    memory: skein_world::Memory,
) -> Run {
    assert_eq!(memory, skein_world::Memory::Checked);
    drive(Revisions { from: repository, to: None }, &[], seed, limits, memory, bound)
}
#[derive(Clone, Copy)]
struct Revisions<'a> {
    from: &'a Repository,
    to: Option<&'a Repository>,
}
impl<'a> Revisions<'a> {
    fn listing(self, revision: &[u8], request: skein_lib::Token) -> (&'a Repository, Event) {
        let (repository, commit) = if revision == b"after" {
            (self.to.expect("second revision"), fixtures::oid(b'b'))
        } else {
            (self.from, fixtures::oid(b'a'))
        };
        (repository, Event::Listed { request, commit, entries: repository.entries.clone().into_boxed_slice() })
    }
    fn event(self, targets: &[&[u8]]) -> Event {
        if self.to.is_some() {
            Event::Diff {
                resolve: false,
                repository: Box::from(b"/repository/.git".as_slice()),
                from: Box::from(b"before".as_slice()),
                to: Box::from(b"after".as_slice()),
                targets: targets.iter().map(|target| Box::from(*target)).collect(),
            }
        } else {
            render_event(targets)
        }
    }
}
#[must_use]
pub fn diff(from: &Repository, to: &Repository, seed: u64, limits: limits::Limits) -> Run {
    drive(Revisions { from, to: Some(to) }, &[], seed, limits, skein_world::Memory::Unchecked, 0)
}
#[must_use]
pub fn checked_diff(from: &Repository, to: &Repository, seed: u64, limits: limits::Limits, bound: u64) -> Run {
    drive(Revisions { from, to: Some(to) }, &[], seed, limits, skein_world::Memory::Checked, bound)
}
fn drive(
    revisions: Revisions<'_>,
    targets: &[&[u8]],
    seed: u64,
    limits: limits::Limits,
    memory: skein_world::Memory,
    bound: u64,
) -> Run {
    let meter = (memory == skein_world::Memory::Checked).then(skein_world::domain::heap::Meter::new);
    let mut render = Render::new(&limits);
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut queue = Queue::with_capacity(1);
    let mut schedule = Schedule::new();
    let mut ledger = Ledger::new("render request");
    let mut rng = Rng::new(seed);
    let mut run = Run { answers: Vec::new(), outcome: Outcome::Done, left_out: 0, requests: Vec::new() };
    if let Some(meter) = &meter {
        meter.start();
    }
    let event = revisions.event(targets);
    let mut repository = revisions.from;
    render::step(&mut render, &env, event, &mut queue);
    if let Some(meter) = &meter {
        let measured = meter.end();
        meter.check(measured, bound, &"initial call");
    }
    for iteration in 0..10_000 {
        if let Some(meter) = &meter {
            meter.start();
        }
        if let Some(output) = queue.pop() {
            match output {
                Output::Answer(answer) => match answer {
                    Answer::Spec(spec) => run.answers.push(Answer::Spec(spec)),
                    Answer::Error(error) => run.answers.push(Answer::Error(error)),
                    Answer::Change(change) => run.answers.push(Answer::Change(change)),
                    Answer::Rendered { outcome, diff: _, left_out } => {
                        run.outcome = outcome;
                        run.left_out = left_out;
                    }
                },
                Output::Request(request) => {
                    let mut at = env.now;
                    let events = match &request {
                        Request::Record { request, .. } => vec![Event::Recorded { request: *request }],
                        Request::Resolve { .. } => unreachable!("resolve disabled"),
                        Request::Find { request, .. } => vec![Event::Absent { request: *request }],
                        Request::Keep { request, sha256, .. } => {
                            vec![Event::Kept { request: *request, sha256: *sha256 }]
                        }
                        Request::List { request, revision, .. } => {
                            let (selected, event) = revisions.listing(revision, *request);
                            repository = selected;
                            vec![event]
                        }
                        Request::Hunks { request, .. } => vec![hunk_failure(*request)],
                        Request::Read { request, blobs, .. } => {
                            blobs.iter().map(|blob| blob_event(repository, blob, *request, &limits)).collect()
                        }
                    };
                    let token = match &request {
                        Request::Resolve { request, .. }
                        | Request::Record { request, .. }
                        | Request::Find { request, .. }
                        | Request::Keep { request, .. }
                        | Request::List { request, .. }
                        | Request::Read { request, .. }
                        | Request::Hunks { request, .. } => *request,
                    };
                    ledger.open(token, events.len());
                    for event in events {
                        at = at.checked_add(Span::millis(1, 9).draw(&mut rng)).expect("world time");
                        schedule.send(at, event);
                    }
                    run.requests.push(request);
                }
            }
        }
        if render.work_pending() {
            render::resume(&mut render, &env, &mut queue);
        } else if let Some(at) = schedule.next_time() {
            env.now = at;
            let event = schedule.next(at).expect("scheduled event");
            let token = event_token(&event);
            let left = ledger.get_mut(token).expect("request in flight");
            *left -= 1;
            if *left == 0 {
                ledger.end(token);
            }
            render::step(&mut render, &env, event, &mut queue);
        }
        if let Some(meter) = &meter {
            let measured = meter.end();
            meter.check(measured, bound, &iteration);
        }
        if render.is_empty() && queue.is_empty() {
            ledger.assert_settled();
            assert!(schedule.is_empty(), "no late answer");
            return run;
        }
    }
    panic!("render settles: seed {seed}")
}

fn render_event(targets: &[&[u8]]) -> Event {
    Event::Render {
        resolve: false,
        repository: Box::from(b"/repository/.git".as_slice()),
        revision: Box::from(b"HEAD".as_slice()),
        targets: targets.iter().map(|target| Box::from(*target)).collect(),
    }
}

pub(crate) fn blob_event(
    repository: &Repository,
    blob: &render::vocabulary::Blob,
    request: skein_lib::Token,
    limits: &limits::Limits,
) -> Event {
    if blob.kind == render::vocabulary::As::Cached {
        let body = repository.blobs.iter().find(|(oid, _)| *oid == blob.oid).expect("scripted blob");
        let mut hash = skein_lib::sha256::Sha256::new();
        hash.update(&body.1);
        Event::Cached {
            request,
            oid: blob.oid.clone(),
            size: blob.size,
            sha256: hearth_lib::encode::Digest(hash.finish()),
        }
    } else if blob.kind == render::vocabulary::As::Link {
        let body = &repository.blobs.iter().find(|(oid, _)| *oid == blob.oid).expect("scripted link").1;
        Event::Read { request, oid: blob.oid.clone(), read: render::vocabulary::Decoded::Link(body.clone()) }
    } else if blob.kind == render::vocabulary::As::Template {
        if let Some((_, read)) = repository.documents.iter().find(|(oid, _)| *oid == blob.oid) {
            return Event::Read { request, oid: blob.oid.clone(), read: read.clone() };
        }
        let body = &repository.blobs.iter().find(|(oid, _)| *oid == blob.oid).expect("scripted template").1;
        let read = match hearth_protocol_template::template::compile(
            body,
            &hearth_protocol_template::limits::Limits {
                template: limits.template,
                expression: limits.expression,
                statements: limits.statements,
                nesting: limits.nesting,
            },
        ) {
            Ok(template) => render::vocabulary::Decoded::Template(
                hearth_protocol_builder::translate::instructions::template(template),
            ),
            Err(error) => {
                let (line, column) = hearth_protocol_template::template::place(body, error.at);
                render::vocabulary::Decoded::Refused {
                    at: render::revision::Position { line, column },
                    fault: render::errors::Decode::Syntax(hearth_protocol_builder::translate::errors::syntax(
                        error.kind,
                    )),
                    also: None,
                }
            }
        };
        Event::Read { request, oid: blob.oid.clone(), read }
    } else {
        Event::Read {
            request,
            oid: blob.oid.clone(),
            read: repository.documents.iter().find(|(oid, _)| *oid == blob.oid).expect("scripted document").1.clone(),
        }
    }
}

fn event_token(event: &Event) -> skein_lib::Token {
    match event {
        Event::Recorded { request }
        | Event::Stanza { request, .. }
        | Event::Inst { request, .. }
        | Event::Resolved { request, .. }
        | Event::Found { request, .. }
        | Event::Absent { request }
        | Event::Kept { request, .. }
        | Event::Listed { request, .. }
        | Event::Hunks { request, .. }
        | Event::Read { request, .. }
        | Event::Cached { request, .. }
        | Event::Unresolved { request }
        | Event::Failed { request, .. } => *request,
        Event::Render { .. } | Event::Diff { .. } => unreachable!("only answers scheduled"),
    }
}

// This world exercises render reads; a parent can decline text materialization.
fn hunk_failure(request: skein_lib::Token) -> Event {
    Event::Failed {
        request,
        class: render::vocabulary::Class::Operational,
        at: None,
        failure: render::vocabulary::Failure::Store,
    }
}
