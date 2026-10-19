//! The in-process client's call and replies (client/README.md, 3.1).
use crate::{build, render, store};
use alloc::boxed::Box;
use skein_lib::{Duration, ReplyTo, Token};
#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    Index {
        request: Token,
        arrival: Token,
        sha256: render::vocabulary::Digest,
        size: u64,
        took: Duration,
    },
    Produced {
        request: Token,
        path: Box<[u8]>,
        arrival: Token,
        sha256: render::vocabulary::Digest,
        size: u64,
        took: Duration,
    },
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
    Build(build::vocabulary::Event),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Call {
    Diff {
        resolve: bool,
        repository: Box<[u8]>,
        from: Box<[u8]>,
        to: Box<[u8]>,
        targets: Box<[Box<[u8]>]>,
    },
    Render {
        resolve: bool,
        repository: Box<[u8]>,
        revision: Box<[u8]>,
        targets: Box<[Box<[u8]>]>,
    },
    /// One target's output, rendered with its resolution (domain/build.md, 4.1).
    Build {
        repository: Box<[u8]>,
        revision: Box<[u8]>,
        target: Box<[u8]>,
    },
}
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    Progress(Progress),
    Render(render::vocabulary::Request),
    Store(store::vocabulary::Request),
    /// A build step, for the protocol layer to run (domain/build.md, 4.3).
    Run {
        step: build::Step,
    },
    Unopened {
        to: ReplyTo,
        why: store::vocabulary::Why,
    },
    Answer(Answer),
    Reply {
        to: ReplyTo,
        outcome: render::vocabulary::Outcome,
        diff: Option<render::diff::Tally>,
        left_out: u32,
    },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    Change(render::diff::Change),
    Spec(render::parts::Spec),
    Error(render::errors::Error),
    /// A build's layer: its key, and whether it was found (client/README.md, 3.1).
    Layer {
        layer: build::Layer,
        key: render::vocabulary::Digest,
        state: State,
    },
    /// The step that ended a build, and why.
    Failed {
        step: build::Step,
        why: build::vocabulary::Why,
    },
    /// The store at a limit render's errors do not name: its layers (domain/store.md, 8.1).
    Full {
        limit: store::vocabulary::Limit,
        most: u32,
    },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum State {
    Found,
    ToBuild,
}

/// Store notices go to standard error (client/README.md, 3.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Progress {
    Resolving { key: render::vocabulary::Digest },
    Resolved { key: render::vocabulary::Digest, took: Duration },
    Unresolved { key: render::vocabulary::Digest },
    Waiting,
    Strays { count: u64, first: store::vocabulary::Stray },
}
