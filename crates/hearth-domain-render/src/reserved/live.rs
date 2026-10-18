//! Consumers may not act on boot configuration or core services (format.md, 5.14).
use super::{Installed, Refused, refused};
use crate::{
    entries::{Expansion, Located},
    errors::Rule,
    revision::declaration::{Map, Name},
};
use alloc::boxed::Box;
use skein_lib::List;
fn unit(unit: &[u8]) -> bool {
    let name = unit.strip_suffix(b".service").unwrap_or(unit);
    name.starts_with(b"systemd-")
        || name.starts_with(b"hearth-")
        || name.starts_with(b"getty@")
        || name.starts_with(b"serial-getty@")
        || [b"dbus".as_slice(), b"dbus.socket", b"NetworkManager", b"console-getty", b"apparmor"].contains(&name)
}
fn path(path: &[u8], expansion: &Expansion, installed: &[Installed<'_>]) -> bool {
    let path = path.strip_suffix(b"/").unwrap_or(path);
    if !(path.starts_with(b"/etc/") || path.starts_with(b"/usr/local/") || path.starts_with(b"/opt/")) {
        return false;
    }
    if super::paths::never(path) {
        return true;
    }
    for install in installed {
        if install.task == b"hearth" && super::paths::beneath(path, install.path) {
            return true;
        }
    }
    for download in expansion.of(Map::Software) {
        if download.task == b"hearth"
            && let Some(root) = download.entry.text(Name::Path)
            && super::paths::beneath(path, root)
        {
            return true;
        }
    }
    false
}
fn entry(row: &Located<'_>, expansion: &Expansion, installed: &[Installed<'_>]) -> Option<Refused> {
    for field in &row.entry.fields {
        if field.name == Name::Unit
            && let Some(name) = row.entry.text(Name::Unit)
        {
            if row.entry.text(Name::Action) == Some(b"none") {
                return Some(refused(row, field.at, Rule::LiveNone, name));
            }
            if unit(name) {
                return Some(refused(row, field.at, Rule::NeverLiveUnit, name));
            }
        }
    }
    for field in &row.entry.fields {
        if field.name == Name::Paths {
            for name in row.entry.strings(Name::Paths).unwrap_or(&[]) {
                if path(name, expansion, installed) {
                    return Some(refused(row, field.at, Rule::NeverLivePath, name));
                }
            }
        }
    }
    None
}
pub(super) fn live(expansion: &Expansion, installed: &[Installed<'_>]) -> Box<[Refused]> {
    let entries = expansion.of(Map::Live);
    let mut faults = List::with_capacity(u32::try_from(entries.len()).expect("entry bound"));
    for row in entries {
        if let Some(fault) = entry(&row, expansion, installed) {
            faults.push(fault).expect("entry bound");
        }
    }
    faults.into_boxed()
}
