# Base run B8b (hearth 26, 44, 27, 46, 28, 29, 30, 47), skein f981746, hearth 175e92f

Machine: git 2.47.3 (the plan's literals are git 2.53.0 on x1). Scratch: `tmp/plan/hearth-render-scratch/B8-hearth` (crates `scratch/{26,44,27,27r,46,28a,28w,29,30}`), `B8-skein`.

## Sketches (compile pass, planner-9, 2026-10-17)
- hearth 26 (`scratch/26`), 44 (`scratch/44`), 27 (`scratch/27`, `27r`), 46 (`scratch/46`), 28 (`scratch/28a`, `28w`), 29 (`scratch/29`), 30 (`scratch/30`): `heavy cargo clippy --workspace --all-targets -- -D warnings` in B8-hearth: exit 0, `Finished dev profile`; compile pass, planner-9, 2026-10-17.
- Defects found by it, fixed in the session files: 44.1 `tree_order` closure `key` (lifetime error; now a nested `fn key`); 44.1 `Tree` derives `Default` (new_without_default); 30.4 `impl Default for Oracle` added (new_without_default).
- Stub-only (plan unchanged): an empty `Repository` stub in `BTreeMap` trips zero_sized_map_values; the scratch gives it a field.
- Three stale doc comments in scratch 27r/46 re-synced to the files.

## Done when and Proof commands
- Every `git log ... grep -c -x -E 'hearth-NN\.[a-b]'` (26, 44, 27, 46, 28, 29, 30, 47), `ls-tree` and `git grep` of 28, 29, 30: run on hearth `origin/main`: exit 0 or 1, prints 0 (the pre-session state); as the plan says for base state; the counts they expect after landing: at ready.
- 30 Done 3 `git grep -l -i minijinja ... | wc -l`: prints 0; as the plan says. `git grep -c '=2.20.0' ... Cargo.toml`: at ready.
- `heavy cargo nextest run ...` of 26, 44, 27, 46, 28, 29, 30, 47: at ready (name code a session lands); each `-E` parsed with `heavy cargo nextest list --workspace -E`: exit 0 for `test(=..) | test(=..)`, `binary(properties) | binary(budgets)`, the 29 `test(/golden|.../)` and `test(/^merge_/)`, the 30 `binary(/^fuzzy_(oracle|...)$/)` (lists scratch-30::fuzzy_oracle); `binary(/^fuzzy_(protocol|simulation)$/)`: exit 94 "no binary names matched" only because the scratch has no such binary; well-formed, at ready.
- Filters against test files: `each_git_case_prints_on_the_fake_what_real_git_prints` (44), `the_fake_prints_each_mmdebstrap_recording_as_recorded`, `the_committed_universe_is_what_the_cases_read_of_the_index`, `fake_mmdebstrap_leaves_what_real_mmdebstrap_recorded_for_each_case` (46), 28's two `diff_*` names (builder `tests/*.rs`, integration, bare): as the plan says.
- Counts: 26 lists 24 (>=24); 27 lists 27 (+24 of 26 = 51); 28 fuzzy 1+2 = 3; 30 fuzzy 2+2+2+2+1 = 9; 47 (30's 2 + 47's 2 + budgets 4) = 8.
- Digests (read-only): `printf 'a\n..h\n' | sha256sum` = 44.2 BEFORE a8cdd766...; with `E` = AFTER 890cc14f...; `Welcome, and be kind.\n` = a603a55b...e8b2: all agree. 27.3's 6e80ba38... (MAWK_KEPT, 306) not recomputed here (the bytes are not in the plan).
