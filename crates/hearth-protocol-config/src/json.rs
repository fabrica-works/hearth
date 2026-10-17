//! One owned tree from tokenizer demands (format.md, 2.2; render.md, 3.1).
use crate::{
    Error, Fault, Position,
    limits::{self, Limits},
};
use alloc::boxed::Box;
use core::mem;
use skein_json::{Token, tokenizer as tk};
use skein_lib::{
    Env, List, Map, Queue, Stack,
    stream::{Down, Up},
};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    Decode,
    Close,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Decoded(Value),
    Failed(Error),
    Closed,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Waiting {
    Decode,
    Bytes,
    Close,
    Nothing,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Value {
    pub at: Position,
    pub kind: Kind,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Null,
    Bool(bool),
    Int(i64),
    List(Box<[Value]>),
    String(Box<[u8]>),
    Map(Box<[Entry]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub key: Box<[u8]>,
    pub at: Position,
    pub value: Value,
}
#[derive(Debug)]
pub struct Decoder {
    tokenizer: tk::Tokenizer,
    events: Queue<tk::Event>,
    open: Stack<Frame>,
    root: Option<Value>,
    state: Waiting,
}
#[derive(Debug)]
enum Frame {
    Object { at: Position, entries: Map<Box<[u8]>, Held>, key: Option<Box<[u8]>> },
    Array { at: Position, values: List<Value> },
}
#[derive(Debug)]
struct Held {
    at: Position,
    value: Option<Value>,
}
pub const UP_MAX_OUT: tk::MaxOut = tk::MaxOut { above: 1, below: 1 };
pub const DOWN_MAX_OUT: tk::MaxOut = UP_MAX_OUT;
impl Decoder {
    #[must_use]
    pub fn new(limits: &Limits) -> Decoder {
        Decoder {
            tokenizer: tk::Tokenizer::new(&limits::tokenizer(limits)),
            events: Queue::with_capacity(1),
            open: Stack::with_capacity(limits.depth),
            root: None,
            state: Waiting::Decode,
        }
    }
    #[must_use]
    pub fn waiting(&self) -> Waiting {
        self.state
    }
}
fn environment(env: &Env<Limits>) -> Env<tk::Limits> {
    Env { now: env.now, wall: env.wall, limits: limits::tokenizer(&env.limits) }
}
pub fn up(decoder: &mut Decoder, env: &Env<Limits>, ev: Up, above: &mut Queue<Event>, below: &mut Queue<Down>) {
    tk::up(&mut decoder.tokenizer, &environment(env), ev, &mut decoder.events, below);
    drive(decoder, env, above, below);
}
pub fn down(decoder: &mut Decoder, env: &Env<Limits>, rq: Request, above: &mut Queue<Event>, below: &mut Queue<Down>) {
    match rq {
        Request::Decode => {
            assert!(decoder.state == Waiting::Decode, "one Decode before its outcome");
            decoder.state = Waiting::Bytes;
            next(decoder, env, below);
            drive(decoder, env, above, below);
        }
        Request::Close => {
            assert!(decoder.state != Waiting::Nothing, "one Close");
            stop(decoder, env, below);
            clear(decoder);
            decoder.state = Waiting::Nothing;
            above.push(Event::Closed);
        }
    }
}
fn next(decoder: &mut Decoder, env: &Env<Limits>, below: &mut Queue<Down>) {
    tk::down(&mut decoder.tokenizer, &environment(env), tk::Request::Next, &mut decoder.events, below);
}
fn stop(decoder: &mut Decoder, env: &Env<Limits>, below: &mut Queue<Down>) {
    match decoder.tokenizer.waiting() {
        tk::Waiting::Nothing => {}
        tk::Waiting::Next | tk::Waiting::Bytes | tk::Waiting::Close => {
            tk::down(&mut decoder.tokenizer, &environment(env), tk::Request::Close, &mut decoder.events, below);
            assert!(decoder.events.pop() == Some(tk::Event::Closed), "the tokenizer closes once");
        }
    }
}
fn clear(decoder: &mut Decoder) {
    decoder.root = None;
    for _ in 0..decoder.open.len() {
        decoder.open.pop().expect("the current open count");
    }
}
fn drive(decoder: &mut Decoder, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Queue<Down>) {
    // Held lookahead can answer the next demand without another delivery.
    for _ in 0..env.limits.document.saturating_add(2) {
        let Some(event) = decoder.events.pop() else {
            return;
        };
        let at = decoder.tokenizer.position();
        let result = match event {
            tk::Event::Token(token) => accept(decoder, &env.limits, token, at),
            tk::Event::Done => {
                above.push(Event::Decoded(decoder.root.take().expect("a root before Done")));
                decoder.state = Waiting::Close;
                return;
            }
            tk::Event::Failed(error) => Err(Error {
                at,
                fault: match error {
                    tk::Error::Stream(fault) => Fault::Stream(fault),
                    tk::Error::Unexpected
                    | tk::Error::Trailing
                    | tk::Error::TooLong
                    | tk::Error::TooDeep
                    | tk::Error::StringTooLong
                    | tk::Error::NumberTooLong
                    | tk::Error::Number
                    | tk::Error::Escape
                    | tk::Error::Surrogate
                    | tk::Error::Utf8
                    | tk::Error::Control
                    | tk::Error::Truncated => Fault::Json(error),
                },
            }),
            tk::Event::Long(_) | tk::Event::Skipped(_) | tk::Event::Closed => unreachable!("only Next is demanded"),
        };
        match result {
            Ok(()) => next(decoder, env, below),
            Err(error) => {
                stop(decoder, env, below);
                clear(decoder);
                decoder.state = Waiting::Close;
                above.push(Event::Failed(error));
                return;
            }
        }
    }
    unreachable!("each immediate answer consumes a token within the document bound");
}
fn accept(decoder: &mut Decoder, limits: &Limits, token: Token, at: Position) -> Result<(), Error> {
    if decoder.open.is_empty() && decoder.root.is_none() && token != Token::ObjectStart {
        return Err(Error { at, fault: Fault::NotObject });
    }
    match decoder.open.top() {
        Some(Frame::Object { key: Some(key), .. }) if &**key == b"_note" => match &token {
            Token::String(_) => {}
            Token::ObjectStart
            | Token::ObjectEnd
            | Token::ArrayStart
            | Token::ArrayEnd
            | Token::Key(_)
            | Token::Number(_)
            | Token::True
            | Token::False
            | Token::Null => return Err(Error { at, fault: Fault::Note }),
        },
        Some(Frame::Object { .. } | Frame::Array { .. }) | None => {}
    }
    let kind = match token {
        Token::ObjectStart => {
            decoder
                .open
                .push(Frame::Object { at, entries: Map::with_capacity(limits.keys), key: None })
                .expect("the tokenizer enforces depth");
            return Ok(());
        }
        Token::ArrayStart => {
            decoder
                .open
                .push(Frame::Array { at, values: List::with_capacity(0) })
                .expect("the tokenizer enforces depth");
            return Ok(());
        }
        Token::Key(key) => return key_read(decoder, key, at),
        Token::ObjectEnd | Token::ArrayEnd => {
            let frame = decoder.open.pop().expect("the tokenizer balanced containers");
            return place(decoder, finish(frame));
        }
        Token::Null => Kind::Null,
        Token::True => Kind::Bool(true),
        Token::False => Kind::Bool(false),
        Token::String(text) => {
            if text.contains(&0) {
                return Err(Error { at, fault: Fault::Nul });
            }
            Kind::String(text)
        }
        Token::Number(text) => match integer(&text) {
            Ok(value) => Kind::Int(value),
            Err(fault) => return Err(Error { at, fault }),
        },
    };
    place(decoder, Value { at, kind })
}
fn key_read(decoder: &mut Decoder, key: Box<[u8]>, at: Position) -> Result<(), Error> {
    if key.contains(&0) {
        return Err(Error { at, fault: Fault::Nul });
    }
    match decoder.open.top_mut().expect("keys belong to an object") {
        Frame::Object { entries, key: pending, .. } => {
            if let Some(first) = entries.get(&*key) {
                return Err(Error { at, fault: Fault::Duplicate { key, first: first.at } });
            }
            let stored = key.clone();
            if entries.insert(stored, Held { at, value: None }).is_err() {
                return Err(Error { at, fault: Fault::Keys });
            }
            *pending = Some(key);
        }
        Frame::Array { .. } => unreachable!("the tokenizer places keys only in objects"),
    }
    Ok(())
}
fn integer(text: &[u8]) -> Result<i64, Fault> {
    for byte in text {
        if *byte == b'.' || *byte == b'e' || *byte == b'E' {
            return Err(Fault::NotInteger);
        }
    }
    let negative = text.first() == Some(&b'-');
    let digits = if negative { text.get(1..).expect("a leading minus") } else { text };
    let mut value = 0_i64;
    for digit in digits {
        let digit = i64::from(digit.checked_sub(b'0').expect("validated number digit"));
        let shifted = value.checked_mul(10).ok_or(Fault::Range)?;
        value = if negative { shifted.checked_sub(digit) } else { shifted.checked_add(digit) }.ok_or(Fault::Range)?;
    }
    Ok(value)
}
fn place(decoder: &mut Decoder, value: Value) -> Result<(), Error> {
    match decoder.open.top_mut() {
        None => decoder.root = Some(value),
        Some(Frame::Object { entries, key, .. }) => {
            let key = key.take().expect("a value follows a key");
            if &*key == b"_note" {
                match value.kind {
                    Kind::String(_) => return Ok(()),
                    Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::Map(_) => {
                        return Err(Error { at: value.at, fault: Fault::Note });
                    }
                }
            }
            entries.get_mut(&*key).expect("the key was counted").value = Some(value);
        }
        Some(Frame::Array { values, .. }) => {
            if values.room() == 0 {
                let capacity = values.capacity().saturating_mul(2).max(1);
                let old = mem::replace(values, List::with_capacity(capacity));
                for item in old.into_boxed() {
                    values.push(item).expect("doubled capacity");
                }
            }
            values.push(value).expect("room reserved");
        }
    }
    Ok(())
}
fn finish(frame: Frame) -> Value {
    match frame {
        Frame::Array { at, values } => Value { at, kind: Kind::List(values.into_boxed()) },
        Frame::Object { at, mut entries, .. } => {
            let mut output = List::with_capacity(entries.len());
            for _ in 0..entries.len() {
                let key = entries.first().expect("remaining entries").0.clone();
                let held = entries.remove(&*key).expect("the first entry");
                if let Some(value) = held.value {
                    output.push(Entry { key, at: held.at, value }).expect("one entry per key");
                }
            }
            Value { at, kind: Kind::Map(output.into_boxed()) }
        }
    }
}

/// The decoder's peak heap, including the tree under construction
/// (programming-model.md, 6.3; render.md, 2.3).
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let tokenizer = tk::worst_case(&limits::tokenizer(limits))?;
    let frames = Stack::<Frame>::worst_case(limits.depth)?;
    let events = Queue::<tk::Event>::worst_case(1)?;
    let maps = Map::<Box<[u8]>, Held>::worst_case(limits.keys)?.checked_mul(u64::from(limits.depth))?;
    let values = u64::from(limits.document).checked_div(2)?.checked_add(1)?;
    // The tree, a doubling list's old and new storage, and object finishing's
    // entry list may coexist; all have at most this many values/entries.
    let records = u64::try_from(size_of::<Value>().max(size_of::<Entry>())).ok()?;
    let storage = values.checked_mul(records)?.checked_mul(4)?;
    // Decoded text, counted keys, pending keys and a finishing key's copy.
    let text = u64::from(limits.document).checked_mul(4)?;
    tokenizer.checked_add(frames)?.checked_add(events)?.checked_add(maps)?.checked_add(storage)?.checked_add(text)
}
