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

use crate::{
    face::{self, Faults, Place, Read, Run, Want},
    process::{Git, Progress},
    repository::{Repositories, Repository},
};
use hearth_fake_machine::{
    Machine,
    hearth::Tool,
    linux::{At, Exit, Handle, How, Ids, Item, Op, Pid, Pipe, Reply, Sizes, Spawn, Way},
};
pub const OWNER: Ids = Ids { user: 1000, group: 1000 };
pub const BEFORE: &[u8] = b"a\nb\nc\nd\ne\nf\ng\nh\n";
pub const AFTER: &[u8] = b"a\nb\nc\nd\nE\nf\ng\nh\n";
pub const BEFORE_PATH: &[u8] = b"/srv/store/cache/a8cdd76642f0ecda0067f4d780d027935959e42dc16520220f295191b913efba";
pub const AFTER_PATH: &[u8] = b"/srv/store/cache/890cc14fc1b27d596b26d5fa9d0e4eeec320fe1f59af07f741b7ec3c24bcbc9c";
pub const ENV: &[&[u8]] = &[
    b"LC_ALL=C",
    b"GIT_CONFIG_NOSYSTEM=1",
    b"GIT_CONFIG_GLOBAL=/dev/null",
    b"GIT_NO_REPLACE_OBJECTS=1",
    b"GIT_TERMINAL_PROMPT=0",
    b"HOME=/dev/null",
];
pub const BATCH: &[&[u8]] = &[b"--git-dir=/srv/hello.git", b"cat-file", b"--batch"];
pub const BLOB_LINE: &[u8] = b"ce013625030ba8dba906f756967f9e9ca394464a\n";
pub fn repositories(machine: &mut hearth_fake_machine::linux::Machine) -> Repositories {
    let mut repositories = Repositories::default();
    for (path, commit) in [(b"/srv/hello.git".as_slice(), hello()), (b"/srv/mixed.git", mixed())] {
        let mut repository = Repository::new(b"refs/heads/main");
        repository.commit(&commit);
        repositories.lay(machine, path, OWNER, repository);
    }
    repositories
}
#[derive(Debug)]
pub struct Printed {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit: u8,
}
pub fn run(args: &[&[u8]], place: Place, input: &[u8]) -> Printed {
    let mut machine = hearth_fake_machine::linux::Machine::new(1, Sizes::tiny());
    let repositories = repositories(&mut machine);
    let args: Vec<_> = args.iter().map(|a| Box::from(*a)).collect();
    let env: Vec<_> = ENV.iter().map(|e| Box::from(*e)).collect();
    let mut run = Run::new(face::check(&args, &env), place);
    let mut printed = Printed { stdout: Vec::new(), stderr: Vec::new(), exit: 255 };
    loop {
        match run.next(&repositories) {
            Want::Input => {
                run.input(input);
                run.end();
            }
            Want::Stdout(bytes) => printed.stdout.extend_from_slice(&bytes),
            Want::Stderr(bytes) => printed.stderr.extend_from_slice(&bytes),
            Want::Exit(exit) => {
                printed.exit = exit;
                return printed;
            }
        }
    }
}
pub fn on_machine() -> (Machine, Repositories) {
    let mut machine = Machine::new(1, Sizes::tiny());
    let repositories = repositories(machine.linux_mut());
    machine.linux_mut().lay(&[
        Item::directory(b"/usr", 0o755),
        Item::directory(b"/usr/bin", 0o755),
        Item::directory(b"/srv/store", 0o755),
        Item::directory(b"/srv/store/cache", 0o755),
        Item::file(BEFORE_PATH, 0o644, BEFORE),
        Item::file(AFTER_PATH, 0o644, AFTER),
    ]);
    machine.install(b"/usr/bin/git", Tool::Git);
    (machine, repositories)
}
pub fn builder(machine: &mut Machine) -> Pid {
    machine.linux_mut().login(OWNER, b"/srv")
}
pub fn spawn_git(machine: &mut Machine, builder: Pid, args: &[&[u8]]) -> (Pid, [Handle; 3]) {
    spawn_git_in(machine, builder, b"srv", args)
}
pub fn spawn_git_in(machine: &mut Machine, builder: Pid, dir: &[u8], args: &[&[u8]]) -> (Pid, [Handle; 3]) {
    let Reply::Opened(root) = machine
        .call(builder, Op::Open { at: At::Root, path: b"/".as_slice().into(), how: How::Directory })
        .expect("root opens")
    else {
        panic!("opened root");
    };
    let Reply::Spawned { child, pipes } = machine
        .call(
            builder,
            Op::Spawn(Spawn {
                program: b"/usr/bin/git".as_slice().into(),
                args: args.iter().map(|a| Box::from(*a)).collect(),
                env: ENV.iter().map(|a| Box::from(*a)).collect(),
                root,
                dir: dir.into(),
                pipes: Box::new([
                    Pipe { child: 0, way: Way::In },
                    Pipe { child: 1, way: Way::Out },
                    Pipe { child: 2, way: Way::Out },
                ]),
            }),
        )
        .expect("spawn git")
    else {
        panic!("spawned child");
    };
    (child, pipes.into_vec().try_into().expect("three pipes"))
}
pub fn drain(machine: &mut Machine, parent: Pid, pipe: Handle) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        match machine.call(parent, Op::Take { pipe, most: 4096 }).expect("read pipe") {
            Reply::Took(part) => bytes.extend_from_slice(&part),
            Reply::Empty | Reply::Ended => return bytes,
            answer @ (Reply::Opened(_)
            | Reply::Read(_)
            | Reply::Done
            | Reply::Stat(_)
            | Reply::Listed(_)
            | Reply::Spawned { .. }
            | Reply::Sent(_)
            | Reply::Running
            | Reply::Exited(_)) => panic!("pipe result: {answer:?}"),
        }
    }
}
pub fn settle(git: &mut Git, machine: &mut Machine, repositories: &Repositories) {
    for _ in 0..100 {
        if git.step(machine, repositories) == Progress::Exited {
            return;
        }
    }
    panic!("git settles within scenario's steps");
}
pub fn batch() -> (Machine, Pid, Pid, Handle) {
    let (mut machine, repositories) = on_machine();
    let builder = builder(&mut machine);
    let (child, [stdin, stdout, _]) = spawn_git(&mut machine, builder, BATCH);
    let mut git = Git::start(&mut machine, child, Faults { stall_after: None });
    assert_eq!(machine.call(builder, Op::Send { pipe: stdin, bytes: BLOB_LINE.into() }), Ok(Reply::Sent(41)));
    assert_eq!(machine.call(builder, Op::Close { handle: stdin }), Ok(Reply::Done));
    settle(&mut git, &mut machine, &repositories);
    (machine, builder, child, stdout)
}
pub fn diff_args<'a>(before: &'a [u8], after: &'a [u8]) -> Vec<&'a [u8]> {
    vec![
        b"--git-dir=/dev/null",
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
    ]
}
pub fn wait(machine: &mut Machine, parent: Pid, child: Pid) -> Exit {
    let Reply::Exited(exit) = machine.call(parent, Op::Wait { child }).expect("wait child") else {
        panic!("child exited");
    };
    exit
}

pub fn changed_diff() -> Printed {
    run(
        &diff_args(b"before", b"after"),
        Place::Files {
            before: Read::Contents { bytes: BEFORE.into(), executable: false },
            after: Read::Contents { bytes: AFTER.into(), executable: false },
        },
        b"",
    )
}
