#[test]
fn each_resolution_change_translates_and_back() {
    for (row, input) in crate::tests::fixtures::resolution_changes().into_iter().enumerate() {
        let translated = crate::translate::changes::resolution(input.clone());
        assert_eq!(crate::tests::fixtures::resolution_change_back(translated), input, "row {row}: resolution change");
    }
}
