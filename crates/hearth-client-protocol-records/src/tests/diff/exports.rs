use crate::{form::Form, tests::fixtures};
type Row = (Option<&'static [u8]>, Option<&'static [u8]>, &'static str);
const TEXT: &[Row] = &[
    (
        Some(b"194.164.162.157"),
        Some(b"194.164.162.158"),
        concat!(
            "ion-prod                                            nothing to build\n",
            "  exports  ~ vpn.endpoint   \"194.164.162.157\" → \"194.164.162.158\"\n"
        ),
    ),
    (
        None,
        Some(b"a"),
        concat!(
            "ion-prod                                            nothing to build\n",
            "  exports  + vpn.endpoint   \"a\"\n"
        ),
    ),
    (
        Some(b"a"),
        None,
        concat!(
            "ion-prod                                            nothing to build\n",
            "  exports  - vpn.endpoint   \"a\"\n"
        ),
    ),
];
#[test]
fn exports_text_names_each_side_with_its_change_mark() {
    for (row, &(before, after, expected)) in TEXT.iter().enumerate() {
        assert_eq!(
            fixtures::printed_export_change(Form::Text, before, after).as_ref(),
            expected.as_bytes(),
            "row {row}"
        );
    }
}
const JSON: &[Row] = &[
    (
        Some(b"a"),
        Some(b"b"),
        concat!(
            r#"{"record":"change","target":"ion-prod","kind":"changed","changed":{"parts":[{"kind":"exports","exports":[{"after":"b","before":"a","name":"endpoint","task":"vpn"}]}],"rebuild":{"kind":"nothing"}}}"#,
            "\n"
        ),
    ),
    (
        None,
        Some(b"a"),
        concat!(
            r#"{"record":"change","target":"ion-prod","kind":"changed","changed":{"parts":[{"kind":"exports","exports":[{"after":"a","name":"endpoint","task":"vpn"}]}],"rebuild":{"kind":"nothing"}}}"#,
            "\n"
        ),
    ),
    (
        Some(b"a"),
        None,
        concat!(
            r#"{"record":"change","target":"ion-prod","kind":"changed","changed":{"parts":[{"kind":"exports","exports":[{"before":"a","name":"endpoint","task":"vpn"}]}],"rebuild":{"kind":"nothing"}}}"#,
            "\n"
        ),
    ),
];
#[test]
fn exports_json_keeps_present_sides_and_leaves_absent_sides_out() {
    for (row, &(before, after, expected)) in JSON.iter().enumerate() {
        assert_eq!(
            fixtures::printed_export_change(Form::JsonLines, before, after).as_ref(),
            expected.as_bytes(),
            "row {row}"
        );
    }
}
