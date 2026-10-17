use crate::{errors::Repository, form::Form, tests::fixtures};
const TEMPLATES: &[(Repository, &str, &str)] = &[
    (
        Repository::NoScript,
        "no script at this path\n",
        "{\"record\":\"error\",\"kind\":\"repository\",\"repository\":\"no_script\",\"class\":\"configuration\"}\n",
    ),
    (
        Repository::NoTemplate,
        "no template at this path\n",
        "{\"record\":\"error\",\"kind\":\"repository\",\"repository\":\"no_template\",\"class\":\"configuration\"}\n",
    ),
    (
        Repository::Unused,
        "a template no entry names\n",
        "{\"record\":\"error\",\"kind\":\"repository\",\"repository\":\"unused\",\"class\":\"configuration\"}\n",
    ),
];
#[test]
fn template_repository_faults_print_their_messages() {
    for (row, (why, expected, _)) in TEMPLATES.iter().enumerate() {
        assert_eq!(
            fixtures::print_error(Form::Text, fixtures::repository_error(why.clone())).as_deref(),
            Ok(expected.as_bytes()),
            "row {row}"
        );
    }
}
#[test]
fn template_repository_faults_print_their_json() {
    for (row, (why, _, expected)) in TEMPLATES.iter().enumerate() {
        assert_eq!(
            fixtures::print_error(Form::JsonLines, fixtures::repository_error(why.clone())).as_deref(),
            Ok(expected.as_bytes()),
            "row {row}"
        );
    }
}
