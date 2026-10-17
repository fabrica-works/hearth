# Findings: skein's test kit for hearth's M1

Read at skein 7e8fb70 (jig ad61bcfc, temper 118f7fb70) on 2026-10-16. Paths are relative to `~/src/rust/`.

## 1. skein-world: domain worlds, the referee, seeds, replay

- The domain kit is `skein_world::domain` (skein/testing/skein-world/src/domain.rs:1-26). It was extracted from temper's harness, and it holds no IO, no clock and no service policy. The world writes its own loop and acts as the domain's parent.
- `Schedule<D>` (domain/schedule.rs:27): `send(at, d) -> Key` (:47), `withdraw`, `next(now)` (:59), `next_time`. Deliveries come out in time and serial order, each once.
- `Stage<L, E, R>` (domain/stage.rs:20): `env`, `inbox`, `out`, `max_out`; `tick`, `push`, `next_event`, `has_room`. It holds an event back until the output queue has room for `max_out`.
- `Ledger<K, V>` (domain/ledger.rs:16): `open`, `end`, `take`, `assert_settled` (:83). It allows one terminal per open key.
- `Referee<X: Expectations>` (domain/referee.rs:55), `trait Expectations { Seen, Name, Stimulus; observe(seen, &mut Judge) }` (:22). `Judge` (:174) gives `check`, `fail`, `expect(name, within)`, `meet`, `rearm`, `withdraw`, `inject`, `inject_now` and `stop`. `Verdict` (:306) is `Passed`, `Open`, `Stopped` or `Failed(Failure)`. The referee also has `assert_passed(seed)`.
- `Trace` (domain/trace.rs:14) has `log(now, line)` and `lines()`. `assert_replays(seed, other, run: Fn(u64) -> (Vec<String>, T))` (domain/trace.rs:36) runs the seed twice and checks that it gives the same trace and the same `T`, then checks that `other` gives a different trace.
- `Span::millis(min, max).draw(rng)` (domain.rs:41) draws a latency. `domain::heap` re-exports `Counting`, `Measured` and `Meter` (domain.rs:32).
- `Outcome<P, M>` (world.rs:113) belongs to the simulated `World`, not to the domain kit. Its fields are `machine`, `killed`, `seed`, `trace`, `procs`, `iterations`, `end`, `heap` and `held`. A domain world defines its own outcome, usually `Stats` plus a `Verdict`.
- State digests (`state_digest`, the `digests` option) are designed in skein's testing.md section 5. They are not built: grep finds no `state_digest` (testing.md, section 8).
- Pinned seeds have no kit. They are plain constants. skein/tests/echo/tests/scenarios.rs:108 `a_stream_that_fails_while_its_line_is_answered_settles` is one pinned seed kept as a scenario. jig/tests/client/tests/fuzzy_joint.rs:5-7 has `FIRST_SEED`, `SEEDS` and `const PINNED: &[u64]`, chained into the sweep at :10.
- `FINDINGS` exists only in prose: jig/docs/development/workflow.md:95, temper/docs/development/workflow.md:72 and skein/docs/foundation/testing-strategy.md:498. No repository has a `FINDINGS` list. No world has an ignored test that replays one. The only `#[ignore]` anywhere is temper/testing/temper-throwaway-forgejo/tests/throwaway.rs:34.
- **Child domain world (reference):** jig/tests/people/tests/simulation.rs:22 `the_people_stories_and_both_restart_cuts`. A scripted parent plays the store, commits and cold restores, and uses `Referee` and `Trace` (jig/tests/people/src/lib.rs:1-4). For replay copy simulation.rs:399 `facts_change_nothing_and_a_seed_replays`. For the sweep copy tests/fuzzy_simulation.rs:5 `random_people_settle_and_reach_every_ending`.
- **Child domain world over the fake checkout:** temper/tests/worker/checkout/tests/simulation.rs:33 `a_client_prepares_edits_pushes_and_its_change_lands_on_a_base_branch_created_for_it`. Here the world plays the protocol layer and io on skein-fake-checkout with `Schedule`, `Stage`, `Ledger` and `Trace` (src/world.rs:4). Replay is at :228 `a_seed_replays_to_the_same_run`. The sweep is tests/fuzzy_simulation.rs:18, 1000 seeds.
- **Root domain with its children:** temper/tests/worker/domain/tests/whole.rs:5 `a_prepared_run_delivers_the_complete_tree_and_answers`.
- **Referee unit tests:** skein/tests/world/tests/domain_referee.rs:79 `a_test_passes_once_every_expectation_is_met_and_nothing_is_left_to_inject`.

