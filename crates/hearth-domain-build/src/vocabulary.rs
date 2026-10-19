//! A build's events, requests and terminals (domain/build.md, 4.1 and 4.3).
use crate::{
    keys::Layer,
    plan::{Plan, Step},
};
use hearth_lib::encode::Digest;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    /// The spec's output parts and the closure digest of its resolution.
    Build { closure: Digest, packages: Digest, software: Digest, config: Digest },
    /// The store holds the layer of this key, published.
    Have { key: Digest },
    /// The store holds no layer of this key.
    Free { key: Digest },
    /// The store made the layer's writable copy.
    Staged { key: Digest },
    /// The step ran.
    Ran { step: Step },
    /// The store published the layer.
    Sealed { key: Digest },
    /// The store dropped the layer's writable copy.
    Discarded { key: Digest },
    /// Nothing runs this step yet.
    Unavailable { step: Step },
    /// The step ran and failed.
    StepFailed { step: Step },
    /// The store could not do what was asked of this layer's key.
    Refused { key: Digest },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    /// Whether the store holds a published layer of this key.
    Want {
        key: Digest,
    },
    /// A writable copy of the parent layer, to become the layer of this key (domain/store.md, 4.2).
    Stage {
        key: Digest,
        parent: Digest,
    },
    Run {
        step: Step,
    },
    /// Walk, freeze, record and publish the staged layer.
    Seal {
        key: Digest,
    },
    /// Drop the staged layer after a failure past its staging.
    Discard {
        key: Digest,
    },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    /// Every layer of the plan is published.
    Built { plan: Plan },
    /// `step` is the step that failed; for `Why::Stage` it is the layer's first step, and for
    /// `Why::Seal` its last, so the report names a step in the failed layer.
    Failed { plan: Plan, step: Step, why: Why },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    /// Nothing runs the step.
    Unavailable,
    /// The step ran and failed.
    Failed,
    /// The store could not stage the layer.
    Stage,
    /// The store could not seal the layer.
    Seal,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Output {
    Request(Request),
    Answer(Answer),
}
/// The layer the domain asks the store about, top down from config.
pub(crate) const fn beneath(layer: Layer) -> Option<Layer> {
    match layer {
        Layer::Config => Some(Layer::Software),
        Layer::Software => Some(Layer::Packages),
        Layer::Packages => None,
    }
}
