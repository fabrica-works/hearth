#[test]
fn each_boot_field_translates_to_the_records_and_back() {
    for (row, input) in crate::tests::fixtures::boot_records().into_iter().enumerate() {
        let translated = crate::translate::parts::boot::boot(input.clone());
        assert_eq!(back(translated), input, "row {row}: every boot field");
    }
}
fn back(boot: crate::records::parts::boot::Boot) -> crate::render::parts::boot::Boot {
    let crate::records::parts::boot::Boot { params, mutable } = boot;
    crate::render::parts::boot::Boot { params, mutable }
}
pub(super) fn part_back(value: crate::records::parts::boot::Boot) -> crate::render::parts::PartRecord {
    crate::render::parts::PartRecord::Boot(back(value))
}
