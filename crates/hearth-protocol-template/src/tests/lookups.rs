use super::fixtures::{bytes, compile, item, key, name, op};
use crate::{
    Error, ErrorKind, Expected, Filter, Op, expression,
    limits::{Limits, SHIPPED},
};
#[test]
fn a_chain_of_keys_and_items_compiles_left_to_right() {
    let e = compile(b"a.b['c'][0]").expect("chain");
    assert_eq!(
        *e.operations,
        [name(0, b"a"), key(2, b"b"), op(4, Op::String(bytes(b"c"))), item(3), op(9, Op::Integer(0)), item(8)]
    );
    assert_eq!(e.values, 2);
}
#[test]
fn u_keys_is_always_the_key_keys() {
    for (text, expected) in [(b"u.keys".as_slice(), None), (b"u.keys | keys", Some(Filter::Keys))] {
        let e = compile(text).expect("key");
        match expected {
            None => assert_eq!(*e.operations, [name(0, b"u"), key(2, b"keys")]),
            Some(filter) => assert_eq!(*e.operations, [name(0, b"u"), key(2, b"keys"), op(9, Op::Filter(filter))]),
        }
    }
}
#[test]
fn a_list_literal_pops_its_elements_in_order() {
    let e = compile(b"[1, 'x', []]").expect("nested list");
    assert_eq!(
        *e.operations,
        [
            op(1, Op::Integer(1)),
            op(4, Op::String(bytes(b"x"))),
            op(9, Op::List { length: 0 }),
            op(0, Op::List { length: 3 })
        ]
    );
    assert_eq!(e.values, 3);
}
const FILTERS: &[(&[u8], Filter, u32)] = &[
    (b"x|default(1)", Filter::Default, 1),
    (b"x|join(1)", Filter::Join, 1),
    (b"x|split(1)", Filter::Split, 1),
    (b"x|first", Filter::First, 0),
    (b"x|last()", Filter::Last, 0),
    (b"x|length", Filter::Length, 0),
    (b"x|replace(1,2)", Filter::Replace, 2),
    (b"x|items", Filter::Items, 0),
    (b"x|keys", Filter::Keys, 0),
    (b"x|values", Filter::Values, 0),
    (b"x|sort", Filter::Sort, 0),
    (b"x|tojson", Filter::Tojson, 0),
    (b"x|shell_quote", Filter::ShellQuote, 0),
    (b"x|systemd_escape", Filter::SystemdEscape, 0),
    (b"x|systemd_escape_path", Filter::SystemdEscapePath, 0),
    (b"x|basename", Filter::Basename, 0),
    (b"x|dirname", Filter::Dirname, 0),
];
#[test]
fn each_filter_compiles_with_the_arguments_seven_five_gives() {
    for (text, filter, given) in FILTERS {
        let e = compile(text).expect("filter");
        match given {
            0 => assert_eq!(*e.operations, [name(0, b"x"), op(2, Op::Filter(*filter))]),
            1 => {
                let at = match filter {
                    Filter::Default => 10,
                    Filter::Join => 7,
                    Filter::Split => 8,
                    Filter::First
                    | Filter::Last
                    | Filter::Length
                    | Filter::Replace
                    | Filter::Items
                    | Filter::Keys
                    | Filter::Values
                    | Filter::Sort
                    | Filter::Tojson
                    | Filter::ShellQuote
                    | Filter::SystemdEscape
                    | Filter::SystemdEscapePath
                    | Filter::Basename
                    | Filter::Dirname => unreachable!("one argument rows"),
                };
                assert_eq!(*e.operations, [name(0, b"x"), op(at, Op::Integer(1)), op(2, Op::Filter(*filter))]);
            }
            2 => assert_eq!(
                *e.operations,
                [name(0, b"x"), op(10, Op::Integer(1)), op(12, Op::Integer(2)), op(2, Op::Filter(*filter))]
            ),
            _ => unreachable!("arity rows"),
        }
    }
}
#[test]
fn a_filter_given_another_count_is_refused_at_its_name() {
    for (text, filter, given) in [
        (b"a|join".as_slice(), Filter::Join, 0),
        (b"a|first(1)", Filter::First, 1),
        (b"a|replace(1)", Filter::Replace, 1),
    ] {
        assert_eq!(compile(text), Err(Error { at: 2, kind: ErrorKind::Arity { filter, given } }));
    }
}
#[test]
fn an_unknown_filter_is_refused_at_its_name() {
    assert_eq!(compile(b"x|unknown"), Err(Error { at: 2, kind: ErrorKind::UnknownFilter }));
}
/// One open level suffices to prove every nesting construct refuses a second.
const ONE: Limits = Limits { expression: SHIPPED.expression, nesting: 1 };
#[test]
fn nesting_past_the_limit_is_refused_where_the_level_opens() {
    for (text, at) in [(b"((1))".as_slice(), 1), (b"[[1]]", 1), (b"[a[1]]", 2), (b"[a|first()]", 8)] {
        assert_eq!(expression::compile(text, &ONE), Err(Error { at, kind: ErrorKind::TooDeep }));
    }
}
#[test]
fn calls_methods_slices_and_map_literals_are_refused_where_they_start() {
    for (text, at, kind) in [
        (b"range(3)".as_slice(), 5, ErrorKind::Expected(Expected::Operator)),
        (b"u.keys()", 6, ErrorKind::Expected(Expected::Operator)),
        (b"a[1:2]", 3, ErrorKind::Character),
        (b"{'a': 1}", 0, ErrorKind::Character),
    ] {
        assert_eq!(compile(text), Err(Error { at, kind }));
    }
}
#[test]
fn a_word_after_a_dot_is_a_key() {
    for (text, key) in
        [(b"a.and".as_slice(), b"and".as_slice()), (b"a.in", b"in"), (b"a.true", b"true"), (b"a.none", b"none")]
    {
        assert_eq!(*compile(text).expect("word key").operations, [name(0, b"a"), super::fixtures::key(2, key)]);
    }
}
#[test]
fn a_lookup_after_a_filter_is_refused_at_its_dot() {
    for (text, at) in [(b"a|first.x".as_slice(), 7), (b"a|first[0]", 7)] {
        assert_eq!(compile(text), Err(Error { at, kind: ErrorKind::Misplaced }));
    }
}
#[test]
fn parentheses_group_a_value_without_an_operation() {
    let e = compile(b"([1])[0]").expect("grouped list");
    assert_eq!(*e.operations, [op(2, Op::Integer(1)), op(1, Op::List { length: 1 }), op(6, Op::Integer(0)), item(5)]);
    assert_eq!(e.values, 2);
}
#[test]
fn each_open_construct_requires_its_matching_close_and_complete_elements() {
    for (text, at, expected) in [
        (b"(1".as_slice(), 2, Expected::CloseParen),
        (b"[1", 2, Expected::CloseBracket),
        (b"[1)", 2, Expected::CloseBracket),
        (b"[1,]", 3, Expected::Operand),
        (b"a[]", 2, Expected::Operand),
        (b"()", 1, Expected::Operand),
    ] {
        assert_eq!(compile(text), Err(Error { at, kind: ErrorKind::Expected(expected) }));
    }
}
