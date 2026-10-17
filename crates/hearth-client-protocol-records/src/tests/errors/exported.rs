use crate::{form::Form, tests::fixtures};
const EXPORTED: &[(Form, &[u8])] = &[
    (Form::Text, b"tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of ion-prod: not a string\n"),
    (Form::JsonLines, b"{\"record\":\"error\",\"kind\":\"exported\",\"exported\":{\"cause\":{\"schema\":{\"expected\":\"string\",\"kind\":\"wrong\"}},\"exporter\":\"ion-prod\",\"task\":\"vpn\"},\"class\":\"configuration\",\"at\":{\"column\":7,\"line\":22,\"path\":\"tasks/boot/task.json\"},\"target\":\"carbon-prod\",\"task\":\"vpn\"}\n"),
];
#[test]
fn an_exported_fault_names_the_exporter_in_text_and_json() {
    for (row, (form, expected)) in EXPORTED.iter().enumerate() {
        assert_eq!(fixtures::print_error(*form, fixtures::exported_error()).as_deref(), Ok(*expected), "row {row}");
    }
}
