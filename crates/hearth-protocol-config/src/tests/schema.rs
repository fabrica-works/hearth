use super::fixtures::{at, declaration};
use crate::{Declared, Error, Expected, Fault, Object, Required, declaration::Number, schema::Form};
const PARAMS: &[u8] = b"{\"settings\": {\"params\": {\n  \"type\": {\"map\": {\"object\": {\n    \"param\": {\"type\": \"string\", \"doc\": \"One parameter.\"},\n    \"order\": {\"type\": \"int\", \"min\": 0, \"max\": 999, \"default\": 500, \"doc\": \"Its place.\"}}}},\n  \"default\": {}, \"doc\": \"Parameters, by name.\"}}}";
#[test]
fn nested_types_are_laid_out_each_before_its_elements() {
    let declaration = declaration(PARAMS).expect("boot's params, 5.4");
    let [params] = &declaration.settings[..] else { panic!("one setting") };
    assert_eq!(
        (&*params.name.text, params.name.at, params.at, params.ty),
        (b"params".as_slice(), at(14, 1, 15), at(24, 1, 25), 0)
    );
    let [map, object, int, string] = &declaration.types[..] else { panic!("four types, in pre-order") };
    assert_eq!((map.at, &map.form, object.at), (at(36, 2, 11), &Form::Map(1), at(44, 2, 19)));
    let Form::Object(fields) = &object.form else { panic!("the map's values are objects") };
    let [order, param] = &fields[..] else { panic!("two fields, in name order") };
    assert_eq!(
        (order.at, order.ty, order.min, order.max),
        (
            at(128, 4, 14),
            2,
            Some(Number { at: at(151, 4, 37), value: 0 }),
            Some(Number { at: at(161, 4, 47), value: 999 })
        )
    );
    assert_eq!(
        (int.at, &int.form, string.at, &string.form, param.ty),
        (at(137, 4, 23), &Form::Int, at(79, 3, 23), &Form::String, 3)
    );
}

