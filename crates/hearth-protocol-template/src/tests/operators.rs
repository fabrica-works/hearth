use super::fixtures::compile;
use crate::{Error, ErrorKind, Missing, Op, Operation};
use Missing::{Error as E, Undefined as U};
use View::{Key as K, Name as N, Other as O};

// Borrow boxed payloads so expected operations remain immutable literal const tables.
#[derive(Debug, PartialEq, Eq)]
enum View<'a> {
    Name(&'a [u8], Missing),
    Key(&'a [u8], Missing),
    String(&'a [u8]),
    Other(Op),
}
fn observed(operations: &[Operation]) -> Box<[(u32, View<'_>)]> {
    let mut out = skein_lib::List::with_capacity(u32::try_from(operations.len()).expect("operations fit text"));
    for operation in operations {
        let view = match &operation.op {
            Op::Name { name, missing } => View::Name(name, *missing),
            Op::Key { name, missing } => View::Key(name, *missing),
            Op::String(text) => View::String(text),
            Op::Item { .. }
            | Op::Integer(_)
            | Op::Boolean(_)
            | Op::List { .. }
            | Op::Filter(_)
            | Op::Not
            | Op::Equal
            | Op::NotEqual
            | Op::Less
            | Op::LessOrEqual
            | Op::Greater
            | Op::GreaterOrEqual
            | Op::In
            | Op::NotIn
            | Op::Concat
            | Op::Add
            | Op::Subtract
            | Op::Defined
            | Op::NotDefined => View::Other(operation.op.clone()),
        };
        out.push((operation.at, view)).expect("one observed view per operation");
    }
    out.into_boxed()
}
type Case = (&'static [u8], &'static [(u32, View<'static>)]);
fn check(cases: &[Case]) {
    for (text, want) in cases {
        let got = compile(text).expect("the expression compiles");
        assert_eq!(observed(&got.operations).as_ref(), *want, "{text:?}");
    }
}
const PRECEDENCE: &[Case] = &[
    (b"a ~ b + c", &[(0, N(b"a", E)), (4, N(b"b", E)), (8, N(b"c", E)), (6, O(Op::Add)), (2, O(Op::Concat))]),
    (b"not a == b", &[(4, N(b"a", E)), (9, N(b"b", E)), (6, O(Op::Equal)), (0, O(Op::Not))]),
    (
        b"a + b | length",
        &[(0, N(b"a", E)), (4, N(b"b", E)), (8, O(Op::Filter(crate::Filter::Length))), (2, O(Op::Add))],
    ),
    (
        b"'x' in a ~ b",
        &[(0, View::String(b"x")), (7, N(b"a", E)), (11, N(b"b", E)), (9, O(Op::Concat)), (4, O(Op::In))],
    ),
    (b"not a is defined", &[(4, N(b"a", E)), (6, O(Op::Defined)), (0, O(Op::Not))]),
];
#[test]
fn operators_bind_as_seven_four_s_grammar_nests_them() {
    check(PRECEDENCE);
}
const ASSOCIATION: &[Case] = &[
    (b"a+b-c", &[(0, N(b"a", E)), (2, N(b"b", E)), (1, O(Op::Add)), (4, N(b"c", E)), (3, O(Op::Subtract))]),
    (b"a~b~c", &[(0, N(b"a", E)), (2, N(b"b", E)), (1, O(Op::Concat)), (4, N(b"c", E)), (3, O(Op::Concat))]),
    (b"a-1", &[(0, N(b"a", E)), (2, O(Op::Integer(1))), (1, O(Op::Subtract))]),
];
#[test]
fn sums_and_joins_associate_to_the_left() {
    check(ASSOCIATION);
}
const COMPARISONS: &[Case] = &[
    (b"a == b", &[(0, N(b"a", E)), (5, N(b"b", E)), (2, O(Op::Equal))]),
    (b"a != b", &[(0, N(b"a", E)), (5, N(b"b", E)), (2, O(Op::NotEqual))]),
    (b"a < b", &[(0, N(b"a", E)), (4, N(b"b", E)), (2, O(Op::Less))]),
    (b"a <= b", &[(0, N(b"a", E)), (5, N(b"b", E)), (2, O(Op::LessOrEqual))]),
    (b"a > b", &[(0, N(b"a", E)), (4, N(b"b", E)), (2, O(Op::Greater))]),
    (b"a >= b", &[(0, N(b"a", E)), (5, N(b"b", E)), (2, O(Op::GreaterOrEqual))]),
    (b"a in b", &[(0, N(b"a", E)), (5, N(b"b", E)), (2, O(Op::In))]),
    (b"a not in b", &[(0, N(b"a", E)), (9, N(b"b", E)), (2, O(Op::NotIn))]),
];
#[test]
fn comparisons_compile_with_their_own_operation() {
    check(COMPARISONS);
}
const SECOND: &[(&[u8], u32)] =
    &[(b"a < b < c", 6), (b"a is defined is defined", 13), (b"a == not b", 5), (b"a is defined + b", 13)];
#[test]
fn a_second_comparison_is_refused_at_its_operator() {
    for (text, at) in SECOND {
        assert_eq!(compile(text), Err(Error { at: *at, kind: ErrorKind::Misplaced }), "{text:?}");
    }
}
const UNKNOWN: &[(&[u8], u32)] = &[(b"a is odd", 5), (b"a is not odd", 9)];
#[test]
fn a_test_other_than_defined_is_refused_at_its_name() {
    for (text, at) in UNKNOWN {
        assert_eq!(compile(text), Err(Error { at: *at, kind: ErrorKind::UnknownTest }), "{text:?}");
    }
}
const DEFINED: &[Case] = &[
    (b"peer.endpoint is defined", &[(0, N(b"peer", E)), (5, K(b"endpoint", U)), (14, O(Op::Defined))]),
    (
        b"a.b[0].c is not defined",
        &[
            (0, N(b"a", E)),
            (2, K(b"b", U)),
            (4, O(Op::Integer(0))),
            (3, O(Op::Item { missing: U })),
            (7, K(b"c", U)),
            (9, O(Op::NotDefined)),
        ],
    ),
];
#[test]
fn a_chain_into_is_defined_gives_undefined_at_each_link() {
    check(DEFINED);
}
const DEFAULT: &[Case] = &[
    (
        b"a.b | default('x')",
        &[(0, N(b"a", E)), (2, K(b"b", U)), (14, View::String(b"x")), (6, O(Op::Filter(crate::Filter::Default)))],
    ),
    (
        b"a.b[0].c | default(1)",
        &[
            (0, N(b"a", E)),
            (2, K(b"b", U)),
            (4, O(Op::Integer(0))),
            (3, O(Op::Item { missing: U })),
            (7, K(b"c", U)),
            (19, O(Op::Integer(1))),
            (11, O(Op::Filter(crate::Filter::Default))),
        ],
    ),
];
#[test]
fn a_chain_into_default_gives_undefined_at_each_link() {
    check(DEFAULT);
}
const ORDINARY: &[Case] = &[
    (
        b"a.b | length is defined",
        &[(0, N(b"a", E)), (2, K(b"b", E)), (6, O(Op::Filter(crate::Filter::Length))), (13, O(Op::Defined))],
    ),
    (
        b"a ~ b.c is defined",
        &[(0, N(b"a", E)), (4, N(b"b", E)), (6, K(b"c", E)), (2, O(Op::Concat)), (8, O(Op::Defined))],
    ),
    (
        b"x[a.b] is defined",
        &[(0, N(b"x", E)), (2, N(b"a", E)), (4, K(b"b", E)), (1, O(Op::Item { missing: U })), (7, O(Op::Defined))],
    ),
];
#[test]
fn a_lookup_used_otherwise_fails_when_it_finds_nothing() {
    check(ORDINARY);
}
const ROOTS: &[Case] = &[
    (b"wifi is defined", &[(0, N(b"wifi", E)), (5, O(Op::Defined))]),
    (
        b"wifi | default('x')",
        &[(0, N(b"wifi", E)), (15, View::String(b"x")), (7, O(Op::Filter(crate::Filter::Default)))],
    ),
];
#[test]
fn a_name_is_no_link_whatever_follows_it() {
    check(ROOTS);
}
const NONCHAINS: &[Case] = &[
    (b"(peer.endpoint) is defined", &[(1, N(b"peer", E)), (6, K(b"endpoint", E)), (16, O(Op::Defined))]),
    (
        b"[1][5] is defined",
        &[
            (1, O(Op::Integer(1))),
            (0, O(Op::List { length: 1 })),
            (4, O(Op::Integer(5))),
            (3, O(Op::Item { missing: E })),
            (7, O(Op::Defined)),
        ],
    ),
];
#[test]
fn a_parenthesized_or_literal_chain_marks_nothing() {
    check(NONCHAINS);
}
