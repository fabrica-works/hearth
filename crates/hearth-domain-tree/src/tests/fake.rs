//! A filesystem as a table of nodes, and the helper's side of the walk: it answers each request
//! from the table, in the worst order it can (names backwards), and records what it was asked.
use crate::{
    Tree,
    limits::Limits,
    step,
    vocabulary::{
        Answer, Cause, Entry, Event, Fault, Item, Kernel, Kind, Local, Output, Parent, Request, Stat, TreeFile,
    },
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{Env, List, Queue, Time, Wall, bytes};

/// One entry of the fake: its path, its stat, its attributes (names and values, covered or not),
/// its target if a link, the bytes a hash reads, and the byte its content digest is made of
/// (its inode's, unless set).
#[derive(Clone, Copy)]
pub struct Node {
    pub path: &'static [u8],
    pub stat: Stat,
    pub attrs: &'static [(&'static [u8], &'static [u8])],
    pub target: &'static [u8],
    pub read: u64,
    pub content: Option<u8>,
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
    Node { path, stat, attrs: &[], target: b"", read: stat.size, content: None }
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
    /// The change time and modification time, as a write moves them.
    pub const fn changed(self, ctime: (u64, u32)) -> Node {
        Node { stat: Stat { ctime, mtime: ctime, ..self.stat }, ..self }
    }
    /// A file's content, as the byte its digest is made of.
    pub const fn holds(self, content: u8) -> Node {
        Node { content: Some(content), ..self }
    }
}

pub const TREE: Digest = Digest([7; 32]);
pub const CHANGES: Digest = Digest([8; 32]);
pub const FILE: Digest = Digest([9; 32]);
/// The parent's record, which the stream of a parent ends as.
pub const PARENT: Parent =
    Parent { tree: Digest([21; 32]), tree_file: TreeFile { sha256: Digest([22; 32]), size: 4_321 } };
pub const KEY: Digest = Digest([5; 32]);
/// What `Finished` says of the tree file.
pub const FILE_SIZE: u64 = 99;

/// The parent's tree file as the fake streams it: the entries it gives, in order, and how the
/// stream ends.
#[derive(Clone, Debug)]
pub struct Stream {
    pub entries: Box<[(Entry, Local)]>,
    pub tree: Digest,
    pub boot: Box<[Kernel]>,
    pub sha256: Digest,
    pub size: u64,
}
impl Stream {
    /// The stream of `entries`, ending as the record `PARENT` says.
    pub fn of(entries: Box<[(Entry, Local)]>) -> Stream {
        Stream {
            entries,
            tree: PARENT.tree,
            boot: Box::default(),
            sha256: PARENT.tree_file.sha256,
            size: PARENT.tree_file.size,
        }
    }
    /// The stream of what `run` wrote: the entries with their locals, which a child walks against.
    pub fn after(run: &Run) -> Stream {
        let mut entries = List::with_capacity(run.items.len());
        for item in &run.items {
            if let Item::Entry(entry, local) = item {
                assert!(entries.push((entry.clone(), *local)).is_ok(), "room");
            }
        }
        Stream::of(entries.into_boxed())
    }
}

/// What a walk asked and wrote, and how it ended.
#[derive(Debug)]
pub struct Run {
    pub requests: List<Request>,
    /// Every item written, in order.
    pub items: List<Item>,
    pub entries: List<Entry>,
    pub puts: List<Box<[u8]>>,
    pub deletes: List<Box<[u8]>>,
    /// The sizes, in items, of the writes.
    pub writes: List<u32>,
    pub answer: Answer,
}
impl Run {
    /// The failure: its path and cause.
    pub fn failed(&self) -> (&[u8], Cause) {
        match &self.answer {
            Answer::Failed { path, cause } => (path, *cause),
            Answer::Walked { .. } => panic!("the walk failed: {:?}", self.answer),
        }
    }
    /// What the walk did, path by path in order: `=` an entry that is no change, `+` an entry
    /// and its put, `-` a delete.
    pub fn changes(&self) -> List<(u8, Box<[u8]>)> {
        let mut changes: List<(u8, Box<[u8]>)> = List::with_capacity(self.items.len());
        for item in &self.items {
            match item {
                Item::Entry(entry, _) => assert!(changes.push((b'=', bytes::copy_of(&entry.path))).is_ok(), "room"),
                Item::Put { path } => {
                    let last = changes.get_mut(changes.len().saturating_sub(1)).expect("a put follows its entry");
                    assert_eq!(&*last.1, &**path, "a put follows its own entry");
                    last.0 = b'+';
                }
                Item::Delete { path } => assert!(changes.push((b'-', bytes::copy_of(path))).is_ok(), "room"),
            }
        }
        changes
    }
    /// How many requests of a kind the walk made.
    pub fn count(&self, kind: fn(&Request) -> bool) -> u32 {
        let mut count = 0_u32;
        for request in &self.requests {
            if kind(request) {
                count = count.saturating_add(1);
            }
        }
        count
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
/// Walks `nodes` against the parent's stream.
pub fn merge(nodes: &[Node], stream: &Stream, limits: Limits) -> Run {
    go(nodes, limits, None, Fault::NotFound, Some(stream))
}
/// The same, with `fault` as the request, counting from 0, that fails.
pub fn merge_with(nodes: &[Node], stream: &Stream, limits: Limits, fault: Option<u32>, kind: Fault) -> Run {
    go(nodes, limits, fault, kind, Some(stream))
}
/// The assertion that `changes` are `expected`.
pub fn assert_changes(run: &Run, expected: &[(u8, &[u8])]) {
    let found = run.changes();
    let mut shown = List::with_capacity(found.len());
    for (what, path) in &found {
        assert!(shown.push((char::from(*what), bytes::copy_of(path))).is_ok(), "room");
    }
    assert_eq!(usize::try_from(found.len()).expect("small"), expected.len(), "changes: {shown:?}");
    for ((what, path), (wanted, at)) in found.iter().zip(expected) {
        assert_eq!((*what, &**path), (*wanted, *at), "changes: {shown:?}");
    }
}
/// Walks `nodes` with `fault` as the request, counting from 0, that fails with `NotFound`.
pub fn run_with(nodes: &[Node], limits: Limits, fault: Option<u32>) -> Run {
    drive(nodes, limits, fault, Fault::NotFound)
}
pub fn drive(nodes: &[Node], limits: Limits, fault: Option<u32>, kind: Fault) -> Run {
    go(nodes, limits, fault, kind, None)
}
fn go(nodes: &[Node], limits: Limits, fault: Option<u32>, kind: Fault, stream: Option<&Stream>) -> Run {
    let mut tree = Tree::new(&limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut run = Run {
        requests: List::with_capacity(8_192),
        items: List::with_capacity(2_048),
        deletes: List::with_capacity(1_024),
        entries: List::with_capacity(1_024),
        puts: List::with_capacity(1_024),
        writes: List::with_capacity(256),
        answer: Answer::Failed { path: bytes::copy_of(b""), cause: Cause::Unexpected },
    };
    let mut cursor: Option<(Box<[u8]>, u32)> = None;
    let mut fed = 0_usize;
    let mut event = Event::Walk { key: KEY, parent: if stream.is_some() { Some(PARENT) } else { None } };
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
                    answer(nodes, stream, &request, &mut cursor, &mut fed, &mut run)
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

fn answer(
    nodes: &[Node],
    stream: Option<&Stream>,
    request: &Request,
    cursor: &mut Option<(Box<[u8]>, u32)>,
    fed: &mut usize,
    run: &mut Run,
) -> Event {
    match request {
        Request::Stat { path } => match find(nodes, path) {
            Some(node) => Event::Stat(node.stat),
            None => NOT_FOUND,
        },
        Request::List { path, max } => {
            let event = page(&children(nodes, path), 0, *max);
            let taken = match &event {
                Event::Names { names, .. } => count(names.len()),
                Event::Walk { .. }
                | Event::Stat(_)
                | Event::Value { .. }
                | Event::Target { .. }
                | Event::FileHashed { .. }
                | Event::Done
                | Event::Digested { .. }
                | Event::Finished { .. }
                | Event::ParentEntry { .. }
                | Event::ParentEnd { .. }
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
                let byte = match node.content {
                    Some(byte) => byte,
                    None => u8::try_from(node.stat.inode).expect("a small inode"),
                };
                Event::FileHashed { sha256: Digest([byte; 32]), size: node.read }
            }
            None => NOT_FOUND,
        },
        Request::Create => Event::Done,
        Request::Write { items } => {
            assert!(run.writes.push(count(items.len())).is_ok(), "room");
            for item in items {
                assert!(run.items.push(item.clone()).is_ok(), "room");
                match item {
                    Item::Entry(entry, _) => assert!(run.entries.push(entry.clone()).is_ok(), "room"),
                    Item::Put { path } => assert!(run.puts.push(bytes::copy_of(path)).is_ok(), "room"),
                    Item::Delete { path } => assert!(run.deletes.push(bytes::copy_of(path)).is_ok(), "room"),
                }
            }
            Event::Done
        }
        Request::Digests { .. } => Event::Digested { tree: TREE, changes: CHANGES },
        Request::Finish { .. } => Event::Finished { sha256: FILE, size: FILE_SIZE },
        Request::ParentNext => {
            let stream = stream.expect("a parent is streamed only when there is one");
            match stream.entries.get(*fed) {
                Some((entry, local)) => {
                    *fed = fed.saturating_add(1);
                    Event::ParentEntry { entry: entry.clone(), local: *local }
                }
                None => Event::ParentEnd {
                    tree: stream.tree,
                    boot: stream.boot.clone(),
                    sha256: stream.sha256,
                    size: stream.size,
                },
            }
        }
    }
}
