//! One client call, routed through render (render.md, 8.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
pub use hearth_domain_render as render;
pub use hearth_domain_store as store;
mod resolution;
pub mod vocabulary;
use render::vocabulary::{Answer as RenderAnswer, Event as RenderEvent, Output};
use skein_lib::{Env, Queue, ReplyTo, Time, Token};
use vocabulary::{Answer, Call, Event, Request};
/// A notice or store error before render forwards its output.
pub const MAX_OUT: u32 = 2;
/// A store error and a render answer may become ready together.
pub const MAX_OUT_RESUME: u32 = 2;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub render: render::limits::Limits,
    pub store: store::limits::Limits,
}
#[derive(Debug)]
pub struct Builder {
    render: render::Render,
    store: store::Store,
    call: Option<Call>,
    store_output: Queue<store::vocabulary::Output>,
    reply: Option<ReplyTo>,
    output: Queue<Output>,
    arrival: Option<(Token, render::revision::Oid, u64, bool, render::vocabulary::Digest)>,
    produced: Option<(Token, render::vocabulary::Digest)>,
    refused: Option<Token>,
    remaining: usize,
    store_error: bool,
    finding: Option<Token>,
    resolving: Option<render::vocabulary::Request>,
    resolution: Option<(Token, render::vocabulary::Digest)>,
    index: Option<Token>,
}
impl Builder {
    #[must_use]
    pub fn new(limits: &Limits, uid: u32) -> Self {
        Self {
            arrival: None,
            produced: None,
            refused: None,
            remaining: 0,
            store_error: false,
            finding: None,
            resolving: None,
            resolution: None,
            index: None,
            render: render::Render::new(&limits.render),
            store: store::Store::new(&limits.store, uid),
            call: None,
            store_output: Queue::with_capacity(2),
            reply: None,
            output: Queue::with_capacity(2),
        }
    }
    #[must_use]
    pub fn work_pending(&self) -> bool {
        self.render.work_pending()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reply.is_none()
    }
    fn forward(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) {
        if let Some(output) = self.output.pop() {
            if let Output::Request(render::vocabulary::Request::Find { request, key }) = output {
                self.finding = Some(request);
                store_step(self, env, store::vocabulary::Event::Find { key }, out);
                return;
            }
            if let Output::Request(render::vocabulary::Request::Record { request, resolution }) = output {
                self.finding = Some(request);
                store_step(
                    self,
                    env,
                    store::vocabulary::Event::Record { resolution: resolution::stored(resolution) },
                    out,
                );
                return;
            }
            if let Output::Request(request @ render::vocabulary::Request::Resolve { .. }) = output {
                self.resolving = Some(request);
                store_step(self, env, store::vocabulary::Event::Room, out);
                return;
            }
            out.push(match output {
                Output::Request(request) => {
                    if let render::vocabulary::Request::Read { blobs, .. } = &request {
                        self.remaining = blobs.len();
                    }
                    Request::Render(request)
                }
                Output::Answer(RenderAnswer::Spec(spec)) => Request::Answer(Answer::Spec(spec)),
                Output::Answer(RenderAnswer::Error(error)) => Request::Answer(Answer::Error(error)),
                Output::Answer(RenderAnswer::Change(change)) => Request::Answer(Answer::Change(change)),
                Output::Answer(RenderAnswer::Rendered { outcome, diff, left_out }) => Request::Reply {
                    to: self.reply.take().expect("active call owns its reply"),
                    outcome: if self.store_error {
                        outcome.worst(render::vocabulary::Outcome::TryAgain)
                    } else {
                        outcome
                    },
                    diff,
                    left_out,
                },
            });
        }
    }
}
pub fn step(builder: &mut Builder, env: &Env<Limits>, ev: Event, out: &mut Queue<Request>) {
    let event = match ev {
        Event::Index { request, arrival, sha256, size, took } => {
            let (active, key) = builder.resolution.expect("index during resolve");
            assert_eq!(request, active, "current resolution");
            builder.index = Some(request);
            return store_step(
                builder,
                env,
                store::vocabulary::Event::Arrived {
                    arrival,
                    sha256,
                    size,
                    origin: store::vocabulary::Origin::Index { resolution: key },
                    took,
                },
                out,
            );
        }
        Event::Produced { request, path, arrival, sha256, size, took } => {
            builder.produced = Some((request, sha256));
            return store_step(
                builder,
                env,
                store::vocabulary::Event::Arrived {
                    arrival,
                    sha256,
                    size,
                    origin: store::vocabulary::Origin::Rendered { path },
                    took,
                },
                out,
            );
        }
        Event::Arrived { request, oid, size, last, arrival, sha256, took } => {
            let origin = store::vocabulary::Origin::Blob { oid: oid.0.clone() };
            builder.arrival = Some((request, oid, size, last, sha256));
            return store_step(
                builder,
                env,
                store::vocabulary::Event::Arrived { arrival, sha256, size, origin, took },
                out,
            );
        }
        Event::Call { reply_to, call } => {
            assert!(builder.reply.is_none(), "one builder call at a time");
            builder.reply = Some(reply_to);
            builder.call = Some(call);
            return store_step(builder, env, store::vocabulary::Event::Open, out);
        }
        Event::Store(event) => return store_step(builder, env, event, out),
        Event::Render(event) => {
            let event = resolution::progress(builder, event, out);
            if let RenderEvent::Read { request, .. } | RenderEvent::Cached { request, .. } = &event {
                builder.remaining = builder.remaining.saturating_sub(1);
                if builder.refused == Some(*request) {
                    if builder.remaining != 0 {
                        return;
                    }
                    RenderEvent::Failed {
                        request: *request,
                        class: render::vocabulary::Class::Operational,
                        at: None,
                        failure: render::vocabulary::Failure::Store,
                    }
                } else {
                    event
                }
            } else {
                event
            }
        }
    };
    let render_env = Env { now: env.now, wall: env.wall, limits: env.limits.render };
    render::step(&mut builder.render, &render_env, event, &mut builder.output);
    builder.forward(env, out);
}
pub fn resume(builder: &mut Builder, env: &Env<Limits>, out: &mut Queue<Request>) {
    let render_env = Env { now: env.now, wall: env.wall, limits: env.limits.render };
    render::resume(&mut builder.render, &render_env, &mut builder.output);
    builder.forward(env, out);
}
#[cfg(test)]
mod tests;

