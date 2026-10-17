//! One client call, routed through render (render.md, 8.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub use hearth_domain_render as render;
pub use hearth_domain_store as store;
pub mod vocabulary;
use render::vocabulary::{Answer as RenderAnswer, Event as RenderEvent, Output};
use skein_lib::{Env, Queue, ReplyTo, Time};
use vocabulary::{Answer, Call, Event, Request};
pub const MAX_OUT: u32 = 1;
pub const MAX_OUT_RESUME: u32 = 1;
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
}
impl Builder {
    #[must_use]
    pub fn new(limits: &Limits, uid: u32) -> Self {
        Self {
            render: render::Render::new(&limits.render),
            store: store::Store::new(&limits.store, uid),
            call: None,
            store_output: Queue::with_capacity(1),
            reply: None,
            output: Queue::with_capacity(1),
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
    fn forward(&mut self, out: &mut Queue<Request>) {
        if let Some(output) = self.output.pop() {
            out.push(match output {
                Output::Request(request) => Request::Render(request),
                Output::Answer(RenderAnswer::Spec(spec)) => Request::Answer(Answer::Spec(spec)),
                Output::Answer(RenderAnswer::Error(error)) => Request::Answer(Answer::Error(error)),
                Output::Answer(RenderAnswer::Change(change)) => Request::Answer(Answer::Change(change)),
                Output::Answer(RenderAnswer::Rendered { outcome, diff, left_out }) => Request::Reply {
                    to: self.reply.take().expect("active call owns its reply"),
                    outcome,
                    diff,
                    left_out,
                },
            });
        }
    }
}
pub fn step(builder: &mut Builder, env: &Env<Limits>, ev: Event, out: &mut Queue<Request>) {
    let event = match ev {
        Event::Call { reply_to, call } => {
            assert!(builder.reply.is_none(), "one builder call at a time");
            builder.reply = Some(reply_to);
            builder.call = Some(call);
            return store_step(builder, env, store::vocabulary::Event::Open, out);
        }
        Event::Store(event) => return store_step(builder, env, event, out),
        Event::Render(event) => event,
    };
    let render_env = Env { now: env.now, wall: env.wall, limits: env.limits.render };
    render::step(&mut builder.render, &render_env, event, &mut builder.output);
    builder.forward(out);
}
pub fn resume(builder: &mut Builder, env: &Env<Limits>, out: &mut Queue<Request>) {
    let render_env = Env { now: env.now, wall: env.wall, limits: env.limits.render };
    render::resume(&mut builder.render, &render_env, &mut builder.output);
    builder.forward(out);
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
    use store::vocabulary::{Answer, Output};
    match builder.store_output.pop() {
        Some(Output::Request(request)) => out.push(Request::Store(request)),
        Some(Output::Answer(Answer::Ready | Answer::Full { .. } | Answer::Unusable { .. })) => {
            let event = match builder.call.take().expect("opening call") {
                Call::Diff { repository, from, to, targets } => RenderEvent::Diff { repository, from, to, targets },
                Call::Render { repository, revision, targets } => RenderEvent::Render { repository, revision, targets },
            };
            let env = Env { now: env.now, wall: env.wall, limits: env.limits.render };
            render::step(&mut builder.render, &env, event, &mut builder.output);
            builder.forward(out);
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