## 2. skein-heap: the counting allocator

- `Counting` (skein/testing/skein-heap/src/lib.rs:57) is an `unsafe impl GlobalAlloc` (:281). A binary declares it with `#[global_allocator] static HEAP: Counting = Counting;`. Counts are kept per thread.
- `Meter` (:341): `new`, `held`, `start` (:363), `end() -> Measured` (:380), `check(step, bound, &what) -> u64` (:394). `check` measures a step's peak less what it handed out. The test drops the step's requests before it calls `check`.
- `Span::start()` (:439) and `end() -> Grown` (:448) measure the signed growth of the heap around one call. The simulated `World` uses them.
- In a simulated world, `Memory::Checked` (skein-world/src/heap.rs:27) checks each process's heap against `Host::worst_case()` (skein-shell/src/drive.rs:45) at every iteration. `Outcome.heap` gives `(most, bound)` for each process.
- A memory test is a binary of its own, `tests/memory.rs`, because the global allocator covers the whole binary (skein testing.md, section 6).
- **Domain step against its worst case (reference):** jig/tests/people/tests/memory.rs:71 `full_people_roles_signins_pending_waiters_answers_and_expiry_fit`. A temper example is temper/tests/worker/checkout/tests/memory.rs:258 `a_domain_with_every_workspace_held_at_its_limits_stays_within_its_worst_case`.
- **Machine against its worst case:** skein/tests/json/tests/memory.rs:134 `every_entry_point_holds_no_more_than_its_worst_case_at_its_limits`.
- **Simulated world:** skein/tests/echo/tests/scenarios.rs:130 `memory_is_checked_at_every_iteration_against_the_worst_cases`.

## 3. Machine worlds and transcripts

- skein's JSON world is `skein-json-world` in skein/tests/json. Its `world::run(document, &Settings, seed) -> Run` (src/world.rs:199) and `check` (:271) play the side below and the side above. The side below delivers bytes cut at random, late, ends early or fails. The side above demands slowly, stops, and closes in every state. `Settings::calm` (:66), `chaotic` (:87) and `Settings.cut` set the cuts.
- Its reference parser is `reference::parse(document, &limits) -> Decoded` (src/reference.rs:25). Its generators are `generate::tokens` (src/generate.rs:21), `render` (:137) and `mutate(rng, doc)` (:229). All of these are local to the world and cannot be reused.
- A JSON transcript pair is `transcripts/<name>.json` beside `<name>.expect` (src/transcript.rs:1-24). The `.expect` file starts with `#` comment lines, then an optional `limits …` line, then one token per line, then `done` or `failed <error>`. `transcript::all()` (:54) loads every pair, and `render` (:149) drafts an expectation. Names start with the source: `anthropic-…`, `forgejo-…`, `hostile-…`.
- HTTP uses the same shape: `<name>.http` beside `<name>.expect`, 95 files in skein/tests/http/transcripts.
- `skein_world::stream::Wire` (skein-world/src/stream.rs:16) can be reused. It is a seeded stream that delivers the peer's bytes in fragments of 1 to 30 bytes and grants room late. It checks one demand at a time and `Send` within the granted room. Its methods are `new(seed)`, `write`, `take(Down)` and `answer() -> Option<Up>`.
- temper's git machine works from raw captures. Each case is a directory `tools/git-transcripts/transcripts/<case>/` with the files `argv`, `exit`, `stdout` and `stderr`, ids masked to counters (temper/tools/git-transcripts/README.md:1-15). A capture tool runs the real git against a throwaway Forgejo. `transcripts::read(case)` and `mutations(bytes)` drop a line, cut a line, raise a hex digit or drop the terminator (temper/tests/worker/protocol-git/src/transcripts.rs:13,23).
- **Reference (cut at random):** skein/tests/json/tests/transcripts.rs:43 `every_transcript_decodes_to_its_expectation_whatever_the_cuts_and_scans`. For a missing expectation copy :24 `every_transcript_has_an_expectation_and_the_reference_parser_agrees_with_it`. For cut, mutated and failed transcripts in a sweep copy tests/fuzzy_documents.rs:142 `transcripts_cut_mutated_and_failed_read_as_the_reference_reads_them`.
- **Reference (git output, mutated):** temper/tests/worker/protocol-git/tests/transcripts.rs:111 `mutated_outputs_parse_to_none_never_to_another_value`.

