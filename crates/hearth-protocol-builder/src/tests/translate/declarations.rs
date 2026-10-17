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
    let [map, object, int, string] = declaration.types.as_ref() else { panic!("four types") };
    assert_eq!((&map.form, map.at), (&target::Form::Map(1), render::revision::Position { line: 2, column: 11 }));
    assert_eq!((&int.form, int.at), (&target::Form::Int, render::revision::Position { line: 4, column: 23 }));
    assert_eq!((&string.form, string.at), (&target::Form::String, render::revision::Position { line: 3, column: 23 }));
    let target::Form::Object(fields) = &object.form else { panic!("object fields") };
    let [order, param] = fields.as_ref() else { panic!("two fields") };
    assert_eq!(
        (&*order.name.text, order.ty, &*param.name.text, param.ty),
        (b"order".as_slice(), 2, b"param".as_slice(), 3)
    );
}
#[test]
fn a_declaration_s_bounds_keep_their_numbers_and_positions() {
    let declaration = fixtures::read_declaration(fixtures::PARAMS);
    let [_, object, _, _] = declaration.types.as_ref() else { panic!("four types") };
    let target::Form::Object(fields) = &object.form else { panic!("object fields") };
    let [order, _] = fields.as_ref() else { panic!("two fields") };
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
    let [_, object, _, _] = declaration.types.as_ref() else { panic!("four types") };
    let target::Form::Object(fields) = &object.form else { panic!("object fields") };
    let [order, _] = fields.as_ref() else { panic!("two fields") };
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
