use super::{Repository, settings};
use hearth_domain_render::limits::Limits;
// A KiB per shipped MiB: the fleet holds several specs, then rejects each excess.
pub const SPEC_BYTES: u64 = 65_536;
pub const LIMITS: Limits = Limits { specs: SPEC_BYTES, ..settings::LIMITS };
pub const MEMORY_SEED: u64 = 0x28_54;
#[must_use]
pub fn repository() -> Repository {
    settings::every_target()
}
#[must_use]
pub fn bound() -> u64 {
    hearth_domain_render::limits::worst_case(&LIMITS).expect("specs bound")
}
