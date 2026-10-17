//! The store's bounds and lock pace (store.md, 8.1, 8.2).
use skein_lib::Duration;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub resolutions: u32,
    pub cache_files: u32,
    pub page: u32,
    pub retry: Duration,
    pub said: Duration,
}
pub const SHIPPED: Limits = Limits {
    resolutions: 1_024,
    cache_files: 262_144,
    page: 1_024,
    retry: Duration::from_millis(200),
    said: Duration::from_secs(1),
};
/// The deadline table; the index and page join with the startup scan.
#[must_use]
pub fn worst_case(_limits: &Limits) -> Option<u64> {
    skein_lib::Deadlines::<crate::Timer>::worst_case(1)
}
