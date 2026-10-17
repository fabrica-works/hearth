//! The seventeen filters of format.md, 7.5; all results are charged before allocation.
use super::{
    Budget, ELEMENT, Evaluator, FaultKind, Missing, Operand, Value, Written, escape, lookup,
    walk::{self, View},
};
use skein_lib::{Decimal, List, Writer, bytes};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Filter {
    Default,
    Join,
    Split,
    First,
    Last,
    Length,
    Replace,
    Items,
    Keys,
    Values,
    Sort,
    Tojson,
    ShellQuote,
    SystemdEscape,
    SystemdEscapePath,
    Basename,
    Dirname,
}
impl Filter {
    /// The arguments popped after the input (format.md, 7.5).
    #[must_use]
    pub const fn arity(self) -> u32 {
        match self {
            Self::Default | Self::Join | Self::Split => 1,
            Self::Replace => 2,
            Self::First
            | Self::Last
            | Self::Length
            | Self::Items
            | Self::Keys
            | Self::Values
            | Self::Sort
            | Self::Tojson
            | Self::ShellQuote
            | Self::SystemdEscape
            | Self::SystemdEscapePath
            | Self::Basename
            | Self::Dirname => 0,
        }
    }
}
impl<'c> Evaluator<'c> {
    pub(super) fn filter(&mut self, filter: Filter, budget: &mut Budget) -> Result<Operand<'c>, FaultKind> {
        let second = if filter.arity() == 2 { self.pop() } else { Operand::Undefined };
        let first = if filter.arity() != 0 { self.pop() } else { Operand::Undefined };
        let input = self.pop();
        if filter == Filter::Default && input.undefined() {
            return Ok(first);
        }
        if input.undefined()
            || (filter.arity() != 0 && first.undefined())
            || (filter.arity() == 2 && second.undefined())
        {
            return Err(FaultKind::Undefined);
        }
        match filter {
            Filter::Default => Ok(input),
            Filter::Length => {
                let length = match walk::view(&input) {
                    View::String(text) => {
                        let mut length = 0_u64;
                        for byte in text {
                            if byte & 0xc0 != 0x80 {
                                length = length.saturating_add(1);
                            }
                        }
                        length
                    }
                    View::List(items) => u64::try_from(items.len()).expect("bounded list"),
                    View::Map(entries) => {
                        let mut length = 0_u64;
                        for entry in entries {
                            if entry.value.is_some() {
                                length = length.saturating_add(1);
                            }
                        }
                        length
                    }
                    View::Undefined => return Err(FaultKind::Undefined),
                    View::Bool(_) | View::Int(_) => return Err(FaultKind::Type),
                };
                Ok(Operand::Int(match i64::try_from(length) {
                    Ok(length) => length,
                    Err(_) => return Err(FaultKind::Limit),
                }))
            }
            Filter::First | Filter::Last => {
                let View::List(items) = walk::view(&input) else {
                    return Err(FaultKind::Type);
                };
                if items.len() == 0 {
                    return Err(FaultKind::Empty);
                }
                let index = if filter == Filter::First { 0 } else { items.len().saturating_sub(1) };
                lookup::item(input, Operand::Int(i64::try_from(index).expect("bounded list")), Missing::Error)
            }
            Filter::Join => join(&input, text(&first)?, budget),
            Filter::Split => split(text(&input)?, text(&first)?, budget, self.nesting),
            Filter::Replace => replace(text(&input)?, text(&first)?, text(&second)?, budget),
            Filter::Items | Filter::Keys | Filter::Values => entries(&input, filter, budget, self.nesting),
            Filter::Sort => sort(input, budget, self.nesting),
            Filter::Tojson => escape::json(&input, budget, self.walk_depth, self.walk_steps),
            Filter::ShellQuote
            | Filter::SystemdEscape
            | Filter::SystemdEscapePath
            | Filter::Basename
            | Filter::Dirname => escape::text(filter, text(&input)?, budget),
        }
    }
}
fn text<'v>(value: &'v Operand<'_>) -> Result<&'v [u8], FaultKind> {
    match walk::view(value) {
        View::String(text) => Ok(text),
        View::Undefined => Err(FaultKind::Undefined),
        View::Bool(_) | View::Int(_) | View::List(_) | View::Map(_) => Err(FaultKind::Type),
    }
}
fn scalar(value: View<'_>) -> Result<Written<'_>, FaultKind> {
    match value {
        View::String(v) => Ok(Written::Bytes(v)),
        View::Int(v) => Ok(Written::Integer { negative: v < 0, digits: Decimal::of(v.unsigned_abs()) }),
        View::Undefined => Err(FaultKind::Undefined),
        View::Bool(_) | View::List(_) | View::Map(_) => Err(FaultKind::Type),
    }
}
fn join<'c>(input: &Operand<'c>, separator: &[u8], budget: &mut Budget) -> Result<Operand<'c>, FaultKind> {
    let View::List(items) = walk::view(input) else {
        return Err(FaultKind::Type);
    };
    let mut length = separator.len().checked_mul(items.len().saturating_sub(1)).ok_or(FaultKind::Limit)?;
    for index in 0..items.len() {
        length = length.checked_add(scalar(items.get(index).expect("within length"))?.len()).ok_or(FaultKind::Limit)?;
    }
    budget.charge(u64::try_from(length).expect("bytes fit"))?;
    let mut writer = Writer::new(length);
    for index in 0..items.len() {
        if index != 0 {
            writer.put(separator).expect("measured");
        }
        scalar(items.get(index).expect("within length"))?.put(&mut writer);
    }
    Ok(Operand::String(writer.finish()))
}
fn split<'c>(text: &[u8], separator: &[u8], budget: &mut Budget, nesting: u32) -> Result<Operand<'c>, FaultKind> {
    if separator.is_empty() {
        return Err(FaultKind::Empty);
    }
    let count = bytes::count(text, separator, u32::MAX).checked_add(1).ok_or(FaultKind::Limit)?;
    let bytes = text
        .len()
        .saturating_sub(usize::try_from(count.saturating_sub(1)).expect("count fits").saturating_mul(separator.len()));
    budget.charge(u64::from(count).saturating_mul(ELEMENT).saturating_add(u64::try_from(bytes).expect("bytes fit")))?;
    if nesting == 0 {
        return Err(FaultKind::Limit);
    }
    let mut items = List::with_capacity(count);
    let mut from = 0;
    for _ in 0..count {
        let end = bytes::find_from(text, separator, from).unwrap_or(text.len());
        items.push(Operand::String(bytes::copy_of(text.get(from..end).expect("match bounds")))).expect("counted");
        from = end.saturating_add(separator.len());
    }
    Ok(Operand::List { depth: 1, items: items.into_boxed() })
}
fn replace<'c>(text: &[u8], pattern: &[u8], replacement: &[u8], budget: &mut Budget) -> Result<Operand<'c>, FaultKind> {
    if pattern.is_empty() {
        return Err(FaultKind::Empty);
    }
    let count = usize::try_from(bytes::count(text, pattern, u32::MAX)).expect("count fits");
    let length = text
        .len()
        .checked_sub(count.saturating_mul(pattern.len()))
        .ok_or(FaultKind::Limit)?
        .checked_add(count.checked_mul(replacement.len()).ok_or(FaultKind::Limit)?)
        .ok_or(FaultKind::Limit)?;
    budget.charge(u64::try_from(length).expect("bytes fit"))?;
    let mut writer = Writer::new(length);
    let mut from = 0;
    for _ in 0..count {
        let end = bytes::find_from(text, pattern, from).expect("counted");
        writer.put(text.get(from..end).expect("match bounds")).expect("measured");
        writer.put(replacement).expect("measured");
        from = end.saturating_add(pattern.len());
    }
    writer.put(text.get(from..).expect("match bounds")).expect("measured");
    Ok(Operand::String(writer.finish()))
}
fn entries<'c>(
    input: &Operand<'c>,
    filter: Filter,
    budget: &mut Budget,
    nesting: u32,
) -> Result<Operand<'c>, FaultKind> {
    let Operand::Value(Value::Map(entries)) = input else {
        return Err(FaultKind::Type);
    };
    let mut count = 0_u32;
    let mut charge = 0_u64;
    for entry in entries {
        if entry.value.is_none() {
            continue;
        }
        count = count.saturating_add(1);
        charge = charge.saturating_add(ELEMENT);
        if filter != Filter::Values {
            charge = charge.saturating_add(u64::try_from(entry.key.len()).expect("bytes fit"));
        }
        if filter == Filter::Items {
            charge = charge.saturating_add(2 * ELEMENT);
        }
    }
    let depth = if filter == Filter::Items && count != 0 { 2 } else { 1 };
    if depth > nesting {
        return Err(FaultKind::Limit);
    }
    budget.charge(charge)?;
    let mut items = List::with_capacity(count);
    for entry in entries {
        let Some(value) = &entry.value else {
            continue;
        };
        let item = match filter {
            Filter::Keys => Operand::String(bytes::copy_of(&entry.key)),
            Filter::Values => Operand::Value(value),
            Filter::Items => {
                let mut pair = List::with_capacity(2);
                pair.push(Operand::String(bytes::copy_of(&entry.key))).expect("two reserved");
                pair.push(Operand::Value(value)).expect("two reserved");
                Operand::List { depth: 1, items: pair.into_boxed() }
            }
            Filter::Default
            | Filter::Join
            | Filter::Split
            | Filter::First
            | Filter::Last
            | Filter::Length
            | Filter::Replace
            | Filter::Sort
            | Filter::Tojson
            | Filter::ShellQuote
            | Filter::SystemdEscape
            | Filter::SystemdEscapePath
            | Filter::Basename
            | Filter::Dirname => unreachable!("map filter"),
        };
        items.push(item).expect("counted");
    }
    Ok(Operand::List { depth, items: items.into_boxed() })
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Key<'v> {
    Int(i64),
    Text(&'v [u8]),
}
fn sort<'c>(mut input: Operand<'c>, budget: &mut Budget, nesting: u32) -> Result<Operand<'c>, FaultKind> {
    let View::List(items) = walk::view(&input) else {
        return Err(FaultKind::Type);
    };
    let count = u32::try_from(items.len()).expect("bounded list");
    if nesting == 0 {
        return Err(FaultKind::Limit);
    }
    // The key/index table and the output coexist; both are priced before either is allocated.
    budget.charge(u64::from(count).saturating_mul(2 * ELEMENT))?;
    let mut keys = List::with_capacity(count);
    let mut previous = None;
    for index in 0..items.len() {
        let key = match items.get(index).expect("within length") {
            View::Int(v) => Key::Int(v),
            View::String(v) => Key::Text(v),
            View::Undefined => return Err(FaultKind::Undefined),
            View::Bool(_) | View::List(_) | View::Map(_) => return Err(FaultKind::Type),
        };
        let kind = match key {
            Key::Int(_) => false,
            Key::Text(_) => true,
        };
        if previous.is_some() && previous != Some(kind) {
            return Err(FaultKind::Type);
        }
        previous = Some(kind);
        keys.push((key, index)).expect("counted");
    }
    let mut keys = keys.into_boxed();
    keys.sort_unstable();
    // Retain indices only, so the keys' borrows end before moving the operands.
    let mut indices = List::with_capacity(count);
    for (_, index) in &keys {
        indices.push(*index).expect("counted");
    }
    drop(keys);
    let mut output = List::with_capacity(count);
    for index in indices.into_boxed() {
        let value = match &mut input {
            Operand::Value(Value::List(items)) => Operand::Value(items.get(index).expect("within length")),
            Operand::List { items, depth: _ } => {
                core::mem::replace(items.get_mut(index).expect("within length"), Operand::Undefined)
            }
            Operand::Undefined
            | Operand::Bool(_)
            | Operand::Int(_)
            | Operand::String(_)
            | Operand::Value(Value::Bool(_) | Value::Int(_) | Value::String(_) | Value::Map(_)) => unreachable!("list"),
        };
        output.push(value).expect("counted");
    }
    Ok(Operand::List { depth: 1, items: output.into_boxed() })
}
