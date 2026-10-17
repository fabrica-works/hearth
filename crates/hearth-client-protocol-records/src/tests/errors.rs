use super::fixtures;
use crate::{
    errors::{Place, Position},
    form::{self, Form},
    limits,
    outcome::Outcome,
    records::{Record, Terminal, Type},
    startup::{Misuse, Opened, Startup},
};
use skein_lib::bytes::copy_of;
const GUARD: &str =
    "\ntasks/boot/task.json:22:7: carbon-lab: boot: requires modules: no initramfs modules for this hardware\n";
#[test]
fn a_false_guard_prints_as_render_md_shows() {
    let printed =
        form::print(Form::Text, &Record::Error(fixtures::modules_guard()), Some(Type::Spec), &limits::SHIPPED);
    assert_eq!(printed.as_deref(), Ok(GUARD.as_bytes()), "render.md, 7.3, after a spec's blank line");
}
const WITHOUT_TARGET: &str =
    "tasks/boot/task.json:22:7: boot: requires modules: no initramfs modules for this hardware\n";
#[test]
fn an_error_without_a_target_leaves_out_its_prefix() {
    let mut error = fixtures::modules_guard();
    error.target = None;
    assert_eq!(print(&Record::Error(error.clone())).as_deref(), Ok(WITHOUT_TARGET.as_bytes()));
}
const ALSO: [(Option<Position>, &str); 2] = [
    (
        Some(Position { line: 9, column: 3 }),
        "tasks/boot/task.json:22:7: carbon-lab: boot: requires modules: no initramfs modules for this hardware (tasks/other/task.json:9:3)\n",
    ),
    (
        None,
        "tasks/boot/task.json:22:7: carbon-lab: boot: requires modules: no initramfs modules for this hardware (tasks/other/task.json)\n",
    ),
];
#[test]
fn a_second_place_prints_in_parentheses_last() {
    for (position, expected) in ALSO {
        let mut error = fixtures::modules_guard();
        error.also = Some(Place { path: copy_of(b"tasks/other/task.json"), position });
        assert_eq!(print(&Record::Error(error)).as_deref(), Ok(expected.as_bytes()), "{position:?}");
    }
}
const PATH_ONLY: &str =
    "tasks/boot/task.json: carbon-lab: boot: requires modules: no initramfs modules for this hardware\n";
