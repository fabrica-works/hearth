//! Counts and repository setup from render session 30's test shapes.
use crate::{
    parent::Run,
    repositories::{self, Edit, Generated, Shape},
};
use hearth_domain_render::{errors::Error, vocabulary::Answer};
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
/// Build each focused case once at its seed.
pub fn cases() -> impl Iterator<Item = (u64, Generated)> {
    CASES.map(|case| (case, generated(case)))
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
    for (case, generated) in cases() {
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
/// One broken base removes its two environments from SMALL's six targets.
pub const HEALTHY_TARGETS: u32 = LIMITS.targets - LIMITS.targets / Shape::SMALL.bases;
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
        let edits = edits();
        let index = usize::try_from(rng.below(u64::try_from(edits.len()).expect("edits"))).expect("edit");
        let edit = &edits[index];
        let edited = repositories::edit(&mut rng, &generated, edit);
        assert_eq!(
            changed(&digests, &self::digests(&run(&edited, case))),
            repositories::touched(&generated, edit),
            "case {case}: {edit:?}"
        );
        let isolated = isolated(&generated, case);
        mutation_property(&isolated, &repositories::mutate(&mut rng, &isolated), case);
    }
}

/// Named parties of 30.2's production and environment edits.
pub const PROD_TARGET: &str = "b0-prod";
pub const LAB: &str = "lab";
/// Order tests cover both an explicit reverse and a seeded permutation.
pub const ORDERS: &[Order] = &[Order::Reverse, Order::Shuffle];
#[derive(Clone, Copy, Debug)]
pub enum Order {
    Reverse,
    Shuffle,
}
#[must_use]
pub fn edited(generated: &Generated, edit: &Edit, seed: u64) -> Generated {
    repositories::edit(&mut Rng::new(SEED + seed), generated, edit)
}
#[must_use]
pub fn changed(before: &[(String, Digest)], after: &[(String, Digest)]) -> Vec<String> {
    let mut names: Vec<_> = before
        .iter()
        .chain(after)
        .filter(|entry| !before.contains(entry) || !after.contains(entry))
        .map(|(name, _)| name.clone())
        .collect();
    names.sort();
    names.dedup();
    names
}
pub fn locality(edit: &Edit) {
    for (case, generated) in cases() {
        let before = digests(&run(&generated, case));
        let after = digests(&run(&edited(&generated, edit, case), case));
        assert_eq!(changed(&before, &after), repositories::touched(&generated, edit), "case {case}: {edit:?}");
    }
}
#[must_use]
pub fn environment<T: Clone>(digests: &[(String, T)], environment: &str) -> Vec<(String, T)> {
    digests.iter().filter(|(name, _)| name.ends_with(&format!("-{environment}"))).cloned().collect()
}
#[must_use]
pub fn mutations(case: u64) -> (Generated, Generated) {
    let generated = isolated(&generated(case), case);
    let mutated = repositories::mutate(&mut Rng::new(SEED + case), &generated);
    (generated, mutated)
}
/// Ruling 197 isolates base-file faults from the separate peer-export property.
#[must_use]
pub fn isolated(generated: &Generated, case: u64) -> Generated {
    repositories::without_peers(&mut Rng::new(SEED + case), generated)
}
#[must_use]
pub fn errors(run: &Run) -> Vec<&Error> {
    run.answers
        .iter()
        .filter_map(|answer| match answer {
            Answer::Error(error) => Some(error),
            Answer::Spec(_) | Answer::Change(_) | Answer::Rendered { .. } => None,
        })
        .collect()
}
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ErrorOrder<'a> {
    unplaced: bool,
    path: &'a [u8],
    position: (u32, u32),
    target: Option<&'a [u8]>,
}
#[must_use]
pub fn error_key(error: &Error) -> ErrorOrder<'_> {
    ErrorOrder {
        unplaced: error.at.is_none(),
        path: error.at.as_ref().map_or(b"".as_slice(), |at| at.path.as_ref()),
        position: error.at.as_ref().and_then(|at| at.position).map_or((0, 0), |at| (at.line, at.column)),
        target: error.target.as_deref(),
    }
}
#[must_use]
pub fn ordered_targets(generated: &Generated, order: Order, seed: u64) -> Vec<&[u8]> {
    let mut names: Vec<_> = generated.model.targets.iter().map(|target| target.name.as_bytes()).collect();
    match order {
        Order::Reverse => names.reverse(),
        Order::Shuffle => {
            let mut rng = Rng::new(SEED + seed);
            for index in (1..names.len()).rev() {
                let other = usize::try_from(rng.below(u64::try_from(index + 1).expect("targets"))).expect("target");
                names.swap(index, other);
            }
        }
    }
    names
}
/// Every generated target meets the same two task claims (30.2).
pub const COLLISIONS: &[(&str, &str, &str)] = &[
    ("b0-lab", "tasks/dup_a/tree/etc/dup", "tasks/dup_b/tree/etc/dup"),
    ("b0-prod", "tasks/dup_a/tree/etc/dup", "tasks/dup_b/tree/etc/dup"),
    ("b1-lab", "tasks/dup_a/tree/etc/dup", "tasks/dup_b/tree/etc/dup"),
    ("b1-prod", "tasks/dup_a/tree/etc/dup", "tasks/dup_b/tree/etc/dup"),
    ("b2-lab", "tasks/dup_a/tree/etc/dup", "tasks/dup_b/tree/etc/dup"),
    ("b2-prod", "tasks/dup_a/tree/etc/dup", "tasks/dup_b/tree/etc/dup"),
];
#[must_use]
pub fn collision_places(run: &Run) -> Vec<(&str, &str, &str)> {
    errors(run)
        .into_iter()
        .map(|error| {
            (
                std::str::from_utf8(error.target.as_deref().expect("target")).expect("target text"),
                std::str::from_utf8(&error.at.as_ref().expect("at").path).expect("path text"),
                std::str::from_utf8(&error.also.as_ref().expect("also").path).expect("path text"),
            )
        })
        .collect()
}
pub fn mutation_property(generated: &Generated, mutated: &Generated, case: u64) {
    let names = repositories::healthy(mutated);
    assert_eq!(names.len(), usize::try_from(HEALTHY_TARGETS).expect("targets"), "case {case}: model survivors");
    let before: Vec<_> = digests(&run(generated, case)).into_iter().filter(|(name, _)| names.contains(name)).collect();
    let after = run(mutated, case);
    let digests: Vec<_> = self::digests(&after).into_iter().filter(|(name, _)| names.contains(name)).collect();
    assert_eq!(digests, before, "case {case}: healthy targets");
}

