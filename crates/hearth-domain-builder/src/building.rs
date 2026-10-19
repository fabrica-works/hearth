//! A build's routes: render's spec in, the store's index and the protocol's steps
//! between, its layers out to the client (domain/build.md, 4.1; client/README.md, 3.1).
use crate::{
    Builder, Limits, build, render, store_step,
    vocabulary::{Answer, Request, State},
};
use build::{
    LAYERS, Layer, Plan, Step,
    vocabulary::{Event, Output, Why},
};
use skein_lib::{Env, Queue};
#[derive(Debug, Hash)]
pub(crate) enum Building {
    /// Render resolves and renders the one target.
    Rendering,
    /// The build domain asks the store, then runs its first step.
    Planning,
    /// The plan's layers, then its failure, go to the client one by one.
    Reporting(Report),
    Reported,
}
#[derive(Debug, Hash)]
pub(crate) struct Report {
    plan: Plan,
    failed: Option<(Step, Why)>,
    next: usize,
}
impl Report {
    fn next(&mut self) -> Option<Answer> {
        let at = self.next;
        self.next = at.saturating_add(1);
        if let Some(layer) = LAYERS.get(at) {
            let state = match self.plan.found {
                Some(found) if *layer <= found => State::Found,
                Some(Layer::Packages | Layer::Software | Layer::Config) | None => State::ToBuild,
            };
            return Some(Answer::Layer { layer: *layer, key: self.plan.keys.of(*layer), state });
        }
        if at == LAYERS.len()
            && let Some((step, why)) = self.failed
        {
            return Some(Answer::Failed { step, why });
        }
        None
    }
}
/// The spec of a build's target: its keys go to the build domain, the spec no further.
pub(crate) fn spec(builder: &mut Builder, env: &Env<Limits>, spec: render::parts::Spec, out: &mut Queue<Request>) {
    let Some(render::keys::Resolution::Known { key, .. }) = spec.resolution.as_deref() else { return };
    let Some((resolved, closure)) = builder.resolved else { return };
    if resolved != *key || builder.store_error {
        return;
    }
    builder.building = Some(Building::Planning);
    let event = Event::Build {
        closure,
        packages: render::keys::part(&spec, b"packages"),
        software: render::keys::part(&spec, b"software"),
        config: render::keys::part(&spec, b"config"),
    };
    step(builder, env, event, out);
}
pub(crate) fn step(builder: &mut Builder, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    let build_env = Env { now: env.now, wall: env.wall, limits: env.limits.build };
    build::step(&mut builder.build, &build_env, event, &mut builder.build_output);
    match builder.build_output.pop() {
        Some(Output::Request(build::vocabulary::Request::Want { key })) => {
            store_step(builder, env, crate::store::vocabulary::Event::Want { key }, out);
        }
        Some(Output::Request(build::vocabulary::Request::Stage { key, parent })) => {
            builder.staging = Some(key);
            store_step(builder, env, crate::store::vocabulary::Event::Stage { key, parent }, out);
        }
        Some(Output::Request(build::vocabulary::Request::Run { step })) => out.push(Request::Run { step }),
        Some(Output::Request(build::vocabulary::Request::Seal { key })) => {
            builder.staging = Some(key);
            store_step(builder, env, crate::store::vocabulary::Event::Seal { key }, out);
        }
        Some(Output::Request(build::vocabulary::Request::Discard { key })) => {
            builder.staging = Some(key);
            store_step(builder, env, crate::store::vocabulary::Event::Discard { key }, out);
        }
        Some(Output::Answer(build::vocabulary::Answer::Built { plan })) => {
            report(builder, Report { plan, failed: None, next: 0 }, out);
        }
        Some(Output::Answer(build::vocabulary::Answer::Failed { plan, step, why })) => {
            report(builder, Report { plan, failed: Some((step, why)), next: 0 }, out);
        }
        None => {}
    }
}
/// The store answers a stage, a seal or a discard: the question is settled.
pub(crate) fn staged(builder: &mut Builder, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    builder.staging = None;
    step(builder, env, event, out);
}
fn report(builder: &mut Builder, report: Report, out: &mut Queue<Request>) {
    builder.building = Some(Building::Reporting(report));
    resume(builder, out);
}
/// Whether render's reply waits for the build's report.
pub(crate) fn holds(builder: &Builder) -> bool {
    match &builder.building {
        Some(Building::Planning | Building::Reporting(_)) => true,
        Some(Building::Rendering | Building::Reported) | None => false,
    }
}
pub(crate) fn reporting(builder: &Builder) -> bool {
    match &builder.building {
        Some(Building::Reporting(_)) => true,
        Some(Building::Rendering | Building::Planning | Building::Reported) | None => false,
    }
}
/// One record of the report, or, once it is all out, render's held reply.
pub(crate) fn resume(builder: &mut Builder, out: &mut Queue<Request>) {
    let Some(Building::Reporting(report)) = &mut builder.building else { return };
    if let Some(answer) = report.next() {
        out.push(Request::Answer(answer));
        return;
    }
    builder.building = Some(Building::Reported);
    if let Some((outcome, diff, left_out)) = builder.held.take() {
        builder.building = None;
        let to = builder.reply.take().expect("active call owns its reply");
        out.push(Request::Reply { to, outcome, diff, left_out });
    }
}