#[test]
fn a_place_without_a_position_prints_its_path_alone() {
    let mut error = fixtures::modules_guard();
    error.at.as_mut().expect("fixture place").position = None;
    assert_eq!(print(&Record::Error(error.clone())).as_deref(), Ok(PATH_ONLY.as_bytes()));
}
const LEFT_OUT: &str = "44 more errors\n";
#[test]
fn the_errors_left_out_print_their_count_after_the_last() {
    assert_eq!(print(&terminal(44)).as_deref(), Ok(LEFT_OUT.as_bytes()));
}
const ONE_LEFT_OUT: &str = "1 more error\n";
#[test]
fn one_error_left_out_prints_1_more_error() {
    assert_eq!(print(&terminal(1)).as_deref(), Ok(ONE_LEFT_OUT.as_bytes()));
}
const JSON_GUARD: &str = concat!(
    r#"{"record":"error","kind":"guard","guard":{"message":"no initramfs modules for this hardware","name":"modules"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab","task":"boot"}"#,
    "\n",
);
#[test]
fn json_gives_an_error_with_its_class_and_place() {
    let printed =
        form::print(Form::JsonLines, &Record::Error(fixtures::modules_guard()), Some(Type::Error), &limits::SHIPPED);
    assert_eq!(printed.as_deref(), Ok(JSON_GUARD.as_bytes()));
}
const OPTION_REFUSAL: [(Form, &str); 2] = [
    (Form::Text, "hearth: unknown option: --bogus\n"),
    (
        Form::JsonLines,
        concat!(
            r#"{"record":"version","client":1}"#,
            "\n",
            r#"{"record":"startup","kind":"usage","outcome":"usage","usage":{"misuse":"option","word":"--bogus"}}"#,
            "\n"
        ),
    ),
];
#[test]
fn a_startup_refusal_prints_its_line_or_its_record_after_the_version() {
    for (form, expected) in OPTION_REFUSAL {
        let record = Record::Startup(Startup::Usage { misuse: Misuse::Option, word: copy_of(b"--bogus") });
        assert_eq!(form::print(form, &record, None, &limits::SHIPPED).as_deref(), Ok(expected.as_bytes()), "{form:?}");
    }
}
const UNOPENED: [(Opened, Option<&[u8]>, &str); 4] = [
    (Opened::Store, Some(b"/store"), "hearth: cannot open the store /store: permission denied\n"),
    (Opened::Repository, Some(b"/repo"), "hearth: cannot open the repository /repo: permission denied\n"),
    (Opened::Store, None, "hearth: cannot open the store: permission denied\n"),
    (Opened::Repository, None, "hearth: cannot open the repository: permission denied\n"),
];
#[test]
fn a_startup_refusal_for_the_store_or_repository_prints_its_line() {
    for (what, path, expected) in UNOPENED {
        let startup = match path {
            Some(path) => Startup::Unopened { what, path: Some(copy_of(path)), error: copy_of(b"permission denied") },
            None => Startup::Unopened { what, path: None, error: copy_of(b"permission denied") },
        };
        let record = Record::Startup(startup);
        assert_eq!(print(&record).as_deref(), Ok(expected.as_bytes()), "{what:?} {path:?}");
    }
}
const MISUSES: [(Misuse, &[u8], &str); 3] = [
    (Misuse::Command, b"bogus", "hearth: unknown command: bogus\n"),
    (Misuse::Value, b"--store", "hearth: option needs a value: --store\n"),
    (Misuse::Revision, b"bad..rev", "hearth: malformed revision: bad..rev\n"),
];
#[test]
fn a_startup_refusal_for_a_command_value_or_revision_prints_its_line() {
    for (misuse, word, expected) in MISUSES {
        assert_eq!(
            print(&Record::Startup(Startup::Usage { misuse, word: copy_of(word) })).as_deref(),
            Ok(expected.as_bytes()),
            "{misuse:?}"
        );
    }
}
const DIFF_LEFT_OUT: &str = "44 more errors\n\n0 of 1 target change; 0 are unchanged.\n";
#[test]
fn omitted_errors_precede_the_diff_tally() {
    let record =
        Record::Terminal(Terminal { outcome: Outcome::Failed, diff: Some(fixtures::tally(1, 0, 0)), left_out: 44 });
    assert_eq!(print(&record).as_deref(), Ok(DIFF_LEFT_OUT.as_bytes()));
}
const JSON_LEFT_OUT: &str = concat!(r#"{"record":"outcome","outcome":"failed","left_out":44}"#, "\n");
#[test]
fn json_reports_the_omitted_error_count_last() {
    assert_eq!(
        form::print(Form::JsonLines, &terminal(44), Some(Type::Error), &limits::SHIPPED).as_deref(),
        Ok(JSON_LEFT_OUT.as_bytes())
    );
}
const STARTUP_OUTCOMES: [(Opened, Outcome); 2] =
    [(Opened::Store, Outcome::TryAgain), (Opened::Repository, Outcome::TryAgain)];
#[test]
fn an_unopened_resource_gives_try_again() {
    for (what, expected) in STARTUP_OUTCOMES {
        let startup = Startup::Unopened { what, path: None, error: copy_of(b"denied") };
        assert_eq!(startup.outcome(), expected, "{what:?}");
    }
}
const USAGE_OUTCOMES: [(Misuse, Outcome); 4] = [
    (Misuse::Command, Outcome::Usage),
    (Misuse::Option, Outcome::Usage),
    (Misuse::Value, Outcome::Usage),
    (Misuse::Revision, Outcome::Usage),
];
#[test]
fn every_usage_refusal_gives_usage() {
    for (misuse, expected) in USAGE_OUTCOMES {
        assert_eq!(Startup::Usage { misuse, word: copy_of(b"bogus") }.outcome(), expected, "{misuse:?}");
    }
}
fn print(record: &Record) -> Result<Box<[u8]>, form::Refusal> {
    form::print(Form::Text, record, Some(Type::Error), &limits::SHIPPED)
}
fn terminal(left_out: u32) -> Record {
    Record::Terminal(Terminal { outcome: Outcome::Failed, diff: None, left_out })
}

#[test]
fn a_decode_error_prints_its_place_message_and_also() {
    assert_eq!(
        print(&Record::Error(fixtures::duplicate())).as_deref(),
        Ok(b"envs/lab.json:1:14: duplicate key \"motd\" (envs/lab.json:1:2)\n".as_slice())
    );
}
#[test]
fn a_decode_error_prints_its_json_fields_and_places() {
    assert_eq!(form::print(Form::JsonLines, &Record::Error(fixtures::duplicate()), Some(Type::Error), &limits::SHIPPED).as_deref(),
        Ok(concat!(r#"{"record":"error","kind":"decode","decode":{"key":"motd","kind":"duplicate"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"},"also":{"column":2,"line":1,"path":"envs/lab.json"}}"#, "\n").as_bytes()));
}

const DECODE_LINES: &[(crate::errors::Decode, &str)] = &[
    (crate::errors::Decode::Unexpected, "envs/lab.json:1:14: not JSON: unexpected byte\n"),
    (
        crate::errors::Decode::TooLarge { limit: crate::errors::Limit::Document, max: 1_048_576 },
        "envs/lab.json:1:14: a document past 1048576 bytes\n",
    ),
    (
        crate::errors::Decode::TooLarge { limit: crate::errors::Limit::Decoded, max: 64 },
        "envs/lab.json:1:14: decoded documents past 64 bytes\n",
    ),
    (
        crate::errors::Decode::TooLarge { limit: crate::errors::Limit::Static, max: 8 },
        "envs/lab.json:1:14: a static file past 8 bytes\n",
    ),
];
#[test]
fn decode_faults_print_their_messages_and_bounds() {
    for (decode, expected) in DECODE_LINES {
        let mut error = fixtures::duplicate();
        error.kind = crate::errors::Kind::Decode(decode.clone());
        error.also = None;
        assert_eq!(print(&Record::Error(error)).as_deref(), Ok(expected.as_bytes()));
    }
}
#[test]
fn a_decode_key_prints_quotes_and_controls_on_one_line() {
    let mut error = fixtures::duplicate();
    error.kind = crate::errors::Kind::Decode(crate::errors::Decode::Duplicate { key: copy_of(b"a\n\"\\\x01") });
    error.also = None;
    assert_eq!(
        print(&Record::Error(error)).as_deref(),
        Ok(b"envs/lab.json:1:14: duplicate key \"a\\n\\\"\\\\\\x01\"\n".as_slice())
    );
}

const DECLARATION_LINES: &[(crate::errors::Declared, &str)] = &[
    (
        crate::errors::Declared::Unknown { of: crate::errors::Object::Declaration },
        "tasks/motd/task.json:1:2: motd: no such key in a declaration\n",
    ),
    (
        crate::errors::Declared::Unknown { of: crate::errors::Object::Entry(crate::errors::Map::Files) },
        "tasks/motd/task.json:1:2: motd: no such key in a files entry\n",
    ),
    (
        crate::errors::Declared::Wrong { expected: crate::errors::Expected::String },
        "tasks/motd/task.json:1:2: motd: not a string\n",
    ),
    (crate::errors::Declared::Optional, "tasks/motd/task.json:1:2: motd: optional beside a default\n"),
];
#[test]
fn a_declaration_fault_prints_its_object_after_its_place() {
    for (row, (declared, line)) in DECLARATION_LINES.iter().enumerate() {
        let record = Record::Error(crate::errors::Error {
            kind: crate::errors::Kind::Decode(crate::errors::Decode::Declaration(*declared)),
            class: crate::errors::Class::Configuration,
            at: Some(Place { path: copy_of(b"tasks/motd/task.json"), position: Some(Position { line: 1, column: 2 }) }),
            also: None,
            target: None,
            task: Some(copy_of(b"motd")),
        });
        assert_eq!(print(&record).as_deref(), Ok(line.as_bytes()), "row {row}");
    }
}
#[test]
fn a_declaration_fault_prints_its_json_fields() {
    let record = Record::Error(crate::errors::Error {
        kind: crate::errors::Kind::Decode(crate::errors::Decode::Declaration(crate::errors::Declared::Unknown {
            of: crate::errors::Object::Declaration,
        })),
        class: crate::errors::Class::Configuration,
        at: None,
        also: None,
        target: None,
        task: None,
    });
    assert_eq!(form::print(Form::JsonLines, &record, Some(Type::Error), &limits::SHIPPED).as_deref(), Ok(b"{\"record\":\"error\",\"kind\":\"decode\",\"decode\":{\"declared\":{\"kind\":\"unknown\",\"of\":\"declaration\"},\"kind\":\"declaration\"},\"class\":\"configuration\"}\n".as_slice()));
}

const SCHEMA_LINES: &[(fixtures::SchemaInput, &str)] = &[
    (fixtures::SchemaInput::Task(b"mail"), "envs/lab.json:1:14: reads mail, which tasks/ lacks\n"),
    (
        fixtures::SchemaInput::Unread { task: b"core", setting: b"domian", declared: &[b"domain"] },
        "envs/lab.json:1:14: reads core.domian, which core does not declare; core declares domain\n",
    ),
    (
        fixtures::SchemaInput::Unread { task: b"core", setting: b"domian", declared: &[] },
        "envs/lab.json:1:14: reads core.domian, which core does not declare; core declares no settings\n",
    ),
];
#[test]
fn each_schema_fault_prints_its_message() {
    for (row, &(input, line)) in SCHEMA_LINES.iter().enumerate() {
        let mut error = fixtures::duplicate();
        error.kind = crate::errors::Kind::Schema(fixtures::schema_fault(input));
        error.also = None;
        assert_eq!(print(&Record::Error(error)).as_deref(), Ok(line.as_bytes()), "row {row}");
    }
}
const SCHEMA_JSON: &[(fixtures::SchemaInput, &str)] = &[
    (
        fixtures::SchemaInput::Task(b"mail"),
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"kind":"task","task":"mail"},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        fixtures::SchemaInput::Unread { task: b"core", setting: b"domian", declared: &[b"domain"] },
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"declared":["domain"],"kind":"unread","setting":"domian","task":"core"},"class":"configuration"}"#,
            "\n"
        ),
    ),
];
#[test]
fn each_schema_fault_prints_its_json_fields() {
    for (row, &(input, line)) in SCHEMA_JSON.iter().enumerate() {
        let record = Record::Error(crate::errors::Error {
            kind: crate::errors::Kind::Schema(fixtures::schema_fault(input)),
            class: crate::errors::Class::Configuration,
            at: None,
            also: None,
            target: None,
            task: None,
        });
        assert_eq!(
            form::print(Form::JsonLines, &record, Some(Type::Error), &limits::SHIPPED).as_deref(),
            Ok(line.as_bytes()),
            "row {row}"
        );
    }
}
