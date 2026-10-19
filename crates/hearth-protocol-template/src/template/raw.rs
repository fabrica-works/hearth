//! Raw tags use ASCII whitespace inside and ordinary edge rules outside (protocol/format.md, 7.3).
use super::{
    scan::{self, Tag},
    whitespace,
};
use crate::{Error, ErrorKind, Expected};
fn skip(text: &[u8], mut at: usize) -> usize {
    for &byte in text.get(at..).expect("raw cursor") {
        if !byte.is_ascii_whitespace() {
            break;
        }
        at = at.saturating_add(1);
    }
    at
}
fn head(text: &[u8], at: usize) -> usize {
    let mut start = at.saturating_add(2);
    if [Some(&b'-'), Some(&b'+')].contains(&text.get(start)) {
        start = start.saturating_add(1);
    }
    skip(text, start)
}
fn end(text: &[u8], start: usize) -> Option<Tag> {
    let mut at = skip(text, start);
    let close = match text.get(at).copied() {
        Some(b'-' | b'+') => {
            let marker = text.get(at).copied();
            at = at.saturating_add(1);
            marker
        }
        Some(_) | None => None,
    };
    if !text.get(at..)?.starts_with(b"%}") {
        return None;
    }
    Some(Tag { start, end: start, after: at.saturating_add(2), kind: b'%', close })
}
pub(super) fn read(text: &[u8], at: usize) -> Result<Option<(usize, usize, Tag)>, Error> {
    if text.get(at.saturating_add(1)) != Some(&b'%') {
        return Ok(None);
    }
    let word = head(text, at);
    let rest = text.get(word..).expect("raw head");
    if !rest.starts_with(b"raw") {
        return Ok(None);
    }
    let next = word.saturating_add(3);
    if let Some(byte) = text.get(next)
        && (byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        return Ok(None);
    }
    let opened = end(text, next)
        .ok_or(Error { at: scan::offset(skip(text, next)), kind: ErrorKind::Expected(Expected::TagEnd) })?;
    let start = whitespace::after(text, opened.after, b'%', opened.close);
    let mut search = opened.after;
    for _ in opened.after..text.len() {
        let Some(close_at) = skein_lib::bytes::find_from(text, b"{%", search) else {
            break;
        };
        search = close_at.saturating_add(2);
        let word = head(text, close_at);
        if !text.get(word..).expect("endraw head").starts_with(b"endraw") {
            continue;
        }
        let Some(closed) = end(text, word.saturating_add(6)) else {
            continue;
        };
        let marker = text.get(close_at.saturating_add(2)).copied();
        let start = start.min(close_at);
        let body = text.get(start..close_at).expect("raw body");
        let kept_end = start.saturating_add(whitespace::before(body, 0, body.len(), b'%', marker));
        return Ok(Some((start.min(kept_end), kept_end, closed)));
    }
    Err(Error { at: scan::offset(at), kind: ErrorKind::Unclosed })
}
