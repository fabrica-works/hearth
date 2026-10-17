use super::fixtures::{self, composition::at};
use crate::{errors::Schema, revision::declaration::Text, schema::Fault};
use skein_lib::bytes::copy_of;

#[test]
fn a_read_of_a_setting_its_task_lacks_is_refused_at_it() {
    let core = fixtures::declaration(&[fixtures::declared_spec(b"domain")], &[]);
    let motd =
        fixtures::declaration(&[], &[fixtures::read(b"core", &[Text { at: at(1, 34), text: copy_of(b"domian") }])]);
    let documents = fixtures::declarations(&[(b"core", Some(core)), (b"motd", Some(motd))]);
    assert_eq!(
        fixtures::check_declarations(&documents).as_ref(),
        &[(
            copy_of(b"motd"),
            Fault {
                schema: Schema::Unread {
                    task: copy_of(b"core"),
                    setting: copy_of(b"domian"),
                    declared: Box::new([copy_of(b"domain")])
                },
                document: 1,
                at: at(1, 34)
            }
        )],
        "format.md, 6.5: the reader's, at its setting"
    );
}
#[test]
fn a_read_of_a_task_tasks_lacks_is_refused_at_its_key() {
    let motd = fixtures::declaration(&[], &[fixtures::read(b"mail", &[])]);
    let documents = fixtures::declarations(&[(b"motd", Some(motd))]);
    assert_eq!(
        fixtures::check_declarations(&documents).as_ref(),
        &[(copy_of(b"motd"), Fault { schema: Schema::Task { task: copy_of(b"mail") }, document: 0, at: at(1, 12) })]
    );
}
const READ_FLAGS: &[(bool, bool)] = &[(true, false), (false, true)];
#[test]
fn an_optional_or_exporting_read_of_a_task_tasks_lacks_is_still_refused() {
    for (row, &(optional, exports)) in READ_FLAGS.iter().enumerate() {
        let read = revision::declaration::Read { exports, optional, ..fixtures::read(b"mail", &[]) };
        let documents = fixtures::declarations(&[(b"motd", Some(fixtures::declaration(&[], &[read])))]);
        assert_eq!(
            fixtures::check_declarations(&documents).as_ref(),
            &[(
                copy_of(b"motd"),
                Fault { schema: Schema::Task { task: copy_of(b"mail") }, document: 0, at: at(1, 12) }
            )],
            "row {row}"
        );
    }
}
#[test]
fn a_read_of_a_refused_task_is_not_checked() {
    let motd =
        fixtures::declaration(&[], &[fixtures::read(b"core", &[Text { at: at(1, 34), text: copy_of(b"domian") }])]);
    let documents = fixtures::declarations(&[(b"core", None), (b"motd", Some(motd))]);
    assert_eq!(fixtures::check_declarations(&documents).as_ref(), &[]);
}
#[test]
fn a_read_fault_makes_only_its_reading_task_unsound() {
    let motd = fixtures::declaration(&[], &[fixtures::read(b"mail", &[])]);
    let documents = fixtures::declarations(&[(b"core", Some(fixtures::declaration(&[], &[]))), (b"motd", Some(motd))]);
    let mut faults = skein_lib::List::with_capacity(4);
    let defaults = crate::schema::declarations(
        &documents,
        documents.entries(),
        &mut crate::schema::Validator::new(&crate::limits::SHIPPED),
        &mut crate::compose::Merge::new(&crate::limits::SHIPPED),
        &mut faults,
    );
    assert_eq!((defaults.sound(b"core"), defaults.sound(b"motd")), (true, false));
}
#[test]
fn sound_tasks_are_looked_up_in_name_order() {
    let documents = fixtures::declarations(&[
        (b"a_b", Some(fixtures::declaration(&[], &[]))),
        (b"a", Some(fixtures::declaration(&[], &[]))),
    ]);
    let mut faults = skein_lib::List::with_capacity(4);
    let defaults = crate::schema::declarations(
        &documents,
        documents.entries(),
        &mut crate::schema::Validator::new(&crate::limits::SHIPPED),
        &mut crate::compose::Merge::new(&crate::limits::SHIPPED),
        &mut faults,
    );
    assert_eq!((defaults.sound(b"a"), defaults.sound(b"a_b")), (true, true));
}

