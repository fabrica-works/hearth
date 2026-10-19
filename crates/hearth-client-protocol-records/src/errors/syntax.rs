//! Template syntax faults (protocol/format.md, 7.2 to 7.5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Syntax {
    TooLong,
    Character,
    Unterminated,
    Escape,
    Integer,
    TooDeep,
    Misplaced,
    UnknownFilter,
    UnknownTest,
    Utf8,
    Return,
    Unclosed,
    UnknownStatement,
    Unmatched,
    Unended,
    Shadows,
    Interpolation,
    Expected(Expected),
    Arity { filter: Box<[u8]>, expected: u32, given: u32 },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Expected {
    Operand,
    Operator,
    Name,
    CloseParen,
    CloseBracket,
    In,
    Else,
    Statement,
    Assign,
    TagEnd,
}
use super::{Box, Encoder, Lines, field};
impl Syntax {
    pub(super) fn text(&self, lines: &mut Lines) {
        match self {
            Self::TooLong => lines.put(b"a template past its length limit"),
            Self::Character => lines.put(b"an unexpected character"),
            Self::Unterminated => lines.put(b"a string with no end"),
            Self::Escape => lines.put(b"an invalid escape"),
            Self::Integer => lines.put(b"an integer outside i64"),
            Self::TooDeep => lines.put(b"too deep"),
            Self::Misplaced => lines.put(b"a misplaced expression"),
            Self::UnknownFilter => lines.put(b"an unknown filter"),
            Self::UnknownTest => lines.put(b"an unknown test"),
            Self::Utf8 => lines.put(b"not UTF-8"),
            Self::Return => lines.put(b"a carriage return"),
            Self::Unclosed => lines.put(b"a tag with no end"),
            Self::UnknownStatement => lines.put(b"an unknown statement"),
            Self::Unmatched => lines.put(b"a close with no open block"),
            Self::Unended => lines.put(b"a block left open"),
            Self::Shadows => lines.put(b"a root of the context bound"),
            Self::Interpolation => lines.put(b"a statement in an interpolated string"),
            Self::Expected(expected) => {
                lines.put(b"expected ");
                lines.put(expected.name());
            }
            Self::Arity { filter, expected, given } => {
                lines.quote(filter);
                lines.put(b" takes ");
                lines.decimal(u64::from(*expected));
                lines.put(if *expected == 1 { b" argument, given " } else { b" arguments, given " });
                lines.decimal(u64::from(*given));
            }
        }
    }
    pub(super) fn json(&self, json: &mut Encoder) {
        json.object_start();
        field(json, b"kind", b"syntax");
        match self {
            Self::TooLong => field(json, b"fault", b"too_long"),
            Self::Character => field(json, b"fault", b"character"),
            Self::Unterminated => field(json, b"fault", b"unterminated"),
            Self::Escape => field(json, b"fault", b"escape"),
            Self::Integer => field(json, b"fault", b"integer"),
            Self::TooDeep => field(json, b"fault", b"too_deep"),
            Self::Misplaced => field(json, b"fault", b"misplaced"),
            Self::UnknownFilter => field(json, b"fault", b"unknown_filter"),
            Self::UnknownTest => field(json, b"fault", b"unknown_test"),
            Self::Utf8 => field(json, b"fault", b"utf8"),
            Self::Return => field(json, b"fault", b"return"),
            Self::Unclosed => field(json, b"fault", b"unclosed"),
            Self::UnknownStatement => field(json, b"fault", b"unknown_statement"),
            Self::Unmatched => field(json, b"fault", b"unmatched"),
            Self::Unended => field(json, b"fault", b"unended"),
            Self::Shadows => field(json, b"fault", b"shadows"),
            Self::Interpolation => field(json, b"fault", b"interpolation"),
            Self::Expected(expected) => {
                field(json, b"fault", b"expected");
                field(json, b"expected", expected.name());
            }
            Self::Arity { filter, expected, given } => {
                field(json, b"fault", b"arity");
                field(json, b"filter", filter);
                json.key(b"expected");
                json.unsigned(u64::from(*expected));
                json.key(b"given");
                json.unsigned(u64::from(*given));
            }
        }
        json.object_end();
    }
}
impl Expected {
    pub(super) fn name(self) -> &'static [u8] {
        match self {
            Self::Operand => b"operand",
            Self::Operator => b"operator",
            Self::Name => b"name",
            Self::CloseParen => b")",
            Self::CloseBracket => b"]",
            Self::In => b"in",
            Self::Else => b"else",
            Self::Statement => b"statement",
            Self::Assign => b"=",
            Self::TagEnd => b"tag end",
        }
    }
}
