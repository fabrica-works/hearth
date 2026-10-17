//! Scoped, bounded iteration over borrowed contexts and made lists.
use super::{Binding, Bindings, Budget, Entry, FaultKind, Operand, Value, template::Names};
use skein_lib::bytes;
#[derive(Debug)]
pub(super) enum Source<'c> {
    Values(core::slice::Iter<'c, Value>),
    Keys(core::slice::Iter<'c, Entry>),
    Made(alloc::vec::IntoIter<Operand<'c>>),
}
#[derive(Debug)]
pub(super) struct Frame<'c> {
    pub source: Source<'c>,
    pub names: &'c Names,
    pub length: u32,
    pub index0: u32,
    pub at: u32,
    pub base: u32,
    pub outer: Budget,
    pub iteration: Budget,
}
impl<'c> Frame<'c> {
    pub fn new(
        value: Operand<'c>,
        names: &'c Names,
        at: u32,
        base: u32,
        outer: Budget,
        iteration: Budget,
    ) -> Result<Self, FaultKind> {
        let (source, length) = match value {
            Operand::Value(Value::List(items)) => (Source::Values(items.iter()), items.len()),
            Operand::Value(Value::Map(entries)) => {
                let mut length = 0_usize;
                for entry in entries {
                    if entry.value.is_some() {
                        length = length.saturating_add(1);
                    }
                }
                (Source::Keys(entries.iter()), length)
            }
            Operand::List { items, depth: _ } => {
                let length = items.len();
                (Source::Made(items.into_iter()), length)
            }
            Operand::Undefined => return Err(FaultKind::Undefined),
            Operand::Loop { .. }
            | Operand::Bool(_)
            | Operand::Int(_)
            | Operand::String(_)
            | Operand::Value(Value::Bool(_) | Value::Int(_) | Value::String(_)) => return Err(FaultKind::Type),
        };
        Ok(Self {
            source,
            names,
            length: u32::try_from(length).expect("bounded iterable"),
            index0: 0,
            at,
            base,
            outer,
            iteration,
        })
    }
    pub fn bind(
        &mut self,
        bindings: &mut Bindings<'c>,
        budget: &mut Budget,
        iterations: &mut u32,
    ) -> Result<bool, FaultKind> {
        let next = match &mut self.source {
            Source::Values(items) => {
                let Some(value) = items.next() else { return Ok(false) };
                Some(Operand::Value(value))
            }
            Source::Made(items) => items.next(),
            Source::Keys(entries) => {
                let mut next = None;
                for entry in entries.by_ref() {
                    if entry.value.is_some() {
                        budget.charge(u64::try_from(entry.key.len()).expect("bounded key"))?;
                        next = Some(Operand::String(bytes::copy_of(&entry.key)));
                        break;
                    }
                }
                next
            }
        };
        let Some(value) = next else { return Ok(false) };
        *iterations = iterations.checked_sub(1).ok_or(FaultKind::Limit)?;
        match self.names {
            Names::One { name } => push(bindings, name, value)?,
            Names::Pair { first, second } => {
                let (a, b) = pair(value)?;
                push(bindings, first, a)?;
                push(bindings, second, b)?;
            }
        }
        push(bindings, b"loop", Operand::Loop { index0: self.index0, length: self.length })?;
        self.index0 = self.index0.saturating_add(1);
        Ok(true)
    }
}
pub(super) fn push<'c>(bindings: &mut Bindings<'c>, name: &'c [u8], value: Operand<'c>) -> Result<(), FaultKind> {
    match bindings.push(Binding { name, value }) {
        Ok(()) => Ok(()),
        Err(_binding) => Err(FaultKind::Limit),
    }
}
fn pair(value: Operand<'_>) -> Result<(Operand<'_>, Operand<'_>), FaultKind> {
    match value {
        Operand::Value(Value::List(items)) => {
            let [first, second] = items.as_ref() else { return Err(FaultKind::Type) };
            Ok((Operand::Value(first), Operand::Value(second)))
        }
        Operand::List { items, depth: _ } if items.len() == 2 => {
            let mut items = items.into_iter();
            Ok((items.next().expect("two elements"), items.next().expect("two elements")))
        }
        Operand::Undefined
        | Operand::Loop { .. }
        | Operand::Bool(_)
        | Operand::Int(_)
        | Operand::String(_)
        | Operand::List { .. }
        | Operand::Value(Value::Bool(_) | Value::Int(_) | Value::String(_) | Value::Map(_)) => Err(FaultKind::Type),
    }
}
