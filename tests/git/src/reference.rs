//! A recursive reading of the revision alphabet and git's one-line answer (render.md, 2.2).
use crate::fixtures::revisions::{REVISION_LIMIT, SHA1_DIGITS, SHA256_DIGITS};
use hearth_protocol_git::{
    invocation::{Exit, Refused},
    rev_parse::Error,
};
/// # Errors
/// Return the guard's first refusal, without asking the git guard.
pub fn revision(bytes: &[u8]) -> Result<Vec<u8>, Refused> {
    if bytes.is_empty() {
        return Err(Refused::Empty);
    }
    if bytes.len() > REVISION_LIMIT {
        return Err(Refused::Long);
    }
    if bytes.starts_with(b"-") {
        return Err(Refused::Dash);
    }
    alphabet(bytes, 0)?;
    Ok(bytes.to_vec())
}
fn alphabet(bytes: &[u8], at: u32) -> Result<(), Refused> {
    let Some((byte, tail)) = bytes.split_first() else {
        return Ok(());
    };
    if !b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._/-".contains(byte) {
        return Err(Refused::Byte { at });
    }
    alphabet(tail, at + 1)
}
/// # Errors
/// Nonzero exits other than one fail; a successful answer is one lowercase id and LF.
pub fn answer(bytes: &[u8], exit: Exit) -> Result<Option<Vec<u8>>, Error> {
    match exit {
        Exit::Code(1) => Ok(None),
        Exit::Code(0) => {
            let Some((b'\n', digits)) = bytes.split_last() else {
                return Err(Error::Malformed);
            };
            if ![SHA1_DIGITS, SHA256_DIGITS].contains(&digits.len()) || !hex(digits) {
                return Err(Error::Malformed);
            }
            Ok(Some(digits.to_vec()))
        }
        Exit::Code(_) | Exit::Signal(_) => Err(Error::Exit(exit)),
    }
}
fn hex(bytes: &[u8]) -> bool {
    match bytes.split_first() {
        None => true,
        Some((byte, tail)) => b"0123456789abcdef".contains(byte) && hex(tail),
    }
}
