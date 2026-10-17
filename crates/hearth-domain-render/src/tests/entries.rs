use super::fixtures::{self, Roots, composition::at, expr, guard, key, name, op, port_is, requires};
use crate::{
    entries,
    errors::{Guard, Kind},
    evaluate::{Missing, Op},
    limits::SHIPPED,
    revision::declaration::Condition,
};
use skein_lib::bytes::copy_of;
#[test]
fn a_true_guard_passes_and_a_false_one_errs_at_its_key_with_its_message() {
    let declaration = requires([
        guard(b"ssh", at(3, 5), port_is(22, at(3, 14)), b"ssh must listen on 22"),
        guard(b"web", at(4, 5), port_is(2222, at(4, 14)), b"web needs 2222"),
    ]);
    assert_eq!(
        entries::guards(&declaration, &Roots::x1_prod().context(), &SHIPPED).as_ref(),
        &[entries::Fault {
            kind: Kind::Guard(Guard { name: copy_of(b"ssh"), message: copy_of(b"ssh must listen on 22") }),
            at: at(3, 5),
            also: None
        }]
    );
}
#[test]
fn every_false_guard_of_every_present_task_errs() {
    let (mut side, checked) = fixtures::guarded_tasks();
    crate::call::guards::check(&side.documents, &mut side.errors, &SHIPPED, &fixtures::meta(), &checked);
    let (errors, _) = side.errors.finish();
    let mut names = skein_lib::List::with_capacity(u32::try_from(errors.len()).expect("fault count"));
    for error in &errors {
        names.push(error.task.as_deref()).expect("fault count");
    }
    assert_eq!(names.as_slice(), &[Some(b"core".as_slice()), Some(b"motd".as_slice())]);
}
#[test]
fn a_guard_reads_settings_tasks_and_target() {
    let declaration = requires([guard(
        b"context",
        at(3, 5),
        Condition {
            at: at(3, 20),
            expression: expr(
                4,
                [
                    name(0, b"settings", Missing::Error),
                    key(9, b"port", Missing::Error),
                    name(20, b"tasks", Missing::Error),
                    key(26, b"core", Missing::Error),
                    key(31, b"domain", Missing::Error),
                    name(40, b"target", Missing::Error),
                    key(47, b"name", Missing::Error),
                    op(55, Op::List { length: 3 }),
                ],
            ),
        },
        b"context is wrong",
    )]);
    assert_eq!(entries::guards(&declaration, &Roots::x1_prod().context(), &SHIPPED).as_ref(), &[]);
}
#[test]
fn a_guard_that_cannot_be_evaluated_errs_at_its_condition() {
    let mut declaration = fixtures::optional_guard();
    fixtures::only_mut(&mut declaration.requires).when.expression = expr(
        2,
        [
            name(0, b"settings", Missing::Error),
            key(9, b"wifi", Missing::Error),
            op(17, Op::Integer(1)),
            op(14, Op::Equal),
        ],
    );
    assert_eq!(
        entries::guards(&declaration, &Roots::x1_prod().context(), &SHIPPED).as_ref(),
        &[entries::Fault { kind: Kind::Evaluation(FaultKind::Undefined), at: at(3, 20), also: None }]
    );
}
#[test]
fn a_guard_over_an_undefined_optional_setting_is_false_not_an_error() {
    assert_eq!(
        entries::guards(&fixtures::optional_guard(), &Roots::x1_prod().context(), &SHIPPED).as_ref(),
        &[entries::Fault {
            kind: Kind::Guard(Guard { name: copy_of(b"wifi"), message: copy_of(b"wifi needed") }),
            at: at(3, 5),
            also: None
        }]
    );
}

