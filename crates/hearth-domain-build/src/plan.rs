//! The plan as data: the layers above the highest published one (domain/build.md, 4.1).
use crate::keys::{Keys, LAYERS, Layer};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::List;
/// Every step of the rootfs recipe, a closed set (domain/build.md, 5 to 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    Fetch(Layer),
    Bootstrap,
    Verify,
    Place,
    Software,
    Users,
    Files,
    Tmpfiles,
    Presets,
    Customize,
}
/// How the packages layer is made (domain/build.md, 4.2): `Reconcile { base }` comes with its task.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum How {
    Scratch,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Planned {
    pub layer: Layer,
    pub key: Digest,
    pub steps: Box<[Step]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Plan {
    pub keys: Keys,
    pub found: Option<Layer>,
    pub build: Box<[Planned]>,
    pub how: How,
}
/// The most steps one layer runs: the config layer's.
pub const MAX_STEPS: u32 = 5;
/// Each layer's steps, in order (domain/build.md, 3).
#[must_use]
pub const fn steps(layer: Layer) -> &'static [Step] {
    match layer {
        Layer::Packages => &[Step::Fetch(Layer::Packages), Step::Bootstrap, Step::Verify],
        Layer::Software => &[Step::Fetch(Layer::Software), Step::Place, Step::Software],
        Layer::Config => &[Step::Users, Step::Files, Step::Tmpfiles, Step::Presets, Step::Customize],
    }
}
/// Each layer above `found`, bottom-up; every layer when none is published.
#[must_use]
pub fn plan(keys: Keys, found: Option<Layer>) -> Plan {
    let mut build = List::with_capacity(3);
    for layer in LAYERS {
        if let Some(found) = found
            && layer <= found
        {
            continue;
        }
        let steps = steps(layer);
        let mut list = List::with_capacity(MAX_STEPS);
        for step in steps {
            list.push(*step).expect("a layer's steps fit");
        }
        build.push(Planned { layer, key: keys.of(layer), steps: list.into_boxed() }).expect("three layers");
    }
    Plan { keys, found, build: build.into_boxed(), how: How::Scratch }
}
impl Plan {
    /// The first step of the lowest layer to build, if any.
    #[must_use]
    pub fn first(&self) -> Option<Step> {
        self.build.first()?.steps.first().copied()
    }
}
