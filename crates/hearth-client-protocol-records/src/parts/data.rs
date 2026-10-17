//! Persistent data records (render.md, 4.3).
use crate::form::{json::field, text::Lines};
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Data {
    pub entries: Box<[DataEntry]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DataEntry {
    pub name: Box<[u8]>,
    pub value: DataValue,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DataValue {
    pub class: DataClass,
    pub path: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DataClass {
    Core,
    Data,
    Cache,
}
impl DataClass {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Core => b"core",
            Self::Data => b"data",
            Self::Cache => b"cache",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DataChange {
    pub entries: Box<[DataKeyed]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DataKeyed {
    Added { key: Box<[u8]>, value: DataValue },
    Removed { key: Box<[u8]>, value: DataValue },
    Changed { key: Box<[u8]>, before: DataValue, after: DataValue },
}

impl DataValue {
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        json.key(b"class");
        json.object_start();
        field(json, b"kind", self.class.name());
        json.object_end();
        field(json, b"path", &self.path);
        json.object_end();
    }
}
impl Data {
    pub(crate) fn text(&self, lines: &mut Lines) {
        for entry in &self.entries {
            lines.put(b"  ");
            lines.put(&entry.name);
            lines.pad_to(13);
            lines.put(&entry.value.path);
            lines.put(b"\n");
        }
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        json.key(b"entries");
        json.object_start();
        for entry in &self.entries {
            json.key(&entry.name);
            entry.value.json(json);
        }
        json.object_end();
        json.object_end();
    }
}
