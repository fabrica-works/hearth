//! Counts and repository setup from render session 30's test shapes.
use crate::{
    parent::Run,
    repositories::{self, Edit, Generated, Shape},
};
use hearth_domain_render::vocabulary::Answer;
use hearth_lib::encode::Digest;
use skein_lib::Rng;
/// Twelve miniature fleets per focused property (30.1 and 30.2).
pub const CASES: core::ops::Range<u64> = 0..12;
/// The generated-repository sweep's 750 rounds (30.1).
pub const ROUNDS: u64 = 750;
/// Ruling 195 divides 750 rounds among fifteen independent tests.
pub const ROUNDS_PER_BATCH: u64 = ROUNDS / 15;
/// Ruling 195 divides twelve fleets among three determinism tests.
pub const CASES_PER_BATCH: u64 = 12 / 3;
/// The seven alternative arrival schedules in 30.1.
pub const SCHEDULES: core::ops::Range<u64> = 1..8;
/// Session 30's assigned seed block.
pub const SEED: u64 = 0x0030_0000;
/// The process shape asks for two repositories.
pub const PROCESS_CASES: core::ops::Range<u64> = 0..2;
pub const REPRESENTATIONS: &[Edit] = &[Edit::Note, Edit::Reorder, Edit::Respace];
#[must_use]
pub fn generated(case: u64) -> Generated {
    repositories::generate(&mut Rng::new(SEED + case), Shape::SMALL)
}
#[must_use]
pub fn digests(run: &Run) -> Vec<(String, Digest)> {
    let mut digests: Vec<_> = run
        .answers
        .iter()
        .filter_map(|answer| match answer {
            Answer::Spec(spec) => Some((String::from_utf8(spec.meta.target.to_vec()).expect("target"), spec.digest)),
            Answer::Error(_) | Answer::Change(_) | Answer::Rendered { .. } => None,
        })
        .collect();
    digests.sort_by(|a, b| a.0.cmp(&b.0));
    digests
}
#[must_use]
pub fn run(generated: &Generated, seed: u64) -> Run {
    render(&super::repository_of(&generated.leaves), &[], seed)
}
pub fn unchanged(edit: &Edit) {
    for case in CASES {
        let generated = generated(case);
        let edited = repositories::edit(&mut Rng::new(SEED + case), &generated, edit);
        assert_eq!(digests(&run(&edited, case)), digests(&run(&generated, case)), "case {case}: {edit:?}");
    }
}

/// Six targets and two tasks; two templates fit within a KiB.
pub const LIMITS: hearth_domain_render::limits::Limits = hearth_domain_render::limits::Limits {
    targets: 6,
    tasks: 4,
    template: 1024,
    statements: 4,
    expression: 256,
    nesting: 8,
    expansion: 32,
    made: 16_384,
    ..hearth_domain_render::limits::SHIPPED
};
#[must_use]
pub fn render(repository: &super::Repository, targets: &[&[u8]], seed: u64) -> Run {
    crate::parent::run_with(repository, targets, seed, LIMITS)
}

/// One of ruling 195's three `every_seed_gives_the_same_digests_<k>` batches.
pub fn deterministic(batch: u64) {
    let start = batch * CASES_PER_BATCH;
    for case in start..start + CASES_PER_BATCH {
        let generated = generated(case);
        let repository = super::repository_of(&generated.leaves);
        let digests = self::digests(&render(&repository, &[], 0));
        for seed in SCHEDULES {
            assert_eq!(self::digests(&render(&repository, &[], seed)), digests, "case {case}: seed {seed}");
        }
    }
}
/// One of ruling 195's fifteen `generated_repositories_and_edits_keep_every_property_<k>` batches.
pub fn sweep(batch: u64) {
    let start = batch * ROUNDS_PER_BATCH;
    for case in start..start + ROUNDS_PER_BATCH {
        let generated = generated(case);
        let digests = self::digests(&run(&generated, case));
        let mut rng = Rng::new(SEED + case);
        let index = usize::try_from(rng.below(u64::try_from(REPRESENTATIONS.len()).expect("edits"))).expect("edit");
        let edit = &REPRESENTATIONS[index];
        let edited = repositories::edit(&mut rng, &generated, edit);
        assert_eq!(self::digests(&run(&edited, case)), digests, "case {case}: {edit:?}");
    }
}
