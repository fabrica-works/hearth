//! Typed changes translated to client records (render.md, 6.1).
use crate::{
    records::diff as target,
    render::{diff as source, parts::config},
};
use alloc::boxed::Box;
use skein_lib::List;
#[must_use]
pub fn change(change: source::Change) -> target::Change {
    target::Change { target: change.target, change: target_change(change.change) }
}
#[must_use]
pub fn target_change(change: source::TargetChange) -> target::TargetChange {
    match change {
        source::TargetChange::Added { spec_digest } => target::TargetChange::Added { spec_digest: spec_digest.0 },
        source::TargetChange::Removed => target::TargetChange::Removed,
        source::TargetChange::Same => target::TargetChange::Same,
        source::TargetChange::Failed { side: value } => target::TargetChange::Failed { side: side(value) },
        source::TargetChange::Changed { parts, rebuild: value } => {
            let mut translated = List::with_capacity(u32::try_from(parts.len()).expect("bounded parts"));
            for part in parts {
                translated.push(part_change(part)).expect("part capacity");
            }
            target::TargetChange::Changed { parts: translated.into_boxed(), rebuild: rebuild(value) }
        }
    }
}
#[must_use]
pub const fn side(side: source::Side) -> target::Side {
    match side {
        source::Side::From => target::Side::From,
        source::Side::To => target::Side::To,
        source::Side::Both => target::Side::Both,
    }
}
#[must_use]
pub const fn rebuild(rebuild: source::Rebuild) -> target::Rebuild {
    match rebuild {
        source::Rebuild::FromPackages { resolution } => target::Rebuild::FromPackages { resolution: needs(resolution) },
        source::Rebuild::FromSoftware => target::Rebuild::FromSoftware,
        source::Rebuild::FromConfig => target::Rebuild::FromConfig,
        source::Rebuild::SlotOnly => target::Rebuild::SlotOnly,
        source::Rebuild::Nothing => target::Rebuild::Nothing,
    }
}
#[must_use]
pub const fn needs(needs: source::Needs) -> target::Needs {
    match needs {
        source::Needs::Known => target::Needs::Known,
        source::Needs::Needed => target::Needs::Needed,
    }
}
#[must_use]
pub fn part_change(part: source::PartChange) -> target::PartChange {
    match part {
        source::PartChange::Packages(value) => target::PartChange::Packages(super::parts::packages::change(value)),
        source::PartChange::Secrets(value) => target::PartChange::Secrets(super::parts::secrets::change(value)),
        source::PartChange::Software(value) => target::PartChange::Software(super::parts::software::change(value)),
        source::PartChange::Live(value) => target::PartChange::Live(super::parts::live::change(value)),
        source::PartChange::Data(value) => target::PartChange::Data(super::parts::data::change(value)),
        source::PartChange::Exports(value) => target::PartChange::Exports(super::parts::exports::change(value)),
        source::PartChange::Boot(value) => target::PartChange::Boot(super::parts::boot::change(value)),
        source::PartChange::Meta(meta) => target::PartChange::Meta(target::MetaChange {
            base: optional_name_pair(meta.base),
            member: optional_member_pair(meta.member),
            environment: optional_name_pair(meta.environment),
            placement: optional_placement_pair(meta.placement),
        }),
        source::PartChange::Config(config::Change { tree, groups, users, members, presets, tmpfiles, scripts }) => {
            let mut translated = List::with_capacity(u32::try_from(tree.len()).expect("bounded tree"));
            for entry in tree {
                translated.push(entry_change(entry)).expect("tree capacity");
            }
            target::PartChange::Config(target::ConfigChange {
                tree: translated.into_boxed(),
                presets: directives_changes::presets(presets),
                tmpfiles: directives_changes::tmpfiles(tmpfiles),
                scripts: directives_changes::scripts(scripts),
                groups: account_changes::groups(groups),
                users: account_changes::users(users),
                members: account_changes::members(members),
            })
        }
    }
}
#[must_use]
pub fn entry_change(entry: config::EntryChange) -> target::EntryChange {
    match entry {
        config::EntryChange::Added { path, node: value } => target::EntryChange::Added { path, node: node(value) },
        config::EntryChange::Removed { path, node: value } => target::EntryChange::Removed { path, node: node(value) },
        config::EntryChange::Changed { path, before, after, sizes, hunks: value } => target::EntryChange::Changed {
            path,
            before: node(before),
            after: node(after),
            sizes: optional_sizes(sizes),
            hunks: optional_hunks(value),
        },
    }
}
fn node(node: config::Node) -> crate::records::parts::config::Node {
    match node {
        config::Node::Link { target, owner, group } => {
            crate::records::parts::config::Node::Link { target, owner, group }
        }
        config::Node::Directory { mode, owner, group } => {
            crate::records::parts::config::Node::Directory { mode, owner, group }
        }
        config::Node::File { content_digest, mode, owner, group } => {
            crate::records::parts::config::Node::File { content_digest: content_digest.0, mode, owner, group }
        }
    }
}
#[must_use]
pub fn tally(tally: source::Tally) -> target::Tally {
    target::Tally {
        from: tally.from.0,
        to: tally.to.0,
        targets: tally.targets,
        changed: tally.changed,
        unchanged: tally.unchanged,
    }
}

#[must_use]
pub fn optional_tally(value: Option<source::Tally>) -> Option<target::Tally> {
    Some(tally(value?))
}

