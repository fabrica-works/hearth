use super::fixtures::document;
use crate::{
    Fault, Position, Shape,
    json::{Kind, Value},
};
#[test]
fn a_task_value_is_its_settings_object_or_null() {
    const TASKS: &[(&[u8], Position, Position)] = &[
        (b"core", Position { offset: 16, line: 1, column: 17 }, Position { offset: 24, line: 1, column: 25 }),
        (b"wifi", Position { offset: 44, line: 1, column: 45 }, Position { offset: 52, line: 1, column: 53 }),
    ];
    let doc = document(br#"{"_note": "x1", "core": {"hostname": "x1"}, "wifi": null}"#).expect("document");
    assert_eq!(doc.layer.tasks.len(), TASKS.len());
    for (task, expected) in doc.layer.tasks.iter().zip(TASKS) {
        assert_eq!((&*task.name, task.at, task.value.at), *expected);
    }
    assert_eq!(doc.layer.tasks[1].value, Value { at: Position { offset: 52, line: 1, column: 53 }, kind: Kind::Null });
}
#[test]
fn debian_is_kept_as_written_with_its_positions() {
    let value = document(br#"{"debian":{"suite":"forky"}}"#).expect("debian").layer.debian.expect("present");
    assert_eq!(value.at, Position { offset: 10, line: 1, column: 11 });
    let Kind::Map(entries) = value.kind else {
        panic!("object kept");
    };
    assert_eq!(entries[0].key.as_ref(), b"suite");
    assert_eq!(entries[0].at, Position { offset: 11, line: 1, column: 12 });
    assert_eq!(
        entries[0].value,
        Value { at: Position { offset: 19, line: 1, column: 20 }, kind: Kind::String(b"forky".as_slice().into()) }
    );
}

#[test]
fn a_task_value_neither_object_nor_null_is_refused_naming_the_task() {
    const CASES: &[&[u8]] = &[br#"{"core":1}"#, br#"{"core":[]}"#, br#"{"core":false}"#, br#"{"core":"x"}"#];
    for input in CASES {
        let error = document(input).expect_err("settings");
        assert_eq!(error.at, Position { offset: 8, line: 1, column: 9 });
        assert_eq!(error.fault, Fault::Shape(Shape::Settings { task: b"core".as_slice().into() }));
    }
}
#[test]
fn a_key_that_is_no_task_name_is_refused_at_the_key() {
    const CASES: &[&[u8]] = &[br#"{"A":{}}"#, br#"{"a-b":{}}"#, br#"{"":{}}"#, br#"{"1":{}}"#];
    for input in CASES {
        let error = document(input).expect_err("name");
        assert_eq!(error.at, Position { offset: 1, line: 1, column: 2 });
        assert_eq!(error.fault, Fault::Shape(Shape::Name));
    }
}
#[test]
fn placement_and_members_are_refused_until_they_are_decoded() {
    const CASES: &[&[u8]] = &[br#"{"placement":{}}"#, br#"{"members":{}}"#];
    for input in CASES {
        let error = document(input).expect_err("reserved");
        assert_eq!(error.at, Position { offset: 1, line: 1, column: 2 });
        assert_eq!(error.fault, Fault::Shape(Shape::Reserved));
    }
}
