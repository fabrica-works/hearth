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
pub const ERROR: Token = Token::new(203);
pub mod bounds;
pub mod git;
pub const OID: &[u8] = b"0123456789abcdef0123456789abcdef01234567";
pub const DOCUMENT: &[u8] = b"{\"placement\":{\"kind\":\"metal\",\"firmware\":\"efi\",\"hardware\":\"7th-gen\"}}";

pub fn call() -> builder::vocabulary::Call {
    builder::vocabulary::Call::Render {
        resolve: false,
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
            protocol: Protocol::new(&limits, call(), form, roots(), b"/store", 7),
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
    pub fn bind_error(&mut self) {
        self.protocol.bind_error(ERROR);
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
        let pipes: Box<[Token]> = if input { Box::new([OUTPUT, INPUT, ERROR]) } else { Box::new([OUTPUT, ERROR]) };
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
        let Asked::Io(request) = request else { panic!("git fixture has no cache request") };
        match request {
            io::Request::Spawn { owner, spawn } => self.spawned(owner, spawn.pipes.len() == 3, exit),
            io::Request::Stream { stream, down } => match down {
                Down::Demand { read: Read::Nothing, room: 0 } | Down::Send(_) | Down::Finish => {}
                Down::Demand { read: Read::Nothing, room: _ } => {
                    self.up(io::Event::Stream { owner: stream, up: Up::Room });
                }
                Down::Demand { read, room: 0 } => {
                    let up = if stream == ERROR { Up::End } else { held.answer(read).expect("active read") };
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
    pub fn listing_error(&mut self, error: crate::git_machine::listing::Error) {
        let Some(crate::Mode::Listing { above, .. }) = &mut self.protocol.mode else { panic!("listing phase") };
        above.push(crate::git_machine::listing::Event::Failed(error));
    }
    pub fn run(&mut self, stdout: &[u8], exit: Exit) -> builder::vocabulary::Event {
        self.run_stderr(stdout, b"", exit)
    }
    pub fn run_stderr(&mut self, stdout: &[u8], stderr: &[u8], exit: Exit) -> builder::vocabulary::Event {
        let mut errors = Held::new(bytes::copy_of(stderr));
        let mut held = Held::new(bytes::copy_of(stdout));
        for _ in 0..stdout.len().saturating_mul(10).saturating_add(4096) {
            if let Some(event) = self.told() {
                return event;
            }
            match self.asked() {
                Some(Asked::Io(io::Request::Stream { stream: ERROR, down: Down::Demand { read, room: 0 } })) => {
                    if read != Read::Nothing {
                        let up = errors.answer(read).expect("stderr demand");
                        self.up(io::Event::Stream { owner: ERROR, up });
                    }
                }
                Some(request) => self.answer(request, &mut held, exit),
                None => return self.told().expect("protocol answer"),
            }
        }
        panic!("bounded child settles");
    }
}

pub fn spec() -> render::parts::Spec {
    render::parts::Spec {
        resolution: None,
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

pub const CACHE_OWNER: Token = Token::new(10);
pub const CACHE_FILE: Token = Token::new(300);
pub fn cache() -> crate::cache::Cache {
    crate::cache::Cache::new(Fd::new(4), &mut skein_lib::Rng::new(7))
}
pub fn synced_cache() -> crate::cache::Cache {
    use skein_io::file::Event;
    let mut cache = cache();
    drop(cache.take_request());
    cache.up(Event::Opened { owner: CACHE_OWNER, file: CACHE_FILE, len: 0 });
    cache.piece(bytes::copy_of(b"abc"));
    drop(cache.take_request());
    cache.up(Event::Written { owner: CACHE_OWNER });
    cache.finish();
    drop(cache.take_request());
    cache.up(Event::Synced { owner: CACHE_OWNER });
    drop(cache.take_request());
    cache.up(Event::Closed { owner: CACHE_OWNER });
    cache
}

pub fn opening_cache() -> (Layer, Token) {
    let mut layer = Layer::new();
    let mut blob = blob(b"abc");
    blob.kind = render::vocabulary::As::Cached;
    layer.read(Box::new([blob]));
    let mut held = Held::new(batch(b"abc"));
    for _ in 0_u32..64 {
        let request = layer.asked().expect("cached read starts");
        if let Asked::File(io::file::Request::Create { .. }) = request {
            let owner = layer.owner.expect("spawned child");
            return (layer, owner);
        }
        match request {
            Asked::Io(io::Request::Spawn { owner, .. }) => layer.spawn_pending(owner, true),
            other @ (Asked::Io(_) | Asked::File(_) | Asked::Beneath(_)) => {
                layer.answer(other, &mut held, Exit::Code(0));
            }
        }
    }
    panic!("cache create reached");
}
impl Layer {
    pub fn file_up(&mut self, event: io::file::Event) {
        up(&mut self.protocol, &self.env, Told::File(event), &mut self.above, &mut self.below);
    }
}

pub fn config_change() -> render::diff::Change {
    use render::{diff, parts::config, revision::Mode};
    diff::Change {
        target: bytes::copy_of(b"x1-prod"),
        change: diff::TargetChange::Changed {
            parts: Box::new([diff::PartChange::Config(config::Change {
                presets: Box::new([]),
                tmpfiles: Box::new([]),
                scripts: None,
                groups: Box::new([]),
                users: Box::new([]),
                members: Box::new([]),
                tree: Box::new([config::EntryChange::Changed {
                    path: bytes::copy_of(b"/etc/motd"),
                    before: config::file(b"etc/motd", Mode::File, hearth_lib::encode::Digest([1; 32])).node,
                    after: config::file(b"etc/motd", Mode::Executable, hearth_lib::encode::Digest([2; 32])).node,
                    sizes: Some(config::Sizes { before: 9, after: 22 }),
                    hunks: Some(diff::Hunks { lines: Box::new([]), left_out: 401, binary: false }),
                }]),
            })]),
            rebuild: diff::Rebuild::FromConfig,
        },
    }
}

pub fn roots() -> crate::Roots {
    crate::Roots {
        store: crate::Root { fd: Fd::new(3), token: Token::new(31) },
        cache: crate::Root { fd: Fd::new(4), token: Token::new(32) },
        resolutions: crate::Root { fd: Fd::new(5), token: Token::new(33) },
        staging: crate::Root { fd: Fd::new(6), token: Token::new(34) },
    }
}

pub const STORE_OWNER: Token = Token::new(20);
pub fn store_record() -> builder::store::vocabulary::StoreRecord {
    builder::store::vocabulary::StoreRecord { layout: 1, made: Wall::from_nanos(77), range: None, uid: 1000 }
}
impl Layer {
    pub fn startup(&mut self) -> Option<Box<[u8]>> {
        self.protocol.take_startup()
    }
}

pub fn store_cursor(layer: &mut Layer, directory: builder::store::vocabulary::Directory, max: u32) {
    layer.client(builder::vocabulary::Request::Store(builder::store::vocabulary::Request::List { directory, max }));
    drop(layer.asked());
    layer.file_up(io::file::Event::Opened { owner: STORE_OWNER, file: Token::new(88), len: 0 });
}

/// Every operation, with distinct payloads where it carries one.
#[must_use]
pub fn every_op() -> [hearth_protocol_template::Op; 26] {
    use hearth_protocol_template::{Filter, Missing, Op};
    [
        Op::String(bytes::copy_of(b"text")),
        Op::Integer(-17),
        Op::Boolean(true),
        Op::Name { name: bytes::copy_of(b"settings"), missing: Missing::Error },
        Op::Key { name: bytes::copy_of(b"wifi"), missing: Missing::Undefined },
        Op::Item { missing: Missing::Undefined },
        Op::List { length: 7 },
        Op::Filter(Filter::Default),
        Op::Not,
        Op::Equal,
        Op::NotEqual,
        Op::Less,
        Op::LessOrEqual,
        Op::Greater,
        Op::GreaterOrEqual,
        Op::In,
        Op::NotIn,
        Op::Concat,
        Op::Add,
        Op::Subtract,
        Op::Defined,
        Op::NotDefined,
        Op::JumpIfFalseKeeping { over: 11 },
        Op::JumpIfTrueKeeping { over: 13 },
        Op::JumpIfFalse { over: 17 },
        Op::Jump { over: 19 },
    ]
}

impl Layer {
    pub fn hunks(&mut self) {
        self.down(render::vocabulary::Request::Hunks {
            request: REQUEST,
            before: hearth_lib::encode::Digest([1; 32]),
            after: hearth_lib::encode::Digest([2; 32]),
        });
    }
}

#[must_use]
pub fn render_hunks() -> [render::diff::Hunks; 2] {
    use render::diff::{Hunks, Line, LineKind};
    [
        Hunks {
            lines: Box::new([
                Line { kind: LineKind::Hunk, text: bytes::copy_of(b"@@ -1 +1 @@") },
                Line { kind: LineKind::Context, text: bytes::copy_of(b"context") },
                Line { kind: LineKind::Removed, text: bytes::copy_of(b"old") },
                Line { kind: LineKind::Added, text: bytes::copy_of(b"new") },
                Line { kind: LineKind::NoNewline, text: bytes::copy_of(b" No newline at end of file") },
            ]),
            left_out: 401,
            binary: false,
        },
        Hunks { lines: Box::new([]), left_out: 0, binary: true },
    ]
}

#[must_use]
pub fn every_form() -> [crate::config::schema::Form; 13] {
    use crate::config::{
        Position,
        declaration::Text,
        schema::{Form, Spec},
    };
    let at = Position { offset: 0, line: 3, column: 7 };
    [
        Form::Bool,
        Form::Int,
        Form::String,
        Form::Path,
        Form::Mode,
        Form::Duration,
        Form::Secret,
        Form::Target,
        Form::File,
        Form::Enum(Box::new([Text { at, text: bytes::copy_of(b"main") }])),
        Form::List(17),
        Form::Map(23),
        Form::Object(Box::new([Spec {
            name: Text { at, text: bytes::copy_of(b"field") },
            at,
            ty: 29,
            default: None,
            optional: true,
            min: None,
            max: None,
            max_length: None,
        }])),
    ]
}
mod schemas;
pub use schemas::schema_faults;

mod declarations;
pub use declarations::*;

pub const PARAMS: &[u8] = b"{\"settings\": {\"params\": {\n  \"type\": {\"map\": {\"object\": {\n    \"param\": {\"type\": \"string\", \"doc\": \"One parameter.\"},\n    \"order\": {\"type\": \"int\", \"min\": 0, \"max\": 999, \"default\": 500, \"doc\": \"Its place.\"}}}},\n  \"default\": {}, \"doc\": \"Parameters, by name.\"}}}";
pub const READS: &[u8] = br#"{"reads":{"core":{"settings":["domain"],"optional":true,"exports":true}}}"#;

pub const ABC: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61,
    0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
]);
pub fn cache_record(origin: builder::store::vocabulary::Origin) -> builder::store::vocabulary::CacheFile {
    builder::store::vocabulary::CacheFile {
        at: Wall::from_nanos(123),
        origin,
        sha256: ABC,
        size: 3,
        took: skein_lib::Duration::from_millis(3),
    }
}
pub fn arriving() -> Layer {
    let mut layer = Layer::new();
    let mut first = blob(b"abc");
    first.kind = render::vocabulary::As::Cached;
    layer.read(Box::new([first.clone(), first]));
    let mut held = Held::new(batch(b"abc"));
    for _ in 0_u32..128 {
        if let Some(event) = layer.told() {
            let builder::vocabulary::Event::Arrived { last: false, .. } = event else { panic!("first arrival") };
            return layer;
        }
        let Some(request) = layer.asked() else {
            continue;
        };
        if let Asked::File(request) = request {
            use io::file::{Event, Request};
            let event = match request {
                Request::Create { owner, .. } => Event::Opened { owner, file: CACHE_FILE, len: 0 },
                Request::WriteAt { owner, .. } => Event::Written { owner },
                Request::Sync { owner, .. } => Event::Synced { owner },
                Request::Close { owner, .. } => Event::Closed { owner },
                Request::Lock { .. }
                | Request::MakeDirectory { .. }
                | Request::OpenPrivate { .. }
                | Request::CreateNoFollow { .. }
                | Request::OpenRead { .. }
                | Request::OpenReadNoFollow { .. }
                | Request::OpenWrite { .. }
                | Request::OpenWriteNoFollow { .. }
                | Request::OpenDirectory { .. }
                | Request::Load { .. }
                | Request::Scan { .. }
                | Request::Store { .. }
                | Request::Stat { .. }
                | Request::ReadAt { .. }
                | Request::Cut { .. }
                | Request::SyncDirectory { .. }
                | Request::Rename { .. }
                | Request::Remove { .. }
                | Request::List { .. }
                | Request::ListPage { .. } => panic!("only the temporary is written before arrival"),
            };
            layer.file_up(event);
        } else {
            layer.answer(request, &mut held, Exit::Code(0));
        }
    }
    panic!("arrival reached");
}
#[must_use]
pub fn cache_records() -> [builder::store::vocabulary::CacheFile; 4] {
    use builder::store::vocabulary::Origin;
    [
        cache_record(Origin::Fetch { uri: bytes::copy_of(b"https://mirror/a") }),
        cache_record(Origin::Index { resolution: ABC }),
        cache_record(Origin::Blob { oid: bytes::copy_of(OID) }),
        cache_record(Origin::Rendered { path: bytes::copy_of(b"tasks/core/tree/etc/motd.j2") }),
    ]
}

pub fn walk_entry(name: &[u8], kind: io::kernel::Kind) -> io::file::Entry {
    io::file::Entry { name: bytes::copy_of(name), kind }
}
pub fn walking(walk: limits::Walk) -> Layer {
    let mut layer = Layer::with_form(crate::Form::Text, limits::Limits { walk, ..limits::SHIPPED });
    layer.client(builder::vocabulary::Request::Store(builder::store::vocabulary::Request::Empty));
    drop(layer.asked());
    layer.file_up(io::file::Event::Opened { owner: STORE_OWNER, file: Token::new(88), len: 0 });
    drop(layer.asked());
    layer
}
#[must_use]
pub fn every_inputs() -> [crate::tree::resolution::Inputs; 2] {
    use hearth_lib::encode::Item;
    [
        resolution_inputs(),
        crate::tree::resolution::Inputs {
            sources: Box::new([Item::List(4), Item::None, Item::True, Item::False, Item::Uint(7)]),
            request: Box::new([Item::List(2), Item::Digest(ABC), Item::Bytes(b"x".as_slice().into())]),
            variant: Box::new([Item::Bytes(b"minbase".as_slice().into())]),
            apt: Box::new([Item::Map(1), Item::Bytes(b"a".as_slice().into()), Item::List(0)]),
        },
    ]
}

pub fn descending() -> Layer {
    let mut layer = walking(limits::SHIPPED.walk);
    layer.file_up(io::file::Event::Listed {
        owner: STORE_OWNER,
        entries: Box::new([walk_entry(b"root", io::kernel::Kind::Directory)]),
    });
    drop(layer.asked());
    layer.file_up(io::file::Event::Opened { owner: STORE_OWNER, file: Token::new(89), len: 0 });
    drop(layer.asked());
    layer
}
pub const CLOSE_CHILD: Asked = Asked::File(io::file::Request::Close { owner: STORE_OWNER, file: Token::new(89) });
pub const CLOSE_STAGING: Asked = Asked::File(io::file::Request::Close { owner: STORE_OWNER, file: Token::new(88) });
pub fn listing(path: &[u8], count: u32) -> Box<[u8]> {
    let header = b"100644 blob 0123456789abcdef0123456789abcdef01234567       2\t";
    let mut listing = skein_lib::Writer::new(
        header
            .len()
            .saturating_add(path.len())
            .saturating_add(1)
            .saturating_mul(usize::try_from(count).expect("fixture count")),
    );
    for _ in 0..count {
        listing.put(header).expect("record");
        listing.put(path).expect("path");
        listing.put(b"\0").expect("terminator");
    }
    listing.finish()
}

#[must_use]
pub fn every_does() -> [hearth_protocol_template::instruction::Does; 7] {
    use hearth_protocol_template::instruction::{Does, Names};
    let expression = hearth_protocol_template::Expression { operations: Box::new([]), values: 0 };
    [
        Does::Text(bytes::copy_of(b"text")),
        Does::Output(expression.clone()),
        Does::Branch { condition: expression.clone(), over: 2 },
        Does::Jump { over: 3 },
        Does::Enter {
            names: Names::Pair { first: bytes::copy_of(b"k"), second: bytes::copy_of(b"v") },
            iterable: expression.clone(),
            over: 4,
        },
        Does::Advance { back: 5 },
        Does::Set { name: bytes::copy_of(b"x"), value: expression },
    ]
}
#[must_use]
pub fn every_syntax() -> [hearth_protocol_template::error::ErrorKind; 19] {
    use hearth_protocol_template::error::{ErrorKind as E, Expected};
    [
        E::TooLong,
        E::Character,
        E::Unterminated,
        E::Escape,
        E::Integer,
        E::TooDeep,
        E::Misplaced,
        E::UnknownFilter,
        E::UnknownTest,
        E::Utf8,
        E::Return,
        E::Unclosed,
        E::UnknownStatement,
        E::Unmatched,
        E::Unended,
        E::Shadows,
        E::Interpolation,
        E::Expected(Expected::Operand),
        E::Arity { filter: hearth_protocol_template::Filter::Join, given: 2 },
    ]
}

#[must_use]
pub fn meta_changes() -> [render::parts::meta::Change; 2] {
    use crate::render::{
        parts::meta::{Change, MemberPair, NamePair, Placement, PlacementPair},
        revision::{Firmware, Runs},
    };
    [
        Change {
            base: Some(NamePair { before: b"x1".as_slice().into(), after: b"x2".as_slice().into() }),
            member: Some(MemberPair { before: None, after: Some(b"a".as_slice().into()) }),
            environment: Some(NamePair { before: b"lab".as_slice().into(), after: b"prod".as_slice().into() }),
            placement: Some(PlacementPair {
                before: Placement { kind: Runs::Metal, firmware: Some(Firmware::Efi), hardware: None, on: None },
                after: Placement {
                    kind: Runs::Container,
                    firmware: None,
                    hardware: None,
                    on: Some(b"x1-prod".as_slice().into()),
                },
            }),
        },
        Change {
            base: None,
            member: Some(MemberPair { before: Some(b"a".as_slice().into()), after: None }),
            environment: None,
            placement: None,
        },
    ]
}

#[must_use]
pub fn decode_error() -> render::errors::Error {
    use render::errors as render_errors;
    use render::revision as target;
    render_errors::Error {
        side: None,
        kind: render_errors::Kind::Decode(render_errors::Decode::Duplicate { key: bytes::copy_of(b"a") }),
        class: render::vocabulary::Class::Configuration,
        at: Some(render_errors::Place {
            path: bytes::copy_of(b"tasks/motd/task.json"),
            position: Some(target::Position { line: 1, column: 10 }),
        }),
        also: Some(render_errors::Place {
            path: bytes::copy_of(b"tasks/motd/task.json"),
            position: Some(target::Position { line: 1, column: 2 }),
        }),
        target: None,
        task: Some(bytes::copy_of(b"motd")),
    }
}

pub fn member_document() -> crate::config::document::Document {
    use crate::config::{
        Position,
        document::{Document, Layer, Member, Members},
    };
    Document {
        at: Position { offset: 0, line: 1, column: 1 },
        layer: Layer { tasks: Box::new([]), debian: None, placement: None },
        members: Some(Members {
            at: Position { offset: 11, line: 1, column: 12 },
            each: Box::new([
                Member {
                    name: bytes::copy_of(b"1"),
                    at: Position { offset: 12, line: 1, column: 13 },
                    layer: Some(Layer { tasks: Box::new([]), debian: None, placement: None }),
                },
                Member { name: bytes::copy_of(b"2"), at: Position { offset: 19, line: 1, column: 20 }, layer: None },
            ]),
        }),
    }
}

pub fn evaluation_back(kind: crate::records::errors::Evaluation) -> render::evaluate::FaultKind {
    use crate::records::errors::Evaluation;
    match kind {
        Evaluation::Unknown => render::evaluate::FaultKind::Unknown,
        Evaluation::NoKey => render::evaluate::FaultKind::NoKey,
        Evaluation::NoIndex => render::evaluate::FaultKind::NoIndex,
        Evaluation::Undefined => render::evaluate::FaultKind::Undefined,
        Evaluation::Type => render::evaluate::FaultKind::Type,
        Evaluation::Overflow => render::evaluate::FaultKind::Overflow,
        Evaluation::Empty => render::evaluate::FaultKind::Empty,
        Evaluation::Path => render::evaluate::FaultKind::Path,
        Evaluation::Output => render::evaluate::FaultKind::Output,
        Evaluation::Limit => render::evaluate::FaultKind::Limit,
    }
}

mod reserved;
pub use reserved::{composition_kinds, reserved_rules};
mod keep;
pub use keep::*;

mod debian;
pub use debian::*;

mod software;
pub use software::software_records;
mod live;
pub use live::{live_records, live_specs};
mod secrets;
pub use secrets::secret_records;

mod exports;
pub mod files;
mod packages;
pub use exports::*;
pub use packages::*;

mod resolution;
pub use resolution::*;

mod resolve;
pub use resolve::*;
mod accounts;
pub use accounts::*;

mod indices;
pub use indices::*;
pub mod data;
mod directives;
pub use directives::*;

mod diff_resolution;
pub use diff_resolution::*;
mod ids;
pub use ids::*;

mod collisions;
pub use collisions::*;
mod part_changes;
pub use part_changes::*;

mod lists;
pub use lists::only;
