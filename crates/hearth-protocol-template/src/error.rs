use crate::Filter;
/// A decode refusal, placed in its file by the caller (render.md, 7.2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Error {
    pub at: u32,
    pub kind: ErrorKind,
}
/// Why expression text was refused (format.md, 7.4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ErrorKind {
    TooLong,
    Character,
    Unterminated,
    Escape,
    Integer,
    Expected(Expected),
    TooDeep,
    Misplaced,
    UnknownFilter,
    Arity { filter: Filter, given: u32 },
}
/// The next grammatical token required.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Expected {
    Operand,
    Operator,
    Name,
    CloseParen,
    CloseBracket,
}
