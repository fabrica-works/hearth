//! Printed units (render.md, 3.8 and 4).
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
    Waiting(crate::progress::Waiting),
    Strays(crate::progress::Strays),
    Spec(Box<Spec>),
    Terminal(Terminal),
    Change(Change),
    Error(Error),
    Startup(Startup),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
    Waiting,
    Strays,
    Spec,
    Terminal,
    Change,
    Error,
    Startup,
}
impl Record {
    #[must_use]
    pub fn kind(&self) -> Type {
        match self {
            Self::Waiting(_) => Type::Waiting,
            Self::Strays(_) => Type::Strays,
            Self::Spec(_) => Type::Spec,
            Self::Terminal(_) => Type::Terminal,
            Self::Change(_) => Type::Change,
            Self::Error(_) => Type::Error,
            Self::Startup(_) => Type::Startup,
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
    Secrets(crate::parts::secrets::Secrets),
    Packages(crate::parts::packages::Packages),
}
impl PartRecord {
    pub(crate) fn name(&self) -> &'static [u8] {
        match self {
            Self::Config(_) => b"config",
            Self::Secrets(_) => b"secrets",
            Self::Packages(_) => b"packages",
        }
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        match self {
            Self::Config(config) => config.json(json),
            Self::Secrets(secrets) => secrets.json(json),
            Self::Packages(packages) => packages.json(json),
        }
    }
    pub(crate) fn text(&self, lines: &mut Lines) {
        match self {
            Self::Config(config) => config.text(lines),
            Self::Secrets(secrets) => secrets.text(lines),
            Self::Packages(packages) => packages.text(lines),
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Terminal {
    pub outcome: Outcome,
    pub diff: Option<Tally>,
    pub left_out: u32,
}

/// The resolution key and, when known, its layer keys (render.md, 4.5).
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
