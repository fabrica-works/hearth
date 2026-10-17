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
    Failed(Failure),
    Revision(Resolved),
    Hunks(hearth_protocol_git::hunks::Hunks),
    Listing(Box<[Entry]>),
    Blobs(Box<[Blob]>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    Exit(hearth_protocol_git::invocation::Exit),
    Decoder(Box<str>),
}
pub const CASE_NAMES: &[&str] = &[
    "rev-parse-head",
    "rev-parse-prefix",
    "rev-parse-no-repository",
    "ls-tree-slash-order",
    "ls-tree-absent-commit",
    "diff-6-apart",
    "diff-7-apart",
    "diff-equal",
    "diff-missing-side",
    "diff-unreadable-side",
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
    "diff-line-changed",
    "diff-line-added",
    "diff-line-removed",
    "diff-lines-appended",
    "diff-no-newline",
    "diff-binary",
    "diff-past-200-lines",
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
    let mut hunks = Vec::new();
    for line in text.lines() {
        let words: Vec<_> = line.split(' ').collect();
        match words[0] {
            "failed" => {
                return Expected::Failed(match words[1] {
                    "exit" => {
                        Failure::Exit(hearth_protocol_git::invocation::Exit::Code(words[2].parse().expect("exit byte")))
                    }
                    "signal" => {
                        Failure::Exit(hearth_protocol_git::invocation::Exit::Signal(words[2].parse().expect("signal")))
                    }
                    error => Failure::Decoder(error.into()),
                });
            }
            "line" => hunks.push(hearth_protocol_git::hunks::Line {
                kind: kind(words[1]),
                text: unhex(words[2]).into_boxed_slice(),
            }),
            "hunks" => {
                return Expected::Hunks(hearth_protocol_git::hunks::Hunks {
                    lines: hunks.into_boxed_slice(),
                    left_out: words[1].parse().expect("left out"),
                    binary: false,
                });
            }
            "binary" => {
                return Expected::Hunks(hearth_protocol_git::hunks::Hunks {
                    lines: Box::new([]),
                    left_out: 0,
                    binary: true,
                });
            }
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
        Expected::Failed(failure) => match failure {
            Failure::Exit(hearth_protocol_git::invocation::Exit::Code(code)) => {
                writeln!(text, "failed exit {code}").expect("String write");
            }
            Failure::Exit(hearth_protocol_git::invocation::Exit::Signal(signal)) => {
                writeln!(text, "failed signal {signal}").expect("String write");
            }
            Failure::Decoder(error) => writeln!(text, "failed {error}").expect("String write"),
        },
        Expected::Hunks(hunks) => {
            if hunks.binary {
                text.push_str("binary\n");
            } else {
                for line in &hunks.lines {
                    let kind = match line.kind {
                        hearth_protocol_git::hunks::Kind::Hunk => "hunk",
                        hearth_protocol_git::hunks::Kind::Context => "context",
                        hearth_protocol_git::hunks::Kind::Removed => "removed",
                        hearth_protocol_git::hunks::Kind::Added => "added",
                        hearth_protocol_git::hunks::Kind::NoNewline => "no-newline",
                    };
                    writeln!(text, "line {kind} {}", hex(&line.text)).expect("String write");
                }
                writeln!(text, "hunks {}", hunks.left_out).expect("String write");
            }
        }
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

fn kind(text: &str) -> hearth_protocol_git::hunks::Kind {
    use hearth_protocol_git::hunks::Kind;
    match text {
        "hunk" => Kind::Hunk,
        "context" => Kind::Context,
        "removed" => Kind::Removed,
        "added" => Kind::Added,
        "no-newline" => Kind::NoNewline,
        other => panic!("line kind {other}"),
    }
}
