use hearth_config_world::{
    examples, generate,
    world::{self, Outcome, Settings},
};
use hearth_protocol_config::{
    Fault, Position,
    document::{self, Firmware, Runs},
    json::Waiting,
    limits::{Limits, SHIPPED},
};
use skein_lib::{Rng, stream::Fault as StreamFault};
#[test]
fn every_example_decodes_as_the_reference_reads_it_cut_anywhere() {
    assert_eq!(examples::all().len(), 9, "format.md's nine documents");
    for example in examples::all() {
        for seed in 0..8 {
            let settings = Settings::calm(&mut Rng::new(seed), SHIPPED);
            let run = world::check(example.bytes, &settings, seed);
            assert_eq!(run.outcome, Some(Outcome::Decoded), "{}, seed {seed}", example.name);
        }
    }
}
#[test]
fn generated_documents_decode_as_the_reference_reads_them() {
    for seed in 0..64 {
        let mut rng = Rng::new(seed);
        let limits = Limits { document: 4096, depth: 16, keys: 32 };
        let document = generate::document(&mut rng, &limits);
        let settings = Settings::calm(&mut rng, limits);
        assert_eq!(world::check(&document, &settings, seed).outcome, Some(Outcome::Decoded));
    }
}
#[test]
fn a_document_cut_after_any_byte_fails_as_its_prefix_reads() {
    const CASES: &[&[u8]] = &[r#"{"a":[1,-2,"é",{}]}"#.as_bytes(), b"{\r\n\"b\":true}"];
    for document in CASES {
        for cut in 0..document.len() {
            let mut settings = Settings::calm(&mut Rng::new(cut as u64), SHIPPED);
            settings.cut = Some(cut);
            drop(world::check(document, &settings, cut as u64));
        }
    }
}
#[test]
fn a_close_in_each_waiting_state_answers_closed_once_and_last() {
    const CASES: &[Waiting] = &[Waiting::Decode, Waiting::Bytes, Waiting::Close];
    for &waiting in CASES {
        let mut settings = Settings::calm(&mut Rng::new(7), SHIPPED);
        settings.close = Some(waiting);
        assert_eq!(world::check(b"{\"a\":[1,2]}", &settings, 7).closed_at, waiting);
    }
}
#[test]
fn a_failed_stream_fails_the_decode_with_its_fault() {
    const CASES: &[StreamFault] = &[StreamFault::Reset, StreamFault::Invalid, StreamFault::Other];
    for &fault in CASES {
        let mut settings = Settings::calm(&mut Rng::new(8), SHIPPED);
        settings.failure = Some((0, fault));
        let run = world::check(b"{\"a\":1}", &settings, 8);
        let Some(Outcome::Failed(error)) = run.outcome else {
            panic!("failed stream");
        };
        assert_eq!(error.fault, Fault::Stream(fault));
    }
}

#[test]
fn a_number_cut_before_its_delimiter_is_truncated_before_its_value_is_judged() {
    const CASES: &[(&[u8], usize, hearth_protocol_config::Position)] = &[
        (br#"{"a":9223372036854775808}"#, 24, hearth_protocol_config::Position { offset: 24, line: 1, column: 25 }),
        (br#"{"a":1.2}"#, 8, hearth_protocol_config::Position { offset: 8, line: 1, column: 9 }),
    ];
    for &(document, cut, at) in CASES {
        let seed = 12_586_237;
        let mut settings = Settings::calm(&mut Rng::new(seed), SHIPPED);
        settings.cut = Some(cut);
        assert_eq!(
            world::check(document, &settings, seed).outcome,
            Some(Outcome::Failed(hearth_protocol_config::Error {
                at,
                fault: Fault::Json(skein_json::tokenizer::Error::Truncated)
            }))
        );
    }
}

type Host = Option<(&'static [u8], Position)>;
type Placed = (&'static str, Runs, Option<Firmware>, Option<&'static [u8]>, Host, Position);
const PLACED: &[Placed] = &[
    ("x1-prod", Runs::Metal, Some(Firmware::Efi), Some(b"7th-gen"), None, Position { offset: 17, line: 2, column: 16 }),
    ("ion-prod", Runs::Vm, Some(Firmware::Bios), Some(b"ionos"), None, Position { offset: 15, line: 1, column: 16 }),
    (
        "chat-prod",
        Runs::Container,
        None,
        None,
        Some((b"rhi-prod", Position { offset: 44, line: 1, column: 45 })),
        Position { offset: 15, line: 1, column: 16 },
    ),
];
#[test]
fn the_placed_examples_decode_to_their_placements_cut_anywhere() {
    for expected in PLACED {
        let example = examples::all().iter().find(|example| example.name == expected.0).expect("placed example");
        for seed in 0..8 {
            let settings = Settings::calm(&mut Rng::new(seed), SHIPPED);
            let run = world::check(example.bytes, &settings, seed);
            let placement =
                document::document(run.value.expect("decoded")).expect("document").layer.placement.expect("placement");
            assert_eq!(
                (
                    example.name,
                    placement.kind,
                    placement.firmware,
                    placement.hardware.as_deref(),
                    placement.on.as_ref().map(|on| (on.target.as_ref(), on.at)),
                    placement.at
                ),
                *expected,
                "{}, seed {seed}",
                example.name
            );
        }
    }
}

const DOCUMENTS: &[(&str, &[&[u8]])] = &[
    ("ion-prod", &[]),
    ("chat-prod", &[]),
    ("stoa", &[b"1", b"2", b"3"]),
    ("fleet", &[]),
    ("x1", &[]),
    ("x1-prod", &[]),
    ("mail", &[]),
    ("mail-lab", &[]),
    ("carbon-lab", &[]),
];
#[test]
fn every_example_decodes_as_a_document_cut_anywhere() {
    for &(name, expected) in DOCUMENTS {
        let example = examples::all().iter().find(|example| example.name == name).expect("document example");
        for seed in 0..8 {
            let settings = Settings::calm(&mut Rng::new(seed), SHIPPED);
            let run = world::check(example.bytes, &settings, seed);
            let document = document::document(run.value.expect("decoded")).expect("document");
            let names: Vec<&[u8]> = document
                .members
                .as_ref()
                .map(|members| members.each.iter().map(|member| member.name.as_ref()).collect())
                .unwrap_or_default();
            assert_eq!(names.as_slice(), expected, "{name}, seed {seed}");
        }
    }
}
