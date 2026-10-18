use crate::tests::fixtures::{self, ChangedPart};
const JSON: &[(ChangedPart, &str)] = &[
    (
        ChangedPart::Secrets,
        concat!(
            r#"{"record":"change","target":"ion-prod","kind":"changed","changed":{"parts":[{"kind":"secrets","secrets":{"files":[{"key":"new","kind":"added"},{"key":"gone","kind":"removed"},{"key":"key","kind":"changed"}],"passwords":[{"key":"free","kind":"changed"}],"identity":[{"key":"ssh","kind":"changed"}]}}],"rebuild":{"kind":"nothing"}}}"#,
            "\n"
        ),
    ),
    (
        ChangedPart::Software,
        concat!(
            r#"{"record":"change","target":"ion-prod","kind":"changed","changed":{"parts":[{"kind":"software","software":{"downloads":[{"after":{"group":"root","owner":"root","sha256":"0202020202020202020202020202020202020202020202020202020202020202","strip":1,"unpack":"tar"},"before":{"group":"root","owner":"root","sha256":"0101010101010101010101010101010101010101010101010101010101010101","strip":1,"unpack":"tar"},"kind":"changed","key":"/opt/tool"}]}}],"rebuild":{"kind":"nothing"}}}"#,
            "\n"
        ),
    ),
    (
        ChangedPart::Live,
        concat!(
            r#"{"record":"change","target":"ion-prod","kind":"changed","changed":{"parts":[{"kind":"live","live":{"paths":[{"after":[{"action":"none","task":"mail"}],"before":[],"kind":"changed","key":"/etc/mailname"}]}}],"rebuild":{"kind":"nothing"}}}"#,
            "\n"
        ),
    ),
    (
        ChangedPart::Seeds,
        concat!(
            r#"{"record":"change","target":"ion-prod","kind":"changed","changed":{"parts":[{"kind":"packages","packages":{"seeds":{"groups":[{"after":122,"before":120,"kind":"changed","key":"mail"}],"users":[{"after":{"gid":122,"uid":121},"before":{"gid":null,"uid":120},"kind":"changed","key":"mail"}]},"resolution":{"kind":"same"}}}],"rebuild":{"kind":"nothing"}}}"#,
            "\n"
        ),
    ),
];
#[test]
fn each_secrets_software_live_and_seeds_change_has_its_json_record() {
    for (part, expected) in JSON {
        assert_eq!(fixtures::diff_json(&fixtures::part_change(*part)).as_deref(), Ok(expected.as_bytes()), "{part:?}");
    }
}
const TEXT: &[(ChangedPart, &str)] = &[
    (
        ChangedPart::Secrets,
        concat!(
            "ion-prod                                            nothing to build\n",
            "  secrets  + secret new\n  secrets  - secret gone\n  secrets  ~ secret key\n  secrets  ~ password free\n  secrets  ~ identity ssh\n"
        ),
    ),
    (
        ChangedPart::Seeds,
        concat!(
            "ion-prod                                            nothing to build\n",
            "  packages   resolution same\n",
            "  packages ~ seed user mail                         121 122\n",
            "  packages ~ seed group mail                        122\n"
        ),
    ),
];
#[test]
fn changed_names_and_seed_rows_have_their_text_forms() {
    for (part, expected) in TEXT {
        assert_eq!(fixtures::diff_text(&fixtures::part_change(*part)).as_deref(), Ok(expected.as_bytes()), "{part:?}");
    }
}
