use crate::objects::{Commit, Entry, Mode, Oid, Signature, Tree};
pub fn hearth() -> Signature {
    Signature {
        name: b"hearth".as_slice().into(),
        email: b"hearth@example.org".as_slice().into(),
        seconds: 1_700_000_000,
        zone: *b"+0000",
    }
}
pub fn hello() -> Commit {
    let mut tree = Tree::new();
    tree.insert(b"a.txt", Entry::Blob { mode: Mode::File, bytes: b"hello\n".as_slice().into() });
    Commit { tree, parents: Box::new([]), author: hearth(), committer: hearth(), message: b"first\n".as_slice().into() }
}
pub fn mixed() -> Commit {
    let mut commit = hello();
    for (path, mode, bytes) in [
        (b"a/b".as_slice(), Mode::File, b"x".as_slice()),
        (b"empty", Mode::File, b""),
        (b"link", Mode::Link, b"a.txt"),
        (b"run", Mode::Executable, b"#!/bin/sh\n"),
    ] {
        commit.tree.insert(path, Entry::Blob { mode, bytes: bytes.into() });
    }
    commit.tree.insert(
        b"sub",
        Entry::Submodule(Oid::from_hex(b"0123456789012345678901234567890123456789").expect("literal oid")),
    );
    commit
}
