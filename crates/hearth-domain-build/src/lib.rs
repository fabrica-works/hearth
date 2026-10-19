//! A build's plan, from its keys and what the store has published (domain/build.md, 4.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod keys;
pub mod limits;
pub mod plan;
pub mod vocabulary;
pub use keys::{Keys, LAYERS, Layer, keys};
use limits::Limits;
pub use plan::{How, Plan, Planned, Step};
use skein_lib::{Env, Queue};
use vocabulary::{Answer, Event, Output, Request, Why};
/// One request or terminal per event.
pub const MAX_OUT: u32 = 1;
#[derive(Debug, Hash)]
pub struct Build {
    keys: Option<Keys>,
    asking: Option<Layer>,
    plan: Option<Plan>,
}
impl Build {
    #[must_use]
    #[expect(clippy::new_without_default, reason = "step code names no Default (programming-model.md, 10.2)")]
    pub const fn new() -> Self {
        Self { keys: None, asking: None, plan: None }
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.keys.is_none() && self.plan.is_none()
    }
}
pub fn step(build: &mut Build, env: &Env<Limits>, ev: Event, out: &mut Queue<Output>) {
    match ev {
        Event::Build { closure, packages, software, config } => {
            assert!(build.is_empty(), "one build at a time");
            let keys = keys(closure, packages, software, config);
            build.keys = Some(keys);
            want(build, Layer::Config, out);
        }
        Event::Have { key } => {
            let layer = answered(build, key);
            planned(build, env, Some(layer), out);
        }
        Event::Free { key } => match vocabulary::beneath(answered(build, key)) {
            Some(beneath) => want(build, beneath, out),
            None => planned(build, env, None, out),
        },
        Event::Unavailable { step } => {
            let plan = build.plan.take().expect("a step runs for a plan");
            build.keys = None;
            out.push(Output::Answer(Answer::Failed { plan, step, why: Why::Unavailable }));
        }
    }
}
fn want(build: &mut Build, layer: Layer, out: &mut Queue<Output>) {
    let key = build.keys.expect("keys before questions").of(layer);
    build.asking = Some(layer);
    out.push(Output::Request(Request::Want { key }));
}
fn answered(build: &mut Build, key: hearth_lib::encode::Digest) -> Layer {
    let keys = build.keys.expect("a build asks the store");
    let layer = build.asking.take().expect("one store question at a time");
    assert_eq!(key, keys.of(layer), "the store answers the key asked");
    layer
}
fn planned(build: &mut Build, env: &Env<Limits>, found: Option<Layer>, out: &mut Queue<Output>) {
    let keys = build.keys.expect("keys before the plan");
    let plan = plan::plan(keys, found);
    assert!(
        plan.build.len() <= usize::try_from(env.limits.layers).expect("u32 fits usize"),
        "the plan's layers fit its bound"
    );
    if let Some(step) = plan.first() {
        build.plan = Some(plan);
        out.push(Output::Request(Request::Run { step }));
    } else {
        build.keys = None;
        out.push(Output::Answer(Answer::Built { plan }));
    }
}
#[cfg(test)]
mod tests;
