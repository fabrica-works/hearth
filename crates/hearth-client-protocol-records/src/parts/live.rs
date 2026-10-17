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
                lines.put(&consumer.task);
                lines.put(b" ");
                lines.put(consumer.unit.as_deref().unwrap_or(b"-"));
                lines.put(b" ");
                lines.put(consumer.action.name());
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
                json.object_start();
                json.key(b"action");
                json.string(consumer.action.name());
                json.key(b"task");
                json.string(&consumer.task);
                if let Some(unit) = &consumer.unit {
                    json.key(b"unit");
                    json.string(unit);
                }
                json.object_end();
            }
            json.array_end();
        }
        json.object_end();
    }
}
