use super::fixtures::{layout, template};
use crate::{
    Does, Error, ErrorKind, Expected, Expression, Missing, Names, Op, Operation,
    limits::{Limits, SHIPPED},
};
#[test]
fn a_condition_branches_over_its_body_to_after_endif() {
    let compiled = template(b"{% if c %}A{% endif %}").expect("condition");
    assert_eq!(&*layout(&compiled), &[(0, "branch", 1), (10, "text", 0)]);
    assert_eq!(
        compiled.instructions[0].does,
        Does::Branch {
            condition: Expression {
                values: 1,
                operations: [Operation {
                    at: 6,
                    op: Op::Name { name: b"c".as_slice().into(), missing: Missing::Error }
                }]
                .into()
            },
            over: 1
        }
    );
}
#[test]
fn elif_and_else_each_end_the_branch_before_with_a_jump() {
    let compiled = template(b"{% if a %}x{% elif b %}y{% else %}z{% endif %}").expect("branches");
    assert_eq!(
        &*layout(&compiled),
        &[
            (0, "branch", 2),
            (10, "text", 0),
            (11, "jump", 4),
            (11, "branch", 2),
            (23, "text", 0),
            (24, "jump", 1),
            (34, "text", 0)
        ]
    );
}
#[test]
fn a_loop_enters_over_its_body_and_advances_back_to_it() {
    let compiled = template(b"{% for x in e %}B{% endfor %}").expect("loop");
    assert_eq!(&*layout(&compiled), &[(0, "enter", 2), (16, "text", 0), (17, "advance", 1)]);
    assert_eq!(
        compiled.instructions[0].does,
        Does::Enter {
            names: Names::One { name: b"x".as_slice().into() },
            iterable: Expression {
                values: 1,
                operations: [Operation {
                    at: 12,
                    op: Op::Name { name: b"e".as_slice().into(), missing: Missing::Error }
                }]
                .into()
            },
            over: 2
        }
    );
}
#[test]
fn a_loop_over_pairs_binds_both_names_in_order() {
    let compiled = template(b"{% for k, v in e %}{% endfor %}").expect("pair loop");
    let Does::Enter { names, .. } = &compiled.instructions[0].does else { panic!("enter") };
    assert_eq!(*names, Names::Pair { first: b"k".as_slice().into(), second: b"v".as_slice().into() });
}
#[test]
fn a_third_name_in_a_loop_is_refused_at_its_comma() {
    assert_eq!(
        template(b"{% for a, b, c in x | items %}"),
        Err(Error { at: 11, kind: ErrorKind::Expected(Expected::In) })
    );
}
#[test]
fn set_binds_one_name_to_its_expression() {
    let compiled = template(b"{% set x = e %}").expect("assignment");
    assert_eq!(
        compiled.instructions[0].does,
        Does::Set {
            name: b"x".as_slice().into(),
            value: Expression {
                values: 1,
                operations: [Operation {
                    at: 11,
                    op: Op::Name { name: b"e".as_slice().into(), missing: Missing::Error }
                }]
                .into()
            }
        }
    );
}
const EXCLUDED: &[(&[u8], ErrorKind, u32)] = &[
    (b"{% macro x %}", ErrorKind::UnknownStatement, 3),
    (b"{% call x %}", ErrorKind::UnknownStatement, 3),
    (b"{% include x %}", ErrorKind::UnknownStatement, 3),
    (b"{% import x %}", ErrorKind::UnknownStatement, 3),
    (b"{% from x %}", ErrorKind::UnknownStatement, 3),
    (b"{% extends x %}", ErrorKind::UnknownStatement, 3),
    (b"{% block x %}", ErrorKind::UnknownStatement, 3),
    (b"{% filter x %}", ErrorKind::UnknownStatement, 3),
    (b"{% with x %}", ErrorKind::UnknownStatement, 3),
    (b"{% do x %}", ErrorKind::UnknownStatement, 3),
    (b"{% break %}", ErrorKind::UnknownStatement, 3),
    (b"{% continue %}", ErrorKind::UnknownStatement, 3),
    (b"{% autoescape x %}", ErrorKind::UnknownStatement, 3),
    (b"{% for x in a %}{% else %}", ErrorKind::Unmatched, 19),
    (b"{% for x in a if c %}", ErrorKind::Misplaced, 14),
    (b"{% for x in a recursive %}", ErrorKind::Expected(Expected::Operator), 14),
    (b"{% set x %}", ErrorKind::Expected(Expected::Assign), 9),
    (b"{% set a, b = c %}", ErrorKind::Expected(Expected::Assign), 8),
];
#[test]
fn each_statement_seven_three_excludes_is_refused_at_the_offset_its_table_gives() {
    for &(input, ref kind, at) in EXCLUDED {
        assert_eq!(template(input), Err(Error { at, kind: kind.clone() }), "{input:?}");
    }
}
#[test]
fn a_conditional_after_in_must_be_parenthesised() {
    assert_eq!(template(b"{% for x in a if c else b %}"), Err(Error { at: 14, kind: ErrorKind::Misplaced }));
}
#[test]
fn a_parenthesised_conditional_after_in_compiles() {
    let compiled = template(b"{% for x in (a if c else b) %}{% endfor %}").expect("conditional iterable");
    assert_eq!(&*layout(&compiled), &[(0, "enter", 1), (30, "advance", 0)]);
}
const ROOTS: &[&[u8]] = &[b"target", b"debian", b"settings", b"tasks", b"exports", b"key", b"value", b"loop"];
#[test]
fn a_root_of_the_context_is_refused_as_a_name_bound() {
    for root in ROOTS {
        let input = super::fixtures::binding(b"{% set ", root, b" = 1 %}");
        assert_eq!(template(&input), Err(Error { at: 7, kind: ErrorKind::Shadows }));
        let input = super::fixtures::binding(b"{% for ", root, b" in x %}");
        assert_eq!(template(&input), Err(Error { at: 7, kind: ErrorKind::Shadows }));
    }
}
const UNMATCHED: &[(&[u8], u32)] = &[
    (b"{% endif %}", 3),
    (b"{% if a %}{% endfor %}", 13),
    (b"{% if a %}{% else %}{% elif b %}", 23),
    (b"{% for x in a %}{% endif %}", 19),
];
#[test]
fn a_close_no_open_block_takes_is_refused_at_its_word() {
    for &(input, at) in UNMATCHED {
        assert_eq!(template(input), Err(Error { at, kind: ErrorKind::Unmatched }));
    }
}
const UNENDED: &[(&[u8], u32)] = &[
    (b"{% if a %}", 0),
    (b"{% if a %}{% for x in a %}", 10),
    (b"{% if a %}{% elif b %}", 0),
    (b"{% if a %}{% else %}", 0),
];
#[test]
fn a_block_open_at_the_end_is_refused_at_its_tag() {
    for &(input, at) in UNENDED {
        assert_eq!(template(input), Err(Error { at, kind: ErrorKind::Unended }));
    }
}
const TOO_DEEP: &[(&[u8], u32)] = &[(b"{% if a %}{% if b %}", 10), (b"{% for x in a %}{% if b %}", 16)];
#[test]
fn statements_nested_past_the_limit_are_refused_where_the_level_opens() {
    const ONE: Limits = Limits { statements: 1, ..SHIPPED };
    for &(input, at) in TOO_DEEP {
        assert_eq!(crate::template::compile(input, &ONE), Err(Error { at, kind: ErrorKind::TooDeep }));
    }
}
const WORDS: &[&[u8]] =
    &[b"and", b"or", b"not", b"if", b"else", b"is", b"in", b"true", b"false", b"none", b"None", b"True", b"False"];
#[test]
fn grammar_words_are_refused_as_bound_names() {
    for word in WORDS {
        let input = super::fixtures::binding(b"{% set ", word, b" = 1 %}");
        assert_eq!(template(&input), Err(Error { at: 7, kind: ErrorKind::Expected(Expected::Name) }));
    }
}
const EXTRA: &[(&[u8], u32)] =
    &[(b"{% if a %}{% else x %}", 18), (b"{% if a %}{% endif x %}", 19), (b"{% for x in a %}{% endfor x %}", 26)];
#[test]
fn closing_statements_take_no_trailing_expression() {
    for &(input, at) in EXTRA {
        assert_eq!(template(input), Err(Error { at, kind: ErrorKind::Expected(Expected::TagEnd) }));
    }
}
