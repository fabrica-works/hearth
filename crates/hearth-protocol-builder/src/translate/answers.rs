//! Render answers become the client's typed records (render.md, 8.1).
use crate::{
    records::{outcome as target, parts::meta, records},
    render::{parts as source, revision, vocabulary::Outcome},
};
use skein_lib::List;
#[must_use]
pub fn spec(spec: source::Spec) -> records::Spec {
    let source::Spec { meta, meta_digest, commit, digest, parts } = spec;
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
        resolution: None,
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
