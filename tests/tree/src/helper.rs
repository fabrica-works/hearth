//! The helper's side of a walk, played as h06's protocol layer will: it answers the domain's
//! requests from a fake filesystem, turns the domain's items into h05a's, writes them with its
//! writer into a byte file, reads the file back with its `Reader` for the digests, and ends the
//! file with the end item and the real header. Against a parent, it streams the parent's tree
//! file through the same `Reader`, change items skipped, as typed entries.
use crate::fs::Fs;
use hearth_domain_tree::{
    MAX_OUT, Tree,
    limits::Limits,
    step,
    vocabulary::{
        Answer, Cause, Counts, End, Entry, Event, Fault, Item, Kernel, Local, Output, Parent, Payload, Request,
        TreeFile, Xattr,
    },
};
use hearth_lib::encode::Digest;
use hearth_protocol_tree::{
    layer,
    limits::{self as protocol, Limits as ProtocolLimits},
    tree::{self, ChangesDigest, Next, Op, Reader, TreeDigest},
};
use skein_lib::{Env, Queue, Time, Wall, sha256};

/// A domain entry as h05a's typed entry: a total function.
#[must_use]
pub fn tree_entry(entry: &Entry) -> tree::Entry {
    tree::Entry {
        path: entry.path.clone(),
        mode: entry.mode,
        uid: entry.uid,
        gid: entry.gid,
        xattrs: entry
            .xattrs
            .iter()
            .map(|Xattr { name, value }| tree::Xattr { name: name.clone(), value: value.clone() })
            .collect(),
        payload: match &entry.payload {
            Payload::Directory => tree::Payload::Directory,
            Payload::File { size, content } => tree::Payload::File { size: *size, content: *content },
            Payload::Symlink { target } => tree::Payload::Symlink { target: target.clone() },
            Payload::Hardlink { path } => tree::Payload::Hardlink { path: path.clone() },
            Payload::Char { major, minor } => tree::Payload::Char { major: *major, minor: *minor },
            Payload::Block { major, minor } => tree::Payload::Block { major: *major, minor: *minor },
            Payload::Fifo => tree::Payload::Fifo,
            Payload::Socket => tree::Payload::Socket,
        },
    }
}
/// h05a's entry as the domain's: the other total function.
#[must_use]
pub fn domain_entry(entry: &tree::Entry) -> Entry {
    Entry {
        path: entry.path.clone(),
        mode: entry.mode,
        uid: entry.uid,
        gid: entry.gid,
        xattrs: entry
            .xattrs
            .iter()
            .map(|tree::Xattr { name, value }| Xattr { name: name.clone(), value: value.clone() })
            .collect(),
        payload: match &entry.payload {
            tree::Payload::Directory => Payload::Directory,
            tree::Payload::File { size, content } => Payload::File { size: *size, content: *content },
            tree::Payload::Symlink { target } => Payload::Symlink { target: target.clone() },
            tree::Payload::Hardlink { path } => Payload::Hardlink { path: path.clone() },
            tree::Payload::Char { major, minor } => Payload::Char { major: *major, minor: *minor },
            tree::Payload::Block { major, minor } => Payload::Block { major: *major, minor: *minor },
            tree::Payload::Fifo => Payload::Fifo,
            tree::Payload::Socket => Payload::Socket,
        },
    }
}
/// A domain item as h05a's.
#[must_use]
pub fn tree_item(item: &Item) -> tree::Item {
    match item {
        Item::Entry(entry, Local { inode, ctime, mtime }) => {
            tree::Item::Entry(tree_entry(entry), tree::Local { inode: *inode, ctime: *ctime, mtime: *mtime })
        }
        Item::Put { path } => tree::Item::Change { op: Op::Put, path: path.clone() },
        Item::Delete { path } => tree::Item::Change { op: Op::Delete, path: path.clone() },
    }
}
#[must_use]
pub fn tree_counts(counts: &Counts) -> layer::Counts {
    layer::Counts {
        bytes: counts.bytes,
        deletes: counts.deletes,
        entries: counts.entries,
        files: counts.files,
        hashed: counts.hashed,
        puts: counts.puts,
    }
}
/// The end item the domain asks for.
#[must_use]
pub fn tree_end(end: &End) -> tree::Item {
    tree::Item::End(tree::End {
        key: end.key,
        parent_tree: end.parent_tree,
        tree: end.tree,
        changes: end.changes,
        counts: tree_counts(&end.counts),
        boot: end
            .boot
            .iter()
            .map(|Kernel { version, kernel, modules, initramfs }| tree::Kernel {
                version: version.clone(),
                kernel: *kernel,
                modules: *modules,
                initramfs: *initramfs,
            })
            .collect(),
    })
}

