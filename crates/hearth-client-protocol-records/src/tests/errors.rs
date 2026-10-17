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
            side: None,
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
        side: None,
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
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::Bool), "envs/lab.json:1:14: not a bool\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::Int), "envs/lab.json:1:14: not an int\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::String), "envs/lab.json:1:14: not a string\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::Enum), "envs/lab.json:1:14: not an enum\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::List), "envs/lab.json:1:14: not a list\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::Map), "envs/lab.json:1:14: not a map\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::Object), "envs/lab.json:1:14: not an object\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::Path), "envs/lab.json:1:14: not a path\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::Mode), "envs/lab.json:1:14: not a mode\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::Duration), "envs/lab.json:1:14: not a duration\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::Secret), "envs/lab.json:1:14: not a secret\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::Target), "envs/lab.json:1:14: not a target\n"),
    (fixtures::SchemaInput::Wrong(&crate::errors::TypeName::File), "envs/lab.json:1:14: not a file\n"),
    (
        fixtures::SchemaInput::Enum { value: b"x", listed: &[b"main", b"contrib"] },
        "envs/lab.json:1:14: \"x\" is not one of main, contrib\n",
    ),
    (fixtures::SchemaInput::Below(-1000), "envs/lab.json:1:14: below its min -1000\n"),
    (fixtures::SchemaInput::Above(59999), "envs/lab.json:1:14: past its max 59999\n"),
    (fixtures::SchemaInput::Length(8), "envs/lab.json:1:14: past its max_length 8\n"),
    (fixtures::SchemaInput::Null, "envs/lab.json:1:14: null in a default\n"),
    (
        fixtures::SchemaInput::UndeclaredField {
            field: b"shel",
            declared: &[b"groups", b"password", b"shell", b"uid"],
        },
        "envs/lab.json:1:14: undeclared field \"shel\"; the object declares groups, password, shell, uid\n",
    ),
    (fixtures::SchemaInput::MissingField(b"uid"), "envs/lab.json:1:14: missing required field uid\n"),
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
        fixtures::SchemaInput::Wrong(&crate::errors::TypeName::Int),
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"expected":"int","kind":"wrong"},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        fixtures::SchemaInput::Enum { value: b"x", listed: &[b"main", b"contrib"] },
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"kind":"enum","listed":["main","contrib"],"value":"x"},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        fixtures::SchemaInput::Below(-1000),
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"kind":"below","min":-1000},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        fixtures::SchemaInput::Above(59999),
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"kind":"above","max":59999},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        fixtures::SchemaInput::Length(8),
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"kind":"length","max_length":8},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        fixtures::SchemaInput::Null,
        concat!(r#"{"record":"error","kind":"schema","schema":{"kind":"null"},"class":"configuration"}"#, "\n"),
    ),
    (
        fixtures::SchemaInput::UndeclaredField { field: b"shel", declared: &[b"shell", b"uid"] },
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"declared":["shell","uid"],"field":"shel","kind":"undeclared_field"},"class":"configuration"}"#,
            "\n"
        ),
    ),
    (
        fixtures::SchemaInput::MissingField(b"uid"),
        concat!(
            r#"{"record":"error","kind":"schema","schema":{"field":"uid","kind":"missing_field"},"class":"configuration"}"#,
            "\n"
        ),
    ),
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
            side: None,
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

const TOOL_LINES: &[(crate::errors::Command, u8, &[u8], &[u8])] = &[
    (crate::errors::Command::LsTree, 128,
     b"git ls-tree exited 128\n  fatal: not a tree object\n",
     b"{\"record\":\"error\",\"kind\":\"tool\",\"tool\":{\"command\":\"ls_tree\",\"ended\":{\"code\":128,\"kind\":\"exit\"},\"tail\":[\"fatal: not a tree object\"]},\"class\":\"operational\"}\n"),
    (crate::errors::Command::Diff, 1,
     b"git diff exited 1\n  fatal: not a tree object\n",
     b"{\"record\":\"error\",\"kind\":\"tool\",\"tool\":{\"command\":\"diff\",\"ended\":{\"code\":1,\"kind\":\"exit\"},\"tail\":[\"fatal: not a tree object\"]},\"class\":\"operational\"}\n"),
];
#[test]
fn a_tool_error_prints_its_tail_below_it() {
    for (row, &(command, code, text, _)) in TOOL_LINES.iter().enumerate() {
        assert_eq!(print(&Record::Error(fixtures::git_failure(command, code))).as_deref(), Ok(text), "row {row}");
    }
}
#[test]
fn a_tool_error_prints_its_command_end_and_tail_as_json() {
    for (row, &(command, code, _, json)) in TOOL_LINES.iter().enumerate() {
        assert_eq!(
            form::print(
                Form::JsonLines,
                &Record::Error(fixtures::git_failure(command, code)),
                Some(Type::Error),
                &limits::SHIPPED
            )
            .as_deref(),
            Ok(json),
            "row {row}"
        );
    }
}

