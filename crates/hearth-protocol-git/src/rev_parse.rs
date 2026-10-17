//! One bounded rev-parse answer (render.md, 2.2).
use crate::{invocation::Exit, oid::Oid};
/// Longest valid output: a SHA-256 id and LF. Keep one more byte to detect excess.
pub const MAX_STDOUT: u32 = 65;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Resolved {
    Commit(Oid),
    Unresolved,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    Exit(Exit),
    Malformed,
}
/// Decode after stdout has ended, retaining at most `MAX_STDOUT + 1` bytes.
pub fn decode(stdout: &[u8], exit: Exit) -> Result<Resolved, Error> {
    match exit {
        Exit::Code(1) => Ok(Resolved::Unresolved),
        Exit::Code(0) => {
            let line = stdout.strip_suffix(b"\n").ok_or(Error::Malformed)?;
            let oid = Oid::parse(line).ok_or(Error::Malformed)?;
            Ok(Resolved::Commit(oid))
        }
        Exit::Code(_) | Exit::Signal(_) => Err(Error::Exit(exit)),
    }
}
