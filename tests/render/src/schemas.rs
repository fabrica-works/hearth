//! Generated schema arenas and an independent recursive validator (format.md, 6).
use hearth_domain_render::{
    compose::{Composed, Composite, Layer, Pair, Provenance},
    errors::{Schema, TypeName},
    evaluate::{Entry, Value},
    limits,
    revision::{
        self,
        declaration::{Declaration, Form, Number, Spec, Text, Type},
    },
    schema::{self, Against, Check, Fault, Validator},
};
use skein_lib::{List, Rng};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub depth: u32,
    pub width: u32,
}
pub const SMALL: Shape = Shape { depth: 2, width: 2 };
pub const WIDE: Shape = Shape { depth: 4, width: 3 };
pub const KINDS: &[&str] = &["wrong", "enum", "below", "above", "length", "undeclared", "missing", "malformed"];
const AT: revision::Position = revision::Position { line: 2, column: 7 };
#[must_use]
pub fn declaration(rng: &mut Rng, shape: Shape) -> Declaration {
    let mut types = Vec::new();
    let mut serial = 1;
    let ty = generate_type(rng, shape, &mut types, &mut serial);
    let spec = generate_spec(rng, b"x", ty, &types, &mut serial);
    Declaration {
        at: AT,
        types: types.into_boxed_slice(),
        settings: Box::new([spec]),
        requires: Box::new([]),
        scripts: Box::new([]),
        exports: Box::new([]),
        entries: Box::new([]),
        reads: Box::new([]),
    }
}
fn generate_type(rng: &mut Rng, shape: Shape, types: &mut Vec<Type>, serial: &mut u32) -> u32 {
    let index = u32::try_from(types.len()).expect("small arena");
    types.push(Type { at: AT, form: Form::Bool });
    let form = match rng.below(if shape.depth == 0 { 10 } else { 13 }) {
        0 => Form::Bool,
        1 => Form::Int,
        2 => Form::String,
        3 => Form::Path,
        4 => Form::Mode,
        5 => Form::Duration,
        6 => Form::Secret,
        7 => Form::Target,
        8 => Form::File,
        9 => Form::Enum(Box::new([text(b"a", AT), text(b"b", AT)])),
        10 => Form::List(generate_type(rng, Shape { depth: shape.depth.saturating_sub(1), ..shape }, types, serial)),
        11 => Form::Map(generate_type(rng, Shape { depth: shape.depth.saturating_sub(1), ..shape }, types, serial)),
        12 => {
            let mut fields = Vec::new();
            for index in 0..=rng.below(u64::from(shape.width)) {
                let name = format!("f{index}");
                let ty = generate_type(rng, Shape { depth: shape.depth.saturating_sub(1), ..shape }, types, serial);
                fields.push(generate_spec(rng, name.as_bytes(), ty, types, serial));
            }
            Form::Object(fields.into_boxed_slice())
        }
        _ => unreachable!("bounded choice"),
    };
    types.get_mut(usize::try_from(index).expect("small")).expect("placed type").form = form;
    index
}
fn generate_spec(rng: &mut Rng, name: &[u8], ty: u32, types: &[Type], serial: &mut u32) -> Spec {
    let at = revision::Position { line: 1, column: *serial };
    *serial = serial.saturating_add(1);
    let form = &types.get(usize::try_from(ty).expect("small")).expect("type").form;
    let default = if rng.chance(300) { canonical(form, at) } else { None };
    Spec {
        name: text(name, at),
        at,
        ty,
        optional: default.is_none() && rng.chance(300),
        default,
        min: if matches!(form, Form::Int) { Some(Number { at, value: 0 }) } else { None },
        max: if matches!(form, Form::Int) { Some(Number { at, value: 3 }) } else { None },
        max_length: if matches!(form, Form::String | Form::List(_) | Form::Map(_)) {
            Some(Number { at, value: 2 })
        } else {
            None
        },
    }
}
fn canonical(form: &Form, at: revision::Position) -> Option<revision::Value> {
    let kind = match form {
        Form::Bool => revision::Kind::Bool(true),
        Form::Int => revision::Kind::Int(2),
        Form::String | Form::Enum(_) | Form::Secret | Form::Target | Form::File => {
            revision::Kind::String(Box::from(b"a".as_slice()))
        }
        Form::Path => revision::Kind::String(Box::from(b"/a".as_slice())),
        Form::Mode => revision::Kind::String(Box::from(b"644".as_slice())),
        Form::Duration => revision::Kind::String(Box::from(b"1s".as_slice())),
        Form::List(_) | Form::Map(_) | Form::Object(_) => return None,
    };
    Some(revision::Value { at, kind })
}
fn text(name: &[u8], at: revision::Position) -> Text {
    Text { at, text: Box::from(name) }
}
fn origin(layer: Layer, document: u32, at: revision::Position) -> Provenance {
    Provenance { layer, document, at }
}
#[must_use]
pub fn settings(rng: &mut Rng, declaration: &Declaration) -> Composed {
    let mut pairs = Vec::new();
    for spec in &declaration.settings {
        if rng.chance(800) {
            pairs.push(Pair {
                key: spec.name.text.clone(),
                from: origin(Layer::Base, 2, AT),
                value: setting(rng, declaration, spec.ty),
            });
        }
    }
    Composed { from: origin(Layer::Base, 2, AT), kind: Composite::Map(pairs.into_boxed_slice()) }
}
fn setting(rng: &mut Rng, declaration: &Declaration, ty: u32) -> Composed {
    let form = &declaration.types.get(usize::try_from(ty).expect("small")).expect("type").form;
    let kind = if rng.chance(200) {
        if matches!(form, Form::Bool) { Composite::Int(9) } else { Composite::Bool(true) }
    } else {
        match form {
            Form::Bool => Composite::Bool(rng.chance(500)),
            Form::Int => Composite::Int(i64::try_from(rng.below(8)).expect("small") - 2),
            Form::Path => Composite::String(Box::from(if rng.chance(500) { b"/a".as_slice() } else { b"a/../b" })),
            Form::Mode => Composite::String(Box::from(if rng.chance(500) { b"644".as_slice() } else { b"0844" })),
            Form::Duration => Composite::String(Box::from(if rng.chance(500) { b"14d".as_slice() } else { b"05s" })),
            Form::Secret => {
                Composite::String(Box::from(if rng.chance(500) { b"users/free".as_slice() } else { b"Users/free" }))
            }
            Form::String | Form::Target | Form::File | Form::Enum(_) => {
                Composite::String(Box::from(if rng.chance(500) { b"a".as_slice() } else { b"xxxx" }))
            }
            Form::List(ty) => Composite::List((0..rng.below(5)).map(|_| setting(rng, declaration, *ty)).collect()),
            Form::Map(ty) => Composite::Map(
                (0..rng.below(5))
                    .map(|index| Pair {
                        key: format!("k{index}").as_bytes().into(),
                        from: origin(Layer::Base, 2, AT),
                        value: setting(rng, declaration, *ty),
                    })
                    .collect(),
            ),
            Form::Object(fields) => {
                let mut pairs = Vec::new();
                for spec in fields {
                    if rng.chance(700) {
                        pairs.push(Pair {
                            key: spec.name.text.clone(),
                            from: origin(Layer::Base, 2, AT),
                            value: setting(rng, declaration, spec.ty),
                        });
                    }
                }
                if rng.chance(300) {
                    pairs.push(Pair {
                        key: Box::from(b"zz".as_slice()),
                        from: origin(Layer::Base, 2, AT),
                        value: Composed { from: origin(Layer::Base, 2, AT), kind: Composite::Bool(true) },
                    });
                }
                Composite::Map(pairs.into_boxed_slice())
            }
        }
    };
    Composed { from: origin(Layer::Base, 2, AT), kind }
}
/// # Errors
/// Returns all schema faults from the real bounded validator.
pub fn validate(declaration: &Declaration, settings: &Composed) -> Result<Value, Box<[Fault]>> {
    let entry = super::fixtures::entry(b"tasks/task/task.json", b'7');
    let listing = [super::fixtures::entry(b"a", b'8'), entry];
    let mut documents = revision::Documents::new(&listing);
    documents.set_declaration(&super::fixtures::oid(b'7'), declaration.clone());
    let mut faults = List::with_capacity(16);
    let defaults = schema::declarations(
        &documents,
        &listing,
        &mut Validator::new(&limits::SHIPPED),
        &mut hearth_domain_render::compose::Merge::new(&limits::SHIPPED),
        &mut faults,
    );
    assert_eq!(faults.len(), 0, "generated defaults are sound");
    let against =
        Against { declaration, defaults: defaults.of(b"task").expect("sound declaration"), listing: &listing };
    let mut faults = List::with_capacity(16);
    let mut left = u64::MAX;
    match Validator::new(&limits::SHIPPED).settings(settings, &against, &Check::Default, &mut left, &mut faults) {
        Some(value) => Ok(value),
        None => Err(faults.into_boxed()),
    }
}
/// # Errors
/// Returns independently derived schema faults, in field order.
pub fn reference(declaration: &Declaration, settings: &Composed) -> Result<Value, Box<[Fault]>> {
    let mut faults = Vec::new();
    let value = object(declaration, settings, &declaration.settings, &mut faults, true);
    if faults.is_empty() { Ok(value) } else { Err(faults.into_boxed_slice()) }
}
fn issue(faults: &mut Vec<Fault>, value: &Composed, schema: Schema) {
    faults.push(Fault { schema, document: value.from.document, at: value.from.at });
}
fn object(declaration: &Declaration, value: &Composed, fields: &[Spec], faults: &mut Vec<Fault>, root: bool) -> Value {
    let Composite::Map(pairs) = &value.kind else {
        issue(faults, value, Schema::Wrong { expected: TypeName::Object });
        return Value::Bool(false);
    };
    let pairs: BTreeMap<&[u8], &Pair> = pairs.iter().map(|pair| (pair.key.as_ref(), pair)).collect();
    let fields: BTreeMap<&[u8], &Spec> = fields.iter().map(|spec| (spec.name.text.as_ref(), spec)).collect();
    let keys: BTreeSet<&[u8]> = pairs.keys().chain(fields.keys()).copied().collect();
    let declared: Box<[Box<[u8]>]> = fields.keys().map(|name| Box::from(*name)).collect();
    let mut entries = Vec::new();
    for key in keys {
        match (pairs.get(key), fields.get(key)) {
            (Some(pair), Some(spec)) => entries.push(Entry {
                key: Box::from(key),
                value: Some(checked(declaration, &pair.value, spec.ty, Some(spec), faults)),
            }),
            (Some(pair), None) => faults.push(Fault {
                schema: if root {
                    Schema::Undeclared { setting: Box::from(key), declared: declared.clone() }
                } else {
                    Schema::UndeclaredField { field: Box::from(key), declared: declared.clone() }
                },
                document: pair.from.document,
                at: pair.from.at,
            }),
            (None, Some(spec)) => {
                let filled = match &spec.default {
                    Some(default) => Some(checked(declaration, &written(default), spec.ty, Some(spec), faults)),
                    None if spec.optional => None,
                    None => {
                        issue(
                            faults,
                            value,
                            if root {
                                Schema::Missing { setting: Box::from(key) }
                            } else {
                                Schema::MissingField { field: Box::from(key) }
                            },
                        );
                        None
                    }
                };
                entries.push(Entry { key: Box::from(key), value: filled });
            }
            (None, None) => unreachable!("union key"),
        }
    }
    Value::Map(entries.into_boxed_slice())
}
fn written(value: &revision::Value) -> Composed {
    let kind = match &value.kind {
        revision::Kind::Bool(value) => Composite::Bool(*value),
        revision::Kind::Int(value) => Composite::Int(*value),
        revision::Kind::String(value) => Composite::String(value.clone()),
        revision::Kind::List(values) => Composite::List(values.iter().map(written).collect()),
        revision::Kind::Map(pairs) => Composite::Map(
            pairs
                .iter()
                .map(|pair| Pair {
                    key: pair.key.clone(),
                    from: origin(Layer::Defaults, 0, pair.at),
                    value: written(&pair.value),
                })
                .collect(),
        ),
        revision::Kind::Null => panic!("generator never writes null defaults"),
    };
    Composed { from: origin(Layer::Defaults, 0, value.at), kind }
}
fn checked(
    declaration: &Declaration,
    value: &Composed,
    ty: u32,
    spec: Option<&Spec>,
    faults: &mut Vec<Fault>,
) -> Value {
    let form = &declaration.types.get(usize::try_from(ty).expect("small")).expect("type").form;
    let expected = match form {
        Form::Bool => TypeName::Bool,
        Form::Int => TypeName::Int,
        Form::String => TypeName::String,
        Form::Path => TypeName::Path,
        Form::Mode => TypeName::Mode,
        Form::Duration => TypeName::Duration,
        Form::Secret => TypeName::Secret,
        Form::Target => TypeName::Target,
        Form::File => TypeName::File,
        Form::Enum(_) => TypeName::Enum,
        Form::List(_) => TypeName::List,
        Form::Map(_) => TypeName::Map,
        Form::Object(_) => TypeName::Object,
    };
    let fits = matches!(
        (form, &value.kind),
        (Form::Bool, Composite::Bool(_))
            | (Form::Int, Composite::Int(_))
            | (
                Form::String
                    | Form::Path
                    | Form::Mode
                    | Form::Duration
                    | Form::Secret
                    | Form::Target
                    | Form::File
                    | Form::Enum(_),
                Composite::String(_)
            )
            | (Form::List(_), Composite::List(_))
            | (Form::Map(_) | Form::Object(_), Composite::Map(_))
    );
    if !fits {
        issue(faults, value, Schema::Wrong { expected });
        return Value::Bool(false);
    }
    if let Some(spec) = spec {
        if let Composite::Int(number) = &value.kind {
            if let Some(min) = &spec.min
                && *number < min.value
            {
                issue(faults, value, Schema::Below { min: min.value });
                return Value::Bool(false);
            }
            if let Some(max) = &spec.max
                && *number > max.value
            {
                issue(faults, value, Schema::Above { max: max.value });
                return Value::Bool(false);
            }
        }
        let length = match &value.kind {
            Composite::String(v) => v.len(),
            Composite::List(v) => v.len(),
            Composite::Map(v) => v.len(),
            Composite::Bool(_) | Composite::Int(_) => 0,
        };
        if let Some(max) = &spec.max_length
            && i64::try_from(length).expect("small") > max.value
        {
            issue(faults, value, Schema::Length { max_length: max.value });
            return Value::Bool(false);
        }
    }
    match (&value.kind, form) {
        (Composite::Bool(v), _) => Value::Bool(*v),
        (Composite::Int(v), _) => Value::Int(*v),
        (Composite::String(v), Form::Enum(listed)) => {
            if !listed.iter().any(|name| name.text == *v) {
                issue(
                    faults,
                    value,
                    Schema::Enum { value: v.clone(), listed: listed.iter().map(|name| name.text.clone()).collect() },
                );
            }
            Value::String(v.clone())
        }
        (Composite::String(v), form) => checked_string(form, value, v, faults),
        (Composite::List(values), Form::List(ty)) => {
            Value::List(values.iter().map(|value| checked(declaration, value, *ty, None, faults)).collect())
        }
        (Composite::Map(pairs), Form::Map(ty)) => Value::Map(
            pairs
                .iter()
                .map(|pair| Entry {
                    key: pair.key.clone(),
                    value: Some(checked(declaration, &pair.value, *ty, None, faults)),
                })
                .collect(),
        ),
        (Composite::Map(_), Form::Object(fields)) => object(declaration, value, fields, faults, false),
        _ => unreachable!("type checked"),
    }
}
#[must_use]
pub const fn kind(schema: &Schema) -> &'static str {
    match schema {
        Schema::Export { .. } => "export",
        Schema::Wrong { .. } => "wrong",
        Schema::Enum { .. } => "enum",
        Schema::Below { .. } => "below",
        Schema::Above { .. } => "above",
        Schema::Length { .. } => "length",
        Schema::UndeclaredField { .. } | Schema::Undeclared { .. } => "undeclared",
        Schema::MissingField { .. } | Schema::Missing { .. } => "missing",
        Schema::Absent { .. } => "absent",
        Schema::Null => "null",
        Schema::Task { .. } => "task",
        Schema::Unread { .. } => "unread",
        Schema::Malformed { .. } => "malformed",
        Schema::Target { .. } => "target",
        Schema::File { .. } => "file",
        Schema::Large { .. } => "large",
        Schema::Unlisted { reader: _, root: _, task: _, setting: _ } => "unlisted",
        Schema::Dynamic { root: _ } => "dynamic",
        Schema::Component { package: _, component: _ } => "component",
        Schema::Main => "main",
        Schema::Security => "security",
        Schema::Source { .. } => "source",
        Schema::Beneath { .. } => "beneath",
    }
}

