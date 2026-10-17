//! Apt's configuration verdict and the bounded closure (render.md, 5.5, 7.2).
use super::{Box, Encoder, Lines, field};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Resolution {
    Unknown { package: Box<[u8]> },
    Unsatisfiable { package: Option<Box<[u8]>> },
    Absent { uri: Box<[u8]>, suite: Box<[u8]> },
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
            Self::TooMany { max } => {
                field(json, b"kind", b"too_many");
                json.key(b"max");
                json.unsigned(u64::from(*max));
            }
        }
        json.object_end();
    }
}
