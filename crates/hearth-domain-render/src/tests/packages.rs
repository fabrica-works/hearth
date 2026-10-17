use super::fixtures::{self, bytes};
use crate::{
    errors::{Format, Schema},
    parts::packages::{self, Component, Variant},
    revision::Runs,
};
use hearth_lib::encode::Item;
use skein_lib::bytes::copy_of;
#[test]
fn the_resolution_inputs_are_the_four_values_as_the_record_encodes_them() {
    let inputs = packages::inputs(&fixtures::thin_packages());
    assert_eq!(
        &*inputs.sources,
        &[
            Item::Map(5),
            bytes(b"architecture"),
            bytes(b"amd64"),
            bytes(b"archive"),
            bytes(b"20261001T030405Z"),
            bytes(b"components"),
            Item::List(1),
            bytes(b"main"),
            bytes(b"suite"),
            bytes(b"forky"),
            bytes(b"updates"),
            Item::False
        ],
        "store.md 3.1"
    );
    assert_eq!((&*inputs.request, &*inputs.variant), (&[Item::List(0)][..], &[bytes(b"apt")][..]), "render.md 5.1");
    assert_eq!(&*inputs.apt, &[Item::Map(1), bytes(b"install_recommends"), Item::False]);
}
#[test]
fn the_packages_digest_is_h_of_part_packages_over_its_record() {
    assert_eq!(
        packages::digest(&fixtures::thin_packages()),
        fixtures::digest("de38a7ed6276e7c90818dd41185f34731cad522522bb65e8ba08bb626b1fe35d")
    );
}
#[test]
fn a_request_is_the_union_of_the_tasks_names_sorted_once() {
    let packages = fixtures::built_packages(&[(b"a", b"tzdata", None), (b"b", b"curl", None), (b"c", b"tzdata", None)])
        .expect("valid packages");
    assert_eq!(packages.request.as_ref(), [copy_of(b"curl"), copy_of(b"tzdata")]);
}
const INVALID: &[&[u8]] = &[b"a", b"Curl", b"-curl", b"curl:amd64", b"curl_x", b"curl "];
#[test]
fn a_name_out_of_policys_form_is_refused_at_its_entry() {
    for (row, name) in INVALID.iter().enumerate() {
        assert_eq!(
            crate::kinds::packages::entry(&fixtures::package(name, None)),
            Err((Schema::Malformed { format: Format::Debian }, fixtures::composition::at(1, 14))),
            "row {row}"
        );
    }
}
const LENGTHS: &[(usize, bool)] = &[
    (fixtures::MIN_PACKAGE_NAME - 1, false),
    (fixtures::MIN_PACKAGE_NAME, true),
    (fixtures::MAX_PACKAGE_NAME, true),
    (fixtures::MAX_PACKAGE_NAME + 1, false),
];
#[test]
fn package_names_have_policys_length_bounds() {
    for (row, &(length, valid)) in LENGTHS.iter().enumerate() {
        assert_eq!(
            crate::kinds::packages::entry(&fixtures::package(
                fixtures::PACKAGE_NAME_BYTES.get(..length).expect("name bound"),
                None
            ))
            .is_ok(),
            valid,
            "row {row}"
        );
    }
}
#[test]
fn one_package_of_two_tasks_with_two_components_is_one_name_each_checked() {
    let root = fixtures::packages_root(&[(b"contrib", true)], false, b"20261001T030405Z");
    let expansion = fixtures::package_expansion(&[(b"a", b"curl", None), (b"b", b"curl", Some(b"contrib"))]);
    let packages = packages::build(&root, &fixtures::containers::placement(None), &expansion, None)
        .expect("both components enabled");
    assert_eq!(packages.request.as_ref(), [copy_of(b"curl")]);
}
#[test]
fn a_component_the_target_does_not_enable_names_both() {
    let errors = fixtures::built_packages(&[(b"base", b"firmware-linux", Some(b"non-free-firmware"))])
        .expect_err("component off");
    assert_eq!(
        fixtures::only(&errors).kind,
        crate::errors::Kind::Schema(Schema::Component {
            package: copy_of(b"firmware-linux"),
            component: copy_of(b"non-free-firmware")
        })
    );
}
const VARIANTS: &[(Runs, Variant)] =
    &[(Runs::Metal, Variant::Apt), (Runs::Vm, Variant::Apt), (Runs::Container, Variant::Container)];
