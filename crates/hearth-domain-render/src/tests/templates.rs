use super::fixtures::{
    self, Roots, advance, branch, enter, expr, jump, key, name, one, op, output, pair, set, template, text,
};
use crate::{
    evaluate::{Fault, FaultKind, Filter, Missing, Op, Value},
    limits::{Limits, SHIPPED},
};
use alloc::boxed::Box;
#[test]
fn a_loop_binds_each_element_in_turn() {
    let template = template(
        &[0],
        [
            enter(0, one(b"t"), expr(1, [name(12, b"settings", Missing::Error), key(21, b"tags", Missing::Error)]), 3),
            output(28, expr(1, [name(31, b"t", Missing::Error)])),
            text(36, b","),
            advance(37, 2),
        ],
    );
    assert_eq!(
        fixtures::run(&Roots::x1_prod(), &template),
        Ok(Box::from(&b"sudo,wheel,"[..])),
        "protocol/format.md, 7.3"
    );
}
#[test]
fn a_loop_over_a_map_takes_its_present_keys_in_order() {
    let template = template(
        &[0],
        [
            enter(0, one(b"t"), expr(1, [name(12, b"settings", Missing::Error)]), 2),
            output(28, expr(1, [name(31, b"t", Missing::Error)])),
            advance(37, 1),
        ],
    );
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Ok(Box::from(&b"enablednamepeersporttags"[..])));
}
#[test]
fn a_pair_loop_unpacks_two_element_lists() {
    let template = template(
        &[0],
        [
            enter(
                0,
                pair(b"k", b"v"),
                expr(
                    1,
                    [
                        name(12, b"settings", Missing::Error),
                        key(21, b"peers", Missing::Error),
                        op(27, Op::Filter(Filter::Items)),
                    ],
                ),
                3,
            ),
            output(28, expr(1, [name(31, b"k", Missing::Error)])),
            output(38, expr(1, [name(41, b"v", Missing::Error), key(43, b"port", Missing::Error)])),
            advance(48, 2),
        ],
    );
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Ok(Box::from(&b"a51820b51821"[..])));
}
const BAD_PAIRS: &[&[fixtures::Input]] =
    &[&[], &[fixtures::Input::Int(1)], &[fixtures::Input::Int(1), fixtures::Input::Int(2), fixtures::Input::Int(3)]];
