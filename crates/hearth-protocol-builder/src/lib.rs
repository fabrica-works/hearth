//! The root's requests as git processes and decoded documents (render.md, 8.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use builder::{
    render,
    vocabulary::{Call, Event, Request},
};
use hearth_client_protocol_records as records;
pub use hearth_domain_builder as builder;
use hearth_protocol_config as config;
use hearth_protocol_git as git_machine;
mod git;
pub mod limits;
mod read;
pub mod translate;
use alloc::boxed::Box;
use git_machine::{invocation, listing, oid::Oid, rev_parse};
use limits::Limits;
use render::vocabulary::{Class, Event as RenderEvent};
use skein_io::{self as io, kernel::Fd};
use skein_lib::{
    Env, List, Queue, Token, bytes,
    stream::{Down, Read as StreamRead, Up},
};
#[derive(PartialEq, Eq, Debug)]
pub enum Asked {
    Io(io::Request),
}
#[derive(PartialEq, Eq, Debug)]
pub enum Told {
    Io(io::Event),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaxOut {
    pub above: u32,
    pub below: u32,
}
pub const MAX_OUT_UP: MaxOut = MaxOut { above: 0, below: 1 };
pub const MAX_OUT_DOWN: MaxOut = MaxOut { above: 1, below: 1 };
pub const MAX_OUT_RESUME: MaxOut = MaxOut { above: 1, below: 1 };
#[derive(Debug)]
enum Mode {
    Rev {
        request: Token,
        repository: Box<[u8]>,
        revision: invocation::Revision,
        stdout: List<u8>,
        ended: bool,
    },
    Listing {
        request: Token,
        commit: Oid,
        machine: listing::Listing,
        above: Queue<listing::Event>,
        below: Queue<Down>,
        answer: Option<Result<Box<[listing::Entry]>, ()>>,
    },
    Read(Box<read::Read>),
}
#[derive(Debug)]
pub struct Protocol {
    root: Fd,
    next: u64,
    child: Option<git::Child>,
    mode: Option<Mode>,
}
impl Protocol {
    #[must_use]
    pub fn new(_limits: &Limits, _call: Call, _form: records::form::Form, root: Fd) -> Self {
        Self { root, next: 100, child: None, mode: None }
    }
    #[must_use]
    pub const fn code(&self) -> Option<u8> {
        None
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mode.is_none() && self.child.is_none()
    }
    #[must_use]
    pub fn work_pending(&self) -> bool {
        let Some(child) = &self.child else {
            return false;
        };
        if child.closing {
            return child.work_pending() || child.done();
        }
        if child.failed || child.done() {
            return true;
        }
        match &self.mode {
            Some(Mode::Rev { ended, .. }) => *ended && child.exit.is_some(),
            Some(Mode::Listing { above, below, answer, .. }) => {
                !above.is_empty()
                    || (child.output.is_some() && !below.is_empty())
                    || *answer == Some(Err(()))
                    || (answer.is_some() && child.exit.is_some())
            }
            Some(Mode::Read(read)) => {
                read.failed()
                    || (read.ready() && child.exit.is_some())
                    || (!read.ready() && child.output.is_some() && read.work_pending())
            }
            None => false,
        }
    }
    fn spawn(&mut self, command: invocation::Command<'_>, below: &mut Queue<Asked>) {
        let owner = Token::new(self.next);
        self.next = self.next.checked_add(1).expect("bounded commands in one call");
        self.child = Some(git::Child::spawn(owner, self.root, command, below));
    }
}
pub fn down(
    protocol: &mut Protocol,
    env: &Env<Limits>,
    rq: Request,
    above: &mut Queue<Event>,
    below: &mut Queue<Asked>,
) {
    match rq {
        Request::Render(request) => {
            assert!(protocol.is_empty(), "one git request at a time");
            match request {
                render::vocabulary::Request::List { request, repository, revision } => {
                    let Ok(revision) = invocation::revision(&revision) else {
                        above.push(Event::Render(RenderEvent::Unresolved { request }));
                        return;
                    };
                    protocol
                        .spawn(invocation::Command::RevParse { repository: &repository, revision: &revision }, below);
                    protocol.mode = Some(Mode::Rev {
                        request,
                        repository,
                        revision,
                        stdout: List::with_capacity(rev_parse::MAX_STDOUT.checked_add(1).expect("bounded stdout")),
                        ended: false,
                    });
                }
                render::vocabulary::Request::Read { request, repository, commit: _, blobs } => {
                    protocol.spawn(invocation::Command::CatFile { repository: &repository }, below);
                    protocol.mode = Some(Mode::Read(Box::new(read::Read::new(request, blobs, env))));
                }
            }
        }
        Request::Answer(_) | Request::Reply { .. } => unreachable!("printer joins in 11.3"),
    }
}
pub fn up(protocol: &mut Protocol, env: &Env<Limits>, ev: Told, _above: &mut Queue<Event>, below: &mut Queue<Asked>) {
    let Told::Io(event) = ev;
    let child = protocol.child.as_mut().expect("git event for a child");
    match event {
        io::Event::Spawned { owner, child: token, pipes } => {
            assert_eq!(owner, child.owner, "current child owner");
            child.spawned(token, &pipes);
            if let Some(Mode::Rev { .. }) = &protocol.mode {
                child.stream(
                    Down::Demand {
                        read: StreamRead::Arrived {
                            max: rev_parse::MAX_STDOUT.checked_add(1).expect("bounded stdout"),
                        },
                        room: 0,
                    },
                    below,
                );
            }
        }
        io::Event::Stream { owner, up } => {
            if child.closing {
                return;
            }
            if child.input == Some(owner) {
                match up {
                    Up::Room => {
                        if let Some(Mode::Read(read)) = &mut protocol.mode {
                            read.input_room();
                        }
                    }
                    Up::Failed(_) => child.failed = true,
                    Up::Bytes(_) | Up::End => unreachable!("write-only stdin"),
                }
            } else {
                assert_eq!(child.output, Some(owner), "current stdout");
                match protocol.mode.as_mut().expect("child mode") {
                    Mode::Rev { stdout, ended, .. } => match up {
                        Up::Bytes(bytes) => {
                            for byte in bytes {
                                if stdout.room() > 0 {
                                    stdout.push(byte).expect("stdout room");
                                }
                            }
                            if stdout.room() == 0 {
                                child.failed = true;
                            } else {
                                child.stream(
                                    Down::Demand { read: StreamRead::Arrived { max: stdout.room() }, room: 0 },
                                    below,
                                );
                            }
                        }
                        Up::End => {
                            *ended = true;
                            child.stream(Down::Demand { read: StreamRead::Nothing, room: 0 }, below);
                        }
                        Up::Failed(_) => child.failed = true,
                        Up::Room => unreachable!("read-only stdout"),
                    },
                    Mode::Listing { machine, above, below, .. } => {
                        let env = Env { now: env.now, wall: env.wall, limits: env.limits.git };
                        listing::up(machine, &env, up, above, below);
                    }
                    Mode::Read(read) => read.up(env, up),
                }
            }
        }
        io::Event::Exited { owner, exit } => {
            assert_eq!(owner, child.owner, "current child exit");
            child.exited(exit);
            let valid = match &protocol.mode {
                Some(Mode::Rev { repository, revision, .. }) => invocation::succeeded(
                    invocation::Command::RevParse { repository, revision },
                    child.exit.expect("exit recorded"),
                ),
                Some(Mode::Listing { commit, .. }) => invocation::succeeded(
                    invocation::Command::LsTree { repository: b"", commit: *commit },
                    child.exit.expect("exit recorded"),
                ),
                Some(Mode::Read(_)) => invocation::succeeded(
                    invocation::Command::CatFile { repository: b"" },
                    child.exit.expect("exit recorded"),
                ),
                None => false,
            };
            if !valid {
                child.failed = true;
            }
        }
        io::Event::Failed { .. } => child.failed = true,
        io::Event::Closed { owner } => child.closed(owner),
        io::Event::Listening { .. }
        | io::Event::Accepted { .. }
        | io::Event::Connecting { .. }
        | io::Event::Connected { .. }
        | io::Event::Output { .. }
        | io::Event::Usage { .. }
        | io::Event::Window { .. }
        | io::Event::Resized
        | io::Event::Shutdown { .. } => unreachable!("git child events only"),
    }
}
pub fn resume(protocol: &mut Protocol, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Queue<Asked>) {
    let Some(child) = &mut protocol.child else {
        return;
    };
    let mode = protocol.mode.as_mut().expect("child has mode");
    let ready = match mode {
        Mode::Rev { ended, .. } => *ended,
        Mode::Listing { answer, .. } => {
            if *answer == Some(Err(())) {
                child.failed = true;
            }
            answer.is_some()
        }
        Mode::Read(read) => {
            if read.failed() {
                child.failed = true;
            }
            read.ready()
        }
    };
    if child.failed || (ready && child.exit.is_some()) {
        child.closing = true;
    }
    if child.resume(below) {
        return;
    }
    if child.done() {
        finish(protocol, env, above, below);
        return;
    }
    if child.closing || child.output.is_none() {
        return;
    }
    match mode {
        Mode::Rev { .. } => {}
        Mode::Listing { machine: _, above: events, below: requests, answer, .. } => {
            if let Some(event) = events.pop() {
                *answer = Some(match event {
                    listing::Event::Listed(entries) => Ok(entries),
                    listing::Event::Failed(_) => Err(()),
                    listing::Event::Closed => unreachable!("listing is not closed"),
                });
            } else if let Some(down) = requests.pop() {
                child.stream(down, below);
            }
        }
        Mode::Read(read) => {
            if let Some(down) = read.take_request() {
                child.stream(down, below);
            } else if !read.input(child, below) {
                read.resume(env, above);
            }
        }
    }
}
fn finish(protocol: &mut Protocol, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Queue<Asked>) {
    let child = protocol.child.take().expect("closed child");
    let mode = protocol.mode.take().expect("closed mode");
    let (request, event) = match mode {
        Mode::Rev { request, repository, revision: _, stdout, ended: _ } => {
            if !child.failed {
                if let Ok(rev_parse::Resolved::Commit(commit)) =
                    rev_parse::decode(stdout.as_slice(), child.exit.expect("exit observed"))
                {
                    let mut machine = listing::Listing::new(&env.limits.git);
                    let mut events = Queue::with_capacity(1);
                    let mut requests = Queue::with_capacity(1);
                    let git_env = Env { now: env.now, wall: env.wall, limits: env.limits.git };
                    listing::down(&mut machine, &git_env, listing::Request::List, &mut events, &mut requests);
                    protocol.spawn(invocation::Command::LsTree { repository: &repository, commit }, below);
                    protocol.mode =
                        Some(Mode::Listing { request, commit, machine, above: events, below: requests, answer: None });
                    return;
                }
                if rev_parse::decode(stdout.as_slice(), child.exit.expect("exit observed"))
                    == Ok(rev_parse::Resolved::Unresolved)
                {
                    above.push(Event::Render(RenderEvent::Unresolved { request }));
                    return;
                }
            }
            (request, None)
        }
        Mode::Listing { request, commit, answer, .. } => {
            let event = if child.failed {
                None
            } else {
                match answer.expect("listing settled") {
                    Ok(entries) => {
                        let mut translated =
                            List::with_capacity(u32::try_from(entries.len()).expect("bounded entries"));
                        for entry in entries {
                            translated.push(translate::listing::entry(entry)).expect("entry capacity");
                        }
                        Some(RenderEvent::Listed {
                            request,
                            commit: render::revision::Oid(bytes::copy_of(commit.hex())),
                            entries: translated.into_boxed(),
                        })
                    }
                    Err(()) => None,
                }
            };
            (request, event)
        }
        Mode::Read(mut read) => {
            let request = read.request();
            let event = if child.failed || read.failed() { None } else { Some(read.take_answer()) };
            (request, event)
        }
    };
    above.push(Event::Render(match event {
        Some(event) => event,
        None => RenderEvent::Failed { request, class: Class::Operational },
    }));
}
#[cfg(test)]
mod tests;
