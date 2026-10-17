//! The packages record and its four resolution inputs (render.md, 4.2, 5.1).
use alloc::boxed::Box;
use hearth_lib::{
    encode::{Digest, Item},
    hash::H,
};
use skein_lib::{List, bytes};
mod build;
pub use build::{build, read};
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
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Inputs {
    pub sources: Box<[Item]>,
    pub request: Box<[Item]>,
    pub variant: Box<[Item]>,
    pub apt: Box<[Item]>,
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
fn text(value: &[u8]) -> Item {
    Item::Bytes(bytes::copy_of(value))
}
fn boolean(value: bool) -> Item {
    if value { Item::True } else { Item::False }
}
fn push(items: &mut List<Item>, item: Item) {
    items.push(item).expect("record item count");
}
fn field(items: &mut List<Item>, name: &[u8], value: Item) {
    push(items, text(name));
    push(items, value);
}
fn count(length: usize) -> u64 {
    u64::try_from(length).expect("bounded record")
}
#[must_use]
pub fn inputs(packages: &Packages) -> Inputs {
    let source = &packages.sources;
    let mut sources = List::with_capacity(19);
    push(&mut sources, Item::Map(5_u64.saturating_add(u64::from(source.security.is_some()))));
    field(&mut sources, b"architecture", text(&source.architecture));
    field(&mut sources, b"archive", text(&source.archive));
    field(&mut sources, b"components", Item::List(count(source.components.len())));
    for component in &source.components {
        push(&mut sources, text(component.name()));
    }
    if let Some(security) = &source.security {
        field(&mut sources, b"security", text(security));
    }
    field(&mut sources, b"suite", text(&source.suite));
    field(&mut sources, b"updates", boolean(source.updates));
    let mut request =
        List::with_capacity(u32::try_from(packages.request.len()).expect("bounded request").saturating_add(1));
    push(&mut request, Item::List(count(packages.request.len())));
    for name in &packages.request {
        push(&mut request, text(name));
    }
    Inputs {
        sources: sources.into_boxed(),
        request: request.into_boxed(),
        variant: Box::new([text(packages.variant.name())]),
        apt: Box::new([Item::Map(1), text(b"install_recommends"), boolean(packages.apt.install_recommends)]),
    }
}
#[must_use]
pub fn digest(packages: &Packages) -> Digest {
    let inputs = inputs(packages);
    let mut hash = H::new(b"part:packages", 1);
    let record = hash.values();
    record.map(5_u64.saturating_add(u64::from(packages.initramfs.is_some())));
    record.key(b"apt");
    for item in &inputs.apt {
        record.item(item);
    }
    if let Some(initramfs) = &packages.initramfs {
        record.key(b"initramfs");
        record.map(2);
        record.key(b"microcode");
        record.bytes(initramfs.microcode.name());
        record.key(b"modules");
        record.list(count(initramfs.modules.len()));
        for module in &initramfs.modules {
            record.bytes(module);
        }
    }
    record.key(b"preseeds");
    record.map(count(packages.preseeds.len()));
    for preseed in &packages.preseeds {
        record.key(&preseed.question);
        record.map(3);
        record.key(b"owner");
        record.bytes(&preseed.owner);
        record.key(b"type");
        record.bytes(preseed.kind.name());
        record.key(b"value");
        record.bytes(&preseed.value);
    }
    for (name, items) in
        [(b"request".as_slice(), &inputs.request), (b"sources", &inputs.sources), (b"variant", &inputs.variant)]
    {
        record.key(name);
        for item in items {
            record.item(item);
        }
    }
    hash.finish()
}
