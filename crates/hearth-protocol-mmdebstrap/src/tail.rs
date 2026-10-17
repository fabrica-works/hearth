//! Failure tails and causes, deny before allow (render.md, 5.5 and 7.1).
use crate::{limits::Limits, output::Reader, vector::Name};
use alloc::boxed::Box;
use skein_lib::{List, Queue, bytes};
/// Whether the same revision may succeed later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Operational,
    Configuration,
}
/// The cause found over all admitted lines, before their text is shortened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cause {
    Fetch,
    Resolve,
    Space,
    Absent,
    Unknown(Name),
    Unsatisfiable(Option<Name>),
    Other,
}
/// The error kind the cause gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Resolution,
    Tool,
}
impl Cause {
    #[must_use]
    pub const fn class(&self) -> Class {
        match self {
            Self::Fetch | Self::Resolve | Self::Space => Class::Operational,
            Self::Absent | Self::Unknown(_) | Self::Unsatisfiable(_) | Self::Other => Class::Configuration,
        }
    }
    #[must_use]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::Absent | Self::Unknown(_) | Self::Unsatisfiable(_) => Kind::Resolution,
            Self::Fetch | Self::Resolve | Self::Space | Self::Other => Kind::Tool,
        }
    }
}
/// A pipe's last admitted lines, their continuation state and cause.
#[derive(Debug)]
pub struct Tail {
    lines: Queue<Box<[u8]>>,
    cause: Cause,
    continuation: bool,
    package: bool,
}
impl Tail {
    #[must_use]
    pub fn new(limits: &Limits) -> Tail {
        Tail {
            lines: Queue::with_capacity(limits.tail_lines),
            cause: Cause::Other,
            continuation: false,
            package: false,
        }
    }
    #[must_use]
    pub fn lines(&self) -> &Queue<Box<[u8]>> {
        &self.lines
    }
    #[must_use]
    pub const fn cause(&self) -> &Cause {
        &self.cause
    }
    /// The queue, retained text and at most one named cause.
    #[must_use]
    pub fn worst_case(limits: &Limits) -> Option<u64> {
        Queue::<Box<[u8]>>::worst_case(limits.tail_lines)?
            .checked_add(u64::from(limits.tail_lines).checked_mul(u64::from(limits.tail_line))?)?
            .checked_add(u64::from(limits.name))
    }
}
/// A failed run's selected pipe tail and cause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub cause: Cause,
    pub tail: Box<[Box<[u8]>]>,
}
const UNMET: &[u8] = b"The following packages have unmet dependencies:";
const REPLACE: &[u8] = b"However the following packages replace it:";
const ALLOW: &[&[u8]] = &[
    b"dpkg: error",
    b"E: ",
    b"Err:",
    b"No space left on device",
    b"Hash Sum mismatch",
    b"Failed to fetch",
    b"Unable to ",
    b"Could not ",
];
fn has(line: &[u8], text: &[u8]) -> bool {
    bytes::find(line, text).is_some()
}
fn denied(line: &[u8]) -> bool {
    has(line, b"E: mmdebstrap failed to run")
        || has(line, b"E: setup failed: ")
        || has(line, b"W: hooklistener errored out")
        || (has(line, b"I: PID ") && has(line, b" exited with exit code"))
        || (has(line, b"E: apt-get update") && has(line, b"failed: process exited with"))
}
/// Admit a bounded complete line, normalize tab/CR, classify before cutting.
pub fn admit(tail: &mut Tail, line: &[u8], limits: &Limits) {
    // Reject lines with none of the allow patterns' first bytes before copying
    // or searching full patterns. Continuations still pass through unchanged.
    let mut possible = line.starts_with(b" ") || line.starts_with(b"\t") || line.starts_with(b"\r");
    for pattern in ALLOW {
        if let Some(first) = pattern.first() {
            possible |= line.contains(first);
        }
    }
    for pattern in [UNMET, REPLACE] {
        if let Some(first) = pattern.first() {
            possible |= line.contains(first);
        }
    }
    if !possible {
        tail.continuation = false;
        tail.package = false;
        return;
    }
    let bound = usize::try_from(limits.line).expect("u32 fits usize");
    let mut line = bytes::copy_of(line.get(..line.len().min(bound)).expect("bounded prefix"));
    for byte in &mut line {
        if *byte == b'\t' || *byte == b'\r' {
            *byte = b' ';
        }
    }
    if denied(&line) {
        tail.continuation = false;
        tail.package = false;
        return;
    }
    let indented = line.starts_with(b" ");
    let mut keep = tail.continuation && indented;
    for pattern in ALLOW {
        keep |= has(&line, pattern);
    }
    keep |= has(&line, UNMET) || has(&line, REPLACE);
    if !keep {
        tail.continuation = false;
        tail.package = false;
        return;
    }
    if tail.package && indented {
        if let Some(name) = dependency(&line, limits) {
            match &tail.cause {
                Cause::Unsatisfiable(None) => tail.cause = Cause::Unsatisfiable(Some(name)),
                Cause::Fetch
                | Cause::Resolve
                | Cause::Space
                | Cause::Absent
                | Cause::Unknown(_)
                | Cause::Unsatisfiable(Some(_))
                | Cause::Other => {}
            }
        }
        tail.package = false;
    }
    let candidate = cause(&line, limits);
    if candidate != Cause::Other
        && (tail.cause == Cause::Other
            || (tail.cause.class() == Class::Configuration && candidate.class() == Class::Operational))
    {
        tail.cause = candidate;
    }
    tail.package = has(&line, UNMET);
    tail.continuation = (tail.continuation && indented)
        || has(&line, b"dpkg: error")
        || has(&line, b"E: ")
        || has(&line, UNMET)
        || has(&line, REPLACE);
    if limits.tail_lines != 0 {
        if tail.lines.room() == 0 {
            drop(tail.lines.pop());
        }
        let bound = usize::try_from(limits.tail_line).expect("u32 fits usize");
        tail.lines.push(bytes::copy_of(line.get(..line.len().min(bound)).expect("bounded tail prefix")));
    }
}
fn dependency(line: &[u8], limits: &Limits) -> Option<Name> {
    let mut start = 0_usize;
    for byte in line {
        if *byte != b' ' {
            break;
        }
        start = start.checked_add(1)?;
    }
    let line = line.get(start..)?;
    let end = bytes::find(line, b" :")?;
    Name::new(line.get(..end)?, limits)
}
fn after(line: &[u8], text: &[u8]) -> Option<usize> {
    bytes::find(line, text)?.checked_add(text.len())
}
fn cause(line: &[u8], limits: &Limits) -> Cause {
    if has(line, b"Temporary failure resolving") || has(line, b"Could not resolve") {
        return Cause::Resolve;
    }
    if has(line, b"No space left on device") {
        return Cause::Space;
    }
    let fetch = has(line, b"Err:") || has(line, b"Failed to fetch");
    if has(line, b"does not have a Release file")
        || (fetch && (has(line, b"404  Not Found") || has(line, b"NOT FOUND")))
    {
        return Cause::Absent;
    }
    if fetch {
        return Cause::Fetch;
    }
    if let Some(start) = after(line, b"Unable to locate package ")
        && let Some(name) = Name::new(line.get(start..).expect("pattern end within line"), limits)
    {
        return Cause::Unknown(name);
    }
    if let Some(start) = after(line, b"Package '") {
        let rest = line.get(start..).expect("pattern end within line");
        if let Some(name) = rest.strip_suffix(b"' has no installation candidate")
            && let Some(name) = Name::new(name, limits)
        {
            return Cause::Unknown(name);
        }
    }
    if has(line, b"Unable to correct problems") || has(line, UNMET) {
        return Cause::Unsatisfiable(None);
    }
    Cause::Other
}
/// Prefer stderr whenever it admitted a line; otherwise take stdout.
#[must_use]
pub fn failure(stderr: &Reader, stdout: &Reader) -> Failure {
    let tail = if stderr.tail().lines().is_empty() { stdout.tail() } else { stderr.tail() };
    let mut lines = List::with_capacity(tail.lines.len());
    for line in &tail.lines {
        lines.push(line.clone()).expect("tail count bounded");
    }
    Failure { cause: tail.cause.clone(), tail: lines.into_boxed() }
}
