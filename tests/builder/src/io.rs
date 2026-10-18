//! The protocol world's io face over process and file operations.
mod child;
mod stream;
use crate::{
    files::Files,
    fixtures::{self, Fault, Scenario, Shared},
};
use child::Child;
use hearth_fake_machine::linux::{self, At, Handle, How, Op, Pid, Reply};
use hearth_protocol_builder::{Asked, Beneath, Told};
use skein_io::{self as records, file, kernel::Fd};
use skein_lib::{Rng, Time, Token};
use std::collections::{BTreeMap, VecDeque};
use stream::Pipe;

#[derive(Debug)]
pub struct Below {
    pub files: Files,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub fault_fell: bool,
    pub blocked_writes: u64,
    pub reaped: u32,
    pub spawned: u32,
    pub ended_pipes: u32,
    pub opened_pipes: u32,
    pub lock_times: Vec<Time>,
    pub vectors: Vec<Box<[Box<[u8]>]>>,
    pid: Pid,
    root: Handle,
    children: BTreeMap<Token, Child>,
    pipes: BTreeMap<Token, Pipe>,
    events: VecDeque<Told>,
    next: u64,
    rng: Rng,
    fault: Option<Fault>,
    held: Option<(Pid, Handle, Time)>,
}
impl Below {
    #[must_use]
    pub fn new(scenario: &Scenario<'_>, seed: u64, pipe: u32) -> Self {
        let shared = Shared::for_scenario(scenario, seed, pipe);
        fixtures::serve(&shared);
        let files = Files::new(shared.clone(), scenario.fault, seed);
        shared.drain_ledger();
        let pid = shared
            .borrow()
            .ledger
            .iter()
            .find(|entry| entry.ids == fixtures::BUILDER)
            .expect("roots opened as the builder")
            .pid;
        let root = {
            let mut fake = shared.borrow_mut();
            let Reply::Opened(root) = fake
                .behind
                .machine_mut()
                .call(pid, Op::Open { at: At::Root, path: fixtures::STORE.into(), how: How::Directory })
                .expect("store directory")
            else {
                unreachable!("Open opens")
            };
            root
        };
        shared.drain_ledger();
        Self {
            files,
            stdout: Vec::new(),
            stderr: Vec::new(),
            fault_fell: false,
            blocked_writes: 0,
            reaped: 0,
            spawned: 0,
            ended_pipes: 0,
            opened_pipes: 0,
            lock_times: Vec::new(),
            vectors: Vec::new(),
            pid,
            root,
            children: BTreeMap::new(),
            pipes: BTreeMap::new(),
            events: VecDeque::new(),
            next: fixtures::PROTOCOL_ENTITY,
            rng: Rng::new(seed),
            fault: scenario.fault,
            held: None,
        }
    }
    fn call(&self, op: Op) -> Result<Reply, linux::Refusal> {
        let result = self.files.shared.borrow_mut().behind.machine_mut().call(self.pid, op);
        self.files.shared.drain_ledger();
        result
    }
    fn name(&mut self) -> Token {
        self.next += 1;
        Token::new(self.next)
    }
    pub fn answer(&mut self, asked: Asked, now: Time) {
        match asked {
            Asked::Io(request) => self.io(request),
            Asked::File(request) => {
                if matches!(request, file::Request::Lock { .. }) {
                    self.lock_times.push(now);
                }
                if let file::Request::Rename { to, .. } = &request
                    && let Some(Fault::CacheFileGone { sha256 }) = self.fault
                    && to.as_ref() == fixtures::hex_digest(sha256.0).as_bytes()
                {
                    self.fault_fell = true;
                }
                let event = self.files.answer(request);
                self.events.push_back(Told::File(event));
            }
            Asked::Beneath(request) => self.answer(Asked::File(beneath(request)), now),
        }
    }
    fn io(&mut self, request: records::Request) {
        match request {
            records::Request::Spawn { owner, spawn } => self.spawn(owner, spawn),
            records::Request::Stream { stream, down } => self.stream(stream, down),
            records::Request::Close { entity } | records::Request::Abort { entity } => self.close(entity),
            records::Request::Signal { child, signal, to } => {
                let child = self.children.get(&child).expect("issued child").pid;
                let signal = match signal {
                    records::kernel::Signal::Kill => linux::Signal::Kill,
                    records::kernel::Signal::Terminate => linux::Signal::Terminate,
                };
                let to = match to {
                    records::kernel::Target::Child => linux::Target::Child,
                    records::kernel::Target::Group => linux::Target::Group,
                };
                let _ = self.call(Op::Signal { child, signal, to });
            }
            records::Request::Listen { .. }
            | records::Request::Connect { .. }
            | records::Request::Bind { .. }
            | records::Request::Reject { .. }
            | records::Request::Output { .. }
            | records::Request::Usage { .. }
            | records::Request::Window { .. } => unreachable!("hearth-28: builder io uses children and streams"),
        }
    }
    pub fn step(&mut self, now: Time) {
        if let Some((pid, file, at)) = self.held
            && now >= at
        {
            let mut fake = self.files.shared.borrow_mut();
            let machine = fake.behind.machine_mut();
            machine.call(pid, Op::Close { handle: file }).expect("release contender");
            machine.call(pid, Op::Exit { exit: linux::Exit::Code(0) }).expect("contender finishes");
            drop(fake);
            self.files.shared.drain_ledger();
            self.held = None;
        }
        self.step_children();
        self.step_pipes();
    }
    pub fn pop(&mut self) -> Option<Told> {
        self.events.pop_front()
    }
    #[must_use]
    pub fn active(&self) -> bool {
        !self.children.is_empty() || !self.pipes.is_empty() || !self.events.is_empty()
    }
    pub fn hold_lock_until(&mut self, at: Time) {
        let mut fake = self.files.shared.borrow_mut();
        let machine = fake.behind.machine_mut();
        let pid = machine.linux_mut().login(fixtures::BUILDER, b"/");
        let Reply::Opened(file) = machine
            .call(pid, Op::Open { at: At::Root, path: fixtures::STORE.into(), how: How::Directory })
            .expect("contender root")
        else {
            unreachable!("Open opens")
        };
        machine.call(pid, Op::Lock { file }).expect("contender holds lock");
        drop(fake);
        self.files.shared.drain_ledger();
        self.held = Some((pid, file, at));
    }
    pub fn settle(&mut self) {
        assert_eq!(self.files.open_files(), fixtures::WORLD_ROOTS, "only the startup roots remain open");
        assert!(!self.active(), "every child closed, pipe closed and event delivered");
        assert_eq!(self.spawned, self.reaped, "each child reaped");
        assert_eq!(self.opened_pipes, self.ended_pipes, "each readable pipe reached its end");
        assert_eq!(crate::referee::strays(&self.files), 0, "no staging or temporary entry");
        assert_eq!(self.call(Op::Close { handle: self.root }), Ok(Reply::Done), "release spawn root");
    }
    pub fn finish(self) {
        let Self { files, pid, .. } = self;
        let shared = files.shared.clone();
        drop(files);
        shared
            .borrow_mut()
            .behind
            .machine_mut()
            .call(pid, Op::Exit { exit: linux::Exit::Code(0) })
            .expect("settled builder exits");
        shared.drain_ledger();
    }
}
fn descriptor(token: Token) -> Fd {
    Fd::new(i32::try_from(token.raw()).expect("synthetic file descriptor"))
}
fn beneath(request: Beneath) -> file::Request {
    match request {
        Beneath::OpenDirectory { owner, root, name, no_follow } => {
            file::Request::OpenDirectory { owner, root: descriptor(root), name, no_follow }
        }
        Beneath::Rename { owner, from_root, from, to_root, to } => {
            file::Request::Rename { owner, from_root: descriptor(from_root), from, to_root, to }
        }
        Beneath::Remove { owner, root, name, directory } => {
            file::Request::Remove { owner, root: descriptor(root), name, directory }
        }
    }
}
