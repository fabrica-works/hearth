use crate::Filter;
/// A decode refusal, placed in its file by the caller (domain/render.md, 7.2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Error {
    pub at: u32,
    pub kind: ErrorKind,
}
/// Why expression text was refused (protocol/format.md, 7.4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ErrorKind {
    /// A statement, comment or minus marker in an interpolated string (protocol/format.md, 5.2).
    Interpolation,
    Unmatched,
    Unended,
    Shadows,
    Utf8,
    Return,
    Unclosed,
    UnknownStatement,
    TooLong,
    Character,
    Unterminated,
    Escape,
    Integer,
    Expected(Expected),
    TooDeep,
    Misplaced,
    UnknownFilter,
    UnknownTest,
    Arity {
        filter: Filter,
        given: u32,
    },
}
/// The next grammatical token required.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Expected {
    Statement,
    Assign,
    TagEnd,
    Operand,
    Operator,
    Name,
    CloseParen,
    CloseBracket,
    In,
    Else,
}
