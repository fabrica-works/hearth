#[test]
fn each_packages_field_translates_to_the_records_and_back() {
    for (row, input) in crate::tests::fixtures::package_records().into_iter().enumerate() {
        let translated = crate::translate::parts::packages::packages(input.clone());
        assert_eq!(crate::tests::fixtures::packages_back(translated), input, "row {row}: every packages field");
    }
}

pub(super) fn part_back(value: crate::records::parts::packages::Packages) -> crate::render::parts::PartRecord {
    crate::render::parts::PartRecord::Packages(crate::tests::fixtures::packages_back(value))
}
