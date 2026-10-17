//! Typed changes and their forms (render.md, 6.1 and 6.3).
use crate::{
    form::{
        json::{digest, field},
        text::Lines,
    },
    parts::config::Node,
};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub target: Box<[u8]>,
    pub change: TargetChange,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TargetChange {
    Added { spec_digest: [u8; 32] },
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
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PartChange {
    Config(ConfigChange),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ConfigChange {
    pub tree: Box<[EntryChange]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EntryChange {
    Added { path: Box<[u8]>, node: Node },
    Removed { path: Box<[u8]>, node: Node },
    Changed { path: Box<[u8]>, before: Node, after: Node, sizes: Option<Sizes>, hunks: Option<Hunks> },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Sizes {
    pub before: u64,
    pub after: u64,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Hunks {
    pub lines: Box<[HunkLine]>,
    pub left_out: u32,
    pub binary: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HunkLine {
    pub kind: LineKind,
    pub text: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LineKind {
    Header,
    Context,
    Removed,
    Added,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tally {
    pub from: Box<[u8]>,
    pub to: Box<[u8]>,
    pub targets: u32,
    pub changed: u32,
    pub unchanged: u32,
}
impl Change {
    pub(crate) fn text(&self, lines: &mut Lines) {
        if self.change == TargetChange::Same {
            return;
        }
        lines.put(&self.target);
        lines.pad_to(52.max(self.target.len().saturating_add(2)));
        match &self.change {
            TargetChange::Same => {}
            TargetChange::Removed => lines.put(b"removed"),
            TargetChange::Added { spec_digest } => {
                lines.put(b"added spec ");
                lines.hex12(spec_digest);
            }
            TargetChange::Failed { side } => {
                lines.put(b"failed (");
                lines.put(side.name());
                lines.put(b")");
            }
            TargetChange::Changed { rebuild, .. } => lines.put(rebuild.text()),
        }
        lines.put(b"\n");
        match &self.change {
            TargetChange::Changed { parts, .. } => {
                for part in &**parts {
                    match part {
                        PartChange::Config(config) => config.text(lines),
                    }
                }
            }
            TargetChange::Same | TargetChange::Removed | TargetChange::Added { .. } | TargetChange::Failed { .. } => {}
        }
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"record", b"change");
        field(json, b"target", &self.target);
        match &self.change {
            TargetChange::Added { spec_digest } => {
                field(json, b"kind", b"added");
                json.key(b"added");
                json.object_start();
                json.key(b"spec_digest");
                digest(json, spec_digest);
                json.object_end();
            }
            TargetChange::Removed => field(json, b"kind", b"removed"),
            TargetChange::Same => field(json, b"kind", b"same"),
            TargetChange::Failed { side } => {
                field(json, b"kind", b"failed");
                json.key(b"failed");
                json.object_start();
                field(json, b"side", side.name());
                json.object_end();
            }
            TargetChange::Changed { parts, rebuild } => {
                field(json, b"kind", b"changed");
                json.key(b"changed");
                json.object_start();
                json.key(b"parts");
                json.array_start();
                for part in &**parts {
                    match part {
                        PartChange::Config(config) => config.json(json),
                    }
                }
                json.array_end();
                json.key(b"rebuild");
                rebuild.json(json);
                json.object_end();
            }
        }
    }
}
impl Side {
    pub(crate) fn name(self) -> &'static [u8] {
        match self {
            Self::From => b"from",
            Self::To => b"to",
            Self::Both => b"both",
        }
    }
}
impl Rebuild {
    fn text(self) -> &'static [u8] {
        match self {
            Self::FromPackages { resolution: Needs::Known } => b"rebuild from packages",
            Self::FromPackages { resolution: Needs::Needed } => b"rebuild from packages, resolution needed",
            Self::FromSoftware => b"rebuild from software",
            Self::FromConfig => b"rebuild from config",
            Self::SlotOnly => b"new slot only",
            Self::Nothing => b"nothing to build",
        }
    }
    fn json(self, json: &mut Encoder) {
        json.object_start();
        field(
            json,
            b"kind",
            match self {
                Self::FromPackages { .. } => b"from_packages",
                Self::FromSoftware => b"from_software",
                Self::FromConfig => b"from_config",
                Self::SlotOnly => b"slot_only",
                Self::Nothing => b"nothing",
            },
        );
        match self {
            Self::FromPackages { resolution } => field(
                json,
                b"resolution",
                match resolution {
                    Needs::Known => b"known",
                    Needs::Needed => b"needed",
                },
            ),
            Self::FromSoftware | Self::FromConfig | Self::SlotOnly | Self::Nothing => {}
        }
        json.object_end();
    }
}
impl ConfigChange {
    fn text(&self, lines: &mut Lines) {
        let mut column = 52;
        for entry in &*self.tree {
            column = column.max(entry.path().len().saturating_add(15));
        }
        for entry in &*self.tree {
            lines.put(b"  config   ");
            match entry {
                EntryChange::Added { .. } => lines.put(b"+ "),
                EntryChange::Removed { .. } => lines.put(b"- "),
                EntryChange::Changed { .. } => lines.put(b"~ "),
            }
            lines.put(entry.path());
            lines.pad_to(column);
            let node = match entry {
                EntryChange::Added { node, .. } | EntryChange::Removed { node, .. } => node,
                EntryChange::Changed { after, .. } => after,
            };
            match node {
                Node::File { mode, owner, group, .. } => {
                    lines.mode(*mode);
                    lines.put(b" ");
                    lines.put(owner);
                    lines.put(b":");
                    lines.put(group);
                }
            }
            lines.put(b"\n");
            match entry {
                EntryChange::Changed { hunks: Some(hunks), .. } => hunks.text(lines),
                EntryChange::Changed { hunks: None, .. } | EntryChange::Added { .. } | EntryChange::Removed { .. } => {}
            }
        }
    }
    fn json(&self, json: &mut Encoder) {
        json.object_start();
        field(json, b"kind", b"config");
        json.key(b"tree");
        json.array_start();
        for entry in &*self.tree {
            entry.json(json);
        }
        json.array_end();
        json.object_end();
    }
}
impl EntryChange {
    fn path(&self) -> &[u8] {
        match self {
            Self::Added { path, .. } | Self::Removed { path, .. } | Self::Changed { path, .. } => path,
        }
    }
    fn json(&self, json: &mut Encoder) {
        json.object_start();
        match self {
            Self::Added { node, .. } | Self::Removed { node, .. } => {
                field(
                    json,
                    b"kind",
                    match self {
                        Self::Added { .. } => b"added",
                        Self::Removed { .. } => b"removed",
                        Self::Changed { .. } => unreachable!("added or removed arm"),
                    },
                );
                json.key(b"node");
                node.json(json);
                field(json, b"path", self.path());
            }
            Self::Changed { before, after, sizes, hunks, .. } => {
                json.key(b"after");
                after.json(json);
                json.key(b"before");
                before.json(json);
                if let Some(hunks) = hunks {
                    json.key(b"hunks");
                    hunks.json(json);
                }
                field(json, b"kind", b"changed");
                field(json, b"path", self.path());
                if let Some(sizes) = sizes {
                    json.key(b"sizes");
                    json.object_start();
                    json.key(b"after");
                    json.unsigned(sizes.after);
                    json.key(b"before");
                    json.unsigned(sizes.before);
                    json.object_end();
                }
            }
        }
        json.object_end();
    }
}
impl Hunks {
    fn text(&self, lines: &mut Lines) {
        if self.binary {
            return;
        }
        for line in &*self.lines {
            lines.put(b"               ");
            lines.put(match line.kind {
                LineKind::Header => b"",
                LineKind::Context => b" ",
                LineKind::Removed => b"-",
                LineKind::Added => b"+",
            });
            lines.put(&line.text);
            lines.put(b"\n");
        }
        if self.left_out != 0 {
            lines.put(b"               ... ");
            lines.decimal(u64::from(self.left_out));
            lines.put(b" lines left out\n");
        }
    }
    fn json(&self, json: &mut Encoder) {
        json.object_start();
        json.key(b"binary");
        json.boolean(self.binary);
        json.key(b"left_out");
        json.unsigned(u64::from(self.left_out));
        json.key(b"lines");
        json.array_start();
        for line in &*self.lines {
            json.object_start();
            field(
                json,
                b"kind",
                match line.kind {
                    LineKind::Header => b"header",
                    LineKind::Context => b"context",
                    LineKind::Removed => b"removed",
                    LineKind::Added => b"added",
                },
            );
            field(json, b"text", &line.text);
            json.object_end();
        }
        json.array_end();
        json.object_end();
    }
}
impl Tally {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"\n");
        lines.decimal(u64::from(self.changed));
        lines.put(b" of ");
        lines.decimal(u64::from(self.targets));
        lines.put(if self.targets == 1 { b" target " } else { b" targets " });
        lines.put(if self.changed == 1 { b"changes; " } else { b"change; " });
        lines.decimal(u64::from(self.unchanged));
        lines.put(if self.unchanged == 1 { b" is unchanged.\n" } else { b" are unchanged.\n" });
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"from", &self.from);
        field(json, b"to", &self.to);
        json.key(b"targets");
        json.unsigned(u64::from(self.targets));
        json.key(b"changed");
        json.unsigned(u64::from(self.changed));
        json.key(b"unchanged");
        json.unsigned(u64::from(self.unchanged));
    }
}
