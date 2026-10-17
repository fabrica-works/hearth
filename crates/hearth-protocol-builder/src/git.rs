//! One git child, held until its exit and all its resources close (render.md, 2.2).
use crate::{Asked, git_machine::invocation};
use alloc::boxed::Box;
use skein_io::{
    self as io,
    kernel::{self, Fd, Pipe, Spawn, Way},
};
use skein_lib::{Queue, Token, bytes, stream::Down};
#[derive(Debug)]
pub(crate) struct Child {
    pub owner: Token,
    pub process: Option<Token>,
    pub output: Option<Token>,
    pub error: Option<Token>,
    pub roles: Box<[u8]>,
    pub tail: crate::git_machine::tail::Tail,
    pub output_state: Intake,
    pub error_state: Intake,
    pub command: crate::render::errors::Command,
    pub fault: Option<crate::render::errors::Ended>,
    pub input: Option<Token>,
    pub exit: Option<invocation::Exit>,
    pub failed: bool,
    pub closing: bool,
    pub closed: bool,
    pub closing_resources: Closing,
}
#[derive(Debug)]
pub(crate) struct Closing {
    pub output: bool,
    pub error: Option<()>,
    pub input: bool,
    pub process: bool,
}
impl Child {
    pub fn spawn(
        owner: Token,
        root: Fd,
        command: invocation::Command<'_>,
        limits: &crate::git_machine::limits::Limits,
        below: &mut Queue<Asked>,
    ) -> Self {
        let pipes: Box<[Pipe]> = match command {
            invocation::Command::CatFile { .. } => Box::new([
                Pipe { child: 1, way: Way::Out, parent: None },
                Pipe { child: 0, way: Way::In, parent: None },
                Pipe { child: 2, way: Way::Out, parent: None },
            ]),
            invocation::Command::RevParse { .. }
            | invocation::Command::LsTree { .. }
            | invocation::Command::Diff { .. } => Box::new([
                Pipe { child: 1, way: Way::Out, parent: None },
                Pipe { child: 2, way: Way::Out, parent: None },
            ]),
        };
        let roles: Box<[u8]> = match command {
            invocation::Command::CatFile { .. } => Box::new([1, 0, 2]),
            invocation::Command::RevParse { .. }
            | invocation::Command::LsTree { .. }
            | invocation::Command::Diff { .. } => Box::new([1, 2]),
        };
        let named = match command {
            invocation::Command::RevParse { .. } => crate::render::errors::Command::RevParse,
            invocation::Command::LsTree { .. } => crate::render::errors::Command::LsTree,
            invocation::Command::CatFile { .. } => crate::render::errors::Command::CatFile,
            invocation::Command::Diff { .. } => crate::render::errors::Command::Diff,
        };
        below.push(Asked::Io(io::Request::Spawn {
            owner,
            spawn: Spawn {
                program: bytes::copy_of(invocation::GIT),
                args: invocation::arguments(command),
                env: invocation::environment(),
                root,
                dir: Box::new([]),
                pipes,
            },
        }));
        Self {
            owner,
            process: None,
            output: None,
            error: None,
            roles,
            tail: crate::git_machine::tail::Tail::new(limits),
            output_state: Intake::Waiting,
            error_state: Intake::Demand,
            command: named,
            fault: None,
            input: None,
            exit: None,
            failed: false,
            closing: false,
            closed: false,
            closing_resources: Closing { output: false, error: None, input: false, process: false },
        }
    }
    pub fn spawned(&mut self, child: Token, pipes: &[Token]) {
        self.process = Some(child);
        for (role, &pipe) in self.roles.iter().zip(pipes) {
            match role {
                0 => self.input = Some(pipe),
                1 => self.output = Some(pipe),
                2 => self.error = Some(pipe),
                _ => unreachable!("git pipes"),
            }
        }
    }
    pub fn refused(&mut self) {
        self.failed = true;
        self.error_state = Intake::Ended;
        self.output_state = Intake::Ended;
    }
    pub fn exited(&mut self, exit: kernel::Exit) {
        self.exit = Some(match exit {
            kernel::Exit::Code(code) => invocation::Exit::Code(code),
            kernel::Exit::Signal(signal) => invocation::Exit::Signal(signal),
        });
    }
    pub fn closed(&mut self, owner: Token) {
        if owner == self.owner {
            self.closed = true;
            self.process = None;
        }
        if self.output == Some(owner) {
            self.output = None;
        }
        if self.error == Some(owner) {
            self.error = None;
        }
        if self.input == Some(owner) {
            self.input = None;
        }
    }
    pub fn done(&self) -> bool {
        self.closed && self.output.is_none() && self.input.is_none() && self.error.is_none()
    }
    pub fn stream(&mut self, down: Down, below: &mut Queue<Asked>) {
        if let Down::Demand { read, .. } = &down
            && self.output_state != Intake::Ended
            && self.output_state != Intake::Draining
        {
            self.output_state =
                if *read == skein_lib::stream::Read::Nothing { Intake::Waiting } else { Intake::Reading };
        }
        below.push(Asked::Io(io::Request::Stream { stream: self.output.expect("spawned stdout"), down }));
    }
    pub fn resume(&mut self, below: &mut Queue<Asked>) -> bool {
        if !self.closing {
            return false;
        }
        let entity = if self.output.is_some() && !self.closing_resources.output {
            self.closing_resources.output = true;
            self.output
        } else if self.error.is_some() && self.closing_resources.error.is_none() {
            self.closing_resources.error = Some(());
            self.error
        } else if self.input.is_some() && !self.closing_resources.input {
            self.closing_resources.input = true;
            self.input
        } else if self.process.is_some() && !self.closing_resources.process {
            self.closing_resources.process = true;
            self.process
        } else {
            None
        };
        if let Some(entity) = entity {
            below.push(Asked::Io(io::Request::Close { entity }));
            return true;
        }
        false
    }
    pub fn work_pending(&self) -> bool {
        self.closing
            && ((self.output.is_some() && !self.closing_resources.output)
                || (self.error.is_some() && self.closing_resources.error.is_none())
                || (self.input.is_some() && !self.closing_resources.input)
                || (self.process.is_some() && !self.closing_resources.process))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Intake {
    Waiting,
    Reading,
    Demand,
    Draining,
    Ended,
}
