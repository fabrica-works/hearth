//! Unified diff driven through the same cut and whole stream sources.
use crate::world::{Settings, Source};
use hearth_protocol_git::{
    hunks::{self, Diff, Error, Event, Hunks, Request},
    limits::Limits,
};
use skein_lib::{
    Env, Queue, Time, Wall,
    stream::{Down, Fault, Read, Up},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffOutcome {
    Done(Hunks),
    Failed(Error),
    Closed,
}
#[must_use]
pub fn diff(stdout: &[u8], limits: &Limits, settings: &Settings) -> DiffOutcome {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut machine = Diff::new(limits);
    let mut source = Source::quiet(stdout, settings);
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    hunks::down(&mut machine, &env, Request::Read, &mut above, &mut below);
    let mut calls = 0;
    for _ in 0..stdout.len().saturating_mul(30).saturating_add(100) {
        while let Some(down) = below.pop() {
            source.take(down);
        }
        if settings.close == Some(calls) {
            closed(&mut machine, &env, &mut source);
            return DiffOutcome::Closed;
        }
        if let Some(event) = above.pop() {
            let outcome = match event {
                Event::Hunks(hunks) => DiffOutcome::Done(hunks),
                Event::Failed(error) => DiffOutcome::Failed(error),
                Event::Closed => DiffOutcome::Closed,
            };
            closed(&mut machine, &env, &mut source);
            return outcome;
        }
        if let Some(event) = source.answer() {
            let event = match event {
                Up::End if settings.fail => Up::Failed(Fault::Reset),
                other @ (Up::Bytes(_) | Up::Room | Up::End | Up::Failed(_)) => other,
            };
            hunks::up(&mut machine, &env, event, &mut above, &mut below);
            calls += 1;
        }
    }
    panic!("diff did not settle: {settings:?}")
}
fn closed(machine: &mut Diff, env: &Env<Limits>, source: &mut Source) {
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    hunks::down(machine, env, Request::Close, &mut above, &mut below);
    assert_eq!(above.pop(), Some(Event::Closed), "Close answers Closed");
    while let Some(down) = below.pop() {
        assert_eq!(down, Down::Demand { read: Read::Nothing, room: 0 }, "withdraw only");
        source.take(down);
    }
    assert_eq!(source.answer(), None, "closed has no demand");
}
