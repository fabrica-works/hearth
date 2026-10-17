use crate::{
    Asked, Protocol, Told,
    builder::{self, render},
    down, limits, resume, up,
};
use alloc::boxed::Box;
use skein_io::{
    self as io,
    kernel::{Exit, Fd},
};
use skein_lib::{
    Env, List, Queue, Time, Token, Wall, bytes,
    stream::{Down, Held, Read, Up},
};

pub const CLIENT: Token = Token::new(90);
pub const REQUEST: Token = Token::new(1);
pub const OUTPUT: Token = Token::new(201);
pub const INPUT: Token = Token::new(202);
pub const OID: &[u8] = b"0123456789abcdef0123456789abcdef01234567";
pub const DOCUMENT: &[u8] = b"{\"placement\":{\"kind\":\"metal\",\"firmware\":\"efi\",\"hardware\":\"7th-gen\"}}";

pub fn call() -> builder::vocabulary::Call {
    builder::vocabulary::Call::Render {
        repository: bytes::copy_of(b"/repo"),
        revision: bytes::copy_of(b"HEAD"),
        targets: Box::new([]),
    }
}
pub fn oid() -> render::revision::Oid {
    render::revision::Oid(bytes::copy_of(OID))
}
pub fn blob(body: &[u8]) -> render::vocabulary::Blob {
    blob_named(OID, body)
}
pub fn blob_named(oid: &[u8], body: &[u8]) -> render::vocabulary::Blob {
    render::vocabulary::Blob {
        oid: render::revision::Oid(bytes::copy_of(oid)),
        size: u64::try_from(body.len()).expect("small fixture"),
        kind: render::vocabulary::As::Document,
    }
}
pub fn batch(body: &[u8]) -> Box<[u8]> {
    let size = skein_lib::Decimal::of(u64::try_from(body.len()).expect("small fixture"));
    let mut bytes = skein_lib::Writer::new(
        OID.len().saturating_add(body.len()).saturating_add(size.as_bytes().len()).saturating_add(8),
    );
    bytes.put(OID).expect("room");
    bytes.put(b" blob ").expect("room");
    bytes.put(size.as_bytes()).expect("room");
    bytes.put(b"\n").expect("room");
    bytes.put(body).expect("room");
    bytes.put(b"\n").expect("room");
    bytes.finish()
}
#[derive(Debug)]
pub struct Layer {
    protocol: Protocol,
    env: Env<limits::Limits>,
    above: Queue<builder::vocabulary::Event>,
    below: Queue<Asked>,
    owner: Option<Token>,
}
impl Layer {
    pub fn new() -> Self {
        Self::with_form(crate::Form::Text, limits::SHIPPED)
    }
    pub fn with_form(form: crate::Form, limits: limits::Limits) -> Self {
        let mut layer = Self {
            protocol: Protocol::new(&limits, call(), form, Fd::new(3)),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            above: Queue::with_capacity(1),
            below: Queue::with_capacity(1),
            owner: None,
        };
        resume(&mut layer.protocol, &layer.env, &mut layer.above, &mut layer.below);
        drop(layer.above.pop());
        layer
    }
    pub fn client(&mut self, request: builder::vocabulary::Request) {
        down(&mut self.protocol, &self.env, request, &mut self.above, &mut self.below);
    }
    pub fn bind_output(&mut self) {
        self.protocol.bind_output(CLIENT);
    }
    pub fn code(&self) -> Option<u8> {
        self.protocol.code()
    }
    pub fn printed(&mut self) -> Box<[Down]> {
        let mut requests = List::with_capacity(16);
        for _ in 0_u32..64 {
            let Some(Asked::Io(request)) = self.asked() else {
                return requests.into_boxed();
            };
            match request {
                io::Request::Stream { stream, down } => {
                    let room = match &down {
                        Down::Demand { read: Read::Nothing, room } => *room > 0,
                        Down::Send(_) | Down::Finish => false,
                        Down::Demand { .. } => panic!("output reads nothing"),
                    };
                    requests.push(down).expect("bounded printed record");
                    if room {
                        self.up(io::Event::Stream { owner: stream, up: Up::Room });
                    }
                }
                io::Request::Close { entity } => self.up(io::Event::Closed { owner: entity }),
                io::Request::Spawn { .. }
                | io::Request::Listen { .. }
                | io::Request::Connect { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Output { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. }
                | io::Request::Abort { .. } => panic!("client output operations"),
            }
        }
        panic!("bounded print settles");
    }
    pub fn down(&mut self, request: render::vocabulary::Request) {
        down(
            &mut self.protocol,
            &self.env,
            builder::vocabulary::Request::Render(request),
            &mut self.above,
            &mut self.below,
        );
    }
    pub fn list(&mut self) {
        self.down(render::vocabulary::Request::List {
            request: REQUEST,
            repository: bytes::copy_of(b"/repo"),
            revision: bytes::copy_of(b"HEAD"),
        });
    }
    pub fn read(&mut self, blobs: Box<[render::vocabulary::Blob]>) {
        self.down(render::vocabulary::Request::Read {
            request: REQUEST,
            repository: bytes::copy_of(b"/repo"),
            commit: oid(),
            blobs,
        });
    }
    pub fn up(&mut self, event: io::Event) {
        up(&mut self.protocol, &self.env, Told::Io(event), &mut self.above, &mut self.below);
    }
    pub fn asked(&mut self) -> Option<Asked> {
        for _ in 0_u32..4096 {
            if let Some(request) = self.below.pop() {
                return Some(request);
            }
            if !self.above.is_empty() || !self.protocol.work_pending() {
                return None;
            }
            resume(&mut self.protocol, &self.env, &mut self.above, &mut self.below);
        }
        panic!("bounded layer settles");
    }
    pub fn told(&mut self) -> Option<builder::vocabulary::Event> {
        self.above.pop()
    }
    pub fn spawn_pending(&mut self, owner: Token, input: bool) {
        self.owner = Some(owner);
        let pipes: Box<[Token]> = if input { Box::new([OUTPUT, INPUT]) } else { Box::new([OUTPUT]) };
        self.up(io::Event::Spawned { owner, child: Token::new(200), pipes });
    }
    pub fn spawned(&mut self, owner: Token, input: bool, exit: Exit) {
        self.spawn_pending(owner, input);
        self.up(io::Event::Exited { owner, exit });
    }
    pub fn resolve(&mut self) -> Asked {
        self.list();
        let mut held = Held::new(bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567\n"));
        for _ in 0_u32..32 {
            let request = self.asked().expect("rev-parse work");
            if let Asked::Io(io::Request::Spawn { spawn, .. }) = &request
                && spawn.args.get(1).expect("command").as_ref() == b"ls-tree"
            {
                return request;
            }
            self.answer(request, &mut held, Exit::Code(0));
        }
        panic!("rev-parse settles");
    }
    pub fn answer(&mut self, request: Asked, held: &mut Held, exit: Exit) {
        let Asked::Io(request) = request;
        match request {
            io::Request::Spawn { owner, spawn } => self.spawned(owner, spawn.pipes.len() == 2, exit),
            io::Request::Stream { stream, down } => match down {
                Down::Demand { read: Read::Nothing, room: 0 } | Down::Send(_) | Down::Finish => {}
                Down::Demand { read: Read::Nothing, room: _ } => {
                    self.up(io::Event::Stream { owner: stream, up: Up::Room });
                }
                Down::Demand { read, room: 0 } => {
                    let up = held.answer(read).expect("active read");
                    self.up(io::Event::Stream { owner: stream, up });
                }
                Down::Demand { .. } => panic!("one stream direction"),
            },
            io::Request::Close { entity } => {
                let owner = if entity == Token::new(200) { self.owner.expect("spawn owner") } else { entity };
                self.up(io::Event::Closed { owner });
            }
            io::Request::Listen { .. }
            | io::Request::Connect { .. }
            | io::Request::Bind { .. }
            | io::Request::Reject { .. }
            | io::Request::Output { .. }
            | io::Request::Signal { .. }
            | io::Request::Usage { .. }
            | io::Request::Window { .. }
            | io::Request::Abort { .. } => panic!("git operations"),
        }
    }
    pub fn run(&mut self, stdout: &[u8], exit: Exit) -> builder::vocabulary::Event {
        let mut held = Held::new(bytes::copy_of(stdout));
        for _ in 0_u32..4096 {
            if let Some(event) = self.told() {
                return event;
            }
            match self.asked() {
                Some(request) => self.answer(request, &mut held, exit),
                None => return self.told().expect("protocol answer"),
            }
        }
        panic!("bounded child settles");
    }
}

pub fn spec() -> render::parts::Spec {
    render::parts::Spec {
        meta: render::parts::meta::Meta {
            target: bytes::copy_of(b"x1-prod"),
            base: bytes::copy_of(b"x1"),
            member: None,
            environment: bytes::copy_of(b"prod"),
            placement: render::parts::meta::Placement {
                kind: render::revision::Runs::Metal,
                firmware: Some(render::revision::Firmware::Efi),
                hardware: Some(bytes::copy_of(b"7th-gen")),
                on: None,
            },
        },
        meta_digest: hearth_lib::encode::Digest([1; 32]),
        commit: oid(),
        digest: hearth_lib::encode::Digest([2; 32]),
        parts: Box::new([]),
    }
}
