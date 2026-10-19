use super::fixtures::{self, D1, D2, D3};
use crate::{
    limits::SHIPPED,
    vocabulary::{
        Answer, Cause, Counts, Event, Helped, Helper, Layer, LayerInputs, Output, Record, Renaming, Request, Step,
        TreeFile, Walk,
    },
};
use hearth_lib::encode::Digest;
use skein_lib::{Time, Wall};

const UUID: [u8; 16] = [7; 16];
const PARENT_UUID: [u8; 16] = [8; 16];
const KEY: Digest = Digest([0x42; 32]);
fn walk() -> Walk {
    Walk {
        tree: D1,
        changes: D2,
        tree_file: TreeFile { sha256: D3, size: 99 },
        counts: Counts { bytes: 5, deletes: 1, entries: 4, files: 2, hashed: 5, puts: 3 },
    }
}
fn one(output: Box<[Output]>) -> Output {
    let [output] = &*output else { panic!("one output: {output:?}") };
    output.clone()
}
fn helped(store: &mut fixtures::Harness, answer: Helped) -> Output {
    one(store.step(Time::ZERO, Event::Helped { answer }))
}
fn helper(helper: Helper) -> Output {
    Output::Request(Request::Helper(helper))
}
fn refused(step: Step) -> Output {
    Output::Answer(Answer::Unusable { step })
}
/// A ready store whose index holds the empty layer and a packages layer.
fn published() -> fixtures::Harness {
    let mut store = fixtures::ready(SHIPPED, &[]);
    store.plant(fixtures::empty_layer());
    store.plant(fixtures::packages_layer());
    store
}
/// Stages `KEY` over the packages layer.
fn staged(store: &mut fixtures::Harness) {
    let parent = fixtures::packages_layer().key;
    assert_eq!(
        one(store.step(Time::ZERO, Event::Stage { key: KEY, parent })),
        helper(Helper::Snapshot { parent, staging: KEY })
    );
    assert_eq!(
        helped(store, Helped::Snapshotted { uuid: UUID, parent_uuid: PARENT_UUID }),
        Output::Answer(Answer::Staged { key: KEY })
    );
}
fn sealing() -> Event {
    Event::Seal { key: KEY, parent: fixtures::packages_layer().key, inputs: LayerInputs::Software { part: D3 } }
}
#[test]
fn a_stage_snapshots_a_published_parent_and_answers_staged() {
    let mut store = published();
    staged(&mut store);
    assert!(!store.published(&KEY), "staged is not published (4.2)");
}
#[test]
fn a_stage_of_the_empty_key_first_makes_the_empty_layer_then_snapshots_it() {
    let empty = fixtures::empty_layer();
    let mut store = fixtures::ready(SHIPPED, &[]);
    let wall = Wall::EPOCH;
    assert_eq!(
        one(store.step(Time::ZERO, Event::Stage { key: KEY, parent: empty.key })),
        Output::Request(Request::Create { key: empty.key }),
        "4.1: the directory, as staging"
    );
    assert_eq!(one(store.step(Time::ZERO, Event::Created { uuid: UUID })), helper(Helper::Hash { layer: empty.key }));
    assert_eq!(helped(&mut store, Helped::Hashed(walk())), helper(Helper::Freeze { layer: empty.key }));
    assert_eq!(
        helped(&mut store, Helped::Frozen),
        Output::Request(Request::Rename { key: empty.key, rename: Renaming::Tree })
    );
    let record = Record {
        layer: Layer { key: empty.key, parent: None, inputs: LayerInputs::Empty, tree: D1, uuid: UUID, at: wall },
        walk: walk(),
        parent_uuid: None,
    };
    assert_eq!(
        one(store.step(Time::ZERO, Event::Done)),
        Output::Request(Request::WriteLayer { record: Box::new(record) }),
        "6.2: the record names the walk and the subvolume"
    );
    assert_eq!(
        one(store.step(Time::ZERO, Event::Done)),
        Output::Request(Request::Rename { key: empty.key, rename: Renaming::Publish })
    );
    assert!(!store.published(&empty.key), "not before the rename is done");
    assert_eq!(one(store.step(Time::ZERO, Event::Done)), helper(Helper::Snapshot { parent: empty.key, staging: KEY }));
    assert!(store.published(&empty.key), "the empty layer is published, and indexed");
    assert_eq!(
        helped(&mut store, Helped::Snapshotted { uuid: PARENT_UUID, parent_uuid: UUID }),
        Output::Answer(Answer::Staged { key: KEY })
    );
}
#[test]
fn a_seal_walks_against_the_parent_freezes_records_and_publishes() {
    let parent = fixtures::packages_layer().key;
    let mut store = published();
    staged(&mut store);
    assert_eq!(
        one(store.step_at(Time::ZERO, Wall::EPOCH, sealing())),
        helper(Helper::Walk { layer: KEY, parent: Some(parent) })
    );
    assert_eq!(helped(&mut store, Helped::Walked(walk())), helper(Helper::Freeze { layer: KEY }));
    assert_eq!(
        helped(&mut store, Helped::Frozen),
        Output::Request(Request::Rename { key: KEY, rename: Renaming::Tree })
    );
    let layer = Layer {
        key: KEY,
        parent: Some(parent),
        inputs: LayerInputs::Software { part: D3 },
        tree: D1,
        uuid: UUID,
        at: Wall::EPOCH,
    };
    assert_eq!(
        one(store.step(Time::ZERO, Event::Done)),
        Output::Request(Request::WriteLayer {
            record: Box::new(Record { layer, walk: walk(), parent_uuid: Some(PARENT_UUID) })
        }),
        "the uuids are the snapshot's"
    );
    assert_eq!(
        one(store.step(Time::ZERO, Event::Done)),
        Output::Request(Request::Rename { key: KEY, rename: Renaming::Publish })
    );
    assert_eq!(one(store.step(Time::ZERO, Event::Done)), Output::Answer(Answer::Sealed { key: KEY }));
    assert!(store.published(&KEY), "a later Want finds it");
    assert_eq!(one(store.step(Time::ZERO, Event::Want { key: KEY })), Output::Answer(Answer::Have { key: KEY }));
}
#[test]
fn a_helper_failure_refuses_the_stage_and_the_seal() {
    for cause in [Cause::NotFound, Cause::NoSpace] {
        let mut store = published();
        let parent = fixtures::packages_layer().key;
        drop(store.step(Time::ZERO, Event::Stage { key: KEY, parent }));
        assert_eq!(helped(&mut store, Helped::Failed { cause }), refused(Step::Stage), "{cause:?}");
        staged(&mut store);
        drop(store.step(Time::ZERO, sealing()));
        assert_eq!(helped(&mut store, Helped::Failed { cause }), refused(Step::Seal), "{cause:?}");
    }
}
#[test]
fn a_failed_file_request_refuses_the_chain_it_is_in() {
    let mut store = published();
    staged(&mut store);
    drop(store.step(Time::ZERO, sealing()));
    drop(store.step(Time::ZERO, Event::Helped { answer: Helped::Walked(walk()) }));
    drop(store.step(Time::ZERO, Event::Helped { answer: Helped::Frozen }));
    assert_eq!(one(store.step(Time::ZERO, Event::Failed)), refused(Step::Seal), "the rename failed");
    assert!(!store.published(&KEY), "nothing is published past a failure");
}
#[test]
fn an_answer_the_chain_did_not_ask_for_refuses_it() {
    for answer in [Helped::Thawed, Helped::Applied, Helped::Emptied { removed: 1 }, Helped::Deleted, Helped::Frozen] {
        let mut store = published();
        let parent = fixtures::packages_layer().key;
        drop(store.step(Time::ZERO, Event::Stage { key: KEY, parent }));
        assert_eq!(helped(&mut store, answer), refused(Step::Stage), "{answer:?}");
    }
}
#[test]
fn a_second_stage_is_refused_not_queued_and_so_is_a_seal_of_what_is_not_staged() {
    let parent = fixtures::packages_layer().key;
    let mut store = published();
    assert_eq!(one(store.step(Time::ZERO, sealing())), refused(Step::Seal), "nothing is staged");
    staged(&mut store);
    assert_eq!(
        one(store.step(Time::ZERO, Event::Stage { key: KEY, parent })),
        refused(Step::Stage),
        "4.6: a key in flight is refused, not queued"
    );
    assert_eq!(
        one(store.step(Time::ZERO, Event::Seal { key: D1, parent, inputs: LayerInputs::Software { part: D3 } })),
        refused(Step::Seal),
        "another key is not the staged one"
    );
}
#[test]
fn a_stage_over_a_parent_that_is_not_published_is_refused() {
    let mut store = fixtures::ready(SHIPPED, &[]);
    assert_eq!(one(store.step(Time::ZERO, Event::Stage { key: KEY, parent: D1 })), refused(Step::Stage));
}
