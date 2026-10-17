//! The in-process client's call and replies (client/README.md, 3.1).
use crate::{render, store};
use alloc::boxed::Box;
use skein_lib::{Duration, ReplyTo, Token};
#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    Arrived {
        request: Token,
        oid: render::revision::Oid,
        size: u64,
        last: bool,
        arrival: Token,
        sha256: render::vocabulary::Digest,
        took: Duration,
    },
    Call {
        reply_to: ReplyTo,
        call: Call,
    },
    Render(render::vocabulary::Event),
    Store(store::vocabulary::Event),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Call {
    Diff { repository: Box<[u8]>, from: Box<[u8]>, to: Box<[u8]>, targets: Box<[Box<[u8]>]> },
    Render { repository: Box<[u8]>, revision: Box<[u8]>, targets: Box<[Box<[u8]>]> },
}
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    Render(render::vocabulary::Request),
    Store(store::vocabulary::Request),
    Unopened { to: ReplyTo, why: store::vocabulary::Why },
    Answer(Answer),
    Reply { to: ReplyTo, outcome: render::vocabulary::Outcome, diff: Option<render::diff::Tally>, left_out: u32 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    Change(render::diff::Change),
    Spec(render::parts::Spec),
    Error(render::errors::Error),
}
