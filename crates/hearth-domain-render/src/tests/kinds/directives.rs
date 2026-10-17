use super::super::fixtures::{
    self,
    kinds::{self as fixture, text},
};
use crate::{
    errors::{Format, Kind, Repository, Schema},
    parts::config::Action,
    revision::{
        Position,
        declaration::{Fragment, Name},
    },
};
const LINE_TYPES: &[(&[u8], crate::parts::config::LineKind)] = &[
    (b"d", crate::parts::config::LineKind::D),
    (b"D", crate::parts::config::LineKind::CapitalD),
    (b"e", crate::parts::config::LineKind::E),
    (b"f", crate::parts::config::LineKind::F),
    (b"f+", crate::parts::config::LineKind::FPlus),
    (b"w", crate::parts::config::LineKind::W),
    (b"w+", crate::parts::config::LineKind::WPlus),
    (b"L", crate::parts::config::LineKind::CapitalL),
    (b"L+", crate::parts::config::LineKind::CapitalLPlus),
    (b"C", crate::parts::config::LineKind::CapitalC),
    (b"z", crate::parts::config::LineKind::Z),
    (b"Z", crate::parts::config::LineKind::CapitalZ),
    (b"r", crate::parts::config::LineKind::R),
    (b"R", crate::parts::config::LineKind::CapitalR),
];
#[test]
fn each_of_the_fourteen_tmpfiles_types_is_typed() {
    for (row, &(name, kind)) in LINE_TYPES.iter().enumerate() {
        assert_eq!(fixture::tmpfile(Some(name), None).expect("type").line.kind, kind, "row {row}");
    }
}
#[test]
fn an_unknown_tmpfiles_type_lists_the_fourteen_types() {
    assert_eq!(
        fixture::tmpfile(Some(b"q"), None).expect_err("type").kind,
        Kind::Schema(Schema::Enum {
            value: b"q".as_slice().into(),
            listed: Box::new([
                b"d".as_slice().into(),
                b"D".as_slice().into(),
                b"e".as_slice().into(),
                b"f".as_slice().into(),
                b"f+".as_slice().into(),
                b"w".as_slice().into(),
                b"w+".as_slice().into(),
                b"L".as_slice().into(),
                b"L+".as_slice().into(),
                b"C".as_slice().into(),
                b"z".as_slice().into(),
                b"Z".as_slice().into(),
                b"r".as_slice().into(),
                b"R".as_slice().into(),
            ])
        })
    );
}
const ACTIONS: &[(&[u8], Action)] = &[(b"enable", Action::Enable), (b"disable", Action::Disable)];
#[test]
fn a_preset_takes_enable_or_disable() {
    for (row, &(value, action)) in ACTIONS.iter().enumerate() {
        let preset = fixture::preset([text(Name::Action, value)]).expect("action");
        assert_eq!(preset.action, action, "row {row}");
    }
}
#[test]
fn an_unknown_preset_action_is_an_enum_fault() {
    let fault = fixture::preset([text(Name::Action, b"start")]).expect_err("action");
    assert_eq!(
        fault.kind,
        Kind::Schema(Schema::Enum {
            value: b"start".as_slice().into(),
            listed: Box::new([b"enable".as_slice().into(), b"disable".as_slice().into()])
        })
    );
}
#[test]
fn a_preset_without_action_is_missing() {
    let fault = fixture::preset([]).expect_err("action");
    assert_eq!(fault.kind, Kind::Schema(Schema::MissingField { field: b"action".as_slice().into() }));
}
#[test]
fn a_tmpfiles_line_needs_its_type() {
    assert_eq!(
        fixture::tmpfile(None, None).expect_err("missing type").kind,
        Kind::Schema(Schema::MissingField { field: b"type".as_slice().into() })
    );
}
const BAD_MODES: &[&[u8]] = &[b"0888", b"64", b"10000"];
#[test]
fn a_tmpfiles_line_checks_its_mode() {
    for (row, &mode) in BAD_MODES.iter().enumerate() {
        assert_eq!(
            fixture::tmpfile(Some(b"d"), Some(mode)).expect_err("bad mode").kind,
            Kind::Schema(Schema::Malformed { format: Format::Mode }),
            "row {row}"
        );
    }
}
#[test]
fn a_declared_script_with_no_file_is_a_repository_error() {
    let faults = fixtures::missing_script();
    assert_eq!(fixtures::only(&faults).kind, Kind::Repository(Repository::NoScript));
}
#[test]
fn a_missing_script_is_at_its_declared_key() {
    let faults = fixtures::missing_script();
    assert_eq!(
        fixtures::only(&faults).at,
        Some(crate::errors::Place {
            path: b"tasks/motd/task.json".as_slice().into(),
            position: Some(Position { line: 3, column: 5 })
        })
    );
}
const UNDECLARED: &[&[u8]] = &[fixtures::CUSTOMIZE, fixtures::SOFTWARE];
#[test]
fn a_script_file_nothing_declares_is_stray() {
    for (row, &path) in UNDECLARED.iter().enumerate() {
        let faults = fixtures::script_plan(&fixtures::declaration(&[], &[]), &[path]).expect_err("undeclared script");
        assert_eq!(fixtures::only(&faults).kind, Kind::Repository(Repository::Stray), "row {row}");
    }
}
#[test]
fn a_software_script_is_checked_not_digested() {
    assert_eq!(
        fixtures::script_plan(&fixtures::script_declaration(Fragment::Software), &[fixtures::SOFTWARE])
            .expect("software exists")
            .as_ref(),
        []
    );
}
#[test]
fn customize_sh_j2_is_digested_as_rendered() {
    let config = fixtures::rendered_scripts(&[(b"motd", fixtures::DEFAULT_SCRIPT_ORDER)], true);
    assert_eq!(fixtures::only(&config.scripts).content_digest, fixtures::HELLO_SHA256);
}
const ORDERED: &[(&[u8], u16)] = &[(b"z", 200), (b"b", 100), (b"a", 100)];
#[test]
fn scripts_sort_by_order_then_task() {
    let config = fixtures::rendered_scripts(ORDERED, false);
    let mut scripts = skein_lib::List::with_capacity(u32::try_from(config.scripts.len()).expect("task bound"));
    for row in &config.scripts {
        scripts.push((row.task.as_ref(), row.order)).expect("task bound");
    }
    assert_eq!(scripts.as_slice(), [(b"a".as_slice(), 100), (b"b".as_slice(), 100), (b"z".as_slice(), 200)]);
}
#[test]
fn a_typo_in_a_script_template_is_found_at_its_line() {
    let mut side =
        fixtures::template_call(fixtures::CUSTOMIZE_TEMPLATE, crate::revision::Mode::File, 30, crate::limits::SHIPPED);
    side.documents
        .set_declaration(&crate::revision::Oid(b"declaration".as_slice().into()), fixtures::declaration(&[], &[]));
    side.decoded(
        &crate::revision::Oid(b"template".as_slice().into()),
        crate::vocabulary::Decoded::Template(fixtures::template(
            &[0, 12],
            [fixtures::output(12, fixtures::reference_typo())],
        )),
    );
    assert_eq!(
        fixtures::only(&fixtures::kept(&side)).at,
        Some(crate::errors::Place {
            path: fixtures::CUSTOMIZE_TEMPLATE.into(),
            position: Some(Position { line: 2, column: 10 })
        })
    );
}
