use crate::{
    form::{self, Form},
    limits,
    records::Type,
    tests::fixtures,
};
#[test]
fn packages_json_lists_added_and_removed_names() {
    let record = fixtures::change_record(fixtures::requested_packages_change());
    assert_eq!(form::print(Form::JsonLines, &record, Some(Type::Change), &limits::SHIPPED).as_deref(), Ok(concat!(
        r#"{"record":"change","target":"ion-prod","kind":"changed","changed":{"parts":[{"kind":"packages","packages":{"request":{"added":["vim"],"removed":["nano"]}}}],"rebuild":{"kind":"nothing"}}}"#, "\n").as_bytes()));
}
#[test]
fn an_equal_request_is_left_out_of_the_json() {
    let record = fixtures::change_record(fixtures::unchanged_packages_change());
    assert_eq!(form::print(Form::JsonLines, &record, Some(Type::Change), &limits::SHIPPED).as_deref(), Ok(concat!(
        r#"{"record":"change","target":"ion-prod","kind":"changed","changed":{"parts":[{"kind":"packages","packages":{}}],"rebuild":{"kind":"nothing"}}}"#, "\n").as_bytes()));
}
#[test]
fn boot_json_keeps_the_before_and_after() {
    let record = fixtures::change_record(fixtures::boot_change());
    assert_eq!(form::print(Form::JsonLines, &record, Some(Type::Change), &limits::SHIPPED).as_deref(), Ok(concat!(
        r#"{"record":"change","target":"ion-prod","kind":"changed","changed":{"parts":[{"kind":"boot","boot":{"after":{"mutable":true,"params":[]},"before":{"mutable":false,"params":["quiet"]}}}],"rebuild":{"kind":"nothing"}}}"#, "\n").as_bytes()));
}
#[test]
fn boot_text_keeps_the_before_and_after() {
    let record = fixtures::change_record(fixtures::boot_change());
    assert_eq!(
        form::print(Form::Text, &record, Some(Type::Change), &limits::SHIPPED).as_deref(),
        Ok(concat!(
            "ion-prod                                            nothing to build\n",
            "  boot     ~ {\"mutable\":false,\"params\":[\"quiet\"]} → {\"mutable\":true,\"params\":[]}\n"
        )
        .as_bytes())
    );
}
#[test]
fn packages_text_lists_added_and_removed_names() {
    let record = fixtures::change_record(fixtures::requested_packages_change());
    assert_eq!(
        form::print(Form::Text, &record, Some(Type::Change), &limits::SHIPPED).as_deref(),
        Ok(concat!(
            "ion-prod                                            nothing to build\n",
            "  packages + vim\n",
            "  packages - nano\n"
        )
        .as_bytes())
    );
}

#[test]
fn a_preseed_only_change_prints_preseeds() {
    let record = fixtures::change_record(fixtures::preseed_change());
    assert_eq!(form::print(Form::Text, &record, Some(Type::Change), &limits::SHIPPED).as_deref(), Ok(concat!(
        "ion-prod                                            nothing to build\n",
        r#"  packages ~ preseeds {} → {"a/answer":{"owner":"a","type":"string","value":"word"},"tzdata/Areas":{"owner":"other","type":"select","value":"Europe"}}"#, "\n").as_bytes()));
}
#[test]
fn an_initramfs_only_change_prints_initramfs() {
    let record = fixtures::change_record(fixtures::initramfs_change());
    assert_eq!(
        form::print(Form::Text, &record, Some(Type::Change), &limits::SHIPPED).as_deref(),
        Ok(concat!(
            "ion-prod                                            nothing to build\n",
            r#"  packages ~ initramfs null → {"microcode":"generic","modules":["nvme"]}"#,
            "\n"
        )
        .as_bytes())
    );
}
