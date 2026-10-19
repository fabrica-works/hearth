//! Byte validation and tag boundaries (protocol/format.md, 2.3, 7.2).
use crate::{Error, ErrorKind, Expected, limits::Limits, utf8::Utf8};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
pub(super) fn bytes(text: &[u8], limits: &Limits) -> Result<(Box<[u32]>, u32), Error> {
    if text.len() > usize::try_from(limits.template).expect("u32 fits usize") {
        return Err(Error { at: limits.template, kind: ErrorKind::TooLong });
    }
    let mut lines = List::with_capacity(bytes::count(text, b"\n", u32::MAX).saturating_add(1));
    lines.push(0).expect("room for first line");
    let mut utf8 = Utf8::Between;
    let mut openings = 0_u32;
    for (at, &byte) in text.iter().enumerate() {
        utf8 = utf8.next(byte).ok_or(Error { at: offset(at), kind: ErrorKind::Utf8 })?;
        if byte == b'\r' {
            return Err(Error { at: offset(at), kind: ErrorKind::Return });
        }
        if byte == b'\n' {
            lines.push(offset(at).saturating_add(1)).expect("counted lines");
        }
        if opening(text, at) {
            openings = openings.saturating_add(1);
        }
    }
    match utf8 {
        Utf8::Between => Ok((lines.into_boxed(), openings)),
        Utf8::Within { .. } => Err(Error { at: offset(text.len()), kind: ErrorKind::Utf8 }),
    }
}
pub(crate) fn opening(text: &[u8], at: usize) -> bool {
    text.get(at) == Some(&b'{') && [Some(&b'{'), Some(&b'%'), Some(&b'#')].contains(&text.get(at.saturating_add(1)))
}
#[derive(Debug)]
pub(crate) struct Tag {
    pub start: usize,
    pub end: usize,
    pub after: usize,
    pub kind: u8,
    pub close: Option<u8>,
}
pub(crate) fn tag(text: &[u8], at: usize) -> Result<Tag, Error> {
    let kind = *text.get(at.saturating_add(1)).expect("opening kind");
    let mut start = at.saturating_add(2);
    if marker(text.get(start).copied(), kind) {
        start = start.saturating_add(1);
    }
    let mut quote = None;
    let mut escaped = false;
    let mut quote_at = start;
    for end in start..text.len() {
        let byte = *text.get(end).expect("scan within text");
        if kind != b'#' {
            if let Some(twin) = quote {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == twin {
                    quote = None;
                }
                continue;
            }
            if byte == b'\'' || byte == b'"' {
                quote = Some(byte);
                quote_at = end;
                continue;
            }
        }
        let closes = text.get(end.saturating_add(1)) == Some(&b'}')
            && if kind == b'#' { byte == b'#' } else { byte == b'}' || byte == b'%' };
        if !closes {
            continue;
        }
        if byte != if kind == b'{' { b'}' } else { kind } {
            return Err(Error { at: offset(end), kind: ErrorKind::Expected(Expected::TagEnd) });
        }
        let before = end.saturating_sub(1);
        let body_end = if end > start && marker(text.get(before).copied(), kind) { before } else { end };
        let close = if body_end < end { text.get(body_end).copied() } else { None };
        return Ok(Tag { start, end: body_end, after: end.saturating_add(2), kind, close });
    }
    Err(match quote {
        Some(_) => Error { at: offset(quote_at), kind: ErrorKind::Unterminated },
        None => Error { at: offset(at), kind: ErrorKind::Unclosed },
    })
}
fn marker(byte: Option<u8>, kind: u8) -> bool {
    byte == Some(b'-') || (kind != b'{' && byte == Some(b'+'))
}
pub(crate) fn offset(at: usize) -> u32 {
    u32::try_from(at).expect("validated template length")
}
