//! The pinned local mirror for test tiers (testing.md, 6.2).
pub mod mirror;
pub mod pin;
pub use mirror::{CACHE, Mirror, Missing};
pub use pin::{Pin, PinError, Target, Wrong};
#[cfg(test)]
mod tests;
