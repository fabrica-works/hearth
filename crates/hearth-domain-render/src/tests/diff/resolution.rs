use crate::{
    diff::{self, Listed, Moved, ResolutionChange},
    tests::fixtures,
    vocabulary::{Answer, Event, Output},
};
#[test]
fn equal_resolution_keys_are_the_same_resolution() {
    assert_eq!(diff::resolution(fixtures::KEY_A, fixtures::KEY_A, None, None), ResolutionChange::Same);
}
#[test]
fn two_recorded_closures_list_packages_added_removed_and_moved() {
    let fixtures::Closures { before, after } = fixtures::changed_closures();
    assert_eq!(
        diff::resolution(fixtures::KEY_A, fixtures::KEY_B, Some(&before), Some(&after)),
        ResolutionChange::Resolved {
            added: Box::new([Listed {
                name: b"vim".as_slice().into(),
                architecture: b"amd64".as_slice().into(),
                version: b"9".as_slice().into()
            }]),
            removed: Box::new([Listed {
                name: b"nano".as_slice().into(),
                architecture: b"amd64".as_slice().into(),
                version: b"8".as_slice().into()
            }]),
            versions_changed: Box::new([Moved {
                name: b"apt".as_slice().into(),
                architecture: b"amd64".as_slice().into(),
                before: b"2".as_slice().into(),
                after: b"3".as_slice().into()
            }])
        }
    );
}
const RECORDED: &[(bool, bool, ResolutionChange)] = &[
    (false, false, ResolutionChange::NewKey),
    (true, false, ResolutionChange::NewKey),
    (false, true, ResolutionChange::NewKey),
];
#[test]
fn a_closure_unrecorded_on_either_side_is_a_new_key() {
    for (before, after, expected) in RECORDED {
        let (mut render, env, mut out) = fixtures::resolution_diff(*before, *after, false);
        assert_eq!(fixtures::diff_resolution(&mut render, &env, &mut out), *expected, "{before}/{after}");
    }
}
#[test]
fn the_rebuild_from_packages_is_known_once_the_later_key_is_recorded() {
    let (mut render, env, mut out) = fixtures::resolution_diff(false, true, false);
    let change = fixtures::next_diff_change(&mut render, &env, &mut out);
    let diff::TargetChange::Changed { rebuild, .. } = change.change else { panic!("changed") };
    assert_eq!(rebuild, fixtures::KNOWN_PACKAGES_REBUILD);
}
#[test]
fn a_failed_resolve_keeps_new_key_and_its_error_follows_the_changes() {
    let (mut render, env, mut out) = fixtures::resolution_diff(false, true, true);
    let (key, _) = fixtures::next_diff_resolve(&mut render, &env, &mut out);
    assert_eq!(key, fixtures::KEY_A);
    fixtures::fail_diff_fetch(&mut render, &env, &mut out);
    assert_eq!(fixtures::diff_resolution(&mut render, &env, &mut out), ResolutionChange::NewKey);
    let Output::Answer(Answer::Error(error)) = fixtures::next_diff_output(&mut render, &env, &mut out) else {
        panic!("error after change")
    };
    assert_eq!(error.side, Some(diff::Side::From));
}
const RESOLUTION_KEYS: &[hearth_lib::encode::Digest] = &[fixtures::KEY_A, fixtures::KEY_B];
#[test]
fn changed_keys_are_resolved_once_from_then_to_and_recorded_before_the_changes() {
    let (mut render, env, mut out) = fixtures::crossed_resolution_diff();
    for key in RESOLUTION_KEYS {
        assert_eq!(fixtures::record_next_diff_resolution(&mut render, &env, &mut out), (*key, *key), "{key:?}");
    }
    let change = fixtures::next_diff_change(&mut render, &env, &mut out);
    let next = fixtures::next_diff_change(&mut render, &env, &mut out);
    assert_eq!((change.target.as_ref(), next.target.as_ref()), (b"x1-prod".as_slice(), b"z1-prod".as_slice()));
}
#[test]
fn a_shared_failed_key_is_each_holders_error_on_both_sides() {
    let (mut render, env, mut out) = fixtures::crossed_resolution_diff();
    for key in RESOLUTION_KEYS {
        let (asked, _) = fixtures::next_diff_resolve(&mut render, &env, &mut out);
        assert_eq!(asked, *key, "{key:?}");
        fixtures::fail_diff_fetch(&mut render, &env, &mut out);
    }
    let answers = fixtures::finish_render(&mut render, &env, &mut out);
    let mut errors = skein_lib::List::with_capacity(env.limits.errors);
    for answer in &answers {
        if let Answer::Error(error) = answer {
            errors.push((error.side, error.target.as_deref())).expect("bounded errors");
        }
    }
    let errors = errors.into_boxed();
    assert_eq!(
        errors.as_ref(),
        &[
            (Some(diff::Side::From), Some(b"x1-prod".as_slice())),
            (Some(diff::Side::To), Some(b"x1-prod".as_slice())),
            (Some(diff::Side::To), Some(b"z1-prod".as_slice())),
            (Some(diff::Side::From), Some(b"z1-prod".as_slice()))
        ]
    );
}
#[test]
fn a_declined_record_makes_the_diff_try_again() {
    let (mut render, env, mut out) = fixtures::resolution_diff(false, true, true);
    let (_, names) = fixtures::next_diff_resolve(&mut render, &env, &mut out);
    fixtures::complete_diff_resolution(&mut render, &env, &mut out, &names);
    drop(out.pop());
    crate::step(
        &mut render,
        &env,
        Event::Failed {
            request: fixtures::RECORD_REQUEST,
            class: crate::vocabulary::Class::Operational,
            at: None,
            failure: crate::vocabulary::Failure::Store,
        },
        &mut out,
    );
    let answers = fixtures::finish_render(&mut render, &env, &mut out);
    let mut outcomes = skein_lib::List::with_capacity(fixtures::DIFF_TERMINALS);
    for answer in &answers {
        if let Answer::Rendered { outcome, .. } = answer {
            outcomes.push(*outcome).expect("one terminal");
        }
    }
    assert_eq!(*fixtures::only(outcomes.as_slice()), crate::vocabulary::Outcome::TryAgain);
}

const DISJOINT_KEYS: &[hearth_lib::encode::Digest] =
    &[fixtures::KEY_A, fixtures::KEY_B, fixtures::KEY_C, fixtures::KEY_D];
#[test]
fn every_changed_targets_keys_are_recorded_before_the_first_change() {
    let (mut render, env, mut out) = fixtures::disjoint_resolution_diff();
    for key in DISJOINT_KEYS {
        assert_eq!(fixtures::record_next_diff_resolution(&mut render, &env, &mut out), (*key, *key), "{key:?}");
    }
    assert_eq!(
        [
            fixtures::next_diff_rebuild(&mut render, &env, &mut out),
            fixtures::next_diff_rebuild(&mut render, &env, &mut out),
        ],
        [
            (b"x1-prod".as_slice().into(), fixtures::KNOWN_PACKAGES_REBUILD),
            (b"z1-prod".as_slice().into(), fixtures::KNOWN_PACKAGES_REBUILD),
        ]
    );
}