#[test]
fn a_pair_loop_refuses_elements_without_exactly_two_values() {
    for (row, input) in BAD_PAIRS.iter().enumerate() {
        let mut roots = Roots::x1_prod();
        roots.settings =
            fixtures::list(&[fixtures::owned(fixtures::operand(fixtures::Input::List(input))).expect("list")]);
        let template = template(
            &[0],
            [enter(5, pair(b"k", b"v"), expr(1, [name(12, b"settings", Missing::Error)]), 1), advance(20, 0)],
        );
        assert_eq!(fixtures::run(&roots, &template), Err(Fault { at: 5, kind: FaultKind::Type }), "row {row}");
    }
}
const SCALAR_ITERABLES: &[fixtures::Input] = &[fixtures::Input::Int(7), fixtures::Input::Text(b"abc")];
#[test]
fn a_for_over_a_string_or_an_integer_is_a_type_fault() {
    for (row, input) in SCALAR_ITERABLES.iter().enumerate() {
        let mut roots = Roots::x1_prod();
        roots.settings = fixtures::owned(fixtures::operand(*input)).expect("defined scalar");
        let template =
            template(&[0], [enter(3, one(b"x"), expr(1, [name(12, b"settings", Missing::Error)]), 1), advance(20, 0)]);
        assert_eq!(fixtures::run(&roots, &template), Err(Fault { at: 3, kind: FaultKind::Type }), "row {row}");
    }
}
const BRANCHES: &[(bool, bool, &[u8])] = &[(true, true, b"if"), (false, true, b"elif"), (false, false, b"else")];
#[test]
fn elif_and_else_take_exactly_one_branch() {
    for (row, &(first, second, expected)) in BRANCHES.iter().enumerate() {
        let template = template(
            &[0],
            [
                branch(0, expr(1, [op(1, Op::Boolean(first))]), 2),
                text(5, b"if"),
                jump(7, 4),
                branch(9, expr(1, [op(10, Op::Boolean(second))]), 2),
                text(14, b"elif"),
                jump(18, 1),
                text(20, b"else"),
            ],
        );
        assert_eq!(fixtures::run(&Roots::x1_prod(), &template).as_deref(), Ok(expected), "row {row}");
    }
}
#[test]
fn loop_first_index0_and_length_are_their_literals() {
    let template = template(
        &[0],
        [
            enter(0, one(b"t"), expr(1, [name(12, b"settings", Missing::Error), key(21, b"tags", Missing::Error)]), 6),
            output(28, expr(1, [name(31, b"loop", Missing::Error), key(36, b"first", Missing::Error)])),
            output(44, expr(1, [name(47, b"loop", Missing::Error), key(52, b"index0", Missing::Error)])),
            output(60, expr(1, [name(63, b"loop", Missing::Error), key(68, b"length", Missing::Error)])),
            output(76, expr(1, [name(79, b"loop", Missing::Error), key(84, b"index", Missing::Error)])),
            output(92, expr(1, [name(95, b"loop", Missing::Error), key(100, b"last", Missing::Error)])),
            advance(108, 5),
        ],
    );
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Ok(Box::from(&b"true021falsefalse122true"[..])));
}
#[test]
fn loop_outside_a_for_is_unknown() {
    let template = template(&[0], [output(0, expr(1, [name(3, b"loop", Missing::Error)]))]);
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Err(Fault { at: 3, kind: FaultKind::Unknown }));
}
#[test]
fn loop_whole_is_a_true_condition_and_defined() {
    let template = template(
        &[0],
        [
            enter(0, one(b"t"), expr(1, [name(12, b"settings", Missing::Error), key(21, b"tags", Missing::Error)]), 4),
            branch(28, expr(1, [name(34, b"loop", Missing::Error)]), 1),
            text(40, b"yes"),
            output(43, expr(1, [name(46, b"loop", Missing::Undefined), op(51, Op::Defined)])),
            advance(62, 3),
        ],
    );
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Ok(Box::from(&b"yestrueyestrue"[..])));
}
#[test]
fn loop_whole_cannot_be_output() {
    let template = template(
        &[0],
        [
            enter(0, one(b"t"), expr(1, [name(12, b"settings", Missing::Error), key(21, b"tags", Missing::Error)]), 2),
            output(28, expr(1, [name(31, b"loop", Missing::Error)])),
            advance(40, 1),
        ],
    );
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Err(Fault { at: 28, kind: FaultKind::Output }));
}
const LOOP_OPERATORS: &[Op] =
    &[Op::Filter(Filter::Default), Op::Filter(Filter::Length), Op::Equal, Op::Concat, Op::Add];
