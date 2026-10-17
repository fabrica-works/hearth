use super::fixtures::{self, Roots, key, list, map, name, op, string};
use crate::evaluate::{Fault, FaultKind, Filter, Missing, Op, Value};
use alloc::boxed::Box;
use skein_lib::bytes;

const DEFAULTS: &[(&[u8], bool)] = &[(b"wifi", false), (b"enabled", true)];
#[test]
fn default_gives_its_argument_only_where_undefined() {
    for (field, expected) in DEFAULTS {
        let operations = [
            name(0, b"settings", Missing::Error),
            key(1, field, Missing::Error),
            op(2, Op::Boolean(false)),
            op(3, Op::Filter(Filter::Default)),
        ];
        assert_eq!(fixtures::evaluate(&Roots::x1_prod(), &operations), Ok(Some(Value::Bool(*expected))), "{field:?}");
    }
}
const UNDEFINED: &[Filter] = &[Filter::Length, Filter::Tojson, Filter::Join];
#[test]
fn a_filter_of_undefined_but_default_fails_with_undefined() {
    for filter in UNDEFINED {
        let mut operations = skein_lib::List::with_capacity(4);
        operations.push(name(0, b"settings", Missing::Error)).expect("room");
        operations.push(key(1, b"wifi", Missing::Error)).expect("room");
        if filter.arity() != 0 {
            operations.push(op(2, Op::String(bytes::copy_of(b",")))).expect("room");
        }
        operations.push(op(3, Op::Filter(*filter))).expect("room");
        assert_eq!(
            fixtures::evaluate(&Roots::x1_prod(), operations.as_slice()),
            Err(Fault { at: 3, kind: FaultKind::Undefined }),
            "{filter:?}"
        );
    }
}
type SplitCase = (&'static [u8], &'static [u8], &'static [&'static [u8]]);
const SPLIT: &[SplitCase] =
    &[(b":a::", b":", &[b"", b"a", b"", b""]), (b"", b":", &[b""]), (b"ab--cd", b"--", &[b"ab", b"cd"])];
