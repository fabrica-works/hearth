use hearth_config_world::{
    fixtures::Tally,
    generate,
    world::{self, Settings},
};
use hearth_protocol_config::{Error, Fault};
use skein_lib::Rng;
const FIRST_SEED: u64 = 0x0035_F000;
const ROUNDS: u64 = 512;
#[test]
fn generated_declarations_broken_once_fail_at_the_break() {
    let mut tally = Tally::default();
    for seed in FIRST_SEED..FIRST_SEED + ROUNDS {
        let mut rng = Rng::new(seed);
        let limits = world::declaring(&mut rng);
        let written = generate::declaration(&mut rng, &limits);
        let broken = generate::break_once(&mut rng, &written);
        let outcome = world::declare(&broken.bytes, &Settings::calm(&mut rng, limits), seed);
        assert_eq!(
            outcome.err(),
            Some(Error { at: broken.at, fault: Fault::Declaration(broken.fault) }),
            "seed {seed}"
        );
        tally.record(broken.fault);
    }
    tally.assert_fell(Tally::EVERY_DECLARED);
}
