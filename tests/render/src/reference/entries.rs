//! Recursive declaration semantics, independent of translated instructions.
use crate::{
    declarations::{Concrete, Declaration, Expansion, Group, Literal, Piece, Text, Tree},
    expressions::{Roots, Tree as Expression},
};
use hearth_domain_render::{
    entries::Fielded,
    errors::Takes,
    evaluate::{ELEMENT, Value},
    limits::Limits,
    revision::declaration::Name,
};
use std::collections::BTreeSet;
/// # Errors
/// Returns the first declaration evaluation, type, collision or budget fault.
pub fn expand(tree: &Tree, roots: &Roots, limits: &Limits) -> Expansion {
    let mut roots = roots.clone();
    roots.bindings.clear();
    let mut groups = Vec::new();
    let mut keys = Keys { seen: BTreeSet::new(), count: 0 };
    let mut left = limits.expanded;
    for declaration in &tree.0 {
        keys.seen.clear();
        let mut entries = Vec::new();
        let mut contexts = Vec::new();
        match &declaration.each {
            None => contexts.push(roots.clone()),
            Some(expression) => {
                let iterable = evaluate(expression, &roots, limits)?.ok_or("undefined")?;
                let values = match iterable {
                    Value::List(items) => items
                        .into_iter()
                        .enumerate()
                        .map(|(index, value)| (Value::Int(i64::try_from(index).expect("small index")), value))
                        .collect::<Vec<_>>(),
                    Value::Map(entries) => entries
                        .into_iter()
                        .filter_map(|entry| entry.value.map(|value| (Value::String(entry.key), value)))
                        .collect(),
                    Value::Bool(_) | Value::Int(_) | Value::String(_) => return Err("type"),
                };
                for (key, value) in values {
                    let mut roots = roots.clone();
                    roots.bindings = vec![(b"key", key), (b"value", value)];
                    contexts.push(roots);
                }
            }
        }
        for context in &contexts {
            entry(declaration, context, limits, &mut keys, &mut left, &mut entries)?;
        }
        entries.sort_by(|a, b| a.key.cmp(&b.key));
        groups.push(Group { map: declaration.map, entries });
    }
    Ok(groups)
}
fn evaluate(tree: &Expression, roots: &Roots, limits: &Limits) -> Result<Option<Value>, &'static str> {
    super::expression(tree, roots, limits, &mut { limits.made }).map(|made| made.value)
}
struct Keys {
    seen: BTreeSet<Vec<u8>>,
    count: u32,
}
fn entry(
    declaration: &Declaration,
    roots: &Roots,
    limits: &Limits,
    keys: &mut Keys,
    left: &mut u64,
    entries: &mut Vec<Concrete>,
) -> Result<(), &'static str> {
    if let Some(when) = &declaration.when
        && !super::truthy(evaluate(when, roots, limits)?.as_ref())
    {
        return Ok(());
    }
    if keys.count >= limits.expansion {
        return Err("limit");
    }
    let key = text(&declaration.key, roots, limits, left)?;
    if !keys.seen.insert(key.clone()) {
        return Err("collision");
    }
    charge_bytes(ELEMENT, left)?;
    keys.count += 1;
    let mut fields = Vec::new();
    for (name, value) in &declaration.fields {
        if let Some(value) = literal(value, name.takes(), roots, limits, left)? {
            charge_field(*name, left)?;
            fields.push((*name, value));
        }
    }
    entries.push(Concrete { key, fields });
    Ok(())
}
fn charge(text: Vec<u8>, left: &mut u64) -> Result<Vec<u8>, &'static str> {
    charge_bytes(u64::try_from(text.len()).expect("bounded text"), left)?;
    Ok(text)
}
fn charge_bytes(bytes: u64, left: &mut u64) -> Result<(), &'static str> {
    *left = left.checked_sub(bytes).ok_or("limit")?;
    Ok(())
}
fn charge_field(name: Name, left: &mut u64) -> Result<(), &'static str> {
    charge_bytes(ELEMENT + u64::try_from(name.key().len()).expect("field name"), left)
}
fn range_value(base: i64, count: i64, left: &mut u64) -> Result<Fielded, &'static str> {
    charge_field(Name::Base, left)?;
    charge_field(Name::Count, left)?;
    Ok(Fielded::Range { base, count })
}
fn charge_elements(count: usize, left: &mut u64) -> Result<(), &'static str> {
    charge_bytes(u64::try_from(count).expect("bounded list") * ELEMENT, left)
}
fn text(text: &Text, roots: &Roots, limits: &Limits, left: &mut u64) -> Result<Vec<u8>, &'static str> {
    let mut output = Vec::new();
    for part in &text.0 {
        output.extend(match part {
            Piece::Text(text) => text.clone(),
            Piece::Output(expression) => match evaluate(expression, roots, limits)?.ok_or("undefined")? {
                Value::String(text) => text.into_vec(),
                Value::Int(value) => value.to_string().into_bytes(),
                Value::Bool(_) => return Err("type"),
                Value::List(_) | Value::Map(_) => return Err("output"),
            },
        });
    }
    charge(output, left)
}
fn literal(
    value: &Literal,
    takes: Takes,
    roots: &Roots,
    limits: &Limits,
    left: &mut u64,
) -> Result<Option<Fielded>, &'static str> {
    Ok(Some(match value {
        Literal::Int(value) => Fielded::Int(*value),
        Literal::Bool(value) => Fielded::Bool(*value),
        Literal::Text(input) => {
            if let [Piece::Output(expression)] = input.0.as_slice()
                && evaluate(expression, roots, limits)?.is_none()
            {
                return Ok(None);
            }
            Fielded::Text(text(input, roots, limits, left)?.into_boxed_slice())
        }
        Literal::Strings(each) => {
            charge_elements(each.len(), left)?;
            let strings = each.iter().map(|input| text(input, roots, limits, left).map(Vec::into_boxed_slice));
            Fielded::Strings(strings.collect::<Result<Box<[_]>, _>>()?)
        }
        Literal::Range(base, count) => {
            let Some(Fielded::Int(base)) = literal(base, Takes::Int, roots, limits, left)? else {
                return Err("undefined");
            };
            let Some(Fielded::Int(count)) = literal(count, Takes::Int, roots, limits, left)? else {
                return Err("undefined");
            };
            range_value(base, count, left)?
        }
        Literal::Typed(expression) => {
            let Some(value) = evaluate(expression, roots, limits)? else {
                return Ok(None);
            };
            match (takes, value) {
                (Takes::Int, Value::Int(value)) => Fielded::Int(value),
                (Takes::Bool, Value::Bool(value)) => Fielded::Bool(value),
                (Takes::Strings, Value::List(each)) => {
                    charge_elements(each.len(), left)?;
                    Fielded::Strings(
                        each.into_iter()
                            .map(|value| {
                                let Value::String(text) = value else {
                                    return Err("wrong");
                                };
                                charge(text.into_vec(), left).map(Vec::into_boxed_slice)
                            })
                            .collect::<Result<Box<[_]>, _>>()?,
                    )
                }
                (Takes::Range, Value::Map(fields)) => {
                    let mut base = None;
                    let mut count = None;
                    for field in fields {
                        let Some(Value::Int(value)) = field.value else {
                            return Err("wrong");
                        };
                        match field.key.as_ref() {
                            b"base" => base = Some(value),
                            b"count" => count = Some(value),
                            _ => return Err("wrong"),
                        }
                    }
                    range_value(base.ok_or("wrong")?, count.ok_or("wrong")?, left)?
                }
                (
                    Takes::Text | Takes::Int | Takes::Bool | Takes::Strings | Takes::Range,
                    Value::Bool(_) | Value::Int(_) | Value::String(_) | Value::List(_) | Value::Map(_),
                ) => return Err("wrong"),
            }
        }
    }))
}
