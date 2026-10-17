# Base run B2 (hearth 02, 34, 03, 07, 38, 08, 41), skein f981746, hearth 175e92f

Machine debian-8gb-hel1-1: git 2.47.3, mmdebstrap 1.5.7, apt 3.0.3 (trixie), nextest 0.9.137.
Scratch: `tmp/plan/hearth-render-scratch/B2-hearth` (crates `scratch/s*`); skein by path `tmp/plan/hearth-render-skein`.
Stubs (no owner sketch): Position/skein 02 (absent at f981746), fixtures, generate, world, Scenario, Run, Recording, Mask.

## 02
- `git -C ~/src/rust/hearth ls-tree -d --name-only origin/main crates/hearth-protocol-git tests/git | wc -l`: exit 0, prints 0; before the session, runs.
- Done when 2 (`log ... | grep -c -x -E 'hearth-02\.[1-3]'`): exit 1, prints 0; runs.
- `echo $((65536*4096))`: 268435456 (256 MiB: the paths alone reach the plan's 256 MiB).
- Real git 2.47.3, scratch repos: rev-parse, `ls-tree -r -l -z --full-tree` (padded 7, `-` submodule), `cat-file --batch` (`<oid> missing`), `a..a` two lines exit 1: all as the plan says.
- `heavy cargo clippy -p scratch-s02 --all-targets -- -D warnings`: first exit 101 (`Iterator::map` disallowed in the listing test); DEFECT, fixed (for loop). Then exit 0.
- `... -p scratch-s02w ...` (world test, stubs `generate`, `world`, `fixtures::SMALL`): exit 0.
- `heavy cargo nextest run -p hearth-git-world -E 'binary(memory)'`: at ready (filter parses with a binary `memory`).
- kernel.rs:523 cited for `Spawn::args`: it is :525; DEFECT, fixed.
## 34
- Done when 1/2/3/5: exit 0/0/1/1, print 0 (nothing), run.
- Profile text of 34.1 written into the scratch `.config/nextest.toml` with 02.2/03.2's two lines: `grep -c -F 'binary(/^conformance_/)'` prints 4; `cargo nextest run -p scratch-filters --profile conformance|fuzzy|measure|default` pass, `Summary` names tests; `-E 'binary(=conformance_resolve)'`, `binary(/^conformance_/)` parse. Verdict as the plan says.
- Real git diff --no-index with the 2.2 environment: `a/./cache/<64 hex>`, no-newline, `Binary files`, missing file exit 1 (stderr only): as the plan says (2.47.3).
- Eleven git cases and the sha256 repository built with plumbing under one identity: works; 7 diff cases and 11 git cases = 18.
- `git status ... | grep -vc '/diff-'`, `ls -d ... | wc -l`, Done when 2 loop: run in a scratch repo, as the plan says; the live runs are at ready.
- Sketches `scratch-s34` (hunks, Limits growth): defect, fixed (`NoNewline` needs backticks, doc_markdown); `scratch-s34w` (conformance test): exit 0.
## 03
- Done when 1 and 3 greps: exit 1, print 0 / nothing; run. `git log origin/main --all-match --grep ... --grep '^Step: skein-02\.'`: prints nothing (skein 02 not landed); `merge-base --is-ancestor`, `grep -o 'skein.git#...' | sort -u`: parse.
- `heavy cargo clippy -p scratch-s03 ...` (all sketches, both unit tests): exit 0. `-p scratch-s03w` (world, memory, fuzzy tests): exit 0.
- 35.2 premise "03's `json::worst_case` covers `declaration`": does not (new boxes while the `Value` is held); DEFECT, fixed in 35.2.
- Filters `binary(fuzzy_documents)` with `--profile measure -j 1 --ignore-default-filter`: parse and pass on a scratch binary.
## 07
- Done when greps: exit 1, print 0 / `fatal: path ... does not exist` (at ready).
- Vector run for real, offline (`unshare --user --map-user=$(id -u) --net`, not a live run): mmdebstrap 1.5.7 accepts the 5.2 vector, exits 25 with `Err:1`/`E: Failed to fetch ... Temporary failure resolving`; matches 07.3's patterns.
- apt 3.0.3 on trixie lists: `Unable to locate package`, `Package '...' has no installation candidate`, `unmet dependencies` / `Unable to correct problems` wording, `Inst` line form: as 07.3 says; `--variant=apt` closure 69 packages (> 50).
- `Acquire::Languages "none"` set by mmdebstrap (two files at the fetch).
- `heavy cargo clippy -p scratch-s07 -p scratch-s07w ...`: exit 0 (limits, vector, output, tail, three tests, world test).
- 38's real-list check changed `Limits::stanza` default: see 38.
## 38
- Done when: exit 1, prints 0; at ready.
- Real trixie `Packages` (149,228 stanzas, local apt lists): six fields once each on one line, Version/Package/Filename/SHA256 forms hold, 0 violations; but the longest stanza is 76,338 bytes (`librust-winapi-dev`, one `Provides` line of 75,649): DEFECT, 64 KiB refuses a real list. Fixed: 07 `SHIPPED.stanza` 1 MiB, 38.1 tests, `[Q?]` added. Apt keeps lists uncompressed here.
- `heavy cargo clippy -p scratch-s38 -p scratch-s38w ...`: exit 0.
## 08
- Done when 1-4: exit 0/1/0/1, print 0, error `path does not exist`, empty, 0; run in a scratch repo with a pin and trailers: 4/2/1..3 sums as the plan says.
- sha256 of `signed\n` and `xz\n` equal the fixture constants. `sqv --keyring ... --cleartext --output`, `awk` entries from a signed InRelease, `xz`, `curl`, `sh` (dash): work. `unshare --map-root-user --net true` exit 0.
- Not run: `testing/mirror/fetch` and any curl to snapshot.debian.org (the plan runs them at 08.2, not at the base).
- `heavy cargo clippy -p scratch-s08 ...` (kit, all four tests): first exit 101: `missing_errors_doc` on the kit's `Result` fns; DEFECT, fixed in Choices (`missing_errors_doc = "allow"`). Then exit 0.
## 41
- Done when 1, 2, 4: exit 0/1/1, print 0; run. `git grep -n -F 'Recording {' -- testing tests`: exit 1, nothing.
- Trixie lists: test-boot's ten names, both MTAs (providing and conflicting), `mail-transport-agent` virtual only, no `hearth-absent`: as the plan says.
- `heavy cargo clippy -p scratch-s41 ...` (resolve.rs, run.rs, kit fields, test): exit 0.
- `HEARTH_RECORD=1 heavy cargo nextest run -p hearth-conformance-world --profile conformance -E 'binary(=conformance_resolve)'`: at ready.

## Compile pass, planner-9 (2026-10-17)

Every sketch block of the seven session files compared line by line with its
scratch crate and re-synced (scratch worktree `B2-hearth`); then `heavy cargo clippy
--workspace --exclude scratch-filters --all-targets -- -D warnings` in it: exit 0.
- 02: `scratch-s02`, `scratch-s02w` (and `scratch-s34`, which carries 02's blocks too): `Command::Diff` doc now says `--git-dir=/dev/null`; exit 0.
- 34: `scratch-s34`, `scratch-s34w`: `Kind` doc cites render.md 6.1, 8.2; exit 0.
- 03: `scratch-s03`, `scratch-s03w`: `Fault::Shape`, `Shape::Missing` comments as now written; exit 0.
- 07: `scratch-s07`, `scratch-s07w`: `SHIPPED.stanza` 1_048_576 (the base fix, missing in the crate), comments of Limits, Variant, Seen, Cause, Kind; test `a_failed_fetch_keeps_its_err_and_e_lines_and_is_operational` re-pasted (Could not connect, no longer 404); exit 0.
- 38: `scratch-s38`, `scratch-s38w`: unchanged, as the blocks say; exit 0.
- 08: `scratch-s08`: `Pin::digest` doc cites testing.md 6.2; exit 0.
- 41: `scratch-s41`: `Resolve` doc cites testing.md 4.5; stubs `Recording`, `Mask::new`, `Run` agree with 34's Shape (hearth_recordings, line 99-102); exit 0.
No defect of the plan found in this pass.
