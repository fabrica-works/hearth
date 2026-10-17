# Session 45: the diff's meta, errors, `hearth diff` and resolution

render.md, section 6, after hearth 25's typed diff, rebuild and hunks:
45.1: meta's change, both sides' errors with their side, a failed
`Hunks`' `tool` error, and `hearth diff` in the binary; 45.2, after
hearth 24 and 31.1: the packages change's resolution and `--resolve` on
both sides.

## Read first

- hearth's `render.md` (at `175e92f`): 2.3, 4.5, 5.1, 5.4, 5.7, 6.1, 7.1,
  8.2; `client/README.md`: 3.1 to 3.3, 3.5; `store.md`: 6.4;
  `testing.md`: 2.7.
- `decisions.md`, 2 and 6. The drafts: `hearth/25-diff.md` (whole: the
  frame, `Made`, `PartChange`, `Hunks`), `40-client-diff-errors.md` (40.1,
  40.2), hearth 12's (render's errors), 42.1's (the `tool` kind) and 22.1's
  (the collector), `32-static-binary.md` (32.2's usage line), `44-fake-git.md`
  (44.2's missing side); hearth 24's (`Resolve`, `Resolved`, the spec's
  resolution, `--resolve`) for 45.2.
- skein's `programming-model.md`: 2, 3, 4, 5.2, 6.3, 7;
  `testing-strategy.md`: 2.8, 2.9 (by number only).

## What the design asks

- **The typed diff** (6.1): meta's "base, member, environment, placement:
  each before, after, when it changed"; packages' "resolution: same | new
  key | new key, resolved: added, removed, versions changed"; `failed {
  side: from | to | both, errors }`.
- **The command** (client/README.md, 3.1, 3.2, 3.5): `hearth diff <from>
  <to> [--target <t>]… [--repo] [--store] [--json] [--resolve]`; errors
  "after what did render"; `--resolve`: "the keys of the targets whose
  resolution key changed, on both sides" (render.md, 5.7).
- **Errors** (2.3, 7.1): "errors kept from one render: 256, then a
  count"; a diff renders twice.

## Choices this plan makes

- **A diff's errors carry their `side`** (`from` or `to`), printed after the
  changes (render.md 7.1); bounded once over the answer, the two sides
  together: errors past 256 are counted (`<n> more errors`, `left_out`)
  and still decide the exit code (7.1, 7.3). A target failing on both
  sides is `failed (both)` with each side's errors.
- **Meta's change** is any of base, member, environment, placement moved,
  each as its before and after, a field that did not move absent; it
  rebuilds nothing (layer `Other`, meta enters no key; 4.4), and no
  changed target has an empty list of part changes (render.md 6.1).
- **`--resolve` in a diff** (5.7): a resolution that fails is that
  target's error after the changes, the change standing with `new key`;
  an added or removed target resolves nothing (render.md 5.7, 6.1).
- **Transitional:** hearth 25.2's failed hunks without an error (ends in
  45.1); hearth 25's `Needed` always (ends in 45.2).

## Rules that bind it

- **The meeting point** (README, Rules): each increment proves on the
  real loop or the binary, and extends `translate/changes.rs` (both ways
  in `src/tests/translate.rs`) with each variant it adds to a vocabulary
  crossing `hearth-protocol-builder`.
- **Shared files** (README, Rules): render's `src/{lib, vocabulary,
  call}.rs`, `src/parts/`; `translate/`; the records' `records.rs`,
  `diff.rs`, `form/` (version 1, hearth 10's and 40's: added to, never
  renamed); `tests/builder/src/fixtures.rs`. Landing second, rebase and
  keep both.
- **No expected value computed by the code under test** (code-org., 6):
  printed lines and hunks are git's literals, run once, pasted (git 2.47.3 and
  2.53.0 print the same).

## Increments

### 45.1 Meta's change, both sides' errors, and `hearth diff`
Scope: 6.1's meta change; both sides' errors, each with its side, after
the changes (3.2); a failed `Hunks`' `tool` error; `hearth diff <from>
<to> [--target <t>]… [--repo] [--store] [--json]` (3.1, 3.5);
`end_to_end_diff` (testing.md, 2.7).
Needs: hearth 25 (25.1, 25.2); hearth 22.1 (the collector: errors sorted,
at most 256, then a count; `Answer::Error`); hearth 42.1 (the `tool` kind);
hearth 32.2 (the binary's usage line).
Files: render `src/{call, diff, errors}.rs`, `src/parts/meta.rs`,
`src/tests/diff.rs`; protocol `src/git.rs`, `src/translate/{changes, errors}.rs`,
`src/tests/{git, translate}.rs`; records `src/{diff.rs, errors.rs, form/,
tests/diff.rs}`;
`crates/hearth/{src/argv.rs, src/tests.rs, tests/end_to_end.rs}`;
`tests/builder/tests/real.rs`.
Shape: the sketches. `PartChange::Meta` (hearth 25's) leads the parts
when any meta field differs (Choices). Both sides' errors go to
hearth 22.1's one collector: 7.1's order, one 256 bound and count over
the answer (Choices), each with its `side` (hearth 12's error and
records' `Error` gain `side: Option<Side>`, `None` in a render, omitted in
JSON then; text leads 7.3's line with `from: ` or `to: `), after
the last change; errors equal in every other key sort `from` before `to`. A
failed `Hunks` leaves its file bare and adds the target's `tool` error: git.rs
gives 25.2's `Failed` its `Tool` (`Command::Diff`, the exit, the tail:
hearth 42.1's).
`argv`: `diff`, two revisions checked by `invocation::revision` before
the loop (64), `--target` repeated, `--repo`, `--store`, `--json` as
`render` takes them; anything else (`--running`: M3) a usage line.
Test shape: unit (render): `each_error_of_a_diff_names_its_side`,
`the_diffs_errors_are_bounded_once_over_both_sides`,
`a_failed_hunks_request_leaves_its_file_bare_then_a_tool_error` (the
`Change`, the error, `TryAgain`),
`a_meta_change_holds_each_moved_field_before_and_after_and_no_other`,
`a_member_changed_alone_is_a_meta_change_rebuilding_nothing`,
`a_target_failing_on_both_sides_has_each_sides_errors_with_its_side`;
records `a_meta_change_prints_each_moved_field_before_and_after` (text
`  meta     ~ <field>  <before> → <after>` a field); argv
`diffs_revisions_and_each_target_become_the_call`,
`a_malformed_from_or_to_is_usage_before_the_loop`. Real:
`the_diffs_errors_follow_its_changes_and_precede_the_outcome` (JSON,
`side` `to`). The real loop cannot make git's diff fail (the cache files
are the call's own; 32.1 replaces one it cannot open), so a failed
`Hunks` proves in hearth 28's protocol world, fake git failing the diff
(44.2: a missing side, exit 1, no hunk), which Needs hearth 45.1:
`a_diff_whose_git_fails_prints_the_change_bare_then_its_tool_error_and_exits_75`.
End to end, under 0.5 s: `end_to_end_diff` (`MOTD` at two commits, the
binary run with `XDG_CACHE_HOME` a scratch directory and `HOME`
`repository.home()`, `diff <from> <to>`) asserts stdout is
`fixtures::MOTD_DIFF_TEXT`, the real loop's lines, and `Exit::Code(1)`.
Proof: process, `heavy cargo nextest run -p hearth -E
'test(=end_to_end_diff)'`: the shipped binary finds its repository and store,
diffs two commits, exits 1. Bound 0.5 s.
Don't: sort, bound or count errors anew (hearth 22.1's collector does);
try a socket; take `--resolve` (45.2).
Size: at most 220 lines.
Removes: 32.2's usage line for `diff`; hearth 25.2's errorless failed hunks.


### 45.2 The resolution's change, and `--resolve` on both sides
Scope: 6.1's packages `resolution`; the rebuild's `known | needed`;
`hearth diff --resolve` (5.7).
Needs: hearth 24 (the spec's resolution key, recorded or not; a recorded
key's closure through the root; `Resolve`, `Resolved`; `--resolve`);
hearth 31.1 (`Record`: each resolved key recorded).
Files: render `src/{vocabulary, call, diff}.rs`, `src/parts/packages.rs`,
`src/tests/diff.rs`; the root's `src/vocabulary.rs`; protocol
`src/translate/changes.rs`, `src/tests/translate.rs`; records
`src/parts/packages.rs` (hearth 17's change), `src/form/`; `crates/hearth/src/{argv, tests}.rs`;
`tests/builder/{src/fixtures.rs, tests/real.rs}`.
Shape: the sketches. `diff::resolution`: equal keys, `Same`; both
closures recorded, `Resolved`, one walk of the two by name, then
architecture (store.md, 6.4): in `to` alone `added`, in `from` alone
`removed`, at two versions `versions_changed`; else `NewKey`. The
rebuild's resolution is `Known` when `to`'s key is recorded. `Call::Diff`
and `Event::Diff` gain `resolve`: once both sides render, for each target
present on both sides whose resolution keys differ, render asks `Resolve` for
each unrecorded key of the two, then compares.
Test shape: unit: `equal_resolution_keys_are_the_same_resolution`,
`two_recorded_closures_list_packages_added_removed_and_moved`,
`a_closure_unrecorded_on_either_side_is_a_new_key`,
`the_rebuild_from_packages_is_known_once_the_later_key_is_recorded`,
`a_failed_resolve_keeps_new_key_and_its_error_follows_the_changes`; argv `diff_takes_resolve_as_render_does`. Real, "its resolutions already
in the store by key so no mmdebstrap runs" (testing.md, 2.7):
`two_recorded_resolutions_show_the_packages_a_change_moves` (hearth 24's
fixture repository at two requests, both records in the store first, keys
literals; JSON; the later closure adds one package, drops one and moves one
to another version).
Proof: process, `heavy cargo nextest run -p hearth-builder-world -E
'test(=two_recorded_resolutions_show_the_packages_a_change_moves)'`: the
packages change lists what the later closure adds, removes and moves, rebuild
`from packages`, known; bound 0.5 s. That `--resolve` resolves only the changed keys, of both
sides, one at a time, each recorded before the diff prints, is proved in
hearth 28's simulated world with fake mmdebstrap
(`diff_resolve_resolves_each_changed_key_once_in_turn_before_printing`;
hearth 28 Needs hearth 45.2 for it).
Don't: resolve a key whose target's key did not change, or for an added
or removed target; run mmdebstrap in the gate.
Size: at most 250 lines.
Removes: the rebuild's fixed `Needed`; `--resolve` refused for `diff`.


## Interface sketches

Types derive as hearth 11's do; hearth 25's sketches (`diff.rs`, the
vocabularies, `translate/`) are extended here.

```rust
// crates/hearth-domain-render/src/diff.rs (45.1 adds `Meta`; 45.2 the rest)
/// hearth 25's `PartChange`, meta first (45.1); mirrors records' `PartChange`.
pub enum PartChange { Meta(parts::meta::Change), Config(parts::config::Change) }
/// render.md 6.1's packages `resolution`; hearth 17's packages change holds it.
pub enum ResolutionChange {
    /// Equal keys.
    Same,
    /// The keys differ, and a side's closure is not recorded.
    NewKey,
    /// Both recorded: by name, then architecture (store.md, 6.4).
    Resolved { added: Box<[Listed]>, removed: Box<[Listed]>, versions_changed: Box<[Moved]> },
}
/// A closure's package as the change lists it (not 24's `resolution::Package`).
pub struct Listed { pub name: Box<[u8]>, pub architecture: Box<[u8]>, pub version: Box<[u8]> }
/// A package in both closures, at two versions.
pub struct Moved { pub name: Box<[u8]>, pub architecture: Box<[u8]>, pub before: Box<[u8]>, pub after: Box<[u8]> }

// crates/hearth-domain-render/src/parts/meta.rs (45.1)
/// meta's change (6.1): each field that moved, `None` for one that did not; mirrors records' `MetaChange`.
pub struct Change {
    pub base: Option<Pair<Box<[u8]>>>, pub member: Option<Pair<Option<Box<[u8]>>>>,
    pub environment: Option<Pair<Box<[u8]>>>, pub placement: Option<Pair<Placement>>,
}
/// A field before and after; mirrors records' `Pair`.
pub struct Pair<T> { pub before: T, pub after: T }

// the vocabularies (45.2): `Event::Diff` and the root's `Call::Diff` gain `resolve: bool`

// crates/hearth-client-protocol-records/src/diff.rs (hearth 40.1's, 25's; 45.1 adds)
pub enum PartChange { Meta(MetaChange), Config(ConfigChange) }    // Meta: 45.1
pub struct MetaChange { pub base: Option<Pair<Box<[u8]>>>, pub member: Option<Pair<Option<Box<[u8]>>>>, pub environment: Option<Pair<Box<[u8]>>>, pub placement: Option<Pair<Placement>> }
pub struct Pair<T> { pub before: T, pub after: T }
// src/errors.rs (hearth 40.2's; 45.1): `Error` gains `pub side: Option<Side>`, as hearth 12's error does
// render's errors.rs and records' (45.1): `Command`'s `Diff` is hearth 42.1's (ruling 147)
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-45\.[1-2]'` prints 2.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest
   run -p hearth-domain-render -p hearth-protocol-builder -p
   hearth-client-protocol-records -p hearth-builder-world -p hearth` passes, and `heavy cargo nextest run
   -p hearth -E 'test(/^end_to_end_/)'` prints a `Summary` naming at
   least 2 tests, `end_to_end_diff` among them.
