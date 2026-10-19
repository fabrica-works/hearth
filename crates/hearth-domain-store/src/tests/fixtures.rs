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
        let mut output = Queue::with_capacity(2);
        crate::step(&mut self.store, &Env { now, wall, limits: self.limits }, event, &mut output);
        collect(&mut output)
    }
    pub fn fire(&mut self, now: Time) -> Box<[Output]> {
        let mut output = Queue::with_capacity(2);
        crate::fire(&mut self.store, &Env { now, wall: Wall::EPOCH, limits: self.limits }, &mut output);
        collect(&mut output)
    }
    pub fn page(&mut self, entries: &[Entry], last: bool) -> Box<[Output]> {
        self.step(Time::ZERO, Event::Page { entries: entries.into(), last })
    }
    pub fn resolved(&self, key: &hearth_lib::encode::Digest) -> bool {
        self.store.scan.resolved(key)
    }
    pub fn indexed(&self, digest: &hearth_lib::encode::Digest) -> bool {
        self.store.scan.indexed(digest)
    }
    pub fn published(&self, key: &hearth_lib::encode::Digest) -> bool {
        self.store.scan.layers.contains_key(key)
    }
    pub fn next_deadline(&self) -> Option<Time> {
        crate::next_deadline(&self.store)
    }
}
fn collect(output: &mut Queue<Output>) -> Box<[Output]> {
    let mut list = List::with_capacity(2);
    for _ in 0..output.capacity() {
        if let Some(value) = output.pop() {
            list.push(value).expect("two outputs");
        }
    }
    list.into_boxed()
}
pub fn recording() -> Harness {
    let mut store = Harness::new(crate::limits::SHIPPED, MIN_UID);
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
    let mut store = Harness::new(limits, MIN_UID);
    drop(store.step(Time::ZERO, Event::Open));
    drop(store.step(Time::ZERO, Event::Locked));
    drop(store.step(
        Time::ZERO,
        Event::Loaded { record: Some(StoreRecord { layout: 1, made: Wall::EPOCH, range: None, uid: MIN_UID }) },
    ));
    drop(store.page(&[], true));
    drop(store.page(&[], true));
    drop(store.page(&[], true));
    store
}
/// A scan listing `layers/`, its first two passes done.
pub fn layering(limits: Limits) -> Harness {
    let mut store = Harness::new(limits, MIN_UID);
    drop(store.step(Time::ZERO, Event::Open));
    drop(store.step(Time::ZERO, Event::Locked));
    drop(store.step(
        Time::ZERO,
        Event::Loaded { record: Some(StoreRecord { layout: 1, made: Wall::EPOCH, range: None, uid: MIN_UID }) },
    ));
    drop(store.page(&[], true));
    drop(store.page(&[], true));
    store
}
pub fn empty_layer() -> crate::vocabulary::Layer {
    crate::vocabulary::Layer {
        key: hearth_lib::hash::H::new(b"empty", 0).finish(),
        parent: None,
        inputs: crate::vocabulary::LayerInputs::Empty,
        tree: D3,
        uuid: [7; 16],
        at: Wall::EPOCH,
    }
}
/// H("packages", resolution digest, part digest), written out (domain/store.md, 3.3).
pub fn packages_layer() -> crate::vocabulary::Layer {
    let mut hash = hearth_lib::hash::H::new(b"packages", 2);
    hash.values().digest(&D2);
    hash.values().digest(&D1);
    crate::vocabulary::Layer {
        key: hash.finish(),
        parent: Some(empty_layer().key),
        inputs: crate::vocabulary::LayerInputs::Packages { part: D1, resolution: D2 },
        ..empty_layer()
    }
}
pub fn record_name(key: hearth_lib::encode::Digest) -> Box<[u8]> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut name = skein_lib::Writer::new(71_usize);
    for byte in key.0 {
        name.put(&[HEX[usize::from(byte >> 4_u8)], HEX[usize::from(byte & 15_u8)]]).expect("hex room");
    }
    name.put(b".record").expect("suffix room");
    name.finish()
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
    drop(store.step(Time::ZERO, Event::Done));
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

