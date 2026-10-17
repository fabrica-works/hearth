# Session 44: fake git, and git's conformance scenario

Fake git, `testing/hearth-fake-git`, keeps repositories under git's
SHA-1 ids (decisions.md, 13), resolves, lists, hands out objects and
diffs two files; its face reads render.md 2.2's vectors and prints what
git prints, as a child on hearth 26's fake machine. Last, hearth 34's
git scenario runs on the fake too, compared with real git.

## Read first

- hearth's testing.md: 3.1, 3.11, 4.1, 4.2, 4.4, 4.5, 5.1 to 5.3, 5.5;
  render.md, 2.2 at `31bed37`; decisions.md, 13, 16, 17;
  code-organization.md, 1 and 3; later.md's rows for the fake.
- `findings/skein-testing.md`, 4, 5, Gaps; `findings/skein-io.md`, 2, 3, 5.
- `hearth/26-fake-git.md`'s sketch (`linux`, `hearth`); hearth 34's
  session file, for 44.3. skein's `testing-strategy.md`: 4, 4.3, 5 (by
  number only).

## What the design asks

- testing.md 4.2: "`rev-parse`, `ls-tree` and `cat-file --batch` on a
  repository, and its `diff --no-index` of two files, by a line diff of
  the fake's own"; 4.1: "Every line it prints has the shape of a recorded
  real run", "run as hearth runs them"; 4.5: "One recording, two uses".
- 5.2: "revisions resolved, listings, blobs; hunks of `diff --no-index` on
  changes whose shortest diff is unique"; 5.3: "Allowed differences are
  data, each with its reason"; 5.5: "It runs when a fake or a recording
  changes".
- render.md 2.2: "git is `/usr/bin/git`, ... skein's `Spawn` searches no
  `PATH`"; "The environment is given whole: `LC_ALL=C`,
  `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`,
  `GIT_NO_REPLACE_OBJECTS=1`, `GIT_TERMINAL_PROMPT=0`, and `HOME=/dev/null`":
  six names, no `PATH` (5.2's is mmdebstrap's).

## Choices this plan makes

- **Settled by the plan:** decisions.md 13, 16, 17; code-organization.md
  1 and 3; later.md's rows for the fake.
- **`hearth-fake-git`** has its own `[lints]` as
  `skein/tests/json/Cargo.toml:17-33`, and a copy of the machine's
  `clippy.toml`.
- **A pure face and a driver:** `face::Run` turns stdin into output and
  an exit; `process::Git` opens and reads as the child and moves its
  pipes' bytes; hearth 28's hosted `Host` wraps the same `Run`.
- **One fault, a stall** (`Faults::stall_after`); the sweep and each
  step's time are hearth 28's. Not in M1's fake: SHA-256 repositories,
  ambiguous hex prefixes (it asserts).
