use super::fixtures::declaration;
use crate::{
    Declared, Error, Expected, Fault, Object, Position,
    entries::{Literal, Map, Name, Takes},
};

#[test]
fn an_entry_hands_up_its_key_loop_condition_and_fields_as_text() {
    let declaration = declaration(b"{\"files\": {\"/etc/ssh/authorized_keys_{{ key }}\": {\n  \"for_each\": \"settings.authorized\", \"if\": \"key in settings.allow\", \"template\": \"templates/authorized_keys.j2\"}}}").expect("ssh's entry, 5.5");
    let [map] = &declaration.entries[..] else { panic!("one map") };
    let [entry] = &map.entries[..] else { panic!("one entry") };
    assert_eq!((map.map, map.at), (Map::Files, Position { offset: 1, line: 1, column: 2 }));
    assert_eq!(
        (&*entry.key.text, entry.key.at, entry.at),
        (
            b"/etc/ssh/authorized_keys_{{ key }}".as_slice(),
            Position { offset: 11, line: 1, column: 12 },
            Position { offset: 49, line: 1, column: 50 }
        )
    );
    let when = entry.when.as_ref().expect("if");
    let for_each = entry.for_each.as_ref().expect("for_each");
    assert_eq!(
        (&*for_each.text, for_each.at),
        (b"settings.authorized".as_slice(), Position { offset: 65, line: 2, column: 15 })
    );
    assert_eq!(
        (&*when.text, when.at),
        (b"key in settings.allow".as_slice(), Position { offset: 94, line: 2, column: 44 })
    );
    let [field] = &entry.fields[..] else { panic!("one field") };
    let Literal::Plain(template) = &field.value else { panic!("template is plain text") };
    assert_eq!((field.name, field.at), (Name::Template, Position { offset: 119, line: 2, column: 69 }));
    assert_eq!(
        (&*template.text, template.at),
        (b"templates/authorized_keys.j2".as_slice(), Position { offset: 131, line: 2, column: 81 })
    );
}

