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
        min: Some(Number { at: at(1, 40), value: 1000 }),
        max: Some(Number { at: at(1, 50), value: 59999 }),
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
    let documents = declarations(&[(b"task", Some(declaration.clone()))]);
    let mut faults = skein_lib::List::with_capacity(16);
    let defaults = schema::declarations(
        &documents,
        &mut Validator::new(&limits::SHIPPED),
        &mut Merge::new(&limits::SHIPPED),
        &mut faults,
    );
    (defaults, faults.into_boxed())
}
pub fn validate(declaration: &Declaration, value: &Composed, check: &Check<'_>) -> Result<Value, Box<[Fault]>> {
    let (defaults, errors) = defaults(declaration);
    assert_eq!(errors.as_ref(), &[], "fixture defaults are sound");
    let against = Against { declaration, defaults: defaults.of(b"task").expect("sound fixture"), listing: &[] };
    let mut faults = skein_lib::List::with_capacity(16);
    let mut left = u64::MAX;
    match Validator::new(&limits::SHIPPED).settings(value, &against, check, &mut left, &mut faults) {
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
