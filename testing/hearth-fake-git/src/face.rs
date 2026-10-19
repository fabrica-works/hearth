//! Git's four argument vectors and byte face (domain/render.md, 2.2).
use crate::{
    diff::{self, Diff, Line, Span},
    objects::{Oid, Type, hash},
    repository::{Repositories, Resolved},
};
use alloc::{boxed::Box, collections::VecDeque};
#[derive(Debug)]
pub enum Command {
    RevParse { git_dir: Box<[u8]>, revision: Box<[u8]> },
    LsTree { git_dir: Box<[u8]>, commit: Oid },
    CatFileBatch { git_dir: Box<[u8]> },
    DiffNoIndex { before: Box<[u8]>, after: Box<[u8]> },
}
#[derive(Debug)]
pub enum Place {
    Repository,
    NotARepository,
    Files { before: Read, after: Read },
}
#[derive(Debug)]
pub enum Read {
    Contents { bytes: Box<[u8]>, executable: bool },
    Missing,
    Denied,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Want {
    Input,
    Stdout(Box<[u8]>),
    Stderr(Box<[u8]>),
    Exit(u8),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Faults {
    pub stall_after: Option<u64>,
}
#[must_use]
pub fn check(args: &[Box<[u8]>], env: &[Box<[u8]>]) -> Command {
    const ENV: &[&[u8]] = &[
        b"LC_ALL=C",
        b"GIT_CONFIG_NOSYSTEM=1",
        b"GIT_CONFIG_GLOBAL=/dev/null",
        b"GIT_NO_REPLACE_OBJECTS=1",
        b"GIT_TERMINAL_PROMPT=0",
        b"HOME=/dev/null",
    ];
    assert_eq!(env.len(), ENV.len(), "exactly domain/render.md 2.2's six environment names");
    for expected in ENV {
        assert_eq!(env.iter().filter(|e| e.as_ref() == *expected).count(), 1, "domain/render.md 2.2's environment");
    }
    let dir = args.first().and_then(|s| s.strip_prefix(b"--git-dir=")).expect("domain/render.md 2.2's git-dir");
    let tail: Vec<_> = args[1..].iter().map(AsRef::as_ref).collect();
    match tail.as_slice() {
        [b"rev-parse", b"--verify", b"--quiet", revision] => Command::RevParse {
            git_dir: dir.into(),
            revision: revision.strip_suffix(b"^{commit}").expect("peel a commit").into(),
        },
        [b"ls-tree", b"-r", b"-l", b"-z", b"--full-tree", commit] => {
            Command::LsTree { git_dir: dir.into(), commit: Oid::from_hex(commit).expect("full commit id") }
        }
        [b"cat-file", b"--batch"] => Command::CatFileBatch { git_dir: dir.into() },
        [
            b"diff",
            b"--no-index",
            b"--no-color",
            b"--no-ext-diff",
            b"--no-textconv",
            b"--diff-algorithm=myers",
            b"--unified=3",
            b"--",
            before,
            after,
        ] => {
            assert_eq!(dir, b"/dev/null", "diff discovers no repository");
            Command::DiffNoIndex { before: (*before).into(), after: (*after).into() }
        }
        _ => panic!("only domain/render.md 2.2's four vectors"),
    }
}
#[derive(Debug)]
pub struct Run {
    command: Command,
    place: Place,
    started: bool,
    input: Vec<u8>,
    ended: bool,
    pending: VecDeque<Want>,
}
impl Run {
    #[must_use]
    pub fn new(command: Command, place: Place) -> Self {
        Self { command, place, started: false, input: Vec::new(), ended: false, pending: VecDeque::new() }
    }
    pub fn input(&mut self, bytes: &[u8]) {
        assert!(matches!(self.command, Command::CatFileBatch { .. }) && !self.ended, "live batch stdin");
        self.input.extend_from_slice(bytes);
    }
    pub fn end(&mut self) {
        self.ended = true;
    }
    pub fn next(&mut self, repositories: &Repositories) -> Want {
        if !self.started {
            self.started = true;
            self.begin(repositories);
        }
        if let Some(want) = self.pending.pop_front() {
            return want;
        }
        let Command::CatFileBatch { git_dir } = &self.command else {
            panic!("no next after exit");
        };
        let length = self
            .input
            .iter()
            .position(|b| *b == b'\n')
            .or_else(|| (self.ended && !self.input.is_empty()).then_some(self.input.len()));
        if let Some(length) = length {
            let line = self.input[..length].to_vec();
            let consumed = (length + 1).min(self.input.len());
            self.input.drain(..consumed);
            let oid = Oid::from_hex(&line).expect("batch asks full object ids");
            let repository = repositories.0.get(git_dir.as_ref()).expect("repository checked at start");
            let bytes = match repository.object(oid) {
                Some((kind, bytes)) => [
                    format!("{} {} {}\n", String::from_utf8_lossy(&oid.hex()), kind.name(), bytes.len()).as_bytes(),
                    bytes,
                    b"\n",
                ]
                .concat(),
                None => [line.as_slice(), b" missing\n"].concat(),
            };
            return Want::Stdout(bytes.into());
        }
        if self.ended { Want::Exit(0) } else { Want::Input }
    }
    fn error(&mut self, bytes: Vec<u8>, exit: u8) {
        self.pending.push_back(Want::Stderr(bytes.into()));
        self.pending.push_back(Want::Exit(exit));
    }
    fn begin(&mut self, repositories: &Repositories) {
        if let Command::DiffNoIndex { before, after } = &self.command {
            let Place::Files { before: left, after: right } = &self.place else {
                panic!("diff has two file reads");
            };
            let (bytes, error, exit) = print_diff(before, after, left, right);
            if !bytes.is_empty() {
                self.pending.push_back(Want::Stdout(bytes.into()));
            }
            if !error.is_empty() {
                self.pending.push_back(Want::Stderr(error.into()));
            }
            self.pending.push_back(Want::Exit(exit));
            return;
        }
        let dir = match &self.command {
            Command::RevParse { git_dir, .. } | Command::LsTree { git_dir, .. } | Command::CatFileBatch { git_dir } => {
                git_dir
            }
            Command::DiffNoIndex { .. } => unreachable!("diff handled"),
        };
        let repository = repositories.0.get(dir.as_ref());
        if matches!(self.place, Place::NotARepository) || repository.is_none() {
            self.error([b"fatal: not a git repository: '", dir.as_ref(), b"'\n"].concat(), 128);
            return;
        }
        let repository = repository.expect("present repository");
        match &self.command {
            Command::RevParse { revision, .. } => match repository.resolve(revision) {
                Resolved::Commit(oid) => {
                    self.pending.push_back(Want::Stdout([oid.hex().as_slice(), b"\n"].concat().into()));
                    self.pending.push_back(Want::Exit(0));
                }
                Resolved::Missing => self.pending.push_back(Want::Exit(1)),
                Resolved::NotACommit { kind } => self.error(
                    [
                        b"error: ",
                        revision.as_ref(),
                        b"^{commit}: expected commit type, but the object dereferences to ",
                        kind.name().as_bytes(),
                        b" type\n",
                    ]
                    .concat(),
                    1,
                ),
            },
            Command::LsTree { commit, .. } => {
                if let Some(listed) = repository.list(*commit) {
                    for entry in listed {
                        let size = entry.size.map_or_else(|| "-".to_owned(), |s| s.to_string());
                        let kind = if entry.size.is_some() { "blob" } else { "commit" };
                        let mut record = format!(
                            "{:06o} {kind} {} {size:>7}\t",
                            entry.mode.number(),
                            String::from_utf8_lossy(&entry.oid.hex())
                        )
                        .into_bytes();
                        record.extend_from_slice(&entry.path);
                        record.push(0);
                        self.pending.push_back(Want::Stdout(record.into()));
                    }
                    self.pending.push_back(Want::Exit(0));
                } else {
                    self.error(b"fatal: not a tree object\n".to_vec(), 128);
                }
            }
            Command::CatFileBatch { .. } => (),
            Command::DiffNoIndex { .. } => unreachable!("diff handled"),
        }
    }
}
fn print_diff(before: &[u8], after: &[u8], left: &Read, right: &Read) -> (Vec<u8>, Vec<u8>, u8) {
    for (path, read) in [(before, left), (after, right)] {
        match read {
            Read::Missing => return (Vec::new(), [b"error: Could not access '", path, b"'\n"].concat(), 1),
            Read::Denied => {
                return (
                    Vec::new(),
                    [b"error: open(\"", path, b"\"): Permission denied\nfatal: cannot hash ", path, b"\n"].concat(),
                    128,
                );
            }
            Read::Contents { .. } => (),
        }
    }
    let (Read::Contents { bytes: a, executable }, Read::Contents { bytes: b, .. }) = (left, right) else {
        unreachable!("reads checked");
    };
    let diff = diff::lines(a, b);
    if diff == Diff::Same {
        return (Vec::new(), Vec::new(), 0);
    }
    let left = [b"a/", before.strip_prefix(b"/").unwrap_or(before)].concat();
    let right = [b"b/", after.strip_prefix(b"/").unwrap_or(after)].concat();
    let mut out = [b"diff --git ".as_slice(), &left, b" ", &right, b"\n"].concat();
    out.extend_from_slice(
        format!(
            "index {}..{} {}\n",
            String::from_utf8_lossy(&hash(Type::Blob, a).hex()[..7]),
            String::from_utf8_lossy(&hash(Type::Blob, b).hex()[..7]),
            if *executable { "100755" } else { "100644" }
        )
        .as_bytes(),
    );
    match diff {
        Diff::Same => unreachable!("same returned"),
        Diff::Binary => {
            out.extend_from_slice(&[b"Binary files ".as_slice(), &left, b" and ", &right, b" differ\n"].concat());
        }
        Diff::Hunks(hunks) => {
            out.extend_from_slice(&[b"--- ".as_slice(), &left, b"\n+++ ", &right, b"\n"].concat());
            for hunk in hunks {
                out.extend_from_slice(format!("@@ -{} +{} @@", span(hunk.before), span(hunk.after)).as_bytes());
                if !hunk.function.is_empty() {
                    out.push(b' ');
                    out.extend_from_slice(&hunk.function);
                }
                out.push(b'\n');
                for (kind, line) in hunk.lines {
                    out.push(match kind {
                        Line::Context => b' ',
                        Line::Removed => b'-',
                        Line::Added => b'+',
                    });
                    out.extend_from_slice(&line);
                    if !line.ends_with(b"\n") {
                        out.extend_from_slice(b"\n\\ No newline at end of file\n");
                    }
                }
            }
        }
    }
    (out, Vec::new(), 1)
}
fn span(span: Span) -> String {
    if span.count == 1 { span.start.to_string() } else { format!("{},{}", span.start, span.count) }
}
