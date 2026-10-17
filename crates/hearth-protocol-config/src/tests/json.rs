use super::fixtures::{decode, decode_under};
use crate::{
    Fault, Position,
    json::{Kind, Value},
    limits::{Limits, SHIPPED},
};

#[test]
fn a_value_carries_the_line_and_column_of_its_first_byte() {
    const KEYS: &[(&[u8], Position, Position)] = &[
        (b"core", Position { offset: 4, line: 2, column: 3 }, Position { offset: 12, line: 2, column: 11 }),
        (b"n", Position { offset: 43, line: 3, column: 3 }, Position { offset: 48, line: 3, column: 8 }),
    ];
    let value = decode(b"{\n  \"core\": {\"domain\": \"ekanayaka.io\"},\r\n  \"n\": [1, -2]\n}").expect("strict JSON");
    assert_eq!(value.at, Position { offset: 0, line: 1, column: 1 });
    let Kind::Map(entries) = &value.kind else {
        panic!("object");
    };
    assert_eq!(entries.len(), KEYS.len());
    for (entry, expected) in entries.iter().zip(KEYS) {
        assert_eq!((&*entry.key, entry.at, entry.value.at), *expected);
    }
}

#[test]
fn a_duplicate_key_fails_at_the_second_naming_the_first() {
    const CASES: &[(&[u8], &[u8], u32, Position)] = &[
        (br#"{"a":1,"a":2}"#, b"a", 7, Position { offset: 1, line: 1, column: 2 }),
        (br#"{"o":{"a":1,"a":2}}"#, b"a", 12, Position { offset: 6, line: 1, column: 7 }),
        (br#"{"a":[{"x":1,"x":2}]}"#, b"x", 13, Position { offset: 7, line: 1, column: 8 }),
        (br#"{"_note":"a","_note":"b"}"#, b"_note", 13, Position { offset: 1, line: 1, column: 2 }),
    ];
    for &(input, key, second, first) in CASES {
        let error = decode(input).expect_err("duplicate");
        assert_eq!(error.at.offset, second, "{input:?}");
        assert_eq!(error.fault, Fault::Duplicate { key: key.into(), first });
    }
}

#[test]
fn a_fraction_or_an_exponent_fails_at_the_number() {
    const CASES: &[&[u8]] = &[br#"{"n":1.2}"#, br#"{"n":1e2}"#, br#"{"n":1E-2}"#];
    for input in CASES {
        let error = decode(input).expect_err("not integer");
        assert_eq!(error.at, Position { offset: 5, line: 1, column: 6 });
        assert_eq!(error.fault, Fault::NotInteger);
    }
}

#[test]
fn integers_decode_to_i64_s_edges_and_fail_one_past() {
    const CASES: &[(&[u8], Result<i64, Fault>)] = &[
        (br#"{"n":9223372036854775807}"#, Ok(9_223_372_036_854_775_807)),
        (br#"{"n":-9223372036854775808}"#, Ok(-9_223_372_036_854_775_808)),
        (br#"{"n":9223372036854775808}"#, Err(Fault::Range)),
        (br#"{"n":-9223372036854775809}"#, Err(Fault::Range)),
    ];
    for (input, expected) in CASES {
        let found = match decode(input) {
            Ok(value) => {
                let Kind::Map(entries) = value.kind else {
                    panic!("object");
                };
                let Kind::Int(n) = entries[0].value.kind else {
                    panic!("integer");
                };
                Ok(n)
            }
            Err(error) => Err(error.fault),
        };
        assert_eq!(&found, expected, "{input:?}");
    }
}

#[test]
fn u0000_in_a_string_or_a_key_fails_at_its_opening_quote() {
    const CASES: &[(&[u8], Position)] = &[
        (br#"{"n":"\u0000"}"#, Position { offset: 5, line: 1, column: 6 }),
        (br#"{"\u0000":1}"#, Position { offset: 1, line: 1, column: 2 }),
    ];
    for (input, at) in CASES {
        let error = decode(input).expect_err("NUL");
        assert_eq!(error.at, *at);
        assert_eq!(error.fault, Fault::Nul);
    }
}

#[test]
fn a_note_is_dropped_from_every_object() {
    const CASES: &[(&[u8], u32)] =
        &[(br#"{"_note":"x"}"#, 0), (br#"{"a":{"_note":"x","b":1}}"#, 2), (br#"{"a":[{"_note":"x"}]}"#, 1)];
    for &(input, count) in CASES {
        assert_eq!(count_keys(&decode(input).expect("notes")), count);
    }
}
fn count_keys(value: &Value) -> u32 {
    let mut count = 0_u32;
    match &value.kind {
        Kind::Map(entries) => {
            for entry in entries {
                assert_ne!(&*entry.key, b"_note");
                count = count.saturating_add(1).saturating_add(count_keys(&entry.value));
            }
        }
        Kind::List(values) => {
            for item in values {
                count = count.saturating_add(count_keys(item));
            }
        }
        Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::String(_) => {}
    }
    count
}

#[test]
fn a_note_that_is_not_a_string_fails_at_its_value() {
    const CASES: &[&[u8]] =
        &[br#"{"_note":null}"#, br#"{"_note":true}"#, br#"{"_note":{}}"#, br#"{"_note":[]}"#, br#"{"_note":1}"#];
    for input in CASES {
        let error = decode(input).expect_err("note");
        assert_eq!(error.at, Position { offset: 9, line: 1, column: 10 });
        assert_eq!(error.fault, Fault::Note);
    }
}
#[test]
fn a_document_that_is_not_an_object_fails_at_its_first_byte() {
    const CASES: &[&[u8]] = &[b"null", b"[]", b"true", b"1", br#""s""#];
    for input in CASES {
        let error = decode(input).expect_err("object required");
        assert_eq!(error.at, Position { offset: 0, line: 1, column: 1 });
        assert_eq!(error.fault, Fault::NotObject);
    }
}
#[test]
fn the_shipped_limits_refuse_one_past_each() {
    const EXPECTED: &[Fault] = &[
        Fault::Json(skein_json::tokenizer::Error::TooDeep),
        Fault::Keys,
        Fault::Json(skein_json::tokenizer::Error::TooLong),
    ];
    let inputs = super::fixtures::past_shipped();
    for (input, expected) in inputs.iter().zip(EXPECTED) {
        assert_eq!(decode(input).expect_err("one past").fault, *expected);
    }
}

#[test]
fn a_note_counts_as_a_key_at_the_limit() {
    const CASES: &[(&[u8], Result<(), Fault>)] =
        &[(br#"{"_note":"x","a":0}"#, Err(Fault::Keys)), (br#"{"_note":"x"}"#, Ok(()))];
    let limits = Limits { keys: 1, ..SHIPPED };
    for (input, expected) in CASES {
        let found = match decode_under(input, &limits) {
            Ok(_) => Ok(()),
            Err(error) => Err(error.fault),
        };
        assert_eq!(&found, expected);
    }
}

#[test]
fn a_bad_escape_in_a_string_or_a_key_fails_at_its_backslash() {
    const CASES: &[(&[u8], Position)] = &[
        (br#"{"n":"a\q"}"#, Position { offset: 7, line: 1, column: 8 }),
        (br#"{"a\q":1}"#, Position { offset: 3, line: 1, column: 4 }),
    ];
    for &(input, at) in CASES {
        let error = decode(input).expect_err("bad escape");
        assert_eq!(error.at, at);
        assert_eq!(error.fault, Fault::Json(skein_json::tokenizer::Error::Escape));
    }
}
