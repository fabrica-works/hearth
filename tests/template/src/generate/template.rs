use crate::{fixtures::Shape, generate};
use skein_lib::Rng;
#[must_use]
pub fn text(rng: &mut Rng, shape: Shape) -> Vec<u8> {
    nested(rng, shape, 3)
}
fn nested(rng: &mut Rng, shape: Shape, depth: u32) -> Vec<u8> {
    let mut out = Vec::new();
    for _ in 0..=rng.below(u64::from(shape.width.min(8))) {
        const PIECES: &[&[u8]] = &[
            b"plain { }} %} -\n",
            b"{# {{ ' \" #}",
            b"{{ '}} %} #}' }}",
            b"{{- a -}}",
            b"{#+ x +#}",
            b"{% raw %}{{ {% {% endif %} {%endraw{% endraw %}",
            b"{% raw -%} \n x \n {%- endraw +%}\n",
        ];
        if depth > 0 && rng.below(3) == 0 {
            let body = nested(rng, Shape { width: shape.width.min(3) }, depth - 1);
            match rng.below(4) {
                0 => {
                    out.extend(tag(rng, b"{% if a %}"));
                    out.extend(body);
                    out.extend(tag(rng, b"{% elif b %}"));
                    out.extend_from_slice(b"yes");
                    out.extend(tag(rng, b"{% else %}"));
                    out.extend_from_slice(b"no");
                    out.extend(tag(rng, b"{% endif %}"));
                }
                1 => {
                    out.extend(tag(rng, b"{% for x in (a if c else b) %}"));
                    out.extend(tag(rng, b"{{ loop.index }}"));
                    out.extend(body);
                    out.extend(tag(rng, b"{% endfor %}"));
                }
                2 => {
                    out.extend(tag(rng, b"{% for k, v in a | items %}"));
                    out.extend(tag(rng, b"{{ loop.first }}"));
                    out.extend(body);
                    out.extend(tag(rng, b"{% endfor %}"));
                }
                _ => out.extend(tag(rng, b"{% set local = settings.port %}")),
            }
        } else if rng.below(3) == 0 {
            let value = [b"{{ ".as_slice(), &generate::text(rng, shape), b" }}"].concat();
            out.extend(tag(rng, &value));
        } else {
            let piece = PIECES[usize::try_from(rng.below(PIECES.len() as u64)).expect("piece index")];
            if piece.starts_with(b"{% raw") || !piece.starts_with(b"{") {
                out.extend_from_slice(piece);
            } else {
                out.extend(tag(rng, piece));
            }
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

fn tag(rng: &mut Rng, source: &[u8]) -> Vec<u8> {
    const SPACES: &[&[u8]] = &[b"", b" ", b"\t", b"\n", b"\x0b", b"\x0c", b"\xc2\xa0", b"\xe3\x80\x80"];
    const INSIDE: &[&[u8]] = &[b" ", b"\t", b"\n"];
    let mut out = SPACES[usize::try_from(rng.below(SPACES.len() as u64)).expect("space index")].to_vec();
    out.extend_from_slice(&source[..2]);
    let markers: &[&[u8]] = if source[1] == b'{' { &[b"", b"-"] } else { &[b"", b"-", b"+"] };
    out.extend_from_slice(markers[usize::try_from(rng.below(markers.len() as u64)).expect("marker index")]);
    out.extend_from_slice(INSIDE[usize::try_from(rng.below(3)).expect("separator index")]);
    let body = &source[2..source.len() - 2];
    let body = body.strip_prefix(b"-").or_else(|| body.strip_prefix(b"+")).unwrap_or(body);
    out.extend_from_slice(body.strip_suffix(b"-").or_else(|| body.strip_suffix(b"+")).unwrap_or(body));
    out.extend_from_slice(INSIDE[usize::try_from(rng.below(3)).expect("separator index")]);
    out.extend_from_slice(markers[usize::try_from(rng.below(markers.len() as u64)).expect("marker index")]);
    out.extend_from_slice(&source[source.len() - 2..]);
    out.extend_from_slice(SPACES[usize::try_from(rng.below(SPACES.len() as u64)).expect("space index")]);
    out
}

/// Declaration keys and fields, including one output standing alone.
#[must_use]
pub fn string(rng: &mut Rng, shape: Shape) -> Vec<u8> {
    const PIECES: &[&[u8]] = &[b"/home/", b"/.ssh", b" \t\n\r", b"\xc2\xa0", b"${{ '{#' }}a[@]}", b"{ plain"];
    let mut out = Vec::new();
    for _ in 0..=rng.below(u64::from(shape.width.min(8))) {
        if rng.below(2) == 0 {
            out.extend_from_slice(b"{{ ");
            out.extend(generate::text(rng, shape));
            out.extend_from_slice(b" }}");
        } else {
            out.extend_from_slice(PIECES[usize::try_from(rng.below(PIECES.len() as u64)).expect("string piece")]);
        }
    }
    out
}
