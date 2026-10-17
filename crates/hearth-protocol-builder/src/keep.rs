//! A rendered file's single temporary, handed to the store after sync.
use crate::{Asked, Limits, builder::vocabulary::Event, cache::Cache, render};
use alloc::boxed::Box;
use render::vocabulary::{Class, Digest, Failure};
use skein_io::{file, kernel::Fd};
use skein_lib::{Env, Queue, Rng, Time, Token};
#[derive(Debug)]
pub(super) struct Keep {
    cache: Cache,
    root: Fd,
    request: Token,
    path: Option<Box<[u8]>>,
    sha256: Digest,
    bytes: Option<Box<[u8]>>,
    size: u64,
    began: Time,
    removing: bool,
    failed: bool,
}
impl Keep {
    pub fn new(
        root: Fd,
        rng: &mut Rng,
        request: Token,
        path: Box<[u8]>,
        sha256: Digest,
        bytes: Box<[u8]>,
        began: Time,
    ) -> Self {
        let size = u64::try_from(bytes.len()).expect("rendered file bound");
        Self {
            cache: Cache::new(root, rng),
            root,
            request,
            path: Some(path),
            sha256,
            bytes: Some(bytes),
            size,
            began,
            removing: false,
            failed: false,
        }
    }
    pub fn temporary(&self) -> Box<[u8]> {
        self.cache.name()
    }
    pub fn work_pending(&self) -> bool {
        !self.removing
            && (self.failed
                || self.cache.work_pending()
                || self.cache.readable()
                || (self.cache.done() && self.path.is_some())
                || self.cache.failed())
    }
    pub fn up(&mut self, event: file::Event) {
        if self.removing {
            self.removing = false;
            self.failed = true;
        } else {
            self.cache.up(event);
        }
    }
    pub fn resume(&mut self, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Queue<Asked>) {
        self.cache.written();
        if self.failed {
            above.push(Event::Render(render::vocabulary::Event::Failed {
                request: self.request,
                class: Class::Operational,
                at: None,
                failure: Failure::Store,
            }));
        } else if let Some(request) = self.cache.take_request() {
            below.push(Asked::File(request));
        } else if self.cache.failed() || (self.cache.done() && self.cache.digest() != self.sha256) {
            self.removing = true;
            below.push(Asked::File(file::Request::Remove {
                owner: Token::new(10),
                root: self.root,
                name: self.cache.name(),
                directory: false,
            }));
        } else if self.cache.readable() {
            if let Some(bytes) = self.bytes.take() {
                self.cache.piece(bytes);
            } else {
                self.cache.finish();
            }
        } else if self.cache.done() {
            above.push(Event::Produced {
                request: self.request,
                path: self.path.take().expect("one arrival"),
                arrival: Token::new(10),
                sha256: self.sha256,
                size: self.size,
                took: env.now.saturating_since(self.began),
            });
        }
    }
    pub fn failed(&self) -> bool {
        self.failed
    }
}

impl crate::Protocol {
    pub(super) fn temporary(&mut self) -> Option<Box<[u8]>> {
        if let Some(keep) = &self.keeping {
            return Some(keep.temporary());
        }
        Some(crate::read::current(&mut self.mode)?.temporary())
    }
    pub(super) fn keep(&mut self, request: render::vocabulary::Request, env: &Env<Limits>) {
        let render::vocabulary::Request::Keep { request, path, sha256, bytes } = request else {
            unreachable!("keep request")
        };
        self.keeping = Some(Keep::new(self.roots.cache.fd, &mut self.rng, request, path, sha256, bytes, env.now));
    }
}
