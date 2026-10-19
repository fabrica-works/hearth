//! Kernel parameters safe in GRUB's script (protocol/format.md, 5.4).
use crate::{
    entries::Concrete,
    errors::{Format, Schema},
    revision::{Position, declaration::Name},
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
#[derive(Debug)]
pub struct Parameter {
    pub order: u16,
    pub param: Box<[u8]>,
}
pub fn entry(entry: &Concrete) -> Result<Parameter, (Schema, Position)> {
    if let Err(fault) = super::required(entry, Name::Param) {
        return Err((fault, entry.at));
    }
    let param = entry.text(Name::Param).expect("checked text field");
    let mut at = entry.at;
    for field in &entry.fields {
        if field.name == Name::Param {
            at = field.at;
        }
    }
    let malformed = (Schema::Malformed { format: Format::Param }, at);
    if param.is_empty() {
        return Err(malformed);
    }
    for byte in param {
        if !(0x21..=0x7e).contains(byte) || b"\"'\\$;&|<>{}#".contains(byte) {
            return Err(malformed);
        }
    }
    let order = entry.int(Name::Order).unwrap_or(500);
    for field in &entry.fields {
        if field.name == Name::Order {
            at = field.at;
        }
    }
    if order < 0 {
        return Err((Schema::Below { min: 0 }, at));
    }
    if order > 999 {
        return Err((Schema::Above { max: 999 }, at));
    }
    Ok(Parameter { order: u16::try_from(order).expect("checked order"), param: copy_of(param) })
}
