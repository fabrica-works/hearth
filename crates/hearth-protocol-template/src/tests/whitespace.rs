use super::fixtures::{branch, expr, name, template, text, text_bytes};
use crate::{Error, ErrorKind, Expected, template::whitespace::WHITE_SPACE};

type Case = (&'static [u8], &'static [u8]);
const TRIM: &[Case] = &[
    (b"a{% set x = 1 %}\n\nb", b"a\nb"),
    (b"a{# c #}\n\nb", b"a\nb"),
    (b"a{{ '' }}\n\nb", b"a\n\nb"),
    (b"a{#-#}\nb", b"ab"),
];
const LSTRIP: &[Case] = &[
    (b"a\n  {# c #}b", b"a\nb"),
    (b"  {# c #}b", b"b"),
    (b"a  {# c #}b", b"a  b"),
    (b"a\n  {{ '' }}b", b"a\n  b"),
    (b"a\n\x0b\x0c\xc2\xa0\xe3\x80\x80{# c #}b", b"a\nb"),
];
const MINUS: &[Case] = &[
    (b"a \xc2\xa0\xe3\x80\x80\x0b\n{{- '' }}b", b"ab"),
    (b"a{{ '' -}} \xc2\xa0\xe3\x80\x80\x0b\nb", b"ab"),
    (b"a\n {#- c -#} \n b", b"ab"),
    (b"a\n {%- set x = 1 -%} \n b", b"ab"),
];
const PLUS: &[Case] = &[
    (b"  {#+ c #}b", b"  b"),
    (b"a{# c +#}\nb", b"a\nb"),
    (b"  {%+ set x = 1 %}b", b"  b"),
    (b"menu{% if true %} x{% endif +%}\n", b"menu x\n"),
];
const TRAILING: &[Case] = &[(b"a\n", b"a\n"), (b"a{{ '' }}\n", b"a\n"), (b"a{# c #}\n", b"a")];
const RAW: &[Case] = &[
    (b"{% raw %}{{ x }}{% endif %}{%endraw{% endraw %}", b"{{ x }}{% endif %}{%endraw"),
    (b"a{% raw %}\n  x\n  {% endraw %}\nb", b"a  x\nb"),
    (b"a{% raw -%} \n x \n {%- endraw -%} \n b", b"axb"),
    (b"a{% raw +%}\n  x\n  {%+ endraw +%}\nb", b"a\n  x\n  \nb"),
    (b"x{% raw %}  {% endraw %}z", b"xz"),
    (b"x{% raw %}{% endraw %}z", b"xz"),
    (b"{%\x0craw\x0c%}x{%\x0cendraw\x0c%}", b"x"),
    (b"{% raw %}x{%\xc2\xa0endraw%}y{% endraw %}", b"x{%\xc2\xa0endraw%}y"),
];
#[test]
fn a_statement_alone_on_its_line_leaves_nothing_behind() {
    let compiled = template(b"a\n  {% if x %}\n  b\n  {% endif %}\nc\n").expect("a condition compiles");
    assert_eq!(
        *compiled.instructions,
        [text(0, b"a\n"), branch(4, expr(1, [name(10, b"x")]), 1), text(15, b"  b\n"), text(33, b"c\n")]
    );
}
#[test]
fn trim_blocks_removes_one_newline_after_statements_and_comments_only() {
    for &(source, expected) in TRIM {
        assert_eq!(&*text_bytes(&template(source).expect("tag compiles")), expected, "{source:?}");
    }
}
#[test]
fn lstrip_blocks_removes_indentation_only_at_the_start_of_a_line() {
    for &(source, expected) in LSTRIP {
        assert_eq!(&*text_bytes(&template(source).expect("tag compiles")), expected, "{source:?}");
    }
}
#[test]
fn a_minus_marker_strips_all_whitespace_on_its_side() {
    for &(source, expected) in MINUS {
        assert_eq!(&*text_bytes(&template(source).expect("tag compiles")), expected, "{source:?}");
    }
}
#[test]
fn a_plus_marker_keeps_what_trim_and_lstrip_would_remove() {
    for &(source, expected) in PLUS {
        assert_eq!(&*text_bytes(&template(source).expect("tag compiles")), expected, "{source:?}");
    }
}
#[test]
fn trailing_newlines_are_kept_as_written() {
    for &(source, expected) in TRAILING {
        assert_eq!(&*text_bytes(&template(source).expect("tag compiles")), expected, "{source:?}");
    }
}
const OUTPUT_PLUS: &[(&[u8], Error)] = &[
    (b"{{+ a }}", Error { at: 2, kind: ErrorKind::Expected(Expected::Operand) }),
    (b"{{ a +}}", Error { at: 6, kind: ErrorKind::Expected(Expected::Operand) }),
];
#[test]
fn a_plus_beside_an_output_tag_is_an_error() {
    for (source, expected) in OUTPUT_PLUS {
        assert_eq!(template(source).as_ref(), Err(expected), "{source:?}");
    }
}
#[test]
fn raw_outputs_its_body_with_its_tags_whitespace_rules() {
    for &(source, expected) in RAW {
        assert_eq!(&*text_bytes(&template(source).expect("raw compiles")), expected, "{source:?}");
    }
}
const UNCLOSED_RAW: &[(&[u8], Error)] = &[
    (b"{% raw %}{{ x }}", Error { at: 0, kind: ErrorKind::Unclosed }),
    (b"a{% raw %}x{%\x0bendraw%}", Error { at: 1, kind: ErrorKind::Unclosed }),
];
#[test]
fn a_raw_without_endraw_is_refused_at_its_tag() {
    for (source, expected) in UNCLOSED_RAW {
        assert_eq!(template(source).as_ref(), Err(expected), "{source:?}");
    }
}
#[test]
fn the_table_holds_the_25_white_space_code_points() {
    assert_eq!(WHITE_SPACE.len(), 25);
    let mut count = 0_usize;
    for value in char::MIN..=char::MAX {
        if value.is_whitespace() {
            let mut encoded = [0; 4];
            assert_eq!(WHITE_SPACE.get(count), Some(&value.encode_utf8(&mut encoded).as_bytes()));
            count = count.saturating_add(1);
        }
    }
    assert_eq!(count, 25);
}
