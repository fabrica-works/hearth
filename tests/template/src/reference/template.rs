//! Recursive descent over template blocks, independent of the machine's stack.
use hearth_protocol_template::{Does, Expression, Instruction, Names, Template, limits::Limits};
/// Reads a whole template using recursive blocks and the expression reference.
/// # Errors
/// Returns the first refused byte offset.
pub fn compile(text: &[u8], limits: &Limits) -> Result<Template, u32> {
    if text.len() > limits.template as usize {
        return Err(limits.template);
    }
    validate(text)?;
    let lines = std::iter::once(0)
        .chain(text.iter().enumerate().filter(|(_, b)| **b == b'\n').map(|(at, _)| pos(at + 1)))
        .collect();
    let mut parser = Parser { text, cursor: 0, limits };
    let (instructions, close) = parser.body(0)?;
    if let Some(close) = close {
        return Err(pos(close.word));
    }
    Ok(Template { instructions: instructions.into_boxed_slice(), lines })
}
fn validate(text: &[u8]) -> Result<(), u32> {
    let mut at = 0;
    while at < text.len() {
        let lead = text[at];
        if lead == b'\r' {
            return Err(u32::try_from(at).expect("validated template length"));
        }
        if lead < 128 {
            at += 1;
            continue;
        }
        let width = match lead {
            0xc2..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf4 => 4,
            _ => return Err(u32::try_from(at).expect("validated template length")),
        };
        for n in 1..width {
            let Some(&byte) = text.get(at + n) else {
                return Err(u32::try_from(text.len()).expect("validated template length"));
            };
            let range = match (lead, n) {
                (0xe0, 1) => 0xa0..=0xbf,
                (0xed, 1) => 0x80..=0x9f,
                (0xf0, 1) => 0x90..=0xbf,
                (0xf4, 1) => 0x80..=0x8f,
                _ => 0x80..=0xbf,
            };
            if !range.contains(&byte) {
                return Err(u32::try_from(at + n).expect("validated template length"));
            }
        }
        at += width;
    }
    Ok(())
}