const MAPS: &[(&[u8], Map)] = &[
    (br#"{"packages":{"entry":{}}}"#, Map::Packages),
    (br#"{"preseeds":{"entry":{}}}"#, Map::Preseeds),
    (br#"{"initramfs":{"entry":{}}}"#, Map::Initramfs),
    (br#"{"cmdline":{"entry":{}}}"#, Map::Cmdline),
    (br#"{"files":{"entry":{}}}"#, Map::Files),
    (br#"{"users":{"entry":{}}}"#, Map::Users),
    (br#"{"groups":{"entry":{}}}"#, Map::Groups),
    (br#"{"presets":{"entry":{}}}"#, Map::Presets),
    (br#"{"tmpfiles":{"entry":{}}}"#, Map::Tmpfiles),
    (br#"{"data":{"entry":{}}}"#, Map::Data),
    (br#"{"secrets":{"entry":{}}}"#, Map::Secrets),
    (br#"{"identity":{"entry":{}}}"#, Map::Identity),
    (br#"{"software":{"entry":{}}}"#, Map::Software),
    (br#"{"live":{"entry":{}}}"#, Map::Live),
];
#[test]
fn every_map_of_m1_takes_entries() {
    for &(input, expected) in MAPS {
        let declaration = declaration(input).expect("entry map");
        let [map] = &declaration.entries[..] else { panic!("one map") };
        let [entry] = &map.entries[..] else { panic!("one entry") };
        assert_eq!(map.map, expected, "{input:?}");
        assert_eq!(
            (&*entry.key.text, entry.when.is_none(), entry.for_each.is_none(), entry.fields.len()),
            (b"entry".as_slice(), true, true, 0)
        );
    }
}

const A_FIELD_ITS_MAP_LACKS_IS_REFUSED_AT_ITS_KEY: &[(&[u8], Position, Declared)] = &[
    (
        br#"{"packages":{"pkg":{"mode":"0640"}}}"#,
        Position { offset: 20, line: 1, column: 21 },
        Declared::Unknown { of: Object::Entry(Map::Packages) },
    ),
    (
        br#"{"initramfs":{"pkg":{"component":"main"}}}"#,
        Position { offset: 21, line: 1, column: 22 },
        Declared::Unknown { of: Object::Entry(Map::Initramfs) },
    ),
    (
        br#"{"users":{"user":{"subordinate":{"size":2}}}}"#,
        Position { offset: 33, line: 1, column: 34 },
        Declared::Unknown { of: Object::Range },
    ),
];
#[test]
fn a_field_its_map_lacks_is_refused_at_its_key() {
    for &(input, at, expected) in A_FIELD_ITS_MAP_LACKS_IS_REFUSED_AT_ITS_KEY {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_FIELD_OF_ANOTHER_FORM_IS_REFUSED_AT_ITS_VALUE: &[(&[u8], Position, Declared)] = &[
    (
        br#"{"files":{"/etc/file":{"mode":640}}}"#,
        Position { offset: 30, line: 1, column: 31 },
        Declared::Wrong { expected: Expected::Field(Takes::Text) },
    ),
    (
        br#"{"users":{"user":{"uid":true}}}"#,
        Position { offset: 24, line: 1, column: 25 },
        Declared::Wrong { expected: Expected::Field(Takes::Int) },
    ),
    (
        br#"{"files":{"/etc/file":{"directory":1}}}"#,
        Position { offset: 35, line: 1, column: 36 },
        Declared::Wrong { expected: Expected::Field(Takes::Bool) },
    ),
    (
        br#"{"users":{"user":{"groups":1}}}"#,
        Position { offset: 27, line: 1, column: 28 },
        Declared::Wrong { expected: Expected::Field(Takes::Strings) },
    ),
    (
        br#"{"users":{"user":{"subordinate":[]}}}"#,
        Position { offset: 32, line: 1, column: 33 },
        Declared::Wrong { expected: Expected::Field(Takes::Range) },
    ),
    (
        br#"{"users":{"user":{"subordinate":{"base":false}}}}"#,
        Position { offset: 40, line: 1, column: 41 },
        Declared::Wrong { expected: Expected::Field(Takes::Int) },
    ),
    (
        br#"{"users":{"user":{"groups":[1]}}}"#,
        Position { offset: 28, line: 1, column: 29 },
        Declared::Wrong { expected: Expected::String },
    ),
    (
        br#"{"users":{"user":{"if":true}}}"#,
        Position { offset: 23, line: 1, column: 24 },
        Declared::Wrong { expected: Expected::String },
    ),
    (
        br#"{"users":{"user":{"for_each":1}}}"#,
        Position { offset: 29, line: 1, column: 30 },
        Declared::Wrong { expected: Expected::String },
    ),
];
#[test]
fn a_field_of_another_form_is_refused_at_its_value() {
    for &(input, at, expected) in A_FIELD_OF_ANOTHER_FORM_IS_REFUSED_AT_ITS_VALUE {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const AN_ENTRY_OR_A_MAP_THAT_IS_NOT_AN_OBJECT_IS_REFUSED_AT_ITS_VALUE: &[(&[u8], Position, Declared)] = &[
    (
        br#"{"packages":1}"#,
        Position { offset: 12, line: 1, column: 13 },
        Declared::Wrong { expected: Expected::Object },
    ),
    (
        br#"{"packages":{"pkg":false}}"#,
        Position { offset: 19, line: 1, column: 20 },
        Declared::Wrong { expected: Expected::Object },
    ),
];
#[test]
fn an_entry_or_a_map_that_is_not_an_object_is_refused_at_its_value() {
    for &(input, at, expected) in AN_ENTRY_OR_A_MAP_THAT_IS_NOT_AN_OBJECT_IS_REFUSED_AT_ITS_VALUE {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

#[test]
fn a_download_s_artifact_is_refused_at_its_key_until_m6() {
    assert_eq!(
        declaration(br#"{"software":{"stoa":{"artifact":1}}}"#),
        Err(Error { at: Position { offset: 21, line: 1, column: 22 }, fault: Fault::Declaration(Declared::Later) })
    );
}

type ScalarCase = (&'static [u8], Option<i64>, Option<&'static [u8]>, Position);
const SCALARS: &[ScalarCase] = &[
    (br#"{"users":{"user":{"uid":59999}}}"#, Some(59999), None, Position { offset: 24, line: 1, column: 25 }),
    (
        br#"{"users":{"user":{"uid":"{{ value.uid }}"}}}"#,
        None,
        Some(b"{{ value.uid }}"),
        Position { offset: 24, line: 1, column: 25 },
    ),
];
#[test]
fn a_field_of_another_type_is_its_literal_or_typed_text() {
    for &(input, integer, text, at) in SCALARS {
        let declaration = declaration(input).expect("integer or typed string");
        let [map] = &declaration.entries[..] else { panic!("one map") };
        let [entry] = &map.entries[..] else { panic!("one entry") };
        let [field] = &entry.fields[..] else { panic!("one field") };
        let observed = match &field.value {
            Literal::Int(number) => (Some(number.value), None, number.at),
            Literal::Typed(text) => (None, Some(&*text.text), text.at),
            Literal::Text(_)
            | Literal::Plain(_)
            | Literal::Bool { .. }
            | Literal::Strings { .. }
            | Literal::Range { .. } => panic!("integer or typed string"),
        };
        assert_eq!(field.name, Name::Uid);
        assert_eq!(observed, (integer, text, at), "{input:?}");
    }
}
const TYPED: &[(&[u8], Name)] = &[
    (br#"{"users":{"user":{"uid":"{{ value }}"}}}"#, Name::Uid),
    (br#"{"files":{"file":{"directory":"{{ value }}"}}}"#, Name::Directory),
    (br#"{"users":{"user":{"groups":"{{ value }}"}}}"#, Name::Groups),
    (br#"{"users":{"user":{"subordinate":"{{ value }}"}}}"#, Name::Subordinate),
];
#[test]
fn each_nontext_field_accepts_typed_text() {
    for &(input, name) in TYPED {
        let declaration = declaration(input).expect("typed string");
        let [map] = &declaration.entries[..] else { panic!("one map") };
        let [entry] = &map.entries[..] else { panic!("one entry") };
        let [field] = &entry.fields[..] else { panic!("one field") };
        let Literal::Typed(text) = &field.value else { panic!("typed string") };
        assert_eq!(field.name, name, "{input:?}");
        assert_eq!(&*text.text, b"{{ value }}", "{input:?}");
    }
}
const BOOLS: &[(&[u8], bool)] =
    &[(br#"{"files":{"file":{"directory":true}}}"#, true), (br#"{"files":{"file":{"directory":false}}}"#, false)];
#[test]
fn a_boolean_field_keeps_its_literal_boolean_at_its_value() {
    for &(input, expected) in BOOLS {
        let declaration = declaration(input).expect("boolean field");
        let [map] = &declaration.entries[..] else { panic!("one map") };
        let [entry] = &map.entries[..] else { panic!("one entry") };
        let [field] = &entry.fields[..] else { panic!("one field") };
        assert_eq!(field.value, Literal::Bool { at: Position { offset: 30, line: 1, column: 31 }, value: expected });
    }
}
const STRINGS: &[(&[u8], Position)] = &[
    (b"{{ first }}", Position { offset: 28, line: 1, column: 29 }),
    (b"second", Position { offset: 42, line: 1, column: 43 }),
];
#[test]
fn a_list_of_strings_keeps_each_text_at_its_string() {
    let declaration = declaration(br#"{"users":{"user":{"groups":["{{ first }}","second"]}}}"#).expect("strings");
    let [map] = &declaration.entries[..] else { panic!("one map") };
    let [entry] = &map.entries[..] else { panic!("one entry") };
    let [field] = &entry.fields[..] else { panic!("one field") };
    let Literal::Strings { at, each } = &field.value else { panic!("string list") };
    assert_eq!(*at, Position { offset: 27, line: 1, column: 28 });
    assert_eq!(each.len(), 2);
    for (text, expected) in each.iter().zip(STRINGS) {
        assert_eq!((&*text.text, text.at), *expected);
    }
}
const RANGE_INTS: &[(Name, Position, Position, i64)] = &[
    (Name::Base, Position { offset: 33, line: 1, column: 34 }, Position { offset: 40, line: 1, column: 41 }, 65536),
    (Name::Count, Position { offset: 46, line: 1, column: 47 }, Position { offset: 54, line: 1, column: 55 }, 1024),
];
#[test]
fn a_subordinate_range_keeps_its_integer_fields_at_their_values() {
    let declaration = declaration(br#"{"users":{"user":{"subordinate":{"base":65536,"count":1024}}}}"#).expect("range");
    let [map] = &declaration.entries[..] else { panic!("one map") };
    let [entry] = &map.entries[..] else { panic!("one entry") };
    let [field] = &entry.fields[..] else { panic!("one field") };
    let Literal::Range { at, fields } = &field.value else { panic!("range") };
    assert_eq!(*at, Position { offset: 32, line: 1, column: 33 });
    assert_eq!(fields.len(), 2);
    for (field, &(name, at, number_at, value)) in fields.iter().zip(RANGE_INTS) {
        assert_eq!((field.name, field.at), (name, at));
        assert_eq!(field.value, Literal::Int(crate::declaration::Number { at: number_at, value }));
    }
}
const RANGE_TEXTS: &[(Name, Position, &[u8])] = &[
    (Name::Base, Position { offset: 40, line: 1, column: 41 }, b"{{ value.base }}"),
    (Name::Count, Position { offset: 67, line: 1, column: 68 }, b"{{ value.count }}"),
];
#[test]
fn a_subordinate_range_accepts_typed_text_in_its_integer_fields() {
    let declaration =
        declaration(br#"{"users":{"user":{"subordinate":{"base":"{{ value.base }}","count":"{{ value.count }}"}}}}"#)
            .expect("range");
    let [map] = &declaration.entries[..] else { panic!("one map") };
    let [entry] = &map.entries[..] else { panic!("one entry") };
    let [field] = &entry.fields[..] else { panic!("one field") };
    let Literal::Range { fields, .. } = &field.value else { panic!("range") };
    assert_eq!(fields.len(), 2);
    for (field, &(name, at, expected)) in fields.iter().zip(RANGE_TEXTS) {
        let Literal::Typed(text) = &field.value else { panic!("typed integer") };
        assert_eq!((field.name, text.at, &*text.text), (name, at, expected));
    }
}

const TEXT_FIELDS: &[(&[u8], Name)] = &[
    (br#"{"packages":{"pkg":{"component":"{{ value }}"}}}"#, Name::Component),
    (br#"{"files":{"file":{"mode":"{{ value }}"}}}"#, Name::Mode),
];
#[test]
fn a_string_field_keeps_interpolated_text_as_written() {
    for &(input, name) in TEXT_FIELDS {
        let declaration = declaration(input).expect("string field");
        let [map] = &declaration.entries[..] else { panic!("one map") };
        let [entry] = &map.entries[..] else { panic!("one entry") };
        let [field] = &entry.fields[..] else { panic!("one field") };
        let Literal::Text(text) = &field.value else { panic!("interpolated text") };
        assert_eq!(field.name, name, "{input:?}");
        assert_eq!(&*text.text, b"{{ value }}", "{input:?}");
    }
}