/// What a damaged parent stream does: the helper tells the domain something the parent's file
/// does not say.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tamper {
    /// The end names another tree digest.
    Tree,
    /// The end names another SHA-256.
    Sha256,
    /// The end names another size.
    Size,
    /// The entry numbered so, counting from 0, comes after the one that follows it.
    Swap(usize),
    /// The entry numbered so comes twice.
    Repeat(usize),
}

/// The parent's tree file as the helper streams it.
#[derive(Debug)]
struct Stream<'a> {
    file: &'a [u8],
    reader: Reader,
    at: usize,
    /// Entries given so far.
    given: usize,
    /// An entry read ahead, to give next.
    stash: Option<(Entry, Local)>,
    tamper: Option<Tamper>,
}
enum Streamed {
    Entry(Entry, Local),
    End { tree: Digest, boot: Box<[Kernel]> },
}

/// What the helper made of a walk.
#[derive(Debug)]
pub struct Walked {
    pub requests: Vec<Request>,
    /// The tree file, whole if the walk was.
    pub file: Vec<u8>,
    pub answer: Answer,
}
impl Walked {
    /// The failure: its path and cause.
    #[must_use]
    pub fn failed(&self) -> (&[u8], Cause) {
        match &self.answer {
            Answer::Failed { path, cause } => (path, *cause),
            Answer::Walked { .. } => panic!("the walk failed: {:?}", self.answer),
        }
    }
}

