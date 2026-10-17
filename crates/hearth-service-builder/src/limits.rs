//! The layers' bounds and the queues between them (programming-model.md, 7).
use crate::{domain, protocol};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub io: skein_io::Limits,
    pub files: Files,
    pub protocol: protocol::limits::Limits,
    pub domain: domain::Limits,
    pub queue: u32,
}

use skein_lib::Duration;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Files {
    pub files: u32,
    pub read: u32,
    pub entries: u32,
    pub whole: u32,
    pub timeout: Duration,
}
pub const SHIPPED: Limits = Limits {
    io: skein_io::Limits {
        sockets: 8,
        names: 0,
        refusals: 1,
        intake: 65_536,
        receive: 16_384,
        output: 65_536,
        sends: 8,
        accepts: 1,
        backlog: 1,
        close_timeout: Duration::from_secs(5),
        retry: Duration::from_millis(50),
    },
    files: Files { files: 32, read: 16_384, entries: 1_024, whole: 4_194_328, timeout: Duration::from_secs(5) },
    protocol: protocol::limits::SHIPPED,
    domain: domain::Limits { render: domain::render::limits::SHIPPED, store: domain::store::limits::SHIPPED },
    queue: 32,
};
