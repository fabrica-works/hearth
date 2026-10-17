//! Strict configuration decoding (format.md, 2.2; render.md, 3.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod declaration;
pub mod document;
pub mod entries;
mod error;
pub mod json;
pub mod limits;
pub mod names;
pub use error::{Declared, Error, Expected, Fault, Field, Object, Required, Shape};
pub use skein_json::tokenizer::Position;
#[cfg(test)]
mod tests;
