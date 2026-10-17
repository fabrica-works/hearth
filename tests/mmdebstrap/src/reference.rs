use crate::world::End;
use hearth_protocol_mmdebstrap::{
    limits::Limits,
    output::{Inst, OutputError},
    tail::{Cause, Class},
    vector::Name,
};
use skein_lib::stream::Fault;
/// The observed triples, last fetched count and terminal error.
#[derive(Debug, PartialEq, Eq)]
pub struct Decoded {
    pub insts: Vec<Inst>,
    pub fetched: u64,
    pub error: Option<OutputError>,
    pub tail: Vec<Vec<u8>>,
    pub cause: Cause,
}
#[must_use]
pub fn output(bytes: &[u8], end: End, limits: &Limits) -> Decoded {
    let (bytes, broken) = match end {
        End::Whole => (bytes, false),
        End::Cut(at) => (&bytes[..at.min(bytes.len())], false),
        End::Break(at) => (&bytes[..at.min(bytes.len())], true),
    };
    let mut decoded = Decoded { insts: Vec::new(), fetched: 0, error: None, tail: Vec::new(), cause: Cause::Other };
    let mut count = 0;
    let max = usize::try_from(limits.line.max(1)).expect("bounded fixture value");
    let mut continuation = false;
    let mut package = false;
    for raw in bytes.split_inclusive(|b| *b == b'\n') {
        if !raw.ends_with(b"\n") && raw.len() < max {
            continue;
        }
        let line = &raw[..raw.len().min(max)];
        let line = line.strip_suffix(b"\n").unwrap_or(line);
        admit(&mut decoded, &mut continuation, &mut package, line, limits);
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

fn admit(decoded: &mut Decoded, continuation: &mut bool, package: &mut bool, bytes: &[u8], limits: &Limits) {
    let line: Vec<u8> = bytes
        .iter()
        .take(usize::try_from(limits.line).expect("line bound"))
        .map(|b| if *b == b'\t' || *b == b'\r' { b' ' } else { *b })
        .collect();
    let text = String::from_utf8_lossy(&line);
    let has = |pattern: &[u8]| text.contains(std::str::from_utf8(pattern).expect("ASCII pattern"));
    let denied = has(b"E: mmdebstrap failed to run")
        || has(b"E: setup failed: ")
        || has(b"W: hooklistener errored out")
        || (has(b"I: PID ") && has(b" exited with exit code"))
        || (has(b"E: apt-get update") && has(b"failed: process exited with"));
    let allow: &[&[u8]] = &[
        b"dpkg: error",
        b"E: ",
        b"Err:",
        b"No space left on device",
        b"Hash Sum mismatch",
        b"Failed to fetch",
        b"Unable to ",
        b"Could not ",
        b"The following packages have unmet dependencies:",
        b"However the following packages replace it:",
    ];
    let indented = line.starts_with(b" ");
    if denied || !(allow.iter().any(|p| has(p)) || *continuation && indented) {
        *continuation = false;
        *package = false;
        return;
    }
    if *package && indented {
        let name =
            line.iter().position(|b| *b != b' ').map(|at| &line[at..]).and_then(|line| {
                line.windows(2).position(|p| p == b" :").and_then(|at| Name::new(&line[..at], limits))
            });
        if let Cause::Unsatisfiable(None) = decoded.cause
            && let Some(name) = name
        {
            decoded.cause = Cause::Unsatisfiable(Some(name));
        }
    }
    let candidate = failure_cause(&line, limits);
    if candidate != Cause::Other
        && (decoded.cause == Cause::Other
            || (decoded.cause.class() == Class::Configuration && candidate.class() == Class::Operational))
    {
        decoded.cause = candidate;
    }
    *package = has(b"The following packages have unmet dependencies:");
    *continuation = (*continuation && indented)
        || has(b"dpkg: error")
        || has(b"E: ")
        || *package
        || has(b"However the following packages replace it:");
    if limits.tail_lines > 0 {
        decoded.tail.push(line[..line.len().min(usize::try_from(limits.tail_line).expect("tail bound"))].to_vec());
        if decoded.tail.len() > usize::try_from(limits.tail_lines).expect("tail count") {
            decoded.tail.remove(0);
        }
    }
}
fn failure_cause(line: &[u8], limits: &Limits) -> Cause {
    let text = String::from_utf8_lossy(line);
    let has = |pattern: &[u8]| text.contains(std::str::from_utf8(pattern).expect("ASCII pattern"));
    if has(b"Temporary failure resolving") || has(b"Could not resolve") {
        return Cause::Resolve;
    }
    if has(b"No space left on device") {
        return Cause::Space;
    }
    let fetch = has(b"Err:") || has(b"Failed to fetch");
    if has(b"does not have a Release file") || (fetch && (has(b"404  Not Found") || has(b"NOT FOUND"))) {
        return Cause::Absent;
    }
    if fetch {
        return Cause::Fetch;
    }
    let marker = b"Unable to locate package ";
    if let Some(at) = line.windows(marker.len()).position(|p| p == marker)
        && let Some(name) = Name::new(&line[at + marker.len()..], limits)
    {
        return Cause::Unknown(name);
    }
    let marker = b"Package '";
    if let Some(at) = line.windows(marker.len()).position(|p| p == marker)
        && let Some(name) = line[at + marker.len()..]
            .strip_suffix(b"' has no installation candidate")
            .and_then(|name| Name::new(name, limits))
    {
        return Cause::Unknown(name);
    }
    if has(b"Unable to correct problems") || has(b"The following packages have unmet dependencies:") {
        return Cause::Unsatisfiable(None);
    }
    Cause::Other
}
