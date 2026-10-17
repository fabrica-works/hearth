//! Git's deterministic objects and face (testing.md, 4.2 and 4.4).
#![forbid(unsafe_code)]
extern crate alloc;
pub mod diff;
pub mod objects;
pub mod repository;
#[cfg(test)]
mod tests;