## 4. skein-fake-checkout

- `Checkout` (skein/testing/skein-fake-checkout/src/lib.rs:39) keeps files in memory under byte paths. Its methods are `mkdir`, `write -> version` (:393), `link`, `special`, `remove`, `exists`, `content`, `version`, `files`, `tree(at)` (:470), `replace_tree` (:489), `root`, `load` (:514), `scan` (:534), `store(.., Expect)` (:563) and `search` (:313). It has scripted commands: `program(cmd, Program)` (:239), `spawn` (:248) and `finish` (:284). It has no modes, owners, permissions or open handles (fake-checkout.md, section 2).
- The git module (src/git.rs): `type Tree = BTreeMap<Vec<u8>, Vec<u8>>` (:23) maps a path to bytes, with no mode, no symbolic link and no blob object. `trait Remote` (:31), which the caller implements, has `heads`, `fetch`, `create`, `push`, `parent`, `merge_parent`, `tree`, `message` and `store`. Commit ids are `u64` counters that increase topologically.
- The git operations are `clone_repository` (:156), `fetch` (:184), `create`, `check_out` (:214), `commit` (:230), `push` (:253), `merge` (:280), `commit_merging` (:346) and `push_expected` (:378). Local state is kept in marker files: `.git/objects/<id>`, `.git/temper-head` and `.git/MERGE_HEAD`.
- It has typed operations only. There is no git text face: no `rev-parse`, `ls-tree`, `cat-file` or `diff`, no object ids in hex, and no SHA. The `Remote` fixture is private to its tests (tests/common/mod.rs:22 `Store`).
- temper adds a text face on top of it. temper-fake-checkout re-exports skein's crate (temper/tests/fake-checkout/src/lib.rs:10). temper-fake-git checks argv and env (`check`, temper/testing/temper-fake-git/src/check.rs:33; the fixed vectors are in `parse`, :187). It answers bytes and an exit (`answer`, src/answer.rs:36; `Answer { exit, stdout, stderr }`, :12). Its ids are counted ids printed as 40 hex digits (src/id.rs:4). It runs as a hosted process on the fake machine (`Git`, src/program.rs:44; `impl Host`, :355). Its commands are clone, fetch, checkout, clean, merge, diff-files, add, diff-index, write-tree, commit-tree, merge-base and push. It has no ls-tree or cat-file.
- **Reference:** skein/testing/skein-fake-checkout/tests/git.rs:12 `clone_and_fetch_import_both_parent_chains_without_rewriting_existing_objects`. For the text face copy temper/testing/temper-fake-git/tests/transcripts.rs:49 `each_transcript_s_exit_and_masked_stdout_match_the_fake_s`, which compares the fake with the real git's captures.

## 5. Simulated worlds

