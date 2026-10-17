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
    Declaration { task: &'a [u8] },
    Tree { task: &'a [u8], path: &'a [u8] },
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
    if let Some(rest) = path.strip_prefix(b"tasks/")
        && let Some(slash) = bytes::find(rest, b"/")
    {
        let (task, rest) = rest.split_at_checked(slash).expect("found slash");
        if !task.is_empty() {
            if rest == b"/task.json" {
                return Class::Declaration { task };
            }
            if let Some(path) = rest.strip_prefix(b"/tree/")
                && !path.is_empty()
            {
                return Class::Tree { task, path };
            }
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
    pub tasks: Box<[Task]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Task {
    pub name: Box<[u8]>,
    pub at: Position,
    pub value: Value,
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

/// Decoded configuration values with positions (render.md, 3.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Value {
    pub at: Position,
    pub kind: Kind,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Null,
    Bool(bool),
    Int(i64),
    String(Box<[u8]>),
    List(Box<[Value]>),
    Map(Box<[Pair]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Pair {
    pub key: Box<[u8]>,
    pub at: Position,
    pub value: Value,
}
/// Documents in listing path order, including refused documents.
#[derive(Debug)]
pub struct Documents {
    entries: Box<[Entry]>,
    decoded: Box<[Option<Document>]>,
}
impl Documents {
    #[must_use]
    pub fn new(entries: &[Entry]) -> Self {
        let capacity = u32::try_from(entries.len()).expect("bounded listing");
        let mut listed = skein_lib::Map::with_capacity(capacity);
        let mut decoded = skein_lib::List::with_capacity(capacity);
        for entry in entries {
            match class(&entry.path) {
                Class::Fleet | Class::Environment { .. } | Class::Host { .. } => {
                    listed.insert(entry.path.clone(), entry.clone()).expect("listing capacity");
                    decoded.push(None).expect("listing capacity");
                }
                Class::Declaration { .. } | Class::Tree { .. } | Class::Other => {}
            }
        }
        let mut entries = skein_lib::List::with_capacity(listed.len());
        for (_, entry) in &listed {
            entries.push(entry.clone()).expect("listing capacity");
        }
        Self { entries: entries.into_boxed(), decoded: decoded.into_boxed() }
    }
    #[must_use]
    pub fn path(&self, document: u32) -> &[u8] {
        &self.entries.get(usize::try_from(document).expect("u32 fits")).expect("document index").path
    }
    #[must_use]
    pub fn get(&self, document: u32) -> Option<&Document> {
        self.decoded.get(usize::try_from(document).ok()?)?.as_ref()
    }
    pub fn set(&mut self, oid: &Oid, document: Document) {
        for (entry, decoded) in self.entries.iter().zip(self.decoded.iter_mut()) {
            if &entry.oid == oid {
                *decoded = Some(document.clone());
            }
        }
    }
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    #[must_use]
    pub fn find(&self, path: &[u8]) -> Option<u32> {
        for (index, entry) in self.entries.iter().enumerate() {
            if entry.path.as_ref() == path {
                return u32::try_from(index).ok();
            }
        }
        None
    }
}
/// Listed bytes admitted before decoding a revision (render.md, 2.3).
#[derive(Debug)]
pub struct Budget {
    held: u64,
    max: u64,
}
impl Budget {
    #[must_use]
    pub const fn new(limits: &crate::limits::Limits) -> Self {
        Self { held: 0, max: limits.decoded }
    }
    #[must_use]
    pub fn take(&mut self, size: u64) -> bool {
        let Some(held) = self.held.checked_add(size) else {
            return false;
        };
        if held > self.max {
            return false;
        }
        self.held = held;
        true
    }
}
