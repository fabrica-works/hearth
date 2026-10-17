//! The listing and typed documents (format.md, 2.1 and 3.4).
use alloc::boxed::Box;
use skein_lib::bytes;
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Oid(pub Box<[u8]>);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Mode {
    File,
    Executable,
    Link,
    Submodule,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub path: Box<[u8]>,
    pub mode: Mode,
    pub oid: Oid,
    pub size: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class<'a> {
    Fleet,
    Environment { env: &'a [u8] },
    Host { base: &'a [u8], name: &'a [u8] },
    Other,
}
#[must_use]
pub fn class(path: &[u8]) -> Class<'_> {
    if path == b"fleet.json" {
        return Class::Fleet;
    }
    if let Some(rest) = path.strip_prefix(b"envs/")
        && let Some(env) = rest.strip_suffix(b".json")
        && !env.is_empty()
        && bytes::find(env, b"/").is_none()
    {
        return Class::Environment { env };
    }
    if let Some(rest) = path.strip_prefix(b"hosts/")
        && let Some(slash) = bytes::find(rest, b"/")
    {
        let (base, rest) = rest.split_at_checked(slash).expect("found slash");
        let rest = rest.strip_prefix(b"/").expect("found slash");
        if let Some(name) = rest.strip_suffix(b".json")
            && !base.is_empty()
            && !name.is_empty()
            && bytes::find(name, b"/").is_none()
        {
            return Class::Host { base, name };
        }
    }
    Class::Other
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Document {
    pub at: Position,
    pub layer: Layer,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Layer {
    pub placement: Option<Placement>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Placement {
    pub at: Position,
    pub kind: Runs,
    pub firmware: Option<Firmware>,
    pub hardware: Option<Box<[u8]>>,
    pub on: Option<On>,
    pub mutable: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Runs {
    Metal,
    Vm,
    Container,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Firmware {
    Efi,
    Bios,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct On {
    pub target: Box<[u8]>,
    pub at: Position,
}
