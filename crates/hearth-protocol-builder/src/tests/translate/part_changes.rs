use crate::{render::diff::PartChange, tests::fixtures, translate::changes};
#[test]
fn each_secrets_software_and_live_change_translates_back() {
    for (row, input) in fixtures::part_changes().into_iter().enumerate() {
        let back = match changes::part_change(input.clone()) {
            crate::records::diff::PartChange::Secrets(value) => {
                PartChange::Secrets(fixtures::secrets_change_back(value))
            }
            crate::records::diff::PartChange::Software(value) => {
                PartChange::Software(fixtures::software_change_back(value))
            }
            crate::records::diff::PartChange::Live(value) => PartChange::Live(fixtures::live_change_back(value)),
            crate::records::diff::PartChange::Config(_)
            | crate::records::diff::PartChange::Data(_)
            | crate::records::diff::PartChange::Meta(_)
            | crate::records::diff::PartChange::Packages(_)
            | crate::records::diff::PartChange::Boot(_)
            | crate::records::diff::PartChange::Exports(_) => panic!("part change"),
        };
        assert_eq!(back, input, "row {row}: all change fields survive");
    }
}
