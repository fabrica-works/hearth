//! Configuration to specs, without parsing or effects (render.md, 8.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
mod call;
pub mod compose;
pub mod diff;
pub mod entries;
pub mod errors;
pub mod evaluate;
pub mod keys;
pub mod kinds;
pub mod limits;
pub mod parts;
pub mod revision;
pub mod schema;
pub mod vocabulary;
use limits::Limits;
use skein_lib::{Env, Queue, Token};
use vocabulary::{Class, Event, Outcome, Output, Request};
pub const MAX_OUT: u32 = 1;
pub const MAX_OUT_RESUME: u32 = 1;
#[derive(Debug)]
pub struct Render {
    call: Option<call::Call>,
}
impl Render {
    #[must_use]
    pub const fn new(_limits: &Limits) -> Render {
        Render { call: None }
    }
    #[must_use]
    pub fn work_pending(&self) -> bool {
        match &self.call {
            Some(call) => call.side.ready,
            None => false,
        }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.call.is_none()
    }
}
pub fn step(render: &mut Render, env: &Env<Limits>, ev: Event, out: &mut Queue<Output>) {
    match ev {
        Event::Render { repository, revision, targets } => {
            assert!(render.call.is_none(), "one render call at a time");
            let request = Request::List { request: Token::new(1), repository: repository.clone(), revision };
            render.call = Some(call::Call::new(repository, targets, &env.limits));
            out.push(Output::Request(request));
        }
        Event::Diff { repository, from, to, targets } => {
            assert!(render.call.is_none(), "one render call at a time");
            let request = Request::List { request: Token::new(1), repository: repository.clone(), revision: from };
            render.call = Some(call::Call::diff(repository, targets, to, &env.limits));
            out.push(Output::Request(request));
        }
        Event::Listed { request, commit, entries } => {
            assert_eq!(request, Token::new(1), "listing token");
            render.call.as_mut().expect("active call").side.listed(env.limits, commit, &entries, out);
        }
        Event::Read { request, oid, read } => {
            assert_eq!(request, Token::new(2), "read token");
            render.call.as_mut().expect("active call").side.decoded(&oid, read);
        }
        Event::Kept { request, sha256 } => {
            assert_eq!(request, Token::new(4), "keep token");
            render.call.as_mut().expect("active call").side.kept(Some(sha256));
        }
        Event::Cached { request, oid, size, sha256 } => {
            assert_eq!(request, Token::new(2), "read token");
            render.call.as_mut().expect("active call").side.cached(&oid, size, sha256);
        }
        Event::Unresolved { request } => {
            assert_eq!(request, Token::new(1), "listing token");
            render.call.as_mut().expect("active call").side.fail(Outcome::Usage);
        }
        Event::Found { request, digest } => {
            assert_eq!(request, Token::new(5), "find token");
            render.call.as_mut().expect("active call").side.found(Some(digest), false);
        }
        Event::Absent { request } => {
            assert_eq!(request, Token::new(5), "find token");
            render.call.as_mut().expect("active call").side.found(None, false);
        }
        Event::Hunks { request, hunks } => {
            assert_eq!(request, Token::new(3), "hunks token");
            render.call.as_mut().expect("active call").hunks(Some(hunks));
        }
        Event::Failed { request, class, at, failure } => {
            if request == Token::new(3) {
                render.call.as_mut().expect("active call").hunk_failure(class, failure);
                return;
            }
            let side = &mut render.call.as_mut().expect("active call").side;
            if request == Token::new(5) {
                side.found(None, true);
                return;
            }
            if request == Token::new(4) {
                side.kept(None);
                return;
            }
            let kind = match failure {
                vocabulary::Failure::Tool(tool) => Some(errors::Kind::Tool(tool)),
                vocabulary::Failure::TooLarge { limit, max } => {
                    Some(errors::Kind::Decode(errors::Decode::TooLarge { limit, max }))
                }
                vocabulary::Failure::Store => None,
            };
            if let Some(kind) = kind {
                let at = failure_place(at);
                errors::push(
                    &mut side.errors,
                    errors::Error { side: None, kind, class, at, also: None, target: None, task: None },
                );
            }
            let outcome = match class {
                Class::Configuration => Outcome::Failed,
                Class::Operational => Outcome::TryAgain,
            };
            render.call.as_mut().expect("active call").side.fail(outcome);
        }
    }
}
pub fn resume(render: &mut Render, env: &Env<Limits>, out: &mut Queue<Output>) {
    if !render.work_pending() {
        return;
    }
    if render.call.as_mut().expect("ready call").resume(env.limits, out) {
        render.call = None;
    }
}
#[cfg(test)]
mod tests;

fn failure_place(path: Option<Box<[u8]>>) -> Option<errors::Place> {
    Some(errors::Place { path: path?, position: None })
}

pub mod exports;
