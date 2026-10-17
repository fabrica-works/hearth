//! Conditions before concrete entries (format.md, 5.9; render.md, 3.5).
use crate::{
    errors::{Guard, Kind},
    evaluate::{self, Bindings, Budget, Context, Evaluator},
    limits::Limits,
    revision::{Position, declaration::Declaration},
};
use alloc::boxed::Box;
use skein_lib::List;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Fault {
    pub kind: Kind,
    pub at: Position,
    pub also: Option<Position>,
}
/// Every false or unevaluable guard, in name order.
#[must_use]
pub fn guards(declaration: &Declaration, context: &Context<'_>, limits: &Limits) -> Box<[Fault]> {
    let mut faults = List::with_capacity(u32::try_from(declaration.requires.len()).expect("bounded guards"));
    let mut evaluator = Evaluator::new(limits);
    let bindings = Bindings::with_capacity(0);
    for guard in &declaration.requires {
        let fault = match evaluate::expression(
            &mut evaluator,
            &guard.when.expression,
            context,
            &bindings,
            &mut Budget::new(limits),
        ) {
            Ok(value) => {
                if evaluate::truthy(&value) {
                    continue;
                }
                Fault {
                    kind: Kind::Guard(Guard { name: guard.name.text.clone(), message: guard.message.text.clone() }),
                    at: guard.name.at,
                    also: None,
                }
            }
            Err(fault) => Fault { kind: Kind::Evaluation(fault.kind), at: guard.when.at, also: None },
        };
        faults.push(fault).expect("guard count");
    }
    faults.into_boxed()
}

