//! The packages record in client forms (render.md, 4.2).
use crate::form::text::Lines;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Packages {
    pub sources: Sources,
    pub request: Box<[Box<[u8]>]>,
    pub variant: Variant,
    pub preseeds: Box<[Preseed]>,
    pub initramfs: Option<Initramfs>,
    pub apt: Apt,
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
        if let Some(initramfs) = &self.initramfs {
            lines.put(b"  microcode  ");
            lines.put(initramfs.microcode.name());
            lines.put(b"\n");
        }
        lines.put(b"  apt        install_recommends ");
        lines.put(if self.apt.install_recommends { b"true" } else { b"false" });
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        json.key(b"apt");
        json.object_start();
        json.key(b"install_recommends");
        json.boolean(self.apt.install_recommends);
        json.object_end();
        if let Some(initramfs) = &self.initramfs {
            json.key(b"initramfs");
            json.object_start();
            json.key(b"microcode");
            json.string(initramfs.microcode.name());
            json.key(b"modules");
            names(json, &initramfs.modules);
            json.object_end();
        }
        json.key(b"preseeds");
        json.object_start();
        for preseed in &self.preseeds {
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
        json.key(b"request");
        names(json, &self.request);
        json.key(b"sources");
        json.object_start();
        json.key(b"architecture");
        json.string(&self.sources.architecture);
        json.key(b"archive");
        json.string(&self.sources.archive);
        json.key(b"components");
        json.array_start();
        for component in &self.sources.components {
            json.string(component.name());
        }
        json.array_end();
        if let Some(security) = &self.sources.security {
            json.key(b"security");
            json.string(security);
        }
        json.key(b"suite");
        json.string(&self.sources.suite);
        json.key(b"updates");
        json.boolean(self.sources.updates);
        json.object_end();
        json.key(b"variant");
        json.string(self.variant.name());
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
