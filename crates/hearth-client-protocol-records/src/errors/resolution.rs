//! Apt's configuration verdict and the bounded closure (domain/render.md, 5.5, 7.2).
use super::{Box, Encoder, Lines, field};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Resolution {
    TooLarge { max: u64 },
    Unknown { package: Box<[u8]> },
    Unsatisfiable { package: Option<Box<[u8]>> },
    Absent { uri: Box<[u8]>, suite: Box<[u8]> },
    Missing { package: Box<[u8]> },
    Unmatched { name: Box<[u8]>, version: Box<[u8]>, architecture: Box<[u8]> },
    Disagree { name: Box<[u8]>, version: Box<[u8]>, architecture: Box<[u8]> },
    TooMany { max: u32 },
}
impl Resolution {
    pub(super) fn text(&self, lines: &mut Lines) {
        match self {
            Self::Unknown { package } => {
                lines.put(b"apt cannot locate package ");
                lines.quote(package);
            }
            Self::Unsatisfiable { package } => {
                lines.put(b"unsatisfiable dependencies");
                if let Some(package) = package {
                    lines.put(b" of ");
                    lines.quote(package);
                }
            }
            Self::Absent { uri, suite } => {
                lines.put(b"the archive ");
                lines.put(uri);
                lines.put(b" does not have ");
                lines.put(suite);
            }
            Self::Missing { package } => {
                lines.quote(package);
                lines.put(b" was requested, and apt installs no package of that name");
            }
            Self::Unmatched { name, version, architecture } => {
                lines.put(b"apt would install ");
                triple(lines, name, version, architecture);
                lines.put(b", which no Packages index lists");
            }
            Self::Disagree { name, version, architecture } => {
                lines.put(b"the Packages indices disagree on ");
                triple(lines, name, version, architecture);
            }
            Self::TooLarge { max } => {
                lines.put(b"a resolution record past ");
                lines.decimal(*max);
                lines.put(b" bytes");
            }
            Self::TooMany { max } => {
                lines.put(b"a closure past ");
                lines.decimal(u64::from(*max));
                lines.put(b" packages");
            }
        }
    }
    pub(super) fn json(&self, json: &mut Encoder) {
        json.object_start();
        match self {
            Self::Unknown { package } => {
                field(json, b"kind", b"unknown");
                field(json, b"package", package);
            }
            Self::Unsatisfiable { package } => {
                field(json, b"kind", b"unsatisfiable");
                if let Some(package) = package {
                    field(json, b"package", package);
                }
            }
            Self::Absent { uri, suite } => {
                field(json, b"kind", b"absent");
                field(json, b"uri", uri);
                field(json, b"suite", suite);
            }
            Self::Missing { package } => {
                field(json, b"kind", b"missing");
                field(json, b"package", package);
            }
            Self::Unmatched { name, version, architecture } => {
                field(json, b"kind", b"unmatched");
                json_triple(json, name, version, architecture);
            }
            Self::Disagree { name, version, architecture } => {
                field(json, b"kind", b"disagree");
                json_triple(json, name, version, architecture);
            }
            Self::TooLarge { max } => {
                field(json, b"kind", b"too_large");
                json.key(b"max");
                json.unsigned(*max);
            }
            Self::TooMany { max } => {
                field(json, b"kind", b"too_many");
                json.key(b"max");
                json.unsigned(u64::from(*max));
            }
        }
        json.object_end();
    }
}

fn triple(lines: &mut Lines, name: &[u8], version: &[u8], architecture: &[u8]) {
    lines.put(name);
    lines.put(b" ");
    lines.put(version);
    lines.put(b" ");
    lines.put(architecture);
}
fn json_triple(json: &mut Encoder, name: &[u8], version: &[u8], architecture: &[u8]) {
    field(json, b"name", name);
    field(json, b"version", version);
    field(json, b"architecture", architecture);
}
