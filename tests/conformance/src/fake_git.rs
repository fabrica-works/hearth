//! The real scenario's data, laid under the recording's masked paths.
use crate::{
    git::{self, Case, Kind},
    run::Run,
};
use hearth_fake_git::{
    face::Faults,
    objects::{Commit, Entry, Mode, Signature, Tag, Tree},
    process::{Git, Progress},
    repository::{Repositories, Repository},
};
use hearth_fake_machine::{
    Machine,
    hearth::Tool,
    linux::{self, At, Exit, Handle, How, Ids, Item, Op, Pid, Pipe, Reply, Sizes, Spawn, Way},
};
use hearth_recordings::Recording;
use std::sync::OnceLock;
#[derive(Debug)]
pub struct Allowed {
    pub case: &'static str,
    pub what: &'static str,
    pub reason: &'static str,
}
pub const ALLOWED: &[Allowed] = &[
    Allowed {
        case: "rev-parse-sha256",
        what: "SHA-256 repository",
        reason: "M1's fake stores SHA-1 objects only (decisions.md, 13)",
    },
    Allowed {
        case: "ls-tree-sha256",
        what: "SHA-256 repository",
        reason: "M1's fake stores SHA-1 objects only (decisions.md, 13)",
    },
];
const OWNER: Ids = Ids { user: 1000, group: 1000 };
fn signature(role: &str) -> Signature {
    let value = |field: &str| {
        git::IDENTITY.iter().find(|(key, _)| *key == format!("GIT_{role}_{field}")).expect("scenario identity field").1
    };
    let (seconds, zone) = value("DATE").split_once(' ').expect("git date");
    Signature {
        name: value("NAME").as_bytes().into(),
        email: value("EMAIL").as_bytes().into(),
        seconds: seconds.parse().expect("seconds"),
        zone: zone.as_bytes().try_into().expect("git zone"),
    }
}
fn commit(tree: Tree, message: &[u8]) -> Commit {
    Commit {
        tree,
        parents: Box::new([]),
        author: signature("AUTHOR"),
        committer: signature("COMMITTER"),
        message: message.into(),
    }
}
fn repositories() -> &'static Repositories {
    static REPOSITORIES: OnceLock<Repositories> = OnceLock::new();
    REPOSITORIES.get_or_init(|| {
        let mut filesystem = linux::Machine::new(44, Sizes::tiny());
        let mut repositories = Repositories::default();
        for data in git::REPOSITORIES.iter().filter(|repo| repo.format == "sha1") {
            let mut repository = Repository::new(git::BRANCH.as_bytes());
            let sub = repository.commit(&commit(Tree::new(), git::SUBMODULE_MESSAGE));
            let mut tree = Tree::new();
            for file in data.files {
                let entry = if file.mode == "160000" {
                    Entry::Submodule(sub)
                } else {
                    let mode = match file.mode {
                        "100644" => Mode::File,
                        "100755" => Mode::Executable,
                        "120000" => Mode::Link,
                        other => panic!("scenario mode {other}"),
                    };
                    let bytes = if file.length == file.contents.len() {
                        file.contents.into()
                    } else {
                        vec![file.contents[0]; file.length].into()
                    };
                    Entry::Blob { mode, bytes }
                };
                tree.insert(file.path.as_bytes(), entry);
            }
            let oid = repository.commit(&commit(tree, git::COMMIT_MESSAGE));
            repository.tag(&Tag {
                name: git::TAG.as_bytes().into(),
                commit: oid,
                tagger: signature("COMMITTER"),
                message: [git::TAG_MESSAGE.as_bytes(), b"\n"].concat().into(),
            });
            repositories.lay(&mut filesystem, format!("/scratch/{}", data.name).as_bytes(), OWNER, repository);
        }
        repositories
    })
}
fn machine(case: &Case, recording: &Recording) -> Machine {
    let mut machine = Machine::new(44, Sizes::tiny());
    machine.linux_mut().lay(&[
        Item::directory(b"/scratch", 0o755),
        Item::directory(b"/scratch/store", 0o755),
        Item::directory(b"/scratch/store/cache", 0o755),
        Item::directory(b"/usr", 0o755),
        Item::directory(b"/usr/bin", 0o755),
    ]);
    machine.install(b"/usr/bin/git", Tool::Git);
    for repo in git::REPOSITORIES.iter().filter(|repo| repo.format == "sha1") {
        machine.linux_mut().lay(&[
            Item::directory(format!("/scratch/{}", repo.name).as_bytes(), 0o755).owner(OWNER),
            Item::file(
                format!("/scratch/{}/HEAD", repo.name).as_bytes(),
                0o644,
                format!("ref: {}\n", git::BRANCH).as_bytes(),
            )
            .owner(OWNER),
        ]);
    }
    if let Some(change) = git::change(case) {
        let paths = &recording.argv[recording.argv.len() - 2..];
        for (i, (path, bytes)) in paths.iter().zip([change.before, change.after]).enumerate() {
            if i == 1 && matches!(case.kind, Kind::DiffMissing) {
                continue;
            }
            let mode = if i == 1 && matches!(case.kind, Kind::DiffDenied) { 0o000 } else { 0o644 };
            let bytes = if matches!(case.kind, Kind::DiffEqual) { change.before } else { bytes };
            let path = [recording.dir.as_slice(), b"/", path.strip_prefix(b"./").unwrap_or(path)].concat();
            machine.linux_mut().lay(&[Item::file(&path, mode, bytes).owner(OWNER)]);
        }
    }
    machine
}
fn take(machine: &mut Machine, parent: Pid, pipe: Handle, out: &mut Vec<u8>) {
    loop {
        let reply = machine.call(parent, Op::Take { pipe, most: 4096 }).expect("parent pipe");
        if let Reply::Took(bytes) = reply {
            out.extend_from_slice(&bytes);
        } else {
            assert!(matches!(reply, Reply::Empty | Reply::Ended), "pipe answers bytes or an end");
            return;
        }
    }
}
#[must_use]
pub fn run(name: &str, recording: &Recording) -> Run {
    let case = git::CASES.iter().find(|case| case.name == name).expect("named scenario case");
    let mut machine = machine(case, recording);
    let parent = machine.linux_mut().login(OWNER, b"/scratch/store");
    let Reply::Opened(root) = machine
        .call(parent, Op::Open { at: At::Root, path: b"/".as_slice().into(), how: How::Directory })
        .expect("open root")
    else {
        panic!("opened root");
    };
    let Reply::Spawned { child, pipes } = machine
        .call(
            parent,
            Op::Spawn(Spawn {
                program: recording.argv[0].as_slice().into(),
                args: recording.argv[1..].iter().map(|a| a.clone().into()).collect(),
                env: recording.env.iter().map(|e| e.clone().into()).collect(),
                root,
                dir: recording.dir.strip_prefix(b"/").unwrap_or(&recording.dir).into(),
                pipes: Box::new([
                    Pipe { child: 0, way: Way::In },
                    Pipe { child: 1, way: Way::Out },
                    Pipe { child: 2, way: Way::Out },
                ]),
            }),
        )
        .expect("spawn recorded git")
    else {
        panic!("spawned child");
    };
    if let Some(input) = &recording.stdin {
        assert_eq!(
            machine.call(parent, Op::Send { pipe: pipes[0], bytes: input.clone().into() }),
            Ok(Reply::Sent(u32::try_from(input.len()).expect("bounded batch stdin")))
        );
    }
    machine.call(parent, Op::Close { handle: pipes[0] }).expect("stdin ends");
    let mut git = Git::start(&mut machine, child, Faults { stall_after: None });
    let mut captured = recording.clone();
    captured.stdout.clear();
    captured.stderr.clear();
    for _ in 0..10000 {
        let progress = git.step(&mut machine, repositories());
        take(&mut machine, parent, pipes[1], &mut captured.stdout);
        take(&mut machine, parent, pipes[2], &mut captured.stderr);
        if progress == Progress::Exited {
            let Reply::Exited(exit) = machine.call(parent, Op::Wait { child }).expect("wait git") else {
                panic!("exited child");
            };
            let exit = match exit {
                Exit::Code(code) => {
                    captured.exit = code.to_string().into();
                    hearth_protocol_git::invocation::Exit::Code(code)
                }
                Exit::Signal(signal) => {
                    captured.exit = format!("signal {signal}").into();
                    hearth_protocol_git::invocation::Exit::Signal(signal)
                }
            };
            return Run { exit, captured };
        }
    }
    panic!("recorded git settles within scenario's bound")
}

/// Whole outputs are compared; the diagnostic identifies the first unequal byte.
pub fn assert_same(case: &str, run: &Recording, expected: &Recording) {
    let fields = [
        ("stdout", run.stdout.as_slice(), expected.stdout.as_slice()),
        ("stderr", run.stderr.as_slice(), expected.stderr.as_slice()),
        ("exit", run.exit.as_bytes(), expected.exit.as_bytes()),
    ];
    let difference = fields.iter().find(|(_, a, b)| a != b).map(|(field, a, b)| {
        let at = a.iter().zip(*b).position(|(a, b)| a != b).unwrap_or(a.len().min(b.len()));
        format!("{field}, first differing byte {at}")
    });
    assert_eq!(
        (&run.stdout, &run.stderr, &run.exit),
        (&expected.stdout, &expected.stderr, &expected.exit),
        "{case}: {difference:?}"
    );
}
