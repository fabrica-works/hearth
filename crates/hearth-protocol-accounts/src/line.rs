//! One line's fields (domain/store.md, 5.3).
use crate::{File, Ids, Line, Password, What, limits::Limits};
use alloc::boxed::Box;
use skein_lib::bytes;
/// The most fields of any file's line: `/etc/shadow`'s.
const MOST_FIELDS: usize = 9;
/// A layer holds one range of 65,536 ids (domain/store.md, 9.2).
pub const MAX_ID: u32 = 65_535;
/// How many fields a line of `file` has.
const fn fields(file: File) -> usize {
    match file {
        File::Passwd => 7,
        File::Group | File::Gshadow => 4,
        File::Shadow => 9,
    }
}
/// The line's fields, if there are exactly `count`; the rest of the array is empty.
fn split(line: &[u8], count: usize) -> Option<[&[u8]; MOST_FIELDS]> {
    let mut parts: [&[u8]; MOST_FIELDS] = [&[]; MOST_FIELDS];
    let mut found = 0_usize;
    let mut start = 0_usize;
    for (at, byte) in line.iter().enumerate() {
        if *byte == b':' {
            // The field this colon ends is the last one: a colon past it is one too many.
            if found.checked_add(1)? >= count {
                return None;
            }
            *parts.get_mut(found)? = line.get(start..at)?;
            found = found.checked_add(1)?;
            start = at.checked_add(1)?;
        }
    }
    if found.checked_add(1)? != count {
        return None;
    }
    *parts.get_mut(found)? = line.get(start..)?;
    Some(parts)
}
/// A name is not empty, at most `limits.name` bytes, and not a comment or a NIS line.
fn name_of(field: &[u8], limits: &Limits) -> Result<Box<[u8]>, What> {
    match u32::try_from(field.len()) {
        Ok(length) if length > 0 && length <= limits.name => {}
        Ok(_) | Err(_) => return Err(What::Name),
    }
    match field.first() {
        Some(b'#' | b'+' | b'-') | None => Err(What::Name),
        Some(_) => Ok(bytes::copy_of(field)),
    }
}
/// Decimal digits, no leading zero but `0` itself, at most `MAX_ID`.
fn id_of(field: &[u8]) -> Result<u32, What> {
    let Some(first) = field.first() else {
        return Err(What::Id);
    };
    for byte in field {
        if !byte.is_ascii_digit() {
            return Err(What::Id);
        }
    }
    if *first == b'0' && field.len() > 1 {
        return Err(What::Id);
    }
    // Without a leading zero, six digits are at least 100,000.
    if field.len() > 5 {
        return Err(What::Range);
    }
    let mut value = 0_u32;
    for byte in field {
        let digit = u32::from(byte.saturating_sub(b'0'));
        value = value.checked_mul(10).ok_or(What::Range)?.checked_add(digit).ok_or(What::Range)?;
    }
    if value > MAX_ID {
        return Err(What::Range);
    }
    Ok(value)
}
/// A password field is locked when it is one of the values that no password hashes to; the empty
/// field is not one of them.
fn password(file: File, field: &[u8]) -> Password {
    let locked = match file {
        File::Passwd | File::Group => matches_any(field, &[b"x", b"*"]),
        File::Shadow | File::Gshadow => matches_any(field, &[b"*", b"!", b"!!", b"!*"]),
    };
    if locked { Password::Locked } else { Password::Set }
}
fn matches_any(field: &[u8], values: &[&[u8]]) -> bool {
    for value in values {
        if field == *value {
            return true;
        }
    }
    false
}
/// The line's entry, or what was wrong with it.
pub fn parse(file: File, text: &[u8], limits: &Limits) -> Result<Line, What> {
    let Some(parts) = split(text, fields(file)) else {
        return Err(What::Fields);
    };
    let [first, second, third, fourth, ..] = parts;
    let name = name_of(first, limits)?;
    let ids = match file {
        File::Passwd => Ids::User { uid: id_of(third)?, gid: id_of(fourth)? },
        File::Group => Ids::Group { gid: id_of(third)? },
        File::Shadow | File::Gshadow => Ids::Shadow,
    };
    Ok(Line { name, ids, password: password(file, second) })
}
