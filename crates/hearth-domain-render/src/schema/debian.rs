//! The fixed Debian schema (format.md, 5.3); composition supplies its bottom layer.
use super::{
    Against, Box, Check, Composed, Composite, Declaration, Evaluated, Fault, Form, Layer, List, Merge, Position,
    Provenance, Schema, Spec, Validator, Wanted, bytes, fault, forms, revision, task_defaults,
};
use crate::{
    errors::Format,
    revision::declaration::{Text, Type},
};
const AT: Position = Position { line: 1, column: 1 };
const COMPONENTS: &[&[u8]] = &[b"contrib", b"main", b"non-free", b"non-free-firmware"];
/// Check a composed Debian object. Its components have already been merged
/// over `{"main": true}`, so a deleted main is distinguishable from omission.
pub fn debian(debian: Option<&Composed>, document: u32, faults: &mut List<Fault>) -> Option<Evaluated> {
    let Some(debian) = debian else {
        fault(
            faults,
            Provenance { layer: Layer::Base, document, at: AT },
            Schema::Missing { setting: bytes::copy_of(b"debian") },
        );
        return None;
    };
    let before = faults.len();
    let mut input = debian.clone();
    let security = match member(member(Some(&input), b"pockets"), b"security") {
        Some(Composed { kind: Composite::Bool(value), .. }) => *value,
        Some(Composed {
            kind: Composite::Int(_) | Composite::String(_) | Composite::List(_) | Composite::Map(_),
            ..
        })
        | None => true,
    };
    if !security {
        strip(&mut input, b"security");
    }
    let declaration = declaration();
    let mut validator = Validator::new(&crate::limits::SHIPPED);
    let defaults =
        task_defaults(&declaration, document, &[], &mut validator, &mut Merge::new(&crate::limits::SHIPPED), faults);
    let against = Against { declaration: &declaration, defaults: &defaults, listing: &[] };
    let value = validator.walk(input.clone(), Wanted::Type(0), &against, &Check::Default, faults);
    names(&input, faults);
    if let Some(components) = member(Some(&input), b"components")
        && let Composite::Map(pairs) = &components.kind
    {
        match member(Some(components), b"main") {
            Some(Composed { kind: Composite::Bool(false), from }) => fault(faults, *from, Schema::Main),
            Some(Composed {
                kind:
                    Composite::Bool(true)
                    | Composite::Int(_)
                    | Composite::String(_)
                    | Composite::List(_)
                    | Composite::Map(_),
                ..
            }) => {}
            None => fault(faults, pair(Some(&input), b"components").expect("present components").from, Schema::Main),
        }
        for pair in pairs {
            if !COMPONENTS.contains(&pair.key.as_ref()) {
                fault(faults, pair.from, Schema::Enum { value: pair.key.clone(), listed: component_names() });
            }
        }
    }
    let snapshot = member(Some(&input), b"snapshot");
    if security
        && let Some(snapshot) = snapshot
        && let Composite::Map(_) = snapshot.kind
        && member(Some(snapshot), b"security").is_none()
    {
        fault(faults, snapshot.from, Schema::Security);
    }
    if faults.len() == before { value } else { None }
}
fn member<'a>(value: Option<&'a Composed>, name: &[u8]) -> Option<&'a Composed> {
    Some(&pair(value, name)?.value)
}
fn pair<'a>(value: Option<&'a Composed>, name: &[u8]) -> Option<&'a crate::compose::Pair> {
    let Composite::Map(pairs) = &value?.kind else {
        return None;
    };
    let mut found = None;
    for pair in pairs {
        if pair.key.as_ref() == name {
            found = Some(pair);
            break;
        }
    }
    found
}
fn names(value: &Composed, faults: &mut List<Fault>) {
    for name in [b"suite".as_slice(), b"architecture"] {
        if let Some(value) = member(Some(value), name)
            && let Composite::String(bytes) = &value.kind
            && !forms::check(Format::Debian, bytes)
        {
            fault(faults, value.from, Schema::Malformed { format: Format::Debian });
        }
    }
    for name in [b"archive".as_slice(), b"security"] {
        if let Some(value) = member(member(Some(value), b"snapshot"), name)
            && let Composite::String(bytes) = &value.kind
            && !forms::check(Format::Timestamp, bytes)
        {
            fault(faults, value.from, Schema::Malformed { format: Format::Timestamp });
        }
    }
}
fn strip(value: &mut Composed, name: &[u8]) {
    let Composite::Map(pairs) = &mut value.kind else {
        return;
    };
    for pair in pairs {
        if pair.key.as_ref() == b"snapshot"
            && let Composite::Map(fields) = &pair.value.kind
        {
            let mut kept = List::with_capacity(u32::try_from(fields.len()).expect("bounded fields"));
            for field in fields {
                if field.key.as_ref() != name {
                    kept.push(field.clone()).expect("field count");
                }
            }
            pair.value.kind = Composite::Map(kept.into_boxed());
        }
    }
}
fn component_names() -> Box<[Box<[u8]>]> {
    let mut names = List::with_capacity(4);
    for name in COMPONENTS {
        names.push(bytes::copy_of(name)).expect("four components");
    }
    names.into_boxed()
}
fn spec(name: &[u8], ty: u32, column: u32, default: Option<revision::Value>, optional: bool) -> Spec {
    Spec {
        name: Text { at: AT, text: bytes::copy_of(name) },
        at: Position { column, ..AT },
        ty,
        default,
        optional,
        min: None,
        max: None,
        max_length: None,
    }
}
fn written(kind: revision::Kind) -> revision::Value {
    revision::Value { at: AT, kind }
}
fn declaration() -> Declaration {
    let bool_default = Some(written(revision::Kind::Bool(true)));
    let fields = Box::new([
        spec(b"architecture", 1, 1, None, false),
        spec(
            b"components",
            2,
            2,
            Some(written(revision::Kind::Map(Box::new([revision::Pair {
                key: bytes::copy_of(b"main"),
                at: AT,
                value: revision::Value { at: AT, kind: revision::Kind::Bool(true) },
            }])))),
            false,
        ),
        spec(b"install_recommends", 3, 3, Some(written(revision::Kind::Bool(false))), false),
        spec(b"pockets", 4, 4, Some(written(revision::Kind::Map(Box::new([])))), false),
        spec(b"snapshot", 5, 5, None, false),
        spec(b"suite", 1, 6, None, false),
    ]);
    let types = Box::new([
        Type { at: AT, form: Form::Object(fields) },
        Type { at: AT, form: Form::String },
        Type { at: AT, form: Form::Map(3) },
        Type { at: AT, form: Form::Bool },
        Type {
            at: AT,
            form: Form::Object(Box::new([
                spec(b"security", 3, 7, bool_default.clone(), false),
                spec(b"updates", 3, 8, bool_default.clone(), false),
            ])),
        },
        Type {
            at: AT,
            form: Form::Object(Box::new([spec(b"archive", 1, 9, None, false), spec(b"security", 1, 10, None, true)])),
        },
    ]);
    Declaration {
        at: AT,
        types,
        settings: Box::new([]),
        requires: Box::new([]),
        entries: Box::new([]),
        reads: Box::new([]),
    }
}
