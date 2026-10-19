//! A build's records: each layer, and the step that ended it (client/README.md, 3.1 and 3.2).
use crate::form::{
    json::{digest, field},
    text::Lines,
};
use skein_json::writer::Encoder;
/// One layer of the output, bottom-up.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Layer {
    pub kind: Kind,
    pub key: [u8; 32],
    pub state: State,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Packages,
    Software,
    Config,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum State {
    Found,
    ToBuild,
    Built,
}
/// The step a build ended on, in its layer.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Failed {
    pub layer: Kind,
    pub step: Step,
    pub why: Why,
}
/// A layer's steps, by their names in a layer's costs (build.md, 5 to 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    Fetch,
    Mmdebstrap,
    Verify,
    Place,
    Software,
    Users,
    Tree,
    Tmpfiles,
    Presets,
    Customize,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    Unavailable,
}
impl Kind {
    const fn name(self) -> &'static [u8] {
        match self {
            Self::Packages => b"packages",
            Self::Software => b"software",
            Self::Config => b"config",
        }
    }
}
impl State {
    const fn name(self) -> &'static [u8] {
        match self {
            Self::Found => b"found",
            Self::ToBuild => b"to build",
            Self::Built => b"built",
        }
    }
    const fn json(self) -> &'static [u8] {
        match self {
            Self::Found => b"found",
            Self::ToBuild => b"to_build",
            Self::Built => b"built",
        }
    }
}
impl Step {
    const fn name(self) -> &'static [u8] {
        match self {
            Self::Fetch => b"fetch",
            Self::Mmdebstrap => b"mmdebstrap",
            Self::Verify => b"verify",
            Self::Place => b"place",
            Self::Software => b"software.sh",
            Self::Users => b"users",
            Self::Tree => b"tree",
            Self::Tmpfiles => b"tmpfiles",
            Self::Presets => b"presets",
            Self::Customize => b"customize.sh",
        }
    }
}
impl Why {
    const fn name(self) -> &'static [u8] {
        match self {
            Self::Unavailable => b"unavailable",
        }
    }
}
const COLUMN: usize = 13;
impl Layer {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(self.kind.name());
        lines.pad_to(COLUMN);
        lines.hex12(&self.key);
        lines.put(b"   ");
        lines.put(self.state.name());
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"record", b"layer");
        field(json, b"layer", self.kind.name());
        json.key(b"key");
        digest(json, &self.key);
        field(json, b"state", self.state.json());
    }
}
impl Failed {
    pub(crate) fn text(self, lines: &mut Lines) {
        lines.put(b"failed");
        lines.pad_to(COLUMN);
        lines.put(self.step.name());
        lines.put(b" ");
        lines.put(self.layer.name());
        lines.put(b"   ");
        lines.put(self.why.name());
        lines.put(b"\n");
    }
    pub(crate) fn json(self, json: &mut Encoder) {
        field(json, b"record", b"failed");
        field(json, b"layer", self.layer.name());
        field(json, b"step", self.step.name());
        field(json, b"why", self.why.name());
    }
}
