//! The recording's independent typed expectations (testing.md, 4.5).
use crate::world::Blob;
use hearth_protocol_git::{
    listing::{Entry, Mode},
    oid::Oid,
    rev_parse::Resolved,
};
use std::fmt::Write;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expected {
    Revision(Resolved),
    Listing(Box<[Entry]>),
    Blobs(Box<[Blob]>),
}
pub const CASE_NAMES: &[&str] = &[
    "rev-parse-branch",
    "rev-parse-commit",
    "rev-parse-tag",
    "rev-parse-missing",
    "rev-parse-tree",
    "rev-parse-sha256",
    "ls-tree-modes",
    "ls-tree-empty",
    "ls-tree-sha256",
    "cat-file-blobs",
    "cat-file-missing",
];
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        write!(text, "{byte:02x}").expect("write to String");
    }
    text
}
#[must_use]
pub fn unhex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).expect("hex text"), 16).expect("hex pair"))
        .collect()
}
fn oid(text: &str) -> Oid {
    Oid::parse(text.as_bytes()).expect("expected object id")
}
#[must_use]
pub fn decode(bytes: &[u8]) -> Expected {
    let text = std::str::from_utf8(bytes).expect("expected UTF-8");
    let mut entries = Vec::new();
    let mut blobs = Vec::new();
    for line in text.lines() {
        let words: Vec<_> = line.split(' ').collect();
        match words[0] {
            "resolved" => return Expected::Revision(Resolved::Commit(oid(words[1]))),
            "unresolved" => return Expected::Revision(Resolved::Unresolved),
            "entry" => entries.push(Entry {
                mode: mode(words[1]),
                oid: oid(words[2]),
                size: words[3].parse().expect("size"),
                path: unhex(words[4]).into_boxed_slice(),
            }),
            "listed" => {
                assert_eq!(entries.len(), words[1].parse::<usize>().expect("count"));
                return Expected::Listing(entries.into_boxed_slice());
            }
            "blob" => {
                let bytes = unhex(words[3]);
                assert_eq!(bytes.len(), words[2].parse::<usize>().expect("size"));
                blobs.push(Blob { oid: oid(words[1]), bytes: Some(bytes.into_boxed_slice()) });
            }
            "missing" => blobs.push(Blob { oid: oid(words[1]), bytes: None }),
            other => panic!("unknown expectation {other}"),
        }
    }
    Expected::Blobs(blobs.into_boxed_slice())
}
fn mode(text: &str) -> Mode {
    match text {
        "100644" => Mode::File,
        "100755" => Mode::Executable,
        "120000" => Mode::Link,
        "160000" => Mode::Submodule,
        other => panic!("mode {other}"),
    }
}
#[must_use]
pub fn encode(expected: &Expected) -> Vec<u8> {
    let mut text = String::new();
    match expected {
        Expected::Revision(Resolved::Commit(oid)) => {
            writeln!(text, "resolved {}", String::from_utf8_lossy(oid.hex())).expect("String write");
        }
        Expected::Revision(Resolved::Unresolved) => text.push_str("unresolved\n"),
        Expected::Listing(entries) => {
            for e in entries {
                let mode = match e.mode {
                    Mode::File => "100644",
                    Mode::Executable => "100755",
                    Mode::Link => "120000",
                    Mode::Submodule => "160000",
                };
                writeln!(text, "entry {mode} {} {} {}", String::from_utf8_lossy(e.oid.hex()), e.size, hex(&e.path))
                    .expect("String write");
            }
            writeln!(text, "listed {}", entries.len()).expect("String write");
        }
        Expected::Blobs(blobs) => {
            for b in blobs {
                let oid = String::from_utf8_lossy(b.oid.hex());
                match &b.bytes {
                    Some(bytes) => {
                        writeln!(text, "blob {oid} {} {}", bytes.len(), hex(bytes)).expect("String write");
                    }
                    None => {
                        writeln!(text, "missing {oid}").expect("String write");
                    }
                }
            }
        }
    }
    text.into_bytes()
}
