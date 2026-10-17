# base-B4: hearth 09, 39, 10, 40, 11, 32 at the bases

skein f981746 by path (`~/src/rust/tmp/plan/hearth-render-skein`, no skein change), hearth 175e92f, scratch
worktree `~/src/rust/tmp/plan/hearth-render-scratch/B4-hearth` (members `scratch/*`, `libc = "0.2.186"` in
`[workspace.dependencies]`; the one `heavy cargo fetch` resolved libc 0.2.191). `H` is `git -C ~/src/rust/hearth`.
Every sketch compiled with `heavy cargo clippy -p <crate> --all-targets -- -D warnings`; final run of all nine
crates in one command: exit 0, `Finished dev profile`.

Scratch crates: `h01` (stub of hearth 01's `Digest`, `Item`), `s09` (09 and 39.1's two functions), `w39` (39's world
stub and three reference tests), `s10` (10 and its four reference tests), `s40` (10 grown by 40, two reference tests),
`s11` (11 and 32.1's render, builder, protocol, service sketches), `s11h` (`crates/hearth`: Client, umask, argv; lints
and clippy.toml of skein-shell), `w11` (11.1, 11.3, 32.2 reference tests). Stubs: hearth 01 `Digest`, `Item`,
`Encoder`; 02 `git::Limits`, `listing::Entry`; 03 `config::Limits`, `document::Document`; the worlds' `Rng`,
`generate`, `check`, `fixtures`, `parent`, `Repository`; `render::Limits` and `builder::Limits` (no sketch gives
them: defect, fixed); the private `Open`, `Lines`.

## hearth 09
- `H log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | grep -c -x -E 'hearth-09\.[1-3]'` (Done 1), exit 1, prints `0`: as the plan says before the session.
- `H grep -n -F 'hearth\0r' origin/main -- crates` (Done 3), exit 1, nothing: parses; at ready for the one line.
- `heavy cargo nextest list -p scratch-s09 -E 'test(a_framed_body_has_the_header_store_md_gives_byte_for_byte)'`, exit 0, lists `tests::frame::a_framed_body...`: the Proof's filter form works.
- `heavy cargo tree -p scratch-s09 -e normal --depth 1`, exit 0, prints the crate, hearth-lib, skein-lib: Done 3's three lines (shape only); the nextest counts (Done 2, 28 tests) counted by name: 8 + 10 + 10 = 28, as the plan says; at ready.
- Sketch limits.rs, frame.rs and decode.rs, store/cache_file/resolution: compiled (`s09`). Defect, fixed: the sketch doc "its line's InRelease first" fails `doc_markdown`; now in backticks.
- Reference tests 09.1 and 09.2 compiled. Defect, fixed: 09.1's assert message `map { \"a\": uint 1 }` is an invalid format string; now `{{ ... }}`. By hand: 09.1's frame header 28 = 0x1c and 09.2's store body 76 = 0x4c bytes are right; 09.2's cache-file body is 224 bytes (computed), key order at, origin, sha256, size, took; 0x186cc6acd4b00000 is 1_760_000_000_000_000_000.

## hearth 39
- Done 1 command as 09's: exit 1, `0`.
- Sketch `worst_case`, `held`: compiled in `s09`'s `limits`. Reference tests (`no_cut_of_a_generated_frame_reads_as_a_record`, the memory test, the fuzzy one) compiled in `w39` under the world lints and skein's `tests/clippy.toml`, with skein-heap.
- skein `testing/skein-heap/src/lib.rs:57` (`Counting`) and `:341` (`Meter`) at f981746 and origin/main: as cited. jig `tests/client/tests/fuzzy_joint.rs` at origin/main: the three consts are lines 5 to 7, not 5 to 10; defect, fixed.
- Done 2 counts (9 focused, 2 fuzzy) counted by name: as the plan says; at ready.

