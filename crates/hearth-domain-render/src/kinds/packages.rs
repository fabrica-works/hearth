//! Debian Policy 5.6.1 names and format.md 5.3 components.
use crate::{
    entries::Concrete,
    errors::{Format, Schema},
    parts::packages::Component,
    revision::{Position, declaration::Name},
};
use skein_lib::{List, bytes};
pub fn entry(entry: &Concrete) -> Result<Component, (Schema, Position)> {
    let malformed = (Schema::Malformed { format: Format::Debian }, entry.at);
    let Some((first, rest)) = entry.key.split_first() else { return Err(malformed) };
    if !(2..=256).contains(&entry.key.len()) || !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return Err(malformed);
    }
    for byte in rest {
        if !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"+-.".contains(byte)) {
            return Err(malformed);
        }
    }
    let Some(value) = entry.text(Name::Component) else { return Ok(Component::Main) };
    for component in Component::ALL {
        if component.name() == value {
            return Ok(component);
        }
    }
    let mut listed = List::with_capacity(4);
    for component in Component::ALL {
        listed.push(bytes::copy_of(component.name())).expect("four components");
    }
    let mut at = entry.at;
    for field in &entry.fields {
        if field.name == Name::Component {
            at = field.at;
        }
    }
    Err((Schema::Enum { value: bytes::copy_of(value), listed: listed.into_boxed() }, at))
}
