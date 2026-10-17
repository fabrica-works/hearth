//! Repeatable repositories built with git's plumbing under one identity and date.
use crate::{
    git::{self, Case, Kind},
    run::{self, Run},
};
use hearth_git_world::{expect::Expected, world::Blob};
use hearth_protocol_git::{
    invocation::{self, Command, Exit},
    listing::{Entry, Mode},
    oid::Oid,
    rev_parse::Resolved,
};
use hearth_recordings::Mask;
use std::{path::PathBuf, sync::OnceLock};
const IDENTITY: &[(&str, &str)] = &[
    ("GIT_AUTHOR_NAME", "hearth"),
    ("GIT_AUTHOR_EMAIL", "hearth@hearth.invalid"),
    ("GIT_AUTHOR_DATE", "1700000000 +0000"),
    ("GIT_COMMITTER_NAME", "hearth"),
    ("GIT_COMMITTER_EMAIL", "hearth@hearth.invalid"),
    ("GIT_COMMITTER_DATE", "1700000000 +0000"),
];
const MISSING: &[u8] = b"0000000000000000000000000000000000000000";
#[derive(Debug)]
struct Repository {
    path: PathBuf,
    commit: Oid,
    tree: Oid,
    entries: Box<[Entry]>,
    blobs: Box<[Blob]>,
}
#[derive(Debug)]
pub struct Scenario {
    scratch: skein_scratch::Scratch,
    store: PathBuf,
    pub version: Box<str>,
    repositories: Vec<Repository>,
}
impl Scenario {
    #[must_use]
    pub fn new() -> &'static Self {
        static SCENARIO: OnceLock<Scenario> = OnceLock::new();
        SCENARIO.get_or_init(Self::build)
    }
    fn build() -> Self {
        let scratch = skein_scratch::Scratch::new("git-conformance");
        let version = run::git(&[b"--version".to_vec()], scratch.path(), None, &[]);
        assert_eq!(version.exit, Exit::Code(0), "conformance requires /usr/bin/git");
        let version = String::from_utf8(version.captured.stdout).expect("git version text");
        assert!(version.starts_with("git version 2."), "conformance requires git version 2., found {version}");
        let store = scratch.path().join("store");
        std::fs::create_dir_all(store.join("cache")).expect("store cache");
        let mut scenario = Self {
            scratch,
            store,
            version: version.trim().strip_prefix("git version ").expect("git version prefix").into(),
            repositories: Vec::new(),
        };
        for repo in git::REPOSITORIES {
            scenario.repositories.push(scenario.repository(repo));
        }
        scenario
    }
    fn repository(&self, repo: &git::Repository) -> Repository {
        let path = self.scratch.path().join(repo.name);
        self.plumb(
            &[
                "init".into(),
                "--bare".into(),
                "--quiet".into(),
                format!("--object-format={}", repo.format),
                path.to_string_lossy().into_owned(),
            ],
            None,
        );
        let prefix = format!("--git-dir={}", path.display());
        let empty_tree = self.plumb(&[prefix.clone(), "write-tree".into()], None);
        let sub = self.plumb(&[prefix.clone(), "commit-tree".into(), empty_tree], Some(b"submodule\n"));
        let mut entries = Vec::new();
        let mut blobs = Vec::new();
        for file in repo.files {
            let contents = if file.contents.len() == file.length {
                file.contents.to_vec()
            } else {
                vec![file.contents[0]; file.length]
            };
            let oid = if file.mode == "160000" {
                sub.clone()
            } else {
                self.plumb(&[prefix.clone(), "hash-object".into(), "-w".into(), "--stdin".into()], Some(&contents))
            };
            self.plumb(
                &[
                    prefix.clone(),
                    "update-index".into(),
                    "--add".into(),
                    "--cacheinfo".into(),
                    file.mode.into(),
                    oid.clone(),
                    file.path.into(),
                ],
                None,
            );
            let oid = Oid::parse(oid.as_bytes()).expect("git object id");
            let mode = match file.mode {
                "100644" => Mode::File,
                "100755" => Mode::Executable,
                "120000" => Mode::Link,
                "160000" => Mode::Submodule,
                other => panic!("fixture mode {other}"),
            };
            entries.push(Entry {
                path: file.path.as_bytes().into(),
                mode,
                oid,
                size: u64::try_from(file.length).expect("file size"),
            });
            if file.mode != "160000" && file.length < 4096 {
                blobs.push(Blob { oid, bytes: Some(contents.into_boxed_slice()) });
            }
        }
        let tree = self.plumb(&[prefix.clone(), "write-tree".into()], None);
        let commit = self.plumb(&[prefix.clone(), "commit-tree".into(), tree.clone()], Some(b"scenario\n"));
        self.plumb(&[prefix.clone(), "update-ref".into(), "refs/heads/main".into(), commit.clone()], None);
        self.plumb(
            &[prefix, "tag".into(), "-a".into(), "v1".into(), "-m".into(), "scenario tag".into(), commit.clone()],
            None,
        );
        Repository {
            path,
            commit: Oid::parse(commit.as_bytes()).expect("commit id"),
            tree: Oid::parse(tree.as_bytes()).expect("tree id"),
            entries: entries.into_boxed_slice(),
            blobs: blobs.into_boxed_slice(),
        }
    }
    fn plumb(&self, args: &[String], stdin: Option<&[u8]>) -> String {
        let args: Vec<_> = args.iter().map(|s| s.as_bytes().to_vec()).collect();
        let run = run::git(&args, &self.store, stdin, IDENTITY);
        assert_eq!(run.exit, Exit::Code(0), "{args:?}: {}", run.stderr_text());
        String::from_utf8(run.captured.stdout).expect("git plumbing text").trim().to_owned()
    }
    #[must_use]
    pub fn expected(&self, case: &Case) -> Expected {
        let repo = &self.repositories[case.repository];
        match case.kind {
            Kind::Revision("missing") | Kind::Tree => Expected::Revision(Resolved::Unresolved),
            Kind::Revision(_) | Kind::Commit => Expected::Revision(Resolved::Commit(repo.commit)),
            Kind::Listing => Expected::Listing(repo.entries.clone()),
            Kind::Blobs => Expected::Blobs(repo.blobs.clone()),
            Kind::Diff(change) => Expected::Hunks(hearth_protocol_git::hunks::Hunks {
                lines: change
                    .lines
                    .iter()
                    .map(|(kind, text)| hearth_protocol_git::hunks::Line { kind: *kind, text: (*text).into() })
                    .collect(),
                left_out: change.left_out,
                binary: change.binary,
            }),
            Kind::Missing => {
                Expected::Blobs(Box::new([Blob { oid: Oid::parse(MISSING).expect("missing oid"), bytes: None }]))
            }
        }
    }
    #[must_use]
    pub fn run(&self, case: &Case) -> Run {
        let repo = &self.repositories[case.repository];
        let repository = repo.path.as_os_str().as_encoded_bytes();
        let revision = match case.kind {
            Kind::Revision(name) => name.as_bytes(),
            Kind::Commit => repo.commit.hex(),
            Kind::Tree => repo.tree.hex(),
            Kind::Listing | Kind::Blobs | Kind::Missing | Kind::Diff(_) => b"main",
        };
        let revision = invocation::revision(revision).expect("fixture revision");
        let command = match case.kind {
            Kind::Revision(_) | Kind::Commit | Kind::Tree => Command::RevParse { repository, revision: &revision },
            Kind::Diff(_) => Command::Diff { store: b".", before: &git::BEFORE, after: &git::AFTER },
            Kind::Listing => Command::LsTree { repository, commit: repo.commit },
            Kind::Blobs | Kind::Missing => Command::CatFile { repository },
        };
        let stdin = match self.expected(case) {
            Expected::Blobs(blobs) => Some(
                blobs
                    .iter()
                    .flat_map(|b| hearth_protocol_git::batch::request_line(b.oid).into_vec())
                    .collect::<Vec<_>>(),
            ),
            Expected::Listing(_) | Expected::Revision(_) | Expected::Hunks(_) => None,
        };
        if let Kind::Diff(change) = case.kind {
            let names = invocation::arguments(command);
            for (path, contents) in names.iter().rev().take(2).rev().zip([change.before, change.after]) {
                std::fs::write(
                    self.store.join(std::ffi::OsStr::new(core::str::from_utf8(path).expect("cache path"))),
                    contents,
                )
                .expect("cache content");
            }
        }
        let args: Vec<_> = invocation::arguments(command).iter().map(|b| b.to_vec()).collect();
        let mut run = run::git(&args, &self.store, stdin.as_deref(), &[]);
        run.captured.version = self.version.clone();
        run.captured.expect = Some(hearth_git_world::expect::encode(&self.expected(case)));
        Mask::new(self.scratch.path()).recording(&mut run.captured);
        run
    }
}

/// A tiny Packages input for resolve's independent stanza cutter.
pub const RESOLVE_INDEX: &[u8] =
    b"Package: apt\nVersion: 3.3\nArchitecture: amd64\nOther: kept\n\nPackage: base\nVersion: 1\nArchitecture: all\n\n";