use crate::{
    errors::Collision,
    evaluate::{Binding, FaultKind, Operand, Value, Written, template::Does},
    revision::declaration::{Condition, Entry, Interpolated, Map},
};
use skein_lib::{Map as Keys, Writer, bytes};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Expanded {
    pub task: Box<[u8]>,
    pub groups: Box<[Group]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Group {
    pub map: Map,
    pub entries: Box<[Concrete]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Concrete {
    pub key: Box<[u8]>,
    pub at: Position,
    pub element: Option<u32>,
}
fn failure(kind: FaultKind, at: Position) -> Fault {
    Fault { kind: Kind::Evaluation(kind), at, also: None }
}
fn condition<'c>(
    when: &Condition,
    context: &Context<'c>,
    bindings: &Bindings<'c>,
    limits: &Limits,
) -> Result<Operand<'c>, Fault> {
    located(
        evaluate::expression(
            &mut Evaluator::new(limits),
            &when.expression,
            context,
            bindings,
            &mut Budget::new(limits),
        ),
        when.at,
    )
}
/// Expand keys once per selected element, sharing collision and count bounds across maps.
pub fn expand(
    declaration: &Declaration,
    context: &Context<'_>,
    limits: &Limits,
    left: &mut u64,
) -> Result<Expanded, Fault> {
    let mut keys = Keys::<Box<[u8]>, Position>::with_capacity(limits.expansion);
    let mut groups = List::with_capacity(u32::try_from(declaration.entries.len()).expect("map count"));
    for map in &declaration.entries {
        let mut ordered = Keys::<Box<[u8]>, Concrete>::with_capacity(limits.expansion);
        for entry in &map.entries {
            let mut source = match &entry.for_each {
                Some(when) => Source::new(condition(when, context, &Bindings::with_capacity(0), limits)?, when.at)?,
                None => Source::Once(true),
            };
            for index in 0..source.len() {
                let Some(pair) = source.next() else { break };
                let mut bindings = Bindings::with_capacity(2);
                let element = match pair {
                    Selected::Pair(key, value) => {
                        bindings.push(Binding { name: b"key", value: key }).expect("two bindings");
                        bindings.push(Binding { name: b"value", value }).expect("two bindings");
                        Some(index)
                    }
                    Selected::Plain => None,
                };
                if let Some(when) = &entry.when
                    && !evaluate::truthy(&condition(when, context, &bindings, limits)?)
                {
                    continue;
                }
                let at = entry.key.at();
                if keys.len() == limits.expansion {
                    let at = match &entry.for_each {
                        Some(when) => when.at,
                        None => at,
                    };
                    return Err(failure(FaultKind::Limit, at));
                }
                let key = interpolate(&entry.key, context, &bindings, limits, left)?;
                if let Some(first) = keys.get(&key) {
                    return Err(Fault { kind: Kind::Collision(Collision::Key { key }), at, also: Some(*first) });
                }
                keys.insert(key.clone(), at).expect("entry limit checked");
                ordered.insert(key.clone(), Concrete { key, at, element }).expect("entry limit checked");
            }
        }
        let mut entries = List::with_capacity(ordered.len());
        for _ in 0..ordered.len() {
            let (key, _) = ordered.first().expect("counted entries");
            let key = key.clone();
            entries.push(ordered.remove(&key).expect("first existed")).expect("entry count");
        }
        groups.push(Group { map: map.map, entries: entries.into_boxed() }).expect("map count");
    }
    Ok(Expanded { task: Box::new([]), groups: groups.into_boxed() })
}
fn located<T>(value: Result<T, evaluate::Fault>, at: Position) -> Result<T, Fault> {
    match value {
        Ok(value) => Ok(value),
        Err(fault) => Err(failure(fault.kind, at)),
    }
}
#[derive(Debug)]
enum Selected<'c> {
    Plain,
    Pair(Operand<'c>, Operand<'c>),
}
#[derive(Debug)]
enum Source<'c> {
    Values(core::iter::Enumerate<core::slice::Iter<'c, Value>>),
    Keys(core::slice::Iter<'c, evaluate::Entry>),
    Made(core::iter::Enumerate<alloc::vec::IntoIter<Operand<'c>>>),
    Once(bool),
}
impl<'c> Source<'c> {
    fn len(&self) -> u32 {
        let length = match self {
            Self::Values(items) => items.len(),
            Self::Keys(entries) => entries.len(),
            Self::Made(items) => items.len(),
            Self::Once(pending) => usize::from(*pending),
        };
        u32::try_from(length).expect("bounded iterable")
    }
    fn new(value: Operand<'c>, at: Position) -> Result<Self, Fault> {
        match value {
            Operand::Value(Value::List(items)) => Ok(Self::Values(items.iter().enumerate())),
            Operand::Value(Value::Map(entries)) => Ok(Self::Keys(entries.iter())),
            Operand::List { items, depth: _ } => Ok(Self::Made(items.into_iter().enumerate())),
            Operand::Undefined => Err(failure(FaultKind::Undefined, at)),
            Operand::Loop { .. }
            | Operand::Bool(_)
            | Operand::Int(_)
            | Operand::String(_)
            | Operand::Value(Value::Bool(_) | Value::Int(_) | Value::String(_)) => Err(failure(FaultKind::Type, at)),
        }
    }
    fn next(&mut self) -> Option<Selected<'c>> {
        match self {
            Self::Values(items) => {
                let (index, value) = items.next()?;
                Some(Selected::Pair(Operand::Int(i64::try_from(index).expect("bounded index")), Operand::Value(value)))
            }
            Self::Made(items) => {
                let (index, value) = items.next()?;
                Some(Selected::Pair(Operand::Int(i64::try_from(index).expect("bounded index")), value))
            }
            Self::Keys(entries) => {
                for entry in entries.by_ref() {
                    if let Some(value) = &entry.value {
                        return Some(Selected::Pair(
                            Operand::String(bytes::copy_of(&entry.key)),
                            Operand::Value(value),
                        ));
                    }
                }
                None
            }
            Self::Once(pending) => {
                if *pending {
                    *pending = false;
                    Some(Selected::Plain)
                } else {
                    None
                }
            }
        }
    }
}
/// Rebind the index among present elements of an entry's iterable.
pub fn element<'c>(
    entry: &Entry,
    index: u32,
    context: &Context<'c>,
    limits: &Limits,
) -> Result<(Operand<'c>, Operand<'c>), Fault> {
    let when = entry.for_each.as_ref().expect("concrete element has for_each");
    let mut source = Source::new(condition(when, context, &Bindings::with_capacity(0), limits)?, when.at)?;
    for current in 0..=index {
        let Some(Selected::Pair(key, value)) = source.next() else {
            return Err(failure(FaultKind::NoIndex, when.at));
        };
        if current == index {
            return Ok((key, value));
        }
    }
    unreachable!("inclusive index")
}
fn written<'v>(value: &'v Operand<'_>) -> Result<Written<'v>, FaultKind> {
    match value {
        Operand::Bool(_) | Operand::Value(Value::Bool(_)) => Err(FaultKind::Type),
        Operand::Undefined
        | Operand::Loop { .. }
        | Operand::Int(_)
        | Operand::String(_)
        | Operand::List { .. }
        | Operand::Value(_) => evaluate::output(value),
    }
}
fn length(value: Written<'_>) -> usize {
    match value {
        Written::Bytes(bytes) => bytes.len(),
        Written::Integer { negative, digits } => digits.as_bytes().len().saturating_add(usize::from(negative)),
    }
}
/// Declaration strings accept text and integers, charging every emitted byte.
pub fn interpolate<'c>(
    text: &Interpolated,
    context: &Context<'c>,
    bindings: &Bindings<'c>,
    limits: &Limits,
    left: &mut u64,
) -> Result<Box<[u8]>, Fault> {
    let at = text.at();
    match text {
        Interpolated::Plain { at: _, text } => {
            *left = left
                .checked_sub(u64::try_from(text.len()).expect("bounded text"))
                .ok_or(failure(FaultKind::Limit, at))?;
            Ok(bytes::copy_of(text))
        }
        Interpolated::Parts { at: _, parts } => {
            let mut values = List::with_capacity(u32::try_from(parts.len()).expect("bounded parts"));
            let mut budget = Budget::new(limits);
            let mut evaluator = Evaluator::new(limits);
            let mut size = 0_usize;
            for part in parts {
                let value = match &part.does {
                    Does::Text(text) => Operand::String(bytes::copy_of(text)),
                    Does::Output(expression) => {
                        located(evaluate::expression(&mut evaluator, expression, context, bindings, &mut budget), at)?
                    }
                    Does::Branch { .. }
                    | Does::Jump { .. }
                    | Does::Enter { .. }
                    | Does::Advance { .. }
                    | Does::Set { .. } => unreachable!("compiled interpolation has only text and output"),
                };
                let written = match written(&value) {
                    Ok(value) => value,
                    Err(kind) => return Err(failure(kind, at)),
                };
                size = size.checked_add(length(written)).ok_or(failure(FaultKind::Limit, at))?;
                values.push(value).expect("part count");
            }
            *left =
                left.checked_sub(u64::try_from(size).expect("bounded text")).ok_or(failure(FaultKind::Limit, at))?;
            let mut writer = Writer::new(size);
            for value in values.into_boxed() {
                match written(&value).expect("checked output") {
                    Written::Bytes(bytes) => writer.put(bytes).expect("measured"),
                    Written::Integer { negative, digits } => {
                        if negative {
                            writer.put(b"-").expect("measured");
                        }
                        writer.put(digits.as_bytes()).expect("measured");
                    }
                }
            }
            Ok(writer.finish())
        }
    }
}