#[must_use]
pub fn hunks(hunks: source::Hunks) -> target::Hunks {
    let mut lines = List::with_capacity(u32::try_from(hunks.lines.len()).expect("bounded lines"));
    for line in hunks.lines {
        lines.push(target::HunkLine { kind: line_kind(line.kind), text: line.text }).expect("line capacity");
    }
    target::Hunks { lines: lines.into_boxed(), left_out: hunks.left_out, binary: hunks.binary }
}
#[must_use]
pub const fn line_kind(kind: source::LineKind) -> target::LineKind {
    match kind {
        source::LineKind::Hunk => target::LineKind::Header,
        source::LineKind::Context => target::LineKind::Context,
        source::LineKind::Removed => target::LineKind::Removed,
        source::LineKind::Added => target::LineKind::Added,
        source::LineKind::NoNewline => target::LineKind::NoNewline,
    }
}

fn optional_sizes(value: Option<config::Sizes>) -> Option<target::Sizes> {
    let value = value?;
    Some(target::Sizes { before: value.before, after: value.after })
}
fn optional_hunks(value: Option<source::Hunks>) -> Option<target::Hunks> {
    Some(hunks(value?))
}

fn optional_name_pair(pair: Option<crate::render::parts::meta::NamePair>) -> Option<target::NamePair> {
    let pair = pair?;
    Some(target::NamePair { before: pair.before, after: pair.after })
}
fn optional_member_pair(pair: Option<crate::render::parts::meta::MemberPair>) -> Option<target::MemberPair> {
    let pair = pair?;
    Some(target::MemberPair { before: pair.before, after: pair.after })
}
fn optional_placement_pair(pair: Option<crate::render::parts::meta::PlacementPair>) -> Option<target::PlacementPair> {
    let pair = pair?;
    Some(target::PlacementPair { before: placement(pair.before), after: placement(pair.after) })
}
fn optional_firmware(
    firmware: Option<crate::render::revision::Firmware>,
) -> Option<crate::records::parts::meta::Firmware> {
    Some(super::answers::boot(firmware?))
}
fn placement(placement: crate::render::parts::meta::Placement) -> crate::records::parts::meta::Placement {
    use super::answers;
    crate::records::parts::meta::Placement {
        kind: answers::runs(placement.kind),
        firmware: optional_firmware(placement.firmware),
        hardware: placement.hardware,
        on: placement.on,
    }
}

mod account_changes {
    use crate::translate::parts::config::{group, member, user};
    use crate::{records::diff as target, render::parts::config as source};
    use alloc::boxed::Box;
    pub(super) fn groups(entries: Box<[source::GroupKeyed]>) -> Box<[target::GroupKeyed]> {
        let mut translated = skein_lib::List::with_capacity(u32::try_from(entries.len()).expect("entry count"));
        for entry in entries {
            translated
                .push(match entry {
                    source::GroupKeyed::Added { key, value } => target::GroupKeyed::Added { key, value: group(value) },
                    source::GroupKeyed::Removed { key, value } => {
                        target::GroupKeyed::Removed { key, value: group(value) }
                    }
                    source::GroupKeyed::Changed { key, before, after } => {
                        target::GroupKeyed::Changed { key, before: group(before), after: group(after) }
                    }
                })
                .expect("entry count");
        }
        translated.into_boxed()
    }
    pub(super) fn users(entries: Box<[source::UserKeyed]>) -> Box<[target::UserKeyed]> {
        let mut translated = skein_lib::List::with_capacity(u32::try_from(entries.len()).expect("entry count"));
        for entry in entries {
            translated
                .push(match entry {
                    source::UserKeyed::Added { key, value } => target::UserKeyed::Added { key, value: user(value) },
                    source::UserKeyed::Removed { key, value } => target::UserKeyed::Removed { key, value: user(value) },
                    source::UserKeyed::Changed { key, before, after } => {
                        target::UserKeyed::Changed { key, before: user(before), after: user(after) }
                    }
                })
                .expect("entry count");
        }
        translated.into_boxed()
    }
    pub(super) fn members(entries: Box<[source::MemberChange]>) -> Box<[target::MemberChange]> {
        let mut translated = skein_lib::List::with_capacity(u32::try_from(entries.len()).expect("entry count"));
        for entry in entries {
            translated
                .push(match entry {
                    source::MemberChange::Added(value) => target::MemberChange::Added(member(value)),
                    source::MemberChange::Removed(value) => target::MemberChange::Removed(member(value)),
                })
                .expect("entry count");
        }
        translated.into_boxed()
    }
}

mod directives_changes;

#[must_use]
pub fn resolution(change: source::ResolutionChange) -> target::ResolutionChange {
    match change {
        source::ResolutionChange::Same => target::ResolutionChange::Same,
        source::ResolutionChange::NewKey => target::ResolutionChange::NewKey,
        source::ResolutionChange::Resolved { added, removed, versions_changed } => {
            let mut moved = List::with_capacity(u32::try_from(versions_changed.len()).expect("bounded closure"));
            for source::Moved { name, architecture, before, after } in versions_changed {
                moved.push(target::Moved { name, architecture, before, after }).expect("closure capacity");
            }
            target::ResolutionChange::Resolved {
                added: listed(added),
                removed: listed(removed),
                versions_changed: moved.into_boxed(),
            }
        }
    }
}
fn listed(packages: Box<[source::Listed]>) -> Box<[target::Listed]> {
    let mut translated = List::with_capacity(u32::try_from(packages.len()).expect("bounded closure"));
    for source::Listed { name, architecture, version } in packages {
        translated.push(target::Listed { name, architecture, version }).expect("closure capacity");
    }
    translated.into_boxed()
}
