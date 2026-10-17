//! The builder's two passes and reclaim point (programming-model.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub use hearth_domain_builder as domain;
pub use hearth_protocol_builder as protocol;
pub mod limits;
use domain::{
    Builder,
    vocabulary::{Call, Event, Request},
};
pub use limits::Limits;
use protocol::{Asked, Protocol, Told};
use skein_io::{
    self as io, Io,
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{Env, Queue, Time, Wall};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Configuration {
    pub limits: Limits,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Roots {
    pub store: Fd,
}
#[derive(Debug)]
pub struct Service {
    io: Io,
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
    pub fn new(configuration: &Configuration, roots: Roots, call: Call, form: protocol::Form) -> Self {
        let limits = &configuration.limits;
        assert!(limits.io.is_usable() && limits.queue >= 3, "usable io and queues holding every MAX_OUT");
        Self {
            io: Io::new(&limits.io),
            io_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.io },
            protocol: Protocol::new(&limits.protocol, call, form, roots.store),
            protocol_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.protocol },
            domain: Builder::new(&limits.domain),
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
    pub fn code(&self) -> Option<u8> {
        if self.is_empty() { self.protocol.code() } else { None }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.io.is_empty()
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
            || self.protocol.work_pending()
            || self.domain.work_pending()
            || !self.completions.is_empty()
            || !self.told.is_empty()
            || !self.asked.is_empty()
            || !self.calls.is_empty()
            || !self.answers.is_empty()
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
}
#[must_use]
pub fn operations(limits: &Limits) -> Option<u32> {
    io::operations(&limits.io)
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
}
fn io_up(service: &mut Service) {
    for _ in 0..service.completions.capacity() {
        if !service.io.is_ready() || !io_room(service, io::MAX_OUT_RESUME) {
            break;
        }
        let marks = (service.told.len(), service.submissions.len());
        io::resume(&mut service.io, &service.io_env, &mut service.told, &mut service.submissions);
        io_within(service, marks, io::MAX_OUT_RESUME);
    }
    if service.io.is_ready() {
        return;
    }
    for _ in 0..service.completions.capacity() {
        if !io_room(service, io::MAX_OUT_UP) {
            break;
        }
        let Some(complete) = service.completions.pop() else {
            break;
        };
        let marks = (service.told.len(), service.submissions.len());
        io::up(&mut service.io, &service.io_env, complete, &mut service.told, &mut service.submissions);
        io_within(service, marks, io::MAX_OUT_UP);
    }
    if !service.completions.is_empty() {
        return;
    }
    for _ in 0..service.io_env.limits.sockets.saturating_mul(2) {
        if !service.io.is_due(service.io_env.now) || !io_room(service, io::MAX_OUT_FIRE) {
            break;
        }
        let marks = (service.told.len(), service.submissions.len());
        io::fire(&mut service.io, &service.io_env, &mut service.told, &mut service.submissions);
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
        if !service.io.takes() || !io_room(service, io::MAX_OUT_DOWN) {
            break;
        }
        let Some(Asked::Io(request)) = service.asked.pop() else {
            break;
        };
        let marks = (service.told.len(), service.submissions.len());
        io::down(&mut service.io, &service.io_env, request, &mut service.submissions);
        io_within(service, marks, io::MAX_OUT_DOWN);
    }
}
fn io_room(service: &Service, max: io::MaxOut) -> bool {
    service.told.room() >= max.events && service.submissions.room() >= max.submissions
}
fn protocol_room(service: &Service, max: protocol::MaxOut) -> bool {
    service.calls.room() >= max.above && service.asked.room() >= max.below
}
fn io_within(service: &Service, (up, down): (u32, u32), max: io::MaxOut) {
    within(up, service.told.len(), max.events);
    within(down, service.submissions.len(), max.submissions);
}
fn protocol_within(service: &Service, (up, down): (u32, u32), max: protocol::MaxOut) {
    within(up, service.calls.len(), max.above);
    within(down, service.asked.len(), max.below);
}
fn within(before: u32, after: u32, max: u32) {
    assert!(after.checked_sub(before).expect("steps add to outputs") <= max, "an entry point keeps its MAX_OUT");
}
#[cfg(test)]
mod tests;
