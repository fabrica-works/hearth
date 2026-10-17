//! Files and process-owned handles, testing.md, 3.1 and 3.2.
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet, VecDeque},
    vec::Vec,
};
use skein_lib::Rng;
mod files;
mod ledger;
mod processes;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Pid(u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Handle(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Node(u64);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ids {
    pub user: u32,
    pub group: u32,
}
/// Scenario-owned bounds. No host resources enter this machine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sizes {
    pub files: u32,
    pub bytes: u64,
    pub processes: u32,
    pub pipe: u32,
    pub ledger: u32,
}
impl Sizes {
    #[must_use]
    pub const fn tiny() -> Self {
        Self { files: 64, bytes: 1_048_576, processes: 8, pipe: 4096, ledger: 256 }
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Item {
    pub path: Box<[u8]>,
    pub made: Made,
    pub mode: u32,
    pub ids: Ids,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Made {
    File(Box<[u8]>),
    Directory,
    Link(Box<[u8]>),
    HardLink(Box<[u8]>),
}
impl Item {
    #[must_use]
    pub fn file(path: &[u8], mode: u32, bytes: &[u8]) -> Self {
        Self::make(path, mode, Made::File(bytes.into()))
    }
    #[must_use]
    pub fn directory(path: &[u8], mode: u32) -> Self {
        Self::make(path, mode, Made::Directory)
    }
    #[must_use]
    pub fn link(path: &[u8], target: &[u8]) -> Self {
        Self::make(path, 0o777, Made::Link(target.into()))
    }
    #[must_use]
    pub fn hard_link(path: &[u8], existing: &[u8]) -> Self {
        Self::make(path, 0, Made::HardLink(existing.into()))
    }
    #[must_use]
    pub const fn owner(mut self, ids: Ids) -> Self {
        self.ids = ids;
        self
    }
    fn make(path: &[u8], mode: u32, contents: Made) -> Self {
        Self { path: path.into(), made: contents, mode, ids: Ids { user: 0, group: 0 } }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum At {
    Root,
    Cwd,
    Beneath(Handle),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum How {
    PathNoFollow,
    Read,
    ReadNoFollow,
    Directory,
    DirectoryNoFollow,
    Create { mode: Option<u32> },
    CreateNoFollow { mode: Option<u32> },
    Write,
    WriteNoFollow,
}
#[derive(PartialEq, Eq, Debug)]
pub enum Op {
    Open { at: At, path: Box<[u8]>, how: How },
    Read { file: Handle, at: u64, most: u32 },
    Write { file: Handle, at: u64, bytes: Box<[u8]> },
    Truncate { file: Handle, length: u64 },
    Sync { file: Handle },
    Stat { handle: Handle },
    Rename { from_dir: Handle, from: Box<[u8]>, to_dir: Handle, to: Box<[u8]> },
    Remove { dir: Handle, path: Box<[u8]>, directory: bool },
    MakeDirectory { dir: Handle, path: Box<[u8]>, mode: u32 },
    List { dir: Handle, most: u32 },
    Lock { file: Handle },
    Close { handle: Handle },
    Spawn(Spawn),
    Send { pipe: Handle, bytes: Box<[u8]> },
    Take { pipe: Handle, most: u32 },
    Signal { child: Pid, signal: Signal, to: Target },
    Wait { child: Pid },
    Exit { exit: Exit },
}
#[derive(PartialEq, Eq, Debug)]
pub enum Reply {
    Opened(Handle),
    Read(Box<[u8]>),
    Done,
    Stat(Facts),
    Listed(Box<[(Kind, Box<[u8]>)]>),
    Spawned { child: Pid, pipes: Box<[Handle]> },
    Sent(u32),
    Took(Box<[u8]>),
    Empty,
    Ended,
    Running,
    Exited(Exit),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    NotFound,
    Exists,
    NotADirectory,
    IsADirectory,
    NotEmpty,
    Permission,
    NotPermitted,
    Escape,
    Loop,
    NameTooLong,
    Invalid,
    BrokenPipe,
    Busy,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Facts {
    pub kind: Kind,
    pub size: u64,
    pub mode: u32,
    pub user: u32,
    pub group: u32,
    pub links: u32,
    pub changes: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Kind {
    File,
    Directory,
    Link,
}
#[derive(Debug)]
struct Inode {
    facts: Facts,
    bytes: Vec<u8>,
    entries: BTreeMap<Box<[u8]>, Node>,
    parent: Node,
}
#[derive(Debug)]
struct Open {
    pid: Pid,
    node: Node,
    how: How,
    listed: Option<Vec<(Kind, Box<[u8]>)>>,
    cursor: usize,
    locked: bool,
    end: Option<End>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Way {
    In,
    Out,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Signal {
    Terminate,
    Kill,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    Child,
    Group,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Exit {
    Code(u8),
    Signal(u32),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pipe {
    pub child: u32,
    pub way: Way,
}
#[derive(PartialEq, Eq, Debug)]
pub struct Spawn {
    pub program: Box<[u8]>,
    pub args: Box<[Box<[u8]>]>,
    pub env: Box<[Box<[u8]>]>,
    pub root: Handle,
    pub dir: Box<[u8]>,
    pub pipes: Box<[Pipe]>,
}
#[derive(PartialEq, Eq, Debug)]
pub struct Invocation {
    pub program: Box<[u8]>,
    pub args: Box<[Box<[u8]>]>,
    pub env: Box<[Box<[u8]>]>,
    pub dir: Box<[u8]>,
    pub cwd: Handle,
}
#[derive(PartialEq, Eq, Debug)]
pub struct Entry {
    pub pid: Pid,
    pub ids: Ids,
    pub node: Option<Node>,
    pub bytes: u64,
    pub refused: Option<Refusal>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Counts {
    pub read: u64,
    pub written: u64,
    pub changed: u64,
    pub spawned: u32,
}
impl Counts {
    const ZERO: Self = Self { read: 0, written: 0, changed: 0, spawned: 0 };
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum End {
    Pipe { id: u64, way: Way },
    Null { way: Way },
}
#[derive(Debug)]
struct Process {
    ids: Ids,
    cwd: Node,
    root: Node,
    umask: u32,
    parent: Option<Pid>,
    group: Pid,
    program: Node,
    invocation: Option<Invocation>,
    descriptors: BTreeMap<u32, Handle>,
    exit: Option<Exit>,
    waited: bool,
    pending: Option<Signal>,
    counts: Counts,
    changed: BTreeSet<Node>,
}
impl Process {
    fn new(ids: Ids, cwd: Node, root: Node, umask: u32) -> Self {
        Self {
            ids,
            cwd,
            root,
            umask,
            parent: None,
            group: Pid(0),
            program: Node(0),
            invocation: None,
            descriptors: BTreeMap::new(),
            exit: None,
            waited: false,
            pending: None,
            counts: Counts::ZERO,
            changed: BTreeSet::new(),
        }
    }
}
#[derive(Debug)]
pub struct Machine {
    nodes: BTreeMap<Node, Inode>,
    handles: BTreeMap<Handle, Open>,
    processes: BTreeMap<Pid, Process>,
    rng: Rng,
    sizes: Sizes,
    next_node: u64,
    next_handle: u64,
    next_pid: u32,
    pipes: BTreeMap<u64, VecDeque<u8>>,
    next_pipe: u64,
    ledger: VecDeque<Entry>,
}
impl Machine {
    #[must_use]
    pub fn new(seed: u64, sizes: Sizes) -> Self {
        let mut machine = Self {
            nodes: BTreeMap::new(),
            handles: BTreeMap::new(),
            processes: BTreeMap::new(),
            rng: Rng::new(seed),
            sizes,
            next_node: 0,
            next_handle: 0,
            next_pid: 0,
            pipes: BTreeMap::new(),
            next_pipe: 0,
            ledger: VecDeque::new(),
        };
        machine.add(Node(1), Kind::Directory, 0o755, Ids { user: 0, group: 0 }, &[]);
        machine
    }
    pub fn login(&mut self, ids: Ids, dir: &[u8]) -> Pid {
        assert_ne!(ids.user, 0, "no process runs as uid 0");
        assert!(
            self.processes.len() < usize::try_from(self.sizes.processes).expect("process bound fits"),
            "scenario process bound"
        );
        let cwd = self.laid(dir).expect("login directory was laid");
        assert_eq!(self.inode(cwd).facts.kind, Kind::Directory, "login cwd is a directory");
        self.next_pid = self.next_pid.checked_add(1).expect("process identities fit");
        let pid = Pid(self.next_pid);
        let mut process = Process::new(ids, cwd, Node(1), 0o022);
        process.group = pid;
        self.processes.insert(pid, process);
        self.null_descriptors(pid);
        pid
    }
    fn process(&self, pid: Pid) -> &Process {
        let process = self.processes.get(&pid).expect("a process it started");
        assert_eq!(process.exit, None, "a process it started, still running");
        process
    }
    fn inode(&self, node: Node) -> &Inode {
        self.nodes.get(&node).expect("a node the machine issued")
    }
    fn inode_mut(&mut self, node: Node) -> &mut Inode {
        self.nodes.get_mut(&node).expect("a node the machine issued")
    }
    fn handle(&self, pid: Pid, handle: Handle) -> &Open {
        let open = self.handles.get(&handle).expect("a handle it issued");
        assert_eq!(open.pid, pid, "a handle it issued to this process");
        open
    }
    fn opened(&mut self, pid: Pid, node: Node, how: How) -> Handle {
        self.next_handle = self.next_handle.checked_add(1).expect("handle identities fit");
        let handle = Handle(self.next_handle);
        self.handles.insert(handle, Open { pid, node, how, listed: None, cursor: 0, locked: false, end: None });
        handle
    }
    #[must_use]
    pub fn facts(&self, path: &[u8]) -> Option<Facts> {
        Some(self.inode(self.laid(path)?).facts)
    }
    #[must_use]
    pub fn contents(&self, path: &[u8]) -> Option<&[u8]> {
        Some(&self.inode(self.laid(path)?).bytes)
    }
}
