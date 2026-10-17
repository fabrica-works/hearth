//! Direct reading of format.md, 7.4; shares only the output vocabulary.
use hearth_protocol_template::{Expression, Op, Operation, limits::Limits};
/// Compiles with a recursive grammar reference, returning the first offset.
///
/// # Errors
/// Gives a byte offset when the text is outside the grammar or its bounds.
pub fn compile(text: &[u8], limits: &Limits) -> Result<Expression, u32> {
    if text.len() > usize::try_from(limits.expression).expect("bounded by fixture or checked text") {
        return Err(limits.expression);
    }
    let mut parser = Parser { text, at: 0 };
    parser.space();
    let at = parser.at;
    let op = parser.primary()?;
    parser.space();
    if parser.at != text.len() {
        let trailing = parser.at;
        parser.token(false)?;
        return Err(u32::try_from(trailing).expect("text length checked"));
    }
    Ok(Expression {
        operations: vec![Operation { at: u32::try_from(at).expect("bounded by fixture or checked text"), op }]
            .into_boxed_slice(),
        values: 1,
    })
}
struct Parser<'a> {
    text: &'a [u8],
    at: usize,
}
impl Parser<'_> {
    fn space(&mut self) {
        while let Some(b' ' | b'\t' | b'\n') = self.text.get(self.at) {
            self.at += 1;
        }
    }
    fn primary(&mut self) -> Result<Op, u32> {
        self.token(true)
    }
    fn token(&mut self, operand: bool) -> Result<Op, u32> {
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
