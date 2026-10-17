use super::fixtures::{self, ARRIVAL, D1, MOTD};
use crate::{
    limits,
    vocabulary::{Answer, CacheFile, Event, Limit, Origin, Output, Request, Step},
};
use skein_lib::{Duration, Time, Wall};
#[test]
fn a_new_file_is_kept_with_its_record_and_answered_cached() {
    let mut store = fixtures::ready(limits::SHIPPED, &[]);
    let wall = Wall::from_nanos(1_760_000_000_000_000_000);
    let outputs = store.step_at(Time::ZERO, wall, fixtures::blob(ARRIVAL, MOTD));
    assert_eq!(
        &*outputs,
        &[Output::Request(Request::Keep {
            arrival: ARRIVAL,
            record: CacheFile {
                at: wall,
                origin: Origin::Blob { oid: b"d23c59a84714a16a9597f6d92ce69390d6d13f9b".as_slice().into() },
                sha256: MOTD,
                size: 15,
                took: Duration::from_millis(3)
            }
        })]
    );
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Answer(Answer::Cached { arrival: ARRIVAL })]);
    assert!(store.indexed(&MOTD));
}
#[test]
fn a_file_already_indexed_discards_its_temporary() {
    let mut store = fixtures::ready(limits::SHIPPED, core::slice::from_ref(&MOTD));
    assert_eq!(
        &*store.step(Time::ZERO, fixtures::blob(ARRIVAL, MOTD)),
        &[Output::Request(Request::Discard { arrival: ARRIVAL })]
    );
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Answer(Answer::Cached { arrival: ARRIVAL })]);
}
#[test]
fn a_file_past_the_cache_limit_is_refused_and_discarded() {
    let mut store = fixtures::ready(limits::Limits { cache_files: 1, ..limits::SHIPPED }, core::slice::from_ref(&D1));
    assert_eq!(
        &*store.step(Time::ZERO, fixtures::blob(ARRIVAL, MOTD)),
        &[Output::Request(Request::Discard { arrival: ARRIVAL })]
    );
    assert_eq!(
        &*store.step(Time::ZERO, Event::Done),
        &[
            Output::Answer(Answer::Full { limit: Limit::CacheFiles, most: 1 }),
            Output::Answer(Answer::Refused { arrival: ARRIVAL })
        ]
    );
    assert!(store.indexed(&D1));
}
#[test]
fn a_full_store_refuses_every_arrival() {
    let mut store = fixtures::full();
    assert_eq!(
        &*store.step(Time::ZERO, fixtures::blob(ARRIVAL, MOTD)),
        &[Output::Request(Request::Discard { arrival: ARRIVAL })]
    );
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Answer(Answer::Refused { arrival: ARRIVAL })]);
}
#[test]
fn a_failed_keep_removes_its_index_entry_and_refuses_later_arrivals() {
    let mut store = fixtures::ready(limits::SHIPPED, &[]);
    drop(store.step(Time::ZERO, fixtures::blob(ARRIVAL, MOTD)));
    assert_eq!(
        &*store.step(Time::ZERO, Event::Failed),
        &[Output::Answer(Answer::Unusable { step: Step::Keep }), Output::Answer(Answer::Refused { arrival: ARRIVAL })]
    );
    assert!(!store.indexed(&MOTD));
    assert_eq!(
        &*store.step(Time::ZERO, fixtures::blob(ARRIVAL, MOTD)),
        &[Output::Request(Request::Discard { arrival: ARRIVAL })]
    );
}
