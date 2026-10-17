use super::fixtures;
use crate::{
    limits,
    vocabulary::{Answer, Decline, Event, Output, Request, Step},
};
use skein_lib::Time;
#[test]
fn a_find_of_an_unindexed_key_is_absent_without_a_load() {
    let mut store = fixtures::ready(limits::SHIPPED, &[]);
    assert_eq!(&*store.step(Time::ZERO, Event::Find { key: fixtures::KEY }), &[Output::Answer(Answer::Absent)]);
}
#[test]
fn a_find_of_an_indexed_key_loads_the_record_whole() {
    let mut store = fixtures::resolved_ready();
    assert_eq!(
        &*store.step(Time::ZERO, Event::Find { key: fixtures::KEY }),
        &[Output::Request(Request::Load { key: fixtures::KEY })]
    );
    let record = fixtures::resolution_record();
    assert_eq!(
        &*store.step(Time::ZERO, Event::Resolution { record: record.clone() }),
        &[Output::Answer(Answer::Found { resolution: record })]
    );
}
#[test]
fn a_find_without_an_index_is_declined() {
    let mut store = fixtures::full();
    assert_eq!(
        &*store.step(Time::ZERO, Event::Find { key: fixtures::KEY }),
        &[Output::Answer(Answer::Declined { why: Decline::Unusable { step: Step::List } })]
    );
}
#[test]
fn an_unreadable_find_is_absent() {
    let mut store = fixtures::resolved_ready();
    drop(store.step(Time::ZERO, Event::Find { key: fixtures::KEY }));
    assert_eq!(&*store.step(Time::ZERO, Event::Unreadable), &[Output::Answer(Answer::Absent)]);
}

#[test]
fn a_record_is_written_then_indexed() {
    let mut store = fixtures::ready(fixtures::ONE_RESOLUTION, &[]);
    assert_eq!(
        &*store.step(Time::ZERO, Event::Record { resolution: fixtures::resolution_record() }),
        &[Output::Request(Request::Write { resolution: fixtures::resolution_record() })]
    );
    assert!(!store.resolved(&fixtures::KEY));
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Answer(Answer::Recorded)]);
    assert!(store.resolved(&fixtures::KEY));
}
#[test]
fn a_record_past_the_resolution_limit_is_declined_and_not_written() {
    let mut store = fixtures::recorded_ready();
    assert_eq!(
        &*store.step(Time::ZERO, Event::Record { resolution: fixtures::resolution_record() }),
        &[Output::Answer(Answer::Declined {
            why: Decline::Full {
                limit: crate::vocabulary::Limit::Resolutions,
                most: fixtures::ONE_RESOLUTION.resolutions
            }
        })]
    );
}
#[test]
fn a_record_the_writer_refuses_declines_and_the_store_stays_usable() {
    let mut store = fixtures::ready(fixtures::ONE_RESOLUTION, &[]);
    drop(store.step(Time::ZERO, Event::Record { resolution: fixtures::resolution_record() }));
    assert_eq!(
        &*store.step(Time::ZERO, Event::Unwritable { max: fixtures::RECORD_BOUND }),
        &[Output::Answer(Answer::Declined { why: Decline::Bound { max: fixtures::RECORD_BOUND } })]
    );
    assert_eq!(&*store.step(Time::ZERO, Event::Room), &[Output::Answer(Answer::HasRoom)]);
}
const ROOM: &[(bool, bool, Answer)] = &[
    (true, false, Answer::HasRoom),
    (
        true,
        true,
        Answer::Declined {
            why: Decline::Full {
                limit: crate::vocabulary::Limit::Resolutions,
                most: fixtures::ONE_RESOLUTION.resolutions,
            },
        },
    ),
    (false, false, Answer::Declined { why: Decline::Unusable { step: Step::List } }),
];
#[test]
fn room_follows_the_resolution_limit() {
    for (row, (usable, recorded, expected)) in ROOM.iter().enumerate() {
        let mut store = fixtures::room_store(*usable, *recorded);
        assert_eq!(&*store.step(Time::ZERO, Event::Room), &[Output::Answer(expected.clone())], "row {row}");
    }
}