fn pos(at: usize) -> u32 {
    u32::try_from(at).expect("validated template length")
}
struct Parser<'a> {
    text: &'a [u8],
    cursor: usize,
    limits: &'a Limits,
}
#[derive(Clone, Copy)]
struct Tag {
    at: usize,
    end: usize,
    word: usize,
    head_end: usize,
}
impl Parser<'_> {
    fn skip(&self, mut at: usize, end: usize) -> usize {
        while at < end && b" \t\n".contains(&self.text[at]) {
            at += 1;
        }
        at
    }
    fn word_end(&self, mut at: usize, end: usize) -> usize {
        while at < end && (self.text[at].is_ascii_alphanumeric() || self.text[at] == b'_') {
            at += 1;
        }
        at
    }
    fn expr(&self, start: usize, end: usize, iterable: bool) -> Result<Expression, u32> {
        let mut value = if iterable {
            super::iterable(&self.text[start..end], self.limits)
        } else {
            super::compile(&self.text[start..end], self.limits)
        }
        .map_err(|e| e + pos(start))?;
        for op in &mut value.operations {
            op.at += pos(start);
        }
        Ok(value)
    }
    fn next(&mut self, out: &mut Vec<Instruction>) -> Result<Option<Tag>, u32> {
        while self.cursor < self.text.len() {
            let at = (self.cursor..self.text.len().saturating_sub(1))
                .find(|&i| self.text[i] == b'{' && b"{%#".contains(&self.text[i + 1]))
                .unwrap_or(self.text.len());
            if at > self.cursor {
                out.push(Instruction { at: pos(self.cursor), does: Does::Text(self.text[self.cursor..at].into()) });
            }
            if at == self.text.len() {
                self.cursor = at;
                return Ok(None);
            }
            let kind = self.text[at + 1];
            let marker = |b| b == b'-' || (kind != b'{' && b == b'+');
            let start = at + 2 + usize::from(self.text.get(at + 2).copied().is_some_and(marker));
            let mut end = start;
            loop {
                if end >= self.text.len() {
                    return Err(pos(at));
                }
                if kind != b'#' && b"\'\"".contains(&self.text[end]) {
                    let quote_at = end;
                    let quote = self.text[end];
                    end += 1;
                    while end < self.text.len() && self.text[end] != quote {
                        end += if self.text[end] == b'\\' { 2 } else { 1 };
                    }
                    if end >= self.text.len() {
                        return Err(pos(quote_at));
                    }
                } else if self.text.get(end + 1) == Some(&b'}')
                    && if kind == b'#' { self.text[end] == b'#' } else { b"}%".contains(&self.text[end]) }
                {
                    if self.text[end] != if kind == b'{' { b'}' } else { kind } {
                        return Err(pos(end));
                    }
                    break;
                }
                end += 1;
            }
            self.cursor = end + 2;
            let end = end - usize::from(end > start && marker(self.text[end - 1]));
            if kind == b'{' {
                out.push(Instruction { at: pos(at), does: Does::Output(self.expr(start, end, false)?) });
            } else if kind == b'%' {
                let word = self.skip(start, end);
                return Ok(Some(Tag { at, end, word, head_end: self.word_end(word, end) }));
            }
        }
        Ok(None)
    }
    fn body(&mut self, depth: u32) -> Result<(Vec<Instruction>, Option<Tag>), u32> {
        let mut out = Vec::new();
        while let Some(tag) = self.next(&mut out)? {
            match &self.text[tag.word..tag.head_end] {
                b"elif" | b"else" | b"endif" | b"endfor" => {
                    if &self.text[tag.word..tag.head_end] != b"elif" && self.skip(tag.head_end, tag.end) != tag.end {
                        return Err(pos(self.skip(tag.head_end, tag.end)));
                    }
                    return Ok((out, Some(tag)));
                }
                b"if" => {
                    let condition = self.expr(tag.head_end, tag.end, false)?;
                    if depth == self.limits.statements {
                        return Err(pos(tag.at));
                    }
                    let opened = tag.at;
                    out.extend(self.condition(tag, condition, depth + 1, opened)?);
                }
                b"for" => {
                    let (first, next) = self.name(self.skip(tag.head_end, tag.end), tag.end)?;
                    let mut next = self.skip(next, tag.end);
                    let names = if self.text.get(next) == Some(&b',') {
                        let (second, end) = self.name(self.skip(next + 1, tag.end), tag.end)?;
                        next = self.skip(end, tag.end);
                        Names::Pair { first, second }
                    } else {
                        Names::One { name: first }
                    };
                    let end = self.word_end(next, tag.end);
                    if &self.text[next..end] != b"in" {
                        return Err(pos(next));
                    }
                    let iterable = self.expr(end, tag.end, true)?;
                    if depth == self.limits.statements {
                        return Err(pos(tag.at));
                    }
                    let (body, close) = self.body(depth + 1)?;
                    let close = close.ok_or(pos(tag.at))?;
                    if &self.text[close.word..close.head_end] != b"endfor" {
                        return Err(pos(close.word));
                    }
                    out.push(Instruction {
                        at: pos(tag.at),
                        does: Does::Enter { names, iterable, over: pos(body.len() + 1) },
                    });
                    let back = pos(body.len());
                    out.extend(body);
                    out.push(Instruction { at: pos(close.at), does: Does::Advance { back } });
                }
                b"set" => {
                    let (name, next) = self.name(self.skip(tag.head_end, tag.end), tag.end)?;
                    let assign = self.skip(next, tag.end);
                    if self.text.get(assign) != Some(&b'=') {
                        return Err(pos(assign));
                    }
                    out.push(Instruction {
                        at: pos(tag.at),
                        does: Does::Set { name, value: self.expr(assign + 1, tag.end, false)? },
                    });
                }
                _ => return Err(pos(tag.word)),
            }
        }
        Ok((out, None))
    }
    fn condition(
        &mut self,
        tag: Tag,
        condition: Expression,
        depth: u32,
        opened: usize,
    ) -> Result<Vec<Instruction>, u32> {
        let (body, close) = self.body(depth)?;
        let close = close.ok_or(pos(opened))?;
        let close_at = close.at;
        let head = &self.text[close.word..close.head_end];
        let tail = match head {
            b"endif" => Vec::new(),
            b"elif" => {
                let condition = self.expr(close.head_end, close.end, false)?;
                self.condition(close, condition, depth, opened)?
            }
            b"else" => {
                let (body, end) = self.body(depth)?;
                let end = end.ok_or(pos(opened))?;
                if &self.text[end.word..end.head_end] != b"endif" {
                    return Err(pos(end.word));
                }
                body
            }
            _ => return Err(pos(close.word)),
        };
        let mut out = vec![Instruction {
            at: pos(tag.at),
            does: Does::Branch { condition, over: pos(body.len() + usize::from(head != b"endif")) },
        }];
        out.extend(body);
        if head != b"endif" {
            out.push(Instruction { at: pos(close_at), does: Does::Jump { over: pos(tail.len()) } });
        }
        out.extend(tail);
        Ok(out)
    }
    fn name(&self, start: usize, end: usize) -> Result<(Box<[u8]>, usize), u32> {
        let next = self.word_end(start, end);
        let name = &self.text[start..next];
        if name.is_empty()
            || !(name[0].is_ascii_alphabetic() || name[0] == b'_')
            || [
                b"and".as_slice(),
                b"or",
                b"not",
                b"if",
                b"else",
                b"is",
                b"in",
                b"true",
                b"false",
                b"none",
                b"None",
                b"True",
                b"False",
                b"target",
                b"debian",
                b"settings",
                b"tasks",
                b"exports",
                b"key",
                b"value",
                b"loop",
            ]
            .contains(&name)
        {
            return Err(pos(start));
        }
        Ok((name.into(), next))
    }
}
