use super::fixtures;
use crate::{
    form::Form,
    records::{PartRecord, Resolution},
};
#[test]
fn the_packages_line_ends_with_its_resolution_as_render_md_shows() {
    let spec = fixtures::with_resolution(
        fixtures::packages_spec(),
        Resolution::Needed { key: fixtures::digest([0x0c, 0x7d, 0x9e, 0x41, 0xc0, 0xd2]) },
    );
    assert!(
        skein_lib::bytes::find(
            &fixtures::packages_text(spec),
            b"packages     part 4be1c0d2a7f1   resolution 0c7d9e41c0d2 needed\n"
        )
        .is_some()
    );
}
#[test]
fn a_spec_without_a_resolution_ends_the_line_at_its_digest() {
    assert!(
        skein_lib::bytes::find(
            &fixtures::packages_text(fixtures::packages_spec()),
            b"packages     part 4be1c0d2a7f1\n"
        )
        .is_some()
    );
}
#[test]
fn a_known_resolution_ends_the_line_known() {
    let spec = fixtures::with_resolution(fixtures::packages_spec(), fixtures::known());
    assert!(
        skein_lib::bytes::find(
            &fixtures::packages_text(spec),
            b"packages     part 4be1c0d2a7f1   resolution 0c7d9e41c0d2 known\n"
        )
        .is_some()
    );
}
#[test]
fn all_four_components_print_in_fixed_order() {
    assert!(skein_lib::bytes::find(&fixtures::packages_text(fixtures::packages_spec()), b"  sources    forky amd64 main contrib non-free non-free-firmware 20261001T030405Z updates security 20261002T030405Z\n").is_some());
}
#[test]
fn json_gives_the_packages_record_in_key_order() {
    let json = fixtures::packages_json(fixtures::packages_spec());
    let start = skein_lib::bytes::find(&json, br#"{"record":"part","target":"x1-prod","name":"packages""#)
        .expect("packages part");
    assert_eq!(json.get(start..).expect("part suffix"), concat!(r#"{"record":"part","target":"x1-prod","name":"packages","digest":"4be1c0d2a7f10000000000000000000000000000000000000000000000000000","packages":{"apt":{"install_recommends":false},"initramfs":{"microcode":"generic","modules":[]},"preseeds":{},"request":["curl","tzdata"],"sources":{"architecture":"amd64","archive":"20261001T030405Z","components":["main","contrib","non-free","non-free-firmware"],"security":"20261002T030405Z","suite":"forky","updates":true},"variant":"apt"}}"#, "\n").as_bytes());
}
#[test]
fn a_component_error_names_the_package_and_the_component() {
    let mut error = fixtures::modules_guard();
    error.kind = crate::errors::Kind::Schema(crate::errors::Schema::Component {
        package: b"firmware-linux".as_slice().into(),
        component: b"non-free-firmware".as_slice().into(),
    });
    let text = fixtures::print_error(Form::Text, error).expect("component error");
    assert_eq!(text.as_ref(), b"tasks/boot/task.json:22:7: carbon-lab: boot: firmware-linux needs component non-free-firmware, which debian.components does not enable\n");
}
#[test]
fn preseeds_and_modules_print_as_render_md_9_2_shows() {
    let text = fixtures::packages_text(fixtures::preseeded_spec());
    assert!(skein_lib::bytes::find(&text, b"  variant    apt\n  preseed    a/answer string word\n  preseed    tzdata/Areas select Europe owner other\n  initramfs  btrfs nvme\n  microcode  generic\n").is_some());
}

#[test]
fn json_gives_the_initramfs_file_beside_the_packages_digest() {
    let mut spec = fixtures::packages_spec();
    for part in &mut spec.parts {
        if let PartRecord::Packages(packages) = &mut part.record {
            packages.initramfs_file = Some([7; 32]);
        }
    }
    let json = fixtures::packages_json(spec);
    assert!(skein_lib::bytes::find(&json, br#""digest":"4be1c0d2a7f10000000000000000000000000000000000000000000000000000","initramfs_file":"0707070707070707070707070707070707070707070707070707070707070707","packages":{"#).is_some());
}
