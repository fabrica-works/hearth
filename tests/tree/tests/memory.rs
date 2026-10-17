use hearth_lib::encode;
use hearth_protocol_tree::{
    frame::Kind,
    limits::{self, Limits},
    resolution,
};
use hearth_tree_world::{
    check::{self, Record},
    fixtures, generate, mutate,
};
use skein_heap::{Counting, Meter};
use skein_lib::Rng;

#[global_allocator]
static HEAP: Counting = Counting;

const OVERFLOW: &[(Kind, bool)] = &[(Kind::Store, false), (Kind::CacheFile, false), (Kind::Resolution, true)];
const DEPTH_EXPECTATIONS: &[(Kind, Option<u64>)] =
    &[(Kind::Store, None), (Kind::CacheFile, None), (Kind::Resolution, None)];
const SEEDS: &[u64] = &[0, 1, 2, 3];

fn measure(kind: Kind, frame: &[u8], limits: &Limits, seed: u64) {
    let own = limits::worst_case(kind, limits).expect("fits a u64");
    let held = limits::held(kind, limits).expect("fits a u64");
    let meter = Meter::new();
    meter.start();
    let read = check::read(kind, frame, limits);
    let step = meter.end();
    assert!(step.held() <= held, "record: {} of {held}, seed {seed}", step.held());
    drop(read);
    meter.check(step, own, &(kind, seed));
}

#[test]
fn reading_a_resolution_at_its_limits_holds_no_more_than_its_worst_case() {
    let limits = fixtures::limits();
    let record = Record::Resolution(generate::resolution_at_its_limits(&limits));
    let frame = check::write(&record, &limits);
    let own = limits::worst_case(Kind::Resolution, &limits).expect("fits a u64");
    let held = limits::held(Kind::Resolution, &limits).expect("fits a u64");
    let meter = Meter::new();
    meter.start();
    let read = resolution::read(&frame, &limits).expect("a record at its limits reads");
    let step = meter.end();
    assert!(step.held() <= held, "the record it hands out: {} of {held}", step.held());
    drop(read);
    meter.check(step, own, &"a resolution at its limits, besides its record");
}

#[test]
fn reading_generated_cut_and_mutated_frames_holds_no_more_than_the_worst_case() {
    let limits = fixtures::limits();
    for &seed in SEEDS {
        let mut rng = Rng::new(0x7EE0_4000 + seed);
        for record in generate::one_of_each(&mut rng, &limits) {
            let frame = check::write(&record, &limits);
            measure(record.kind(), &frame, &limits, seed);
            for changed in mutate::frames(&mut rng, &frame) {
                measure(record.kind(), &changed, &limits, seed);
            }
        }
    }
}

#[test]
fn worst_case_and_held_are_none_past_a_u64_for_a_resolution_only() {
    let mut limits = fixtures::limits();
    limits.resolution.lines = u32::MAX;
    limits.resolution.indices = u32::MAX;
    limits.resolution.inputs = u32::MAX;
    for &(kind, expected) in OVERFLOW {
        assert_eq!(limits::worst_case(kind, &limits).is_none(), expected, "own: {kind:?}");
        assert_eq!(limits::held(kind, &limits).is_none(), expected, "held: {kind:?}");
    }
}

#[test]
fn worst_case_and_held_are_none_past_the_encoders_depth() {
    let mut limits = fixtures::limits();
    limits.resolution.lines = u32::MAX;
    limits.resolution.indices = u32::MAX;
    limits.resolution.inputs = u32::MAX;
    limits.depth = encode::DEPTH + 1;
    for &(kind, expected) in DEPTH_EXPECTATIONS {
        assert_eq!(limits::worst_case(kind, &limits), expected, "own: {kind:?}");
        assert_eq!(limits::held(kind, &limits), expected, "held: {kind:?}");
    }
}
