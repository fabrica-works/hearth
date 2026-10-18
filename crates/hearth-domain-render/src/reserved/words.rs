//! Closed word rules and the EFI command-line bound (machine.md, 6.2).
use super::{Refused, refused};
use crate::{
    entries::{Expansion, Located},
    errors::Rule,
    revision::declaration::{Map, Name},
};
use skein_lib::{List, Map as Sorted, bytes::copy_of};
pub(super) fn entry(row: &Located<'_>, map: Map, container: bool) -> Option<Refused> {
    let entry = row.entry;
    let key = &entry.key;
    let key_rule = match map {
        Map::Initramfs if container => Some(Rule::ContainerModule),
        Map::Cmdline if container => Some(Rule::ContainerParam),
        Map::Presets if key.starts_with(b"hearth-trial") || key.starts_with(b"hearth-check-") => Some(Rule::TrialName),
        Map::Tmpfiles if glob(key) => Some(Rule::TmpfilesPath),
        Map::Initramfs
        | Map::Cmdline
        | Map::Presets
        | Map::Tmpfiles
        | Map::Packages
        | Map::Preseeds
        | Map::Files
        | Map::Users
        | Map::Groups
        | Map::Data
        | Map::Secrets
        | Map::Identity
        | Map::Software
        | Map::Live => None,
    };
    if let Some(rule) = key_rule {
        return Some(refused(row, entry.at, rule, key));
    }
    for field in &entry.fields {
        let text = entry.text(field.name).unwrap_or(b"");
        let rule = if map == Map::Cmdline && field.name == Name::Param {
            if let Some(rule) = command(text) {
                return Some(refused(row, field.at, rule, command_key(text)));
            }
            None
        } else if map == Map::Preseeds && field.name == Name::Type && text == b"password" {
            Some(Rule::PasswordPreseed)
        } else if map == Map::Presets
            && field.name == Name::Action
            && text == b"enable"
            && (key.as_ref() == b"systemd-sysext.service" || key.as_ref() == b"systemd-confext.service")
        {
            return Some(refused(row, field.at, Rule::Overlay, key));
        } else if map == Map::Tmpfiles && field.name == Name::Type {
            if [b"v".as_slice(), b"q", b"Q"].contains(&text) {
                Some(Rule::TmpfilesSubvolume)
            } else if [b"w".as_slice(), b"w+", b"f+", b"L+"].contains(&text)
                && !beneath(key, b"/var")
                && !beneath(key, b"/run")
                && !beneath(key, b"/tmp")
            {
                Some(Rule::TmpfilesWrite)
            } else {
                None
            }
        } else {
            None
        };
        if let Some(rule) = rule {
            return Some(refused(row, field.at, rule, text));
        }
    }
    None
}
fn command(param: &[u8]) -> Option<Rule> {
    let key = command_key(param);
    if [b"root".as_slice(), b"rootflags", b"ro", b"rw", b"initrd", b"panic"].contains(&key)
        || key.starts_with(b"hearth.")
    {
        Some(Rule::ActivatorKey)
    } else if [b"rootfstype".as_slice(), b"rootdelay", b"init"].contains(&key) {
        Some(Rule::RootKey)
    } else if key == b"rootwait" {
        Some(Rule::RootWait)
    } else {
        None
    }
}
fn beneath(path: &[u8], root: &[u8]) -> bool {
    if path == root {
        return true;
    }
    match path.strip_prefix(root) {
        Some(suffix) => suffix.starts_with(b"/"),
        None => false,
    }
}
pub(super) fn length(expansion: &mut Expansion, target: &[u8], faults: &mut List<Refused>) {
    let entries = expansion.of(Map::Cmdline);
    let mut ordered = Sorted::with_capacity(u32::try_from(entries.len()).expect("entry count"));
    let mut names = Sorted::with_capacity(u32::try_from(entries.len()).expect("entry count"));
    for row in &entries {
        if names.contains_key(&row.entry.key) {
            continue;
        }
        names.insert(row.entry.key.clone(), ()).expect("entry count");
        ordered
            .insert((row.entry.int(Name::Order).unwrap_or(500), row.entry.key.clone(), copy_of(row.task)), *row)
            .expect("entry count");
    }
    let mut bytes = 118_u32.saturating_add(u32::try_from(target.len()).expect("target bound").saturating_mul(3));
    let mut crossing = None;
    for (_, row) in &ordered {
        let Some(param) = row.entry.text(Name::Param) else { continue };
        bytes = bytes.saturating_add(1).saturating_add(u32::try_from(param.len()).expect("expanded bound"));
        if bytes > 2048 {
            for field in &row.entry.fields {
                if field.name == Name::Param {
                    crossing = Some((
                        refused(row, field.at, Rule::CmdlineLength { bytes }, param),
                        copy_of(row.task),
                        row.entry.key.clone(),
                    ));
                }
            }
            break;
        }
    }
    drop(ordered);
    drop(entries);
    if let Some((fault, task, key)) = crossing {
        expansion.refuse(Map::Cmdline, &task, &key);
        faults.push(fault).expect("entry count");
    }
}

fn glob(key: &[u8]) -> bool {
    for byte in key {
        if b"*?[%".contains(byte) {
            return true;
        }
    }
    false
}

fn command_key(param: &[u8]) -> &[u8] {
    match skein_lib::bytes::find(param, b"=") {
        Some(end) => param.get(..end).expect("found equals"),
        None => param,
    }
}
