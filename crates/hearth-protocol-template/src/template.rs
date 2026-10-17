//! Templates compiled left to right to flat instructions (format.md, 7.2; render.md, 8.3).
mod raw;
mod scan;
mod statement;
pub(crate) mod whitespace;
use crate::{Does, Error, Expression, Instruction, Template, expression, limits::Limits};
use skein_lib::{List, bytes};
/// Compiles a whole template, or gives its first refusal at a template offset.
pub fn compile(text: &[u8], limits: &Limits) -> Result<Template, Error> {
    let (lines, openings) = scan::bytes(text, limits)?;
    let capacity = openings.saturating_mul(3).saturating_add(1);
    let mut out = List::with_capacity(capacity);
    let mut statements = statement::Statements::new(limits, openings);
    let mut cursor = 0;
    for _ in 0..=text.len() {
        let mut next = None;
        let mut search = cursor;
        for _ in cursor..text.len() {
            let Some(at) = bytes::find_from(text, b"{", search) else {
                break;
            };
            if scan::opening(text, at) {
                next = Some(at);
                break;
            }
            search = at.saturating_add(1);
        }
        let at = next.unwrap_or(text.len());
        let kind = text.get(at.saturating_add(1)).copied().unwrap_or(b'{');
        let marker = text.get(at.saturating_add(2)).copied();
        let end = if at < text.len() { whitespace::before(text, cursor, at, kind, marker) } else { at };
        if end > cursor {
            let kept = text.get(cursor..end).expect("text run within template");
            out.push(Instruction { at: scan::offset(cursor), does: Does::Text(bytes::copy_of(kept)) })
                .expect("counted runs");
        }
        if at == text.len() {
            statements.finish()?;
            return Ok(Template { instructions: out.into_boxed(), lines });
        }
        if let Some((start, end, tag)) = raw::read(text, at)? {
            if end > start {
                out.push(Instruction {
                    at: scan::offset(start),
                    does: Does::Text(bytes::copy_of(text.get(start..end).expect("raw body"))),
                })
                .expect("counted raw body");
            }
            cursor = whitespace::after(text, tag.after, tag.kind, tag.close);
            continue;
        }
        let tag = scan::tag(text, at)?;
        match tag.kind {
            b'{' => {
                let value = expression(text, tag.start, tag.end, limits)?;
                out.push(Instruction { at: scan::offset(at), does: Does::Output(value) }).expect("counted outputs");
            }
            b'#' => {}
            b'%' => statements.read(text, at, &tag, &mut out, limits)?,
            _ => unreachable!("opening kinds checked"),
        }
        cursor = whitespace::after(text, tag.after, tag.kind, tag.close);
    }
    unreachable!("each tag advances the cursor")
}
fn expression(text: &[u8], start: usize, end: usize, limits: &Limits) -> Result<Expression, Error> {
    let offset = scan::offset(start);
    let mut compiled = match expression::compile(text.get(start..end).expect("tag body"), limits) {
        Ok(value) => value,
        Err(mut error) => {
            error.at = error.at.saturating_add(offset);
            return Err(error);
        }
    };
    for operation in &mut compiled.operations {
        operation.at = operation.at.saturating_add(offset);
    }
    Ok(compiled)
}
/// One-based line and byte column, counting only line feeds.
#[must_use]
pub fn place(text: &[u8], at: u32) -> (u32, u32) {
    let mut line = 1_u32;
    let mut first = 0_u32;
    for (offset, &byte) in text.iter().enumerate().take(usize::try_from(at).expect("u32 fits usize")) {
        if byte == b'\n' {
            line = line.saturating_add(1);
            first = scan::offset(offset).saturating_add(1);
        }
    }
    (line, at.saturating_sub(first).saturating_add(1))
}
