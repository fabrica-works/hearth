use super::fixtures;
use crate::{
    errors::{Format, Kind, Schema},
    parts::live::{self, Action, Consumer, Covered, Live},
};
#[test]
fn two_entries_on_one_path_make_two_consumers() {
    let mut input = fixtures::live_entry(b"reload", Some(b"postfix.service"), &[b"/etc/mailname"]);
    let other = fixtures::live_entry(b"restart", Some(b"nginx.service"), &[b"/etc/mailname"]);
    input.tasks = Box::new([fixtures::only(&input.tasks).clone(), fixtures::only(&other.tasks).clone()]);
    assert_eq!(
        live::build(&input).expect("live"),
        Live {
            paths: Box::new([Covered {
                path: Box::from(&b"/etc/mailname"[..]),
                consumers: Box::new([
                    Consumer {
                        task: Box::from(&b"motd"[..]),
                        unit: Some(Box::from(&b"nginx.service"[..])),
                        action: Action::Restart
                    },
                    Consumer {
                        task: Box::from(&b"motd"[..]),
                        unit: Some(Box::from(&b"postfix.service"[..])),
                        action: Action::Reload
                    },
                ])
            }])
        }
    );
}
#[test]
fn equal_consumers_of_one_path_make_one() {
    let mut input = fixtures::live_entry(b"none", None, &[b"/opt/tool"]);
    input.tasks = Box::new([fixtures::only(&input.tasks).clone(), fixtures::only(&input.tasks).clone()]);
    let live = live::build(&input).expect("live");
    assert_eq!(
        fixtures::only(&live.paths).consumers.as_ref(),
        &[Consumer { task: Box::from(&b"motd"[..]), unit: None, action: Action::None }]
    );
}
#[test]
fn a_unit_is_required_but_with_none() {
    assert_eq!(
        fixtures::live_fault(b"reload", None, &[b"/etc/file"]).kind,
        Kind::Schema(Schema::MissingField { field: Box::from(&b"unit"[..]) })
    );
}
#[test]
fn none_needs_no_unit() {
    let live = live::build(&fixtures::live_entry(b"none", None, &[b"/opt/tool"])).expect("none");
    assert_eq!(fixtures::only(&fixtures::only(&live.paths).consumers).unit, None);
}
const INVALID_UNITS: &[&[u8]] = &[b"not a unit", b"/etc/tool.service", b"tool.unknown", b".service"];
#[test]
fn a_unit_that_is_no_unit_name_is_malformed() {
    for (row, unit) in INVALID_UNITS.iter().enumerate() {
        assert_eq!(
            fixtures::live_fault(b"reload", Some(unit), &[b"/etc/file"]).kind,
            Kind::Schema(Schema::Malformed { format: Format::Key }),
            "row {row}"
        );
    }
}
#[test]
fn an_empty_paths_is_a_missing_field() {
    assert_eq!(
        fixtures::live_fault(b"none", None, &[]).kind,
        Kind::Schema(Schema::MissingField { field: Box::from(&b"paths"[..]) })
    );
}
#[test]
fn a_path_twice_in_one_entry_is_malformed() {
    assert_eq!(
        fixtures::live_fault(b"none", None, &[b"/etc/file", b"/etc/file"]).kind,
        Kind::Schema(Schema::Malformed { format: Format::Path })
    );
}
const OUTSIDE: &[&[u8]] = &[b"/etcetera", b"/etc/", b"/opt", b"/usr/local", b"/usr/localx/tool"];
#[test]
fn a_path_outside_the_three_roots_or_one_itself_is_beneath() {
    for (row, path) in OUTSIDE.iter().enumerate() {
        assert_eq!(
            fixtures::live_fault(b"none", None, &[path]).kind,
            Kind::Schema(Schema::Beneath {
                allowed: Box::new([Box::from(&b"/etc"[..]), Box::from(&b"/usr/local"[..]), Box::from(&b"/opt"[..])])
            }),
            "row {row}"
        );
    }
}
#[test]
fn a_trailing_slash_covers_a_subtree_and_is_kept() {
    let live = live::build(&fixtures::live_entry(b"none", None, &[b"/opt/tool/"])).expect("subtree");
    assert_eq!(fixtures::only(&live.paths).path.as_ref(), b"/opt/tool/");
}
#[test]
fn no_entry_makes_an_empty_part() {
    assert_eq!(
        live::build(&fixtures::concrete_expansion(crate::revision::declaration::Map::Live, &[])).expect("empty"),
        Live { paths: Box::new([]) }
    );
}

#[test]
fn none_keeps_an_optional_unit() {
    let live =
        live::build(&fixtures::live_entry(b"none", Some(b"tool.service"), &[b"/opt/tool"])).expect("none with unit");
    assert_eq!(fixtures::only(&fixtures::only(&live.paths).consumers).unit.as_deref(), Some(&b"tool.service"[..]));
}