fn store_step(builder: &mut Builder, env: &Env<Limits>, event: store::vocabulary::Event, out: &mut Queue<Request>) {
    store::step(
        &mut builder.store,
        &Env { now: env.now, wall: env.wall, limits: env.limits.store },
        event,
        &mut builder.store_output,
    );
    store_forward(builder, env, out);
}
fn store_forward(builder: &mut Builder, env: &Env<Limits>, out: &mut Queue<Request>) {
    for _ in 0..store::MAX_OUT {
        store_one(builder, env, out);
    }
}
fn store_one(builder: &mut Builder, env: &Env<Limits>, out: &mut Queue<Request>) {
    use store::vocabulary::{Answer, Output};
    match builder.store_output.pop() {
        Some(Output::Request(request)) => out.push(Request::Store(request)),
        Some(Output::Answer(answer @ (Answer::Ready | Answer::Full { .. } | Answer::Unusable { .. }))) => {
            store_error(builder, &answer, out);
            if builder.call.is_none() {
                return;
            }
            let event = match builder.call.take().expect("opening call") {
                Call::Diff { repository, from, to, targets } => RenderEvent::Diff { repository, from, to, targets },
                Call::Render { resolve, repository, revision, targets } => {
                    RenderEvent::Render { resolve, repository, revision, targets }
                }
            };
            let render_env = Env { now: env.now, wall: env.wall, limits: env.limits.render };
            render::step(&mut builder.render, &render_env, event, &mut builder.output);
            builder.forward(env, out);
        }
        Some(Output::Answer(answer @ (Answer::Cached { .. } | Answer::Refused { .. }))) => {
            if let Some(request) = builder.index.take() {
                if let Answer::Refused { .. } = answer {
                    builder.refused = Some(request);
                }
                return;
            }
            if let Some((request, sha256)) = builder.produced.take() {
                let event = match answer {
                    Answer::Cached { .. } => RenderEvent::Kept { request, sha256 },
                    Answer::Refused { .. } => RenderEvent::Failed {
                        request,
                        class: render::vocabulary::Class::Operational,
                        at: None,
                        failure: render::vocabulary::Failure::Store,
                    },
                    Answer::Recorded
                    | Answer::HasRoom
                    | Answer::Found { .. }
                    | Answer::Absent
                    | Answer::Declined { .. }
                    | Answer::Ready
                    | Answer::Full { .. }
                    | Answer::Unusable { .. }
                    | Answer::Waiting
                    | Answer::Strays { .. }
                    | Answer::Unopened { .. } => unreachable!("arrival terminal"),
                };
                step(builder, env, Event::Render(event), out);
                return;
            }
            let (request, oid, size, last, sha256) = builder.arrival.take().expect("arrival answer");
            if let Answer::Refused { .. } = answer {
                builder.refused = Some(request);
            }
            if builder.refused == Some(request) {
                builder.remaining = builder.remaining.saturating_sub(1);
                if last {
                    step(
                        builder,
                        env,
                        Event::Render(RenderEvent::Failed {
                            request,
                            class: render::vocabulary::Class::Operational,
                            at: None,
                            failure: render::vocabulary::Failure::Store,
                        }),
                        out,
                    );
                }
            } else {
                step(builder, env, Event::Render(RenderEvent::Cached { request, oid, size, sha256 }), out);
            }
        }
        Some(Output::Answer(
            answer @ (Answer::Found { .. } | Answer::Absent | Answer::Recorded | Answer::Declined { .. }),
        )) => {
            resolution::terminal(builder, env, answer, out);
        }
        Some(Output::Answer(Answer::HasRoom)) => {
            let request = builder.resolving.take().expect("room before resolve");
            if let render::vocabulary::Request::Resolve { request, key, .. } = &request {
                builder.resolution = Some((*request, *key));
                out.push(Request::Progress(vocabulary::Progress::Resolving { key: *key }));
            }
            out.push(Request::Render(request));
        }
        Some(Output::Answer(Answer::Waiting)) => out.push(Request::Progress(vocabulary::Progress::Waiting)),
        Some(Output::Answer(Answer::Strays { count, first })) => {
            out.push(Request::Progress(vocabulary::Progress::Strays { count, first }));
        }
        Some(Output::Answer(Answer::Unopened { why })) => {
            builder.call = None;
            out.push(Request::Unopened { to: builder.reply.take().expect("opening call reply"), why });
        }
        None => {}
    }
}
pub fn fire(builder: &mut Builder, env: &Env<Limits>, out: &mut Queue<Request>) {
    store::fire(
        &mut builder.store,
        &Env { now: env.now, wall: env.wall, limits: env.limits.store },
        &mut builder.store_output,
    );
    store_forward(builder, env, out);
}
#[must_use]
pub fn next_deadline(builder: &Builder) -> Option<Time> {
    store::next_deadline(&builder.store)
}

