//! Expressions compiled to postfix operations (format.md, 7.4; render.md, 8.3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod error;
pub mod expression;
pub mod filter;
pub mod instruction;
pub mod limits;
pub mod operation;
pub mod template;
mod utf8;
pub use error::{Error, ErrorKind, Expected};
pub use filter::Filter;
pub use instruction::{Does, Instruction, Names, Template};
pub use operation::{Expression, Missing, Op, Operation};
#[cfg(test)]
mod tests;
