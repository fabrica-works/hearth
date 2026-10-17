use super::fixtures::{at, declaration, text};
use crate::{
    Declared, Error, Expected, Fault, Object, Position, Required,
    declaration::{self, Fragment},
    json::{Kind, Value},
    limits::SHIPPED,
};

#[test]
fn an_empty_object_is_a_task_that_declares_nothing() {
    let declaration = declaration(b"{}").expect("empty declaration");
    assert_eq!(
        (
            declaration.at,
            declaration.reads.len(),
            declaration.requires.len(),
            declaration.scripts.len(),
            declaration.bless,
            declaration.entries.len(),
        ),
        (Position { offset: 0, line: 1, column: 1 }, 0, 0, 0, None, 0)
    );
}
#[test]
fn a_nonobject_root_is_refused_at_its_value() {
    let input = text(at(0, 1, 1), b"task");
    let value = Value { at: input.at, kind: Kind::String(input.text) };
    assert_eq!(
        declaration::declaration(value, &SHIPPED),
        Err(Error { at: Position { offset: 0, line: 1, column: 1 }, fault: Fault::NotObject })
    );
}
#[test]
fn a_guard_hands_up_its_condition_and_message_as_text() {
    let declaration = declaration(b"{\"doc\": \"The kernel.\",\n \"requires\": {\"kernel\": {\"if\": \"target.placement.kind != 'container'\", \"message\": \"a container has no kernel\"}}}").expect("declaration");
    let [kernel] = &declaration.requires[..] else { panic!("one guard") };
    assert_eq!(
        (&*kernel.name.text, kernel.name.at, kernel.at),
        (
            b"kernel".as_slice(),
            Position { offset: 37, line: 2, column: 15 },
            Position { offset: 47, line: 2, column: 25 }
        )
    );
    assert_eq!(
        (&*kernel.when.text, kernel.when.at),
        (b"target.placement.kind != 'container'".as_slice(), Position { offset: 54, line: 2, column: 32 })
    );
    assert_eq!(
        (&*kernel.message.text, kernel.message.at),
        (b"a container has no kernel".as_slice(), Position { offset: 105, line: 2, column: 83 })
    );
}
#[test]
fn checks_are_refused_at_their_key_until_m5() {
    assert_eq!(
        declaration(br#"{"checks":5}"#),
        Err(Error { at: Position { offset: 1, line: 1, column: 2 }, fault: Fault::Declaration(Declared::Later) })
    );
}
#[test]
fn a_script_5_8_does_not_name_is_refused_at_its_key() {
    assert_eq!(
        declaration(br#"{"scripts":{"build.sh":5}}"#),
        Err(Error { at: Position { offset: 12, line: 1, column: 13 }, fault: Fault::Declaration(Declared::Name) })
    );
}
#[test]
fn a_script_without_order_is_at_4_3s_default_500() {
    let declaration = declaration(br#"{"scripts":{"customize.sh":{}}}"#).expect("default order");
    let [script] = &declaration.scripts[..] else { panic!("one script") };
    assert_eq!(
        (script.fragment, script.at, script.order),
        (Fragment::Customize, Position { offset: 12, line: 1, column: 13 }, 500)
    );
}
#[test]
fn a_task_doc_of_two_lines_is_refused_at_its_value() {
    assert_eq!(
        declaration(br#"{"doc":"one\ntwo"}"#),
        Err(Error { at: Position { offset: 7, line: 1, column: 8 }, fault: Fault::Declaration(Declared::Doc) })
    );
}

const A_KEY_THE_DECLARATION_LACKS_IS_REFUSED_AT_THE_KEY: &[(&[u8], Position, Declared)] = &[
    (br#"{"pakages":0}"#, Position { offset: 1, line: 1, column: 2 }, Declared::Unknown { of: Object::Declaration }),
    (br#"{"setting":0}"#, Position { offset: 1, line: 1, column: 2 }, Declared::Unknown { of: Object::Declaration }),
];
#[test]
fn a_key_the_declaration_lacks_is_refused_at_the_key() {
    for &(input, at, expected) in A_KEY_THE_DECLARATION_LACKS_IS_REFUSED_AT_THE_KEY {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_GUARD_WITHOUT_IF_OR_MESSAGE_IS_REFUSED_AT_ITS_BRACE: &[(&[u8], Position, Declared)] = &[
    (
        br#"{"requires":{"kernel":{"message":"no"}}}"#,
        Position { offset: 22, line: 1, column: 23 },
        Declared::Missing { key: Required::If },
    ),
    (
        br#"{"requires":{"kernel":{"if":"true"}}}"#,
        Position { offset: 22, line: 1, column: 23 },
        Declared::Missing { key: Required::Message },
    ),
    (
        br#"{"requires":{"kernel":{}}}"#,
        Position { offset: 22, line: 1, column: 23 },
        Declared::Missing { key: Required::If },
    ),
];
#[test]
fn a_guard_without_if_or_message_is_refused_at_its_brace() {
    for &(input, at, expected) in A_GUARD_WITHOUT_IF_OR_MESSAGE_IS_REFUSED_AT_ITS_BRACE {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_GUARD_NAMED_OUT_OF_FORM_IS_REFUSED_AT_ITS_KEY: &[(&[u8], Position, Declared)] = &[
    (br#"{"requires":{"Kernel":5}}"#, Position { offset: 13, line: 1, column: 14 }, Declared::Name),
    (br#"{"requires":{"no-kernel":5}}"#, Position { offset: 13, line: 1, column: 14 }, Declared::Name),
    (br#"{"requires":{"":5}}"#, Position { offset: 13, line: 1, column: 14 }, Declared::Name),
];
#[test]
fn a_guard_named_out_of_form_is_refused_at_its_key() {
    for &(input, at, expected) in A_GUARD_NAMED_OUT_OF_FORM_IS_REFUSED_AT_ITS_KEY {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_READ_S_TASK_OR_SETTING_OUT_OF_FORM_IS_REFUSED_AT_IT: &[(&[u8], Position, Declared)] = &[
    (br#"{"reads":{"Core":5}}"#, Position { offset: 10, line: 1, column: 11 }, Declared::Name),
    (br#"{"reads":{"core":{"settings":["a-b"]}}}"#, Position { offset: 30, line: 1, column: 31 }, Declared::Name),
];
#[test]
fn a_read_s_task_or_setting_out_of_form_is_refused_at_it() {
    for &(input, at, expected) in A_READ_S_TASK_OR_SETTING_OUT_OF_FORM_IS_REFUSED_AT_IT {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_SCRIPT_ORDER_OUTSIDE_0_TO_999_IS_REFUSED_AT_ITS_VALUE: &[(&[u8], Position, Declared)] = &[
    (br#"{"scripts":{"customize.sh":{"order":-1}}}"#, Position { offset: 36, line: 1, column: 37 }, Declared::Order),
    (br#"{"scripts":{"customize.sh":{"order":1000}}}"#, Position { offset: 36, line: 1, column: 37 }, Declared::Order),
    (br#"{"scripts":{"customize.sh":{"order":"500"}}}"#, Position { offset: 36, line: 1, column: 37 }, Declared::Order),
    (br#"{"scripts":{"customize.sh":{"order":true}}}"#, Position { offset: 36, line: 1, column: 37 }, Declared::Order),
];
#[test]
fn a_script_order_outside_0_to_999_is_refused_at_its_value() {
    for &(input, at, expected) in A_SCRIPT_ORDER_OUTSIDE_0_TO_999_IS_REFUSED_AT_ITS_VALUE {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const THE_KEYS_DECODED_LATER_ARE_REFUSED_UNTIL_THEIR_INCREMENT: &[(&[u8], Position, Declared)] = &[
    (br#"{"settings":5}"#, Position { offset: 1, line: 1, column: 2 }, Declared::Undecoded),
    (br#"{"exports":5}"#, Position { offset: 1, line: 1, column: 2 }, Declared::Undecoded),
];
#[test]
fn the_keys_decoded_later_are_refused_until_their_increment() {
    for &(input, at, expected) in THE_KEYS_DECODED_LATER_ARE_REFUSED_UNTIL_THEIR_INCREMENT {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_PLAIN_OBJECT_FIELD_ITS_OBJECT_LACKS_IS_REFUSED_AT_ITS_KEY: &[(&[u8], Position, Declared)] = &[
    (
        br#"{"reads":{"core":{"unknown":0}}}"#,
        Position { offset: 18, line: 1, column: 19 },
        Declared::Unknown { of: Object::Read },
    ),
    (
        br#"{"requires":{"core":{"unknown":0}}}"#,
        Position { offset: 21, line: 1, column: 22 },
        Declared::Unknown { of: Object::Guard },
    ),
    (
        br#"{"scripts":{"customize.sh":{"unknown":0}}}"#,
        Position { offset: 28, line: 1, column: 29 },
        Declared::Unknown { of: Object::Script },
    ),
    (
        br#"{"bless":{"unknown":0}}"#,
        Position { offset: 10, line: 1, column: 11 },
        Declared::Unknown { of: Object::Bless },
    ),
];
#[test]
fn a_plain_object_field_its_object_lacks_is_refused_at_its_key() {
    for &(input, at, expected) in A_PLAIN_OBJECT_FIELD_ITS_OBJECT_LACKS_IS_REFUSED_AT_ITS_KEY {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const A_PLAIN_FIELD_OF_ANOTHER_FORM_IS_REFUSED_AT_ITS_VALUE: &[(&[u8], Position, Declared)] = &[
    (br#"{"doc":0}"#, Position { offset: 7, line: 1, column: 8 }, Declared::Wrong { expected: Expected::String }),
    (br#"{"reads":0}"#, Position { offset: 9, line: 1, column: 10 }, Declared::Wrong { expected: Expected::Object }),
    (
        br#"{"reads":{"core":0}}"#,
        Position { offset: 17, line: 1, column: 18 },
        Declared::Wrong { expected: Expected::Object },
    ),
    (
        br#"{"reads":{"core":{"settings":0}}}"#,
        Position { offset: 29, line: 1, column: 30 },
        Declared::Wrong { expected: Expected::Strings },
    ),
    (
        br#"{"reads":{"core":{"settings":[0]}}}"#,
        Position { offset: 30, line: 1, column: 31 },
        Declared::Wrong { expected: Expected::String },
    ),
    (
        br#"{"reads":{"core":{"exports":0}}}"#,
        Position { offset: 28, line: 1, column: 29 },
        Declared::Wrong { expected: Expected::Bool },
    ),
    (
        br#"{"reads":{"core":{"optional":0}}}"#,
        Position { offset: 29, line: 1, column: 30 },
        Declared::Wrong { expected: Expected::Bool },
    ),
    (
        br#"{"requires":{"core":{"if":0}}}"#,
        Position { offset: 26, line: 1, column: 27 },
        Declared::Wrong { expected: Expected::String },
    ),
    (
        br#"{"requires":{"core":{"if":"true","message":0}}}"#,
        Position { offset: 43, line: 1, column: 44 },
        Declared::Wrong { expected: Expected::String },
    ),
    (
        br#"{"scripts":{"customize.sh":0}}"#,
        Position { offset: 27, line: 1, column: 28 },
        Declared::Wrong { expected: Expected::Object },
    ),
    (
        br#"{"bless":{"ignore_failed":[0]}}"#,
        Position { offset: 27, line: 1, column: 28 },
        Declared::Wrong { expected: Expected::String },
    ),
];
#[test]
fn a_plain_field_of_another_form_is_refused_at_its_value() {
    for &(input, at, expected) in A_PLAIN_FIELD_OF_ANOTHER_FORM_IS_REFUSED_AT_ITS_VALUE {
        assert_eq!(declaration(input), Err(Error { at, fault: Fault::Declaration(expected) }), "{input:?}");
    }
}

const ORDERS: &[(&[u8], u16)] =
    &[(br#"{"scripts":{"customize.sh":{"order":0}}}"#, 0), (br#"{"scripts":{"customize.sh":{"order":999}}}"#, 999)];
#[test]
fn a_script_order_is_an_integer_from_0_to_999() {
    for &(input, expected) in ORDERS {
        let declaration = declaration(input).expect("order in range");
        let [script] = &declaration.scripts[..] else { panic!("one script") };
        assert_eq!(script.order, expected, "{input:?}");
    }
}
#[test]
fn the_first_fault_in_key_order_is_returned() {
    assert_eq!(
        declaration(br#"{"requires":{"bad-name":0},"doc":"a\nb"}"#),
        Err(Error { at: Position { offset: 33, line: 1, column: 34 }, fault: Fault::Declaration(Declared::Doc) })
    );
}
