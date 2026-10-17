use super::{At, Counts, Entry, Handle, How, Machine, Node, Op, Pid, Refusal, Reply};
use alloc::collections::{BTreeMap, vec_deque::Drain};
impl Machine {
    pub fn call(&mut self, pid: Pid, op: Op) -> Result<Reply, Refusal> {
        let ids = self.process(pid).ids;
        assert!(
            self.ledger.len() < usize::try_from(self.sizes.ledger).expect("ledger bound fits"),
            "bounded queue the referee drains: ledger"
        );
        let before: BTreeMap<_, _> = self.nodes.iter().map(|(node, inode)| (*node, inode.facts.changes)).collect();
        let mut node = self.touched(pid, &op);
        let reading = matches!(op, Op::Read { .. });
        let writing = matches!(op, Op::Write { .. });
        let length = match &op {
            Op::Write { bytes, .. } => u64::try_from(bytes.len()).expect("write length fits"),
            Op::Open { .. }
            | Op::Read { .. }
            | Op::Truncate { .. }
            | Op::Sync { .. }
            | Op::Stat { .. }
            | Op::Rename { .. }
            | Op::Remove { .. }
            | Op::MakeDirectory { .. }
            | Op::List { .. }
            | Op::Lock { .. }
            | Op::Close { .. }
            | Op::Spawn(_)
            | Op::Send { .. }
            | Op::Take { .. }
            | Op::Signal { .. }
            | Op::Wait { .. }
            | Op::Exit { .. } => 0,
        };
        let result = self.operate(pid, op);
        if let Ok(Reply::Opened(handle)) = &result {
            node = Some(self.handles.get(handle).expect("new handle").node);
        }
        if let Ok(Reply::Spawned { child, .. }) = &result {
            node = Some(self.program(*child));
        }
        let bytes = match &result {
            Ok(Reply::Read(bytes) | Reply::Took(bytes)) => u64::try_from(bytes.len()).expect("read length fits"),
            Ok(Reply::Sent(length)) => u64::from(*length),
            Ok(Reply::Done) if writing => length,
            _ => 0,
        };
        let process = self.processes.get_mut(&pid).expect("process retained after exit");
        if reading {
            process.counts.read = process.counts.read.checked_add(bytes).expect("read count fits");
        }
        if writing {
            process.counts.written = process.counts.written.checked_add(bytes).expect("write count fits");
        }
        for (node, inode) in &self.nodes {
            if before.get(node) != Some(&inode.facts.changes) {
                process.changed.insert(*node);
            }
        }
        for node in before.keys() {
            if !self.nodes.contains_key(node) {
                process.changed.insert(*node);
            }
        }
        process.counts.changed = u64::try_from(process.changed.len()).expect("changed count fits");
        self.ledger.push_back(Entry { pid, ids, node, bytes, refused: result.as_ref().err().copied() });
        result
    }
    fn touched(&self, pid: Pid, op: &Op) -> Option<Node> {
        let handle: Option<Handle> = match op {
            Op::Read { file, .. }
            | Op::Write { file, .. }
            | Op::Truncate { file, .. }
            | Op::Sync { file }
            | Op::Lock { file } => Some(*file),
            Op::Stat { handle } | Op::Close { handle } => Some(*handle),
            Op::List { dir, .. } => Some(*dir),
            Op::Rename { from_dir, from, .. } => {
                return self
                    .resolve(pid, At::Beneath(*from_dir), from, How::PathNoFollow)
                    .ok()
                    .and_then(|found| found.node);
            }
            Op::Remove { dir, path, .. } | Op::MakeDirectory { dir, path, .. } => {
                return self.resolve(pid, At::Beneath(*dir), path, How::PathNoFollow).ok().and_then(|found| found.node);
            }
            Op::Open { .. }
            | Op::Spawn(_)
            | Op::Send { .. }
            | Op::Take { .. }
            | Op::Signal { .. }
            | Op::Wait { .. }
            | Op::Exit { .. } => None,
        };
        handle.map(|handle| self.handle(pid, handle).node)
    }
    pub fn ledger(&mut self) -> Drain<'_, Entry> {
        self.ledger.drain(..)
    }
    #[must_use]
    pub fn counts(&self, pid: Pid) -> Counts {
        self.processes.get(&pid).expect("a process it started").counts
    }
}