#[must_use]
pub fn edits() -> Vec<Edit> {
    let mut edits = REPRESENTATIONS.to_vec();
    edits.extend([
        Edit::Unread(PROD_TARGET.into()),
        Edit::Own(PROD_TARGET.into()),
        Edit::Exported(PROD_TARGET.into()),
        Edit::Task,
        Edit::Env(LAB.into()),
        Edit::Collide,
    ]);
    edits
}
/// The three production-only setting edits named in 30.2.
#[must_use]
pub fn production_edits() -> [Edit; 3] {
    [Edit::Unread(PROD_TARGET.into()), Edit::Own(PROD_TARGET.into()), Edit::Exported(PROD_TARGET.into())]
}

/// The twelve fleets over eight arrival orders, shared across three bounded tests.
pub fn collision_order(batch: u64) {
    let start = batch * CASES_PER_BATCH;
    for case in start..start + CASES_PER_BATCH {
        let generated = edited(&generated(case), &Edit::Collide, case);
        let repository = super::repository_of(&generated.leaves);
        let run = render(&repository, &[], 0);
        assert_eq!(collision_places(&run), COLLISIONS, "case {case}");
        for seed in SCHEDULES {
            assert_eq!(errors(&render(&repository, &[], seed)), errors(&run), "case {case}: seed {seed}");
        }
    }
}
