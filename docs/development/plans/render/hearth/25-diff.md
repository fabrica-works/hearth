# Session 25: the typed diff, rebuild, hunks, `hearth diff`

render.md, section 6, on the thin story's path: `hearth diff <from>
<to>` renders both revisions in one call, compares each target's two
specs into 6.1's typed change, and prints it through hearth 40's diff
records. 25.1: the two sides, a change per target, the rebuild and the
tally, through the real loop; 25.2: a changed file's sizes and git's
hunks, git run on the two cache files. Meta's change,
both sides' errors, `hearth diff` in the binary and the packages
change's resolution are hearth 45's.

## Read first

- hearth's `render.md` (at `31bed37`): 1, 2.2, 4.1, 4.5, 5.1, 5.4, 6.1 to
  6.3, 8.2, 8.4, 10; `client/README.md`: 3.1 to 3.3; `store.md`: 2.1;
  `format.md`: 3.3, 3.4; `testing.md`: 2.7.
- `decisions.md`, 2 and 6; `findings/skein-io.md`, 2 and 6.
- The drafts: `hearth/02-git.md` (02.1's `invocation`, `Command::Diff`,
  `succeeded`), `34-conformance.md` (34.2's `hunks`),
  `40-client-diff-errors.md` (40.1), `11-thin-story.md` (Choices; "Where
  later sessions add"; 11.3, 32; the sketches), `44-fake-git.md` (44.2's
  `DiffNoIndex`).
- skein's `programming-model.md`: 2, 3, 4, 5.2, 6.3, 7;
  `testing-strategy.md`: 2.8, 2.9 (by number only).

## What the design asks

- **The typed diff** (6.1): `change = added { spec digest } | removed |
  same | changed { parts, rebuild } | failed { side: from | to | both, errors }` per
  target, with `from` and `to`; config's tree entries (meta's placement
  and packages' resolution are hearth 45's).
- **Hunks** (6.1, 8.2, 2.2): a changed file's digests, sizes and hunks
  when both sides are under 1 MiB: at most 200 lines a file, the count
  left out, or binary; git "runs outside any repository, on two cache
  files, and exits 1 when they differ" (`--git-dir=/dev/null`); "Step code
  computes no text diff."
- **Touched and rebuilt** (6.2): a change touches the targets whose spec
  digest differs, and those added or removed; "Rebuild is the lowest
  layer whose key changed", printed as 6.3 shows.
- **The command** (client/README.md, 3.1 to 3.3): `hearth diff <from>
  <to>`; "1 if anything differs ... a failed target ... 2 or 75, never 1".

## Choices this plan makes

- **Two sides, in turn:** `from`, then `to`, each hearth 11's
  one-revision pipeline (`call::Side`) holding its specs, one git child
  at a time. A named target on neither side, or a revision a side cannot
  resolve, ends the call `Usage`: a revision naming no commit is 64, the
  known targets render, the unknown one is an error of the answer, 64
  ranking worst; each target once, in name order (client/README.md 3.1,
  3.3).
- **A change a target, in name order,** `same` included (text leaves it
  out, 40.1); a target's hunks asked one pair at a time and held until
  its `Change` is answered, no bound of render's own (render.md 6.1, 7.2;
  the printer's refusal of a record is 40's wording).
- **Hunk lines:** five kinds, `no newline` among them, kept and counted;
  the 200 count from the first `@@`, `@@` lines included (render.md 6.1,
  8.2).
- **Rebuild from the parts' digests,** 6.2's rule: the packages key moves
  only with the resolution digest or the packages part's (4.5), whose
  values make the resolution key (5.1), whose closure a builder keeps
  (5.4); so the lowest output part whose digest differs is the lowest
  layer whose key does. Its resolution is `Needed` until hearth 45.2 (5.7).
- **A part's change is its part's:** `diff.rs` holds the frame and the
  matches over `PartRecord`; each part's module its `change`, a side
  without the part comparing as its empty record. Here config's tree
  (meta's is hearth 45.1's); each part's session, which Needs 25.1,
  writes that part's change: `live`'s (hearth 19) its paths
  added, removed and changed, layer `Other`, so rebuild `nothing`
  (render.md 6.1, 4.4).
- **Sizes and hunks** (25.2): the call keeps each cached digest's size
  (`Cached`; hearth 15's `Keep`, whichever lands second); `Hunks` only
  for two files of differing digests, both under 1 MiB. `LineKind` takes
  34.2's names (records' `Header` is `Hunk`; 25.2 adds `NoNewline`).
- **git's diff** is 02.1's `Command::Diff { store: b".", .. }` beneath the
  store's root (`dir` empty, as hearth 11 runs git): `./cache/<64 hex>`,
  its vector `git --git-dir=/dev/null diff --no-index …` as 02.1 has it
  (render.md 2.2): no repository wherever it runs, a work tree's
  configuration and attributes included.
- **The tally** counts the targets compared, `changed` those added,
  removed or changed (6.2's touched), `unchanged` the same, a failed one
  in `targets` alone; a call ended before both sides list has none.
- **A failed side** is `failed { side }` whenever a side that has the
  target did not render, `both` when each has it and neither did; a
  failed resolution leaves the change standing; each side's errors keep
  their side (render.md 6.1, 6.3); its class outranks `Negative`.
- **Transitional:** 25.1's files without sizes or hunks (ends in 25.2);
  25.2's failed hunks without an error (hearth 45.1); `Needed` always
  (hearth 45.2).

## Rules that bind it

- **The meeting point** (README, Rules): each increment proves on the
  real loop or the binary, and extends `translate/changes.rs` or
  `hunks.rs` (both ways in `src/tests/translate.rs`) with each variant it
  adds to a vocabulary crossing `hearth-protocol-builder`.
- **Shared files** (README, Rules): render's `src/{lib, vocabulary,
  call}.rs`, `src/parts/`; `translate/`; the records' `records.rs`,
  `diff.rs`, `form/` (version 1, hearth 10's and 40's: added to, never renamed);
  `tests/builder/src/fixtures.rs`. Landing second, rebase and keep both.
- **No expected value computed by the code under test** (code-org., 6):
  printed lines and hunks are git's literals, run once, pasted (git 2.47.3
  and 2.53.0 print the same).


## Increments

### 25.1 Two revisions, a change a target, through the real loop
Scope: 6.1's frame and 6.2: the diff call's two sides; a change per
target (`added`, `removed`, `same`, `changed` with config's tree
entries, `failed`); the rebuild; the tally; `Outcome::Negative`; the
root's `Call::Diff` and `Answer::Change`; `translate/changes.rs`.
Needs: hearth 11; hearth 40.1 (`Change`, `TargetChange`, `Rebuild`,
`Tally`, `Terminal`'s `diff`); hearth 32.1 (`src/parts/config.rs`).
Files: render `src/{lib, vocabulary, call}.rs` (shared), new
`src/diff.rs`, `src/parts/config.rs` (32.1's), `src/tests.rs`, new
`src/tests/diff.rs`, `src/tests/fixtures.rs`, `src/tests/outcome.rs`
(`OUTCOMES` and `WORST` grow to 5 and 5 by 5, `Negative` between
`TryAgain` and `Done`); the root's `src/{lib,
vocabulary}.rs`, `src/tests.rs`; protocol `src/translate.rs`, new
`src/translate/changes.rs`, `src/translate/answers.rs`,
`src/tests/translate.rs`; `tests/builder/{src/fixtures.rs,
tests/real.rs}`; the render world's `tests/render/src/parent.rs` (`:43`'s
`Answer::Rendered { outcome }` takes `diff`, and explicit arms for
`Answer::Change`, `Request::Hunks`, `Event::Diff` and `Hunks`: the world
denies `wildcard_enum_match_arm`) and `tests/render/tests/thin.rs`
(`:102`'s match gains an `Answer::Change` arm). A further exhaustive match
on these enums gains its arms the same way, named in the commit body.
Shape: the sketches. `call.rs`: hearth 11's one-revision state becomes
`call::Side`; a render call holds one and answers as it goes, a diff
call two, `from` then `to`, each keeping what it made of each target
(`diff::Made`), then answers `diff::compare` per target, one `Answer::Change`
per `resume`, targets in name order, then `Rendered { outcome, diff:
Some(tally) }`. `compare`: a
side that has it failed, `Failed { side }` (`Both` if each has it and neither
rendered); else a side
lacking it, `Added` or `Removed`; equal spec digests, `Same`; else
`Changed`, each part whose digest differs (by name) and `rebuild` over
their layers. `parts::config::change`
walks both trees by path, in byte order. The outcome is the worst of
each side's failures, then `Negative` if a change is not `Same`.
Test shape: unit (`crates/hearth-domain-render/src/tests/diff.rs`, new,
each under 20 ms; `fixtures::spec(target, parts)` beside `meta()` in
`src/tests/fixtures.rs`):
```rust
// crates/hearth-domain-render/src/tests/diff.rs
const REBUILDS: &[(&[Layer], Rebuild)] = &[
    (&[], Rebuild::Nothing),
    (&[Layer::Other], Rebuild::Nothing),
    (&[Layer::Slot, Layer::Other], Rebuild::SlotOnly),
    (&[Layer::Software, Layer::Slot], Rebuild::FromSoftware),
    (&[Layer::Config, Layer::Slot], Rebuild::FromConfig),
    (&[Layer::Config, Layer::Packages], Rebuild::FromPackages { resolution: Needs::Needed }),
];
#[test]
fn the_lowest_changed_layer_is_the_rebuild() {
    for (changed, expected) in REBUILDS {
        assert_eq!(diff::rebuild(changed, Needs::Needed), *expected, "render.md, 6.2: {changed:?}");
    }
}
```
Then `equal_spec_digests_are_same_whatever_the_commits`,
`a_target_on_one_side_alone_is_added_or_removed`,
`tree_entries_are_added_removed_or_changed_by_path`,
`the_sides_render_in_turn_from_then_to` (`to`'s first request follows `from`'s
last answer); translate:
`each_change_rebuild_and_side_translates_to_the_records_and_back`.
Real, each under 0.5 s; fixtures `Repository::head()`, `next(files,
removed)` (a commit as `make`'s), `diff(&Repository, &Asked) -> Rendered`
(`Asked::new(from, to)`: every target, text, `<scratch>/store`);
`MOTD`: `{}` for `fleet.json`, `envs/{prod,lab}.json`,
`hosts/<h>/<h>.json` (`<h>` `ion`, `x1`); `hosts/<h>/<h>-prod.json` THIN's
`placement` (hearth 11's `x1-prod.json`) and `"motd": {}`,
`hosts/<h>/<h>-lab.json` the `placement` alone; `tasks/motd/task.json` `{}`,
`tasks/motd/tree/etc/motd` `Welcome.\n` (`MOTD_KIND`: `Welcome, and be
kind.\n`):
`hearth_diff_names_each_changed_target_and_its_rebuild` (`from =
repository.head()`, `to = repository.next(MOTD_KIND, &[])`) asserts
stdout whole and `Exit::Code(1)` (client/README.md, 3.3), 6.3's lines
with hunks aside:
```text
ion-prod                                            rebuild from config
  config   ~ /etc/motd                              0644 root:root
