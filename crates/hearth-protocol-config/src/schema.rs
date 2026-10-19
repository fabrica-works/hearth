//! Settings' schema and its pre-order type arena (protocol/format.md, 6.1 and 6.2).
use crate::{
    Declared, Error, Expected, Object, Position, Required,
    declaration::{Number, Text, fault, object, text, texts, unknown, wrong},
    json::{Entry, Kind, Value},
    limits::Limits,
    names,
};
use alloc::{boxed::Box, vec::IntoIter};
use skein_lib::{List, Stack};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Type {
    pub at: Position,
    pub form: Form,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Form {
    Bool,
    Int,
    String,
    Path,
    Mode,
    Duration,
    Secret,
    Target,
    File,
    Enum(Box<[Text]>),
    List(u32),
    Map(u32),
    Object(Box<[Spec]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Spec {
    pub name: Text,
    pub at: Position,
    pub ty: u32,
    pub default: Option<Value>,
    pub optional: bool,
    pub min: Option<Number>,
    pub max: Option<Number>,
    pub max_length: Option<Number>,
}
/// An export's declared type and its expression text (protocol/format.md, 8.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Export {
    pub name: Text,
    pub at: Position,
    pub ty: u32,
    pub optional: bool,
    pub value: Text,
}
#[derive(Debug)]
struct Pending {
    spec: Spec,
    optional: Option<Position>,
    min: Option<Position>,
    max: Option<Position>,
    length: Option<Position>,
    value: Option<Box<Text>>,
}
#[derive(Debug)]
enum Frame {
    Fields { entries: IntoIter<Entry>, specs: List<Spec>, owner: Option<u32> },
    Finish(Pending),
    Type(Value),
}

type Decoded = (Box<[Spec]>, Box<[Export]>);
pub(crate) fn read(value: Value, limits: &Limits, types: &mut List<Type>, exported: bool) -> Result<Decoded, Error> {
    let entries = object(value)?;
    let count = u32::try_from(entries.len()).expect("document bounded by u32");
    let mut stack = Stack::with_capacity(limits.depth);
    stack
        .push(Frame::Fields { entries: entries.into_iter(), specs: List::with_capacity(count), owner: None })
        .expect("decoded object fits depth");
    let mut exports = List::with_capacity(if exported { count } else { 0 });
    for _ in 0..limits.document {
        let Some(frame) = stack.pop() else { unreachable!("root returns its specs") };
        match frame {
            Frame::Fields { mut entries, specs, owner } => {
                if let Some(entry) = entries.next() {
                    if !exported && owner.is_none() && specs.len() == limits.settings {
                        return Err(fault(entry.at, Declared::Settings));
                    }
                    let (pending, ty) = spec(entry, types.len(), exported && owner.is_none())?;
                    stack.push(Frame::Fields { entries, specs, owner }).expect("decoded nesting fits stack");
                    stack.push(Frame::Finish(pending)).expect("spec occupies a JSON object level");
                    stack.push(Frame::Type(ty)).expect("type occupies a JSON value level");
                } else if let Some(index) = owner {
                    types.get_mut(index).expect("object was placed first").form = Form::Object(specs.into_boxed());
                } else {
                    return Ok((specs.into_boxed(), exports.into_boxed()));
                }
            }
            Frame::Finish(pending) => {
                check(&pending, &types.get(pending.spec.ty).expect("type already read").form)?;
                let Some(Frame::Fields { specs, .. }) = stack.top_mut() else { unreachable!("spec belongs to fields") };
                if let Some(value) = pending.value {
                    let spec = pending.spec;
                    exports
                        .push(Export {
                            name: spec.name,
                            at: spec.at,
                            ty: spec.ty,
                            optional: spec.optional,
                            value: *value,
                        })
                        .expect("one export per root field");
                } else {
                    specs.push(pending.spec).expect("one spec per field");
                }
            }
            Frame::Type(value) => read_type(value, types, &mut stack, exported)?,
        }
    }
    unreachable!("each walk step consumes fewer than one document byte")
}
fn spec(entry: Entry, ty: u32, exported: bool) -> Result<(Pending, Value), Error> {
    if !names::identifier(&entry.key) {
        return Err(fault(entry.at, Declared::Name));
    }
    let at = entry.value.at;
    let mut pending = Pending {
        spec: Spec {
            name: Text { at: entry.at, text: entry.key },
            at,
            ty,
            default: None,
            optional: false,
            min: None,
            max: None,
            max_length: None,
        },
        optional: None,
        min: None,
        max: None,
        length: None,
        value: None,
    };
    let mut value = None;
    let mut doc = false;
    for field in object(entry.value)? {
        match &*field.key {
            b"type" => value = Some(field.value),
            b"doc" => {
                documentation(field.value)?;
                doc = true;
            }
            b"default" if !exported => pending.spec.default = Some(field.value),
            b"value" if exported => pending.value = Some(Box::new(text(field.value)?)),
            b"optional" => {
                optional(field.value)?;
                pending.spec.optional = true;
                pending.optional = Some(field.at);
            }
            b"min" if !exported => {
                pending.spec.min = Some(number(field.value)?);
                pending.min = Some(field.at);
            }
            b"max" if !exported => {
                pending.spec.max = Some(number(field.value)?);
                pending.max = Some(field.at);
            }
            b"max_length" if !exported => {
                pending.spec.max_length = Some(number(field.value)?);
                pending.length = Some(field.at);
            }
            _ => return Err(unknown(field.at, if exported { Object::Export } else { Object::Spec })),
        }
    }
    let value = value.ok_or(fault(at, Declared::Missing { key: Required::Type }))?;
    if !doc {
        return Err(fault(at, Declared::Missing { key: Required::Doc }));
    }
    if exported && pending.value.is_none() {
        return Err(fault(at, Declared::Missing { key: Required::Value }));
    }
    Ok((pending, value))
}
fn documentation(value: Value) -> Result<(), Error> {
    let doc = text(value)?;
    if doc.text.len() > 200 || doc.text.contains(&b'\n') {
        return Err(fault(doc.at, Declared::Doc));
    }
    Ok(())
}
fn optional(value: Value) -> Result<(), Error> {
    if value.kind != Kind::Bool(true) {
        return Err(wrong(value.at, Expected::True));
    }
    Ok(())
}
fn number(value: Value) -> Result<Number, Error> {
    let Kind::Int(number) = value.kind else {
        return Err(wrong(value.at, Expected::Int));
    };
    Ok(Number { at: value.at, value: number })
}
fn check(pending: &Pending, form: &Form) -> Result<(), Error> {
    if pending.spec.default.is_some()
        && let Some(at) = pending.optional
    {
        return Err(fault(at, Declared::Optional));
    }
    if form != &Form::Int {
        if let Some(at) = pending.min {
            return Err(fault(at, Declared::Bound));
        }
        if let Some(at) = pending.max {
            return Err(fault(at, Declared::Bound));
        }
    }
    if let Some(number) = pending.spec.max_length {
        if number.value < 0 {
            return Err(fault(number.at, Declared::Bound));
        }
        match form {
            Form::String | Form::List(_) | Form::Map(_) => {}
            Form::Bool
            | Form::Int
            | Form::Path
            | Form::Mode
            | Form::Duration
            | Form::Secret
            | Form::Target
            | Form::File
            | Form::Enum(_)
            | Form::Object(_) => return Err(fault(pending.length.expect("bound's key"), Declared::Bound)),
        }
    }
    if let Some(min) = pending.spec.min
        && let Some(max) = pending.spec.max
        && min.value > max.value
    {
        return Err(fault(min.at, Declared::Above));
    }
    Ok(())
}
fn read_type(value: Value, types: &mut List<Type>, stack: &mut Stack<Frame>, exported: bool) -> Result<(), Error> {
    let at = value.at;
    let index = types.len();
    let next = index.checked_add(1).expect("document bounded by u32");
    let mut child = None;
    let form = match value.kind {
        Kind::String(name) => match &*name {
            b"bool" => Form::Bool,
            b"int" => Form::Int,
            b"string" => Form::String,
            b"path" => Form::Path,
            b"mode" => Form::Mode,
            b"duration" => Form::Duration,
            b"secret" => Form::Secret,
            b"target" => Form::Target,
            b"file" => Form::File,
            _ => return Err(wrong(at, Expected::Type)),
        },
        Kind::Map(entries) => {
            if entries.len() != 1 {
                return Err(wrong(at, Expected::Type));
            }
            let entry = entries.into_iter().next().expect("one type constructor");
            match &*entry.key {
                b"list" => {
                    child = Some(entry.value);
                    Form::List(next)
                }
                b"map" => {
                    child = Some(entry.value);
                    Form::Map(next)
                }
                b"enum" => {
                    let at = entry.value.at;
                    let values = texts(entry.value, false)?;
                    if values.is_empty() {
                        return Err(fault(at, Declared::Enum));
                    }
                    for (index, value) in values.iter().enumerate() {
                        for previous in values.iter().take(index) {
                            if previous.text == value.text {
                                return Err(fault(value.at, Declared::Enum));
                            }
                        }
                    }
                    Form::Enum(values)
                }
                b"object" => {
                    let entries = object(entry.value)?;
                    let specs = List::with_capacity(u32::try_from(entries.len()).expect("document bounded"));
                    stack
                        .push(Frame::Fields { entries: entries.into_iter(), specs, owner: Some(index) })
                        .expect("object occupies a JSON nesting level");
                    Form::Object(Box::new([]))
                }
                _ => return Err(wrong(at, Expected::Type)),
            }
        }
        Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::List(_) => return Err(wrong(at, Expected::Type)),
    };
    if exported && form == Form::Secret {
        return Err(fault(at, Declared::Secret));
    }
    types.push(Type { at, form }).expect("no more types than document values");
    if let Some(value) = child {
        stack.push(Frame::Type(value)).expect("child replaces its parent frame");
    }
    Ok(())
}

/// Additional peak heap while decoded values and all new schema lists coexist.
pub(crate) fn worst_case(limits: &Limits) -> Option<u64> {
    use core::mem::size_of;
    let entries = u64::from(limits.document / 2).checked_add(1)?;
    let largest = size_of::<Type>().max(size_of::<Spec>()).max(size_of::<Export>());
    // Every source value funds at most one output slot. Export roots also
    // reserve a spec list; boxing a list may temporarily hold both buffers.
    let outputs = entries.checked_mul(u64::try_from(largest).ok()?)?.checked_mul(3)?;
    let arena = List::<Type>::worst_case(limits.document.saturating_div(2).saturating_add(1))?;
    let pending = u64::from(limits.depth).checked_mul(u64::try_from(size_of::<Text>()).ok()?)?;
    outputs.checked_add(arena)?.checked_add(pending)?.checked_add(Stack::<Frame>::worst_case(limits.depth)?)
}
