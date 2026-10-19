//! The lines of `/etc/passwd`, `/etc/group`, `/etc/shadow` and `/etc/gshadow` (store.md, 5.3;
//! build.md, 9.3; activator.md, 2.3).
//!
//! A file is read from its whole bytes into typed entries: each name with its ids, and whether its
//! password field is locked. The files are an output's, so untrusted: the read is bounded, never
//! panics, and an error names a line number and what was wrong, never the content.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod limits;
mod line;
use alloc::boxed::Box;
use limits::Limits;
pub use line::MAX_ID;
use skein_lib::List;
/// Which file a read is of; it fixes the fields' count and what an id and a password are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum File {
    Passwd,
    Group,
    Shadow,
    Gshadow,
}
/// One line: its name, ids and whether its password field is locked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub name: Box<[u8]>,
    pub ids: Ids,
    pub password: Password,
}
/// The ids of a line, each at most `MAX_ID`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ids {
    /// `/etc/passwd`: the user and primary group.
    User { uid: u32, gid: u32 },
    /// `/etc/group`.
    Group { gid: u32 },
    /// `/etc/shadow` and `/etc/gshadow` carry none.
    Shadow,
}
/// A password field is `Locked` when no password hashes to it; anything else, the empty field
/// included, is `Set` (a group's fields are read as a user's). Its bytes are not kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Password {
    Locked,
    Set,
}
/// Why a read failed; no variant carries the file's content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The bytes past `limits.file`.
    File,
    /// More lines than `limits.lines`.
    Lines,
    /// The first line, counted from 1, that does not parse.
    Line { number: u32, what: What },
}
/// What was wrong with a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum What {
    /// Not the file's count of fields: an empty line, too, has one.
    Fields,
    /// Empty, longer than `limits.name`, or the start of a comment or of a NIS entry (`#`, `+`, `-`).
    Name,
    /// Not decimal digits alone, or a leading zero.
    Id,
    /// An id past `MAX_ID`.
    Range,
    /// A name an earlier line of the file has.
    Duplicate,
}
/// The lines of a file: the last may lack its `\n`, an empty file has none, and every line is
/// checked in order, the first that fails ending the read. Fields past those the entries keep are
/// not interpreted.
pub fn read(file: File, bytes: &[u8], limits: &Limits) -> Result<Box<[Line]>, Refused> {
    match u32::try_from(bytes.len()) {
        Ok(length) if length <= limits.file => {}
        Ok(_) | Err(_) => return Err(Refused::File),
    }
    // Counted before anything is set aside.
    let count = count(bytes);
    if count > limits.lines {
        return Err(Refused::Lines);
    }
    let mut lines: List<Line> = List::with_capacity(count);
    let mut start = 0_usize;
    for (at, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            take(file, bytes.get(start..at), limits, &mut lines)?;
            start = at.saturating_add(1);
        }
    }
    if start < bytes.len() {
        take(file, bytes.get(start..), limits, &mut lines)?;
    }
    Ok(lines.into_boxed())
}
/// The lines of `bytes`: its `\n`s, and a last line that lacks one.
fn count(bytes: &[u8]) -> u32 {
    let mut count = 0_u32;
    for byte in bytes {
        if *byte == b'\n' {
            count = count.saturating_add(1);
        }
    }
    match bytes.last() {
        Some(b'\n') | None => count,
        Some(_) => count.saturating_add(1),
    }
}
/// Parse one line and append it, once its name is known to be new.
///
/// Names are compared one by one with the earlier lines': at most `limits.lines` squared
/// comparisons, each of at most `limits.name` bytes and ended at once by a length that differs, so
/// no hashing and no sorting, and the first line to fail is the one reported.
fn take(file: File, text: Option<&[u8]>, limits: &Limits, lines: &mut List<Line>) -> Result<(), Refused> {
    let number = lines.len().saturating_add(1);
    let Some(text) = text else {
        return Err(Refused::Line { number, what: What::Fields });
    };
    let line = match line::parse(file, text, limits) {
        Ok(line) => line,
        Err(what) => return Err(Refused::Line { number, what }),
    };
    for earlier in lines.iter() {
        if earlier.name == line.name {
            return Err(Refused::Line { number, what: What::Duplicate });
        }
    }
    match lines.push(line) {
        Ok(()) => Ok(()),
        Err(_) => Err(Refused::Lines),
    }
}
/// The heap a read holds, its result included: the list of lines, and the names, which are the
/// file's bytes at most.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let names = u64::from(limits.lines).checked_mul(u64::from(limits.name))?.min(u64::from(limits.file));
    List::<Line>::worst_case(limits.lines)?.checked_add(names)
}
#[cfg(test)]
mod tests;
