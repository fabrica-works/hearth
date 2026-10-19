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
pub use hearth_protocol_mmdebstrap::vector::Proxy;
pub use records::form::Form;
mod cache;
mod client;
mod events;
mod git;
mod indices;
mod keep;
mod terminal;
use events::stream_up;
use terminal::listing_failed;
pub mod limits;
mod read;
mod resolve;
mod store;
pub use store::names::shape as store_shape;
mod walk;
use hearth_protocol_tree as tree;
pub mod translate;
use alloc::boxed::Box;
use git::Intake;
use git_machine::{hunks, invocation, listing, oid::Oid, rev_parse};
use limits::Limits;
use render::vocabulary::{Class, Event as RenderEvent};
use skein_io::{self as io, kernel::Fd};
use skein_lib::{
    Env, List, Queue, Token, bytes,
    stream::{Down, Read as StreamRead, Up},
};
mod beneath;
pub use beneath::{Asked, Beneath};
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
        answer: Option<Result<Box<[listing::Entry]>, listing::Error>>,
    },
    Read(Box<read::Read>),
    Hunks {
        request: Token,
        machine: hunks::Diff,
        above: Queue<hunks::Event>,
        below: Queue<Down>,
        answer: Option<Result<hunks::Hunks, ()>>,
    },
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
    progress: client::Client,
    store_path: Box<[u8]>,
    next: u64,
    child: Option<git::Child>,
    mode: Option<Mode>,
    keeping: Option<keep::Keep>,
    resolving: Option<resolve::Resolve>,
    proxy: Option<Proxy>,
    file_read: u32,
    ran: Option<builder::build::Step>,
}
impl Protocol {
    #[must_use]
    pub fn new(limits: &Limits, call: Call, form: Form, roots: Roots, store: &[u8], seed: u64) -> Self {
        Self {
            roots,
            store: store::Store::new(),
            form,
            startup: None,
            rng: skein_lib::Rng::new(seed),
            call: Some(call),
            client: client::Client::new(form, limits.printer),
            progress: client::Client::new(form, limits.printer),
            store_path: bytes::copy_of(store),
            next: 100,
            child: None,
            mode: None,
            keeping: None,
            resolving: None,
            proxy: None,
            file_read: limits.mmdebstrap.chunk.max(1),
            ran: None,
        }
    }
    #[must_use]
    pub fn with_file_read(mut self, max: u32) -> Self {
        assert!(max > 0, "file reads make progress");
        self.file_read = max;
        self
    }
    #[must_use]
    pub fn with_proxy(mut self, proxy: Option<Proxy>) -> Self {
        self.proxy = proxy;
        self
    }
    #[must_use]
    pub fn take_startup(&mut self) -> Option<Box<[u8]>> {
        self.startup.take()
    }
    #[must_use]
    pub fn code(&self) -> Option<u8> {
        if self.progress.bound() && !self.progress.is_empty() {
            return None;
        }
        let code = self.client.code()?;
        Some(if self.progress.code() == Some(75) { 75 } else { code })
    }
    #[must_use]
    pub fn output_bound(&self) -> bool {
        self.client.bound()
    }
    pub fn bind_output(&mut self, output: Token) {
        self.client.bind(output);
    }
    pub fn bind_error(&mut self, output: Token) {
        self.progress.bind(output);
    }
    #[must_use]
    pub fn error_bound(&self) -> bool {
        self.progress.bound()
    }
    #[must_use]
    pub fn takes(&self) -> bool {
        self.client.takes() && self.progress.takes()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.resolving.is_none()
            && self.ran.is_none()
            && self.keeping.is_none()
            && self.mode.is_none()
            && self.child.is_none()
            && self.call.is_none()
            && self.client.is_empty()
            && (!self.progress.bound() || self.progress.is_empty())
    }
    #[must_use]
    pub fn work_pending(&self) -> bool {
        if self.store.ready()
            || self.call.is_some()
            || self.ran.is_some()
            || self.client.work_pending()
            || self.progress.work_pending()
        {
            return true;
        }
        if let Some(resolve) = &self.resolving {
            return resolve.ready();
        }
        if let Some(keep) = &self.keeping {
            return keep.work_pending();
        }
        let Some(child) = &self.child else {
            return false;
        };
        if child.closing {
            return child.work_pending() || child.done();
        }
        if child.failed && child.process.is_none() {
            return child.closed;
        }
        let mode = self.mode.as_ref().expect("child has mode");
        let ready = match mode {
            Mode::Rev { ended, .. } => *ended,
            Mode::Listing { answer, .. } => answer.is_some(),
            Mode::Hunks { answer, .. } => answer.is_some(),
            Mode::Read(read) => read.ready(),
        };
        if child.error.is_some() && child.error_state == Intake::Demand {
            return true;
        }
        if child.failed || ready {
            if let Mode::Read(read) = mode
                && child.failed
                && let Some(pending) = read.cache_work_pending()
            {
                return pending;
            }
            return (child.output.is_some()
                && child.output_state != Intake::Ended
                && child.output_state != Intake::Draining)
                || (child.output_state == Intake::Ended && child.error_state == Intake::Ended && child.exit.is_some());
        }
        match mode {
            Mode::Rev { .. } => false,
            Mode::Listing { above, below, .. } => !above.is_empty() || (child.output.is_some() && !below.is_empty()),
            Mode::Hunks { above, below, .. } => !above.is_empty() || (child.output.is_some() && !below.is_empty()),
            Mode::Read(read) => read.failed() || (child.output.is_some() && read.work_pending()),
        }
    }
    /// Every step is unavailable until the task that runs it (build.md, 4.3).
    fn resume_run(&mut self, above: &mut Queue<Event>) -> bool {
        let Some(step) = self.ran.take() else { return false };
        above.push(Event::Build(builder::build::vocabulary::Event::Unavailable { step }));
        true
    }
    fn spawn(
        &mut self,
        command: invocation::Command<'_>,
        limits: &git_machine::limits::Limits,
        below: &mut Queue<Asked>,
    ) {
        let owner = Token::new(self.next);
        self.next = self.next.checked_add(1).expect("bounded commands in one call");
        self.child = Some(git::Child::spawn(owner, self.roots.store.fd, command, limits, below));
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
        Request::Progress(progress) => protocol.progress.progress(
            &Env { now: env.now, wall: env.wall, limits: env.limits.printer },
            progress,
            &protocol.store_path,
        ),
        Request::Run { step } => {
            assert!(protocol.ran.is_none(), "one step at a time");
            protocol.ran = Some(step);
        }
        Request::Store(request) => protocol.store_request(request, &env.limits, below),
        Request::Unopened { to, why } => {
            assert_eq!(to.into_token(), Token::new(0), "initial call");
            protocol.startup = Some(
                records::form::print(
                    protocol.form,
                    &records::records::Record::Startup(store::startup(why, &protocol.store_path)),
                    None,
                    &env.limits.printer,
                )
                .expect("startup fits printer limit"),
            );
            protocol.client.stop();
            if protocol.progress.bound() {
                protocol.progress.stop();
            }
        }
        Request::Render(request) => {
            assert!(protocol.mode.is_none() && protocol.child.is_none(), "one git request at a time");
            assert!(protocol.keeping.is_none(), "one cache request at a time");
            match request {
                request @ render::vocabulary::Request::Resolve { .. } => protocol.resolve(request, env, below),
                render::vocabulary::Request::Record { .. } | render::vocabulary::Request::Find { .. } => {
                    unreachable!("root routes finds to store")
                }
                request @ render::vocabulary::Request::Keep { .. } => protocol.keep(request, env),
                render::vocabulary::Request::List { request, repository, revision } => {
                    let Ok(revision) = invocation::revision(&revision) else {
                        above.push(Event::Render(RenderEvent::Unresolved { request }));
                        return;
                    };
                    protocol.spawn(
                        invocation::Command::RevParse { repository: &repository, revision: &revision },
                        &env.limits.git,
                        below,
                    );
                    protocol.mode = Some(Mode::Rev {
                        request,
                        repository,
                        revision,
                        stdout: List::with_capacity(rev_parse::MAX_STDOUT.checked_add(1).expect("bounded stdout")),
                        ended: false,
                    });
                }
                render::vocabulary::Request::Hunks { request, before, after } => {
                    protocol.spawn(
                        invocation::Command::Diff { store: b".", before: &before.0, after: &after.0 },
                        &env.limits.git,
                        below,
                    );
                    let mut machine = hunks::Diff::new(&env.limits.git);
                    let mut events = Queue::with_capacity(1);
                    let mut requests = Queue::with_capacity(1);
                    hunks::down(
                        &mut machine,
                        &Env { now: env.now, wall: env.wall, limits: env.limits.git },
                        hunks::Request::Read,
                        &mut events,
                        &mut requests,
                    );
                    protocol.mode =
                        Some(Mode::Hunks { request, machine, above: events, below: requests, answer: None });
                }
                render::vocabulary::Request::Read { request, repository, commit: _, blobs } => {
                    protocol.spawn(invocation::Command::CatFile { repository: &repository }, &env.limits.git, below);
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
        Request::Answer(answer) => protocol.client.answer(
            &Env { now: env.now, wall: env.wall, limits: env.limits.printer },
            answer,
            &protocol.store_path,
        ),
        Request::Reply { to, outcome, diff, left_out } => {
            let env = Env { now: env.now, wall: env.wall, limits: env.limits.printer };
            if protocol.progress.bound() {
                protocol.progress.finish(&env);
            }
            protocol.client.reply(&env, to, outcome, diff, left_out);
        }
    }
}
pub fn up(protocol: &mut Protocol, env: &Env<Limits>, ev: Told, above: &mut Queue<Event>, below: &mut Queue<Asked>) {
    if let Some(resolve) = &mut protocol.resolving {
        let owned = match &ev {
            Told::File(event) => event.owner() == resolve::OWNER,
            Told::Io(event) => !protocol.client.owns(event) && !protocol.progress.owns(event),
        };
        if owned {
            resolve.up(ev, protocol.roots, env, above, below);
            return;
        }
    }
    let event = match ev {
        Told::Io(event) => event,
        Told::File(event) => {
            if event.owner() == store::OWNER {
                protocol.store.up(event, &env.limits, &protocol.roots, read::current(&mut protocol.mode), above, below);
                if !above.is_empty() {
                    if let Some(resolve) = &mut protocol.resolving {
                        resolve.index_settled();
                    }
                    protocol.keeping = None;
                }
                return;
            }
            if let Some(keep) = &mut protocol.keeping {
                keep.up(event);
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
    if protocol.progress.owns(&event) {
        protocol.progress.up(&Env { now: env.now, wall: env.wall, limits: env.limits.printer }, event);
        return;
    }
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
            let Some(up) = stream_up(child, owner, up) else {
                return;
            };
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
                    Mode::Rev { stdout, ended, .. } => revision_stdout(stdout, ended, up, child, below),
                    Mode::Listing { machine, above: events, below: requests, .. } => {
                        let env = Env { now: env.now, wall: env.wall, limits: env.limits.git };
                        listing::up(machine, &env, up, events, requests);
                        if let Some(down) = requests.pop() {
                            child.stream(down, below);
                        }
                    }
                    Mode::Hunks { machine, above: events, below: requests, .. } => {
                        hunks::up(
                            machine,
                            &Env { now: env.now, wall: env.wall, limits: env.limits.git },
                            up,
                            events,
                            requests,
                        );
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
        io::Event::Failed { .. } => child.refused(),
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
fn revision_stdout(stdout: &mut List<u8>, ended: &mut bool, up: Up, child: &mut git::Child, below: &mut Queue<Asked>) {
    match up {
        Up::Bytes(bytes) => {
            for byte in bytes {
                if stdout.room() > 0 {
                    stdout.push(byte).expect("stdout room");
                }
            }
            if stdout.room() == 0 {
                child.failed = true;
            } else {
                child.stream(Down::Demand { read: StreamRead::Arrived { max: stdout.room() }, room: 0 }, below);
            }
        }
        Up::End => {
            *ended = true;
            child.stream(Down::Demand { read: StreamRead::Nothing, room: 0 }, below);
        }
        Up::Failed(_) => child.failed = true,
        Up::Room => unreachable!("read-only stdout"),
    }
}
fn succeeded(mode: &Mode, exit: invocation::Exit) -> bool {
    let command = match mode {
        Mode::Rev { repository, revision, .. } => invocation::Command::RevParse { repository, revision },
        Mode::Listing { commit, .. } => invocation::Command::LsTree { repository: b"", commit: *commit },
        Mode::Read(_) => invocation::Command::CatFile { repository: b"" },
        Mode::Hunks { .. } => invocation::Command::Diff { store: b".", before: &[0; 32], after: &[0; 32] },
    };
    invocation::succeeded(command, exit)
}
pub fn resume(protocol: &mut Protocol, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Queue<Asked>) {
    if let Some(call) = protocol.call.take() {
        above.push(Event::Call { reply_to: skein_lib::ReplyTo::new(Token::new(0)), call });
        return;
    }
    if protocol.progress.work_pending() {
        protocol.progress.resume(&Env { now: env.now, wall: env.wall, limits: env.limits.printer }, below);
        return;
    }
    if protocol.client.work_pending() {
        protocol.client.resume(&Env { now: env.now, wall: env.wall, limits: env.limits.printer }, below);
        return;
    }
    if protocol.resume_run(above) || protocol.resume_store(above) || protocol.resume_resolve(env, above, below) {
        return;
    }
    if let Some(keep) = &mut protocol.keeping {
        keep.resume(env, above, below);
        if keep.failed() {
            protocol.keeping = None;
        }
        return;
    }
    let Some(child) = &mut protocol.child else {
        return;
    };
    let mode = protocol.mode.as_mut().expect("child has mode");
    let ready = match mode {
        Mode::Rev { ended, .. } => *ended,
        Mode::Listing { answer, .. } => {
            if listing_failed(answer.as_ref()) {
                child.failed = true;
            }
            answer.is_some()
        }
        Mode::Hunks { answer, .. } => {
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
    if !child.closing
        && let Some(stream) = child.error
        && child.error_state == Intake::Demand
    {
        child.error_state = Intake::Waiting;
        below.push(Asked::Io(io::Request::Stream {
            stream,
            down: Down::Demand { read: StreamRead::Arrived { max: env.limits.git.chunk.max(1) }, room: 0 },
        }));
        return;
    }
    if (child.failed || ready)
        && child.output_state != Intake::Ended
        && child.output.is_some()
        && child.output_state != Intake::Draining
    {
        let pending = child.output_state == Intake::Reading;
        child.output_state = Intake::Draining;
        if pending {
            return;
        }
        child.stream(Down::Demand { read: StreamRead::Arrived { max: env.limits.git.chunk.max(1) }, room: 0 }, below);
        return;
    }
    if (child.failed || ready)
        && child.output_state == Intake::Ended
        && child.error_state == Intake::Ended
        && child.exit.is_some()
    {
        child.closing = true;
    }
    if child.process.is_none() && child.closed {
        child.closing = true;
    }
    if child.resume(below) {
        return;
    }
    if child.done() {
        finish(protocol, env, above, below);
        return;
    }
    if child.closing || child.failed || ready || child.output.is_none() {
        return;
    }
    resume_mode(mode, child, env, above, below);
}
fn resume_mode(
    mode: &mut Mode,
    child: &mut git::Child,
    env: &Env<Limits>,
    above: &mut Queue<Event>,
    below: &mut Queue<Asked>,
) {
    match mode {
        Mode::Rev { .. } => {}
        Mode::Listing { machine: _, above: events, below: requests, answer, .. } => {
            if let Some(event) = events.pop() {
                *answer = Some(match event {
                    listing::Event::Listed(entries) => Ok(entries),
                    listing::Event::Failed(error) => Err(error),
                    listing::Event::Closed => unreachable!("listing is not closed"),
                });
            } else if let Some(down) = requests.pop() {
                child.stream(down, below);
            }
        }
        Mode::Hunks { above: events, below: requests, answer, .. } => {
            if let Some(event) = events.pop() {
                *answer = Some(match event {
                    hunks::Event::Hunks(hunks) => Ok(hunks),
                    hunks::Event::Failed(_) => Err(()),
                    hunks::Event::Closed => unreachable!("hunks are not closed"),
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
    let failure = failure(&mode, &child, &env.limits.git);
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
                    protocol.spawn(
                        invocation::Command::LsTree { repository: &repository, commit },
                        &env.limits.git,
                        below,
                    );
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
                    Err(_) => None,
                }
            };
            (request, event)
        }
        Mode::Hunks { request, answer, .. } => {
            let event = if child.failed {
                None
            } else {
                match answer.expect("hunks settled") {
                    Ok(hunks) => Some(RenderEvent::Hunks { request, hunks: translate::hunks::hunks(hunks) }),
                    Err(()) => None,
                }
            };
            (request, event)
        }
        Mode::Read(mut read) => {
            let request = read.request();
            if !child.failed
                && let Some(event) = read.take_terminal()
            {
                above.push(event);
                return;
            }
            let event = if child.failed || read.failed() { None } else { Some(read.take_answer()) };
            (request, event)
        }
    };
    above.push(Event::Render(match event {
        Some(event) => event,
        None => {
            let (class, at, failure) = failure;
            RenderEvent::Failed { request, class, at, failure }
        }
    }));
}

fn failure(
    mode: &Mode,
    child: &git::Child,
    limits: &git_machine::limits::Limits,
) -> (Class, Option<Box<[u8]>>, render::vocabulary::Failure) {
    use render::{
        errors::{Ended, Limit, Tool},
        vocabulary::Failure,
    };
    if let Mode::Listing { answer: Some(Err(error)), .. } = mode {
        let refused = match error {
            listing::Error::TooMany { path } => Some((path.clone(), Limit::Entries, limits.entries)),
            listing::Error::PathTooLong { prefix } => Some((prefix.clone(), Limit::Path, limits.path)),
            listing::Error::PathsTooLarge { path } => Some((path.clone(), Limit::Paths, limits.paths_total)),
            listing::Error::Malformed { .. } | listing::Error::Truncated | listing::Error::Stream(_) => None,
        };
        if let Some((path, limit, max)) = refused {
            return (Class::Configuration, Some(path), Failure::TooLarge { limit, max: u64::from(max) });
        }
    }
    let ended = match child.exit {
        Some(invocation::Exit::Code(code)) if !succeeded(mode, invocation::Exit::Code(code)) => Ended::Exit(code),
        Some(invocation::Exit::Signal(signal)) => Ended::Signal(signal),
        Some(invocation::Exit::Code(_)) => match mode {
            Mode::Read(read) => read.ended(),
            Mode::Listing { answer: Some(Err(listing::Error::Stream(_))), .. } => Ended::Stream,
            Mode::Rev { .. } | Mode::Listing { .. } | Mode::Hunks { .. } => match &child.fault {
                Some(ended) => ended.clone(),
                None => Ended::Output,
            },
        },
        None => Ended::Start,
    };
    (Class::Operational, None, Failure::Tool(Tool { command: child.command, ended, tail: child.tail.lines() }))
}
#[cfg(test)]
mod tests;

pub fn fire(protocol: &mut Protocol, env: &Env<Limits>, _above: &mut Queue<Event>, _below: &mut Queue<Asked>) {
    if let Some(resolve) = &mut protocol.resolving {
        resolve.fire(env);
    }
}
#[must_use]
pub fn next_deadline(protocol: &Protocol) -> Option<skein_lib::Time> {
    match &protocol.resolving {
        Some(resolve) => resolve.next_deadline(),
        None => None,
    }
}
