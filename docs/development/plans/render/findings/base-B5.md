# Base run B5 (hearth 12, 42, 13, 14, 43, 15, 16), skein f981746, hearth 175e92f

Machine debian-8gb-hel1-1: git 2.47.3, systemd 257 (257.13), coreutils 9.7. Scratch: `tmp/plan/hearth-render-scratch/B5-hearth` (crates `scratch/s12 s42 s13 s14 s43 s15 s16`, world crate `scratch/w`); skein by path `tmp/plan/hearth-render-skein`. Stubs (no owner sketch or other session's): Position (skein 02), config's Fault/Limits/Declaration, 06's template types, hearth 11's Token/Oid/Class/Outcome/Meta, the fixtures and harness (Repository, Form, Rendered, Roots, builders), `Documents::declaration` as a free fn (foreign inherent impl).

## Done when (all sessions)
- `git log origin/main --grep='^Plan: render$' ... | grep -c -x -E 'hearth-NN\.[a-b]'` for 12, 13, 14, 15, 16, 42, 43: exit 1, prints 0; at ready (runs).
- `git ls-tree --name-only origin/main crates/hearth-protocol-builder/src/translate/`: exit 0, prints nothing; at ready.
- the `git grep ... | wc -l` counts (12.3 `Decode(Decode)`, 42.3, 13.3 `Schema(Schema)`, 15.4, 16.4, 16.3 ls-tree grep -c): print 0; at ready. `-E` patterns of all (and `hearth-[0-9]+\.[1-5]`) parse (`grep -E <p> /dev/null` exit 1).
- `git grep -n -i minijinja origin/main -- crates/hearth-domain-render tests/render/src` (14.3, 43.4, 15.5, 16.5): exit 1, prints nothing; as the plan says.
- `git grep -n -E ', \.\. ?\}|\(\.\.\)' ... translate | wc -l` (13.4, 43.5, 16.6): prints 0; as the plan says.
- `heavy cargo nextest run ...` of the Done whens and `binary(=memory)`, `binary(=fuzzy_schema)`, `binary(=fuzzy_entries)`: at ready (name crates and binaries that the sessions add).

## 12
- 12.1 sketch block + `a_later_layer_s_scalar_replaces_and_keeps_its_own_place`: `heavy cargo clippy -p scratch-s12 --all-targets -- -D warnings`: first exit 101 (2x must_use_candidate: translate `fault`, `error`); DEFECT, fixed (`#[must_use]`). Then exit 0.
- 12.2 `a_broken_document_prints_its_error_after_...` in world crate (`-p scratch-w`, tests/s12.rs): exit 0.
- Decode's 11 tokenizer variants checked against skein-json `tokenizer::Error` (tokenizer.rs:221): all match (TooLong, Stream left out).
- BROKEN_LAB `1:14`/`1:2`, `{"a": 1, "a": 2}` `1:10`/`1:2`, `775324fc…` = sha256 of `Welcome to x1.\n`: python/sha256sum, as the plan says.
- 12.1 LAYERS expected lines: THIN's motd also lists `/usr/local/bin/motd`, which `lines_starting(.., "  /")` keeps; DEFECT, fixed (line `f487088ec8b1` added to x1-prod's block).
- skein citations tokenizer.rs:221, stack.rs:13, list.rs:14: as the plan says.
## 42
- `-p scratch-s42` (sketches, `the_tail_keeps_the_last_lines_each_cut_to_its_limit`): exit 0.
- Real git 2.47.3, scratch repo, root tree loose object removed: `rev-parse --verify --quiet HEAD^{commit}` exit 0; `ls-tree -r -l -z --full-tree <commit>` exit 128, `fatal: not a tree object`; as the plan says.
## 13
- `-p scratch-s13` (sketch block, `a_read_of_a_setting_its_task_lacks_is_refused_at_it`, FORMS test): first exit 101 (3x must_use_candidate on translate `declaration`, `map`, `declared`); DEFECT, fixed. Then exit 0.
- Positions 1:34 (13.2), 1:54 (13.3), 2:16 3:22 4:7 (13.4), 1:28 1:54 (13.5): recomputed by script, as the plan says.
## 14
- `-p scratch-s14` (sketches, both unit tests): exit 0. `-p scratch-w` tests/s14.rs (world test): exit 0.
- skein citations stack.rs:20/48/57, list.rs:55/104, bytes.rs:12/40, decimal.rs:18, writer.rs:32/49/60: as the plan says.
## 43
- Escape rows, run here (systemd 257, coreutils 9.7; the plan's: 261, 9.10): `systemd-escape -- .x` `\x2ex`; `-- a-b` `a\x2db`; `--path /` `-`; `--path /a//b/` `a-b`; `--path /a/./b` `a-b`; `--path /a/../b` exit 1 (refused); `--path /..` `-` (as Q2 says of 261); `basename /`,`dirname /` `/`,`/`; `basename a`,`dirname a` `a`,`.`: all as the plan says.
- DIFFERENCE (finding): `systemd-escape --path a/b` here (257): exit 0, stdout `a-b`, stderr warning "not an absolute file system path, escaping is likely not going to be reversible"; the plan says refused (261 on x1). Row kept; hearth's `Path` refusal of a relative path is hearth's rule either way. Also `--path ..` and `/a/..` exit 1 here.
- `-p scratch-s43` (sketches, tojson test, `each_operation_translates_to_render_and_back`): first exit 101 (5x must_use_candidate on translate/operations.rs); DEFECT, fixed. Then exit 0. `-p scratch-w` tests/s43mem.rs (memory test, skein-heap): exit 0.
- 05's 26 `Op`s = 14's 9 + 16 + `Filter`: counted, as the plan says; skein writer.rs:530, tests/json/tests/memory.rs:134: as the plan says.
## 15
- `-p scratch-s15`: exit 0 after the same fix (6x must_use on translate). sha256 of `x1-prod\n` 21c50ade…, the 32-byte issue e75f92b4…, `hello\n` 5891b5b5…: as the plan says; positions 2:1, 1:11, 1:39 recomputed: as the plan says.
- DEFECT, fixed: 15.1's `[ErrorKind; 19]` and `Syntax::Interpolation` need 36.2's `ErrorKind::Interpolation` (06 has 18), not in Needs; 36.2 added to 15.1's Needs and the README row (and chain text).
## 16
- `-p scratch-s16` (sketches, guard test, references test): first exit 101 (`expand<'c>` needless_lifetimes; translate `name` must_use); DEFECT, fixed. Then exit 0. `Name` has 37 variants: as the plan says.
- Positions 3:5, 3:21, 4:21, 1:27 recomputed: as the plan says. skein map.rs:39/59/68/88, decimal.rs:36: as the plan says.
## nextest `test(=…)`
- No `test(=…)` in these seven files: nothing to change.

## Compile pass, planner-9 (2026-10-17)

Blocks of 12 (3), 42 (2), 13 (3), 14 (4), 43 (4), 15 (2), 16 (3) compared with the scratch crates (B5-hearth), then
`heavy cargo clippy --workspace --all-targets -- -D warnings`: exit 0 (8 crates: s12, s13, s14, s15, s16, s42, s43, w).
`scratch-s12` failed (101: `must_use_candidate` on translate `fault` and `error`): the block says `#[must_use]`;
re-synced (both attributes, the test's assertion text, the `Kind` doc). s14 (six doc comments), s42 (`Members` doc),
s43 (the tojson assertion text) re-synced; 13, 15, 16 as the blocks say (stubs differ only in module paths).
Compile pass, planner-9, 2026-10-17.
