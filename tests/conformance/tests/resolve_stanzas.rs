use hearth_conformance_world::{fixtures::RESOLVE_INDEX, resolve::stanzas};

const STANZA_CASES: &[(&[&[u8]], &[u8])] = &[
    (&[b"Inst apt (3.3 Debian:unstable [amd64]) []", b"Inst apt (3.3 Debian:unstable [amd64]) []"],
        b"Package: apt\nVersion: 3.3\nArchitecture: amd64\nOther: kept\n\n"),
    (&[b"Inst base (1 Debian:unstable [all]) []", b"Inst apt [3.2] (3.3 Debian:unstable, Debian:testing [amd64]) []"],
        b"Package: apt\nVersion: 3.3\nArchitecture: amd64\nOther: kept\n\nPackage: base\nVersion: 1\nArchitecture: all\n\n"),
];
#[test]
fn matching_stanzas_keep_index_order_and_all_fields_without_duplicates() {
    for (insts, expected) in STANZA_CASES {
        assert_eq!(stanzas(RESOLVE_INDEX, insts).expect("matching stanzas"), *expected, "{insts:?}");
    }
}
const MISSING_CASES: &[(&[u8], &[u8])] = &[
    (b"Inst apt (3.2 Debian:unstable [amd64]) []", b"Inst apt (3.2 Debian:unstable [amd64]) []"),
    (b"Inst apt (3.3 Debian:unstable [arm64]) []", b"Inst apt (3.3 Debian:unstable [arm64]) []"),
    (b"Inst absent (1 Debian:unstable [all]) []", b"Inst absent (1 Debian:unstable [all]) []"),
    (b"Inst apt malformed", b"Inst apt malformed"),
];
#[test]
fn a_missing_or_malformed_triple_names_its_first_inst_line() {
    for (line, expected) in MISSING_CASES {
        assert_eq!(
            stanzas(RESOLVE_INDEX, std::slice::from_ref(line)).expect_err("missing stanza").as_ref(),
            *expected,
            "{line:?}"
        );
    }
}

#[test]
fn an_earlier_missing_stanza_precedes_a_later_malformed_line() {
    const INSTS: &[&[u8]] = &[b"Inst absent (1 Debian:unstable [all]) []", b"Inst malformed"];
    assert_eq!(
        stanzas(RESOLVE_INDEX, INSTS).expect_err("first missing stanza").as_ref(),
        b"Inst absent (1 Debian:unstable [all]) []"
    );
}
