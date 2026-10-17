//! Statement heads and the bounded open-block stack (format.md, 7.3).
use super::scan::{self, Tag};
use crate::{Does, Error, ErrorKind, Expected, Expression, Instruction, Names, expression, limits::Limits};
use alloc::boxed::Box;
use skein_lib::{List, Stack, bytes::copy_of};
#[derive(Debug)]
pub(crate) enum Block {
    If { branch: u32, jumps: u32, at: u32 },
    Else { jumps: u32, at: u32 },
    For { enter: u32, at: u32 },
}
pub(super) struct Statements {
    blocks: Stack<Block>,
    jumps: List<u32>,
}
impl Statements {
    pub(super) fn new(limits: &Limits, openings: u32) -> Self {
        Self { blocks: Stack::with_capacity(limits.statements.min(openings)), jumps: List::with_capacity(openings) }
    }
    pub(super) fn finish(&self) -> Result<(), Error> {
        match self.blocks.top() {
            Some(Block::If { at, .. } | Block::Else { at, .. } | Block::For { at, .. }) => {
                Err(Error { at: *at, kind: ErrorKind::Unended })
            }
            None => Ok(()),
        }
    }
    pub(super) fn read(
        &mut self,
        text: &[u8],
        at: usize,
        tag: &Tag,
        out: &mut List<Instruction>,
        limits: &Limits,
    ) -> Result<(), Error> {
        let word = skip(text, tag.start, tag.end);
        let end = word_end(text, word, tag.end);
        let head = text.get(word..end).expect("head within tag");
        let tail = skip(text, end, tag.end);
        let opening = scan::offset(at);
        if word == tag.end {
            return Err(error(word, ErrorKind::Expected(Expected::Statement)));
        }
        match head {
            b"if" => {
                let condition = super::expression(text, end, tag.end, limits)?;
                self.open(Block::If { branch: out.len(), jumps: self.jumps.len(), at: opening }, opening)?;
                emit(out, opening, Does::Branch { condition, over: 0 });
            }
            b"elif" | b"else" => {
                if head == b"else" {
                    empty(tail, tag.end)?;
                }
                let (branch, jumps, first) = match self.blocks.top() {
                    Some(Block::If { branch, jumps, at }) => (*branch, *jumps, *at),
                    Some(Block::Else { .. } | Block::For { .. }) | None => {
                        return Err(error(word, ErrorKind::Unmatched));
                    }
                };
                let condition =
                    if head == b"elif" { Some(super::expression(text, end, tag.end, limits)?) } else { None };
                self.jumps.push(out.len()).expect("one jump per elif or else tag");
                emit(out, opening, Does::Jump { over: 0 });
                patch(out, branch);
                self.blocks.pop().expect("open condition");
                let block = match condition {
                    Some(condition) => {
                        let branch = out.len();
                        emit(out, opening, Does::Branch { condition, over: 0 });
                        Block::If { branch, jumps, at: first }
                    }
                    None => Block::Else { jumps, at: first },
                };
                self.blocks.push(block).expect("replaced open condition");
            }
            b"endif" => {
                empty(tail, tag.end)?;
                let jumps = match self.blocks.pop() {
                    Some(Block::If { branch, jumps, .. }) => {
                        patch(out, branch);
                        jumps
                    }
                    Some(Block::Else { jumps, .. }) => jumps,
                    Some(Block::For { .. }) | None => return Err(error(word, ErrorKind::Unmatched)),
                };
                for index in jumps..self.jumps.len() {
                    patch(out, *self.jumps.get(index).expect("saved jump"));
                }
                self.jumps.truncate(jumps);
            }
            b"for" => {
                let (first, next) = name(text, tail, tag.end)?;
                let mut cursor = skip(text, next, tag.end);
                let names = if text.get(cursor) == Some(&b',') {
                    let (second, next) = name(text, cursor.saturating_add(1), tag.end)?;
                    cursor = skip(text, next, tag.end);
                    Names::Pair { first, second }
                } else {
                    Names::One { name: first }
                };
                let end_in = word_end(text, cursor, tag.end);
                if text.get(cursor..end_in) != Some(b"in") {
                    return Err(error(cursor, ErrorKind::Expected(Expected::In)));
                }
                let iterable = iterable(text, end_in, tag.end, limits)?;
                self.open(Block::For { enter: out.len(), at: opening }, opening)?;
                emit(out, opening, Does::Enter { names, iterable, over: 0 });
            }
            b"endfor" => {
                empty(tail, tag.end)?;
                let enter = match self.blocks.pop() {
                    Some(Block::For { enter, .. }) => enter,
                    Some(Block::If { .. } | Block::Else { .. }) | None => {
                        return Err(error(word, ErrorKind::Unmatched));
                    }
                };
                let back = out.len().checked_sub(enter).expect("body after enter").saturating_sub(1);
                emit(out, opening, Does::Advance { back });
                patch(out, enter);
            }
            b"set" => {
                let (name, next) = name(text, tail, tag.end)?;
                let assign = skip(text, next, tag.end);
                if text.get(assign) != Some(&b'=') {
                    return Err(error(assign, ErrorKind::Expected(Expected::Assign)));
                }
                let value = super::expression(text, assign.saturating_add(1), tag.end, limits)?;
                emit(out, opening, Does::Set { name, value });
            }
            _ => return Err(error(word, ErrorKind::UnknownStatement)),
        }
        Ok(())
    }
    fn open(&mut self, block: Block, at: u32) -> Result<(), Error> {
        match self.blocks.push(block) {
            Ok(()) => Ok(()),
            Err(_) => Err(Error { at, kind: ErrorKind::TooDeep }),
        }
    }
}
fn emit(out: &mut List<Instruction>, at: u32, does: Does) {
    out.push(Instruction { at, does }).expect("three instructions per opening plus text");
}
fn patch(out: &mut List<Instruction>, index: u32) {
    let over = out.len().checked_sub(index).expect("patch before end").saturating_sub(1);
    match &mut out.get_mut(index).expect("saved instruction").does {
        Does::Branch { over: skip, .. } | Does::Jump { over: skip } | Does::Enter { over: skip, .. } => *skip = over,
        Does::Text(_) | Does::Output(_) | Does::Advance { .. } | Does::Set { .. } => {
            unreachable!("only forward jumps are patched")
        }
    }
}
fn skip(text: &[u8], start: usize, end: usize) -> usize {
    let mut cursor = start;
    for &byte in text.get(start..end).expect("head range") {
        if ![b' ', b'\t', b'\n'].contains(&byte) {
            break;
        }
        cursor = cursor.saturating_add(1);
    }
    cursor
}
fn word_end(text: &[u8], start: usize, end: usize) -> usize {
    let mut cursor = start;
    for &byte in text.get(start..end).expect("word range") {
        if !byte.is_ascii_alphanumeric() && byte != b'_' {
            break;
        }
        cursor = cursor.saturating_add(1);
    }
    cursor
}
fn name(text: &[u8], start: usize, end: usize) -> Result<(Box<[u8]>, usize), Error> {
    let start = skip(text, start, end);
    let next = word_end(text, start, end);
    let value = text.get(start..next).expect("name range");
    let first = value.first().copied().unwrap_or(0);
    if !(first.is_ascii_alphabetic() || first == b'_')
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
        ]
        .contains(&value)
    {
        return Err(error(start, ErrorKind::Expected(Expected::Name)));
    }
    if [b"target".as_slice(), b"debian", b"settings", b"tasks", b"exports", b"key", b"value", b"loop"].contains(&value)
    {
        return Err(error(start, ErrorKind::Shadows));
    }
    Ok((copy_of(value), next))
}
fn empty(start: usize, end: usize) -> Result<(), Error> {
    if start != end {
        return Err(error(start, ErrorKind::Expected(Expected::TagEnd)));
    }
    Ok(())
}
fn error(at: usize, kind: ErrorKind) -> Error {
    Error { at: scan::offset(at), kind }
}
fn iterable(text: &[u8], start: usize, end: usize, limits: &Limits) -> Result<Expression, Error> {
    let offset = scan::offset(start);
    let mut value = match expression::iterable(text.get(start..end).expect("iterable body"), limits) {
        Ok(value) => value,
        Err(mut e) => {
            e.at = e.at.saturating_add(offset);
            return Err(e);
        }
    };
    for operation in &mut value.operations {
        operation.at = operation.at.saturating_add(offset);
    }
    Ok(value)
}