use super::fixtures::{
    composition::{int, map, string},
    schemas::{self, composed, schema_spec, typed_declaration, validate},
};
use crate::{
    errors::TypeName,
    evaluate::{Entry as E, Value as V},
    revision::{
        self,
        declaration::{Form, Number, Spec, Type},
    },
    schema::Check,
};
#[test]
fn an_account_gets_the_shell_its_object_type_defaults() {
    let settings = composed(&map(
        at(2, 1),
        &[(
            b"accounts",
            at(2, 2),
            map(at(2, 14), &[(b"free", at(2, 15), map(at(2, 23), &[(b"uid", at(2, 24), int(at(2, 31), 1000))]))]),
        )],
    ));
    let V::Map(settings) = validate(&schemas::users(), &settings, &Check::Default).expect("valid") else {
        panic!("settings")
    };
    let Some(V::Map(accounts)) = &fixtures::only(&settings).value else { panic!("accounts") };
    assert_eq!(
        &fixtures::only(accounts).value,
        &Some(V::Map(Box::new([
            E { key: copy_of(b"groups"), value: Some(V::List(Box::new([]))) },
            E { key: copy_of(b"password"), value: None },
            E { key: copy_of(b"shell"), value: Some(V::String(copy_of(b"/bin/bash"))) },
            E { key: copy_of(b"uid"), value: Some(V::Int(1000)) },
        ])))
    );
}
#[test]
fn an_object_element_of_a_list_gets_its_defaults() {
    let mut declaration = schemas::users();
    let [accounts, ..] = declaration.types.as_mut() else { panic!("types") };
    accounts.form = Form::List(1);
    let setting = declaration.settings.first_mut().expect("accounts");
    setting.default = Some(revision::Value { at: at(1, 60), kind: revision::Kind::List(Box::new([])) });
    let value = revision::Value {
        at: at(2, 14),
        kind: revision::Kind::List(Box::new([map(at(2, 15), &[(b"uid", at(2, 16), int(at(2, 23), 1000))])])),
    };
    let settings = composed(&map(at(2, 1), &[(b"accounts", at(2, 2), value)]));
    let V::Map(settings) = validate(&declaration, &settings, &Check::Default).expect("valid") else {
        panic!("settings")
    };
    assert_eq!(
        &fixtures::only(&settings).value,
        &Some(V::List(Box::new([V::Map(Box::new([
            E { key: copy_of(b"groups"), value: Some(V::List(Box::new([]))) },
            E { key: copy_of(b"password"), value: None },
            E { key: copy_of(b"shell"), value: Some(V::String(copy_of(b"/bin/bash"))) },
            E { key: copy_of(b"uid"), value: Some(V::Int(1000)) },
        ]))])))
    );
}

