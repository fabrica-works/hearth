//! Writes one record at a time over the stream write half (lib.md, 7).
use core::mem;

use alloc::boxed::Box;

use skein_lib::stream::{Down, Fault, Read, Up};
use skein_lib::{Env, Queue, bytes};

use crate::{
    form::{self, Form, Refusal},
    limits::Limits,
    records::{Record, Type},
};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaxOut {
    pub above: u32,
    pub below: u32,
}

#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.chunk == 0 {
        return None;
    }
    Some(u64::from(limits.record))
}

pub const UP_MAX_OUT: MaxOut = MaxOut { above: 1, below: 2 };

pub const DOWN_MAX_OUT: MaxOut = MaxOut { above: 1, below: 1 };

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    Print(Record),
    Finish,
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Printed,
    Refused(Refusal),
    Failed(Fault),
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Waiting {
    Above,
    Room,
    Close,
    Nothing,
}

#[derive(Debug)]
pub struct Printer {
    form: Form,
    after: Option<Type>,
    state: State,
}

impl Printer {
    #[must_use]
    pub fn new(form: Form, limits: &Limits) -> Printer {
        assert!(limits.chunk > 0, "room for a byte at least");
        Printer { form, after: None, state: State::Idle(Below::Open) }
    }

    #[must_use]
    pub fn waiting(&self) -> Waiting {
        match self.state {
            State::Idle(_) => Waiting::Above,
            State::Writing(_) => Waiting::Room,
            State::Finished | State::Over => Waiting::Close,
            State::Closed => Waiting::Nothing,
        }
    }
}

pub fn up(writer: &mut Printer, env: &Env<Limits>, ev: Up, above: &mut Queue<Event>, below: &mut Queue<Down>) {
    let limits = &env.limits;
    let state = mem::replace(&mut writer.state, State::Closed);
    writer.state = match ev {
        Up::Room => match state {
            State::Writing(writing) => room(writing, limits.chunk, above, below),
            State::Closed => State::Closed,
            State::Idle(_) | State::Finished | State::Over => unreachable!("room answers a demand for room"),
        },
        Up::Failed(fault) => match state {
            State::Writing(_) => {
                above.push(Event::Failed(fault));
                State::Over
            }
            State::Idle(Below::Open) => State::Idle(Below::Failed(fault)),
            state @ (State::Idle(Below::Failed(_)) | State::Finished | State::Over | State::Closed) => state,
        },
        Up::End => state,
        Up::Bytes(_) => unreachable!("the writer reads nothing"),
    };
}

pub fn down(writer: &mut Printer, env: &Env<Limits>, rq: Request, above: &mut Queue<Event>, below: &mut Queue<Down>) {
    let limits = &env.limits;
    let state = mem::replace(&mut writer.state, State::Closed);
    writer.state = match rq {
        Request::Print(record) => match state {
            State::Idle(told) => {
                let frame = form::print(writer.form, &record, writer.after, limits);
                if frame.is_ok() {
                    writer.after = Some(record.kind());
                }
                write(told, frame, limits.chunk, above, below)
            }
            State::Writing(_) => unreachable!("an event while another is being written"),
            State::Finished => unreachable!("an event after Finish"),
            State::Over => unreachable!("an event after the stream failed"),
            State::Closed => unreachable!("an event after Closed"),
        },
        Request::Finish => match state {
            State::Idle(Below::Open) => {
                below.push(Down::Finish);
                State::Finished
            }
            State::Idle(Below::Failed(_)) => State::Finished,
            State::Writing(_) => unreachable!("a Finish while an event is being written"),
            State::Finished => unreachable!("a second Finish"),
            State::Over => unreachable!("a Finish after the stream failed"),
            State::Closed => unreachable!("a Finish after Closed"),
        },
        Request::Close => {
            match state {
                State::Writing(_) => below.push(Down::Demand { read: Read::Nothing, room: 0 }),
                State::Idle(_) | State::Finished | State::Over => {}
                State::Closed => unreachable!("a Close after Closed"),
            }
            above.push(Event::Closed);
            State::Closed
        }
    };
}

#[derive(Debug)]
enum State {
    Idle(Below),
    Writing(Writing),
    Finished,
    Over,
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Below {
    Open,
    Failed(Fault),
}

#[derive(Debug)]
struct Writing {
    frame: Box<[u8]>,
    sent: u32,
}

fn write(
    told: Below,
    frame: Result<Box<[u8]>, Refusal>,
    chunk: u32,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) -> State {
    let frame = match frame {
        Ok(frame) => frame,
        Err(refusal) => {
            above.push(Event::Refused(refusal));
            return State::Idle(told);
        }
    };
    match told {
        Below::Open => {}
        Below::Failed(fault) => {
            above.push(Event::Failed(fault));
            return State::Over;
        }
    }
    if frame.is_empty() {
        above.push(Event::Printed);
        return State::Idle(Below::Open);
    }
    let writing = Writing { frame, sent: 0 };
    below.push(Down::Demand { read: Read::Nothing, room: next(&writing, chunk) });
    State::Writing(writing)
}

fn next(writing: &Writing, chunk: u32) -> u32 {
    let len = u32::try_from(writing.frame.len()).expect("an event within Limits::record");
    let left = len.checked_sub(writing.sent).expect("no more sent than the event");
    left.min(chunk)
}

fn room(mut writing: Writing, chunk: u32, above: &mut Queue<Event>, below: &mut Queue<Down>) -> State {
    let piece = next(&writing, chunk);
    let len = writing.frame.len();
    if writing.sent == 0 && usize::try_from(piece).expect("a u32 fits a usize") == len {
        below.push(Down::Send(writing.frame));
        above.push(Event::Printed);
        return State::Idle(Below::Open);
    }
    let from = usize::try_from(writing.sent).expect("a u32 fits a usize");
    let to = from.checked_add(usize::try_from(piece).expect("a u32 fits a usize")).expect("within the event");
    below.push(Down::Send(bytes::copy_of(writing.frame.get(from..to).expect("a piece of the event"))));
    writing.sent = writing.sent.checked_add(piece).expect("within the event");
    if usize::try_from(writing.sent).expect("a u32 fits a usize") == len {
        above.push(Event::Printed);
        return State::Idle(Below::Open);
    }
    below.push(Down::Demand { read: Read::Nothing, room: next(&writing, chunk) });
    State::Writing(writing)
}
