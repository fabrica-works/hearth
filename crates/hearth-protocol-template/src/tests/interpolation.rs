use super::fixtures::{bytes, expr, name, op, output, text};
use crate::{Error, ErrorKind, Expected, Interpolated, Op, interpolation, limits::SHIPPED};
const PLAIN: &[&[u8]] = &[b"", b"/home/user/.ssh", b"a\r\n{b} and ${value}"];
#[test]
fn a_string_without_tags_is_plain() {
    for source in PLAIN {
        assert_eq!(interpolation::compile(source, &SHIPPED), Ok(Interpolated::Plain));
    }
}
#[test]
fn one_output_alone_is_parts_of_one_output() {
    assert_eq!(
        interpolation::compile(b"{{ value }}", &SHIPPED),
        Ok(Interpolated::Parts([output(0, expr(1, [name(3, b"value")]))].into()))
    );
}
#[test]
fn text_and_outputs_alternate_in_order() {
    assert_eq!(
        interpolation::compile(b"/home/{{ key }}/.ssh", &SHIPPED),
        Ok(Interpolated::Parts([text(0, b"/home/"), output(6, expr(1, [name(9, b"key")])), text(15, b"/.ssh")].into()))
    );
}
const STATEMENTS: &[(&[u8], Error)] = &[
    (b"{% set x = 1 %}", Error { at: 0, kind: ErrorKind::Interpolation }),
    (b"a{% if x %}b{% endif %}", Error { at: 1, kind: ErrorKind::Interpolation }),
];
#[test]
fn a_statement_in_a_string_is_refused_at_its_brace() {
    for (source, expected) in STATEMENTS {
        assert_eq!(interpolation::compile(source, &SHIPPED).as_ref(), Err(expected));
    }
}
const FORBIDDEN: &[(&[u8], Error)] = &[
    (b"a{# note #}", Error { at: 1, kind: ErrorKind::Interpolation }),
    (b"{{- x }}", Error { at: 2, kind: ErrorKind::Interpolation }),
    (b"{{ x -}}", Error { at: 5, kind: ErrorKind::Interpolation }),
    (b"{{+ a }}", Error { at: 2, kind: ErrorKind::Expected(Expected::Operand) }),
    (b"{{ a +}}", Error { at: 6, kind: ErrorKind::Expected(Expected::Operand) }),
];
#[test]
fn a_comment_or_a_marker_in_a_string_is_refused() {
    for (source, expected) in FORBIDDEN {
        assert_eq!(interpolation::compile(source, &SHIPPED).as_ref(), Err(expected));
    }
}
#[test]
fn the_escape_writes_a_comment_opener_as_text() {
    assert_eq!(
        interpolation::compile(b"${{ '{#' }}a[@]}", &SHIPPED),
        Ok(Interpolated::Parts(
            [text(0, b"$"), output(1, expr(1, [op(4, Op::String(bytes(b"{#")))])), text(11, b"a[@]}")].into()
        ))
    );
}
const UNCLOSED: &[(&[u8], Error)] =
    &[(b"{{ x", Error { at: 0, kind: ErrorKind::Unclosed }), (b"a{{ x", Error { at: 1, kind: ErrorKind::Unclosed })];
#[test]
fn an_unclosed_output_in_a_string_is_refused_at_its_braces() {
    for (source, expected) in UNCLOSED {
        assert_eq!(interpolation::compile(source, &SHIPPED).as_ref(), Err(expected));
    }
}
