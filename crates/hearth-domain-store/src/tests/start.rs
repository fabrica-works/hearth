use super::fixtures::{self, Harness};
use crate::{
    limits,
    vocabulary::{Answer, Directory, Event, Output, Range, Request, StoreRecord, Why},
};
use skein_lib::{Time, Wall};
#[test]
fn a_busy_lock_is_asked_again_after_200_ms_and_no_sooner() {
    let mut store = Harness::new(limits::SHIPPED, 1000);
    assert_eq!(&*store.step(Time::ZERO, Event::Open), &[Output::Request(Request::Lock)]);
    assert!(store.step(Time::ZERO, Event::Busy).is_empty());
    let retry = Time::from_nanos(200_000_000);
    assert_eq!(store.next_deadline(), Some(Time::from_nanos(200_000_000)));
    assert!(store.fire(Time::from_nanos(199_999_999)).is_empty());
    assert_eq!(&*store.fire(retry), &[Output::Request(Request::Lock)]);
    assert_eq!(&*store.step(retry, Event::Locked), &[Output::Request(Request::LoadRecord)]);
    assert_eq!(store.next_deadline(), None);
}
#[test]
fn opening_the_store_asks_for_its_lock_first() {
    let mut store = Harness::new(limits::SHIPPED, 1000);
    assert_eq!(&*store.step(Time::ZERO, Event::Open), &[Output::Request(Request::Lock)]);
}
#[test]
fn an_absent_or_unreadable_record_is_written_as_layout_1_by_its_user() {
    let mut store = fixtures::recording();
    assert_eq!(
        &*store.step_at(Time::ZERO, Wall::from_nanos(77), Event::Loaded { record: None }),
        &[Output::Request(Request::WriteRecord {
            record: StoreRecord { layout: 1, made: Wall::from_nanos(77), range: None, uid: 1000 }
        })]
    );
    assert_eq!(
        &*store.step(Time::ZERO, Event::Done),
        &[Output::Request(Request::List { directory: Directory::Top, max: 1_024 })]
    );
}
#[test]
fn a_readable_record_of_layout_2_is_written_again_as_layout_1() {
    let mut store = fixtures::recording();
    let record = StoreRecord { layout: 2, made: Wall::EPOCH, range: None, uid: 1000 };
    assert_eq!(
        &*store.step_at(Time::ZERO, Wall::from_nanos(77), Event::Loaded { record: Some(record) }),
        &[Output::Request(Request::WriteRecord {
            record: StoreRecord { layout: 1, made: Wall::from_nanos(77), range: None, uid: 1000 }
        })]
    );
}
#[test]
fn a_readable_record_is_kept_with_its_range_and_the_store_is_ready() {
    let mut store = fixtures::recording();
    let record = StoreRecord {
        layout: 1,
        made: Wall::from_nanos(77),
        range: Some(Range { base: 100_000, count: 65_536 }),
        uid: 1000,
    };
    assert_eq!(
        &*store.step(Time::ZERO, Event::Loaded { record: Some(record) }),
        &[Output::Request(Request::List { directory: Directory::Top, max: 1_024 })]
    );
}
const FAILURES: &[(bool, bool, Why)] =
    &[(false, false, Why::Lock), (true, false, Why::Record), (true, true, Why::Record)];
#[test]
fn a_failed_lock_or_record_leaves_the_store_unopened() {
    for &(locked, writing, why) in FAILURES {
        let mut store = Harness::new(limits::SHIPPED, 1000);
        drop(store.step(Time::ZERO, Event::Open));
        if locked {
            drop(store.step(Time::ZERO, Event::Locked));
        }
        if writing {
            drop(store.step(Time::ZERO, Event::Loaded { record: None }));
        }
        assert_eq!(
            &*store.step(Time::ZERO, Event::Failed),
            &[Output::Answer(Answer::Unopened { why })],
            "{locked} {writing}"
        );
        assert_eq!(store.next_deadline(), None);
    }
}
#[test]
fn a_record_of_another_uid_leaves_the_store_unopened_naming_both() {
    let mut store = fixtures::recording();
    let record = StoreRecord { layout: 1, made: Wall::EPOCH, range: None, uid: 1001 };
    assert_eq!(
        &*store.step(Time::ZERO, Event::Loaded { record: Some(record) }),
        &[Output::Answer(Answer::Unopened { why: Why::Owner { recorded: 1001, builder: 1000 } })]
    );
}

#[test]
fn the_wait_is_said_once_a_second_after_the_first_busy() {
    let mut store = fixtures::waiting();
    assert_eq!(store.next_deadline(), Some(Time::from_nanos(1_000_000_000)));
    assert!(store.fire(Time::from_nanos(999_999_999)).is_empty());
    assert_eq!(&*store.fire(Time::from_nanos(1_000_000_000)), &[Output::Answer(Answer::Waiting)]);
    assert!(store.fire(Time::from_nanos(1_000_000_001)).is_empty());
    drop(store.step(Time::from_nanos(1_000_000_001), Event::Busy));
    assert_eq!(store.next_deadline(), Some(Time::from_nanos(3_000_000_001)));
}
#[test]
fn a_lock_taken_within_the_second_says_nothing() {
    let mut store = fixtures::waiting();
    drop(store.step(Time::from_nanos(500_000_000), Event::Locked));
    assert_eq!(store.next_deadline(), None);
    assert!(store.fire(Time::from_nanos(1_000_000_000)).is_empty());
}
