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
pub use records::form::Form;
mod cache;
mod client;
mod git;
pub mod limits;
mod read;
mod store;
use hearth_protocol_tree as tree;
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
    File(io::file::Request),
}
#[derive(PartialEq, Eq, Debug)]
pub enum Told {
    Io(io::Event),
    File(io::file::Event),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaxOut {
    pub above: u32,
    pub below: u32,
}
pub const MAX_OUT_UP: MaxOut = MaxOut { above: 1, below: 1 };
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Root {
    pub fd: Fd,
    pub token: Token,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Roots {
    pub store: Root,
    pub cache: Root,
    pub resolutions: Root,
    pub staging: Root,
}
#[derive(Debug)]
pub struct Protocol {
    roots: Roots,
    store: store::Store,
    form: Form,
    startup: Option<Box<[u8]>>,
    rng: skein_lib::Rng,
    call: Option<Call>,
    client: client::Client,
    next: u64,
    child: Option<git::Child>,
    mode: Option<Mode>,
}
impl Protocol {
    #[must_use]
    pub fn new(limits: &Limits, call: Call, form: Form, roots: Roots, seed: u64) -> Self {
        Self {
            roots,
            store: store::Store::new(),
            form,
            startup: None,
            rng: skein_lib::Rng::new(seed),
            call: Some(call),
            client: client::Client::new(form, limits.printer),
            next: 100,
            child: None,
            mode: None,
        }
    }
    #[must_use]
    pub fn take_startup(&mut self) -> Option<Box<[u8]>> {
        self.startup.take()
    }
    #[must_use]
    pub fn code(&self) -> Option<u8> {
        self.client.code()
    }
    #[must_use]
    pub fn output_bound(&self) -> bool {
        self.client.bound()
    }
    pub fn bind_output(&mut self, output: Token) {
        self.client.bind(output);
    }
    #[must_use]
    pub fn takes(&self) -> bool {
        self.client.takes()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mode.is_none() && self.child.is_none() && self.call.is_none() && self.client.is_empty()
    }
    #[must_use]
    pub fn work_pending(&self) -> bool {
        if self.call.is_some() || self.client.work_pending() {
            return true;
        }
        let Some(child) = &self.child else {
            return false;
        };
        if child.closing {
            return child.work_pending() || child.done();
        }
        if child.failed
            && let Some(Mode::Read(read)) = &self.mode
        {
            return read.cache_pending();
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
        self.child = Some(git::Child::spawn(owner, self.roots.store.fd, command, below));
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
        Request::Store(request) => protocol.store.down(request, &protocol.roots, &env.limits, below),
        Request::Unopened { to, why } => {
            assert_eq!(to.into_token(), Token::new(0), "initial call");
            protocol.startup = Some(
                records::form::print(
                    protocol.form,
                    &records::records::Record::Startup(store::startup(why)),
                    None,
                    &env.limits.printer,
                )
                .expect("startup fits printer limit"),
            );
            protocol.client.stop();
        }
        Request::Render(request) => {
            assert!(protocol.mode.is_none() && protocol.child.is_none(), "one git request at a time");
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
                    protocol.mode = Some(Mode::Read(Box::new(read::Read::new(
                        request,
                        blobs,
                        env,
                        protocol.roots.cache.fd,
                        protocol.rng.next_u64(),
                    ))));
                }
            }
        }
        request @ (Request::Answer(_) | Request::Reply { .. }) => {
            protocol.client.down(&Env { now: env.now, wall: env.wall, limits: env.limits.printer }, request);
        }
    }
}
pub fn up(protocol: &mut Protocol, env: &Env<Limits>, ev: Told, above: &mut Queue<Event>, below: &mut Queue<Asked>) {
    let event = match ev {
        Told::Io(event) => event,
        Told::File(event) => {
            if event.owner() == store::OWNER {
                protocol.store.up(event, &env.limits, above, below);
                return;
            }
            let Some(Mode::Read(read)) = &mut protocol.mode else { unreachable!("cache read exists") };
            read.file_up(event);
            return;
        }
    };
    io_up(protocol, env, event, below);
}
fn io_up(protocol: &mut Protocol, env: &Env<Limits>, event: io::Event, below: &mut Queue<Asked>) {
    if protocol.client.owns(&event) {
        protocol.client.up(&Env { now: env.now, wall: env.wall, limits: env.limits.printer }, event);
        return;
    }
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
                    Mode::Listing { machine, above: events, below: requests, .. } => {
                        let env = Env { now: env.now, wall: env.wall, limits: env.limits.git };
                        listing::up(machine, &env, up, events, requests);
                        if let Some(down) = requests.pop() {
                            child.stream(down, below);
                        }
                    }
                    Mode::Read(read) => {
                        read.up(env, up);
                        if let Some(down) = read.take_request() {
                            child.stream(down, below);
                        }
                    }
                }
            }
        }
        io::Event::Exited { owner, exit } => {
            assert_eq!(owner, child.owner, "current child exit");
            child.exited(exit);
            let valid = succeeded(protocol.mode.as_ref().expect("child mode"), child.exit.expect("exit recorded"));
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
fn succeeded(mode: &Mode, exit: invocation::Exit) -> bool {
    let command = match mode {
        Mode::Rev { repository, revision, .. } => invocation::Command::RevParse { repository, revision },
        Mode::Listing { commit, .. } => invocation::Command::LsTree { repository: b"", commit: *commit },
        Mode::Read(_) => invocation::Command::CatFile { repository: b"" },
    };
    invocation::succeeded(command, exit)
}
pub fn resume(protocol: &mut Protocol, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Queue<Asked>) {
    if let Some(call) = protocol.call.take() {
        above.push(Event::Call { reply_to: skein_lib::ReplyTo::new(Token::new(0)), call });
        return;
    }
    if protocol.client.work_pending() {
        protocol.client.resume(&Env { now: env.now, wall: env.wall, limits: env.limits.printer }, below);
        return;
    }
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
    if child.failed
        && let Mode::Read(read) = mode
        && !read.close_cache(below)
    {
        return;
    }
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
                read.resume(env, above, below);
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
