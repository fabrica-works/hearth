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
