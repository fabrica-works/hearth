use crate::{Error, ErrorKind, Missing, Op};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
#[derive(Debug)]
pub(crate) struct Lexer {
    pub text: Box<[u8]>,
    pub at: usize,
}
#[derive(Debug)]
pub(crate) struct Lexed {
    pub token: Token,
    pub at: u32,
}
#[derive(Debug)]
pub(crate) enum Token {
    Value(Op),
    Word,
    Symbol(u8),
    End,
}
impl Lexer {
    pub(crate) fn next(&mut self, operand: bool) -> Result<Lexed, Error> {
        for at in self.at..self.text.len() {
            match self.text.get(at) {
                Some(b' ' | b'\t' | b'\n') => self.at = at.saturating_add(1),
                Some(_) | None => break,
            }
        }
        let start = self.at;
        let at = u32::try_from(start).expect("expression length checked");
        let token = match self.text.get(start).copied() {
            None => Token::End,
            Some(b'\'' | b'"') => Token::Value(Op::String(self.string()?)),
            Some(b'-') if operand && digit(self.text.get(start.saturating_add(1))) => {
                Token::Value(Op::Integer(self.integer()?))
            }
            Some(byte @ (b'.' | b'|' | b'[' | b']' | b'(' | b')' | b',' | b'+' | b'-' | b'~')) => {
                self.at = self.at.saturating_add(1);
                Token::Symbol(byte)
            }
            Some(b'0'..=b'9') => Token::Value(Op::Integer(self.integer()?)),
            Some(byte @ (b'=' | b'!' | b'<' | b'>')) => {
                self.at = self.at.saturating_add(1);
                if self.text.get(self.at) == Some(&b'=') {
                    self.at = self.at.saturating_add(1);
                } else if byte == b'=' || byte == b'!' {
                    return Err(Error { at, kind: ErrorKind::Character });
                }
                Token::Symbol(byte)
            }
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => self.name(),
            Some(_) => return Err(Error { at, kind: ErrorKind::Character }),
        };
        Ok(Lexed { token, at })
    }
    pub(super) fn spelling(&self, at: u32) -> Box<[u8]> {
        copy_of(self.text.get(usize::try_from(at).expect("offset fits")..self.at).expect("name span"))
    }
    fn name(&mut self) -> Token {
        let start = self.at;
        for at in start..self.text.len() {
            match self.text.get(at) {
                Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_') => self.at = at.saturating_add(1),
                Some(_) | None => break,
            }
        }
        match self.text.get(start..self.at).expect("scanned name") {
            b"true" => Token::Value(Op::Boolean(true)),
            b"false" => Token::Value(Op::Boolean(false)),
            b"and" | b"or" | b"not" | b"if" | b"else" | b"is" | b"in" | b"none" | b"None" | b"True" | b"False" => {
                Token::Word
            }
            name => Token::Value(Op::Name { name: copy_of(name), missing: Missing::Error }),
        }
    }
    fn integer(&mut self) -> Result<i64, Error> {
        let start = self.at;
        let negative = self.text.get(start) == Some(&b'-');
        if negative {
            self.at = self.at.saturating_add(1);
        }
        let mut value = Some(0_i64);
        for at in self.at..self.text.len() {
            let byte = *self.text.get(at).expect("in text");
            if !byte.is_ascii_digit() {
                break;
            }
            self.at = at.saturating_add(1);
            value = match value {
                Some(n) => match n.checked_mul(10) {
                    Some(n) => n.checked_sub(i64::from(byte.saturating_sub(b'0'))),
                    None => None,
                },
                None => None,
            };
        }
        let value = match value {
            Some(n) if negative => Some(n),
            Some(n) => n.checked_neg(),
            None => None,
        };
        value.ok_or(Error { at: u32::try_from(start).expect("length checked"), kind: ErrorKind::Integer })
    }
    fn string(&mut self) -> Result<Box<[u8]>, Error> {
        let start = self.at;
        let quote = *self.text.get(start).expect("opening quote");
        let mut out = List::with_capacity(u32::try_from(self.text.len()).expect("length checked"));
        let mut escape = None;
        self.at = start.saturating_add(1);
        for at in self.at..self.text.len() {
            let byte = *self.text.get(at).expect("in text");
            self.at = at.saturating_add(1);
            if let Some(backslash) = escape.take() {
                let decoded = match byte {
                    b'\\' | b'\'' | b'"' => byte,
                    b'n' => b'\n',
                    b't' => b'\t',
                    _ => return Err(Error { at: backslash, kind: ErrorKind::Escape }),
                };
                out.push(decoded).expect("decoded bytes fit text");
            } else if byte == quote {
                return Ok(out.into_boxed());
            } else if byte == b'\\' {
                escape = Some(u32::try_from(at).expect("length checked"));
            } else {
                out.push(byte).expect("decoded bytes fit text");
            }
        }
        Err(Error { at: u32::try_from(start).expect("length checked"), kind: ErrorKind::Unterminated })
    }
}

fn digit(byte: Option<&u8>) -> bool {
    match byte {
        Some(b'0'..=b'9') => true,
        Some(_) | None => false,
    }
}
