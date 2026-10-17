use super::fixtures::{self, D1, D2, D3};
use crate::{
    limits,
    vocabulary::{Answer, Directory, Event, Limit, Output, Removal, Request, Shape, Step},
};
use skein_lib::Time;
#[test]
fn a_cache_file_without_its_record_is_removed_in_the_second_pass() {
    let mut store = fixtures::scanning(limits::SHIPPED);
    drop(store.page(
        core::slice::from_ref(&fixtures::entry(
            b"0101010101010101010101010101010101010101010101010101010101010101.record",
            Shape::Record(D1),
        )),
        true,
    ));
    let entries = [
        fixtures::entry(b"0101010101010101010101010101010101010101010101010101010101010101", Shape::File(D1)),
        fixtures::entry(b"0202020202020202020202020202020202020202020202020202020202020202", Shape::File(D2)),
    ];
    assert_eq!(
        &*store.page(&entries, true),
        &[Output::Request(Request::Remove { directory: Directory::Cache, name: Removal::File(D2) })]
    );
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Request(Request::Empty)]);
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Answer(Answer::Ready)]);
    assert!(store.indexed(&D1));
    assert!(!store.indexed(&D2));
}
#[test]
fn temporaries_are_removed_as_they_come_and_strays_left() {
    let mut store = fixtures::scanning(limits::SHIPPED);
    let entries = [
        fixtures::entry(b"notes.tmp", Shape::Stray),
        fixtures::entry(b"0101010101010101010101010101010101010101010101010101010101010101.tmp", Shape::Temporary),
    ];
    assert_eq!(
        &*store.page(&entries, false),
        &[Output::Request(Request::Remove {
            directory: Directory::Cache,
            name: Removal::Named(
                b"0101010101010101010101010101010101010101010101010101010101010101.tmp".as_slice().into()
            )
        })]
    );
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Request(Request::Next { max: 1_024 })]);
    assert_eq!(&*store.page(&[], true), &[Output::Request(Request::List { directory: Directory::Cache, max: 1_024 })]);
}
#[test]
fn a_record_without_its_file_is_removed() {
    let mut store = fixtures::scanning(limits::SHIPPED);
    drop(store.page(
        core::slice::from_ref(&fixtures::entry(
            b"0101010101010101010101010101010101010101010101010101010101010101.record",
            Shape::Record(D1),
        )),
        true,
    ));
    assert_eq!(&*store.page(&[], true), &[Output::Request(Request::Empty)]);
    assert_eq!(
        &*store.step(Time::ZERO, Event::Done),
        &[Output::Request(Request::Remove { directory: Directory::Cache, name: Removal::Record(D1) })]
    );
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Answer(Answer::Ready)]);
    assert!(!store.indexed(&D1));
}
#[test]
fn the_recorded_file_past_the_cache_limit_stops_the_scan_full() {
    let mut store = fixtures::scanning(limits::Limits { cache_files: 2, ..limits::SHIPPED });
    drop(store.page(
        &[
            fixtures::entry(
                b"0101010101010101010101010101010101010101010101010101010101010101.record",
                Shape::Record(D1),
            ),
            fixtures::entry(
                b"0202020202020202020202020202020202020202020202020202020202020202.record",
                Shape::Record(D2),
            ),
            fixtures::entry(
                b"0303030303030303030303030303030303030303030303030303030303030303.record",
                Shape::Record(D3),
            ),
        ],
        true,
    ));
    assert_eq!(
        &*store.page(
            &[
                fixtures::entry(b"0101010101010101010101010101010101010101010101010101010101010101", Shape::File(D1)),
                fixtures::entry(b"0202020202020202020202020202020202020202020202020202020202020202", Shape::File(D2)),
                fixtures::entry(b"0303030303030303030303030303030303030303030303030303030303030303", Shape::File(D3))
            ],
            false
        ),
        &[Output::Request(Request::Close)]
    );
    assert_eq!(
        &*store.step(Time::ZERO, Event::Done),
        &[Output::Answer(Answer::Full { limit: Limit::CacheFiles, most: 2 })]
    );
    assert!(!store.indexed(&D1));
}
#[test]
fn a_file_without_its_record_counts_toward_no_limit() {
    let mut store = fixtures::scanning(limits::Limits { cache_files: 2, ..limits::SHIPPED });
    drop(store.page(
        &[
            fixtures::entry(
                b"0101010101010101010101010101010101010101010101010101010101010101.record",
                Shape::Record(D1),
            ),
            fixtures::entry(
                b"0202020202020202020202020202020202020202020202020202020202020202.record",
                Shape::Record(D2),
            ),
        ],
        true,
    ));
    assert_eq!(
        &*store.page(
            &[
                fixtures::entry(b"0101010101010101010101010101010101010101010101010101010101010101", Shape::File(D1)),
                fixtures::entry(b"0202020202020202020202020202020202020202020202020202020202020202", Shape::File(D2)),
                fixtures::entry(b"0303030303030303030303030303030303030303030303030303030303030303", Shape::File(D3))
            ],
            true
        ),
        &[Output::Request(Request::Remove { directory: Directory::Cache, name: Removal::File(D3) })]
    );
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Request(Request::Empty)]);
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Answer(Answer::Ready)]);
}
const FAILURES: &[(bool, Step)] = &[(false, Step::List), (true, Step::Remove)];
#[test]
fn a_failed_scan_leaves_no_index() {
    for &(removing, step) in FAILURES {
        let mut store = fixtures::scanning(limits::SHIPPED);
        if removing {
            drop(store.page(
                core::slice::from_ref(&fixtures::entry(
                    b"0101010101010101010101010101010101010101010101010101010101010101.tmp",
                    Shape::Temporary,
                )),
                true,
            ));
        }
        assert_eq!(&*store.step(Time::ZERO, Event::Failed), &[Output::Answer(Answer::Unusable { step })], "{step:?}");
        assert!(!store.indexed(&D1));
    }
}
#[test]
fn unpaired_records_are_removed_in_digest_order() {
    let mut store = fixtures::scanning(limits::SHIPPED);
    drop(store.page(
        &[
            fixtures::entry(
                b"0303030303030303030303030303030303030303030303030303030303030303.record",
                Shape::Record(D3),
            ),
            fixtures::entry(
                b"0101010101010101010101010101010101010101010101010101010101010101.record",
                Shape::Record(D1),
            ),
            fixtures::entry(
                b"0202020202020202020202020202020202020202020202020202020202020202.record",
                Shape::Record(D2),
            ),
        ],
        true,
    ));
    assert_eq!(
        &*store.page(
            core::slice::from_ref(&fixtures::entry(
                b"0202020202020202020202020202020202020202020202020202020202020202",
                Shape::File(D2)
            )),
            true
        ),
        &[Output::Request(Request::Empty)]
    );
    assert_eq!(
        &*store.step(Time::ZERO, Event::Done),
        &[Output::Request(Request::Remove { directory: Directory::Cache, name: Removal::Record(D1) })]
    );
    assert_eq!(
        &*store.step(Time::ZERO, Event::Done),
        &[Output::Request(Request::Remove { directory: Directory::Cache, name: Removal::Record(D3) })]
    );
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Answer(Answer::Ready)]);
    assert!(store.indexed(&D2));
}
#[test]
fn the_records_index_is_bounded_before_pairing() {
    let mut store = fixtures::scanning(limits::Limits { cache_files: 1, ..limits::SHIPPED });
    let entries = [
        fixtures::entry(b"0101010101010101010101010101010101010101010101010101010101010101.record", Shape::Record(D1)),
        fixtures::entry(b"0202020202020202020202020202020202020202020202020202020202020202.record", Shape::Record(D2)),
        fixtures::entry(b"0303030303030303030303030303030303030303030303030303030303030303.record", Shape::Record(D3)),
    ];
    assert_eq!(&*store.page(&entries, true), &[Output::Answer(Answer::Full { limit: Limit::CacheFiles, most: 1 })]);
    assert!(!store.indexed(&D1));
}

