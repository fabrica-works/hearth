//! A tag-at-a-time reference, independent of the bounded compiler.
use hearth_protocol_template::{Does, Instruction, Template, limits::Limits};
/// Reads a whole template with the expression reference.
/// # Errors
/// Returns the first refused byte offset.
pub fn compile(text: &[u8], limits: &Limits) -> Result<Template, u32> {
    if text.len() > limits.template as usize {
        return Err(limits.template);
    }
    validate(text)?;
    let lines = std::iter::once(0)
        .chain(
            text.iter()
                .enumerate()
                .filter(|(_, b)| **b == b'\n')
                .map(|(at, _)| u32::try_from(at + 1).expect("validated template length")),
        )
        .collect();
    let mut instructions = Vec::new();
    read(text, 0, limits, &mut instructions)?;
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
fn read(text: &[u8], cursor: usize, limits: &Limits, out: &mut Vec<Instruction>) -> Result<(), u32> {
    let next = (cursor..text.len().saturating_sub(1)).find(|&i| text[i] == b'{' && b"{%#".contains(&text[i + 1]));
    let at = next.unwrap_or(text.len());
    if at > cursor {
        out.push(Instruction {
            at: u32::try_from(cursor).expect("validated template length"),
            does: Does::Text(text[cursor..at].into()),
        });
    }
    if at == text.len() {
        return Ok(());
    }
    let kind = text[at + 1];
    let marker = |b| b == b'-' || (kind != b'{' && b == b'+');
    let start = at + 2 + usize::from(text.get(at + 2).copied().is_some_and(marker));
    let mut end = start;
    while end < text.len() {
        if kind != b'#' && (text[end] == b'\'' || text[end] == b'"') {
            let opening = end;
            let quote = text[end];
            end += 1;
            while end < text.len() && text[end] != quote {
                end += if text[end] == b'\\' { 2 } else { 1 };
            }
            if end >= text.len() {
                return Err(u32::try_from(opening).expect("validated template length"));
            }
        } else if text.get(end + 1) == Some(&b'}')
            && if kind == b'#' { text[end] == b'#' } else { text[end] == b'}' || text[end] == b'%' }
        {
            if text[end] != if kind == b'{' { b'}' } else { kind } {
                return Err(u32::try_from(end).expect("validated template length"));
            }
            let stop = end - usize::from(end > start && marker(text[end - 1]));
            if kind == b'{' {
                let mut e = super::compile(&text[start..stop], limits)
                    .map_err(|e| e + u32::try_from(start).expect("validated template length"))?;
                for op in &mut e.operations {
                    op.at += u32::try_from(start).expect("validated template length");
                }
                out.push(Instruction {
                    at: u32::try_from(at).expect("validated template length"),
                    does: Does::Output(e),
                });
            } else if kind == b'%' {
                let word = (start..stop).find(|&i| !b" \t\n".contains(&text[i])).unwrap_or(stop);
                return Err(u32::try_from(word).expect("validated template length"));
            }
            return read(text, end + 2, limits, out);
        }
        end += 1;
    }
    Err(u32::try_from(at).expect("validated template length"))
}
