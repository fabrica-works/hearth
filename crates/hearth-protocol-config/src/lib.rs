//! Strict configuration decoding (format.md, 2.2; render.md, 3.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod document;
mod error;
pub mod json;
pub mod limits;
pub mod names;
pub use error::{Error, Fault, Field, Shape};
pub use skein_json::tokenizer::Position;
#[cfg(test)]
mod tests;
