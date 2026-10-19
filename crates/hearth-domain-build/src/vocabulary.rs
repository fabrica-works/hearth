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
    /// Nothing runs this step yet.
    Unavailable { step: Step },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    /// Whether the store holds a published layer of this key.
    Want {
        key: Digest,
    },
    Run {
        step: Step,
    },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    /// Every layer of the plan is published.
    Built {
        plan: Plan,
    },
    Failed {
        plan: Plan,
        step: Step,
        why: Why,
    },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    Unavailable,
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
