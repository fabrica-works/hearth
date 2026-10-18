//! Four arrival orders per repository, as session 29 prescribes.
pub const GOLDEN_SEEDS: core::ops::Range<u64> = 0..4;
pub fn check_digests(name: &str) {
    let golden = crate::golden::load(name);
    let repository = golden.repository();
    for seed in GOLDEN_SEEDS {
        let run = crate::parent::run(&repository, &[], seed);
        let digests: Vec<_> = run
            .answers
            .iter()
            .map(|answer| {
                let hearth_domain_render::vocabulary::Answer::Spec(spec) = answer else {
                    panic!("seed {seed}: {answer:?}")
                };
                let target = std::str::from_utf8(&spec.meta.target).expect("target name");
                (target, spec.digest.0.iter().take(6).copied().collect::<Vec<_>>())
            })
            .collect();
        let targets: Vec<_> =
            golden.target_digests("").into_iter().map(|(name, digest)| (name, digest.to_vec())).collect();
        let digests: Vec<_> = digests.into_iter().map(|(name, digest)| (name.to_owned(), digest)).collect();
        assert_eq!(digests, targets, "{name}: seed {seed}");
    }
}

/// The JSON stream has one outstanding read and one terminal result.
pub const JSON_QUEUE: u32 = 1;
/// Each byte can advance both decoder directions and its bounded containers.
pub const JSON_TURNS_PER_BYTE: usize = 8;
/// Opening, end-of-stream and terminal delivery fit within thirty-two turns.
pub const JSON_TURN_GRACE: usize = 32;
