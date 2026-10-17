use super::fixtures;
use crate::{
    limits::{self, Limits},
    lists::{self, Listed},
    packages::{self, Field, Found, StanzaError},
    vector,
};
use skein_lib::{List, Writer, bytes, stream::Up};
fn read(input: &[u8], limits: &Limits) -> List<Found> {
    let (mut env, _, mut above, mut below) = fixtures::opened_list();
    env.limits = *limits;
    let mut reader = packages::Stanzas::new(limits);
    packages::open(&mut reader, &env, &mut below);
    drop(below.pop());
    let mut found = List::with_capacity(16);
    let mut start = 0;
    for (at, byte) in input.iter().enumerate() {
        let end = at.checked_add(1).unwrap();
        if *byte == b'\n'
            || end.checked_sub(start).unwrap() == usize::try_from(packages::largest_demand(limits)).unwrap()
            || end == input.len()
        {
            packages::up(&mut reader, &env, Up::Bytes(bytes::copy_of(&input[start..=at])), &mut above, &mut below);
            start = end;
            drop(below.pop());
            for _ in 0..packages::UP_MAX_OUT.above {
                if let Some(event) = above.pop() {
                    found.push(event).unwrap();
                }
            }
        }
    }
    packages::up(&mut reader, &env, Up::End, &mut above, &mut below);
    for _ in 0..packages::UP_MAX_OUT.above {
        if let Some(event) = above.pop() {
            found.push(event).unwrap();
        }
    }
    found
}
#[test]
fn a_stanza_gives_its_six_fields_and_skips_the_rest() {
    let (env, mut stanzas, mut above, mut below) = fixtures::opened_list();
    for line in fixtures::APT_LINES {
        packages::up(&mut stanzas, &env, Up::Bytes(bytes::copy_of(line)), &mut above, &mut below);
        assert!(below.pop().is_some(), "each scan is followed by the next demand");
    }
    assert_eq!(above.pop(), Some(Found::Stanza(fixtures::apt_stanza())));
    assert_eq!(above.pop(), None);
}
#[test]
fn field_names_match_whatever_their_case() {
    const CASES: &[&[u8]] = &[b"pAcKaGe: apt\nvErSiOn: 3.1.6\nARCHITECTURE: amd64\nfilename: pool/main/a/apt/apt_3.1.6_amd64.deb\nSIZE: 1736\nsHa256: 2C26B46B68FFC68FF99B453C1D30413413422D706483BFA0F98A5E886266E7AE\n"];
    for input in CASES {
        let found = read(input, &limits::SHIPPED);
        assert_eq!(found.get(0), Some(&Found::Stanza(fixtures::apt_stanza())));
    }
}
#[test]
fn the_last_stanza_ends_at_the_end_of_the_list() {
    let found = read(fixtures::APT, &limits::SHIPPED);
    assert_eq!(found.get(0), Some(&Found::Stanza(fixtures::apt_stanza())));
    assert_eq!(found.get(1), Some(&Found::Ended));
    assert_eq!(found.len(), 2);
}
#[test]
fn a_stanza_past_its_limit_fails_the_list() {
    // The stanza budget is deliberately small; no MiB fixture is needed.
    const LIMITS: Limits = Limits { stanza: 1_024, ..limits::SHIPPED };
    const EXPECTED: [Found; 1] = [Found::Failed(StanzaError::TooLong)];
    let mut input = Writer::new(1_025);
    input.put(b"Description: ").unwrap();
    for _ in 13_u32..1_025 {
        input.put(b"x").unwrap();
    }
    assert_eq!(read(&input.finish(), &LIMITS).as_slice(), EXPECTED);
}
#[test]
fn a_stanza_past_64_kib_with_one_long_provides_line_is_read() {
    let mut input = Writer::new(fixtures::APT.len() + 70_012);
    input.put(fixtures::APT).unwrap();
    input.put(b"Provides: ").unwrap();
    for _ in 0_u32..70_000 {
        input.put(b"x").unwrap();
    }
    input.put(b"\n\n").unwrap();
    let found = read(&input.finish(), &limits::SHIPPED);
    assert_eq!(found.get(0), Some(&Found::Stanza(fixtures::apt_stanza())));
    assert_eq!(found.get(1), Some(&Found::Ended));
}
const MISSING_REPEATED: &[(&[u8], StanzaError)] = &[
    (b"Version: 1\n\n", StanzaError::Missing(Field::Package)),
    (b"Package: apt\nPackage: apt\n", StanzaError::Repeated(Field::Package)),
];
#[test]
fn a_missing_or_repeated_field_fails_naming_it() {
    for (input, error) in MISSING_REPEATED {
        assert_eq!(read(input, &limits::SHIPPED).get(0), Some(&Found::Failed(error.clone())), "{input:?}");
    }
}
const MALFORMED: &[&[u8]] = &[b" orphan\n", b"not-a-field\n", b": value\n"];
#[test]
fn a_line_without_a_field_or_preceding_field_fails() {
    const EXPECTED: Found = Found::Failed(StanzaError::Malformed);
    for input in MALFORMED {
        assert_eq!(read(input, &limits::SHIPPED).get(0), Some(&EXPECTED), "{input:?}");
    }
}
#[test]
fn a_required_field_cannot_be_continued() {
    const CASES: &[(&[u8], Found)] =
        &[(b"Package: apt\n continued\n", Found::Failed(StanzaError::Value(Field::Package)))];
    for (input, expected) in CASES {
        assert_eq!(read(input, &limits::SHIPPED).get(0), Some(expected), "{input:?}");
    }
}
const BAD_VALUES: &[(&[u8], Field)] = &[
    (b"Size: 18446744073709551616\n", Field::Size),
    (b"Size: -1\n", Field::Size),
    (b"SHA256: 2c\n", Field::Sha256),
    (b"SHA256: zc26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae\n", Field::Sha256),
    (b"Filename: /pool/apt.deb\n", Field::Filename),
    (b"Filename: pool/../apt.deb\n", Field::Filename),
    (b"Filename: ./apt.deb\n", Field::Filename),
    (b"Filename: pool/apt deb\n", Field::Filename),
    (b"Version: bad_version\n", Field::Version),
    (b"Package: Apt\n", Field::Package),
    (b"Architecture: am_d64\n", Field::Architecture),
];
#[test]
fn a_size_digest_or_filename_outside_its_form_fails() {
    for (replacement, field) in BAD_VALUES {
        let mut parts = List::with_capacity(9);
        let mut length = 0;
        for line in fixtures::APT_LINES.into_iter().take(8) {
            let part = match bytes::find(line, b":") {
                Some(colon) if replacement.starts_with(&line[..=colon]) => *replacement,
                Some(_) | None => line,
            };
            length += part.len();
            parts.push(part).unwrap();
        }
        let mut input = Writer::new(length);
        for part in &parts {
            input.put(part).unwrap();
        }
        assert_eq!(
            read(&input.finish(), &limits::SHIPPED).get(0),
            Some(&Found::Failed(StanzaError::Value(*field))),
            "{replacement:?}"
        );
    }
}
const INDEX_NAMES: &[(&[u8], &[u8])] = &[
    (b"InRelease", b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_InRelease"),
    (
        b"main/binary-amd64/Packages",
        b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_main_binary-amd64_Packages",
    ),
];
#[test]
fn each_source_line_names_its_inrelease_and_packages_lists() {
    let lines = vector::lines(&fixtures::forky_main()).unwrap();
    let indices = lists::indices(&lines, &fixtures::name(b"amd64"));
    assert_eq!(indices.len(), INDEX_NAMES.len());
    for (index, (path, name)) in indices.iter().zip(INDEX_NAMES) {
        assert_eq!((&*index.path, &*index.name, index.line), (*path, *name, 0));
    }
}
const LISTED: &[(&[u8], Listed)] = &[
    (
        b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_main_binary-amd64_Packages.xz",
        Listed::Compressed(1),
    ),
    (
        b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_main_binary-amd64_Packages.gz",
        Listed::Compressed(1),
    ),
    (
        b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_main_binary-amd64_Packages.lz4",
        Listed::Compressed(1),
    ),
    (b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_InRelease", Listed::Index(0)),
    (b"lock", Listed::Other),
    (b"partial/", Listed::Other),
];
#[test]
fn a_compressed_list_is_named_compressed_never_an_index() {
    let lines = vector::lines(&fixtures::forky_main()).unwrap();
    let indices = lists::indices(&lines, &fixtures::name(b"amd64"));
    for (name, expected) in LISTED {
        assert_eq!(lists::listed(name, &indices), *expected, "{name:?}");
    }
}
