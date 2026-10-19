//! Unified hunks with bounded lines and retained text (domain/render.md, 2.2 and 6.1).
use crate::{MaxOut, limits::Limits};
use alloc::boxed::Box;
use core::mem;
use skein_lib::{
    Env, List, Queue, bytes,
    stream::{Delimiter, Down, Fault, Read, Up},
};
pub const UP_MAX_OUT: MaxOut = MaxOut { above: 1, below: 1 };
pub const DOWN_MAX_OUT: MaxOut = MaxOut { above: 1, below: 1 };
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Kind {
    Hunk,
    Context,
    Removed,
    Added,
    NoNewline,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line {
    pub kind: Kind,
    pub text: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Hunks {
    pub lines: Box<[Line]>,
    pub left_out: u32,
    pub binary: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    Read,
    Close,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Hunks(Hunks),
    Failed(Error),
    Closed,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    Malformed,
    TooLarge,
    Truncated,
    Stream(Fault),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum State {
    Idle,
    First,
    Line(u8),
    Ended,
    Broken(Fault),
    Over,
    Closed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Diff {
    state: State,
    lines: List<Line>,
    pending: List<u8>,
    length: u32,
    text: u64,
    left_out: u32,
    started: bool,
    binary: bool,
}
impl Diff {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        assert!(limits.chunk > 0, "positive progress chunk");
        Self {
            state: State::Idle,
            lines: List::with_capacity(limits.hunk_lines),
            pending: List::with_capacity(limits.hunk_file),
            length: 0,
            text: 0,
            left_out: 0,
            started: false,
            binary: false,
        }
    }
}
fn reading(state: State, limits: &Limits) -> Read {
    match state {
        State::First => Read::Fill(1),
        State::Line(_) => Read::Scan { until: Delimiter::LF, max: limits.chunk },
        State::Idle | State::Ended | State::Broken(_) | State::Over | State::Closed => Read::Nothing,
    }
}
fn demand(machine: &Diff, limits: &Limits, below: &mut Queue<Down>) {
    below.push(Down::Demand { read: reading(machine.state, limits), room: 0 });
}
fn clear(machine: &mut Diff) {
    machine.lines = List::with_capacity(0);
    machine.pending = List::with_capacity(0);
}
fn fail(machine: &mut Diff, error: Error, above: &mut Queue<Event>) {
    clear(machine);
    machine.state = State::Over;
    above.push(Event::Failed(error));
}
fn finish(machine: &mut Diff, above: &mut Queue<Event>) {
    if !machine.started && !machine.binary {
        fail(machine, Error::Malformed, above);
        return;
    }
    let lines = mem::replace(&mut machine.lines, List::with_capacity(0));
    machine.pending = List::with_capacity(0);
    machine.state = State::Over;
    above.push(Event::Hunks(Hunks { lines: lines.into_boxed(), left_out: machine.left_out, binary: machine.binary }));
}
/// One Read before its terminal result; Close is accepted in every state.
pub fn down(
    machine: &mut Diff,
    env: &Env<Limits>,
    request: Request,
    above: &mut Queue<Event>,
    below: &mut Queue<Down>,
) {
    match request {
        Request::Read => match machine.state {
            State::Idle => {
                machine.state = State::First;
                demand(machine, &env.limits, below);
            }
            State::Ended => finish(machine, above),
            State::Broken(fault) => fail(machine, Error::Stream(fault), above),
            State::First | State::Line(_) | State::Over | State::Closed => unreachable!("one Read before outcome"),
        },
        Request::Close => {
            let pending = reading(machine.state, &env.limits) != Read::Nothing;
            clear(machine);
            machine.state = State::Closed;
            if pending {
                demand(machine, &env.limits, below);
            }
            above.push(Event::Closed);
        }
    }
}
/// Fill the first byte, then scan chunks to the line feed without a carry below.
pub fn up(machine: &mut Diff, env: &Env<Limits>, event: Up, above: &mut Queue<Event>, below: &mut Queue<Down>) {
    let old = machine.state;
    if old == State::Closed || old == State::Over {
        return;
    }
    match event {
        Up::Bytes(input) => {
            let result = match old {
                State::First => {
                    let first = *input.first().expect("Fill(1)");
                    machine.length = 1;
                    machine.state = State::Line(first);
                    if env.limits.hunk_file == 0 {
                        Err(Error::TooLarge)
                    } else if first == b'@' {
                        machine.pending.push(first).expect("one byte within file limit");
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
                State::Line(first) => append(machine, first, &input, &env.limits),
                State::Idle | State::Ended | State::Broken(_) | State::Over | State::Closed => {
                    unreachable!("bytes require demand")
                }
            };
            if let Err(error) = result {
                fail(machine, error, above);
            }
            if reading(machine.state, &env.limits) != Read::Nothing {
                demand(machine, &env.limits, below);
            }
        }
        Up::End => {
            match old {
                State::First => finish(machine, above),
                State::Line(_) => fail(machine, Error::Truncated, above),
                State::Idle => machine.state = State::Ended,
                State::Ended | State::Broken(_) | State::Over | State::Closed => {}
            }
            if reading(old, &env.limits) != Read::Nothing {
                demand(machine, &env.limits, below);
            }
        }
        Up::Failed(fault) => {
            match old {
                State::Idle | State::Ended => machine.state = State::Broken(fault),
                State::First | State::Line(_) => fail(machine, Error::Stream(fault), above),
                State::Broken(_) | State::Over | State::Closed => {}
            }
            if reading(old, &env.limits) != Read::Nothing {
                demand(machine, &env.limits, below);
            }
        }
        Up::Room => unreachable!("diff requests no room"),
    }
}
fn append(machine: &mut Diff, first: u8, input: &[u8], limits: &Limits) -> Result<(), Error> {
    let ended = input.ends_with(b"\n");
    let bytes = input.strip_suffix(b"\n").unwrap_or(input);
    machine.length =
        machine.length.checked_add(u32::try_from(bytes.len()).expect("chunk fits u32")).ok_or(Error::TooLarge)?;
    if machine.length > limits.hunk_file {
        return Err(Error::TooLarge);
    }
    for byte in bytes {
        machine.pending.push(*byte).expect("line length bounded");
    }
    if ended {
        line(machine, first, limits)?;
        machine.pending.clear();
        machine.state = State::First;
    }
    Ok(())
}
fn line(machine: &mut Diff, first: u8, limits: &Limits) -> Result<(), Error> {
    let text = machine.pending.as_slice();
    if !machine.started && first != b'@' {
        if first == b'B' && text.starts_with(b"inary files ") && text.ends_with(b" differ") {
            machine.binary = true;
        }
        return Ok(());
    }
    if machine.binary {
        return Err(Error::Malformed);
    }
    let kind = match first {
        b'@' => {
            if !text.starts_with(b"@@ ") || bytes::find(text, b" @@").is_none() {
                return Err(Error::Malformed);
            }
            machine.started = true;
            Kind::Hunk
        }
        b' ' => Kind::Context,
        b'-' => Kind::Removed,
        b'+' => Kind::Added,
        b'\\' => {
            if text != b" No newline at end of file" {
                return Err(Error::Malformed);
            }
            Kind::NoNewline
        }
        _ => return Err(Error::Malformed),
    };
    if machine.lines.len() == limits.hunk_lines {
        machine.left_out = machine.left_out.checked_add(1).ok_or(Error::TooLarge)?;
        return Ok(());
    }
    machine.text = machine.text.checked_add(u64::try_from(text.len()).expect("bounded line")).ok_or(Error::TooLarge)?;
    if machine.text > u64::from(limits.hunk_file).checked_mul(2).expect("u32 twice fits u64") {
        return Err(Error::TooLarge);
    }
    let text = bytes::copy_of(text);
    machine.lines.push(Line { kind, text }).expect("line count checked");
    Ok(())
}
