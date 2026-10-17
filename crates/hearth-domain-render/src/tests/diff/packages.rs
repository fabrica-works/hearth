use crate::{
    diff::{self, Layer, Rebuild, TargetChange},
    parts::{PartRecord, boot, packages::*},
    tests::fixtures,
};
use skein_lib::bytes::copy_of;
#[test]
fn the_packages_change_lists_added_and_removed_names_sorted() {
    let mut before = fixtures::thin_packages();
    before.request = Box::new([copy_of(b"tzdata"), copy_of(b"curl"), copy_of(b"zstd")]);
    let mut after = before.clone();
    after.request = Box::new([copy_of(b"vim"), copy_of(b"curl"), copy_of(b"apt")]);
    assert_eq!(
        change(&before, &after).request,
        Names {
            added: Box::new([copy_of(b"apt"), copy_of(b"vim")]),
            removed: Box::new([copy_of(b"tzdata"), copy_of(b"zstd")]),
        }
    );
}
#[test]
fn a_field_equal_on_both_sides_is_left_out() {
    let packages = fixtures::thin_packages();
    assert_eq!(
        change(&packages, &packages),
        Change {
            resolution: diff::ResolutionChange::Same,
            request: Names { added: Box::new([]), removed: Box::new([]) },
            sources: None,
            variant: None,
            preseeds: None,
            initramfs: None,
            apt: None,
        }
    );
}
#[test]
fn a_changed_architecture_is_a_sources_pair() {
    let before = fixtures::thin_packages();
    let mut after = before.clone();
    after.sources.architecture = copy_of(b"arm64");
    assert_eq!(
        change(&before, &after).sources,
        Some(SourcesPair {
            before: Sources {
                suite: copy_of(b"forky"),
                architecture: copy_of(b"amd64"),
                components: Box::new([Component::Main]),
                archive: copy_of(b"20261001T030405Z"),
                updates: false,
                security: None
            },
            after: Sources {
                suite: copy_of(b"forky"),
                architecture: copy_of(b"arm64"),
                components: Box::new([Component::Main]),
                archive: copy_of(b"20261001T030405Z"),
                updates: false,
                security: None
            },
        })
    );
}
#[test]
fn boot_changes_as_its_before_and_after() {
    let before = boot::Boot { params: Box::new([copy_of(b"quiet")]), mutable: false };
    let after = boot::Boot { params: Box::new([]), mutable: true };
    assert_eq!(
        boot::change(&before, &after),
        boot::Change {
            before: boot::Boot { params: Box::new([copy_of(b"quiet")]), mutable: false },
            after: boot::Boot { params: Box::new([]), mutable: true },
        }
    );
}
#[test]
fn the_boot_part_is_a_slot_layer_and_packages_the_lowest() {
    assert_eq!(diff::layer(&PartRecord::Boot(boot::Boot { params: Box::new([]), mutable: false })), Layer::Slot);
    assert_eq!(diff::layer(&PartRecord::Packages(fixtures::thin_packages())), Layer::Packages);
}
#[test]
fn a_preseed_only_change_rebuilds_from_packages() {
    let before = fixtures::thin_packages();
    let mut after = before.clone();
    after.preseeds = Box::new([fixtures::preseed_answer()]);
    assert_eq!(inputs(&before), inputs(&after));
    let TargetChange::Changed { rebuild, .. } =
        fixtures::packages::parts_changed(PartRecord::Packages(before), PartRecord::Packages(after))
    else {
        panic!("changed packages")
    };
    assert_eq!(rebuild, Rebuild::FromPackages { resolution: diff::Needs::Known });
}
#[test]
fn an_initramfs_only_change_rebuilds_from_packages() {
    let before = fixtures::thin_packages();
    let mut after = before.clone();
    after.initramfs.as_mut().expect("metal").modules = Box::new([copy_of(b"nvme")]);
    assert_eq!(inputs(&before), inputs(&after));
    let TargetChange::Changed { rebuild, .. } =
        fixtures::packages::parts_changed(PartRecord::Packages(before), PartRecord::Packages(after))
    else {
        panic!("changed packages")
    };
    assert_eq!(rebuild, Rebuild::FromPackages { resolution: diff::Needs::Known });
}

#[test]
fn a_changed_variant_keeps_its_before_and_after() {
    let before = fixtures::thin_packages();
    let mut after = before.clone();
    after.variant = Variant::Container;
    assert_eq!(change(&before, &after).variant, Some(VariantPair { before: Variant::Apt, after: Variant::Container }));
}
#[test]
fn a_changed_apt_option_keeps_its_before_and_after() {
    let before = fixtures::thin_packages();
    let mut after = before.clone();
    after.apt.install_recommends = true;
    assert_eq!(
        change(&before, &after).apt,
        Some(AptPair { before: Apt { install_recommends: false }, after: Apt { install_recommends: true } })
    );
}
#[test]
fn a_changed_initramfs_keeps_its_before_and_after() {
    let before = fixtures::thin_packages();
    let mut after = before.clone();
    after.initramfs = None;
    assert_eq!(
        change(&before, &after).initramfs,
        Some(InitramfsPair {
            before: Some(Initramfs { modules: Box::new([]), microcode: Microcode::Generic }),
            after: None
        })
    );
}
