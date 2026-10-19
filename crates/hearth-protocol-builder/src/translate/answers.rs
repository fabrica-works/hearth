//! Render answers become the client's typed records (domain/render.md, 8.1).
use crate::{
    records::{outcome as target, parts::meta, records},
    render::{parts as source, revision, vocabulary::Outcome},
};
use alloc::boxed::Box;
use skein_lib::List;
#[must_use]
pub fn spec(spec: source::Spec) -> records::Spec {
    let source::Spec { meta, meta_digest, commit, digest, parts, resolution } = spec;
    let mut translated = List::with_capacity(u32::try_from(parts.len()).expect("bounded parts"));
    for part in parts {
        translated.push(super::parts::part(part)).expect("part capacity");
    }
    records::Spec {
        meta: metadata(meta),
        meta_digest: meta_digest.0,
        commit: commit.0,
        digest: digest.0,
        parts: translated.into_boxed(),
        resolution: resolved(resolution),
    }
}
#[must_use]
pub fn metadata(meta: source::meta::Meta) -> meta::Meta {
    let source::meta::Meta { target, base, member, environment, placement } = meta;
    let source::meta::Placement { kind, firmware, hardware, on } = placement;
    meta::Meta {
        target,
        base,
        member,
        environment,
        placement: meta::Placement { kind: runs(kind), firmware: optional_firmware(firmware), hardware, on },
    }
}
fn optional_firmware(firmware: Option<revision::Firmware>) -> Option<meta::Firmware> {
    Some(boot(firmware?))
}
#[must_use]
pub fn runs(kind: revision::Runs) -> meta::PlacementKind {
    match kind {
        revision::Runs::Metal => meta::PlacementKind::Metal,
        revision::Runs::Vm => meta::PlacementKind::Vm,
        revision::Runs::Container => meta::PlacementKind::Container,
    }
}
#[must_use]
pub fn boot(firmware: revision::Firmware) -> meta::Firmware {
    match firmware {
        revision::Firmware::Efi => meta::Firmware::Efi,
        revision::Firmware::Bios => meta::Firmware::Bios,
    }
}
#[must_use]
pub fn outcome(outcome: Outcome) -> target::Outcome {
    match outcome {
        Outcome::Usage => target::Outcome::Usage,
        Outcome::Failed => target::Outcome::Failed,
        Outcome::TryAgain => target::Outcome::TryAgain,
        Outcome::Negative => target::Outcome::Negative,
        Outcome::Done => target::Outcome::Done,
    }
}

fn resolved(resolution: Option<Box<crate::render::keys::Resolution>>) -> Option<records::Resolution> {
    let resolution = resolution?;
    Some(match *resolution {
        crate::render::keys::Resolution::Needed { key } => records::Resolution::Needed { key: key.0 },
        crate::render::keys::Resolution::Known { key, keys } => records::Resolution::Known {
            key: key.0,
            keys: records::Keys { packages: keys.packages.0, software: keys.software.0, config: keys.config.0 },
        },
    })
}

/// A build's layer, by kind, key and state (client/README.md, 3.1).
#[must_use]
pub fn layer(
    layer: crate::builder::build::Layer,
    key: crate::render::vocabulary::Digest,
    state: crate::builder::vocabulary::State,
) -> crate::records::build::Layer {
    use crate::{builder::vocabulary::State, records::build};
    build::Layer {
        kind: kind(layer),
        key: key.0,
        state: match state {
            State::Found => build::State::Found,
            State::ToBuild => build::State::ToBuild,
        },
    }
}
fn kind(layer: crate::builder::build::Layer) -> crate::records::build::Kind {
    use crate::{builder::build::Layer, records::build::Kind};
    match layer {
        Layer::Packages => Kind::Packages,
        Layer::Software => Kind::Software,
        Layer::Config => Kind::Config,
    }
}
/// The step a build ended on, named with its layer (domain/build.md, 3).
#[must_use]
pub fn failed(
    step: crate::builder::build::Step,
    why: crate::builder::build::vocabulary::Why,
) -> crate::records::build::Failed {
    use crate::{
        builder::build::{Layer, Step, vocabulary::Why},
        records::build,
    };
    let (layer, step) = match step {
        Step::Fetch(layer) => (layer, build::Step::Fetch),
        Step::Bootstrap => (Layer::Packages, build::Step::Mmdebstrap),
        Step::Verify => (Layer::Packages, build::Step::Verify),
        Step::Place => (Layer::Software, build::Step::Place),
        Step::Software => (Layer::Software, build::Step::Software),
        Step::Users => (Layer::Config, build::Step::Users),
        Step::Files => (Layer::Config, build::Step::Tree),
        Step::Tmpfiles => (Layer::Config, build::Step::Tmpfiles),
        Step::Presets => (Layer::Config, build::Step::Presets),
        Step::Customize => (Layer::Config, build::Step::Customize),
    };
    build::Failed {
        layer: kind(layer),
        step,
        why: match why {
            Why::Unavailable => build::Why::Unavailable,
        },
    }
}
/// The store full at a limit, as an operational error of kind store (domain/store.md, 8.1).
#[must_use]
pub fn full(limit: crate::builder::store::vocabulary::Limit, most: u32, store: &[u8]) -> crate::records::errors::Error {
    use crate::{
        builder::store::vocabulary::Limit,
        records::errors::{Class, Error, Kind, StoreError, StoreFault, StoreLimit},
    };
    let limit = match limit {
        Limit::Resolutions => StoreLimit::Resolutions,
        Limit::Layers => StoreLimit::Layers,
        Limit::CacheFiles => StoreLimit::CacheFiles,
    };
    Error {
        side: None,
        kind: Kind::Store(StoreError {
            store: skein_lib::bytes::copy_of(store),
            fault: StoreFault::Full { limit, most },
        }),
        class: Class::Operational,
        at: None,
        also: None,
        target: None,
        task: None,
    }
}
