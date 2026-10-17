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