const WRONG_TYPES: &[(TypeName, revision::Kind, Schema)] = &[
    (TypeName::Bool, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::Bool }),
    (TypeName::Int, revision::Kind::Bool(true), Schema::Wrong { expected: TypeName::Int }),
    (TypeName::String, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::String }),
    (TypeName::Enum, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::Enum }),
    (TypeName::List, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::List }),
    (TypeName::Map, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::Map }),
    (TypeName::Object, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::Object }),
    (TypeName::Path, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::Path }),
    (TypeName::Mode, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::Mode }),
    (TypeName::Duration, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::Duration }),
    (TypeName::Secret, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::Secret }),
    (TypeName::Target, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::Target }),
    (TypeName::File, revision::Kind::Int(7), Schema::Wrong { expected: TypeName::File }),
];
#[test]
fn a_value_of_another_type_is_refused_at_it() {
    for (row, (name, kind, expected)) in WRONG_TYPES.iter().enumerate() {
        let declaration = typed_declaration(schemas::schema_form(name.clone()));
        let settings =
            composed(&map(at(2, 1), &[(b"x", at(2, 2), revision::Value { at: at(2, 7), kind: kind.clone() })]));
        assert_eq!(
            validate(&declaration, &settings, &Check::Default),
            Err(Box::from([Fault { schema: expected.clone(), document: 2, at: at(2, 7) }])),
            "row {row}"
        );
    }
}
#[test]
fn an_enum_value_not_listed_is_refused_with_the_listed() {
    let declaration = typed_declaration(Form::Enum(Box::new([
        Text { at: at(1, 1), text: copy_of(b"main") },
        Text { at: at(1, 1), text: copy_of(b"contrib") },
    ])));
    let settings = composed(&map(at(2, 1), &[(b"x", at(2, 2), string(at(2, 7), b"other"))]));
    assert_eq!(
        validate(&declaration, &settings, &Check::Default),
        Err(Box::from([Fault {
            schema: Schema::Enum {
                value: copy_of(b"other"),
                listed: Box::new([copy_of(b"main"), copy_of(b"contrib")])
            },
            document: 2,
            at: at(2, 7)
        }]))
    );
}
const BOUNDS: &[(i64, Schema)] = &[(999, Schema::Below { min: 1000 }), (60000, Schema::Above { max: 59999 })];
#[test]
fn ints_outside_min_and_max_are_refused_at_them() {
    for (row, (value, expected)) in BOUNDS.iter().enumerate() {
        let declaration = revision::declaration::Declaration {
            settings: Box::new([Spec {
                min: Some(Number { at: at(1, 1), value: 1000 }),
                max: Some(Number { at: at(1, 1), value: 59999 }),
                ..schema_spec(b"x", 0)
            }]),
            ..typed_declaration(Form::Int)
        };
        let settings = composed(&map(at(2, 1), &[(b"x", at(2, 2), int(at(2, 7), *value))]));
        assert_eq!(
            validate(&declaration, &settings, &Check::Default),
            Err(Box::from([Fault { schema: expected.clone(), document: 2, at: at(2, 7) }])),
            "row {row}"
        );
    }
}
#[test]
fn an_undeclared_field_is_refused_and_its_siblings_still_checked() {
    let declaration = typed_declaration(Form::Object(Box::new([schema_spec(b"uid", 1)])));
    let declaration = revision::declaration::Declaration {
        types: Box::new([
            Type { at: at(1, 1), form: fixtures::only(&declaration.types).form.clone() },
            Type { at: at(1, 2), form: Form::Int },
        ]),
        ..declaration
    };
    let settings = composed(&map(
        at(2, 1),
        &[(
            b"x",
            at(2, 2),
            map(
                at(2, 7),
                &[(b"shel", at(2, 8), string(at(2, 16), b"/bin/sh")), (b"uid", at(2, 28), string(at(2, 35), b"1000"))],
            ),
        )],
    ));
    assert_eq!(
        validate(&declaration, &settings, &Check::Default),
        Err(Box::from([
            Fault {
                schema: Schema::UndeclaredField { field: copy_of(b"shel"), declared: Box::new([copy_of(b"uid")]) },
                document: 2,
                at: at(2, 8)
            },
            Fault { schema: Schema::Wrong { expected: TypeName::Int }, document: 2, at: at(2, 35) },
        ]))
    );
}
#[test]
fn a_null_in_a_default_is_refused_at_it() {
    let declaration = revision::declaration::Declaration {
        settings: Box::new([Spec {
            default: Some(map(at(1, 10), &[(b"a", at(1, 11), fixtures::composition::null(at(1, 16)))])),
            ..schema_spec(b"x", 0)
        }]),
        ..typed_declaration(Form::Map(1))
    };
    assert_eq!(
        fixtures::check_declarations(&fixtures::declarations(&[(b"task", Some(declaration))])).as_ref(),
        &[(copy_of(b"task"), Fault { schema: Schema::Null, document: 0, at: at(1, 16) })]
    );
}
#[test]
fn a_walk_at_the_shipped_depth_fits_its_stack() {
    let mut types = skein_lib::List::with_capacity(crate::limits::SHIPPED.depth);
    let mut value = int(at(2, 7), 42);
    for depth in 0..crate::limits::SHIPPED.depth.saturating_sub(1) {
        types.push(Type { at: at(1, 1), form: Form::List(depth.saturating_add(1)) }).expect("depth");
        value = revision::Value { at: at(2, 7), kind: revision::Kind::List(Box::new([value])) };
    }
    types.push(Type { at: at(1, 1), form: Form::Int }).expect("depth");
    let declaration = revision::declaration::Declaration { types: types.into_boxed(), ..typed_declaration(Form::Int) };
    let settings = composed(&map(at(2, 1), &[(b"x", at(2, 2), value)]));
    let V::Map(settings) = validate(&declaration, &settings, &Check::Default).expect("at depth") else {
        panic!("settings")
    };
    let mut value = fixtures::only(&settings).value.clone().expect("x");
    for _ in 0..crate::limits::SHIPPED.depth.saturating_sub(1) {
        let V::List(items) = value else { panic!("nested list") };
        value = fixtures::only(&items).clone();
    }
    assert_eq!(value, V::Int(42));
}

const LENGTHS: &[(schemas::LengthInput, Schema)] = &[
    (schemas::LengthInput::Bytes, Schema::Length { max_length: 2 }),
    (schemas::LengthInput::Elements, Schema::Length { max_length: 2 }),
    (schemas::LengthInput::Entries, Schema::Length { max_length: 2 }),
];
#[test]
fn max_length_bounds_bytes_elements_and_entries() {
    for (row, (input, expected)) in LENGTHS.iter().enumerate() {
        let (declaration, value) = schemas::length_input(*input);
        let settings = composed(&map(at(2, 1), &[(b"x", at(2, 2), value)]));
        assert_eq!(
            validate(&declaration, &settings, &Check::Default),
            Err(Box::from([Fault { schema: expected.clone(), document: 2, at: at(2, 7) }])),
            "row {row}"
        );
    }
}

