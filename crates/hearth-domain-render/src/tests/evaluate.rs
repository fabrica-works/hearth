use super::fixtures::{self, Roots, key, name, op};
use crate::evaluate::{Fault, FaultKind, Missing, Op, Value};
use crate::limits;
use skein_lib::bytes;
#[test]
fn a_key_of_a_root_gives_the_maps_value() {
    assert_eq!(
        fixtures::evaluate(&Roots::x1_prod(), &[name(0, b"settings", Missing::Error), key(9, b"port", Missing::Error)]),
        Ok(Some(Value::Int(2222)))
    );
}
const UNKNOWN: &[bool] = &[false, true];
#[test]
fn an_unknown_name_fails_at_its_offset() {
    for defined in UNKNOWN {
        let mut operations = skein_lib::List::with_capacity(2);
        operations.push(name(7, b"nope", Missing::Error)).expect("room");
        if *defined {
            operations.push(op(12, Op::Defined)).expect("room");
        }
        assert_eq!(
            fixtures::evaluate(&Roots::x1_prod(), operations.as_slice()),
            Err(Fault { at: 7, kind: FaultKind::Unknown })
        );
    }
}
const WRONG: &[(&[u8], bool, Missing)] = &[
    (b"tags", false, Missing::Error),
    (b"port", true, Missing::Error),
    (b"peers", true, Missing::Undefined),
    (b"tags", false, Missing::Undefined),
];
#[test]
fn a_lookup_of_the_wrong_type_fails_even_in_a_chain() {
    for (field, index, missing) in WRONG {
        let mut ops = skein_lib::List::with_capacity(4);
        ops.push(name(0, b"settings", Missing::Error)).expect("room");
        ops.push(key(9, field, *missing)).expect("room");
        if *index {
            ops.push(op(14, Op::Integer(0))).expect("room");
            ops.push(op(13, Op::Item { missing: *missing })).expect("room");
        } else {
            ops.push(key(13, b"x", *missing)).expect("room");
        }
        assert_eq!(
            fixtures::evaluate(&Roots::x1_prod(), ops.as_slice()),
            Err(Fault { at: 13, kind: FaultKind::Type }),
            "{field:?} {missing:?}"
        );
    }
}
#[test]
fn a_missing_key_fails_at_the_keys_offset() {
    assert_eq!(
        fixtures::evaluate(
            &Roots::x1_prod(),
            &[name(0, b"settings", Missing::Error), key(9, b"absent", Missing::Error)]
        ),
        Err(Fault { at: 9, kind: FaultKind::NoKey })
    );
}
#[test]
fn a_key_named_like_a_method_is_only_a_key() {
    assert_eq!(
        fixtures::evaluate(&Roots::x1_prod(), &[name(0, b"settings", Missing::Error), key(9, b"keys", Missing::Error)]),
        Err(Fault { at: 9, kind: FaultKind::NoKey })
    );
}
const OPTIONAL: &[(&[u8], Missing)] = &[
    (b"settings", Missing::Error),
    (b"settings", Missing::Undefined),
    (b"tasks", Missing::Error),
    (b"tasks", Missing::Undefined),
];
#[test]
fn absent_optional_values_read_as_undefined_however_marked() {
    for (root, missing) in OPTIONAL {
        assert_eq!(
            fixtures::evaluate(&Roots::x1_prod(), &[name(0, root, Missing::Error), key(9, b"wifi", *missing)]),
            Ok(None),
            "{root:?} {missing:?}"
        );
    }
}
#[test]
fn an_undefined_list_element_fails_with_undefined() {
    assert_eq!(
        fixtures::evaluate(
            &Roots::x1_prod(),
            &[name(1, b"settings", Missing::Error), key(10, b"wifi", Missing::Error), op(0, Op::List { length: 1 })]
        ),
        Err(Fault { at: 0, kind: FaultKind::Undefined })
    );
}
const INDICES: &[i64] = &[2, -1];
#[test]
fn an_index_outside_the_list_fails_at_its_bracket() {
    for index in INDICES {
        assert_eq!(
            fixtures::evaluate(
                &Roots::x1_prod(),
                &[
                    name(0, b"settings", Missing::Error),
                    key(9, b"tags", Missing::Error),
                    op(14, Op::Integer(*index)),
                    op(13, Op::Item { missing: Missing::Error })
                ]
            ),
            Err(Fault { at: 13, kind: FaultKind::NoIndex }),
            "{index}"
        );
    }
}
#[test]
fn a_chain_into_is_defined_is_false_at_its_first_missing_link() {
    let operations = [
        name(0, b"settings", Missing::Error),
        key(9, b"peers", Missing::Undefined),
        key(15, b"c", Missing::Undefined),
        key(17, b"port", Missing::Undefined),
        op(22, Op::Defined),
    ];
    assert_eq!(fixtures::evaluate(&Roots::x1_prod(), &operations), Ok(Some(Value::Bool(false))));
}
#[test]
fn looking_into_undefined_outside_a_chain_fails() {
    assert_eq!(
        fixtures::evaluate(
            &Roots::x1_prod(),
            &[name(0, b"settings", Missing::Error), key(9, b"wifi", Missing::Error), key(14, b"ssid", Missing::Error)]
        ),
        Err(Fault { at: 14, kind: FaultKind::Undefined })
    );
}
#[test]
fn target_holds_metas_names_an_undefined_member_and_no_containers() {
    assert_eq!(
        crate::evaluate::context::target(&fixtures::meta()),
        Value::Map(Box::new([
            crate::evaluate::Entry { key: bytes::copy_of(b"base"), value: Some(Value::String(bytes::copy_of(b"x1"))) },
            crate::evaluate::Entry { key: bytes::copy_of(b"containers"), value: Some(Value::Map(Box::new([]))) },
            crate::evaluate::Entry {
                key: bytes::copy_of(b"environment"),
                value: Some(Value::String(bytes::copy_of(b"prod")))
            },
            crate::evaluate::Entry { key: bytes::copy_of(b"member"), value: None },
            crate::evaluate::Entry {
                key: bytes::copy_of(b"name"),
                value: Some(Value::String(bytes::copy_of(b"x1-prod")))
            },
            crate::evaluate::Entry {
                key: bytes::copy_of(b"placement"),
                value: Some(Value::Map(Box::new([
                    crate::evaluate::Entry {
                        key: bytes::copy_of(b"firmware"),
                        value: Some(Value::String(bytes::copy_of(b"efi")))
                    },
                    crate::evaluate::Entry {
                        key: bytes::copy_of(b"hardware"),
                        value: Some(Value::String(bytes::copy_of(b"7th-gen")))
                    },
                    crate::evaluate::Entry {
                        key: bytes::copy_of(b"kind"),
                        value: Some(Value::String(bytes::copy_of(b"metal")))
                    },
                ])))
            },
        ]))
    );
}
#[test]
fn an_absent_placement_field_is_no_key_not_undefined() {
    assert_eq!(
        fixtures::evaluate(
            &Roots::x1_prod(),
            &[name(0, b"target", Missing::Error), key(7, b"placement", Missing::Error), key(17, b"on", Missing::Error)]
        ),
        Err(Fault { at: 17, kind: FaultKind::NoKey })
    );
}
#[test]
fn an_absent_placement_field_in_a_chain_is_not_defined() {
    assert_eq!(
        fixtures::evaluate(
            &Roots::x1_prod(),
            &[
                name(0, b"target", Missing::Error),
                key(7, b"placement", Missing::Undefined),
                key(17, b"on", Missing::Undefined),
                op(20, Op::Defined)
            ]
        ),
        Ok(Some(Value::Bool(false)))
    );
}
#[test]
fn a_literal_past_the_budget_fails_with_limit() {
    assert_eq!(
        fixtures::evaluate_at(&Roots::x1_prod(), &[op(3, Op::String(bytes::copy_of(b"hello")))], &limits::FOUR_MADE),
        Err(Fault { at: 3, kind: FaultKind::Limit })
    );
}

#[test]
fn a_later_element_fault_precedes_an_undefined_list_element() {
    let operations = [
        name(1, b"settings", Missing::Error),
        key(10, b"wifi", Missing::Error),
        name(15, b"settings", Missing::Error),
        key(24, b"missing", Missing::Error),
        op(0, Op::List { length: 2 }),
    ];
    assert_eq!(fixtures::evaluate(&Roots::x1_prod(), &operations), Err(Fault { at: 24, kind: FaultKind::NoKey }));
}

#[test]
fn an_operand_costs_no_more_than_an_elements_charge() {
    assert!(u64::try_from(size_of::<crate::evaluate::Operand<'_>>()).expect("size fits") <= crate::evaluate::ELEMENT);
}
