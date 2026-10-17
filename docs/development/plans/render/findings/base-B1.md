# B1: the plan's commands at its base, skein 02 to 05 and hearth 01

Run 2026-10-17 on debian-8gb-hel1-1, through `heavy`. Scratch worktrees (kept):
`~/src/rust/tmp/plan/hearth-render-scratch/B1-skein` (skein `f981746`, the
sketches' stubs and the variants of 03 to 05, uncommitted), `B1-hearth` (hearth
`175e92f`, member `scratch/01`), and for the downstream search `B1-smith`
(smith `11067216`), `B1-smith-old` (smith `29fa97c9`, the pin of jig and
temper), `B1-jig` (jig `b6a63f8c`), `B1-temper` (temper `524738af`), each with
`[patch."https://github.com/forgetic/skein.git"]` (and smith, jig) to the
scratch. Verdicts: "as the plan says", "defect, fixed", "at ready".

## skein 02

- `rg -n 'size_of::<[A-Za-z_:]*(Tokenizer|Collector)>' crates tests testing`, B1-skein: exit 0, one line `crates/skein-llm/src/openai/common.rs:69`: as the plan says.
- `heavy cargo nextest run -p skein-json-world -E 'test(=every_transcript_decodes_to_its_expectation_whatever_the_cuts_and_scans)'` (02.1, 02.3), B1-skein: exit 0, `1 test run: 1 passed`, 0.085 s: passes; the plan's "0.208 s at the base" is the whole crate's `Summary` (30 tests): defect, fixed (0.085 s).
- `heavy cargo nextest run -p skein-json-world -E 'test(=the_stream_ending_after_any_byte_decodes_as_that_prefix)'` (02.2), B1-skein: exit 0, 0.007 s: passes; "0.044 s" was not this test's: defect, fixed (0.007 s).
- `heavy cargo nextest run -p skein-json-world -E 'test(=every_entry_point_holds_no_more_than_its_worst_case_at_its_limits)'` (Don't, memory.rs:134), B1-skein: exit 0, 0.020 s: as the plan says (passes at the base).
- `heavy cargo nextest run -p skein-json-world --profile fuzzy`, B1-skein: exit 0, `8 passed`, 6.05 s (sweeps 6.05, 2.83, 2.68 s): the plan's 15.04, 6.97 and 6.33 s are x1's under the gate; the aim stands, no defect.
- Done 1 `git -C ~/src/rust/skein grep -n 'pub fn position(&self) -> Position' origin/main -- crates/skein-json/src/tokenizer.rs`: exit 1, nothing: as before the session.
- Done 2 `... log origin/main --grep='^Plan: render$' ... | grep -c -x -E 'skein-02\.[1-3]'`: prints 0: as before the session.
- Done 3 `git -C ~/src/rust/skein diff --stat 7e8fb70 origin/main -- crates/skein-json/src/collector.rs crates/skein-json/src/collector tests/json/transcripts`: exit 0, nothing: as the plan says (7e8fb70 exists).
- Done 4 `git -C ~/src/rust/skein grep -n 'world::run(' origin/main -- tests/json/tests/transcripts.rs`: `transcripts.rs:53`: before the session (the session moves it to `check`).
- Counts: 39 transcript pairs, `anthropic-message-tool-use.json` 41 lines, `openai-chat-tool-calls.json` 46, the 46-byte document of `world.rs:51`, `tokenizer.rs` 988 lines, `reference::parse` 8 calls in `tests/json` (13 in skein, 5 in `tests/http` and `tests/protocol`, kept), `Run {` built once (`world.rs:251`), `reference::demands` one user (`world.rs:616`): as the plan says. 24 line references of Files (tokenizer.rs, string.rs, writer.rs:275, reference.rs, world.rs, transcripts.rs:43/:53): as the plan says (`up (:364)` is a site inside `up`, :353).
- 54 position rows of the Test shapes (02.1 to 02.3), each recounted from its document and offset by `/tmp/B1pos.py`: all as the plan says.
- Users: `Tokenizer` is `size_of`'d only at `skein-llm/src/openai/common.rs:69` (summed into `encoding_worst_case`; `skein-llm-connection/src/limits.rs:87` subtracts two of them; smith `config.rs:1479` prints it, `tests/protocol-llm` compare bounds with measured): computed, no literal; smith, jig and temper use `Token`, `Event`, `Error` only (no signature moves): as the plan says. No premise needs a number.
- Sketch `crates/skein-json/src/tokenizer/position.rs` (`Position`, `Lines`) and `Tokenizer::position`, B1-skein: `heavy cargo clippy -p skein-json -p skein-json-world --all-targets -- -D warnings`: exit 0, as the plan says.
- Sketch `tests/json/src/reference.rs` (`Reading`, `reading`, `position`, `demands`' 3-tuple under another name) and `Run::positions`, B1-skein: the same command, exit 0.
- Reference tests `src/tests/position.rs` (with a `placed` stub) and `tests/json/tests/positions.rs`: the same command, exit 0; their runs at ready.
- Proof lines of 02.1 to 02.3 after the session, Done 1 to 4 after the session: at ready.

## skein 03

- `rg -n 'pending_moves|locked' testing/skein-fake-machine/src/fs.rs`, B1-skein: exit 1, nothing: as the plan says at the base.
- `rg -n 'fn state_digest' testing/skein-fake-machine/src`, B1-skein: exit 1, nothing: as the plan says.
- `rg -n --type rust 'impl(<[^>]*>)? +(\w+::)?Host +for' <repo> -g '!target'` for skein, smith, jig, temper (at `origin/main`, by `git grep`): 38, 24, 15 and 21 (plan: temper 18, total 95): defect, fixed (temper 21, total 98). `Machine`: 11, 5, 5, 1: as the plan says (22).
- Proofs of 03.1 and 03.2 as written, `test(=a_usize_digests_as_the_eight_bytes_of_its_u64)`, B1-skein: exit 4, `0 tests run`: a unit test in `skein-lib` is named `tests::digest::<name>`, so `=name` matches nothing: defect, fixed (`test(=tests::digest::...)`, exit 0, 1 passed). 03.3 and 03.4 name integration tests (`tests/echo`), whose names carry no path: at ready.
- `printf '\002\001' | sha256sum`: `25dfd29c...12a0`; the first eight bytes `0x25dfd29c09617dcc`; `printf 'abc'`: `ba7816bf...15ad`: as the plan says. The other `printf` rows are computed by their tests (at ready).
- Done 1 prints 0 (before the session); Done 2 `git grep -n -E 'pub struct StateHasher|fn state_digest[(<]|Digest\(\[u8; 32\]\)|pub enum Digests|pub const MACHINE' origin/main -- crates testing`: exit 1, nothing (the regex parses; the sketches give 9 lines as listed: `digest.rs` 2, `drive.rs`, `program.rs`, `writer_pipe.rs`, `world.rs`, `fs.rs`, `trace.rs`, `sim.rs`); Done 3 `git diff 7e8fb70 origin/main -- ... | grep -E '^-\s*(pub )?fn '`: nothing: as the plan says.
- Done 4: `heavy cargo nextest list ... -E 'test(/digest/)'` at `f981746` (in `hearth-render-skein`): 4 tests; the plan's 31 recounted from the test names (6, 10, 8, 7), all with `digest`: as the plan says; skein 04.1 and 05.1 add a digest test each (their Rules): Done 4 now says so (defect, fixed).
- 41 line references of Files (drive.rs:12-49, program.rs:90-126, world.rs:113-136/:210/:398/:409/:541/:575, writer_pipe.rs:279, lib.rs:18-68, trace.rs:14-35, sim.rs:32-45/:204/:517, sha256.rs:21, list.rs:143-147, containers.rs:22, echo lib.rs:77/:84, shell lib.rs:26, tests/io files.rs:270, echo proc.rs:51): as the plan says.
- Sketch `crates/skein-lib/src/digest.rs` (`StateHasher`, `state_digest`), reference test of 03.1: `heavy cargo clippy -p skein-lib --all-targets -- -D warnings`: exit 0; the test passes (`nextest run -p skein-lib -E 'test(=tests::digest::a_u16_digests_as_its_two_little_endian_bytes)'`).
- Sketch `Host::state_digest`, `Machine::state_digest`, `StateDigests`, `Digests`, `World::digests`, `Event::Digest`, `Pid::MACHINE`, `Sim::trace_digest`, B1-skein: `heavy cargo clippy --workspace --all-targets -- -D warnings`: exit 0 (with the planned arms).
- Reference tests of 03.3 (`tests/world/tests/digests.rs`, with `Stamped` and `Script::noting` stubs) and 03.4 (`tests/sim/tests/digests.rs`): pass (`nextest run -p skein-world-tests -p skein-sim-tests -E 'test(/digest/)'`, 2 passed).
- Users of `Event`: exhaustive matches are exactly the plan's eight (`census.rs` 3, `sign_in.rs`, `fuzzy_files.rs`, `fuzzy_exchange.rs` 2, found by the compiler across the workspace); smith `terminal.rs:34`, temper `host.rs:49/:52` are `matches!` / `if let`; jig none; hearth none. No type downstream has a `state_digest` of its own.
- Stubs by name: `Stamped`, `Script::noting` (tests/world), `placed` (skein-json tests, 02), `Lines` and `Position` of 02 (none of 03's).
- Proofs of 03.3 and 03.4 (`a_seed_replays_to_the_same_echo_digest_at_its_end`, `a_seed_replays_to_the_same_digests_after_every_iteration`; `assert_replays` is not used by `tests/echo` at the base, `Service::domain` does not exist): at ready.

## skein 04

- `git -C ~/src/rust/skein show d21d90d`: commit exists: as the plan says.
- `rg -n 'fn state_digest' testing/skein-fake-machine/src`: exit 1, nothing: as the plan says.
- Done 1 prints 0 (before the session); Done 2 `git grep -n -E '^    ListPage \{$|^        from_root: Fd,$' origin/main -- crates/skein-io/src/file.rs`: nothing at the base; the stubs in B1-skein print the two lines (`file.rs:153`, `:159`): at ready. Done 3 (`nextest run ... a_rename_across_over_the_ring... | a_directory_listed_by_pages...`): at ready.
- 36 line references of Files (file.rs:150, file_layer.rs:30/:263-278/:410/:446/:515/:544/:576-582/:648/:724/:734/:851/:993, kv store.rs:977, fs.rs:408/:426/:472/:509, tests/file.rs:34/:62, tests/store.rs:92, fake-machine tests.rs:331, tests/io ring.rs:194, files.rs:19-21, cuts.rs:272, conformance files.rs:525, store.rs:228, world files.rs:207/:228, private_tests.rs:111/:119/:367/:374): as the plan says.
- Users of `Request::Rename` and its `root`: `file_layer.rs:576`, `skein-kv/src/store.rs:977`; `{ owner, .. }` / `{ .. }` at `world/files.rs:228`, `private_tests.rs:119/:374`; downstream `{ .. }` in smith `protocol-machine/src/tests.rs:594`, jig 3 files, temper `service/src/accounts.rs:438`, and the field destructure `jig/testing/jig-test-store-files/src/files.rs:104`: as the plan and README say; nothing else names `root` of a `Rename`. hearth: none.
- Exhaustive matches of the new variants (`Request::ListPage`, `Request::Lock`, `Event::Locked`, `Event::Busy`, `Op::Lock`, `Error::Busy`, `Ask::Lock`, `Summary::Lock`), found by the compiler from `heavy cargo check --workspace --all-targets --keep-going --message-format=json` with the suggested arms applied: skein 14 files in skein-io, shell `ring.rs`, 5 in skein-sim, oauth-accounts 2, `face.rs`, conformance `run.rs`, world `files.rs`/`real.rs`, `echo/service`, `skein-echo-client`, `tests/io/tests/memory.rs`, `tests/ring`: all in 04/05's Files or the 34-file list; smith (both pins) 15 matches in 13 files, jig 9 in 8, temper 13 in 8: as README's Repin says.
- Sketch `Request::Rename`/`ListPage` and the reference step test of 04.1 (`src/tests/directory.rs`, `complete` and `listed` `pub(super)`), B1-skein: clippy with warnings denied is clean but for the stubs' own `match_same_arms`; the test passes (`nextest run -p skein-io -E 'test(a_rename_names_the_directory)'`, 1 passed).
- Sketch `Move` (derives `Hash`) and the crash algorithm of 04.1, B1-skein, `heavy cargo clippy -p skein-fake-machine --all-targets --no-deps -- -D warnings`: E0499, `entries_of(&mut surviving, ..)` twice in one tuple: defect, fixed (one borrow a statement); then only dead-code of the stub.
- Premise "worst_case unchanged" (`file_layer.rs:263-278`): read, not run: a page holds at most `max <= max_entries` entries, within the bound's `(max_entries + 32) * 2` cells and `+ 33` names, plus the one kernel `List` record; the world test `a_page_of_max_entries_stays_within_file_ios_worst_case` is the check: at ready.
- Reference world test of 04.1 (`renames::world`), 04.2 and 04.1's Proof, 04.2's Proof, hearth 32.1/23 callers: at ready.

## skein 05

- `rg -l 'Op::ReadSignal|Error::NotAFile' crates testing tests examples`, B1-skein: 34 files: as the plan and README say.
- Done 1 prints 0 (before); Done 2 `git grep -n -E '^    (Lock|Locked|Busy) \{$|^    Busy,$' origin/main -- crates/skein-io/src/file.rs crates/skein-io/src/kernel.rs`: nothing at the base; the stubs print 5 lines (`file.rs` 165, 293, 297; `kernel.rs` 363, 787): at ready. Done 3 and 4: at ready (`skein-conformance-ring`, `-sim`, `skein-io-world` exist; the scenario's test names are top-level).
- 38 line references of Files (kernel.rs, ring.rs, sim.rs, sim/files.rs, access.rs, trace.rs, machine.rs, face.rs, conformance run.rs/processes.rs, fs.rs:116/:241/:472, file_layer.rs:515/:724/:851, cuts.rs:272): as the plan says.
- Sketches `Op::Lock`, `Error::Busy`, `Ask::Lock`, `Request::Lock`, `Event::Locked`/`Busy`, `Machine::lock` (with `Refusal::Held`, `Handle::locked`, `Refusal::Held => Error::Busy`), B1-skein: `heavy cargo clippy --workspace --all-targets -- -D warnings -A unused-qualifications -A clippy::match-same-arms`: exit 0 but one: `stream.rs`'s `decode` (:648) is at clippy's 100 lines, an arm makes 101 (`too_many_lines`): defect, fixed (05.1's Files names the `#[expect]`).
- Reference scenario `testing/skein-conformance/src/locks.rs` (`Locks`, `Check`, `lock`, `ended`, `Run::lock`, `command` and `spawned` `pub(crate)`): `heavy cargo clippy -p skein-conformance --all-targets --no-deps -- -D warnings`: exit 0; runs at ready.
- Downstream: as 04.
- `flock ~/src/rust/tmp/hearth/render/land-skein.lock ...` (every Rules): exit 66, the directory is not there: `plan start` makes `tmp/hearth/render/` (a run's state): at ready.
- Question 1's `file_layer.rs:724/:851`: as the plan says. Rest of 05.2's Proof and step tests: at ready.

## hearth 01

- Scratch crate `scratch/01` in B1-hearth (member, `[lints] workspace = true`, skein by path): the whole of 01.1 and 01.2's sketches with bodies, and all 22 tests of the Test shapes with store.md 3.2's vectors pasted: `heavy cargo clippy -p hearth-lib-scratch --all-targets -- -D warnings`: exit 0; `heavy cargo nextest run -p hearth-lib-scratch`: exit 0, `22 tests run: 22 passed`: as the plan says (22, every vector reproduced: `8f43cea6`, `22d5156f`, `6022d1fb`, `8291ea66`, `72653b99`, `ab3793df`, 577 bytes at depth 64).
- Proof lines `nextest run -p hearth-lib -E 'test(a_list_of_texts_encodes_as_store_md_writes_it)'` and `test(h_with_staffs_gid_absent_matches_store_md)` (substring form), on the scratch crate: 1 passed each: as the plan says (the form works for `src/tests`).
- Done 1 prints 0 (before); Done 3 `git grep -n -F 'b"hearth/1"' origin/main -- crates ':!*/tests/*' ':!*/tests.rs'`: exit 1 at the base, one line in `hash.rs` in the scratch; Done 4's regex `mod limits|pub fn (hashing|hashed|text)|i64|f64`: parses (`grep -E ... /dev/null` exits 1), nothing in the scratch's `src`; `cargo tree -p hearth-lib-scratch -e normal`: two lines (`hearth-lib-scratch`, `skein-lib`): as the plan says; the git URL form: at ready.
- `git -C ~/src/rust/skein merge-base --is-ancestor 0f093bb f981746`: exit 0: as the plan says. `grep -o 'skein.git#[0-9a-f]*' Cargo.lock | sort -u` and the repin check: at ready (hearth has no `Cargo.lock`).
- References: hearth `Cargo.toml` lines 13-17, `bytes.rs:233` (`finishing_short_is_a_bug`), render.md depth 32: as the plan says.

## Compile pass, planner-9 (2026-10-17)

Every ```rust block of skein 02 to 05 and hearth 01 compared with its scratch copy (text, comments aside), then compiled; scratch worktrees B1-skein (skein f981746) and B1-hearth.

- skein 02 (blocks 1 to 3: `position.rs`, `reference.rs`, tests), B1-skein: in sync. skein 03 (blocks 1 to 4: `digest.rs`, `Host`/`Machine`/`Event`, tests), in sync. skein 05 (`Op::Lock` to `Machine::lock`, `locks.rs`), in sync.
- skein 04 block 2 (`Move`, the crash algorithm of 04.1) was missing from B1-skein: re-added as `testing/skein-fake-machine/src/fs/moves.rs` (`Move`, `entries_of`, `restore`, `Machine::settle_moves` with the loop verbatim), field `pending_moves`, called from `crash`: `heavy cargo clippy -p skein-fake-machine --all-targets --no-deps -- -D warnings`: exit 0. Compile pass, planner-9, 2026-10-17.
- All of skein, B1-skein: `heavy cargo clippy --workspace --all-targets --keep-going -- -D warnings -A unused-qualifications -A clippy::match-same-arms`: exit 0 (the two allows are for the stub arms only, as at the base). Compile pass, planner-9, 2026-10-17.
- hearth 01 (4 blocks), B1-hearth, crate `hearth-lib-scratch` (`scratch/01`): in sync; `heavy cargo clippy --workspace --all-targets -- -D warnings`: exit 0. Compile pass, planner-9, 2026-10-17.