#[test]
fn loop_whole_cannot_be_an_operator_or_filter_operand() {
    for (row, operation) in LOOP_OPERATORS.iter().enumerate() {
        let template = template(
            &[0],
            [
                enter(
                    0,
                    one(b"t"),
                    expr(1, [name(12, b"settings", Missing::Error), key(21, b"tags", Missing::Error)]),
                    2,
                ),
                output(
                    28,
                    expr(
                        2,
                        [
                            name(31, b"loop", Missing::Error),
                            name(36, b"loop", Missing::Error),
                            op(41, operation.clone()),
                        ],
                    ),
                ),
                advance(45, 1),
            ],
        );
        assert_eq!(
            fixtures::run(&Roots::x1_prod(), &template),
            Err(Fault { at: 41, kind: FaultKind::Type }),
            "row {row}"
        );
    }
}
const TREE_NAMES: &[&[u8]] = &[b"key", b"value"];
#[test]
fn a_tree_template_has_no_key_or_value() {
    for (row, input) in TREE_NAMES.iter().enumerate() {
        let template = template(&[0], [output(0, expr(1, [name(3, input, Missing::Error)]))]);
        assert_eq!(
            fixtures::run(&Roots::x1_prod(), &template),
            Err(Fault { at: 3, kind: FaultKind::Unknown }),
            "row {row}"
        );
    }
}
#[test]
fn set_lasts_one_iteration_in_a_loop_and_to_the_end_outside() {
    let template = template(
        &[0],
        [
            set(0, b"x", expr(1, [op(11, Op::Integer(7))])),
            enter(20, one(b"t"), expr(1, [name(32, b"settings", Missing::Error), key(41, b"tags", Missing::Error)]), 4),
            output(50, expr(1, [name(53, b"x", Missing::Error)])),
            set(60, b"x", expr(1, [op(71, Op::Integer(9))])),
            output(74, expr(1, [name(77, b"x", Missing::Error)])),
            advance(82, 3),
            output(94, expr(1, [name(97, b"x", Missing::Error)])),
        ],
    );
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Ok(Box::from(&b"79797"[..])));
}
#[test]
fn the_outer_binding_returns_after_a_nested_for_ends() {
    let template = template(
        &[0],
        [
            enter(0, one(b"t"), expr(1, [name(12, b"settings", Missing::Error), key(21, b"tags", Missing::Error)]), 5),
            enter(28, one(b"t"), expr(1, [op(40, Op::Integer(1)), op(41, Op::List { length: 1 })]), 2),
            output(44, expr(1, [name(47, b"t", Missing::Error)])),
            advance(50, 1),
            output(60, expr(1, [name(63, b"t", Missing::Error)])),
            advance(67, 4),
        ],
    );
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Ok(Box::from(&b"1sudo1wheel"[..])));
}
#[test]
fn an_undefined_set_value_fails_at_the_set() {
    let template = template(
        &[0],
        [set(5, b"x", expr(1, [name(16, b"settings", Missing::Undefined), key(25, b"wifi", Missing::Undefined)]))],
    );
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Err(Fault { at: 5, kind: FaultKind::Undefined }));
}
#[test]
fn an_undefined_condition_is_false() {
    let template = template(
        &[0],
        [
            branch(5, expr(1, [name(11, b"settings", Missing::Undefined), key(20, b"wifi", Missing::Undefined)]), 1),
            text(28, b"never"),
            text(44, b"done"),
        ],
    );
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Ok(Box::from(&b"done"[..])));
}
#[test]
fn an_undefined_iterable_fails() {
    let template = template(
        &[0],
        [
            enter(
                5,
                one(b"x"),
                expr(1, [name(17, b"settings", Missing::Undefined), key(26, b"wifi", Missing::Undefined)]),
                1,
            ),
            advance(34, 0),
        ],
    );
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Err(Fault { at: 5, kind: FaultKind::Undefined }));
}
#[test]
fn loop_iterations_past_the_limit_fail_at_the_for() {
    let mut roots = Roots::x1_prod();
    roots.settings = fixtures::list(&[Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);
    let template = template(
        &[0],
        [enter(5, one(b"x"), expr(1, [name(17, b"settings", Missing::Error)]), 2), text(28, b"x"), advance(29, 1)],
    );
    assert_eq!(
        fixtures::run_at(&roots, &template, &Limits { iterations: 3, ..SHIPPED }),
        Err(Fault { at: 5, kind: FaultKind::Limit })
    );
}
#[test]
fn a_file_past_its_limit_fails_at_the_instruction_that_passes_it() {
    let template = template(&[0], [text(0, b"four"), text(4, b"!")]);
    assert_eq!(
        fixtures::run_at(&Roots::x1_prod(), &template, &Limits { rendered_file: 4, ..SHIPPED }),
        Err(Fault { at: 4, kind: FaultKind::Limit })
    );
}

#[test]
fn a_loop_restores_the_made_budget_for_each_iteration() {
    let template = template(
        &[0],
        [
            enter(0, one(b"t"), expr(1, [name(12, b"settings", Missing::Error), key(21, b"tags", Missing::Error)]), 3),
            set(28, b"x", expr(1, [op(39, Op::String(b"abc".as_slice().into()))])),
            output(44, expr(1, [name(47, b"x", Missing::Error)])),
            advance(50, 2),
        ],
    );
    assert_eq!(
        fixtures::run_at(&Roots::x1_prod(), &template, &Limits { made: 6, ..SHIPPED }),
        Ok(Box::from(&b"abcabc"[..]))
    );
}
#[test]
fn loop_whole_cannot_be_set_to_another_name() {
    let template = template(
        &[0],
        [
            enter(0, one(b"t"), expr(1, [name(12, b"settings", Missing::Error), key(21, b"tags", Missing::Error)]), 2),
            set(28, b"x", expr(1, [name(39, b"loop", Missing::Error)])),
            advance(45, 1),
        ],
    );
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Err(Fault { at: 28, kind: FaultKind::Type }));
}

const LARGE_TEXT: &[u8] = &[b'x'; 8193];
#[test]
fn text_past_the_initial_buffer_grows_within_the_file_limit() {
    let template = template(&[0], [text(0, LARGE_TEXT)]);
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template).as_deref(), Ok(LARGE_TEXT));
}
const INTEGERS: &[(i64, &[u8])] =
    &[(i64::MIN, b"-9223372036854775808"), (-1, b"-1"), (0, b"0"), (i64::MAX, b"9223372036854775807")];