- **Assumptions about hearth 34:** 44.3 checks each against landed
  hearth 34, and where one differs the landed file wins. Recordings at
  `testing/recordings/git/<version>/<case>/` (`argv`,
  `env`, `dir`, `stdin` where read, `stdout`, `stderr`, `exit`, `expect`
  in `hearth_git_world::expect`'s lines; the scratch path masked to
  `/scratch`) read by `testing/hearth-recordings`;
  `tests/conformance/src/git.rs`'s `REPOSITORIES` and `CASES` as data
  (files by path, mode, contents and length, `git.rs:4-14`; the identity
  and date, the commit messages, the tag `v1` and `refs/heads/main` are
  code in `src/fixtures.rs:15-22` and `:77-110`, which 44.3 makes data
  both sides read); the real side `run::git(args, dir, stdin, env) ->
  Run` (`run.rs:26`), each case run from the scratch store's root as hearth
  runs git; the test file `tests/conformance/tests/conformance_git.rs`;
  cases `rev-parse-sha256` and `ls-tree-sha256`; 34.2's seven diff
  cases, `diff-{line-changed, line-added, line-removed, lines-appended,
  no-newline, binary, past-200-lines}`, each side `./cache/<64 hex>`
  from there (02.1's `Command::Diff { store: b".", .. }`, as hearth 25.2).

## Rules that bind it

- **No expected value computed by the code under test:** ids, listings
  and hunks are literals from real git 2.53.0, run once on x1 on
  2026-10-16 (code-organization.md, 6).
- **No hand-written recording, no network:** recordings come from the
  conformance profile with `HEARTH_RECORD=1`; git runs on scratch
  directories only (README, "Live services").
- **Shared files:** the root `Cargo.toml` and `Cargo.lock` (44.1);
  `testing/recordings/` and `tests/conformance/` (44.3).
- **A fake or a recording changed** (testing.md, 5.5): from 44.3, the
  gate's `the_fake_prints_each_git_recording_byte_for_byte` holds fake git
  to the recordings at every landing, and a landing that changes
  `testing/hearth-fake-{machine,git}/` or `testing/recordings/git/` also
  runs `heavy cargo nextest run --workspace --profile conformance` once.

## Increments

### 44.1 Fake git's domain: objects named as git names them
Scope: blobs, trees, commits and annotated tags under git's ids, refs,
`HEAD`; resolution, listings, objects; a line diff (testing.md, 4.2).
Needs: hearth 26.1 (the crate `hearth-fake-machine`, `linux::Machine`).
Files: root `Cargo.toml` (shared: `hearth-fake-git`, `sha1`),
`Cargo.lock` (shared); new `testing/hearth-fake-git/`: `Cargo.toml`
(`hearth-fake-machine`, `sha1` `=0.10.7`), `clippy.toml`, `src/lib.rs`,
`src/objects.rs`, `src/repository.rs`, `src/diff.rs`; tests
`src/tests.rs`, `src/tests/fixtures.rs`, `src/tests/domain.rs`.
Shape: the sketch's 44.1 part. An id is SHA-1 over `<type>
<length>\0` and the body: a tree's, per entry in `tree_order`, `<octal
mode, 40000 for a tree> <name>\0` and the 20-byte id; a commit's, `tree
<hex>\n`, `parent <hex>\n` per parent, `author <name> <<email>>
<seconds> <zone>\n`, `committer ...\n`, `\n`, the message; a tag's,
`object <hex>\ntype commit\ntag <name>\ntagger ...\n\n`, the message.
`resolve`: 40 hex digits naming a stored object; else the first of
`<name>`, `refs/<name>`, `refs/tags/<name>`, `refs/heads/<name>`,
`refs/remotes/<name>`, `refs/remotes/<name>/HEAD` that exists; else 4
to 39 hex digits prefixing one object; a tag peeled; then a commit or
`NotACommit`. `lay` keeps the repository under `path` and lays on the
machine `path` as a directory (0o755) holding `HEAD` (0o644, `ref:
<head>\n`), both owned by `owner`, and absent parents (0o755, root).
`list` gives each blob and submodule in full-path byte
order (git's). `lines` is `Binary` on a NUL in either's first 8,000
bytes; else a shortest line edit script's hunks, 3 lines of context,
changes at most 6 unchanged lines apart sharing one (git 2.53: lines 2
and 9 of 20 share, 2 and 10 do not); an empty `Span` starts at the line
before; `function` is the last line of `before` above the hunk's first
printed line (its first context line) that starts with a letter, `_` or
`$`, cut to 80 bytes, trailing whitespace dropped, then a UTF-8
sequence the cut split dropped whole (git 2.47.3).
Test shape: `src/tests/fixtures.rs` (new: `hearth()`, `hearth
<hearth@example.org> 1700000000 +0000`; `hello()`, `a.txt` "hello\n";
`mixed()`, `a.txt` "hello\n", `a/b` "x", `empty` "", `link` to `a.txt`,
`run` "#!/bin/sh\n" executable, `sub` a submodule at `0123456789` × 4;
both messages "first\n"); `src/tests/domain.rs`, each under 20 ms:
`a_commit_the_world_lays_has_the_id_real_git_gives_it`,
`files_a_link_and_a_submodule_give_git_s_commit_id` (`739a369fdfee74b857461469d8a3a358285a3579`),
`a_tag_and_a_branch_of_one_name_resolve_to_the_tag`,
`an_annotated_tag_peels_to_its_commit`,
`a_tree_s_id_resolves_to_no_commit_naming_the_tree`,
`a_unique_seven_digit_prefix_resolves_its_commit`,
`one_changed_line_gives_one_hunk_with_three_lines_of_context` (`e` to `E`
in `a` to `h`: `@@ -2,7 +2,7 @@ a`, three lines either side),
`changes_six_lines_apart_share_a_hunk_and_seven_do_not`,
`a_nul_in_the_first_8000_bytes_makes_the_pair_binary`.

```rust
#[test]
fn a_commit_the_world_lays_has_the_id_real_git_gives_it() {
    let mut repository = Repository::new(b"refs/heads/main");
    let commit = repository.commit(&fixtures::hello());
    // git 2.53.0 on x1, once: `git commit -m first` of a.txt, as fixtures::hearth().
    assert_eq!(&commit.hex(), b"663d5cdec21ff28ca3e39f3f92e67bbd9b5d72f6", "the commit's id");
    let listed = repository.list(commit).expect("a stored commit lists");
    assert_eq!(&listed[0].oid.hex(), b"ce013625030ba8dba906f756967f9e9ca394464a", "the blob's id");
    assert_eq!((listed[0].mode, listed[0].size), (Mode::File, Some(6)), "a 6-byte regular file");
}
```
Proof: unit, `a_commit_the_world_lays_has_the_id_real_git_gives_it`:
the fake names a commit as real git does. Nothing calls it yet: 44.2
prints it, 44.3 holds it to real git. Bound: 0.02 s.
Don't: depend on `hearth-protocol-git` or a hearth crate but the machine
(testing-strategy.md, 4: "No shared types"); keep a working tree.
Size: at most 300 lines.
Removes: nothing.

### 44.2 Fake git's face, as a process on the fake machine
Scope: 2.2's vectors checked, git's output printed, `cat-file --batch`
line by line, run as a child on the fake machine (testing.md, 4.1, 4.4).
Needs: hearth 26.2 (processes, pipes); 44.1.
Files: new `src/face.rs`, `src/process.rs`; `src/lib.rs`; tests
`src/tests/face.rs` (new), `src/tests/fixtures.rs` (`repositories()`,
`hello()` and `mixed()` at `/srv/{hello,mixed}.git`; `run(..) -> Printed
{ stdout, stderr, exit }`; `on_machine()`, git installed,
the consts `BEFORE` (`a` to `h`, a line each; sha-256
`a8cdd76642f0ecda0067f4d780d027935959e42dc16520220f295191b913efba`) and
`AFTER` (`e` as `E`; `890cc14fc1b27d596b26d5fa9d0e4eeec320fe1f59af07f741b7ec3c24bcbc9c`)
laid at `/srv/store/cache/<that hex>`; `builder`; `spawn_git(..) ->
(Pid, [Handle; 3])`, pipes 0 in, 1, 2 out, and `spawn_git_in(.., dir, ..)`;
`drain`).
Shape: the sketch's 44.2 part. `check` takes the four vectors with 2.2's
options in its order and exactly its six environment names, `HOME` being
`/dev/null`; the diff's is `--git-dir=/dev/null diff --no-index …` (render.md
2.2); anything else asserts (testing.md 4.1). `Run::next` gives, as git 2.53
printed:
- `RevParse`: `<40 hex>\n`, 0; missing: nothing, 1; another type: stderr
  `error: <revision>^{commit}: expected commit type, but the object
  dereferences to <type> type\n`, 1.
- `LsTree`: one `<mode> <type> <hex> <size>\t<path>\0` chunk a record,
  six octal digits, the size right-aligned in 7 columns, `-` for a
  submodule; a missing commit: `fatal: not a tree object\n`, 128.
- `CatFileBatch`: each 40-hex line as it arrives, `<hex> <type>
  <size>\n<bytes>\n` or `<line> missing\n`; another line asserts; 0 at
  stdin's end.
- `DiffNoIndex`: `Same`: nothing, 0. Else `diff --git a/<before>
  b/<after>\n`, `index <7 hex>..<7 hex> <mode>\n` (`100755` for the
  owner's execute bit, else `100644`), then `Binary files a/<before> and
  b/<after> differ\n`, or `--- a/<before>\n`, `+++ b/<after>\n` and each
  hunk: `@@ -<span> +<span> @@`, ` <function>` if any, `,count` left out
  when 1; lines after ` `, `-`, `+`; `\ No newline at end of file\n`
  after a line without one; exit 1. Paths as given, a leading `/`
  dropped (`./cache/<hex>` prints `a/./cache/<hex>`, as 2.53.0).
  Missing: `error: Could not access '<path>'\n`, 1, git's answer when it cannot read a file (render.md 2.2);
  unreadable: `error: open("<path>"): Permission denied\nfatal: cannot
  hash <path>\n`, 128.
- A git-dir the child cannot open, or `Repositories` lacks: `fatal: not
  a git repository: '<git_dir>'\n`, 128.

`Git::start` asserts `Tool::Git` from `/usr/bin/git`; it opens the git-dir, or reads the two files whole, as
the child: an absolute path `At::Root`, a relative one `At::Cwd` beneath
`Invocation::dir` (hearth 25.2's `./cache/<64 hex>`, from the store's
root). `step` ends the child on a pending TERM (`Exit::Signal(15)`) or a
broken stdout (`Exit::Signal(13)`), feeds stdin to `Run`, and sends
output as the pipes have room; it sends exactly `stall_after` stdout
bytes, then nothing more.
Test shape: `src/tests/face.rs`, each under 20 ms, literals git 2.53.0's:
`rev_parse_prints_the_commit_a_branch_names`,
`rev_parse_of_a_tree_exits_1_saying_it_is_no_commit`,
`ls_tree_lists_each_blob_and_submodule_in_tree_order` (real git's
records of `mixed()`, pasted),
`cat_file_answers_each_line_and_names_a_missing_one`,
`diff_prints_its_headers_and_one_hunk_and_exits_1` (`BEFORE` and `AFTER`,
`index 71ac1b5..1877275 100644`),
`a_spawned_git_answers_a_batch_of_blobs_over_its_pipes`,
`a_spawned_diff_reads_relative_sides_beneath_its_working_directory`
(`spawn_git_in` `/srv/store`, `./cache/` `BEFORE`'s and `AFTER`'s hex:
`diff --git a/./cache/<a> b/./cache/<b>`, exit 1),
`a_git_dir_the_child_cannot_open_is_not_a_repository`,
`a_stalled_git_sends_nothing_more_until_terminated`.

```rust
#[test]
fn a_spawned_git_answers_a_batch_of_blobs_over_its_pipes() {
    let (mut machine, repositories) = fixtures::on_machine();
    let builder = fixtures::builder(&mut machine);
    let (child, [stdin, stdout, _]) = fixtures::spawn_git(&mut machine, builder, &[b"--git-dir=/srv/hello.git", b"cat-file", b"--batch"]);
    let mut git = Git::start(&mut machine, child, Faults { stall_after: None });
    let line = b"ce013625030ba8dba906f756967f9e9ca394464a\n".as_slice().into();
    assert_eq!(machine.call(builder, Op::Send { pipe: stdin, bytes: line }), Ok(Reply::Sent(41)), "stdin has room");
    assert_eq!(machine.call(builder, Op::Close { handle: stdin }), Ok(Reply::Done), "stdin ends");
    while git.step(&mut machine, &repositories) == Progress::Working {}
    let printed = fixtures::drain(&mut machine, builder, stdout);
    assert_eq!(printed, b"ce013625030ba8dba906f756967f9e9ca394464a blob 6\nhello\n\n", "header, bytes, newline");
    assert_eq!(machine.call(builder, Op::Wait { child }), Ok(Reply::Exited(Exit::Code(0))), "0 at stdin's end");
}
```
Proof: unit, `a_spawned_git_answers_a_batch_of_blobs_over_its_pipes`:
2.2's `cat-file --batch` spawned through the io face, a blob answered
over the child's pipes, its exit seen by the parent. Nothing above
exists: hearth 28 routes the builder to it; 44.3 holds it to real git.
Bound: 0.02 s.
Don't: give a face to a vector 2.2 lacks; read recordings at run time;
keep a clock.
Size: at most 300 lines.
Removes: nothing.

### 44.3 Git's conformance scenario: the fake against real git
Scope: hearth 34's git scenario on the fake too, compared with real git
live, and with its recordings in the gate (testing.md, 4.5, 5.1 to 5.3,
5.5; development.md, 5.3).
Needs: hearth 34's conformance world, scenario and recordings (Choices).
Files: hearth 34's `tests/conformance/` (`Cargo.toml`, `src/lib.rs`,
`src/git.rs`, `src/fixtures.rs` shared, `tests/conformance_git.rs`); new
`src/fake_git.rs` and `tests/recorded_git.rs` there; hearth 34's
`tests/git/src/{expect.rs, world.rs}`; `testing/recordings/git/<version>/`
(shared, captured); `testing/hearth-fake-git/src/` where a case shows it
wrong.
Shape: `fake_git::run(case, &Recording) -> Run` lays `REPOSITORIES` into
a `Machine` and `Repositories`, and the case's files with their modes,
at the masked paths (`/scratch/…`, so a recording's words replay as they
stand); logs in the test user; spawns the
recording's `argv` (`/usr/bin/git` first), `env`, `dir` and `stdin`;
steps to the exit; returns hearth 34's `Run { exit, captured }`
(`tests/conformance/src/run.rs:10`), `captured.exit` as its text ("0",
"128", "signal N"). The recording's `dir`, `/scratch/store`, is spawned
as `scratch/store` under the root handle, as 44.2's `tests/fixtures.rs`
does: an absolute one is `Refusal::Escape` (`linux/files.rs:118`). `big`
is 10,000,000 bytes of `x`, by `length`; the repositories are bare at
`/scratch/<name>`, the store at `/scratch/store`. The live test gives it
real git's run, masked; the gate test a recording. `ALLOWED: &[Allowed {
case, what, reason }]` holds hearth 34's two SHA-256 cases,
`rev-parse-sha256` and `ls-tree-sha256`, run on real git alone, and
nothing else at first: the difference is the fake's own, which keeps
SHA-1 repositories only (decisions.md, 13; Choices, "Not in M1's fake"),
not 34's. 34.1's and 34.2's cases run as they stand, the seven `diff-*`
among them. Ten cases added, where 34 has none like them: rev-parse of
`HEAD` (`rev-parse-head`), a 7-digit prefix (`rev-parse-prefix`) and a
missing git-dir (`rev-parse-no-repository`: `fatal: not a git
repository`, 128); ls-tree of names `/` orders (`ls-tree-slash-order`:
`a-b`, `a/x`, `a.b`, `a0`) and of an absent commit
(`ls-tree-absent-commit`: `fatal: not a tree object`, 128); diffs of
changes 6 and 7 lines apart (`diff-6-apart`, `diff-7-apart`), equal files
(`diff-equal`, 0), a missing side (`diff-missing-side`, 1, as git exits; render.md 2.2)
and a side of mode 0o000 (`diff-unreadable-side`: `error: open(…)`,
`fatal: cannot hash`, 128; the profile runs as a regular user, 5.5), so
each line 44.2 prints has a recorded run (4.1). Five of them,
`rev-parse-no-repository`, `ls-tree-absent-commit`, `diff-equal`,
`diff-missing-side` and `diff-unreadable-side`, decode to no answer
(02.1's `succeeded` for four, 34.2's `Malformed` for
`diff-missing-side`, exit 1): `hearth_git_world::expect` gains the ten
case names in `CASE_NAMES` and `Expected::Failed`, a line `failed exit <n>` | `failed signal <n>`
| `failed <error>` (the machine's, by name: `malformed` for the missing
side), which `world::replay` answers, so each recording keeps its
`expect`. The ten are recorded once, with `HEARTH_RECORD=1 heavy cargo nextest run -p hearth-conformance-world --profile conformance -E 'test(=each_git_case_prints_on_the_fake_what_real_git_prints)'`.
Test shape: `each_git_case_prints_on_the_fake_what_real_git_prints` in
`tests/conformance_git.rs`: one table test over `CASES`, stdout, stderr
and exit compared whole but for `ALLOWED`, naming the case and the first
differing byte; under 20 s. In the gate (a binary outside
`binary(/^conformance_/)`), `tests/recorded_git.rs`'s
`the_fake_prints_each_git_recording_byte_for_byte`: each recording of
every version, but the cases `ALLOWED` names, through `fake_git::run`,
compared as the live test compares (4.5: "the shape the fake's face
prints"; 5.5); no real git; under 2 s. 34.1's
`every_recording_decodes_to_its_expectation_whatever_the_cuts` takes the
new cases and their `failed` lines as it stands.

```rust
#[test]
fn the_fake_prints_each_git_recording_byte_for_byte() {
    let recordings = hearth_recordings::all("git"); // (case name, Recording), every version
    assert!(!recordings.is_empty(), "recordings are committed");
    for (case, recording) in recordings.iter().filter(|(case, _)| !ALLOWED.iter().any(|a| a.case == &**case)) {
        let run = fake_git::run(case, recording);
        assert_eq!(
            (&run.captured.stdout, &run.captured.stderr, &run.captured.exit),
            (&recording.stdout, &recording.stderr, &recording.exit),
            "{case}"
        );
    }
}
```
Proof: live, `heavy cargo nextest run -p hearth-conformance-world
--profile conformance -E 'test(=each_git_case_prints_on_the_fake_what_real_git_prints)'`:
real git and the fake print the same bytes and exit for each case, on
scratch directories, no network. Bound: 20 s of the profile's 2 minutes.
Then the gate holds the fake to the recordings at every landing.
Don't: write a recording by hand; allow a difference without a reason;
change hearth 34's real side but by adding cases; run real git in
`recorded_git.rs`.
Size: at most 80 lines (fixes to the fake; world lines are not counted).
Removes: nothing.

## Interface sketches

The fake's domain types field by field, the rest as signatures; elided
fields are private.

```rust
// testing/hearth-fake-git/src/{objects,repository,diff}.rs (44.1)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Oid(pub [u8; 20]);  // A git object id: SHA-1 of the object's header and body.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Type { Blob, Tree, Commit, Tag }  // An object's type, as git names it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Mode { File, Executable, Link, Submodule }  // A listed entry's mode: 100644, 100755, 120000 (the blob is the target), 160000.
#[derive(Clone, PartialEq, Eq, Debug)] pub enum Entry { Blob { mode: Mode, bytes: Box<[u8]> }, Submodule(Oid) }  // What a tree's path holds: a blob of any mode but `Submodule`, or another repository's commit.
#[derive(Clone, PartialEq, Eq, Debug, Default)] pub struct Tree(alloc::collections::BTreeMap<Box<[u8]>, Entry>);  // A commit's tree by full path; `insert` asserts a valid path.
/// An author, committer or tagger; `zone` as git prints it, `*b"+0000"`.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Signature { pub name: Box<[u8]>, pub email: Box<[u8]>, pub seconds: i64, pub zone: [u8; 5] }
/// A commit the world lays.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Commit { pub tree: Tree, pub parents: Box<[Oid]>, pub author: Signature, pub committer: Signature, pub message: Box<[u8]> }
/// An annotated tag of a commit, laid under `refs/tags/<name>`.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Tag { pub name: Box<[u8]>, pub commit: Oid, pub tagger: Signature, pub message: Box<[u8]> }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Resolved { Commit(Oid), Missing, NotACommit { kind: Type } }  // What a revision resolves to.
/// One `ls-tree -r -l` record; no size for a submodule.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Listed { pub mode: Mode, pub oid: Oid, pub size: Option<u64>, pub path: Box<[u8]> }
#[derive(Debug)] pub struct Repository { /* objects by id, refs by name, HEAD's target */ }  // One repository's objects, refs and `HEAD`.
#[derive(Debug)] pub struct Repositories(alloc::collections::BTreeMap<Box<[u8]>, Repository>);  // A world's repositories, by their git-dir's absolute path.
#[derive(Clone, PartialEq, Eq, Debug)] pub enum Diff { Same, Binary, Hunks(Box<[Hunk]>) }  // Two files' line diff, `--unified=3`: equal; a NUL in a first 8,000 bytes; the hunks in order.
/// One hunk; each line keeps its newline if it has one.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Hunk { pub before: Span, pub after: Span, pub function: Box<[u8]>, pub lines: Box<[(Line, Box<[u8]>)]> }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Span { pub start: u32, pub count: u32 }  // A hunk's lines on one side: from line `start`, `count` of them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Line { Context, Removed, Added }  // A hunk line's kind.
impl Oid { pub fn hex(self) -> [u8; 40]; pub fn from_hex(hex: &[u8]) -> Option<Oid>; }
impl Tree { pub fn new() -> Tree; pub fn insert(&mut self, path: &[u8], entry: Entry); }
impl Repository { pub fn new(head: &[u8]) -> Repository; pub fn commit(&mut self, commit: &Commit) -> Oid; pub fn tag(&mut self, tag: &Tag) -> Oid; }
impl Repository { pub fn set_ref(&mut self, name: &[u8], to: Oid); pub fn resolve(&self, revision: &[u8]) -> Resolved; }
impl Repository { pub fn list(&self, commit: Oid) -> Option<Box<[Listed]>>; pub fn object(&self, oid: Oid) -> Option<(Type, &[u8])>; }
impl Repositories { pub fn lay(&mut self, machine: &mut hearth_fake_machine::linux::Machine, path: &[u8], owner: hearth_fake_machine::linux::Ids, repository: Repository); }
pub fn lines(before: &[u8], after: &[u8]) -> Diff;
/// git's order of a tree's entries: a tree sorts as its name and `/`.
fn tree_order(a: &[u8], a_tree: bool, b: &[u8], b_tree: bool) -> core::cmp::Ordering {
    fn key(name: &[u8], tree: bool) -> impl Iterator<Item = u8> {
        name.iter().copied().chain(tree.then_some(b'/'))
    }
    key(a, a_tree).cmp(key(b, b_tree))
}

// testing/hearth-fake-git/src/{face,process}.rs (44.2)
pub enum Command { RevParse { git_dir: Box<[u8]>, revision: Box<[u8]> }, LsTree { git_dir: Box<[u8]>, commit: Oid }, CatFileBatch { git_dir: Box<[u8]> }, DiffNoIndex { before: Box<[u8]>, after: Box<[u8]> } }
pub enum Place { Repository, NotARepository, Files { before: Read, after: Read } }
pub enum Read { Contents { bytes: Box<[u8]>, executable: bool }, Missing, Denied }
pub enum Want { Input, Stdout(Box<[u8]>), Stderr(Box<[u8]>), Exit(u8) }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Faults { pub stall_after: Option<u64> }
pub struct Run { /* … */ }  // The pure face: a command and its place, stepped by `next`, `input` and `end`; hearth 28 hosts it.
pub struct Git { /* … */ }  // `Run` as a child on the machine: opens, reads and moves its pipes' bytes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Progress { Working, Waiting, Exited }
pub fn check(args: &[Box<[u8]>], env: &[Box<[u8]>]) -> Command;
impl Run { pub fn new(command: Command, place: Place) -> Run; pub fn next(&mut self, repositories: &Repositories) -> Want; }
impl Run { pub fn input(&mut self, bytes: &[u8]); pub fn end(&mut self); }
impl Git { pub fn start(machine: &mut hearth_fake_machine::Machine, child: hearth_fake_machine::linux::Pid, faults: Faults) -> Git; }
impl Git { pub fn step(&mut self, machine: &mut hearth_fake_machine::Machine, repositories: &Repositories) -> Progress; }
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth ls-tree -d --name-only origin/main
   testing/hearth-fake-git | wc -l` prints 1.
2. In a worktree at `origin/main`, `heavy cargo nextest run -p
   hearth-fake-git` passes, naming at least 18 tests.
3. There, 44.3's Proof command passes.
4. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-44\.[1-3]'` prints 3.