- skein-sim (skein/crates/skein-sim): `Sim::new(seed, Config)` (src/sim.rs:150), `Config::calm` (src/config.rs:142) and `Config::chaos` (:155). It also has `spawn_process`, `submit`, `reap`, `advance` and `trace`. The machine seam is `Call` (src/machine.rs:48) and `Answer` (:102). `Program::Service` (:121) runs a spawned child as a process that the world hosts.
- skein-world `World<P, R, M>` (src/world.rs:65): `new(seed, config, referee, Memory)`, `new_controlled`, `with_machine(M)` (:210) and `host(HostedProgram { program, make, instances, operations })` (:239). A spawn of `program` starts `make` as a hosted `Host`. It also has `host_roots`, `spawn`, `spawn_root`, `spawn_restartable`, `cut(host, before, Cut::{Kill, PowerLoss})` (:280) and `run() -> Outcome` (:350). It checks the teardown and quiescence of the settled world.
- `trait Machine { step(call, answers); cut; open_root; open_append; … }` (src/program.rs:90) is the seam where a fake machine plugs in. `NoMachine` (:130) refuses every call.
- skein-fake-machine is minimal: `Machine::for_user`, `lay(&[Item])`, `crash(seed)`, `step` and `serve` (skein/testing/skein-fake-machine/src/fs.rs:262,291,316,426; src/face.rs:19,110). Its own docs say a service's fake machine is the service's own (src/lib.rs:20).
- A world that runs a service's `iterate` on the simulator with a fake machine and a fake git spawned through it already exists: temper/tests/one-process/src/world.rs:612-660. It hosts `/usr/bin/git` as `temper_fake_git::Git` and runs `World::new_controlled(..).with_machine(Machine(skein_fake_machine::Machine))`.
- **Reference (service and spawned fake tool):** temper/tests/one-process/tests/one_process.rs:43 `an_adopted_repository_gets_its_change_pushed_through_git`.
- **Smallest hosting example:** skein/tests/world/tests/hosted.rs:20 `a_spawn_hosts_the_child_with_its_arguments_environment_and_pipes`.
- **A service on the simulator with no machine:** skein/tests/echo/tests/scenarios.rs:40 `many_clients_are_served_past_both_refusals_at_the_entrance`.

## 6. Real loop and end to end

- `skein_world::real::World` (skein-world/src/real.rs:133) takes the same `Host`s and `HostedProgram`s over the real shared ring and clock: `new(referee)`, `spawn_with_fds`, `host` and `run(&Clock, patience)` (:242). A spawn whose program is not hosted runs the real executable.
- `skein_world::end_to_end` provides `Binary::start(Command { program, arguments, directory, environment }, Mode::Pipes | terminal, stderr_limit)` (src/end_to_end.rs:131), plus `signal`, `signal_group`, `end`, `stderr` and `tree_status`. skein-scratch `Scratch::new(label)` (skein/testing/skein-scratch/src/lib.rs:31) makes a scratch directory that is removed on drop.
- **Real loop, real child process:** skein/tests/world/tests/real_conformance.rs:249 `real_conformance_fixture_and_hosted_equivalent_return_the_same_parent_records`. The parent spawns the real `world_process_fixture` binary, then its hosted twin.
- **A service's real loop:** skein/tests/echo/tests/real.rs:25 `the_echo_serves_refuses_and_idles_out_its_clients_on_the_real_ring`.
- **End to end:** skein/tests/echo/tests/end_to_end.rs:185 `the_shipped_echo_answers_two_loopback_clients_and_terminates_successfully`. For a one-shot command whose output and exit are checked, copy temper/crates/temper/tests/process.rs:7 `temper_serve_prints_its_link_and_drains_on_a_signal` (support/mod.rs:82 `serve`).
- Real git runs only in temper's end-to-end test process.rs:71, `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart`, which needs a throwaway Forgejo, and in temper's capture tool (tools/git-transcripts). No gate test runs real git on a scratch repository.

## 7. A world crate's layout

- A world crate under `tests/` is ordinary Rust. Its Cargo.toml has a lints table of its own, not `workspace = true` (skein/tests/json/Cargo.toml:17-33):
  - `[lints.rust]`: `unsafe_code = "forbid"`, `elided_lifetimes_in_paths` and `unused_must_use` set to `"deny"`;
  - `[lints.clippy]`: `pedantic` warns, `missing_panics_doc` is allowed, and these deny: `wildcard_enum_match_arm`, `match_wildcard_for_single_variants`, `unwrap_used`, `allow_attributes`, `allow_attributes_without_reason`, `disallowed_types`, `disallowed_methods`.
