//! Public evaluated values become typed client values (protocol/format.md, 8).
use crate::{
    records::parts::exports as target,
    render::{evaluate as source, parts::exports::Exports},
};
use alloc::boxed::Box;
use skein_lib::{List, Stack};
#[must_use]
pub fn exports(exports: Exports) -> target::Exports {
    let mut tasks = List::with_capacity(u32::try_from(exports.tasks.len()).expect("task count"));
    for task in exports.tasks {
        let mut values = List::with_capacity(u32::try_from(task.values.len()).expect("export count"));
        for entry in task.values {
            if let Some(value) = entry.value {
                values.push(target::Named { name: entry.key, value: self::value(value) }).expect("export count");
            }
        }
        tasks.push(target::TaskExports { task: task.task, values: values.into_boxed() }).expect("task count");
    }
    target::Exports { tasks: tasks.into_boxed() }
}
#[derive(Debug)]
enum Frame<'a> {
    List { each: core::slice::Iter<'a, source::Value>, made: List<target::Value> },
    Map { each: core::slice::Iter<'a, source::Entry>, made: List<(Box<[u8]>, target::Value)>, name: Box<[u8]> },
}
#[must_use]
pub fn value(value: source::Value) -> target::Value {
    let mut stack = Stack::with_capacity(crate::render::limits::SHIPPED.nesting);
    let mut next = &value;
    for _ in 0..crate::render::limits::SHIPPED.expanded.saturating_mul(3).saturating_add(3) {
        let mut value = match next {
            source::Value::Bool(value) => target::Value::Bool(*value),
            source::Value::Int(value) => target::Value::Int(*value),
            source::Value::String(value) => target::Value::String(value.clone()),
            source::Value::List(items) => {
                let made = List::with_capacity(u32::try_from(items.len()).expect("list count"));
                let mut each = items.iter();
                if let Some(first) = each.next() {
                    stack.push(Frame::List { each, made }).expect("export depth");
                    next = first;
                    continue;
                }
                target::Value::List(made.into_boxed())
            }
            source::Value::Map(entries) => {
                let made = List::with_capacity(u32::try_from(entries.len()).expect("map count"));
                let mut each = entries.iter();
                if let Some((name, first)) = present(&mut each) {
                    stack.push(Frame::Map { each, made, name }).expect("export depth");
                    next = first;
                    continue;
                }
                target::Value::Map(made.into_boxed())
            }
        };
        for _ in 0..=crate::render::limits::SHIPPED.nesting {
            let Some(frame) = stack.pop() else { return value };
            match frame {
                Frame::List { mut each, mut made } => {
                    made.push(value).expect("list count");
                    if let Some(first) = each.next() {
                        stack.push(Frame::List { each, made }).expect("popped frame fits");
                        next = first;
                        break;
                    }
                    value = target::Value::List(made.into_boxed());
                }
                Frame::Map { mut each, mut made, name } => {
                    made.push((name, value)).expect("map count");
                    if let Some((name, first)) = present(&mut each) {
                        stack.push(Frame::Map { each, made, name }).expect("popped frame fits");
                        next = first;
                        break;
                    }
                    value = target::Value::Map(made.into_boxed());
                }
            }
        }
    }
    unreachable!("bounded exports bound translation")
}
fn present<'a>(each: &mut core::slice::Iter<'a, source::Entry>) -> Option<(Box<[u8]>, &'a source::Value)> {
    for entry in each.by_ref() {
        if let Some(value) = &entry.value {
            return Some((entry.key.clone(), value));
        }
    }
    None
}

#[must_use]
pub fn change(change: crate::render::parts::exports::Change) -> crate::records::diff::ExportsChange {
    let mut entries = List::with_capacity(u32::try_from(change.entries.len()).expect("export count"));
    for entry in change.entries {
        entries
            .push(crate::records::diff::ExportChange {
                task: entry.task,
                name: entry.name,
                before: optional_value(entry.before),
                after: optional_value(entry.after),
            })
            .expect("export count");
    }
    crate::records::diff::ExportsChange { entries: entries.into_boxed() }
}

fn optional_value(value: Option<source::Value>) -> Option<target::Value> {
    Some(self::value(value?))
}
