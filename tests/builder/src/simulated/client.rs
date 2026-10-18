//! The startup directories belong to the world and close through its kernel.
use crate::{fixtures, hosted::named};
use hearth::service::{Configuration, Roots};
use skein_io::kernel::{Complete, Exit, Fd, Op, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use skein_world::Inherited;
use std::collections::VecDeque;
#[derive(Debug)]
pub struct Client {
    inner: hearth::Client,
    startup: VecDeque<Fd>,
    closing: bool,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Client {
    #[must_use]
    pub fn new(inherited: &Inherited, scenario: &fixtures::Scenario<'_>, seed: u64) -> Self {
        let limits = hearth::limits();
        let mut client = hearth::Client::with_roots(
            Configuration { proxy: None, limits, uid: fixtures::BUILDER.user, store: fixtures::STORE.into() },
            Roots {
                store: named(&inherited.roots, b"store"),
                cache: named(&inherited.roots, b"cache"),
                resolutions: named(&inherited.roots, b"resolutions"),
                staging: named(&inherited.roots, b"staging"),
            },
            scenario.call.clone(),
            scenario.form,
            named(&inherited.appends, b"output"),
            seed,
        );
        client.adopt_error_file(named(&inherited.appends, b"error")).expect("error append slot");
        Self {
            inner: client,
            startup: inherited.roots.iter().map(|(_, fd)| *fd).chain([inherited.signal]).collect(),
            closing: false,
            completions: Queue::with_capacity(limits.queue),
            submissions: Queue::with_capacity(limits.queue),
        }
    }
}
impl Host for Client {
    fn iterate(&mut self, now: Time, wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            if complete.op == fixtures::STARTUP_CLOSE {
                complete.result.expect("close the startup descriptor");
                self.closing = false;
            } else {
                self.inner.completions().push(complete);
            }
        }
        self.inner.iterate(now, wall);
        self.inner.drain();
        while let Some(submit) = self.inner.submissions().pop() {
            self.submissions.push(submit);
        }
        if self.inner.is_empty()
            && !self.closing
            && let Some(fd) = self.startup.pop_front()
        {
            self.closing = true;
            self.submissions.push(Submit { op: fixtures::STARTUP_CLOSE, kind: Op::Close { fd } });
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.inner.work_pending(now) || (self.inner.is_empty() && !self.closing && !self.startup.is_empty())
    }
    fn next_deadline(&self) -> Option<Time> {
        self.inner.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.inner.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.inner.is_empty() && self.startup.is_empty() && !self.closing
    }
    fn exit(&self) -> Option<Exit> {
        if self.is_empty() { self.inner.exit() } else { None }
    }
    fn operations(&self) -> u32 {
        self.inner.operations()
    }
    fn worst_case(&self) -> u64 {
        self.inner.worst_case()
    }
}
