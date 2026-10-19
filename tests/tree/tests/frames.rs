use hearth_protocol_tree::frame::{self, Kind, Unreadable};
use hearth_tree_world::{check, fixtures, generate, mutate};
use skein_lib::Rng;

const SEEDS: &[u64] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
const CUT_CASES: &[(u64, bool)] = &[
    (0, true),
    (1, true),
    (2, true),
    (3, true),
    (4, true),
    (5, true),
    (6, true),
    (7, true),
    (8, true),
    (9, true),
    (10, true),
    (11, true),
    (12, true),
    (13, true),
    (14, true),
    (15, true),
];
const WRONG_KINDS: &[(Kind, Kind, Unreadable)] = &[
    (Kind::Store, Kind::Layer, Unreadable::Kind { found: 1 }),
    (Kind::Store, Kind::Resolution, Unreadable::Kind { found: 1 }),
    (Kind::Store, Kind::CacheFile, Unreadable::Kind { found: 1 }),
    (Kind::Store, Kind::Tree, Unreadable::Kind { found: 1 }),
    (Kind::Layer, Kind::Store, Unreadable::Kind { found: 2 }),
    (Kind::Layer, Kind::Resolution, Unreadable::Kind { found: 2 }),
    (Kind::Layer, Kind::CacheFile, Unreadable::Kind { found: 2 }),
    (Kind::Layer, Kind::Tree, Unreadable::Kind { found: 2 }),
    (Kind::Tree, Kind::Store, Unreadable::Kind { found: 3 }),
    (Kind::Tree, Kind::Layer, Unreadable::Kind { found: 3 }),
    (Kind::Tree, Kind::Resolution, Unreadable::Kind { found: 3 }),
    (Kind::Tree, Kind::CacheFile, Unreadable::Kind { found: 3 }),
    (Kind::Resolution, Kind::Store, Unreadable::Kind { found: 4 }),
    (Kind::Resolution, Kind::Layer, Unreadable::Kind { found: 4 }),
    (Kind::Resolution, Kind::CacheFile, Unreadable::Kind { found: 4 }),
    (Kind::Resolution, Kind::Tree, Unreadable::Kind { found: 4 }),
    (Kind::CacheFile, Kind::Store, Unreadable::Kind { found: 5 }),
    (Kind::CacheFile, Kind::Layer, Unreadable::Kind { found: 5 }),
    (Kind::CacheFile, Kind::Resolution, Unreadable::Kind { found: 5 }),
    (Kind::CacheFile, Kind::Tree, Unreadable::Kind { found: 5 }),
];

#[test]
fn every_generated_record_reads_back_as_it_was_generated() {
    let limits = fixtures::limits();
    for &seed in SEEDS {
        for record in generate::one_of_each(&mut Rng::new(0x7EE0_0000 + seed), &limits) {
            let frame = check::write(&record, &limits);
            assert_eq!(check::read(record.kind(), &frame, &limits), Ok(record), "seed {seed}");
        }
    }
}

#[test]
fn no_cut_of_a_generated_frame_reads_as_a_record() {
    let limits = fixtures::limits();
    for &(seed, expected) in CUT_CASES {
        for record in generate::one_of_each(&mut Rng::new(0x7EE0_0000 + seed), &limits) {
            let frame = check::write(&record, &limits);
            for cut in 0..frame.len() {
                let refused = matches!(
                    check::read(record.kind(), &frame[..cut], &limits),
                    Err(Unreadable::Short | Unreadable::Length { .. })
                );
                assert_eq!(refused, expected, "seed {seed}, cut {cut}");
            }
        }
    }
}

#[test]
fn a_mutated_frame_is_unreadable_or_its_records_one_encoding() {
    let limits = fixtures::limits();
    for &seed in SEEDS {
        let mut rng = Rng::new(0x7EE0_1000 + seed);
        for record in generate::one_of_each(&mut rng, &limits) {
            let frame = check::write(&record, &limits);
            for changed in mutate::frames(&mut rng, &frame) {
                if let Ok(read) = check::read(record.kind(), &changed, &limits) {
                    assert_eq!(check::write(&read, &limits).as_ref(), changed, "seed {seed}");
                }
            }
        }
    }
}

#[test]
fn every_body_the_decoder_takes_the_encoder_writes_again_byte_for_byte() {
    let limits = fixtures::limits();
    for &seed in SEEDS {
        let mut rng = Rng::new(0x7EE0_2000 + seed);
        for record in generate::one_of_each(&mut rng, &limits) {
            let bytes = check::write(&record, &limits);
            for frame in std::iter::once(bytes.to_vec()).chain(mutate::frames(&mut rng, &bytes)) {
                if let Ok(body) = frame::body(&frame, record.kind(), &limits) {
                    check::canonical_body(body, &limits, seed);
                }
            }
        }
    }
}

#[test]
fn a_frame_of_each_kind_is_unreadable_as_the_other_four() {
    let limits = fixtures::limits();
    let records = generate::one_of_each(&mut Rng::new(0x7EE0_3000), &limits);
    for &(own, other, expected) in WRONG_KINDS {
        let record = records.iter().find(|record| record.kind() == own).expect("each kind generated");
        let frame = check::write(record, &limits);
        assert_eq!(check::read(other, &frame, &limits), Err(expected), "{own:?} as {other:?}");
    }
}
