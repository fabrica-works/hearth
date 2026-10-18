//! The two service passes with the world's io face and scheduled completions.
use crate::{
    fixtures::{self, Scenario, Shared},
    io::Below,
};
use hearth_domain_builder::{
    self as builder,
    vocabulary::{Event, Request},
};
use hearth_protocol_builder::{self as protocol, Asked, Protocol, Told};
use skein_lib::{Env, Queue, Time, Wall};
use skein_world::domain::Schedule;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Run {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub code: Option<u8>,
    pub shared: Shared,
    pub fault_fell: bool,
    pub blocked_writes: u64,
    pub spawned: u32,
    pub reaped: u32,
    pub opened_pipes: u32,
    pub ended_pipes: u32,
    pub lock_times: Vec<Time>,
    pub vectors: Vec<Box<[Box<[u8]>]>>,
    pub now: Time,
}
struct World {
    below: Below,
    domain: builder::Builder,
    protocol: Protocol,
    domain_env: Env<builder::Limits>,
    protocol_env: Env<protocol::limits::Limits>,
    calls: Queue<Event>,
    answers: Queue<Request>,
    asked: Queue<Asked>,
    told: Queue<Told>,
    schedule: Schedule<Told>,
    rng: skein_lib::Rng,
    ordered: BTreeMap<skein_lib::Token, Time>,
}
#[must_use]
pub fn run(scenario: &Scenario<'_>, seed: u64) -> Run {
    with_pipe(scenario, seed, fixtures::WORLD_PIPE)
}
#[must_use]
pub fn with_pipe(scenario: &Scenario<'_>, seed: u64, pipe: u32) -> Run {
    World::new(scenario, seed, pipe).finish()
}
#[must_use]
pub fn with_lock(scenario: &Scenario<'_>, seed: u64, release: Time) -> Run {
    let mut world = World::new(scenario, seed, fixtures::WORLD_PIPE);
    world.below.hold_lock_until(release);
    world.finish()
}
impl World {
    fn new(scenario: &Scenario<'_>, seed: u64, pipe: u32) -> Self {
        let limits = hearth::limits();
        let below = Below::new(scenario, seed, pipe);
        let mut protocol = Protocol::new(
            &limits.protocol,
            scenario.call.clone(),
            scenario.form,
            below.files.roots,
            fixtures::STORE,
            seed,
        )
        .with_file_read(limits.files.read);
        protocol.bind_output(fixtures::PROTOCOL_OUTPUT);
        protocol.bind_error(fixtures::PROTOCOL_ERROR);
        Self {
            below,
            domain: builder::Builder::new(&limits.domain, fixtures::BUILDER.user),
            protocol,
            domain_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.domain },
            protocol_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.protocol },
            calls: Queue::with_capacity(fixtures::PROTOCOL_QUEUE),
            answers: Queue::with_capacity(fixtures::PROTOCOL_QUEUE),
            asked: Queue::with_capacity(fixtures::PROTOCOL_QUEUE),
            told: Queue::with_capacity(fixtures::PROTOCOL_QUEUE),
            schedule: Schedule::new(),
            ordered: BTreeMap::new(),
            rng: skein_lib::Rng::new(seed),
        }
    }
    fn finish(mut self) -> Run {
        for _ in 0..fixtures::PROTOCOL_TURNS {
            let now = self.protocol_env.now;
            self.below.step(now);
            self.schedule_events(now);
            if self.told.room() > 0
                && let Some(told) = self.schedule.next(now)
            {
                self.told.push(told);
            }
            self.iterate();
            if let Some(asked) = self.asked.pop() {
                self.below.answer(asked, now);
            }
            self.schedule_events(now);
            if self.settled() {
                self.below.settle();
                let run = Run {
                    stdout: core::mem::take(&mut self.below.stdout),
                    stderr: core::mem::take(&mut self.below.stderr),
                    code: self.protocol.code(),
                    shared: self.below.files.shared.clone(),
                    fault_fell: self.below.fault_fell,
                    blocked_writes: self.below.blocked_writes,
                    spawned: self.below.spawned,
                    reaped: self.below.reaped,
                    opened_pipes: self.below.opened_pipes,
                    ended_pipes: self.below.ended_pipes,
                    lock_times: core::mem::take(&mut self.below.lock_times),
                    vectors: core::mem::take(&mut self.below.vectors),
                    now,
                };
                self.below.finish();
                return run;
            }
            let next = [
                self.schedule.next_time(),
                builder::next_deadline(&self.domain),
                protocol::next_deadline(&self.protocol),
            ]
            .into_iter()
            .flatten()
            .filter(|at| *at > now)
            .min();
            let tick = now.saturating_add(fixtures::PROTOCOL_TICK);
            let now = if self.below.active() || self.ready() || self.schedule.is_due(now) {
                tick
            } else {
                next.expect("world has scheduled work")
            };
            self.protocol_env.now = now;
            self.domain_env.now = now;
        }
        panic!("hearth-28 protocol world exceeded its bounded turns");
    }
    fn schedule_events(&mut self, now: Time) {
        for _ in 0..fixtures::PROTOCOL_QUEUE {
            let Some(told) = self.below.pop() else { break };
            let owner = owner(&told);
            let at = now.saturating_add(fixtures::PROTOCOL_LATENCY.draw(&mut self.rng));
            let at = self.ordered.get(&owner).copied().map_or(at, |previous| previous.max(at));
            self.ordered.insert(owner, at);
            self.schedule.send(at, told);
        }
    }
    fn iterate(&mut self) {
        if self.calls.room() > 0 && self.asked.room() > 0 {
            protocol::fire(&mut self.protocol, &self.protocol_env, &mut self.calls, &mut self.asked);
            if self.protocol.work_pending() {
                protocol::resume(&mut self.protocol, &self.protocol_env, &mut self.calls, &mut self.asked);
            } else if let Some(told) = self.told.pop() {
                protocol::up(&mut self.protocol, &self.protocol_env, told, &mut self.calls, &mut self.asked);
            }
        }
        if self.answers.room() >= builder::MAX_OUT_RESUME.max(builder::MAX_OUT) {
            if self.domain.work_pending() {
                builder::resume(&mut self.domain, &self.domain_env, &mut self.answers);
            } else if let Some(event) = self.calls.pop() {
                builder::step(&mut self.domain, &self.domain_env, event, &mut self.answers);
            } else {
                builder::fire(&mut self.domain, &self.domain_env, &mut self.answers);
            }
        }
        if self.protocol.takes()
            && self.calls.room() > 0
            && self.asked.room() > 0
            && let Some(request) = self.answers.pop()
        {
            protocol::down(&mut self.protocol, &self.protocol_env, request, &mut self.calls, &mut self.asked);
        }
    }
    fn ready(&self) -> bool {
        self.domain.work_pending()
            || self.protocol.work_pending()
            || !self.calls.is_empty()
            || !self.answers.is_empty()
            || !self.asked.is_empty()
            || !self.told.is_empty()
    }
    fn settled(&self) -> bool {
        !self.ready()
            && !self.below.active()
            && self.schedule.is_empty()
            && self.domain.is_empty()
            && self.protocol.is_empty()
    }
}

fn owner(told: &Told) -> skein_lib::Token {
    match told {
        Told::File(event) => event.owner(),
        Told::Io(event) => match event {
            skein_io::Event::Spawned { owner, .. }
            | skein_io::Event::Stream { owner, .. }
            | skein_io::Event::Exited { owner, .. }
            | skein_io::Event::Failed { owner, .. }
            | skein_io::Event::Closed { owner } => *owner,
            skein_io::Event::Listening { .. }
            | skein_io::Event::Accepted { .. }
            | skein_io::Event::Connecting { .. }
            | skein_io::Event::Connected { .. }
            | skein_io::Event::Output { .. }
            | skein_io::Event::Usage { .. }
            | skein_io::Event::Window { .. }
            | skein_io::Event::Resized
            | skein_io::Event::Shutdown { .. } => unreachable!("the builder uses tool and printer events"),
        },
    }
}
