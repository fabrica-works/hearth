use hearth_domain_tree::{
    limits::{Limits, SHIPPED},
    vocabulary::{Answer, Cause, Fault, Request},
};
use hearth_lib::encode::Digest;
use hearth_protocol_tree::{limits as protocol, tree::Item};
use hearth_tree_world::{
    check::{self, Record},
    expected, fixtures, generate,
    helper::{self, Helper},
    layers, mutate, oracle,
};
use skein_lib::Rng;

const FIRST_SEED: u64 = 0x7EE0_F000;
const SEEDS: u64 = 2_000;
const LAYER_SEEDS: u64 = 2_000;
const RESOLUTION_SEEDS: u64 = 200;
const TREE_SEEDS: u64 = 400;
const PINNED: &[u64] = &[];

#[test]
fn random_store_and_cache_file_frames_cut_and_mutated_read_as_the_oracle_says() {
    let limits = fixtures::limits();
    for seed in (FIRST_SEED..FIRST_SEED + SEEDS).chain(PINNED.iter().copied()) {
        let mut rng = Rng::new(seed);
        let record = generate::small(&mut rng, &limits);
        check::sweep(&mut rng, &record, &limits, seed);
    }
}

#[test]
fn random_resolution_frames_cut_and_mutated_read_as_the_oracle_says() {
    let limits = fixtures::limits();
    for seed in (FIRST_SEED..FIRST_SEED + RESOLUTION_SEEDS).chain(PINNED.iter().copied()) {
        let mut rng = Rng::new(seed);
        let packages = u32::try_from(rng.between(0, 512)).expect("small closure");
        let record = Record::Resolution(generate::resolution(&mut rng, &limits, packages));
        check::sweep(&mut rng, &record, &limits, seed);
    }
}

#[test]
fn random_layer_frames_cut_and_mutated_read_as_the_oracle_says() {
    let limits = fixtures::limits();
    for seed in (FIRST_SEED..FIRST_SEED + LAYER_SEEDS).chain(PINNED.iter().copied()) {
        let mut rng = Rng::new(seed);
        let record = Record::Layer(Box::new(generate::layer(&mut rng, &limits)));
        check::sweep(&mut rng, &record, &limits, seed);
    }
}

#[test]
fn random_tree_files_cut_and_mutated_read_as_the_oracle_says() {
    let limits = fixtures::limits();
    for seed in (FIRST_SEED..FIRST_SEED + TREE_SEEDS).chain(PINNED.iter().copied()) {
        let mut rng = Rng::new(seed);
        let size = rng.between(1, 120);
        let items = generate::tree_file(&mut rng, size);
        assert!(oracle::items_ok(&items), "the generator makes tree files, seed {seed}");
        let record = Record::Tree(items.into());
        check::sweep(&mut rng, &record, &limits, seed);
        let frame = check::write(&record, &limits);
        for changed in mutate::frames(&mut rng, &frame) {
            if let Ok(items) = check::read_tree(&changed, &limits) {
                assert!(oracle::items_ok(&items), "a mutated file is read only if it is a tree file, seed {seed}");
            }
        }
    }
}

const HASH_FIRST_SEED: u64 = 0x7EE0_F100;
const HASH_SEEDS: u64 = 300;
const FAULT_SEEDS: u64 = 300;
const KEY: Digest = Digest([0x24; 32]);

/// Limits that make the walk page, batch and bound in small steps.
fn limits(rng: &mut Rng) -> Limits {
    Limits {
        page: u32::try_from(rng.between(1, 40)).expect("small"),
        batch: u32::try_from(rng.between(1, 6_000)).expect("small"),
        ..SHIPPED
    }
}

#[test]
fn random_layers_hash_to_what_the_oracle_lists() {
    for seed in HASH_FIRST_SEED..HASH_FIRST_SEED + HASH_SEEDS {
        let mut rng = Rng::new(seed);
        let fs = if rng.chance(100) {
            let (width, depth) = (rng.between(1, 200), rng.between(1, 60));
            layers::wide_and_deep(&mut rng, width, depth)
        } else {
            let size = rng.between(0, 250);
            layers::layer(&mut rng, size)
        };
        let limits = limits(&mut rng);
        let walked = helper::walk(&fs, &limits, KEY);
        let Answer::Hashed { tree, changes, counts, tree_file } = &walked.answer else {
            panic!("seed {seed}: {:?}", walked.failed().1)
        };
        let items = expected::items(&fs, KEY);
        let Some(Item::End(end)) = items.last() else { panic!("the end item is last") };
        assert_eq!((*tree, *changes), (end.tree, end.changes), "digests, seed {seed}");
        assert_eq!(helper::tree_counts(counts), end.counts, "counts, seed {seed}");
        assert!(oracle::items_ok(&items), "seed {seed}");
        let read = check::read_tree(&walked.file, &protocol::SHIPPED).expect("a tree file");
        assert_eq!(read.as_ref(), items.as_slice(), "file, seed {seed}");
        assert_eq!(tree_file.size, u64::try_from(walked.file.len()).expect("a size"), "seed {seed}");
    }
}

#[test]
fn an_io_failure_anywhere_ends_the_walk_at_that_requests_path_and_never_panics() {
    for seed in HASH_FIRST_SEED..HASH_FIRST_SEED + FAULT_SEEDS {
        let mut rng = Rng::new(seed);
        let size = rng.between(0, 40);
        let fs = layers::layer(&mut rng, size);
        let limits = limits(&mut rng);
        let clean = helper::walk(&fs, &limits, KEY);
        assert!(matches!(clean.answer, Answer::Hashed { .. }), "seed {seed}");
        let at = usize::try_from(rng.below(clean.requests.len() as u64)).expect("an index");
        let fault = [Fault::NotFound, Fault::Permission, Fault::NoSpace, Fault::Bound, Fault::Other]
            [usize::try_from(rng.below(5)).expect("an index")];
        let mut helper = Helper::new(&fs);
        helper.fault = Some((at, fault));
        let walked = helper::walk_with(helper, &limits, KEY);
        let (path, cause) = walked.failed();
        let wanted: &[u8] = match &clean.requests[at] {
            Request::Stat { path }
            | Request::List { path, .. }
            | Request::Attributes { path }
            | Request::Attribute { path, .. }
            | Request::ReadLink { path }
            | Request::HashFile { path, .. } => path,
            Request::Next { .. } => {
                let mut open: &[u8] = b"";
                for request in &clean.requests[..at] {
                    if let Request::List { path, .. } = request {
                        open = path;
                    }
                }
                open
            }
            Request::Write { items } => match &items[0] {
                hearth_domain_tree::vocabulary::Item::Entry(entry, _) => &entry.path,
                hearth_domain_tree::vocabulary::Item::Put { path } => path,
            },
            Request::Create | Request::Digests { .. } | Request::Finish { .. } => b"",
        };
        assert_eq!(path, wanted, "seed {seed}, request {at}");
        assert_ne!(cause, Cause::Unexpected, "seed {seed}");
        assert_eq!(walked.requests.len(), at + 1, "seed {seed}");
    }
}
