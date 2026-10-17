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
        fixtures::kept(&call).as_ref(),
        &[Error {
            side: None,
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
        fixtures::only(&fixtures::kept(&call)).also,
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
            side: None,
            kind: Kind::Decode(Decode::Unexpected),
            class: Class::Configuration,
            at: Some(Place { path: copy_of(b"envs/lab.json"), position: None }),
            also: None,
            target: None,
            task: None
        })
    );
    assert_eq!(call.answer(), Answer::Rendered { outcome: Outcome::Failed, diff: None, left_out: 0 });
}
#[test]
fn a_document_past_its_limit_errs_at_its_path_and_fails_its_targets() {
    let mut call = fixtures::error_call(&limits::Limits { document: 16, ..limits::SHIPPED });
    let mut out = skein_lib::Queue::with_capacity(1);
    call.files(limits::SHIPPED, &mut out);
    assert_eq!(
        call.answer(),
        Answer::Error(Error {
            side: None,
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
        fixtures::kept(&call).as_ref(),
        &[Error {
            side: None,
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
            side: None,
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

use crate::errors::Collector;
use fixtures::Spot;
const SORT_INPUTS: &[Spot<'_>] =
    &[(b"b.json", 1, 1, b"t"), (b"a.json", 2, 9, b"t"), (b"a.json", 2, 3, b"u"), (b"a.json", 2, 3, b"t")];
const SORTED: &[Spot<'_>] =
    &[(b"a.json", 2, 3, b"t"), (b"a.json", 2, 3, b"u"), (b"a.json", 2, 9, b"t"), (b"b.json", 1, 1, b"t")];
#[test]
fn errors_sort_by_path_then_line_then_column_then_target() {
    let mut collector = Collector::new(&fixtures::limits_with_errors(8));
    for &(path, line, column, target) in SORT_INPUTS {
        collector.push(fixtures::error_at(path, line, column, target));
    }
    let (kept, _) = collector.finish();
    assert_eq!(fixtures::places(&kept).as_ref(), SORTED, "7.1");
}
const UNPLACED: &[Spot<'_>] = &[(b"z.json", 0, 0, b"z"), (b"", 0, 0, b"a"), (b"", 0, 0, b"b")];
#[test]
fn an_error_without_a_place_follows_the_placed_by_target() {
    let mut collector = Collector::new(&fixtures::limits_with_errors(3));
    collector.push(fixtures::unplaced(b"b"));
    collector.push(fixtures::unplaced(b"a"));
    let mut placed = errors::refused(b"z.json", None, Decode::Unexpected, None);
    placed.target = Some(copy_of(b"z"));
    collector.push(placed);
    let (kept, _) = collector.finish();
    assert_eq!(fixtures::places(&kept).as_ref(), UNPLACED);
}
#[test]
fn equal_errors_keep_their_arrival_order() {
    let mut collector = Collector::new(&fixtures::limits_with_errors(2));
    let mut first = fixtures::error_at(b"a", 1, 1, b"t");
    first.kind = Kind::Decode(Decode::Trailing);
    collector.push(first);
    collector.push(fixtures::error_at(b"a", 1, 1, b"t"));
    let (kept, _) = collector.finish();
    let mut kinds = skein_lib::List::with_capacity(2);
    for error in kept {
        kinds.push(error.kind).expect("two errors");
    }
    assert_eq!(kinds.as_slice(), &[Kind::Decode(Decode::Trailing), Kind::Decode(Decode::Unexpected)]);
}
#[test]
fn errors_past_the_bound_are_counted_not_kept() {
    let (kept, left_out) = fixtures::collector(3).finish();
    assert_eq!(kept.len(), 3);
    assert_eq!(left_out, 1);
}
const FIRST_THREE: &[Spot<'_>] = &[(b"a", 1, 1, b"t"), (b"b", 1, 1, b"t"), (b"c", 1, 1, b"t")];
#[test]
fn a_smaller_error_displaces_the_last_kept_one() {
    let (kept, _) = fixtures::collector(3).finish();
    assert_eq!(fixtures::places(&kept).as_ref(), FIRST_THREE);
}
#[test]
fn a_left_out_operational_error_still_makes_the_call_try_again() {
    let mut call = crate::call::Side::new(copy_of(b"/repo"), Box::new([]), &fixtures::limits_with_errors(0));
    let mut error = fixtures::unplaced(b"t");
    error.class = Class::Operational;
    call.errors.push(error);
    assert_eq!(call.answer(), Answer::Rendered { outcome: Outcome::TryAgain, diff: None, left_out: 1 });
}
const BOUNDS: &[u32] = &[0, 1, 3, 8];
#[test]
fn the_collector_never_holds_more_than_its_bound() {
    for &bound in BOUNDS {
        let mut collector = Collector::new(&fixtures::limits_with_errors(bound));
        for &line in &[9, 3, 8, 1, 7, 2, 6, 4, 5] {
            collector.push(fixtures::error_at(b"a", line, 1, b"t"));
            assert!(collector.errors().len() <= usize::try_from(bound).expect("small bound"), "bound {bound}");
        }
    }
}
