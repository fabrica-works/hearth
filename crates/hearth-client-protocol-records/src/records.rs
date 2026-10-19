//! Printed units (domain/render.md, 3.8 and 4).
use crate::{
    diff::{Change, Tally},
    errors::Error,
    form::text::Lines,
    outcome::Outcome,
    parts::{config::Config, meta::Meta},
    startup::Startup,
};
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Record {
    Resolving(crate::progress::Resolving),
    Resolved(crate::progress::Resolved),
    Unresolved(crate::progress::Unresolved),
    Waiting(crate::progress::Waiting),
    Strays(crate::progress::Strays),
    Spec(Box<Spec>),
    Terminal(Terminal),
    Change(Change),
    Error(Error),
    Startup(Startup),
    Layer(crate::build::Layer),
    Failed(crate::build::Failed),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
    Resolving,
    Resolved,
    Unresolved,
    Waiting,
    Strays,
    Spec,
    Terminal,
    Change,
    Error,
    Startup,
    Layer,
    Failed,
}
impl Record {
    #[must_use]
    pub fn kind(&self) -> Type {
        match self {
            Self::Resolving(_) => Type::Resolving,
            Self::Resolved(_) => Type::Resolved,
            Self::Unresolved(_) => Type::Unresolved,
            Self::Waiting(_) => Type::Waiting,
            Self::Strays(_) => Type::Strays,
            Self::Spec(_) => Type::Spec,
            Self::Terminal(_) => Type::Terminal,
            Self::Change(_) => Type::Change,
            Self::Error(_) => Type::Error,
            Self::Startup(_) => Type::Startup,
            Self::Layer(_) => Type::Layer,
            Self::Failed(_) => Type::Failed,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Spec {
    pub meta: Meta,
    pub meta_digest: [u8; 32],
    pub commit: Box<[u8]>,
    pub digest: [u8; 32],
    pub parts: Box<[Part]>,
    pub resolution: Option<Resolution>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Part {
    pub digest: [u8; 32],
    pub record: PartRecord,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PartRecord {
    Config(Config),
    Data(crate::parts::data::Data),
    Secrets(crate::parts::secrets::Secrets),
    Packages(crate::parts::packages::Packages),
    Live(crate::parts::live::Live),
    Software(crate::parts::software::Software),
    Boot(crate::parts::boot::Boot),
    Exports(crate::parts::exports::Exports),
}
impl PartRecord {
    pub(crate) fn name(&self) -> &'static [u8] {
        match self {
            Self::Config(_) => b"config",
            Self::Data(_) => b"data",
            Self::Secrets(_) => b"secrets",
            Self::Packages(_) => b"packages",
            Self::Live(_) => b"live",
            Self::Software(_) => b"software",
            Self::Boot(_) => b"boot",
            Self::Exports(_) => b"exports",
        }
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        match self {
            Self::Config(config) => config.json(json),
            Self::Data(data) => data.json(json),
            Self::Secrets(secrets) => secrets.json(json),
            Self::Packages(packages) => packages.json(json),
            Self::Live(live) => live.json(json),
            Self::Software(software) => software.json(json),
            Self::Boot(boot) => boot.json(json),
            Self::Exports(exports) => exports.json(json),
        }
    }
    pub(crate) fn text(&self, lines: &mut Lines) {
        match self {
            Self::Config(config) => config.text(lines),
            Self::Data(data) => data.text(lines),
            Self::Secrets(secrets) => secrets.text(lines),
            Self::Packages(packages) => packages.text(lines),
            Self::Live(live) => live.text(lines),
            Self::Software(software) => software.text(lines),
            Self::Boot(boot) => boot.text(lines),
            Self::Exports(exports) => exports.text(lines),
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Terminal {
    pub outcome: Outcome,
    pub diff: Option<Tally>,
    pub left_out: u32,
}

/// The resolution key and, when known, its layer keys (domain/render.md, 4.5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Resolution {
    Needed { key: [u8; 32] },
    Known { key: [u8; 32], keys: Keys },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Keys {
    pub packages: [u8; 32],
    pub software: [u8; 32],
    pub config: [u8; 32],
}
