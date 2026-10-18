//! Live consumers (render.md, 4.4).
use crate::form::text::Lines;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Live {
    pub paths: Box<[Covered]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Covered {
    pub path: Box<[u8]>,
    pub consumers: Box<[Consumer]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Consumer {
    pub task: Box<[u8]>,
    pub unit: Option<Box<[u8]>>,
    pub action: Action,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    Reload,
    Restart,
    ReloadOrRestart,
    Start,
    None,
}
impl Action {
    fn name(self) -> &'static [u8] {
        match self {
            Self::Reload => b"reload",
            Self::Restart => b"restart",
            Self::ReloadOrRestart => b"reload-or-restart",
            Self::Start => b"start",
            Self::None => b"none",
        }
    }
}
impl Live {
    pub(crate) fn text(&self, lines: &mut Lines) {
        for covered in &self.paths {
            for consumer in &covered.consumers {
                lines.put(b"  path");
                lines.pad_to(13);
                lines.put(&covered.path);
                lines.put(b" ");
                consumer.text(lines);
                lines.put(b"\n");
            }
        }
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        for covered in &self.paths {
            json.key(&covered.path);
            json.array_start();
            for consumer in &covered.consumers {
                consumer.json(json);
            }
            json.array_end();
        }
        json.object_end();
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub paths: Box<[ConsumersKeyed]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ConsumersKeyed {
    Added { key: Box<[u8]>, value: Box<[Consumer]> },
    Removed { key: Box<[u8]>, value: Box<[Consumer]> },
    Changed { key: Box<[u8]>, before: Box<[Consumer]>, after: Box<[Consumer]> },
}

impl Consumer {
    fn text(&self, lines: &mut Lines) {
        lines.put(&self.task);
        lines.put(b" ");
        lines.put(self.unit.as_deref().unwrap_or(b"-"));
        lines.put(b" ");
        lines.put(self.action.name());
    }
    fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        json.key(b"action");
        json.string(self.action.name());
        json.key(b"task");
        json.string(&self.task);
        if let Some(unit) = &self.unit {
            json.key(b"unit");
            json.string(unit);
        }
        json.object_end();
    }
}

type ConsumerSides<'a> = (&'a [u8], Option<&'a [Consumer]>, Option<&'a [Consumer]>);
impl ConsumersKeyed {
    fn sides(&self) -> ConsumerSides<'_> {
        match self {
            Self::Added { key, value } => (key, None, Some(value.as_ref())),
            Self::Removed { key, value } => (key, Some(value.as_ref()), None),
            Self::Changed { key, before, after } => (key, Some(before.as_ref()), Some(after.as_ref())),
        }
    }
}
impl Change {
    pub(crate) fn text(&self, lines: &mut Lines) {
        for entry in &self.paths {
            let (key, before, after) = entry.sides();
            for consumer in after.or(before).expect("changed path") {
                crate::diff::placed::header(lines, b"live", key, before.is_some(), after.is_some());
                lines.pad_to(52);
                consumer.text(lines);
                lines.put(b"\n");
            }
        }
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        crate::form::json::field(json, b"kind", b"live");
        json.key(b"live");
        json.object_start();
        json.key(b"paths");
        json.array_start();
        for entry in &self.paths {
            let (key, before, after) = entry.sides();
            crate::diff::placed::keyed(json, key, before, after, consumers);
        }
        json.array_end();
        json.object_end();
        json.object_end();
    }
}
fn consumers(values: &[Consumer], json: &mut skein_json::writer::Encoder) {
    json.array_start();
    for value in values {
        value.json(json);
    }
    json.array_end();
}
