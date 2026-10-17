//! Store progress on standard error (client/README.md, 3.2).
use crate::form::{json::field, text::Lines};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Waiting {
    pub store: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Strays {
    pub store: Box<[u8]>,
    pub count: u64,
    pub first: Box<[u8]>,
}
impl Waiting {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"waiting for the store's lock: ");
        lines.put(&self.store);
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"record", b"waiting");
        field(json, b"store", &self.store);
    }
}
impl Strays {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"strays in the store ");
        lines.put(&self.store);
        lines.put(b": ");
        lines.decimal(self.count);
        lines.put(b", the first ");
        lines.put(&self.first);
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"record", b"strays");
        field(json, b"store", &self.store);
        json.key(b"count");
        json.unsigned(self.count);
        field(json, b"first", &self.first);
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StoreError {
    pub store: Box<[u8]>,
    pub fault: StoreFault,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StoreFault {
    Full { limit: StoreLimit, most: u32 },
    Failed { step: Step },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StoreLimit {
    Resolutions,
    CacheFiles,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    Write,
    List,
    Remove,
    Load,
    Empty,
    Keep,
}
impl Step {
    fn name(self) -> &'static [u8] {
        match self {
            Self::List => b"list",
            Self::Remove => b"remove",
            Self::Load => b"load",
            Self::Empty => b"empty",
            Self::Keep => b"keep",
            Self::Write => b"write",
        }
    }
}
impl StoreError {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"the store ");
        lines.put(&self.store);
        match self.fault {
            StoreFault::Full { limit, most } => {
                lines.put(b" is full: ");
                lines.decimal(u64::from(most));
                lines.put(match limit {
                    StoreLimit::Resolutions => b" resolutions",
                    StoreLimit::CacheFiles => b" cache files",
                });
            }
            StoreFault::Failed { step } => {
                lines.put(b": its ");
                lines.put(step.name());
                lines.put(b" failed");
            }
        }
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"store", &self.store);
        match self.fault {
            StoreFault::Full { limit, most } => {
                field(
                    json,
                    b"limit",
                    match limit {
                        StoreLimit::Resolutions => b"resolutions",
                        StoreLimit::CacheFiles => b"cache_files",
                    },
                );
                json.key(b"most");
                json.unsigned(u64::from(most));
            }
            StoreFault::Failed { step } => field(json, b"step", step.name()),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Resolving {
    pub key: [u8; 32],
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Resolved {
    pub key: [u8; 32],
    pub took: skein_lib::Duration,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Unresolved {
    pub key: [u8; 32],
}
impl Resolving {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"resolving ");
        lines.hex12(&self.key);
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        resolution(json, b"resolving", &self.key);
    }
}
impl Resolved {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"resolved ");
        lines.hex12(&self.key);
        lines.put(b" in ");
        lines.decimal(self.took.as_nanos() / 1_000_000);
        lines.put(b" ms\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        resolution(json, b"resolved", &self.key);
        json.key(b"took_ms");
        json.unsigned(self.took.as_nanos() / 1_000_000);
    }
}
impl Unresolved {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"resolution ");
        lines.hex12(&self.key);
        lines.put(b" failed\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        resolution(json, b"unresolved", &self.key);
    }
}
fn resolution(json: &mut Encoder, kind: &[u8], key: &[u8; 32]) {
    field(json, b"record", kind);
    json.key(b"key");
    crate::form::json::digest(json, key);
}