pub fn resolving(limits: Limits, key: hearth_lib::encode::Digest) -> Harness {
    let mut store = Harness::new(limits, MIN_UID);
    drop(store.step(Time::ZERO, Event::Open));
    drop(store.step(Time::ZERO, Event::Locked));
    drop(store.step(
        Time::ZERO,
        Event::Loaded { record: Some(StoreRecord { layout: 1, made: Wall::EPOCH, range: None, uid: MIN_UID }) },
    ));
    drop(store.page(&[], true));
    let entry = entry(b"record", crate::vocabulary::Shape::Record(key));
    drop(store.page(core::slice::from_ref(&entry), true));
    store
}
pub fn inputs() -> crate::vocabulary::Inputs {
    use hearth_lib::encode::Item;
    crate::vocabulary::Inputs {
        sources: Box::new([Item::Map(0)]),
        request: Box::new([Item::List(1), Item::Bytes(b"hello".as_slice().into())]),
        variant: Box::new([Item::Bytes(b"apt".as_slice().into())]),
        apt: Box::new([Item::Map(0)]),
    }
}
pub fn closure() -> Box<[crate::vocabulary::Closed]> {
    Box::new([crate::vocabulary::Closed {
        source: 0,
        filename: b"pool/apt.deb".as_slice().into(),
        size: 7,
        name: b"apt".as_slice().into(),
        architecture: b"amd64".as_slice().into(),
        version: b"3.0.3".as_slice().into(),
        sha256: D1,
    }])
}
pub fn resolution() -> Event {
    Event::Resolution { record: resolution_record() }
}
pub const KEY: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    1, 234, 129, 26, 252, 153, 48, 138, 159, 204, 51, 108, 148, 73, 102, 199, 249, 221, 152, 133, 123, 39, 89, 47, 224,
    10, 31, 184, 75, 212, 157, 0,
]);
pub const CLOSURE: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    242, 251, 234, 142, 207, 175, 92, 30, 225, 209, 84, 99, 193, 172, 188, 52, 212, 154, 210, 160, 229, 114, 68, 15,
    124, 191, 52, 95, 192, 174, 126, 227,
]);
pub const EMPTY_CLOSURE: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    193, 88, 154, 151, 28, 76, 229, 4, 43, 122, 140, 15, 92, 154, 246, 155, 61, 70, 254, 100, 184, 171, 76, 218, 172,
    88, 93, 176, 234, 58, 213, 48,
]);

pub fn waiting() -> Harness {
    let mut store =
        Harness::new(Limits { retry: skein_lib::Duration::from_secs(2), ..crate::limits::SHIPPED }, MIN_UID);
    drop(store.step(Time::ZERO, Event::Open));
    drop(store.step(Time::ZERO, Event::Busy));
    store
}

pub fn resolution_record() -> crate::vocabulary::Resolution {
    use crate::vocabulary::{Costs, Line, Made, Resolution};
    Resolution {
        key: KEY,
        inputs: inputs(),
        closure: closure(),
        digest: CLOSURE,
        sources: Box::new([Line {
            uri: b"https://deb.example/debian".as_slice().into(),
            suite: b"stable".as_slice().into(),
            components: Box::new([b"main".as_slice().into()]),
            indices: Box::new([]),
        }]),
        made: Made {
            at: Wall::EPOCH,
            apt: b"3.0.3".as_slice().into(),
            mmdebstrap: b"1.5".as_slice().into(),
            costs: Costs { fetched: 0, took: skein_lib::Duration::ZERO },
        },
    }
}

pub fn resolved_ready() -> Harness {
    let mut store = ready(crate::limits::SHIPPED, &[]);
    store.store.scan.resolutions.insert(KEY, ()).expect("fixture resolution");
    store
}

pub const ONE_RESOLUTION: Limits = Limits { resolutions: 1, ..crate::limits::SHIPPED };
pub const RECORD_BOUND: u64 = 4 * 1024 * 1024;
pub fn recorded_ready() -> Harness {
    let mut store = ready(ONE_RESOLUTION, &[]);
    drop(store.step(Time::ZERO, Event::Record { resolution: resolution_record() }));
    drop(store.step(Time::ZERO, Event::Done));
    store
}

pub fn room_store(usable: bool, recorded: bool) -> Harness {
    let mut store = if recorded { recorded_ready() } else { ready(ONE_RESOLUTION, &[]) };
    store.store.usable = usable;
    store
}

pub const MIN_UID: u32 = 1000;
