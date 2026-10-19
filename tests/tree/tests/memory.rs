use hearth_lib::encode;
use hearth_protocol_tree::{
    frame::Kind,
    layer,
    limits::{self, Limits},
    resolution,
    tree::{self, Next, Reader},
};
use hearth_tree_world::{
    check::{self, Record},
    fixtures, generate, mutate,
};
use skein_heap::{Counting, Meter};
use skein_lib::Rng;

#[global_allocator]
static HEAP: Counting = Counting;

const OVERFLOW: &[(Kind, bool)] = &[
    (Kind::Store, false),
    (Kind::Layer, false),
    (Kind::Tree, false),
    (Kind::CacheFile, false),
    (Kind::Resolution, true),
];
const DEPTH_EXPECTATIONS: &[(Kind, Option<u64>)] =
    &[(Kind::Store, None), (Kind::Layer, None), (Kind::Tree, None), (Kind::CacheFile, None), (Kind::Resolution, None)];
const SEEDS: &[u64] = &[0, 1, 2, 3];

/// A tree file's reader, item by item: what each step holds of its own, and
/// the item it hands out, against the kind's worst case.
fn measure_tree(frame: &[u8], limits: &Limits, seed: u64) {
    let own = limits::worst_case(Kind::Tree, limits).expect("fits a u64");
    let held = limits::held(Kind::Tree, limits).expect("fits a u64");
    let last = u64::from(hearth_lib::path::PATH);
    let Some(header) = frame.get(..24) else { return };
    let mut reader = Reader::new(limits);
    if reader.header(header.try_into().expect("24 bytes")).is_err() {
        return;
    }
    let meter = Meter::new();
    let mut at = 24_usize;
    for _ in 0..=frame.len() {
        let fed = frame.get(at..).unwrap_or_default();
        meter.start();
        let next = reader.next(fed);
        let step = meter.end();
        assert!(step.held() <= held + last, "item and last path: {} of {held}, seed {seed}, at {at}", step.held());
        let used = match &next {
            Ok(Next::Item(_, used)) => Some(usize::try_from(*used).expect("small")),
            Ok(Next::Need(_)) | Err(_) => None,
        };
        drop(next);
        meter.check(step, own, &(seed, at));
        match used {
            Some(used) if !reader.ended() => at += used,
            Some(_) | None => return,
        }
    }
}
fn measure(kind: Kind, frame: &[u8], limits: &Limits, seed: u64) {
    if kind == Kind::Tree {
        return measure_tree(frame, limits, seed);
    }
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
fn reading_a_layer_at_its_limits_holds_no_more_than_its_worst_case() {
    let limits = fixtures::limits();
    let record = Record::Layer(Box::new(generate::layer_at_its_limits(&limits)));
    let frame = check::write(&record, &limits);
    let own = limits::worst_case(Kind::Layer, &limits).expect("fits a u64");
    let held = limits::held(Kind::Layer, &limits).expect("fits a u64");
    let meter = Meter::new();
    meter.start();
    let read = layer::read(&frame, &limits).expect("a record at its limits reads");
    let step = meter.end();
    assert!(step.held() <= held, "the record it hands out: {} of {held}", step.held());
    drop(read);
    meter.check(step, own, &"a layer at its limits, besides its record");
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

#[test]
fn reading_a_tree_file_at_its_limits_holds_no_more_than_its_worst_case() {
    let limits = fixtures::limits();
    let items = [generate::entry_at_its_limits(&limits), generate::end_at_its_limits(&limits)];
    let file = check::write_tree(&items, &limits);
    assert!(file.len() > 4_000_000, "the entry is large: {}", file.len());
    measure_tree(&file, &limits, 0);
    assert_eq!(check::read_tree(&file, &limits).as_deref(), Ok(items.as_slice()), "read whole");
}

mod walk {
    use super::HEAP;
    use hearth_domain_tree::{
        MAX_OUT, Tree,
        limits::{Limits, SHIPPED, worst_case},
        step,
        vocabulary::{Event, Kind, Output},
    };
    use hearth_lib::encode::Digest;
    use hearth_tree_world::{
        fs::{Fs, Node},
        helper::{self, Helper},
        layers,
    };
    use skein_heap::Meter;
    use skein_lib::{Env, Queue, Rng, Time, Wall};

    const KEY: Digest = Digest([0x42; 32]);
    const _: &skein_heap::Counting = &HEAP;

    /// Walks `fs` with the machine metered step by step, and returns the most it held at once of
    /// its own: its state, and the event it was given.
    fn peak(fs: &Fs, limits: &Limits, what: &dyn core::fmt::Debug) -> u64 {
        let bound = worst_case(limits).expect("the limits fit a u64");
        // The file has its room before the meter, so that the helper's growth is not the walk's.
        let reserve = helper::walk(fs, limits, KEY).file.len() + 64;
        let mut helper = Helper::reserving(fs, reserve);
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
        let meter = Meter::new();
        let mut machine = Tree::new(limits);
        let mut event = Event::Hash { key: KEY };
        let mut most = 0;
        loop {
            let mut out = Queue::with_capacity(MAX_OUT);
            meter.start();
            step(&mut machine, &env, event, &mut out);
            let step = meter.end();
            let (next, done) = match out.pop().expect("a step has an output") {
                Output::Request(request) => {
                    let next = helper.answer(&request);
                    drop(request);
                    (Some(next), false)
                }
                Output::Answer(answer) => {
                    if let hearth_domain_tree::vocabulary::Answer::Failed { cause, .. } = &answer {
                        panic!("{what:?}: the walk failed: {cause:?}");
                    }
                    drop(answer);
                    (None, true)
                }
            };
            most = most.max(meter.check(step, bound, what));
            match next {
                Some(next) if !done => event = next,
                Some(_) | None => return most,
            }
        }
    }

    /// Limits small enough that a generated layer comes near them.
    fn small() -> Limits {
        Limits {
            entries: 1_000,
            names: 256,
            depth: 16,
            held: 1_024,
            name: 32,
            path: 512,
            target: 64,
            attributes: 8,
            attribute_name: 32,
            attribute_value: 256,
            linked: 64,
            linked_paths: 4_096,
            page: 8,
            listed: 16,
            batch: 4_000,
        }
    }

    #[test]
    fn walking_generated_layers_holds_no_more_than_the_worst_case() {
        for seed in 0..6_u64 {
            let mut rng = Rng::new(0x7EE0_7000 + seed);
            let fs = layers::layer(&mut rng, 60 + seed * 30);
            let shipped = peak(&fs, &SHIPPED, &("shipped", seed));
            let tight = peak(&fs, &small(), &("small", seed));
            assert!(tight <= worst_case(&small()).expect("fits"), "seed {seed}");
            assert!(shipped > 0, "the walk holds something, seed {seed}");
        }
    }

    #[test]
    fn walking_wide_and_deep_layers_holds_no_more_than_the_worst_case() {
        let mut rng = Rng::new(0x7EE0_7100);
        let fs = layers::wide_and_deep(&mut rng, 2_000, 12);
        peak(
            &fs,
            &Limits { entries: 10_000, names: 4_096, held: 8_192, depth: 16, ..small() },
            &"wide and deep, small",
        );
        peak(&fs, &SHIPPED, &"wide and deep, shipped");
    }

    #[test]
    fn walking_a_layer_at_its_limits_holds_no_more_than_the_worst_case() {
        let mut fs = Fs::new();
        // An entry with its attributes full: 64 names of 255 bytes, 64 values of 64 KiB.
        let mut full = Node::file(2, b"x");
        for number in 0..64 {
            full = full.attr(format!("user.{number:0>250}").as_bytes(), &vec![b'v'; 65_536]);
        }
        fs.add(b"full", full);
        // A link target of 4096 bytes, and two hard links to it.
        let link = fs.add(b"target", Node::symlink(3, &[b't'; 4_096]));
        fs.link(b"target2", link);
        // A directory of names of 255 bytes.
        fs.add(b"names", Node::new(Kind::Directory, 4));
        for number in 0..400_u64 {
            fs.add(format!("names/{number:0>255}").as_bytes(), Node::file(1_000 + number, b""));
        }
        // A path of 4096 bytes.
        let mut path = Vec::new();
        for _ in 0..16 {
            if !path.is_empty() {
                path.push(b'/');
            }
            path.extend_from_slice(&[b'd'; 254]);
            fs.add(&path, Node::new(Kind::Directory, 9_000 + u64::try_from(path.len()).expect("small")));
        }
        path.push(b'/');
        path.extend_from_slice(&[b'f'; 16]);
        fs.add(&path, Node::file(9, b""));
        let held = peak(&fs, &SHIPPED, &"at the limits");
        assert!(held > 4_000_000, "the full entry alone is more: {held}");
    }
}
