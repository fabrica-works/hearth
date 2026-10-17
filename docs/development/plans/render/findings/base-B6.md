# B6: hearth 17 to 22 at the bases (skein f981746, hearth 175e92f)

Scratch: `~/src/rust/tmp/plan/hearth-render-scratch/B6-hearth` (members `scratch/s17` to `s22`, skein-lib by path on `~/src/rust/tmp/plan/hearth-render-skein`). Clippy for each: `heavy cargo clippy -p scratch-sNN --all-targets -- -D warnings`.

## Done-when commands (run exactly, from any directory; hearth `origin/main` = 175e92f)
Each session's item 1 (`git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' ... | grep -c -x -E 'hearth-NN\.[1-n]'`): exit 0 (grep -c), prints `0`; the check of landed state, runs, regex parses: as the plan says before the session (prints 5/5/5/4/4/4 after).
- 17: item 3 (`ls-tree ... parts/ | grep -c`): exit 0, `0`; item 4 (`grep -c -E 'pub fn inputs\(|debian: _'`): exit 1, nothing; item 5 (Vec/String/`_ =>`): exit 1, nothing. Before the session: as the plan says.
- 18: item 3 (`ls-tree -r ... | grep -c`): `0`; items 4, 5: exit 1, nothing.
- 19: item 3: `0`; item 4 (`grep -l a_software_script_is_checked_not_digested`): exit 1, nothing; item 5: exit 1, nothing.
- 20: item 3: `0`; item 4 (`... | wc -l`): `0`; item 5 (`const DEPTH: u32 = 64;`): `0` (the base has no crate; prints 1 after); item 6: exit 1, nothing.
- 21: item 3: `0`; item 4 (`grep -l -F tasks/hearth/task.json ... | wc -l`): `0`; item 5: exit 1, nothing.
- 22: items 1, 3, 4, 5, 6: `0`, `0`, exit 1 nothing, exit 1 nothing, exit 1 nothing. Items 2, 3, 4 were wrapped across lines inside their backticks: defect, fixed (each on one line).
- Each regex of those items: `grep -E '<pattern>' /dev/null` exits 1 (parses): `\bVec\b|\bString\b|^\s*_ =>`, `minijinja|legacy|workaround`, `pub fn inputs\(|debian: _`, `exports: _`, `const DEPTH: u32 = 64;`, `left_out: 0`.
- Item 2 of every session (`heavy cargo nextest run -p hearth-domain-render ...`): at ready (no crate at the base); crate names match code-organization.md.
- 17 ripple `git grep -n -w -e THIN_TEXT -e d5942fa1a047 -e 9970fb4d8f97 -- tests crates` and 17.1 `git grep -n -F fleet.json -- tests crates`, in B6-hearth: exit 1, nothing (no tests or crates at the base); both were wrapped across lines: defect, fixed.
- `test(=` filters in 17 to 22: none.

## Digests (printf ... | sha256sum, and a Python store.md 3.1 encoder for the parts)
- 17.1 `f745f2a642a2` = sha256 of `forky amd64 20261001T030405Z contrib,main,non-free-firmware` LF: as the plan says.
- 17.2 `de38a7ed6276e7c9...fe35d` (packages part of THIN), `51cab42f8d2b` (PACKAGES), `cf667f7c3bf8` (THIN spec after 17.2); 17.3 `552be03a9eec` (PRESEEDED); 17.4 `bc6cab417252...a077` (THIN boot), `68ef6aa14d25` (CMDLINE boot), `dd576929cc4d` (THIN spec after 17.4): all as the plan says, with the record `{apt, initramfs: {microcode: "generic", modules: []}, preseeds: {}, request, sources, variant}` for packages (a preseed record `{owner, type, value}` by question), `{mutable, params}` for boot; reproduces 9970fb4d8f97 and e2475ba7ef36 of 32.1. Not recomputable (the scripted `x1_prod` world's records are not in the file): `db6a7238154a`, `a46334dcd355`; the design's `4be1c0d2a7f1`, `0c7d9e41c0d2` (render.md 9.2 text).
- 18: `775324fcea7f`, `f487088ec8b1` (THIN's files), `b0b0dd8b10a8`, `e55838f39d99` (`Hello dev|ops on x1-prod.` LF), `4cb3bd448645` (`lab certificate` LF), `b40d08aa8c5c` (`update-ca-certificates` LF), `4b1f03482366` (that plus `systemctl daemon-reload` LF): as the plan says.
- 19: `33165223a0a3...b1c9` = `printf ripgrep | sha256sum`, `1632d8753eb0` (`rg --version` LF), `d2571cab3997...8bde` (`ripgrep 15.1.1` LF): as the plan says; forgejo's `7b36e8a38247...47ed` has no input in the file (a fixture's): not recomputable.
- 20: `01ba4719c80b` (LF), `9e40aaeae922` (`mail-lab=container@rhi-lab;` LF), `755c9769e418` (`chat-prod=container@rhi-prod;mail-prod=container@rhi-prod;` LF), `afd16377136a`, `0ed834effddb`: as the plan says.
- 21: `ca2dff16...fc7fab9` = `hearth 0.1.0` LF: as the plan says.
- 22: `474a6f5bdf9c` (`ci:100000:65536` LF `free:165536:65536` LF), `6cb7c9647dfa...caa8` (the four lines of `tiny-initramfs.conf`, 70 bytes): as the plan says.

## Positions (python, 1-based columns of the key or value the plan names)
17.2 `1:60` (`"component"`), 17.4 `1:41` (`"order"`), 19.1 `1:14`, 19.2 `1:220`, 19.4 `1:71`, 20.2 `2:53` (`"{{` string of line 2, which starts with a space), 20.3 `1:92` (the `7`), 21.1 `1:23` twice and `1:1040`, 2141 bytes (139 + 1001 + 1001), 21.2 `1:12` and `1:20`, 21.3 `1:19` and `1:18`, 21.4 `1:56`, 22.1 `1:14` / `1:2`, 22.3 `1:12` and `1:11`, 22.4 `1:12`: all as the plan says. The `52`-column diff rows of 18 and 22.2: all at 52. Counts: 21's `Rule` has 30 variants (third round: `TmpfilesPath`, `ContainerModule`, `ContainerParam`; s21 re-run with `entries(.., container: bool)`: clippy -D warnings exit 0); 22.1's 300 - 256 = 44.

## Sketches (one scratch crate each, clippy -D warnings, exit 0 after the fixes)
- s17 (17 Interface sketch + the two Test-shape references): defect, fixed: `translate::packages` and `boot` lacked `#[must_use]` (must_use_candidate). Stubs: Digest, Item, Value, Position, Place, Error, Expansion, Concrete, Schema, Target, Documents, Merge, Composed, Placement, Pair<T>, Layer, Limits, fixtures.
- s18 (18 sketch): defect, fixed: `files::dropped` lacked `#[must_use]`. Stubs: Digest, Oid, Position, Fault, Declaration, Template, Limits, EntryChange, TreeEntry, Located, Context, Operand.
- s19 (19 sketch + references of 19.1, 19.2, 19.4): defect, fixed: `translate::secrets` lacked `#[must_use]`. Stubs: Digest, Expansion, Error, Position, Accounts, ScriptsChange, Script (18.4's), Keyed (18's), Map, Name, Kind, fixtures (needs a `type Row` alias: type_complexity).
- s20 third round (`containers` returns `Result`, `evaluate`/`conform` take `Against`/`Check`, `Cause` widened, `build` returns `Exports`): `heavy cargo clippy -p scratch-s20 --all-targets -- -D warnings` exit 0.
- s20 (20 sketch + the 20.2 reference): clean (body of `exports::build` consumes its box: boxed_local). Stubs: Digest, Position, Fault, Declaration, Limits, Target, Documents, Meta, Text, Interpolated, Entry (14's), Value (14's), Schema, Item, FaultKind, Root, Revision.
- s21 (21 sketch): defect, fixed: `reserved::tree` and `live` lacked `#[must_use]`. Stubs: Map, Position, Expansion.
- s22 (22 sketch + the 22.1 and 22.2 references): clean (`worst_case`, `new`, `finish` have `#[must_use]`). Stubs: Limits, Error, Accounts, Tmpfile, Initramfs, Expansion, Plan, Preset, Preseed, Parameter, DataEntry.
- Not compiled (they need fixtures only their sessions define): the references of 17.x/19/20/21/22 that call `fixtures::reserved`, `Repository::make`, `form::print`, and the real tests.

## Users and premises
Nothing in 17 to 22 changes a thing that exists at the bases (no skein change; hearth has no crate). skein cites at f981746: `map.rs` `get`:68 `insert`:88 `remove`:103 `last`:119 `iter`:124 `Map`:24, `list.rs` `push`:55, `stack.rs`, `sha256.rs`: as cited. Also fixed: 20.2's reference `Record::Spec(spec)` is `Record::Spec(Box::new(spec))` (10.3 boxes `Spec`), with `*fixtures::x1_prod()`.

## Left for the planner
18.4 repeats what 04.1 decodes (`scripts`: only `customize.sh` and `software.sh`, `order` 0 to 999, else `Order` at the value): its `a_third_script_name_is_an_enum_fault` and `script_order_defaults_to_500_and_ranges_over_0_to_999` test a refusal 04.1 makes first; 13's `Map` has no `scripts` (read from `Declaration::scripts`). Not changed.

## Compile pass, planner-9 (2026-10-17, CP-C)

scratch-s17..s22 re-synced with 17.x-22.x blocks verbatim (new test modules: s19 integ, s20 integ, s21 unit and integ, s22 tests incl. 22.1-22.3). `heavy cargo clippy -p <crate> --all-targets -- -D warnings`: exit 0 for each of the six. One plan defect: 20.3's test used banned `.concat()` and `.any(closure)`; fixed in 20-exports.md (fixtures::has_run).
