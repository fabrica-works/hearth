//! Seeded work durations for hosted tools (domain/testing.md, 4.1).
use skein_lib::{Duration, Rng};
use skein_world::domain::Span;
#[derive(Clone, Copy, Debug)]
pub struct Pace {
    pub git: Span,
    pub per_mib: Duration,
    pub mmdebstrap: Span,
    pub per_package: Duration,
}
impl Pace {
    #[must_use]
    pub const fn calm() -> Self {
        Self {
            git: Span::millis(1, 5),
            per_mib: Duration::from_millis(2),
            mmdebstrap: Span::millis(20, 80),
            per_package: Duration::from_millis(1),
        }
    }
    #[must_use]
    pub fn git(self, rng: &mut Rng, bytes: u64) -> Duration {
        self.git
            .draw(rng)
            .saturating_add(Duration::from_nanos(self.per_mib.as_nanos().saturating_mul(bytes).div_ceil(1 << 20)))
    }
    #[must_use]
    pub fn mmdebstrap(self, rng: &mut Rng, packages: u64) -> Duration {
        self.mmdebstrap
            .draw(rng)
            .saturating_add(Duration::from_nanos(self.per_package.as_nanos().saturating_mul(packages)))
    }
}