fn reference_format(form: &Form) -> Option<hearth_domain_render::errors::Format> {
    use hearth_domain_render::errors::Format;
    match form {
        Form::Path => Some(Format::Path),
        Form::Mode => Some(Format::Mode),
        Form::Duration => Some(Format::Duration),
        Form::Secret => Some(Format::Secret),
        Form::File => Some(Format::File),
        Form::Bool
        | Form::Int
        | Form::String
        | Form::Target
        | Form::Enum(_)
        | Form::List(_)
        | Form::Map(_)
        | Form::Object(_) => None,
    }
}
fn valid_form(format: hearth_domain_render::errors::Format, value: &[u8]) -> bool {
    use hearth_domain_render::errors::Format;
    let string = std::str::from_utf8(value).expect("generated UTF-8");
    let segments = |s: &str| s.split('/').all(|s| !["", ".", ".."].contains(&s));
    match format {
        Format::Path => string == "/" || string.strip_prefix('/').is_some_and(segments),
        Format::Mode => (3..=4).contains(&string.len()) && string.chars().all(|c| ('0'..='7').contains(&c)),
        Format::Duration => duration(value).is_some(),
        Format::Secret => {
            string.len() <= 255
                && segments(string)
                && string.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-/".contains(c))
        }
        Format::File => segments(string),
        Format::Link
        | Format::Key
        | Format::Timestamp
        | Format::Debian
        | Format::Preseed
        | Format::Module
        | Format::Param => {
            unreachable!("only generated forms")
        }
    }
}
fn duration(value: &[u8]) -> Option<i64> {
    let string = std::str::from_utf8(value).ok()?;
    let (number, unit) = string.split_at_checked(string.len().checked_sub(1)?)?;
    if number.starts_with('0') || number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let multiplier = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        _ => return None,
    };
    number.parse::<i64>().ok()?.checked_mul(multiplier)
}

fn checked_string(form: &Form, value: &Composed, v: &[u8], faults: &mut Vec<Fault>) -> Value {
    if let Some(format) = reference_format(form)
        && !valid_form(format, v)
    {
        issue(faults, value, Schema::Malformed { format });
    }
    if matches!(form, Form::File) && v != b"a" {
        issue(faults, value, Schema::File { path: Box::from(v) });
    }
    if matches!(form, Form::Duration) && valid_form(hearth_domain_render::errors::Format::Duration, v) {
        Value::Int(duration(v).expect("independently checked duration"))
    } else {
        Value::String(Box::from(v))
    }
}