#[test]
fn split_cuts_at_each_separator_keeping_empty_pieces() {
    for (text, separator, pieces) in SPLIT {
        let Some(Value::List(items)) =
            fixtures::filter(string(text), Filter::Split, &[string(separator)]).expect("split")
        else {
            panic!("a list")
        };
        assert_eq!(items.len(), pieces.len(), "{text:?}");
        for (item, piece) in items.iter().zip(*pieces) {
            assert_eq!(item, &Value::String(Box::from(*piece)));
        }
    }
}
const JOIN: &[(&[i64], &[u8], &[u8])] = &[(&[1, -2, 30], b":", b"1:-2:30"), (&[], b",", b"")];
#[test]
fn join_writes_strings_and_integers_between_separators() {
    for (integers, separator, expected) in JOIN {
        let mut values = skein_lib::List::with_capacity(u32::try_from(integers.len()).expect("small"));
        for integer in *integers {
            values.push(Value::Int(*integer)).expect("room");
        }
        assert_eq!(
            fixtures::filter(Value::List(values.into_boxed()), Filter::Join, &[string(separator)]),
            Ok(Some(Value::String(Box::from(*expected))))
        );
    }
    assert_eq!(
        fixtures::filter(list(&[string(b"x"), Value::Int(-12)]), Filter::Join, &[string(b"/")]),
        Ok(Some(Value::String(Box::from(b"x/-12".as_slice()))))
    );
}
type ReplaceCase = (&'static [u8], &'static [u8], &'static [u8], &'static [u8]);
const REPLACE: &[ReplaceCase] = &[(b"aaa", b"aa", b"b", b"ba"), (b"aaa", b"a", b"", b""), (b"x", b"q", b"z", b"x")];
#[test]
fn replace_replaces_every_occurrence_left_to_right() {
    for (text, pattern, replacement, expected) in REPLACE {
        assert_eq!(
            fixtures::filter(string(text), Filter::Replace, &[string(pattern), string(replacement)]),
            Ok(Some(Value::String(Box::from(*expected)))),
            "{text:?}"
        );
    }
}
const ENDS: &[Filter] = &[Filter::First, Filter::Last];
#[test]
fn first_or_last_of_an_empty_list_fails_with_empty() {
    for filter in ENDS {
        assert_eq!(fixtures::filter(list(&[]), *filter, &[]), Err(Fault { at: 3, kind: FaultKind::Empty }));
    }
}
const EMPTY_PATTERNS: &[(Filter, &[&[u8]])] = &[(Filter::Split, &[b""]), (Filter::Replace, &[b"", b"b"])];
#[test]
fn an_empty_split_separator_or_replace_pattern_fails_with_empty() {
    for (filter, arguments) in EMPTY_PATTERNS {
        let mut values = skein_lib::List::with_capacity(2);
        for argument in *arguments {
            values.push(string(argument)).expect("room");
        }
        assert_eq!(
            fixtures::filter(string(b"abc"), *filter, values.as_slice()),
            Err(Fault { at: 3, kind: FaultKind::Empty })
        );
    }
}
const LENGTHS: &[(&[u8], i64)] = &[(b"", 0), (b"\xc3\xa9", 1), (b"a\xf0\x9f\x98\x80", 2)];
#[test]
fn length_counts_code_points_elements_or_present_entries() {
    for (text, length) in LENGTHS {
        assert_eq!(fixtures::filter(string(text), Filter::Length, &[]), Ok(Some(Value::Int(*length))));
    }
    assert_eq!(
        fixtures::filter(list(&[Value::Int(1), Value::Bool(false)]), Filter::Length, &[]),
        Ok(Some(Value::Int(2)))
    );
    assert_eq!(
        fixtures::filter(map(&[(b"a", Some(Value::Int(1))), (b"b", None)]), Filter::Length, &[]),
        Ok(Some(Value::Int(1)))
    );
}
const MAP_FILTERS: &[(Filter, &[u8])] =
    &[(Filter::Items, br#"[["a",1],["c",3]]"#), (Filter::Keys, br#"["a","c"]"#), (Filter::Values, b"[1,3]")];
#[test]
fn items_keys_and_values_follow_the_keys_byte_order() {
    for (filter, expected) in MAP_FILTERS {
        let mut roots = Roots::x1_prod();
        roots.settings = map(&[(b"a", Some(Value::Int(1))), (b"b", None), (b"c", Some(Value::Int(3)))]);
        let operations =
            [name(0, b"settings", Missing::Error), op(1, Op::Filter(*filter)), op(2, Op::Filter(Filter::Tojson))];
        assert_eq!(
            fixtures::evaluate(&roots, &operations),
            Ok(Some(Value::String(Box::from(*expected)))),
            "{filter:?}"
        );
    }
}
type SortCase = (&'static [&'static [u8]], &'static [&'static [u8]]);
const SORT: &[SortCase] = &[(&[b"b", b"a", b"b"], &[b"a", b"b", b"b"]), (&[b"a", b"B"], &[b"B", b"a"]), (&[], &[])];
#[test]
fn sort_orders_by_bytes_and_refuses_a_mixed_list() {
    for (input, expected) in SORT {
        let mut values = skein_lib::List::with_capacity(u32::try_from(input.len()).expect("small"));
        for value in *input {
            values.push(string(value)).expect("room");
        }
        let Some(Value::List(items)) =
            fixtures::filter(Value::List(values.into_boxed()), Filter::Sort, &[]).expect("sort")
        else {
            panic!("a list")
        };
        assert_eq!(items.len(), expected.len());
        for (item, expected) in items.iter().zip(*expected) {
            assert_eq!(item, &Value::String(Box::from(*expected)));
        }
    }
}
#[test]
fn sort_refuses_a_mixed_list() {
    assert_eq!(
        fixtures::filter(list(&[string(b"a"), Value::Int(1)]), Filter::Sort, &[]),
        Err(Fault { at: 3, kind: FaultKind::Type })
    );
}
#[test]
fn tojson_is_compact_with_keys_in_order_and_controls_escaped() {
    let roots = Roots::x1_prod();
    let peers =
        [name(0, b"settings", Missing::Error), key(9, b"peers", Missing::Error), op(17, Op::Filter(Filter::Tojson))];
    assert_eq!(
        fixtures::evaluate(&roots, &peers),
        Ok(Some(Value::String(Box::from(br#"{"a":{"port":51820},"b":{"port":51821}}"#.as_slice()))))
    );
    let text = [op(0, Op::String(bytes::copy_of(b"\"\\\n\x01</>\xc3\xa9"))), op(9, Op::Filter(Filter::Tojson))];
    assert_eq!(
        fixtures::evaluate(&roots, &text),
        Ok(Some(Value::String(Box::from(b"\"\\\"\\\\\\n\\u0001</>\xc3\xa9\"".as_slice()))))
    );
}
const SHELL: &[(&[u8], &[u8])] = &[(b"", b"''"), (b"a'b'", b"'a'\\''b'\\'''"), (b"x y", b"'x y'")];
#[test]
fn shell_quote_wraps_in_single_quotes_and_rewrites_each_quote() {
    for (text, expected) in SHELL {
        assert_eq!(
            fixtures::filter(string(text), Filter::ShellQuote, &[]),
            Ok(Some(Value::String(Box::from(*expected))))
        );
    }
}
const SYSTEMD: &[(Filter, &[u8], &[u8])] = &[
    (Filter::SystemdEscape, b".x", br"\x2ex"),
    (Filter::SystemdEscape, b"a-b", br"a\x2db"),
    (Filter::SystemdEscape, b"a/b c", br"a-b\x20c"),
    (Filter::SystemdEscapePath, b"/", b"-"),
    (Filter::SystemdEscapePath, b"/dev/nvme0n1p3", b"dev-nvme0n1p3"),
];
#[test]
fn the_systemd_filters_match_systemd_escape_261() {
    for (filter, text, expected) in SYSTEMD {
        assert_eq!(
            fixtures::filter(string(text), *filter, &[]),
            Ok(Some(Value::String(Box::from(*expected)))),
            "{text:?}"
        );
    }
}
const BAD_PATHS: &[(&[u8], Filter)] = &[
    (b"/a//b/", Filter::SystemdEscapePath),
    (b"/a/./b", Filter::SystemdEscapePath),
    (b"/a/../b", Filter::SystemdEscapePath),
    (b"a/b", Filter::SystemdEscapePath),
    (b"", Filter::SystemdEscapePath),
    (b"", Filter::Basename),
    (b"a/", Filter::Basename),
    (b"/a/../b", Filter::Basename),
    (b"", Filter::Dirname),
    (b"a/", Filter::Dirname),
    (b"/a/../b", Filter::Dirname),
];
#[test]
fn path_filters_refuse_values_outside_six_twos_path_forms() {
    for (text, filter) in BAD_PATHS {
        assert_eq!(
            fixtures::filter(string(text), *filter, &[]),
            Err(Fault { at: 3, kind: FaultKind::Path }),
            "{text:?}"
        );
    }
}
const PATHS: &[(&[u8], &[u8], &[u8])] =
    &[(b"/", b"/", b"/"), (b"/a", b"a", b"/"), (b"a", b"a", b"."), (b"a/b", b"b", b"a")];
#[test]
fn basename_and_dirname_give_7_5s_edges() {
    for (text, basename, dirname) in PATHS {
        assert_eq!(
            fixtures::filter(string(text), Filter::Basename, &[]),
            Ok(Some(Value::String(Box::from(*basename))))
        );
        assert_eq!(fixtures::filter(string(text), Filter::Dirname, &[]), Ok(Some(Value::String(Box::from(*dirname)))));
    }
}
