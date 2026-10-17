use super::fixtures::{bytes, compile, name, op};
use crate::{Error, ErrorKind, Expected, Op, limits::SHIPPED};
#[test]
fn a_name_compiles_to_one_lookup_of_that_name() {
    let e = compile(b" hello_2 ").expect("name");
    assert_eq!(*e.operations, [name(1, b"hello_2")]);
    assert_eq!(e.values, 1);
}
#[test]
fn a_string_literal_decodes_each_of_the_five_escapes() {
    let e = compile(br#"'a\\b\'c\"d\ne\tf'"#).expect("five escapes");
    assert_eq!(*e.operations, [op(0, Op::String(bytes(b"a\\b'c\"d\ne\tf")))]);
    assert_eq!(e.values, 1);
}
#[test]
fn an_escape_outside_the_five_is_refused_at_its_backslash() {
    assert_eq!(compile(br"'a\r'"), Err(Error { at: 2, kind: ErrorKind::Escape }));
}
#[test]
fn an_unterminated_string_is_refused_at_its_opening_quote() {
    for text in [b" 'a".as_slice(), b" 'a\\"] {
        assert_eq!(compile(text), Err(Error { at: 1, kind: ErrorKind::Unterminated }));
    }
}
#[test]
fn integer_literals_hold_the_range_of_i64_and_no_more() {
    for (text, n) in [(b"9223372036854775807".as_slice(), i64::MAX), (b"-9223372036854775808", i64::MIN)] {
        assert_eq!(*compile(text).expect("i64 end").operations, [op(0, Op::Integer(n))]);
    }
    for text in [b"9223372036854775808".as_slice(), b"-9223372036854775809"] {
        assert_eq!(compile(text), Err(Error { at: 0, kind: ErrorKind::Integer }));
    }
}
#[test]
fn leading_zeros_change_neither_an_integer_nor_its_range() {
    for (text, n) in
        [(b"007".as_slice(), 7), (b"-0", 0), (b"0000000000000000000000001", 1), (b"-0009223372036854775808", i64::MIN)]
    {
        assert_eq!(*compile(text).expect("padded i64").operations, [op(0, Op::Integer(n))]);
    }
    assert_eq!(compile(b"-0009223372036854775809"), Err(Error { at: 0, kind: ErrorKind::Integer }));
}
#[test]
fn a_byte_that_begins_no_token_is_refused_where_it_stands() {
    for text in [b"@".as_slice(), "é".as_bytes(), b"\r"] {
        assert_eq!(compile(text), Err(Error { at: 0, kind: ErrorKind::Character }));
    }
}
#[test]
fn a_word_is_no_operand() {
    const WORDS: &[(&[u8], u32)] = &[
        (b"and", 0),
        (b"or", 0),
        (b"not", 3),
        (b"in", 0),
        (b"is", 0),
        (b"if", 0),
        (b"else", 0),
        (b"none", 0),
        (b"None", 0),
        (b"True", 0),
        (b"False", 0),
    ];
    for (text, at) in WORDS {
        assert_eq!(compile(text), Err(Error { at: *at, kind: ErrorKind::Expected(Expected::Operand) }));
    }
    assert_eq!(*compile(b"true").expect("boolean").operations, [op(0, Op::Boolean(true))]);
    assert_eq!(*compile(b"false").expect("boolean").operations, [op(0, Op::Boolean(false))]);
}
#[test]
fn a_text_past_the_limit_is_refused_at_the_first_byte_past() {
    let text = [b'a'; 4097];
    assert_eq!(compile(&text), Err(Error { at: SHIPPED.expression, kind: ErrorKind::TooLong }));
}
