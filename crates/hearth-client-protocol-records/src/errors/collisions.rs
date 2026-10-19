//! Both claimants and their namespace (domain/render.md, 3.7 and 7.3).
use crate::form::{json::field, text::Lines};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
/// A key produced twice by one task's expanded maps (protocol/format.md, 5.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Collision {
    Key { key: Box<[u8]> },
    Given { subject: Subject, name: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    Differs { subject: Subject, name: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    Shared { id: Id, value: u32, first: Box<[u8]>, second: Box<[u8]> },
    Hidden { path: Box<[u8]>, data: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    Overlap { subject: Subject, first: Box<[u8]>, second: Box<[u8]> },
}

/// The namespace of a collision (domain/render.md, 3.7).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Subject {
    Tree,
    DataName,
    DataPath,
    Secret,
    Identity,
    Download,
    User,
    Group,
    Preset,
    Tmpfiles,
    Preseed,
    Parameter,
    Range,
}
/// The account id namespace (protocol/format.md, 5.6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Id {
    Uid,
    Gid,
}

impl Collision {
    pub(super) fn text(&self, lines: &mut Lines) {
        match self {
            Self::Key { key } => {
                lines.put(b"the key ");
                lines.quote(key);
                lines.put(b" is made twice");
            }
            Self::Given { subject, name, first, second } => {
                subject.prefix(lines);
                lines.put(name);
                lines.put(b" is given by ");
                lines.put(first);
                lines.put(b" and by ");
                lines.put(second);
            }
            Self::Differs { subject, name, first, second } => {
                subject.prefix(lines);
                if *subject == Subject::Tmpfiles {
                    lines.put(name);
                } else {
                    lines.quote(name);
                }
                lines.put(b" differs between ");
                lines.put(first);
                lines.put(b" and ");
                lines.put(second);
            }
            Self::Shared { id, value, first, second } => {
                lines.put(id.name());
                lines.put(b" ");
                lines.decimal(u64::from(*value));
                lines.put(b" is shared by ");
                lines.put(first);
                lines.put(b" and ");
                lines.put(second);
            }
            Self::Hidden { path, data, .. } => {
                lines.put(path);
                lines.put(b" is under the data path ");
                lines.put(data);
                lines.put(b" (a mount would hide it)");
            }
            Self::Overlap { subject: Subject::Range, first, second } => {
                lines.put(b"subordinate ids of ");
                lines.put(first);
                lines.put(b" overlap ");
                lines.put(second);
                lines.put(b"'s");
            }
            Self::Overlap { subject, first, second } => {
                subject.prefix(lines);
                lines.put(first);
                lines.put(b" overlaps ");
                lines.put(second);
            }
        }
    }
    pub(super) fn json(&self, json: &mut Encoder) {
        json.object_start();
        match self {
            Self::Key { key } => {
                field(json, b"key", key);
                field(json, b"kind", b"key");
            }
            Self::Given { subject, name, first, second } => {
                field(json, b"kind", b"given");
                field(json, b"subject", subject.name());
                field(json, b"name", name);
                field(json, b"first", first);
                field(json, b"second", second);
            }
            Self::Differs { subject, name, first, second } => {
                field(json, b"kind", b"differs");
                field(json, b"subject", subject.name());
                field(json, b"name", name);
                field(json, b"first", first);
                field(json, b"second", second);
            }
            Self::Shared { id, value, first, second } => {
                field(json, b"kind", b"shared");
                field(json, b"id", id.name());
                json.key(b"value");
                json.unsigned(u64::from(*value));
                field(json, b"first", first);
                field(json, b"second", second);
            }
            Self::Hidden { path, data, first, second } => {
                field(json, b"kind", b"hidden");
                field(json, b"path", path);
                field(json, b"data", data);
                field(json, b"first", first);
                field(json, b"second", second);
            }
            Self::Overlap { subject, first, second } => {
                field(json, b"kind", b"overlap");
                field(json, b"subject", subject.name());
                field(json, b"first", first);
                field(json, b"second", second);
            }
        }
        json.object_end();
    }
}
impl Subject {
    fn name(self) -> &'static [u8] {
        match self {
            Self::Tree => b"tree",
            Self::DataName => b"data_name",
            Self::DataPath => b"data_path",
            Self::Secret => b"secret",
            Self::Identity => b"identity",
            Self::Download => b"download",
            Self::User => b"user",
            Self::Group => b"group",
            Self::Preset => b"preset",
            Self::Tmpfiles => b"tmpfiles",
            Self::Preseed => b"preseed",
            Self::Parameter => b"parameter",
            Self::Range => b"range",
        }
    }
    fn prefix(self, lines: &mut Lines) {
        lines.put(match self {
            Self::Tree => b"",
            Self::DataName => b"data ",
            Self::DataPath => b"data path ",
            Self::Secret => b"secret ",
            Self::Identity => b"identity ",
            Self::Download => b"download ",
            Self::User => b"user ",
            Self::Group => b"group ",
            Self::Preset => b"preset ",
            Self::Tmpfiles => b"tmpfiles ",
            Self::Preseed => b"preseed ",
            Self::Parameter => b"boot parameter ",
            Self::Range => b"subordinate ids ",
        });
    }
}

impl Id {
    const fn name(self) -> &'static [u8] {
        match self {
            Self::Uid => b"uid",
            Self::Gid => b"gid",
        }
    }
}
