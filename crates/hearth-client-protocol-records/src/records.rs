//! Printed units (render.md, 3.8 and 4).
use crate::{
    form::text::Lines,
    outcome::Outcome,
    parts::{config::Config, meta::Meta},
};
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Record {
    Spec(Box<Spec>),
    Terminal(Terminal),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
    Spec,
    Terminal,
}
impl Record {
    #[must_use]
    pub fn kind(&self) -> Type {
        match self {
            Self::Spec(_) => Type::Spec,
            Self::Terminal(_) => Type::Terminal,
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
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Part {
    pub digest: [u8; 32],
    pub record: PartRecord,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PartRecord {
    Config(Config),
}
impl PartRecord {
    pub(crate) fn name(&self) -> &'static [u8] {
        match self {
            Self::Config(_) => b"config",
        }
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        match self {
            Self::Config(config) => config.json(json),
        }
    }
    pub(crate) fn text(&self, lines: &mut Lines) {
        match self {
            Self::Config(config) => config.text(lines),
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Terminal {
    pub outcome: Outcome,
}
