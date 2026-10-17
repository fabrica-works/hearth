//! Operators and scalar output (format.md, 7.4).
use super::{
    Budget, FaultKind, Op, Operand,
    walk::{self, View},
};
use core::cmp::Ordering;
use skein_lib::{Decimal, Writer, bytes};
#[must_use]
pub fn truthy(operand: &Operand<'_>) -> bool {
    match walk::view(operand) {
        View::Undefined => false,
        View::Bool(v) => v,
        View::Int(v) => v != 0,
        View::String(v) => !v.is_empty(),
        View::List(v) => v.len() != 0,
        View::Map(entries) => {
            for entry in entries {
                if entry.value.is_some() {
                    return true;
                }
            }
            false
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Written<'v> {
    Bytes(&'v [u8]),
    Integer { negative: bool, digits: Decimal },
}
pub fn output<'v>(operand: &'v Operand<'_>) -> Result<Written<'v>, FaultKind> {
    match walk::view(operand) {
        View::Undefined => Err(FaultKind::Undefined),
        View::Bool(v) => Ok(Written::Bytes(if v { b"true" } else { b"false" })),
        View::String(v) => Ok(Written::Bytes(v)),
        View::Int(v) => Ok(Written::Integer { negative: v < 0, digits: Decimal::of(v.unsigned_abs()) }),
        View::List(_) | View::Map(_) => Err(FaultKind::Output),
    }
}
impl Written<'_> {
    pub(super) fn len(self) -> usize {
        match self {
            Self::Bytes(v) => v.len(),
            Self::Integer { negative, digits } => digits.as_bytes().len().saturating_add(usize::from(negative)),
        }
    }
    pub(super) fn put(self, writer: &mut Writer) {
        match self {
            Self::Bytes(v) => writer.put(v).expect("measured"),
            Self::Integer { negative, digits } => {
                if negative {
                    writer.put(b"-").expect("measured");
                }
                writer.put(digits.as_bytes()).expect("measured");
            }
        }
    }
}
pub(super) fn binary<'c>(
    op: &Op,
    left: Operand<'c>,
    right: Operand<'c>,
    budget: &mut Budget,
    capacity: u32,
    steps: u64,
) -> Result<Operand<'c>, FaultKind> {
    let (a, b) = (walk::view(&left), walk::view(&right));
    if left.undefined() || right.undefined() {
        return Err(FaultKind::Undefined);
    }
    Ok(match op {
        Op::Equal => Operand::Bool(walk::equal(a, b, capacity, steps)?),
        Op::NotEqual => Operand::Bool(!walk::equal(a, b, capacity, steps)?),
        Op::Less => Operand::Bool(order(a, b)?.is_lt()),
        Op::LessOrEqual => Operand::Bool(!order(a, b)?.is_gt()),
        Op::Greater => Operand::Bool(order(a, b)?.is_gt()),
        Op::GreaterOrEqual => Operand::Bool(!order(a, b)?.is_lt()),
        Op::In => Operand::Bool(contains(a, b, capacity, steps)?),
        Op::NotIn => Operand::Bool(!contains(a, b, capacity, steps)?),
        Op::Add | Op::Subtract => {
            let (View::Int(a), View::Int(b)) = (a, b) else {
                return Err(FaultKind::Type);
            };
            Operand::Int((if *op == Op::Add { a.checked_add(b) } else { a.checked_sub(b) }).ok_or(FaultKind::Overflow)?)
        }
        Op::Concat => {
            let a = joined(&left)?;
            let b = joined(&right)?;
            let length = a.len().checked_add(b.len()).ok_or(FaultKind::Limit)?;
            budget.charge(u64::try_from(length).expect("bytes fit"))?;
            let mut writer = Writer::new(length);
            a.put(&mut writer);
            b.put(&mut writer);
            Operand::String(writer.finish())
        }
        Op::String(_)
        | Op::Integer(_)
        | Op::Boolean(_)
        | Op::Name { .. }
        | Op::Key { .. }
        | Op::Item { .. }
        | Op::List { .. }
        | Op::Defined
        | Op::NotDefined
        | Op::Not
        | Op::JumpIfFalseKeeping { .. }
        | Op::JumpIfTrueKeeping { .. }
        | Op::JumpIfFalse { .. }
        | Op::Jump { .. } => unreachable!("binary operator"),
    })
}
fn joined<'v>(operand: &'v Operand<'_>) -> Result<Written<'v>, FaultKind> {
    match walk::view(operand) {
        View::Int(_) | View::String(_) => output(operand),
        View::Undefined => Err(FaultKind::Undefined),
        View::Bool(_) | View::List(_) | View::Map(_) => Err(FaultKind::Type),
    }
}
fn order(a: View<'_>, b: View<'_>) -> Result<Ordering, FaultKind> {
    match (a, b) {
        (View::Int(a), View::Int(b)) => Ok(a.cmp(&b)),
        (View::String(a), View::String(b)) => Ok(a.cmp(b)),
        (View::Undefined | View::Bool(_) | View::List(_) | View::Map(_), _)
        | (View::Int(_), View::Undefined | View::Bool(_) | View::String(_) | View::List(_) | View::Map(_))
        | (View::String(_), View::Undefined | View::Bool(_) | View::Int(_) | View::List(_) | View::Map(_)) => {
            Err(FaultKind::Type)
        }
    }
}
fn contains(needle: View<'_>, haystack: View<'_>, capacity: u32, steps: u64) -> Result<bool, FaultKind> {
    match haystack {
        View::String(text) => {
            let View::String(needle) = needle else {
                return Err(FaultKind::Type);
            };
            Ok(bytes::find_from(text, needle, 0).is_some())
        }
        View::Map(entries) => {
            let View::String(needle) = needle else {
                return Err(FaultKind::Type);
            };
            Ok(match super::lookup::find(entries, needle) {
                Some(entry) => entry.value.is_some(),
                None => false,
            })
        }
        View::List(items) => {
            for index in 0..items.len() {
                if walk::equal(needle, items.get(index).expect("within length"), capacity, steps)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        View::Undefined => Err(FaultKind::Undefined),
        View::Bool(_) | View::Int(_) => Err(FaultKind::Type),
    }
}
