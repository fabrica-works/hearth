//! Template syntax faults (format.md, 7.2 to 7.5).
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
    Arity { filter: crate::evaluate::Filter, given: u32 },
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