x1-prod                                             rebuild from config
  config   ~ /etc/motd                              0644 root:root

2 of 4 targets change; 2 are unchanged.
```
Then `json_lines_give_each_targets_change_then_the_tally_in_the_outcome`,
`a_diff_of_a_commit_with_itself_exits_0`,
`a_named_target_neither_revision_has_exits_64`,
`a_document_fixed_in_the_later_revision_fails_its_target_from` (MOTD
with `hosts/x1/x1-lab.json` not JSON, then fixed: x1-lab's `change` line
`failed`, side `from`, exit 2), and
`a_failure_beats_every_difference_and_exits_2` (MOTD_KIND with
`hosts/ion/ion-lab.json` not JSON: the two `changed` lines, ion-lab
`failed` side `to`, exit 2, never 1); unit
`a_target_one_side_has_and_failed_there_is_failed_not_added`,
`a_target_failing_on_both_sides_is_failed_both_with_each_sides_errors`.
Proof: process, `heavy cargo nextest run -p hearth-builder-world -E
'test(=hearth_diff_names_each_changed_target_and_its_rebuild)'`:
two commits through real git, both sides in one call, 6.3's target and
file lines and the tally, exit 1. Bound 0.5 s.
Don't: render the two sides at once; read a cache file; format in
render; give a rebuild's time (6.2's "Time" needs builds: M2).
Size: at most 300 lines.
Removes: nothing.

### 25.2 A changed file's sizes and hunks
Scope: 6.1's sizes and hunks; 8.2's `Hunks` both ways; 2.2's `diff
--no-index` in `git.rs`; `translate/hunks.rs`; records' `NoNewline`.
Needs: hearth 02 (02.1's `Command::Diff`, its vector as render.md 2.2 has it;
25.2 edits no git file), hearth 34.2 (`hunks::{Diff, Hunks, Line,
Kind}`).
Files: render `src/{vocabulary, call, diff, limits}.rs`,
`src/parts/config.rs`, `src/tests/diff.rs`; protocol `src/{lib, git}.rs`,
new `src/translate/hunks.rs`, `src/translate/changes.rs`,
`src/tests/{git, translate}.rs`; records `src/{diff.rs, form/,
tests/diff.rs}`; `tests/builder/{src/fixtures.rs, tests/real.rs}`.
Shape: the sketches. `call::Sizes` from `Cached`; render's `SHIPPED`
gains `hunk_file: 1_048_576`. A `Changed` of two files gets `sizes`, and
`Request::Hunks` when the digests differ, both under `hunk_file`.
`git.rs`: `Spawn` of `invocation::GIT`, `arguments(Command::Diff {
store: b".", before, after })`, `environment()`, the store's `root`,
`dir` empty, stdout to 34.2's `hunks::Diff`; `Event::Hunks` with
`succeeded`'s exit answers `render::Event::Hunks`, else `Failed {
class: Operational }`: the file stays bare, the change stands, the target's
`tool` error (hearth 45.1), the outcome `TryAgain`, exit 75 (render.md 2.2).
Test shape: unit, each under 20 ms, at render's and git's
`limits::SHIPPED` (1 MiB; 200 lines):
`a_hunks_request_spawns_gits_diff_beneath_the_stores_root` (as 11.2's
reference: program, `root`, empty `dir`, the last two words
`./cache/<64 hex>`), `diff_exiting_0_or_past_1_fails_the_request_as_operational`
(protocol); `hunks_are_asked_only_for_two_files_under_1_mib_whose_digests_differ`
(render; a link or a directory gets none),
`a_failed_hunks_request_leaves_the_file_bare_and_the_call_try_again` (render:
the `Change` without hunks, outcome `TryAgain`, exit 75),
`a_line_without_a_final_newline_prints_as_text_and_as_no_newline_json`
(records); `each_hunk_line_kind_translates_and_back`. Real, each under
0.5 s: 25.1's reference, its literal grown to `fixtures::MOTD_DIFF_TEXT`,
each file line followed by git's `@@ -1 +1 @@`, `-Welcome.` and
`+Welcome, and be kind.` at 15 spaces (real git at `--unified=3`, as in
6.3's example). Also
`a_store_inside_a_work_tree_diffs_as_outside_any_repository`
(`fixtures::work_tree(at, config)`: `git init`, each entry set in it,
here `diff.interHunkContext=10`, the store beneath it via `Asked { store:
Some(&store), ..Asked::new(&from, &to) }`; `TWENTY_LINES`, `/etc/motd`
`line 1` to `line 20`, then `TWENTY_LINES_TWO_CHANGED`, lines 2 and 12:
two `@@` lines, git's `@@ -1,5 +1,5 @@` and `@@ -9,7 +9,7 @@ line 8`, exit 1:
render.md, 2.2) and `a_work_trees_gitattributes_do_not_make_a_file_binary`
(the same tree with `.gitattributes` `* -diff` committed: the hunks, never
`Binary files … differ`). Then
`json_counts_the_hunk_lines_left_out_past_200` (300 lines all changed:
`left_out` 401: 601 lines from the first `@@`, 200 kept),
`a_binary_file_answers_binary_with_no_hunk_line`.
Proof: process, `heavy cargo nextest run -p hearth-builder-world -E
'test(=hearth_diff_names_each_changed_target_and_its_rebuild)'` (the
literal grown): real git's hunks of two cache files, decoded by 34.2's
machine, printed under their file as 6.3 shows. Bound 0.5 s.
Don't: compute a text diff in step code (6.1); ask hunks of equal
digests, a link or a directory; read a cache file in the protocol layer.
Size: at most 220 lines.
Removes: 25.1's changed files without sizes or hunks.

## Interface sketches

Types derive as hearth 11's do. render owns `diff.rs`; records' types
are hearth 10's and 40's, extended here; a part session after 25.1 adds a
`PartChange` variant, its `layer` arm and its translation; hearth 45 adds
meta's and the resolution's.

```rust
// crates/hearth-domain-render/src/vocabulary.rs (hearth 11's; 25.1, 25.2 add; `resolve` hearth 45.2)
pub enum Event {
    /// Diff `targets` (none: either side's every one): `Change`s, errors, `Rendered` (`resolve`: 45.2).
    Diff { repository: Box<[u8]>, from: Box<[u8]>, to: Box<[u8]>, targets: Box<[Box<[u8]>]> },
    /// `Hunks`' terminal: git's lines for the pair asked (25.2).
    Hunks { request: Token, hunks: Hunks },
}
pub enum Request {
    /// Two cache files' hunks, by SHA-256 (render.md, 8.2): `Hunks` or `Failed` (25.2).
    Hunks { request: Token, before: Digest, after: Digest },
}
pub enum Answer {
    /// A target's change, in name order (25.1).
    Change(Change),
    /// The call's last answer: its worst outcome, and a diff's tally (25.1 adds `diff`).
    Rendered { outcome: Outcome, diff: Option<Tally> },
}
/// What a call reached, the worst first; `Negative`: a diff found differences (25.1).
pub enum Outcome { Usage, Failed, TryAgain, Negative, Done }

