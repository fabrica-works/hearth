use crate::tests::fixtures;
use crate::{
    config::{self, schema as source},
    render::{self, revision::declaration as target},
    translate::declarations,
};
use skein_lib::bytes::copy_of;

fn position_back(at: render::revision::Position) -> config::Position {
    config::Position { offset: 0, line: at.line, column: at.column }
}
fn text_back(text: target::Text) -> config::declaration::Text {
    let target::Text { at, text } = text;
    config::declaration::Text { at: position_back(at), text }
}
fn form_back(form: target::Form) -> source::Form {
    match form {
        target::Form::Bool => source::Form::Bool,
        target::Form::Int => source::Form::Int,
        target::Form::String => source::Form::String,
        target::Form::Path => source::Form::Path,
        target::Form::Mode => source::Form::Mode,
        target::Form::Duration => source::Form::Duration,
        target::Form::Secret => source::Form::Secret,
        target::Form::Target => source::Form::Target,
        target::Form::File => source::Form::File,
        target::Form::Enum(values) => {
            let mut texts = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("fixture count"));
            for value in values {
                texts.push(text_back(value)).expect("room");
            }
            source::Form::Enum(texts.into_boxed())
        }
        target::Form::List(ty) => source::Form::List(ty),
        target::Form::Map(ty) => source::Form::Map(ty),
        target::Form::Object(specs) => {
            let mut fields = skein_lib::List::with_capacity(u32::try_from(specs.len()).expect("fixture count"));
            for spec in specs {
                fields.push(spec_back(spec)).expect("room");
            }
            source::Form::Object(fields.into_boxed())
        }
    }
}
fn number_back(number: Option<target::Number>) -> Option<config::declaration::Number> {
    let target::Number { at, value } = number?;
    Some(config::declaration::Number { at: position_back(at), value })
}
fn spec_back(spec: target::Spec) -> source::Spec {
    let target::Spec { name, at, ty, default, optional, min, max, max_length } = spec;
    assert_eq!(default, None, "form fixture has no defaults");
    source::Spec {
        name: text_back(name),
        at: position_back(at),
        ty,
        default: None,
        optional,
        min: number_back(min),
        max: number_back(max),
        max_length: number_back(max_length),
    }
}
#[test]
fn each_form_translates_to_render_and_back() {
    for (row, input) in fixtures::every_form().into_iter().enumerate() {
        let back = form_back(declarations::form(input.clone(), &crate::limits::SHIPPED));
        assert_eq!(back, input, "row {row}: form");
    }
}
fn schema_back(schema: crate::records::errors::Schema) -> render::errors::Schema {
    match schema {
        crate::records::errors::Schema::Unlisted { reader, root, task, setting } => {
            render::errors::Schema::Unlisted { reader, root: reference_root_back(root), task, setting }
        }
        crate::records::errors::Schema::Dynamic { root } => {
            render::errors::Schema::Dynamic { root: reference_root_back(root) }
        }
        crate::records::errors::Schema::Malformed { format } => {
            render::errors::Schema::Malformed { format: super::forms::format_back(format) }
        }
        crate::records::errors::Schema::Target { name, environment } => {
            render::errors::Schema::Target { name, environment }
        }
        crate::records::errors::Schema::File { path } => render::errors::Schema::File { path },
        crate::records::errors::Schema::Large { path, max } => render::errors::Schema::Large { path, max },
        crate::records::errors::Schema::Main => render::errors::Schema::Main,
        crate::records::errors::Schema::Security => render::errors::Schema::Security,
        crate::records::errors::Schema::Undeclared { setting, declared } => {
            render::errors::Schema::Undeclared { setting, declared }
        }
        crate::records::errors::Schema::Missing { setting } => render::errors::Schema::Missing { setting },
        crate::records::errors::Schema::Absent { task } => render::errors::Schema::Absent { task },
        crate::records::errors::Schema::Wrong { expected } => {
            render::errors::Schema::Wrong { expected: type_name_back(expected) }
        }
        crate::records::errors::Schema::Enum { value, listed } => render::errors::Schema::Enum { value, listed },
        crate::records::errors::Schema::Below { min } => render::errors::Schema::Below { min },
        crate::records::errors::Schema::Above { max } => render::errors::Schema::Above { max },
        crate::records::errors::Schema::Length { max_length } => render::errors::Schema::Length { max_length },
        crate::records::errors::Schema::Null => render::errors::Schema::Null,
        crate::records::errors::Schema::UndeclaredField { field, declared } => {
            render::errors::Schema::UndeclaredField { field, declared }
        }
        crate::records::errors::Schema::MissingField { field } => render::errors::Schema::MissingField { field },
        crate::records::errors::Schema::Task { task } => render::errors::Schema::Task { task },
        crate::records::errors::Schema::Unread { task, setting, declared } => {
            render::errors::Schema::Unread { task, setting, declared }
        }
    }
}
#[test]
fn each_schema_fault_translates_and_back() {
    for (row, input) in fixtures::schema_faults().into_iter().enumerate() {
        let back = schema_back(crate::translate::errors::schema(input.clone()));
        assert_eq!(back, input, "row {row}: schema fault");
    }
}
#[test]
fn a_declaration_s_types_settings_and_reads_cross_unchanged() {
    let declaration = fixtures::read_declaration(fixtures::PARAMS);
    let setting = fixtures::only(&declaration.settings);
    assert_eq!(
        (declaration.types.len(), &*setting.name.text, setting.ty, declaration.reads.as_ref()),
        (4, b"params".as_slice(), 0, &[][..])
    );
}
#[test]
fn a_declaration_s_nested_types_keep_their_forms_positions_and_indexes() {
    let declaration = fixtures::read_declaration(fixtures::PARAMS);
    let types = fixtures::parameter_types(&declaration);
    assert_eq!(
        (&types.map.form, types.map.at),
        (&target::Form::Map(1), render::revision::Position { line: 2, column: 11 })
    );
    assert_eq!(
        (&types.int.form, types.int.at),
        (&target::Form::Int, render::revision::Position { line: 4, column: 23 })
    );
    assert_eq!(
        (&types.string.form, types.string.at),
        (&target::Form::String, render::revision::Position { line: 3, column: 23 })
    );
    assert_eq!(
        (&*types.order.name.text, types.order.ty, &*types.param.name.text, types.param.ty),
        (b"order".as_slice(), 2, b"param".as_slice(), 3)
    );
}
#[test]
fn a_declaration_s_bounds_keep_their_numbers_and_positions() {
    let declaration = fixtures::read_declaration(fixtures::PARAMS);
    let types = fixtures::parameter_types(&declaration);
    let order = types.order;
    assert_eq!(
        (&order.min, &order.max),
        (
            &Some(target::Number { at: render::revision::Position { line: 4, column: 37 }, value: 0 }),
            &Some(target::Number { at: render::revision::Position { line: 4, column: 47 }, value: 999 })
        )
    );
}
#[test]
fn a_declaration_s_defaults_keep_their_values_and_positions() {
    let declaration = fixtures::read_declaration(fixtures::PARAMS);
    let types = fixtures::parameter_types(&declaration);
    let order = types.order;
    assert_eq!(
        &order.default,
        &Some(render::revision::Value {
            at: render::revision::Position { line: 4, column: 63 },
            kind: render::revision::Kind::Int(500)
        })
    );
    let setting = fixtures::only(&declaration.settings);
    assert_eq!(
        &setting.default,
        &Some(render::revision::Value {
            at: render::revision::Position { line: 5, column: 14 },
            kind: render::revision::Kind::Map(Box::new([]))
        })
    );
}
#[test]
fn a_declaration_s_reads_keep_their_positions_settings_and_flags() {
    let declaration = fixtures::read_declaration(fixtures::READS);
    assert_eq!(
        declaration.reads.as_ref(),
        &[target::Read {
            task: target::Text { at: render::revision::Position { line: 1, column: 11 }, text: copy_of(b"core") },
            at: render::revision::Position { line: 1, column: 18 },
            settings: Box::new([target::Text {
                at: render::revision::Position { line: 1, column: 31 },
                text: copy_of(b"domain")
            }]),
            exports: true,
            optional: true
        }]
    );
}

