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
    pub fields: Box<[Made]>,
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
/// Expand keys once per selected element, checking collisions per map and counting across maps.
pub fn expand(
    declaration: &Declaration,
    context: &Context<'_>,
    limits: &Limits,
    left: &mut u64,
) -> Result<Expanded, Fault> {
    let mut count = 0_u32;
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
                if count == limits.expansion {
                    let at = match &entry.for_each {
                        Some(when) => when.at,
                        None => at,
                    };
                    return Err(failure(FaultKind::Limit, at));
                }
                let key = interpolate(&entry.key, context, &bindings, limits, left)?;
                if let Some(first) = ordered.get(&key) {
                    return Err(Fault { kind: Kind::Collision(Collision::Key { key }), at, also: Some(first.at) });
                }
                charge(evaluate::ELEMENT, at, left)?;
                count = count.checked_add(1).expect("entry limit checked");
                ordered
                    .insert(
                        key.clone(),
                        Concrete { key, at, element, fields: fields(entry, context, &bindings, limits, left)? },
                    )
                    .expect("entry limit checked");
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
            charge(u64::try_from(text.len()).expect("bounded text"), at, left)?;
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
            charge(u64::try_from(size).expect("bounded text"), at, left)?;
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

use crate::{
    errors::{Schema, Takes, TypeName},
    revision::declaration::{Field, Literal, Name},
};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Made {
    pub name: Name,
    pub at: Position,
    pub value: Fielded,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Fielded {
    Text(Box<[u8]>),
    Int(i64),
    Bool(bool),
    Strings(Box<[Box<[u8]>]>),
    Range { base: i64, count: i64 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Expansion {
    pub tasks: Box<[Expanded]>,
}
#[derive(Clone, Copy, Debug)]
pub struct Located<'a> {
    pub task: &'a [u8],
    pub entry: &'a Concrete,
}
impl Expansion {
    /// A reserved entry cannot reach a kind and yield a second error.
    pub fn refuse(&mut self, map: Map, task: &[u8], key: &[u8]) {
        for expanded in &mut self.tasks {
            if expanded.task.as_ref() != task {
                continue;
            }
            for group in &mut expanded.groups {
                if group.map != map {
                    continue;
                }
                let mut kept = List::with_capacity(u32::try_from(group.entries.len()).expect("entry count"));
                for entry in core::mem::replace(&mut group.entries, Box::new([])) {
                    if entry.key.as_ref() != key {
                        kept.push(entry).expect("entry count");
                    }
                }
                group.entries = kept.into_boxed();
            }
        }
    }

    #[must_use]
    pub fn of(&self, map: Map) -> Box<[Located<'_>]> {
        let mut count = 0_u32;
        for task in &self.tasks {
            for group in &task.groups {
                if group.map == map {
                    count = count.saturating_add(u32::try_from(group.entries.len()).expect("entry count"));
                }
            }
        }
        let mut entries = List::with_capacity(count);
        for task in &self.tasks {
            for group in &task.groups {
                if group.map == map {
                    for entry in &group.entries {
                        entries.push(Located { task: &task.task, entry }).expect("entry count");
                    }
                }
            }
        }
        entries.into_boxed()
    }
}
impl Concrete {
    fn field(&self, name: Name) -> Option<&Fielded> {
        for field in &self.fields {
            if field.name == name {
                return Some(&field.value);
            }
        }
        None
    }
    #[must_use]
    pub fn text(&self, name: Name) -> Option<&[u8]> {
        match self.field(name)? {
            Fielded::Text(text) => Some(text),
            Fielded::Int(_) | Fielded::Bool(_) | Fielded::Strings(_) | Fielded::Range { .. } => None,
        }
    }
    #[must_use]
    pub fn int(&self, name: Name) -> Option<i64> {
        match self.field(name)? {
            Fielded::Int(value) => Some(*value),
            Fielded::Text(_) | Fielded::Bool(_) | Fielded::Strings(_) | Fielded::Range { .. } => None,
        }
    }
    #[must_use]
    pub fn bool(&self, name: Name) -> Option<bool> {
        match self.field(name)? {
            Fielded::Bool(value) => Some(*value),
            Fielded::Text(_) | Fielded::Int(_) | Fielded::Strings(_) | Fielded::Range { .. } => None,
        }
    }
    #[must_use]
    pub fn strings(&self, name: Name) -> Option<&[Box<[u8]>]> {
        match self.field(name)? {
            Fielded::Strings(each) => Some(each),
            Fielded::Text(_) | Fielded::Int(_) | Fielded::Bool(_) | Fielded::Range { .. } => None,
        }
    }
    #[must_use]
    pub fn range(&self, name: Name) -> Option<(i64, i64)> {
        match self.field(name)? {
            Fielded::Range { base, count } => Some((*base, *count)),
            Fielded::Text(_) | Fielded::Int(_) | Fielded::Bool(_) | Fielded::Strings(_) => None,
        }
    }
}
fn wrong(takes: Takes, at: Position) -> Fault {
    let expected = match takes {
        Takes::Text => TypeName::String,
        Takes::Int => TypeName::Int,
        Takes::Bool => TypeName::Bool,
        Takes::Strings => TypeName::List,
        Takes::Range => TypeName::Object,
    };
    Fault { kind: Kind::Schema(Schema::Wrong { expected }), at, also: None }
}
fn single(text: &Interpolated) -> Option<&evaluate::Expression> {
    match text {
        Interpolated::Plain { at: _, text: _ } => None,
        Interpolated::Parts { at: _, parts } => {
            if let [evaluate::template::Instruction { at: _, does: Does::Output(expression) }] = parts.as_ref() {
                Some(expression)
            } else {
                None
            }
        }
    }
}
fn literal_position(value: &Literal) -> Position {
    match value {
        Literal::Text(text) | Literal::Typed(text) => text.at(),
        Literal::Plain { at, text: _ }
        | Literal::Bool { at, value: _ }
        | Literal::Strings { at, each: _ }
        | Literal::Range { at, fields: _ } => *at,
        Literal::Int(number) => number.at,
    }
}
fn fields<'c>(
    entry: &Entry,
    context: &Context<'c>,
    bindings: &Bindings<'c>,
    limits: &Limits,
    left: &mut u64,
) -> Result<Box<[Made]>, Fault> {
    let mut fields = List::with_capacity(u32::try_from(entry.fields.len()).expect("field count"));
    for field in &entry.fields {
        if let Some(value) = literal(&field.value, field.name.takes(), context, bindings, limits, left)? {
            charge_field(field.name, literal_position(&field.value), left)?;
            fields.push(Made { name: field.name, at: field.at, value }).expect("field count");
        }
    }
    Ok(fields.into_boxed())
}
fn literal<'c>(
    literal: &Literal,
    takes: Takes,
    context: &Context<'c>,
    bindings: &Bindings<'c>,
    limits: &Limits,
    left: &mut u64,
) -> Result<Option<Fielded>, Fault> {
    Ok(Some(match literal {
        Literal::Text(text) => {
            if let Some(expression) = single(text) {
                let value = optional(expression, text.at(), context, bindings, limits)?;
                if value.is_none() {
                    return Ok(None);
                }
            }
            Fielded::Text(interpolate(text, context, bindings, limits, left)?)
        }
        Literal::Plain { at, text } => Fielded::Text(charged(text, *at, left)?),
        Literal::Typed(text) => {
            let expression = single(text).ok_or(wrong(takes, text.at()))?;
            let value = optional(expression, text.at(), context, bindings, limits)?;
            let Some(value) = value else {
                return Ok(None);
            };
            typed(value, takes, text.at(), left)?
        }
        Literal::Int(number) => Fielded::Int(number.value),
        Literal::Bool { at: _, value } => Fielded::Bool(*value),
        Literal::Strings { at, each } => {
            charge_elements(each.len(), *at, left)?;
            let mut strings = List::with_capacity(u32::try_from(each.len()).expect("string count"));
            for text in each {
                strings.push(interpolate(text, context, bindings, limits, left)?).expect("string count");
            }
            Fielded::Strings(strings.into_boxed())
        }
        Literal::Range { at, fields } => range(fields, *at, context, bindings, limits, left)?,
    }))
}
fn typed(value: Operand<'_>, takes: Takes, at: Position, left: &mut u64) -> Result<Fielded, Fault> {
    match (takes, value) {
        (Takes::Int, Operand::Int(value)) => Ok(Fielded::Int(value)),
        (Takes::Int, Operand::Value(Value::Int(value))) => Ok(Fielded::Int(*value)),
        (Takes::Bool, Operand::Bool(value)) => Ok(Fielded::Bool(value)),
        (Takes::Bool, Operand::Value(Value::Bool(value))) => Ok(Fielded::Bool(*value)),
        (Takes::Strings, Operand::Value(Value::List(each))) => {
            charge_elements(each.len(), at, left)?;
            let mut strings = List::with_capacity(u32::try_from(each.len()).expect("string count"));
            for item in each {
                let Value::String(text) = item else {
                    return Err(wrong(takes, at));
                };
                strings.push(charged(text, at, left)?).expect("string count");
            }
            Ok(Fielded::Strings(strings.into_boxed()))
        }
        (Takes::Strings, Operand::List { items, depth: _ }) => {
            charge_elements(items.len(), at, left)?;
            let mut strings = List::with_capacity(u32::try_from(items.len()).expect("string count"));
            for item in items {
                let text = match &item {
                    Operand::String(text) | Operand::Value(Value::String(text)) => text,
                    Operand::Undefined => return Err(failure(FaultKind::Undefined, at)),
                    Operand::Loop { .. }
                    | Operand::Bool(_)
                    | Operand::Int(_)
                    | Operand::List { .. }
                    | Operand::Value(Value::Bool(_) | Value::Int(_) | Value::List(_) | Value::Map(_)) => {
                        return Err(wrong(takes, at));
                    }
                };
                strings.push(charged(text, at, left)?).expect("string count");
            }
            Ok(Fielded::Strings(strings.into_boxed()))
        }
        (Takes::Range, Operand::Value(Value::Map(fields))) => {
            let mut base = None;
            let mut count = None;
            for field in fields {
                let Some(Value::Int(value)) = &field.value else {
                    return Err(wrong(takes, at));
                };
                if field.key.as_ref() == b"base" {
                    base = Some(*value);
                } else if field.key.as_ref() == b"count" {
                    count = Some(*value);
                } else {
                    return Err(wrong(takes, at));
                }
            }
            range_value(base, count, at, left)
        }
        (
            Takes::Text | Takes::Int | Takes::Bool | Takes::Strings | Takes::Range,
            Operand::Undefined
            | Operand::Loop { .. }
            | Operand::Bool(_)
            | Operand::Int(_)
            | Operand::String(_)
            | Operand::List { .. }
            | Operand::Value(_),
        ) => Err(wrong(takes, at)),
    }
}
fn charged(text: &[u8], at: Position, left: &mut u64) -> Result<Box<[u8]>, Fault> {
    charge(u64::try_from(text.len()).expect("bounded text"), at, left)?;
    Ok(bytes::copy_of(text))
}
fn charge(bytes: u64, at: Position, left: &mut u64) -> Result<(), Fault> {
    *left = left.checked_sub(bytes).ok_or(failure(FaultKind::Limit, at))?;
    Ok(())
}
fn charge_field(name: Name, at: Position, left: &mut u64) -> Result<(), Fault> {
    charge(evaluate::ELEMENT.saturating_add(u64::try_from(name.key().len()).expect("field name")), at, left)
}
fn range_value(base: Option<i64>, count: Option<i64>, at: Position, left: &mut u64) -> Result<Fielded, Fault> {
    let base = base.ok_or(wrong(Takes::Range, at))?;
    let count = count.ok_or(wrong(Takes::Range, at))?;
    charge_field(Name::Base, at, left)?;
    charge_field(Name::Count, at, left)?;
    Ok(Fielded::Range { base, count })
}
fn charge_elements(count: usize, at: Position, left: &mut u64) -> Result<(), Fault> {
    charge(u64::try_from(count).expect("bounded list").saturating_mul(evaluate::ELEMENT), at, left)
}
fn range<'c>(
    fields: &[Field],
    at: Position,
    context: &Context<'c>,
    bindings: &Bindings<'c>,
    limits: &Limits,
    left: &mut u64,
) -> Result<Fielded, Fault> {
    let mut base = None;
    let mut count = None;
    for field in fields {
        let value = match &field.value {
            Literal::Int(number) => number.value,
            Literal::Typed(text) => {
                let expression = single(text).ok_or(wrong(Takes::Int, text.at()))?;
                let value = optional(expression, text.at(), context, bindings, limits)?;
                let value = value.ok_or(failure(FaultKind::Undefined, text.at()))?;
                let Fielded::Int(value) = typed(value, Takes::Int, text.at(), left)? else {
                    unreachable!("integer type");
                };
                value
            }
            Literal::Text(_)
            | Literal::Plain { .. }
            | Literal::Bool { .. }
            | Literal::Strings { .. }
            | Literal::Range { .. } => return Err(wrong(Takes::Int, field.at)),
        };
        match field.name {
            Name::Base => base = Some(value),
            Name::Count => count = Some(value),
            Name::Action
            | Name::Age
            | Name::Algorithm
            | Name::Argument
            | Name::Class
            | Name::Component
            | Name::Copy
            | Name::Directory
            | Name::From
            | Name::Gecos
            | Name::Gid
            | Name::Group
            | Name::Groups
            | Name::Home
            | Name::Kind
            | Name::Link
            | Name::Mode
            | Name::Order
            | Name::Owner
            | Name::Param
            | Name::Password
            | Name::Path
            | Name::Paths
            | Name::PublicPath
            | Name::Sha256
            | Name::Shell
            | Name::Strip
            | Name::Subordinate
            | Name::Template
            | Name::Type
            | Name::Uid
            | Name::Unit
            | Name::Unpack
            | Name::Url
            | Name::Value => unreachable!("compiled range names"),
        }
    }
    range_value(base, count, at, left)
}

