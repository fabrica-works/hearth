# Session 34: git's conformance tier, its recordings replayed, the hunks

The rest of git's machines (render.md, 2.2, 6.1), on hearth 02's. 34.1 is
the first tier outside the gate: the conformance tier (`tests/conformance`,
`hearth-conformance-world`, the `conformance` profile), real git's side of
testing.md 5.2's "git" and its recordings, replayed cut and mutated by
`tests/git`; 34.2 `diff --no-index`'s hunks. hearth 02's machines run on
real git here before hearth 11 calls them; hearth 41, 44 and 46 use the
kit and the scenario.

## Read first

- hearth's `render.md` (at `175e92f`): 1 (second bullet), 2.1, 2.2, 6.1,
  8.2, 9.4; `client/README.md`, 3.5; `workflow.md`, 1 and 2.
- `testing.md`: 2 (table, 2.4, 2.8, 2.9), 4.1, 4.2 (git's row), 4.5, 5.1 to
  5.3, 5.5, 9.2; `.config/nextest.toml`.
- `decisions.md`, 9, 12, 13; `findings/skein-testing.md`, 3, 6, 7.
- temper's `tools/git-transcripts/src/{lib.rs,mask.rs}` (captures, masks).
- `hearth/02-git.md` whole (the machines, `invocation`, `Limits`, the
  sketches), and `hearth/25-diff.md`, 25.2 (it spawns diff).

## What the design asks

- **render.md 6.1:** "exits 1 when they differ", "the headers before the
  first `@@`, which are skipped", one `Binary files … differ` line, "An
  exit past 1 is a tool error", "at most 200 lines a file, with the count
  of those left out".
- **testing.md 4.1 to 9.2:** "Captured, never written by hand"; real git
  "on scratch repositories"; a 2-minute ceiling; without git "it refuses
  to start, naming what is missing".
- **testing.md 2.10:** real tiers "refuse to start, rather than skip".

## Choices this plan makes

- **Two increments:** the old 02.4 and 02.5 of hearth 02, split off by
  decisions.md, 20. hearth 41.1 needs 34.1; hearth 44.3 all of it;
  hearth 25.2 needs 34.2.
- **Machines as hearth 02's:** `Fill(1)` at a line's start, `Close` answered
  by `Closed`, `MaxOut { above: 1, below: 1 }`; `Hunks` stands only with
  `invocation::succeeded`'s exit (hearth 02, Choices). Up goes one `Hunks`
  (render.md 8.2).
- **The recordings' layout** (code-organization.md, 3):
  `testing/recordings/<tool>/<version>/<case>/` holds a file for each
  `Recording` field (34.1): `argv` an argument a line, argv[0] first; `env`
  `NAME=value` a line; `dir` always, `stdin` where read (testing.md, 4.5); `exit`
  `<code>` or `signal <n>`; `expect` in the tool's world's format, where
  the scenario knows it. `<version>` is `git --version` less `git version
  `. The scratch path, with or without its first `/`, is masked to
  `/scratch` or `scratch`; ids stay. `HEARTH_RECORD=1` writes and passes;
  without it a run unlike its recording, or a version with none, fails
  naming the case and the flag. One kit, `testing/hearth-recordings`, for
  hearth 41, 44, 46 too.
- **Conformance** is `tests/conformance/tests/conformance_<scenario>.rs`,
  selected by `binary(/^conformance_/)`, its repositories built by git's
  plumbing under one identity and date, so ids repeat and fake git can
  make them (decisions.md, 13), in `skein-scratch` under the system's
  temporary directory (`CARGO_TARGET_TMPDIR` lies in hearth's checkout).
  **Each case runs as hearth runs git** (testing.md, 4.1): hearth 02.1's
  `arguments`, never typed again, from the scratch store's root (hearth 11:
  `Spawn::root`, `dir` empty), under `environment()`; a
  diff's sides are `./cache/<64 hex>`, as hearth 25.2 spawns them, under
  02.1's `--git-dir=/dev/null diff --no-index …` (render.md 2.2: no
  repository wherever it runs, a work tree's attributes included).

## Rules that bind it

- **Profiles outside the gate** (README): 34.1 adds the `conformance`
  profile "with its first test, and its binaries join the default and
  fuzzy profiles' exclusions in the same landing"; "never in the gate".
- **No hand-written recording** (code-organization.md, 6): each file under
  `testing/recordings/git/` comes from `HEARTH_RECORD=1 heavy cargo nextest
  run --workspace --profile conformance`; a renamed case's old directory
  goes in the same landing.

## Increments

### 34.1 The conformance tier; git's recordings, replayed
Scope: testing.md 5's tier for M1's git: the profile (9.2), the real side
of 5.2's "git" for revisions, listings and blobs, the recordings (4.5),
their replay in the machine world (2.4).
Needs: nothing beyond hearth 02 (02.3; git on the machine: README, Prerequisites, 2).
Files: `testing/hearth-recordings/` (new: `Cargo.toml` with its own lints
table, `clippy.toml` as `tests/clippy.toml`, `src/{lib.rs, tests.rs}`);
`tests/conformance/` (new: `Cargo.toml`, `src/{lib.rs, fixtures.rs, run.rs,
git.rs}`, `tests/conformance_git.rs`); `tests/git/{Cargo.toml,
src/lib.rs (`pub mod expect, mutate`), src/expect.rs, src/mutate.rs,
src/world.rs, tests/recordings.rs,
tests/fuzzy_git.rs}`; `testing/recordings/git/<version>/` (shared,
captured); `.config/nextest.toml` (shared); root `Cargo.toml`
(`skein-scratch`, the two crates), `Cargo.lock`.
Shape:
- `.config/nextest.toml`: `[profile.conformance]` with `default-filter =
  "binary(/^conformance_/)"`, `global-timeout = "2m"`, `slow-timeout = {
  period = "30s", terminate-after = 2 }`, `retries = 0`, `fail-fast =
  false`, under a comment naming its command and `HEARTH_RECORD=1`; `and
  not binary(/^conformance_/)` on the default and fuzzy filters, and
  `default-filter = "not binary(/^conformance_/)"` on `measure`. No other
  `global-timeout` or `period` line changes (README, Done when 8).
- `hearth_recordings`: `Recording { argv, env, dir, stdin, stdout, stderr,
  exit, expect }` (`stdin` and `expect` optional), `read(tool, version,
  case)`, `all(tool)` (every version's), `write(..)` (the case's directory
  replaced whole), `Mask::new(scratch)`, `recording_on()`.
- `run::git(args, dir, stdin) -> Run`, real git from ordinary Rust:
  `Command::new` of `invocation::GIT`, `env_clear` then
  `environment()`'s pairs, `current_dir(dir)`, stdin written whole and
  closed, `wait_with_output`. Bound: stdin under 4 KiB (41 bytes an id, at
  most 16); outputs of a few KiB; a hung git ended by the profile's
  `terminate-after` (run.rs has no timer).
- `git::{REPOSITORIES, CASES}` as data (files by path, mode and contents),
  which hearth 44 lays into fake git. `fixtures::Scenario::new()` checks
  that `GIT --version` prints `git version 2.` (or fails naming it), makes
  `store/cache/`, and builds each repository by `init --bare`,
  `hash-object -w --stdin`, `update-index --add --cacheinfo`, `write-tree`,
  `commit-tree`, `tag -a` and `update-ref`, as `hearth
  <hearth@hearth.invalid>` at `1700000000 +0000`; the ids they print are
  the expectations. Eleven cases: `rev-parse-{branch, commit, tag, missing,
  tree, sha256}`, `ls-tree-{modes, empty, sha256}` (the four modes, a
  nested path, a path with a space and a byte past ASCII, an empty file, a
  10,000,000-byte file never read), `cat-file-{blobs, missing}`.
- `hearth_git_world::expect::Expected::{Revision, Listing, Blobs}`, one
  item a line: `resolved <oid>` | `unresolved`; `entry <mode> <oid> <size>
  <path hex>` … `listed <n>`; `blob <oid> <size> <hex>` | `missing <oid>`.
  `world::replay(&Recording, &Expected, &limits, &Settings)`;
  `mutate::all(stdout)`: each record or line cut after its first, middle and
  last-but-one byte, each id's first
  digit raised to `A`, the last byte dropped.
Test shape: `tests/conformance/tests/conformance_git.rs`, under 2 s each:
`real_git_resolves_each_revision_as_the_scenario_made_it`,
`real_git_lists_each_scenario_tree_with_its_modes_and_sizes` (reference),
`real_git_s_batch_gives_each_blob_the_scenario_wrote`,
`each_case_s_run_is_its_recording_for_this_git_version`. Kit:
`a_recording_written_reads_back_file_for_file`,
`the_scratch_path_is_masked_with_and_without_its_slash`. In the gate, in
`tests/git/tests/recordings.rs`, each case through `world::replay` at seeds
0 to 3: `every_recording_decodes_to_its_expectation_whatever_the_cuts` (at
least eleven recordings), `every_recording_ran_as_hearth_runs_git` (argv[0]
`GIT`, the rest hearth 02.1's `arguments` of what they name, `env`
`environment()`, `dir` `/scratch/store`),
`cut_and_raised_recordings_fail_or_decode_as_recorded`,
`every_recording_directory_is_a_case` (each `testing/recordings/git/*/*`
is named by `expect::CASE_NAMES`, the case names of `git::CASES`); fuzzy
`random_recordings_cut_mutated_ended_and_failed_settle` (256 seeds).

```rust
// tests/conformance/tests/conformance_git.rs
#[test]
fn real_git_lists_each_scenario_tree_with_its_modes_and_sizes() {
    let scenario = Scenario::new();
    for case in git::CASES.iter().filter(|case| case.is_listing()) {
        let run = scenario.run(case);
        assert_eq!(run.exit, Exit::Code(0), "{}: {}", case.name, run.stderr_text());
        let expected = scenario.expected(case);
        let decoded = world::replay(&run.recording(), &expected, &limits::SHIPPED, &Settings::whole());
        assert_eq!(decoded, expected, "{}: as the scenario laid it out", case.name);
    }
}
```
Proof: live: the profile once with `HEARTH_RECORD=1` (eleven cases
written), then once without: four tests pass,
`real_git_lists_each_scenario_tree_with_its_modes_and_sizes` among them:
hearth's vectors run real git, and the machines read what it printed as
the scenario made it; bound 30 s a run; world: in the gate,
`every_recording_decodes_to_its_expectation_whatever_the_cuts`.
Don't: run real git in the gate; `#[ignore]` a test in place of the
profile; mask ids (decisions.md, 13); write the fake side (hearth 44).
Size: at most 140 lines (the kit; worlds are not counted). Removes: nothing.

### 34.2 The hunks: `diff --no-index`, at most 200 lines a file
Scope: render.md 2.2's and 6.1's hunks; 5.2's cases on changes whose
shortest diff is unique, recorded and replayed.
Needs: nothing beyond 34.1.
Files: `src/{lib.rs, limits.rs, hunks.rs}`, `src/tests.rs`,
`src/tests/hunks.rs`; `tests/git/src/{fixtures.rs (`at_the_limits` gains
`Diff` rows), generate.rs, world.rs, expect.rs}`,
`tests/git/tests/{hunks.rs, fuzzy_git.rs}`; `tests/conformance/src/git.rs`,
`tests/conformance/tests/conformance_git.rs`;
`testing/recordings/git/<version>/diff-*/`.
Shape: `Limits` gains `hunk_lines` (200) and `hunk_file` (1 MiB);
`Decoder` gains `Diff` (its lines' `List` at `hunk_lines`, kept text at
twice `hunk_file`, one delivery of `chunk`), and `at_the_limits` its rows;
`hunks::{Kind, Line, Hunks, Request::{Read, Close}, Event::{Hunks(Hunks),
Failed(Error), Closed}, Diff, up, down, UP_MAX_OUT, DOWN_MAX_OUT}`;
`Error::{Malformed, TooLarge, Truncated, Stream(Fault)}`. A line:
`Fill(1)`, then scans `{ until: LF, max: chunk }` to its `\n`; the 200
count runs from the first `@@`, each `@@` line among them (render.md 6.1),
and `\ No newline at end of file` is a kind, kept and counted; a line past
`hunk_file`, or kept text past twice it, is `TooLarge`; an end with no
hunk and no binary line, `Malformed`: so are git's exit 0 and its exit 1
with nothing on stdout (`succeeded` false), each a tool error that leaves
the change without its hunks (render.md 2.2). `expect` gains `line <kind> <hex>` …
`hunks <left out>` | `binary`. Seven cases, each run as hearth 25.2 runs it
(Choices): `arguments(Command::Diff { store: b".", before, after })` of two
literal 32-byte names, from the scratch store's root, so git reads
`./cache/<64 hex>` and prints `a/./cache/<64 hex>` (2.53.0):
`diff-{line-changed, line-added, line-removed, lines-appended,
no-newline, binary, past-200-lines}`, their lines literals of the case.
Test shape: unit, each as hearth 02.2's reference:
`the_headers_before_the_first_hunk_are_skipped` (headers, `@@ -1 +1 @@`,
`-a`, `+b`: three lines), `each_hunk_line_decodes_to_its_kind_and_text`,
`lines_past_the_limit_are_counted_never_kept`,
`a_binary_files_line_answers_binary_with_no_line`,
`an_output_with_no_hunk_and_no_binary_line_fails`,
`a_line_longer_than_a_chunk_is_read_whole`; hearth 02.3's memory test takes the
`Diff` rows as it stands; world
`generated_hunks_decode_to_their_lines_whatever_the_cuts`; fuzzy
`random_hunks_cut_ended_failed_and_closed_settle` (256 seeds); conformance
`real_git_s_hunks_are_the_unique_shortest_diff_of_each_change`; 34.1's
replay and `every_recording_ran_as_hearth_runs_git` take the new cases as
they stand.
Proof: live: 34.1's two runs, once each:
`real_git_s_hunks_are_the_unique_shortest_diff_of_each_change` passes, and
after the first, `git status --short -uall testing/recordings | grep -vc
'/diff-'` prints 0 and `ls -d testing/recordings/git/*/diff-*/ | wc -l`
prints 7; bound 30 s a run.
Don't: compute a text diff in step code (6.1).
Size: at most 240 lines. Removes: nothing.


## Interface sketches

This session owns these; hearth 25 translates the `Hunks`
(code-organization.md, 3). hearth 02's derives apply; `Kind` is also `Copy, PartialOrd, Ord`.
`Limits` (hearth 02.2) gains `hunk_lines` and `hunk_file`.

```rust
// crates/hearth-protocol-git/src/limits.rs (34.2 adds to hearth 02's)
/// git's machines' limits; the new fields: 6.1's hunk lines and a hunk file's bytes.
pub struct Limits { /* entries, path, paths_total, chunk, */ pub hunk_lines: u32, pub hunk_file: u32 }
/// render.md 6.1's defaults join `SHIPPED`.
pub const SHIPPED: Limits = Limits { /* … */ hunk_lines: 200, hunk_file: 1_048_576 };
/// `Decoder` gains `Diff`: `hunks::Diff`.
pub enum Decoder { /* Listing, Batch, */ Diff }

// crates/hearth-protocol-git/src/hunks.rs (34.2)
/// A hunk line's kind, by its first byte: `@@ … @@` Hunk (its text the whole line), ` ` Context, `-` Removed,
/// `+` Added, `\ No newline at end of file` `NoNewline` (render.md 6.1, 8.2).
pub enum Kind { Hunk, Context, Removed, Added, NoNewline }
/// One line, its first byte and `\n` left out (a `Hunk` line kept whole).
pub struct Line { pub kind: Kind, pub text: Box<[u8]> }
/// Mirrors render.md 8.2's `Hunks { lines: (kind, text), left out, binary }`; render's mirrors it (hearth 25).
pub struct Hunks { pub lines: Box<[Line]>, pub left_out: u32, pub binary: bool }
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth ls-tree -d --name-only origin/main
   tests/conformance testing/hearth-recordings testing/recordings/git |
   wc -l` prints 3.
2. `for v in $(git -C ~/src/rust/hearth ls-tree --name-only origin/main
   testing/recordings/git/); do git -C ~/src/rust/hearth ls-tree -r
   --name-only origin/main "$v" | grep -c '/exit$'; done` prints, for each
   version directory, at least 18 (hearth 44.3 adds).
3. `git -C ~/src/rust/hearth show origin/main:.config/nextest.toml |
   grep -c -F 'binary(/^conformance_/)'` prints 4.
4. In a worktree of hearth at `origin/main`, `heavy cargo nextest run
   --workspace --profile conformance` passes once, its `Summary` naming
   at least the five `conformance_git` tests (hearth 41, 44 and 46 add
   theirs), within the 2-minute ceiling.
5. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-34\.[1-2]'` prints 2.
