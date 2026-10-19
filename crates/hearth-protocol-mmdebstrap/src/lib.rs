//! Resolution's mmdebstrap machine (domain/render.md, 5.2 to 5.5).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod limits;
pub mod lists;
pub mod output;
pub mod packages;
pub mod tail;
pub mod vector;
/// Maximum emissions into the caller's reserved queues.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaxOut {
    pub above: u32,
    pub below: u32,
}
#[cfg(test)]
mod tests;
