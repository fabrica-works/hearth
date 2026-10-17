use super::fixtures;
use crate::{
    errors::{self, Decode, Error, Kind, Limit, Place},
    limits,
    revision::{Oid, Position},
    vocabulary::{Answer, Class, Decoded, Outcome},
};
use skein_lib::bytes::copy_of;
#[test]
fn a_refused_document_s_error_names_its_path_and_position() {
    let mut call = fixtures::error_call(&limits::SHIPPED);
    call.decoded(
        &Oid(copy_of(b"environment")),
        Decoded::Refused { at: Position { line: 2, column: 3 }, fault: Decode::Unexpected, also: None },
    );
    assert_eq!(
        call.errors.as_slice(),
        &[Error {
            kind: Kind::Decode(Decode::Unexpected),
            class: Class::Configuration,
            at: Some(Place { path: copy_of(b"envs/prod.json"), position: Some(Position { line: 2, column: 3 }) }),
            also: None,
            target: None,
            task: None,
        }]
    );
}
#[test]
fn a_duplicate_key_s_error_gives_the_first_as_also() {
    let mut call = fixtures::error_call(&limits::SHIPPED);
    call.decoded(
        &Oid(copy_of(b"environment")),
        Decoded::Refused {
            at: Position { line: 1, column: 14 },
            fault: Decode::Duplicate { key: copy_of(b"motd") },
            also: Some(Position { line: 1, column: 2 }),
        },
    );
    assert_eq!(
        fixtures::only(call.errors.as_slice()).also,
        Some(Place { path: copy_of(b"envs/prod.json"), position: Some(Position { line: 1, column: 2 }) })
    );
}
#[test]
fn a_refused_task_json_s_error_names_its_task() {
    let error =
        errors::refused(b"tasks/motd/task.json", Some(Position { line: 1, column: 2 }), Decode::Unexpected, None);
    assert_eq!(error.task.as_deref(), Some(b"motd".as_slice()));
    assert_eq!(error.target, None);
}
#[test]
fn errors_follow_the_last_spec_and_precede_rendered() {
    let (mut call, target) = fixtures::removed_task();
    call.targets = Box::new([target]);
    call.commit = Some(Oid(copy_of(b"commit")));
    errors::push(&mut call.errors, errors::refused(b"envs/lab.json", None, Decode::Unexpected, None));
    let Answer::Spec(spec) = call.answer() else { panic!("spec first") };
    assert_eq!(spec.meta.target.as_ref(), b"x1-prod");
    assert_eq!(
        call.answer(),
        Answer::Error(Error {
            kind: Kind::Decode(Decode::Unexpected),
            class: Class::Configuration,
            at: Some(Place { path: copy_of(b"envs/lab.json"), position: None }),
            also: None,
            target: None,
            task: None
        })
    );
    assert_eq!(call.answer(), Answer::Rendered { outcome: Outcome::Done, diff: None });
}
#[test]
fn a_document_past_its_limit_errs_at_its_path_and_fails_its_targets() {
    let mut call = fixtures::error_call(&limits::Limits { document: 16, ..limits::SHIPPED });
    let mut out = skein_lib::Queue::with_capacity(1);
    call.files(limits::SHIPPED, &mut out);
    assert_eq!(
        call.answer(),
        Answer::Error(Error {
            kind: Kind::Decode(Decode::TooLarge { limit: Limit::Document, max: 16 }),
            class: Class::Configuration,
            at: Some(Place { path: copy_of(b"envs/prod.json"), position: None }),
            also: None,
            target: None,
            task: None
        })
    );
    assert_eq!(call.outcome, Outcome::Failed);
}
#[test]
fn the_budget_s_error_names_the_path_that_passed_it() {
    let call = fixtures::error_call(&limits::Limits { decoded: 64, ..limits::SHIPPED });
    assert_eq!(
        call.errors.as_slice(),
        &[Error {
            kind: Kind::Decode(Decode::TooLarge { limit: Limit::Decoded, max: 64 }),
            class: Class::Configuration,
            at: Some(Place { path: copy_of(b"hosts/x1/x1.json"), position: None }),
            also: None,
            target: None,
            task: None
        }]
    );
    assert_eq!(call.targets.as_ref(), []);
}
#[test]
fn a_static_file_past_its_limit_errs_and_no_read_asks_it() {
    let (mut call, mut target) = fixtures::removed_task();
    target.base_env = target.environment_document;
    call.targets = Box::new([target]);
    call.commit = Some(Oid(copy_of(b"commit")));
    let mut entries = skein_lib::List::with_capacity(3);
    for entry in call.entries.clone() {
        entries.push(entry).expect("fixture");
    }
    entries
        .push(crate::revision::Entry {
            path: copy_of(b"tasks/motd/tree/etc/motd"),
            mode: crate::revision::Mode::File,
            oid: Oid(copy_of(b"file")),
            size: 9,
        })
        .expect("fixture");
    call.entries = entries.into_boxed();
    let mut out = skein_lib::Queue::with_capacity(1);
    assert!(!call.files(limits::Limits { static_file: 8, ..limits::SHIPPED }, &mut out));
    assert_eq!(
        call.answer(),
        Answer::Error(Error {
            kind: Kind::Decode(Decode::TooLarge { limit: Limit::Static, max: 8 }),
            class: Class::Configuration,
            at: Some(Place { path: copy_of(b"tasks/motd/tree/etc/motd"), position: None }),
            also: None,
            target: None,
            task: Some(copy_of(b"motd"))
        })
    );
    assert_eq!(out.pop(), None);
}
