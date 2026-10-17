use super::fixtures::{expr, key, name, output, template, text};
use crate::{
    Does, Error, ErrorKind, Expected,
    limits::{Limits, SHIPPED},
};
#[test]
fn text_without_tags_compiles_to_one_text_as_written() {
    assert_eq!(*template(b"plain { }} %} -\n").expect("text").instructions, [text(0, b"plain { }} %} -\n")]);
}
#[test]
fn an_output_compiles_its_expression_at_offsets_in_the_template() {
    let compiled = template(b"Port {{ settings.port }}\n").expect("output");
    assert_eq!(
        *compiled.instructions,
        [text(0, b"Port "), output(5, expr(1, [name(8, b"settings"), key(17, b"port")])), text(24, b"\n")]
    );
    assert_eq!(&*compiled.lines, &[0, 25]);
}
const COMMENTS: &[(&[u8], u32, u32)] = &[(b"a{# {{ '\" #}b", 0, 12), (b"a{#-#}b", 0, 6)];
#[test]
fn a_comment_leaves_nothing_and_splits_the_text_around_it() {
    for &(input, first, second) in COMMENTS {
        assert_eq!(*template(input).expect("comment").instructions, [text(first, b"a"), text(second, b"b")]);
    }
}
const QUOTED: &[(&[u8], Result<(), Error>)] = &[(b"{{ '}}' }}", Ok(())), (b"{% if a == '%}' %}{% endif %}", Ok(()))];
#[test]
fn closing_delimiters_inside_a_string_literal_do_not_end_the_tag() {
    for &(input, ref want) in QUOTED {
        let got = match template(input) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
        assert_eq!(&got, want);
    }
}
const UNCLOSED: &[(&[u8], ErrorKind, u32)] = &[
    (b"a{{ x", ErrorKind::Unclosed, 1),
    (b"a{% if x", ErrorKind::Unclosed, 1),
    (b"a{# x", ErrorKind::Unclosed, 1),
    (b"{{ 'x", ErrorKind::Unterminated, 3),
];
#[test]
fn a_tag_without_its_end_is_refused_at_its_opening_brace() {
    for &(input, ref kind, at) in UNCLOSED {
        assert_eq!(template(input), Err(Error { at, kind: kind.clone() }));
    }
}
const WRONG_END: &[(&[u8], u32)] = &[(b"{{ a %}", 5), (b"{% if a }}", 8), (b"{{ a -%}", 6)];
#[test]
fn an_end_of_the_other_kind_is_refused_where_it_stands() {
    for &(input, at) in WRONG_END {
        assert_eq!(template(input), Err(Error { at, kind: ErrorKind::Expected(Expected::TagEnd) }));
    }
}
const RETURNS: &[(&[u8], u32)] = &[(b"a\rb", 1), (b"{{ 'a\rb' }}", 5), (b"{#\r#}", 2)];
#[test]
fn a_carriage_return_is_refused_wherever_it_stands() {
    for &(input, at) in RETURNS {
        assert_eq!(template(input), Err(Error { at, kind: ErrorKind::Return }));
    }
}
const UTF8: &[(&[u8], u32)] = &[(b"a\xff", 1), (b"\x80", 0), (b"\xe0\x80", 1), (b"\xf4\x90", 1), (b"a\xc2", 2)];
#[test]
fn bytes_that_are_not_utf8_are_refused_at_the_first() {
    for &(input, at) in UTF8 {
        assert_eq!(template(input), Err(Error { at, kind: ErrorKind::Utf8 }));
    }
}
const PLACES: &[(u32, (u32, u32))] = &[(0, (1, 1)), (1, (1, 2)), (2, (1, 3)), (3, (2, 1)), (4, (2, 2)), (5, (3, 1))];
#[test]
fn the_line_table_places_each_offset_at_its_line_and_column() {
    const LINES: &[u32] = &[0, 3, 5];
    assert_eq!(&*template(b"\xc3\xa9\nx\n").expect("utf8 text").lines, LINES);
    for &(at, want) in PLACES {
        assert_eq!(crate::template::place(b"\xc3\xa9\nx\n", at), want);
    }
}
#[test]
fn a_template_past_the_limit_is_refused_at_the_first_byte_past() {
    const LIMIT: Limits = Limits { template: 2, ..SHIPPED };
    const WANT: Error = Error { at: 2, kind: ErrorKind::TooLong };
    assert_eq!(crate::template::compile(b"abc", &LIMIT), Err(WANT));
}
#[test]
fn an_empty_template_emits_no_empty_text() {
    const WANT: &[crate::Instruction] = &[];
    assert_eq!(&*template(b"").expect("empty template").instructions, WANT);
}
#[test]
fn an_output_keeps_a_closing_delimiter_in_its_literal() {
    const WANT: &[u8] = b"}}";
    let compiled = template(b"{{ '}}' }}").expect("quoted delimiter");
    let Does::Output(e) = &compiled.instructions[0].does else { panic!("output") };
    assert_eq!(*e.operations, [super::fixtures::op(3, crate::Op::String(super::fixtures::bytes(WANT)))]);
}