fn type_name_back(name: crate::records::errors::TypeName) -> render::errors::TypeName {
    match name {
        crate::records::errors::TypeName::Bool => render::errors::TypeName::Bool,
        crate::records::errors::TypeName::Int => render::errors::TypeName::Int,
        crate::records::errors::TypeName::String => render::errors::TypeName::String,
        crate::records::errors::TypeName::Enum => render::errors::TypeName::Enum,
        crate::records::errors::TypeName::List => render::errors::TypeName::List,
        crate::records::errors::TypeName::Map => render::errors::TypeName::Map,
        crate::records::errors::TypeName::Object => render::errors::TypeName::Object,
        crate::records::errors::TypeName::Path => render::errors::TypeName::Path,
        crate::records::errors::TypeName::Mode => render::errors::TypeName::Mode,
        crate::records::errors::TypeName::Duration => render::errors::TypeName::Duration,
        crate::records::errors::TypeName::Secret => render::errors::TypeName::Secret,
        crate::records::errors::TypeName::Target => render::errors::TypeName::Target,
        crate::records::errors::TypeName::File => render::errors::TypeName::File,
    }
}

const TYPE_NAMES: &[render::errors::TypeName] = &[
    render::errors::TypeName::Bool,
    render::errors::TypeName::Int,
    render::errors::TypeName::String,
    render::errors::TypeName::Enum,
    render::errors::TypeName::List,
    render::errors::TypeName::Map,
    render::errors::TypeName::Object,
    render::errors::TypeName::Path,
    render::errors::TypeName::Mode,
    render::errors::TypeName::Duration,
    render::errors::TypeName::Secret,
    render::errors::TypeName::Target,
    render::errors::TypeName::File,
];
#[test]
fn each_schema_type_name_translates_and_back() {
    for (row, input) in TYPE_NAMES.iter().enumerate() {
        let back = type_name_back(crate::translate::errors::type_name(input.clone()));
        assert_eq!(back, *input, "row {row}: type name");
    }
}

#[test]
fn a_declarations_requires_cross_to_render_and_back() {
    for (row, (guard, input)) in fixtures::guard_conditions().into_iter().enumerate() {
        let guard = declarations::guard(guard, &crate::limits::SHIPPED.template).expect("guard compiles");
        let back = super::back::expression(guard.when.expression);
        assert_eq!(back, input, "row {row}: guard condition");
    }
}

fn reference_root_back(root: crate::records::errors::Root) -> render::errors::Root {
    match root {
        crate::records::errors::Root::Tasks => render::errors::Root::Tasks,
        crate::records::errors::Root::Exports => render::errors::Root::Exports,
    }
}