#[test]
fn the_variant_follows_the_placement() {
    for (row, &(kind, expected)) in VARIANTS.iter().enumerate() {
        let mut placement = fixtures::containers::placement(None);
        placement.kind = kind;
        let packages =
            packages::build(&fixtures::checked_debian(), &placement, &fixtures::package_expansion(&[]), None)
                .expect("valid debian");
        assert_eq!(packages.variant, expected, "row {row}");
    }
}
#[test]
fn every_value_schema_debian_makes_reads_back() {
    assert_eq!(
        packages::read(&fixtures::checked_debian()),
        Some((
            packages::Sources {
                suite: copy_of(b"trixie"),
                architecture: copy_of(b"amd64"),
                components: Box::new([Component::Main]),
                archive: copy_of(b"20261001T030405Z"),
                updates: true,
                security: Some(copy_of(b"20261001T030405Z"))
            },
            packages::Apt { install_recommends: false }
        ))
    );
}
#[test]
fn a_security_timestamp_with_its_pocket_off_changes_nothing() {
    let before = fixtures::packages_root(&[], false, b"20261001T030405Z");
    let after = fixtures::packages_root(&[], false, b"20261101T030405Z");
    assert_eq!(packages::read(&before), packages::read(&after));
}
#[test]
fn the_suite_changes_the_resolution_inputs() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.sources.suite = copy_of(b"trixie");
    assert_ne!(packages::inputs(&input), packages::inputs(&edited));
}
#[test]
fn the_architecture_changes_the_resolution_inputs() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.sources.architecture = copy_of(b"arm64");
    assert_ne!(packages::inputs(&input), packages::inputs(&edited));
}
#[test]
fn the_components_changes_the_resolution_inputs() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.sources.components = Box::new([Component::Main, Component::Contrib]);
    assert_ne!(packages::inputs(&input), packages::inputs(&edited));
}
#[test]
fn the_archive_changes_the_resolution_inputs() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.sources.archive = copy_of(b"20261101T030405Z");
    assert_ne!(packages::inputs(&input), packages::inputs(&edited));
}
#[test]
fn the_updates_changes_the_resolution_inputs() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.sources.updates = true;
    assert_ne!(packages::inputs(&input), packages::inputs(&edited));
}
#[test]
fn the_security_changes_the_resolution_inputs() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.sources.security = Some(copy_of(b"20261101T030405Z"));
    assert_ne!(packages::inputs(&input), packages::inputs(&edited));
}
#[test]
fn the_request_changes_the_resolution_inputs() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.request = Box::new([copy_of(b"curl")]);
    assert_ne!(packages::inputs(&input), packages::inputs(&edited));
}
#[test]
fn the_variant_changes_the_resolution_inputs() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.variant = Variant::Container;
    assert_ne!(packages::inputs(&input), packages::inputs(&edited));
}
#[test]
fn the_install_recommends_changes_the_resolution_inputs() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.apt.install_recommends = true;
    assert_ne!(packages::inputs(&input), packages::inputs(&edited));
}

#[test]
fn a_second_tasks_disabled_component_is_checked_even_for_a_requested_name() {
    let errors = fixtures::built_packages(&[(b"a", b"curl", None), (b"b", b"curl", Some(b"contrib"))])
        .expect_err("second component off");
    assert_eq!(fixtures::only(&errors).task.as_deref(), Some(b"b".as_slice()));
}
#[test]
fn a_component_the_target_does_not_enable_keeps_its_setting_place() {
    let set =
        crate::errors::Place { path: copy_of(b"hosts/x1/x1.json"), position: Some(fixtures::composition::at(1, 22)) };
    let errors = packages::build(
        &fixtures::checked_debian(),
        &fixtures::containers::placement(None),
        &fixtures::package_expansion(&[(b"base", b"firmware-linux", Some(b"non-free-firmware"))]),
        Some(&set),
    )
    .expect_err("component off");
    assert_eq!(
        fixtures::only(&errors).also,
        Some(crate::errors::Place {
            path: copy_of(b"hosts/x1/x1.json"),
            position: Some(fixtures::composition::at(1, 22))
        })
    );
}

#[path = "packages/preseeds.rs"]
mod preseeds;
