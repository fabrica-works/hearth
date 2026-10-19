//! The store's bounds and lock pace (store.md, 8.1, 8.2).
use skein_lib::Duration;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub resolutions: u32,
    pub layers: u32,
    pub cache_files: u32,
    pub page: u32,
    pub retry: Duration,
    pub said: Duration,
}
pub const SHIPPED: Limits = Limits {
    resolutions: 1_024,
    layers: 4_096,
    cache_files: 262_144,
    page: 1_024,
    retry: Duration::from_millis(200),
    said: Duration::from_secs(1),
};
/// The indices, one page of names, and both lock deadlines.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    use crate::vocabulary::Entry;
    let index = crate::index::Index::worst_case(limits.cache_files.checked_mul(2)?)?;
    let page = u64::from(limits.page).checked_mul(u64::try_from(size_of::<Entry>()).ok()?.checked_add(255)?)?;
    index
        .checked_add(skein_lib::Map::<hearth_lib::encode::Digest, ()>::worst_case(limits.resolutions)?)?
        .checked_add(skein_lib::Map::<hearth_lib::encode::Digest, crate::vocabulary::Layer>::worst_case(
            limits.layers,
        )?)?
        .checked_add(page)?
        .checked_add(255)?
        .checked_add(skein_lib::Deadlines::<crate::Timer>::worst_case(2)?)
}
