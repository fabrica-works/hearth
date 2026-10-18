use crate::{
    records::parts::{live as live_from, secrets as secrets_from, software as software_from},
    render::parts::{live as live_to, secrets as secrets_to, software as software_to},
};
use skein_lib::bytes::copy_of;
#[must_use]
pub fn secrets_change_back(value: secrets_from::Change) -> secrets_to::Change {
    secrets_to::Change { files: named(value.files), passwords: named(value.passwords), identity: named(value.identity) }
}
fn named(values: Box<[secrets_from::Named]>) -> Box<[secrets_to::Named]> {
    let mut rows = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("entry count"));
    for value in values {
        rows.push(match value {
            secrets_from::Named::Added(key) => secrets_to::Named::Added(key),
            secrets_from::Named::Removed(key) => secrets_to::Named::Removed(key),
            secrets_from::Named::Changed(key) => secrets_to::Named::Changed(key),
        })
        .expect("entry count");
    }
    rows.into_boxed()
}

pub fn download_back(value: software_from::Download) -> software_to::Download {
    let software_from::Download { path, sha256, owner, group, how } = value;
    software_to::Download {
        path,
        sha256: hearth_lib::encode::Digest(sha256),
        owner,
        group,
        how: match how {
            software_from::How::File { mode } => software_to::How::File { mode },
            software_from::How::Tar { strip } => software_to::How::Tar { strip },
        },
    }
}

#[must_use]
pub fn software_change_back(value: software_from::Change) -> software_to::Change {
    let values = value.downloads;
    let downloads = download_changes(values);
    software_to::Change {
        downloads,
        scripts: if let Some(value) = value.scripts {
            Some(crate::render::parts::config::ScriptsChange {
                before: crate::tests::translate::directives::scripts_back(value.before),
                after: crate::tests::translate::directives::scripts_back(value.after),
            })
        } else {
            None
        },
    }
}
fn download_changes(values: Box<[software_from::DownloadKeyed]>) -> Box<[software_to::DownloadKeyed]> {
    let mut rows = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("entry count"));
    for value in values {
        rows.push(match value {
            software_from::DownloadKeyed::Added { key, value } => {
                software_to::DownloadKeyed::Added { key, value: download_back(value) }
            }
            software_from::DownloadKeyed::Removed { key, value } => {
                software_to::DownloadKeyed::Removed { key, value: download_back(value) }
            }
            software_from::DownloadKeyed::Changed { key, before, after } => {
                software_to::DownloadKeyed::Changed { key, before: download_back(before), after: download_back(after) }
            }
        })
        .expect("entry count");
    }
    rows.into_boxed()
}

pub fn consumers_back(values: Box<[live_from::Consumer]>) -> Box<[live_to::Consumer]> {
    let mut rows = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("consumer count"));
    for live_from::Consumer { task, unit, action } in values {
        rows.push(live_to::Consumer {
            task,
            unit,
            action: match action {
                live_from::Action::Reload => live_to::Action::Reload,
                live_from::Action::Restart => live_to::Action::Restart,
                live_from::Action::ReloadOrRestart => live_to::Action::ReloadOrRestart,
                live_from::Action::Start => live_to::Action::Start,
                live_from::Action::None => live_to::Action::None,
            },
        })
        .expect("consumer count");
    }
    rows.into_boxed()
}

#[must_use]
pub fn live_change_back(value: live_from::Change) -> live_to::Change {
    live_to::Change { paths: changes(value.paths) }
}
fn changes(values: Box<[live_from::ConsumersKeyed]>) -> Box<[live_to::ConsumersKeyed]> {
    let mut rows = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("entry count"));
    for value in values {
        rows.push(match value {
            live_from::ConsumersKeyed::Added { key, value } => {
                live_to::ConsumersKeyed::Added { key, value: consumers_back(value) }
            }
            live_from::ConsumersKeyed::Removed { key, value } => {
                live_to::ConsumersKeyed::Removed { key, value: consumers_back(value) }
            }
            live_from::ConsumersKeyed::Changed { key, before, after } => {
                live_to::ConsumersKeyed::Changed { key, before: consumers_back(before), after: consumers_back(after) }
            }
        })
        .expect("entry count");
    }
    rows.into_boxed()
}

