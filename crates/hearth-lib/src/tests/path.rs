use crate::path::{path_valid, tree_order};
use core::cmp::Ordering;

#[test]
fn a_path_is_valid_by_3_4s_rules() {
    let longest = [b'a'; 4_096];
    let too_long = [b'a'; 4_097];
    let valid: &[&[u8]] = &[b"", b"a", b"a/b", b"a b", b".a", b"a.", b"..a", b"a..", b"\xc3\xa9", b"a-b", &longest];
    let invalid: &[&[u8]] =
        &[b"/", b"/a", b"a/", b"a//b", b".", b"..", b"a/.", b"a/..", b"./a", b"../a", b"a\0", b"\0", &too_long];
    for path in valid {
        assert!(path_valid(path), "{path:?}");
    }
    for path in invalid {
        assert!(!path_valid(path), "{path:?}");
    }
}

#[test]
fn tree_order_compares_component_by_component() {
    let rows: &[(&[u8], &[u8], Ordering)] = &[
        (b"", b"", Ordering::Equal),
        (b"", b"a", Ordering::Less),
        (b"a", b"a", Ordering::Equal),
        (b"a", b"a/b", Ordering::Less),
        (b"a/b", b"a-b", Ordering::Less),
        (b"a-b", b"b", Ordering::Less),
        (b"a", b"ab", Ordering::Less),
        (b"a/b", b"ab", Ordering::Less),
        (b"a/b/c", b"a/b", Ordering::Greater),
        (b"a/c", b"a/b/z", Ordering::Greater),
        (b"a b", b"a/b", Ordering::Greater),
        (b"a.b", b"a/b", Ordering::Greater),
        (b"b", b"a/z", Ordering::Greater),
        (b"a/", b"a", Ordering::Greater),
        (b"/", b"", Ordering::Greater),
    ];
    for (a, b, expected) in rows {
        assert_eq!(tree_order(a, b), *expected, "{a:?} against {b:?}");
        assert_eq!(tree_order(b, a), expected.reverse(), "{b:?} against {a:?}");
    }
}
