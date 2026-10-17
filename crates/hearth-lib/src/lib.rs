//! The canonical encoding of hearth's values (store.md, 3.1).

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod encode;
pub mod hash;

#[cfg(test)]
mod tests;
