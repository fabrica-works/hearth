//! cat-file --batch's bounded header and streamed body (render.md, 2.2–2.3).
use crate::{MaxOut, limits::Limits, oid::Oid};
use alloc::boxed::Box;
use skein_lib::{
    Env, Queue, Writer, bytes,
    stream::{Delimiter, Down, Fault, Read, Up},
};
pub const HEADER: u32 = 93;
pub const UP_MAX_OUT: MaxOut = MaxOut { above: 1, below: 1 };
pub const DOWN_MAX_OUT: MaxOut = MaxOut { above: 1, below: 1 };
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    Read { oid: Oid, size: u64, max: u64 },
    Next,
    Close,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Piece(Box<[u8]>),
    Whole { oid: Oid, size: u64 },
    Missing { oid: Oid },
    Failed(Error),
    Closed,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    Malformed,
    Other { oid: Oid },
    Size { oid: Oid, listed: u64, header: u64 },
    TooLarge { oid: Oid, size: u64, max: u64 },
    Truncated,
    Stream(Fault),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Blob {
    oid: Oid,
    size: u64,
    max: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum State {
    Idle,
    Stopped(Error),
    First(Blob),
    Header { first: u8, blob: Blob },
    Body { blob: Blob, left: u64 },
    Paused { blob: Blob, left: u64 },
    Line(Blob),
    Over,
    Closed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Batch {
    state: State,
}
impl Batch {
    pub const START: Self = Self { state: State::Idle };
}
#[must_use]
pub fn largest_demand(limits: &Limits) -> u32 {
    HEADER.max(limits.chunk)
}
/// The caller sends this line to stdin; the decoder only reads stdout.
#[must_use]
pub fn request_line(oid: Oid) -> Box<[u8]> {
    let mut writer = Writer::new(oid.hex().len().checked_add(1).expect("bounded id"));
    writer.put(oid.hex()).expect("id sized first");
    writer.put(b"\n").expect("line feed sized first");
    writer.finish()
}
fn reading(state: State, limits: &Limits) -> Read {
    match state {
        State::First(_) | State::Line(_) => Read::Fill(1),
        State::Header { .. } => Read::Scan { until: Delimiter::LF, max: HEADER },
        State::Body { left, .. } => {
            assert!(limits.chunk > 0, "positive progress chunk");
            Read::Arrived { max: u32::try_from(left.min(u64::from(limits.chunk))).expect("bounded chunk") }
        }
        State::Idle | State::Stopped(_) | State::Paused { .. } | State::Over | State::Closed => Read::Nothing,
    }
}
fn demand(machine: &Batch, limits: &Limits, below: &mut Queue<Down>) {
    below.push(Down::Demand { read: reading(machine.state, limits), room: 0 });
}
fn fail(machine: &mut Batch, error: Error, above: &mut Queue<Event>) {
    machine.state = State::Over;
    above.push(Event::Failed(error));
}
/// Read starts one object; Next asks for its next piece or its Whole.
pub fn down(
    machine: &mut Batch,
    env: &Env<Limits>,
    request: Request,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    match request {
        Request::Read { oid, size, max } => {
            match machine.state {
                State::Stopped(error) => {
                    fail(machine, error, above);
                    return;
                }
                State::Idle
                | State::First(_)
                | State::Header { .. }
                | State::Body { .. }
                | State::Paused { .. }
                | State::Line(_)
                | State::Over
                | State::Closed => {}
            }
            assert_eq!(machine.state, State::Idle, "Read follows Whole or Missing");
            machine.state = State::First(Blob { oid, size, max });
            demand(machine, &env.limits, below);
        }
        Request::Next => {
            match machine.state {
                State::Stopped(error) => {
                    fail(machine, error, above);
                    return;
                }
                State::Idle
                | State::First(_)
                | State::Header { .. }
                | State::Body { .. }
                | State::Paused { .. }
                | State::Line(_)
                | State::Over
                | State::Closed => {}
            }
            machine.state = match machine.state {
                State::Paused { blob, left: 0 } => State::Line(blob),
                State::Paused { blob, left } => State::Body { blob, left },
                State::Idle
                | State::Stopped(_)
                | State::First(_)
                | State::Header { .. }
                | State::Body { .. }
                | State::Line(_)
                | State::Over
                | State::Closed => unreachable!("Next follows Piece"),
            };
            demand(machine, &env.limits, below);
        }
        Request::Close => {
            let pending = reading(machine.state, &env.limits) != Read::Nothing;
            machine.state = State::Closed;
            if pending {
                demand(machine, &env.limits, below);
            }
            above.push(Event::Closed);
        }
    }
}
/// One demanded delivery; Piece moves its delivery directly to the receiver.
pub fn up(machine: &mut Batch, env: &Env<Limits>, event: Up, above: &mut Queue<Event>, below: &mut Queue<Down>) {
    if machine.state == State::Closed || machine.state == State::Over {
        return;
    }
    match event {
        Up::Bytes(bytes) => {
            match machine.state {
                State::First(blob) => machine.state = State::Header { first: *bytes.first().expect("Fill(1)"), blob },
                State::Header { first, blob } => match header(first, &bytes, blob) {
                    Ok(true) => {
                        machine.state = State::Idle;
                        above.push(Event::Missing { oid: blob.oid });
                    }
                    Ok(false) => {
                        machine.state =
                            if blob.size == 0 { State::Line(blob) } else { State::Body { blob, left: blob.size } }
                    }
                    Err(error) => fail(machine, error, above),
                },
                State::Body { blob, left } => {
                    let left = left
                        .checked_sub(u64::try_from(bytes.len()).expect("chunk fits u64"))
                        .expect("delivery meets demand");
                    machine.state = State::Paused { blob, left };
                    above.push(Event::Piece(bytes));
                }
                State::Line(blob) => {
                    if &*bytes == b"\n" {
                        machine.state = State::Idle;
                        above.push(Event::Whole { oid: blob.oid, size: blob.size });
                    } else {
                        fail(machine, Error::Malformed, above);
                    }
                }
                State::Idle | State::Stopped(_) | State::Paused { .. } | State::Over | State::Closed => {
                    unreachable!("bytes require demand")
                }
            }
            if reading(machine.state, &env.limits) != Read::Nothing {
                demand(machine, &env.limits, below);
            }
        }
        Up::End | Up::Failed(_) => {
            let pending = reading(machine.state, &env.limits) != Read::Nothing;
            let error = match event {
                Up::End => Error::Truncated,
                Up::Failed(fault) => Error::Stream(fault),
                Up::Bytes(_) | Up::Room => unreachable!("end or fault"),
            };
            if pending {
                fail(machine, error, above);
                demand(machine, &env.limits, below);
            } else {
                machine.state = State::Stopped(error);
            }
        }
        Up::Room => unreachable!("batch asks for no room"),
    }
}
fn header(first: u8, bytes: &[u8], blob: Blob) -> Result<bool, Error> {
    let line = bytes.strip_suffix(b"\n").ok_or(Error::Malformed)?;
    let at = bytes::find(line, b" ").ok_or(Error::Malformed)?;
    let hex = line.get(..at).ok_or(Error::Malformed)?;
    let mut digits = [first; 64];
    for (slot, byte) in digits.iter_mut().skip(1).zip(hex) {
        *slot = *byte;
    }
    let length = hex.len().checked_add(1).ok_or(Error::Malformed)?;
    let oid = Oid::parse(digits.get(..length).ok_or(Error::Malformed)?).ok_or(Error::Malformed)?;
    if oid != blob.oid {
        return Err(Error::Other { oid });
    }
    let rest = line.get(at.checked_add(1).ok_or(Error::Malformed)?..).ok_or(Error::Malformed)?;
    if rest == b"missing" {
        return Ok(true);
    }
    let Some(size) = rest.strip_prefix(b"blob ") else {
        return Err(Error::Other { oid });
    };
    let header = crate::listing::number(size).ok_or(Error::Malformed)?;
    if header != blob.size {
        return Err(Error::Size { oid, listed: blob.size, header });
    }
    if header > blob.max {
        return Err(Error::TooLarge { oid, size: header, max: blob.max });
    }
    Ok(false)
}
