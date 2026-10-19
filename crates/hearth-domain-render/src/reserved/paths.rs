//! One table for every path-bearing kind (protocol/format.md, 5.11).
use super::{Installed, Refused, Shape, refused};
use crate::{
    entries::Located,
    errors::Rule,
    revision::declaration::{Map, Name},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
#[derive(Clone, Copy)]
enum Match {
    Exact(&'static [u8]),
    Beneath(&'static [u8]),
    Pattern(&'static [u8], &'static [u8], &'static [u8]),
    Dropin,
    Trial,
    OverlayLink,
    Log,
    DataBoundary,
    DataRoot,
    SystemdConf,
    SystemdDropin,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Except {
    None,
    Secret,
    Tmpfiles,
    Local,
    Data,
    OtherKinds,
    Live,
}
struct Row {
    rule: Rule,
    pattern: Match,
    except: Except,
}
use Except::{Data, Live, Local, None as Never, OtherKinds, Secret, Tmpfiles};
use Match::{Beneath, Exact, Pattern};
const ROWS: &[Row] = &[
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/systemd/system.conf"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/systemd/system.conf.d"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/usr/local/lib/systemd/system.conf.d"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/systemd/system-generators"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/usr/local/lib/systemd/system-generators"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/systemd/system-environment-generators"), except: Live },
    Row {
        rule: Rule::NeverLivePath,
        pattern: Beneath(b"/usr/local/lib/systemd/system-environment-generators"),
        except: Live,
    },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/udev"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/usr/local/lib/udev"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/modprobe.d"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/usr/local/lib/modprobe.d"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/modules"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/modules-load.d"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/usr/local/lib/modules-load.d"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/sysctl.conf"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/sysctl.d"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/usr/local/lib/sysctl.d"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/binfmt.d"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/usr/local/lib/binfmt.d"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/kernel"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/usr/local/lib/kernel"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/initramfs-tools"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/tiny-initramfs"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/crypttab"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/default/grub"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/fstab"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/hostname"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/localtime"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/locale.conf"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/vconsole.conf"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/machine-id"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/os-release"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Beneath(b"/etc/apparmor.d"), except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Match::SystemdConf, except: Live },
    Row { rule: Rule::NeverLivePath, pattern: Match::SystemdDropin, except: Live },
    Row { rule: Rule::DataRoot, pattern: Match::DataBoundary, except: OtherKinds },
    Row { rule: Rule::ActivatorFile, pattern: Exact(b"/etc/fstab"), except: Never },
    Row { rule: Rule::ActivatorFile, pattern: Beneath(b"/.hearth"), except: Never },
    Row { rule: Rule::Dropin, pattern: Match::Dropin, except: Never },
    Row { rule: Rule::TrialName, pattern: Match::Trial, except: Never },
    Row { rule: Rule::Accounts, pattern: Exact(b"/etc/passwd"), except: Never },
    Row { rule: Rule::Accounts, pattern: Exact(b"/etc/group"), except: Never },
    Row { rule: Rule::Accounts, pattern: Exact(b"/etc/shadow"), except: Never },
    Row { rule: Rule::Accounts, pattern: Exact(b"/etc/gshadow"), except: Never },
    Row { rule: Rule::Accounts, pattern: Exact(b"/etc/subuid"), except: Never },
    Row { rule: Rule::Accounts, pattern: Exact(b"/etc/subgid"), except: Never },
    Row { rule: Rule::Accounts, pattern: Exact(b"/etc/machine-id"), except: Never },
    Row { rule: Rule::Accounts, pattern: Exact(b"/var/lib/dbus/machine-id"), except: Never },
    Row { rule: Rule::SecretPath, pattern: Pattern(b"/etc/ssh", b"ssh_host_", b"_key"), except: Secret },
    Row { rule: Rule::SecretPath, pattern: Pattern(b"/etc/ssh", b"ssh_host_", b"_key.pub"), except: Secret },
    Row { rule: Rule::SecretPath, pattern: Pattern(b"/etc/dropbear", b"dropbear_", b"_host_key"), except: Secret },
    Row { rule: Rule::SecretPath, pattern: Beneath(b"/etc/ssl/private"), except: Secret },
    Row { rule: Rule::SecretPath, pattern: Exact(b"/var/lib/systemd/random-seed"), except: Secret },
    Row { rule: Rule::SecretPath, pattern: Exact(b"/var/lib/systemd/credential.secret"), except: Secret },
    Row { rule: Rule::HearthState, pattern: Beneath(b"/var/lib/hearth"), except: Never },
    Row { rule: Rule::PackagesLayer, pattern: Beneath(b"/usr"), except: Local },
    Row { rule: Rule::ClassMount, pattern: Beneath(b"/srv/core"), except: Data },
    Row { rule: Rule::ClassMount, pattern: Beneath(b"/srv/data"), except: Data },
    Row { rule: Rule::ClassMount, pattern: Beneath(b"/srv/cache"), except: Data },
    Row { rule: Rule::Runtime, pattern: Beneath(b"/dev"), except: Never },
    Row { rule: Rule::Runtime, pattern: Beneath(b"/proc"), except: Never },
    Row { rule: Rule::Runtime, pattern: Beneath(b"/sys"), except: Never },
    Row { rule: Rule::Runtime, pattern: Beneath(b"/run"), except: Tmpfiles },
    Row { rule: Rule::Runtime, pattern: Beneath(b"/tmp"), except: Tmpfiles },
    Row { rule: Rule::Runtime, pattern: Beneath(b"/var/tmp"), except: Tmpfiles },
    Row { rule: Rule::Runtime, pattern: Beneath(b"/etc/.git"), except: Never },
    Row { rule: Rule::Runtime, pattern: Beneath(b"/etc/.etckeeper"), except: Never },
    Row { rule: Rule::LogFile, pattern: Match::Log, except: Never },
    Row { rule: Rule::InitramfsConf, pattern: Exact(b"/etc/tiny-initramfs/tiny-initramfs.conf"), except: Never },
    Row { rule: Rule::Overlay, pattern: Beneath(b"/etc/extensions"), except: Never },
    Row { rule: Rule::Overlay, pattern: Beneath(b"/usr/lib/confexts"), except: Never },
    Row { rule: Rule::Overlay, pattern: Beneath(b"/usr/local/lib/confexts"), except: Never },
    Row { rule: Rule::Overlay, pattern: Beneath(b"/var/lib/extensions"), except: Never },
    Row { rule: Rule::Overlay, pattern: Beneath(b"/var/lib/confexts"), except: Never },
    Row { rule: Rule::Overlay, pattern: Match::OverlayLink, except: Never },
    Row { rule: Rule::DataRoot, pattern: Match::DataRoot, except: OtherKinds },
];
pub(super) fn beneath(path: &[u8], root: &[u8]) -> bool {
    path == root || path.strip_prefix(root).unwrap_or(b"").starts_with(b"/")
}
fn child<'a>(path: &'a [u8], root: &[u8]) -> Option<&'a [u8]> {
    let name = path.strip_prefix(root)?.strip_prefix(b"/")?;
    if name.contains(&b'/') { None } else { Some(name) }
}
fn parent(path: &[u8]) -> (&[u8], &[u8]) {
    let mut split = 0;
    for (index, byte) in path.iter().enumerate() {
        if *byte == b'/' {
            split = index;
        }
    }
    (path.get(..split).expect("split"), path.get(split.saturating_add(1)..).expect("split"))
}
fn matched(pattern: Match, path: &[u8], shape: Shape) -> bool {
    let (directory, name) = parent(path);
    match pattern {
        Exact(root) => path == root,
        Beneath(root) => beneath(path, root),
        Pattern(root, prefix, suffix) => {
            child(path, root).is_some() && name.starts_with(prefix) && name.ends_with(suffix)
        }
        Match::Dropin => {
            child(path, b"/etc/systemd/system.conf.d").is_some()
                && (name.starts_with(b"zz-hearth") || name > b"zz-hearth-watchdog.conf")
        }
        Match::Trial => name.starts_with(b"hearth-trial") || name.starts_with(b"hearth-check-"),
        Match::OverlayLink => {
            shape == Shape::Link
                && (directory.ends_with(b".wants") || directory.ends_with(b".requires"))
                && [b"systemd-sysext.service".as_slice(), b"systemd-confext.service"].contains(&name)
        }
        Match::Log => beneath(path, b"/var/log") && shape == Shape::File { empty: false },
        Match::DataBoundary => [b"/srv".as_slice(), b"/var", b"/home"].contains(&path),
        Match::SystemdConf => child(path, b"/etc/systemd").is_some() && name.ends_with(b".conf"),
        Match::SystemdDropin => child(path, b"/usr/local/lib/systemd").is_some() && name.ends_with(b".conf.d"),
        Match::DataRoot => !strict(path, b"/srv") && !strict(path, b"/var") && !strict(path, b"/home"),
    }
}
fn above(pattern: Match, path: &[u8]) -> bool {
    match pattern {
        Exact(root) | Beneath(root) => path != root && beneath(root, path),
        Pattern(root, ..) => beneath(root, path),
        Match::Dropin => beneath(b"/etc/systemd/system.conf.d", path),
        Match::Log => path != b"/var/log" && beneath(b"/var/log", path),
        Match::Trial
        | Match::OverlayLink
        | Match::DataBoundary
        | Match::DataRoot
        | Match::SystemdConf
        | Match::SystemdDropin => false,
    }
}
fn check(path: &[u8], shape: Shape, map: Option<Map>, own_data: bool) -> Option<Rule> {
    for row in ROWS {
        if row.except == Live {
            continue;
        }
        let exempt = match row.except {
            Live => true,
            Never => false,
            Secret => map == Some(Map::Secrets) || map == Some(Map::Identity),
            Tmpfiles => map == Some(Map::Tmpfiles),
            Local => beneath(path, b"/usr/local"),
            Data => own_data,
            OtherKinds => map != Some(Map::Data),
        };
        if (!exempt && matched(row.pattern, path, shape))
            || ((shape != Shape::Directory || map == Some(Map::Data) || map == Some(Map::Software))
                && above(row.pattern, path))
        {
            return Some(row.rule);
        }
    }
    None
}
fn strict(path: &[u8], root: &[u8]) -> bool {
    path != root && beneath(path, root)
}
fn own_data(row: &Located<'_>, path: &[u8]) -> bool {
    let (directory, name) = parent(path);
    let class = row.entry.text(Name::Class).unwrap_or(b"data");
    child(directory, b"/srv") == Some(class) && name == row.entry.key.as_ref()
}
pub(super) fn entry(row: &Located<'_>, map: Map) -> Option<Refused> {
    let entry = row.entry;
    let directory = map == Map::Tmpfiles
        && [b"d".as_slice(), b"e", b"z"].contains(&entry.text(Name::Type).unwrap_or(b""))
        && entry.text(Name::Age).is_none();
    let shape = if directory || map == Map::Data || (map == Map::Software && entry.text(Name::Unpack) == Some(b"tar")) {
        Shape::Directory
    } else {
        Shape::File { empty: false }
    };
    if map == Map::Tmpfiles
        && let Some(rule) = check(&entry.key, shape, Some(map), false)
    {
        return Some(refused(row, entry.at, rule, &entry.key));
    }
    for field in &entry.fields {
        if [Map::Data, Map::Software, Map::Secrets, Map::Identity].contains(&map)
            && [Name::Path, Name::PublicPath].contains(&field.name)
            && let Some(path) = entry.text(field.name)
            && let Some(rule) = check(path, shape, Some(map), map == Map::Data && own_data(row, path))
        {
            return Some(refused(row, field.at, rule, path));
        }
    }
    None
}
pub(super) fn tree(installed: &[Installed<'_>]) -> Box<[Refused]> {
    let mut faults = List::with_capacity(u32::try_from(installed.len()).expect("install bound"));
    for install in installed {
        if let Some(rule) = check(install.path, install.shape, None, false) {
            faults
                .push(Refused {
                    task: copy_of(install.task),
                    spot: match install.at {
                        Some(at) => super::Spot::Declared(at),
                        None => super::Spot::Tree,
                    },
                    also: None,
                    reserved: crate::errors::Reserved { rule, what: copy_of(install.path) },
                })
                .expect("install bound");
        }
    }
    faults.into_boxed()
}

pub(super) fn never(path: &[u8]) -> bool {
    for row in ROWS {
        if row.except == OtherKinds || (row.except == Local && beneath(path, b"/usr/local")) {
            continue;
        }
        for end in 1..=path.len() {
            if end != path.len() && path.get(end) != Some(&b'/') {
                continue;
            }
            if matched(row.pattern, path.get(..end).expect("path prefix"), Shape::Link) {
                return true;
            }
        }
    }
    false
}
