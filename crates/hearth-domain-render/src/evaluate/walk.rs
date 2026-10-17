//! Iterative deep comparison, with absent entries omitted (format.md, 7.4, 7.6).
use super::{Entry, FaultKind, Operand, Value};
use skein_lib::Stack;
#[derive(Clone, Copy, Debug)]
pub(super) enum View<'v> {
    Undefined,
    Bool(bool),
    Int(i64),
    String(&'v [u8]),
    List(Items<'v>),
    Map(&'v [Entry]),
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Items<'v> {
    Values(&'v [Value]),
    Operands(&'v [Operand<'v>]),
}
impl<'v> Items<'v> {
    pub(super) fn len(self) -> usize {
        match self {
            Self::Values(v) => v.len(),
            Self::Operands(v) => v.len(),
        }
    }
    pub(super) fn get(self, index: usize) -> Option<View<'v>> {
        match self {
            Self::Values(v) => Some(context(v.get(index)?)),
            Self::Operands(v) => Some(view(v.get(index)?)),
        }
    }
}
pub(super) fn view<'v>(operand: &'v Operand<'v>) -> View<'v> {
    match operand {
        Operand::Undefined => View::Undefined,
        Operand::Bool(v) => View::Bool(*v),
        Operand::Int(v) => View::Int(*v),
        Operand::String(v) => View::String(v),
        Operand::Value(v) => context(v),
        Operand::List { items, depth: _ } => View::List(Items::Operands(items)),
    }
}
pub(super) fn context(value: &Value) -> View<'_> {
    match value {
        Value::Bool(v) => View::Bool(*v),
        Value::Int(v) => View::Int(*v),
        Value::String(v) => View::String(v),
        Value::List(v) => View::List(Items::Values(v)),
        Value::Map(v) => View::Map(v),
    }
}
#[derive(Debug)]
enum Frame<'v> {
    List { left: Items<'v>, right: Items<'v>, next: usize },
    Map { left: core::slice::Iter<'v, Entry>, right: core::slice::Iter<'v, Entry> },
}
fn present<'v>(entries: &mut core::slice::Iter<'v, Entry>) -> Option<&'v Entry> {
    for entry in entries.by_ref() {
        let Some(_value) = &entry.value else {
            continue;
        };
        return Some(entry);
    }
    None
}
impl<'v> Frame<'v> {
    fn next(&mut self) -> Result<Option<(View<'v>, View<'v>)>, FaultKind> {
        match self {
            Self::List { left, right, next } => {
                let pair = match (left.get(*next), right.get(*next)) {
                    (Some(a), Some(b)) => Some((a, b)),
                    (None, None) => None,
                    (Some(_), None) | (None, Some(_)) => unreachable!("equal lengths"),
                };
                *next = next.saturating_add(1);
                Ok(pair)
            }
            Self::Map { left, right } => match (present(left), present(right)) {
                (Some(a), Some(b)) => {
                    if a.key != b.key {
                        return Err(FaultKind::Type);
                    }
                    Ok(Some((context(a.value.as_ref().expect("present")), context(b.value.as_ref().expect("present")))))
                }
                (None, None) => Ok(None),
                (Some(_), None) | (None, Some(_)) => Err(FaultKind::Type),
            },
        }
    }
}
pub(super) fn equal<'v>(mut left: View<'v>, mut right: View<'v>, capacity: u32, steps: u64) -> Result<bool, FaultKind> {
    let mut frames = Stack::with_capacity(capacity);
    for _ in 0..steps {
        let same = match (left, right) {
            (View::Undefined, _) | (_, View::Undefined) => return Err(FaultKind::Undefined),
            (View::Bool(a), View::Bool(b)) => a == b,
            (View::Int(a), View::Int(b)) => a == b,
            (View::String(a), View::String(b)) => a == b,
            (View::List(a), View::List(b)) => {
                if a.len() != b.len() {
                    return Ok(false);
                }
                if frames.push(Frame::List { left: a, right: b, next: 0 }).is_err() {
                    return Err(FaultKind::Limit);
                }
                true
            }
            (View::Map(a), View::Map(b)) => {
                if frames.push(Frame::Map { left: a.iter(), right: b.iter() }).is_err() {
                    return Err(FaultKind::Limit);
                }
                true
            }
            (View::Bool(_), View::Int(_) | View::String(_) | View::List(_) | View::Map(_))
            | (View::Int(_), View::Bool(_) | View::String(_) | View::List(_) | View::Map(_))
            | (View::String(_), View::Bool(_) | View::Int(_) | View::List(_) | View::Map(_))
            | (View::List(_), View::Bool(_) | View::Int(_) | View::String(_) | View::Map(_))
            | (View::Map(_), View::Bool(_) | View::Int(_) | View::String(_) | View::List(_)) => false,
        };
        if !same {
            return Ok(false);
        }
        for _ in 0..=capacity {
            let Some(frame) = frames.top_mut() else {
                return Ok(true);
            };
            match frame.next() {
                Ok(Some((a, b))) => {
                    left = a;
                    right = b;
                    break;
                }
                Ok(None) => {
                    let _ended = frames.pop().expect("top existed");
                }
                Err(FaultKind::Type) => return Ok(false),
                Err(
                    FaultKind::Unknown
                    | FaultKind::NoKey
                    | FaultKind::NoIndex
                    | FaultKind::Undefined
                    | FaultKind::Overflow
                    | FaultKind::Empty
                    | FaultKind::Path
                    | FaultKind::Output
                    | FaultKind::Limit,
                ) => unreachable!("a frame only signals inequality"),
            }
        }
    }
    Err(FaultKind::Limit)
}
