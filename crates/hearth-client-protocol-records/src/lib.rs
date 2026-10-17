//! Typed client records and their forms (client/README.md, 3.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod diff;
pub mod errors;
pub mod form;
pub mod limits;
pub mod outcome;
pub mod parts;
pub mod printer;
pub mod progress;
pub mod records;
pub mod startup;
/// The client protocol's major version (client/README.md, 3.2).
pub const CLIENT_VERSION: u32 = 1;
#[cfg(test)]
mod tests;
