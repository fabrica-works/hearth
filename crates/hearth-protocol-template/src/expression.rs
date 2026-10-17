//! One expression's whole text, read left to right (format.md, 7.4).
mod lexer;
use crate::{Error, ErrorKind, Expected, Expression, Operation, limits::Limits};
use lexer::{Lexer, Token};
use skein_lib::{List, bytes::copy_of};
/// Compiles one expression, or gives the first refusal with its byte offset.
pub fn compile(text: &[u8], limits: &Limits) -> Result<Expression, Error> {
    if text.len() > usize::try_from(limits.expression).expect("u32 fits usize") {
        return Err(Error { at: limits.expression, kind: ErrorKind::TooLong });
    }
    let mut lexer = Lexer { text: copy_of(text), at: 0 };
    let first = lexer.next(true)?;
    let op = match first.token {
        Token::Value(op) => op,
        Token::Word | Token::End => return Err(Error { at: first.at, kind: ErrorKind::Expected(Expected::Operand) }),
    };
    let end = lexer.next(false)?;
    match end.token {
        Token::End => {}
        Token::Value(_) | Token::Word => {
            return Err(Error { at: end.at, kind: ErrorKind::Expected(Expected::Operator) });
        }
    }
    let mut out = List::with_capacity(1);
    out.push(Operation { at: first.at, op }).expect("one primary");
    Ok(Expression { operations: out.into_boxed(), values: 1 })
}
