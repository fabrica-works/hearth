use crate::{records, render, tests::fixtures, translate::errors};
#[test]
fn a_guard_fault_translates_to_its_record_and_back() {
    for (row, input) in fixtures::guard_faults().into_iter().enumerate() {
        let mut error = fixtures::decode_error();
        error.kind = render::errors::Kind::Guard(input.clone());
        let records::errors::Kind::Guard(records::errors::Guard { name, message }) = errors::error(error).kind else {
            panic!("guard record")
        };
        let back = render::errors::Guard { name, message };
        assert_eq!(back, input, "row {row}: guard fault");
    }
}
