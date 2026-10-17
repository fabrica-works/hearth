use crate::{fixtures::Shape, generate};
use skein_lib::Rng;
#[must_use]
pub fn text(rng: &mut Rng, shape: Shape) -> Vec<u8> {
    nested(rng, shape, 3)
}
fn nested(rng: &mut Rng, shape: Shape, depth: u32) -> Vec<u8> {
    let mut out = Vec::new();
    for _ in 0..=rng.below(u64::from(shape.width.min(8))) {
        const PIECES: &[&[u8]] =
            &[b"plain { }} %} -\n", b"{# {{ ' \" #}", b"{{ '}} %} #}' }}", b"{{- a -}}", b"{#+ x +#}"];
        if depth > 0 && rng.below(3) == 0 {
            let body = nested(rng, Shape { width: shape.width.min(3) }, depth - 1);
            match rng.below(4) {
                0 => {
                    out.extend_from_slice(b"{% if a %}");
                    out.extend(body);
                    out.extend_from_slice(b"{% elif b %}yes{% else %}no{% endif %}");
                }
                1 => {
                    out.extend_from_slice(b"{% for x in (a if c else b) %}{{ loop.index }}");
                    out.extend(body);
                    out.extend_from_slice(b"{% endfor %}");
                }
                2 => {
                    out.extend_from_slice(b"{% for k, v in a | items %}{{ loop.first }}");
                    out.extend(body);
                    out.extend_from_slice(b"{% endfor %}");
                }
                _ => out.extend_from_slice(b"{% set local = settings.port %}"),
            }
        } else if rng.below(3) == 0 {
            out.extend_from_slice(b"{{ ");
            out.extend(generate::text(rng, shape));
            out.extend_from_slice(b" }}");
        } else {
            out.extend_from_slice(PIECES[usize::try_from(rng.below(PIECES.len() as u64)).expect("piece index")]);
        }
    }
    out
}
#[must_use]
pub fn mutate(rng: &mut Rng, text: &[u8]) -> Vec<u8> {
    const PIECES: &[&[u8]] = &[
        b"{% set settings = 1 %}",
        b"{% endfor %}",
        b"{% for x in a %}",
        b"{% macro x %}",
        b"{{",
        b"}}",
        b"{%",
        b"%}",
        b"{#",
        b"#}",
        b"-",
        b"+",
        b"\r",
        b"\xff",
        b"\x80",
        b"{% macro x %}",
        b"{{ a | nope }}",
        b"{{ a is nope }}",
        b"{{ a | join }}",
        b"{{ 9223372036854775808 }}",
        b"{{ '@\\q' }}",
        b"{{ @ }}",
    ];
    let mut out = text.to_vec();
    let at = usize::try_from(rng.below(out.len() as u64 + 1)).expect("template offset");
    let piece = PIECES[usize::try_from(rng.below(PIECES.len() as u64)).expect("piece index")];
    match rng.below(3) {
        0 => {
            out.splice(at..at, piece.iter().copied());
        }
        1 if at < out.len() => {
            out.splice(at..=at, piece.iter().copied());
        }
        2 if at < out.len() => {
            out.remove(at);
        }
        _ => out.extend_from_slice(piece),
    }
    out
}
