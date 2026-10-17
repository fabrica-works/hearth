use hearth_mmdebstrap_world::{
    fixtures::{self, LIMITS},
    recordings, reference,
    world::{self, End},
};
use hearth_protocol_mmdebstrap::{
    lists::{self, Listed},
    tail::{Cause, Class, Kind},
    vector,
};
#[test]
fn every_recording_has_an_expectation_and_the_reference_agrees() {
    let cases = recordings::all();
    assert!(!cases.is_empty(), "captured mmdebstrap runs");
    for case in cases {
        let decoded = reference::recording(&case.recording, &LIMITS);
        let expected = case.expected.as_ref().unwrap_or_else(|| {
            panic!("{}: missing expectation:\n{}", case.case, reference::draft(&decoded, &case.recording.exit))
        });
        assert_eq!(&decoded, expected, "{}", case.case);
        let insts = case
            .recording
            .stderr
            .split(|b| *b == b'\n')
            .chain(case.recording.stdout.split(|b| *b == b'\n'))
            .filter(|line| line.starts_with(b"Inst "))
            .count();
        assert_eq!(expected.insts.len(), insts, "{}: every captured Inst line", case.case);
    }
}
#[test]
fn every_recording_decodes_to_its_expectation_whatever_the_cuts() {
    for case in recordings::all() {
        let expected = case.expected.as_ref().expect("a hand-reviewed expectation");
        for seed in 0..2 {
            assert_eq!(&world::read_recording(&case, seed, &LIMITS).decoded, expected, "{}, seed {seed}", case.case);
        }
    }
}
#[test]
fn every_recording_decodes_to_its_expectation_with_further_cuts() {
    for case in recordings::all() {
        let expected = case.expected.as_ref().expect("a hand-reviewed expectation");
        for seed in 2..4 {
            assert_eq!(&world::read_recording(&case, seed, &LIMITS).decoded, expected, "{}, seed {seed}", case.case);
        }
    }
}
#[test]
fn the_recorded_vectors_are_the_ones_this_machine_encodes() {
    for case in recordings::all() {
        let invocation = fixtures::invocation(&case);
        let argv: Vec<_> = std::iter::once(vector::PROGRAM).chain(invocation.args.iter().map(AsRef::as_ref)).collect();
        let captured: Vec<_> = case.recording.argv.iter().map(Vec::as_slice).collect();
        assert_eq!(argv, captured, "{}", case.case);
        let env: Vec<_> = invocation.env.iter().map(AsRef::as_ref).collect();
        let captured: Vec<_> = case.recording.env.iter().map(Vec::as_slice).collect();
        assert_eq!(env, captured, "{}", case.case);
    }
}
type Failure = (&'static str, &'static str, Option<&'static [u8]>, Class, Kind);
const FAILURES: &[Failure] = &[
    ("unknown-package", "unknown", Some(b"hearth-absent"), Class::Configuration, Kind::Resolution),
    ("no-candidate", "unknown", Some(b"mail-transport-agent"), Class::Configuration, Kind::Resolution),
    ("unsatisfiable", "unsatisfiable", Some(b"exim4-config"), Class::Configuration, Kind::Resolution),
    ("unreachable-mirror", "fetch", None, Class::Operational, Kind::Tool),
    ("absent-snapshot", "absent", None, Class::Configuration, Kind::Resolution),
];
#[test]
fn the_recorded_failures_have_the_causes_5_5_gives_them() {
    let cases = recordings::all();
    for (name, label, package, class, kind) in FAILURES {
        let case = cases.iter().find(|case| case.case.as_ref() == *name).expect("recorded failure");
        let decoded = world::read_recording(case, 0, &LIMITS).decoded;
        let (found, named) = match &decoded.cause {
            Cause::Unknown(name) => ("unknown", Some(name.as_bytes())),
            Cause::Unsatisfiable(name) => ("unsatisfiable", name.as_ref().map(vector::Name::as_bytes)),
            Cause::Fetch => ("fetch", None),
            Cause::Absent => ("absent", None),
            Cause::Resolve => ("resolve", None),
            Cause::Space => ("space", None),
            Cause::Other => ("other", None),
        };
        assert_eq!(
            (found, named, decoded.cause.class(), decoded.cause.kind()),
            (*label, *package, *class, *kind),
            "{name}"
        );
    }
}
const FETCH_FAILURES: &[&str] = &["absent-snapshot", "unreachable-mirror"];
#[test]
fn the_recorded_lists_hold_each_index_and_no_other_list() {
    for case in recordings::all() {
        let simulate = fixtures::simulate(&case.case);
        let indices = lists::indices(&vector::lines(&simulate.sources).unwrap(), &simulate.sources.architecture);
        let files = fixtures::listed_files(&case.recording);
        if !FETCH_FAILURES.contains(&case.case.as_ref()) {
            for index in &indices {
                assert!(files.contains(&("file", &*index.name)), "{}: missing {:?}", case.case, index.name);
            }
        }
        for (kind, name) in files {
            if kind == "file" && (name.windows(9).any(|part| part == b"_Packages") || name.ends_with(b"_InRelease")) {
                match lists::listed(name, &indices) {
                    Listed::Index(at) => {
                        assert_eq!(&*indices[usize::try_from(at).unwrap()].name, name, "{}", case.case);
                    }
                    bad @ (Listed::Compressed(_) | Listed::Other) => {
                        panic!("{}: refused list {name:?}: {bad:?}", case.case)
                    }
                }
            } else {
                assert_eq!(lists::listed(name, &indices), Listed::Other, "{}: {name:?}", case.case);
            }
        }
    }
}
#[test]
fn the_recorded_stanzas_decode_as_the_reference_whatever_the_cuts() {
    let mut count = 0;
    for case in recordings::all() {
        if let Some(stanzas) = &case.recording.stanzas {
            count += 1;
            let expected = reference::index(stanzas, End::Whole, &LIMITS);
            assert_eq!(expected.error, None, "{}", case.case);
            for seed in 0..4 {
                assert_eq!(
                    world::read_index(stanzas, seed, &LIMITS, End::Whole),
                    expected,
                    "{}, seed {seed}",
                    case.case
                );
            }
        }
    }
    assert!(count > 0, "at least test-min's stanzas are recorded");
}
#[test]
fn every_inst_line_of_test_min_has_a_recorded_stanza() {
    let cases = recordings::all();
    for case in cases.iter().filter(|case| case.case.as_ref() == "test-min") {
        let decoded = world::read_recording(case, 0, &LIMITS).decoded;
        let stanzas = world::read_index(
            case.recording.stanzas.as_ref().expect("recorded closure stanzas"),
            0,
            &LIMITS,
            End::Whole,
        );
        assert!(decoded.insts.len() > 50, "the essential set and apt");
        assert_eq!(stanzas.error, None, "{}", case.case);
        for inst in decoded.insts {
            let matches = stanzas
                .stanzas
                .iter()
                .filter(|stanza| {
                    stanza.package == inst.name
                        && stanza.version == inst.version
                        && stanza.architecture == inst.architecture
                })
                .count();
            assert_eq!(matches, 1, "{}: {inst:?}", case.case);
        }
    }
}
#[test]
fn a_recording_cut_short_reads_as_the_reference_reads_its_prefix() {
    for case in recordings::all() {
        for bytes in [&case.recording.stderr, &case.recording.stdout] {
            for seed in 0..4 {
                let end = End::Cut(bytes.len() * usize::try_from(seed + 1).unwrap() / 5);
                assert_eq!(
                    world::read_output(bytes, seed, &LIMITS, end).decoded,
                    reference::output(bytes, end, &LIMITS),
                    "{}: seed {seed}",
                    case.case
                );
            }
        }
    }
}
