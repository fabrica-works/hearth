//! The machine's generic Linux half (testing.md, 3.1 and 3.2).
#![forbid(unsafe_code)]
extern crate alloc;
pub mod hearth;
pub mod linux;
pub use hearth::Machine;
#[cfg(test)]
mod tests;
