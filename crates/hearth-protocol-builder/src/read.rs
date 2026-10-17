//! One cat-file response at a time, then the bounded JSON machine (render.md, 2.2).
use crate::{
    Asked, Limits, builder, config,
    git::Child,
    git_machine::{batch, oid::Oid},
    render::{
        self,
        vocabulary::{Blob, Decoded},
    },
    translate,
};
use alloc::boxed::Box;
use skein_io as io;
use skein_lib::{
    Env, Queue, Token, Writer,
    stream::{self, Down, Held, Up},
};
#[derive(Debug)]
struct Decode {
    kind: render::vocabulary::As,
    held: Held,
    machine: config::json::Decoder,
    above: Queue<config::json::Event>,
    below: Queue<Down>,
}
impl Decode {
    fn new(bytes: Box<[u8]>, kind: render::vocabulary::As, env: &Env<Limits>) -> Self {
        let mut decode = Self {
            kind,
            held: Held::new(bytes),
            machine: config::json::Decoder::new(&env.limits.config),
            above: Queue::with_capacity(1),
            below: Queue::with_capacity(1),
        };
        let env = Env { now: env.now, wall: env.wall, limits: env.limits.config };
        config::json::down(
            &mut decode.machine,
            &env,
            config::json::Request::Decode,
            &mut decode.above,
            &mut decode.below,
        );
        decode
    }
    fn resume(&mut self, env: &Env<Limits>) -> Option<Result<Decoded, ()>> {
        if let Some(event) = self.above.pop() {
            let result = match event {
                config::json::Event::Decoded(value) => match self.kind {
                    render::vocabulary::As::Document => match config::document::document(value) {
                        Ok(document) => Ok(Decoded::Document(translate::documents::document(document))),
                        Err(error) => Err(error),
                    },
                    render::vocabulary::As::Declaration => {
                        match config::declaration::declaration(value, &env.limits.config) {
                            Ok(_) => Ok(Decoded::Declaration),
                            Err(error) => Err(error),
                        }
                    }
                    render::vocabulary::As::Cached => unreachable!("static bytes are never decoded"),
                },
                config::json::Event::Failed(error) => Err(error),
                config::json::Event::Closed => unreachable!("decoder is not closed"),
            };
            return Some(match result {
                Ok(decoded) => Ok(decoded),
                Err(config::Error { at, fault }) => match translate::errors::fault(fault, &env.limits.config) {
                    Some((fault, also)) => Ok(Decoded::Refused { at: translate::documents::position(at), fault, also }),
                    None => Err(()),
                },
            });
        }
        if let Some(Down::Demand { read, room: 0 }) = self.below.pop() {
            let event = self.held.answer(read).expect("JSON has an active read");
            let env = Env { now: env.now, wall: env.wall, limits: env.limits.config };
            config::json::up(&mut self.machine, &env, event, &mut self.above, &mut self.below);
        }
        None
    }
}
#[derive(Debug)]
pub(crate) struct Read {
    root: skein_io::kernel::Fd,
    rng: skein_lib::Rng,
    cache: Option<crate::cache::Cache>,
    request: Token,
    blobs: Box<[Blob]>,
    index: usize,
    sent: usize,
    room: Option<bool>,
    finished_input: bool,
    batch: batch::Batch,
    above: Queue<batch::Event>,
    below: Queue<Down>,
    body: Option<Writer>,
    decode: Option<Decode>,
    decoded: Option<Decoded>,
    failed: bool,
}
impl Read {
    pub fn new(request: Token, blobs: Box<[Blob]>, env: &Env<Limits>, root: skein_io::kernel::Fd, seed: u64) -> Self {
        assert!(!blobs.is_empty(), "Read asks for blobs");
        let mut read = Self {
            root,
            rng: skein_lib::Rng::new(seed),
            cache: None,
            request,
            blobs,
            index: 0,
            sent: 0,
            room: None,
            finished_input: false,
            batch: batch::Batch::START,
            above: Queue::with_capacity(1),
            below: Queue::with_capacity(1),
            body: None,
            decode: None,
            decoded: None,
            failed: false,
        };
        read.start(env);
        read
    }
    fn start(&mut self, env: &Env<Limits>) {
        let blob = self.blobs.get(self.index).expect("current blob");
        match blob.kind {
            render::vocabulary::As::Document | render::vocabulary::As::Declaration => {
                self.body = Some(Writer::new(usize::try_from(blob.size).expect("render checked document bound")));
            }
            render::vocabulary::As::Cached => self.cache = Some(crate::cache::Cache::new(self.root, &mut self.rng)),
        }
        let request =
            batch::Request::Read { oid: Oid::parse(&blob.oid.0).expect("listed oid"), size: blob.size, max: blob.size };
        self.down(env, request);
    }
    fn down(&mut self, env: &Env<Limits>, request: batch::Request) {
        let env = Env { now: env.now, wall: env.wall, limits: env.limits.git };
        batch::down(&mut self.batch, &env, request, &mut self.above, &mut self.below);
    }
    pub fn up(&mut self, env: &Env<Limits>, event: Up) {
        let env = Env { now: env.now, wall: env.wall, limits: env.limits.git };
        batch::up(&mut self.batch, &env, event, &mut self.above, &mut self.below);
    }
    pub fn input_room(&mut self) {
        assert_eq!(self.room, Some(false), "one room demand");
        self.room = Some(true);
    }
    pub fn input(&mut self, child: &Child, below: &mut Queue<Asked>) -> bool {
        let Some(input) = child.input else {
            return false;
        };
        let down = if let Some(blob) = self.blobs.get(self.sent) {
            match self.room {
                None => {
                    self.room = Some(false);
                    Down::Demand {
                        read: stream::Read::Nothing,
                        room: u32::try_from(blob.oid.0.len()).expect("bounded oid").checked_add(1).expect("line fits"),
                    }
                }
                Some(false) => return false,
                Some(true) => {
                    self.room = None;
                    self.sent = self.sent.checked_add(1).expect("bounded blobs");
                    Down::Send(batch::request_line(Oid::parse(&blob.oid.0).expect("listed oid")))
                }
            }
        } else if !self.finished_input {
            self.finished_input = true;
            Down::Finish
        } else {
            return false;
        };
        below.push(Asked::Io(io::Request::Stream { stream: input, down }));
        true
    }
    pub fn work_pending(&self) -> bool {
        if let Some(cache) = &self.cache {
            if cache.work_pending() || cache.done() {
                return true;
            }
            if cache.busy() {
                return false;
            }
        }
        self.decode.is_some()
            || !self.above.is_empty()
            || !self.below.is_empty()
            || self.decoded.is_some()
            || (!self.finished_input && self.room != Some(false))
    }
    pub fn take_request(&mut self) -> Option<Down> {
        if match &self.cache {
            Some(cache) => cache.busy(),
            None => false,
        } {
            return None;
        }
        self.below.pop()
    }
    pub fn close_cache(&mut self, below: &mut Queue<Asked>) -> bool {
        let Some(cache) = &mut self.cache else {
            return true;
        };
        if cache.abort() {
            return true;
        }
        if let Some(request) = cache.take_request() {
            below.push(Asked::File(request));
        }
        false
    }
    pub fn cache_pending(&self) -> bool {
        match &self.cache {
            Some(cache) => !cache.closing() || cache.work_pending() || cache.done() || cache.failed(),
            None => true,
        }
    }
    pub fn failed(&self) -> bool {
        self.failed
            || (match &self.cache {
                Some(cache) => cache.failed(),
                None => false,
            })
    }
    pub fn request(&self) -> Token {
        self.request
    }
    pub fn ready(&self) -> bool {
        (self.decoded.is_some()
            || (match &self.cache {
                Some(cache) => cache.done(),
                None => false,
            }))
            && self.index.checked_add(1) == Some(self.blobs.len())
    }
    pub fn take_answer(&mut self) -> render::vocabulary::Event {
        let blob = self.blobs.get(self.index).expect("current blob");
        if match &self.cache {
            Some(cache) => cache.done(),
            None => false,
        } {
            return render::vocabulary::Event::Cached {
                request: self.request,
                oid: blob.oid.clone(),
                size: blob.size,
                sha256: self.cache.take().expect("cached blob").digest(),
            };
        }
        render::vocabulary::Event::Read {
            request: self.request,
            oid: blob.oid.clone(),
            read: self.decoded.take().expect("decoded blob"),
        }
    }
    pub fn file_up(&mut self, event: skein_io::file::Event) {
        self.cache.as_mut().expect("cache event").up(event);
    }
    pub fn resume(
        &mut self,
        env: &Env<Limits>,
        above: &mut Queue<builder::vocabulary::Event>,
        below: &mut Queue<Asked>,
    ) {
        if let Some(cache) = &mut self.cache {
            if let Some(request) = cache.take_request() {
                below.push(Asked::File(request));
                return;
            }
            if cache.written() {
                self.down(env, batch::Request::Next);
                return;
            }
            if !cache.readable() && !cache.done() {
                return;
            }
        }
        if let Some(decode) = &mut self.decode {
            if let Some(result) = decode.resume(env) {
                self.decode = None;
                match result {
                    Ok(decoded) => self.decoded = Some(decoded),
                    Err(()) => self.failed = true,
                }
            }
        } else if let Some(event) = self.above.pop() {
            match event {
                batch::Event::Piece(bytes) => {
                    if let Some(cache) = &mut self.cache {
                        cache.piece(bytes);
                    } else {
                        self.body.as_mut().expect("current blob buffer").put(&bytes).expect("batch bounded piece");
                        self.down(env, batch::Request::Next);
                    }
                }
                batch::Event::Whole { .. } => {
                    if let Some(cache) = &mut self.cache {
                        cache.finish();
                    } else {
                        self.decode = Some(Decode::new(
                            self.body.take().expect("whole blob").finish(),
                            self.blobs.get(self.index).expect("blob").kind,
                            env,
                        ));
                    }
                }
                batch::Event::Missing { .. } | batch::Event::Failed(_) => self.failed = true,
                batch::Event::Closed => unreachable!("batch is not closed"),
            }
        } else if (self.decoded.is_some()
            || (match &self.cache {
                Some(cache) => cache.done(),
                None => false,
            }))
            && !self.ready()
        {
            above.push(builder::vocabulary::Event::Render(self.take_answer()));
            self.index = self.index.checked_add(1).expect("bounded blobs");
            self.start(env);
        }
    }
}
