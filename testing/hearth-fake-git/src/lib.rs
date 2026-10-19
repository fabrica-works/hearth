//! Git's deterministic objects and face (domain/testing.md, 4.2 and 4.4).
#![forbid(unsafe_code)]
extern crate alloc;
pub mod diff;
pub mod face;
pub mod objects;
pub mod process;
pub mod repository;
#[cfg(test)]
mod tests;