#[test]
fn integers_are_written_with_their_sign_and_decimal_digits() {
    for (row, &(value, expected)) in INTEGERS.iter().enumerate() {
        let template = template(&[0], [output(0, expr(1, [op(3, Op::Integer(value))]))]);
        assert_eq!(fixtures::run(&Roots::x1_prod(), &template).as_deref(), Ok(expected), "row {row}");
    }
}
#[test]
fn an_empty_loop_skips_its_body() {
    let mut roots = Roots::x1_prod();
    roots.settings = fixtures::list(&[]);
    let template = template(
        &[0],
        [
            enter(0, one(b"x"), expr(1, [name(12, b"settings", Missing::Error)]), 2),
            text(24, b"never"),
            advance(29, 1),
            text(40, b"after"),
        ],
    );
    assert_eq!(fixtures::run(&roots, &template), Ok(Box::from(&b"after"[..])));
}

#[test]
fn a_rendered_file_is_kept_before_the_next_run_starts() {
    use crate::vocabulary::{Event, Output, Request};
    let (mut render, env, mut out) = fixtures::keeping_templates();
    crate::resume(&mut render, &env, &mut out);
    let Some(Output::Request(Request::Keep { request, sha256, bytes, path })) = out.pop() else { panic!("first keep") };
    assert_eq!(path.as_ref(), b"tasks/motd/tree/etc/issue.j2");
    assert_eq!(bytes.as_ref(), b"abc");
    assert_eq!(sha256, fixtures::digest("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
    assert!(!render.work_pending());
    crate::resume(&mut render, &env, &mut out);
    assert_eq!(out.pop(), None);
    crate::step(&mut render, &env, Event::Kept { request, sha256 }, &mut out);
    crate::resume(&mut render, &env, &mut out);
    assert_eq!(
        out.pop(),
        Some(Output::Request(Request::Keep {
            request: skein_lib::Token::new(4),
            path: b"tasks/motd/tree/etc/motd.j2".as_slice().into(),
            sha256: fixtures::digest("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
            bytes: b"abc".as_slice().into(),
        }))
    );
}
#[test]
fn a_failed_keep_leaves_its_target_out_and_the_call_try_again() {
    use crate::vocabulary::{Answer, Class, Event, Failure, Output, Request};
    let (mut render, env, mut out) = fixtures::keeping_templates();
    crate::resume(&mut render, &env, &mut out);
    let Some(Output::Request(Request::Keep { request, .. })) = out.pop() else { panic!("keep") };
    crate::step(
        &mut render,
        &env,
        Event::Failed { request, class: Class::Operational, at: None, failure: Failure::Store },
        &mut out,
    );
    crate::resume(&mut render, &env, &mut out);
    let Some(Output::Request(Request::Keep { request, sha256, .. })) = out.pop() else { panic!("second keep") };
    crate::step(&mut render, &env, Event::Kept { request, sha256 }, &mut out);
    assert_eq!(
        fixtures::finish_render(&mut render, &env, &mut out).as_ref(),
        &[Answer::Rendered { outcome: crate::vocabulary::Outcome::TryAgain, diff: None, left_out: 0 }]
    );
}

#[test]
fn a_targets_rendered_bytes_past_the_limit_fail_its_template() {
    let answers = fixtures::target_limited_answers();
    let [crate::vocabulary::Answer::Error(error), crate::vocabulary::Answer::Rendered { .. }] = answers.as_ref() else {
        panic!("fault then outcome")
    };
    assert_eq!(error.kind, crate::errors::Kind::Evaluation(FaultKind::Limit));
    assert_eq!(
        error.at,
        Some(crate::errors::Place {
            path: b"tasks/motd/tree/etc/motd.j2".as_slice().into(),
            position: Some(crate::revision::Position { line: 1, column: 1 })
        })
    );
}
#[test]
fn a_target_past_its_rendered_bytes_is_left_out_and_fails_the_call() {
    let answers = fixtures::target_limited_answers();
    let [crate::vocabulary::Answer::Error(_), outcome] = answers.as_ref() else { panic!("no target spec") };
    assert_eq!(
        *outcome,
        crate::vocabulary::Answer::Rendered { outcome: crate::vocabulary::Outcome::Failed, diff: None, left_out: 0 }
    );
}
const OVERFLOWING_RUN_LIMITS: &[Limits] =
    &[Limits { made: u64::MAX, ..SHIPPED }, Limits { statements: u32::MAX, ..SHIPPED }];
#[test]
fn a_run_whose_worst_case_cannot_fit_returns_none() {
    for (row, limits) in OVERFLOWING_RUN_LIMITS.iter().enumerate() {
        assert_eq!(crate::evaluate::template::worst_case(limits), None, "row {row}");
    }
}
