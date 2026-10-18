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
    crate::call::guards::check(
        &side.documents,
        &mut side.errors,
        &SHIPPED,
        &fixtures::meta(),
        &checked,
        &Roots::x1_prod().debian,
    );
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
    let expanded = fixtures::expanded(&declared_entries([list_entry()]));
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
    let expanded = fixtures::expanded(&declared_entries([entry]));
    assert_eq!(expanded_keys(&expanded).as_ref(), &[b"/etc/0-sudo".as_slice()]);
}
#[test]
fn an_entry_if_over_an_undefined_optional_setting_leaves_the_entry_out() {
    let declared = declared_entries([entry(plain(b"ops"), None, Some(setting(b"wifi")))]);
    let expanded = fixtures::expanded(&declared);
    assert_eq!(only(&expanded.groups).entries.len(), 0);
}
const UNBOUND: &[&[u8]] = &[b"key", b"value"];
#[test]
fn key_and_value_in_a_plain_entrys_if_are_unknown() {
    for (row, name) in UNBOUND.iter().enumerate() {
        let declared =
            declared_entries([entry(plain(b"ops"), None, Some(condition([fixtures::name(0, name, Missing::Error)])))]);
        assert_eq!(
            fixtures::expansion(&declared, &SHIPPED, u64::MAX),
            Err(fixtures::evaluation_fault(FaultKind::Unknown, at(3, 20))),
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
            fixtures::expansion(&declared, &SHIPPED, u64::MAX),
            Err(fixtures::evaluation_fault(kind, at(3, 20))),
            "row {row}"
        );
    }
}
#[test]
fn two_elements_making_one_key_are_a_collision_naming_the_first() {
    let declared = declared_entries([entry(plain(b"ops"), Some(setting(b"tags")), None)]);
    assert_eq!(
        fixtures::expansion(&declared, &SHIPPED, u64::MAX),
        Err(fixtures::key_collision(b"ops", at(3, 5), at(3, 5)))
    );
}
#[test]
fn two_entries_making_one_key_collide() {
    let mut second = entry(plain(b"ops"), None, None);
    second.key = crate::revision::declaration::Interpolated::Plain { at: at(4, 5), text: copy_of(b"ops") };
    let declared = declared_entries([entry(plain(b"ops"), None, None), second]);
    assert_eq!(
        fixtures::expansion(&declared, &SHIPPED, u64::MAX),
        Err(fixtures::key_collision(b"ops", at(4, 5), at(3, 5)))
    );
}
#[test]
fn one_key_in_two_maps_makes_an_entry_in_each() {
    let declared = entry_maps([
        entry_map(Map::Software, [entry(plain(b"hearth"), None, None)]),
        entry_map(Map::Users, [entry(plain(b"hearth"), None, None)]),
    ]);
    let expanded = fixtures::expanded(&declared);
    let mut groups = skein_lib::List::with_capacity(SHIPPED.expansion);
    for group in &expanded.groups {
        groups.push((group.map, only(&group.entries).key.as_ref())).expect("bounded groups");
    }
    assert_eq!(groups.as_slice(), [(Map::Software, b"hearth".as_slice()), (Map::Users, b"hearth".as_slice())]);
}
#[test]
fn an_if_false_entry_makes_no_collision() {
    let declared = declared_entries([
        entry(plain(b"ops"), None, None),
        entry(plain(b"ops"), Some(setting(b"tags")), Some(condition([op(0, Op::Boolean(false))]))),
    ]);
    let expanded = fixtures::expanded(&declared);
    assert_eq!(expanded_keys(&expanded).as_ref(), &[b"ops".as_slice()]);
}
#[test]
fn expanding_past_the_entry_limit_fails_at_the_for_each() {
    assert_eq!(
        fixtures::expansion(&declared_entries([numeric_entry(THREE_ENTRIES.expansion + 1)]), &THREE_ENTRIES, u64::MAX),
        Err(fixtures::evaluation_fault(FaultKind::Limit, at(3, 20)))
    );
}
#[test]
fn exactly_the_entry_limit_passes() {
    let expanded =
        fixtures::expansion(&declared_entries([numeric_entry(THREE_ENTRIES.expansion)]), &THREE_ENTRIES, u64::MAX)
            .expect("expand");
    assert_eq!(expanded_keys(&expanded).as_ref(), &[b"0".as_slice(), b"1".as_slice(), b"2".as_slice()]);
}
#[test]
fn the_entry_count_runs_across_a_tasks_maps() {
    let declared = entry_maps([
        entry_map(Map::Groups, [numeric_entry(THREE_ENTRIES.expansion)]),
        entry_map(Map::Users, [list_entry()]),
    ]);
    assert_eq!(
        fixtures::expansion(&declared, &THREE_ENTRIES, u64::MAX),
        Err(fixtures::evaluation_fault(FaultKind::Limit, at(3, 20)))
    );
}
#[test]
fn the_entry_count_skips_if_false_elements() {
    let mut entry = numeric_entry(THREE_ENTRIES.expansion * 2);
    entry.when = Some(condition([
        name(0, b"key", Missing::Error),
        op(7, Op::Integer(i64::from(THREE_ENTRIES.expansion))),
        op(4, Op::GreaterOrEqual),
    ]));
    let expanded = fixtures::expansion(&declared_entries([entry]), &THREE_ENTRIES, u64::MAX).expect("expand");
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
    let expanded = fixtures::expanded(&declared_entries([list_entry()]));
    let group = only(&expanded.groups);
    let mut elements = skein_lib::List::with_capacity(SHIPPED.expansion);
    for entry in &group.entries {
        elements.push(entry.element).expect("bounded entries");
    }
    assert_eq!(elements.as_slice(), &[Some(0), Some(1)]);
}

