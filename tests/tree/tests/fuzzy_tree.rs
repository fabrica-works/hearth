use hearth_tree_world::{
    check::{self, Record},
    fixtures, generate,
};
use skein_lib::Rng;

const FIRST_SEED: u64 = 0x7EE0_F000;
const SEEDS: u64 = 2_000;
const LAYER_SEEDS: u64 = 2_000;
const RESOLUTION_SEEDS: u64 = 200;
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
