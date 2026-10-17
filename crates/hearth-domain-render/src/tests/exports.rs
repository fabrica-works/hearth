use super::fixtures::{self, composition::at, entry_output, key, name, only, op, parts};
use crate::{
    errors::{ExportFault, Kind, Root, Schema, TypeName},
    evaluate::{Missing, Op, Value},
    parts::exports,
    revision::declaration::Form,
};
use hearth_lib::encode::Item;
use skein_lib::bytes::copy_of;
#[test]
fn an_integer_export_enters_the_record_as_its_decimal_text() {
    let exports = fixtures::exports(&[(b"vpn", &[(b"hub", Value::Bool(true)), (b"port", Value::Int(51820))])]);
    assert_eq!(
        exports::items(&exports).as_ref(),
        [
            Item::Map(1),
            Item::Bytes(copy_of(b"vpn")),
            Item::Map(2),
            Item::Bytes(copy_of(b"hub")),
            Item::True,
            Item::Bytes(copy_of(b"port")),
            Item::Bytes(copy_of(b"51820"))
        ]
    );
}
const NEGATIVE: &[(i64, &[u8])] = &[(-7, b"-7"), (i64::MIN, b"-9223372036854775808")];
#[test]
fn a_negative_integer_enters_as_its_decimal_text() {
    for (row, (number, expected)) in NEGATIVE.iter().enumerate() {
        assert_eq!(exports::value_items(&Value::Int(*number)).as_ref(), [Item::Bytes(copy_of(expected))], "row {row}");
    }
}
#[test]
fn items_walk_a_map_in_key_byte_order() {
    let value = fixtures::map(&[(b"z", Some(Value::Int(2))), (b"a", Some(Value::Int(1)))]);
    assert_eq!(
        exports::value_items(&value).as_ref(),
        [
            Item::Map(2),
            Item::Bytes(copy_of(b"a")),
            Item::Bytes(copy_of(b"1")),
            Item::Bytes(copy_of(b"z")),
            Item::Bytes(copy_of(b"2"))
        ]
    );
}
#[test]
fn a_value_with_text_around_its_tag_is_refused() {
    let value = parts([crate::evaluate::template::Does::Text(copy_of(b"v ")), entry_output([op(0, Op::Integer(7))])]);
    let declaration = fixtures::declared(Form::Int, value, false);
    assert_eq!(only(&fixtures::checked(&declaration)).schema, Schema::Export { fault: ExportFault::Form });
}
const ROOTS: &[(&[u8], Root)] = &[(b"tasks", Root::Tasks), (b"exports", Root::Exports), (b"debian", Root::Debian)];
#[test]
fn an_export_names_settings_and_target_alone() {
    for (row, (root, expected)) in ROOTS.iter().enumerate() {
        let declaration =
            fixtures::declared(Form::String, parts([entry_output([name(0, root, Missing::Error)])]), false);
        assert_eq!(
            only(&fixtures::checked(&declaration)).schema,
            Schema::Export { fault: ExportFault::Root { root: *expected } },
            "row {row}"
        );
    }
}
#[test]
fn an_export_may_name_its_target() {
    let declaration = fixtures::declared(
        Form::String,
        parts([entry_output([name(0, b"target", Missing::Error), key(7, b"name", Missing::Error)])]),
        false,
    );
    assert_eq!(fixtures::checked(&declaration).as_ref(), []);
}
#[test]
fn an_export_naming_an_undeclared_setting_is_refused() {
    let declaration = fixtures::declared(
        Form::String,
        parts([entry_output([name(0, b"settings", Missing::Error), key(9, b"pot", Missing::Error)])]),
        false,
    );
    assert_eq!(
        only(&fixtures::checked(&declaration)).schema,
        Schema::Undeclared { setting: copy_of(b"pot"), declared: Box::new([copy_of(b"port"), copy_of(b"wifi")]) }
    );
}
#[test]
fn an_optional_export_undefined_is_absent() {
    let declaration = fixtures::optional_export(true);
    assert_eq!(only(&fixtures::evaluate_exports(&declaration).expect("optional").values).value, None);
}
#[test]
fn a_required_undefined_export_errs() {
    let declaration = fixtures::optional_export(false);
    assert_eq!(
        fixtures::evaluate_exports(&declaration),
        Err(crate::entries::Fault {
            kind: Kind::Evaluation(crate::evaluate::FaultKind::Undefined),
            at: at(3, 5),
            also: None
        })
    );
}
const WRONG: &[(Form, Value, TypeName)] = &[
    (Form::Bool, Value::Int(7), TypeName::Bool),
    (Form::Int, Value::Bool(true), TypeName::Int),
    (Form::String, Value::Int(7), TypeName::String),
    (Form::Path, Value::Bool(true), TypeName::Path),
    (Form::Mode, Value::Int(7), TypeName::Mode),
    (Form::Duration, Value::Bool(true), TypeName::Duration),
    (Form::List(1), Value::Int(7), TypeName::List),
    (Form::Map(1), Value::Bool(true), TypeName::Map),
];
#[test]
fn an_exports_value_must_have_its_declared_type() {
    for (row, (form, value, expected)) in WRONG.iter().enumerate() {
        assert_eq!(
            fixtures::conform(value, form.clone()),
            Err(Schema::Wrong { expected: expected.clone() }),
            "row {row}"
        );
    }
}
#[test]
fn a_list_made_deeper_than_its_type_is_refused() {
    assert_eq!(
        fixtures::conform(&fixtures::lists(crate::limits::SHIPPED.nesting, Value::Int(1)), Form::List(1)),
        Err(Schema::Wrong { expected: TypeName::Int })
    );
}
#[test]
fn an_export_of_type_target_is_checked_on_each_target() {
    assert_eq!(
        fixtures::conform(&fixtures::string(b"other-prod"), Form::Target),
        Err(Schema::Target { name: copy_of(b"other-prod"), environment: copy_of(b"prod") })
    );
}
#[test]
fn an_export_of_type_file_is_checked_on_each_target() {
    assert_eq!(
        fixtures::conform(&fixtures::string(b"vpn/ca.crt"), Form::File),
        Err(Schema::File { path: copy_of(b"vpn/ca.crt") })
    );
}
#[test]
fn no_export_makes_an_empty_part() {
    assert_eq!(exports::items(&exports::build(Box::new([]))).as_ref(), [Item::Map(0)]);
}

#[test]
fn an_integer_export_evaluates_to_an_integer() {
    let exported = fixtures::evaluate_exports(&fixtures::integer(51820)).expect("integer export");
    assert_eq!(only(&exported.values).value, Some(Value::Int(51820)));
}
#[test]
fn the_empty_exports_digest_hashes_its_canonical_record() {
    assert_eq!(
        exports::digest(&exports::build(Box::new([]))),
        fixtures::digest("970579fd41bed6fd7965e6a303fb756dfa50b8b3e3a9d94d24489233967aaf46")
    );
}
