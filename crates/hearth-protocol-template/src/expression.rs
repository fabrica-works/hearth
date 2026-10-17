//! One bounded token pass with explicit nesting (format.md, 7.4; render.md, 8.3).
mod chain;
mod jumps;
mod lexer;
mod pending;
use crate::{Error, ErrorKind, Expected, Expression, Filter, Missing, Op, Operation, limits::Limits};
use chain::Chain;
use lexer::{Lexed, Lexer, Token};
use pending::Pending;
use skein_lib::{List, Stack, bytes::copy_of};
#[derive(Debug)]
enum Phase {
    Operand,
    Operator,
    Key,
    Filter,
    In { at: u32 },
    Test { at: u32, negated: bool },
}
#[derive(Debug)]
struct Parser {
    out: List<Operation>,
    pending: Stack<Pending>,
    phase: Phase,
    filter: Option<(Filter, u32)>,
    filtered: bool,
    chain: Chain,
    tested: bool,
    nesting: u32,
    limit: u32,
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
        pending: Stack::with_capacity(length),
        phase: Phase::Operand,
        filter: None,
        filtered: false,
        chain: Chain { links: List::with_capacity(length), start: 0, active: false },
        tested: false,
        nesting: 0,
        limit: limits.nesting,
        depth: 0,
        values: 0,
    };
    for _ in 0..=length {
        let operand = match parser.phase {
            Phase::Operand => true,
            Phase::Operator | Phase::Key | Phase::Filter | Phase::In { .. } | Phase::Test { .. } => false,
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
        if self.nesting == self.limit {
            return Err(Error { at, kind: ErrorKind::TooDeep });
        }
        self.nesting = self.nesting.saturating_add(1);
        match self.pending.push(pending) {
            Ok(()) => {
                self.phase = Phase::Operand;
                self.filtered = false;
                self.tested = false;
                Ok(())
            }
            Err(_) => Err(Error { at, kind: ErrorKind::TooDeep }),
        }
    }
    fn apply(&mut self, filter: Filter, at: u32, given: u32) -> Result<(), Error> {
        if given != filter.arity() {
            return Err(Error { at, kind: ErrorKind::Arity { filter, given } });
        }
        self.chain.end();
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
                    self.open(
                        Pending::Arguments {
                            start: self.out.len(),
                            mark: self.out.len(),
                            filter,
                            given: 0,
                            at: name_at,
                        },
                        at,
                    )?;
                    return Ok(false);
                }
                Token::Symbol(_) | Token::Value(_) | Token::Word | Token::End => self.apply(filter, name_at, 0)?,
            }
        }
        match self.phase {
            Phase::Key | Phase::Filter => {
                if !word_start(lexer.spelling(at).first()) {
                    return Err(Error { at, kind: ErrorKind::Expected(Expected::Name) });
                }
                let name = lexer.spelling(at);
                match self.phase {
                    Phase::Key => {
                        self.chain.link(self.out.len());
                        self.emit(at, Op::Key { name, missing: Missing::Error }, 1);
                    }
                    Phase::Filter => {
                        let filter = Filter::named(&name).ok_or(Error { at, kind: ErrorKind::UnknownFilter })?;
                        if filter == Filter::Default {
                            self.chain.mark(&mut self.out);
                        }
                        self.chain.end();
                        self.filter = Some((filter, at));
                    }
                    Phase::Operand | Phase::Operator | Phase::In { .. } | Phase::Test { .. } => {
                        unreachable!("name phase")
                    }
                }
                self.phase = Phase::Operator;
            }
            Phase::Operand => self.read_operand(token, at, lexer)?,
            Phase::Operator => return self.read_operator(token, at, lexer),
            Phase::In { at: not_at } => {
                if lexer.spelling(at).as_ref() != b"in" {
                    return Err(Error { at, kind: ErrorKind::Expected(Expected::In) });
                }
                self.binary(Op::NotIn, not_at)?;
            }
            Phase::Test { at: is_at, negated } => match lexer.spelling(at).as_ref() {
                b"not" if !negated => self.phase = Phase::Test { at: is_at, negated: true },
                b"defined" => {
                    self.emit(is_at, if negated { Op::NotDefined } else { Op::Defined }, 1);
                    self.phase = Phase::Operator;
                    self.tested = true;
                }
                _ => return Err(Error { at, kind: ErrorKind::UnknownTest }),
            },
        }
        Ok(false)
    }
    fn read_operand(&mut self, token: Token, at: u32, lexer: &Lexer) -> Result<(), Error> {
        match token {
            Token::Value(op) => {
                self.chain.end();
                self.chain.active = match &op {
                    Op::Name { .. } => true,
                    Op::String(_)
                    | Op::Integer(_)
                    | Op::Boolean(_)
                    | Op::Key { .. }
                    | Op::Item { .. }
                    | Op::List { .. }
                    | Op::Filter(_)
                    | Op::Not
                    | Op::Equal
                    | Op::NotEqual
                    | Op::Less
                    | Op::LessOrEqual
                    | Op::Greater
                    | Op::GreaterOrEqual
                    | Op::In
                    | Op::NotIn
                    | Op::Concat
                    | Op::Add
                    | Op::Subtract
                    | Op::Defined
                    | Op::NotDefined
                    | Op::JumpIfFalseKeeping { .. }
                    | Op::JumpIfTrueKeeping { .. }
                    | Op::JumpIfFalse { .. }
                    | Op::Jump { .. } => false,
                };
                self.emit(at, op, 0);
                self.phase = Phase::Operator;
            }
            Token::Word if lexer.spelling(at).as_ref() == b"not" => {
                match self.pending.top() {
                    Some(Pending::Binary { .. }) => return Err(Error { at, kind: ErrorKind::Misplaced }),
                    Some(
                        Pending::Not { .. }
                        | Pending::Group { .. }
                        | Pending::List { .. }
                        | Pending::Index { .. }
                        | Pending::Arguments { .. }
                        | Pending::And { .. }
                        | Pending::Or { .. }
                        | Pending::If { .. }
                        | Pending::Else { .. },
                    )
                    | None => {}
                }
                self.chain.end();
                self.open(Pending::Not { at }, at)?;
            }
            Token::Symbol(b'(') => {
                self.chain.end();
                self.open(Pending::Group { start: self.out.len() }, at)?;
            }
            Token::Symbol(b'[') => {
                self.chain.end();
                self.open(Pending::List { start: self.out.len(), mark: self.out.len(), length: 0, at }, at)?;
            }
            Token::Symbol(b']' | b')') => self.close(at, token, true)?,
            Token::Word | Token::Symbol(_) | Token::End => {
                return Err(Error { at, kind: ErrorKind::Expected(Expected::Operand) });
            }
        }
        Ok(())
    }
    fn read_operator(&mut self, token: Token, at: u32, lexer: &Lexer) -> Result<bool, Error> {
        if self.tested {
            match token {
                Token::Symbol(b'.' | b'[' | b'|') => return Err(Error { at, kind: ErrorKind::Misplaced }),
                Token::Symbol(_) | Token::Value(_) | Token::Word | Token::End => {}
            }
        }
        match token {
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
                let chain = self.chain.enter();
                self.open(Pending::Index { start: self.out.len(), at, chain }, at)?;
            }
            Token::Symbol(b'|') => self.phase = Phase::Filter,
            Token::Symbol(b']' | b')') => self.close(at, token, false)?,
            Token::Symbol(b',') => self.comma(at)?,
            Token::End => {
                self.finish(at)?;
                if let Some(pending) = self.pending.top() {
                    return Err(Error { at, kind: ErrorKind::Expected(closing(pending)) });
                }
                return Ok(true);
            }
            Token::Word => self.word(at, &lexer.spelling(at))?,
            Token::Symbol(b'+' | b'-' | b'~' | b'=' | b'!' | b'<' | b'>') => {
                let op = match lexer.spelling(at).as_ref() {
                    b"+" => Op::Add,
                    b"-" => Op::Subtract,
                    b"~" => Op::Concat,
                    b"==" => Op::Equal,
                    b"!=" => Op::NotEqual,
                    b"<" => Op::Less,
                    b"<=" => Op::LessOrEqual,
                    b">" => Op::Greater,
                    b">=" => Op::GreaterOrEqual,
                    _ => unreachable!("lexed operator"),
                };
                self.binary(op, at)?;
            }
            Token::Value(_) | Token::Symbol(_) => {
                return Err(Error { at, kind: ErrorKind::Expected(Expected::Operator) });
            }
        }
        Ok(false)
    }
    fn word(&mut self, at: u32, word: &[u8]) -> Result<(), Error> {
        match word {
            b"and" => self.short(false, at),
            b"or" => self.short(true, at),
            b"if" => self.conditional(at),
            b"else" => self.otherwise(at),
            b"in" => self.binary(Op::In, at),
            b"not" => {
                if self.tested {
                    return Err(Error { at, kind: ErrorKind::Misplaced });
                }
                self.reduce(4, at)?;
                self.phase = Phase::In { at };
                Ok(())
            }
            b"is" => {
                if self.tested {
                    return Err(Error { at, kind: ErrorKind::Misplaced });
                }
                self.reduce(3, at)?;
                self.chain.mark(&mut self.out);
                self.chain.end();
                self.phase = Phase::Test { at, negated: false };
                Ok(())
            }
            _ => Err(Error { at, kind: ErrorKind::Expected(Expected::Operator) }),
        }
    }
    fn jump(&mut self, at: u32, op: Op) -> u32 {
        let index = self.out.len();
        self.out.push(Operation { at, op }).expect("jumps fit token bytes");
        index
    }
    fn short(&mut self, truth: bool, at: u32) -> Result<(), Error> {
        self.reduce(u8::from(!truth), at)?;
        self.chain.end();
        self.tested = false;
        let op = if truth { Op::JumpIfTrueKeeping { over: 0 } } else { Op::JumpIfFalseKeeping { over: 0 } };
        let jump = self.jump(at, op);
        self.pending
            .push(if truth { Pending::Or { jump } } else { Pending::And { jump } })
            .expect("operators fit text");
        self.depth = self.depth.checked_sub(1).expect("left operand");
        self.phase = Phase::Operand;
        self.filtered = false;
        Ok(())
    }
    fn conditional(&mut self, at: u32) -> Result<(), Error> {
        self.reduce(0, at)?;
        let start = match self.pending.top() {
            Some(Pending::If { .. }) => return Err(Error { at, kind: ErrorKind::Misplaced }),
            Some(Pending::Group { start } | Pending::Index { start, .. } | Pending::Else { start, .. }) => *start,
            Some(Pending::List { mark, .. } | Pending::Arguments { mark, .. }) => *mark,
            Some(Pending::Not { .. } | Pending::Binary { .. } | Pending::And { .. } | Pending::Or { .. }) => {
                unreachable!("operators reduced")
            }
            None => 0,
        };
        self.chain.end();
        self.depth = self.depth.checked_sub(1).expect("first branch");
        self.open(Pending::If { start, mid: self.out.len(), at }, at)
    }
    fn otherwise(&mut self, at: u32) -> Result<(), Error> {
        self.reduce(0, at)?;
        let (start, mid, if_at) = match self.pending.pop() {
            Some(Pending::If { start, mid, at }) => (start, mid, at),
            Some(
                Pending::Group { .. }
                | Pending::Index { .. }
                | Pending::List { .. }
                | Pending::Arguments { .. }
                | Pending::Not { .. }
                | Pending::Binary { .. }
                | Pending::And { .. }
                | Pending::Or { .. }
                | Pending::Else { .. },
            )
            | None => return Err(Error { at, kind: ErrorKind::Misplaced }),
        };
        self.chain.end();
        self.depth = self.depth.checked_sub(1).expect("condition");
        let over = mid.checked_sub(start).expect("branch extent").saturating_add(1);
        self.jump(if_at, Op::JumpIfFalse { over });
        jumps::rotate(&mut self.out, start, mid);
        let jump = self.jump(at, Op::Jump { over: 0 });
        self.pending.push(Pending::Else { jump, start: self.out.len() }).expect("room of replaced If");
        self.phase = Phase::Operand;
        self.filtered = false;
        self.tested = false;
        Ok(())
    }
    fn finish(&mut self, at: u32) -> Result<(), Error> {
        for _ in 0..=self.pending.len() {
            self.reduce(0, at)?;
            match self.pending.top() {
                Some(Pending::If { .. }) => return Err(Error { at, kind: ErrorKind::Expected(Expected::Else) }),
                Some(Pending::Else { jump, .. }) => {
                    jumps::patch(&mut self.out, *jump);
                    self.pending.pop().expect("else entry");
                    self.nesting = self.nesting.saturating_sub(1);
                }
                Some(
                    Pending::Group { .. }
                    | Pending::Index { .. }
                    | Pending::List { .. }
                    | Pending::Arguments { .. }
                    | Pending::Not { .. }
                    | Pending::Binary { .. }
                    | Pending::And { .. }
                    | Pending::Or { .. },
                )
                | None => break,
            }
        }
        Ok(())
    }
    fn binary(&mut self, op: Op, at: u32) -> Result<(), Error> {
        if self.tested {
            return Err(Error { at, kind: ErrorKind::Misplaced });
        }
        self.reduce(precedence(&op), at)?;
        self.chain.end();
        self.pending.push(Pending::Binary { op, at }).expect("pending operators fit text");
        self.phase = Phase::Operand;
        self.filtered = false;
        Ok(())
    }
    fn reduce(&mut self, level: u8, at: u32) -> Result<(), Error> {
        for _ in 0..self.pending.len() {
            let rank = match self.pending.top().expect("bounded reductions") {
                Pending::And { .. } => 1,
                Pending::Or { .. } => 0,
                Pending::Not { .. } => 2,
                Pending::Binary { op, .. } => precedence(op),
                Pending::Group { .. }
                | Pending::List { .. }
                | Pending::Index { .. }
                | Pending::Arguments { .. }
                | Pending::If { .. }
                | Pending::Else { .. } => {
                    break;
                }
            };
            if rank < level {
                break;
            }
            if rank == 4 && level == 4 {
                return Err(Error { at, kind: ErrorKind::Misplaced });
            }
            match self.pending.pop().expect("reducible entry") {
                Pending::And { jump } | Pending::Or { jump } => {
                    self.chain.end();
                    jumps::patch(&mut self.out, jump);
                }
                Pending::Not { at } => {
                    self.nesting = self.nesting.saturating_sub(1);
                    self.emit(at, Op::Not, 1);
                }
                Pending::Binary { op, at } => {
                    self.chain.end();
                    self.emit(at, op, 2);
                }
                Pending::Group { .. }
                | Pending::List { .. }
                | Pending::Index { .. }
                | Pending::Arguments { .. }
                | Pending::If { .. }
                | Pending::Else { .. } => {
                    unreachable!("operator entry")
                }
            }
        }
        Ok(())
    }
    fn comma(&mut self, at: u32) -> Result<(), Error> {
        self.finish(at)?;
        self.chain.end();
        self.tested = false;
        match self.pending.top_mut() {
            Some(Pending::List { length, mark, .. }) => {
                *length = length.saturating_add(1);
                *mark = self.out.len();
            }
            Some(Pending::Arguments { given, mark, .. }) => {
                *given = given.saturating_add(1);
                *mark = self.out.len();
            }
            Some(
                Pending::Group { .. }
                | Pending::Index { .. }
                | Pending::Not { .. }
                | Pending::Binary { .. }
                | Pending::And { .. }
                | Pending::Or { .. }
                | Pending::If { .. }
                | Pending::Else { .. },
            )
            | None => {
                return Err(Error { at, kind: ErrorKind::Misplaced });
            }
        }
        self.phase = Phase::Operand;
        self.filtered = false;
        Ok(())
    }
    fn close(&mut self, at: u32, token: Token, empty: bool) -> Result<(), Error> {
        if !empty {
            self.finish(at)?;
        }
        let pending = self.pending.pop().ok_or(Error { at, kind: ErrorKind::Misplaced })?;
        self.nesting = self.nesting.saturating_sub(1);
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
            Pending::Not { .. }
            | Pending::Binary { .. }
            | Pending::And { .. }
            | Pending::Or { .. }
            | Pending::If { .. }
            | Pending::Else { .. } => {
                return Err(Error { at, kind: ErrorKind::Expected(Expected::Operand) });
            }
        };
        let vacant = self.out.len() == start;
        if empty && !vacant {
            return Err(Error { at, kind: ErrorKind::Expected(Expected::Operand) });
        }
        self.filtered = false;
        match pending {
            Pending::Not { .. }
            | Pending::Binary { .. }
            | Pending::And { .. }
            | Pending::Or { .. }
            | Pending::If { .. }
            | Pending::Else { .. } => unreachable!("closed nesting entry"),
            Pending::Group { .. } => {
                self.chain.end();
                if empty {
                    return Err(Error { at, kind: ErrorKind::Expected(Expected::Operand) });
                }
            }
            Pending::Index { at: open, chain, .. } => {
                if empty {
                    return Err(Error { at, kind: ErrorKind::Expected(Expected::Operand) });
                }
                self.chain.leave(chain);
                self.chain.link(self.out.len());
                self.emit(open, Op::Item { missing: Missing::Error }, 2);
            }
            Pending::List { length, at: open, .. } => {
                self.chain.end();
                let length = length.saturating_add(u32::from(!vacant));
                self.emit(open, Op::List { length }, length);
            }
            Pending::Arguments { filter, given, at: name_at, .. } => {
                self.apply(filter, name_at, given.saturating_add(u32::from(!vacant)))?;
            }
        }
        self.phase = Phase::Operator;
        self.tested = false;
        Ok(())
    }
}
fn closing(pending: &Pending) -> Expected {
    match pending {
        Pending::Group { .. } | Pending::Arguments { .. } => Expected::CloseParen,
        Pending::List { .. } | Pending::Index { .. } => Expected::CloseBracket,
        Pending::Not { .. }
        | Pending::Binary { .. }
        | Pending::And { .. }
        | Pending::Or { .. }
        | Pending::If { .. }
        | Pending::Else { .. } => Expected::Operand,
    }
}

fn precedence(op: &Op) -> u8 {
    match op {
        Op::Equal
        | Op::NotEqual
        | Op::Less
        | Op::LessOrEqual
        | Op::Greater
        | Op::GreaterOrEqual
        | Op::In
        | Op::NotIn => 4,
        Op::Concat => 5,
        Op::Add | Op::Subtract => 6,
        Op::String(_)
        | Op::Integer(_)
        | Op::Boolean(_)
        | Op::Name { .. }
        | Op::Key { .. }
        | Op::Item { .. }
        | Op::List { .. }
        | Op::Filter(_)
        | Op::Not
        | Op::Defined
        | Op::NotDefined
        | Op::JumpIfFalseKeeping { .. }
        | Op::JumpIfTrueKeeping { .. }
        | Op::JumpIfFalse { .. }
        | Op::Jump { .. } => unreachable!("binary operator"),
    }
}

fn word_start(byte: Option<&u8>) -> bool {
    match byte {
        Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => true,
        Some(_) | None => false,
    }
}
