use crate::{
    errors::{Evaluation, Kind, Place, Position},
    form::{self, Form},
    limits,
    records::Record,
    tests::fixtures::{self, SchemaInput},
};
use skein_lib::bytes::copy_of;
const EVALUATIONS: &[(Evaluation, &str, &str)] = &[
    (
        Evaluation::Unknown,
        "an unknown name\n",
        concat!(r#"{"record":"error","kind":"evaluation","evaluation":"unknown","class":"configuration"}"#, "\n"),
    ),
    (
        Evaluation::NoKey,
        "a missing key\n",
        concat!(r#"{"record":"error","kind":"evaluation","evaluation":"no_key","class":"configuration"}"#, "\n"),
    ),
    (
        Evaluation::NoIndex,
        "an index outside the list\n",
        concat!(r#"{"record":"error","kind":"evaluation","evaluation":"no_index","class":"configuration"}"#, "\n"),
    ),
    (
        Evaluation::Undefined,
        "an undefined value used\n",
        concat!(r#"{"record":"error","kind":"evaluation","evaluation":"undefined","class":"configuration"}"#, "\n"),
    ),
    (
        Evaluation::Type,
        "a value of the wrong type\n",
        concat!(r#"{"record":"error","kind":"evaluation","evaluation":"type","class":"configuration"}"#, "\n"),
    ),
    (
        Evaluation::Overflow,
        "an integer overflow\n",
        concat!(r#"{"record":"error","kind":"evaluation","evaluation":"overflow","class":"configuration"}"#, "\n"),
    ),
    (
        Evaluation::Empty,
        "an empty list or pattern\n",
        concat!(r#"{"record":"error","kind":"evaluation","evaluation":"empty","class":"configuration"}"#, "\n"),
    ),
    (
        Evaluation::Path,
        "a path systemd_escape_path refuses\n",
        concat!(r#"{"record":"error","kind":"evaluation","evaluation":"path","class":"configuration"}"#, "\n"),
    ),
    (
        Evaluation::Output,
        "a list or a map output\n",
        concat!(r#"{"record":"error","kind":"evaluation","evaluation":"output","class":"configuration"}"#, "\n"),
    ),
    (
        Evaluation::Limit,
        "a limit exceeded\n",
        concat!(r#"{"record":"error","kind":"evaluation","evaluation":"limit","class":"configuration"}"#, "\n"),
    ),
];
#[test]
fn each_evaluation_kind_prints_its_message() {
    for (row, (kind, expected, _)) in EVALUATIONS.iter().enumerate() {
        let mut error = fixtures::duplicate();
        error.kind = Kind::Evaluation(*kind);
        error.at = None;
        error.also = None;
        let printed = form::print(Form::Text, &Record::Error(error), None, &limits::SHIPPED);
        assert_eq!(printed.as_deref(), Ok(expected.as_bytes()), "row {row}");
    }
}
#[test]
fn each_evaluation_kind_prints_its_json() {
    for (row, (kind, _, expected)) in EVALUATIONS.iter().enumerate() {
        let mut error = fixtures::duplicate();
        error.kind = Kind::Evaluation(*kind);
        error.at = None;
        error.also = None;
        let printed =
            form::print(Form::JsonLines, &Record::Error(error), Some(crate::records::Type::Error), &limits::SHIPPED);
        assert_eq!(printed.as_deref(), Ok(expected.as_bytes()), "row {row}");
    }
}
const SETTINGS: &[(SchemaInput, &str)] = &[
    (
        SchemaInput::Undeclared {
            setting: b"adress",
            declared: &[b"address", b"endpoint", b"hub", b"port", b"public_key"],
        },
        "hosts/ion/ion-prod.json:4:5: ion-prod: vpn: undeclared setting \"adress\"; vpn declares address, endpoint, hub, port, public_key\n",
    ),
    (
        SchemaInput::Undeclared { setting: b"greeting", declared: &[] },
        "hosts/ion/ion-prod.json:4:5: ion-prod: vpn: undeclared setting \"greeting\"; vpn declares no settings\n",
    ),
    (
        SchemaInput::Undeclared { setting: b"a\"\\\n\x1b", declared: &[] },
        "hosts/ion/ion-prod.json:4:5: ion-prod: vpn: undeclared setting \"a\\\"\\\\\\n\\x1b\"; vpn declares no settings\n",
    ),
    (
        SchemaInput::Missing(b"greeting"),
        "hosts/ion/ion-prod.json:4:5: ion-prod: vpn: missing required setting greeting\n",
    ),
    (SchemaInput::Absent(b"wifi"), "hosts/ion/ion-prod.json:4:5: ion-prod: vpn: reads wifi, which this target lacks\n"),
];
#[test]
fn target_schema_faults_print_their_setting_or_read() {
    for (row, &(input, expected)) in SETTINGS.iter().enumerate() {
        let mut error = fixtures::duplicate();
        error.kind = Kind::Schema(fixtures::schema_fault(input));
        error.also = None;
        error.at =
            Some(Place { path: copy_of(b"hosts/ion/ion-prod.json"), position: Some(Position { line: 4, column: 5 }) });
        error.target = Some(copy_of(b"ion-prod"));
        error.task = Some(copy_of(b"vpn"));
        let printed = form::print(Form::Text, &Record::Error(error), None, &limits::SHIPPED);
        assert_eq!(printed.as_deref(), Ok(expected.as_bytes()), "row {row}");
    }
}

const SETTINGS_JSON: &[(SchemaInput, &str)] = &[
    (
        SchemaInput::Undeclared { setting: b"greting", declared: &[b"greeting"] },
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"declared":["greeting"],"kind":"undeclared","setting":"greting"},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        SchemaInput::Missing(b"greeting"),
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"kind":"missing","setting":"greeting"},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        SchemaInput::Absent(b"wifi"),
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"kind":"absent","task":"wifi"},"class":"configuration"}"#,
            "\n"
        ),
    ),
];
#[test]
fn target_schema_faults_print_their_json_fields() {
    for (row, &(input, expected)) in SETTINGS_JSON.iter().enumerate() {
        let mut error = fixtures::duplicate();
        error.kind = Kind::Schema(fixtures::schema_fault(input));
        error.at = None;
        error.also = None;
        let printed =
            form::print(Form::JsonLines, &Record::Error(error), Some(crate::records::Type::Error), &limits::SHIPPED);
        assert_eq!(printed.as_deref(), Ok(expected.as_bytes()), "row {row}");
    }
}

#[test]
fn an_evaluation_error_prints_its_target_task_and_message() {
    let mut error = fixtures::duplicate();
    error.kind = Kind::Evaluation(Evaluation::NoKey);
    error.at = Some(Place {
        path: copy_of(b"tasks/motd/tree/etc/issue.j2"),
        position: Some(Position { line: 1, column: 11 }),
    });
    error.also = None;
    error.target = Some(copy_of(b"x1-prod"));
    error.task = Some(copy_of(b"motd"));
    assert_eq!(
        form::print(Form::Text, &Record::Error(error), None, &limits::SHIPPED).as_deref(),
        Ok(b"tasks/motd/tree/etc/issue.j2:1:11: x1-prod: motd: a missing key\n".as_slice())
    );
}
const REFERENCES: &[(SchemaInput, &str)] = &[
    (
        SchemaInput::Unlisted {
            reader: b"vpn",
            root: crate::errors::Root::Tasks,
            task: b"core",
            setting: Some(b"domain"),
        },
        "tasks.core.domain is not in vpn's reads\n",
    ),
    (
        SchemaInput::Unlisted { reader: b"mail", root: crate::errors::Root::Exports, task: b"mail", setting: None },
        "exports.mail is not in mail's reads\n",
    ),
    (
        SchemaInput::Unlisted { reader: b"vpn", root: crate::errors::Root::Tasks, task: b"wifi", setting: None },
        "tasks.wifi is not an optional read of vpn\n",
    ),
    (SchemaInput::Dynamic(crate::errors::Root::Tasks), "tasks takes only literal names\n"),
    (SchemaInput::Dynamic(crate::errors::Root::Exports), "exports takes only literal names\n"),
];
#[test]
fn an_unlisted_read_prints_the_reader_and_the_reference() {
    for (row, &(input, expected)) in REFERENCES.iter().enumerate() {
        assert_eq!(
            form::print(Form::Text, &Record::Error(fixtures::reference_error(input)), None, &limits::SHIPPED)
                .as_deref(),
            Ok(expected.as_bytes()),
            "row {row}"
        );
    }
}
const REFERENCES_JSON: &[(SchemaInput, &str)] = &[
    (
        SchemaInput::Unlisted {
            reader: b"vpn",
            root: crate::errors::Root::Tasks,
            task: b"core",
            setting: Some(b"domain"),
        },
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"kind":"unlisted","reader":"vpn","root":"tasks","setting":"domain","task":"core"},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        SchemaInput::Unlisted { reader: b"mail", root: crate::errors::Root::Exports, task: b"mail", setting: None },
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"kind":"unlisted","reader":"mail","root":"exports","setting":null,"task":"mail"},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        SchemaInput::Dynamic(crate::errors::Root::Tasks),
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"kind":"dynamic","root":"tasks"},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        SchemaInput::Dynamic(crate::errors::Root::Exports),
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"kind":"dynamic","root":"exports"},"class":"configuration"}"#,
            "\n"
        ),
    ),
];
#[test]
fn a_reference_fault_prints_its_json_fields() {
    for (row, &(input, expected)) in REFERENCES_JSON.iter().enumerate() {
        assert_eq!(
            form::print(
                Form::JsonLines,
                &Record::Error(fixtures::reference_error(input)),
                Some(crate::records::Type::Error),
                &limits::SHIPPED
            )
            .as_deref(),
            Ok(expected.as_bytes()),
            "row {row}"
        );
    }
}
