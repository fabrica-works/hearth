//! Configured render bounds (domain/render.md, 2.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub closure: u32,
    pub resolve: skein_lib::Duration,
    pub targets: u32,
    pub errors: u32,
    pub error_bytes: u64,
    pub tasks: u32,
    pub decoded: u64,
    pub composed: u64,
    pub specs: u64,
    pub validated: u64,
    pub document: u32,
    pub template: u32,
    pub statements: u32,
    pub iterations: u32,
    pub rendered_file: u32,
    pub rendered_target: u64,
    pub depth: u32,
    pub expression: u32,
    pub nesting: u32,
    pub made: u64,
    pub static_file: u32,
    pub hunk_file: u32,
    pub expansion: u32,
    pub expanded: u64,
    pub exported: u64,
}
pub const SHIPPED: Limits = Limits {
    closure: 8_192,
    resolve: skein_lib::Duration::from_secs(600),
    targets: 1_024,
    errors: 256,
    error_bytes: 16_777_216,
    tasks: 1_024,
    decoded: 16_777_216,
    composed: 16_777_216,
    specs: 16_777_216,
    validated: 16_777_216,
    document: 262_144,
    template: 262_144,
    statements: 32,
    iterations: 1_048_576,
    rendered_file: 16_777_216,
    rendered_target: 268_435_456,
    depth: 32,
    expression: 4_096,
    nesting: 64,
    made: 16_777_216,
    static_file: 67_108_864,
    hunk_file: 1_048_576,
    expansion: 65_536,
    expanded: 16_777_216,
    exported: 16_777_216,
};

/// A literal that is five bytes cannot fit, for budget refusal tests.
pub const FOUR_MADE: Limits = Limits { made: 4, ..SHIPPED };

/// Retained revision, targets and resolution records, plus one evaluation's
/// scratch space (domain/render.md, 8.4; programming-model.md, 6.3).
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    use crate::{compose, diff, evaluate, revision, vocabulary};
    use alloc::boxed::Box;
    use skein_lib::{List, Map};
    // Render's domain receives the listing at domain/render.md, 8.4's bounds.
    const ENTRIES: u32 = 16_384;
    const PATHS: u64 = 4_194_304;
    let listing = List::<revision::Entry>::worst_case(ENTRIES)?
        .checked_add(List::<vocabulary::Blob>::worst_case(ENTRIES)?)?
        .checked_add(Map::<revision::Oid, hearth_lib::encode::Digest>::worst_case(ENTRIES)?)?
        .checked_add(Map::<revision::Oid, Box<[u8]>>::worst_case(ENTRIES)?)?
        .checked_add(Map::<revision::Oid, ()>::worst_case(ENTRIES)?)?
        .checked_add(Map::<hearth_lib::encode::Digest, u64>::worst_case(ENTRIES)?)?
        .checked_add(PATHS.checked_mul(6)?)?
        .checked_add(u64::from(ENTRIES).checked_mul(64 * 5 + 1)?)?;
    let decoded = revision::decoded_worst_case(limits)?;
    // Provenance enlarges the 32-byte value records. During a merge, the old
    // value, its patch, and the rebuilt value may coexist.
    let settings = compose::worst_case(limits)?;
    let targets = List::<compose::Target>::worst_case(limits.targets)?
        .checked_add(List::<Box<[compose::Index]>>::worst_case(limits.targets)?)?
        .checked_add(
            u64::from(limits.targets)
                .checked_mul(u64::from(limits.tasks).checked_add(1)?)?
                .checked_mul(u64::try_from(size_of::<compose::Index>()).ok()?)?,
        )?
        .checked_add(Map::<Box<[u8]>, diff::Made>::worst_case(limits.targets)?)?;
    // A find or record arrival holds one full record transiently; retained
    // specs, file sizes and diff closures share the command budget.
    // A frame byte can introduce one input item; package strings additionally
    // follow domain/store.md, 6.1's fixed M1 name/path limits.
    let record = u64::from(4_194_304_u32)
        .checked_mul(u64::try_from(size_of::<hearth_lib::encode::Item>()).ok()?)?
        .checked_add(List::<crate::resolution::Package>::worst_case(limits.closure.max(8_192))?)?
        .checked_add(u64::from(limits.closure.max(8_192)).checked_mul(3 * 256 + 4_096)?)?;
    let resolutions =
        Map::<hearth_lib::encode::Digest, Option<crate::call::resolution::Known>>::worst_case(limits.targets)?
            .checked_add(record)?
            .checked_add(limits.specs.checked_mul(4)?)?;
    let retained = listing
        .checked_add(decoded)?
        .checked_add(targets)?
        .checked_add(resolutions)?
        .checked_add(limits.validated)?
        .checked_add(limits.exported)?
        .checked_add(crate::errors::Collector::worst_case(limits)?)?;
    // Both sides share the specs allowance and move successful decoded blobs.
    // Expanded entries remain live during template execution. Composition
    // finishes before expansion; each phase prices its own overlapping buffers.
    let scratch = settings
        .max(crate::entries::worst_case(limits)?)
        .max(evaluate::worst_case(limits)?)
        .max(evaluate::template::worst_case(limits)?.checked_add(crate::entries::held_worst_case(limits)?)?);
    retained.checked_add(scratch)
}
