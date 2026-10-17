use super::fixtures;
use crate::hearth::{
    universe::{IndexError, Priority, Relation, Universe, Wrong},
    version,
};
use core::cmp::Ordering;

// The orders were checked with dpkg --compare-versions.
const VERSIONS: &[(&[u8], &[u8], Ordering)] = &[
    (b"1.0~rc1", b"1.0", Ordering::Less),
    (b"2.41-12", b"2.41-9", Ordering::Greater),
    (b"1:5.3.2-1", b"1.3.4.20260302-2", Ordering::Greater),
    (b"0.1-8+b+~", b"0.1-8", Ordering::Greater),
    (b"1.0", b"1.0-0", Ordering::Equal),
    (b"1.0-0", b"0:1.0", Ordering::Equal),
    (b"1.0+dfsg", b"1.0", Ordering::Greater),
    (b"1.0a", b"1.0+", Ordering::Less),
    (b"0000000000000000000000001:2", b"1:2", Ordering::Equal),
    (b"1.0~~", b"1.0~", Ordering::Less),
];
#[test]
fn versions_compare_as_dpkg_compares_them() {
    for &(a, b, expected) in VERSIONS {
        assert_eq!(version::compare(a, b), expected, "{a:?} versus {b:?}");
    }
}
#[test]
fn a_real_stanza_written_back_keeps_its_kept_fields_byte_for_byte() {
    let universe = Universe::read(fixtures::MAWK).expect("a real stanza reads");
    assert_eq!(universe.packages().len(), 1);
    assert_eq!(&*universe.index(), fixtures::MAWK_KEPT, "unknown fields are dropped");
    assert_eq!(universe.package(b"mawk").map(|package| package.priority), Some(Priority::Required));
}
type ReadAlternative<'a> = (&'a [u8], Option<&'a [u8]>, Option<(Relation, &'a [u8])>);
const ALTERNATIVES: &[ReadAlternative<'_>] =
    &[(b"perl", Some(b"any"), Some((Relation::LaterOrEqual, b"1:2.0-1"))), (b"awk", None, None)];
#[test]
fn relations_read_back_with_alternatives_versions_and_qualifiers() {
    let text = core::str::from_utf8(fixtures::MAWK_KEPT)
        .expect("ASCII")
        .replace("libc6 (>= 2.38)", "perl:any (>= 1:2.0-1) | awk");
    let universe = Universe::read(text.as_bytes()).expect("relations read");
    let package = universe.package(b"mawk").expect("mawk");
    let group = &package.depends[0];
    let got: Vec<_> =
        group.iter().map(|a| (&*a.name, a.qualifier.as_deref(), a.version.as_ref().map(|(r, v)| (*r, &**v)))).collect();
    assert_eq!(got, ALTERNATIVES);
}
#[test]
fn a_provided_name_lists_its_providers_in_index_order() {
    let other = core::str::from_utf8(fixtures::MAWK_KEPT).expect("ASCII").replace("Package: mawk", "Package: gawk");
    let index = [fixtures::MAWK_KEPT, other.as_bytes()].concat();
    let universe = Universe::read(&index).expect("distinct names");
    assert_eq!(universe.providers(b"awk"), &[0, 1]);
    assert_eq!(universe.providers(b"missing"), &[]);
}
const REQUIRED: &[(&str, Wrong)] = &[
    ("Package: mawk\n", Wrong::Missing("Package")),
    ("Version: 1.3.4.20260302-2\n", Wrong::Missing("Version")),
    ("Architecture: amd64\n", Wrong::Missing("Architecture")),
    ("Section: interpreters\n", Wrong::Missing("Section")),
    ("Priority: required\n", Wrong::Missing("Priority")),
    ("Filename: pool/main/m/mawk/mawk_1.3.4.20260302-2_amd64.deb\n", Wrong::Missing("Filename")),
    ("Size: 144700\n", Wrong::Missing("Size")),
    ("SHA256: 6fcd463ea272c05627d5159716336b53bfa14b35b1ac7ba29a878f0ee1352419\n", Wrong::Missing("SHA256")),
];
#[test]
fn a_stanza_missing_a_kept_field_is_refused_naming_its_line() {
    for &(remove, wrong) in REQUIRED {
        let text = core::str::from_utf8(fixtures::MAWK_KEPT).expect("ASCII").replace(remove, "");
        assert_eq!(Universe::read(text.as_bytes()).err(), Some(IndexError { line: 1, wrong }), "{remove}");
    }
}
#[test]
fn a_package_named_twice_is_refused_naming_the_second() {
    let index = [fixtures::MAWK_KEPT, fixtures::MAWK_KEPT].concat();
    assert_eq!(Universe::read(&index).err(), Some(IndexError { line: 13, wrong: Wrong::Duplicate }));
}
const TWICE: &[(&[u8], IndexError)] = &[
    (b"Package: mawk\n", IndexError { line: 2, wrong: Wrong::Twice("Package") }),
    (b"Version: 1.0\n", IndexError { line: 3, wrong: Wrong::Twice("Version") }),
];
#[test]
fn a_kept_field_given_twice_is_refused_as_twice() {
    for &(prefix, expected) in TWICE {
        let index = [prefix, fixtures::MAWK_KEPT].concat();
        assert_eq!(Universe::read(&index).err(), Some(expected), "{prefix:?}");
    }
}
const FORMS: &[(&str, &str, IndexError)] = &[
    ("Package: mawk", "Package: bad_name", IndexError { line: 1, wrong: Wrong::Form("Package") }),
    ("Version: 1.3.4.20260302-2", "Version: 1.0-", IndexError { line: 2, wrong: Wrong::Form("Version") }),
    ("Installed-Size: 299", "Installed-Size: -1", IndexError { line: 3, wrong: Wrong::Form("Installed-Size") }),
    ("Architecture: amd64", "Architecture: two words", IndexError { line: 4, wrong: Wrong::Form("Architecture") }),
    ("Provides: awk", "Provides: awk (>= 1)", IndexError { line: 5, wrong: Wrong::Form("Provides") }),
    ("Depends: libc6 (>= 2.38)", "Depends: libc6 (> 2.38)", IndexError { line: 6, wrong: Wrong::Form("Depends") }),
    ("Priority: required", "Priority: unknown", IndexError { line: 8, wrong: Wrong::Form("Priority") }),
    ("Size: 144700", "Size: 18446744073709551616", IndexError { line: 10, wrong: Wrong::Form("Size") }),
    (
        "SHA256: 6fcd463ea272c05627d5159716336b53bfa14b35b1ac7ba29a878f0ee1352419",
        "SHA256: abc",
        IndexError { line: 11, wrong: Wrong::Form("SHA256") },
    ),
];
#[test]
fn a_value_out_of_form_is_refused_as_form() {
    for &(from, to, expected) in FORMS {
        let text = core::str::from_utf8(fixtures::MAWK_KEPT).expect("ASCII").replace(from, to);
        assert_eq!(Universe::read(text.as_bytes()).err(), Some(expected), "{to}");
    }
}
const ESSENTIAL: &[(&[u8], Result<bool, IndexError>)] = &[
    (b"", Ok(false)),
    (b"Essential: yes\n", Ok(true)),
    (b"Essential: no\n", Err(IndexError { line: 1, wrong: Wrong::Form("Essential") })),
    (b"Essential: true\n", Err(IndexError { line: 1, wrong: Wrong::Form("Essential") })),
];
#[test]
fn essential_is_true_for_yes_alone() {
    for &(prefix, expected) in ESSENTIAL {
        let index = [prefix, fixtures::MAWK_KEPT].concat();
        let got = Universe::read(&index).map(|u| u.packages()[0].essential);
        assert_eq!(got, expected, "{prefix:?}");
    }
}
#[test]
#[should_panic(expected = "one package a name")]
fn a_universe_made_with_one_name_twice_asserts() {
    let universe = Universe::read(fixtures::MAWK_KEPT).expect("fixture reads");
    let package = universe.packages()[0].clone();
    drop(Universe::new(Box::new([package.clone(), package])));
}
#[test]
fn a_real_stanza_reads_its_conflicts() {
    let universe = Universe::read(fixtures::TINY_INITRAMFS).expect("the real stanza reads");
    let package = universe.package(b"tiny-initramfs").expect("the named package");
    let conflicts: Vec<_> = package.conflicts.iter().map(|a| &*a.name).collect();
    assert_eq!(conflicts, &[b"dracut".as_slice(), b"initramfs-tools", b"live-tools"]);
}

const RELATIONS: &[(Relation, &[u8], &[u8], bool)] = &[
    (Relation::Earlier, b"1.0~rc1", b"1.0", true),
    (Relation::Earlier, b"1.0", b"1.0", false),
    (Relation::EarlierOrEqual, b"1.0-0", b"1.0", true),
    (Relation::EarlierOrEqual, b"2", b"1", false),
    (Relation::Equal, b"0:1.0", b"1.0-0", true),
    (Relation::Equal, b"2", b"1", false),
    (Relation::LaterOrEqual, b"1.0", b"1.0-0", true),
    (Relation::LaterOrEqual, b"1", b"2", false),
    (Relation::Later, b"2.41-12", b"2.41-9", true),
    (Relation::Later, b"1", b"1", false),
];
#[test]
fn relations_use_debians_version_order() {
    for &(relation, have, wanted, expected) in RELATIONS {
        assert_eq!(relation.holds(have, wanted), expected, "{relation:?} {have:?} {wanted:?}");
    }
}
#[test]
fn a_versioned_provide_keeps_its_version() {
    let text =
        core::str::from_utf8(fixtures::MAWK_KEPT).expect("ASCII").replace("Provides: awk", "Provides: awk (= 2.0)");
    let universe = Universe::read(text.as_bytes()).expect("versioned provide");
    let provide = &universe.packages()[0].provides[0];
    assert_eq!((&*provide.name, provide.version.as_deref()), (b"awk".as_slice(), Some(b"2.0".as_slice())));
}
