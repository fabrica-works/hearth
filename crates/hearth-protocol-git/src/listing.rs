//! ls-tree's NUL records, with carry held below (render.md, 2.2–2.3).
use crate::{MaxOut, limits::Limits, oid::Oid};
use alloc::boxed::Box;
use core::mem;
use skein_lib::{
    Env, List, Queue, bytes,
    stream::{Delimiter, Down, Fault, Read, Up},
};
/// Fixed fields' allowance before a path.
pub const RECORD: u32 = 100;
pub const UP_MAX_OUT: MaxOut = MaxOut { above: 1, below: 1 };
pub const DOWN_MAX_OUT: MaxOut = MaxOut { above: 1, below: 1 };
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Mode {
    File,
    Executable,
    Link,
    Submodule,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub path: Box<[u8]>,
    pub mode: Mode,
    pub oid: Oid,
    pub size: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    List,
    Close,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Listed(Box<[Entry]>),
    Failed(Error),
    Closed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    Malformed { entry: u32 },
    TooMany { path: Box<[u8]> },
    PathTooLong { prefix: Box<[u8]> },
    PathsTooLarge { path: Box<[u8]> },
    Truncated,
    Stream(Fault),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum State {
    Idle,
    First,
    Record(u8),
    Ended,
    Broken(Fault),
    Over,
    Closed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Listing {
    state: State,
    entries: List<Entry>,
    paths: u64,
}
impl Listing {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self { state: State::Idle, entries: List::with_capacity(limits.entries), paths: 0 }
    }
}
#[must_use]
pub fn largest_demand(limits: &Limits) -> u32 {
    RECORD.checked_add(limits.path).expect("listing demand fits u32")
}
fn reading(state: State, limits: &Limits) -> Read {
    match state {
        State::First => Read::Fill(1),
        State::Record(_) => {
            Read::Scan { until: Delimiter::new(b"\0").expect("one-byte delimiter"), max: largest_demand(limits) }
        }
        State::Idle | State::Ended | State::Broken(_) | State::Over | State::Closed => Read::Nothing,
    }
}
fn emit_read(state: State, limits: &Limits, below: &mut Queue<Down>) {
    let read = reading(state, limits);
    below.push(Down::Demand { read, room: 0 });
}
fn fail(machine: &mut Listing, error: Error, above: &mut Queue<Event>) {
    machine.entries = List::with_capacity(0);
    machine.state = State::Over;
    above.push(Event::Failed(error));
}
fn finish(machine: &mut Listing, above: &mut Queue<Event>) {
    let entries = mem::replace(&mut machine.entries, List::with_capacity(0));
    machine.state = State::Over;
    above.push(Event::Listed(entries.into_boxed()));
}
/// One request above; close withdraws any outstanding read.
pub fn down(
    machine: &mut Listing,
    env: &Env<Limits>,
    request: Request,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    match request {
        Request::List => match machine.state {
            State::Idle => {
                machine.state = State::First;
                emit_read(machine.state, &env.limits, below);
            }
            State::Ended => finish(machine, above),
            State::Broken(fault) => fail(machine, Error::Stream(fault), above),
            State::First | State::Record(_) | State::Over | State::Closed => unreachable!("one List before outcome"),
        },
        Request::Close => {
            let pending = reading(machine.state, &env.limits) != Read::Nothing;
            machine.state = State::Closed;
            machine.entries = List::with_capacity(0);
            if pending {
                emit_read(machine.state, &env.limits, below);
            }
            above.push(Event::Closed);
        }
    }
}
/// One stream delivery; at most one record is consumed.
pub fn up(machine: &mut Listing, env: &Env<Limits>, event: Up, above: &mut Queue<Event>, below: &mut Queue<Down>) {
    let old = machine.state;
    if old == State::Closed || old == State::Over {
        return;
    }
    match event {
        Up::Bytes(bytes) => {
            match old {
                State::First => machine.state = State::Record(*bytes.first().expect("Fill(1)")),
                State::Record(first) => {
                    match record(first, &bytes, &env.limits, machine.entries.len(), machine.paths) {
                        Ok(entry) => {
                            machine.paths = machine
                                .paths
                                .checked_add(u64::try_from(entry.path.len()).expect("path fits u64"))
                                .expect("bounded path sum");
                            machine.entries.push(entry).expect("entry limit checked");
                            machine.state = State::First;
                        }
                        Err(error) => fail(machine, error, above),
                    }
                }
                State::Idle | State::Ended | State::Broken(_) | State::Over | State::Closed => {
                    unreachable!("bytes require demand")
                }
            }
            if reading(machine.state, &env.limits) != Read::Nothing {
                emit_read(machine.state, &env.limits, below);
            }
        }
        Up::End => {
            match old {
                State::First => finish(machine, above),
                State::Record(_) => fail(machine, Error::Truncated, above),
                State::Idle => machine.state = State::Ended,
                State::Ended | State::Broken(_) | State::Over | State::Closed => {}
            }
            if reading(old, &env.limits) != Read::Nothing {
                emit_read(machine.state, &env.limits, below);
            }
        }
        Up::Failed(fault) => {
            match old {
                State::Idle | State::Ended => machine.state = State::Broken(fault),
                State::First | State::Record(_) => fail(machine, Error::Stream(fault), above),
                State::Broken(_) | State::Over | State::Closed => {}
            }
            if reading(old, &env.limits) != Read::Nothing {
                emit_read(machine.state, &env.limits, below);
            }
        }
        Up::Room => unreachable!("listing asks for no room"),
    }
}
fn split(input: &[u8], delimiter: u8) -> Option<(&[u8], &[u8])> {
    let at = bytes::find(input, &[delimiter])?;
    Some((input.get(..at)?, input.get(at.checked_add(1)?..)?))
}
fn number(input: &[u8]) -> Option<u64> {
    if input.is_empty() || (input.len() > 1 && input.starts_with(b"0")) {
        return None;
    }
    let mut value = 0_u64;
    for byte in input {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}
fn fields(first: u8, header: &[u8]) -> Option<(Mode, Oid, u64)> {
    if first != b'1' {
        return None;
    }
    let (mode, rest) = split(header, b' ')?;
    let (kind, rest) = split(rest, b' ')?;
    let (hex, padded) = split(rest, b' ')?;
    let mode = match mode {
        b"00644" => Mode::File,
        b"00755" => Mode::Executable,
        b"20000" => Mode::Link,
        b"60000" => Mode::Submodule,
        _ => return None,
    };
    let oid = Oid::parse(hex)?;
    let mut start = 0_usize;
    for byte in padded {
        if *byte != b' ' {
            break;
        }
        start = start.checked_add(1)?;
    }
    let digits = padded.get(start..)?;
    if padded.len() != digits.len().max(7) {
        return None;
    }
    let size = match mode {
        Mode::Submodule => {
            if kind != b"commit" || digits != b"-" {
                return None;
            }
            0
        }
        Mode::File | Mode::Executable | Mode::Link => {
            if kind != b"blob" {
                return None;
            }
            number(digits)?
        }
    };
    Some((mode, oid, size))
}
fn record(first: u8, bytes: &[u8], limits: &Limits, count: u32, total: u64) -> Result<Entry, Error> {
    let malformed = Error::Malformed { entry: count };
    let (header, path) = split(bytes, b'\t').ok_or(malformed.clone())?;
    let maximum = usize::try_from(limits.path).expect("u32 fits usize");
    let Some(path) = path.strip_suffix(b"\0") else {
        return Err(Error::PathTooLong {
            prefix: bytes::copy_of(path.get(..path.len().min(maximum)).expect("prefix within path")),
        });
    };
    let (mode, oid, size) = fields(first, header).ok_or(malformed.clone())?;
    if path.is_empty() {
        return Err(malformed);
    }
    if path.len() > maximum {
        return Err(Error::PathTooLong { prefix: bytes::copy_of(path.get(..maximum).expect("long path")) });
    }
    if count == limits.entries {
        return Err(Error::TooMany { path: bytes::copy_of(path) });
    }
    if total.checked_add(u64::try_from(path.len()).expect("path fits u64")).expect("u32 limits bound total")
        > u64::from(limits.paths_total)
    {
        return Err(Error::PathsTooLarge { path: bytes::copy_of(path) });
    }
    Ok(Entry { path: bytes::copy_of(path), mode, oid, size })
}
