//! Bounded deb822 Packages stanzas (render.md, 5.3).
use crate::{MaxOut, limits::Limits, vector::Name};
use alloc::boxed::Box;
use skein_lib::{
    Env, List, Queue,
    stream::{Delimiter, Down, Fault, Read, Up},
};
/// The six fields used to match and fetch a resolved package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stanza {
    pub package: Name,
    pub version: Box<[u8]>,
    pub architecture: Name,
    pub filename: Box<[u8]>,
    pub size: u64,
    pub sha256: [u8; 32],
}
/// A required single-line field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Package,
    Version,
    Architecture,
    Filename,
    Size,
    Sha256,
}
const FIELDS: [Field; 6] =
    [Field::Package, Field::Version, Field::Architecture, Field::Filename, Field::Size, Field::Sha256];
/// The first fault ends the list without reading further.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StanzaError {
    TooLong,
    Malformed,
    Missing(Field),
    Repeated(Field),
    Value(Field),
    Stream(Fault),
}
/// A decoded stanza, or the list's single terminal event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Found {
    Stanza(Stanza),
    Ended,
    Failed(StanzaError),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Start,
    Header,
    Value,
    Continuation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Idle,
    Reading,
    Ended,
}
/// Only the selected values and a bounded field-name prefix are retained.
#[derive(Debug)]
pub struct Stanzas {
    state: State,
    count: u64,
    part: Part,
    nonspace: bool,
    field: Option<Field>,
    preceding: bool,
    trailing: bool,
    buffer: List<u8>,
    values: [Option<Box<[u8]>>; 6],
}
impl Stanzas {
    #[must_use]
    pub fn new(limits: &Limits) -> Stanzas {
        Stanzas {
            state: State::Idle,
            count: 0,
            part: Part::Start,
            nonspace: false,
            field: None,
            preceding: false,
            trailing: false,
            buffer: List::with_capacity(limits.name.max(limits.filename).max(64)),
            values: [None, None, None, None, None, None],
        }
    }
}
pub const UP_MAX_OUT: MaxOut = MaxOut { above: 2, below: 1 };
#[must_use]
pub fn largest_demand(limits: &Limits) -> u32 {
    limits.chunk.max(1)
}
/// The value scratch, selected fields, delivery and decoded names in flight.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    List::<u8>::worst_case(limits.name.max(limits.filename).max(64))?
        .checked_add(u64::from(limits.name).checked_mul(5)?)?
        .checked_add(u64::from(limits.filename))?
        .checked_add(84)?
        .checked_add(u64::from(largest_demand(limits)))
}
fn demand(stanzas: &mut Stanzas, limits: &Limits, below: &mut Queue<Down>) {
    stanzas.state = State::Reading;
    below.push(Down::Demand { read: Read::Scan { until: Delimiter::LF, max: largest_demand(limits) }, room: 0 });
}
/// Begin the list, once.
pub fn open(stanzas: &mut Stanzas, env: &Env<Limits>, below: &mut Queue<Down>) {
    if stanzas.state == State::Idle {
        demand(stanzas, &env.limits, below);
    }
}
/// Stop the list and withdraw any outstanding scan.
pub fn close(stanzas: &mut Stanzas, below: &mut Queue<Down>) {
    if stanzas.state == State::Reading {
        below.push(Down::Demand { read: Read::Nothing, room: 0 });
    }
    stanzas.state = State::Ended;
}
/// Read one bounded scan, emitting at most one stanza and its terminal event.
pub fn up(stanzas: &mut Stanzas, env: &Env<Limits>, event: Up, above: &mut Queue<Found>, below: &mut Queue<Down>) {
    if stanzas.state == State::Ended {
        return;
    }
    let result = match event {
        Up::Bytes(bytes) => {
            if stanzas.state != State::Reading {
                return;
            }
            stanzas.state = State::Idle;
            let result = scan(stanzas, &bytes, &env.limits, above);
            if result.is_ok() {
                demand(stanzas, &env.limits, below);
            }
            result
        }
        Up::End => {
            let result = finish(stanzas, &env.limits, above);
            close(stanzas, below);
            if result.is_ok() {
                above.push(Found::Ended);
            }
            result
        }
        Up::Failed(fault) => {
            stanzas.state = State::Idle;
            Err(StanzaError::Stream(fault))
        }
        Up::Room => return,
    };
    if let Err(error) = result {
        close(stanzas, below);
        above.push(Found::Failed(error));
    }
}
fn scan(stanzas: &mut Stanzas, input: &[u8], limits: &Limits, above: &mut Queue<Found>) -> Result<(), StanzaError> {
    for b in input {
        stanzas.count = stanzas.count.checked_add(1).ok_or(StanzaError::TooLong)?;
        if stanzas.count > u64::from(limits.stanza) {
            return Err(StanzaError::TooLong);
        }
        if *b == b'\n' {
            line(stanzas, limits, above)?;
        } else {
            byte(stanzas, *b, limits)?;
        }
    }
    Ok(())
}
fn byte(stanzas: &mut Stanzas, b: u8, limits: &Limits) -> Result<(), StanzaError> {
    let space = b == b' ' || b == b'\t';
    stanzas.nonspace |= !space;
    if stanzas.part == Part::Start {
        stanzas.part = if space { Part::Continuation } else { Part::Header };
    }
    match stanzas.part {
        Part::Start => unreachable!("a first byte chooses a line's part"),
        Part::Header => {
            if b == b':' {
                if stanzas.buffer.is_empty() {
                    return Err(StanzaError::Malformed);
                }
                stanzas.field = field(stanzas.buffer.as_slice());
                if let Some(field) = stanzas.field
                    && stanzas.values.get(slot(field)).expect("six fields").is_some()
                {
                    return Err(StanzaError::Repeated(field));
                }
                stanzas.buffer.clear();
                stanzas.preceding = true;
                stanzas.part = Part::Value;
            } else if stanzas.buffer.len() < 13 {
                stanzas.buffer.push(b.to_ascii_lowercase()).expect("field prefix bounded at thirteen");
            }
        }
        Part::Continuation => {
            if !space {
                if !stanzas.preceding {
                    return Err(StanzaError::Malformed);
                }
                if let Some(field) = stanzas.field {
                    return Err(StanzaError::Value(field));
                }
            }
        }
        Part::Value => {
            if let Some(field) = stanzas.field {
                if space {
                    stanzas.trailing |= !stanzas.buffer.is_empty();
                } else {
                    if stanzas.trailing {
                        return Err(StanzaError::Value(field));
                    }
                    // Leading zeroes need no storage even in an arbitrarily long Size.
                    if field == Field::Size && stanzas.buffer.as_slice() == b"0" {
                        stanzas.buffer.clear();
                    }
                    if stanzas.buffer.len() >= bound(field, limits) {
                        return Err(StanzaError::Value(field));
                    }
                    stanzas.buffer.push(b).expect("field bound fits value scratch");
                }
            }
        }
    }
    Ok(())
}
fn line(stanzas: &mut Stanzas, limits: &Limits, above: &mut Queue<Found>) -> Result<(), StanzaError> {
    if stanzas.nonspace {
        match stanzas.part {
            Part::Header | Part::Start => return Err(StanzaError::Malformed),
            Part::Value => {
                if let Some(field) = stanzas.field {
                    *stanzas.values.get_mut(slot(field)).expect("six fields") = Some(stanzas.buffer.to_boxed());
                }
            }
            Part::Continuation => {}
        }
    } else {
        emit(stanzas, limits, above)?;
        stanzas.count = 0;
        stanzas.preceding = false;
        stanzas.field = None;
    }
    stanzas.buffer.clear();
    stanzas.part = Part::Start;
    stanzas.nonspace = false;
    stanzas.trailing = false;
    Ok(())
}
fn finish(stanzas: &mut Stanzas, limits: &Limits, above: &mut Queue<Found>) -> Result<(), StanzaError> {
    if stanzas.part != Part::Start {
        line(stanzas, limits, above)?;
    }
    emit(stanzas, limits, above)
}
fn emit(stanzas: &mut Stanzas, limits: &Limits, above: &mut Queue<Found>) -> Result<(), StanzaError> {
    if !stanzas.preceding {
        return Ok(());
    }
    let mut values: [Box<[u8]>; 6] =
        [Box::from([]), Box::from([]), Box::from([]), Box::from([]), Box::from([]), Box::from([])];
    for (field, value) in FIELDS.into_iter().zip(&mut values) {
        *value = stanzas.values.get_mut(slot(field)).expect("six fields").take().ok_or(StanzaError::Missing(field))?;
    }
    let [package, version, architecture, filename, size, sha256] = values;
    let package = Name::new(&package, limits).ok_or(StanzaError::Value(Field::Package))?;
    let architecture = Name::new(&architecture, limits).ok_or(StanzaError::Value(Field::Architecture))?;
    if version.is_empty() {
        return Err(StanzaError::Value(Field::Version));
    }
    for b in &version {
        if !b.is_ascii_alphanumeric() && !b".+~:-".contains(b) {
            return Err(StanzaError::Value(Field::Version));
        }
    }
    if !relative(&filename) {
        return Err(StanzaError::Value(Field::Filename));
    }
    let size = decimal(&size).ok_or(StanzaError::Value(Field::Size))?;
    let sha256 = digest(&sha256).ok_or(StanzaError::Value(Field::Sha256))?;
    above.push(Found::Stanza(Stanza { package, version, architecture, filename, size, sha256 }));
    stanzas.preceding = false;
    Ok(())
}
fn field(name: &[u8]) -> Option<Field> {
    match name {
        b"package" => Some(Field::Package),
        b"version" => Some(Field::Version),
        b"architecture" => Some(Field::Architecture),
        b"filename" => Some(Field::Filename),
        b"size" => Some(Field::Size),
        b"sha256" => Some(Field::Sha256),
        _ => None,
    }
}
fn slot(field: Field) -> usize {
    match field {
        Field::Package => 0,
        Field::Version => 1,
        Field::Architecture => 2,
        Field::Filename => 3,
        Field::Size => 4,
        Field::Sha256 => 5,
    }
}
fn bound(field: Field, limits: &Limits) -> u32 {
    match field {
        Field::Package | Field::Version | Field::Architecture => limits.name,
        Field::Filename => limits.filename,
        Field::Size => 20,
        Field::Sha256 => 64,
    }
}
fn decimal(value: &[u8]) -> Option<u64> {
    if value.is_empty() {
        return None;
    }
    let mut number = 0_u64;
    for b in value {
        if !b.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(b.checked_sub(b'0')?))?;
    }
    Some(number)
}
fn digest(value: &[u8]) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut out = [0_u8; 32];
    for (pair, byte) in value.chunks_exact(2).zip(&mut out) {
        for b in pair {
            let digit = match b {
                b'0'..=b'9' => b.checked_sub(b'0')?,
                b'a'..=b'f' => b.checked_sub(b'a')?.checked_add(10)?,
                b'A'..=b'F' => b.checked_sub(b'A')?.checked_add(10)?,
                _ => return None,
            };
            *byte = byte.checked_mul(16)?.checked_add(digit)?;
        }
    }
    Some(out)
}
fn relative(value: &[u8]) -> bool {
    if value.is_empty() || value.starts_with(b"/") || value.ends_with(b"/") {
        return false;
    }
    let mut start = 0_usize;
    for (at, b) in value.iter().enumerate() {
        if !(33..=126).contains(b) {
            return false;
        }
        if *b == b'/' {
            let segment = value.get(start..at).expect("segment within filename");
            if segment.is_empty() || segment == b"." || segment == b".." {
                return false;
            }
            start = at.checked_add(1).expect("filename length fits usize");
        }
    }
    let last = value.get(start..).expect("last segment within filename");
    last != b"." && last != b".."
}
