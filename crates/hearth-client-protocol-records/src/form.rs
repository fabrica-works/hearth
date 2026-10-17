//! Sized record forms (client/README.md, 3.2).
use crate::{
    limits::Limits,
    records::{Record, Type},
};
use alloc::boxed::Box;
pub(crate) mod json;
pub(crate) mod text;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Form {
    Text,
    JsonLines,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    TooLong,
    Text,
    JsonLines,
}
/// Prints a record after the preceding record's type, if any.
pub fn print(form: Form, record: &Record, after: Option<Type>, limits: &Limits) -> Result<Box<[u8]>, Refusal> {
    match form {
        Form::Text => text::print(record, after, limits.record),
        Form::JsonLines => json::print(record, after, limits.record),
    }
}
