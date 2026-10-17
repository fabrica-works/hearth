use super::{At, Facts, Handle, How, Ids, Inode, Item, Kind, Machine, Made, Node, Op, Pid, Refusal, Reply};
use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
    vec::Vec,
};

#[derive(Debug)]
struct Found {
    parent: Node,
    name: Box<[u8]>,
    node: Option<Node>,
}
impl Machine {
    pub(super) fn add(&mut self, parent: Node, kind: Kind, mode: u32, ids: Ids, bytes: &[u8]) -> Node {
        assert!(self.nodes.len() < usize::try_from(self.sizes.files).expect("file bound fits"), "scenario file bound");
        self.room(0, u64::try_from(bytes.len()).expect("contents fit"));
        self.next_node = self.next_node.checked_add(1).expect("node identities fit");
        let node = Node(self.next_node);
        self.nodes.insert(
            node,
            Inode {
                facts: Facts {
                    kind,
                    size: u64::try_from(bytes.len()).expect("contents fit"),
                    mode: mode & 0o7777,
                    user: ids.user,
                    group: ids.group,
                    links: if kind == Kind::Directory { 2 } else { 1 },
                    changes: 0,
                },
                bytes: bytes.into(),
                entries: BTreeMap::new(),
                parent,
            },
        );
        node
    }
    fn room(&self, old: u64, new: u64) {
        let used = self.nodes.values().map(|n| n.facts.size).sum::<u64>();
        assert!(
            used.checked_sub(old).and_then(|n| n.checked_add(new)).expect("bounded contents") <= self.sizes.bytes,
            "scenario byte bound"
        );
    }
    fn change(&mut self, node: Node) {
        let facts = &mut self.inode_mut(node).facts;
        facts.changes = facts.changes.checked_add(1).expect("change counter fits");
    }
    pub(super) fn laid(&self, path: &[u8]) -> Option<Node> {
        let mut node = Node(1);
        for name in path.split(|b| *b == b'/').filter(|n| !n.is_empty()) {
            node = *self.inode(node).entries.get(name)?;
        }
        Some(node)
    }
    pub fn lay(&mut self, items: &[Item]) -> Box<[Node]> {
        let mut laid = Vec::new();
        for item in items {
            assert_eq!(item.path.first(), Some(&b'/'), "laid paths are absolute");
            let cut = item.path.iter().rposition(|b| *b == b'/').expect("absolute path");
            let parent = self.laid(item.path.get(..cut).expect("path prefix")).expect("parents first");
            let name = item.path.get(cut.checked_add(1).expect("path offset")..).expect("path name");
            assert!(!name.is_empty() && !self.inode(parent).entries.contains_key(name), "lay a free name");
            let node = match &item.made {
                Made::File(bytes) => self.add(parent, Kind::File, item.mode, item.ids, bytes),
                Made::Directory => self.add(parent, Kind::Directory, item.mode, item.ids, &[]),
                Made::Link(bytes) => self.add(parent, Kind::Link, item.mode, item.ids, bytes),
                Made::HardLink(path) => {
                    let node = self.laid(path).expect("hard link target was laid");
                    assert_eq!(self.inode(node).facts.kind, Kind::File, "hard link names a file");
                    self.inode_mut(node).facts.links =
                        self.inode(node).facts.links.checked_add(1).expect("link count fits");
                    self.change(node);
                    node
                }
            };
            self.inode_mut(parent).entries.insert(name.into(), node);
            if self.inode(node).facts.kind == Kind::Directory {
                self.inode_mut(parent).facts.links =
                    self.inode(parent).facts.links.checked_add(1).expect("directory link count fits");
            }
            self.change(parent);
            laid.push(node);
        }
        laid.into_boxed_slice()
    }
    fn access(&self, pid: Pid, node: Node, bits: u32) -> Result<(), Refusal> {
        let ids = self.process(pid).ids;
        let facts = self.inode(node).facts;
        let shift = if ids.user == facts.user {
            6
        } else if ids.group == facts.group {
            3
        } else {
            0
        };
        if facts.mode.wrapping_shr(shift) & bits == bits { Ok(()) } else { Err(Refusal::Permission) }
    }
    fn resolve(&self, pid: Pid, at: At, path: &[u8], how: How) -> Result<Found, Refusal> {
        if path.is_empty() {
            return Err(Refusal::NotFound);
        }
        if path.len() >= 4096 {
            return Err(Refusal::NameTooLong);
        }
        let process = self.process(pid);
        let (root, mut dir, beneath) = match at {
            At::Root => (process.root, process.root, false),
            At::Cwd => (process.root, process.cwd, false),
            At::Beneath(handle) => {
                let node = self.handle(pid, handle).node;
                (node, node, true)
            }
        };
        if path.first() == Some(&b'/') {
            if beneath {
                return Err(Refusal::Escape);
            }
            dir = root;
        }
        let mut todo: VecDeque<Vec<u8>> =
            path.split(|b| *b == b'/').filter(|n| !n.is_empty()).map(<[u8]>::to_vec).collect();
        let mut links = 0_u32;
        // Each pass consumes a component; at most forty link expansions add components.
        while let Some(name) = todo.pop_front() {
            if self.inode(dir).facts.kind != Kind::Directory {
                return Err(Refusal::NotADirectory);
            }
            self.access(pid, dir, 1)?;
            if name.len() > 255 {
                return Err(Refusal::NameTooLong);
            }
            if name == b"." {
                continue;
            }
            if name == b".." {
                if dir == root {
                    if beneath {
                        return Err(Refusal::Escape);
                    }
                } else {
                    dir = self.inode(dir).parent;
                }
                continue;
            }
            let found = self.inode(dir).entries.get(name.as_slice()).copied();
            let Some(node) = found else {
                if todo.is_empty() && path.last() != Some(&b'/') {
                    return Ok(Found { parent: dir, name: name.into(), node: None });
                }
                return Err(Refusal::NotFound);
            };
            if self.inode(node).facts.kind == Kind::Link {
                let last = todo.is_empty() && path.last() != Some(&b'/');
                if last && matches!(how, How::PathNoFollow | How::Create { .. } | How::CreateNoFollow { .. }) {
                    return Ok(Found { parent: dir, name: name.into(), node: Some(node) });
                }
                if matches!(
                    how,
                    How::PathNoFollow
                        | How::ReadNoFollow
                        | How::DirectoryNoFollow
                        | How::CreateNoFollow { .. }
                        | How::WriteNoFollow
                ) {
                    return Err(Refusal::Loop);
                }
                links = links.checked_add(1).expect("link count fits");
                if links > 40 {
                    return Err(Refusal::Loop);
                }
                let target = &self.inode(node).bytes;
                if target.is_empty() {
                    return Err(Refusal::NotFound);
                }
                if target.first() == Some(&b'/') {
                    if beneath {
                        return Err(Refusal::Escape);
                    }
                    dir = root;
                }
                for part in target.split(|b| *b == b'/').filter(|n| !n.is_empty()).rev() {
                    todo.push_front(part.to_vec());
                }
                continue;
            }
            if todo.is_empty() && path.last() != Some(&b'/') {
                return Ok(Found { parent: dir, name: name.into(), node: Some(node) });
            }
            dir = node;
        }
        if self.inode(dir).facts.kind != Kind::Directory {
            return Err(Refusal::NotADirectory);
        }
        Ok(Found { parent: dir, name: Box::from(&b"."[..]), node: Some(dir) })
    }
    fn open(&mut self, pid: Pid, at: At, path: &[u8], how: How) -> Result<Reply, Refusal> {
        let found = self.resolve(pid, at, path, how)?;
        let node = match how {
            How::Create { mode } | How::CreateNoFollow { mode } => {
                if found.node.is_some() {
                    return Err(Refusal::Exists);
                }
                self.access(pid, found.parent, 3)?;
                let process = self.process(pid);
                let mode = mode.unwrap_or(0o666) & !process.umask & 0o777;
                let node = self.add(found.parent, Kind::File, mode, process.ids, &[]);
                self.inode_mut(found.parent).entries.insert(found.name, node);
                self.change(found.parent);
                node
            }
            How::PathNoFollow => found.node.ok_or(Refusal::NotFound)?,
            How::Directory
            | How::DirectoryNoFollow
            | How::Read
            | How::ReadNoFollow
            | How::Write
            | How::WriteNoFollow => {
                let node = found.node.ok_or(Refusal::NotFound)?;
                let kind = self.inode(node).facts.kind;
                if matches!(how, How::Directory | How::DirectoryNoFollow) && kind != Kind::Directory {
                    return Err(Refusal::NotADirectory);
                }
                if matches!(how, How::Write | How::WriteNoFollow) && kind == Kind::Directory {
                    return Err(Refusal::IsADirectory);
                }
                self.access(pid, node, if matches!(how, How::Write | How::WriteNoFollow) { 6 } else { 4 })?;
                node
            }
        };
        Ok(Reply::Opened(self.opened(pid, node, how)))
    }
    fn writable(&self, pid: Pid, file: Handle) -> Node {
        let open = self.handle(pid, file);
        assert!(
            matches!(open.how, How::Create { .. } | How::CreateNoFollow { .. } | How::Write | How::WriteNoFollow),
            "file opened for write"
        );
        open.node
    }
    fn resize(&mut self, node: Node, length: u64) {
        self.room(self.inode(node).facts.size, length);
        self.inode_mut(node).bytes.resize(usize::try_from(length).expect("scenario byte bound fits"), 0);
        self.inode_mut(node).facts.size = length;
        self.change(node);
    }
    fn removable(&self, pid: Pid, found: &Found) -> Result<Node, Refusal> {
        let node = found.node.ok_or(Refusal::NotFound)?;
        self.access(pid, found.parent, 3)?;
        let parent = self.inode(found.parent).facts;
        let user = self.process(pid).ids.user;
        if parent.mode & 0o1000 != 0 && user != parent.user && user != self.inode(node).facts.user {
            return Err(Refusal::NotPermitted);
        }
        if found.name.as_ref() == b"." {
            return Err(Refusal::Invalid);
        }
        Ok(node)
    }
    fn unlink(&mut self, found: &Found, node: Node) {
        self.inode_mut(found.parent).entries.remove(&found.name);
        if self.inode(node).facts.kind == Kind::Directory {
            self.inode_mut(node).facts.links = 0;
            self.inode_mut(found.parent).facts.links =
                self.inode(found.parent).facts.links.checked_sub(1).expect("directory child was linked");
        } else {
            self.inode_mut(node).facts.links = self.inode(node).facts.links.checked_sub(1).expect("a named link");
        }
        self.change(node);
        self.change(found.parent);
        self.reclaim(node);
    }
    fn reclaim(&mut self, node: Node) {
        if self.inode(node).facts.links == 0
            && !self.handles.values().any(|h| h.node == node)
            && !self.processes.values().any(|p| p.cwd == node || p.root == node)
        {
            self.nodes.remove(&node);
        }
    }
    fn rename(&mut self, pid: Pid, from_dir: Handle, from: &[u8], to_dir: Handle, to: &[u8]) -> Result<Reply, Refusal> {
        let source = self.resolve(pid, At::Beneath(from_dir), from, How::PathNoFollow)?;
        let node = source.node.ok_or(Refusal::NotFound)?;
        let target = self.resolve(pid, At::Beneath(to_dir), to, How::PathNoFollow)?;
        self.removable(pid, &source)?;
        self.access(pid, target.parent, 3)?;
        if target.name.as_ref() == b"." {
            return Err(Refusal::Invalid);
        }
        if target.node == Some(node) {
            return Ok(Reply::Done);
        }
        let kind = self.inode(node).facts.kind;
        if kind == Kind::Directory {
            if source.parent != target.parent {
                self.access(pid, node, 2)?;
            }
            let mut ancestor = target.parent;
            for _ in 0..self.sizes.files {
                if ancestor == node {
                    return Err(Refusal::Invalid);
                }
                let parent = self.inode(ancestor).parent;
                if parent == ancestor {
                    break;
                }
                ancestor = parent;
            }
        }
        if let Some(old) = target.node {
            self.removable(pid, &target)?;
            let old_kind = self.inode(old).facts.kind;
            if kind == Kind::Directory && old_kind != Kind::Directory {
                return Err(Refusal::NotADirectory);
            }
            if kind != Kind::Directory && old_kind == Kind::Directory {
                return Err(Refusal::IsADirectory);
            }
            if !self.inode(old).entries.is_empty() {
                return Err(Refusal::NotEmpty);
            }
            self.unlink(&target, old);
        }
        self.inode_mut(source.parent).entries.remove(&source.name);
        self.inode_mut(target.parent).entries.insert(target.name, node);
        if kind == Kind::Directory && source.parent != target.parent {
            self.inode_mut(node).parent = target.parent;
            self.inode_mut(source.parent).facts.links =
                self.inode(source.parent).facts.links.checked_sub(1).expect("directory child was linked");
            self.inode_mut(target.parent).facts.links =
                self.inode(target.parent).facts.links.checked_add(1).expect("directory link count fits");
        }
        self.change(node);
        self.change(source.parent);
        if source.parent != target.parent {
            self.change(target.parent);
        }
        Ok(Reply::Done)
    }
    fn list(&mut self, pid: Pid, dir: Handle, most: u32) -> Result<Reply, Refusal> {
        let open = self.handle(pid, dir);
        assert_ne!(open.how, How::PathNoFollow, "directory opened for listing");
        let node = open.node;
        if self.inode(node).facts.kind != Kind::Directory {
            return Err(Refusal::NotADirectory);
        }
        if open.listed.is_none() {
            let mut entries: Vec<_> = self
                .inode(node)
                .entries
                .iter()
                .map(|(name, node)| (self.inode(*node).facts.kind, name.clone()))
                .collect();
            for i in (1..entries.len()).rev() {
                let bound = u64::try_from(i.checked_add(1).expect("listing bound")).expect("listing length fits");
                let other = usize::try_from(self.rng.below(bound)).expect("listing index fits");
                entries.swap(i, other);
            }
            self.handles.get_mut(&dir).expect("held handle").listed = Some(entries);
        }
        let open = self.handles.get_mut(&dir).expect("held handle");
        let entries = open.listed.as_ref().expect("listing was drawn");
        let end = open.cursor.saturating_add(usize::try_from(most).expect("page bound fits")).min(entries.len());
        let page = entries.get(open.cursor..end).expect("page within listing").into();
        open.cursor = end;
        Ok(Reply::Listed(page))
    }
    pub fn call(&mut self, pid: Pid, op: Op) -> Result<Reply, Refusal> {
        self.process(pid);
        match op {
            Op::Open { at, path, how } => self.open(pid, at, &path, how),
            Op::Read { file, at, most } => {
                let open = self.handle(pid, file);
                assert!(
                    matches!(open.how, How::Read | How::ReadNoFollow | How::Write | How::WriteNoFollow),
                    "file opened for read"
                );
                let inode = self.inode(open.node);
                if inode.facts.kind == Kind::Directory {
                    return Err(Refusal::IsADirectory);
                }
                let start = usize::try_from(at.min(inode.facts.size)).expect("file offset fits");
                let end = start.saturating_add(usize::try_from(most).expect("read bound fits")).min(inode.bytes.len());
                Ok(Reply::Read(inode.bytes.get(start..end).expect("read within file").into()))
            }
            Op::Write { file, at, bytes } => {
                let node = self.writable(pid, file);
                let end =
                    at.checked_add(u64::try_from(bytes.len()).expect("write length fits")).expect("write offset fits");
                self.resize(node, self.inode(node).facts.size.max(end));
                let start = usize::try_from(at).expect("file offset fits");
                let end = usize::try_from(end).expect("file offset fits");
                self.inode_mut(node).bytes.get_mut(start..end).expect("resized file").copy_from_slice(&bytes);
                Ok(Reply::Done)
            }
            Op::Truncate { file, length } => {
                let node = self.writable(pid, file);
                self.resize(node, length);
                Ok(Reply::Done)
            }
            Op::Sync { file } => {
                assert_ne!(self.handle(pid, file).how, How::PathNoFollow, "sync needs an open file");
                Ok(Reply::Done)
            }
            Op::Stat { handle } => Ok(Reply::Stat(self.inode(self.handle(pid, handle).node).facts)),
            Op::Rename { from_dir, from, to_dir, to } => self.rename(pid, from_dir, &from, to_dir, &to),
            Op::Remove { dir, path, directory } => {
                let found = self.resolve(pid, At::Beneath(dir), &path, How::PathNoFollow)?;
                let node = self.removable(pid, &found)?;
                let kind = self.inode(node).facts.kind;
                if directory && kind != Kind::Directory {
                    return Err(Refusal::NotADirectory);
                }
                if !directory && kind == Kind::Directory {
                    return Err(Refusal::IsADirectory);
                }
                if !self.inode(node).entries.is_empty() {
                    return Err(Refusal::NotEmpty);
                }
                self.unlink(&found, node);
                Ok(Reply::Done)
            }
            Op::MakeDirectory { dir, path, mode } => {
                let found = self.resolve(pid, At::Beneath(dir), &path, How::PathNoFollow)?;
                if found.node.is_some() {
                    return Err(Refusal::Exists);
                }
                self.access(pid, found.parent, 3)?;
                let process = self.process(pid);
                let node = self.add(found.parent, Kind::Directory, mode & !process.umask, process.ids, &[]);
                self.inode_mut(found.parent).entries.insert(found.name, node);
                self.inode_mut(found.parent).facts.links =
                    self.inode(found.parent).facts.links.checked_add(1).expect("directory link count fits");
                self.change(found.parent);
                Ok(Reply::Done)
            }
            Op::List { dir, most } => self.list(pid, dir, most),
            Op::Lock { file } => {
                let open = self.handle(pid, file);
                assert_ne!(open.how, How::PathNoFollow, "lock needs an open file or directory");
                if self.handles.iter().any(|(h, other)| *h != file && other.node == open.node && other.locked) {
                    return Err(Refusal::Busy);
                }
                self.handles.get_mut(&file).expect("held handle").locked = true;
                Ok(Reply::Done)
            }
            Op::Close { handle } => {
                let node = self.handle(pid, handle).node;
                self.handles.remove(&handle);
                self.reclaim(node);
                Ok(Reply::Done)
            }
        }
    }
}