/// The helper: the filesystem it reads, the file it writes, and the request that is to fail.
#[derive(Debug)]
pub struct Helper<'a> {
    fs: &'a Fs,
    limits: ProtocolLimits,
    /// The directory open, and how many of its names were given. Its names are listed again for
    /// each page, so that the helper holds none between requests.
    open: (Vec<u8>, usize),
    pub file: Vec<u8>,
    /// Requests answered so far.
    pub asked: usize,
    /// The request, counting from 0, that fails, and how.
    pub fault: Option<(usize, Fault)>,
    /// Items written, for the digests.
    items: u64,
    /// The parent's tree file, when the walk has one, and the record it comes with.
    stream: Option<(Stream<'a>, Parent)>,
}
impl<'a> Helper<'a> {
    #[must_use]
    pub fn new(fs: &'a Fs) -> Helper<'a> {
        Helper::reserving(fs, 0)
    }
    /// A helper whose file has room for `capacity` bytes, so that its growth is not a walk's.
    #[must_use]
    pub fn reserving(fs: &'a Fs, capacity: usize) -> Helper<'a> {
        Helper {
            fs,
            limits: protocol::SHIPPED,
            open: (Vec::new(), 0),
            file: Vec::with_capacity(capacity),
            asked: 0,
            fault: None,
            items: 0,
            stream: None,
        }
    }
    /// Walks against `file`, a tree file that a walk of the parent layer wrote.
    #[must_use]
    pub fn against(mut self, file: &'a [u8]) -> Helper<'a> {
        let mut reader = Reader::new(&self.limits);
        reader.header(file[..24].try_into().expect("a header")).expect("the parent is a tree file");
        let tree = end_of(file, &self.limits).tree;
        let record = Parent {
            tree,
            tree_file: TreeFile {
                sha256: Digest(sha256::digest(file)),
                size: u64::try_from(file.len()).expect("a size"),
            },
        };
        self.stream = Some((Stream { file, reader, at: 24, given: 0, stash: None, tamper: None }, record));
        self
    }
    /// Damages the parent stream.
    pub fn tamper(&mut self, tamper: Tamper) {
        let (stream, _) = self.stream.as_mut().expect("a parent is streamed");
        stream.tamper = Some(tamper);
    }
    /// The parent's record that a walk against the file starts with.
    #[must_use]
    pub fn parent(&self) -> Option<Parent> {
        self.stream.as_ref().map(|(_, record)| *record)
    }
    pub fn limit(&mut self, limits: ProtocolLimits) {
        self.limits = limits;
    }

    /// The event that answers `request`.
    pub fn answer(&mut self, request: &Request) -> Event {
        let at = self.asked;
        self.asked += 1;
        if let Some((fails, fault)) = self.fault
            && fails == at
        {
            return Event::Failed { fault };
        }
        match request {
            Request::Stat { path } => match self.fs.stat(path) {
                Some(stat) => Event::Stat(stat),
                None => not_found(),
            },
            Request::List { path, max } => {
                self.open = (path.to_vec(), 0);
                self.page(*max)
            }
            Request::Next { max } => self.page(*max),
            Request::Attributes { path } => match self.fs.lookup(path) {
                Some(at) => Event::Names {
                    names: self.fs.node(at).xattrs.iter().map(|(name, _)| name.as_slice().into()).collect(),
                    last: true,
                },
                None => not_found(),
            },
            Request::Attribute { path, name } => {
                let found = self.fs.lookup(path).and_then(|at| {
                    self.fs
                        .node(at)
                        .xattrs
                        .iter()
                        .find(|(attribute, _)| attribute.as_slice() == &**name)
                        .map(|(_, value)| value.clone())
                });
                match found {
                    Some(value) => Event::Value { bytes: value.into() },
                    None => not_found(),
                }
            }
            Request::ReadLink { path } => match self.fs.lookup(path) {
                Some(at) => Event::Target { bytes: self.fs.node(at).target.as_slice().into() },
                None => not_found(),
            },
            Request::HashFile { path, .. } => match self.fs.lookup(path) {
                Some(at) => {
                    let node = self.fs.node(at);
                    let read = u64::try_from(node.content.len()).expect("a size") + node.grown;
                    Event::FileHashed { sha256: Digest(sha256::digest(&node.content)), size: read }
                }
                None => not_found(),
            },
            Request::Create => {
                self.file.clear();
                self.file.extend_from_slice(&tree::header(0));
                self.items = 0;
                Event::Done
            }
            Request::Write { items } => self.write(items),
            Request::ParentNext => self.parent_next(),
            Request::Digests { parent_tree, entries, changes } => self.digests(*parent_tree, *entries, *changes),
            Request::Finish { end } => self.finish(end),
        }
    }
    fn parent_next(&mut self) -> Event {
        let (stream, _) = self.stream.as_mut().expect("a parent is streamed when it is asked for");
        let number = stream.given;
        if let Some((entry, local)) = stream.stash.take() {
            stream.given += 1;
            return Event::ParentEntry { entry, local };
        }
        match stream.read() {
            Streamed::Entry(entry, local) => {
                stream.given += 1;
                match stream.tamper {
                    Some(Tamper::Swap(at)) if at == number => {
                        if let Streamed::Entry(second, other) = stream.read() {
                            stream.stash = Some((entry, local));
                            return Event::ParentEntry { entry: second, local: other };
                        }
                        Event::ParentEntry { entry, local }
                    }
                    Some(Tamper::Repeat(at)) if at == number => {
                        stream.stash = Some((entry.clone(), local));
                        Event::ParentEntry { entry, local }
                    }
                    Some(Tamper::Swap(_) | Tamper::Repeat(_) | Tamper::Tree | Tamper::Sha256 | Tamper::Size) | None => {
                        Event::ParentEntry { entry, local }
                    }
                }
            }
            Streamed::End { tree, boot } => {
                let flip = |digest: Digest| {
                    let mut bytes = digest.0;
                    bytes[0] ^= 1;
                    Digest(bytes)
                };
                let (mut tree, mut sha256, mut size) =
                    (tree, Digest(sha256::digest(stream.file)), u64::try_from(stream.file.len()).expect("a size"));
                match stream.tamper {
                    Some(Tamper::Tree) => tree = flip(tree),
                    Some(Tamper::Sha256) => sha256 = flip(sha256),
                    Some(Tamper::Size) => size += 1,
                    Some(Tamper::Swap(_) | Tamper::Repeat(_)) | None => {}
                }
                Event::ParentEnd { tree, boot, sha256, size }
            }
        }
    }
    fn page(&mut self, max: u32) -> Event {
        let (path, given) = &mut self.open;
        let Some(names) = self.fs.list(path) else { return not_found() };
        let max = usize::try_from(max).expect("a page");
        let end = (*given + max).min(names.len());
        let page: Box<[Box<[u8]>]> = names[*given..end].iter().map(|name| name.as_slice().into()).collect();
        *given = end;
        Event::Names { names: page, last: end == names.len() }
    }
    fn write(&mut self, items: &[Item]) -> Event {
        for item in items {
            match tree::item(&tree_item(item), &self.limits) {
                Ok(bytes) => self.file.extend_from_slice(&bytes),
                Err(_) => return Event::Failed { fault: Fault::Bound },
            }
            self.items += 1;
        }
        Event::Done
    }
    /// The file's body, read item by item with h05a's reader; each of `count` items goes to `take`.
    fn read(&self, count: u64, take: &mut dyn FnMut(tree::Item)) {
        let body = u64::try_from(self.file.len() - 24).expect("a size");
        let mut reader = Reader::new(&self.limits);
        reader.header(&tree::header(body)).expect("the header is the file's");
        let mut at = 24;
        for _ in 0..count {
            match reader.next(&self.file[at..]).expect("the reader takes what the writer wrote") {
                Next::Item(item, used) => {
                    at += usize::try_from(used).expect("small");
                    take(item);
                }
                Next::Need(_) => panic!("a whole item is in the file"),
            }
        }
        assert_eq!(at, self.file.len(), "every item was read");
    }
    fn digests(&mut self, parent_tree: Option<Digest>, entries: u64, changes: u64) -> Event {
        assert_eq!(entries + changes, self.items, "the counts the walk gives are the items written");
        let mut digest = TreeDigest::new(entries);
        self.read(self.items, &mut |item| {
            if let tree::Item::Entry(entry, _) = &item {
                digest.entry(entry);
            }
        });
        let tree = digest.finish();
        let mut digest = ChangesDigest::new(parent_tree, tree, changes);
        let mut last: Option<tree::Entry> = None;
        self.read(self.items, &mut |item| match item {
            tree::Item::Entry(entry, _) => last = Some(entry),
            tree::Item::Change { op, path } => {
                digest.change(op, &path, if op == Op::Put { last.as_ref() } else { None });
            }
            tree::Item::End(_) => panic!("no end item yet"),
        });
        Event::Digested { tree, changes: digest.finish() }
    }
    fn finish(&mut self, end: &End) -> Event {
        match tree::item(&tree_end(end), &self.limits) {
            Ok(bytes) => self.file.extend_from_slice(&bytes),
            Err(_) => return Event::Failed { fault: Fault::Bound },
        }
        let body = u64::try_from(self.file.len() - 24).expect("a size");
        self.file[..24].copy_from_slice(&tree::header(body));
        Event::Finished {
            sha256: Digest(sha256::digest(&self.file)),
            size: u64::try_from(self.file.len()).expect("a size"),
        }
    }
}
fn not_found() -> Event {
    Event::Failed { fault: Fault::NotFound }
}

impl Stream<'_> {
    /// The next entry of the file, or its end; change items are skipped.
    fn read(&mut self) -> Streamed {
        for _ in 0..=self.file.len() {
            match self.reader.next(&self.file[self.at..]).expect("the reader takes the parent file") {
                Next::Item(item, used) => {
                    self.at += usize::try_from(used).expect("small");
                    match item {
                        tree::Item::Entry(entry, local) => {
                            let local = Local { inode: local.inode, ctime: local.ctime, mtime: local.mtime };
                            return Streamed::Entry(domain_entry(&entry), local);
                        }
                        tree::Item::Change { .. } => {}
                        tree::Item::End(end) => {
                            let boot = end
                                .boot
                                .iter()
                                .map(|kernel| Kernel {
                                    version: kernel.version.clone(),
                                    kernel: kernel.kernel,
                                    modules: kernel.modules,
                                    initramfs: kernel.initramfs,
                                })
                                .collect();
                            return Streamed::End { tree: end.tree, boot };
                        }
                    }
                }
                Next::Need(_) => panic!("the whole file is fed"),
            }
        }
        panic!("each item takes bytes")
    }
}
/// The end item of a tree file.
#[must_use]
pub fn end_of(file: &[u8], limits: &ProtocolLimits) -> tree::End {
    let items = crate::check::read_tree(file, limits).expect("a tree file");
    match items.last() {
        Some(tree::Item::End(end)) => end.clone(),
        Some(tree::Item::Entry(..) | tree::Item::Change { .. }) | None => panic!("a tree file ends with its end item"),
    }
}

/// Walks `fs` with a fresh machine until it answers.
#[must_use]
pub fn walk(fs: &Fs, limits: &Limits, key: Digest) -> Walked {
    walk_with(Helper::new(fs), limits, key)
}
/// The same, with a helper set up: a fault, a smaller bound.
#[must_use]
pub fn walk_with(mut helper: Helper<'_>, limits: &Limits, key: Digest) -> Walked {
    let mut machine = Tree::new(limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut requests = Vec::new();
    let mut event = Event::Walk { key, parent: helper.parent() };
    loop {
        let mut out = Queue::with_capacity(MAX_OUT);
        step(&mut machine, &env, event, &mut out);
        match out.pop().expect("a step has an output") {
            Output::Answer(answer) => return Walked { requests, file: helper.file, answer },
            Output::Request(request) => {
                event = helper.answer(&request);
                requests.push(request);
            }
        }
    }
}
