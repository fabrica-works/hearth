//! One bounded token pass with explicit nesting (format.md, 7.4; render.md, 8.3).
mod lexer;
mod pending;
use crate::{Error, ErrorKind, Expected, Expression, Filter, Op, Operation, limits::Limits};
use lexer::{Lexed, Lexer, Token};
use pending::Pending;
use skein_lib::{List, Stack, bytes::copy_of};
#[derive(Debug)]
enum Phase {
    Operand,
    Operator,
    Key,
    Filter,
}
#[derive(Debug)]
struct Parser {
    out: List<Operation>,
    pending: Stack<Pending>,
    phase: Phase,
    filter: Option<(Filter, u32)>,
    filtered: bool,
    depth: u32,
    values: u32,
}
/// Compiles one expression, or gives the first refusal with its byte offset.
pub fn compile(text: &[u8], limits: &Limits) -> Result<Expression, Error> {
    if text.len() > usize::try_from(limits.expression).expect("u32 fits usize") {
        return Err(Error { at: limits.expression, kind: ErrorKind::TooLong });
    }
    let length = u32::try_from(text.len()).expect("length checked");
    let mut lexer = Lexer { text: copy_of(text), at: 0 };
    let mut parser = Parser {
        out: List::with_capacity(length),
        pending: Stack::with_capacity(limits.nesting.min(length)),
        phase: Phase::Operand,
        filter: None,
        filtered: false,
        depth: 0,
        values: 0,
    };
    for _ in 0..=length {
        let operand = match parser.phase {
            Phase::Operand => true,
            Phase::Operator | Phase::Key | Phase::Filter => false,
        };
        let token = lexer.next(operand)?;
        if parser.read(token, &lexer)? {
            return Ok(Expression { operations: parser.out.into_boxed(), values: parser.values });
        }
    }
    unreachable!("every token consumes at least one byte, then an end token")
}
impl Parser {
    fn emit(&mut self, at: u32, op: Op, pops: u32) {
        self.depth = self.depth.checked_sub(pops).expect("grammar supplies operands").saturating_add(1);
        self.values = self.values.max(self.depth);
        self.out.push(Operation { at, op }).expect("operations fit token bytes");
    }
    fn open(&mut self, pending: Pending, at: u32) -> Result<(), Error> {
        match self.pending.push(pending) {
            Ok(()) => {
                self.phase = Phase::Operand;
                self.filtered = false;
                Ok(())
            }
            Err(_) => Err(Error { at, kind: ErrorKind::TooDeep }),
        }
    }
    fn apply(&mut self, filter: Filter, at: u32, given: u32) -> Result<(), Error> {
        if given != filter.arity() {
            return Err(Error { at, kind: ErrorKind::Arity { filter, given } });
        }
        self.emit(at, Op::Filter(filter), given.saturating_add(1));
        self.phase = Phase::Operator;
        self.filtered = true;
        Ok(())
    }
    fn read(&mut self, input: Lexed, lexer: &Lexer) -> Result<bool, Error> {
        let Lexed { token, at } = input;
        if let Some((filter, name_at)) = self.filter.take() {
            match token {
                Token::Symbol(b'(') => {
                    self.open(Pending::Arguments { start: self.out.len(), filter, given: 0, at: name_at }, at)?;
                    return Ok(false);
                }
                Token::Symbol(_) | Token::Value(_) | Token::Word | Token::End => self.apply(filter, name_at, 0)?,
            }
        }
        match self.phase {
            Phase::Key | Phase::Filter => {
                match &token {
                    Token::Word | Token::Value(Op::Name { .. } | Op::Boolean(_)) => {}
                    Token::Value(
                        Op::String(_) | Op::Integer(_) | Op::Key { .. } | Op::Item | Op::List { .. } | Op::Filter(_),
                    )
                    | Token::Symbol(_)
                    | Token::End => return Err(Error { at, kind: ErrorKind::Expected(Expected::Name) }),
                }
                let name = lexer.spelling(at);
                match self.phase {
                    Phase::Key => self.emit(at, Op::Key { name }, 1),
                    Phase::Filter => {
                        let filter = Filter::named(&name).ok_or(Error { at, kind: ErrorKind::UnknownFilter })?;
                        self.filter = Some((filter, at));
                    }
                    Phase::Operand | Phase::Operator => unreachable!("name phase"),
                }
                self.phase = Phase::Operator;
            }
            Phase::Operand => match token {
                Token::Value(op) => {
                    self.emit(at, op, 0);
                    self.phase = Phase::Operator;
                }
                Token::Symbol(b'(') => self.open(Pending::Group { start: self.out.len() }, at)?,
                Token::Symbol(b'[') => self.open(Pending::List { start: self.out.len(), length: 0, at }, at)?,
                Token::Symbol(b']' | b')') => self.close(at, token, true)?,
                Token::Word | Token::Symbol(_) | Token::End => {
                    return Err(Error { at, kind: ErrorKind::Expected(Expected::Operand) });
                }
            },
            Phase::Operator => match token {
                Token::Symbol(b'.') => {
                    if self.filtered {
                        return Err(Error { at, kind: ErrorKind::Misplaced });
                    }
                    self.phase = Phase::Key;
                }
                Token::Symbol(b'[') => {
                    if self.filtered {
                        return Err(Error { at, kind: ErrorKind::Misplaced });
                    }
                    self.open(Pending::Index { start: self.out.len(), at }, at)?;
                }
                Token::Symbol(b'|') => self.phase = Phase::Filter,
                Token::Symbol(b']' | b')') => self.close(at, token, false)?,
                Token::Symbol(b',') => self.comma(at)?,
                Token::End => {
                    if let Some(pending) = self.pending.top() {
                        return Err(Error { at, kind: ErrorKind::Expected(closing(pending)) });
                    }
                    return Ok(true);
                }
                Token::Value(_) | Token::Word | Token::Symbol(_) => {
                    return Err(Error { at, kind: ErrorKind::Expected(Expected::Operator) });
                }
            },
        }
        Ok(false)
    }
    fn comma(&mut self, at: u32) -> Result<(), Error> {
        match self.pending.top_mut() {
            Some(Pending::List { length, .. }) => *length = length.saturating_add(1),
            Some(Pending::Arguments { given, .. }) => *given = given.saturating_add(1),
            Some(Pending::Group { .. } | Pending::Index { .. }) | None => {
                return Err(Error { at, kind: ErrorKind::Misplaced });
            }
        }
        self.phase = Phase::Operand;
        self.filtered = false;
        Ok(())
    }
    fn close(&mut self, at: u32, token: Token, empty: bool) -> Result<(), Error> {
        let pending = self.pending.pop().ok_or(Error { at, kind: ErrorKind::Misplaced })?;
        let expected = closing(&pending);
        let byte = match token {
            Token::Symbol(byte) => byte,
            Token::Value(_) | Token::Word | Token::End => unreachable!("close token"),
        };
        if (byte == b')') != (expected == Expected::CloseParen) {
            return Err(Error { at, kind: ErrorKind::Expected(expected) });
        }
        let start = match &pending {
            Pending::Group { start }
            | Pending::Index { start, .. }
            | Pending::List { start, .. }
            | Pending::Arguments { start, .. } => *start,
        };
        let vacant = self.out.len() == start;
        if empty && !vacant {
            return Err(Error { at, kind: ErrorKind::Expected(Expected::Operand) });
        }
        self.filtered = false;
        match pending {
            Pending::Group { .. } => {
                if empty {
                    return Err(Error { at, kind: ErrorKind::Expected(Expected::Operand) });
                }
            }
            Pending::Index { at: open, .. } => {
                if empty {
                    return Err(Error { at, kind: ErrorKind::Expected(Expected::Operand) });
                }
                self.emit(open, Op::Item, 2);
            }
            Pending::List { length, at: open, .. } => {
                let length = length.saturating_add(u32::from(!vacant));
                self.emit(open, Op::List { length }, length);
            }
            Pending::Arguments { filter, given, at: name_at, .. } => {
                self.apply(filter, name_at, given.saturating_add(u32::from(!vacant)))?;
            }
        }
        self.phase = Phase::Operator;
        Ok(())
    }
}
fn closing(pending: &Pending) -> Expected {
    match pending {
        Pending::Group { .. } | Pending::Arguments { .. } => Expected::CloseParen,
        Pending::List { .. } | Pending::Index { .. } => Expected::CloseBracket,
    }
}
