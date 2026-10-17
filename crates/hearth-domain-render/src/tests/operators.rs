use super::fixtures::{self, Input, Roots, code, key, name, op};
use crate::evaluate::{self, Fault, FaultKind, Missing, Op, Operand, Value, Written};
use skein_lib::bytes;
#[test]
fn and_keeps_a_false_left_operand_and_never_evaluates_the_right() {
    assert_eq!(
        fixtures::evaluate(
            &Roots::x1_prod(),
            &[op(0, Op::Integer(0)), op(2, Op::JumpIfFalseKeeping { over: 1 }), name(6, b"nope", Missing::Error)]
        ),
        Ok(Some(Value::Int(0)))
    );
}
const CONDITIONS: &[(bool, &[u8], &[u8])] = &[(true, b"settings", b"nope"), (false, b"nope", b"settings")];
#[test]
fn a_conditional_evaluates_its_condition_then_one_branch() {
    for (condition, yes, no) in CONDITIONS {
        let operations = [
            op(0, Op::Boolean(*condition)),
            op(1, Op::JumpIfFalse { over: 3 }),
            name(2, yes, Missing::Error),
            key(3, b"port", Missing::Error),
            op(4, Op::Jump { over: 2 }),
            name(5, no, Missing::Error),
            key(6, b"port", Missing::Error),
        ];
        assert_eq!(fixtures::evaluate(&Roots::x1_prod(), &operations), Ok(Some(Value::Int(2222))));
    }
}
const UNDEFINED_BRANCHES: &[bool] = &[true, false];
#[test]
fn an_undefined_branch_of_a_conditional_fails_with_undefined() {
    for condition in UNDEFINED_BRANCHES {
        let operations = [
            op(0, Op::Boolean(*condition)),
            op(1, Op::JumpIfFalse { over: 3 }),
            name(2, b"settings", Missing::Error),
            key(3, b"wifi", Missing::Error),
            op(4, Op::Jump { over: 2 }),
            name(5, b"settings", Missing::Error),
            key(6, b"wifi", Missing::Error),
            op(7, Op::Defined),
        ];
        assert_eq!(
            fixtures::evaluate(&Roots::x1_prod(), &operations),
            Err(Fault { at: 4, kind: FaultKind::Undefined })
        );
    }
}
#[test]
fn a_nested_false_branch_cannot_hide_an_undefined_outer_branch() {
    let operations = [
        op(0, Op::Boolean(false)),
        op(1, Op::JumpIfFalse { over: 2 }),
        op(2, Op::Integer(1)),
        op(3, Op::Jump { over: 6 }),
        op(4, Op::Boolean(false)),
        op(5, Op::JumpIfFalse { over: 2 }),
        op(6, Op::Integer(1)),
        op(7, Op::Jump { over: 2 }),
        name(8, b"settings", Missing::Error),
        key(9, b"wifi", Missing::Error),
        op(10, Op::Defined),
    ];
    assert_eq!(fixtures::evaluate(&Roots::x1_prod(), &operations), Err(Fault { at: 7, kind: FaultKind::Undefined }));
}
#[test]
fn or_gives_an_undefined_operand_where_both_are_undefined() {
    let operations = [
        name(0, b"settings", Missing::Error),
        key(9, b"wifi", Missing::Error),
        op(14, Op::JumpIfTrueKeeping { over: 2 }),
        name(17, b"tasks", Missing::Error),
        key(23, b"wifi", Missing::Error),
    ];
    assert_eq!(fixtures::evaluate(&Roots::x1_prod(), &operations), Ok(None));
}
const FALSES: &[Input] = &[Input::Bool(false), Input::Int(0), Input::Text(b""), Input::List(&[]), Input::Undefined];
#[test]
fn truthiness_is_false_for_the_six_values() {
    for value in FALSES {
        assert_eq!(evaluate::truthy(&fixtures::operand(*value)), fixtures::FALSE, "{value:?}");
    }
    assert_eq!(evaluate::truthy(&Operand::Value(&Value::Map(Box::new([])))), fixtures::FALSE);
}
#[test]
fn equality_is_deep_and_two_kinds_are_unequal() {
    let operations = code(&[
        Op::Integer(1),
        Op::Integer(2),
        Op::List { length: 1 },
        Op::List { length: 2 },
        Op::Integer(1),
        Op::Integer(2),
        Op::List { length: 1 },
        Op::List { length: 2 },
        Op::Equal,
    ]);
    assert_eq!(fixtures::evaluate(&Roots::x1_prod(), &operations), Ok(Some(Value::Bool(true))));
    assert_eq!(
        fixtures::evaluate(&Roots::x1_prod(), &code(&[Op::Integer(1), Op::String(bytes::copy_of(b"1")), Op::Equal])),
        Ok(Some(Value::Bool(false)))
    );
}
type OrderingCase = (&'static [u8], &'static [u8], Op, Result<Option<Value>, Fault>);
const ORDERING: &[OrderingCase] = &[
    (b"B", b"a", Op::Less, Ok(Some(Value::Bool(true)))),
    (b"b", b"a", Op::LessOrEqual, Ok(Some(Value::Bool(false)))),
    (b"a", b"a", Op::GreaterOrEqual, Ok(Some(Value::Bool(true)))),
];
#[test]
fn ordering_takes_two_integers_or_two_strings_by_bytes() {
    for (a, b, operator, expected) in ORDERING {
        assert_eq!(
            fixtures::evaluate(
                &Roots::x1_prod(),
                &code(&[Op::String(bytes::copy_of(a)), Op::String(bytes::copy_of(b)), operator.clone()])
            ),
            *expected
        );
    }
    assert_eq!(
        fixtures::evaluate(&Roots::x1_prod(), &code(&[Op::Integer(1), Op::String(bytes::copy_of(b"a")), Op::Less])),
        Err(Fault { at: 2, kind: FaultKind::Type })
    );
}
#[test]
fn in_finds_a_key_an_element_or_a_substring() {
    assert_eq!(
        fixtures::evaluate(
            &Roots::x1_prod(),
            &[op(0, Op::String(bytes::copy_of(b"port"))), name(1, b"settings", Missing::Error), op(2, Op::In)]
        ),
        Ok(Some(Value::Bool(true)))
    );
    assert_eq!(
        fixtures::evaluate(
            &Roots::x1_prod(),
            &code(&[Op::Integer(2), Op::Integer(1), Op::Integer(2), Op::List { length: 2 }, Op::In])
        ),
        Ok(Some(Value::Bool(true)))
    );
    assert_eq!(
        fixtures::evaluate(
            &Roots::x1_prod(),
            &code(&[Op::String(bytes::copy_of(b"")), Op::String(bytes::copy_of(b"abc")), Op::In])
        ),
        Ok(Some(Value::Bool(true)))
    );
}
const OVERFLOW: &[(i64, i64, Op)] =
    &[(i64::MAX, 1, Op::Add), (i64::MIN, 1, Op::Subtract), (i64::MIN, -1, Op::Add), (i64::MAX, -1, Op::Subtract)];
