//! The packages record in client forms (render.md, 4.2).
use crate::form::text::Lines;
use alloc::boxed::Box;
mod change;
mod seeds;
pub(crate) use change::value_text;
pub use change::*;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Packages {
    pub initramfs_file: Option<[u8; 32]>,
    pub sources: Sources,
    pub request: Box<[Box<[u8]>]>,
    pub variant: Variant,
    pub preseeds: Box<[Preseed]>,
    pub initramfs: Option<Initramfs>,
    pub apt: Apt,
    pub seeds: Seeds,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Seeds {
    pub users: Box<[SeedUser]>,
    pub groups: Box<[SeedGroup]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SeedUser {
    pub name: Box<[u8]>,
    pub uid: u32,
    pub gid: Option<u32>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SeedGroup {
    pub name: Box<[u8]>,
    pub gid: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Sources {
    pub suite: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub components: Box<[Component]>,
    pub archive: Box<[u8]>,
    pub updates: bool,
    pub security: Option<Box<[u8]>>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Component {
    Main,
    Contrib,
    NonFree,
    NonFreeFirmware,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Variant {
    Apt,
    Container,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Apt {
    pub install_recommends: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Preseed {
    pub question: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub kind: PreseedKind,
    pub value: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PreseedKind {
    Select,
    Multiselect,
    String,
    Boolean,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Initramfs {
    pub modules: Box<[Box<[u8]>]>,
    pub microcode: Microcode,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Microcode {
    None,
    Generic,
}
impl Component {
    pub const ALL: [Self; 4] = [Self::Main, Self::Contrib, Self::NonFree, Self::NonFreeFirmware];
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Main => b"main",
            Self::Contrib => b"contrib",
            Self::NonFree => b"non-free",
            Self::NonFreeFirmware => b"non-free-firmware",
        }
    }
}
impl Variant {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Apt => b"apt",
            Self::Container => b"container",
        }
    }
}
impl PreseedKind {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Select => b"select",
            Self::Multiselect => b"multiselect",
            Self::String => b"string",
            Self::Boolean => b"boolean",
        }
    }
}
impl Microcode {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::None => b"none",
            Self::Generic => b"generic",
        }
    }
}
impl Packages {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"  sources    ");
        lines.put(&self.sources.suite);
        lines.put(b" ");
        lines.put(&self.sources.architecture);
        for component in &self.sources.components {
            lines.put(b" ");
            lines.put(component.name());
        }
        lines.put(b" ");
        lines.put(&self.sources.archive);
        if self.sources.updates {
            lines.put(b" updates");
        }
        if let Some(security) = &self.sources.security {
            lines.put(b" security ");
            lines.put(security);
        }
        lines.put(b"\n");
        if !self.request.is_empty() {
            lines.put(b"  request   ");
            for name in &self.request {
                lines.put(b" ");
                lines.put(name);
            }
            lines.put(b"\n");
        }
        lines.put(b"  variant    ");
        lines.put(self.variant.name());
        lines.put(b"\n");
        for preseed in &self.preseeds {
            lines.put(b"  preseed    ");
            lines.put(&preseed.question);
            lines.put(b" ");
            lines.put(preseed.kind.name());
            lines.put(b" ");
            lines.put(&preseed.value);
            if !preseed.question.starts_with(&preseed.owner) || preseed.question.get(preseed.owner.len()) != Some(&b'/')
            {
                lines.put(b" owner ");
                lines.put(&preseed.owner);
            }
            lines.put(b"\n");
        }
        if let Some(initramfs) = &self.initramfs {
            if !initramfs.modules.is_empty() {
                lines.put(b"  initramfs ");
                for module in &initramfs.modules {
                    lines.put(b" ");
                    lines.put(module);
                }
                lines.put(b"\n");
            }
            lines.put(b"  microcode  ");
            lines.put(initramfs.microcode.name());
            lines.put(b"\n");
        }
        lines.put(b"  apt        install_recommends ");
        lines.put(if self.apt.install_recommends { b"true" } else { b"false" });
        lines.put(b"\n");
        self.seeds.text(lines);
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        json.key(b"apt");
        apt(json, &self.apt);
        if self.initramfs.is_some() {
            json.key(b"initramfs");
            initramfs(json, self.initramfs.as_ref());
        }
        json.key(b"preseeds");
        preseeds(json, &self.preseeds);
        json.key(b"request");
        names(json, &self.request);
        json.key(b"seeds");
        self.seeds.json(json);
        json.key(b"sources");
        sources(json, &self.sources);
        json.key(b"variant");
        variant(json, self.variant);
        json.object_end();
    }
}
fn names(json: &mut skein_json::writer::Encoder, names: &[Box<[u8]>]) {
    json.array_start();
    for name in names {
        json.string(name);
    }
    json.array_end();
}

fn apt(json: &mut skein_json::writer::Encoder, value: &Apt) {
    json.object_start();
    json.key(b"install_recommends");
    json.boolean(value.install_recommends);
    json.object_end();
}

fn initramfs(json: &mut skein_json::writer::Encoder, value: Option<&Initramfs>) {
    if let Some(initramfs) = value {
        json.object_start();
        json.key(b"microcode");
        json.string(initramfs.microcode.name());
        json.key(b"modules");
        names(json, &initramfs.modules);
        json.object_end();
    } else {
        json.null();
    }
}

fn preseeds(json: &mut skein_json::writer::Encoder, value: &[Preseed]) {
    json.object_start();
    for preseed in value {
        json.key(&preseed.question);
        json.object_start();
        json.key(b"owner");
        json.string(&preseed.owner);
        json.key(b"type");
        json.string(preseed.kind.name());
        json.key(b"value");
        json.string(&preseed.value);
        json.object_end();
    }
    json.object_end();
}

fn sources(json: &mut skein_json::writer::Encoder, value: &Sources) {
    json.object_start();
    json.key(b"architecture");
    json.string(&value.architecture);
    json.key(b"archive");
    json.string(&value.archive);
    json.key(b"components");
    json.array_start();
    for component in &value.components {
        json.string(component.name());
    }
    json.array_end();
    if let Some(security) = &value.security {
        json.key(b"security");
        json.string(security);
    }
    json.key(b"suite");
    json.string(&value.suite);
    json.key(b"updates");
    json.boolean(value.updates);
    json.object_end();
}

fn variant(json: &mut skein_json::writer::Encoder, value: Variant) {
    json.string(value.name());
}
