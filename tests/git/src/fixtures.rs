use hearth_protocol_git::limits::{self, Limits};
pub const SHIPPED: Limits = limits::SHIPPED;
/// Small enough to exercise every limit without a shipped-size allocation.
pub const SMALL: Limits = Limits {
    entries: 8,
    path: 32,
    paths_total: 256,
    chunk: 16,
    hunk_lines: 8,
    hunk_file: 64,
    tail_lines: 2,
    tail_line: 8,
};

#[derive(Debug)]
pub struct Case {
    pub what: &'static str,
    pub decoder: limits::Decoder,
    pub stdout: Vec<u8>,
    pub requests: Box<[hearth_protocol_git::batch::Request]>,
}
/// Inputs at and one past every configured decoder bound.
#[must_use]
pub fn at_the_limits(limits: &Limits) -> Vec<Case> {
    use hearth_protocol_git::{batch::Request, limits::Decoder, oid::Oid};
    let mut cases = Vec::new();
    let oid = Oid::parse(b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").expect("literal id");
    let header = b"100644 blob 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef       0\t";
    for (count, length, what) in [
        (limits.entries, limits.path, "entries and paths at their limits"),
        (limits.entries + 1, limits.path, "entry count one past"),
        (1, limits.path + 1, "path one past"),
        (1, limits.path + hearth_protocol_git::listing::RECORD, "full listing scan without NUL"),
    ] {
        let mut stdout = Vec::new();
        for _ in 0..count {
            stdout.extend_from_slice(header);
            stdout.extend(std::iter::repeat_n(b'x', usize::try_from(length).expect("small path")));
            stdout.push(0);
        }
        cases.push(Case { what, decoder: Decoder::Listing, stdout, requests: Box::new([]) });
    }
    for (total, what) in
        [(limits.paths_total, "total paths at limit"), (limits.paths_total + 1, "total paths one past")]
    {
        let mut stdout = Vec::new();
        let mut left = total;
        while left > 0 {
            let length = left.min(limits.path);
            assert!(length > 0, "test profile permits paths");
            stdout.extend_from_slice(header);
            stdout.extend(std::iter::repeat_n(b'x', usize::try_from(length).expect("small path")));
            stdout.push(0);
            left -= length;
        }
        cases.push(Case { what, decoder: Decoder::Listing, stdout, requests: Box::new([]) });
    }
    for (length, max, what) in [
        (limits.chunk, u64::from(limits.chunk), "chunk and kind at limit"),
        (limits.chunk + 1, u64::from(limits.chunk) + 1, "chunk one past streams twice"),
        (limits.chunk + 1, u64::from(limits.chunk), "kind one past"),
    ] {
        let mut stdout = format!("{} blob {length}\n", core::str::from_utf8(oid.hex()).expect("hex")).into_bytes();
        stdout.extend(std::iter::repeat_n(0, usize::try_from(length).expect("small body")));
        stdout.push(b'\n');
        cases.push(Case {
            what,
            decoder: Decoder::Batch,
            stdout,
            requests: Box::new([Request::Read { oid, size: u64::from(length), max }]),
        });
    }
    for (count, what) in [(23, "header scan at limit"), (24, "header scan one past")] {
        let mut stdout = format!("{} blob ", core::str::from_utf8(oid.hex()).expect("hex")).into_bytes();
        stdout.extend(std::iter::repeat_n(b'0', count));
        stdout.push(b'\n');
        cases.push(Case {
            what,
            decoder: Decoder::Batch,
            stdout,
            requests: Box::new([Request::Read { oid, size: 0, max: 0 }]),
        });
    }
    for (lines, length, what) in [
        (limits.hunk_lines, 1, "hunk count at limit"),
        (limits.hunk_lines + 1, 1, "hunk count one past"),
        (1, limits.hunk_file - 1, "hunk file at limit"),
        (1, limits.hunk_file, "hunk file one past"),
    ] {
        let mut stdout = b"@@ -1 +1 @@\n".to_vec();
        for _ in 0..lines {
            stdout.push(b'+');
            stdout.extend(std::iter::repeat_n(b'x', usize::try_from(length).expect("line size")));
            stdout.push(b'\n');
        }
        cases.push(Case { what, decoder: Decoder::Diff, stdout, requests: Box::new([]) });
    }
    let mut stdout = Vec::new();
    for _ in 0..=limits.tail_lines {
        stdout.extend(std::iter::repeat_n(b'x', usize::try_from(limits.tail_line + 1).expect("small tail")));
        stdout.push(b'\n');
    }
    cases.push(Case { what: "tail at both limits", decoder: Decoder::Tail, stdout, requests: Box::new([]) });
    cases
}

use hearth_protocol_git::hunks::Kind;
pub type HunkRow = (&'static [u8], &'static [(Kind, &'static [u8])]);
pub const HUNKS: &[HunkRow] = &[
    (b"@@ -1 +1 @@\n-a\n+b\n", &[(Kind::Hunk, b"@@ -1 +1 @@"), (Kind::Removed, b"a"), (Kind::Added, b"b")]),
    (
        b"@@ -1 +1 @@\n x\n+\n\\ No newline at end of file\n",
        &[
            (Kind::Hunk, b"@@ -1 +1 @@"),
            (Kind::Context, b"x"),
            (Kind::Added, b""),
            (Kind::NoNewline, b" No newline at end of file"),
        ],
    ),
];

/// Recorded files and header paths are below 512 bytes; keep the 200-line limit.
pub const RECORDED: Limits = Limits { hunk_file: 512, ..SHIPPED };

/// The mutation sweep is split over seed batches to stay below the focused ceiling.
pub fn check_recording_mutations(seeds: &[u64]) {
    use crate::{
        expect, mutate,
        world::{self, Settings},
    };
    for (case, mut recording) in hearth_recordings::all("git") {
        let expected = expect::decode(recording.expect.as_ref().expect("recorded expectation"));
        for stdout in mutate::all(&recording.stdout) {
            recording.stdout = stdout;
            for seed in seeds {
                if let Some(decoded) = world::replay(&recording, &expected, &RECORDED, &Settings::cut(*seed)) {
                    assert_eq!(decoded, expected, "{case}, seed {seed}");
                }
            }
        }
    }
}
