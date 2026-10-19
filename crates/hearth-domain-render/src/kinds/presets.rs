//! Unit presets (protocol/format.md, 5.6).
use super::secrets::{fault, required};
use crate::{
    entries::Located,
    errors::{Error, Schema},
    parts::config::{Action, Preset},
    revision::declaration::Name,
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
pub(crate) fn entry(located: &Located<'_>) -> Result<Preset, Box<Error>> {
    let action = match required(located.entry, Name::Action) {
        Ok(b"enable") => Action::Enable,
        Ok(b"disable") => Action::Disable,
        Ok(value) => {
            return Err(Box::new(fault(
                located,
                Some(Name::Action),
                Schema::Enum { value: copy_of(value), listed: Box::new([copy_of(b"enable"), copy_of(b"disable")]) },
            )));
        }
        Err(schema) => return Err(Box::new(fault(located, None, schema))),
    };
    Ok(Preset { unit: located.entry.key.clone(), action })
}
