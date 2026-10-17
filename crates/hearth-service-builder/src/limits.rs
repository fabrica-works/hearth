//! The layers' bounds and the queues between them (programming-model.md, 7).
use crate::{domain, protocol};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub io: skein_io::Limits,
    pub protocol: protocol::limits::Limits,
    pub domain: domain::Limits,
    pub queue: u32,
}

use skein_lib::Duration;
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
    protocol: protocol::limits::SHIPPED,
    domain: domain::Limits { render: domain::render::limits::SHIPPED },
    queue: 32,
};
