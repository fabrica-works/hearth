//! Expressions compiled to postfix operations (format.md, 7.4; render.md, 8.3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod error;
pub mod expression;
pub mod limits;
pub mod operation;
pub use error::{Error, ErrorKind, Expected};
pub use operation::{Expression, Op, Operation};
#[cfg(test)]
mod tests;
