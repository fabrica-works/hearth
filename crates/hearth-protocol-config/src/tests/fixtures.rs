use crate::{
    Error,
    document::{self, Document},
    json::{self, Decoder, Event, Request, Value},
    limits::{Limits, SHIPPED},
};
use skein_lib::{
    Env, Queue, Time, Wall, bytes,
    stream::{Down, Read, Up},
};
pub fn decode(input: &[u8]) -> Result<Value, Error> {
    decode_under(input, &SHIPPED)
}
pub fn document(input: &[u8]) -> Result<Document, Error> {
    document::document(decode(input)?)
}
pub fn decode_under(input: &[u8], limits: &Limits) -> Result<Value, Error> {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut decoder = Decoder::new(limits);
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    json::down(&mut decoder, &env, Request::Decode, &mut above, &mut below);
    let mut offset = 0;
    for _ in 0..input.len().saturating_mul(4).saturating_add(16) {
        if let Some(event) = above.pop() {
            return match event {
                Event::Decoded(value) => Ok(value),
                Event::Failed(error) => Err(error),
                Event::Closed => panic!("no Close asked"),
            };
        }
        let read = match below.pop().expect("one demand") {
            Down::Demand { read, .. } => read,
            Down::Send(_) | Down::Finish => panic!("only reads"),
        };
        let remaining = &input[offset..];
        let count = match read {
            Read::Fill(n) => usize::try_from(n).expect("u32 fits usize"),
            Read::Scan { until, max } => {
                let max = usize::try_from(max).expect("u32 fits usize");
                let scan = &remaining[..remaining.len().min(max)];
                match bytes::find(scan, until.as_bytes()) {
                    Some(n) => n.saturating_add(until.as_bytes().len()),
                    None => max,
                }
            }
            Read::Nothing | Read::Line { .. } | Read::Arrived { .. } => panic!("tokenizer framing"),
        };
        let event = if count <= remaining.len() {
            offset = offset.saturating_add(count);
            Up::Bytes(bytes::copy_of(&remaining[..count]))
        } else {
            Up::End
        };
        json::up(&mut decoder, &env, event, &mut above, &mut below);
    }
    panic!("bounded decode settled");
}

pub fn past_shipped() -> [Box<[u8]>; 3] {
    let mut deepest = skein_lib::List::with_capacity(128);
    for byte in b"{\"a\":" {
        deepest.push(*byte).expect("room");
    }
    for _ in 0_u32..32 {
        deepest.push(b'[').expect("room");
    }
    deepest.push(b'0').expect("room");
    for _ in 0_u32..32 {
        deepest.push(b']').expect("room");
    }
    deepest.push(b'}').expect("room");
    let mut keys = skein_lib::List::with_capacity(60_000);
    keys.push(b'{').expect("room");
    for n in 0_u64..=4096 {
        if n > 0 {
            keys.push(b',').expect("room");
        }
        keys.push(b'"').expect("room");
        keys.push(b'k').expect("room");
        for byte in skein_lib::Decimal::of(n).as_bytes() {
            keys.push(*byte).expect("room");
        }
        for byte in b"\":0" {
            keys.push(*byte).expect("room");
        }
    }
    keys.push(b'}').expect("room");
    let mut longest = bytes::zeroed(1_048_577);
    longest.fill(b'x');
    for (target, byte) in longest.iter_mut().zip(b"{\"a\":\"") {
        *target = *byte;
    }
    longest[1_048_575] = b'"';
    longest[1_048_576] = b'}';
    [deepest.into_boxed(), keys.into_boxed(), longest]
}

pub const STOA: &[u8] = br#"{
  "core": { "hostname": "stoa" },
  "stoa": { "port": 7000 },
  "members": {
    "1": { "stoa": { "index": 1 } },
    "2": { "stoa": { "index": 2 } },
    "3": { "stoa": { "index": 3 } }
  }
}
"#;

pub fn declaration(input: &[u8]) -> Result<crate::declaration::Declaration, Error> {
    crate::declaration::declaration(decode(input)?, &SHIPPED)
}
pub const fn at(offset: u32, line: u32, column: u32) -> crate::Position {
    crate::Position { offset, line, column }
}
pub fn text(at: crate::Position, input: &[u8]) -> crate::declaration::Text {
    crate::declaration::Text { at, text: bytes::copy_of(input) }
}
