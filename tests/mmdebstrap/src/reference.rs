use crate::world::End;
use hearth_protocol_mmdebstrap::{
    limits::Limits,
    output::{Inst, OutputError},
    vector::Name,
};
use skein_lib::stream::Fault;
/// The observed triples, last fetched count and terminal error.
#[derive(Debug, PartialEq, Eq)]
pub struct Decoded {
    pub insts: Vec<Inst>,
    pub fetched: u64,
    pub error: Option<OutputError>,
}
#[must_use]
pub fn output(bytes: &[u8], end: End, limits: &Limits) -> Decoded {
    let (bytes, broken) = match end {
        End::Whole => (bytes, false),
        End::Cut(at) => (&bytes[..at.min(bytes.len())], false),
        End::Break(at) => (&bytes[..at.min(bytes.len())], true),
    };
    let mut decoded = Decoded { insts: Vec::new(), fetched: 0, error: None };
    let mut count = 0;
    let max = usize::try_from(limits.line.max(1)).expect("bounded fixture value");
    for raw in bytes.split_inclusive(|b| *b == b'\n') {
        if !raw.ends_with(b"\n") && raw.len() < max {
            continue;
        }
        let line = &raw[..raw.len().min(max)];
        let line = line.strip_suffix(b"\n").unwrap_or(line);
        if line.starts_with(b"Inst ") && decoded.error.is_none() {
            if count >= limits.insts {
                decoded.error = Some(OutputError::TooMany);
            } else {
                count += 1;
                match inst(line, limits) {
                    Some(inst) => decoded.insts.push(inst),
                    None => decoded.error = Some(OutputError::Inst),
                }
            }
        }
        if let Some(bytes) = fetched(line) {
            decoded.fetched = bytes;
        }
    }
    if broken && decoded.error.is_none() {
        decoded.error = Some(OutputError::Stream(Fault::Reset));
    }
    decoded
}
fn inst(line: &[u8], limits: &Limits) -> Option<Inst> {
    let rest = line.strip_prefix(b"Inst ")?;
    let (name, rest) = rest.split_at(rest.iter().position(|b| *b == b' ')?);
    let name = Name::new(name, limits)?;
    let mut rest = rest.strip_prefix(b" ")?;
    if rest.starts_with(b"[") {
        let at = rest.windows(2).position(|p| p == b"] ")?;
        if at < 2 {
            return None;
        }
        rest = &rest[at + 2..];
    }
    let rest = rest.strip_prefix(b"(")?;
    let at = rest.iter().position(|b| *b == b' ')?;
    if at == 0 {
        return None;
    }
    let version = rest[..at].into();
    let rest = &rest[at + 1..];
    let at = rest.windows(2).position(|p| p == b"])")?;
    let rest = &rest[..at];
    let at = rest.windows(2).rposition(|p| p == b" [")?;
    if at == 0 {
        return None;
    }
    Some(Inst { name, version, architecture: Name::new(&rest[at + 2..], limits)? })
}
fn fetched(line: &[u8]) -> Option<u64> {
    let text = std::str::from_utf8(line).ok()?;
    let text = text.strip_prefix("Fetched ")?;
    let (number, text) = text.split_once(' ')?;
    let (unit, time) = text.split_once(" in ")?;
    if time.is_empty() {
        return None;
    }
    let (whole, decimal) = number.split_once('.').unwrap_or((number, ""));
    if whole.is_empty()
        || (number.contains('.') && decimal.is_empty())
        || !whole.bytes().chain(decimal.bytes()).all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let multiplier: u64 = match unit {
        "B" => 1,
        "kB" => 1000,
        "MB" => 1_000_000,
        "GB" => 1_000_000_000,
        "TB" => 1_000_000_000_000,
        _ => return None,
    };
    let whole = whole.parse::<u64>().ok()?.checked_mul(multiplier)?;
    let fractional = if decimal.is_empty() {
        0
    } else {
        decimal
            .parse::<u64>()
            .ok()?
            .checked_mul(multiplier)?
            .checked_div(10_u64.checked_pow(u32::try_from(decimal.len()).ok()?)?)?
    };
    whole.checked_add(fractional)
}
