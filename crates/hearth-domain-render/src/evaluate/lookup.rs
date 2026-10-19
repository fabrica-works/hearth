//! Strict lookups, with only marked missing links yielding undefined (protocol/format.md, 7.6).
use super::{
    FaultKind, Missing,
    value::{Entry, Operand, Value},
};
use core::cmp::Ordering;
pub(super) fn absent<'c>(missing: Missing, kind: FaultKind) -> Result<Operand<'c>, FaultKind> {
    match missing {
        Missing::Undefined => Ok(Operand::Undefined),
        Missing::Error => Err(kind),
    }
}
pub(super) fn key<'c>(value: Operand<'c>, name: &[u8], missing: Missing) -> Result<Operand<'c>, FaultKind> {
    match value {
        Operand::Value(Value::Map(entries)) => match find(entries, name) {
            Some(entry) => Ok(match &entry.value {
                Some(v) => Operand::Value(v),
                None => Operand::Undefined,
            }),
            None => absent(missing, FaultKind::NoKey),
        },
        Operand::Loop { index0, length } => match name {
            b"index" => Ok(Operand::Int(i64::from(index0).saturating_add(1))),
            b"index0" => Ok(Operand::Int(i64::from(index0))),
            b"first" => Ok(Operand::Bool(index0 == 0)),
            b"last" => Ok(Operand::Bool(index0.saturating_add(1) == length)),
            b"length" => Ok(Operand::Int(i64::from(length))),
            _name => absent(missing, FaultKind::NoKey),
        },
        Operand::Undefined => absent(missing, FaultKind::Undefined),
        Operand::Value(Value::Bool(_) | Value::Int(_) | Value::String(_) | Value::List(_))
        | Operand::Bool(_)
        | Operand::Int(_)
        | Operand::String(_)
        | Operand::List { .. } => Err(FaultKind::Type),
    }
}
pub(super) fn find<'v>(entries: &'v [Entry], name: &[u8]) -> Option<&'v Entry> {
    let (mut lo, mut hi) = (0_usize, entries.len());
    for _ in 0..=usize::BITS {
        if lo == hi {
            return None;
        }
        let mid = lo.checked_add(hi.checked_sub(lo)?.checked_div(2)?)?;
        let entry = entries.get(mid)?;
        match entry.key.as_ref().cmp(name) {
            Ordering::Equal => return Some(entry),
            Ordering::Less => lo = mid.checked_add(1)?,
            Ordering::Greater => hi = mid,
        }
    }
    None
}
pub(super) fn item<'c>(value: Operand<'c>, index: Operand<'c>, missing: Missing) -> Result<Operand<'c>, FaultKind> {
    if value.undefined() {
        return absent(missing, FaultKind::Undefined);
    }
    match index {
        Operand::String(name) => key(value, &name, missing),
        Operand::Value(Value::String(name)) => key(value, name, missing),
        Operand::Int(i) => index_integer(value, i, missing),
        Operand::Value(Value::Int(i)) => index_integer(value, *i, missing),
        Operand::Undefined => Err(FaultKind::Undefined),
        Operand::Loop { .. }
        | Operand::Value(Value::Bool(_) | Value::List(_) | Value::Map(_))
        | Operand::Bool(_)
        | Operand::List { .. } => Err(FaultKind::Type),
    }
}
fn index_integer(value: Operand<'_>, index: i64, missing: Missing) -> Result<Operand<'_>, FaultKind> {
    match value {
        Operand::Value(Value::List(items)) => {
            if let Ok(index) = usize::try_from(index)
                && let Some(v) = items.get(index)
            {
                return Ok(Operand::Value(v));
            }
            absent(missing, FaultKind::NoIndex)
        }
        Operand::List { items, depth: _ } => {
            if let Ok(index) = usize::try_from(index)
                && let Some(v) = items.into_iter().nth(index)
            {
                return Ok(v);
            }
            absent(missing, FaultKind::NoIndex)
        }
        Operand::Undefined => absent(missing, FaultKind::Undefined),
        Operand::Loop { .. }
        | Operand::Value(Value::Bool(_) | Value::Int(_) | Value::String(_) | Value::Map(_))
        | Operand::Bool(_)
        | Operand::Int(_)
        | Operand::String(_) => Err(FaultKind::Type),
    }
}
