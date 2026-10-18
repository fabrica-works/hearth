use crate::{form::Form, tests::fixtures};
const PRINTS: &[(Form, &str)] = &[
    (
        Form::Text,
        "tasks/banner/task.json:1:12: x1-prod: /etc/motd is given by banner and by motd (tasks/motd/tree/etc/motd)\n",
    ),
    (
        Form::JsonLines,
        concat!(
            r#"{"record":"error","kind":"collision","collision":{"kind":"given","subject":"tree","name":"/etc/motd","first":"banner","second":"motd"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
            "\n"
        ),
    ),
];
#[test]
fn a_collision_prints_both_tasks_and_both_places() {
    for (form, expected) in PRINTS {
        assert_eq!(
            fixtures::print_error(*form, fixtures::path_collision()).as_deref(),
            Ok(expected.as_bytes()),
            "{form:?}"
        );
    }
}
const SHARED: &[(Form, &str)] = &[
    (
        Form::Text,
        "tasks/banner/task.json:1:12: x1-prod: uid 1000 is shared by free and ci (tasks/motd/tree/etc/motd)\n",
    ),
    (
        Form::JsonLines,
        concat!(
            r#"{"record":"error","kind":"collision","collision":{"kind":"shared","id":"uid","value":1000,"first":"free","second":"ci"},"class":"configuration","at":{"column":12,"line":1,"path":"tasks/banner/task.json"},"also":{"path":"tasks/motd/tree/etc/motd"},"target":"x1-prod"}"#,
            "\n"
        ),
    ),
];
#[test]
fn a_shared_uid_prints_both_users() {
    for (form, expected) in SHARED {
        assert_eq!(
            fixtures::print_error(*form, fixtures::shared_uid_collision()).as_deref(),
            Ok(expected.as_bytes()),
            "{form:?}"
        );
    }
}
#[test]
fn overlapping_ranges_name_both_accounts() {
    assert_eq!(
        fixtures::print_error(Form::Text, fixtures::range_collision()).as_deref(),
        Ok(b"tasks/banner/task.json:1:12: x1-prod: subordinate ids of free overlap ci's (tasks/motd/tree/etc/motd)\n"
            .as_slice())
    );
}
