//! The builder root, render and store driven over their domain faces.
mod decode;
mod resolve;
mod store;
use crate::{
    files::Files,
    fixtures::{BUILDER, Scenario, Shared},
    referee::Referee,
};
use hearth_domain_builder::{
    self as builder,
    vocabulary::{Answer, Call, Event, Progress, Request},
};
use hearth_domain_render::{
    self as render,
    vocabulary::{Event as RenderEvent, Outcome},
};
use skein_lib::{ReplyTo, Rng, Time, Token};
use skein_world::domain::{Schedule, Span, Stage};
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cut {
    BeforeKeep,
}
#[derive(Debug)]
struct Delivery {
    event: Event,
    terminal: Option<u64>,
}
#[derive(Debug)]
pub struct Run {
    pub answers: Vec<Answer>,
    pub outcome: Option<Outcome>,
    pub progress: Vec<Progress>,
    pub shared: Shared,
    pub referee: Referee,
    pub lock_times: Vec<Time>,
    files: Option<Files>,
    builder: builder::Builder,
    stage: Stage<builder::Limits, Event, Request>,
    schedule: Schedule<Delivery>,
    reads: VecDeque<Delivery>,
    requests: Vec<Request>,
    store: store::Store,
    call: Call,
    seed: u64,
    rng: Rng,
    cut: Option<Cut>,
    held: Option<(skein_sim::Handle, Time)>,
}
impl Run {
    #[must_use]
    pub fn new(scenario: &Scenario<'_>, seed: u64) -> Self {
        Self::over(
            scenario.call.clone(),
            Shared::for_scenario(scenario, seed, crate::fixtures::WORLD_PIPE),
            scenario.fault,
            seed,
        )
    }
    fn over(call: Call, shared: Shared, fault: Option<crate::fixtures::Fault>, seed: u64) -> Self {
        let limits = hearth::limits().domain;
        let mut stage = Stage::new(limits, builder::MAX_OUT, 16);
        stage.push(Event::Call { reply_to: ReplyTo::new(Token::new(1)), call: call.clone() });
        Self {
            answers: Vec::new(),
            outcome: None,
            progress: Vec::new(),
            referee: Referee::default(),
            lock_times: Vec::new(),
            files: Some(Files::new(shared.clone(), fault, seed)),
            builder: builder::Builder::new(&limits, BUILDER.user),
            stage,
            schedule: Schedule::new(),
            reads: VecDeque::new(),
            requests: Vec::new(),
            store: store::Store::default(),
            call,
            shared,
            seed,
            rng: Rng::new(seed),
            cut: None,
            held: None,
        }
    }
    pub fn cut_at(&mut self, cut: Cut) {
        self.cut = Some(cut);
    }
    /// Holds a second independently opened lock until this simulated time.
    pub fn hold_lock_until(&mut self, until: Time) {
        use skein_world::Machine;
        let handle = self.shared.open_root(crate::fixtures::STORE).expect("second store handle");
        assert_eq!(self.shared.answer(skein_sim::Ask::Lock { file: handle }), Ok(skein_sim::Reply::Done));
        self.held = Some((handle, until));
    }
    #[must_use]
    pub fn finish(mut self) -> Self {
        for _ in 0..100_000 {
            self.release();
            if let Some(delivery) = self.schedule.next(self.stage.env.now) {
                if let Some(key) = delivery.terminal {
                    self.referee.end(key);
                }
                self.stage.push(delivery.event);
            }
            if let Some(event) = self.stage.next_event() {
                builder::step(&mut self.builder, &self.stage.env, event, &mut self.stage.out);
            } else if self.builder.work_pending() && self.stage.has_room() {
                builder::resume(&mut self.builder, &self.stage.env, &mut self.stage.out);
            } else if builder::next_deadline(&self.builder).is_some_and(|at| at <= self.stage.env.now)
                && self.stage.has_room()
            {
                builder::fire(&mut self.builder, &self.stage.env, &mut self.stage.out);
            }
            if let Some(request) = self.stage.out.pop() {
                if self.cut == Some(Cut::BeforeKeep)
                    && matches!(request, Request::Store(builder::store::vocabulary::Request::Keep { .. }))
                {
                    self.requests.push(request);
                    return self;
                }
                self.answer(&request);
                self.requests.push(request);
                continue;
            }
            if self.stage.has_events() || self.builder.work_pending() {
                continue;
            }
            if self.schedule.is_empty()
                && let Some(delivery) = self.reads.pop_front()
            {
                self.send(delivery);
            }
            let next = [self.schedule.next_time(), builder::next_deadline(&self.builder), self.held.map(|(_, at)| at)]
                .into_iter()
                .flatten()
                .min();
            if let Some(at) = next {
                self.stage.tick(at);
                continue;
            }
            assert!(self.builder.is_empty(), "the builder settled without a reply");
            self.referee.settled(self.files());
            return self;
        }
        unreachable!("the bounded builder scenario settles")
    }
    fn release(&mut self) {
        if let Some((file, at)) = self.held
            && self.stage.env.now >= at
        {
            self.shared.answer(skein_sim::Ask::Close { file }).expect("release external lock");
            self.held = None;
        }
    }
    fn send(&mut self, delivery: Delivery) {
        let at = self.stage.env.now.saturating_add(Span::millis(1, 20).draw(&mut self.rng));
        self.schedule.send(at, delivery);
    }
    fn answer(&mut self, request: &Request) {
        let key = self.schedule.name();
        match request {
            Request::Render(request) => {
                self.referee.ask(key);
                let events = self.render(request);
                let count = events.len();
                for (index, event) in events.into_iter().enumerate() {
                    self.reads.push_back(Delivery { event, terminal: (index + 1 == count).then_some(key) });
                }
            }
            Request::Store(request) => {
                self.referee.ask(key);
                if *request == builder::store::vocabulary::Request::Lock {
                    self.lock_times.push(self.stage.env.now);
                }
                let mut files = self.files.take().expect("world files");
                let event = self.store.answer(&mut files, request.clone());
                self.files = Some(files);
                self.send(Delivery { event: Event::Store(event), terminal: Some(key) });
            }
            Request::Run { step } => {
                self.referee.ask(key);
                let event = Event::Build(builder::build::vocabulary::Event::Unavailable { step: *step });
                self.send(Delivery { event, terminal: Some(key) });
            }
            Request::Answer(answer) => self.answers.push(answer.clone()),
            Request::Progress(progress) => self.progress.push(progress.clone()),
            Request::Reply { outcome, .. } => {
                assert!(self.outcome.replace(*outcome).is_none(), "one call terminal");
            }
            Request::Unopened { why, .. } => unreachable!("valid world store opens: {why:?}"),
        }
    }
    #[must_use]
    pub fn restart(mut self) -> Self {
        self.files.take();
        Self::over(self.call, self.shared, None, self.seed)
    }
    #[must_use]
    pub fn strays(&self) -> usize {
        crate::referee::strays(self.files())
    }
    #[must_use]
    pub fn requests(&self) -> &[Request] {
        &self.requests
    }
    #[must_use]
    pub fn files(&self) -> &Files {
        self.files.as_ref().expect("world files")
    }
    #[must_use]
    pub const fn now(&self) -> Time {
        self.stage.env.now
    }
    fn render(&mut self, request: &render::vocabulary::Request) -> Vec<Event> {
        use render::vocabulary::Request as R;
        match request {
            R::List { request, repository, revision } => vec![Event::Render(self.list(*request, repository, revision))],
            R::Read { request, repository, blobs, .. } => self.read(*request, repository, blobs),
            R::Keep { request, path, sha256, bytes } => {
                let arrival = self.store.arrival(self.files.as_mut().expect("files"), *sha256, bytes);
                vec![Event::Produced {
                    request: *request,
                    path: path.clone(),
                    arrival,
                    sha256: *sha256,
                    size: bytes.len() as u64,
                    took: skein_lib::Duration::ZERO,
                }]
            }
            R::Resolve { request, sources, names, variant, .. } => self.resolve(*request, sources, names, *variant),
            R::Find { .. } | R::Record { .. } => unreachable!("root handles finding and recording"),
            R::Hunks { .. } => unreachable!("hearth-28.2: domain render scenarios ask no textual hunks"),
        }
    }
    fn list(&self, request: Token, repository: &[u8], revision: &[u8]) -> RenderEvent {
        use hearth_fake_git::{objects::Mode, repository::Resolved};
        let fake = self.shared.borrow();
        let repository = fake.repositories.0.get(repository).expect("drawn repository");
        let Resolved::Commit(commit) = repository.resolve(revision) else {
            return RenderEvent::Unresolved { request };
        };
        let entries = repository
            .list(commit)
            .expect("resolved commit")
            .into_iter()
            .map(|entry| render::revision::Entry {
                path: entry.path,
                mode: match entry.mode {
                    Mode::File => render::revision::Mode::File,
                    Mode::Executable => render::revision::Mode::Executable,
                    Mode::Link => render::revision::Mode::Link,
                    Mode::Submodule => render::revision::Mode::Submodule,
                },
                oid: render::revision::Oid(entry.oid.hex().as_slice().into()),
                size: entry.size.unwrap_or(0),
            })
            .collect();
        RenderEvent::Listed { request, commit: render::revision::Oid(commit.hex().as_slice().into()), entries }
    }
    fn read(&mut self, request: Token, repository: &[u8], blobs: &[render::vocabulary::Blob]) -> Vec<Event> {
        let mut events = Vec::new();
        for (index, blob) in blobs.iter().enumerate() {
            let bytes = {
                let fake = self.shared.borrow();
                let repository = fake.repositories.0.get(repository).expect("drawn repository");
                repository
                    .object(hearth_fake_git::objects::Oid::from_hex(&blob.oid.0).expect("listed oid"))
                    .expect("listed object")
                    .1
                    .to_vec()
            };
            if blob.kind == render::vocabulary::As::Cached {
                let sha256 = hearth_lib::encode::Digest(skein_lib::sha256::digest(&bytes));
                let arrival = self.store.arrival(self.files.as_mut().expect("files"), sha256, &bytes);
                events.push(Event::Arrived {
                    request,
                    oid: blob.oid.clone(),
                    size: bytes.len() as u64,
                    last: index + 1 == blobs.len(),
                    arrival,
                    sha256,
                    took: skein_lib::Duration::ZERO,
                });
            } else {
                events.push(Event::Render(RenderEvent::Read {
                    request,
                    oid: blob.oid.clone(),
                    read: decode::blob(&bytes, blob.kind),
                }));
            }
        }
        events
    }
}
