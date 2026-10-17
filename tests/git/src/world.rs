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
    pub whole: bool,
    pub end: Option<usize>,
    pub fail: bool,
    pub close: Option<u32>,
}
impl Settings {
    #[must_use]
    pub fn whole() -> Self {
        Self { whole: true, ..Self::cut(0) }
    }
    #[must_use]
    pub fn cut(seed: u64) -> Self {
        Self { seed, whole: false, end: None, fail: false, close: None }
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
    let mut wire = Source::new(stdout, settings);
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
fn assert_closed(machine: &mut Listing, env: &Env<Limits>, wire: &mut Source) {
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Blob {
    pub oid: hearth_protocol_git::oid::Oid,
    pub bytes: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BatchOutcome {
    Done(Box<[Blob]>),
    Failed(hearth_protocol_git::batch::Error),
    Closed,
}
/// Play binary blobs through a Wire, delaying Next from the seed.
#[must_use]
pub fn read(
    stdout: &[u8],
    requests: &[hearth_protocol_git::batch::Request],
    limits: &Limits,
    settings: &Settings,
) -> BatchOutcome {
    use hearth_protocol_git::batch::{self, Batch, Event, Request};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut machine = Batch::START;
    let mut wire = Source::new(stdout, settings);
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut requests = requests.iter();
    let mut pending = requests.next().copied();
    let mut result = Vec::new();
    let mut bytes = Vec::new();
    let mut rng = skein_lib::Rng::new(settings.seed);
    let mut calls = 0;
    for _ in 0..stdout.len().saturating_mul(30).saturating_add(100) {
        while let Some(down) = below.pop() {
            wire.take(down);
        }
        if settings.close == Some(calls) {
            batch_closed(&mut machine, &env, &mut wire);
            return BatchOutcome::Closed;
        }
        if let Some(event) = above.pop() {
            match event {
                Event::Piece(piece) => {
                    assert!(piece.len() <= usize::try_from(limits.chunk).expect("chunk fits"), "bounded piece");
                    bytes.extend_from_slice(&piece);
                    pending = Some(Request::Next);
                }
                Event::Whole { oid, size } => {
                    assert_eq!(u64::try_from(bytes.len()).expect("body fits"), size, "whole body size");
                    result.push(Blob { oid, bytes: Some(core::mem::take(&mut bytes).into_boxed_slice()) });
                    pending = requests.next().copied();
                    if pending.is_none() {
                        batch_closed(&mut machine, &env, &mut wire);
                        return BatchOutcome::Done(result.into_boxed_slice());
                    }
                }
                Event::Missing { oid } => {
                    assert!(bytes.is_empty(), "missing has no body");
                    result.push(Blob { oid, bytes: None });
                    pending = requests.next().copied();
                    if pending.is_none() {
                        batch_closed(&mut machine, &env, &mut wire);
                        return BatchOutcome::Done(result.into_boxed_slice());
                    }
                }
                Event::Failed(error) => {
                    batch_closed(&mut machine, &env, &mut wire);
                    return BatchOutcome::Failed(error);
                }
                Event::Closed => return BatchOutcome::Closed,
            }
        }
        if let Some(request) = pending {
            if rng.chance(500) {
                pending = None;
                batch::down(&mut machine, &env, request, &mut above, &mut below);
                calls += 1;
            }
        } else if let Some(event) = wire.answer() {
            let event = match event {
                Up::End if settings.fail => Up::Failed(Fault::Reset),
                other @ (Up::Bytes(_) | Up::Room | Up::End | Up::Failed(_)) => other,
            };
            batch::up(&mut machine, &env, event, &mut above, &mut below);
            calls += 1;
        }
    }
    panic!("batch did not settle for {settings:?}")
}
fn batch_closed(machine: &mut hearth_protocol_git::batch::Batch, env: &Env<Limits>, wire: &mut Source) {
    use hearth_protocol_git::batch::{self, Event, Request};
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    batch::down(machine, env, Request::Close, &mut above, &mut below);
    assert_eq!(above.pop(), Some(Event::Closed), "Close answers Closed");
    while let Some(down) = below.pop() {
        assert_eq!(down, skein_lib::stream::Down::Demand { read: Read::Nothing, room: 0 }, "close only withdraws");
        wire.take(down);
    }
    assert_eq!(wire.answer(), None, "closed has no demand");
}

/// A held-input driver with exactly one decoder entry point per call.
/// `pending` prepares deliveries outside the meter; `drop_events` hands outputs out.
#[derive(Debug)]
pub struct Stepped {
    env: Env<Limits>,
    decoder: hearth_protocol_git::limits::Decoder,
    listing: Option<Listing>,
    batch: Option<hearth_protocol_git::batch::Batch>,
    diff: Option<hearth_protocol_git::hunks::Diff>,
    diff_events: Queue<hearth_protocol_git::hunks::Event>,
    held: skein_lib::stream::Held,
    requests: std::vec::IntoIter<hearth_protocol_git::batch::Request>,
    listing_events: Queue<Event>,
    batch_events: Queue<hearth_protocol_git::batch::Event>,
    below: Queue<skein_lib::stream::Down>,
    read: Option<Read>,
    action: Option<Action>,
    closed: bool,
}
#[derive(Debug)]
enum Action {
    Diff,
    List,
    Batch(hearth_protocol_git::batch::Request),
    Up(Up),
    Close,
}
impl Stepped {
    #[must_use]
    pub fn new(case: &crate::fixtures::Case, limits: &Limits) -> Self {
        Self {
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            decoder: case.decoder,
            listing: None,
            batch: None,
            diff: None,
            diff_events: Queue::with_capacity(1),
            held: skein_lib::stream::Held::new(case.stdout.clone().into_boxed_slice()),
            requests: Vec::from(case.requests.clone()).into_iter(),
            listing_events: Queue::with_capacity(1),
            batch_events: Queue::with_capacity(1),
            below: Queue::with_capacity(1),
            read: None,
            action: None,
            closed: false,
        }
    }
    pub fn open(&mut self) {
        use hearth_protocol_git::{batch::Batch, limits::Decoder};
        match self.decoder {
            Decoder::Diff => {
                self.diff = Some(hearth_protocol_git::hunks::Diff::new(&self.env.limits));
                self.action = Some(Action::Diff);
            }
            Decoder::Listing => {
                self.listing = Some(Listing::new(&self.env.limits));
                self.action = Some(Action::List);
            }
            Decoder::Batch => {
                self.batch = Some(Batch::START);
                self.action = Some(Action::Batch(self.requests.next().expect("batch case has request")));
            }
        }
    }
    pub fn pending(&mut self) -> bool {
        if self.closed {
            return false;
        }
        if self.action.is_none() {
            let read = self.read.take().expect("unfinished machine demands input");
            self.action = Some(Action::Up(self.held.answer(read).expect("active held demand")));
        }
        true
    }
    pub fn call(&mut self) {
        use hearth_protocol_git::{batch, limits::Decoder};
        let action = self.action.take().expect("prepared call");
        match self.decoder {
            Decoder::Diff => {
                use hearth_protocol_git::hunks::{self, Request};
                let machine = self.diff.as_mut().expect("open diff");
                match action {
                    Action::Diff => {
                        hunks::down(machine, &self.env, Request::Read, &mut self.diff_events, &mut self.below);
                    }
                    Action::Close => {
                        hunks::down(machine, &self.env, Request::Close, &mut self.diff_events, &mut self.below);
                    }
                    Action::Up(up) => hunks::up(machine, &self.env, up, &mut self.diff_events, &mut self.below),
                    Action::List | Action::Batch(_) => unreachable!("diff driver"),
                }
            }
            Decoder::Listing => {
                let machine = self.listing.as_mut().expect("open listing");
                match action {
                    Action::List => {
                        listing::down(machine, &self.env, Request::List, &mut self.listing_events, &mut self.below);
                    }
                    Action::Close => {
                        listing::down(machine, &self.env, Request::Close, &mut self.listing_events, &mut self.below);
                    }
                    Action::Up(up) => listing::up(machine, &self.env, up, &mut self.listing_events, &mut self.below),
                    Action::Batch(_) | Action::Diff => unreachable!("listing driver"),
                }
            }
            Decoder::Batch => {
                let machine = self.batch.as_mut().expect("open batch");
                match action {
                    Action::Batch(request) => {
                        batch::down(machine, &self.env, request, &mut self.batch_events, &mut self.below);
                    }
                    Action::Close => {
                        batch::down(machine, &self.env, batch::Request::Close, &mut self.batch_events, &mut self.below);
                    }
                    Action::Up(up) => batch::up(machine, &self.env, up, &mut self.batch_events, &mut self.below),
                    Action::List | Action::Diff => unreachable!("batch driver"),
                }
            }
        }
    }
    pub fn drop_events(&mut self) {
        use hearth_protocol_git::batch;
        if let Some(event) = self.diff_events.pop() {
            match event {
                hearth_protocol_git::hunks::Event::Hunks(_) | hearth_protocol_git::hunks::Event::Failed(_) => {
                    self.action = Some(Action::Close);
                }
                hearth_protocol_git::hunks::Event::Closed => self.closed = true,
            }
            drop(event);
        }
        if let Some(event) = self.listing_events.pop() {
            match event {
                Event::Listed(_) | Event::Failed(_) => self.action = Some(Action::Close),
                Event::Closed => self.closed = true,
            }
            drop(event);
        }
        if let Some(event) = self.batch_events.pop() {
            match event {
                batch::Event::Piece(_) => self.action = Some(Action::Batch(batch::Request::Next)),
                batch::Event::Whole { .. } | batch::Event::Missing { .. } => {
                    self.action = Some(match self.requests.next() {
                        Some(request) => Action::Batch(request),
                        None => Action::Close,
                    });
                }
                batch::Event::Failed(_) => self.action = Some(Action::Close),
                batch::Event::Closed => self.closed = true,
            }
            drop(event);
        }
        if let Some(down) = self.below.pop() {
            match down {
                skein_lib::stream::Down::Demand { read: Read::Nothing, room: 0 } => self.read = None,
                skein_lib::stream::Down::Demand { read, room: 0 } => self.read = Some(read),
                skein_lib::stream::Down::Demand { .. }
                | skein_lib::stream::Down::Send(_)
                | skein_lib::stream::Down::Finish => unreachable!("read only"),
            }
        }
    }
}

pub use crate::replay::replay;

#[derive(Debug)]
pub(crate) enum Source {
    Cut(Wire),
    Quiet {
        input: skein_lib::Intake,
        pending: Box<[u8]>,
        cursor: usize,
        rng: skein_lib::Rng,
        demand: Option<Read>,
        ended: bool,
    },
    Whole {
        held: skein_lib::stream::Held,
        demand: Option<Read>,
    },
}
impl Source {
    pub(crate) fn quiet(stdout: &[u8], settings: &Settings) -> Self {
        if settings.whole {
            return Self::new(stdout, settings);
        }
        let bytes = &stdout[..settings.end.unwrap_or(stdout.len()).min(stdout.len())];
        Self::Quiet {
            input: skein_lib::Intake::with_capacity(32768),
            pending: bytes.into(),
            cursor: 0,
            rng: skein_lib::Rng::new(settings.seed),
            demand: None,
            ended: false,
        }
    }

    pub(crate) fn new(stdout: &[u8], settings: &Settings) -> Self {
        let bytes = &stdout[..settings.end.unwrap_or(stdout.len()).min(stdout.len())];
        if settings.whole {
            Self::Whole { held: skein_lib::stream::Held::new(bytes.into()), demand: None }
        } else {
            let mut wire = Wire::new(settings.seed);
            wire.write(bytes);
            wire.eof = true;
            Self::Cut(wire)
        }
    }
    pub(crate) fn take(&mut self, down: skein_lib::stream::Down) {
        match self {
            Self::Cut(wire) => wire.take(down),
            Self::Quiet { demand, .. } => match down {
                skein_lib::stream::Down::Demand { read: Read::Nothing, room: 0 } => *demand = None,
                skein_lib::stream::Down::Demand { read, room: 0 } => {
                    assert!(demand.is_none(), "one demand");
                    *demand = Some(read);
                }
                skein_lib::stream::Down::Demand { .. }
                | skein_lib::stream::Down::Send(_)
                | skein_lib::stream::Down::Finish => unreachable!("read-only quiet source"),
            },
            Self::Whole { demand, .. } => match down {
                skein_lib::stream::Down::Demand { read: Read::Nothing, room: 0 } => *demand = None,
                skein_lib::stream::Down::Demand { read, room: 0 } => *demand = Some(read),
                skein_lib::stream::Down::Demand { .. }
                | skein_lib::stream::Down::Send(_)
                | skein_lib::stream::Down::Finish => unreachable!("read-only source"),
            },
        }
    }
    pub(crate) fn answer(&mut self) -> Option<Up> {
        match self {
            Self::Cut(wire) => wire.answer(),
            Self::Quiet { input, pending, cursor, rng, demand, ended } => {
                if *cursor < pending.len() && rng.chance(700) {
                    let count =
                        usize::try_from(rng.between(1, 31)).expect("small fragment").min(pending.len() - *cursor);
                    input.append(&pending[*cursor..*cursor + count]).expect("bounded intake");
                    *cursor += count;
                }
                let read = (*demand)?;
                if let Some(bytes) = input.meet(read) {
                    *demand = None;
                    Some(Up::Bytes(bytes))
                } else if *cursor == pending.len() && !*ended {
                    *ended = true;
                    Some(Up::End)
                } else {
                    None
                }
            }
            Self::Whole { held, demand } => demand.take().and_then(|read| held.answer(read)),
        }
    }
}

pub use crate::hunks::{DiffOutcome, diff};
