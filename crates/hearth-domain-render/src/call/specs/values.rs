use super::{ELEMENT, bytes};
use crate::evaluate::{Entry, Value};
use skein_lib::Stack;
#[derive(Debug)]
struct Frame {
    value: Value,
    next: usize,
}
pub(super) fn entries(entries: &[Entry], max: u64) -> u64 {
    // An owning cursor copies one value tree once, then moves each child out.
    // It never keeps a borrow in state or copies descendants at each level.
    let mut frames = Stack::with_capacity(hearth_lib::encode::DEPTH.saturating_add(1));
    frames.push(Frame { value: Value::Map(entries.into()), next: 0 }).expect("root frame");
    let mut held = 0_u64;
    for _ in 0..=max.saturating_mul(2).saturating_add(u64::from(hearth_lib::encode::DEPTH)) {
        let Some(frame) = frames.top_mut() else { return held };
        let mut done = false;
        let value = match &mut frame.value {
            Value::List(values) => match values.get_mut(frame.next) {
                Some(value) => Some(core::mem::replace(value, Value::Bool(false))),
                None => {
                    done = true;
                    None
                }
            },
            Value::Map(entries) => match entries.get_mut(frame.next) {
                Some(entry) => {
                    held = held.saturating_add(ELEMENT).saturating_add(bytes(&entry.key));
                    entry.value.take()
                }
                None => {
                    done = true;
                    None
                }
            },
            Value::Bool(_) | Value::Int(_) | Value::String(_) => unreachable!("container cursor"),
        };
        frame.next = frame.next.saturating_add(1);
        if done {
            frames.pop();
        }
        if let Some(value) = value {
            match value {
                Value::Bool(_) | Value::Int(_) => {}
                Value::String(value) => held = held.saturating_add(bytes(&value)),
                Value::List(values) => {
                    held =
                        held.saturating_add(u64::try_from(values.len()).expect("list length").saturating_mul(ELEMENT));
                    frames.push(Frame { value: Value::List(values), next: 0 }).expect("bounded exports nesting");
                }
                Value::Map(entries) => {
                    frames.push(Frame { value: Value::Map(entries), next: 0 }).expect("bounded exports nesting");
                }
            }
        }
        if held > max {
            return held;
        }
    }
    max.saturating_add(1)
}
