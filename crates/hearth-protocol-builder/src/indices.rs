//! Read apt's verified indices once, sharing each pass with the stanza decoder (render.md, 5.3).
use crate::{Asked, builder::vocabulary::Event, limits::Limits, render, resolve::OWNER, translate};
use alloc::boxed::Box;
use hearth_protocol_mmdebstrap::{lists, packages, vector};
use render::vocabulary::Digest;
use skein_io::{file, kernel::Fd};
use skein_lib::{
    Env, Intake, List, Queue, Token, Writer,
    sha256::Sha256,
    stream::{Down, Read, Up},
};

#[derive(Debug, PartialEq, Eq)]
enum State {
    Directory,
    Listing,
    CloseDirectory,
    Index,
    Reading,
    SyncIndex,
    CloseIndex,
    Arrival,
    Done,
}
#[derive(Debug)]
pub(super) struct Indices {
    request: Token,
    root: Fd,
    path: Box<[u8]>,
    lines: Box<[vector::Line]>,
    files: Box<[lists::Index]>,
    seen: Box<[bool]>,
    records: List<render::resolution::Index>,
    next: usize,
    directory: Option<Token>,
    file: Option<Token>,
    state: State,
    failed: bool,
    waiting: bool,
    listed: u32,
    offset: u64,
    file_read: u32,
    hash: Sha256,
    began: skein_lib::Time,
    intake: Intake,
    demand: Read,
    decoder: packages::Stanzas,
    eof: bool,
}
impl Indices {
    pub fn new(
        request: Token,
        root: Fd,
        scratch: &[u8],
        lines: Box<[vector::Line]>,
        architecture: &vector::Name,
        file_read: u32,
        env: &Env<Limits>,
    ) -> Self {
        let indices = lists::indices(&lines, architecture);
        let count = u32::try_from(indices.len()).expect("bounded indices");
        let mut seen = List::with_capacity(count);
        for _ in 0..count {
            seen.push(false).expect("index count");
        }
        let mut path = Writer::new(scratch.len().saturating_add(lists::LISTS.len()).saturating_add(1));
        path.put(scratch).expect("scratch path");
        path.put(b"/").expect("slash");
        path.put(lists::LISTS).expect("lists path");
        Self {
            request,
            root,
            path: path.finish(),
            lines,
            files: indices,
            seen: seen.into_boxed(),
            records: List::with_capacity(count),
            next: 0,
            directory: None,
            file: None,
            state: State::Directory,
            failed: false,
            waiting: false,
            listed: 0,
            offset: 0,
            file_read,
            hash: Sha256::new(),
            began: env.now,
            intake: Intake::with_capacity(packages::largest_demand(&env.limits.mmdebstrap).max(file_read)),
            demand: Read::Nothing,
            decoder: packages::Stanzas::new(&env.limits.mmdebstrap),
            eof: false,
        }
    }
    pub fn ready(&self) -> bool {
        !self.waiting
    }
    pub fn stop(&mut self) {
        self.failed = true;
    }
    pub fn up(&mut self, event: file::Event, env: &Env<Limits>) {
        self.waiting = false;
        match event {
            file::Event::Opened { file, .. } => match self.state {
                State::Directory => {
                    self.directory = Some(file);
                    self.state = State::Listing;
                }
                State::Index => {
                    self.file = Some(file);
                    self.state = State::Reading;
                }
                State::Listing
                | State::CloseDirectory
                | State::Reading
                | State::SyncIndex
                | State::CloseIndex
                | State::Arrival
                | State::Done => {
                    unreachable!("index open")
                }
            },
            file::Event::Listed { entries, .. } => {
                self.listed = self.listed.saturating_add(u32::try_from(entries.len()).expect("bounded page"));
                self.failed |= self.listed > env.limits.walk.entries;
                for entry in &entries {
                    match lists::listed(&entry.name, &self.files) {
                        lists::Listed::Index(at) => {
                            *self.seen.get_mut(usize::try_from(at).expect("index slot")).expect("known index") = true;
                        }
                        lists::Listed::Compressed(_) => self.failed = true,
                        lists::Listed::Other => self.failed |= index_name(&entry.name),
                    }
                }
                if entries.len() < usize::try_from(env.limits.walk.page).expect("page bound") {
                    for seen in &self.seen {
                        self.failed |= !seen;
                    }
                    self.state = State::Index;
                }
            }
            file::Event::Read { bytes, .. } => {
                self.eof = bytes.is_empty();
                self.offset =
                    self.offset.checked_add(u64::try_from(bytes.len()).expect("bounded read")).expect("file size");
                self.hash.update(&bytes);
                self.intake.append(&bytes).expect("read within intake room");
            }
            file::Event::Closed { .. } => {
                if self.state == State::CloseDirectory {
                    self.directory = None;
                    self.state = State::Done;
                } else {
                    self.file = None;
                    if !self.failed {
                        let index = self.files.get(self.next).expect("current index");
                        self.records
                            .push(render::resolution::Index {
                                path: index.path.clone(),
                                size: self.offset,
                                sha256: Digest(core::mem::replace(&mut self.hash, Sha256::new()).finish()),
                            })
                            .expect("index count");
                        self.state = State::Arrival;
                    }
                }
            }
            file::Event::Synced { .. } => self.state = State::CloseIndex,
            file::Event::Failed { .. } | file::Event::Cancelled { .. } | file::Event::Refused { .. } => {
                if self.state == State::CloseDirectory {
                    self.directory = None;
                }
                if self.state == State::CloseIndex {
                    self.file = None;
                }
                self.failed = true;
            }
            file::Event::Locked { .. }
            | file::Event::Busy { .. }
            | file::Event::Made { .. }
            | file::Event::Loaded { .. }
            | file::Event::Scanned { .. }
            | file::Event::Stored { .. }
            | file::Event::Conflict { .. }
            | file::Event::TooLarge { .. }
            | file::Event::Stated { .. }
            | file::Event::Written { .. }
            | file::Event::Cut { .. }
            | file::Event::Renamed { .. }
            | file::Event::Removed { .. } => unreachable!("indices file operation"),
        }
    }
    pub fn resume(&mut self, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Queue<Asked>) -> Option<bool> {
        if self.waiting {
            return None;
        }
        if self.failed {
            if let Some(file) = self.file.or(self.directory) {
                self.state = if self.file.is_some() { State::CloseIndex } else { State::CloseDirectory };
                self.ask(file::Request::Close { owner: OWNER, file }, below);
                return None;
            }
            self.state = State::Done;
            return Some(false);
        }
        let request = match self.state {
            State::Directory => {
                file::Request::OpenDirectory { owner: OWNER, root: self.root, name: self.path.clone(), no_follow: true }
            }
            State::Listing => file::Request::ListPage {
                owner: OWNER,
                directory: self.directory.expect("lists directory"),
                max: env.limits.walk.page,
            },
            State::CloseDirectory => {
                file::Request::Close { owner: OWNER, file: self.directory.expect("lists directory") }
            }
            State::Index => {
                let Some(index) = self.files.get(self.next) else {
                    self.state = State::CloseDirectory;
                    return None;
                };
                let mut path = Writer::new(self.path.len().saturating_add(index.name.len()).saturating_add(1));
                path.put(&self.path).expect("lists path");
                path.put(b"/").expect("slash");
                path.put(&index.name).expect("index name");
                self.began = env.now;
                self.offset = 0;
                self.eof = false;
                self.intake.clear();
                self.decoder = packages::Stanzas::new(&env.limits.mmdebstrap);
                let mut demands = Queue::with_capacity(1);
                packages::open(&mut self.decoder, &Self::decoder_env(env), &mut demands);
                self.demand = Self::demand(&mut demands);
                file::Request::OpenReadNoFollow { owner: OWNER, root: self.root, name: path.finish() }
            }
            State::Reading => {
                if self.read(env, above) {
                    return None;
                }
                if self.state == State::SyncIndex || self.failed {
                    return None;
                }
                file::Request::ReadAt {
                    owner: OWNER,
                    file: self.file.expect("index file"),
                    offset: self.offset,
                    max: self.intake.room().min(self.file_read),
                }
            }
            State::SyncIndex => file::Request::Sync { owner: OWNER, file: self.file.expect("index file") },
            State::Arrival => {
                let index = self.records.as_slice().last().expect("read index");
                self.waiting = true;
                above.push(Event::Index {
                    request: self.request,
                    arrival: OWNER,
                    sha256: index.sha256,
                    size: index.size,
                    took: env.now.saturating_since(self.began),
                });
                return None;
            }
            State::CloseIndex => file::Request::Close { owner: OWNER, file: self.file.expect("index file") },
            State::Done => return Some(true),
        };
        self.ask(request, below);
        None
    }
    fn ask(&mut self, request: file::Request, below: &mut Queue<Asked>) {
        self.waiting = true;
        below.push(Asked::File(request));
    }
    fn demand(demands: &mut Queue<Down>) -> Read {
        match demands.pop() {
            Some(Down::Demand { read, .. }) => read,
            Some(Down::Send(_) | Down::Finish) | None => Read::Nothing,
        }
    }
    fn decoder_env(env: &Env<Limits>) -> Env<hearth_protocol_mmdebstrap::limits::Limits> {
        Env { now: env.now, wall: env.wall, limits: env.limits.mmdebstrap }
    }
    fn read(&mut self, env: &Env<Limits>, above: &mut Queue<Event>) -> bool {
        let index = self.files.get(self.next).expect("current index");
        if index.component.is_none() {
            self.intake.clear();
            if self.eof {
                self.state = State::SyncIndex;
                return true;
            }
            return false;
        }
        let bytes = match self.intake.meet(self.demand) {
            Some(bytes) => Some(bytes),
            None if self.eof => self.intake.meet(Read::Arrived { max: self.intake.capacity() }),
            None => None,
        };
        let up = match bytes {
            Some(bytes) => Up::Bytes(bytes),
            None if self.eof => Up::End,
            None => return false,
        };
        let mut found = Queue::with_capacity(2);
        let mut demands = Queue::with_capacity(1);
        packages::up(&mut self.decoder, &Self::decoder_env(env), up, &mut found, &mut demands);
        self.demand = Self::demand(&mut demands);
        for _ in 0..packages::UP_MAX_OUT.above {
            match found.pop() {
                Some(packages::Found::Stanza(stanza)) => above.push(Event::Render(render::vocabulary::Event::Stanza {
                    request: self.request,
                    line: u32::from(index.line),
                    stanza: translate::resolution::stanza(stanza),
                })),
                Some(packages::Found::Ended) => self.state = State::SyncIndex,
                Some(packages::Found::Failed(_)) => self.failed = true,
                None => break,
            }
        }
        true
    }
    pub fn arrival(&self) -> Option<(Token, Box<[u8]>)> {
        if self.state != State::Arrival || !self.waiting {
            return None;
        }
        Some((self.directory.expect("lists directory"), self.files.get(self.next).expect("current index").name.clone()))
    }
    pub fn settled(&mut self) {
        if self.state == State::Arrival {
            self.waiting = false;
            self.next = self.next.saturating_add(1);
            self.state = State::Index;
        }
    }
    pub fn lines(&mut self) -> Box<[render::resolution::Line]> {
        let mut lines = List::with_capacity(u32::try_from(self.lines.len()).expect("source count"));
        let mut records = core::mem::replace(&mut self.records, List::with_capacity(0)).into_boxed().into_iter();
        for line in &self.lines {
            let mut indices =
                List::with_capacity(u32::try_from(line.components.len().saturating_add(1)).expect("index count"));
            for _ in 0..=line.components.len() {
                indices.push(records.next().expect("read index")).expect("index count");
            }
            lines.push(translate::resolution::line(line, indices.into_boxed())).expect("source count");
        }
        lines.into_boxed()
    }
}

fn index_name(name: &[u8]) -> bool {
    let mut plain = name;
    for suffix in [b".xz".as_slice(), b".gz", b".bz2", b".lzma", b".lz4", b".zst"] {
        if let Some(prefix) = name.strip_suffix(suffix) {
            plain = prefix;
            break;
        }
    }
    plain.ends_with(b"_Packages") || plain.ends_with(b"_InRelease")
}
