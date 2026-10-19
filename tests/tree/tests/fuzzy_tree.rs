use hearth_tree_world::{
    check::{self, Record},
    fixtures, generate, mutate, oracle,
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
