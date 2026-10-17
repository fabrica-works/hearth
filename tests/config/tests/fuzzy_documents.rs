use hearth_config_world::{
    fixtures::Seen,
    generate,
    world::{self, Settings},
};
use skein_lib::Rng;
const FIRST_SEED: u64 = 0xC0_0300;
const ROUNDS: u64 = 10000;
#[test]
fn generated_documents_mutated_and_cut_read_as_the_reference_reads_them() {
    let mut seen = Seen::default();
    for seed in FIRST_SEED..FIRST_SEED + ROUNDS {
        let mut rng = Rng::new(seed);
        let limits = world::limits(&mut rng);
        let mut document = generate::document(&mut rng, &limits);
        if rng.chance(700) {
            document = generate::mutate(&mut rng, &document);
        }
        let settings = Settings::chaotic(&mut rng, limits, document.len());
        seen.record(&world::check(&document, &settings, seed), &settings);
    }
    seen.assert_fell(Seen::EVERY_FAULT_AND_ENDING);
}
