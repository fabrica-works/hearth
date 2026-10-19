//! The four vectors and complete environment of domain/render.md, 2.2.
use crate::oid::Oid;
use alloc::boxed::Box;
use skein_lib::{Writer, bytes};
/// Fixed executable path; the spawn's program, separate from its arguments.
pub const GIT: &[u8] = b"/usr/bin/git";
/// A revision admitted by the guard; git decides what it resolves to.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Revision(Box<[u8]>);
impl Revision {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refused {
    Empty,
    Long,
    Byte { at: u32 },
    Dash,
}
/// Checks only the revision's alphabet and length, never its git meaning.
pub fn revision(input: &[u8]) -> Result<Revision, Refused> {
    if input.is_empty() {
        return Err(Refused::Empty);
    }
    if input.len() > 255 {
        return Err(Refused::Long);
    }
    if input.starts_with(b"-") {
        return Err(Refused::Dash);
    }
    for (at, byte) in input.iter().enumerate() {
        if !byte.is_ascii_alphanumeric() && !b"._/-".contains(byte) {
            return Err(Refused::Byte { at: u32::try_from(at).expect("revision at most 255 bytes") });
        }
    }
    Ok(Revision(bytes::copy_of(input)))
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Command<'a> {
    RevParse { repository: &'a [u8], revision: &'a Revision },
    LsTree { repository: &'a [u8], commit: Oid },
    CatFile { repository: &'a [u8] },
    Diff { store: &'a [u8], before: &'a [u8; 32], after: &'a [u8; 32] },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Exit {
    Code(u8),
    Signal(u32),
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("owned argument fits address space");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("argument sized first");
    }
    writer.finish()
}
fn cached(store: &[u8], digest: &[u8; 32]) -> Box<[u8]> {
    let mut writer = Writer::new(64);
    for byte in digest {
        for nibble in [byte >> 4_u8, byte & 15] {
            let digit = b"0123456789abcdef".get(usize::from(nibble)).expect("four-bit digit");
            writer.put(&[*digit]).expect("64 digits sized first");
        }
    }
    let hex = writer.finish();
    joined(&[store, b"/cache/", &hex])
}
/// Arguments after the executable; every composite path is one argument.
#[must_use]
pub fn arguments(command: Command<'_>) -> Box<[Box<[u8]>]> {
    match command {
        Command::RevParse { repository, revision } => Box::new([
            joined(&[b"--git-dir=", repository]),
            bytes::copy_of(b"rev-parse"),
            bytes::copy_of(b"--verify"),
            bytes::copy_of(b"--quiet"),
            joined(&[revision.bytes(), b"^{commit}"]),
        ]),
        Command::LsTree { repository, commit } => Box::new([
            joined(&[b"--git-dir=", repository]),
            bytes::copy_of(b"ls-tree"),
            bytes::copy_of(b"-r"),
            bytes::copy_of(b"-l"),
            bytes::copy_of(b"-z"),
            bytes::copy_of(b"--full-tree"),
            bytes::copy_of(commit.hex()),
        ]),
        Command::CatFile { repository } => {
            Box::new([joined(&[b"--git-dir=", repository]), bytes::copy_of(b"cat-file"), bytes::copy_of(b"--batch")])
        }
        Command::Diff { store, before, after } => Box::new([
            bytes::copy_of(b"--git-dir=/dev/null"),
            bytes::copy_of(b"diff"),
            bytes::copy_of(b"--no-index"),
            bytes::copy_of(b"--no-color"),
            bytes::copy_of(b"--no-ext-diff"),
            bytes::copy_of(b"--no-textconv"),
            bytes::copy_of(b"--diff-algorithm=myers"),
            bytes::copy_of(b"--unified=3"),
            bytes::copy_of(b"--"),
            cached(store, before),
            cached(store, after),
        ]),
    }
}
/// The complete envp, in the design's order; nothing is inherited.
#[must_use]
pub fn environment() -> Box<[Box<[u8]>]> {
    Box::new([
        bytes::copy_of(b"LC_ALL=C"),
        bytes::copy_of(b"GIT_CONFIG_NOSYSTEM=1"),
        bytes::copy_of(b"GIT_CONFIG_GLOBAL=/dev/null"),
        bytes::copy_of(b"GIT_NO_REPLACE_OBJECTS=1"),
        bytes::copy_of(b"GIT_TERMINAL_PROMPT=0"),
        bytes::copy_of(b"HOME=/dev/null"),
    ])
}
/// Whether the exit permits trusting this command's decoded answer.
#[must_use]
pub fn succeeded(command: Command<'_>, exit: Exit) -> bool {
    match command {
        Command::RevParse { .. } => exit == Exit::Code(0) || exit == Exit::Code(1),
        Command::LsTree { .. } | Command::CatFile { .. } => exit == Exit::Code(0),
        Command::Diff { .. } => exit == Exit::Code(1),
    }
}
