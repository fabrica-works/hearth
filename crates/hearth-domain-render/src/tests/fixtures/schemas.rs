use super::{
    composition::{at, map, string},
    declaration, declarations, declared_spec,
};
use crate::{
    compose::{Composed, Layer, Merge},
    evaluate::Value,
    limits,
    revision::{
        self,
        declaration::{Declaration, Form, Number, Spec, Type},
    },
    schema::{self, Against, Check, Defaults, Fault, Validator},
};
use alloc::boxed::Box;

pub fn schema_spec(name: &[u8], ty: u32) -> Spec {
    Spec { at: at(1, ty.saturating_add(1)), ty, ..declared_spec(name) }
}
pub fn users() -> Declaration {
    let groups = Spec {
        default: Some(revision::Value { at: at(1, 20), kind: revision::Kind::List(Box::new([])) }),
        ..schema_spec(b"groups", 2)
    };
    let password = Spec { optional: true, ..schema_spec(b"password", 4) };
    let shell = Spec { default: Some(string(at(1, 30), b"/bin/bash")), ..schema_spec(b"shell", 5) };
    let uid = Spec {
        min: Some(Number { at: at(1, 40), value: super::MIN_UID }),
        max: Some(Number { at: at(1, 50), value: super::MAX_UID }),
        ..schema_spec(b"uid", 6)
    };
    Declaration {
        at: at(1, 1),
        types: Box::new([
            Type { at: at(1, 1), form: Form::Map(1) },
            Type { at: at(1, 2), form: Form::Object(Box::new([groups, password, shell, uid])) },
            Type { at: at(1, 3), form: Form::List(3) },
            Type { at: at(1, 4), form: Form::String },
            Type { at: at(1, 5), form: Form::Secret },
            Type { at: at(1, 6), form: Form::Path },
            Type { at: at(1, 7), form: Form::Int },
        ]),
        settings: Box::new([Spec { default: Some(map(at(1, 60), &[])), ..schema_spec(b"accounts", 0) }]),
        requires: Box::new([]),
        exports: Box::new([]),
        entries: Box::new([]),
        reads: Box::new([]),
    }
}
pub fn typed_declaration(form: Form) -> Declaration {
    let mut declaration = declaration(&[schema_spec(b"x", 0)], &[]);
    declaration.types = Box::new([Type { at: at(1, 1), form }]);
    declaration
}
pub fn composed(value: &revision::Value) -> Composed {
    Merge::new(&limits::SHIPPED).merge(None, value, Layer::Base, 2).expect("non-null fixture")
}
type DeclarationFaults = Box<[(Box<[u8]>, Fault)]>;
pub fn defaults(declaration: &Declaration) -> (Defaults, DeclarationFaults) {
    defaults_with_listing(declaration, &[])
}
pub fn defaults_with_listing(declaration: &Declaration, listing: &[revision::Entry]) -> (Defaults, DeclarationFaults) {
    let documents = declarations(&[(b"task", Some(declaration.clone()))]);
    let mut faults = skein_lib::List::with_capacity(16);
    let defaults = schema::declarations(
        &documents,
        listing,
        &mut Validator::new(&limits::SHIPPED),
        &mut Merge::new(&limits::SHIPPED),
        &mut faults,
    );
    (defaults, faults.into_boxed())
}
pub fn validate(declaration: &Declaration, value: &Composed, check: &Check<'_>) -> Result<Value, Box<[Fault]>> {
    validate_with(declaration, value, check, &[], &limits::SHIPPED)
}
pub fn validate_with(
    declaration: &Declaration,
    value: &Composed,
    check: &Check<'_>,
    listing: &[revision::Entry],
    limits: &limits::Limits,
) -> Result<Value, Box<[Fault]>> {
    let (defaults, errors) = defaults_with_listing(declaration, listing);
    assert_eq!(errors.as_ref(), &[], "fixture defaults are sound");
    let against = Against { declaration, defaults: defaults.of(b"task").expect("sound fixture"), listing };
    let mut faults = skein_lib::List::with_capacity(16);
    let mut left = u64::MAX;
    match Validator::new(limits).settings(value, &against, check, &mut left, &mut faults) {
        Some(value) => Ok(value),
        None => Err(faults.into_boxed()),
    }
}
pub fn schema_form(name: crate::errors::TypeName) -> Form {
    use crate::errors::TypeName;
    match name {
        TypeName::Bool => Form::Bool,
        TypeName::Int => Form::Int,
        TypeName::String => Form::String,
        TypeName::Path => Form::Path,
        TypeName::Mode => Form::Mode,
        TypeName::Duration => Form::Duration,
        TypeName::Secret => Form::Secret,
        TypeName::Target => Form::Target,
        TypeName::File => Form::File,
        TypeName::Enum => {
            Form::Enum(Box::new([revision::declaration::Text { at: at(1, 1), text: skein_lib::bytes::copy_of(b"a") }]))
        }
        TypeName::List => Form::List(0),
        TypeName::Map => Form::Map(0),
        TypeName::Object => Form::Object(Box::new([])),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LengthInput {
    Bytes,
    Elements,
    Entries,
}
pub fn length_input(input: LengthInput) -> (Declaration, revision::Value) {
    let (form, kind) = match input {
        LengthInput::Bytes => (Form::String, revision::Kind::String(skein_lib::bytes::copy_of("éé".as_bytes()))),
        LengthInput::Elements => (
            Form::List(1),
            revision::Kind::List(Box::new([
                super::composition::int(at(2, 7), 1),
                super::composition::int(at(2, 7), 2),
                super::composition::int(at(2, 7), 3),
            ])),
        ),
        LengthInput::Entries => (
            Form::Map(1),
            map(
                at(2, 7),
                &[
                    (b"a", at(2, 8), super::composition::int(at(2, 12), 1)),
                    (b"b", at(2, 16), super::composition::int(at(2, 20), 2)),
                    (b"c", at(2, 24), super::composition::int(at(2, 28), 3)),
                ],
            )
            .kind,
        ),
    };
    let mut declaration = typed_declaration(Form::Int);
    declaration.types = Box::new([Type { at: at(1, 1), form }, Type { at: at(1, 2), form: Form::Int }]);
    declaration.settings =
        Box::new([Spec { max_length: Some(Number { at: at(1, 1), value: 2 }), ..schema_spec(b"x", 0) }]);
    (declaration, revision::Value { at: at(2, 7), kind })
}

pub struct TargetSchema {
    pub documents: revision::Documents,
    pub defaults: Defaults,
    pub settings: crate::compose::Settings,
}
pub fn target_schema(tasks: &[(&[u8], Declaration)], values: &[(&[u8], revision::Value)]) -> TargetSchema {
    let mut declared = skein_lib::List::with_capacity(u32::try_from(tasks.len()).expect("task count"));
    for (name, declaration) in tasks {
        declared.push((*name, Some(declaration.clone()))).expect("task count");
    }
    let documents = declarations(declared.as_slice());
    let mut faults = skein_lib::List::with_capacity(16);
    let defaults = schema::declarations(
        &documents,
        documents.entries(),
        &mut Validator::new(&limits::SHIPPED),
        &mut Merge::new(&limits::SHIPPED),
        &mut faults,
    );
    assert_eq!(faults.as_slice(), &[], "sound fixture");
    let mut present = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("task count"));
    for (name, value) in values {
        let settings = composed(value);
        let from = crate::compose::Provenance { at: at(2, 2), ..settings.from };
        present
            .push(crate::compose::Present { name: skein_lib::bytes::copy_of(name), from, settings })
            .expect("task count");
    }
    TargetSchema { documents, defaults, settings: crate::compose::Settings { tasks: present.into_boxed() } }
}
pub fn target_roots(
    input: &TargetSchema,
    limits: &limits::Limits,
) -> (Option<schema::Validated>, DeclarationFaults, bool) {
    let revision = schema::Revision { documents: &input.documents, defaults: &input.defaults, listing: &[] };
    let check = Check::Target { environment: b"prod", targets: &[] };
    let mut validator = Validator::new(limits);
    let mut faults = skein_lib::List::with_capacity(16);
    let mut left = limits.validated;
    let value = schema::validate(&input.settings, &revision, &check, &mut validator, &mut left, &mut faults);
    (value, faults.into_boxed(), validator.limited())
}
pub struct Layers {
    pub fleet: revision::Value,
    pub base: revision::Value,
    pub environment: revision::Value,
    pub overlay: revision::Value,
}
pub fn layers(declaration: &Declaration, layers: Layers) -> (crate::compose::Target, revision::Documents, Defaults) {
    use crate::revision::{Document, Entry, Layer, Mode, Oid, Task};
    use skein_lib::bytes::copy_of;
    const FILES: &[(&[u8], &[u8])] = &[
        (b"fleet.json", b"fleet"),
        (b"hosts/x1/x1.json", b"base"),
        (b"envs/prod.json", b"env"),
        (b"hosts/x1/x1-prod.json", b"overlay"),
    ];
    let mut entries = skein_lib::List::with_capacity(4);
    for (path, oid) in FILES {
        entries
            .push(Entry { path: copy_of(path), mode: Mode::File, oid: Oid(copy_of(oid)), size: 2 })
            .expect("four files");
    }
    let mut documents = revision::Documents::new(entries.as_slice());
    let Layers { fleet, base, environment, overlay } = layers;
    for ((_, oid), value) in FILES.iter().zip([fleet, base, environment, overlay]) {
        documents.set(
            &Oid(copy_of(oid)),
            Document {
                members: None,
                at: at(2, 1),
                layer: Layer {
                    debian: None,
                    placement: None,
                    tasks: Box::new([Task { name: copy_of(b"task"), at: at(2, 2), value }]),
                },
            },
        );
    }
    let target = crate::compose::Target {
        name: copy_of(b"x1-prod"),
        base: copy_of(b"x1"),
        environment: copy_of(b"prod"),
        member: None,
        fleet: documents.find(b"fleet.json"),
        base_document: documents.find(b"hosts/x1/x1.json"),
        environment_document: documents.find(b"envs/prod.json").expect("listed"),
        base_env: documents.find(b"hosts/x1/x1-prod.json").expect("listed"),
    };
    let (defaults, faults) = defaults(declaration);
    assert_eq!(faults.as_ref(), &[]);
    (target, documents, defaults)
}

pub fn nulled_accounts() -> (Declaration, Layers) {
    use super::composition::{int, null};
    let mut declaration = users();
    declaration.settings = Box::new([Spec {
        default: Some(map(
            at(1, 20),
            &[(b"ann", at(1, 21), map(at(1, 28), &[(b"uid", at(1, 29), int(at(1, 35), super::MIN_UID))]))],
        )),
        ..schema_spec(b"accounts", 0)
    }]);
    let patch = map(
        at(2, 1),
        &[(
            b"accounts",
            at(2, 2),
            map(
                at(2, 14),
                &[
                    (b"ann", at(2, 15), null(at(2, 22))),
                    (
                        b"bob",
                        at(2, 28),
                        map(
                            at(2, 35),
                            &[(b"shell", at(2, 36), null(at(2, 45))), (b"uid", at(2, 51), int(at(2, 58), 1001))],
                        ),
                    ),
                ],
            ),
        )],
    );
    (
        declaration,
        Layers { fleet: map(at(2, 1), &[]), base: patch, environment: map(at(3, 1), &[]), overlay: map(at(4, 1), &[]) },
    )
}
