//! Typed public values and their compact JSON forms (format.md, 8.1; render.md, 4.4).
use crate::form::text::Lines;
use alloc::boxed::Box;
use skein_json::writer::{Encoder, Limits};
use skein_lib::Stack;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Exports {
    pub tasks: Box<[TaskExports]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TaskExports {
    pub task: Box<[u8]>,
    pub values: Box<[Named]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Named {
    pub name: Box<[u8]>,
    pub value: Value,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Value {
    Bool(bool),
    Int(i64),
    String(Box<[u8]>),
    List(Box<[Value]>),
    Map(Box<[(Box<[u8]>, Value)]>),
}
impl Exports {
    pub(crate) fn json(&self, json: &mut Encoder) {
        json.object_start();
        for task in &self.tasks {
            json.key(&task.task);
            json.object_start();
            for named in &task.values {
                json.key(&named.name);
                named.value.json(json);
            }
            json.object_end();
        }
        json.object_end();
    }
    pub(crate) fn text(&self, lines: &mut Lines) {
        let mut width = 0_usize;
        for task in &self.tasks {
            for named in &task.values {
                width = width.max(task.task.len().saturating_add(named.name.len()).saturating_add(1));
            }
        }
        for task in &self.tasks {
            for named in &task.values {
                lines.put(b"  ");
                lines.put(&task.task);
                lines.put(b".");
                lines.put(&named.name);
                lines.pad_to(width.saturating_add(5));
                lines.put(&named.value.compact());
                lines.put(b"\n");
            }
        }
    }
}
#[derive(Debug)]
enum Frame<'a> {
    List(core::slice::Iter<'a, Value>),
    Map(core::slice::Iter<'a, (Box<[u8]>, Value)>),
}
impl Value {
    pub(crate) fn json(&self, json: &mut Encoder) {
        let mut stack = Stack::with_capacity(crate::form::json::DEPTH);
        let mut next = Some(self);
        for _ in 0..u64::from(crate::limits::SHIPPED.record).saturating_mul(3).saturating_add(3) {
            if let Some(value) = next.take() {
                match value {
                    Self::Bool(value) => json.boolean(*value),
                    Self::Int(value) => json.signed(*value),
                    Self::String(value) => json.string(value),
                    Self::List(items) => {
                        json.array_start();
                        stack.push(Frame::List(items.iter())).expect("export depth");
                    }
                    Self::Map(entries) => {
                        json.object_start();
                        stack.push(Frame::Map(entries.iter())).expect("export depth");
                    }
                }
            }
            let Some(mut frame) = stack.pop() else { return };
            next = match &mut frame {
                Frame::List(each) => {
                    let next = each.next();
                    if next.is_none() {
                        json.array_end();
                    }
                    next
                }
                Frame::Map(each) => {
                    if let Some((name, value)) = each.next() {
                        json.key(name);
                        Some(value)
                    } else {
                        json.object_end();
                        None
                    }
                }
            };
            if next.is_some() {
                stack.push(frame).expect("popped frame fits");
            }
        }
        unreachable!("bounded records bound the value walk")
    }
    pub(crate) fn compact(&self) -> Box<[u8]> {
        let limits = Limits { depth: crate::form::json::DEPTH, length: u32::MAX };
        let mut json = Encoder::measure(&limits);
        self.json(&mut json);
        let length = json.measured().expect("typed export JSON");
        let mut json = Encoder::write(length, &limits);
        self.json(&mut json);
        json.finish()
    }
}