## hearth 10
- Done 1: exit 1, `0`. Done 3 `H grep -n -E 'CLIENT_VERSION: u32 = 1' origin/main -- crates`: exit 1, nothing: parses.
- Done 2 (20 tests) counted: 8 + 9 + 3 = 20, as the plan says.
- Three Proof commands `test(=<name>)` on unit tests: `heavy cargo nextest run -p scratch-s10 -E 'test(=a_spec_prints_its_meta_and_config_as_render_md_shows)'` exit 4, `no tests to run` (name is `tests::text::...`). Defect, fixed: `test(=tests::text::...)`, `tests::printer::...`, `tests::text::...`.
- Sketches (lib, limits, records, parts, outcome, form, printer, 10.3's): compiled (`s10`) and four reference tests. Defect, fixed: once 10.3 adds `resolution`, `Record::Spec(Spec)` is a 336-byte variant against 1 (clippy `large_enum_variant`, as does `printer::Request`; `Option<Box<Resolution>>` still 208 > 200). Now `Spec(Box<Spec>)` and `x1_prod() -> Box<Spec>` (the references unchanged). Users: `20-exports.md:176` writes `Record::Spec(spec)` (not my file; reported).
- skein `crates/skein-http/src/sse/writer.rs` and its tests' `Machine` at `src/tests/writer.rs:22`: as cited. `skein_json::writer::Encoder`, `Limits { depth, length }`: exist.

## hearth 40
- Done 1: exit 1, `0`. Done 2 (42 tests) counted: 20 + 11 + 11: as the plan says.
- Two Proof commands with `test(=<name>)` on unit tests: same defect as 10, fixed (`tests::diff::...`, `tests::errors::...`).
- Sketches (diff, errors, startup, outcome::of, `Record`, `Terminal`, `Type` grown): compiled on 10's (`s40`), `Record` with all five variants raises no `large_enum_variant`; references 40.1 and 40.2 compiled (with truncated expected text).

## hearth 11
- Done 1: exit 1, `0`. Done 3 `H ls-tree --name-only origin/main crates/hearth-protocol-builder/src/translate/`: exit 0, nothing: as before the session. Done 4 `H grep -n -E -e '\.\.\s*\}' -e '^\s*_ =>' origin/main -- crates/hearth-protocol-builder/src/translate`: exit 1, nothing; `grep -E -e ... /dev/null` exit 1: parses. Done 2 (13 tests) counted 7 + 6: as the plan says.
- Proof of 11.2 `test(=a_documents_blob...)` is a unit test: defect, fixed `test(=tests::read::a_documents_blob...)`. 11.1's and 11.3's are integration tests: bare name kept (`cargo nextest list` shows `a_targets_meta...` and `hearth_render_prints...`).
- Sketches compiled (`s11`, `s11h`) and the references of 11.1 and 11.3 (`w11`). Defects, fixed: `render::Limits` and `builder::Limits` (used by `step`, `Service`'s `Limits`) were sketched nowhere: two lines added; the four `translate` functions lack `#[must_use]` (`must_use_candidate`): added.
- Digests recomputed independently (Python, store.md 3.1 and 3.2): meta `706777e0...b73d`, spec `d5942fa1a047...118c`, both as the plan says. Cites at origin/main: skein `examples/echo/service/src/lib.rs:316`, `crates/skein-shell/src/drive.rs:53`, the echo shell and `tests/echo/tests/real.rs`, smith `git_child.rs`: as cited.
- Size: the file passed 28,000 bytes with the additions; trimmed prose (Read first, What the design asks, Rules, the opening) to 412 lines, 27,888 bytes.

## hearth 32
- Done 1: exit 1, `0`. Done 3 `ls-tree`: exit 0, nothing. Done 4 `H grep -c 'var_os' origin/main -- crates/hearth/src`: exit 1, nothing (the base). Done 2 (29 tests) counted 7 + 3 + 7 + 9 + 1 + 2 = 29: as the plan says.
- Digests recomputed: config `e2475ba7ef36cd9f...c1fd2`, spec `9970fb4d8f97...56d6`, the two files' SHA-256 `775324fc...3123` and `f487088e...e745`: as the plan says (field name `content_digest`, tree only).
- Proofs: `test(=hearth_render_prints...)` (integration) and `test(=end_to_end_render)`: bare names list in nextest; `test(/^end_to_end_/)` lists the one test.
- `libc::umask` sketch compiled (`s11h`, libc 0.2.191 from `libc = "0.2.186"`) under skein-shell's lints. Defects, fixed: (a) `#[allow(unsafe_code)]` fails `allow_attributes` and `allow_attributes_without_reason` (checked): it is `#[expect(unsafe_code, reason = ...)]`, in 32.1 and the README; the binary's `[lints]` were unstated: now skein-shell's, `unsafe_code = "deny"` (a `forbid` could not carry it); (b) `pub fn umask(mask: u16) -> u16` fails `must_use_candidate`, and `#[must_use]` would force `Client::open` to drop a Copy value (`dropping_copy_types`): the old mask is dropped, `pub fn umask(mask: u16)`.
- Sketch 32.1/32.2 vocabulary, `Files`, `Roots`, `Protocol::new(.., cache, seed)`, `form`, `invocation`, `Client::open(.., seed)`: compiled in `s11`/`s11h`; `end_to_end_render` reference compiled (`w11`, a `hearth` bin).
- Cites: smith `crates/smith-agent-service/src/service.rs:968` (the route by `Operation::{Io, Signals, File}`) and temper `crates/temper/tests/support/mod.rs` `serve` :85, `StdoutReader` :119, `until_link_then` :338: as cited at origin/main. skein `file::Request::Rename { owner, root, from, to }` at f981746: as 32.1 says for the base.

## Compile pass, planner-9 (2026-10-17)

Blocks of 09 (5), 39 (4), 10 (5), 40 (3), 11 (3), 32 (2) compared with the scratch crates (B4-hearth), then
`heavy cargo clippy --workspace --all-targets -- -D warnings`: exit 0 (8 crates: h01, s09, s10, s11, s11h, s40, w11, w39).
Re-synced: s10 (10.2's json test: the `meta` and `config` lines, verbatim; two doc comments), s40 (40.1's diff test:
its three hunk lines; the `Unopened` doc), w11 (32.2's `end_to_end_render`: `HOME` is `repository.scratch()`).
09, 39, 11 and 32 already as the blocks say (11's and 32's sketches share crates s11 and s11h, 32's grown forms).
Compile pass, planner-9, 2026-10-17.
