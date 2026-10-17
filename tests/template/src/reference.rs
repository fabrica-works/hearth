//! Recursive descent over format.md, 7.4, independent of the machine's stack.
use hearth_protocol_template::{Expression, Filter, Op, Operation, limits::Limits};
#[derive(Clone)]
enum Token {
    Value(Op),
    Word,
    Symbol(u8),
    End,
}
struct Parser<'a> {
    text: &'a [u8],
    at: usize,
    limits: Limits,
    depth: u32,
    next: Option<(u32, Token)>,
    out: Vec<Operation>,
    values: u32,
    stack: u32,
}
/// Compiles the grammar with recursive descent.
///
/// # Errors
/// Gives the first refused byte offset.
pub fn compile(text: &[u8], limits: &Limits) -> Result<Expression, u32> {
    if text.len() > usize::try_from(limits.expression).expect("u32 fits") {
        return Err(limits.expression);
    }
    let mut p = Parser { text, at: 0, limits: *limits, depth: 0, next: None, out: Vec::new(), values: 0, stack: 0 };
    p.expression()?;
    let (at, token) = p.peek(false)?;
    match token {
        Token::End => Ok(Expression { operations: p.out.into_boxed_slice(), values: p.values }),
        Token::Value(_) | Token::Word | Token::Symbol(_) => Err(at),
    }
}
impl Parser<'_> {
    fn emit(&mut self, at: u32, op: Op, pops: u32) {
        self.stack = self.stack - pops + 1;
        self.values = self.values.max(self.stack);
        self.out.push(Operation { at, op });
    }
    fn peek(&mut self, operand: bool) -> Result<(u32, Token), u32> {
        if let Some(token) = &self.next {
            return Ok(token.clone());
        }
        while let Some(b' ' | b'\t' | b'\n') = self.text.get(self.at) {
            self.at += 1;
        }
        let at = u32::try_from(self.at).expect("length checked");
        let token = match self.text.get(self.at).copied() {
            None => Token::End,
            Some(byte @ (b'.' | b'|' | b'[' | b']' | b'(' | b')' | b',')) => {
                self.at += 1;
                Token::Symbol(byte)
            }
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => {
                let start = self.at;
                while self.text.get(self.at).is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_') {
                    self.at += 1;
                }
                match &self.text[start..self.at] {
                    b"true" => Token::Value(Op::Boolean(true)),
                    b"false" => Token::Value(Op::Boolean(false)),
                    b"and" | b"or" | b"not" | b"if" | b"else" | b"is" | b"in" | b"none" | b"None" | b"True"
                    | b"False" => Token::Word,
                    name => Token::Value(Op::Name { name: name.into() }),
                }
            }
            Some(_) => Token::Value(self.literal(operand)?),
        };
        self.next = Some((at, token.clone()));
        Ok((at, token))
    }
    fn take(&mut self, operand: bool) -> Result<(u32, Token), u32> {
        let token = self.peek(operand)?;
        self.next = None;
        Ok(token)
    }
    fn open(&mut self, at: u32) -> Result<(), u32> {
        if self.depth == self.limits.nesting {
            return Err(at);
        }
        self.depth += 1;
        Ok(())
    }
    fn expect(&mut self, wanted: u8) -> Result<(), u32> {
        let (at, token) = self.take(false)?;
        match token {
            Token::Symbol(byte) if byte == wanted => Ok(()),
            Token::Symbol(_) | Token::Word | Token::Value(_) | Token::End => Err(at),
        }
    }
    fn name(&mut self) -> Result<(u32, Vec<u8>), u32> {
        let (at, token) = self.take(false)?;
        match token {
            Token::Word | Token::Value(Op::Name { .. } | Op::Boolean(_)) => {
                Ok((at, self.text[usize::try_from(at).expect("offset fits")..self.at].to_vec()))
            }
            Token::Value(
                Op::String(_) | Op::Integer(_) | Op::Key { .. } | Op::Item | Op::List { .. } | Op::Filter(_),
            )
            | Token::Symbol(_)
            | Token::End => Err(at),
        }
    }
    fn expression(&mut self) -> Result<(), u32> {
        self.postfix()?;
        while let (pipe, Token::Symbol(b'|')) = self.peek(false)? {
            self.take(false)?;
            let (at, name) = self.name()?;
            let filter = filter(&name).ok_or(at)?;
            let mut given = 0;
            if let (open, Token::Symbol(b'(')) = self.peek(false)? {
                self.take(false)?;
                self.open(open)?;
                if !matches!(self.peek(true)?.1, Token::Symbol(b')')) {
                    loop {
                        self.expression()?;
                        given += 1;
                        if !matches!(self.peek(false)?.1, Token::Symbol(b',')) {
                            break;
                        }
                        self.take(false)?;
                    }
                }
                self.expect(b')')?;
                self.depth -= 1;
            }
            if given != arity(filter) {
                return Err(at);
            }
            self.emit(at, Op::Filter(filter), given + 1);
            if matches!(self.peek(false)?.1, Token::Symbol(b'.' | b'[')) {
                return Err(self.peek(false)?.0);
            }
            let _ = pipe;
        }
        Ok(())
    }
    fn postfix(&mut self) -> Result<(), u32> {
        self.primary()?;
        loop {
            match self.peek(false)? {
                (_, Token::Symbol(b'.')) => {
                    self.take(false)?;
                    let (at, name) = self.name()?;
                    self.emit(at, Op::Key { name: name.into_boxed_slice() }, 1);
                }
                (at, Token::Symbol(b'[')) => {
                    self.take(false)?;
                    self.open(at)?;
                    self.expression()?;
                    self.expect(b']')?;
                    self.depth -= 1;
                    self.emit(at, Op::Item, 2);
                }
                (_, Token::Value(_) | Token::Word | Token::End | Token::Symbol(_)) => break,
            }
        }
        Ok(())
    }
    fn primary(&mut self) -> Result<(), u32> {
        let (at, token) = self.take(true)?;
        match token {
            Token::Value(op) => self.emit(at, op, 0),
            Token::Symbol(b'(') => {
                self.open(at)?;
                self.expression()?;
                self.expect(b')')?;
                self.depth -= 1;
            }
            Token::Symbol(b'[') => {
                self.open(at)?;
                let mut length = 0;
                if !matches!(self.peek(true)?.1, Token::Symbol(b']')) {
                    loop {
                        self.expression()?;
                        length += 1;
                        if !matches!(self.peek(false)?.1, Token::Symbol(b',')) {
                            break;
                        }
                        self.take(false)?;
                    }
                }
                self.expect(b']')?;
                self.depth -= 1;
                self.emit(at, Op::List { length }, length);
            }
            Token::Word | Token::Symbol(_) | Token::End => return Err(at),
        }
        Ok(())
    }
    fn literal(&mut self, operand: bool) -> Result<Op, u32> {
        let start = self.at;
        let error = u32::try_from(start).expect("bounded by fixture or checked text");
        let Some(&byte) = self.text.get(start) else {
            return Err(error);
        };
        if byte == b'\'' || byte == b'"' {
            self.at += 1;
            let mut out = Vec::new();
            while let Some(&b) = self.text.get(self.at) {
                self.at += 1;
                if b == byte {
                    return Ok(Op::String(out.into_boxed_slice()));
                }
                if b != b'\\' {
                    out.push(b);
                    continue;
                }
                let backslash = self.at - 1;
                let Some(&escaped) = self.text.get(self.at) else {
                    return Err(error);
                };
                self.at += 1;
                out.push(match escaped {
                    b'\\' | b'\'' | b'"' => escaped,
                    b'n' => b'\n',
                    b't' => b'\t',
                    _ => return Err(u32::try_from(backslash).expect("bounded by fixture or checked text")),
                });
            }
            return Err(error);
        }
        if byte.is_ascii_digit()
            || (operand && byte == b'-' && self.text.get(start + 1).is_some_and(u8::is_ascii_digit))
        {
            if byte == b'-' {
                self.at += 1;
            }
            while self.text.get(self.at).is_some_and(u8::is_ascii_digit) {
                self.at += 1;
            }
            let negative = byte == b'-';
            let digits = &self.text[start + usize::from(negative)..self.at];
            let mut magnitude = 0_u128;
            for digit in digits {
                magnitude = magnitude.saturating_mul(10).saturating_add(u128::from(digit - b'0'));
            }
            let value = if negative {
                if magnitude > 9_223_372_036_854_775_808 {
                    return Err(error);
                }
                i64::try_from(-i128::try_from(magnitude).expect("bounded by fixture or checked text"))
                    .expect("bounded by fixture or checked text")
            } else {
                i64::try_from(magnitude).map_err(|_| error)?
            };
            return Ok(Op::Integer(value));
        }
        if byte.is_ascii_alphabetic() || byte == b'_' {
            self.at += 1;
            while self.text.get(self.at).is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_') {
                self.at += 1;
            }
            return match &self.text[start..self.at] {
                b"true" => Ok(Op::Boolean(true)),
                b"false" => Ok(Op::Boolean(false)),
                b"and" | b"or" | b"not" | b"if" | b"else" | b"is" | b"in" | b"none" | b"None" | b"True" | b"False" => {
                    Err(error)
                }
                name => Ok(Op::Name { name: name.into() }),
            };
        }
        Err(error)
    }
}
fn filter(name: &[u8]) -> Option<Filter> {
    const FILTERS: &[(&[u8], Filter)] = &[
        (b"default", Filter::Default),
        (b"join", Filter::Join),
        (b"split", Filter::Split),
        (b"first", Filter::First),
        (b"last", Filter::Last),
        (b"length", Filter::Length),
        (b"replace", Filter::Replace),
        (b"items", Filter::Items),
        (b"keys", Filter::Keys),
        (b"values", Filter::Values),
        (b"sort", Filter::Sort),
        (b"tojson", Filter::Tojson),
        (b"shell_quote", Filter::ShellQuote),
        (b"systemd_escape", Filter::SystemdEscape),
        (b"systemd_escape_path", Filter::SystemdEscapePath),
        (b"basename", Filter::Basename),
        (b"dirname", Filter::Dirname),
    ];
    FILTERS.iter().find(|(n, _)| *n == name).map(|(_, f)| *f)
}
fn arity(filter: Filter) -> u32 {
    match filter {
        Filter::Default | Filter::Join | Filter::Split => 1,
        Filter::Replace => 2,
        Filter::First
        | Filter::Last
        | Filter::Length
        | Filter::Items
        | Filter::Keys
        | Filter::Values
        | Filter::Sort
        | Filter::Tojson
        | Filter::ShellQuote
        | Filter::SystemdEscape
        | Filter::SystemdEscapePath
        | Filter::Basename
        | Filter::Dirname => 0,
    }
}
