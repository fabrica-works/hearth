use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Preset {
    pub unit: Box<[u8]>,
    pub action: Action,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    Enable,
    Disable,
}
impl Action {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Enable => b"enable",
            Self::Disable => b"disable",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tmpfile {
    pub path: Box<[u8]>,
    pub line: Line,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line {
    pub kind: LineKind,
    pub mode: Option<u16>,
    pub owner: Option<Box<[u8]>>,
    pub group: Option<Box<[u8]>>,
    pub age: Option<Box<[u8]>>,
    pub argument: Option<Box<[u8]>>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LineKind {
    D,
    CapitalD,
    E,
    F,
    FPlus,
    W,
    WPlus,
    CapitalL,
    CapitalLPlus,
    CapitalC,
    Z,
    CapitalZ,
    R,
    CapitalR,
}
impl LineKind {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::D => b"d",
            Self::CapitalD => b"D",
            Self::E => b"e",
            Self::F => b"f",
            Self::FPlus => b"f+",
            Self::W => b"w",
            Self::WPlus => b"w+",
            Self::CapitalL => b"L",
            Self::CapitalLPlus => b"L+",
            Self::CapitalC => b"C",
            Self::Z => b"z",
            Self::CapitalZ => b"Z",
            Self::R => b"r",
            Self::CapitalR => b"R",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Script {
    pub task: Box<[u8]>,
    pub order: u16,
    pub content_digest: [u8; 32],
}

use crate::form::{
    json::{digest, field},
    text::Lines,
};
use skein_json::writer::Encoder;
impl Action {
    pub(crate) fn text(self, lines: &mut Lines) {
        lines.put(self.name());
    }
    pub(crate) fn json(self, json: &mut Encoder) {
        json.object_start();
        field(json, b"kind", self.name());
        json.object_end();
    }
}
impl Line {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(self.kind.name());
        lines.put(b" ");
        match self.mode {
            Some(mode) => lines.mode(mode),
            None => lines.put(b"-"),
        }
        for value in [&self.owner, &self.group] {
            lines.put(b" ");
            lines.put(value.as_deref().unwrap_or(b"-"));
        }
        if self.age.is_some() || self.argument.is_some() {
            lines.put(b" ");
            lines.put(self.age.as_deref().unwrap_or(b"-"));
        }
        if let Some(argument) = &self.argument {
            lines.put(b" ");
            lines.put(argument);
        }
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        json.object_start();
        optional(json, b"age", self.age.as_deref());
        optional(json, b"argument", self.argument.as_deref());
        optional(json, b"group", self.group.as_deref());
        if let Some(mode) = self.mode {
            json.key(b"mode");
            json.string(&super::octal(mode));
        }
        optional(json, b"owner", self.owner.as_deref());
        field(json, b"type", self.kind.name());
        json.object_end();
    }
}
fn optional(json: &mut Encoder, key: &[u8], value: Option<&[u8]>) {
    if let Some(value) = value {
        field(json, key, value);
    }
}
impl Script {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(&self.task);
        lines.put(b" ");
        lines.decimal(u64::from(self.order));
        lines.put(b" ");
        lines.hex12(&self.content_digest);
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        json.object_start();
        json.key(b"content_digest");
        digest(json, &self.content_digest);
        json.key(b"order");
        json.unsigned(u64::from(self.order));
        field(json, b"task", &self.task);
        json.object_end();
    }
}
impl super::Config {
    pub(super) fn directives_text(&self, lines: &mut Lines) {
        for row in &self.presets {
            lines.put(b"  presets    ");
            lines.put(&row.unit);
            lines.put(b" ");
            row.action.text(lines);
            lines.put(b"\n");
        }
        for row in &self.tmpfiles {
            lines.put(b"  tmpfiles   ");
            lines.put(&row.path);
            lines.put(b" ");
            row.line.text(lines);
            lines.put(b"\n");
        }
        for row in &self.scripts {
            lines.put(b"  scripts    ");
            row.text(lines);
            lines.put(b"\n");
        }
    }
    pub(super) fn directives_json(&self, json: &mut Encoder) {
        json.key(b"presets");
        json.object_start();
        for row in &self.presets {
            json.key(&row.unit);
            row.action.json(json);
        }
        json.object_end();
        json.key(b"scripts");
        scripts_json(&self.scripts, json);
        json.key(b"tmpfiles");
        json.object_start();
        for row in &self.tmpfiles {
            json.key(&row.path);
            row.line.json(json);
        }
        json.object_end();
    }
}
pub(crate) fn scripts_json(scripts: &[Script], json: &mut Encoder) {
    json.array_start();
    for script in scripts {
        script.json(json);
    }
    json.array_end();
}