const SYNTAX_MESSAGES: &[&[u8]] = &[
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: a template past its length limit\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: an unexpected character\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: a string with no end\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: an invalid escape\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: an integer outside i64\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: too deep\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: a misplaced expression\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: an unknown filter\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: an unknown test\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: not UTF-8\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: a carriage return\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: a tag with no end\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: an unknown statement\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: a close with no open block\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: a block left open\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: a root of the context bound\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: a statement in an interpolated string\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: expected operand\n",
    b"tasks/motd/tree/etc/issue.j2:2:1: motd: \"join\" takes 1 argument, given 2\n",
];
#[test]
fn each_syntax_fault_prints_its_message() {
    for (row, (syntax, expected)) in fixtures::syntax_faults().into_iter().zip(SYNTAX_MESSAGES).enumerate() {
        assert_eq!(print(&Record::Error(fixtures::syntax_error(syntax))).as_deref(), Ok(*expected), "row {row}");
    }
}
#[test]
fn a_syntax_error_prints_its_path_task_and_message() {
    assert_eq!(
        print(&Record::Error(fixtures::syntax_error(crate::errors::Syntax::Unclosed))).as_deref(),
        Ok(b"tasks/motd/tree/etc/issue.j2:2:1: motd: a tag with no end\n".as_slice())
    );
}
#[test]
fn a_syntax_error_prints_its_json_kind_and_fault() {
    assert_eq!(form::print(Form::JsonLines, &Record::Error(fixtures::syntax_error(crate::errors::Syntax::Unclosed)), Some(Type::Spec), &limits::SHIPPED).as_deref(), Ok(b"{\"record\":\"error\",\"kind\":\"decode\",\"decode\":{\"kind\":\"syntax\",\"fault\":\"unclosed\"},\"class\":\"configuration\",\"at\":{\"column\":1,\"line\":2,\"path\":\"tasks/motd/tree/etc/issue.j2\"},\"task\":\"motd\"}\n".as_slice()));
}
const DIFF_ERROR_TEXT: &[(crate::diff::Side, &str)] = &[
    (crate::diff::Side::From, "from: envs/lab.json:1:14: duplicate key \"motd\" (envs/lab.json:1:2)\n"),
    (crate::diff::Side::To, "to: envs/lab.json:1:14: duplicate key \"motd\" (envs/lab.json:1:2)\n"),
];
#[test]
fn a_diff_error_leads_its_line_with_the_revision_side() {
    for (row, &(side, expected)) in DIFF_ERROR_TEXT.iter().enumerate() {
        let mut error = fixtures::duplicate();
        error.side = Some(side);
        assert_eq!(print(&Record::Error(error)).as_deref(), Ok(expected.as_bytes()), "row {row}: error prefix");
    }
}

