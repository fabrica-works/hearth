//! Typed changes translated to client records (render.md, 6.1).
use crate::{
    records::diff as target,
    render::{diff as source, parts::config},
};
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
        source::PartChange::Meta(meta) => target::PartChange::Meta(target::MetaChange {
            base: optional_pair(meta.base),
            member: optional_pair(meta.member),
            environment: optional_pair(meta.environment),
            placement: optional_placement_pair(meta.placement),
        }),
        source::PartChange::Config(config::Change { tree }) => {
            let mut translated = List::with_capacity(u32::try_from(tree.len()).expect("bounded tree"));
            for entry in tree {
                translated.push(entry_change(entry)).expect("tree capacity");
            }
            target::PartChange::Config(target::ConfigChange { tree: translated.into_boxed() })
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

fn optional_pair<T>(pair: Option<crate::render::parts::meta::Pair<T>>) -> Option<target::Pair<T>> {
    let pair = pair?;
    Some(target::Pair { before: pair.before, after: pair.after })
}
fn optional_placement_pair(
    pair: Option<crate::render::parts::meta::Pair<crate::render::parts::meta::Placement>>,
) -> Option<target::Pair<crate::records::parts::meta::Placement>> {
    let pair = pair?;
    Some(target::Pair { before: placement(pair.before), after: placement(pair.after) })
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
