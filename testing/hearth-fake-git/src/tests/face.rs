use super::fixtures;
use crate::{
    face::{Faults, Place},
    process::{Git, Progress},
};
use hearth_fake_machine::linux::{Exit, Item, Op, Reply, Signal, Target};
#[test]
fn rev_parse_prints_the_commit_a_branch_names() {
    let printed = fixtures::run(
        &[b"--git-dir=/srv/hello.git", b"rev-parse", b"--verify", b"--quiet", b"main^{commit}"],
        Place::Repository,
        b"",
    );
    assert_eq!(printed.stdout, b"663d5cdec21ff28ca3e39f3f92e67bbd9b5d72f6\n");
}
#[test]
fn rev_parse_of_a_tree_says_it_is_no_commit() {
    let printed = fixtures::run(
        &[
            b"--git-dir=/srv/mixed.git",
            b"rev-parse",
            b"--verify",
            b"--quiet",
            b"a0d646ce740e520d88b7d55063467aef4b370011^{commit}",
        ],
        Place::Repository,
        b"",
    );
    assert_eq!(printed.stderr,b"error: a0d646ce740e520d88b7d55063467aef4b370011^{commit}: expected commit type, but the object dereferences to tree type\n");
}
const REF_EXITS: &[(&[u8], u8)] =
    &[(b"main^{commit}", 0), (b"missing^{commit}", 1), (b"a0d646ce740e520d88b7d55063467aef4b370011^{commit}", 1)];
