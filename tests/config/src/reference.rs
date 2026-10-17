//! Independent recursive reading. JSON syntax faults are left to skein's oracle;
//! hearth's own faults and every successful value carry independently counted positions.
use hearth_protocol_config::{
    Error, Fault, Position,
    json::{Entry, Kind, Value},
    limits::Limits,
};
use std::collections::BTreeMap;

/// Reads a whole document independently.
///
/// # Errors
/// Returns the first JSON or hearth configuration fault.
pub fn decode(bytes: &[u8], limits: &Limits) -> Result<Value, Error> {
    let mut parser = Parser { bytes, pos: 0, limits };
    parser.ws();
    let root = parser.value(0, true, false)?;
    parser.ws();
    if parser.pos != bytes.len() {
        return Err(parser.json());
    }
    Ok(root)
}
struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
    limits: &'a Limits,
}
impl Parser<'_> {
    fn at(&self, offset: usize) -> Position {
        position(self.bytes, offset)
    }
    fn json(&self) -> Error {
        Error { at: self.at(self.pos), fault: Fault::Json(skein_json::tokenizer::Error::Unexpected) }
    }
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }
    fn read(&mut self) -> Result<u8, Error> {
        if self.pos >= self.limits.document as usize {
            return Err(self.json());
        }
        let byte = self.peek().ok_or_else(|| self.json())?;
        self.pos += 1;
        Ok(byte)
    }
    fn ws(&mut self) {
        while self.peek().is_some_and(|b| b" \t\r\n".contains(&b)) && self.pos < self.limits.document as usize {
            self.pos += 1;
        }
    }
    fn exact(&mut self, byte: u8) -> Result<(), Error> {
        if self.read()? != byte {
            return Err(self.json());
        }
        Ok(())
    }
    fn fault(&self, offset: usize, fault: Fault) -> Error {
        Error { at: self.at(offset), fault }
    }
    fn value(&mut self, depth: u32, root: bool, note: bool) -> Result<Value, Error> {
        self.ws();
        let start = self.pos;
        let byte = self.peek().ok_or_else(|| self.json())?;
        let kind = match byte {
            b'{' | b'[' => {
                self.read()?;
                if depth >= self.limits.depth {
                    return Err(self.json());
                }
                if root && byte != b'{' {
                    return Err(self.fault(start, Fault::NotObject));
                }
                if note {
                    return Err(self.fault(start, Fault::Note));
                }
                if byte == b'{' { self.object(depth + 1)? } else { self.array(depth + 1)? }
            }
            b'"' => {
                let text = self.string()?;
                if root {
                    return Err(self.fault(start, Fault::NotObject));
                }
                if text.contains(&0) {
                    return Err(self.fault(start, Fault::Nul));
                }
                Kind::String(text.into_boxed_slice())
            }
            b't' | b'f' | b'n' => {
                let word: &[u8] = match byte {
                    b't' => b"true",
                    b'f' => b"false",
                    b'n' => b"null",
                    _ => unreachable!(),
                };
                for &b in word {
                    self.exact(b)?;
                }
                if root {
                    return Err(self.fault(start, Fault::NotObject));
                }
                if note {
                    return Err(self.fault(start, Fault::Note));
                }
                match byte {
                    b't' => Kind::Bool(true),
                    b'f' => Kind::Bool(false),
                    b'n' => Kind::Null,
                    _ => unreachable!(),
                }
            }
            b'-' | b'0'..=b'9' => {
                let text = self.number()?;
                if self.pos >= self.limits.document as usize || (!root && self.peek().is_none()) {
                    return Err(self.json());
                }
                if root {
                    return Err(self.fault(start, Fault::NotObject));
                }
                if note {
                    return Err(self.fault(start, Fault::Note));
                }
                if text.contains('.') || text.contains('e') || text.contains('E') {
                    return Err(self.fault(start, Fault::NotInteger));
                }
                Kind::Int(text.parse::<i64>().map_err(|_| self.fault(start, Fault::Range))?)
            }
            _ => return Err(self.json()),
        };
        Ok(Value { at: self.at(start), kind })
    }
    fn object(&mut self, depth: u32) -> Result<Kind, Error> {
        let mut keys = BTreeMap::new();
        let mut output = BTreeMap::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.read()?;
            return Ok(Kind::Map(Box::new([])));
        }
        loop {
            self.ws();
            let start = self.pos;
            let key = self.string()?;
            if key.contains(&0) {
                return Err(self.fault(start, Fault::Nul));
            }
            if let Some(first) = keys.get(&key) {
                return Err(self.fault(start, Fault::Duplicate { key: key.into_boxed_slice(), first: *first }));
            }
            if keys.len() >= self.limits.keys as usize {
                return Err(self.fault(start, Fault::Keys));
            }
            keys.insert(key.clone(), self.at(start));
            self.ws();
            self.exact(b':')?;
            let note = key == b"_note";
            let value = self.value(depth, false, note)?;
            if !note {
                output.insert(key.clone(), Entry { key: key.into_boxed_slice(), at: self.at(start), value });
            }
            self.ws();
            match self.read()? {
                b'}' => break,
                b',' => {}
                _ => return Err(self.json()),
            }
        }
        Ok(Kind::Map(output.into_values().collect()))
    }
    fn array(&mut self, depth: u32) -> Result<Kind, Error> {
        let mut values = Vec::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.read()?;
            return Ok(Kind::List(Box::new([])));
        }
        loop {
            values.push(self.value(depth, false, false)?);
            self.ws();
            match self.read()? {
                b']' => break,
                b',' => {}
                _ => return Err(self.json()),
            }
        }
        Ok(Kind::List(values.into_boxed_slice()))
    }
    fn string(&mut self) -> Result<Vec<u8>, Error> {
        self.exact(b'"')?;
        let mut bytes = Vec::new();
        loop {
            match self.read()? {
                b'"' => {
                    std::str::from_utf8(&bytes).map_err(|_| self.json())?;
                    return Ok(bytes);
                }
                b'\\' => match self.read()? {
                    b'"' => bytes.push(b'"'),
                    b'\\' => bytes.push(b'\\'),
                    b'/' => bytes.push(b'/'),
                    b'b' => bytes.push(8),
                    b'f' => bytes.push(12),
                    b'n' => bytes.push(10),
                    b'r' => bytes.push(13),
                    b't' => bytes.push(9),
                    b'u' => {
                        let high = self.hex()?;
                        let code = if (0xd800..=0xdbff).contains(&high) {
                            self.exact(b'\\')?;
                            self.exact(b'u')?;
                            let low = self.hex()?;
                            if !(0xdc00..=0xdfff).contains(&low) {
                                return Err(self.json());
                            }
                            0x10000 + ((high - 0xd800) << 10) + low - 0xdc00
                        } else {
                            high
                        };
                        let ch = char::from_u32(code).ok_or_else(|| self.json())?;
                        bytes.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes());
                    }
                    _ => return Err(self.json()),
                },
                0..=31 => return Err(self.json()),
                byte => bytes.push(byte),
            }
        }
    }
    fn hex(&mut self) -> Result<u32, Error> {
        let mut code = 0;
        for _ in 0..4 {
            let digit = char::from(self.read()?).to_digit(16).ok_or_else(|| self.json())?;
            code = code * 16 + digit;
        }
        Ok(code)
    }
    fn number(&mut self) -> Result<String, Error> {
        let start = self.pos;
        while self.peek().is_some_and(|b| b"0123456789.eE+-".contains(&b)) {
            self.read()?;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos]).expect("ASCII");
        let mut chars = text.bytes().peekable();
        if chars.peek() == Some(&b'-') {
            chars.next();
        }
        match chars.next() {
            Some(b'0') => {
                if chars.peek().is_some_and(u8::is_ascii_digit) {
                    return Err(self.json());
                }
            }
            Some(b'1'..=b'9') => {
                while chars.peek().is_some_and(u8::is_ascii_digit) {
                    chars.next();
                }
            }
            _ => return Err(self.json()),
        }
        if chars.peek() == Some(&b'.') {
            chars.next();
            if !chars.peek().is_some_and(u8::is_ascii_digit) {
                return Err(self.json());
            }
            while chars.peek().is_some_and(u8::is_ascii_digit) {
                chars.next();
            }
        }
        if chars.peek().is_some_and(|b| *b == b'e' || *b == b'E') {
            chars.next();
            if chars.peek().is_some_and(|b| *b == b'+' || *b == b'-') {
                chars.next();
            }
            if !chars.peek().is_some_and(u8::is_ascii_digit) {
                return Err(self.json());
            }
            while chars.peek().is_some_and(u8::is_ascii_digit) {
                chars.next();
            }
        }
        if chars.next().is_some() {
            return Err(self.json());
        }
        Ok(text.to_owned())
    }
}
#[must_use]
pub fn position(bytes: &[u8], offset: usize) -> Position {
    let prefix = &bytes[..offset.min(bytes.len())];
    let mut line = 1;
    let mut column = 1;
    for byte in prefix {
        if *byte == b'\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    Position { offset: u32::try_from(offset).expect("document bound"), line, column }
}
