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
#[test]
fn generated_and_mutated_lists_read_as_the_reference_reads_them() {
    for seed in PINNED.iter().copied().chain(0..SEEDS) {
        let mut rng = Rng::new(0x3801_f000 + seed);
        let mut index = generate::index(&mut rng, &LIMITS);
        if rng.chance(500) {
            index = generate::mutate(&mut rng, &index);
        }
        let end = world::end(&mut rng, &index);
        assert_eq!(
            world::read_index(&index, seed, &LIMITS, end),
            reference::index(&index, end, &LIMITS),
            "seed {seed} end {end:?}"
        );
    }
}
#[test]
fn recordings_cut_and_mutated_read_as_the_reference_reads_them() {
    let cases = hearth_mmdebstrap_world::recordings::all();
    for seed in 0..128 {
        let case = &cases[usize::try_from(seed).unwrap() % cases.len()];
        let mut rng = Rng::new(0x3802_f000 + seed);
        for bytes in [&case.recording.stderr, &case.recording.stdout] {
            let bytes = generate::mutate(&mut rng, bytes);
            let end = world::end(&mut rng, &bytes);
            assert_eq!(
                world::read_output(&bytes, seed, &LIMITS, end).decoded,
                reference::output(&bytes, end, &LIMITS),
                "{}: seed {seed}, {end:?}",
                case.case
            );
        }
        if let Some(bytes) = &case.recording.stanzas {
            let bytes = generate::mutate(&mut rng, bytes);
            let end = world::end(&mut rng, &bytes);
            assert_eq!(
                world::read_index(&bytes, seed, &LIMITS, end),
                reference::index(&bytes, end, &LIMITS),
                "{} stanzas: seed {seed}, {end:?}",
                case.case
            );
        }
    }
}