#[test]
fn a_bad_field_default_is_reported_once_when_an_object_default_uses_it() {
    let field = Spec { default: Some(string(at(1, 30), b"hi")), ..schema_spec(b"y", 1) };
    let declaration = revision::declaration::Declaration {
        types: Box::new([
            Type { at: at(1, 1), form: Form::Object(Box::new([field])) },
            Type { at: at(1, 2), form: Form::Int },
        ]),
        settings: Box::new([Spec { default: Some(map(at(1, 10), &[])), ..schema_spec(b"x", 0) }]),
        ..fixtures::declaration(&[], &[])
    };
    assert_eq!(
        fixtures::check_declarations(&fixtures::declarations(&[(b"task", Some(declaration))])).as_ref(),
        &[(copy_of(b"task"), Fault { schema: Schema::Wrong { expected: TypeName::Int }, document: 0, at: at(1, 30) })]
    );
}

#[test]
fn an_undeclared_setting_lists_the_declared_ones() {
    let declaration = typed_declaration(Form::Int);
    let value =
        composed(&map(at(2, 1), &[(b"greting", at(2, 7), int(at(2, 17), 5)), (b"x", at(2, 20), int(at(2, 25), 1))]));
    assert_eq!(
        validate(&declaration, &value, &Check::Default),
        Err(Box::from([Fault {
            schema: Schema::Undeclared { setting: copy_of(b"greting"), declared: Box::new([copy_of(b"x")]) },
            document: 2,
            at: at(2, 7)
        }]))
    );
}
#[test]
fn a_missing_setting_is_placed_at_the_task_s_key() {
    let input = schemas::target_schema(&[(b"task", typed_declaration(Form::Int))], &[(b"task", map(at(2, 9), &[]))]);
    let (_, faults, _) = schemas::target_roots(&input, &crate::limits::SHIPPED);
    assert_eq!(
        faults.as_ref(),
        &[(copy_of(b"task"), Fault { schema: Schema::Missing { setting: copy_of(b"x") }, document: 2, at: at(2, 2) })]
    );
}
#[test]
fn a_read_gives_the_other_task_s_setting_with_its_default() {
    let mut core = typed_declaration(Form::Int);
    core.settings = Box::new([Spec { default: Some(int(at(1, 20), 42)), ..schema_spec(b"x", 0) }]);
    let reader = fixtures::declaration(&[], &[fixtures::read(b"core", &[Text { at: at(1, 34), text: copy_of(b"x") }])]);
    let input = schemas::target_schema(
        &[(b"core", core), (b"reader", reader)],
        &[(b"core", map(at(2, 9), &[])), (b"reader", map(at(2, 20), &[]))],
    );
    let (roots, _, _) = schemas::target_roots(&input, &crate::limits::SHIPPED);
    let roots = roots.expect("valid reads");
    let mut reader = None;
    for task in &roots.tasks {
        if task.name.as_ref() == b"reader" {
            reader = Some(task);
        }
    }
    let reader = reader.expect("reader");
    assert_eq!(
        reader.tasks,
        V::Map(Box::new([E {
            key: copy_of(b"core"),
            value: Some(V::Map(Box::new([E { key: copy_of(b"x"), value: Some(V::Int(42)) }])))
        }]))
    );
}
#[test]
fn an_optional_read_s_absent_task_is_none() {
    let read = revision::declaration::Read { optional: true, ..fixtures::read(b"core", &[]) };
    let reader = fixtures::declaration(&[], &[read]);
    let input = schemas::target_schema(
        &[(b"core", fixtures::declaration(&[], &[])), (b"reader", reader)],
        &[(b"reader", map(at(2, 9), &[]))],
    );
    let (roots, _, _) = schemas::target_roots(&input, &crate::limits::SHIPPED);
    assert_eq!(
        fixtures::only(&roots.expect("optional read").tasks).tasks,
        V::Map(Box::new([E { key: copy_of(b"core"), value: None }]))
    );
}
#[test]
fn a_required_read_s_absent_task_is_refused_at_the_read() {
    let reader = fixtures::declaration(&[], &[fixtures::read(b"core", &[])]);
    let input = schemas::target_schema(
        &[(b"core", fixtures::declaration(&[], &[])), (b"reader", reader)],
        &[(b"reader", map(at(2, 9), &[]))],
    );
    let (_, faults, _) = schemas::target_roots(&input, &crate::limits::SHIPPED);
    assert_eq!(
        faults.as_ref(),
        &[(
            copy_of(b"reader"),
            Fault { schema: Schema::Absent { task: copy_of(b"core") }, document: 1, at: at(1, 12) }
        )]
    );
}
#[test]
fn validated_settings_past_their_budget_fail_the_target() {
    let input = schemas::target_schema(
        &[(b"task", typed_declaration(Form::String))],
        &[(b"task", map(at(2, 9), &[(b"x", at(2, 10), string(at(2, 15), b"abcdefghijklmnopqrstuvwxyz012345"))]))],
    );
    let (roots, _, limited) =
        schemas::target_roots(&input, &crate::limits::Limits { validated: 64, ..crate::limits::SHIPPED });
    assert_eq!((roots, limited), (None, true));
}
