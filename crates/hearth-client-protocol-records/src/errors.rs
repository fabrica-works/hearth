//! Errors in the answer (render.md, 7.1 to 7.3).
use crate::form::{json::field, text::Lines};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Error {
    pub kind: Kind,
    pub class: Class,
    pub at: Option<Place>,
    pub also: Option<Place>,
    pub target: Option<Box<[u8]>>,
    pub task: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Guard(Guard),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Guard {
    pub name: Box<[u8]>,
    pub message: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class {
    Configuration,
    Operational,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Place {
    pub path: Box<[u8]>,
    pub position: Option<Position>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}
impl Error {
    pub(crate) fn text(&self, lines: &mut Lines) {
        if let Some(at) = &self.at {
            at.text(lines);
            lines.put(b": ");
        }
        if let Some(target) = &self.target {
            lines.put(target);
            lines.put(b": ");
        }
        if let Some(task) = &self.task {
            lines.put(task);
            lines.put(b": ");
        }
        match &self.kind {
            Kind::Guard(guard) => {
                lines.put(b"requires ");
                lines.put(&guard.name);
                lines.put(b": ");
                lines.put(&guard.message);
            }
        }
        if let Some(also) = &self.also {
            lines.put(b" (");
            also.text(lines);
            lines.put(b")");
        }
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"record", b"error");
        match &self.kind {
            Kind::Guard(guard) => {
                field(json, b"kind", b"guard");
                json.key(b"guard");
                json.object_start();
                field(json, b"message", &guard.message);
                field(json, b"name", &guard.name);
                json.object_end();
            }
        }
        field(
            json,
            b"class",
            match self.class {
                Class::Configuration => b"configuration",
                Class::Operational => b"operational",
            },
        );
        if let Some(at) = &self.at {
            json.key(b"at");
            at.json(json);
        }
        if let Some(also) = &self.also {
            json.key(b"also");
            also.json(json);
        }
        if let Some(target) = &self.target {
            field(json, b"target", target);
        }
        if let Some(task) = &self.task {
            field(json, b"task", task);
        }
    }
}
impl Place {
    fn text(&self, lines: &mut Lines) {
        lines.put(&self.path);
        if let Some(position) = self.position {
            lines.put(b":");
            lines.decimal(u64::from(position.line));
            lines.put(b":");
            lines.decimal(u64::from(position.column));
        }
    }
    fn json(&self, json: &mut Encoder) {
        json.object_start();
        if let Some(position) = self.position {
            json.key(b"column");
            json.unsigned(u64::from(position.column));
            json.key(b"line");
            json.unsigned(u64::from(position.line));
        }
        field(json, b"path", &self.path);
        json.object_end();
    }
}
