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
    pub input: bool,
    pub process: bool,
}
impl Child {
    pub fn spawn(owner: Token, root: Fd, command: invocation::Command<'_>, below: &mut Queue<Asked>) -> Self {
        let pipes: Box<[Pipe]> = match command {
            invocation::Command::CatFile { .. } => Box::new([
                Pipe { child: 1, way: Way::Out, parent: None },
                Pipe { child: 0, way: Way::In, parent: None },
            ]),
            invocation::Command::RevParse { .. }
            | invocation::Command::LsTree { .. }
            | invocation::Command::Diff { .. } => Box::new([Pipe { child: 1, way: Way::Out, parent: None }]),
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
            input: None,
            exit: None,
            failed: false,
            closing: false,
            closed: false,
            closing_resources: Closing { output: false, input: false, process: false },
        }
    }
    pub fn spawned(&mut self, child: Token, pipes: &[Token]) {
        self.process = Some(child);
        self.output = pipes.first().copied();
        self.input = pipes.get(1).copied();
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
        if self.input == Some(owner) {
            self.input = None;
        }
    }
    pub fn done(&self) -> bool {
        self.closed && self.output.is_none() && self.input.is_none()
    }
    pub fn stream(&self, down: Down, below: &mut Queue<Asked>) {
        below.push(Asked::Io(io::Request::Stream { stream: self.output.expect("spawned stdout"), down }));
    }
    pub fn resume(&mut self, below: &mut Queue<Asked>) -> bool {
        if !self.closing {
            return false;
        }
        let entity = if self.output.is_some() && !self.closing_resources.output {
            self.closing_resources.output = true;
            self.output
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
                || (self.input.is_some() && !self.closing_resources.input)
                || (self.process.is_some() && !self.closing_resources.process))
    }
}
