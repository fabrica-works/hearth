use super::fixtures;
use crate::resolve::{self, Broken, Failure, Field, Request, Variant};
use hearth_fake_machine::hearth::{draw::Draw, universe::Universe};
use skein_lib::Rng;

#[test]
fn the_apt_variant_takes_the_essential_set_apt_and_their_dependencies() {
    let universe = fixtures::debian();
    let request = Request { universes: &[&universe], variant: Variant::Apt, include: &[] };
    let closed = resolve::closure(&request).expect("the fixture resolves");
    assert_eq!(
        fixtures::names(&[&universe], &closed),
        ["apt", "base-files", "base-passwd", "debconf", "dpkg", "libc6", "mawk"]
    );
}
#[test]
fn a_virtual_dependency_takes_its_highest_priority_provider() {
    let universe = fixtures::debian();
    let closed = resolve::closure(&Request { universes: &[&universe], variant: Variant::Essential, include: &[] })
        .expect("essential resolves");
    assert_eq!(fixtures::names(&[&universe], &closed), ["base-files", "base-passwd", "dpkg", "libc6", "mawk"]);
}
#[test]
fn a_requested_provider_satisfies_a_group_before_its_first_alternative() {
    let universe = fixtures::debian();
    let include = fixtures::includes(&[b"linux-image-amd64", b"tiny-initramfs"]);
    let closed = resolve::closure(&Request { universes: &[&universe], variant: Variant::Essential, include: &include })
        .expect("requested provider resolves");
    assert_eq!(
        fixtures::names(&[&universe], &closed),
        ["base-files", "base-passwd", "dpkg", "libc6", "linux-image-amd64", "mawk", "tiny-initramfs"]
    );
}
#[test]
fn a_versioned_dependency_passes_over_an_alternative_too_old() {
    let universe = fixtures::debian();
    let include = fixtures::includes(&[b"linux-image-amd64"]);
    let closed = resolve::closure(&Request { universes: &[&universe], variant: Variant::Essential, include: &include })
        .expect("a later alternative resolves");
    assert_eq!(
        fixtures::names(&[&universe], &closed),
        ["base-files", "base-passwd", "dpkg", "initramfs-tools", "libc6", "linux-image-amd64", "mawk"]
    );
}
#[test]
fn a_request_no_package_carries_fails_as_unknown() {
    let universe = fixtures::debian();
    let include = fixtures::includes(&[b"unknown"]);
    assert_eq!(
        resolve::closure(&Request { universes: &[&universe], variant: Variant::Apt, include: &include }),
        Err(Failure::Unknown(Box::from(&b"unknown"[..])))
    );
}
#[test]
fn a_virtual_request_with_two_providers_has_no_candidate() {
    let universe = fixtures::debian();
    let include = fixtures::includes(&[b"mail-transport-agent"]);
    assert_eq!(
        resolve::closure(&Request { universes: &[&universe], variant: Variant::Apt, include: &include }),
        Err(Failure::NoCandidate(Box::from(&b"mail-transport-agent"[..])))
    );
}
#[test]
fn two_requests_that_conflict_fail_naming_both() {
    let universe = fixtures::debian();
    let include = fixtures::includes(&[b"exim4-daemon-light", b"postfix"]);
    assert_eq!(
        resolve::closure(&Request { universes: &[&universe], variant: Variant::Apt, include: &include }),
        Err(Failure::Unsatisfiable(Box::new([
            Broken {
                package: Box::from(&b"exim4-daemon-light"[..]),
                field: Field::Conflicts,
                relation: Box::from(&b"mail-transport-agent"[..]),
                against: Some(Box::from(&b"3.10.4-1"[..]))
            },
            Broken {
                package: Box::from(&b"postfix"[..]),
                field: Field::Conflicts,
                relation: Box::from(&b"mail-transport-agent"[..]),
                against: Some(Box::from(&b"4.98.2-1"[..]))
            },
        ])))
    );
}
const PRIORITIES: &[(Variant, &[&str])] = &[
    (Variant::Required, &["apt", "base-files", "base-passwd", "debconf", "dpkg", "libc6", "mawk"]),
    (Variant::Important, &["adduser", "apt", "base-files", "base-passwd", "debconf", "dpkg", "libc6", "mawk"]),
    (Variant::Standard, &["adduser", "apt", "base-files", "base-passwd", "debconf", "dpkg", "gawk", "libc6", "mawk"]),
];
#[test]
fn each_priority_variant_adds_its_priority_to_the_essential_set() {
    let universe = fixtures::debian();
    for &(variant, expected) in PRIORITIES {
        let closed =
            resolve::closure(&Request { universes: &[&universe], variant, include: &[] }).expect("priority resolves");
        assert_eq!(fixtures::names(&[&universe], &closed), expected, "{variant:?}");
    }
}
#[test]
fn every_drawn_universe_resolves_its_apt_variant() {
    for seed in 0..16 {
        let universe = Universe::draw(&mut Rng::new(seed), &Draw { packages: 24, names: &[] });
        assert_eq!(
            resolve::closure(&Request { universes: &[&universe], variant: Variant::Apt, include: &[] }).map(|_| ()),
            Ok(()),
            "seed {seed}"
        );
    }
}
#[test]
fn broken_relations_are_listed_at_most_most_broken() {
    let universe = fixtures::missing_groups(65);
    let result = resolve::closure(&Request { universes: &[&universe], variant: Variant::Essential, include: &[] });
    let Failure::Unsatisfiable(broken) = result.expect_err("missing groups cannot resolve") else {
        panic!("unmet groups are Unsatisfiable");
    };
    assert_eq!(broken.len(), 64);
}
const PROVIDED_VERSIONS: &[(Option<&[u8]>, bool)] = &[(None, false), (Some(b"1"), false), (Some(b"2"), true)];
#[test]
fn a_versioned_dependency_requires_a_sufficient_versioned_provide() {
    for &(version, expected) in PROVIDED_VERSIONS {
        let universe = fixtures::dependency_on_awk(version);
        assert_eq!(
            resolve::closure(&Request { universes: &[&universe], variant: Variant::Essential, include: &[] }).is_ok(),
            expected,
            "provide {version:?}"
        );
    }
}
#[test]
fn an_unmet_group_reports_its_owner_field_and_relation() {
    let universe = fixtures::dependency_on_awk(None);
    assert_eq!(
        resolve::closure(&Request { universes: &[&universe], variant: Variant::Essential, include: &[] }),
        Err(Failure::Unsatisfiable(Box::new([Broken {
            package: Box::from(&b"base-files"[..]),
            field: Field::PreDepends,
            relation: Box::from(&b"awk (>= 2)"[..]),
            against: None,
        }])))
    );
}
#[test]
fn an_alternative_conflicting_with_a_request_is_skipped() {
    let universe = fixtures::conflict_with_initramfs();
    let include = fixtures::includes(&[b"linux-image-amd64", b"postfix"]);
    let closed = resolve::closure(&Request { universes: &[&universe], variant: Variant::Essential, include: &include })
        .expect("the second alternative is free of conflicts");
    assert_eq!(
        fixtures::names(&[&universe], &closed),
        ["base-files", "base-passwd", "dpkg", "initramfs-tools", "libc6", "linux-image-amd64", "mawk", "postfix"]
    );
}
#[test]
fn a_higher_version_from_a_later_source_is_the_candidate() {
    let old = fixtures::debian();
    let mut newer = old.package(b"mawk").expect("fixture mawk").clone();
    newer.version = Box::from(&b"2.0"[..]);
    let new = Universe::new(Box::new([newer]));
    let closed = resolve::closure(&Request { universes: &[&old, &new], variant: Variant::Essential, include: &[] })
        .expect("highest version resolves");
    assert_eq!(
        closed.as_ref(),
        &[
            resolve::Selected { universe: 0, package: 0 },
            resolve::Selected { universe: 0, package: 1 },
            resolve::Selected { universe: 0, package: 2 },
            resolve::Selected { universe: 1, package: 0 },
            resolve::Selected { universe: 0, package: 3 },
        ]
    );
}

#[test]
fn the_read_set_keeps_ranked_providers_in_universe_order() {
    let universe = fixtures::debian();
    let read = resolve::read_by(&Request { universes: &[&universe], variant: Variant::Essential, include: &[] });
    assert_eq!(
        read.as_ref(),
        &[
            resolve::Selected { universe: 0, package: 0 },
            resolve::Selected { universe: 0, package: 1 },
            resolve::Selected { universe: 0, package: 2 },
            resolve::Selected { universe: 0, package: 3 },
            resolve::Selected { universe: 0, package: 4 },
            resolve::Selected { universe: 0, package: 5 },
        ],
        "gawk was ranked against mawk even though it was not marked"
    );
}
