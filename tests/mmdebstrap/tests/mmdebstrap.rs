use hearth_mmdebstrap_world::{
    fixtures::LIMITS,
    generate, reference,
    world::{self, End},
};
use hearth_protocol_mmdebstrap::output::OutputError;
use skein_lib::{Rng, stream::Fault};
#[test]
fn generated_output_decodes_as_the_reference_whatever_the_cuts() {
    for seed in 0..32 {
        let mut rng = Rng::new(0x0702_0000 + seed);
        let output = generate::output(&mut rng, &LIMITS);
        let run = world::read_output(&output, seed, &LIMITS, End::Whole);
        assert_eq!(run.decoded, reference::output(&output, End::Whole, &LIMITS), "seed {seed}");
    }
}
#[test]
fn output_cut_short_reads_as_the_reference_reads_its_prefix() {
    for seed in 0..32 {
        let mut rng = Rng::new(0x0702_1000 + seed);
        let output = generate::output(&mut rng, &LIMITS);
        let at = usize::try_from(rng.between(0, u64::try_from(output.len()).unwrap())).unwrap();
        let end = End::Cut(at);
        assert_eq!(
            world::read_output(&output, seed, &LIMITS, end).decoded,
            reference::output(&output, end, &LIMITS),
            "seed {seed}"
        );
    }
}
#[test]
fn a_pipe_that_breaks_fails_the_reader_with_its_fault() {
    let bytes = b"Inst libc6 (2.41-12 Debian:testing [amd64])\n";
    for at in 0..=bytes.len() {
        let end = End::Break(at);
        let run = world::read_output(bytes, u64::try_from(at).unwrap(), &LIMITS, end);
        assert_eq!(run.decoded, reference::output(bytes, end, &LIMITS), "at {at}");
        assert_eq!(run.decoded.error, Some(OutputError::Stream(Fault::Reset)));
    }
}
#[test]
fn generated_failures_keep_the_reference_s_tail_and_cause() {
    for seed in 0..32 {
        let mut rng = Rng::new(0x0703_0000 + seed);
        let output = generate::failures(&mut rng);
        let run = world::read_output(&output, seed, &LIMITS, End::Whole);
        assert_eq!(run.decoded, reference::output(&output, End::Whole, &LIMITS), "seed {seed}");
    }
}
#[test]
fn generated_lists_decode_as_the_reference_whatever_the_cuts() {
    for seed in 0..32 {
        let mut rng = Rng::new(0x3801_0000 + seed);
        let index = generate::index(&mut rng, &LIMITS);
        assert_eq!(
            world::read_index(&index, seed, &LIMITS, End::Whole),
            reference::index(&index, End::Whole, &LIMITS),
            "seed {seed}"
        );
    }
}
#[test]
fn a_list_cut_short_reads_as_the_reference_reads_its_prefix() {
    for seed in 0..32 {
        let mut rng = Rng::new(0x3801_1000 + seed);
        let index = generate::index(&mut rng, &LIMITS);
        let end = End::Cut(usize::try_from(rng.between(0, u64::try_from(index.len()).unwrap())).unwrap());
        assert_eq!(
            world::read_index(&index, seed, &LIMITS, end),
            reference::index(&index, end, &LIMITS),
            "seed {seed}"
        );
    }
}
