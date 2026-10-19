//! Git vectors and stream machines (domain/render.md, 2.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod invocation;
pub mod oid;
pub mod rev_parse;
#[cfg(test)]
mod tests;

pub mod limits;
pub mod listing;
/// Maximum emissions reserved before calling a machine entry point.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaxOut {
    pub above: u32,
    pub below: u32,
}

pub mod batch;

pub mod hunks;
pub mod tail;
