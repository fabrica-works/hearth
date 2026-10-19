//! Decoded declaration strings compiled without template whitespace rules (protocol/format.md, 5.2).
use crate::{
    Does, Error, ErrorKind, Instruction,
    limits::Limits,
    template::{self, scan},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
/// A decoded string read, without interpreting the field's type.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Interpolated {
    /// No delimiter opening; retain the original text.
    Plain,
    /// Text and outputs in their original order.
    Parts(Box<[Instruction]>),
}
/// Compiles UTF-8, unescaped text supplied by the configuration decoder.
pub fn compile(text: &[u8], limits: &Limits) -> Result<Interpolated, Error> {
    let openings = bytes::count(text, b"{", u32::MAX);
    let mut out = List::with_capacity(crate::limits::instruction_capacity(openings));
    let mut cursor = 0;
    let mut plain = true;
    for at in 0..text.len() {
        if at < cursor || !scan::opening(text, at) {
            continue;
        }
        plain = false;
        if text.get(at.saturating_add(1)) != Some(&b'{') {
            return Err(Error { at: scan::offset(at), kind: ErrorKind::Interpolation });
        }
        if text.get(at.saturating_add(2)) == Some(&b'-') {
            return Err(Error { at: scan::offset(at.saturating_add(2)), kind: ErrorKind::Interpolation });
        }
        let tag = scan::tag(text, at)?;
        if tag.close == Some(b'-') {
            return Err(Error { at: scan::offset(tag.end), kind: ErrorKind::Interpolation });
        }
        if cursor < at {
            out.push(Instruction {
                at: scan::offset(cursor),
                does: Does::Text(bytes::copy_of(text.get(cursor..at).expect("string text"))),
            })
            .expect("counted text runs");
        }
        let value = template::expression(text, tag.start, tag.end, limits)?;
        out.push(Instruction { at: scan::offset(at), does: Does::Output(value) }).expect("counted outputs");
        cursor = tag.after;
    }
    if plain {
        return Ok(Interpolated::Plain);
    }
    if cursor < text.len() {
        out.push(Instruction {
            at: scan::offset(cursor),
            does: Does::Text(bytes::copy_of(text.get(cursor..).expect("string tail"))),
        })
        .expect("counted tail");
    }
    Ok(Interpolated::Parts(out.into_boxed()))
}