mod seeds {
    use crate::{records::parts::packages as from, render::parts::packages as to};
    pub fn seeds_change_back(value: from::SeedsChange) -> to::SeedsChange {
        to::SeedsChange { users: user_changes(value.users), groups: group_changes(value.groups) }
    }
    fn seed_ids(value: from::SeedIds) -> to::SeedIds {
        let from::SeedIds { uid, gid } = value;
        to::SeedIds { uid, gid }
    }
    fn group_id(value: u32) -> u32 {
        value
    }
    fn user_changes(values: Box<[from::SeedUserKeyed]>) -> Box<[to::SeedUserKeyed]> {
        let mut rows = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("entry count"));
        for value in values {
            rows.push(match value {
                from::SeedUserKeyed::Added { key, value } => to::SeedUserKeyed::Added { key, value: seed_ids(value) },
                from::SeedUserKeyed::Removed { key, value } => {
                    to::SeedUserKeyed::Removed { key, value: seed_ids(value) }
                }
                from::SeedUserKeyed::Changed { key, before, after } => {
                    to::SeedUserKeyed::Changed { key, before: seed_ids(before), after: seed_ids(after) }
                }
            })
            .expect("entry count");
        }
        rows.into_boxed()
    }
    fn group_changes(values: Box<[from::SeedGroupKeyed]>) -> Box<[to::SeedGroupKeyed]> {
        let mut rows = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("entry count"));
        for value in values {
            rows.push(match value {
                from::SeedGroupKeyed::Added { key, value } => to::SeedGroupKeyed::Added { key, value: group_id(value) },
                from::SeedGroupKeyed::Removed { key, value } => {
                    to::SeedGroupKeyed::Removed { key, value: group_id(value) }
                }
                from::SeedGroupKeyed::Changed { key, before, after } => {
                    to::SeedGroupKeyed::Changed { key, before: group_id(before), after: group_id(after) }
                }
            })
            .expect("entry count");
        }
        rows.into_boxed()
    }
}
pub use seeds::seeds_change_back;
#[derive(Clone, Copy)]
enum Kind {
    Added,
    Removed,
    Changed,
}
const KINDS: [Kind; 3] = [Kind::Added, Kind::Removed, Kind::Changed];
fn named_input(kind: Kind) -> secrets_to::Named {
    match kind {
        Kind::Added => secrets_to::Named::Added(copy_of(b"added")),
        Kind::Removed => secrets_to::Named::Removed(copy_of(b"removed")),
        Kind::Changed => secrets_to::Named::Changed(copy_of(b"changed")),
    }
}
#[must_use]
pub fn part_changes() -> [crate::render::diff::PartChange; 3] {
    use crate::render::diff::PartChange;
    let software = super::software_records();
    let mut capacity = 0_usize;
    for row in &software {
        capacity =
            capacity.checked_add(row.downloads.len().checked_mul(KINDS.len()).expect("changes")).expect("downloads");
    }
    let mut downloads = skein_lib::List::with_capacity(u32::try_from(capacity).expect("downloads"));
    let mut scripts = None;
    for input in software {
        for before in input.downloads {
            let mut after = before.clone();
            after.sha256 = super::ABC;
            for entry in [
                software_to::DownloadKeyed::Added { key: before.path.clone(), value: before.clone() },
                software_to::DownloadKeyed::Removed { key: before.path.clone(), value: before.clone() },
                software_to::DownloadKeyed::Changed { key: before.path.clone(), before, after },
            ] {
                downloads.push(entry).expect("downloads");
            }
        }
        scripts = Some(crate::render::parts::config::ScriptsChange { before: Box::new([]), after: input.scripts });
    }
    let live = super::live_records();
    let mut capacity = 0_usize;
    for row in &live {
        capacity = capacity.checked_add(row.paths.len().checked_mul(KINDS.len()).expect("changes")).expect("paths");
    }
    let mut paths = skein_lib::List::with_capacity(u32::try_from(capacity).expect("paths"));
    for input in live {
        for before in input.paths {
            for entry in [
                live_to::ConsumersKeyed::Added { key: before.path.clone(), value: before.consumers.clone() },
                live_to::ConsumersKeyed::Removed { key: before.path.clone(), value: before.consumers.clone() },
                live_to::ConsumersKeyed::Changed { key: before.path, before: before.consumers, after: Box::new([]) },
            ] {
                paths.push(entry).expect("paths");
            }
        }
    }
    [
        PartChange::Secrets(secrets_to::Change {
            files: Box::new(KINDS.map(named_input)),
            passwords: Box::new(KINDS.map(named_input)),
            identity: Box::new(KINDS.map(named_input)),
        }),
        PartChange::Software(software_to::Change { downloads: downloads.into_boxed(), scripts }),
        PartChange::Live(live_to::Change { paths: paths.into_boxed() }),
    ]
}