const COMPOSITION_LINES: &[(crate::errors::Composition, &[u8], &[u8])] = &[
    (crate::errors::Composition::Duplicate, b"stoa-1-lab: two targets are named stoa-1-lab\n", b"{\"record\":\"error\",\"kind\":\"composition\",\"composition\":\"duplicate\",\"class\":\"configuration\",\"target\":\"stoa-1-lab\"}\n"),
    (crate::errors::Composition::Long, b"stoa-1-lab: a target name past 63 bytes\n", b"{\"record\":\"error\",\"kind\":\"composition\",\"composition\":\"long\",\"class\":\"configuration\",\"target\":\"stoa-1-lab\"}\n"),
    (crate::errors::Composition::Placement, b"stoa-1-lab: placement in fleet.json, a base or its member\n", b"{\"record\":\"error\",\"kind\":\"composition\",\"composition\":\"placement\",\"class\":\"configuration\",\"target\":\"stoa-1-lab\"}\n"),
    (crate::errors::Composition::Members, b"stoa-1-lab: members in fleet.json or an environment\n", b"{\"record\":\"error\",\"kind\":\"composition\",\"composition\":\"members\",\"class\":\"configuration\",\"target\":\"stoa-1-lab\"}\n"),
    (crate::errors::Composition::Unplaced, b"stoa-1-lab: no placement once composed\n", b"{\"record\":\"error\",\"kind\":\"composition\",\"composition\":\"unplaced\",\"class\":\"configuration\",\"target\":\"stoa-1-lab\"}\n"),
    (crate::errors::Composition::Null, b"stoa-1-lab: null for a task in a default\n", b"{\"record\":\"error\",\"kind\":\"composition\",\"composition\":\"null\",\"class\":\"configuration\",\"target\":\"stoa-1-lab\"}\n"),

];
#[test]
fn each_composition_error_prints_its_text() {
    for (row, (why, text, _)) in COMPOSITION_LINES.iter().enumerate() {
        assert_eq!(print(&Record::Error(fixtures::composition_error(why.clone()))).as_deref(), Ok(*text), "row {row}");
    }
}
#[test]
fn each_composition_error_prints_its_json_fields() {
    for (row, (why, _, json)) in COMPOSITION_LINES.iter().enumerate() {
        assert_eq!(
            form::print(
                Form::JsonLines,
                &Record::Error(fixtures::composition_error(why.clone())),
                Some(Type::Error),
                &limits::SHIPPED
            )
            .as_deref(),
            Ok(*json),
            "row {row}"
        );
    }
}
mod validation;
const ON_LINES: &[(crate::errors::Host, &[u8], &[u8])] = &[
    (crate::errors::Host::Unknown, b"stoa-1-lab: placement's on names rhi-lab, no discovered target\n", b"{\"record\":\"error\",\"kind\":\"composition\",\"composition\":{\"kind\":\"on\",\"on\":\"rhi-lab\",\"host\":\"unknown\"},\"class\":\"configuration\",\"target\":\"stoa-1-lab\"}\n"),
    (crate::errors::Host::Container, b"stoa-1-lab: placement's on names rhi-lab, a container\n", b"{\"record\":\"error\",\"kind\":\"composition\",\"composition\":{\"kind\":\"on\",\"on\":\"rhi-lab\",\"host\":\"container\"},\"class\":\"configuration\",\"target\":\"stoa-1-lab\"}\n"),
];
#[test]
fn an_on_error_prints_its_text() {
    for (row, (host, text, _)) in ON_LINES.iter().enumerate() {
        assert_eq!(print(&Record::Error(fixtures::composition_on(host.clone()))).as_deref(), Ok(*text), "row {row}");
    }
}
#[test]
fn an_on_error_prints_its_json_fields() {
    for (row, (host, _, json)) in ON_LINES.iter().enumerate() {
        assert_eq!(
            form::print(
                Form::JsonLines,
                &Record::Error(fixtures::composition_on(host.clone())),
                Some(Type::Error),
                &limits::SHIPPED
            )
            .as_deref(),
            Ok(*json),
            "row {row}"
        );
    }
}
#[test]
fn an_on_elsewhere_error_prints_its_text() {
    let error = fixtures::composition_on(crate::errors::Host::Elsewhere { environment: Box::from(b"lab".as_slice()) });
    assert_eq!(
        print(&Record::Error(error)).as_deref(),
        Ok(b"stoa-1-lab: placement's on names rhi-lab, a target of lab\n".as_slice())
    );
}
#[test]
fn an_on_elsewhere_error_prints_its_json_fields() {
    let error = fixtures::composition_on(crate::errors::Host::Elsewhere { environment: Box::from(b"lab".as_slice()) });
    assert_eq!(form::print(Form::JsonLines, &Record::Error(error), Some(Type::Error), &limits::SHIPPED).as_deref(), Ok(b"{\"record\":\"error\",\"kind\":\"composition\",\"composition\":{\"kind\":\"on\",\"on\":\"rhi-lab\",\"host\":{\"kind\":\"elsewhere\",\"environment\":\"lab\"}},\"class\":\"configuration\",\"target\":\"stoa-1-lab\"}\n".as_slice()));
}
#[test]
fn a_missing_task_error_names_its_task_in_text() {
    assert_eq!(
        print(&Record::Error(fixtures::missing_task())).as_deref(),
        Ok(b"no task motd under tasks/\n".as_slice())
    );
}
#[test]
fn a_missing_task_error_names_its_task_in_json() {
    assert_eq!(form::print(Form::JsonLines, &Record::Error(fixtures::missing_task()), Some(Type::Error), &limits::SHIPPED).as_deref(), Ok(b"{\"record\":\"error\",\"kind\":\"composition\",\"composition\":\"task\",\"class\":\"configuration\",\"task\":\"motd\"}\n".as_slice()));
}

