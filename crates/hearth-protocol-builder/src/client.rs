//! The ordered client record stream (client/README.md, 3.2 and 3.3).
use crate::{
    Asked,
    builder::vocabulary::Answer,
    records::{self, printer},
    translate::answers,
};
use alloc::boxed::Box;
use skein_io as io;
use skein_lib::{
    Env, Queue, Token,
    stream::{Down, Fault, Up},
};
#[derive(Debug)]
pub(crate) struct Client {
    printer: printer::Printer,
    events: Queue<printer::Event>,
    requests: Queue<Down>,
    output: Option<Token>,
    active: bool,
    terminal: Option<crate::render::vocabulary::Outcome>,
    refused: bool,
    /// Why a build ended on a step, when it did.
    failed: Option<records::build::Why>,
    finished: bool,
    closing: Option<Token>,
    broken: Option<Fault>,
}
impl Client {
    pub fn new(form: records::form::Form, limits: records::limits::Limits) -> Self {
        Self {
            printer: printer::Printer::new(form, &limits),
            events: Queue::with_capacity(1),
            requests: Queue::with_capacity(2),
            output: None,
            active: false,
            terminal: None,
            refused: false,
            failed: None,
            finished: false,
            closing: None,
            broken: None,
        }
    }
    pub fn bound(&self) -> bool {
        self.output.is_some() || self.closing.is_some()
    }
    pub fn bind(&mut self, output: Token) {
        assert!(self.output.is_none(), "one client output");
        self.output = Some(output);
    }
    pub fn owns(&self, event: &io::Event) -> bool {
        match event {
            io::Event::Stream { owner, .. } | io::Event::Failed { owner, .. } | io::Event::Closed { owner } => {
                self.output == Some(*owner)
            }
            io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Output { .. }
            | io::Event::Usage { .. }
            | io::Event::Window { .. }
            | io::Event::Resized
            | io::Event::Shutdown { .. } => false,
        }
    }
    pub fn up(&mut self, env: &Env<records::limits::Limits>, event: io::Event) {
        match event {
            io::Event::Closed { .. } => self.output = None,
            io::Event::Stream { up: Up::Failed(fault), .. } => self.fail(fault),
            io::Event::Failed { .. } => self.fail(Fault::Other),
            io::Event::Stream { up, .. } => {
                if self.broken.is_none() {
                    printer::up(&mut self.printer, env, up, &mut self.events, &mut self.requests);
                }
            }
            io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Output { .. }
            | io::Event::Usage { .. }
            | io::Event::Window { .. }
            | io::Event::Resized
            | io::Event::Shutdown { .. } => unreachable!("client output events"),
        }
    }
    fn fail(&mut self, fault: Fault) {
        self.broken = Some(fault);
        self.active = false;
        self.finished = true;
        for _ in 0..self.requests.len() {
            drop(self.requests.pop());
        }
        for _ in 0..self.events.len() {
            self.events.pop();
        }
    }
    pub fn takes(&self) -> bool {
        !self.active && self.terminal.is_none() && self.requests.is_empty() && self.events.is_empty()
    }
    /// A notice of standard error's (client/README.md, 3.2).
    pub fn progress(
        &mut self,
        env: &Env<records::limits::Limits>,
        progress: crate::builder::vocabulary::Progress,
        store: &[u8],
    ) {
        assert!(self.takes(), "printer takes one record at a time");
        self.print(env, crate::translate::store::progress(progress, store));
    }
    pub fn answer(&mut self, env: &Env<records::limits::Limits>, answer: Answer, store: &[u8]) {
        assert!(self.takes(), "printer takes one record at a time");
        let record = match answer {
            Answer::Spec(spec) => records::records::Record::Spec(Box::new(answers::spec(spec))),
            Answer::Error(mut error) => {
                if let crate::render::errors::Kind::Store(fault) = &mut error.kind {
                    fault.store = skein_lib::bytes::copy_of(store);
                }
                records::records::Record::Error(crate::translate::errors::error(error))
            }
            Answer::Change(change) => records::records::Record::Change(super::translate::changes::change(change)),
            Answer::Layer { layer, key, state } => records::records::Record::Layer(answers::layer(layer, key, state)),
            Answer::Failed { step, why } => {
                let failed = answers::failed(step, why);
                self.failed = Some(failed.why);
                records::records::Record::Failed(failed)
            }
            Answer::Full { limit, most } => records::records::Record::Error(answers::full(limit, most, store)),
        };
        self.print(env, record);
    }
    pub fn reply(
        &mut self,
        env: &Env<records::limits::Limits>,
        to: skein_lib::ReplyTo,
        outcome: crate::render::vocabulary::Outcome,
        diff: Option<crate::render::diff::Tally>,
        left_out: u32,
    ) {
        assert!(self.takes(), "printer takes one record at a time");
        assert_eq!(to.into_token(), Token::new(0), "the initial call's reply");
        self.terminal = Some(outcome);
        let outcome = self.outcome(outcome);
        let diff = super::translate::changes::optional_tally(diff);
        self.print(env, records::records::Record::Terminal(records::records::Terminal { outcome, diff, left_out }));
    }
    /// The terminal's outcome: a refused record fails the answer, and an
    /// unavailable step makes a complete one unavailable (client/README.md, 3.3).
    fn outcome(&self, outcome: crate::render::vocabulary::Outcome) -> records::outcome::Outcome {
        let outcome = answers::outcome(if self.refused {
            outcome.worst(crate::render::vocabulary::Outcome::Failed)
        } else {
            outcome
        });
        match (self.failed, outcome) {
            (Some(records::build::Why::Unavailable), records::outcome::Outcome::Done) => {
                records::outcome::Outcome::Unavailable
            }
            (Some(records::build::Why::Unavailable) | None, _) => outcome,
        }
    }
    fn print(&mut self, env: &Env<records::limits::Limits>, record: records::records::Record) {
        if self.broken.is_none() {
            self.active = true;
            printer::down(
                &mut self.printer,
                env,
                printer::Request::Print(record),
                &mut self.events,
                &mut self.requests,
            );
        }
    }
    pub fn finish(&mut self, env: &Env<records::limits::Limits>) {
        self.terminal = Some(crate::render::vocabulary::Outcome::Done);
        if self.broken.is_none() {
            printer::down(&mut self.printer, env, printer::Request::Finish, &mut self.events, &mut self.requests);
        }
    }
    pub fn stop(&mut self) {
        self.terminal = Some(crate::render::vocabulary::Outcome::TryAgain);
        self.finished = true;
        self.requests.push(Down::Finish);
    }
    pub fn work_pending(&self) -> bool {
        !self.requests.is_empty()
            || !self.events.is_empty()
            || (self.finished && self.output.is_some() && self.closing.is_none())
    }
    pub fn is_empty(&self) -> bool {
        self.finished && self.output.is_none() && self.terminal.is_some()
    }
    pub fn code(&self) -> Option<u8> {
        if !self.is_empty() {
            return None;
        }
        if self.broken.is_some() {
            return Some(75);
        }
        Some(records::outcome::code(self.outcome(self.terminal.expect("terminal outcome"))))
    }
    pub fn resume(&mut self, env: &Env<records::limits::Limits>, below: &mut Queue<Asked>) {
        if let Some(down) = self.requests.pop() {
            if down == Down::Finish {
                self.finished = true;
            }
            below.push(Asked::Io(io::Request::Stream { stream: self.output.expect("bound output"), down }));
            return;
        }
        if let Some(event) = self.events.pop() {
            match event {
                printer::Event::Printed => {}
                printer::Event::Refused(_) => self.refused = true,
                printer::Event::Failed(fault) => {
                    self.fail(fault);
                    return;
                }
                printer::Event::Closed => unreachable!("printer closes with its output"),
            }
            self.active = false;
            if self.terminal.is_some() {
                printer::down(&mut self.printer, env, printer::Request::Finish, &mut self.events, &mut self.requests);
            }
        } else if self.finished
            && self.closing.is_none()
            && let Some(entity) = self.output
        {
            self.closing = Some(entity);
            below.push(Asked::Io(io::Request::Close { entity }));
        }
    }
}