use crate::{
    errors::{Schema, TypeName},
    revision::declaration::{Literal, Name},
};
use fixtures::{EXPANDED_TEXT, FIELD_AT, field, fields, setting_tag};
#[test]
fn a_key_and_a_field_interpolate_text_and_integers_in_order() {
    let mut entry = list_entry();
    entry.fields = Box::new([field(
        Name::From,
        Literal::Text(parts([
            crate::evaluate::template::Does::Text(copy_of(b"x")),
            entry_output([op(0, Op::Integer(-42))]),
            crate::evaluate::template::Does::Text(copy_of(b"y")),
            entry_output([name(0, b"value", Missing::Error)]),
        ])),
    )]);
    let expanded = fixtures::expanded(&declared_entries([entry]));
    let group = only(&expanded.groups);
    let mut fields = skein_lib::List::with_capacity(u32::try_from(group.entries.len()).expect("fixture entries"));
    for entry in &group.entries {
        fields.push(entry.text(Name::From)).expect("entry count");
    }
    assert_eq!(fields.as_slice(), [Some(b"x-42ysudo".as_slice()), Some(b"x-42ywheel".as_slice())]);
}
const BOOLEAN_STRINGS: &[fixtures::StringPlace] = &[
    fixtures::StringPlace::Key,
    fixtures::StringPlace::Field,
    fixtures::StringPlace::Mixed,
    fixtures::StringPlace::List,
];
#[test]
fn a_boolean_in_a_string_fails_with_type_wherever_it_stands() {
    for (row, &place) in BOOLEAN_STRINGS.iter().enumerate() {
        assert_eq!(
            fixtures::expansion(&fixtures::boolean_string(place), &SHIPPED, SHIPPED.expanded),
            Err(fixtures::evaluation_fault(FaultKind::Type, at(3, 5))),
            "row {row}"
        );
    }
}
const COLLECTIONS: &[&[u8]] = &[b"tags", b"peers"];
#[test]
fn a_list_or_map_in_a_string_fails_with_output() {
    for (row, name) in COLLECTIONS.iter().enumerate() {
        let declared = fields([field(Name::From, Literal::Text(setting_tag(name)))]);
        assert_eq!(
            fixtures::expansion(&declared, &SHIPPED, SHIPPED.expanded),
            Err(fixtures::evaluation_fault(FaultKind::Output, FIELD_AT)),
            "row {row}"
        );
    }
}
#[test]
fn a_field_written_as_one_tag_is_absent_when_undefined() {
    let declared = fields([
        field(Name::From, Literal::Text(setting_tag(b"wifi"))),
        field(Name::Uid, Literal::Typed(setting_tag(b"wifi"))),
    ]);
    let expanded = fixtures::expanded(&declared);
    assert_eq!(only(&only(&expanded.groups).entries).fields.as_ref(), &[]);
}
#[test]
fn an_undefined_key_is_an_error() {
    let declared = declared_entries([entry(setting_tag(b"wifi"), None, None)]);
    assert_eq!(fixtures::expansion(&declared, &SHIPPED, SHIPPED.expanded), Err(fixtures::UNDEFINED_FIELD));
}
#[test]
fn an_undefined_value_beside_text_is_an_error() {
    let declared = fields([field(Name::From, Literal::Text(fixtures::mixed_optional()))]);
    assert_eq!(fixtures::expansion(&declared, &SHIPPED, SHIPPED.expanded), Err(fixtures::UNDEFINED_FIELD));
}
const WRONG_TYPED: &[(Name, TypeName)] = &[
    (Name::Uid, TypeName::Int),
    (Name::Directory, TypeName::Bool),
    (Name::Groups, TypeName::List),
    (Name::Subordinate, TypeName::Object),
];
#[test]
fn a_typed_field_must_have_its_fields_type() {
    for (row, (name, expected)) in WRONG_TYPED.iter().enumerate() {
        let declared = fields([field(*name, Literal::Typed(setting_tag(b"name")))]);
        assert_eq!(
            fixtures::expansion(&declared, &SHIPPED, SHIPPED.expanded),
            Err(entries::Fault {
                kind: Kind::Schema(Schema::Wrong { expected: expected.clone() }),
                at: FIELD_AT,
                also: None
            }),
            "row {row}"
        );
    }
}
#[test]
fn a_typed_field_requires_one_output_tag() {
    let declared = fields([field(Name::Uid, Literal::Typed(plain(b"42")))]);
    assert_eq!(
        fixtures::expansion(&declared, &SHIPPED, SHIPPED.expanded),
        Err(entries::Fault { kind: Kind::Schema(Schema::Wrong { expected: TypeName::Int }), at: at(3, 5), also: None })
    );
}
#[test]
fn a_typed_fields_accessor_keeps_its_value() {
    let declared = fields([
        field(Name::Uid, Literal::Typed(setting_tag(b"port"))),
        field(Name::Directory, Literal::Typed(setting_tag(b"enabled"))),
    ]);
    let expanded = fixtures::expanded(&declared);
    let entry = only(&only(&expanded.groups).entries);
    assert_eq!((entry.int(Name::Uid), entry.bool(Name::Directory)), (Some(2222), Some(true)));
}
#[test]
fn a_list_fields_elements_interpolate_one_by_one() {
    let declared = fields([field(
        Name::Groups,
        Literal::Strings { at: FIELD_AT, each: Box::new([setting_tag(b"name"), plain(b"wheel")]) },
    )]);
    let expanded = fixtures::expanded(&declared);
    assert_eq!(
        only(&only(&expanded.groups).entries).strings(Name::Groups),
        Some([copy_of(b"x1"), copy_of(b"wheel")].as_slice())
    );
}
#[test]
fn a_typed_list_keeps_every_string() {
    let declared = fields([field(Name::Groups, Literal::Typed(setting_tag(b"tags")))]);
    let expanded = fixtures::expanded(&declared);
    assert_eq!(
        only(&only(&expanded.groups).entries).strings(Name::Groups),
        Some([copy_of(b"sudo"), copy_of(b"wheel")].as_slice())
    );
}
#[test]
fn an_undefined_value_in_a_list_is_an_error() {
    let declared =
        fields([field(Name::Groups, Literal::Strings { at: FIELD_AT, each: Box::new([setting_tag(b"wifi")]) })]);
    assert_eq!(fixtures::expansion(&declared, &SHIPPED, SHIPPED.expanded), Err(fixtures::UNDEFINED_FIELD));
}
#[test]
fn key_and_value_in_a_plain_entrys_string_field_are_unknown() {
    for (row, name) in UNBOUND.iter().enumerate() {
        let declared = fields([field(
            Name::From,
            Literal::Text(parts([entry_output([fixtures::name(0, name, Missing::Error)])])),
        )]);
        assert_eq!(
            fixtures::expansion(&declared, &SHIPPED, SHIPPED.expanded),
            Err(fixtures::evaluation_fault(FaultKind::Unknown, at(3, 5))),
            "row {row}"
        );
    }
}
#[test]
fn a_required_field_the_entry_lacks_is_a_missing_field_at_the_entry() {
    let expanded = fixtures::expanded(&fields([]));
    assert_eq!(
        crate::kinds::required(only(&only(&expanded.groups).entries), Name::Uid),
        Err(Schema::MissingField { field: copy_of(b"uid") })
    );
}
#[test]
fn expanded_text_past_the_targets_budget_fails_at_the_field() {
    let declared = fields([field(Name::From, Literal::Text(fixtures::budget_text()))]);
    assert_eq!(
        fixtures::expansion(&declared, &EXPANDED_TEXT, EXPANDED_TEXT.expanded),
        Err(fixtures::evaluation_fault(FaultKind::Limit, FIELD_AT))
    );
}
#[test]
fn a_typed_range_keeps_its_base_and_count() {
    let declaration = fields([field(Name::Subordinate, Literal::Typed(setting_tag(b"range")))]);
    let expanded =
        entries::expand(&declaration, &fixtures::range_roots().context(), &SHIPPED, &mut { SHIPPED.expanded })
            .expect("expand");
    assert_eq!(only(&only(&expanded.groups).entries).range(Name::Subordinate), Some((42, 2)));
}
#[test]
fn a_range_member_is_never_absent() {
    let declaration = fields([fixtures::range_field(Literal::Typed(setting_tag(b"wifi")))]);
    assert_eq!(fixtures::expansion(&declaration, &SHIPPED, SHIPPED.expanded), Err(fixtures::UNDEFINED_FIELD));
}
