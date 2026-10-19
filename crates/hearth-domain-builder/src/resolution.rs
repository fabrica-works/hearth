//! Resolution records returned through the store (render.md, 5.7).
use crate::{render, store};
use skein_lib::Token;
/// The store's answers to a find or a record.
#[derive(Debug)]
pub(crate) enum Settled {
    Found(store::vocabulary::Resolution),
    Absent,
    Recorded,
    Declined(store::vocabulary::Decline),
}
pub(crate) fn answer(request: Token, answer: Settled) -> render::vocabulary::Event {
    match answer {
        Settled::Found(resolution) => render::vocabulary::Event::Found { request, resolution: found(resolution) },
        Settled::Absent => render::vocabulary::Event::Absent { request },
        Settled::Declined(store::vocabulary::Decline::Bound { max }) => render::vocabulary::Event::Failed {
            request,
            class: render::vocabulary::Class::Configuration,
            at: None,
            failure: render::vocabulary::Failure::TooLarge { limit: render::errors::Limit::Record, max },
        },
        Settled::Declined(store::vocabulary::Decline::Full { .. } | store::vocabulary::Decline::Unusable { .. }) => {
            render::vocabulary::Event::Failed {
                request,
                class: render::vocabulary::Class::Operational,
                at: None,
                failure: render::vocabulary::Failure::Store,
            }
        }
        Settled::Recorded => render::vocabulary::Event::Recorded { request },
    }
}

#[must_use]
pub fn stored(record: render::resolution::Record) -> store::vocabulary::Resolution {
    let mut closure = skein_lib::List::with_capacity(u32::try_from(record.closure.len()).expect("closure bound"));
    for package in record.closure {
        let render::resolution::Package { name, architecture, version, source, filename, size, sha256 } = package;
        closure
            .push(store::vocabulary::Closed { name, architecture, version, source, filename, size, sha256 })
            .expect("closure bound");
    }
    let mut sources = skein_lib::List::with_capacity(u32::try_from(record.sources.len()).expect("sources bound"));
    for line in record.sources {
        let mut indices = skein_lib::List::with_capacity(u32::try_from(line.indices.len()).expect("indices bound"));
        for index in line.indices {
            indices
                .push(store::vocabulary::Index { path: index.path, size: index.size, sha256: index.sha256 })
                .expect("indices bound");
        }
        sources
            .push(store::vocabulary::Line {
                uri: line.uri,
                suite: line.suite,
                components: line.components,
                indices: indices.into_boxed(),
            })
            .expect("sources bound");
    }
    store::vocabulary::Resolution {
        key: record.key,
        digest: record.digest,
        inputs: store::vocabulary::Inputs {
            sources: record.inputs.sources,
            request: record.inputs.request,
            variant: record.inputs.variant,
            apt: record.inputs.apt,
        },
        sources: sources.into_boxed(),
        closure: closure.into_boxed(),
        made: store::vocabulary::Made {
            at: record.made.at,
            apt: record.made.tools.apt,
            mmdebstrap: record.made.tools.mmdebstrap,
            costs: store::vocabulary::Costs { took: record.made.costs.took, fetched: record.made.costs.fetched },
        },
    }
}

#[must_use]
pub fn found(record: store::vocabulary::Resolution) -> render::resolution::Record {
    let mut closure = skein_lib::List::with_capacity(u32::try_from(record.closure.len()).expect("closure bound"));
    for package in record.closure {
        let store::vocabulary::Closed { name, architecture, version, source, filename, size, sha256 } = package;
        closure
            .push(render::resolution::Package { name, architecture, version, source, filename, size, sha256 })
            .expect("closure bound");
    }
    let mut sources = skein_lib::List::with_capacity(u32::try_from(record.sources.len()).expect("sources bound"));
    for line in record.sources {
        let mut indices = skein_lib::List::with_capacity(u32::try_from(line.indices.len()).expect("indices bound"));
        for index in line.indices {
            indices
                .push(render::resolution::Index { path: index.path, size: index.size, sha256: index.sha256 })
                .expect("indices bound");
        }
        sources
            .push(render::resolution::Line {
                uri: line.uri,
                suite: line.suite,
                components: line.components,
                indices: indices.into_boxed(),
            })
            .expect("sources bound");
    }
    render::resolution::Record {
        key: record.key,
        digest: record.digest,
        inputs: render::parts::packages::Inputs {
            sources: record.inputs.sources,
            request: record.inputs.request,
            variant: record.inputs.variant,
            apt: record.inputs.apt,
        },
        sources: sources.into_boxed(),
        closure: closure.into_boxed(),
        made: render::resolution::Made {
            at: record.made.at,
            tools: render::resolution::Tools { apt: record.made.apt, mmdebstrap: record.made.mmdebstrap },
            costs: render::resolution::Costs { took: record.made.costs.took, fetched: record.made.costs.fetched },
        },
    }
}

pub(super) fn terminal(
    builder: &mut crate::Builder,
    env: &skein_lib::Env<crate::Limits>,
    answer: Settled,
    out: &mut skein_lib::Queue<crate::vocabulary::Request>,
) {
    use store::vocabulary::Answer;
    if let Settled::Declined(why) = &answer {
        let fault = match why {
            store::vocabulary::Decline::Full { limit, most } => Answer::Full { limit: *limit, most: *most },
            store::vocabulary::Decline::Unusable { step } => Answer::Unusable { step: *step },
            store::vocabulary::Decline::Bound { .. } => Answer::Ready,
        };
        crate::store_error(builder, &fault, out);
    }
    let request = if let Some(render::vocabulary::Request::Resolve { request, .. }) = builder.resolving.take() {
        request
    } else {
        builder.finding.take().expect("store request answer")
    };
    let event = self::answer(request, answer);
    crate::step(builder, env, crate::vocabulary::Event::Render(event), out);
}

/// The protocol drains its scratch before the root replaces a refused resolve.
pub(super) fn progress(
    builder: &mut crate::Builder,
    event: render::vocabulary::Event,
    out: &mut skein_lib::Queue<crate::vocabulary::Request>,
) -> render::vocabulary::Event {
    use crate::vocabulary::{Progress, Request};
    use render::vocabulary::{Class, Event, Failure};
    let Some((request, key)) = builder.resolution else { return event };
    let terminal = match &event {
        Event::Resolved { request: token, costs, .. } if *token == request => Some(Some(costs.took)),
        Event::Failed { request: token, .. } if *token == request => Some(None),
        Event::Recorded { .. }
        | Event::Inst { .. }
        | Event::Stanza { .. }
        | Event::Resolved { .. }
        | Event::Found { .. }
        | Event::Absent { .. }
        | Event::Diff { .. }
        | Event::Render { .. }
        | Event::Listed { .. }
        | Event::Unresolved { .. }
        | Event::Read { .. }
        | Event::Kept { .. }
        | Event::Cached { .. }
        | Event::Hunks { .. }
        | Event::Failed { .. } => None,
    };
    let Some(took) = terminal else { return event };
    builder.resolution = None;
    let refused = builder.refused == Some(request);
    out.push(Request::Progress(match took {
        Some(took) if !refused => Progress::Resolved { key, took },
        Some(_) | None => Progress::Unresolved { key },
    }));
    if refused {
        builder.refused = None;
        Event::Failed { request, class: Class::Operational, at: None, failure: Failure::Store }
    } else {
        event
    }
}
