use crate::fixtures;
use crate::fixtures::Shape;
use skein_lib::Rng;
#[must_use]
fn literal(rng: &mut Rng, shape: Shape) -> Vec<u8> {
    let value = match rng.below(6) {
        0 => format!("{}", rng.next_u64()),
        1 => String::from("false"),
        2 => String::from("true"),
        3 => format!("name_{}", rng.below(20)),
        _ => {
            let quote = if rng.below(2) == 0 { '\'' } else { '"' };
            let mut s = String::from(quote);
            let chars = ["a", "é", "\\n", "\\t", "\\\\", "\\'", "\\\""];
            for _ in 0..rng.below(u64::from(shape.width)) {
                s.push_str(chars[usize::try_from(rng.below(7)).expect("bounded by fixture or checked text")]);
            }
            s.push(quote);
            s
        }
    };
    // Keep generated integers within the grammar's i64 range.
    let value = if value.as_bytes()[0].is_ascii_digit() { format!("{}", rng.below(i64::MAX as u64)) } else { value };
    let spaces = ["", " ", "\t", "\n"];
    format!(
        "{}{}{}",
        spaces[usize::try_from(rng.below(4)).expect("bounded by fixture or checked text")],
        value,
        spaces[usize::try_from(rng.below(4)).expect("bounded by fixture or checked text")]
    )
    .into_bytes()
}
#[must_use]
pub fn cut(rng: &mut Rng, text: &[u8]) -> Vec<u8> {
    text[..usize::try_from(rng.below(text.len() as u64 + 1)).expect("bounded by fixture or checked text")].to_vec()
}
#[must_use]
pub fn mutate(rng: &mut Rng, text: &[u8]) -> Vec<u8> {
    let mut out = text.to_vec();
    let at = usize::try_from(rng.below(out.len() as u64 + 1)).expect("bounded by fixture or checked text");
    let pieces: &[&[u8]] =
        &[b"@", b"\r", b"'", b"\\q", b" and ", b" none ", b"9223372036854775808", b"[", b"\xff", b"\"", b" "];
    let piece = pieces[usize::try_from(rng.below(pieces.len() as u64)).expect("bounded by fixture or checked text")];
    match rng.below(4) {
        0 => {
            out.splice(at..at, piece.iter().copied());
        }
        1 if at < out.len() => {
            out.splice(at..=at, piece.iter().copied());
        }
        2 if at < out.len() => {
            out.remove(at);
        }
        _ => {
            let end = (at + 16).min(out.len());
            let span = out[at..end].to_vec();
            out.splice(at..at, span);
        }
    }
    out
}

/// Generates nested grammar forms, including filter arguments and lookups.
#[must_use]
pub fn text(rng: &mut Rng, shape: Shape) -> Vec<u8> {
    expression(rng, shape, if shape.width == fixtures::SMALL.width { 3 } else { 7 })
}
fn expression(rng: &mut Rng, shape: Shape, depth: u32) -> Vec<u8> {
    if depth == 0 {
        return literal(rng, shape);
    }
    let mut out = match rng.below(4) {
        0 => {
            let inner = expression(rng, shape, depth - 1);
            [b"(".as_slice(), &inner, b")"].concat()
        }
        1 => {
            let mut list = Vec::from(b"[".as_slice());
            for i in 0..rng.below(4) {
                if i != 0 {
                    list.push(b',');
                }
                list.extend(expression(rng, shape, depth - 1));
            }
            list.push(b']');
            list
        }
        _ => literal(rng, shape),
    };
    for _ in 0..rng.below(3) {
        if rng.below(2) == 0 {
            out.extend_from_slice(b".and");
        } else {
            out.push(b'[');
            out.extend(expression(rng, shape, depth - 1));
            out.push(b']');
        }
    }
    match rng.below(5) {
        0 => out.extend_from_slice(b" | first"),
        1 => {
            out.extend_from_slice(b" | default(");
            out.extend(expression(rng, shape, depth - 1));
            out.push(b')');
        }
        2 => out.extend_from_slice(b" | replace('a', 'b') | length()"),
        _ => {}
    }
    out
}
