//! Live declarations (format.md, 5.14).
use super::secrets::{fault, formed, required};
use crate::{
    entries::Located,
    errors::{Error, Format, Schema},
    parts::live::{Action, Consumer},
    revision::declaration::Name,
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
pub(crate) fn entry(located: &Located<'_>) -> Result<Consumer, Box<Error>> {
    match check(located) {
        Ok(consumer) => Ok(consumer),
        Err((name, schema)) => Err(Box::new(fault(located, Some(name), schema))),
    }
}
fn check(located: &Located<'_>) -> Result<Consumer, (Name, Schema)> {
    let entry = located.entry;
    let value = match required(entry, Name::Action) {
        Ok(value) => value,
        Err(schema) => return Err((Name::Action, schema)),
    };
    let action = match value {
        b"reload" => Action::Reload,
        b"restart" => Action::Restart,
        b"reload-or-restart" => Action::ReloadOrRestart,
        b"start" => Action::Start,
        b"none" => Action::None,
        value => {
            return Err((
                Name::Action,
                Schema::Enum {
                    value: copy_of(value),
                    listed: Box::new([
                        copy_of(b"reload"),
                        copy_of(b"restart"),
                        copy_of(b"reload-or-restart"),
                        copy_of(b"start"),
                        copy_of(b"none"),
                    ]),
                },
            ));
        }
    };
    let unit = entry.text(Name::Unit);
    if unit.is_none() && action != Action::None {
        return Err((Name::Unit, Schema::MissingField { field: copy_of(b"unit") }));
    }
    if let Some(unit) = unit
        && !unit_name(unit)
    {
        return Err((Name::Unit, Schema::Malformed { format: Format::Key }));
    }
    let paths = entry.strings(Name::Paths).unwrap_or(&[]);
    if paths.is_empty() {
        return Err((Name::Paths, Schema::MissingField { field: copy_of(b"paths") }));
    }
    for (index, path) in paths.iter().enumerate() {
        let checked = path.strip_suffix(b"/").unwrap_or(path);
        if let Err(schema) = formed(checked, Format::Path) {
            return Err((Name::Paths, schema));
        }
        if !(checked.starts_with(b"/etc/") || checked.starts_with(b"/usr/local/") || checked.starts_with(b"/opt/")) {
            return Err((
                Name::Paths,
                Schema::Beneath { allowed: Box::new([copy_of(b"/etc"), copy_of(b"/usr/local"), copy_of(b"/opt")]) },
            ));
        }
        if paths.get(..index).expect("earlier paths").contains(path) {
            return Err((Name::Paths, Schema::Malformed { format: Format::Path }));
        }
    }
    let mut owned = None;
    if let Some(unit) = unit {
        owned = Some(copy_of(unit));
    }
    Ok(Consumer { task: copy_of(located.task), unit: owned, action })
}
fn unit_name(unit: &[u8]) -> bool {
    if unit.len() > 255 {
        return false;
    }
    let mut dot = None;
    for (index, byte) in unit.iter().enumerate() {
        if !byte.is_ascii_alphanumeric() && !b":_.@-\\".contains(byte) {
            return false;
        }
        if *byte == b'.' {
            dot = Some(index);
        }
    }
    let Some(dot) = dot else {
        return false;
    };
    dot > 0
        && [
            &b"service"[..],
            b"socket",
            b"target",
            b"device",
            b"mount",
            b"automount",
            b"swap",
            b"timer",
            b"path",
            b"slice",
            b"scope",
        ]
        .contains(&unit.get(dot.saturating_add(1)..).expect("suffix"))
}