#[test]
fn sums_past_i64_fail_with_overflow_at_the_operator() {
    for (a, b, operator) in OVERFLOW {
        assert_eq!(
            fixtures::evaluate(&Roots::x1_prod(), &code(&[Op::Integer(*a), Op::Integer(*b), operator.clone()])),
            Err(Fault { at: 2, kind: FaultKind::Overflow })
        );
    }
}
#[test]
fn concat_writes_integers_in_decimal() {
    assert_eq!(
        fixtures::evaluate(
            &Roots::x1_prod(),
            &code(&[Op::String(bytes::copy_of(b"a")), Op::Integer(-12), Op::Concat, Op::Integer(3), Op::Concat])
        ),
        Ok(Some(Value::String(bytes::copy_of(b"a-123"))))
    );
}
const UNDEFINED_OPS: &[Op] = &[Op::Equal, Op::Concat];
#[test]
fn undefined_compared_or_joined_fails_with_undefined() {
    for operator in UNDEFINED_OPS {
        assert_eq!(
            fixtures::evaluate(
                &Roots::x1_prod(),
                &[
                    name(0, b"settings", Missing::Error),
                    key(1, b"wifi", Missing::Error),
                    op(2, Op::Integer(1)),
                    op(3, operator.clone())
                ]
            ),
            Err(Fault { at: 3, kind: FaultKind::Undefined })
        );
    }
}
#[test]
fn a_map_is_used_without_its_absent_entries() {
    let mut roots = Roots::x1_prod();
    roots.settings = fixtures::map(&[(b"wifi", None)]);
    roots.tasks = fixtures::map(&[]);
    assert_eq!(evaluate::truthy(&Operand::Value(&roots.settings)), fixtures::FALSE);
    assert_eq!(
        fixtures::evaluate(
            &roots,
            &[op(0, Op::String(bytes::copy_of(b"wifi"))), name(1, b"settings", Missing::Error), op(2, Op::In)]
        ),
        Ok(Some(Value::Bool(false)))
    );
    assert_eq!(
        fixtures::evaluate(
            &roots,
            &[name(0, b"settings", Missing::Error), name(1, b"tasks", Missing::Error), op(2, Op::Equal)]
        ),
        Ok(Some(Value::Bool(true)))
    );
}
const OUTPUTS: &[(Input, bool, &[u8])] = &[
    (Input::Bool(true), false, b"true"),
    (Input::Bool(false), false, b"false"),
    (Input::Int(-12), true, b"12"),
    (Input::Int(i64::MIN), true, b"9223372036854775808"),
];
#[test]
fn output_writes_booleans_as_words_and_integers_in_decimal() {
    for (input, negative, expected) in OUTPUTS {
        let input = fixtures::operand(*input);
        let (sign, bytes): (bool, Box<[u8]>) = match evaluate::output(&input).expect("scalar") {
            Written::Bytes(v) => (false, Box::from(v)),
            Written::Integer { negative, digits } => (negative, Box::from(digits.as_bytes())),
        };
        assert_eq!((sign, bytes.as_ref()), (*negative, *expected));
    }
}
const BAD_OUTPUT: &[(Input, FaultKind)] =
    &[(Input::Undefined, FaultKind::Undefined), (Input::List(&[]), FaultKind::Output)];
#[test]
fn output_refuses_lists_maps_and_undefined() {
    for (input, expected) in BAD_OUTPUT {
        assert_eq!(evaluate::output(&fixtures::operand(*input)), Err(*expected));
    }
    assert_eq!(evaluate::output(&Operand::Value(&Value::Map(Box::new([])))), Err(FaultKind::Output));
}

#[test]
fn a_context_list_equals_a_made_list_deeply() {
    let operations = [
        name(0, b"settings", Missing::Error),
        key(1, b"tags", Missing::Error),
        op(2, Op::String(bytes::copy_of(b"sudo"))),
        op(3, Op::String(bytes::copy_of(b"wheel"))),
        op(4, Op::List { length: 2 }),
        op(5, Op::Equal),
    ];
    assert_eq!(fixtures::evaluate(&Roots::x1_prod(), &operations), Ok(Some(Value::Bool(true))));
}