const TYPES: &[(&[u8], &str)] = &[
    (br#"{"settings":{"x":{"type":"bool","doc":"d"}}}"#, "Bool"),
    (br#"{"settings":{"x":{"type":"int","doc":"d"}}}"#, "Int"),
    (br#"{"settings":{"x":{"type":"string","doc":"d"}}}"#, "String"),
    (br#"{"settings":{"x":{"type":"path","doc":"d"}}}"#, "Path"),
    (br#"{"settings":{"x":{"type":"mode","doc":"d"}}}"#, "Mode"),
    (br#"{"settings":{"x":{"type":"duration","doc":"d"}}}"#, "Duration"),
    (br#"{"settings":{"x":{"type":"secret","doc":"d"}}}"#, "Secret"),
    (br#"{"settings":{"x":{"type":"target","doc":"d"}}}"#, "Target"),
    (br#"{"settings":{"x":{"type":"file","doc":"d"}}}"#, "File"),
    (br#"{"settings":{"x":{"type":{"list":"string"},"doc":"d"}}}"#, "List"),
    (br#"{"settings":{"x":{"type":{"map":"int"},"doc":"d"}}}"#, "Map"),
    (br#"{"settings":{"x":{"type":{"object":{}},"doc":"d"}}}"#, "Object"),
    (br#"{"settings":{"x":{"type":{"enum":["a"]},"doc":"d"}}}"#, "Enum"),
];
#[test]
fn each_of_the_thirteen_types_decodes_from_its_name_or_object() {
    for &(input, expected) in TYPES {
        let declaration = declaration(input).expect("a schema type");
        let actual = match &declaration.types[0].form {
            Form::Bool => "Bool",
            Form::Int => "Int",
            Form::String => "String",
            Form::Path => "Path",
            Form::Mode => "Mode",
            Form::Duration => "Duration",
            Form::Secret => "Secret",
            Form::Target => "Target",
            Form::File => "File",
            Form::Enum(_) => "Enum",
            Form::List(_) => "List",
            Form::Map(_) => "Map",
            Form::Object(_) => "Object",
        };
        assert_eq!(actual, expected, "{input:?}");
    }
}

const A_TYPE_NAME_6_1_LACKS_IS_REFUSED_AT_ITS_STRING: &[(&[u8], crate::Position, Declared)] = &[
    (
        br#"{"settings":{"x":{"type":"integer","doc":"d"}}}"#,
        at(25, 1, 26),
        Declared::Wrong { expected: Expected::Type },
    ),
    (br#"{"settings":{"x":{"type":"float","doc":"d"}}}"#, at(25, 1, 26), Declared::Wrong { expected: Expected::Type }),
    (br#"{"settings":{"x":{"type":"null","doc":"d"}}}"#, at(25, 1, 26), Declared::Wrong { expected: Expected::Type }),
    (
        br#"{"settings":{"x":{"type":{"list":"integer"},"doc":"d"}}}"#,
        at(33, 1, 34),
        Declared::Wrong { expected: Expected::Type },
    ),
];
#[test]
fn a_type_name_6_1_lacks_is_refused_at_its_string() {
    for &(input, at, expected) in A_TYPE_NAME_6_1_LACKS_IS_REFUSED_AT_ITS_STRING {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_SETTING_OR_FIELD_NAME_OUT_OF_2_4S_FORM_IS_REFUSED_AT_ITS_KEY: &[(&[u8], crate::Position, Declared)] = &[
    (br#"{"settings":{"Port":{"type":"bool","doc":"d"}}}"#, at(13, 1, 14), Declared::Name),
    (
        br#"{"settings":{"x":{"type":{"object":{"Port":{"type":"bool","doc":"d"}}},"doc":"d"}}}"#,
        at(36, 1, 37),
        Declared::Name,
    ),
    (br#"{"settings":{"a-b":{"type":"bool","doc":"d"}}}"#, at(13, 1, 14), Declared::Name),
    (
        br#"{"settings":{"x":{"type":{"object":{"a-b":{"type":"bool","doc":"d"}}},"doc":"d"}}}"#,
        at(36, 1, 37),
        Declared::Name,
    ),
    (br#"{"settings":{"1st":{"type":"bool","doc":"d"}}}"#, at(13, 1, 14), Declared::Name),
    (
        br#"{"settings":{"x":{"type":{"object":{"1st":{"type":"bool","doc":"d"}}},"doc":"d"}}}"#,
        at(36, 1, 37),
        Declared::Name,
    ),
];
#[test]
fn a_setting_or_field_name_out_of_2_4s_form_is_refused_at_its_key() {
    for &(input, at, expected) in A_SETTING_OR_FIELD_NAME_OUT_OF_2_4S_FORM_IS_REFUSED_AT_ITS_KEY {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_SPEC_WITHOUT_TYPE_OR_DOC_IS_REFUSED_AT_ITS_BRACE: &[(&[u8], crate::Position, Declared)] = &[
    (br#"{"settings":{"x":{"doc":"d"}}}"#, at(17, 1, 18), Declared::Missing { key: Required::Type }),
    (br#"{"settings":{"x":{"type":"bool"}}}"#, at(17, 1, 18), Declared::Missing { key: Required::Doc }),
];
#[test]
fn a_spec_without_type_or_doc_is_refused_at_its_brace() {
    for &(input, at, expected) in A_SPEC_WITHOUT_TYPE_OR_DOC_IS_REFUSED_AT_ITS_BRACE {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_SPEC_DOC_OF_TWO_LINES_OR_201_BYTES_IS_REFUSED_AT_ITS_VALUE: &[(&[u8], crate::Position, Declared)] = &[
    (br#"{"settings":{"x":{"type":"bool","doc":"a\nb"}}}"#, at(38, 1, 39), Declared::Doc),
    (br#"{"settings":{"x":{"type":"bool","doc":"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}}}"#, at(38, 1, 39), Declared::Doc),
];
#[test]
fn a_spec_doc_of_two_lines_or_201_bytes_is_refused_at_its_value() {
    for &(input, at, expected) in A_SPEC_DOC_OF_TWO_LINES_OR_201_BYTES_IS_REFUSED_AT_ITS_VALUE {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_SPEC_VALUE_OF_ANOTHER_FORM_IS_REFUSED_AT_IT: &[(&[u8], crate::Position, Declared)] = &[
    (
        br#"{"settings":{"x":{"type":"bool","doc":"d","optional":false}}}"#,
        at(53, 1, 54),
        Declared::Wrong { expected: Expected::True },
    ),
    (
        br#"{"settings":{"x":{"type":"int","doc":"d","min":"1"}}}"#,
        at(47, 1, 48),
        Declared::Wrong { expected: Expected::Int },
    ),
    (br#"{"settings":{"x":{"type":5,"doc":"d"}}}"#, at(25, 1, 26), Declared::Wrong { expected: Expected::Type }),
    (
        br#"{"settings":{"x":{"type":{"enum":["a",1]},"doc":"d"}}}"#,
        at(38, 1, 39),
        Declared::Wrong { expected: Expected::String },
    ),
];
#[test]
fn a_spec_value_of_another_form_is_refused_at_it() {
    for &(input, at, expected) in A_SPEC_VALUE_OF_ANOTHER_FORM_IS_REFUSED_AT_IT {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const OPTIONAL_BESIDE_A_DEFAULT_IS_REFUSED_AT_ITS_KEY: &[(&[u8], crate::Position, Declared)] = &[
    (
        br#"{"settings":{"x":{"type":"bool","doc":"d","optional":true,"default":false}}}"#,
        at(42, 1, 43),
        Declared::Optional,
    ),
    (
        br#"{"settings":{"x":{"type":"bool","doc":"d","optional":true,"default":true}}}"#,
        at(42, 1, 43),
        Declared::Optional,
    ),
];
#[test]
fn optional_beside_a_default_is_refused_at_its_key() {
    for &(input, at, expected) in OPTIONAL_BESIDE_A_DEFAULT_IS_REFUSED_AT_ITS_KEY {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const BOUNDS_ON_A_TYPE_THEY_DO_NOT_BOUND_ARE_REFUSED_AT_THEIR_KEY: &[(&[u8], crate::Position, Declared)] = &[
    (br#"{"settings":{"x":{"type":"string","doc":"d","min":1}}}"#, at(44, 1, 45), Declared::Bound),
    (br#"{"settings":{"x":{"type":"bool","doc":"d","max":1}}}"#, at(42, 1, 43), Declared::Bound),
    (br#"{"settings":{"x":{"type":"path","doc":"d","max_length":1}}}"#, at(42, 1, 43), Declared::Bound),
];
#[test]
fn bounds_on_a_type_they_do_not_bound_are_refused_at_their_key() {
    for &(input, at, expected) in BOUNDS_ON_A_TYPE_THEY_DO_NOT_BOUND_ARE_REFUSED_AT_THEIR_KEY {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const MIN_ABOVE_MAX_IS_REFUSED_AT_MIN_S_VALUE: &[(&[u8], crate::Position, Declared)] = &[
    (br#"{"settings":{"x":{"type":"int","doc":"d","min":1,"max":0}}}"#, at(47, 1, 48), Declared::Above),
    (br#"{"settings":{"x":{"type":"int","doc":"d","min":2,"max":0}}}"#, at(47, 1, 48), Declared::Above),
];
#[test]
fn min_above_max_is_refused_at_min_s_value() {
    for &(input, at, expected) in MIN_ABOVE_MAX_IS_REFUSED_AT_MIN_S_VALUE {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const AN_ENUM_WITH_NO_VALUE_OR_ONE_TWICE_IS_REFUSED: &[(&[u8], crate::Position, Declared)] = &[
    (br#"{"settings":{"x":{"type":{"enum":[]},"doc":"d"}}}"#, at(33, 1, 34), Declared::Enum),
    (br#"{"settings":{"x":{"type":{"enum":["a","a"]},"doc":"d"}}}"#, at(38, 1, 39), Declared::Enum),
];
#[test]
fn an_enum_with_no_value_or_one_twice_is_refused() {
    for &(input, at, expected) in AN_ENUM_WITH_NO_VALUE_OR_ONE_TWICE_IS_REFUSED {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_TYPE_OBJECT_OF_NO_KEY_OR_MORE_THAN_ONE_IS_REFUSED: &[(&[u8], crate::Position, Declared)] = &[
    (br#"{"settings":{"x":{"type":{},"doc":"d"}}}"#, at(25, 1, 26), Declared::Wrong { expected: Expected::Type }),
    (
        br#"{"settings":{"x":{"type":{"list":"int","map":"int"},"doc":"d"}}}"#,
        at(25, 1, 26),
        Declared::Wrong { expected: Expected::Type },
    ),
    (
        br#"{"settings":{"x":{"type":{"float":"int"},"doc":"d"}}}"#,
        at(25, 1, 26),
        Declared::Wrong { expected: Expected::Type },
    ),
];
#[test]
fn a_type_object_of_no_key_or_more_than_one_is_refused() {
    for &(input, at, expected) in A_TYPE_OBJECT_OF_NO_KEY_OR_MORE_THAN_ONE_IS_REFUSED {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_NEGATIVE_MAX_LENGTH_IS_REFUSED_AT_ITS_VALUE: &[(&[u8], crate::Position, Declared)] = &[
    (br#"{"settings":{"x":{"type":"string","doc":"d","max_length":-1}}}"#, at(57, 1, 58), Declared::Bound),
    (br#"{"settings":{"x":{"type":{"list":"int"},"doc":"d","max_length":-1}}}"#, at(63, 1, 64), Declared::Bound),
];
#[test]
fn a_negative_max_length_is_refused_at_its_value() {
    for &(input, at, expected) in A_NEGATIVE_MAX_LENGTH_IS_REFUSED_AT_ITS_VALUE {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_SPEC_UNKNOWN_KEY_IS_REFUSED_AT_ITS_KEY: &[(&[u8], crate::Position, Declared)] = &[
    (br#"{"settings":{"x":{"type":"int","doc":"d","typo":0}}}"#, at(41, 1, 42), Declared::Unknown { of: Object::Spec }),
    (
        br#"{"settings":{"x":{"type":"int","doc":"d","value":0}}}"#,
        at(41, 1, 42),
        Declared::Unknown { of: Object::Spec },
    ),
];
#[test]
fn a_spec_unknown_key_is_refused_at_its_key() {
    for &(input, at, expected) in A_SPEC_UNKNOWN_KEY_IS_REFUSED_AT_ITS_KEY {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

#[test]
fn the_513th_setting_is_refused_at_its_key() {
    let input = super::fixtures::settings(513);
    assert_eq!(
        declaration(&input),
        Err(Error { at: at(16909, 1, 16910), fault: Fault::Declaration(Declared::Settings) })
    );
}
