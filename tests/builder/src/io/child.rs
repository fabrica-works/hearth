//! Start and reap the shipped pure tool fakes as machine children.
use super::{Below, Pipe};
use crate::fixtures::{self, Fault};
use hearth_fake_git::{
    face::Faults as GitFaults,
    process::{Git, Progress as GitProgress},
};
use hearth_fake_machine::{
    hearth::Tool,
    linux::{self, Op, Pid, Reply},
};
use hearth_fake_mmdebstrap::process::{Faults as MmFaults, Mmdebstrap, Progress as MmProgress};
use hearth_protocol_builder::Told;
use skein_io::{self as io, kernel};
use skein_lib::Token;

#[derive(Debug)]
enum ToolProcess {
    Git(Git),
    Mmdebstrap(Mmdebstrap),
}
#[derive(Debug)]
pub(super) struct Child {
    pub pid: Pid,
    owner: Token,
    process: ToolProcess,
    killed: bool,
    reaped: bool,
    listing: bool,
}
impl Below {
    pub(super) fn spawn(&mut self, owner: Token, spawn: kernel::Spawn) {
        self.vectors.push(spawn.args.clone());
        let listing = spawn.args.iter().any(|arg| arg.as_ref() == b"ls-tree");
        let ways: Vec<_> = spawn.pipes.iter().map(|pipe| (pipe.child, pipe.way)).collect();
        let result = self.call(Op::Spawn(linux::Spawn {
            program: spawn.program,
            args: spawn.args,
            env: spawn.env,
            root: self.root,
            dir: spawn.dir,
            pipes: spawn
                .pipes
                .iter()
                .map(|pipe| linux::Pipe {
                    child: pipe.child,
                    way: match pipe.way {
                        kernel::Way::In => linux::Way::In,
                        kernel::Way::Out => linux::Way::Out,
                    },
                })
                .collect(),
        }));
        let (pid, handles) = match result {
            Ok(Reply::Spawned { child, pipes }) => (child, pipes),
            Err(linux::Refusal::NotFound | linux::Refusal::Permission) => {
                self.fault_fell = matches!(self.fault, Some(Fault::NotInstalled { .. }));
                self.events.push_back(Told::Io(io::Event::Failed { owner, error: io::Error::Other }));
                self.events.push_back(Told::Io(io::Event::Closed { owner }));
                return;
            }
            reply => panic!("spawn creates a child or refuses its executable: {reply:?}"),
        };
        let process = {
            let mut fake = self.files.shared.borrow_mut();
            let machine = fake.behind.machine_mut();
            match machine.tool(pid).expect("installed tool") {
                Tool::Git => ToolProcess::Git(Git::start(
                    machine,
                    pid,
                    GitFaults {
                        stall_after: match self.fault {
                            Some(Fault::KilledAt { stdout_bytes }) if !self.fault_fell => Some(u64::from(stdout_bytes)),
                            _ => None,
                        },
                    },
                )),
                Tool::Mmdebstrap => ToolProcess::Mmdebstrap(Mmdebstrap::start(
                    machine,
                    pid,
                    MmFaults {
                        stall_after: match self.fault {
                            Some(Fault::Stalled { after }) => Some(after),
                            _ => None,
                        },
                    },
                )),
            }
        };
        self.files.shared.drain_ledger();
        let token = self.name();
        let pipes = handles
            .into_iter()
            .zip(ways)
            .map(|(handle, (fd, way))| {
                let token = self.name();
                if way == kernel::Way::Out {
                    self.opened_pipes += 1;
                }
                self.pipes.insert(token, Pipe::new(handle, pid, fd, way));
                token
            })
            .collect();
        self.children.insert(token, Child { pid, owner, process, killed: false, reaped: false, listing });
        self.spawned += 1;
        self.events.push_back(Told::Io(io::Event::Spawned { owner, child: token, pipes }));
    }
    pub(super) fn step_children(&mut self) {
        let shared = self.files.shared.clone();
        let mut fake = shared.borrow_mut();
        let fixtures::Fake { behind, repositories, .. } = &mut *fake;
        for child in self.children.values_mut() {
            if child.reaped {
                continue;
            }
            let exited = if child.killed {
                true
            } else {
                let (waiting, exited) = match &mut child.process {
                    ToolProcess::Git(git) => {
                        let progress = git.step(behind.machine_mut(), repositories);
                        (progress == GitProgress::Waiting, progress == GitProgress::Exited)
                    }
                    ToolProcess::Mmdebstrap(mmdebstrap) => {
                        let progress = mmdebstrap.step(behind.machine_mut());
                        (progress == MmProgress::Waiting, progress == MmProgress::Exited)
                    }
                };
                if waiting && child.listing {
                    self.blocked_writes += 1;
                }
                exited
            };
            if exited
                && let Reply::Exited(exit) =
                    behind.machine_mut().call(self.pid, Op::Wait { child: child.pid }).expect("wait child")
            {
                child.reaped = true;
                self.reaped += 1;
                self.events.push_back(Told::Io(io::Event::Exited { owner: child.owner, exit: exit_code(exit) }));
            }
        }
        drop(fake);
        shared.drain_ledger();
    }
    pub(super) fn kill(&mut self, pid: Pid) {
        self.call(Op::Signal { child: pid, signal: linux::Signal::Kill, to: linux::Target::Group })
            .expect("kill live child");
        let child = self.children.values_mut().find(|child| child.pid == pid).expect("tracked child");
        child.killed = true;
    }
    pub(super) fn close(&mut self, token: Token) {
        if token == fixtures::PROTOCOL_OUTPUT || token == fixtures::PROTOCOL_ERROR {
            self.events.push_back(Told::Io(io::Event::Closed { owner: token }));
        } else if let Some(child) = self.children.remove(&token) {
            assert!(child.reaped, "the protocol closes its child after reaping it");
            let _ = self.call(Op::Signal { child: child.pid, signal: linux::Signal::Kill, to: linux::Target::Group });
            self.events.push_back(Told::Io(io::Event::Closed { owner: child.owner }));
        } else if let Some(pipe) = self.pipes.remove(&token) {
            assert!(pipe.way == kernel::Way::In || pipe.ended, "every child output read to its end");
            if !pipe.closed {
                self.call(Op::Close { handle: pipe.handle }).expect("close issued pipe");
            }
            self.events.push_back(Told::Io(io::Event::Closed { owner: token }));
        }
    }
}
fn exit_code(exit: linux::Exit) -> kernel::Exit {
    match exit {
        linux::Exit::Code(code) => kernel::Exit::Code(code),
        linux::Exit::Signal(signal) => kernel::Exit::Signal(signal),
    }
}