#[test]
fn a_resolution_whose_key_and_digest_recompute_is_indexed() {
    let mut store = fixtures::resolving(limits::SHIPPED, fixtures::KEY);
    assert_eq!(
        &*store.step(Time::ZERO, fixtures::resolution()),
        &[Output::Request(Request::List { directory: Directory::Cache, max: 1_024 })]
    );
    assert!(store.resolved(&fixtures::KEY));
}
#[test]
fn a_resolution_whose_key_does_not_recompute_is_removed() {
    let mut store = fixtures::resolving(limits::SHIPPED, fixtures::KEY);
    let Event::Resolution { key, mut inputs, closure, digest } = fixtures::resolution() else { panic!("resolution") };
    inputs.variant = Box::new([hearth_lib::encode::Item::Bytes(b"minbase".as_slice().into())]);
    assert_eq!(
        &*store.step(Time::ZERO, Event::Resolution { key, inputs, closure, digest }),
        &[Output::Request(Request::Remove { directory: Directory::Resolutions, name: Removal::Record(fixtures::KEY) })]
    );
}
#[test]
fn a_resolution_whose_digest_does_not_recompute_from_its_closure_is_removed() {
    let mut store = fixtures::resolving(limits::SHIPPED, fixtures::KEY);
    let Event::Resolution { key, inputs, closure, .. } = fixtures::resolution() else { panic!("resolution") };
    assert_eq!(
        &*store.step(Time::ZERO, Event::Resolution { key, inputs, closure, digest: D1 }),
        &[Output::Request(Request::Remove { directory: Directory::Resolutions, name: Removal::Record(fixtures::KEY) })]
    );
}
#[test]
fn a_resolution_named_other_than_its_key_is_removed() {
    let mut store = fixtures::resolving(limits::SHIPPED, D2);
    assert_eq!(
        &*store.step(Time::ZERO, fixtures::resolution()),
        &[Output::Request(Request::Remove { directory: Directory::Resolutions, name: Removal::Record(D2) })]
    );
}
#[test]
fn an_unreadable_resolution_is_left_and_not_counted() {
    let mut store = fixtures::resolving(limits::SHIPPED, fixtures::KEY);
    assert_eq!(
        &*store.step(Time::ZERO, Event::Unreadable),
        &[Output::Request(Request::List { directory: Directory::Cache, max: 1_024 })]
    );
    assert!(!store.resolved(&fixtures::KEY));
}
#[test]
fn a_resolution_past_the_limit_makes_the_store_full() {
    let mut store = fixtures::resolving(limits::Limits { resolutions: 0, ..limits::SHIPPED }, fixtures::KEY);
    assert_eq!(
        &*store.step(Time::ZERO, fixtures::resolution()),
        &[Output::Answer(Answer::Full { limit: Limit::Resolutions, most: 0 })]
    );
    assert!(!store.resolved(&fixtures::KEY));
}
const CLOSURES: &[(bool, hearth_lib::encode::Digest)] = &[(false, fixtures::EMPTY_CLOSURE), (true, fixtures::CLOSURE)];
#[test]
fn the_closure_digest_is_h_over_each_packages_four_fields() {
    for &(populated, digest) in CLOSURES {
        let closure = if populated { fixtures::closure() } else { Box::new([]) };
        assert_eq!(crate::resolutions::digest(&closure), digest, "populated {populated}");
    }
}

#[test]
fn strays_are_said_once_with_their_count_and_the_first() {
    let mut store = fixtures::scanning(limits::SHIPPED);
    let entries = [fixtures::entry(b"a", Shape::Stray), fixtures::entry(b"b", Shape::Stray)];
    drop(store.page(&entries, true));
    drop(store.page(&[], true));
    assert_eq!(
        &*store.step(Time::ZERO, Event::Done),
        &[
            Output::Answer(Answer::Strays {
                count: 2,
                first: crate::vocabulary::Stray { directory: Directory::Cache, name: b"a".as_slice().into() }
            }),
            Output::Answer(Answer::Ready),
        ]
    );
}
