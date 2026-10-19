use super::fixtures::{self, D1};
use crate::{
    limits,
    vocabulary::{Answer, Directory, Event, Layer, LayerInputs, Limit, Output, Request, Shape, Stray},
};
use skein_lib::Time;
fn listed(store: &mut fixtures::Harness, records: &[Layer]) -> Box<[Output]> {
    let mut entries = skein_lib::List::with_capacity(2);
    for record in records {
        entries.push(fixtures::entry(&fixtures::record_name(record.key), Shape::Record(record.key))).expect("two");
    }
    store.page(entries.as_slice(), true)
}
fn settled(store: &mut fixtures::Harness) -> Box<[Output]> {
    drop(store.page(&[], true));
    drop(store.page(&[], true));
    store.step(Time::ZERO, Event::Done)
}
#[test]
fn the_scan_indexes_a_record_whose_key_recomputes() {
    let mut store = fixtures::layering(limits::SHIPPED);
    let record = fixtures::packages_layer();
    assert_eq!(&*listed(&mut store, &[record]), &[Output::Request(Request::LoadLayer { key: record.key })]);
    assert_eq!(
        &*store.step(Time::ZERO, Event::Layer { record }),
        &[Output::Request(Request::List { directory: Directory::Cache, max: 1_024 })],
        "8.2, step 6: then the cache"
    );
    assert_eq!(&*settled(&mut store), &[Output::Answer(Answer::Ready)]);
    assert!(store.published(&record.key), "indexed by its key");
    assert_eq!(
        &*store.step(Time::ZERO, Event::Want { key: record.key }),
        &[Output::Answer(Answer::Have { key: record.key })]
    );
    assert_eq!(&*store.step(Time::ZERO, Event::Want { key: D1 }), &[Output::Answer(Answer::Free { key: D1 })]);
}
/// Records whose key does not recompute from kind, parent and inputs (6.2).
fn damaged() -> [Layer; 4] {
    let good = fixtures::packages_layer();
    [
        Layer { inputs: LayerInputs::Packages { part: D1, resolution: D1 }, ..good },
        Layer { parent: None, ..good },
        Layer { inputs: LayerInputs::Software { part: D1 }, ..good },
        Layer { parent: Some(D1), ..fixtures::empty_layer() },
    ]
}
#[test]
fn a_record_whose_key_does_not_recompute_is_skipped_and_reported() {
    for record in damaged() {
        let mut store = fixtures::layering(limits::SHIPPED);
        drop(listed(&mut store, &[record]));
        drop(store.step(Time::ZERO, Event::Layer { record }));
        assert_eq!(
            &*settled(&mut store),
            &[
                Output::Answer(Answer::Strays {
                    count: 1,
                    first: Stray { directory: Directory::Layers, name: fixtures::record_name(record.key) }
                }),
                Output::Answer(Answer::Ready)
            ],
            "{record:?}"
        );
        assert!(!store.published(&record.key), "{record:?}");
    }
}
#[test]
fn an_unreadable_record_is_skipped_and_reported() {
    let mut store = fixtures::layering(limits::SHIPPED);
    let record = fixtures::packages_layer();
    drop(listed(&mut store, &[record]));
    drop(store.step(Time::ZERO, Event::Unreadable));
    assert_eq!(settled(&mut store).len(), 2, "strays, then ready");
    assert!(!store.published(&record.key));
}
#[test]
fn the_record_past_the_layer_limit_stops_the_scan_full() {
    let mut store = fixtures::layering(limits::Limits { layers: 1, ..limits::SHIPPED });
    let records = [fixtures::empty_layer(), fixtures::packages_layer()];
    drop(listed(&mut store, &records));
    assert_eq!(
        &*store.step(Time::ZERO, Event::Layer { record: records[0] }),
        &[Output::Request(Request::LoadLayer { key: records[1].key })]
    );
    assert_eq!(
        &*store.step(Time::ZERO, Event::Layer { record: records[1] }),
        &[Output::Answer(Answer::Full { limit: Limit::Layers, most: 1 })],
        "8.1: past its limit the store is full"
    );
    assert_eq!(
        &*store.step(Time::ZERO, Event::Want { key: records[0].key }),
        &[Output::Answer(Answer::Free { key: records[0].key })],
        "a full store holds no index"
    );
}
