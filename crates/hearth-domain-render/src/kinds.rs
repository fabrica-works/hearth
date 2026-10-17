//! Shared requirements of concrete declaration kinds (format.md, 5.2).
pub mod identity;
pub mod secrets;

pub mod files;
use crate::{entries::Concrete, errors::Schema, revision::declaration::Name};
pub fn required(entry: &Concrete, name: Name) -> Result<(), Schema> {
    for field in &entry.fields {
        if field.name == name {
            return Ok(());
        }
    }
    Err(Schema::MissingField { field: skein_lib::bytes::copy_of(name.key()) })
}

pub mod packages;
