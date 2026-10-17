//! The in-process client's call and replies (client/README.md, 3.1).
use crate::render;
use alloc::boxed::Box;
use skein_lib::ReplyTo;
#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    Call { reply_to: ReplyTo, call: Call },
    Render(render::vocabulary::Event),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Call {
    Render { repository: Box<[u8]>, revision: Box<[u8]>, targets: Box<[Box<[u8]>]> },
}
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    Render(render::vocabulary::Request),
    Answer(Answer),
    Reply { to: ReplyTo, outcome: render::vocabulary::Outcome },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    Spec(render::parts::Spec),
}
