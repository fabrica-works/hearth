use super::{
    fixtures::compile,
    operators::{View, observed},
};
use crate::{Error, ErrorKind, Expected, Missing, Op};
use Missing::Error as E;
use View::{Name as N, Other as O};
type Case = (&'static [u8], &'static [(u32, View<'static>)]);
fn check(cases: &[Case]) {
    for (text, want) in cases {
        let got = compile(text).expect("jumps compile");
        assert_eq!(observed(&got.operations).as_ref(), *want, "{text:?}");
    }
}
const AND: &[Case] = &[
    (b"a and b", &[(0, N(b"a", E)), (2, O(Op::JumpIfFalseKeeping { over: 1 })), (6, N(b"b", E))]),
    (
        b"a and b and c",
        &[
            (0, N(b"a", E)),
            (2, O(Op::JumpIfFalseKeeping { over: 1 })),
            (6, N(b"b", E)),
            (8, O(Op::JumpIfFalseKeeping { over: 1 })),
            (12, N(b"c", E)),
        ],
    ),
];
#[test]
fn and_keeps_a_false_left_operand_and_skips_the_right() {
    check(AND);
}
const OR: &[Case] = &[
    (b"a or b", &[(0, N(b"a", E)), (2, O(Op::JumpIfTrueKeeping { over: 1 })), (5, N(b"b", E))]),
    (
        b"a or b and not c",
        &[
            (0, N(b"a", E)),
            (2, O(Op::JumpIfTrueKeeping { over: 4 })),
            (5, N(b"b", E)),
            (7, O(Op::JumpIfFalseKeeping { over: 2 })),
            (15, N(b"c", E)),
            (11, O(Op::Not)),
        ],
    ),
];
#[test]
fn or_keeps_a_true_left_operand_and_skips_the_right() {
    check(OR);
}
const CONDITIONAL: &[Case] = &[(
    b"x if c else y",
    &[
        (5, N(b"c", E)),
        (2, O(Op::JumpIfFalse { over: 2 })),
        (0, N(b"x", E)),
        (7, O(Op::Jump { over: 1 })),
        (12, N(b"y", E)),
    ],
)];
#[test]
fn a_conditional_runs_its_condition_first_then_one_branch() {
    check(CONDITIONAL);
}
const NESTED: &[Case] = &[(
    b"a if b else c if d else e",
    &[
        (5, N(b"b", E)),
        (2, O(Op::JumpIfFalse { over: 2 })),
        (0, N(b"a", E)),
        (7, O(Op::Jump { over: 5 })),
        (17, N(b"d", E)),
        (14, O(Op::JumpIfFalse { over: 2 })),
        (12, N(b"c", E)),
        (19, O(Op::Jump { over: 1 })),
        (24, N(b"e", E)),
    ],
)];
#[test]
fn conditionals_nest_to_the_right_of_else() {
    check(NESTED);
}
const MOVED: &[Case] = &[
    (
        b"(a and b) if c or d else e",
        &[
            (13, N(b"c", E)),
            (15, O(Op::JumpIfTrueKeeping { over: 1 })),
            (18, N(b"d", E)),
            (10, O(Op::JumpIfFalse { over: 4 })),
            (1, N(b"a", E)),
            (3, O(Op::JumpIfFalseKeeping { over: 1 })),
            (7, N(b"b", E)),
            (20, O(Op::Jump { over: 1 })),
            (25, N(b"e", E)),
        ],
    ),
    (
        b"[p,x if c else y]",
        &[
            (1, N(b"p", E)),
            (8, N(b"c", E)),
            (5, O(Op::JumpIfFalse { over: 2 })),
            (3, N(b"x", E)),
            (10, O(Op::Jump { over: 1 })),
            (15, N(b"y", E)),
            (0, O(Op::List { length: 2 })),
        ],
    ),
    (
        b"p | replace(q,x if c else y)",
        &[
            (0, N(b"p", E)),
            (12, N(b"q", E)),
            (19, N(b"c", E)),
            (16, O(Op::JumpIfFalse { over: 2 })),
            (14, N(b"x", E)),
            (21, O(Op::Jump { over: 1 })),
            (26, N(b"y", E)),
            (4, O(Op::Filter(crate::Filter::Replace))),
        ],
    ),
];
#[test]
fn jumps_inside_a_moved_branch_keep_their_targets() {
    check(MOVED);
}
const MISSING: &[(&[u8], u32)] = &[(b"a if c", 6), (b"[a if c]", 7)];
#[test]
fn a_conditional_without_else_is_refused_at_the_end() {
    for (text, at) in MISSING {
        assert_eq!(compile(text), Err(Error { at: *at, kind: ErrorKind::Expected(Expected::Else) }), "{text:?}");
    }
}
const VALUES: &[(&[u8], u32)] =
    &[(b"x if c else y", 1), (b"[1,2] if [3,4,5] else [6,7,8,9]", 4), (b"a or [1,2]", 2), (b"[p,x if c else y]", 2)];
#[test]
fn values_counts_the_deepest_stack_on_any_path() {
    for (text, want) in VALUES {
        assert_eq!(compile(text).expect("expression compiles").values, *want, "{text:?}");
    }
}
const MISPLACED: &[(&[u8], u32)] = &[(b"a if b if c else d else e", 7), (b"a else b", 2)];
#[test]
fn a_conditional_condition_is_at_ors_level() {
    for (text, at) in MISPLACED {
        assert_eq!(compile(text), Err(Error { at: *at, kind: ErrorKind::Misplaced }), "{text:?}");
    }
}
