# base-B3: hearth 04, 35, 05, 37, 06, 36 at the bases

skein `f981746` (by path `~/src/rust/tmp/plan/hearth-render-skein`, no skein
change), hearth `175e92f` (scratch worktree
`~/src/rust/tmp/plan/hearth-render-scratch/B3-hearth`, root `Cargo.toml`
members `scratch/*`), MiniJinja 2.20.0 from the cargo registry. Machine
debian-8gb-hel1-1. `H` is `git -C ~/src/rust/hearth`.

Scratch crates (all `[lints] workspace = true` for `s*`; the `w*` worlds take
skein's `tests/clippy.toml` and `tests/json/Cargo.toml`'s lints):
`scratch/s04`, `s35`, `s05`, `s37` (+ `skein-lib`), `s06`, `s36`; worlds
`w04`, `w35`, `w05`, `w37` (+ `skein-heap` at skein `testing/skein-heap`).
Each compiled with `heavy cargo clippy -p <crate> --all-targets -- -D warnings`
from the scratch worktree root. Final run, all ten crates in one command:
exit 0, `Finished dev profile`.

Stubs (names another session defines, copied from its sketch or minimal):
hearth 03's `Limits`, `Value`, `Kind`, `Entry`, `Error`, `Fault` and skein
02's `Position` (04, 35); the worlds' `Rng`, `Settings`, `Tally`, `generate`,
`world`, `examples`, `fixtures`, `reference`, `check` (`unreachable!("sketch")`);
05's `Error` in the 06 crate; the fixtures `compile`, `op`, `bytes`, `name`,
`key`, `expr`, `text`, `output`, `branch`, `jump`; 37.2's `Pending`, `push`,
`rotate_left` around the rotation snippet. Derives: `Clone, PartialEq, Eq,
Hash, Debug`, `Copy` on the Copy-able small types, as each Interface says.

## hearth 04