fn store_error(builder: &mut Builder, answer: &store::vocabulary::Answer, out: &mut Queue<Request>) {
    use render::errors::{Error, Kind, Step, StoreError, StoreFault, StoreLimit};
    use store::vocabulary::{Answer, Limit, Step as StoreStep};
    if builder.store_error {
        return;
    }
    let fault = match answer {
        Answer::Full { limit, most } => StoreFault::Full {
            limit: match limit {
                Limit::CacheFiles => StoreLimit::CacheFiles,
                Limit::Resolutions => StoreLimit::Resolutions,
            },
            most: *most,
        },
        Answer::Unusable { step } => StoreFault::Failed {
            step: match step {
                StoreStep::List => Step::List,
                StoreStep::Remove => Step::Remove,
                StoreStep::Load => Step::Load,
                StoreStep::Empty => Step::Empty,
                StoreStep::Keep => Step::Keep,
                StoreStep::Write => Step::Write,
            },
        },
        Answer::Recorded
        | Answer::HasRoom
        | Answer::Found { .. }
        | Answer::Absent
        | Answer::Declined { .. }
        | Answer::Ready
        | Answer::Cached { .. }
        | Answer::Refused { .. }
        | Answer::Unopened { .. }
        | Answer::Waiting
        | Answer::Strays { .. } => return,
    };
    builder.store_error = true;
    out.push(Request::Answer(vocabulary::Answer::Error(Error {
        side: None,
        kind: Kind::Store(StoreError { store: Box::new([]), fault }),
        class: render::vocabulary::Class::Operational,
        at: None,
        also: None,
        target: None,
        task: None,
    })));
}
