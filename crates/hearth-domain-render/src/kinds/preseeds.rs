//! A single debconf answer and an initramfs module (format.md, 5.3, 5.4).
use crate::{
    entries::Concrete,
    errors::{Format, Schema},
    parts::packages::{Preseed, PreseedKind},
    revision::{Position, declaration::Name},
};
use skein_lib::{List, bytes::copy_of};
fn at(entry: &Concrete, name: Name) -> Position {
    for field in &entry.fields {
        if field.name == name {
            return field.at;
        }
    }
    entry.at
}
fn segment(value: &[u8]) -> bool {
    if value.is_empty() {
        return false;
    }
    for byte in value {
        if !(byte.is_ascii_alphanumeric() || b"._+-".contains(byte)) {
            return false;
        }
    }
    true
}
pub fn entry(entry: &Concrete) -> Result<Preseed, (Schema, Position)> {
    let malformed = Schema::Malformed { format: Format::Preseed };
    let mut start = 0;
    let mut first = None;
    for end in 0..=entry.key.len() {
        if end == entry.key.len() || entry.key.get(end) == Some(&b'/') {
            if !segment(entry.key.get(start..end).expect("segment")) {
                return Err((malformed, entry.at));
            }
            if first.is_none() && end != entry.key.len() {
                first = entry.key.get(..end);
            }
            start = end.saturating_add(1);
        }
    }
    let Some(owner) = first else {
        return Err((malformed, entry.at));
    };
    for name in [Name::Type, Name::Value] {
        if let Err(fault) = super::required(entry, name) {
            return Err((fault, entry.at));
        }
    }
    let kind = entry.text(Name::Type).expect("checked text field");
    let mut selected = None;
    let mut listed = List::with_capacity(4);
    for option in [PreseedKind::Select, PreseedKind::Multiselect, PreseedKind::String, PreseedKind::Boolean] {
        listed.push(copy_of(option.name())).expect("four types");
        if option.name() == kind {
            selected = Some(option);
        }
    }
    let Some(kind) = selected else {
        return Err((Schema::Enum { value: copy_of(kind), listed: listed.into_boxed() }, at(entry, Name::Type)));
    };
    let value = entry.text(Name::Value).expect("checked text field");
    for byte in value {
        if byte.is_ascii_control() {
            return Err((malformed, at(entry, Name::Value)));
        }
    }
    if value.ends_with(b"\\") || (kind == PreseedKind::Boolean && value != b"true" && value != b"false") {
        return Err((malformed, at(entry, Name::Value)));
    }
    let mut owner = owner;
    if let Some(written) = entry.text(Name::Owner) {
        if !segment(written) {
            return Err((malformed, at(entry, Name::Owner)));
        }
        owner = written;
    }
    Ok(Preseed { question: entry.key.clone(), owner: copy_of(owner), kind, value: copy_of(value) })
}
pub fn module(entry: &Concrete) -> Result<(), (Schema, Position)> {
    let fault = (Schema::Malformed { format: Format::Module }, entry.at);
    if !(1..=55).contains(&entry.key.len()) {
        return Err(fault);
    }
    for byte in &entry.key {
        if !(byte.is_ascii_alphanumeric() || b"_-".contains(byte)) {
            return Err(fault);
        }
    }
    Ok(())
}
