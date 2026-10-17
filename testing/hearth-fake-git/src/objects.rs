//! Git's object encoding, independent of the protocol machine.
use alloc::{boxed::Box, collections::BTreeMap};
use sha1::{Digest, Sha1};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Oid(pub [u8; 20]);
impl Oid {
    #[must_use]
    pub fn hex(self) -> [u8; 40] {
        let mut hex = [0; 40];
        for (i, byte) in self.0.iter().enumerate() {
            hex[2 * i] = b"0123456789abcdef"[usize::from(byte >> 4)];
            hex[2 * i + 1] = b"0123456789abcdef"[usize::from(byte & 15)];
        }
        hex
    }
    #[must_use]
    pub fn from_hex(hex: &[u8]) -> Option<Self> {
        if hex.len() != 40 {
            return None;
        }
        let mut bytes = [0; 20];
        for (i, pair) in hex.chunks_exact(2).enumerate() {
            let digit = |b: u8| char::from(b).to_digit(16).and_then(|n| u8::try_from(n).ok());
            bytes[i] = digit(pair[0])? * 16 + digit(pair[1])?;
        }
        Some(Self(bytes))
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Type {
    Blob,
    Tree,
    Commit,
    Tag,
}
impl Type {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Blob => "blob",
            Self::Tree => "tree",
            Self::Commit => "commit",
            Self::Tag => "tag",
        }
    }
}
pub(crate) fn hash(kind: Type, body: &[u8]) -> Oid {
    let mut sha = Sha1::new();
    sha.update(format!("{} {}\0", kind.name(), body.len()));
    sha.update(body);
    Oid(sha.finalize().into())
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    File,
    Executable,
    Link,
    Submodule,
}
impl Mode {
    #[must_use]
    pub const fn number(self) -> u32 {
        match self {
            Self::File => 0o100_644,
            Self::Executable => 0o100_755,
            Self::Link => 0o120_000,
            Self::Submodule => 0o160_000,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Entry {
    Blob { mode: Mode, bytes: Box<[u8]> },
    Submodule(Oid),
}
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Tree(pub(crate) BTreeMap<Box<[u8]>, Entry>);
impl Tree {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, path: &[u8], entry: Entry) {
        assert!(!path.is_empty() && !path.contains(&0), "a nonempty path without NUL");
        assert!(path.split(|b| *b == b'/').all(|p| !p.is_empty() && p != b"." && p != b".."), "valid relative path");
        assert!(!matches!(&entry, Entry::Blob { mode: Mode::Submodule, .. }), "a submodule is not a blob");
        for existing in self.0.keys() {
            assert!(
                !existing.strip_prefix(path).is_some_and(|p| p.starts_with(b"/"))
                    && !path.strip_prefix(existing.as_ref()).is_some_and(|p| p.starts_with(b"/")),
                "no file is a directory"
            );
        }
        self.0.insert(path.into(), entry);
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Signature {
    pub name: Box<[u8]>,
    pub email: Box<[u8]>,
    pub seconds: i64,
    pub zone: [u8; 5],
}
impl Signature {
    pub(crate) fn write(&self, role: &str, to: &mut Vec<u8>) {
        to.extend_from_slice(role.as_bytes());
        to.push(b' ');
        to.extend_from_slice(&self.name);
        to.extend_from_slice(b" <");
        to.extend_from_slice(&self.email);
        to.extend_from_slice(b"> ");
        to.extend_from_slice(self.seconds.to_string().as_bytes());
        to.push(b' ');
        to.extend_from_slice(&self.zone);
        to.push(b'\n');
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Commit {
    pub tree: Tree,
    pub parents: Box<[Oid]>,
    pub author: Signature,
    pub committer: Signature,
    pub message: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Tag {
    pub name: Box<[u8]>,
    pub commit: Oid,
    pub tagger: Signature,
    pub message: Box<[u8]>,
}
