//! Canonical exports, with integers as decimal text (domain/render.md, 4.4, 4.6).
use crate::{
    evaluate::{Entry, Value},
    exports::Exported,
};
use alloc::boxed::Box;
use hearth_lib::{
    encode::{DEPTH, Digest, Item},
    hash::H,
};
use skein_lib::{List, Map, Stack, bytes};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Exports {
    pub tasks: Box<[Exported]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub entries: Box<[ExportChange]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ExportChange {
    pub task: Box<[u8]>,
    pub name: Box<[u8]>,
    pub before: Option<Value>,
    pub after: Option<Value>,
}
/// Pair published values by task and name, comparing their canonical records.
#[must_use]
pub fn change(before: Option<&Exports>, after: Option<&Exports>) -> Change {
    let mut count = 0_usize;
    for part in before.into_iter().chain(after) {
        for task in &part.tasks {
            count = count.saturating_add(task.values.len());
        }
    }
    let mut pairs = Map::with_capacity(u32::try_from(count).expect("bounded exports"));
    for (part, from) in [(before, true), (after, false)] {
        let Some(part) = part else { continue };
        for task in &part.tasks {
            for entry in &task.values {
                if let Some(value) = &entry.value {
                    let key = (task.task.as_ref(), entry.key.as_ref());
                    let (before, after) = pairs.get(&key).copied().unwrap_or((None, None));
                    let pair = if from { (Some(value), after) } else { (before, Some(value)) };
                    pairs.insert(key, pair).expect("export count");
                }
            }
        }
    }
    let mut entries = List::with_capacity(pairs.len());
    for ((task, name), (before, after)) in &pairs {
        let same = match (before, after) {
            (Some(before), Some(after)) => value_items(before) == value_items(after),
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        };
        if !same {
            entries
                .push(ExportChange {
                    task: bytes::copy_of(task),
                    name: bytes::copy_of(name),
                    before: before.cloned(),
                    after: after.cloned(),
                })
                .expect("export count");
        }
    }
    Change { entries: entries.into_boxed() }
}
#[must_use]
pub fn build(exported: Box<[Exported]>) -> Exports {
    let mut tasks = Map::with_capacity(u32::try_from(exported.len()).expect("task count"));
    for task in exported {
        if !task.values.is_empty() {
            tasks.insert(task.task.clone(), task).expect("task count");
        }
    }
    let mut ordered = List::with_capacity(tasks.len());
    for (_, task) in &tasks {
        ordered.push(task.clone()).expect("task count");
    }
    Exports { tasks: ordered.into_boxed() }
}
#[must_use]
pub fn items(exports: &Exports) -> Box<[Item]> {
    let mut record = List::with_capacity(1);
    push(&mut record, Item::Map(u64::try_from(exports.tasks.len()).expect("task count")));
    for task in &exports.tasks {
        push(&mut record, Item::Bytes(task.task.clone()));
        let value = Value::Map(task.values.clone());
        for item in value_items(&value) {
            push(&mut record, item);
        }
    }
    record.into_boxed()
}
#[must_use]
pub fn digest(exports: &Exports) -> Digest {
    let mut hash = H::new(b"part:exports", 1);
    for item in items(exports) {
        hash.values().item(&item);
    }
    hash.finish()
}
#[derive(Debug)]
struct Named<'a> {
    name: &'a [u8],
    value: &'a Value,
}
#[derive(Debug)]
enum Frame<'a> {
    List(core::slice::Iter<'a, Value>),
    Map { entries: Box<[Named<'a>]>, next: usize },
}
fn ordered(entries: &[Entry]) -> Box<[Named<'_>]> {
    let mut ordered = Map::with_capacity(u32::try_from(entries.len()).expect("entry count"));
    for entry in entries {
        if let Some(value) = &entry.value {
            ordered.insert(entry.key.as_ref(), value).expect("entry count");
        }
    }
    let mut names = List::with_capacity(ordered.len());
    for (name, value) in &ordered {
        names.push(Named { name, value }).expect("entry count");
    }
    names.into_boxed()
}
/// The one iterative value record walk, also used by equality (domain/render.md, 4.6).
#[must_use]
pub fn value_items(value: &Value) -> Box<[Item]> {
    let mut record = List::with_capacity(1);
    let mut stack = Stack::with_capacity(DEPTH);
    let mut next = Some(value);
    for _ in 0..crate::limits::SHIPPED.expanded.saturating_mul(3).saturating_add(3) {
        if let Some(value) = next.take() {
            let item = match value {
                Value::Bool(false) => Item::False,
                Value::Bool(true) => Item::True,
                Value::Int(value) => Item::Bytes(signed(*value)),
                Value::String(value) => Item::Bytes(value.clone()),
                Value::List(values) => {
                    stack.push(Frame::List(values.iter())).expect("conforming depth");
                    Item::List(u64::try_from(values.len()).expect("list count"))
                }
                Value::Map(entries) => {
                    let entries = ordered(entries);
                    let count = u64::try_from(entries.len()).expect("map count");
                    stack.push(Frame::Map { entries, next: 0 }).expect("conforming depth");
                    Item::Map(count)
                }
            };
            push(&mut record, item);
        }
        let Some(mut frame) = stack.pop() else { return record.into_boxed() };
        next = match &mut frame {
            Frame::List(each) => each.next(),
            Frame::Map { entries, next } => {
                if let Some(entry) = entries.get(*next) {
                    *next = next.saturating_add(1);
                    push(&mut record, Item::Bytes(bytes::copy_of(entry.name)));
                    Some(entry.value)
                } else {
                    None
                }
            }
        };
        if next.is_some() {
            stack.push(frame).expect("popped frame fits");
        }
    }
    unreachable!("expanded exports bound the record walk")
}
fn signed(value: i64) -> Box<[u8]> {
    let digits = skein_lib::Decimal::of(value.unsigned_abs());
    let mut writer = skein_lib::Writer::new(digits.as_bytes().len().saturating_add(usize::from(value < 0)));
    if value < 0 {
        writer.put(b"-").expect("signed room");
    }
    writer.put(digits.as_bytes()).expect("decimal room");
    writer.finish()
}
fn push(record: &mut List<Item>, item: Item) {
    if record.room() == 0 {
        let mut larger = List::with_capacity(record.capacity().saturating_mul(2));
        for item in &*record {
            larger.push(item.clone()).expect("doubled capacity");
        }
        *record = larger;
    }
    record.push(item).expect("reserved room");
}
