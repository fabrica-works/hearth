//! Inputs and fold shared by session 30.3's merge properties.
use crate::{
    json::{self, Json},
    reference,
};
use hearth_domain_render::{
    compose::{Layer, Merge},
    limits,
    parts::{Part, PartRecord},
    vocabulary::Answer,
};
use skein_lib::Rng;
/// Thirty-two focused inputs retain varied records; the sweep keeps 13,000 rounds.
pub const CASES: core::ops::Range<u64> = 0..32;
pub const ROUNDS: core::ops::Range<u64> = 0..13_000;
pub const SEED: u64 = super::properties::SEED + 0x3000;
/// The focused shape draws to six levels; the sweep reaches eight.
pub const DEPTH: u32 = 6;
pub const WIDE_DEPTH: u32 = 8;
pub const EMPTY: Json = Json::Map(Vec::new());
/// One base and one selected target suffice to produce all eight part records.
pub const DIGEST_SHAPE: crate::repositories::Shape = crate::repositories::Shape { bases: 1 };
/// A scalar and an empty list exercise both whole-replacement forms.
pub const REPLACEMENTS: &[Json] = &[Json::Bool(true), Json::Int(-1), Json::List(Vec::new())];
#[must_use]
pub fn merged(patches: &[&Json]) -> Option<Json> {
    let mut merge = Merge::new(&limits::SHIPPED);
    let mut target = None;
    for (index, patch) in patches.iter().enumerate() {
        target = merge.merge(target, &json::value(patch), Layer::Base, u32::try_from(index).expect("few patches"));
    }
    target.as_ref().map(json::plain)
}
#[must_use]
pub fn patches(seed: u64, depth: u32) -> (Json, Json) {
    let mut rng = Rng::new(SEED + seed);
    (json::generate(&mut rng, depth), json::generate(&mut rng, depth))
}
pub fn cases() -> impl Iterator<Item = (u64, Json, Json)> {
    CASES.map(|seed| {
        let (target, patch) = patches(seed, DEPTH);
        (seed, target, patch)
    })
}
#[must_use]
pub fn object(value: Json) -> Json {
    json::object(&[("kept", value)])
}
pub fn reference_agrees(target: &Json, patch: &Json, seed: u64) {
    assert_eq!(merged(&[target, patch]), reference::merge(reference::merge(None, target), patch), "seed {seed}");
}
pub fn idempotent(target: &Json, patch: &Json, seed: u64) {
    assert_eq!(merged(&[target, patch, patch]), merged(&[target, patch]), "seed {seed}");
}
#[must_use]
pub fn has_null(value: &Json) -> bool {
    match value {
        Json::Null => true,
        Json::Bool(_) | Json::Int(_) | Json::String(_) => false,
        Json::List(values) => values.iter().any(has_null),
        Json::Map(entries) => entries.iter().any(|(_, value)| has_null(value)),
    }
}
#[must_use]
pub fn part_name(record: &PartRecord) -> &'static str {
    match record {
        PartRecord::Config(_) => "config",
        PartRecord::Data(_) => "data",
        PartRecord::Secrets(_) => "secrets",
        PartRecord::Software(_) => "software",
        PartRecord::Packages(_) => "packages",
        PartRecord::Live(_) => "live",
        PartRecord::Boot(_) => "boot",
        PartRecord::Exports(_) => "exports",
    }
}
#[must_use]
pub fn parts() -> std::collections::BTreeMap<&'static str, Vec<Part>> {
    let mut parts = std::collections::BTreeMap::<_, Vec<_>>::new();
    for seed in CASES {
        let generated = crate::repositories::generate(&mut Rng::new(SEED + seed), DIGEST_SHAPE);
        let repository = super::repository_of(&generated.leaves);
        let run = super::properties::render(&repository, &[super::properties::PROD_TARGET.as_bytes()], seed);
        assert!(super::properties::errors(&run).is_empty(), "seed {seed}");
        assert_eq!(
            super::digests(&run).iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(),
            [super::properties::PROD_TARGET]
        );
        for answer in run.answers {
            if let Answer::Spec(spec) = answer {
                for part in spec.parts {
                    let records = parts.entry(part_name(&part.record)).or_default();
                    if !records.contains(&part) {
                        records.push(part);
                    }
                }
            }
        }
    }
    parts
}
