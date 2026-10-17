//! One mmdebstrap run, its pipe readers, deadlines and scratch reclamation (render.md, 5.2).
use crate::{Asked, Roots, Told, builder::vocabulary::Event, limits::Limits, render, translate, walk::Walker};
use alloc::boxed::Box;
use hearth_protocol_mmdebstrap::{
    output::{self, Reader, Seen},
    tail, vector,
};
use render::{
    errors::{Command, Ended, Limit, Tool},
    vocabulary::{Class, Event as RenderEvent, Failure, Request},
};
use skein_io::{
    self as io, file,
    kernel::{Exit, Pipe, Spawn, Way},
};
use skein_lib::{Env, Queue, Time, Token, Writer, bytes, stream::Up};
pub(super) const OWNER: Token = Token::new(20);
#[derive(Debug)]
struct PipeReader {
    token: Option<Token>,
    reader: Reader,
    ended: bool,
    closing: bool,
}
impl PipeReader {
    fn new(limits: &hearth_protocol_mmdebstrap::limits::Limits) -> Self {
        Self { token: None, reader: Reader::new(limits), ended: false, closing: false }
    }
}
#[derive(Debug)]
pub(super) struct Resolve {
    request: Token,
    name: Box<[u8]>,
    invocation: vector::Invocation,
    lines: Box<[vector::Line]>,
    output: PipeReader,
    error: PipeReader,
    process: Option<Token>,
    closing: bool,
    exit: Option<Exit>,
    terminal: Option<RenderEvent>,
    pending: Queue<Asked>,
    deadline: Time,
    idle: Time,
    started: Time,
    fetched: u64,
    count: u32,
    scratch: Scratch,
    walker: Option<Walker>,
}
impl Resolve {
    pub fn new(
        request: Request,
        roots: Roots,
        store: &[u8],
        proxy: Option<&vector::Proxy>,
        env: &Env<Limits>,
        below: &mut Queue<Asked>,
    ) -> Self {
        let Request::Resolve { request, key, sources, names, variant, apt, deadline } = request else {
            unreachable!("resolve request")
        };
        let mut name = Writer::new(64);
        for byte in key.0 {
            crate::cache::hex(&mut name, byte);
        }
        let name = name.finish();
        let mut scratch = Writer::new(store.len().saturating_add(73));
        for part in [store, b"/staging/", &name] {
            scratch.put(part).expect("scratch size");
        }
        let simulate = translate::resolution::simulate(&sources, &names, variant, &apt, &env.limits.mmdebstrap)
            .expect("validated render inputs");
        let lines = vector::lines(&simulate.sources).expect("validated sources");
        let invocation = vector::simulate(&simulate, &scratch.finish(), proxy, &env.limits.mmdebstrap)
            .expect("bounded scratch and request");
        below.push(Asked::File(file::Request::MakeDirectory {
            owner: OWNER,
            root: roots.staging.token,
            path: name.clone(),
            mode: 0o700,
        }));
        Self {
            request,
            name,
            invocation,
            lines,
            output: PipeReader::new(&env.limits.mmdebstrap),
            error: PipeReader::new(&env.limits.mmdebstrap),
            process: None,
            closing: false,
            exit: None,
            terminal: None,
            pending: Queue::with_capacity(4),
            deadline,
            idle: env.now.saturating_add(env.limits.mmdebstrap.idle),
            started: env.now,
            fetched: 0,
            count: 0,
            scratch: Scratch::Absent,
            walker: None,
        }
    }
    pub fn next_deadline(&self) -> Option<Time> {
        if self.terminal.is_some() { None } else { Some(self.deadline.min(self.idle)) }
    }
    pub fn fire(&mut self, env: &Env<Limits>) {
        if match self.next_deadline() {
            Some(at) => at <= env.now,
            None => false,
        } {
            self.fail(Ended::Timeout, None);
        }
    }
    fn fail(&mut self, ended: Ended, cause: Option<render::resolution::Cause>) {
        if self.terminal.is_some() {
            return;
        }
        let class = match &cause {
            Some(cause) => cause.class(),
            None => Class::Operational,
        };
        let failure = tail::failure(&self.error.reader, &self.output.reader);
        self.terminal = Some(RenderEvent::Failed {
            request: self.request,
            class,
            at: None,
            failure: Failure::Run { tool: Tool { command: Command::Mmdebstrap, ended, tail: failure.tail }, cause },
        });
    }
    pub fn up(
        &mut self,
        event: Told,
        roots: Roots,
        env: &Env<Limits>,
        above: &mut Queue<Event>,
        below: &mut Queue<Asked>,
    ) {
        match event {
            Told::File(event) => self.file_up(event, roots, env, below),
            Told::Io(event) => match event {
                io::Event::Spawned { child, pipes, .. } => {
                    self.process = Some(child);
                    let mut pipes = pipes.into_iter();
                    self.output.token = Some(pipes.next().expect("stdout"));
                    self.error.token = Some(pipes.next().expect("stderr"));
                    for pipe in [&mut self.output, &mut self.error] {
                        let mut requests = Queue::with_capacity(1);
                        output::open(
                            &mut pipe.reader,
                            &Env { now: env.now, wall: env.wall, limits: env.limits.mmdebstrap },
                            &mut requests,
                        );
                        if let Some(down) = requests.pop() {
                            self.pending
                                .push(Asked::Io(io::Request::Stream { stream: pipe.token.expect("pipe"), down }));
                        }
                    }
                }
                io::Event::Stream { owner, up } => self.stream(owner, up, env, above),
                io::Event::Exited { exit, .. } => self.exit = Some(exit),
                io::Event::Failed { .. } => self.fail(Ended::Start, None),
                io::Event::Closed { owner } => {
                    if owner == OWNER {
                        self.process = None;
                    }
                    for pipe in [&mut self.output, &mut self.error] {
                        if pipe.token == Some(owner) {
                            pipe.token = None;
                        }
                    }
                }
                io::Event::Listening { .. }
                | io::Event::Accepted { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Output { .. }
                | io::Event::Usage { .. }
                | io::Event::Resized
                | io::Event::Window { .. }
                | io::Event::Shutdown { .. } => unreachable!("resolve child terminal"),
            },
        }
    }
    fn file_up(&mut self, event: file::Event, roots: Roots, env: &Env<Limits>, below: &mut Queue<Asked>) {
        if let Some(walker) = &mut self.walker {
            if walker.up(event, env.limits.walk, below).is_some() {
                self.walker = None;
                self.scratch = Scratch::Removing;
                below.push(Asked::File(file::Request::Remove {
                    owner: OWNER,
                    root: roots.staging.fd,
                    name: self.name.clone(),
                    directory: true,
                }));
            }
            return;
        }
        match event {
            file::Event::Made { .. } => {
                self.scratch = Scratch::Made;
                let mut name = Writer::new(77);
                name.put(&self.name).expect("key");
                name.put(b"/sources.list").expect("suffix");
                below.push(Asked::File(file::Request::Store {
                    owner: OWNER,
                    root: roots.staging.token,
                    path: name.finish(),
                    no_follow: true,
                    bytes: self.invocation.sources_list.clone(),
                    expected: file::Expect::Absent,
                }));
            }
            file::Event::Stored { .. } => below.push(Asked::Io(io::Request::Spawn {
                owner: OWNER,
                spawn: Spawn {
                    program: bytes::copy_of(vector::PROGRAM),
                    args: self.invocation.args.clone(),
                    env: self.invocation.env.clone(),
                    root: roots.store.fd,
                    dir: Box::new([]),
                    pipes: Box::new([
                        Pipe { child: 1, way: Way::Out, parent: None },
                        Pipe { child: 2, way: Way::Out, parent: None },
                    ]),
                },
            })),
            file::Event::Removed { .. } => self.scratch = Scratch::Cleaned,
            file::Event::Refused { .. }
            | file::Event::Conflict { .. }
            | file::Event::Failed { .. }
            | file::Event::Cancelled { .. } => {
                if self.terminal.is_some() {
                    self.scratch = Scratch::Cleaned;
                } else {
                    self.fail(Ended::Start, None);
                }
            }
            file::Event::Locked { .. }
            | file::Event::Busy { .. }
            | file::Event::Opened { .. }
            | file::Event::Loaded { .. }
            | file::Event::Scanned { .. }
            | file::Event::TooLarge { .. }
            | file::Event::Stated { .. }
            | file::Event::Written { .. }
            | file::Event::Read { .. }
            | file::Event::Cut { .. }
            | file::Event::Synced { .. }
            | file::Event::Closed { .. }
            | file::Event::Renamed { .. }
            | file::Event::Listed { .. } => unreachable!("resolve scratch terminal"),
        }
    }
    fn stream(&mut self, owner: Token, up: Up, env: &Env<Limits>, above: &mut Queue<Event>) {
        if self.terminal.is_some() {
            return;
        }
        if let Up::Bytes(_) = &up {
            self.idle = env.now.saturating_add(env.limits.mmdebstrap.idle);
        }
        let pipe = if self.output.token == Some(owner) { &mut self.output } else { &mut self.error };
        assert_eq!(pipe.token, Some(owner), "current resolve pipe");
        let mut seen = Queue::with_capacity(1);
        let mut requests = Queue::with_capacity(1);
        output::up(
            &mut pipe.reader,
            &Env { now: env.now, wall: env.wall, limits: env.limits.mmdebstrap },
            up,
            &mut seen,
            &mut requests,
        );
        if let Some(down) = requests.pop() {
            self.pending.push(Asked::Io(io::Request::Stream { stream: owner, down }));
        }
        if let Some(event) = seen.pop() {
            match event {
                Seen::Inst(inst) => {
                    self.count = self.count.saturating_add(1);
                    if self.count > env.limits.mmdebstrap.insts {
                        self.too_many(env);
                    } else {
                        above.push(Event::Render(RenderEvent::Inst {
                            request: self.request,
                            inst: translate::resolution::inst(inst),
                        }));
                    }
                }
                Seen::Fetched(fetched) => self.fetched = self.fetched.saturating_add(fetched),
                Seen::Ended => pipe.ended = true,
                Seen::Failed(output::OutputError::TooMany) => self.too_many(env),
                Seen::Failed(output::OutputError::Inst | output::OutputError::Stream(_)) => {
                    self.fail(Ended::Output, None);
                }
            }
        }
    }
    fn too_many(&mut self, env: &Env<Limits>) {
        self.terminal = Some(RenderEvent::Failed {
            request: self.request,
            class: Class::Configuration,
            at: None,
            failure: Failure::TooLarge { limit: Limit::Closure, max: u64::from(env.limits.mmdebstrap.insts) },
        });
    }
    pub fn ready(&self) -> bool {
        !self.pending.is_empty()
            || (self.exit.is_some() && self.output.ended && self.error.ended && self.terminal.is_none())
            || (self.terminal.is_some()
                && ((!self.closing && self.process.is_some())
                    || (self.output.token.is_some() && !self.output.closing)
                    || (self.error.token.is_some() && !self.error.closing)
                    || (self.process.is_none()
                        && self.output.token.is_none()
                        && self.error.token.is_none()
                        && self.walker.is_none()
                        && self.scratch != Scratch::Removing)))
    }
    pub fn resume(
        &mut self,
        roots: Roots,
        env: &Env<Limits>,
        above: &mut Queue<Event>,
        below: &mut Queue<Asked>,
    ) -> bool {
        if let Some(request) = self.pending.pop() {
            below.push(request);
            return false;
        }
        if self.terminal.is_none()
            && self.output.ended
            && self.error.ended
            && let Some(exit) = self.exit
        {
            match exit {
                Exit::Code(0) => {
                    self.terminal = Some(RenderEvent::Resolved {
                        request: self.request,
                        lines: Box::new([]),
                        tools: render::resolution::Tools { apt: Box::new([]), mmdebstrap: Box::new([]) },
                        costs: render::resolution::Costs {
                            fetched: self.fetched,
                            took: env.now.saturating_since(self.started),
                        },
                    });
                }
                Exit::Code(code) => {
                    let failure = tail::failure(&self.error.reader, &self.output.reader);
                    let cause = translate::resolution::cause(failure.cause, &self.lines, &failure.tail);
                    self.fail(Ended::Exit(code), Some(cause));
                }
                Exit::Signal(signal) => self.fail(Ended::Signal(signal), None),
            }
        }
        if self.terminal.is_none() {
            return false;
        }
        for pipe in [&mut self.output, &mut self.error] {
            if let Some(entity) = pipe.token
                && !pipe.closing
            {
                pipe.closing = true;
                below.push(Asked::Io(io::Request::Close { entity }));
                return false;
            }
        }
        if let Some(entity) = self.process {
            if !self.closing {
                self.closing = true;
                below.push(Asked::Io(io::Request::Close { entity }));
            }
            return false;
        }
        if self.output.token.is_some() || self.error.token.is_some() {
            return false;
        }
        if self.scratch == Scratch::Removing {
            return false;
        }
        if self.scratch == Scratch::Made {
            if self.walker.is_none() {
                self.walker = Some(Walker::at(env.limits.walk, roots.staging.fd, &self.name, OWNER, below));
            }
            return false;
        }
        above.push(Event::Render(self.terminal.take().expect("resolved terminal")));
        true
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Scratch {
    Absent,
    Made,
    Removing,
    Cleaned,
}
impl crate::Protocol {
    pub(super) fn resolve(&mut self, request: Request, env: &Env<Limits>, below: &mut Queue<Asked>) {
        self.resolving = Some(Resolve::new(request, self.roots, &self.store_path, self.proxy.as_ref(), env, below));
    }
    pub(super) fn resume_resolve(
        &mut self,
        env: &Env<Limits>,
        above: &mut Queue<Event>,
        below: &mut Queue<Asked>,
    ) -> bool {
        let Some(resolve) = &mut self.resolving else { return false };
        if resolve.resume(self.roots, env, above, below) {
            self.resolving = None;
        }
        true
    }
}
