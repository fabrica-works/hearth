use super::{
    At, End, Exit, Handle, How, Invocation, Kind, Machine, Node, Op, Pid, Process, Refusal, Reply, Signal, Spawn,
    Target, Way,
};
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
impl Machine {
    pub(super) fn null_descriptors(&mut self, pid: Pid) {
        for (fd, way) in [(0, Way::In), (1, Way::Out), (2, Way::Out)] {
            let handle = self.end(pid, End::Null { way });
            self.processes.get_mut(&pid).expect("process exists").descriptors.insert(fd, handle);
        }
    }
    fn end(&mut self, pid: Pid, end: End) -> Handle {
        let handle = self.opened(pid, Node(0), How::PathNoFollow);
        self.handles.get_mut(&handle).expect("new pipe handle").end = Some(end);
        handle
    }
    pub(super) fn close(&mut self, pid: Pid, handle: Handle) {
        let node = self.handle(pid, handle).node;
        let end = self.handles.remove(&handle).expect("held handle").end;
        match end {
            None => self.reclaim(node),
            Some(End::Pipe { id, .. }) => {
                if !self.handles.values().any(|h| matches!(h.end, Some(End::Pipe { id: other, .. }) if other == id)) {
                    self.pipes.remove(&id);
                }
            }
            Some(End::Null { .. }) => {}
        }
    }
    pub(crate) fn finish(&mut self, pid: Pid, exit: Exit) {
        let handles: Vec<_> = self.handles.iter().filter_map(|(h, open)| (open.pid == pid).then_some(*h)).collect();
        for handle in handles {
            self.close(pid, handle);
        }
        let process = self.processes.get_mut(&pid).expect("a process it started");
        process.exit = Some(exit);
        process.pending = None;
    }
    fn absolute(&self, node: Node) -> Box<[u8]> {
        let mut node = node;
        let mut names = Vec::new();
        for _ in 0..self.sizes.files {
            let parent = self.inode(node).parent;
            if node == parent {
                break;
            }
            let name = self
                .inode(parent)
                .entries
                .iter()
                .find(|(_, child)| **child == node)
                .expect("named working directory")
                .0;
            names.push(name.as_ref());
            node = parent;
        }
        let mut path = Vec::from(b"/".as_slice());
        for name in names.iter().rev() {
            if path.len() > 1 {
                path.push(b'/');
            }
            path.extend_from_slice(name);
        }
        path.into_boxed_slice()
    }
    fn spawn(&mut self, pid: Pid, spawn: Spawn) -> Result<Reply, Refusal> {
        let dir = if spawn.dir.is_empty() { b".".as_slice() } else { &spawn.dir };
        let cwd = self.resolve(pid, At::Beneath(spawn.root), dir, How::Directory)?.node.ok_or(Refusal::NotFound)?;
        if self.inode(cwd).facts.kind != Kind::Directory {
            return Err(Refusal::NotADirectory);
        }
        self.access(pid, cwd, 1)?;
        let program = self.resolve(pid, At::Root, &spawn.program, How::Read)?.node.ok_or(Refusal::NotFound)?;
        if self.inode(program).facts.kind != Kind::File {
            return Err(Refusal::Permission);
        }
        self.access(pid, program, 1)?;
        for (index, pipe) in spawn.pipes.iter().enumerate() {
            if spawn
                .pipes
                .iter()
                .skip(index.checked_add(1).expect("pipe index fits"))
                .any(|other| pipe.child == other.child)
            {
                return Err(Refusal::Invalid);
            }
        }
        assert!(
            self.processes.len() < usize::try_from(self.sizes.processes).expect("process bound fits"),
            "scenario process bound"
        );
        let parent = self.process(pid);
        let mut process = Process::new(parent.ids, cwd, parent.root, parent.umask);
        self.next_pid = self.next_pid.checked_add(1).expect("process identities fit");
        let child = Pid(self.next_pid);
        process.parent = Some(pid);
        process.group = child;
        process.program = program;
        self.processes.insert(child, process);
        self.null_descriptors(child);
        let cwd_handle = self.opened(child, cwd, How::Directory);
        let invocation = Invocation {
            program: spawn.program,
            args: spawn.args,
            env: spawn.env,
            dir: self.absolute(cwd),
            cwd: cwd_handle,
        };
        self.processes.get_mut(&child).expect("new child").invocation = Some(invocation);
        let mut pipes = Vec::new();
        for pipe in &spawn.pipes {
            self.next_pipe = self.next_pipe.checked_add(1).expect("pipe identities fit");
            let id = self.next_pipe;
            self.pipes.insert(id, VecDeque::new());
            let child_end = self.end(child, End::Pipe { id, way: pipe.way });
            let old = self.processes.get_mut(&child).expect("new child").descriptors.insert(pipe.child, child_end);
            if let Some(old) = old {
                self.close(child, old);
            }
            let way = match pipe.way {
                Way::In => Way::Out,
                Way::Out => Way::In,
            };
            pipes.push(self.end(pid, End::Pipe { id, way }));
        }
        let counts = &mut self.processes.get_mut(&pid).expect("parent").counts;
        counts.spawned = counts.spawned.checked_add(1).expect("spawn count fits");
        Ok(Reply::Spawned { child, pipes: pipes.into_boxed_slice() })
    }
    fn send(&mut self, pid: Pid, pipe: Handle, bytes: &[u8]) -> Result<Reply, Refusal> {
        match self.handle(pid, pipe).end.expect("a pipe it issued") {
            End::Null { way } => {
                assert_eq!(way, Way::Out, "a writable descriptor");
                Ok(Reply::Sent(u32::try_from(bytes.len()).expect("send length fits")))
            }
            End::Pipe { id, way } => {
                assert_eq!(way, Way::Out, "a writable pipe end");
                if !self.handles.values().any(|h| h.end == Some(End::Pipe { id, way: Way::In })) {
                    return Err(Refusal::BrokenPipe);
                }
                let pipe = self.pipes.get_mut(&id).expect("live pipe");
                let room = usize::try_from(self.sizes.pipe)
                    .expect("pipe bound fits")
                    .checked_sub(pipe.len())
                    .expect("bounded pipe");
                let taken = room.min(bytes.len());
                pipe.extend(bytes.get(..taken).expect("bytes taken fit"));
                Ok(Reply::Sent(u32::try_from(taken).expect("pipe bound fits")))
            }
        }
    }
    fn take(&mut self, pid: Pid, pipe: Handle, most: u32) -> Reply {
        match self.handle(pid, pipe).end.expect("a pipe it issued") {
            End::Null { way } => {
                assert_eq!(way, Way::In, "a readable descriptor");
                Reply::Ended
            }
            End::Pipe { id, way } => {
                assert_eq!(way, Way::In, "a readable pipe end");
                let writing = self.handles.values().any(|h| h.end == Some(End::Pipe { id, way: Way::Out }));
                let pipe = self.pipes.get_mut(&id).expect("live pipe");
                if pipe.is_empty() {
                    return if writing { Reply::Empty } else { Reply::Ended };
                }
                let taken = usize::try_from(most).expect("take bound fits").min(pipe.len());
                Reply::Took(pipe.drain(..taken).collect())
            }
        }
    }
    pub(super) fn process_call(&mut self, pid: Pid, op: Op) -> Result<Reply, Refusal> {
        match op {
            Op::Spawn(spawn) => self.spawn(pid, spawn),
            Op::Send { pipe, bytes } => self.send(pid, pipe, &bytes),
            Op::Take { pipe, most } => Ok(self.take(pid, pipe, most)),
            Op::Exit { exit } => {
                self.finish(pid, exit);
                Ok(Reply::Done)
            }
            Op::Signal { child, signal, to } => {
                let process = self.processes.get(&child).expect("a process it started");
                assert_eq!(process.parent, Some(pid), "a process it started as its child");
                if process.exit.is_some() {
                    return Err(Refusal::NotFound);
                }
                let group = process.group;
                let pids: Vec<_> = self
                    .processes
                    .iter()
                    .filter_map(|(p, state)| {
                        (state.exit.is_none()
                            && match to {
                                Target::Child => *p == child,
                                Target::Group => state.group == group,
                            })
                        .then_some(*p)
                    })
                    .collect();
                for child in pids {
                    match signal {
                        Signal::Kill => self.finish(child, Exit::Signal(9)),
                        Signal::Terminate => self.processes.get_mut(&child).expect("live child").pending = Some(signal),
                    }
                }
                Ok(Reply::Done)
            }
            Op::Wait { child } => {
                let process = self.processes.get_mut(&child).expect("a process it started");
                assert_eq!(process.parent, Some(pid), "a process it started as its child");
                assert!(!process.waited, "a process it started, reaped only once");
                match process.exit {
                    None => Ok(Reply::Running),
                    Some(exit) => {
                        process.waited = true;
                        Ok(Reply::Exited(exit))
                    }
                }
            }
            Op::Open { .. }
            | Op::Read { .. }
            | Op::Write { .. }
            | Op::Truncate { .. }
            | Op::Sync { .. }
            | Op::Stat { .. }
            | Op::Rename { .. }
            | Op::Remove { .. }
            | Op::MakeDirectory { .. }
            | Op::List { .. }
            | Op::Lock { .. }
            | Op::Close { .. } => unreachable!("file calls use their own half"),
        }
    }
    #[must_use]
    pub fn invocation(&self, pid: Pid) -> &Invocation {
        self.processes.get(&pid).expect("a process it started").invocation.as_ref().expect("a spawned invocation")
    }
    #[must_use]
    pub fn descriptor(&self, pid: Pid, fd: u32) -> Handle {
        *self.process(pid).descriptors.get(&fd).expect("a descriptor it issued")
    }
    #[must_use]
    pub fn pending(&self, pid: Pid) -> Option<Signal> {
        self.processes.get(&pid).expect("a process it started").pending
    }
    #[must_use]
    pub fn program(&self, pid: Pid) -> Node {
        self.processes.get(&pid).expect("a process it started").program
    }
}