// crates/hearth-domain-render/src/diff.rs (25.1; Hunks 25.2)
/// A target's change (render.md, 6.1); mirrors records' `diff::Change`.
pub struct Change { pub target: Box<[u8]>, pub change: TargetChange }
/// Mirrors records' `TargetChange`.
pub enum TargetChange {
    /// In `to` alone.
    Added { spec_digest: Digest },
    /// In `from` alone.
    Removed,
    /// Equal spec digests.
    Same,
    /// The parts whose digests differ, in render.md 4.1's order (meta first: 45.1).
    Changed { parts: Box<[PartChange]>, rebuild: Rebuild },
    /// A side that has it failed there (`Both`: each has it, neither rendered); its errors are the call's.
    Failed { side: Side },
}
/// Mirrors records' `Side`.
pub enum Side { From, To, Both }
/// What a side made of a target (boxed: clippy's `large_enum_variant`, once 24.1 adds `Spec::resolution`).
pub enum Made { Spec(Box<Spec>), Failed(Class) }
/// Mirrors records' `PartChange`.
pub enum PartChange { Config(parts::config::Change) } // hearth 45.1 adds `Meta(parts::meta::Change)`, leading
/// render.md 6.1's `rebuild`; mirrors records' `Rebuild`.
pub enum Rebuild { FromPackages { resolution: Needs }, FromSoftware, FromConfig, SlotOnly, Nothing }
/// Whether `to`'s resolution is recorded; mirrors records' `Needs`.
pub enum Needs { Known, Needed }
/// What a part enters (render.md, 4.1), lowest first.
pub enum Layer { Packages, Software, Config, Slot, Other }
/// 6.1's `from` and `to`, and 6.3's counts; mirrors records' `Tally`.
pub struct Tally { pub from: Oid, pub to: Oid, pub targets: u32, pub changed: u32, pub unchanged: u32 }
/// Mirrors hearth-protocol-git's `hunks::Hunks` and records' `Hunks`.
pub struct Hunks { pub lines: Box<[Line]>, pub left_out: u32, pub binary: bool }
/// Mirrors git's `hunks::Line` and records' `HunkLine`.
pub struct Line { pub kind: LineKind, pub text: Box<[u8]> }
/// Mirrors git's `hunks::Kind`; records' `LineKind` calls `Hunk` `Header`.
pub enum LineKind { Hunk, Context, Removed, Added, NoNewline }
#[must_use] pub fn layer(part: &PartRecord) -> Layer;
/// The lowest of `changed` (6.2); `Nothing` for none or `Other` alone.
#[must_use] pub fn rebuild(changed: &[Layer], resolution: Needs) -> Rebuild;
/// What each side made of a target; at least one is `Some`.
#[must_use] pub fn compare(from: Option<&Made>, to: Option<&Made>, resolution: Needs) -> TargetChange;