- The crates share `tests/clippy.toml` (skein/tests/clippy.toml:1-22). It bans `HashMap`, `HashSet`, `RandomState`, `Instant`, `SystemTime`, the `thread` spawns and `std::env::var`, and allows unwrap in tests.
- `testing/*` crates such as skein-world use `[lints] workspace = true` and a `clippy.toml` of their own (testing/skein-world/Cargo.toml).
- Suites are configured in skein/.config/nextest.toml:7-21. The default suite filters out `binary(/^fuzzy_/)` and has 15s. The `fuzzy` profile has 60s.
- skein has no world crate with `src/fixtures.rs`; jig and temper have 15.
- **Compact crate to copy, with fixtures and transcripts:** temper/tests/worker/protocol-git, 1364 lines:
  - `Cargo.toml`;
  - `src/lib.rs`, `src/fixtures.rs`, `src/io.rs`, `src/referee.rs`, `src/transcripts.rs` and `src/world.rs`;
  - `tests/protocol_git.rs`, with its memory test, which declares `Counting` at :10;
  - `tests/transcripts.rs`;
  - `tests/fuzzy_protocol_git.rs`, 15 lines, 64 seeds.
- **Domain world shape:** jig/tests/people, 2366 lines: `Cargo.toml`, `src/lib.rs`, `src/referee.rs`, `src/world.rs`, `tests/simulation.rs`, `tests/referee.rs`, `tests/memory.rs` and `tests/fuzzy_simulation.rs`.
- **Smallest in skein:** skein/tests/kv: `Cargo.toml`, `src/lib.rs` (1 line), `tests/store.rs`, `tests/real.rs`, `tests/memory.rs` and `tests/fuzzy_map.rs`.

## 8. Goldens rewritten under an environment variable

- `SKEIN_CODEGEN_WRITE=1` rewrites generated code and golden bytes (skein/tests/codec/tests/drift.rs:14). After writing it passes; it does not fail. jig and temper copy it in jig/tests/store-codecs/tests/drift.rs:16, jig/tests/client-wire/tests/drift.rs:14 and temper/tests/codecs/tests/drift.rs:12. The flag is read with `std::env::var_os`, which tests/clippy.toml does not ban. Only `std::env::var` is banned.
- skein/tests/json/tests/transcripts.rs:24 does something close without a variable. When an expectation is missing, the test prints a draft from the reference parser and fails, and the draft is checked by hand before it is kept.
- skein-ui-text's goldens are hand-written and compared with `include_bytes!` (skein/crates/skein-ui-text/src/tests.rs:13). Nothing rewrites them.
- No repository rewrites goldens and still fails. `HEARTH_BLESS=1` has no precedent. Copy drift.rs:14, then fail after writing.

## Gaps

- skein has no git text face. Neither skein-fake-checkout nor temper-fake-git prints `rev-parse`, `ls-tree -r -l -z`, `cat-file --batch` or `diff --no-index`. hearth writes these outputs.
- skein-fake-checkout trees carry no modes (100644, 100755), no symbolic links (120000), no submodules (160000) and no blob sizes or objects. Commit ids are counters, not SHA-1 or SHA-256. skein has no SHA-1 in code (its SHA-256 is `skein_lib::sha256`), and test code would take a crate, as jig's people world takes `sha2` (jig/tests/people/Cargo.toml:10).
- skein has no shared machine-world kit: the cuts, generators, reference and transcript loaders are local to each world. hearth writes its own for config, template, git and mmdebstrap.
- skein has no transcript format for a tool's run (argv, exit, stdout, stderr). temper's is the only one, and its capture tool needs a Forgejo.
- No fuzz targets exist: fuzzing is seed sweeps (skein testing.md, section 8). hearth's "fuzzed alone" has no cargo-fuzz precedent.
- No `FINDINGS` list exists, and no ignored test replays a finding. Pinned seeds are ad hoc constants.
- Bless-and-still-fail is not built anywhere. hearth reads `HEARTH_BLESS` with `var_os`, because `std::env::var` is banned.
- State digests (`Host::state_digest`, the trace's digests) are designed but not built. `assert_replays` compares only the trace and a value.
- No gate test runs real git on a scratch repository in a real loop.
- skein-fake-machine is minimal, and hearth's fake machine is its own: owners, modes, processes and the universe's index.
