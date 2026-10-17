#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod decode;
pub mod frame;
pub mod limits;

#[cfg(test)]
mod tests;
