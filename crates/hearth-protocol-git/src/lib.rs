//! Git vectors and stream machines (render.md, 2.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod invocation;
pub mod oid;
pub mod rev_parse;
#[cfg(test)]
mod tests;