mod forms;

#[test]
fn a_collision_prints_its_key_and_both_places() {
    assert_eq!(
        print(&Record::Error(fixtures::key_collision())).as_deref(),
        Ok(b"envs/lab.json:1:14: the key \"ops\" is made twice (envs/lab.json:1:2)\n".as_slice())
    );
}
#[test]
fn a_collision_prints_its_json_kind_and_key() {
    let printed =
        form::print(Form::JsonLines, &Record::Error(fixtures::key_collision()), Some(Type::Error), &limits::SHIPPED);
    assert_eq!(printed.as_deref(), Ok(b"{\"record\":\"error\",\"kind\":\"collision\",\"collision\":{\"key\":\"ops\",\"kind\":\"key\"},\"class\":\"configuration\",\"at\":{\"column\":14,\"line\":1,\"path\":\"envs/lab.json\"},\"also\":{\"column\":2,\"line\":1,\"path\":\"envs/lab.json\"}}\n".as_slice()));
}

mod templates;

mod exported;

const RESOLUTION_MESSAGES: &[(fixtures::ResolutionInput, &str)] = &[
    (fixtures::ResolutionInput::Unknown, "apt cannot locate package \"apt\"\n"),
    (fixtures::ResolutionInput::Unsatisfiable, "unsatisfiable dependencies of \"apt\"\n"),
    (fixtures::ResolutionInput::Absent, "the archive http://one/ does not have forky\n"),
    (fixtures::ResolutionInput::Missing, "\"apt\" was requested, and apt installs no package of that name\n"),
    (fixtures::ResolutionInput::Unmatched, "apt would install apt 3.3.3 amd64, which no Packages index lists\n"),
    (fixtures::ResolutionInput::Disagree, "the Packages indices disagree on apt 3.3.3 amd64\n"),
    (fixtures::ResolutionInput::TooMany, "a closure past 8192 packages\n"),
];
#[test]
fn each_resolution_message_prints_as_worded() {
    for (row, &(input, expected)) in RESOLUTION_MESSAGES.iter().enumerate() {
        assert_eq!(
            print(&Record::Error(fixtures::resolution_error(input))).as_deref(),
            Ok(expected.as_bytes()),
            "row {row}: {input:?}"
        );
    }
}
#[test]
fn mmdebstraps_timeout_prints_as_timed_out() {
    assert_eq!(
        print(&Record::Error(fixtures::mmdebstrap_error(crate::errors::Ended::Timeout))).as_deref(),
        Ok(b"mmdebstrap timed out\n".as_slice())
    );
}
#[test]
fn mmdebstraps_refused_start_prints_as_could_not_start() {
    assert_eq!(
        print(&Record::Error(fixtures::mmdebstrap_error(crate::errors::Ended::Start))).as_deref(),
        Ok(b"mmdebstrap could not start\n".as_slice())
    );
}

#[test]
fn a_resolution_error_names_its_package() {
    assert_eq!(form::print(Form::JsonLines,
        &Record::Error(fixtures::resolution_error(fixtures::ResolutionInput::Missing)),
        Some(Type::Error), &limits::SHIPPED).as_deref(),
        Ok(b"{\"record\":\"error\",\"kind\":\"resolution\",\"resolution\":{\"kind\":\"missing\",\"package\":\"apt\"},\"class\":\"configuration\"}\n".as_slice()));
}
