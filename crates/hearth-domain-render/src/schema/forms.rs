//! Byte forms from format.md, 2.4, 5.3 and 6.2.
use super::{Check, Composed, Composite};
use crate::{
    errors::{Format, Schema},
    revision::{Entry, Mode, declaration::Form},
};
use skein_lib::bytes;
#[must_use]
pub fn check(format: Format, value: &[u8]) -> bool {
    match format {
        Format::Preseed | Format::Module | Format::Param => unreachable!("declaration forms checked by kinds"),
        Format::Link => return crate::kinds::files::link(value),
        Format::Path => {
            return value == b"/"
                || match value.strip_prefix(b"/") {
                    Some(rest) => segments(rest),
                    None => false,
                };
        }
        Format::File => return segments(value),
        Format::Duration => return seconds(value).is_some(),
        Format::Key => return !value.is_empty() && value.len() <= 255,
        Format::Secret if value.len() > 255 || !segments(value) => return false,
        Format::Mode if !(3..=4).contains(&value.len()) => return false,
        Format::Timestamp if value.len() != 16 => return false,
        Format::Debian if !(2..=256).contains(&value.len()) => return false,
        Format::Secret | Format::Mode | Format::Timestamp | Format::Debian => {}
    }
    for (index, byte) in value.iter().enumerate() {
        let valid = match format {
            Format::Secret => byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-/".contains(byte),
            Format::Mode => (b'0'..=b'7').contains(byte),
            Format::Timestamp => {
                if index == 8 {
                    *byte == b'T'
                } else if index == 15 {
                    *byte == b'Z'
                } else {
                    byte.is_ascii_digit()
                }
            }
            Format::Debian => {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || (index > 0 && b"+-.".contains(byte))
            }
            Format::Link
            | Format::Path
            | Format::File
            | Format::Duration
            | Format::Key
            | Format::Preseed
            | Format::Module
            | Format::Param => {
                unreachable!("forms checked above")
            }
        };
        if !valid {
            return false;
        }
    }
    true
}
fn segments(value: &[u8]) -> bool {
    let mut start = 0;
    for index in 0..=value.len() {
        if index == value.len() || value.get(index) == Some(&b'/') {
            let part = value.get(start..index).expect("segment within bytes");
            if part.is_empty() || part == b"." || part == b".." {
                return false;
            }
            start = index.saturating_add(1);
        }
    }
    true
}
pub(super) fn seconds(value: &[u8]) -> Option<i64> {
    let (unit, digits) = value.split_last()?;
    if let Some(b'1'..=b'9') = digits.first() {
    } else {
        return None;
    }
    let factor = match unit {
        b's' => 1,
        b'm' => 60,
        b'h' => 3600,
        b'd' => 86400,
        _unit => return None,
    };
    let mut number = 0_i64;
    for digit in digits {
        if !digit.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(i64::from(digit.checked_sub(b'0')?))?;
    }
    number.checked_mul(factor)
}
pub(super) fn fault(value: &Composed, form: &Form, check: &Check<'_>, listing: &[Entry], max: u64) -> Option<Schema> {
    let Composite::String(value) = &value.kind else {
        return None;
    };
    let format = match form {
        Form::Path => Some(Format::Path),
        Form::Mode => Some(Format::Mode),
        Form::Duration => Some(Format::Duration),
        Form::Secret => Some(Format::Secret),
        Form::File => Some(Format::File),
        Form::Bool
        | Form::Int
        | Form::String
        | Form::Target
        | Form::Enum(_)
        | Form::List(_)
        | Form::Map(_)
        | Form::Object(_) => None,
    };
    if let Some(format) = format
        && !self::check(format, value)
    {
        return Some(Schema::Malformed { format });
    }
    if let Form::Target = form
        && let Check::Target { environment, targets } = check
        && targets.binary_search(value).is_err()
    {
        return Some(Schema::Target { name: value.clone(), environment: bytes::copy_of(environment) });
    }
    if let Form::File = form {
        match file(listing, value) {
            Some(Entry { mode: Mode::File | Mode::Executable, size, .. }) => {
                if *size > max {
                    return Some(Schema::Large { path: value.clone(), max });
                }
            }
            Some(Entry { mode: Mode::Link | Mode::Submodule, .. }) | None => {
                return Some(Schema::File { path: value.clone() });
            }
        }
    }
    None
}
fn file<'a>(listing: &'a [Entry], path: &[u8]) -> Option<&'a Entry> {
    let mut low = 0_usize;
    let mut high = listing.len();
    for _ in 0..=usize::BITS {
        if low == high {
            return None;
        }
        let middle = low.saturating_add(high.saturating_sub(low) / 2);
        let entry = listing.get(middle).expect("bisected listing");
        match entry.path.as_ref().cmp(path) {
            core::cmp::Ordering::Less => low = middle.saturating_add(1),
            core::cmp::Ordering::Greater => high = middle,
            core::cmp::Ordering::Equal => return Some(entry),
        }
    }
    unreachable!("bisection bound")
}
