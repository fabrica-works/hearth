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
use vocabulary::{Answer, Event, Inputs, Output, Request, Why};
/// One request or terminal per event.
pub const MAX_OUT: u32 = 1;
/// What the store was asked of the layer in hand, and has not yet answered.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Asked {
    Stage,
    Seal,
    /// The failure to answer once the store has dropped the layer.
    Discard {
        step: Step,
        why: Why,
    },
}
impl Asked {
    /// The failure a discard holds, if this is a discard.
    const fn failure(self) -> Option<(Step, Why)> {
        match self {
            Self::Discard { step, why } => Some((step, why)),
            Self::Stage | Self::Seal => None,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Parts {
    closure: hearth_lib::encode::Digest,
    packages: hearth_lib::encode::Digest,
    software: hearth_lib::encode::Digest,
    config: hearth_lib::encode::Digest,
}
impl Parts {
    /// What the layer's key is made of, besides the key beneath it.
    const fn inputs(&self, layer: Layer) -> Inputs {
        match layer {
            Layer::Packages => Inputs::Packages { part: self.packages, resolution: self.closure },
            Layer::Software => Inputs::Software { part: self.software },
            Layer::Config => Inputs::Config { part: self.config },
        }
    }
}
/// A build runs its plan the way domain/store.md, 4.2 lays out a layer's life: for each
/// layer, bottom-up, stage it, run its steps in order, seal it. One request is in flight.
#[derive(Debug, Hash)]
pub struct Build {
    keys: Option<Keys>,
    /// The resolution's closure digest and the three part digests, which the layers' records name.
    parts: Option<Parts>,
    asking: Option<Layer>,
    plan: Option<Plan>,
    /// The index in the plan of the layer being made.
    layer: usize,
    /// The index in the layer's steps of the step being run.
    running: Option<usize>,
    asked: Option<Asked>,
}
impl Build {
    #[must_use]
    #[expect(clippy::new_without_default, reason = "step code names no Default (programming-model.md, 10.2)")]
    pub const fn new() -> Self {
        Self { keys: None, parts: None, asking: None, plan: None, layer: 0, running: None, asked: None }
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
            build.parts = Some(Parts { closure, packages, software, config });
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
        Event::Staged { key } => {
            store_answered(build, key, Asked::Stage);
            run(build, 0, out);
        }
        Event::Ran { step } => {
            let at = ran(build, step);
            run(build, next(at), out);
        }
        Event::Sealed { key } => {
            store_answered(build, key, Asked::Seal);
            build.layer = next(build.layer);
            if build.layer < planned_layers(build) {
                stage(build, out);
            } else {
                built(build, out);
            }
        }
        Event::Discarded { key } => {
            let asked = build.asked.take().expect("the store answers a question asked");
            let (step, why) = asked.failure().expect("the store discards what it was asked to discard");
            assert_eq!(key, current(build).key, "the store answers the key asked");
            failed(build, step, why, out);
        }
        Event::Unavailable { step } => {
            ran(build, step);
            discard(build, step, Why::Unavailable, out);
        }
        Event::StepFailed { step } => {
            ran(build, step);
            discard(build, step, Why::Failed, out);
        }
        Event::Refused { key } => {
            assert_eq!(key, current(build).key, "the store answers the key asked");
            match build.asked.take().expect("the store refuses what it was asked") {
                Asked::Stage => {
                    let step = current(build).steps.first().copied().expect("a layer runs a step");
                    failed(build, step, Why::Stage, out);
                }
                Asked::Seal => {
                    let step = current(build).steps.last().copied().expect("a layer runs a step");
                    discard(build, step, Why::Seal, out);
                }
                Asked::Discard { step, why } => failed(build, step, why, out),
            }
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
    let runs = plan.first().is_some();
    build.plan = Some(plan);
    build.layer = 0;
    if runs {
        stage(build, out);
    } else {
        built(build, out);
    }
}
/// The layer being made.
fn current(build: &Build) -> &Planned {
    build.plan.as_ref().expect("a plan runs").build.get(build.layer).expect("a layer in the plan")
}
fn planned_layers(build: &Build) -> usize {
    build.plan.as_ref().expect("a plan runs").build.len()
}
const fn next(at: usize) -> usize {
    at.checked_add(1).expect("the plan's indices fit usize")
}
/// Whether nothing is in flight: the build sends one request at a time.
fn idle(build: &Build) -> bool {
    build.asking.is_none() && build.running.is_none() && build.asked.is_none()
}
fn stage(build: &mut Build, out: &mut Queue<Output>) {
    assert!(idle(build), "one request in flight");
    let keys = build.keys.expect("keys before the plan");
    let (layer, key) = (current(build).layer, current(build).key);
    build.asked = Some(Asked::Stage);
    out.push(Output::Request(Request::Stage { key, parent: keys.parent(layer) }));
}
/// Runs the layer's step at `at`, or seals the layer after its last.
fn run(build: &mut Build, at: usize, out: &mut Queue<Output>) {
    assert!(idle(build), "one request in flight");
    match current(build).steps.get(at).copied() {
        Some(step) => {
            build.running = Some(at);
            out.push(Output::Request(Request::Run { step }));
        }
        None => {
            let (layer, key) = (current(build).layer, current(build).key);
            let keys = build.keys.expect("keys before the plan");
            let parts = build.parts.expect("parts before the plan");
            build.asked = Some(Asked::Seal);
            out.push(Output::Request(Request::Seal { key, parent: keys.parent(layer), inputs: parts.inputs(layer) }));
        }
    }
}
fn discard(build: &mut Build, step: Step, why: Why, out: &mut Queue<Output>) {
    assert!(idle(build), "one request in flight");
    let key = current(build).key;
    build.asked = Some(Asked::Discard { step, why });
    out.push(Output::Request(Request::Discard { key }));
}
/// Checks the store's answer is to the question asked, on the key in hand.
fn store_answered(build: &mut Build, key: hearth_lib::encode::Digest, question: Asked) {
    assert_eq!(build.asked.take(), Some(question), "the store answers the question asked");
    assert_eq!(key, current(build).key, "the store answers the key asked");
}
/// Ends the step in flight, which must be `step`; its index in the layer's steps.
fn ran(build: &mut Build, step: Step) -> usize {
    let at = build.running.take().expect("a step ends a step run");
    assert_eq!(current(build).steps.get(at).copied(), Some(step), "the step ended is the step run");
    at
}
fn built(build: &mut Build, out: &mut Queue<Output>) {
    let plan = build.plan.take().expect("a plan runs");
    build.keys = None;
    build.parts = None;
    build.layer = 0;
    out.push(Output::Answer(Answer::Built { plan }));
}
fn failed(build: &mut Build, step: Step, why: Why, out: &mut Queue<Output>) {
    let plan = build.plan.take().expect("a plan runs");
    build.keys = None;
    build.parts = None;
    build.layer = 0;
    out.push(Output::Answer(Answer::Failed { plan, step, why }));
}
#[cfg(test)]
mod tests;
