use crate::{
    compose::{Composed, Composite, Layer, Provenance},
    errors::{Schema, TypeName},
    evaluate::{Entry, Value},
    limits::Limits,
    revision::{
        Position,
        declaration::{Form, Spec},
    },
    schema::{Against, Check},
};
use alloc::boxed::Box;
use skein_lib::{List, Stack};
/// Validate an evaluated value without interpreting or filling it (protocol/format.md, 8.1).
pub fn conform(
    value: &Value,
    against: &Against<'_>,
    check: &Check<'_>,
    ty: u32,
    limits: &Limits,
) -> Result<(), Schema> {
    checked(value, against, check, ty, limits)?;
    Ok(())
}
pub(super) fn checked(
    value: &Value,
    against: &Against<'_>,
    check: &Check<'_>,
    ty: u32,
    limits: &Limits,
) -> Result<u64, Schema> {
    let mut size = 0_u64;
    let mut stack = Stack::with_capacity(limits.depth);
    let mut next = Some((value, ty, None));
    for _ in 0..limits.expanded.saturating_mul(2).saturating_add(1) {
        if let Some((value, ty, spec)) = next.take() {
            let form =
                &against.declaration.types.get(usize::try_from(ty).expect("type index")).expect("declared type").form;
            size = size.saturating_add(scalar(value, form, spec, against, check, limits)?);
            let frame = match (value, form) {
                (Value::List(items), Form::List(ty)) => Some(Frame::List { items: items.iter(), ty: *ty }),
                (Value::Map(entries), Form::Map(ty)) => {
                    for entry in entries {
                        if !crate::schema::forms::check(crate::errors::Format::Key, &entry.key) {
                            return Err(Schema::Malformed { format: crate::errors::Format::Key });
                        }
                    }
                    Some(Frame::Map { entries: entries.iter(), ty: *ty, fields: None })
                }
                (Value::Map(entries), Form::Object(fields)) => {
                    object(entries, fields)?;
                    Some(Frame::Map { entries: entries.iter(), ty: 0, fields: Some(fields) })
                }
                (
                    Value::Bool(_) | Value::Int(_) | Value::String(_) | Value::List(_) | Value::Map(_),
                    Form::Bool
                    | Form::Int
                    | Form::String
                    | Form::Path
                    | Form::Mode
                    | Form::Duration
                    | Form::Secret
                    | Form::Target
                    | Form::File
                    | Form::Enum(_)
                    | Form::List(_)
                    | Form::Map(_)
                    | Form::Object(_),
                ) => None,
            };
            if let Some(frame) = frame
                && stack.push(frame).is_err()
            {
                return Err(Schema::Wrong { expected: TypeName::List });
            }
        }
        let Some(mut frame) = stack.pop() else { return Ok(size) };
        next = frame.next();
        if next.is_some() {
            stack.push(frame).expect("popped frame fits");
        }
    }
    unreachable!("expanded values bound conformance")
}
#[derive(Debug)]
enum Frame<'a> {
    List { items: core::slice::Iter<'a, Value>, ty: u32 },
    Map { entries: core::slice::Iter<'a, Entry>, ty: u32, fields: Option<&'a [Spec]> },
}
impl<'a> Frame<'a> {
    fn next(&mut self) -> Option<(&'a Value, u32, Option<&'a Spec>)> {
        match self {
            Self::List { items, ty } => Some((items.next()?, *ty, None)),
            Self::Map { entries, ty, fields } => {
                for entry in entries.by_ref() {
                    let Some(value) = &entry.value else { continue };
                    if let Some(fields) = fields {
                        for spec in *fields {
                            if spec.name.text == entry.key {
                                return Some((value, spec.ty, Some(spec)));
                            }
                        }
                    } else {
                        return Some((value, *ty, None));
                    }
                }
                None
            }
        }
    }
}
fn object(entries: &[Entry], fields: &[Spec]) -> Result<(), Schema> {
    for entry in entries {
        let mut declared = false;
        for field in fields {
            declared |= field.name.text == entry.key;
        }
        if !declared {
            let mut names = List::with_capacity(u32::try_from(fields.len()).expect("field count"));
            for field in fields {
                names.push(field.name.text.clone()).expect("field count");
            }
            return Err(Schema::UndeclaredField { field: entry.key.clone(), declared: names.into_boxed() });
        }
    }
    for field in fields {
        if field.optional {
            continue;
        }
        let mut present = false;
        for entry in entries {
            present |= entry.key == field.name.text && entry.value.is_some();
        }
        if !present {
            return Err(Schema::MissingField { field: field.name.text.clone() });
        }
    }
    Ok(())
}
fn scalar(
    value: &Value,
    form: &Form,
    spec: Option<&Spec>,
    against: &Against<'_>,
    check: &Check<'_>,
    limits: &Limits,
) -> Result<u64, Schema> {
    let kind = match value {
        Value::Bool(value) => Composite::Bool(*value),
        Value::Int(value) => Composite::Int(*value),
        Value::String(value) => Composite::String(value.clone()),
        Value::List(_) => Composite::List(Box::new([])),
        Value::Map(_) => Composite::Map(Box::new([])),
    };
    if let Some(schema) = crate::schema::type_fault(&kind, form, spec) {
        return Err(schema);
    }
    let length = match value {
        Value::String(text) => text.len(),
        Value::List(items) => items.len(),
        Value::Map(entries) => entries.len(),
        Value::Bool(_) | Value::Int(_) => 0,
    };
    if let Some(spec) = spec
        && let Some(max) = &spec.max_length
        && u64::try_from(length).expect("bounded value") > max.value.unsigned_abs()
    {
        return Err(Schema::Length { max_length: max.value });
    }
    let composed = Composed {
        kind,
        from: Provenance { layer: Layer::Defaults, document: 0, at: Position { line: 1, column: 1 } },
    };
    if let Some(schema) =
        crate::schema::forms::fault(&composed, form, check, against.listing, u64::from(limits.static_file))
    {
        return Err(schema);
    }
    let size = match value {
        Value::String(text) => u64::try_from(text.len()).expect("bounded text"),
        Value::List(items) => {
            u64::try_from(items.len()).expect("bounded list").saturating_mul(crate::evaluate::ELEMENT)
        }
        Value::Map(entries) => {
            let mut size = 0_u64;
            for entry in entries {
                size = size
                    .saturating_add(crate::evaluate::ELEMENT)
                    .saturating_add(u64::try_from(entry.key.len()).expect("bounded key"));
            }
            size
        }
        Value::Bool(_) | Value::Int(_) => 0,
    };
    Ok(size)
}
