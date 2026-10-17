//! Text filters and a two-pass, depth-bounded JSON walk (format.md, 7.5).
use super::{
    Budget, Entry, FaultKind, Filter, Operand, Written,
    walk::{self, Items, View},
};
use core::slice::Iter;
use skein_lib::{Decimal, Stack, Writer};
const HEX: &[u8] = b"0123456789abcdef";
fn put(writer: &mut Option<&mut Writer>, length: &mut usize, bytes: &[u8]) -> Result<(), FaultKind> {
    *length = length.checked_add(bytes.len()).ok_or(FaultKind::Limit)?;
    if let Some(writer) = writer {
        writer.put(bytes).expect("measured");
    }
    Ok(())
}
fn quoted(text: &[u8], writer: &mut Option<&mut Writer>, length: &mut usize) -> Result<(), FaultKind> {
    put(writer, length, b"\"")?;
    for byte in text {
        let escaped: &[u8] = match byte {
            b'"' => b"\\\"",
            b'\\' => b"\\\\",
            8 => b"\\b",
            12 => b"\\f",
            b'\n' => b"\\n",
            b'\r' => b"\\r",
            b'\t' => b"\\t",
            0..=31 => {
                let high = *HEX.get(usize::from(byte / 16)).expect("hex digit");
                let low = *HEX.get(usize::from(byte % 16)).expect("hex digit");
                put(writer, length, &[b'\\', b'u', b'0', b'0', high, low])?;
                continue;
            }
            32..=255 => core::slice::from_ref(byte),
        };
        put(writer, length, escaped)?;
    }
    put(writer, length, b"\"")
}
#[derive(Debug)]
enum Frame<'v> {
    List { items: Items<'v>, next: usize },
    Map { entries: Iter<'v, Entry>, first: bool },
}
pub(super) fn json<'c>(
    input: &Operand<'c>,
    budget: &mut Budget,
    capacity: u32,
    steps: u64,
) -> Result<Operand<'c>, FaultKind> {
    let length = json_pass(walk::view(input), capacity, steps, None)?;
    budget.charge(u64::try_from(length).expect("bytes fit"))?;
    let mut writer = Writer::new(length);
    json_pass(walk::view(input), capacity, steps, Some(&mut writer))?;
    Ok(Operand::String(writer.finish()))
}
fn json_pass(
    mut value: View<'_>,
    capacity: u32,
    steps: u64,
    mut writer: Option<&mut Writer>,
) -> Result<usize, FaultKind> {
    let mut frames = Stack::with_capacity(capacity);
    let mut length = 0;
    for _ in 0..steps {
        match value {
            View::Undefined => return Err(FaultKind::Undefined),
            View::Bool(v) => put(&mut writer, &mut length, if v { b"true" } else { b"false" })?,
            View::Int(v) => {
                let number = Written::Integer { negative: v < 0, digits: Decimal::of(v.unsigned_abs()) };
                length = length.checked_add(number.len()).ok_or(FaultKind::Limit)?;
                if let Some(writer) = &mut writer {
                    number.put(writer);
                }
            }
            View::String(text) => quoted(text, &mut writer, &mut length)?,
            View::List(items) => {
                put(&mut writer, &mut length, b"[")?;
                if frames.push(Frame::List { items, next: 0 }).is_err() {
                    return Err(FaultKind::Limit);
                }
            }
            View::Map(entries) => {
                put(&mut writer, &mut length, b"{")?;
                if frames.push(Frame::Map { entries: entries.iter(), first: true }).is_err() {
                    return Err(FaultKind::Limit);
                }
            }
        }
        for _ in 0..=capacity {
            let Some(frame) = frames.top_mut() else {
                return Ok(length);
            };
            let close = match frame {
                Frame::List { items, next } => {
                    if let Some(item) = items.get(*next) {
                        if *next != 0 {
                            put(&mut writer, &mut length, b",")?;
                        }
                        *next = next.saturating_add(1);
                        value = item;
                        break;
                    }
                    b"]".as_slice()
                }
                Frame::Map { entries, first } => {
                    let mut next = None;
                    for entry in entries.by_ref() {
                        if let Some(item) = &entry.value {
                            next = Some((&entry.key, item));
                            break;
                        }
                    }
                    if let Some((key, item)) = next {
                        if !*first {
                            put(&mut writer, &mut length, b",")?;
                        }
                        *first = false;
                        quoted(key, &mut writer, &mut length)?;
                        put(&mut writer, &mut length, b":")?;
                        value = walk::context(item);
                        break;
                    }
                    b"}".as_slice()
                }
            };
            put(&mut writer, &mut length, close)?;
            let _ended = frames.pop().expect("top existed");
        }
    }
    Err(FaultKind::Limit)
}
fn valid_path(text: &[u8], absolute: bool) -> bool {
    if text == b"/" {
        return true;
    }
    if text.is_empty() || text.last() == Some(&b'/') || (absolute && text.first() != Some(&b'/')) {
        return false;
    }
    let mut from = usize::from(text.first() == Some(&b'/'));
    let start = from;
    for end in start..=text.len() {
        if end != text.len() && text.get(end) != Some(&b'/') {
            continue;
        }
        let part = text.get(from..end).expect("segment bounds");
        if part.is_empty() || part == b"." || part == b".." {
            return false;
        }
        from = end.saturating_add(1);
    }
    true
}
pub(super) fn text<'c>(filter: Filter, input: &[u8], budget: &mut Budget) -> Result<Operand<'c>, FaultKind> {
    let input = match filter {
        Filter::SystemdEscapePath => {
            if !valid_path(input, true) {
                return Err(FaultKind::Path);
            }
            if input == b"/" { b"/".as_slice() } else { input.get(1..).expect("absolute path") }
        }
        Filter::Basename | Filter::Dirname => {
            if !valid_path(input, false) {
                return Err(FaultKind::Path);
            }
            if input == b"/" {
                input
            } else {
                let mut slash = None;
                for (index, byte) in input.iter().enumerate() {
                    if *byte == b'/' {
                        slash = Some(index);
                    }
                }
                match filter {
                    Filter::Basename => input
                        .get(
                            match slash {
                                Some(index) => index.saturating_add(1),
                                None => 0,
                            }..,
                        )
                        .expect("segment"),
                    Filter::Dirname => match slash {
                        Some(0) => b"/",
                        Some(index) => input.get(..index).expect("prefix"),
                        None => b".",
                    },
                    Filter::Default
                    | Filter::Join
                    | Filter::Split
                    | Filter::First
                    | Filter::Last
                    | Filter::Length
                    | Filter::Replace
                    | Filter::Items
                    | Filter::Keys
                    | Filter::Values
                    | Filter::Sort
                    | Filter::Tojson
                    | Filter::ShellQuote
                    | Filter::SystemdEscape
                    | Filter::SystemdEscapePath => unreachable!("path filter"),
                }
            }
        }
        Filter::ShellQuote | Filter::SystemdEscape => input,
        Filter::Default
        | Filter::Join
        | Filter::Split
        | Filter::First
        | Filter::Last
        | Filter::Length
        | Filter::Replace
        | Filter::Items
        | Filter::Keys
        | Filter::Values
        | Filter::Sort
        | Filter::Tojson => unreachable!("text filter"),
    };
    let length = text_pass(filter, input, None)?;
    budget.charge(u64::try_from(length).expect("bytes fit"))?;
    let mut writer = Writer::new(length);
    text_pass(filter, input, Some(&mut writer))?;
    Ok(Operand::String(writer.finish()))
}
fn text_pass(filter: Filter, input: &[u8], mut writer: Option<&mut Writer>) -> Result<usize, FaultKind> {
    let mut length = 0;
    if filter == Filter::ShellQuote {
        put(&mut writer, &mut length, b"'")?;
    }
    for (index, byte) in input.iter().enumerate() {
        match filter {
            Filter::ShellQuote => {
                put(&mut writer, &mut length, if *byte == b'\'' { b"'\\''" } else { core::slice::from_ref(byte) })?;
            }
            Filter::SystemdEscape | Filter::SystemdEscapePath => {
                if *byte == b'/' {
                    put(&mut writer, &mut length, b"-")?;
                } else if byte.is_ascii_alphanumeric()
                    || *byte == b':'
                    || *byte == b'_'
                    || (*byte == b'.' && index != 0)
                {
                    put(&mut writer, &mut length, core::slice::from_ref(byte))?;
                } else {
                    let high = *HEX.get(usize::from(byte / 16)).expect("hex digit");
                    let low = *HEX.get(usize::from(byte % 16)).expect("hex digit");
                    put(&mut writer, &mut length, &[b'\\', b'x', high, low])?;
                }
            }
            Filter::Basename | Filter::Dirname => put(&mut writer, &mut length, core::slice::from_ref(byte))?,
            Filter::Default
            | Filter::Join
            | Filter::Split
            | Filter::First
            | Filter::Last
            | Filter::Length
            | Filter::Replace
            | Filter::Items
            | Filter::Keys
            | Filter::Values
            | Filter::Sort
            | Filter::Tojson => unreachable!("text filter"),
        }
    }
    if filter == Filter::ShellQuote {
        put(&mut writer, &mut length, b"'")?;
    }
    Ok(length)
}