// crates/hearth-domain-render/src/parts/config.rs (25.1; sizes, hunks 25.2)
/// config's change, by path; mirrors records' `ConfigChange`.
pub struct Change { pub tree: Box<[EntryChange]> }
/// Mirrors records' `EntryChange`.
pub enum EntryChange {
    Added { path: Box<[u8]>, node: Node },
    Removed { path: Box<[u8]>, node: Node },
    /// `sizes` when both are files; `hunks` when asked and answered (6.1).
    Changed { path: Box<[u8]>, before: Node, after: Node, sizes: Option<Sizes>, hunks: Option<diff::Hunks> },
}
/// Mirrors records' `Sizes`.
pub struct Sizes { pub before: u64, pub after: u64 }
/// A side without the part is an empty tree.
#[must_use] pub fn change(before: Option<&Config>, after: Option<&Config>) -> Change;

// crates/hearth-domain-builder/src/vocabulary.rs (hearth 11's; 25.1 adds)
pub enum Call {
    /// `hearth diff`: `repository` an absolute git directory (`resolve: bool`: 45.2).
    Diff { repository: Box<[u8]>, from: Box<[u8]>, to: Box<[u8]>, targets: Box<[Box<[u8]>]> },
}
pub enum Answer {
    /// A target's change, in name order.
    Change(render::diff::Change),
}
pub enum Request {
    /// The call's end: its outcome, and a diff's tally (`diff` added).
    Reply { to: ReplyTo, outcome: render::Outcome, diff: Option<render::diff::Tally> },
}

// crates/hearth-protocol-builder/src/translate/: one function a type, both ways in src/tests/translate.rs
pub fn change(change: render::diff::Change) -> records::Change;   // changes.rs (25.1)
pub fn tally(tally: render::diff::Tally) -> records::Tally;       // changes.rs (25.1)
pub fn hunks(hunks: git::hunks::Hunks) -> render::diff::Hunks;    // hunks.rs (25.2)

// crates/hearth-client-protocol-records/src/diff.rs (hearth 40.1's; 25.2 adds)
pub enum LineKind { Header, Context, Removed, Added, NoNewline }  // NoNewline: 25.2, JSON `no_newline`
pub enum PartChange { Config(ConfigChange) }                      // Meta: 45.1
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-25\.[1-2]'` prints 2.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest
   run -p hearth-domain-render -p hearth-protocol-builder -p
   hearth-client-protocol-records -p hearth-builder-world -p hearth` passes.
3. `git -C ~/src/rust/hearth ls-tree --name-only origin/main
   crates/hearth-protocol-builder/src/translate/` lists `changes.rs` and
   `hunks.rs`.
