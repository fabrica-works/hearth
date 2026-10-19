//! Opening the store before a builder call (domain/store.md, 8.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod enter;
mod index;
pub mod layers;
pub mod limits;
pub mod resolutions;
mod scan;
mod staging;
mod start;
pub mod vocabulary;
use limits::Limits;
use skein_lib::{Deadlines, Env, Queue, Time};
use vocabulary::{Event, Output, Request};
/// Strays and readiness, or a failure and its refused arrival.
pub const MAX_OUT: u32 = 2;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum Timer {
    Retry,
    Said,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Phase {
    Closed,
    Locking,
    Recording,
    Writing,
    Scanning,
    Ready,
}
#[derive(Debug, Hash)]
pub struct Store {
    phase: Phase,
    uid: u32,
    timers: Deadlines<Timer>,
    scan: scan::Scan,
    entering: Option<enter::Entering>,
    usable: bool,
    said: bool,
    finding: bool,
    writing: Option<hearth_lib::encode::Digest>,
    decline: vocabulary::Decline,
}
impl Store {
    #[must_use]
    pub fn new(limits: &Limits, uid: u32) -> Self {
        Self {
            phase: Phase::Closed,
            uid,
            timers: Deadlines::with_capacity(2),
            scan: scan::Scan::new(limits),
            entering: None,
            usable: false,
            said: false,
            finding: false,
            writing: None,
            decline: vocabulary::Decline::Unusable { step: vocabulary::Step::List },
        }
    }
}
pub fn step(store: &mut Store, env: &Env<Limits>, ev: Event, out: &mut Queue<Output>) {
    if let Event::Want { key } = ev {
        layers::want(store, key, out);
    } else if let Event::Stage { .. } = ev {
        staging::stage(out);
    } else if let Event::Seal { .. } = ev {
        staging::seal(out);
    } else if let Event::Discard { .. } = ev {
        staging::discard(out);
    } else if store.writing.is_some() {
        resolutions::record(store, env, ev, out);
    } else if let Event::Record { .. } | Event::Room = ev {
        resolutions::record(store, env, ev, out);
    } else if let Event::Find { .. } = ev {
        resolutions::find(store, env, ev, out);
    } else if store.finding {
        resolutions::find(store, env, ev, out);
    } else if store.entering.is_some() {
        enter::step(store, env, ev, out);
    } else if let Event::Arrived { .. } = ev {
        enter::step(store, env, ev, out);
    } else if store.phase == Phase::Scanning {
        scan::step(store, env, ev, out);
    } else {
        start::step(store, env, ev, out);
    }
}
pub fn fire(store: &mut Store, env: &Env<Limits>, out: &mut Queue<Output>) {
    match store.timers.expire(env.now) {
        Some(Timer::Retry) => out.push(Output::Request(Request::Lock)),
        Some(Timer::Said) => out.push(Output::Answer(vocabulary::Answer::Waiting)),
        None => {}
    }
}
#[must_use]
pub fn next_deadline(store: &Store) -> Option<Time> {
    store.timers.next()
}
#[cfg(test)]
mod tests;
