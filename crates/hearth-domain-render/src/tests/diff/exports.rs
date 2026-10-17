use crate::{
    diff::{self, Layer, Rebuild, TargetChange},
    evaluate::Value,
    parts::{
        PartRecord,
        exports::{self, ExportChange},
    },
    tests::fixtures,
};
use skein_lib::bytes::copy_of;
#[test]
fn a_changed_export_lists_its_before_and_after() {
    let before = fixtures::export_endpoint(Value::Int(1));
    let after = fixtures::export_endpoint(Value::Int(2));
    assert_eq!(
        exports::change(Some(&before), Some(&after)).entries.as_ref(),
        &[ExportChange {
            task: copy_of(b"vpn"),
            name: copy_of(b"endpoint"),
            before: Some(Value::Int(1)),
            after: Some(Value::Int(2)),
        }]
    );
}
const ONE_SIDE: &[(bool, Option<Value>, Option<Value>)] =
    &[(true, Some(Value::Int(1)), None), (false, None, Some(Value::Int(1)))];
#[test]
fn an_added_or_removed_export_has_one_side() {
    let part = fixtures::export_endpoint(Value::Int(1));
    for (from, before, after) in ONE_SIDE {
        let (old, new) = if *from { (Some(&part), None) } else { (None, Some(&part)) };
        let change = exports::change(old, new);
        let entry = fixtures::only(&change.entries);
        assert_eq!((&entry.before, &entry.after), (before, after), "from={from}");
    }
}
#[test]
fn an_unchanged_export_is_left_out() {
    let part = fixtures::export_endpoint(Value::Int(1));
    assert_eq!(exports::change(Some(&part), Some(&part)).entries.as_ref(), &[]);
}
#[test]
fn exports_map_equality_uses_its_canonical_items() {
    let before = fixtures::export_endpoint(Value::Map(Box::new([
        crate::evaluate::Entry { key: copy_of(b"b"), value: Some(Value::Int(2)) },
        crate::evaluate::Entry { key: copy_of(b"a"), value: Some(Value::Int(1)) },
    ])));
    let after = fixtures::export_endpoint(Value::Map(Box::new([
        crate::evaluate::Entry { key: copy_of(b"a"), value: Some(Value::Int(1)) },
        crate::evaluate::Entry { key: copy_of(b"b"), value: Some(Value::Int(2)) },
    ])));
    assert_eq!(exports::change(Some(&before), Some(&after)).entries.as_ref(), &[]);
}
#[test]
fn changed_exports_are_ordered_by_task_then_name() {
    let after = fixtures::exports(&[
        (b"vpn", &[(b"z", Value::Int(1)), (b"a", Value::Int(2))]),
        (b"core", &[(b"name", Value::Bool(true))]),
    ]);
    let change = exports::change(None, Some(&after));
    let mut names = skein_lib::List::with_capacity(3);
    for entry in &change.entries {
        names.push((entry.task.as_ref(), entry.name.as_ref())).expect("three exports");
    }
    assert_eq!(names.as_slice(), &[(b"core".as_slice(), b"name".as_slice()), (b"vpn", b"a"), (b"vpn", b"z")]);
}
const REBUILDS: &[(bool, Rebuild)] = &[(false, Rebuild::Nothing), (true, Rebuild::FromConfig)];
#[test]
fn exports_alone_rebuild_nothing_and_with_config_from_config() {
    for (config, expected) in REBUILDS {
        let TargetChange::Changed { rebuild, .. } = fixtures::exports_compared(*config) else { panic!("changed") };
        assert_eq!(rebuild, *expected, "config={config}");
    }
}
#[test]
fn exports_are_an_other_layer() {
    assert_eq!(diff::layer(&PartRecord::Exports(fixtures::export_endpoint(Value::Int(1)))), Layer::Other);
}
