#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod cache_file;
pub mod decode;
mod fields;
pub mod frame;
pub mod layer;
pub mod limits;
pub mod resolution;
pub mod store;

#[cfg(test)]
mod tests;
