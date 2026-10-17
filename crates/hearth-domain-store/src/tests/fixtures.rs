use crate::{
    Store,
    limits::Limits,
    vocabulary::{Entry, Event, Output, StoreRecord},
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Time, Wall};
#[derive(Debug)]
pub struct Harness {
    store: Store,
    limits: Limits,
}
impl Harness {
    pub fn new(limits: Limits, uid: u32) -> Self {
        Self { store: Store::new(&limits, uid), limits }
    }
    pub fn step(&mut self, now: Time, event: Event) -> Box<[Output]> {
        self.step_at(now, Wall::EPOCH, event)
    }
    pub fn step_at(&mut self, now: Time, wall: Wall, event: Event) -> Box<[Output]> {
        let mut output = Queue::with_capacity(1);
        crate::step(&mut self.store, &Env { now, wall, limits: self.limits }, event, &mut output);
        collect(&mut output)
    }
    pub fn fire(&mut self, now: Time) -> Box<[Output]> {
        let mut output = Queue::with_capacity(1);
        crate::fire(&mut self.store, &Env { now, wall: Wall::EPOCH, limits: self.limits }, &mut output);
        collect(&mut output)
    }
    pub fn page(&mut self, entries: &[Entry], last: bool) -> Box<[Output]> {
        self.step(Time::ZERO, Event::Page { entries: entries.into(), last })
    }
    pub fn indexed(&self, digest: &hearth_lib::encode::Digest) -> bool {
        self.store.scan.indexed(digest)
    }
    pub fn next_deadline(&self) -> Option<Time> {
        crate::next_deadline(&self.store)
    }
}
fn collect(output: &mut Queue<Output>) -> Box<[Output]> {
    let mut list = List::with_capacity(1);
    if let Some(value) = output.pop() {
        list.push(value).expect("one output");
    }
    list.into_boxed()
}
pub fn recording() -> Harness {
    let mut store = Harness::new(crate::limits::SHIPPED, 1000);
    drop(store.step(Time::ZERO, Event::Open));
    drop(store.step(Time::ZERO, Event::Locked));
    store
}

pub const D1: hearth_lib::encode::Digest = hearth_lib::encode::Digest([1; 32]);
pub const D2: hearth_lib::encode::Digest = hearth_lib::encode::Digest([2; 32]);
pub const D3: hearth_lib::encode::Digest = hearth_lib::encode::Digest([3; 32]);
pub fn entry(name: &[u8], shape: crate::vocabulary::Shape) -> Entry {
    Entry { name: name.into(), shape }
}
pub fn scanning(limits: Limits) -> Harness {
    let mut store = Harness::new(limits, 1000);
    drop(store.step(Time::ZERO, Event::Open));
    drop(store.step(Time::ZERO, Event::Locked));
    drop(store.step(
        Time::ZERO,
        Event::Loaded { record: Some(StoreRecord { layout: 1, made: Wall::EPOCH, range: None, uid: 1000 }) },
    ));
    drop(store.page(&[], true));
    store
}

pub const ARRIVAL: skein_lib::Token = skein_lib::Token::new(1);
pub const MOTD: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    0x77, 0x53, 0x24, 0xfc, 0xea, 0x7f, 0x2c, 0xdc, 0xbf, 0x11, 0xdf, 0x25, 0x55, 0x0e, 0x36, 0x26, 0x69, 0xca, 0xce,
    0xb6, 0xe6, 0x9c, 0x83, 0x70, 0x37, 0x4d, 0x2d, 0x43, 0x6b, 0xb6, 0x31, 0x23,
]);
pub fn ready(limits: Limits, digests: &[hearth_lib::encode::Digest]) -> Harness {
    let mut store = scanning(limits);
    drop(store.page(&[], true));
    drop(store.page(&[], true));
    for digest in digests {
        store.store.scan.index.insert(*digest, true).expect("fixture cache bound");
    }
    store
}
pub fn blob(arrival: skein_lib::Token, sha256: hearth_lib::encode::Digest) -> Event {
    Event::Arrived {
        arrival,
        sha256,
        size: 15,
        origin: crate::vocabulary::Origin::Blob { oid: b"d23c59a84714a16a9597f6d92ce69390d6d13f9b".as_slice().into() },
        took: skein_lib::Duration::from_millis(3),
    }
}
pub fn full() -> Harness {
    let mut store = ready(crate::limits::SHIPPED, &[]);
    store.store.usable = false;
    store
}
