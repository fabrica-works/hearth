//! Evaluation messages and undeclared-setting diagnostics (domain/render.md, 7.3).
use super::{Encoder, Lines};
use alloc::boxed::Box;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Evaluation {
    Unknown,
    NoKey,
    NoIndex,
    Undefined,
    Type,
    Overflow,
    Empty,
    Path,
    Output,
    Limit,
}
impl Evaluation {
    pub(super) fn message(self) -> &'static [u8] {
        match self {
            Self::Unknown => b"an unknown name",
            Self::NoKey => b"a missing key",
            Self::NoIndex => b"an index outside the list",
            Self::Undefined => b"an undefined value used",
            Self::Type => b"a value of the wrong type",
            Self::Overflow => b"an integer overflow",
            Self::Empty => b"an empty list or pattern",
            Self::Path => b"a path systemd_escape_path refuses",
            Self::Output => b"a list or a map output",
            Self::Limit => b"a limit exceeded",
        }
    }
    pub(super) fn name(self) -> &'static [u8] {
        match self {
            Self::Unknown => b"unknown",
            Self::NoKey => b"no_key",
            Self::NoIndex => b"no_index",
            Self::Undefined => b"undefined",
            Self::Type => b"type",
            Self::Overflow => b"overflow",
            Self::Empty => b"empty",
            Self::Path => b"path",
            Self::Output => b"output",
            Self::Limit => b"limit",
        }
    }
}
pub(super) fn undeclared(lines: &mut Lines, task: &[u8], setting: &[u8], declared: &[Box<[u8]>]) {
    lines.put(b"undeclared setting ");
    lines.quote(setting);
    lines.put(b"; ");
    lines.put(task);
    if declared.is_empty() {
        lines.put(b" declares no settings");
    } else {
        lines.put(b" declares ");
        names(lines, declared);
    }
}

pub(super) fn signed(lines: &mut Lines, value: i64) {
    if value < 0 {
        lines.put(b"-");
    }
    lines.decimal(value.unsigned_abs());
}
pub(super) fn names(lines: &mut Lines, names: &[Box<[u8]>]) {
    for (index, name) in names.iter().enumerate() {
        if index != 0 {
            lines.put(b", ");
        }
        lines.put(name);
    }
}
pub(super) fn json_names(json: &mut Encoder, names: &[Box<[u8]>]) {
    json.array_start();
    for name in names {
        json.string(name);
    }
    json.array_end();
}

impl super::Required {
    pub(super) fn name(self) -> &'static [u8] {
        match self {
            Self::If => b"if",
            Self::Message => b"message",
            Self::Type => b"type",
            Self::Doc => b"doc",
            Self::Value => b"value",
        }
    }
}
