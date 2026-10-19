//! A filesystem in memory: nodes with inodes, devices, links and attributes of every family,
//! answering the walk's questions the way a layer would. Directories list their names in an
//! order of the seed's choosing, since a real directory has none.
use hearth_domain_tree::vocabulary::{Kind, Stat};
use skein_lib::Rng;

/// One inode as the fake keeps it. Two nodes may share an inode number, which no real
/// filesystem does, and so disagree about what a hard link is.
#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    pub inode: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub device: u64,
    pub major: u32,
    pub minor: u32,
    pub ctime: (u64, u32),
    pub mtime: (u64, u32),
    /// A file's contents.
    pub content: Vec<u8>,
    /// Bytes a hash reads past the contents, as when the file grew after its stat.
    pub grown: u64,
    pub target: Vec<u8>,
    /// Every attribute, covered or not, in the order listed.
    pub xattrs: Vec<(Vec<u8>, Vec<u8>)>,
    /// A directory's names, in the order listed, and the nodes they name.
    pub names: Vec<(Vec<u8>, usize)>,
}
impl Node {
    #[must_use]
    pub fn new(kind: Kind, inode: u64) -> Node {
        let mode = match kind {
            Kind::Directory => 0o755,
            Kind::File => 0o644,
            Kind::Symlink => 0o777,
            Kind::Char | Kind::Block | Kind::Fifo | Kind::Socket => 0o600,
        };
        Node {
            kind,
            inode,
            mode,
            uid: 0,
            gid: 0,
            device: 1,
            major: 0,
            minor: 0,
            ctime: (1_700_000_000 + inode, 7),
            mtime: (1_600_000_000 + inode, 9),
            content: Vec::new(),
            grown: 0,
            target: Vec::new(),
            xattrs: Vec::new(),
            names: Vec::new(),
        }
    }
    #[must_use]
    pub fn file(inode: u64, content: &[u8]) -> Node {
        Node { content: content.to_vec(), ..Node::new(Kind::File, inode) }
    }
    #[must_use]
    pub fn symlink(inode: u64, target: &[u8]) -> Node {
        Node { target: target.to_vec(), ..Node::new(Kind::Symlink, inode) }
    }
    #[must_use]
    pub fn device(kind: Kind, inode: u64, major: u32, minor: u32) -> Node {
        Node { major, minor, ..Node::new(kind, inode) }
    }
    #[must_use]
    pub fn attr(mut self, name: &[u8], value: &[u8]) -> Node {
        self.xattrs.push((name.to_vec(), value.to_vec()));
        self
    }
    #[must_use]
    pub fn owned(self, uid: u32, gid: u32) -> Node {
        Node { uid, gid, ..self }
    }
    #[must_use]
    pub fn moded(self, mode: u32) -> Node {
        Node { mode, ..self }
    }
    #[must_use]
    pub fn on(self, device: u64) -> Node {
        Node { device, ..self }
    }
}