/// The evaluator, bindings and expanded task at these bounds.
/// Empty strings still need list slots; their source is bounded by decoded or made input.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let count = u64::from(limits.expansion);
    let fields = count.checked_mul(37)?;
    let input = limits.validated.checked_add(limits.made)?.checked_add(u64::from(limits.document))?;
    evaluate::worst_case(limits)?
        .checked_add(List::<Binding<'_>>::worst_case(2)?)?
        .checked_add(Keys::<Box<[u8]>, Concrete>::worst_case(limits.expansion)?)?
        .checked_add(List::<Concrete>::worst_case(limits.expansion)?)?
        .checked_add(List::<Group>::worst_case(14)?)?
        .checked_add(fields.checked_mul(u64::try_from(size_of::<Made>()).ok()?)?)?
        .checked_add(fields.checked_mul(input)?)?
        .checked_add(limits.expanded.checked_mul(4)?)?
        .checked_add(List::<Operand<'_>>::worst_case(limits.document)?)
}

fn optional<'c>(
    expression: &evaluate::Expression,
    at: Position,
    context: &Context<'c>,
    bindings: &Bindings<'c>,
    limits: &Limits,
) -> Result<Option<Operand<'c>>, Fault> {
    let value = located(
        evaluate::expression(&mut Evaluator::new(limits), expression, context, bindings, &mut Budget::new(limits)),
        at,
    )?;
    if value.undefined() { Ok(None) } else { Ok(Some(value)) }
}
