use hearth_protocol_git::{
    limits::Limits,
    listing::{self, Entry, Error, Event, Listing, Request},
};
use skein_lib::{
    Env, Queue, Time, Wall,
    stream::{Fault, Read, Up},
};
use skein_world::stream::Wire;
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub seed: u64,
    pub end: Option<usize>,
    pub fail: bool,
    pub close: Option<u32>,
}
impl Settings {
    #[must_use]
    pub fn cut(seed: u64) -> Self {
        Self { seed, end: None, fail: false, close: None }
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Done(Box<[Entry]>),
    Failed(Error),
    Closed,
}
/// Play a List and Close above and a seeded fragmented Wire below.
#[must_use]
pub fn list(stdout: &[u8], limits: &Limits, settings: &Settings) -> Outcome {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut machine = Listing::new(limits);
    let mut wire = Wire::new(settings.seed);
    wire.write(&stdout[..settings.end.unwrap_or(stdout.len()).min(stdout.len())]);
    wire.eof = true;
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    if settings.close == Some(0) {
        assert_closed(&mut machine, &env, &mut wire);
        return Outcome::Closed;
    }
    listing::down(&mut machine, &env, Request::List, &mut above, &mut below);
    let mut calls = 1;
    for _ in 0..stdout.len().saturating_mul(20).saturating_add(100) {
        while let Some(down) = below.pop() {
            wire.take(down);
        }
        if let Some(event) = above.pop() {
            let outcome = match event {
                Event::Listed(entries) => Outcome::Done(entries),
                Event::Failed(error) => Outcome::Failed(error),
                Event::Closed => Outcome::Closed,
            };
            assert_closed(&mut machine, &env, &mut wire);
            return outcome;
        }
        if settings.close == Some(calls) {
            assert_closed(&mut machine, &env, &mut wire);
            return Outcome::Closed;
        }
        if let Some(up) = wire.answer() {
            let up = match up {
                Up::End if settings.fail => Up::Failed(Fault::Reset),
                other @ (Up::Bytes(_) | Up::Room | Up::End | Up::Failed(_)) => other,
            };
            listing::up(&mut machine, &env, up, &mut above, &mut below);
            calls += 1;
        }
    }
    panic!("listing did not settle for {settings:?}")
}
fn assert_closed(machine: &mut Listing, env: &Env<Limits>, wire: &mut Wire) {
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    listing::down(machine, env, Request::Close, &mut above, &mut below);
    assert_eq!(above.pop(), Some(Event::Closed), "Close answers Closed");
    while let Some(down) = below.pop() {
        assert_eq!(down, skein_lib::stream::Down::Demand { read: Read::Nothing, room: 0 }, "close only withdraws");
        wire.take(down);
    }
    assert_eq!(wire.answer(), None, "closed has no demand");
}
