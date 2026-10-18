use super::fixtures::{self, KEY_A, KEY_B, RunCause};
use crate::{
    errors::{Kind, Resolution},
    keys, limits,
    vocabulary::{Answer, Class, Failure, Request},
};
use skein_lib::{List, Queue, Token};
#[test]
fn needed_keys_are_resolved_one_at_a_time_in_target_order() {
    let mut side = fixtures::resolving_side();
    let Some(Request::Resolve { key, .. }) = fixtures::next_resolution(&mut side) else { panic!("first resolve") };
    assert_eq!(key, KEY_A);
    drop(fixtures::record_side(&mut side));
    side.recorded(None);
    let Some(Request::Resolve { key, .. }) = fixtures::next_resolution(&mut side) else { panic!("second resolve") };
    assert_eq!(key, KEY_B);
    drop(fixtures::record_side(&mut side));
    side.recorded(None);
    assert_eq!(fixtures::next_resolution(&mut side), None);
}
#[test]
fn a_failed_key_leaves_the_later_keys_resolved() {
    let mut side = fixtures::first_resolving_side();
    side.resolve_failed(fixtures::run_failure(RunCause::Fetch));
    let Some(Request::Resolve { key, .. }) = fixtures::next_resolution(&mut side) else { panic!("later key") };
    assert_eq!(key, KEY_B);
    drop(fixtures::record_side(&mut side));
    side.recorded(None);
    let mut out = Queue::with_capacity(1);
    drop(side.resolved_answer(&mut out));
    let Some(Answer::Spec(spec)) = side.resolved_answer(&mut out) else { panic!("b's spec") };
    let Some(keys::Resolution::Known { key, .. }) = spec.resolution.as_deref() else { panic!("recorded B") };
    assert_eq!(*key, KEY_B);
    assert_eq!(fixtures::places(&side.errors.errors()).as_ref(), fixtures::FAILED_HOLDERS);
}
#[test]
fn without_resolve_no_key_is_resolved() {
    let mut side = fixtures::resolving_side();
    side.finding.resolve(None);
    assert_eq!(fixtures::next_resolution(&mut side), None);
}
#[test]
fn a_failed_run_is_an_error_of_each_target_holding_its_key() {
    let mut side = fixtures::first_resolving_side();
    side.resolve_failed(fixtures::run_failure(RunCause::Fetch));
    for error in side.errors.errors() {
        assert_eq!(error.at, None);
    }
    assert_eq!(fixtures::places(&side.errors.errors()).as_ref(), fixtures::FAILED_HOLDERS);
}
#[test]
fn a_run_too_large_is_the_resolution_error_too_many() {
    let mut side = fixtures::first_resolving_side();
    side.resolve_failed((
        Class::Configuration,
        Failure::TooLarge { limit: crate::errors::Limit::Closure, max: u64::from(limits::SHIPPED.closure) },
    ));
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
        side.resolve_failed(fixtures::run_failure(cause));
        for error in side.errors.errors() {
            let name = match error.kind {
                Kind::Resolution(_) => "resolution",
                Kind::Tool(_) => "tool",
                Kind::Reserved(_)
                | Kind::Ids(_)
                | Kind::Exported(_)
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
    side.resolve_failed(fixtures::run_failure(RunCause::Fetch));
    assert_eq!(
        fixtures::places(&side.errors.errors()).as_ref(),
        [(b"z.json".as_slice(), 1, 1, b"z".as_slice()), (b"", 0, 0, b"a"), (b"", 0, 0, b"c")]
    );
}

#[test]
fn a_package_takes_its_first_lines_stanza_and_a_later_one_must_agree() {
    let mut closure = fixtures::matched_apt_closure();
    let mut later = fixtures::resolution_stanza(b"apt", b"amd64");
    later.filename = b"elsewhere/apt.deb".as_slice().into();
    assert_eq!(closure.stanza(1, later), Ok(()));
    let packages = closure.close(&[]).expect("matched closure");
    let package = fixtures::only(&packages);
    assert_eq!((package.source, package.filename.as_ref()), (0, b"pool/main/a/apt/apt_3.3.3_amd64.deb".as_slice()));
    assert_eq!(keys::closure(&packages), fixtures::APT_CLOSURE);
}
#[test]
fn a_package_found_only_in_a_later_line_takes_that_lines_place() {
    let mut closure = fixtures::apt_closure();
    closure.stanza(0, fixtures::resolution_stanza(b"curl", b"amd64")).expect("unrelated stanza");
    closure.stanza(1, fixtures::resolution_stanza(b"apt", b"amd64")).expect("matching stanza");
    assert_eq!(fixtures::only(&closure.close(&[]).expect("closure")).source, 1);
}
const INSTALLED: &[(&[u8], &[u8])] = &[(b"apt-utils", b"amd64"), (b"apt", b"amd64"), (b"apt", b"all")];
#[test]
fn the_closure_is_sorted_by_name_then_architecture() {
    let mut closure = crate::resolution::Closure::new(&limits::SHIPPED);
    for &(name, architecture) in INSTALLED {
        closure.inst(fixtures::resolution_inst(name, architecture)).expect("triple");
    }
    for &(name, architecture) in INSTALLED {
        closure.stanza(0, fixtures::resolution_stanza(name, architecture)).expect("stanza");
    }
    let packages = closure.close(&[]).expect("closure");
    let mut names = List::with_capacity(limits::SHIPPED.closure);
    for package in &packages {
        names.push((package.name.as_ref(), package.architecture.as_ref())).expect("closure");
    }
    assert_eq!(
        names.as_slice(),
        [(b"apt".as_slice(), b"all".as_slice()), (b"apt", b"amd64"), (b"apt-utils", b"amd64")]
    );
}
#[test]
fn a_packages_filename_and_size_enter_the_record_and_not_the_digest() {
    let mut closure = fixtures::apt_closure();
    let mut stanza = fixtures::resolution_stanza(b"apt", b"amd64");
    stanza.filename = b"pool/changed.deb".as_slice().into();
    stanza.size = 42;
    closure.stanza(0, stanza).expect("stanza");
    let packages = closure.close(&[]).expect("closure");
    let package = fixtures::only(&packages);
    assert_eq!((package.filename.as_ref(), package.size), (b"pool/changed.deb".as_slice(), 42));
    assert_eq!(keys::closure(&packages), fixtures::APT_CLOSURE);
}
#[test]
fn an_inst_triple_with_no_stanza_is_unmatched() {
    assert_eq!(
        fixtures::apt_closure().close(&[]),
        Err(Resolution::Unmatched {
            name: b"apt".as_slice().into(),
            version: b"3.3.3".as_slice().into(),
            architecture: b"amd64".as_slice().into(),
        })
    );
}
const DISAGREEING: &[(u64, hearth_lib::encode::Digest)] =
    &[(1_567_913, fixtures::APT_SHA256), (1_567_912, hearth_lib::encode::Digest([0; 32]))];
#[test]
fn stanzas_disagreeing_on_size_or_digest_fail() {
    for (row, &(size, sha256)) in DISAGREEING.iter().enumerate() {
        let mut closure = fixtures::matched_apt_closure();
        let mut stanza = fixtures::resolution_stanza(b"apt", b"amd64");
        stanza.size = size;
        stanza.sha256 = sha256;
        assert_eq!(
            closure.stanza(1, stanza),
            Err(Resolution::Disagree {
                name: b"apt".as_slice().into(),
                version: b"3.3.3".as_slice().into(),
                architecture: b"amd64".as_slice().into(),
            }),
            "row {row}"
        );
    }
}
#[test]
fn a_requested_package_outside_the_closure_is_missing() {
    assert_eq!(
        fixtures::matched_apt_closure().close(&[b"curl".as_slice().into()]),
        Err(Resolution::Missing { package: b"curl".as_slice().into() })
    );
}
#[test]
fn a_closure_past_its_limit_is_too_many_at_a_small_limit() {
    let mut closure = crate::resolution::Closure::new(&fixtures::SMALL_CLOSURE);
    for &(name, architecture) in
        INSTALLED.iter().take(usize::try_from(fixtures::SMALL_CLOSURE.closure).expect("small limit"))
    {
        closure.inst(fixtures::resolution_inst(name, architecture)).expect("within bound");
    }
    assert_eq!(
        closure.inst(fixtures::resolution_inst(b"curl", b"all")),
        Err(Resolution::TooMany { max: fixtures::SMALL_CLOSURE.closure })
    );
}
#[test]
fn a_repeated_inst_line_is_one_package() {
    let mut closure = fixtures::matched_apt_closure();
    closure.inst(fixtures::resolution_inst(b"apt", b"amd64")).expect("repeated triple");
    assert_eq!(closure.close(&[]).expect("closure").len(), 1);
}
#[test]
fn an_unmatched_closure_is_an_operational_error_of_each_holder() {
    let mut side = fixtures::first_resolving_side();
    side.finding.inst(fixtures::resolution_inst(b"apt", b"amd64"));
    drop(fixtures::record_side(&mut side));
    for error in side.errors.errors() {
        assert_eq!(error.class, Class::Operational);
    }
    assert_eq!(side.errors.errors().len(), 2);
}

#[test]
fn a_closed_closure_is_asked_recorded_with_its_digest_and_inputs() {
    let mut side = fixtures::first_resolving_side();
    side.finding.inst(fixtures::resolution_inst(b"apt", b"amd64"));
    side.finding.stanza(0, fixtures::resolution_stanza(b"apt", b"amd64"));
    let Some(Request::Record { request, resolution }) = fixtures::record_side(&mut side) else { panic!("record") };
    assert_eq!(request, Token::new(7));
    assert_eq!((resolution.key, resolution.digest), (KEY_A, fixtures::APT_CLOSURE));
    assert_eq!(resolution.inputs.request.as_ref(), [hearth_lib::encode::Item::List(0)]);
    assert_eq!(resolution.inputs.variant.as_ref(), [hearth_lib::encode::Item::Bytes(b"apt".as_slice().into())]);
}
#[test]
fn once_recorded_every_target_holding_the_key_is_known() {
    let mut side = fixtures::first_resolving_side();
    drop(fixtures::record_side(&mut side));
    side.recorded(None);
    drop(fixtures::next_resolution(&mut side));
    side.resolve_failed(fixtures::run_failure(RunCause::Fetch));
    let mut names = List::with_capacity(3);
    let mut out = Queue::with_capacity(1);
    for _ in 0_u32..3 {
        if let Some(Answer::Spec(spec)) = side.resolved_answer(&mut out)
            && let Some(keys::Resolution::Known { key, .. }) = spec.resolution.as_deref()
        {
            names.push((spec.meta.target.clone(), *key)).expect("fixture targets");
        }
    }
    assert_eq!(names.as_slice(), [(b"a".as_slice().into(), KEY_A), (b"c".as_slice().into(), KEY_A)]);
}
#[test]
fn a_closure_past_its_limit_is_never_recorded() {
    let mut side = fixtures::resolving_side();
    side.limits = fixtures::SMALL_CLOSURE;
    drop(fixtures::next_resolution(&mut side));
    for name in [b"apt".as_slice(), b"curl", b"dpkg"] {
        side.finding.inst(fixtures::resolution_inst(name, b"amd64"));
    }
    assert_eq!(fixtures::record_side(&mut side), None);
    for error in side.errors.errors() {
        assert_eq!(error.kind, Kind::Resolution(Resolution::TooMany { max: fixtures::SMALL_CLOSURE.closure }));
    }
}