- `H log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | grep -c -x -E 'hearth-04\.[1-2]'` (Done 1), exit 1, prints `0`: as the plan says before the session; the format and regex parse (same command on skein's `Plan: forge-setup` prints `skein-01.3`).
- `H grep -n hearth-protocol-template origin/main -- crates/hearth-protocol-config/Cargo.toml` (Done 3), exit 1, prints nothing: as the plan says.
- `heavy cargo nextest run -p hearth-protocol-config -p hearth-config-world` (Done 2): at ready.
- Sketches: `scratch/s04` (Interface block; tests `a_guard_hands_up...`, `an_entry_hands_up...`) and `scratch/w04` (`the_reads_example_decodes...`), clippy `-D warnings`: exit 0. Verdict: compiles.
- Positions in the three tests, computed by script from the test's text and from format.md 6.4 (dedented): all 6 + 4 + 7 values equal; 04.2's `checks` at(2,1,3) and `artifact` at(38,4,7) from format.md 11 and 10.2 (dedented 2 columns): both equal.
- Counts by `rg`: 5.1 has 22 keys, the Map enum 14 maps, the Name enum covers every field of the 04.2 table; the example names add up to 23 (04: 3 + 14; 35: 4 + 2), as both sessions say.
- skein cites `list.rs` :21, :55, :104: as cited.

## hearth 35

- Done 1, as 04's with `hearth-35\.[1-2]`: exit 1, prints `0`: as the plan says.
- `H grep -n -w Undecoded origin/main -- crates/hearth-protocol-config/src` (Done 3), exit 1, nothing; `H grep -n hearth-protocol-template origin/main -- crates/hearth-protocol-config/Cargo.toml`, exit 1, nothing: as the plan says.
- `H grep -n 'settings: 512' origin/main -- crates/hearth-protocol-config/src/limits.rs` (Done 4): exit 1, nothing before the session; `grep -c 'settings: 512'` on the sketch's `Limits` block prints 1: at ready (one line).
- `git grep -n -e 'Limits {' -- crates tests` (35.1 Files, in the scratch worktree): exit 1, nothing at the base (no crate): the users are 03's, at ready.
- `heavy cargo nextest run -p hearth-protocol-config -p hearth-config-world`: at ready.
- `heavy cargo nextest list -p scratch-w35 --profile fuzzy -E 'binary(=fuzzy_declarations)'`: exit 0, prints nothing at the base (`default-filter = "none()"`); with the two switched filters in the scratch worktree's `.config/nextest.toml`, prints `scratch-w35::fuzzy_declarations generated_declarations_broken_once_fail_at_the_break` (1 test). Verdict: the filter parses and selects one test; the Summary of 1 at ready.
- Sketches: `scratch/s35` (Interface merged with 04's, one test `nested_types_are_laid_out...`) and `scratch/w35` (`generated_declarations_broken_once...`), clippy: exit 0.
- Positions of `PARAMS` (9 values) by script: all equal. Types in pre-order (map, object, int, string) agree with the indices 0..3.

## hearth 05

- Done 1 (`hearth-05\.[1-2]`): exit 1, prints `0`: as before the session.
- `H ls-tree -d --name-only origin/main crates/hearth-protocol-template tests/template | wc -l` (Done 2): exit 0, prints `0` before the session; at ready 2.
- `H grep -n -i minijinja origin/main -- crates/hearth-protocol-template tests/template` (Done 4): exit 1, nothing: as the plan says.
- `heavy cargo nextest run -p hearth-protocol-template -p hearth-template-world` and `... -p hearth-template-world --profile fuzzy` (Done 3, both Proofs): at ready. The `--profile fuzzy` profile exists in `.config/nextest.toml`; `cargo nextest list -p scratch-w05 --profile fuzzy` with the switched filters lists the sweep test.
- Sketches: `scratch/s05` (Interface, `a_string_literal_decodes_each_of_the_five_escapes`) and `scratch/w05` (`cut_and_mutated_texts...`): first run failed, **defect**: `Filter::named` and `Filter::arity` lack `#[must_use]` (clippy `must_use_candidate`); fixed in 05 (as hearth 01's sketches write theirs); then exit 0.
- Offsets of the examples by hand against their texts (`[1, 'x', []]`, `range(3)` at 5, `u.keys()` at 6, `a[1:2]` at 3): as the plan says. 17 filters in 7.5 and 17 in `Filter`, in order; arities 1, 1, 1, 2 as 7.5.
- skein cites (`stack.rs`, `list.rs` :21 :55 :72 :82 :104, tests/json `reference.rs:25`, `generate.rs:229`, `fuzzy_documents.rs:142`, `memory.rs:134`, `Cargo.toml:17-33`): all as cited.

## hearth 37

- Done 1 (`hearth-37\.[1-3]`): exit 1, prints `0`: as before the session. Done 3 as 05's Done 4: nothing.
- `heavy cargo nextest run -p hearth-template-world -E 'binary(memory)'`: `cargo nextest list -p scratch-w37 -E 'binary(memory)'` exit 0 prints `scratch-w37::memory compiling_at_the_limits_holds_no_more_than_the_worst_case`; the run at ready.
- Sketches: `scratch/s37` (Interface, rotation snippet in a stub function around skein-lib's real `List` and `Stack`, test `a_chain_into_is_defined_gives_undefined_at_each_link`) and `scratch/w37` (the memory test against skein's real `skein-heap`: `Counting`, `Meter::{new, start, end, check}` match). First run failed, **defects**: `worst_case` lacks `#[must_use]`; the rotation snippet's `mid - start` ("checked in the code") and `a + 1` trip `arithmetic_side_effects`; fixed with `checked_sub(..).expect(..)` and `checked_add(1).expect(..)`; then exit 0.
- `skein-heap` is in skein at `testing/skein-heap` (a workspace member found by name): as 37.3 says.
- Example offsets (`peer.endpoint is defined` Defined at 14; `x if c else y` c at 5, JumpIfFalse over 2 at 2, Jump over 1 at 7): as the plan says. The rotation (`[a][c][J]` to `[c][J][a]`, `over = len(a) + 1`) agrees with the layout table.
- Premise (37.3): the worst case counts the returned expression and the test drops it after `check`, as `Meter::check` documents ("what it handed out" is what was freed before the check): consistent; the margin computes at ready.

## hearth 06

- Done 1 (`hearth-06\.[1-2]`): exit 1, prints `0`: as before the session.
- `H grep -n 'pub fn compile' origin/main -- crates/hearth-protocol-template/src ':!*/tests/*' ':!*/tests.rs' | wc -l` (Done 3, as fixed): exit 0, prints `0` before the session; the sketches hold 2 (`grep -rn 'pub fn compile'` over `scratch/s05` and `s06`). **Defect**: as written (no exclusion) a `pub fn compile` in `src/tests/fixtures.rs` (05.1's fixture `compile`) would make the count 3; fixed with the exclusions code-organization.md 6 already uses; the pathspec runs.
- `H grep -n 'statements: 32' origin/main -- crates/hearth-protocol-template/src/limits.rs`: exit 1, nothing before; the sketch has it on one line (`grep -c` prints 1): at ready.
- Done 4: nothing; nextest commands (Done 2, both Proofs, `-E 'test(/templates/)'` parses: `cargo nextest list -p scratch-w05 --profile fuzzy -E 'test(/templates/)'` exit 0): at ready.
- Sketches: `scratch/s06` (Interface; tests `an_output_compiles...`, `elif_and_else_...`): first run needed `#[must_use]` on `place`, **defect**, fixed; then exit 0 (05's `Error` stubbed).
- Offsets by hand: `Port {{ settings.port }}\n` (name 8, key 17, text 24, lines [0, 25]); the elif example (branch 0/11, name 6/19, text 10/23/34, jump 11/24; overs 2, 4, 2, 1); `{% for a, b, c` comma at 11; `{{ a %}` at 5; `{% if a }}` at 8: all as the plan says.
- MiniJinja source cites (`lexer.rs` :611, :662, :771): `grep -n 'fn tokenize_root\|...'` prints exactly those lines; the glob path matches one file. skein cites (`bytes.rs` :12, :40, :83; `stack.rs` :48, :57, :69; `utf8.rs:21` `pub(crate) fn next`): as cited.

## hearth 36

- Done 1 (`hearth-36\.[1-3]`): exit 1, prints `0`. Done 3 (as fixed, expecting 3): 0 before; the sketches hold 3 (`s05`, `s06`, `s36`). Done 4: nothing.
- MiniJinja 2.20.0 (`/tmp/b3/mj`, outside any repository, `heavy cargo run --offline`; trim_blocks, lstrip_blocks, keep_trailing_newline, strict undefined): 21 inputs, exit 0. The 36.1 example renders `"a\n  b\nc\n"` (the instructions' texts `a\n`, `  b\n`, `c\n`). Rules confirmed: `-` strips U+00A0, U+3000, VT and `\n`; trim_blocks drops one `\n` after `%}` and `#}`, never after `}}`; lstrip_blocks strips a run (U+00A0, U+3000, FF, VT included) only at the start of a line "as written" (after a trimmed `\n` too), never before `{{`, never after text on the line; `{%+` and `+%}` keep both; `+` on `{{` and `}}` changes nothing; raw drops its body's first `\n`, lstrips before an unmarked `endraw` only when what remains is empty or ends in `\n`, strips both ends under `-`. Test names covered: the rule behind each of `a_statement_alone_on_its_line_leaves_nothing_behind`, `trim_blocks_removes_...`, `lstrip_blocks_removes_...`, `a_minus_marker_strips_all_...`, `a_plus_marker_keeps_...`, `trailing_newlines_are_kept_as_written`, `raw_outputs_its_body_...`. The Rust `char::is_whitespace` set has 25 code points (9, a, b, c, d, 20, 85, a0, 1680, 2000 to 200a, 2028, 2029, 202f, 205f, 3000), as `the_table_holds_the_25_white_space_code_points` says. Cites `lexer.rs` :220, :229, :254, :583, :611, :662, :727, :771: as cited.
- Sketches: `scratch/s36` (Interface; tests `a_statement_alone_...`, `text_and_outputs_alternate_in_order`): exit 0.
- Offsets: `a\n  {% if x %}\n  b\n  {% endif %}\nc\n` (branch 4, name 10, text 15, 33) and `/home/{{ key }}/.ssh` (output 6, name 9, text 15): as the plan says.
- `heavy cargo nextest run ...` commands (Done 2, Proofs): at ready.

## At ready (listed once)

hearth 04: Done 2. hearth 35: Done 2 (two commands). hearth 05: Done 3 (two), Proofs 05.1, 05.2. hearth 37: Done 2, Proofs 37.1 to 37.3. hearth 06: Done 2, Proofs 06.1, 06.2. hearth 36: Done 2, Proofs 36.1 to 36.3. Premise at ready: 35.2's "03's `json::worst_case` covers `declaration`'s memory" (no code at the base; the declaration moves the value's strings but allocates new boxes while the value is still held: 03's memory test should call `declare`).

## Compile pass, planner-9 (2026-10-17)

Every sketch block of the six session files compared line by line with its
scratch crate and re-synced (scratch worktree `B3-hearth`); then `heavy cargo clippy
--workspace --all-targets -- -D warnings` in it (all ten crates): exit 0.
- 04: `scratch-s04`, `scratch-w04`: `Literal::{Text, Plain}` comments; test `an_entry_hands_up_its_key_loop_condition_and_fields_as_text` re-pasted; exit 0.
- 35: `scratch-s35`, `scratch-w35`: blocks agree (04's names merged as stubs); exit 0.
- 05: `scratch-s05`, `scratch-w05`: `Limits`, `Op::Name` doc comments (render.md 2.3, format.md 7.6); exit 0.
- 37: `scratch-s37`, `scratch-w37`: test `a_chain_into_is_defined...` re-pasted (first op `missing: Missing::Error`); exit 0.
- 06: `scratch-s06`: doc comments (render.md 2.3, format.md 7.3); exit 0.
- 36: `scratch-s36`: `ErrorKind::Interpolation` doc as now written; exit 0.
No defect of the plan found in this pass.
