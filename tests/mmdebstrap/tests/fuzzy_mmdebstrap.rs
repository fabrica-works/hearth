use hearth_mmdebstrap_world::{fixtures::LIMITS, generate, reference, world};
use skein_lib::Rng;
const SEEDS: u64 = 1_000;
const PINNED: &[u64] = &[];
#[test]
fn generated_and_mutated_output_reads_as_the_reference_reads_it() {
    for seed in PINNED.iter().copied().chain(0..SEEDS) {
        let mut rng = Rng::new(0x0702_f000 + seed);
        let mut output = generate::output(&mut rng, &LIMITS);
        if rng.chance(500) {
            output = generate::mutate(&mut rng, &output);
        }
        let end = world::end(&mut rng, &output);
        let run = world::read_output(&output, seed, &LIMITS, end);
        assert_eq!(run.decoded, reference::output(&output, end, &LIMITS), "seed {seed} end {end:?}");
    }
}
