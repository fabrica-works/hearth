//! The same git face as a child on the fake machine (domain/testing.md, 4.1).
use crate::{
    face::{self, Command, Faults, Place, Read, Run, Want},
    repository::Repositories,
};
use hearth_fake_machine::{
    Machine,
    hearth::Tool,
    linux::{At, Exit, How, Op, Pid, Refusal, Reply, Signal},
};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Progress {
    Working,
    Waiting,
    Exited,
}
#[derive(Debug)]
pub struct Git {
    child: Pid,
    run: Run,
    pending: Option<(bool, Box<[u8]>, usize)>,
    faults: Faults,
    sent: u64,
    exited: bool,
}
impl Git {
    pub fn start(machine: &mut Machine, child: Pid, faults: Faults) -> Self {
        assert_eq!(machine.tool(child), Some(Tool::Git), "a git child");
        let invocation = machine.linux().invocation(child);
        assert_eq!(invocation.program.as_ref(), b"/usr/bin/git", "git's fixed path");
        let command = face::check(&invocation.args, &invocation.env);
        let place = match &command {
            Command::DiffNoIndex { before, after } => {
                Place::Files { before: read(machine, child, before), after: read(machine, child, after) }
            }
            Command::RevParse { git_dir, .. } | Command::LsTree { git_dir, .. } | Command::CatFileBatch { git_dir } => {
                match open(machine, child, git_dir, How::Directory) {
                    Ok(handle) => {
                        machine.call(child, Op::Close { handle }).expect("close git-dir");
                        Place::Repository
                    }
                    Err(_) => Place::NotARepository,
                }
            }
        };
        Self { child, run: Run::new(command, place), pending: None, faults, sent: 0, exited: false }
    }
    fn exit(&mut self, machine: &mut Machine, exit: Exit) -> Progress {
        machine.call(self.child, Op::Exit { exit }).expect("child exits");
        self.exited = true;
        Progress::Exited
    }
    pub fn step(&mut self, machine: &mut Machine, repositories: &Repositories) -> Progress {
        if self.exited {
            return Progress::Exited;
        }
        if machine.linux().pending(self.child) == Some(Signal::Terminate) {
            return self.exit(machine, Exit::Signal(15));
        }
        if let Some((stderr, bytes, offset)) = &mut self.pending {
            let room = if *stderr {
                usize::MAX
            } else {
                self.faults
                    .stall_after
                    .map_or(usize::MAX, |limit| usize::try_from(limit.saturating_sub(self.sent)).unwrap_or(usize::MAX))
            };
            if room == 0 {
                return Progress::Waiting;
            }
            let end = bytes.len().min(offset.saturating_add(room));
            let pipe = machine.linux().descriptor(self.child, if *stderr { 2 } else { 1 });
            match machine.call(self.child, Op::Send { pipe, bytes: bytes[*offset..end].into() }) {
                Ok(Reply::Sent(count)) => {
                    *offset += usize::try_from(count).expect("pipe count fits");
                    if !*stderr {
                        self.sent += u64::from(count);
                    }
                    if *offset == bytes.len() {
                        self.pending = None;
                    }
                    if count == 0 {
                        return Progress::Waiting;
                    }
                }
                Err(Refusal::BrokenPipe) => return self.exit(machine, Exit::Signal(13)),
                answer => panic!("send answers bytes or EPIPE: {answer:?}"),
            }
            return Progress::Working;
        }
        match self.run.next(repositories) {
            Want::Stdout(bytes) => {
                self.pending = Some((false, bytes, 0));
                Progress::Working
            }
            Want::Stderr(bytes) => {
                self.pending = Some((true, bytes, 0));
                Progress::Working
            }
            Want::Exit(code) => self.exit(machine, Exit::Code(code)),
            Want::Input => {
                let pipe = machine.linux().descriptor(self.child, 0);
                match machine.call(self.child, Op::Take { pipe, most: 4096 }).expect("stdin pipe") {
                    Reply::Took(bytes) => {
                        self.run.input(&bytes);
                        Progress::Working
                    }
                    Reply::Ended => {
                        self.run.end();
                        Progress::Working
                    }
                    Reply::Empty => Progress::Waiting,
                    answer @ (Reply::Opened(_)
                    | Reply::Read(_)
                    | Reply::Done
                    | Reply::Stat(_)
                    | Reply::Listed(_)
                    | Reply::Spawned { .. }
                    | Reply::Sent(_)
                    | Reply::Running
                    | Reply::Exited(_)) => panic!("stdin answers bytes or its end: {answer:?}"),
                }
            }
        }
    }
}
fn open(
    machine: &mut Machine,
    child: Pid,
    path: &[u8],
    how: How,
) -> Result<hearth_fake_machine::linux::Handle, Refusal> {
    let at = if path.starts_with(b"/") { At::Root } else { At::Cwd };
    let Reply::Opened(handle) = machine.call(child, Op::Open { at, path: path.into(), how })? else {
        panic!("open answers a descriptor");
    };
    Ok(handle)
}
fn read(machine: &mut Machine, child: Pid, path: &[u8]) -> Read {
    let file = match open(machine, child, path, How::Read) {
        Ok(file) => file,
        Err(Refusal::Permission) => return Read::Denied,
        Err(_) => return Read::Missing,
    };
    let Reply::Stat(facts) = machine.call(child, Op::Stat { handle: file }).expect("stat an open file") else {
        panic!("stat answers facts");
    };
    let mut bytes = Vec::new();
    while (bytes.len() as u64) < facts.size {
        let Reply::Read(part) =
            machine.call(child, Op::Read { file, at: bytes.len() as u64, most: 4096 }).expect("read an open file")
        else {
            panic!("read answers bytes");
        };
        assert!(!part.is_empty(), "a stored file reaches its stated size");
        bytes.extend_from_slice(&part);
    }
    machine.call(child, Op::Close { handle: file }).expect("close file");
    Read::Contents { bytes: bytes.into(), executable: facts.mode & 0o100 != 0 }
}
