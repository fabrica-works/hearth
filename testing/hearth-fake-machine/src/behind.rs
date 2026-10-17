//! The simulator seam over the machine's operations (testing.md, 3.1).
use crate::{
    Machine,
    linux::{self, At, How, Ids, Op, Pid, Refusal},
};
use alloc::{collections::BTreeMap, vec::Vec};
use skein_io::kernel::{Error, Kind, OpenHow, Stat};
use skein_sim::{Ask, Handle, Reply};

/// One logged-in process and the handles issued to the simulator.
#[derive(Debug, Hash)]
pub struct Behind {
    machine: Machine,
    pid: Pid,
    ids: Ids,
    handles: BTreeMap<Handle, linux::Handle>,
    next: u64,
}
impl Behind {
    #[must_use]
    pub fn new(mut machine: Machine, ids: Ids) -> Self {
        let pid = machine.linux_mut().login(ids, b"/");
        Self { machine, pid, ids, handles: BTreeMap::new(), next: 0 }
    }
    #[must_use]
    pub const fn machine(&self) -> &Machine {
        &self.machine
    }
    pub const fn machine_mut(&mut self) -> &mut Machine {
        &mut self.machine
    }
    fn handle(&self, handle: Handle) -> linux::Handle {
        *self.handles.get(&handle).expect("a seam handle it issued")
    }
    fn call(&mut self, op: Op) -> Result<linux::Reply, Error> {
        self.machine.call(self.pid, op).map_err(refusal)
    }
    fn issue(&mut self, reply: linux::Reply) -> Handle {
        let linux::Reply::Opened(file) = reply else { unreachable!("Open opens a handle") };
        self.next = self.next.checked_add(1).expect("seam identities fit");
        let handle = Handle::new(self.next);
        self.handles.insert(handle, file);
        handle
    }
    fn open(&mut self, at: At, path: &[u8], how: How) -> Result<Handle, Error> {
        let reply = self.call(Op::Open { at, path: path.into(), how })?;
        Ok(self.issue(reply))
    }
    fn close(&mut self, file: Handle) -> Result<Reply, Error> {
        let handle = self.handles.remove(&file).expect("a seam handle it issued");
        let reply = self.call(Op::Close { handle })?;
        assert_eq!(reply, linux::Reply::Done, "Close releases a handle");
        Ok(Reply::Done)
    }
    fn facts(&mut self, handle: linux::Handle) -> Result<linux::Facts, Error> {
        let linux::Reply::Stat(facts) = self.call(Op::Stat { handle })? else { unreachable!("Stat answers facts") };
        Ok(facts)
    }
    fn spawn(&mut self, root: Handle, program: &[u8], dir: &[u8]) -> Result<Reply, Error> {
        let cwd = self.open(At::Beneath(self.handle(root)), if dir.is_empty() { b"." } else { dir }, How::Directory)?;
        let facts = self.facts(self.handle(cwd))?;
        self.close(cwd)?;
        if !self.executable(facts) {
            return Err(Error::Permission);
        }
        let file = self.open(At::Root, program, How::Read)?;
        let facts = self.facts(self.handle(file))?;
        self.close(file)?;
        if facts.kind != linux::Kind::File || !self.executable(facts) {
            return Err(Error::Permission);
        }
        Ok(Reply::Program(skein_sim::Program::Exit(127)))
    }
    fn executable(&self, facts: linux::Facts) -> bool {
        let mask = if self.ids.user == facts.user {
            0o100
        } else if self.ids.group == facts.group {
            0o010
        } else {
            0o001
        };
        facts.mode & mask != 0
    }
    fn list(&mut self, dir: linux::Handle, most: u32, room: u32) -> Result<Reply, Error> {
        let mut entries = Vec::new();
        let mut remaining = room;
        for _ in 0..most {
            if !entries.is_empty() && remaining < 255 {
                break;
            }
            let linux::Reply::Listed(page) = self.call(Op::List { dir, most: 1 })? else {
                unreachable!("List answers entries")
            };
            let Some((kind, name)) = page.into_iter().next() else {
                break;
            };
            let cost = u32::try_from(name.len()).expect("a name fits").checked_add(1).expect("entry cost fits");
            assert!(cost <= remaining, "the seam supplies room for one longest entry");
            remaining = remaining.checked_sub(cost).expect("an entry fits remaining room");
            entries.push((kind_of(kind), name));
        }
        Ok(Reply::Listed(entries))
    }
}
/// Answer one seam request without requiring a simulator ticket.
pub fn answer(behind: &mut Behind, ask: Ask) -> Result<Reply, Error> {
    let op = match ask {
        Ask::Open { root, path, how } => {
            return behind.open(At::Beneath(behind.handle(root)), &path, open_how(how)).map(Reply::Opened);
        }
        Ask::Close { file } => return behind.close(file),
        Ask::Spawn { root, program, dir, .. } => return behind.spawn(root, &program, &dir),
        Ask::List { dir, most, room } => return behind.list(behind.handle(dir), most, room),
        Ask::Read { file, at, len } => Op::Read { file: behind.handle(file), at, most: len },
        Ask::Write { file, at, bytes } => Op::Write { file: behind.handle(file), at, bytes },
        Ask::Append { file, bytes } => {
            let file = behind.handle(file);
            let at = behind.facts(file)?.size;
            Op::Write { file, at, bytes }
        }
        Ask::Truncate { file, length } => Op::Truncate { file: behind.handle(file), length },
        Ask::Sync { file } => Op::Sync { file: behind.handle(file) },
        Ask::Stat { file } => Op::Stat { handle: behind.handle(file) },
        Ask::Lock { file } => Op::Lock { file: behind.handle(file) },
        Ask::Rename { from_dir, from, to_dir, to } => {
            Op::Rename { from_dir: behind.handle(from_dir), from, to_dir: behind.handle(to_dir), to }
        }
        Ask::Remove { dir, name, directory } => Op::Remove { dir: behind.handle(dir), path: name, directory },
        Ask::MakeDirectory { dir, name, mode } => Op::MakeDirectory { dir: behind.handle(dir), path: name, mode },
    };
    Ok(match behind.call(op)? {
        linux::Reply::Done => Reply::Done,
        linux::Reply::Read(bytes) => Reply::Read(bytes),
        linux::Reply::Stat(facts) => Reply::Stat(Stat {
            kind: kind_of(facts.kind),
            size: facts.size,
            mode: facts.mode,
            owner: facts.user,
            links: facts.links,
        }),
        linux::Reply::Opened(_)
        | linux::Reply::Listed(_)
        | linux::Reply::Spawned { .. }
        | linux::Reply::Sent(_)
        | linux::Reply::Took(_)
        | linux::Reply::Empty
        | linux::Reply::Ended
        | linux::Reply::Running
        | linux::Reply::Exited(_) => unreachable!("file requests answer Done, Read or Stat"),
    })
}
fn open_how(how: OpenHow) -> How {
    match how {
        OpenHow::PathNoFollow => How::PathNoFollow,
        OpenHow::Read => How::Read,
        OpenHow::ReadNoFollow => How::ReadNoFollow,
        OpenHow::Directory => How::Directory,
        OpenHow::DirectoryNoFollow => How::DirectoryNoFollow,
        OpenHow::Create { mode } => How::Create { mode },
        OpenHow::CreateNoFollow { mode } => How::CreateNoFollow { mode },
        OpenHow::Write => How::Write,
        OpenHow::WriteNoFollow => How::WriteNoFollow,
    }
}
fn kind_of(kind: linux::Kind) -> Kind {
    match kind {
        linux::Kind::File => Kind::File,
        linux::Kind::Directory => Kind::Directory,
        linux::Kind::Link => Kind::Symlink,
    }
}
pub(crate) const fn refusal(refusal: Refusal) -> Error {
    match refusal {
        Refusal::NotFound => Error::NotFound,
        Refusal::Exists => Error::Exists,
        Refusal::NotADirectory => Error::NotADirectory,
        Refusal::IsADirectory => Error::IsADirectory,
        Refusal::NotEmpty => Error::NotEmpty,
        Refusal::Permission => Error::Permission,
        Refusal::NotPermitted => Error::Other(1),
        Refusal::Escape => Error::Escape,
        Refusal::Loop => Error::Other(40),
        Refusal::NameTooLong => Error::NameTooLong,
        Refusal::Invalid => Error::InvalidArgument,
        Refusal::BrokenPipe => Error::BrokenPipe,
        Refusal::Busy => Error::Busy,
    }
}
impl skein_world::Machine for Behind {
    fn state_digest(&self) -> Option<[u8; 32]> {
        Some(skein_lib::state_digest(self))
    }
    fn cut(&mut self, cut: skein_world::Cut, held: &[Handle], _seed: u64) {
        assert_eq!(cut, skein_world::Cut::Kill, "power loss belongs to M2");
        for handle in held {
            self.close(*handle).expect("cut closes its held handles");
        }
    }
    fn open_root(&mut self, path: &[u8]) -> Result<Handle, Error> {
        self.open(At::Root, path, How::Directory)
    }
    fn close_root(&mut self, root: Handle) {
        self.close(root).expect("startup rollback closes its root");
    }
    fn open_append(&mut self, root: Handle, path: &[u8], mode: u32) -> Result<Handle, Error> {
        match self.open(At::Beneath(self.handle(root)), path, How::Create { mode: Some(mode) }) {
            Err(Error::Exists) => self.open(At::Beneath(self.handle(root)), path, How::Write),
            result => result,
        }
    }
    fn step(&mut self, call: skein_sim::Call, answers: &mut skein_lib::Queue<skein_sim::Answer>) {
        let result = answer(self, call.ask);
        answers.push(skein_sim::Answer { ticket: call.ticket, result });
    }
}