use crate::{
    evaluate::{FaultKind, Operand, Value},
    revision::declaration::Map,
};
use fixtures::{
    THREE_ENTRIES, condition, entries as declared_entries, entry, entry_map, entry_maps, entry_output, expanded_keys,
    list_entry, numeric_entry, only, parts, plain, setting,
};
#[test]
fn a_for_each_entry_stands_for_one_entry_per_list_element_from_zero() {
    let expanded =
        entries::expand(&declared_entries([list_entry()]), &Roots::x1_prod().context(), &SHIPPED, &mut { u64::MAX })
            .expect("expand");
    assert_eq!(expanded_keys(&expanded).as_ref(), &[b"/etc/0-sudo".as_slice(), b"/etc/1-wheel".as_slice()]);
}
#[test]
fn a_for_each_over_a_map_binds_key_and_value_in_key_order_skipping_absent() {
    let key =
        parts([entry_output([name(0, b"key", Missing::Error)]), entry_output([name(0, b"value", Missing::Error)])]);
    let declared = declared_entries([entry(key, Some(setting(b"peers")), None)]);
    let expanded = entries::expand(&declared, &fixtures::peers_with_absent().context(), &SHIPPED, &mut { u64::MAX })
        .expect("expand");
    assert_eq!(expanded_keys(&expanded).as_ref(), &[b"a1".as_slice(), b"b2".as_slice()]);
}
#[test]
fn an_if_false_for_an_element_leaves_that_element_out() {
    let mut entry = list_entry();
    entry.when = Some(condition([name(0, b"key", Missing::Error), op(7, Op::Integer(0)), op(4, Op::Equal)]));
    let expanded =
        entries::expand(&declared_entries([entry]), &Roots::x1_prod().context(), &SHIPPED, &mut { u64::MAX })
            .expect("expand");
    assert_eq!(expanded_keys(&expanded).as_ref(), &[b"/etc/0-sudo".as_slice()]);
}
#[test]
fn an_entry_if_over_an_undefined_optional_setting_leaves_the_entry_out() {
    let declared = declared_entries([entry(plain(b"ops"), None, Some(setting(b"wifi")))]);
    let expanded =
        entries::expand(&declared, &Roots::x1_prod().context(), &SHIPPED, &mut { u64::MAX }).expect("expand");
    assert_eq!(only(&expanded.groups).entries.len(), 0);
}
const UNBOUND: &[&[u8]] = &[b"key", b"value"];
#[test]
fn key_and_value_in_a_plain_entrys_if_are_unknown() {
    for (row, name) in UNBOUND.iter().enumerate() {
        let declared =
            declared_entries([entry(plain(b"ops"), None, Some(condition([fixtures::name(0, name, Missing::Error)])))]);
        assert_eq!(
            entries::expand(&declared, &Roots::x1_prod().context(), &SHIPPED, &mut { u64::MAX }),
            Err(entries::Fault { kind: Kind::Evaluation(FaultKind::Unknown), at: at(3, 20), also: None }),
            "row {row}"
        );
    }
}
const ITERABLE_FAULTS: &[(&[u8], FaultKind)] = &[(b"wifi", FaultKind::Undefined), (b"port", FaultKind::Type)];
#[test]
fn a_for_each_over_undefined_or_a_scalar_fails_at_its_expression() {
    for (row, &(setting_name, kind)) in ITERABLE_FAULTS.iter().enumerate() {
        let declared = declared_entries([entry(plain(b"ops"), Some(setting(setting_name)), None)]);
        assert_eq!(
            entries::expand(&declared, &Roots::x1_prod().context(), &SHIPPED, &mut { u64::MAX }),
            Err(entries::Fault { kind: Kind::Evaluation(kind), at: at(3, 20), also: None }),
            "row {row}"
        );
    }
}
#[test]
fn two_elements_making_one_key_are_a_collision_naming_the_first() {
    let declared = declared_entries([entry(plain(b"ops"), Some(setting(b"tags")), None)]);
    assert_eq!(
        entries::expand(&declared, &Roots::x1_prod().context(), &SHIPPED, &mut { u64::MAX }),
        Err(entries::Fault {
            kind: Kind::Collision(crate::errors::Collision::Key { key: copy_of(b"ops") }),
            at: at(3, 5),
            also: Some(at(3, 5))
        })
    );
}
#[test]
fn two_entries_making_one_key_collide() {
    let mut second = entry(plain(b"ops"), None, None);
    second.key = crate::revision::declaration::Interpolated::Plain { at: at(4, 5), text: copy_of(b"ops") };
    let declared = declared_entries([entry(plain(b"ops"), None, None), second]);
    assert_eq!(
        entries::expand(&declared, &Roots::x1_prod().context(), &SHIPPED, &mut { u64::MAX }),
        Err(entries::Fault {
            kind: Kind::Collision(crate::errors::Collision::Key { key: copy_of(b"ops") }),
            at: at(4, 5),
            also: Some(at(3, 5))
        })
    );
}
#[test]
fn an_if_false_entry_makes_no_collision() {
    let declared = declared_entries([
        entry(plain(b"ops"), None, None),
        entry(plain(b"ops"), Some(setting(b"tags")), Some(condition([op(0, Op::Boolean(false))]))),
    ]);
    let expanded =
        entries::expand(&declared, &Roots::x1_prod().context(), &SHIPPED, &mut { u64::MAX }).expect("expand");
    assert_eq!(expanded_keys(&expanded).as_ref(), &[b"ops".as_slice()]);
}
#[test]
fn expanding_past_the_entry_limit_fails_at_the_for_each() {
    assert_eq!(
        entries::expand(&declared_entries([numeric_entry(4)]), &Roots::x1_prod().context(), &THREE_ENTRIES, &mut {
            u64::MAX
        }),
        Err(entries::Fault { kind: Kind::Evaluation(FaultKind::Limit), at: at(3, 20), also: None })
    );
}
#[test]
fn exactly_the_entry_limit_passes() {
    let expanded =
        entries::expand(&declared_entries([numeric_entry(3)]), &Roots::x1_prod().context(), &THREE_ENTRIES, &mut {
            u64::MAX
        })
        .expect("expand");
    assert_eq!(expanded_keys(&expanded).as_ref(), &[b"0".as_slice(), b"1".as_slice(), b"2".as_slice()]);
}
#[test]
fn the_entry_count_runs_across_a_tasks_maps() {
    let declared = entry_maps([entry_map(Map::Groups, [numeric_entry(3)]), entry_map(Map::Users, [list_entry()])]);
    assert_eq!(
        entries::expand(&declared, &Roots::x1_prod().context(), &THREE_ENTRIES, &mut { u64::MAX }),
        Err(entries::Fault { kind: Kind::Evaluation(FaultKind::Limit), at: at(3, 20), also: None })
    );
}
#[test]
fn the_entry_count_skips_if_false_elements() {
    let mut entry = numeric_entry(6);
    entry.when = Some(condition([name(0, b"key", Missing::Error), op(7, Op::Integer(3)), op(4, Op::GreaterOrEqual)]));
    let expanded =
        entries::expand(&declared_entries([entry]), &Roots::x1_prod().context(), &THREE_ENTRIES, &mut { u64::MAX })
            .expect("expand");
    assert_eq!(expanded_keys(&expanded).as_ref(), &[b"3".as_slice(), b"4".as_slice(), b"5".as_slice()]);
}
const ELEMENTS: &[(u32, i64, &[u8])] = &[(0, 0, b"sudo"), (1, 1, b"wheel")];
#[test]
fn an_elements_index_rebinds_its_key_and_value() {
    let roots = Roots::x1_prod();
    for (row, &(index, key, value)) in ELEMENTS.iter().enumerate() {
        let (bound_key, bound_value) =
            entries::element(&list_entry(), index, &roots.context(), &SHIPPED).expect("element");
        let Operand::Int(bound_key) = bound_key else { panic!("index") };
        let Operand::Value(Value::String(bound_value)) = bound_value else { panic!("list value") };
        assert_eq!((bound_key, bound_value.as_ref()), (key, value), "row {row}");
    }
}
#[test]
fn a_concrete_entry_keeps_its_element_index() {
    let expanded =
        entries::expand(&declared_entries([list_entry()]), &Roots::x1_prod().context(), &SHIPPED, &mut { u64::MAX })
            .expect("expand");
    let group = only(&expanded.groups);
    let mut elements = skein_lib::List::with_capacity(SHIPPED.expansion);
    for entry in &group.entries {
        elements.push(entry.element).expect("bounded entries");
    }
    assert_eq!(elements.as_slice(), &[Some(0), Some(1)]);
}
