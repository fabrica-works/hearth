use hearth_config_world::{
    examples, generate,
    world::{self, Outcome, Settings},
};
use hearth_protocol_config::{
    Fault,
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