#[derive(Clone, Debug)]
pub struct Fs {
    nodes: Vec<Node>,
    /// The kernel's clock for change times, which only moves forward.
    clock: u64,
}
impl Fs {
    /// A root alone: a directory, 0755, owned by 0, as the empty layer is.
    #[must_use]
    pub fn new() -> Fs {
        Fs { nodes: vec![Node::new(Kind::Directory, 1)], clock: 0 }
    }
    #[must_use]
    pub fn root(&self) -> &Node {
        &self.nodes[0]
    }
    pub fn root_mut(&mut self) -> &mut Node {
        &mut self.nodes[0]
    }
    /// Adds `node` at `path`, whose directory exists, and returns its index.
    pub fn add(&mut self, path: &[u8], node: Node) -> usize {
        let (parent, name) = split(path);
        let at = self.nodes.len();
        self.nodes.push(node);
        self.name(parent, name, at);
        at
    }
    /// Names the node at index `at` once more, at `path`: a hard link.
    pub fn link(&mut self, path: &[u8], at: usize) {
        let (parent, name) = split(path);
        self.name(parent, name, at);
    }
    fn name(&mut self, parent: &[u8], name: &[u8], at: usize) {
        let directory = self.lookup(parent).unwrap_or_else(|| panic!("{parent:?} exists"));
        assert_eq!(self.nodes[directory].kind, Kind::Directory, "{parent:?} is a directory");
        self.nodes[directory].names.push((name.to_vec(), at));
    }
    pub fn node_mut(&mut self, at: usize) -> &mut Node {
        &mut self.nodes[at]
    }
    #[must_use]
    pub fn node(&self, at: usize) -> &Node {
        &self.nodes[at]
    }
    /// The index of the node at `path`.
    #[must_use]
    pub fn lookup(&self, path: &[u8]) -> Option<usize> {
        let mut at = 0;
        if path.is_empty() {
            return Some(at);
        }
        for part in path.split(|&byte| byte == b'/') {
            let directory = &self.nodes[at];
            at = directory.names.iter().find(|(name, _)| name == part)?.1;
        }
        Some(at)
    }
    /// The hard links an inode has: names that lead to a node of its number.
    #[must_use]
    pub fn links(&self, inode: u64) -> u32 {
        let mut links = 0;
        for node in &self.nodes {
            for (_, at) in &node.names {
                if self.nodes[*at].inode == inode {
                    links += 1;
                }
            }
        }
        links.max(1)
    }
    /// What `statx` without following a link says.
    #[must_use]
    pub fn stat(&self, path: &[u8]) -> Option<Stat> {
        let node = &self.nodes[self.lookup(path)?];
        let size = match node.kind {
            Kind::File => node.content.len() as u64,
            Kind::Symlink => node.target.len() as u64,
            Kind::Directory | Kind::Char | Kind::Block | Kind::Fifo | Kind::Socket => 0,
        };
        Some(Stat {
            kind: node.kind,
            // The kernel's `st_mode & 0o7777`; the high bits are the kind's.
            mode: node.mode & 0o7777,
            uid: node.uid,
            gid: node.gid,
            size,
            inode: node.inode,
            links: self.links(node.inode),
            device: node.device,
            major: node.major,
            minor: node.minor,
            ctime: node.ctime,
            mtime: node.mtime,
        })
    }
    /// The names of a directory, in the order it lists them.
    #[must_use]
    pub fn list(&self, path: &[u8]) -> Option<Vec<Vec<u8>>> {
        let node = &self.nodes[self.lookup(path)?];
        Some(node.names.iter().map(|(name, _)| name.clone()).collect())
    }
    /// Lists the names of every directory in an order drawn from `rng`.
    pub fn shuffle(&mut self, rng: &mut Rng) {
        for node in &mut self.nodes {
            shuffle(rng, &mut node.names);
            shuffle(rng, &mut node.xattrs);
        }
    }
    /// The nodes, in the order they were added.
    #[must_use]
    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    /// A time later than any given before: what a change reads at.
    pub fn tick(&mut self) -> (u64, u32) {
        self.clock += 1;
        (1_900_000_000 + self.clock, 13)
    }
    /// `chmod`, `chown`, `setxattr`, a link made or lost, a rename: the change time moves.
    pub fn changed(&mut self, at: usize) {
        let time = self.tick();
        self.nodes[at].ctime = time;
    }
    /// A write, or `touch`: the change time and the modification time move.
    pub fn touched(&mut self, at: usize) {
        let time = self.tick();
        self.nodes[at].ctime = time;
        self.nodes[at].mtime = time;
    }
    /// Every name that leads from the root to a node, with the node: a hard link's names each.
    #[must_use]
    pub fn paths(&self) -> Vec<(Vec<u8>, usize)> {
        let mut found = Vec::new();
        self.walk(0, &mut Vec::new(), &mut found);
        found
    }
    fn walk(&self, at: usize, path: &mut Vec<u8>, found: &mut Vec<(Vec<u8>, usize)>) {
        found.push((path.clone(), at));
        for (name, child) in &self.nodes[at].names {
            let before = path.len();
            if !path.is_empty() {
                path.push(b'/');
            }
            path.extend_from_slice(name);
            self.walk(*child, path, found);
            path.truncate(before);
        }
    }
    /// The names that lead to the node.
    #[must_use]
    pub fn names_of(&self, at: usize) -> usize {
        self.nodes.iter().map(|node| node.names.iter().filter(|(_, child)| *child == at).count()).sum()
    }
    /// The largest inode number the nodes have.
    #[must_use]
    pub fn last_inode(&self) -> u64 {
        self.nodes.iter().map(|node| node.inode).max().unwrap_or(0)
    }
    /// Removes the name at `path`, and what is beneath it if it is a directory. The directory it
    /// was in changes; every node that loses a name but keeps another changes, as a link lost does.
    pub fn remove(&mut self, path: &[u8]) {
        let (parent, name) = split(path);
        let directory = self.lookup(parent).unwrap_or_else(|| panic!("{parent:?} exists"));
        let place = self.nodes[directory].names.iter().position(|(found, _)| found == name).expect("the name exists");
        let (_, at) = self.nodes[directory].names.remove(place);
        self.touched(directory);
        let mut gone = Vec::new();
        self.detach(at, &mut gone);
        for node in gone {
            if self.nodes[node].kind != Kind::Directory && self.names_of(node) > 0 {
                self.changed(node);
            }
        }
    }
    fn detach(&mut self, at: usize, gone: &mut Vec<usize>) {
        gone.push(at);
        let names = std::mem::take(&mut self.nodes[at].names);
        for (_, child) in names {
            self.detach(child, gone);
        }
    }
}
impl Default for Fs {
    fn default() -> Fs {
        Fs::new()
    }
}

fn split(path: &[u8]) -> (&[u8], &[u8]) {
    match path.iter().rposition(|&byte| byte == b'/') {
        Some(at) => (&path[..at], &path[at + 1..]),
        None => (b"", path),
    }
}
pub fn shuffle<T>(rng: &mut Rng, items: &mut [T]) {
    for at in (1..items.len()).rev() {
        let other = usize::try_from(rng.below(at as u64 + 1)).expect("an index");
        items.swap(at, other);
    }
}
