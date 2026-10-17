# Session 24: resolution: keys, closures, `--resolve`
render.md, section 5. 24.1: resolution keys and the store's record (`needed`, or
`known` with layer keys: 3.8, 4.5, 5.7). 24.2: `--resolve` runs mmdebstrap.
24.3: apt's lists are read, the closure matched (5.3). Hearth 31 records the
closure (store.md 6.4) and keeps the indices (6.8). The gate runs no mmdebstrap
(testing.md 2.7); `--resolve` proves live.

## Read first
- `render.md` 3.8, 4.1, 4.2, 4.5, 5, 7.1, 7.2, 8.1, 8.2, 10; `store.md` 2.1,
  2.2, 2.4, 3.1 to 3.3, 6.1, 6.4, 8.1
- `client/README.md` 3.1 to 3.3, 3.5; `testing.md` 2.7, 6.2; `README.md` 4.2;
  `decisions.md` 11, 14, 19; `findings/skein-io.md` 2 to 4
- Drafts: 07 whole; 38.1; 08 (`Pin`, `Mirror`, `Proxy`); 41.1; 09.3; 10.3; 11; 12
  (`errors`) and 42.1 (`Failure`, `Tool`); 17; 33 ("What hearth 24 calls"); 45.2
- skein, at hearth's pin 841f487: `crates/skein-lib/src/intake.rs:132`
  (`Intake::meet`), `crates/skein-lib/src/stream.rs:34` (`Read::Scan`),
  `04-store-io.md`; `programming-model.md` 2, 4.5, 5.2, 6.3, 9

## What the design asks
- Keys (5.1, 4.5): `H("resolution", sources, request, variant, apt)`.
- Resolving (5.7, 8.2): `--resolve` resolves "each needed key, one at a time".
- The run (5.2, 5.5): 60 s idle, 10 minutes, scratch removed after.
- The closure (5.3): stanzas agree; indices hashed as read, renamed.
- Bounds (store.md 6.1): 8,192 packages "never a record".

