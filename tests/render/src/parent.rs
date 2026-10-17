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
    let mut render = Render::new(&limits);
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut queue = Queue::with_capacity(1);
    let mut schedule = Schedule::new();
    let mut ledger = Ledger::new("render request");
    let mut rng = Rng::new(seed);
    let mut run = Run { answers: Vec::new(), outcome: Outcome::Done, left_out: 0, requests: Vec::new() };
    render::step(
        &mut render,
        &env,
        Event::Render {
            repository: Box::from(b"/repository/.git".as_slice()),
            revision: Box::from(b"HEAD".as_slice()),
            targets: targets.iter().map(|t| Box::from(*t)).collect(),
        },
        &mut queue,
    );
    for _ in 0..10_000 {
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
                        Request::Keep { request, sha256, .. } => {
                            vec![Event::Kept { request: *request, sha256: *sha256 }]
                        }
                        Request::List { request, .. } => vec![Event::Listed {
                            request: *request,
                            commit: fixtures::oid(b'a'),
                            entries: repository.entries.clone().into_boxed_slice(),
                        }],
                        Request::Hunks { .. } => unreachable!("render worlds do not diff"),
                        Request::Read { request, blobs, .. } => {
                            blobs.iter().map(|blob| blob_event(repository, blob, *request)).collect()
                        }
                    };
                    let token = match &request {
                        Request::Keep { request, .. }
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
            let token = match &event {
                Event::Kept { request, .. }
                | Event::Listed { request, .. }
                | Event::Hunks { request, .. }
                | Event::Read { request, .. }
                | Event::Cached { request, .. }
                | Event::Unresolved { request }
                | Event::Failed { request, .. } => *request,
                Event::Render { .. } | Event::Diff { .. } => unreachable!("only answers scheduled"),
            };
            let left = ledger.get_mut(token).expect("request in flight");
            *left -= 1;
            if *left == 0 {
                ledger.end(token);
            }
            render::step(&mut render, &env, event, &mut queue);
        }
        if render.is_empty() && queue.is_empty() {
            ledger.assert_settled();
            assert!(schedule.is_empty(), "no late answer");
            return run;
        }
    }
    panic!("render settles: seed {seed}")
}

fn blob_event(repository: &Repository, blob: &render::vocabulary::Blob, request: skein_lib::Token) -> Event {
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
    } else if blob.kind == render::vocabulary::As::Template {
        let body = &repository.blobs.iter().find(|(oid, _)| *oid == blob.oid).expect("scripted template").1;
        let read = match hearth_protocol_template::template::compile(body, &hearth_protocol_template::limits::SHIPPED) {
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
