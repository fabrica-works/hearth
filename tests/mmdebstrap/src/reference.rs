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

/// A list's stanzas and first terminal fault, independent of scan cuts.
#[derive(Debug, PartialEq, Eq)]
pub struct IndexDecoded {
    pub stanzas: Vec<hearth_protocol_mmdebstrap::packages::Stanza>,
    pub error: Option<hearth_protocol_mmdebstrap::packages::StanzaError>,
}
#[must_use]
pub fn index(bytes: &[u8], end: End, limits: &Limits) -> IndexDecoded {
    use hearth_protocol_mmdebstrap::packages::{Field, StanzaError};
    let (bytes, broken) = match end {
        End::Whole => (bytes, false),
        End::Cut(at) => (&bytes[..at.min(bytes.len())], false),
        End::Break(at) => (&bytes[..at.min(bytes.len())], true),
    };
    let mut decoded = IndexDecoded { stanzas: Vec::new(), error: None };
    let mut fields: [Option<Vec<u8>>; 6] = [None, None, None, None, None, None];
    let mut preceding = false;
    let mut selected = None;
    let mut count = 0_usize;
    let chunk = usize::try_from(limits.chunk.max(1)).expect("chunk fits");
    let names = [b"package".as_slice(), b"version", b"architecture", b"filename", b"size", b"sha256"];
    let kinds = [Field::Package, Field::Version, Field::Architecture, Field::Filename, Field::Size, Field::Sha256];
    for raw in bytes.split_inclusive(|b| *b == b'\n') {
        let raw = if raw.ends_with(b"\n") { raw } else { &raw[..raw.len() / chunk * chunk] };
        if raw.is_empty() {
            continue;
        }
        if count + raw.len() > usize::try_from(limits.stanza).expect("stanza fits") {
            // An earlier field fault wins over the budget reached later in this delivery.
            let room = usize::try_from(limits.stanza).expect("stanza fits").saturating_sub(count);
            if let Err(error) =
                index_line(&raw[..room], &mut fields, &mut preceding, &mut selected, limits, &names, kinds)
            {
                decoded.error = Some(error);
            } else {
                decoded.error = Some(StanzaError::TooLong);
            }
            return decoded;
        }
        count += raw.len();
        let line = raw.strip_suffix(b"\n").unwrap_or(raw);
        if line.iter().all(|b| b" \t".contains(b)) {
            if preceding {
                match reference_stanza(&mut fields, limits) {
                    Ok(stanza) => decoded.stanzas.push(stanza),
                    Err(error) => {
                        decoded.error = Some(error);
                        return decoded;
                    }
                }
            }
            preceding = false;
            selected = None;
            count = 0;
        } else if let Err(error) = index_line(line, &mut fields, &mut preceding, &mut selected, limits, &names, kinds) {
            decoded.error = Some(error);
            return decoded;
        }
    }
    if broken {
        decoded.error = Some(StanzaError::Stream(Fault::Reset));
    } else if preceding {
        match reference_stanza(&mut fields, limits) {
            Ok(stanza) => decoded.stanzas.push(stanza),
            Err(error) => decoded.error = Some(error),
        }
    }
    decoded
}
fn index_line(
    line: &[u8],
    fields: &mut [Option<Vec<u8>>; 6],
    preceding: &mut bool,
    selected: &mut Option<usize>,
    limits: &Limits,
    names: &[&[u8]; 6],
    kinds: [hearth_protocol_mmdebstrap::packages::Field; 6],
) -> Result<(), hearth_protocol_mmdebstrap::packages::StanzaError> {
    use hearth_protocol_mmdebstrap::packages::StanzaError;
    if line.starts_with(b" ") || line.starts_with(b"\t") {
        if !*preceding {
            return Err(StanzaError::Malformed);
        }
        if let Some(at) = *selected {
            return Err(StanzaError::Value(kinds[at]));
        }
        return Ok(());
    }
    let colon = line.iter().position(|b| *b == b':').filter(|at| *at > 0).ok_or(StanzaError::Malformed)?;
    *selected = names.iter().position(|name| line[..colon].eq_ignore_ascii_case(name));
    *preceding = true;
    if let Some(at) = *selected {
        if fields[at].is_some() {
            return Err(StanzaError::Repeated(kinds[at]));
        }
        let value = &line[colon + 1..];
        let first = value.iter().position(|b| !b" \t".contains(b)).unwrap_or(value.len());
        let last = value.iter().rposition(|b| !b" \t".contains(b)).map_or(first, |at| at + 1);
        let value = &value[first..last];
        let max = match at {
            0..=2 => limits.name,
            3 => limits.filename,
            4 => 20,
            5 => 64,
            _ => unreachable!(),
        };
        let value = if at == 4 {
            let nonzero = value.iter().position(|b| *b != b'0').unwrap_or(value.len().saturating_sub(1));
            &value[nonzero..]
        } else {
            value
        };
        if value.len() > usize::try_from(max).expect("field bound fits") || value.iter().any(|b| b" \t".contains(b)) {
            return Err(StanzaError::Value(kinds[at]));
        }
        fields[at] = Some(value.to_vec());
    }
    Ok(())
}
fn reference_stanza(
    fields: &mut [Option<Vec<u8>>; 6],
    limits: &Limits,
) -> Result<hearth_protocol_mmdebstrap::packages::Stanza, hearth_protocol_mmdebstrap::packages::StanzaError> {
    use hearth_protocol_mmdebstrap::packages::{Field, Stanza, StanzaError};
    const KINDS: [Field; 6] =
        [Field::Package, Field::Version, Field::Architecture, Field::Filename, Field::Size, Field::Sha256];
    let mut values = Vec::new();
    for (value, kind) in fields.iter_mut().zip(KINDS) {
        values.push(value.take().ok_or(StanzaError::Missing(kind))?);
    }
    let package = Name::new(&values[0], limits).ok_or(StanzaError::Value(Field::Package))?;
    let architecture = Name::new(&values[2], limits).ok_or(StanzaError::Value(Field::Architecture))?;
    if values[1].is_empty() || !values[1].iter().all(|b| b.is_ascii_alphanumeric() || b".+~:-".contains(b)) {
        return Err(StanzaError::Value(Field::Version));
    }
    let filename = &values[3];
    if filename.is_empty()
        || filename.iter().any(|b| !(33..=126).contains(b))
        || filename.split(|b| *b == b'/').any(|part| part.is_empty() || part == b"." || part == b"..")
    {
        return Err(StanzaError::Value(Field::Filename));
    }
    let text = std::str::from_utf8(&values[4]).map_err(|_| StanzaError::Value(Field::Size))?;
    if !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(StanzaError::Value(Field::Size));
    }
    let size = text.parse::<u64>().map_err(|_| StanzaError::Value(Field::Size))?;
    if values[5].len() != 64 || !values[5].iter().all(u8::is_ascii_hexdigit) {
        return Err(StanzaError::Value(Field::Sha256));
    }
    let mut sha256 = [0_u8; 32];
    for (pair, byte) in values[5].chunks_exact(2).zip(&mut sha256) {
        let text = std::str::from_utf8(pair).map_err(|_| StanzaError::Value(Field::Sha256))?;
        *byte = u8::from_str_radix(text, 16).map_err(|_| StanzaError::Value(Field::Sha256))?;
    }
    Ok(Stanza {
        package,
        version: values[1].clone().into(),
        architecture,
        filename: filename.clone().into(),
        size,
        sha256,
    })
}
