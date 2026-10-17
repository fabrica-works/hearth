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
    let mut run = Run { answers: Vec::new(), outcome: Outcome::Done, requests: Vec::new() };
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
                    Answer::Rendered { outcome } => {
                        run.outcome = outcome;
                    }
                },
                Output::Request(request) => {
                    let mut at = env.now;
                    let events = match &request {
                        Request::List { request, .. } => vec![Event::Listed {
                            request: *request,
                            commit: fixtures::oid(b'a'),
                            entries: repository.entries.clone().into_boxed_slice(),
                        }],
                        Request::Read { request, blobs, .. } => blobs
                            .iter()
                            .map(|blob| Event::Read {
                                request: *request,
                                oid: blob.oid.clone(),
                                read: repository
                                    .documents
                                    .iter()
                                    .find(|(oid, _)| *oid == blob.oid)
                                    .expect("scripted document")
                                    .1
                                    .clone(),
                            })
                            .collect(),
                    };
                    let token = match &request {
                        Request::List { request, .. } | Request::Read { request, .. } => *request,
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
                Event::Listed { request, .. }
                | Event::Read { request, .. }
                | Event::Unresolved { request }
                | Event::Failed { request, .. } => *request,
                Event::Render { .. } => unreachable!("only answers scheduled"),
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
