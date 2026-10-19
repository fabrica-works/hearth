//! The canonical encoding of hearth's values (domain/store.md, 3.1), and a tree entry's path.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod encode;
pub mod hash;
pub mod path;

#[cfg(test)]
mod tests;
