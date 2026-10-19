//! Hosted tools use the same kernel seam as the shipped client (domain/testing.md, 2.6).
use crate::fixtures::{self, Fault, Shared};
use hearth_fake_git::face::{Read, Want};
use skein_io::kernel::{self as k, Complete, Done, Exit, Fd, Op, OpenHow, Submit};
use skein_lib::{Duration, Queue, Rng, Time, Token, Wall};
use skein_shell::Host;
use skein_world::Inherited;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
pub mod git;
pub mod mmdebstrap;

#[derive(Debug)]
pub(super) enum Tool {
    Git(git::Git),
    Mmdebstrap(mmdebstrap::Mmdebstrap),
}
#[derive(Debug)]
pub(super) enum Job {
    Verify(Box<[u8]>),
    Directory(Box<[u8]>),
    Read(Box<[u8]>),
    Make(Box<[u8]>),
    Write(Box<[u8]>, Box<[u8]>),
    Input,
    Pipe(Fd, Box<[u8]>, u32),
    Close(Fd),
}
#[derive(Debug)]
enum Pending {
    Verify,
    VerifyStat(Fd),
    Directory(Box<[u8]>),
    ReadOpen(Box<[u8]>),
    ReadStat(Box<[u8]>, Fd),
    Read(Box<[u8]>, Fd, Vec<u8>, bool),
    MakeParent(Box<[u8]>),
    Make(Fd),
    WriteOpen(Box<[u8]>),
    Write(Fd),
    Input,
    Pipe(Fd),
    Close(Fd),
}
#[derive(Debug)]
pub struct Hosted {
    tool: Tool,
    shared: Shared,
    root: Fd,
    pipes: BTreeMap<u32, Fd>,
    owned: BTreeSet<Fd>,
    files: BTreeMap<Box<[u8]>, Read>,
    directories: BTreeSet<Box<[u8]>>,
    jobs: VecDeque<Job>,
    pending: Option<Pending>,
    prepared: bool,
    ready: Option<Time>,
    delay: Duration,
    per_mib: Duration,
    work: Duration,
    exit: Option<u8>,
    finished: bool,
    serial: u64,
    sent: u64,
    stall_after: Option<u64>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Hosted {
    pub(super) fn new(
        tool: Tool,
        shared: Shared,
        inherited: &Inherited,
        mut jobs: VecDeque<Job>,
        delay: Duration,
        fault: Option<Fault>,
    ) -> Self {
        let root = named(&inherited.roots, b"root");
        let pipes: BTreeMap<_, _> = inherited.pipes.iter().copied().collect();
        let mut owned: BTreeSet<_> = pipes.values().copied().collect();
        owned.extend(inherited.roots.iter().map(|(_, fd)| *fd));
        owned.insert(inherited.signal);
        jobs.push_front(Job::Close(inherited.signal));
        Self {
            tool,
            shared,
            root,
            pipes,
            owned,
            files: BTreeMap::new(),
            directories: BTreeSet::new(),
            jobs,
            pending: None,
            prepared: false,
            ready: None,
            delay,
            per_mib: Duration::ZERO,
            work: Duration::ZERO,
            exit: None,
            finished: false,
            serial: 0,
            sent: 0,
            stall_after: match fault {
                Some(Fault::Stalled { after }) => Some(after),
                _ => None,
            },
            completions: Queue::with_capacity(fixtures::HOST_OPERATIONS),
            submissions: Queue::with_capacity(fixtures::HOST_OPERATIONS),
        }
    }
    #[must_use]
    pub fn work_bounds(&self, pace: crate::pace::Pace) -> (Duration, Duration) {
        let span = match &self.tool {
            Tool::Git(_) => pace.git,
            Tool::Mmdebstrap(_) => pace.mmdebstrap,
        };
        (span.min.saturating_add(self.work), span.max.saturating_add(self.work))
    }
    fn emit(&mut self, kind: Op, pending: Pending) {
        self.serial += 1;
        self.pending = Some(pending);
        self.submissions.push(Submit { op: Token::new(self.serial), kind });
    }
    fn open(&mut self, path: Box<[u8]>, how: OpenHow, pending: Pending) {
        let absolute = path.starts_with(b"/");
        let path = path.into_vec().into_iter().skip(usize::from(absolute)).collect();
        self.emit(Op::Open { root: self.root, path, how }, pending);
    }
    fn next(&mut self, job: Job) {
        match job {
            Job::Verify(path) => self.open(path, OpenHow::ReadNoFollow, Pending::Verify),
            Job::Directory(path) => self.open(path.clone(), OpenHow::Directory, Pending::Directory(path)),
            Job::Read(path) => self.open(path.clone(), OpenHow::ReadNoFollow, Pending::ReadOpen(path)),
            Job::Make(path) => {
                let parent = path.rsplitn(2, |byte| *byte == b'/').nth(1).expect("absolute directory");
                self.open(parent.into(), OpenHow::Directory, Pending::MakeParent(path));
            }
            Job::Write(path, bytes) => self.open(path, OpenHow::Create { mode: None }, Pending::WriteOpen(bytes)),
            Job::Input => self.emit(
                Op::PipeRead {
                    fd: *self.pipes.get(&0).expect("batch stdin"),
                    buf: vec![0; fixtures::HOST_BUFFER].into(),
                },
                Pending::Input,
            ),
            Job::Pipe(fd, bytes, from) => self.emit(Op::PipeWrite { fd, bytes, from }, Pending::Pipe(fd)),
            Job::Close(fd) => self.emit(Op::Close { fd }, Pending::Close(fd)),
        }
    }
    fn complete(&mut self, complete: Complete) {
        let pending = self.pending.take().expect("one hosted operation");
        match (pending, complete.kind, complete.result) {
            (Pending::Verify, _, Ok(Done::Fd(fd))) => {
                self.owned.insert(fd);
                self.emit(Op::Stat { fd }, Pending::VerifyStat(fd));
            }
            (Pending::VerifyStat(fd), _, Ok(Done::Stat(stat))) => {
                assert_eq!(stat.kind, k::Kind::File, "the hosted program is installed");
                assert_ne!(stat.mode & 0o111, 0, "the hosted program is executable");
                self.jobs.push_front(Job::Close(fd));
            }
            (Pending::Directory(path), _, Ok(Done::Fd(fd))) => {
                self.owned.insert(fd);
                self.directories.insert(path);
                self.jobs.push_front(Job::Close(fd));
            }
            (Pending::Directory(_), _, Err(k::Error::NotFound | k::Error::Permission)) => {}
            (Pending::ReadOpen(path), _, Ok(Done::Fd(fd))) => {
                self.owned.insert(fd);
                self.emit(Op::Stat { fd }, Pending::ReadStat(path, fd));
            }
            (Pending::ReadOpen(path), _, Err(error)) => {
                self.files.insert(path, if error == k::Error::NotFound { Read::Missing } else { Read::Denied });
            }
            (Pending::ReadStat(path, fd), _, Ok(Done::Stat(stat))) => {
                self.read(path, fd, Vec::new(), stat.mode & 0o111 != 0);
            }
            (Pending::Read(path, fd, mut bytes, executable), Op::Read { buf, .. }, Ok(Done::Count(count))) => {
                if count == 0 {
                    self.files.insert(path, Read::Contents { bytes: bytes.into(), executable });
                    self.jobs.push_front(Job::Close(fd));
                } else {
                    bytes.extend_from_slice(&buf[..usize::try_from(count).expect("buffer count")]);
                    self.read(path, fd, bytes, executable);
                }
            }
            (Pending::MakeParent(path), _, Ok(Done::Fd(fd))) => {
                self.owned.insert(fd);
                let name = path.rsplit(|byte| *byte == b'/').next().expect("basename");
                self.emit(Op::MakeDirectory { dir: fd, name: name.into(), mode: 0o755 }, Pending::Make(fd));
            }
            (Pending::Make(fd), _, Ok(Done::Nothing)) => self.jobs.push_front(Job::Close(fd)),
            (Pending::WriteOpen(bytes), _, Ok(Done::Fd(fd))) => {
                self.owned.insert(fd);
                if bytes.is_empty() {
                    self.jobs.push_front(Job::Close(fd));
                } else {
                    self.emit(Op::Write { fd, bytes, from: 0, at: 0 }, Pending::Write(fd));
                }
            }
            (Pending::Write(fd), Op::Write { bytes, from, .. }, Ok(Done::Count(count))) => {
                let from = from + count;
                if usize::try_from(from).expect("written count") == bytes.len() {
                    self.jobs.push_front(Job::Close(fd));
                } else {
                    self.emit(Op::Write { fd, bytes, from, at: u64::from(from) }, Pending::Write(fd));
                }
            }
            (Pending::Input, Op::PipeRead { buf, .. }, Ok(Done::Count(count))) => {
                let Tool::Git(git) = &mut self.tool else { panic!("only git reads stdin") };
                git.input(&buf[..usize::try_from(count).expect("input count")]);
            }
            (Pending::Pipe(fd), Op::PipeWrite { bytes, from, .. }, Ok(Done::Count(count))) => {
                self.sent += u64::from(count);
                let from = from + count;
                if usize::try_from(from).expect("sent count") != bytes.len() {
                    self.jobs.push_front(Job::Pipe(fd, bytes, from));
                }
            }
            (Pending::Pipe(_), _, Err(k::Error::BrokenPipe)) => {
                self.jobs.clear();
                self.exit = Some(128 + 13);
            }
            (Pending::Close(fd), _, Ok(Done::Nothing)) => {
                self.owned.remove(&fd);
            }
            (pending, kind, result) => panic!("hosted kernel operation: {pending:?}, {kind:?}, {result:?}"),
        }
    }
    fn read(&mut self, path: Box<[u8]>, fd: Fd, bytes: Vec<u8>, executable: bool) {
        self.emit(
            Op::Read {
                fd,
                buf: vec![0; fixtures::HOST_BUFFER].into(),
                at: u64::try_from(bytes.len()).expect("file length"),
            },
            Pending::Read(path, fd, bytes, executable),
        );
    }
    fn prepare(&mut self) {
        match &mut self.tool {
            Tool::Git(git) => git.prepare(&mut self.files, &self.directories),
            Tool::Mmdebstrap(mmdebstrap) => {
                let cost = mmdebstrap.prepare(&self.shared, &mut self.files, &mut self.jobs);
                self.work = self.work.saturating_add(cost);
                self.ready = self.ready.map(|ready| ready.saturating_add(cost));
            }
        }
        self.prepared = true;
    }
    fn want(&mut self) {
        let want = match &mut self.tool {
            Tool::Git(git) => git.next(&self.shared),
            Tool::Mmdebstrap(mmdebstrap) => mmdebstrap.next(),
        };
        match want {
            Want::Input => self.jobs.push_back(Job::Input),
            Want::Stdout(bytes) => self.output(1, bytes),
            Want::Stderr(bytes) => self.output(2, bytes),
            Want::Exit(code) => self.exit = Some(code),
        }
    }
    fn output(&mut self, descriptor: u32, mut bytes: Box<[u8]>) {
        if self.per_mib != Duration::ZERO {
            let cost = Duration::from_nanos(
                self.per_mib
                    .as_nanos()
                    .saturating_mul(u64::try_from(bytes.len()).expect("output length"))
                    .div_ceil(1 << 20),
            );
            self.work = self.work.saturating_add(cost);
            self.ready = self.ready.map(|ready| ready.saturating_add(cost));
        }
        if let Some(after) = self.stall_after {
            let room = usize::try_from(after.saturating_sub(self.sent)).unwrap_or(usize::MAX);
            bytes = bytes.into_vec().into_iter().take(room).collect();
        }
        if !bytes.is_empty() {
            self.jobs.push_back(Job::Pipe(*self.pipes.get(&descriptor).expect("output pipe"), bytes, 0));
        }
    }
}
impl Host for Hosted {
    fn iterate(&mut self, now: Time, _wall: Wall) {
        if self.ready.is_none() {
            self.ready = Some(now.saturating_add(self.delay));
        }
        while let Some(complete) = self.completions.pop() {
            self.complete(complete);
        }
        if self.pending.is_some() || self.finished {
            return;
        }
        if self.jobs.is_empty() && !self.prepared {
            self.prepare();
        }
        if self.jobs.is_empty() && self.exit.is_none() {
            if now < self.ready.expect("initial work deadline")
                || self.stall_after.is_some_and(|after| self.sent >= after)
            {
                return;
            }
            self.want();
        }
        if self.jobs.is_empty() && self.exit.is_some() {
            self.jobs.extend(self.owned.iter().copied().map(Job::Close));
            if self.jobs.is_empty() {
                self.finished = true;
            }
        }
        if matches!(self.jobs.front(), Some(Job::Pipe(..))) && now < self.ready.expect("work deadline") {
            return;
        }
        if let Some(job) = self.jobs.pop_front() {
            self.next(job);
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.pending.is_none()
            && !self.finished
            && (!self.jobs.is_empty()
                && (!matches!(self.jobs.front(), Some(Job::Pipe(..))) || self.ready.is_some_and(|ready| now >= ready))
                || !self.prepared
                || self.exit.is_some()
                || (self.ready.is_some_and(|ready| now >= ready)
                    && self.stall_after.is_none_or(|after| self.sent < after)))
    }
    fn next_deadline(&self) -> Option<Time> {
        (self.pending.is_none()
            && (self.jobs.is_empty() || matches!(self.jobs.front(), Some(Job::Pipe(..))))
            && self.exit.is_none()
            && self.stall_after.is_none_or(|after| self.sent < after))
        .then_some(self.ready)
        .flatten()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.finished && self.completions.is_empty() && self.submissions.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then(|| Exit::Code(self.exit.expect("tool exit")))
    }
    fn operations(&self) -> u32 {
        fixtures::HOST_OPERATIONS
    }
    fn worst_case(&self) -> u64 {
        fixtures::FAKE_WORST_CASE
    }
}
#[must_use]
pub fn named(roots: &[(Box<[u8]>, Fd)], name: &[u8]) -> Fd {
    roots.iter().find(|(key, _)| key.as_ref() == name).expect("named startup descriptor").1
}
pub(super) fn rng(seed: u64, spawn: &k::Spawn) -> Rng {
    let hash =
        spawn.args.iter().flatten().fold(seed, |hash, byte| hash.wrapping_mul(31).wrapping_add(u64::from(*byte)));
    Rng::new(hash)
}
