//! The builder's two passes and reclaim point (programming-model.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
pub use hearth_domain_builder as domain;
pub use hearth_protocol_builder as protocol;
pub mod limits;
use domain::{
    Builder,
    vocabulary::{Call, Event, Request},
};
pub use limits::{Files, Limits};
use protocol::{Asked, Protocol, Told};
use skein_io::{
    self as io, Io, file,
    file_layer::{self, FileIo},
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{Env, Queue, Time, Wall};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Configuration {
    pub limits: Limits,
    pub uid: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Roots {
    pub store: Fd,
    pub cache: Fd,
    pub resolutions: Fd,
    pub staging: Fd,
}
#[derive(Debug)]
pub struct Service {
    io: Io,
    files: FileIo,
    file_told: Queue<file::Event>,
    io_submissions: Queue<Submit>,
    file_submissions: Queue<Submit>,
    io_env: Env<io::Limits>,
    protocol: Protocol,
    protocol_env: Env<protocol::limits::Limits>,
    domain: Builder,
    domain_env: Env<domain::Limits>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    told: Queue<io::Event>,
    asked: Queue<Asked>,
    calls: Queue<Event>,
    answers: Queue<Request>,
}
impl Service {
    #[must_use]
    pub fn new(configuration: &Configuration, roots: Roots, call: Call, form: protocol::Form, seed: u64) -> Self {
        let limits = &configuration.limits;
        assert!(limits.io.is_usable() && limits.queue >= 3, "usable io and queues holding every MAX_OUT");
        let mut files = FileIo::with_whole_limit(
            limits.files.files,
            limits.files.read,
            limits.files.entries,
            limits.files.whole,
            limits.files.timeout,
        );
        files.seed_randomness(seed);
        let roots = protocol::Roots {
            store: protocol::Root { fd: roots.store, token: files.adopt_root(roots.store).expect("store root slot") },
            cache: protocol::Root { fd: roots.cache, token: files.adopt_root(roots.cache).expect("cache root slot") },
            resolutions: protocol::Root {
                fd: roots.resolutions,
                token: files.adopt_root(roots.resolutions).expect("resolutions root slot"),
            },
            staging: protocol::Root {
                fd: roots.staging,
                token: files.adopt_root(roots.staging).expect("staging root slot"),
            },
        };
        Self {
            io: Io::new(&limits.io),
            files,
            file_told: Queue::with_capacity(limits.queue),
            io_submissions: Queue::with_capacity(limits.queue),
            file_submissions: Queue::with_capacity(limits.queue),
            io_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.io },
            protocol: Protocol::new(&limits.protocol, call, form, roots, seed),
            protocol_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.protocol },
            domain: Builder::new(&limits.domain, configuration.uid),
            domain_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.domain },
            completions: Queue::with_capacity(limits.queue),
            submissions: Queue::with_capacity(limits.queue),
            told: Queue::with_capacity(limits.queue),
            asked: Queue::with_capacity(limits.queue),
            calls: Queue::with_capacity(limits.queue),
            answers: Queue::with_capacity(limits.queue),
        }
    }
    pub fn adopt_output_pipe(&mut self, fd: Fd) -> Result<(), Fd> {
        if self.protocol.output_bound() {
            return Err(fd);
        }
        let output = self.io.adopt_write_pipe(fd, None)?;
        self.protocol.bind_output(output);
        Ok(())
    }
    pub fn adopt_output_file(&mut self, fd: Fd) -> Result<(), Fd> {
        if self.protocol.output_bound() {
            return Err(fd);
        }
        let output = self.io.adopt_append(fd, self.io_env.limits.close_timeout)?;
        self.protocol.bind_output(output);
        Ok(())
    }
    pub fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    pub fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    #[must_use]
    pub fn take_startup(&mut self) -> Option<Box<[u8]>> {
        self.protocol.take_startup()
    }
    #[must_use]
    pub fn code(&self) -> Option<u8> {
        if self.is_empty() { self.protocol.code() } else { None }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.io.is_empty()
            && self.files.takes()
            && self.files.open_files() == 4
            && self.file_told.is_empty()
            && self.file_submissions.is_empty()
            && self.io_submissions.is_empty()
            && self.protocol.is_empty()
            && self.domain.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
            && self.told.is_empty()
            && self.asked.is_empty()
            && self.calls.is_empty()
            && self.answers.is_empty()
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.io.is_ready()
            || self.io.is_due(now)
            || self.files.is_due(now)
            || !self.file_told.is_empty()
            || !self.file_submissions.is_empty()
            || !self.io_submissions.is_empty()
            || self.protocol.work_pending()
            || match domain::next_deadline(&self.domain) {
                Some(at) => at <= now,
                None => false,
            }
            || self.domain.work_pending()
            || !self.completions.is_empty()
            || !self.told.is_empty()
            || !self.asked.is_empty()
            || !self.calls.is_empty()
            || !self.answers.is_empty()
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        [self.io.next_deadline(), self.files.next_deadline(), domain::next_deadline(&self.domain)]
            .into_iter()
            .flatten()
            .min()
    }
}
#[must_use]
pub fn operations(limits: &Limits) -> Option<u32> {
    io::operations(&limits.io)?.checked_add(2)
}
pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    service.io_env.now = now;
    service.io_env.wall = wall;
    service.protocol_env.now = now;
    service.protocol_env.wall = wall;
    service.domain_env.now = now;
    service.domain_env.wall = wall;
    io_up(service);
    protocol_up(service);
    domain_stage(service);
    protocol_down(service);
    io_down(service);
    service.io.reclaim();
    for _ in 0..service.submissions.capacity() {
        if service.submissions.room() == 0 {
            break;
        }
        if let Some(submit) = service.io_submissions.pop() {
            service.submissions.push(tag(submit, 0));
        } else if let Some(submit) = service.file_submissions.pop() {
            service.submissions.push(tag(submit, 1));
        } else {
            break;
        }
    }
}
fn io_up(service: &mut Service) {
    for _ in 0..service.completions.capacity() {
        if !service.io.is_ready() || !io_room(service, io::MAX_OUT_RESUME) {
            break;
        }
        let marks = (service.told.len(), service.io_submissions.len());
        io::resume(&mut service.io, &service.io_env, &mut service.told, &mut service.io_submissions);
        io_within(service, marks, io::MAX_OUT_RESUME);
    }
    if service.io.is_ready() {
        return;
    }
    for _ in 0..service.completions.capacity() {
        if !io_room(service, io::MAX_OUT_UP) || service.file_told.room() < 1 || service.file_submissions.room() < 2 {
            break;
        }
        let Some(complete) = service.completions.pop() else {
            break;
        };
        let mut complete = complete;
        let file = complete.op.raw() & 1 == 1;
        complete.op = skein_lib::Token::new(complete.op.raw() >> 1);
        if file {
            file_layer::up(&mut service.files, complete, &mut service.file_told, &mut service.file_submissions);
            continue;
        }
        let marks = (service.told.len(), service.io_submissions.len());
        io::up(&mut service.io, &service.io_env, complete, &mut service.told, &mut service.io_submissions);
        io_within(service, marks, io::MAX_OUT_UP);
    }
    if service.files.is_due(service.io_env.now) && service.file_submissions.room() >= 2 {
        file_layer::expire(&mut service.files, service.io_env.now, &mut service.file_submissions);
    }
    if !service.completions.is_empty() {
        return;
    }
    for _ in 0..service.io_env.limits.sockets.saturating_mul(2) {
        if !service.io.is_due(service.io_env.now) || !io_room(service, io::MAX_OUT_FIRE) {
            break;
        }
        let marks = (service.told.len(), service.io_submissions.len());
        io::fire(&mut service.io, &service.io_env, &mut service.told, &mut service.io_submissions);
        io_within(service, marks, io::MAX_OUT_FIRE);
    }
}
fn protocol_up(service: &mut Service) {
    for _ in 0..service.told.capacity() {
        if !service.protocol.work_pending() || !protocol_room(service, protocol::MAX_OUT_RESUME) {
            break;
        }
        let marks = (service.calls.len(), service.asked.len());
        protocol::resume(&mut service.protocol, &service.protocol_env, &mut service.calls, &mut service.asked);
        protocol_within(service, marks, protocol::MAX_OUT_RESUME);
    }
    if service.protocol.work_pending() {
        return;
    }
    for _ in 0..service.file_told.capacity() {
        if service.protocol.work_pending() || !protocol_room(service, protocol::MAX_OUT_UP) {
            break;
        }
        let Some(event) = service.file_told.pop() else {
            break;
        };
        protocol::up(
            &mut service.protocol,
            &service.protocol_env,
            Told::File(event),
            &mut service.calls,
            &mut service.asked,
        );
    }
    for _ in 0..service.told.capacity() {
        if service.protocol.work_pending() || !protocol_room(service, protocol::MAX_OUT_UP) {
            break;
        }
        let Some(event) = service.told.pop() else {
            break;
        };
        let marks = (service.calls.len(), service.asked.len());
        protocol::up(
            &mut service.protocol,
            &service.protocol_env,
            Told::Io(event),
            &mut service.calls,
            &mut service.asked,
        );
        protocol_within(service, marks, protocol::MAX_OUT_UP);
    }
}
fn domain_stage(service: &mut Service) {
    for _ in 0..service.calls.capacity() {
        if !service.domain.work_pending() || service.answers.room() < domain::MAX_OUT_RESUME {
            break;
        }
        let before = service.answers.len();
        domain::resume(&mut service.domain, &service.domain_env, &mut service.answers);
        within(before, service.answers.len(), domain::MAX_OUT_RESUME);
    }
    if service.domain.work_pending() {
        return;
    }
    for _ in 0..service.calls.capacity() {
        if service.answers.room() < domain::MAX_OUT {
            break;
        }
        let Some(event) = service.calls.pop() else {
            break;
        };
        let before = service.answers.len();
        domain::step(&mut service.domain, &service.domain_env, event, &mut service.answers);
        within(before, service.answers.len(), domain::MAX_OUT);
    }
    if service.calls.is_empty() && service.answers.room() >= domain::MAX_OUT {
        domain::fire(&mut service.domain, &service.domain_env, &mut service.answers);
    }
}
fn protocol_down(service: &mut Service) {
    for _ in 0..service.answers.capacity() {
        if !service.protocol.takes() || !protocol_room(service, protocol::MAX_OUT_DOWN) {
            break;
        }
        let Some(request) = service.answers.pop() else {
            break;
        };
        let marks = (service.calls.len(), service.asked.len());
        protocol::down(&mut service.protocol, &service.protocol_env, request, &mut service.calls, &mut service.asked);
        protocol_within(service, marks, protocol::MAX_OUT_DOWN);
    }
}
fn io_down(service: &mut Service) {
    for _ in 0..service.asked.capacity() {
        if !service.io.takes()
            || !service.files.takes()
            || service.file_told.room() < 1
            || service.file_submissions.room() < 2
            || !io_room(service, io::MAX_OUT_DOWN)
        {
            break;
        }
        let Some(request) = service.asked.pop() else {
            break;
        };
        let request = match request {
            Asked::Io(request) => request,
            Asked::File(request) => {
                file_layer::down(
                    &mut service.files,
                    service.io_env.now,
                    request,
                    &mut service.file_told,
                    &mut service.file_submissions,
                );
                continue;
            }
        };
        let marks = (service.told.len(), service.io_submissions.len());
        io::down(&mut service.io, &service.io_env, request, &mut service.io_submissions);
        io_within(service, marks, io::MAX_OUT_DOWN);
    }
}
fn io_room(service: &Service, max: io::MaxOut) -> bool {
    service.told.room() >= max.events && service.io_submissions.room() >= max.submissions
}
fn protocol_room(service: &Service, max: protocol::MaxOut) -> bool {
    service.calls.room() >= max.above && service.asked.room() >= max.below
}
fn io_within(service: &Service, (up, down): (u32, u32), max: io::MaxOut) {
    within(up, service.told.len(), max.events);
    within(down, service.io_submissions.len(), max.submissions);
}
fn protocol_within(service: &Service, (up, down): (u32, u32), max: protocol::MaxOut) {
    within(up, service.calls.len(), max.above);
    within(down, service.asked.len(), max.below);
}
fn within(before: u32, after: u32, max: u32) {
    assert!(after.checked_sub(before).expect("steps add to outputs") <= max, "an entry point keeps its MAX_OUT");
}
fn tag(mut submit: Submit, file: u64) -> Submit {
    submit.op = skein_lib::Token::new(
        submit.op.raw().checked_mul(2).expect("operation tag room").checked_add(file).expect("tag fits"),
    );
    if let io::kernel::Op::Cancel { target } = &mut submit.kind {
        *target = skein_lib::Token::new(
            target.raw().checked_mul(2).expect("operation tag room").checked_add(file).expect("tag fits"),
        );
    }
    submit
}
#[cfg(test)]
mod tests;
