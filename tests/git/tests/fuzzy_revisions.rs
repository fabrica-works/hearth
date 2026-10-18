use hearth_git_world::{fixtures::revisions as inputs, reference};
use hearth_protocol_git::invocation;
use skein_lib::Rng;
#[test]
fn rev_parse_answers_cut_and_mutated_decode_as_the_reference_does() {
    for seed in inputs::CASES {
        let mut rng = Rng::new(inputs::SEED + seed);
        let input = inputs::case(&mut rng);
        for exit in inputs::EXITS {
            assert_eq!(
                inputs::decoded(&input.stdout, *exit),
                reference::answer(&input.stdout, *exit),
                "seed {seed}: {:?}",
                input.stdout
            );
        }
    }
}
#[test]
fn revision_bytes_obey_the_guard_alphabet_and_limit() {
    for seed in inputs::CASES {
        let mut rng = Rng::new(inputs::SEED + seed);
        let input = inputs::case(&mut rng);
        assert_eq!(
            invocation::revision(&input.revision).map(|value| value.bytes().to_vec()),
            reference::revision(&input.revision),
            "seed {seed}: {:?}",
            input.revision
        );
    }
}
