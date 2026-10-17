//! The two pipe readers and apt's install triples (render.md, 5.3).
use crate::{MaxOut, limits::Limits, vector::Name};
use alloc::boxed::Box;
use skein_lib::stream::{Delimiter, Down, Fault, Read, Up};
use skein_lib::{Env, Queue, bytes};

/// One package apt would install.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inst {
    pub name: Name,
    pub version: Box<[u8]>,
    pub architecture: Name,
}
/// A malformed install line, too many triples, or a broken pipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputError {
    Inst,
    TooMany,
    Stream(Fault),
}
/// A triple, apt's fetched bytes, or the pipe's single terminal event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seen {
    Inst(Inst),
    Fetched(u64),
    Ended,
    Failed(OutputError),
}
/// One pipe; scans hold the line below, and oversized suffixes are drained.
#[derive(Debug)]
pub struct Reader {
    waiting: bool,
    ended: bool,
    dropping: bool,
    count: u32,
    error: Option<OutputError>,
}
impl Reader {
    #[must_use]
    pub fn new(_limits: &Limits) -> Reader {
        Reader { waiting: false, ended: false, dropping: false, count: 0, error: None }
    }
    #[must_use]
    pub const fn waiting(&self) -> bool {
        self.waiting
    }
}
pub const UP_MAX_OUT: MaxOut = MaxOut { above: 1, below: 1 };
#[must_use]
pub fn largest_demand(limits: &Limits) -> u32 {
    limits.line.max(1)
}
/// A scan delivery together with the triple decoded from it.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    u64::from(largest_demand(limits)).checked_mul(2)?.checked_add(u64::from(limits.name).checked_mul(2)?)
}
fn demand(reader: &mut Reader, limits: &Limits, below: &mut Queue<Down>) {
    reader.waiting = true;
    below.push(Down::Demand { read: Read::Scan { until: Delimiter::LF, max: largest_demand(limits) }, room: 0 });
}
/// Begin reading one pipe.
pub fn open(reader: &mut Reader, env: &Env<Limits>, below: &mut Queue<Down>) {
    if !reader.waiting && !reader.ended {
        demand(reader, &env.limits, below);
    }
}
/// Consume one scan result; errors are held until the pipe ends.
pub fn up(reader: &mut Reader, env: &Env<Limits>, event: Up, above: &mut Queue<Seen>, below: &mut Queue<Down>) {
    if reader.ended {
        return;
    }
    match event {
        Up::Bytes(bytes) => {
            if !reader.waiting {
                return;
            }
            reader.waiting = false;
            let lf = bytes.ends_with(b"\n");
            if !reader.dropping {
                let scanned = if lf { bytes.strip_suffix(b"\n").expect("LF suffix checked") } else { &bytes };
                if let Some(seen) = line(reader, scanned, &env.limits) {
                    above.push(seen);
                }
            }
            reader.dropping = !lf;
            demand(reader, &env.limits, below);
        }
        Up::End => {
            close(reader, below);
            let seen = match reader.error.take() {
                Some(error) => Seen::Failed(error),
                None => Seen::Ended,
            };
            above.push(seen);
        }
        Up::Failed(fault) => {
            reader.waiting = false;
            reader.ended = true;
            let error = reader.error.take().unwrap_or(OutputError::Stream(fault));
            above.push(Seen::Failed(error));
        }
        Up::Room => {}
    }
}
/// Stop reading and withdraw the outstanding demand, if any.
pub fn close(reader: &mut Reader, below: &mut Queue<Down>) {
    if reader.waiting {
        below.push(Down::Demand { read: Read::Nothing, room: 0 });
    }
    reader.waiting = false;
    reader.ended = true;
}
fn line(reader: &mut Reader, bytes: &[u8], limits: &Limits) -> Option<Seen> {
    if bytes.starts_with(b"Inst ") && reader.error.is_none() {
        if reader.count >= limits.insts {
            reader.error = Some(OutputError::TooMany);
            return None;
        }
        reader.count = reader.count.checked_add(1).expect("below the line count limit");
        match inst(bytes, limits) {
            Some(inst) => return Some(Seen::Inst(inst)),
            None => reader.error = Some(OutputError::Inst),
        }
    }
    if let Some(bytes) = fetched(bytes) {
        return Some(Seen::Fetched(bytes));
    }
    None
}
/// Decode the mandatory install triple, ignoring any broken-package suffix.
#[must_use]
pub fn inst(line: &[u8], limits: &Limits) -> Option<Inst> {
    let rest = line.strip_prefix(b"Inst ")?;
    let split = bytes::find(rest, b" ")?;
    let name = Name::new(rest.get(..split)?, limits)?;
    let mut rest = rest.get(split.checked_add(1)?..)?;
    if rest.starts_with(b"[") {
        let end = bytes::find(rest, b"] ")?;
        if end < 2 {
            return None;
        }
        rest = rest.get(end.checked_add(2)?..)?;
    }
    let rest = rest.strip_prefix(b"(")?;
    let split = bytes::find(rest, b" ")?;
    if split == 0 {
        return None;
    }
    let version = rest.get(..split)?;
    let origins = rest.get(split.checked_add(1)?..)?;
    let closing = bytes::find(origins, b"])")?;
    let origins = origins.get(..closing)?;
    let mut last = None;
    for (at, pair) in origins.windows(2).enumerate() {
        if pair == b" [" {
            last = Some(at);
        }
    }
    let end = last?;
    if end == 0 {
        return None;
    }
    let architecture = origins.get(end.checked_add(2)?..)?;
    let architecture = Name::new(architecture, limits)?;
    Some(Inst { name, version: bytes::copy_of(version), architecture })
}
fn fetched(line: &[u8]) -> Option<u64> {
    let line = line.strip_prefix(b"Fetched ")?;
    let split = bytes::find(line, b" ")?;
    let number = line.get(..split)?;
    let rest = line.get(split.checked_add(1)?..)?;
    let split = bytes::find(rest, b" in ")?;
    if rest.get(split.checked_add(4)?..)?.is_empty() {
        return None;
    }
    let unit = rest.get(..split)?;
    let multiplier: u64 = match unit {
        b"B" => 1,
        b"kB" => 1_000,
        b"MB" => 1_000_000,
        b"GB" => 1_000_000_000,
        b"TB" => 1_000_000_000_000,
        _ => return None,
    };
    let mut mantissa = 0_u64;
    let mut scale = 1_u64;
    let mut decimal = false;
    let mut before = false;
    let mut after = false;
    for b in number {
        if *b == b'.' {
            if decimal || !before {
                return None;
            }
            decimal = true;
        } else {
            if !b.is_ascii_digit() {
                return None;
            }
            mantissa = mantissa.checked_mul(10)?.checked_add(u64::from(b.checked_sub(b'0')?))?;
            if decimal {
                scale = scale.checked_mul(10)?;
                after = true;
            } else {
                before = true;
            }
        }
    }
    if !before || (decimal && !after) {
        return None;
    }
    let whole = mantissa.checked_div(scale)?.checked_mul(multiplier)?;
    let fraction = mantissa.checked_rem(scale)?.checked_mul(multiplier)?.checked_div(scale)?;
    whole.checked_add(fraction)
}
