//! A filesystem as a table of nodes, and the helper's side of the walk: it answers each request
//! from the table, in the worst order it can (names backwards), and records what it was asked.
use crate::{
    Tree,
    limits::Limits,
    step,
    vocabulary::{Answer, Cause, Entry, Event, Fault, Item, Kind, Output, Request, Stat},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{Env, List, Queue, Time, Wall, bytes};

/// One entry of the fake: its path, its stat, its attributes (names and values, covered or not),
/// its target if a link, and the bytes a hash reads.
#[derive(Clone, Copy)]
pub struct Node {
    pub path: &'static [u8],
    pub stat: Stat,
    pub attrs: &'static [(&'static [u8], &'static [u8])],
    pub target: &'static [u8],
    pub read: u64,
}
pub const fn stat(kind: Kind, mode: u32, inode: u64) -> Stat {
    Stat {
        kind,
        mode,
        uid: 0,
        gid: 0,
        size: 0,
        inode,
        links: 1,
        device: 1,
        major: 0,
        minor: 0,
        ctime: (10, 1),
        mtime: (20, 2),
    }
}
const fn node(path: &'static [u8], stat: Stat) -> Node {
    Node { path, stat, attrs: &[], target: b"", read: stat.size }
}
pub const fn root() -> Node {
    node(b"", stat(Kind::Directory, 0o755, 1))
}
pub const fn dir(path: &'static [u8], inode: u64) -> Node {
    node(path, stat(Kind::Directory, 0o755, inode))
}
pub const fn file(path: &'static [u8], inode: u64, size: u64) -> Node {
    node(path, Stat { size, ..stat(Kind::File, 0o644, inode) })
}
#[expect(clippy::as_conversions, reason = "a const fn has no try_from, and a target is short")]
pub const fn link(path: &'static [u8], inode: u64, target: &'static [u8]) -> Node {
    let size = target.len() as u64;
    Node { target, ..node(path, Stat { size, ..stat(Kind::Symlink, 0o777, inode) }) }
}
pub const fn special(path: &'static [u8], inode: u64, kind: Kind) -> Node {
    node(path, Stat { major: 4, minor: 2, ..stat(kind, 0o600, inode) })
}
impl Node {
    pub const fn linked(self, links: u32) -> Node {
        Node { stat: Stat { links, ..self.stat }, ..self }
    }
    pub const fn with(self, attrs: &'static [(&'static [u8], &'static [u8])]) -> Node {
        Node { attrs, ..self }
    }
    pub const fn mode(self, mode: u32) -> Node {
        Node { stat: Stat { mode, ..self.stat }, ..self }
    }
    pub const fn owner(self, uid: u32, gid: u32) -> Node {
        Node { stat: Stat { uid, gid, ..self.stat }, ..self }
    }
    pub const fn device(self, device: u64) -> Node {
        Node { stat: Stat { device, ..self.stat }, ..self }
    }
    pub const fn reads(self, read: u64) -> Node {
        Node { read, ..self }
    }
}

pub const TREE: Digest = Digest([7; 32]);
pub const CHANGES: Digest = Digest([8; 32]);
pub const FILE: Digest = Digest([9; 32]);
pub const KEY: Digest = Digest([5; 32]);
/// What `Finished` says of the tree file.
pub const FILE_SIZE: u64 = 99;

/// What a walk asked and wrote, and how it ended.
#[derive(Debug)]
pub struct Run {
    pub requests: List<Request>,
    pub entries: List<Entry>,
    pub puts: List<Box<[u8]>>,
    /// The sizes, in items, of the writes.
    pub writes: List<u32>,
    pub answer: Answer,
}
impl Run {
    /// The failure: its path and cause.
    pub fn failed(&self) -> (&[u8], Cause) {
        match &self.answer {
            Answer::Failed { path, cause } => (path, *cause),
            Answer::Hashed { .. } => panic!("the walk failed: {:?}", self.answer),
        }
    }
    /// The entry at `path`.
    pub fn entry(&self, path: &[u8]) -> &Entry {
        for entry in &self.entries {
            if &*entry.path == path {
                return entry;
            }
        }
        panic!("{path:?} is listed: {:?}", self.paths());
    }
    pub fn paths(&self) -> List<Box<[u8]>> {
        let mut paths = List::with_capacity(self.entries.len());
        for entry in &self.entries {
            assert!(paths.push(bytes::copy_of(&entry.path)).is_ok(), "room");
        }
        paths
    }
}
/// The paths of `paths` are `expected`, in order.
pub fn assert_paths(paths: &List<Box<[u8]>>, expected: &[&[u8]]) {
    assert_eq!(usize::try_from(paths.len()).expect("small"), expected.len(), "paths: {paths:?}");
    for (found, expected) in paths.iter().zip(expected) {
        assert_eq!(&**found, *expected, "paths: {paths:?}");
    }
}

pub fn run(nodes: &[Node], limits: Limits) -> Run {
    run_with(nodes, limits, None)
}
/// Walks `nodes` with `fault` as the request, counting from 0, that fails with `NotFound`.
pub fn run_with(nodes: &[Node], limits: Limits, fault: Option<u32>) -> Run {
    drive(nodes, limits, fault, Fault::NotFound)
}
pub fn drive(nodes: &[Node], limits: Limits, fault: Option<u32>, kind: Fault) -> Run {
    let mut tree = Tree::new(&limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut run = Run {
        requests: List::with_capacity(8_192),
        entries: List::with_capacity(1_024),
        puts: List::with_capacity(1_024),
        writes: List::with_capacity(256),
        answer: Answer::Failed { path: bytes::copy_of(b""), cause: Cause::Unexpected },
    };
    let mut cursor: Option<(Box<[u8]>, u32)> = None;
    let mut event = Event::Hash { key: KEY };
    for count in 0..8_192_u32 {
        let mut out = Queue::with_capacity(crate::MAX_OUT);
        step(&mut tree, &env, event, &mut out);
        assert_eq!(out.len(), 1, "one output a step");
        match out.pop().expect("one output") {
            Output::Answer(answer) => {
                run.answer = answer;
                return run;
            }
            Output::Request(request) => {
                event = if Some(run.requests.len()) == fault {
                    Event::Failed { fault: kind }
                } else {
                    answer(nodes, &request, &mut cursor, &mut run)
                };
                assert!(run.requests.push(request).is_ok(), "room for the requests, at {count}");
            }
        }
    }
    panic!("the walk ends");
}

fn find<'a>(nodes: &'a [Node], path: &[u8]) -> Option<&'a Node> {
    let mut found = None;
    for node in nodes {
        if node.path == path {
            found = Some(node);
        }
    }
    found
}
fn count(length: usize) -> u32 {
    u32::try_from(length).expect("small")
}
/// The names of `path`'s children, last first.
fn children(nodes: &[Node], path: &[u8]) -> List<Box<[u8]>> {
    let mut names = List::with_capacity(count(nodes.len()));
    for node in nodes.iter().rev() {
        let below = if path.is_empty() {
            Some(node.path)
        } else {
            match node.path.strip_prefix(path) {
                Some(rest) => rest.strip_prefix(b"/"),
                None => None,
            }
        };
        if let Some(name) = below
            && !name.is_empty()
            && !name.contains(&b'/')
        {
            assert!(names.push(bytes::copy_of(name)).is_ok(), "room");
        }
    }
    names
}
fn page(names: &List<Box<[u8]>>, from: u32, max: u32) -> Event {
    let mut page = List::with_capacity(max);
    for at in from..names.len() {
        if page.room() == 0 {
            break;
        }
        assert!(page.push(bytes::copy_of(names.get(at).expect("in range"))).is_ok(), "room");
    }
    let taken = from.saturating_add(page.len());
    Event::Names { names: page.into_boxed(), last: taken >= names.len() }
}
const NOT_FOUND: Event = Event::Failed { fault: Fault::NotFound };

