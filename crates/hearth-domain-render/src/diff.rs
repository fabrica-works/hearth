//! Typed differences and the lowest changed layer (render.md, 6.1, 6.2).
use crate::{
    parts::{PartRecord, Spec, config},
    revision::Oid,
    vocabulary::Class,
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub target: Box<[u8]>,
    pub change: TargetChange,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TargetChange {
    Added { spec_digest: Digest },
    Removed,
    Same,
    Changed { parts: Box<[PartChange]>, rebuild: Rebuild },
    Failed { side: Side },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Side {
    From,
    To,
    Both,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Made {
    Spec(Box<Spec>),
    Failed(Class),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PartChange {
    Config(config::Change),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Rebuild {
    FromPackages { resolution: Needs },
    FromSoftware,
    FromConfig,
    SlotOnly,
    Nothing,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Needs {
    Known,
    Needed,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Layer {
    Packages,
    Software,
    Config,
    Slot,
    Other,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tally {
    pub from: Oid,
    pub to: Oid,
    pub targets: u32,
    pub changed: u32,
    pub unchanged: u32,
}
#[must_use]
pub const fn layer(part: &PartRecord) -> Layer {
    match part {
        PartRecord::Config(_) => Layer::Config,
    }
}
#[must_use]
pub fn rebuild(changed: &[Layer], resolution: Needs) -> Rebuild {
    match changed.iter().min() {
        Some(Layer::Packages) => Rebuild::FromPackages { resolution },
        Some(Layer::Software) => Rebuild::FromSoftware,
        Some(Layer::Config) => Rebuild::FromConfig,
        Some(Layer::Slot) => Rebuild::SlotOnly,
        Some(Layer::Other) | None => Rebuild::Nothing,
    }
}
#[must_use]
pub fn compare(from: Option<&Made>, to: Option<&Made>, resolution: Needs) -> TargetChange {
    match (from, to) {
        (Some(Made::Failed(_)), Some(Made::Failed(_))) => TargetChange::Failed { side: Side::Both },
        (Some(Made::Failed(_)), _) => TargetChange::Failed { side: Side::From },
        (_, Some(Made::Failed(_))) => TargetChange::Failed { side: Side::To },
        (None, Some(Made::Spec(spec))) => TargetChange::Added { spec_digest: spec.digest },
        (Some(Made::Spec(_)), None) => TargetChange::Removed,
        (Some(Made::Spec(before)), Some(Made::Spec(after))) => {
            if before.digest == after.digest {
                return TargetChange::Same;
            }
            let from = config_part(before);
            let to = config_part(after);
            let mut parts = skein_lib::List::with_capacity(1);
            let mut layers = skein_lib::List::with_capacity(1);
            let changed = match (from, to) {
                (Some((old, _)), Some((new, _))) => old != new,
                (None, None) => false,
                (Some(_), None) | (None, Some(_)) => true,
            };
            if changed {
                let old = config_record(from);
                let new = config_record(to);
                parts.push(PartChange::Config(config::change(old, new))).expect("one config");
                layers.push(Layer::Config).expect("one config");
            }
            TargetChange::Changed { parts: parts.into_boxed(), rebuild: rebuild(layers.as_slice(), resolution) }
        }
        (None, None) => unreachable!("a compared target exists on a side"),
    }
}
fn config_record<'a>(part: Option<(&Digest, &'a config::Config)>) -> Option<&'a config::Config> {
    let (_, record) = part?;
    Some(record)
}
fn config_part(spec: &Spec) -> Option<(&Digest, &config::Config)> {
    let mut found = None;
    for part in &spec.parts {
        match &part.record {
            PartRecord::Config(config) => found = Some((&part.digest, config)),
        }
    }
    found
}

/// Git's retained unified lines (render.md, 6.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Hunks {
    pub lines: Box<[Line]>,
    pub left_out: u32,
    pub binary: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line {
    pub kind: LineKind,
    pub text: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LineKind {
    Hunk,
    Context,
    Removed,
    Added,
    NoNewline,
}

pub(crate) fn tree(change: &mut TargetChange) -> &mut [config::EntryChange] {
    let mut tree: &mut [config::EntryChange] = &mut [];
    if let TargetChange::Changed { parts, .. } = change {
        for part in parts {
            match part {
                PartChange::Config(config) => tree = &mut config.tree,
            }
        }
    }
    tree
}
