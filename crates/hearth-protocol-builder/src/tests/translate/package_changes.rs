use crate::{records, render::diff, tests::fixtures, translate::changes};
#[test]
fn each_packages_change_translates_and_back() {
    for (row, input) in fixtures::package_changes().into_iter().enumerate() {
        let records::diff::PartChange::Packages(value) =
            changes::part_change(diff::PartChange::Packages(input.clone()))
        else {
            panic!("packages")
        };
        assert_eq!(fixtures::packages_change_back(value), input, "row {row}: packages change");
    }
}
#[test]
fn each_boot_change_translates_and_back() {
    for (row, input) in fixtures::boot_changes().into_iter().enumerate() {
        let records::diff::PartChange::Boot(value) = changes::part_change(diff::PartChange::Boot(input.clone())) else {
            panic!("boot")
        };
        assert_eq!(fixtures::boot_change_back(value), input, "row {row}: boot change");
    }
}
