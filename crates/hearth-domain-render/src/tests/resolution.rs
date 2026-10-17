use super::fixtures::{self, KEY_A, KEY_B, RunCause};
use crate::{
    errors::{Kind, Resolution},
    keys, limits,
    vocabulary::{Answer, Class, Failure, Request},
};
use skein_lib::{Queue, Token};
#[test]
fn needed_keys_are_resolved_one_at_a_time_in_target_order() {
    let mut side = fixtures::resolving_side();
    let Some(Request::Resolve { key, .. }) = fixtures::next_resolution(&mut side) else { panic!("first resolve") };
    assert_eq!(key, KEY_A);
    side.resolved(None);
    let Some(Request::Resolve { key, .. }) = fixtures::next_resolution(&mut side) else { panic!("second resolve") };
    assert_eq!(key, KEY_B);
    side.resolved(None);
    assert_eq!(fixtures::next_resolution(&mut side), None);
}
#[test]
fn a_failed_key_leaves_the_later_keys_resolved() {
    let mut side = fixtures::resolving_side();
    drop(fixtures::next_resolution(&mut side));
    side.resolved(Some(fixtures::run_failure(RunCause::Fetch)));
    let Some(Request::Resolve { key, .. }) = fixtures::next_resolution(&mut side) else { panic!("later key") };
    assert_eq!(key, KEY_B);
    side.resolved(None);
    let mut out = Queue::with_capacity(1);
    drop(side.resolved_answer(&mut out));
    let Some(Answer::Spec(spec)) = side.resolved_answer(&mut out) else { panic!("b's spec") };
    assert_eq!(spec.resolution.as_deref(), Some(&keys::Resolution::Needed { key: KEY_B }));
    assert_eq!(
        fixtures::places(&side.errors.errors()).as_ref(),
        [(b"".as_slice(), 0, 0, b"a".as_slice()), (b"", 0, 0, b"c")]
    );
}
#[test]
fn without_resolve_no_key_is_resolved() {
    let mut side = fixtures::resolving_side();
    side.finding.resolve(None);
    assert_eq!(fixtures::next_resolution(&mut side), None);
}
#[test]
fn a_failed_run_is_an_error_of_each_target_holding_its_key() {
    let mut side = fixtures::resolving_side();
    drop(fixtures::next_resolution(&mut side));
    side.resolved(Some(fixtures::run_failure(RunCause::Fetch)));
    for error in side.errors.errors() {
        assert_eq!(error.at, None);
    }
    assert_eq!(
        fixtures::places(&side.errors.errors()).as_ref(),
        [(b"".as_slice(), 0, 0, b"a".as_slice()), (b"", 0, 0, b"c")]
    );
}
#[test]
fn a_run_too_large_is_the_resolution_error_too_many() {
    let mut side = fixtures::resolving_side();
    drop(fixtures::next_resolution(&mut side));
    side.resolved(Some((
        Class::Configuration,
        Failure::TooLarge { limit: crate::errors::Limit::Closure, max: u64::from(limits::SHIPPED.closure) },
    )));
    for error in side.errors.errors() {
        assert_eq!(error.kind, Kind::Resolution(Resolution::TooMany { max: fixtures::CLOSURE_LIMIT }));
    }
}
const CAUSES: &[(RunCause, Class, &str)] = &[
    (RunCause::Unknown, Class::Configuration, "resolution"),
    (RunCause::Unsatisfiable, Class::Configuration, "resolution"),
    (RunCause::Absent, Class::Configuration, "resolution"),
    (RunCause::Fetch, Class::Operational, "tool"),
    (RunCause::Resolve, Class::Operational, "tool"),
    (RunCause::Space, Class::Operational, "tool"),
    (RunCause::Start, Class::Operational, "tool"),
    (RunCause::Timeout, Class::Operational, "tool"),
    (RunCause::Other, Class::Configuration, "tool"),
];
#[test]
fn each_cause_takes_its_kind_and_class() {
    for (row, &(cause, class, kind)) in CAUSES.iter().enumerate() {
        let mut side = fixtures::resolving_side();
        drop(fixtures::next_resolution(&mut side));
        side.resolved(Some(fixtures::run_failure(cause)));
        for error in side.errors.errors() {
            let name = match error.kind {
                Kind::Resolution(_) => "resolution",
                Kind::Tool(_) => "tool",
                Kind::Exported(_)
                | Kind::Guard(_)
                | Kind::Collision(_)
                | Kind::Store(_)
                | Kind::Evaluation(_)
                | Kind::Schema(_)
                | Kind::Decode(_)
                | Kind::Repository(_)
                | Kind::Composition(_) => panic!("run error kind"),
            };
            assert_eq!((name, error.class), (kind, class), "row {row}: {cause:?}");
        }
    }
}
#[test]
fn a_container_target_is_never_asked_to_resolve() {
    let mut side = fixtures::container_resolving_side();
    assert_eq!(fixtures::next_resolution(&mut side), None);
    assert_eq!(side.errors.errors().as_ref(), []);
}
#[test]
fn the_shipped_limits_hold_8192_packages_and_600_s() {
    assert_eq!((limits::SHIPPED.closure, limits::SHIPPED.resolve), (fixtures::CLOSURE_LIMIT, fixtures::RESOLVE_TIME));
}
#[test]
fn a_resolve_carries_the_deadline_now_plus_600_s() {
    let mut side = fixtures::resolving_side_at(fixtures::RESOLVE_START);
    let Some(Request::Resolve { request, deadline, .. }) = fixtures::next_resolution(&mut side) else {
        panic!("resolve")
    };
    assert_eq!((request, deadline), (Token::new(6), fixtures::RESOLVE_DEADLINE));
}

#[test]
fn an_error_with_no_place_sorts_after_placed_errors_by_target() {
    let mut side = fixtures::resolving_side();
    side.errors.push(fixtures::error_at(b"z.json", 1, 1, b"z"));
    drop(fixtures::next_resolution(&mut side));
    side.resolved(Some(fixtures::run_failure(RunCause::Fetch)));
    assert_eq!(
        fixtures::places(&side.errors.errors()).as_ref(),
        [(b"z.json".as_slice(), 1, 1, b"z".as_slice()), (b"", 0, 0, b"a"), (b"", 0, 0, b"c")]
    );
}