fn answer(nodes: &[Node], request: &Request, cursor: &mut Option<(Box<[u8]>, u32)>, run: &mut Run) -> Event {
    match request {
        Request::Stat { path } => match find(nodes, path) {
            Some(node) => Event::Stat(node.stat),
            None => NOT_FOUND,
        },
        Request::List { path, max } => {
            let event = page(&children(nodes, path), 0, *max);
            let taken = match &event {
                Event::Names { names, .. } => count(names.len()),
                Event::Hash { .. }
                | Event::Stat(_)
                | Event::Value { .. }
                | Event::Target { .. }
                | Event::FileHashed { .. }
                | Event::Done
                | Event::Digested { .. }
                | Event::Finished { .. }
                | Event::Failed { .. } => 0,
            };
            *cursor = Some((bytes::copy_of(path), taken));
            event
        }
        Request::Next { max } => {
            let (path, from) = cursor.as_mut().expect("a directory is open");
            let event = page(&children(nodes, path), *from, *max);
            if let Event::Names { names, .. } = &event {
                *from = from.saturating_add(count(names.len()));
            }
            event
        }
        Request::Attributes { path } => match find(nodes, path) {
            Some(node) => {
                let mut names = List::with_capacity(count(node.attrs.len()));
                for (name, _) in node.attrs.iter().rev() {
                    assert!(names.push(bytes::copy_of(name)).is_ok(), "room");
                }
                Event::Names { names: names.into_boxed(), last: true }
            }
            None => NOT_FOUND,
        },
        Request::Attribute { path, name } => {
            let Some(node) = find(nodes, path) else { return NOT_FOUND };
            for (attribute, value) in node.attrs {
                if *attribute == &**name {
                    return Event::Value { bytes: bytes::copy_of(value) };
                }
            }
            NOT_FOUND
        }
        Request::ReadLink { path } => match find(nodes, path) {
            Some(node) => Event::Target { bytes: bytes::copy_of(node.target) },
            None => NOT_FOUND,
        },
        Request::HashFile { path, .. } => match find(nodes, path) {
            Some(node) => {
                let byte = u8::try_from(node.stat.inode).expect("a small inode");
                Event::FileHashed { sha256: Digest([byte; 32]), size: node.read }
            }
            None => NOT_FOUND,
        },
        Request::Create => Event::Done,
        Request::Write { items } => {
            assert!(run.writes.push(count(items.len())).is_ok(), "room");
            for item in items {
                match item {
                    Item::Entry(entry, _) => assert!(run.entries.push(entry.clone()).is_ok(), "room"),
                    Item::Put { path } => assert!(run.puts.push(bytes::copy_of(path)).is_ok(), "room"),
                }
            }
            Event::Done
        }
        Request::Digests { .. } => Event::Digested { tree: TREE, changes: CHANGES },
        Request::Finish { .. } => Event::Finished { sha256: FILE, size: FILE_SIZE },
    }
}