#[test]
fn rev_parse_exits_successfully_only_for_a_commit() {
    for &(revision, exit) in REF_EXITS {
        let printed = fixtures::run(
            &[b"--git-dir=/srv/mixed.git", b"rev-parse", b"--verify", b"--quiet", revision],
            Place::Repository,
            b"",
        );
        assert_eq!(printed.exit, exit, "{}", String::from_utf8_lossy(revision));
    }
}
#[test]
fn ls_tree_lists_each_blob_and_submodule_in_tree_order() {
    let printed = fixtures::run(
        &[
            b"--git-dir=/srv/mixed.git",
            b"ls-tree",
            b"-r",
            b"-l",
            b"-z",
            b"--full-tree",
            b"739a369fdfee74b857461469d8a3a358285a3579",
        ],
        Place::Repository,
        b"",
    );
    assert_eq!(printed.stdout,b"100644 blob ce013625030ba8dba906f756967f9e9ca394464a       6\ta.txt\x00100644 blob c1b0730e0133447badcfd47fd144e254807b06e1       1\ta/b\x00100644 blob e69de29bb2d1d6434b8b29ae775ad8c2e48c5391       0\tempty\x00120000 blob 8d14cbf983b3fad683171c9418998d9f68340823       5\tlink\x00100755 blob 1a2485251c33a70432394c93fb89330ef214bfc9      10\trun\x00160000 commit 0123456789012345678901234567890123456789       -\tsub\x00");
}
#[test]
fn cat_file_answers_each_line_and_names_a_missing_one() {
    let printed = fixtures::run(
        fixtures::BATCH,
        Place::Repository,
        b"ce013625030ba8dba906f756967f9e9ca394464a\nffffffffffffffffffffffffffffffffffffffff\n",
    );
    assert_eq!(
        printed.stdout,
        b"ce013625030ba8dba906f756967f9e9ca394464a blob 6\nhello\n\nffffffffffffffffffffffffffffffffffffffff missing\n"
    );
}
#[test]
fn diff_prints_its_headers_and_one_hunk() {
    assert_eq!(fixtures::changed_diff().stdout,b"diff --git a/before b/after\nindex 71ac1b5..1877275 100644\n--- a/before\n+++ b/after\n@@ -2,7 +2,7 @@ a\n b\n c\n d\n-e\n+E\n f\n g\n h\n");
}
#[test]
fn a_changed_diff_exits_1() {
    assert_eq!(fixtures::changed_diff().exit, 1);
}
#[test]
fn a_spawned_git_answers_a_batch_of_blobs_over_its_pipes() {
    let (mut machine, builder, _, stdout) = fixtures::batch();
    assert_eq!(
        fixtures::drain(&mut machine, builder, stdout),
        b"ce013625030ba8dba906f756967f9e9ca394464a blob 6\nhello\n\n"
    );
}
#[test]
fn a_batch_exits_0_when_stdin_ends() {
    let (mut machine, builder, child, _) = fixtures::batch();
    assert_eq!(fixtures::wait(&mut machine, builder, child), Exit::Code(0));
}
#[test]
fn a_spawned_diff_reads_relative_sides_beneath_its_working_directory() {
    let (mut machine, repositories) = fixtures::on_machine();
    let builder = fixtures::builder(&mut machine);
    let args = fixtures::diff_args(
        b"./cache/a8cdd76642f0ecda0067f4d780d027935959e42dc16520220f295191b913efba",
        b"./cache/890cc14fc1b27d596b26d5fa9d0e4eeec320fe1f59af07f741b7ec3c24bcbc9c",
    );
    let (child, [_, stdout, _]) = fixtures::spawn_git_in(&mut machine, builder, b"srv/store", &args);
    let mut git = Git::start(&mut machine, child, Faults { stall_after: None });
    fixtures::settle(&mut git, &mut machine, &repositories);
    assert_eq!(fixtures::drain(&mut machine,builder,stdout),b"diff --git a/./cache/a8cdd76642f0ecda0067f4d780d027935959e42dc16520220f295191b913efba b/./cache/890cc14fc1b27d596b26d5fa9d0e4eeec320fe1f59af07f741b7ec3c24bcbc9c\nindex 71ac1b5..1877275 100644\n--- a/./cache/a8cdd76642f0ecda0067f4d780d027935959e42dc16520220f295191b913efba\n+++ b/./cache/890cc14fc1b27d596b26d5fa9d0e4eeec320fe1f59af07f741b7ec3c24bcbc9c\n@@ -2,7 +2,7 @@ a\n b\n c\n d\n-e\n+E\n f\n g\n h\n");
}
const BAD_DIRS: &[(&[u8], &[u8])] = &[
    (b"--git-dir=/srv/absent.git", b"fatal: not a git repository: '/srv/absent.git'\n"),
    (b"--git-dir=/srv/denied.git", b"fatal: not a git repository: '/srv/denied.git'\n"),
];
#[test]
fn a_git_dir_the_child_cannot_open_is_not_a_repository() {
    for &(dir, expected) in BAD_DIRS {
        let (mut machine, repositories) = fixtures::on_machine();
        machine.linux_mut().lay(&[Item::directory(b"/srv/denied.git", 0o000)]);
        let builder = fixtures::builder(&mut machine);
        let (child, [_, _, stderr]) = fixtures::spawn_git(&mut machine, builder, &[dir, b"cat-file", b"--batch"]);
        let mut git = Git::start(&mut machine, child, Faults { stall_after: None });
        fixtures::settle(&mut git, &mut machine, &repositories);
        assert_eq!(fixtures::drain(&mut machine, builder, stderr), expected);
    }
}
#[test]
fn a_stalled_git_sends_nothing_more_until_terminated() {
    let (mut machine, repositories) = fixtures::on_machine();
    let builder = fixtures::builder(&mut machine);
    let (child, [_, stdout, _]) = fixtures::spawn_git(
        &mut machine,
        builder,
        &[b"--git-dir=/srv/hello.git", b"rev-parse", b"--verify", b"--quiet", b"HEAD^{commit}"],
    );
    let mut git = Git::start(&mut machine, child, Faults { stall_after: Some(5) });
    for _ in 0..10 {
        git.step(&mut machine, &repositories);
    }
    assert_eq!(fixtures::drain(&mut machine, builder, stdout), b"663d5");
    assert_eq!(git.step(&mut machine, &repositories), Progress::Waiting);
    assert_eq!(fixtures::drain(&mut machine, builder, stdout), b"");
    assert_eq!(
        machine.call(builder, Op::Signal { child, signal: Signal::Terminate, to: Target::Child }),
        Ok(Reply::Done)
    );
    assert_eq!(git.step(&mut machine, &repositories), Progress::Exited);
}
#[test]
fn termination_ends_the_git_child_by_signal_15() {
    let (mut machine, repositories) = fixtures::on_machine();
    let builder = fixtures::builder(&mut machine);
    let (child, _) = fixtures::spawn_git(&mut machine, builder, fixtures::BATCH);
    let mut git = Git::start(&mut machine, child, Faults { stall_after: None });
    machine.call(builder, Op::Signal { child, signal: Signal::Terminate, to: Target::Child }).expect("terminate");
    git.step(&mut machine, &repositories);
    assert_eq!(fixtures::wait(&mut machine, builder, child), Exit::Signal(15));
}
#[test]
fn a_broken_stdout_ends_the_git_child_by_signal_13() {
    let (mut machine, repositories) = fixtures::on_machine();
    let builder = fixtures::builder(&mut machine);
    let (child, [_, stdout, _]) = fixtures::spawn_git(
        &mut machine,
        builder,
        &[b"--git-dir=/srv/hello.git", b"rev-parse", b"--verify", b"--quiet", b"HEAD^{commit}"],
    );
    machine.call(builder, Op::Close { handle: stdout }).expect("close reader");
    let mut git = Git::start(&mut machine, child, Faults { stall_after: None });
    fixtures::settle(&mut git, &mut machine, &repositories);
    assert_eq!(fixtures::wait(&mut machine, builder, child), Exit::Signal(13));
}