## Choices this plan makes
- **Encoding (hearth 17's):** key, `inputs` and part digest take each value as
  the part's record encodes it, as `hearth_lib` items via `Encoder::item` under
  `H::new(b"resolution", 4)`, as 33.2's scan.
- **Of hearth 17** (its drafter reads this): `parts/packages.rs`,
  `PartRecord::Packages(Packages)`: `sources: Sources` (`suite`, `architecture`,
  `components`, `archive`, `updates`, `security: Option`), the request as names
  in increasing byte order, `variant: Variant` (`Apt`), `apt: Apt {
  install_recommends }`; `pub fn inputs(&Packages) -> Inputs`, the items its
  part digest encodes; names and timestamps out of form refused before any key
  (format.md 5.3), so 07's `Name::new`, `Timestamp::new` take every `Resolve`; the
  packages line's `resolution <12 hex> needed|known` from
  `records::Spec::resolution`.
- **Failures** (nine rows) by 07's `Cause` (render.md 5.5): `Unknown`,
  `Unsatisfiable` → `resolution`, configuration; `Absent` (a 404 or no Release
  file) → `resolution`, configuration, naming the source line, whose `uri` and
  `suite` the protocol layer takes from the first line the tail's 404 names,
  else the first; `Fetch`, `Resolve`, `Space`, a refused spawn, scratch or
  `sources.list` (`Ended::Start`), a timeout → `tool` (`Command::Mmdebstrap`),
  operational; `Other` → `tool`, configuration. One error per target holding
  the key, its spec `needed`, `at: None`, after the placed errors by target
  (render.md 5.7, 7.1).
- **The store's error** is one `store` error a call; a declined `Find` leaves
  the key `needed` (store.md 8.1). A failed key stops no other, and an error
  with no place sorts after the placed ones, by target (render.md 5.7, 7.1).
- **Live** (testing.md 6.2; render.md 5.2): the mirror digest is the pinned
  `InRelease`'s SHA-256; `test-min` is the `apt` variant with no package; apt
  checks with the host's keys, no `--keyring`; a pin bump re-pastes
  `TEST_MIN_KEY` and the keys line.
- **An index's entry** is its pass's size and SHA-256 (render.md 5.4), path
  `InRelease` or `<component>/binary-<arch>/Packages`.
- **Proxy:** `Configuration::proxy` via `Invocation::proxy`, `None` from
  `argv::invocation`; live tests pass 08.3's `Proxy::url()`;
  `conformance_render.rs` runs mmdebstrap twice here, once on a dead port.

## Rules that bind it
- **Meeting point** (README): each increment proves on the real loop or, for
  what runs mmdebstrap, live; a variant added to a vocabulary crossing the
  protocol layer extends `translate/` and its reverse in a module under
  `src/tests/translate/` (`errors.rs`, `repository.rs`, or a new one).
- **No expected value computed by the code under test:** literals, by hand
  (`python3 -I`, once), from `sha256sum`, or from what hearth 41 recorded.
- **Shared files** (README): root `Cargo.toml`, `Cargo.lock`, and the files
  other sessions share named in Files below: landing second, rebase and keep
  both.

## Increments
### 24.1 The keys, and the store's record found: `needed` or `known`
Scope: `keys` (5.1, 4.5); `Find` through the root; `known`.
Needs: hearth 08 (`testing/repos/test/`, pin), hearth 17.2 (`inputs`), hearth 33.2
(`Load`; hearth 23's `fixtures::lay`).
Files: render new `src/keys.rs`, `src/{lib, vocabulary, call, parts,
limits}.rs`, `src/tests/{fixtures, keys}.rs` (new `keys.rs`); root `src/{lib,
vocabulary}.rs`, new `src/resolution.rs`, `src/tests.rs`,
`src/tests/fixtures.rs` (its `Spec` literal, `spec()`, :12, gains `resolution: None`);
store `src/{lib, vocabulary, resolutions, scan}.rs`, `src/{start, enter}.rs`
(their `Event` matches, :51 and :56, gain `Find`), `src/tests/{fixtures,
scan}.rs`, new `src/tests/find.rs`; protocol `src/store.rs`, `src/lib.rs` (the
`Request` match at :267 gains `Find`), `src/translate/{store, answers}.rs`,
`src/tests/store.rs`, a module under `src/tests/translate/`, and the `Spec`
literals of `src/tests/fixtures.rs` (`spec()`, :251) and `src/tests/translate.rs`
(:153, :177); render `src/call/templates.rs` (its `Answer::Spec(parts::Spec {`
literal); `tests/render/src/parent.rs` (its matches at :53, :67, :87 and :95
gain `Find`, `Found` and `Absent`); the text goldens that gain 24.1's
`resolution … needed` line, and their users: `tests/builder/src/fixtures.rs`'s
`THIN_TEXT` (:163) and `BROKEN_LAB_TEXT` (:439), `tests/builder/tests/real.rs:734`;
`tests/builder/{Cargo.toml (dev: hearth-test-mirror), src/fixtures.rs}`, new
`tests/builder/tests/resolution.rs`. `tests/builder/src/fixtures.rs` gains
`Laid::Resolution(..)` (23-store.md named it; hearth 23 landed `Laid` with
`File` and `Record` alone, :441), `test_min_recorded()`, `TEST_MIN_KEY`,
`TEST_MIN_KEYS_LINE`, and `TEST_REPOSITORY: &[File]`, `testing/repos/test`'s
eight files by `include_bytes!`, for `Repository::make` (:45, which takes
`&[File]`). Any further exhaustive match the compiler names joins the
increment: list it in the commit body.
Shape: the sketches. After the last target, `Find` per distinct key, one at a
time: targets in name order, each key at the first target that holds it;
`Failed { Store }` → `Needed`, `TryAgain` (exit 75, a declined `Find` too);
every part is in the spec (render.md 4.1), so each key takes its digest. Store: `Find` with
no index is `Declined`, an unindexed or unreadable key `Absent`, else `Load`,
whose `Resolution { record }` (33.2's, now whole) answers `Found`; 33.2's scan
takes `record.key`, `record.inputs`.
Test shape: unit, each under 20 ms, at `limits::SHIPPED`:
`the_resolution_key_is_h_over_its_four_inputs_as_items` (33.2's `01ea811a…`),
`the_layer_keys_chain_packages_software_and_config` (inputs `Digest([n; 32])`
for n 0 to 3, the resolution's and the packages, software and config parts':
packages `22d5156ffefadb1f0df4eb2c4d405b2c8937215e044486b06b1725e3506b238d`,
store.md 3.2; software
`f96c81da124d998b00e4cea2ec54f3e46a9dec976cbcf12680feb2925e3af4c0`; config
`a56d06bfeaffd29b71c3a735c3c1057b43fff1f4f1fddf12c90080a3dda45b2e`),
`an_empty_output_part_enters_as_its_record_with_empty_fields` (software's,
`H("part:software", map { downloads: map {}, scripts: list [] })`:
`656d135b92c33782dc40e4dc874400fa4d4b666e9e466b3644075a81279f74a6`),
`each_distinct_key_is_found_once_at_its_first_holder_in_name_order`,
`a_declined_find_leaves_the_key_needed`; store:
`a_find_of_an_unindexed_key_is_absent_without_a_load`,
`a_find_of_an_indexed_key_loads_the_record_whole`,
`a_find_without_an_index_is_declined`; root:
`a_find_reaches_the_store_and_its_answer_render`; protocol:
`a_loaded_resolution_translates_whole_to_the_stores`. Real loop, under 0.5 s
each, on `fixtures::test_min_recorded()` (a closure of `apt` alone, digest
`33b37c0f785ad18398bc02780970e8dcaa1f0b304a3d1225e9b76fc1c4bcd6b6`),
`TEST_MIN_KEY`, `TEST_MIN_KEYS_LINE`. Reference, `tests/resolution.rs`:
```rust
#[test]
fn a_recorded_resolution_prints_known_and_the_layer_keys() {
    let r = Repository::make("known", fixtures::TEST_REPOSITORY);
    fixtures::lay(&r.scratch().join("store"), &[Laid::Resolution(fixtures::test_min_recorded())]);
    let out = fixtures::render(&r, &["test-min"], Form::Text);
    let printed = fixtures::lines_starting(&out.stdout, &["packages ", "keys "]);
    let known = ["resolution ", &fixtures::TEST_MIN_KEY[..12], " known\n", fixtures::TEST_MIN_KEYS_LINE, "\n"].concat();
    assert!(printed.ends_with(&known), "9.2, 4.5: {printed}");
    assert_eq!(out.exit, Exit::Code(0));
}
```
Then `an_unrecorded_resolution_prints_needed_and_no_keys_line` (exit 0 too),
`a_needed_and_a_known_render_give_the_same_spec_and_packages_digests` (`--json`
twice, the record laid between),
`the_json_spec_gives_the_resolution_and_its_keys_whole`,
`the_test_repositorys_targets_render_the_pins_resolution_inputs` (for 08.2: the
JSON spec's `sources` equal 08.1's pin, `forky`, `amd64`, `main`,
`20261016T091523Z`).
Proof: process, `heavy cargo nextest run -p hearth-builder-world -E
'test(=a_recorded_resolution_prints_known_and_the_layer_keys)'`: a laid record
prints `known` with its layer keys. Bound: 0.5 s.
Don't: run mmdebstrap; recompute a record's `digest` (33.2's scan did, store.md
6.4).
Size: at most 290 lines.
Removes: 33.2's `translate::store::{inputs, closed}` and `Event::Resolution {
key, inputs, closure, digest }` (now whole).
### 24.2 `--resolve`: the run, its timers and its failures
Scope: `--resolve` (argv, `Call`, `Event::Render`); render's `Resolve`; the
protocol layer's run (`src/resolve.rs`); the `tool` kind;
`Configuration::proxy`.
Needs: hearth 07 (`vector`, `output`, `tail`); hearth 32.2 (the binary's usage
line); hearth 34.1 (the `conformance` profile); hearth 41.1 (the recordings,
`simulate`); hearth 42.1 (`Failure`, `Tool`).
Files: render `src/{vocabulary, call, errors, limits, lib}.rs`, new
`src/resolution.rs`, `src/tests/{fixtures, resolution}.rs` (new
`resolution.rs`); root `src/{lib, vocabulary}.rs`, `src/tests.rs`; protocol
`Cargo.toml` (`hearth-protocol-mmdebstrap`), `src/{lib, limits, walk}.rs`, new
`src/resolve.rs`, new `src/translate/resolution.rs`, `src/translate/errors.rs`,
`src/tests/{fixtures, translate}.rs`, new `src/tests/resolve.rs`; records
`src/errors.rs`, `src/form/{text, json}.rs`, `src/tests/errors.rs`; service
`src/{lib, limits}.rs` (`fire`, `next_deadline`; `proxy`);
`crates/hearth/src/{lib, argv, tests}.rs`, `crates/hearth/tests/end_to_end.rs`;
`tests/builder/{Cargo.toml, src/fixtures.rs}`, new
`tests/builder/tests/conformance_render.rs`; the `resolve: false` the `Call`
and `Event::Render` literals gain: domain-builder `src/tests.rs:133`,
`src/tests/fixtures.rs:9`, service-builder `src/tests/fixtures.rs:9`,
protocol `src/tests/fixtures.rs:25`, `tests/render/src/parent.rs:36`, whose
matches (:53, :67, :87, :95) gain `Resolve` and each `Answer` 24.2 adds; mmdebstrap `src/limits.rs` (`Limits` derives
`Hash`, as the protocol's `Limits` that holds it does); `Cargo.lock`. Records:
`Command::Mmdebstrap` and `Ended::Timeout` go in `src/errors/repository.rs`
(`Command` :23, `Ended` :30, their arms :96 to :114 in text and :123 to :139
in JSON), in place of
`src/form/{text, json}.rs`, which hold no error kind. Reverse tests: protocol
`src/tests/translate/{errors, repository}.rs`: `Limit::Closure` beside :55,
since 24.2's `TooLarge { limit: Closure }` uses it (24.2 adds `Limit::Closure`
and its arms), and `Command::Mmdebstrap` and `Ended::Timeout` beside :38 and
:48.
Shape: the sketches. With `--resolve`, render sets `deadline = now +
Limits::resolve` and resolves each absent key (targets in name order, each key
at its first holder), one at a time, a failure not stopping the rest, each holder given the error (render.md 5.7);
a `Container` target (variant `container`, 4.2) has its key asked of nobody,
left `needed`, no error (5.7). The run:
`MakeDirectory { staging.token, <hex of the key>, 0o700 }`, `Store {
staging.token, <hex>/sources.list, Expect::Absent }`, `Spawn { PROGRAM, args,
env, root: store.fd, dir: b"", pipes: [1 Out, 2 Out] }`, with `deadline` and
07's `idle` armed; a refused one of the three → `Run { ended: Start, cause: None
}`; a reader's `Inst`/`Stream` error → `Run { ended: Output, cause: None }`, its
`TooMany` → `TooLarge { limit: Closure, max }`; a timer → `Run { ended: Timeout
}` (render never retries, 5.5); exit 0 → (transitional) `Resolved { lines: [] }`, else `tail::failure`
→ `Failed { class: cause.class(), failure: Run { tool, cause } }`; the scratch
is walked away at each ending (33.2's walk). render: `Failed` → an error per
target holding the key; records print `mmdebstrap exited 1`, `… timed out`, `…
could not start`, the tail.
Test shape: unit under 20 ms; `fixtures::resolve(key, names)` is a render
`Resolve` (forky, amd64, `main`, `20261001T030405Z`, no pocket, `Apt`, no
recommends), `KEY` `01…01`. Reference, `src/tests/resolve.rs`:
```rust
#[test]
fn a_resolve_makes_its_scratch_writes_sources_list_then_spawns_mmdebstrap() {
    let mut layer = fixtures::Layer::new(b"/scratch/home");
    layer.down(fixtures::resolve(KEY, &[]));
    let asked = layer.asked();
    let [Asked::File(file::Request::MakeDirectory { mode, .. })] = &*asked else { unreachable!("one request") };
    assert_eq!(*mode, 0o700, "5.2");
    layer.up(fixtures::made());
    let asked = layer.asked();
    let [Asked::File(file::Request::Store { expected, .. })] = &*asked else { unreachable!("one request") };
    assert_eq!(*expected, Expect::Absent, "never over another run's");
    layer.up(fixtures::stored());
    let asked = layer.asked();
    let [Asked::Io(io::Request::Spawn { .. })] = &*asked else { unreachable!("one request") }; // pipes [1 Out, 2 Out], dir `b""`
}
```
Then protocol: `inst_lines_from_either_pipe_reach_render_as_they_come`,
`a_silent_run_is_closed_at_its_idle_timer_and_fails_timed_out` (60 s),
`renders_deadline_ends_a_run_that_still_speaks`,
`a_failed_run_answers_its_cause_and_stderrs_tail`,
`an_absent_run_names_the_source_line_its_404_names` (three lines; the `Err:`
names the second's `uri`; one that names none takes the first),
`a_refused_scratch_sources_list_or_spawn_fails_the_run_as_could_not_start` (a
`const` table), `an_inst_flood_fails_the_run_too_large_at_the_closure_limit`
(`insts: 2`), `the_scratch_is_walked_away_before_the_terminal`,
`a_resolve_of_test_min_translates_to_41_1s_simulate` (for 08.2); render:
`needed_keys_are_resolved_one_at_a_time_in_target_order`,
`a_failed_key_leaves_the_later_keys_resolved` (A fails, B `known`),
`without_resolve_no_key_is_resolved`,
`a_failed_run_is_an_error_of_each_target_holding_its_key` (`at: None`),
`a_run_too_large_is_the_resolution_error_too_many`,
`each_cause_takes_its_kind_and_class` (a `const` table of Choices' nine rows:
`Unknown`, `Unsatisfiable`, `Absent`, `Fetch`, `Resolve`, `Space`, `Start`,
`Timeout`, `Other`),
`a_container_target_is_never_asked_to_resolve` (key `needed`, no error),
`an_error_with_no_place_sorts_after_placed_errors_by_target`,
`the_shipped_limits_hold_8192_packages_and_600_s`,
`a_resolve_carries_the_deadline_now_plus_600_s`; records:
`mmdebstraps_timeout_prints_as_timed_out`,
`mmdebstraps_refused_start_prints_as_could_not_start`; argv:
`render_takes_resolve`; process, `end_to_end.rs`: `end_to_end_render_resolve` (a
laid record, no network: `known`, exit 0). Real loop:
`resolve_with_its_key_recorded_runs_no_mmdebstrap` (a laid record, a dead proxy:
`known`, exit 0, `staging/` empty). Live, in `conformance_render.rs`, with
`fixtures::resolve(&Repository, targets, Form, proxy) -> Rendered` and
`dead_proxy()` (binds `127.0.0.1:0`, reads the port, drops the listener):
```rust
#[test]
fn an_unreachable_mirror_fails_the_resolution_as_mmdebstraps_and_exits_75() {
    let r = Repository::make("unreachable", fixtures::TEST_REPOSITORY);
    let out = fixtures::resolve(&r, &["test-min"], Form::Text, &fixtures::dead_proxy());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("test-min: mmdebstrap exited ") && text.contains(" needed\n"), "7.2: {text}");
    assert_eq!(out.exit, Exit::Code(75));
    assert!(fixtures::listed(&r.scratch().join("store/staging")).is_empty(), "5.2");
}
```
Proof: live, `heavy cargo nextest run -p hearth-builder-world --profile
conformance -E 'binary(=conformance_render)'` once on the run's machine: real
mmdebstrap fails on an unreachable mirror, exit 75, the scratch gone. Bound: 10
s; unit, in the gate,
`a_resolve_makes_its_scratch_writes_sources_list_then_spawns_mmdebstrap`, 0.02
s.
Don't: run mmdebstrap in the gate; retry; keep a proxy in any key.
Size: at most 300 lines.
Removes: 32.2's usage line for `--resolve`; 24.1's "`--resolve` refused".
### 24.3 The indices read, and the closure matched
Scope: apt's lists read, stanzas up, hashed in the same pass; `Resolved`'s
lines; render's `Closure` (5.3); the `resolution` kind.
Needs: hearth 38.1 (`packages`, `lists`); nothing else beyond 24.2.
Files: render `src/{resolution, keys, errors, vocabulary}.rs`,
`src/tests/{fixtures, resolution, keys}.rs`; protocol `src/{resolve, lib}.rs`,
new `src/indices.rs`, `src/translate/{resolution, errors}.rs`,
`src/tests/translate/errors.rs` (`Kind::Resolution`, beside :241; `Limit::Closure`
is 24.2's), new
`src/tests/indices.rs`; records `src/errors.rs` and a new
`src/errors/resolution.rs` beside `errors/repository.rs` (`Kind::Resolution`),
`src/tests/errors.rs`; `tests/render/src/declarations.rs` (its `Kind` match,
:223, gains `| Kind::Resolution(_)`);
`tests/builder/{src/fixtures.rs, tests/conformance_render.rs}`.
Shape: the sketches. Lists: `OpenDirectory` of the scratch's
`root/var/lib/apt/lists`, `ListPage`, `lists::listed` per name against
`lists::indices(lines, architecture)`: an `Index` is read, `Other` skipped; a
`_Packages` or `_InRelease` no line names, a `Compressed` one or a missing index
fails the run `Run { ended: Output, cause: None }` (`lock` and `partial/` are
skipped: render.md 5.3). Each index: `OpenReadNoFollow`, `ReadAt { max: files.read }`, `Intake::meet`, `Stanza { line
}` up, hashed; then `Resolved { lines, tools, costs }`. `Closure::stanza` takes
the first line's stanza, a later one agreeing on size and digest; `close(names)`
(transitional: dropped, key `needed`). Messages (the plan's wording, within
render.md 7.3's forms): `apt cannot locate package "<p>"`, `unsatisfiable
dependencies of "<p>"`, `the archive <uri> does not have <suite>`, `apt would
install <n> <v> <a>, which no Packages index lists`, `"<p>" was requested, and
apt installs no package of that name`, `the Packages indices disagree on <n>
<v> <a>`, `a closure past <max> packages`.
Test shape: unit under 20 ms. Render, `src/tests/resolution.rs`:
`a_package_takes_its_first_lines_stanza_and_a_later_one_must_agree` (`source` 0;
`keys::closure` is 24.1's `33b37c0f…`: render.md 4.5),
`a_package_found_only_in_a_later_line_takes_that_lines_place` (`source` 1),
`the_closure_is_sorted_by_name_then_architecture` (`apt-utils`, `apt` in `amd64`
and `all`), `a_packages_filename_and_size_enter_the_record_and_not_the_digest`,
`an_inst_triple_with_no_stanza_is_unmatched` (operational),
`stanzas_disagreeing_on_size_or_digest_fail` (operational),
`a_requested_package_outside_the_closure_is_missing` (configuration),
`a_closure_past_its_limit_is_too_many_at_a_small_limit` (`closure: 2`),
`a_repeated_inst_line_is_one_package`; protocol, `src/tests/indices.rs`:
`the_lists_are_listed_then_each_index_read_in_order`,
`a_stanza_split_across_two_reads_reaches_render_whole` (`files.read: 64`),
`each_index_is_hashed_in_the_pass_that_reads_it` (`sha256sum`'s literal),
`inrelease_is_hashed_and_never_read_for_stanzas`,
`a_missing_or_compressed_index_fails_the_run`,
`a_packages_list_no_line_names_fails_the_run`,
`names_that_are_no_index_are_skipped`; `src/tests/translate.rs`:
`a_resolved_line_holds_the_sources_uri_suite_components_and_indices`; records:
`a_resolution_error_names_its_package`,
`each_resolution_message_prints_as_worded` (a `const` table over the seven
strings). Live:
`an_unknown_package_fails_its_resolution_naming_it_and_exits_2` (test-min
requesting `hearth-absent`, `tasks/absent/task.json` and
`hosts/test/test-min.json` `{"absent": {}}`: `test-min: apt cannot locate
package "hearth-absent"`, exit 2).
Proof: live, `binary(=conformance_render)` once: apt's verdict on a missing
package is the `resolution` error, exit 2; unit, in the gate,
`a_package_takes_its_first_lines_stanza_and_a_later_one_must_agree`, 0.02 s.
Bound: 15 s a test, as nextest prints it.
Don't: hold more than one `ReadAt` and one demand; decompress; match in the
protocol layer.
Size: at most 300 lines.
Removes: 24.2's "a run that succeeds reads no index".

## Interface sketches
`B` is `Box<[u8]>`. An enum hearth 11, 12, 23, 33 or 25 makes keeps its variants,
these added.
```rust
// crates/hearth-domain-render/src/keys.rs (24.1; closure 24.3): resolution(&Inputs) -> Digest, closure(&[Package]) -> Digest
// (render.md 4.5), layers(&Digest × 4) -> Keys, part(&Spec, &[u8]) -> Digest; `Limits` gains `closure: u32` (8,192), `resolve: skein_lib::Duration` (600 s; clippy.toml:74 disallows `core::time::Duration`)
pub struct Keys { pub packages: Digest, pub software: Digest, pub config: Digest }
pub enum Resolution { Needed { key: Digest }, Known { key: Digest, keys: Keys } } // `Spec.resolution: Option<Box<Resolution>>`: boxed, `Spec` is 208 bytes, not 336
// crates/hearth-domain-render/src/vocabulary.rs; `Event::Render` gains `resolve: bool` (24.2)
pub enum Event {
    Found { request: Token, digest: Digest },                 // Find: the closure's digest (31.1 widens it to `resolution: resolution::Record`)
    Absent { request: Token },                                // Find
    Inst { request: Token, inst: resolution::Inst }, Stanza { request: Token, line: u32, stanza: resolution::Stanza }, // Resolve
    Resolved { request: Token, lines: Box<[resolution::Line]>, tools: resolution::Tools, costs: resolution::Costs }, // terminal
}
pub enum Request {
    Find { request: Token, key: Digest },
    Resolve { request: Token, key: Digest, sources: parts::packages::Sources, names: Box<[B]>,
        variant: parts::packages::Variant, apt: parts::packages::Apt, deadline: Time },
}
// crates/hearth-domain-render/src/resolution.rs (24.2; Stanza, Line, Index 24.3; Record, Made 31.1)
pub struct Inst { pub name: B, pub version: B, pub architecture: B }
pub struct Stanza { pub package: B, pub version: B, pub architecture: B, pub filename: B, pub size: u64, pub sha256: Digest }
pub struct Line { pub uri: B, pub suite: B, pub components: Box<[B]>, pub indices: Box<[Index]> }
pub struct Index { pub path: B, pub size: u64, pub sha256: Digest }
pub struct Package { pub name: B, pub architecture: B, pub version: B, pub source: u32, pub filename: B, pub size: u64, pub sha256: Digest }
pub struct Tools { pub apt: B, pub mmdebstrap: B }
pub struct Costs { pub fetched: u64, pub took: skein_lib::Duration }
// `Closure`, at most `Limits::closure` triples by name then architecture: new(&Limits), inst(Inst) (TooMany), stanza(line, Stanza)
// (Disagree), close(self, names: &[B]) -> Result<Box<[Package]>, errors::Resolution> (Unmatched, Missing)
// crates/hearth-domain-render/src/errors.rs: hearth 12's and 42.1's, these added
pub enum Kind { Resolution(Resolution) }
/// 7.2's `resolution`: configuration but `Unmatched` and `Disagree`, operational (render.md 5.3).
pub enum Resolution {
    Unknown { package: B }, Unsatisfiable { package: Option<B> }, Missing { package: B },
    Absent { uri: B, suite: B }, // the archive lacks the suite: the source line (5.5)
    Unmatched { name: B, version: B, architecture: B }, Disagree { name: B, version: B, architecture: B },
    TooMany { max: u32 }, // past `closure`; hearth 31 adds `TooLarge { max: u64 }`
}
pub enum Command { Mmdebstrap }
pub enum Ended { Timeout }
pub enum Cause { Fetch, Resolve, Space, Absent { uri: B, suite: B }, Unknown(B), Unsatisfiable(Option<B>), Other } // 07's `tail::Cause`; `Absent` gains its line
pub enum Limit { Closure } // hearth 31 adds `Record`
// vocabulary.rs's `Failure` (hearth 42.1's, :84) keeps `Tool(Tool)`, `TooLarge` and `Store` (declined or refused), and gains mmdebstrap's run:
pub enum Failure { Run { tool: Tool, cause: Option<Cause> } }
```
```rust
// crates/hearth-domain-store/src/vocabulary.rs: hearth 23's and 33's, these added (Find 24.1; hearth 31 adds Record, Room)
pub enum Event {
    Find { key: Digest },
    Resolution { record: Resolution }, // `Load`'s answer, whole
}
pub enum Answer {
    Found { resolution: Resolution }, Absent, // Find; Find (or unreadable, 6.8)
    Declined { why: Decline },                // 8.1: no index, or past `resolutions`
}
pub enum Decline { Full { limit: Limit, most: u32 }, Unusable { step: Step } } // hearth 31 adds `Bound`
// `Resolution`: store.md 6.4's fields as render's `Record`, hearth 09's `resolution::Resolution`
// crates/hearth-domain-builder/src/vocabulary.rs: `Call::Render` gains `resolve: bool` (24.2)
// crates/hearth-domain-builder/src/resolution.rs (24.1): found(store::Resolution) -> Digest (31.1: -> render::resolution::Record)
// crates/hearth-protocol-builder/src/lib.rs: `Limits` gains `mmdebstrap: hearth_protocol_mmdebstrap::limits::Limits`; `Protocol::with_proxy(self, proxy: Option<Proxy>) -> Protocol`, called by the service (24.2: `new` has 7 arguments with 33.3's `store`, clippy's limit)
// fire(&mut Protocol, &Env<Limits>, &mut Queue<builder::Event>, &mut Queue<Asked>) for `idle` and `deadline`; next_deadline(&Protocol) -> Option<Time>
// src/translate/resolution.rs (24.2 on), each both ways in src/tests/translate.rs
pub fn simulate(sources: &render::parts::packages::Sources, names: &[B], variant: render::parts::packages::Variant,
    apt: render::parts::packages::Apt, limits: &mmdebstrap::limits::Limits) -> Option<mmdebstrap::vector::Simulate>; // `None`: a name out of form, or `Variant::Container` (render asks none)
// inst(output::Inst) -> Inst; stanza(packages::Stanza) -> Stanza; line(&vector::Line, Box<[Index]>) -> Line
// `Absent` takes the first line whose `uri` the tail's 404 `Err:` names, else the first:
pub fn cause(cause: tail::Cause, lines: &[vector::Line], tail: &[B]) -> Cause;
// src/translate/store.rs: `resolution(tree::resolution::Resolution) -> store::Resolution` (24.1, replacing 33.2's `inputs`, `closed`)
// hearth-service-builder: `Configuration` gains `proxy: Option<B>` (24.2); hearth: `Invocation` likewise
// hearth-client-protocol-records: errors.rs's `Kind::Resolution`, `Command::Mmdebstrap`, `Ended::Timeout`
// mirror render's
```

## Questions to raise when reached
None.

## Done when
1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-24\.[1-3]'` prints 3.
2. `git -C ~/src/rust/hearth ls-tree --name-only origin/main
   crates/hearth-domain-render/src/ crates/hearth-protocol-builder/src/
   crates/hearth-protocol-builder/src/translate/ | grep -c -x -E
   '.*/(keys|resolution|resolve|indices)\.rs'` prints 5.
3. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run -p
   hearth-domain-render -p hearth-domain-store -p hearth-domain-builder -p
   hearth-protocol-builder -p hearth-client-protocol-records -p
   hearth-builder-world -p hearth` passes.
4. Once, on the run's machine, in that worktree: `heavy cargo nextest run -p
   hearth-builder-world --profile conformance -E 'binary(=conformance_render)'`
   passes, its `Summary` naming 2 tests, within the profile's 2 minutes.
