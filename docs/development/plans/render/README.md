# render: hearth's M1, and skein's part of it

2026-10-17. Start commits: hearth `31bed37` ("Answer the M1 planner's
third design questions"), skein `eadbe29` ("Answer hearth M1's third round
for skein"); their code is as at hearth `175e92f` and skein `f981746`
(later commits changed designs, docs and scripts only). Each session
starts from its repository's `origin/main`. smith, jig and temper are read only, but for
their closing repins (Repin, below).

The plan builds M1, Render (hearth's `docs/design/README.md`, section 3;
`STATUS.md` at `31bed37`, section 3, its first two items): `format.md` but for checks
(section 11, M5) and artifacts (10.2, M6); `render.md`; the client's
`render` and `diff`, in-process (`client/README.md`, 3.1 to 3.3 and 3.5), with
`hearth-client-protocol-records`'s records and printer; `hearth-lib`; the store
without layers (`store.md`, 2.4), in the first parts of
`hearth-domain-store` and `hearth-protocol-tree`; the builder's root,
protocol layer and service; the fake machine's M1 parts and the fakes of
git and mmdebstrap; and `testing.md`, section 10's M1 row: every tier up
to end to end, for render, and conformance outside the gate. skein's part
of M1 (README.md, 8.1) lands in skein first, each piece before its first
hearth user: positions on skein-json's tokenizer, a digest of a world's
state, and the io the store needs: a directory's lock, a rename across
directories, a listing in pages. lib's streaming SHA-256 is on skein's
`main` already: `skein_lib::sha256` (`crates/skein-lib/src/sha256.rs`),
landed by temper's forge-setup plan.

## Inputs

- **hearth's design,** at `31bed37`: `README.md` (3, 4.2, 8.1),
  `format.md`, `render.md`, `client/README.md` (2, 3.1 to 3.3, 3.5),
  `store.md` (2.1, 2.4, 3.1, 3.2, 4.4, 6.1, 6.4 to 6.6, 6.8, 7.5, 8.1,
  8.2), `testing.md`
  (sections 2 to 5, 6.2, 8, 9, 10), `steward.md` (2.3, 3.4), by the
  sections each session names.
- **skein's design,** at `eadbe29`: `lib.md` (5.1, 6.2, 10), `json.md`
  (3.6), `testing.md` (4, 5), `shell.md` (12), `simulator.md` (3, 7),
  `io.md` (5) and `kernel.md` (6.1, 8); skein's foundation
  documents `programming-model.md` and `testing-strategy.md`, by the
  sections each session names.
- **The design answers:** `~/src/rust/tmp/hearth-m1-design-answers.md`
  (K1 to K3, H1 to H8; its "Second round", K4 to K6 and H9 to H13; its
  "Third round", K7 to K16 and H14 to H178).
  The answers are in the design documents; the file only indexes them.
- **The workflows:** hearth's `AGENTS.md` and
  `docs/development/workflow.md` (1 to 3); skein's `AGENTS.md` and
  `scripts/check.sh`.
- **`decisions.md`:** the plan's decisions, and what the user may veto.
- **`findings/`:** `skein.md` (skein's asks, and its gate on x1),
  `skein-io.md` (skein's io and shell for hearth's processes),
  `skein-testing.md` (skein's test kit, and the test each tier copies),
  `measurements.md` (skein's suites and gate at the base), `os-lists.md`
  (what 07 and 38 use of os's lists), `digests.md` (every digest literal
  of the plan, computed by one encoder of store.md, 3.1), and `base-*.md`
  (the base runs, one file a runner: development.md, 5.4; they land with
  the plan).
- **`later.md`:** what the design names that this plan does not build.

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Dependencies are by capability:
"hearth 03" means that session's last increment has landed on hearth's
`origin/main`; "hearth 03.1" that increment.

### skein

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `skein/02-positions.md` | skein 02: positions on skein-json's tokenizer | nothing | |
| `skein/03-state-digest.md` | skein 03: state digests: `StateHasher`, lib's `Hash`, replay and the trace | nothing | |
| `skein/04-store-io.md` | skein 04: skein's io for the store: a rename across directories, a listing in pages | nothing | |
| `skein/05-directory-lock.md` | skein 05: skein's io for the store: the kernel's lock on a directory, and io's | nothing | 05.2: skein 04, skein 05.1 |

### hearth

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `hearth/01-lib.md` | hearth 01: `hearth-lib`: the canonical encoding, then `H` and the vectors | nothing | |
| `hearth/02-git.md` | hearth 02: `hearth-protocol-git`; the vectors, and git's three stream machines | nothing | |
| `hearth/03-json.md` | hearth 03: `hearth-protocol-config`, strict JSON with positions and the configuration documents | skein 02 | |
| `hearth/04-declarations.md` | hearth 04: `hearth-protocol-config`: declarations, entries, expressions as text | hearth 03 | |
| `hearth/05-expressions.md` | hearth 05: `hearth-protocol-template`: expressions to postfix operations | nothing | |
| `hearth/06-templates.md` | hearth 06: `hearth-protocol-template`: templates to instructions, statements | hearth 37 | |
| `hearth/07-mmdebstrap.md` | hearth 07: `hearth-protocol-mmdebstrap` | nothing | |
| `hearth/08-mirror.md` | hearth 08: the test mirror and its one fetch | nothing | |
| `hearth/09-records.md` | hearth 09: `hearth-protocol-tree` (M1): the frame, 3.1's decoder, kinds 1, 4 and 5 | hearth 01.1 | |
| `hearth/10-client.md` | hearth 10: `hearth-client-protocol-records`, render's records and their printer | nothing | |
| `hearth/11-thin-story.md` | hearth 11: the thin story: `hearth render` on a repository through real git | hearth 01 | 11.2: hearth 02.3, hearth 03.3; 11.3: hearth 10.2 |
| `hearth/12-compose.md` | hearth 12: layers, RFC 7386, presence, provenance; the first error record | hearth 11, hearth 32.1, hearth 03 | 12.2: hearth 40.2 |
| `hearth/13-schema.md` | hearth 13: schemas, validation and defaults | hearth 12, hearth 04, hearth 35 | 13.3: hearth 14.1 |
| `hearth/14-evaluate.md` | hearth 14: expressions evaluated: values, the context, undefined, the operators | hearth 11 | |
| `hearth/15-files.md` | hearth 15: templates evaluated; the file tree rendered into the cache | hearth 43, hearth 06, hearth 12.2, hearth 36.2, hearth 42.1 | 15.2: hearth 13.4 |
| `hearth/16-entries.md` | hearth 16: declarations to concrete entries: `for_each`, `if`, `requires` | hearth 13, hearth 15 | 16.2: hearth 36.2 |
| `hearth/17-packages.md` | hearth 17: packages, `debian`, preseeds, the initramfs, the command line | hearth 16 | 17.5: hearth 25.1 |
| `hearth/18-tree-users.md` | hearth 18: files, users, groups, presets, tmpfiles, data, scripts | hearth 16, hearth 25.1, hearth 42.1 | |
| `hearth/19-secrets-software.md` | hearth 19: secrets, passwords, identity, downloads, live; fixed ids | hearth 16 | 19.2: hearth 18.4; 19.4: hearth 17.2, hearth 18.3; 19.5: hearth 17.5, hearth 25.1 |
| `hearth/20-exports.md` | hearth 20: exports, placements, a target's containers | hearth 16, hearth 42.3 | 20.4: hearth 25.1 |
| `hearth/21-reserved.md` | hearth 21: what hearth reserves; the `hearth` task | hearth 17, hearth 18, hearth 19 | 21.4: hearth 42.3 |
| `hearth/22-collisions.md` | hearth 22: collisions, render's additions, errors collected, sorted, bounded | hearth 12 | 22.2: hearth 17, hearth 18, hearth 25.1; 22.3: 22.1, hearth 18, hearth 19; 22.4: 22.2, 22.3 |
| `hearth/23-store.md` | hearth 23: `hearth-domain-store` (M1) and the store's io: lock, record, scan of the cache | hearth 09, hearth 11, hearth 32.1, skein 04, skein 05 | |
| `hearth/24-resolution.md` | hearth 24: resolution: keys, closures, `--resolve` | hearth 08, hearth 17.2, hearth 33.2 | 24.2: hearth 07, hearth 32.2, hearth 34.1, hearth 41.1, hearth 42.1; 24.3: hearth 38.1 |
| `hearth/25-diff.md` | hearth 25: the typed diff, rebuild, hunks, `hearth diff` | hearth 11, hearth 40.1 | 25.2: hearth 02, hearth 34.2 |
| `hearth/26-fake-git.md` | hearth 26: the fake machine's M1 parts | nothing | |
| `hearth/27-fake-mmdebstrap.md` | hearth 27: the package universe, the fake mirror and the closure | hearth 26 | |
| `hearth/28-builder-worlds.md` | hearth 28: the builder's worlds: domain, protocol and simulated; the fake machine behind the simulator; replay with digests | hearth 27, hearth 46.1, skein 03, skein 05.1 | 28.2: hearth 24, hearth 31; 28.3: hearth 45.1, hearth 44.2; 28.4: hearth 45.2; 28.5: hearth 39.1 |
| `hearth/29-goldens.md` | hearth 29: render's golden repositories | hearth 21, hearth 15, hearth 28.3 | 29.3: hearth 22, hearth 19, hearth 18; 29.4: hearth 46.1, hearth 20, hearth 22, hearth 19; 29.5: hearth 24.1 |
| `hearth/30-properties.md` | hearth 30: render's properties, the MiniJinja oracle, the fuzzed decoders | hearth 29 | 30.6: hearth 38.2, hearth 39 |
| `hearth/31-record-keep.md` | hearth 31: the resolution recorded, the indices kept | hearth 24, hearth 33.3, hearth 41.1 | |
| `hearth/32-static-binary.md` | hearth 32: static files into the cache, and the binary | hearth 11 | 32.2: hearth 40.2 |
| `hearth/33-store-entries.md` | hearth 33: `hearth-domain-store`: cache files enter, `resolutions/` scanned, what the store says | hearth 23 | 33.3: hearth 12 |
| `hearth/34-conformance.md` | hearth 34: git's conformance tier, its recordings replayed, the hunks | hearth 02 | |
| `hearth/35-schema-exports.md` | hearth 35: `hearth-protocol-config`: the schema, exports, the declarations' sweep | hearth 04 | |
| `hearth/36-template-whitespace.md` | hearth 36: `hearth-protocol-template`: whitespace, `raw`, interpolated strings, worst cases | hearth 06 | |
| `hearth/37-expression-operators.md` | hearth 37: `hearth-protocol-template`: operators, chains, forward jumps, worst case | hearth 05 | |
| `hearth/38-stanzas-recordings.md` | hearth 38: `hearth-protocol-mmdebstrap`: `Packages` stanzas, apt's lists, and real runs | hearth 07.2 | 38.2: hearth 07.3, hearth 41.1 |
| `hearth/39-tree-world.md` | hearth 39: `hearth-tree-world` (M1): the machine world and the worst cases of `hearth-protocol-tree` | hearth 09 | |
| `hearth/40-client-diff-errors.md` | hearth 40: `hearth-client-protocol-records`: the diff's records, the errors and the refusal | hearth 10 | |
| `hearth/41-resolve-recordings.md` | hearth 41: resolve's real side: mmdebstrap through the mirror, and its recordings | hearth 08, hearth 34.1, hearth 07.1 | |
| `hearth/42-listing-members-rules.md` | hearth 42: the listing's refusals, git's failures, members, names, the composition rules | hearth 12 | |
| `hearth/43-evaluate-translated.md` | hearth 43: the filters, hearth 05's operations translated, what one evaluation holds | hearth 14 | 43.2: hearth 37.2; 43.3: hearth 37.3 |
| `hearth/44-fake-git.md` | hearth 44: fake git, and git's conformance scenario | hearth 26.1 | 44.2: hearth 26.2; 44.3: hearth 34 |
| `hearth/45-diff-errors-resolve.md` | hearth 45: the diff's meta, errors, `hearth diff` and resolution | hearth 25, hearth 22.1, hearth 32.2, hearth 42.1 | 45.2: hearth 24, hearth 31.1 |
| `hearth/46-fake-mmdebstrap-resolve.md` | hearth 46: fake mmdebstrap's face; resolve's conformance scenario | hearth 27.3, hearth 38.1, hearth 41 | 46.2: hearth 08, hearth 07.3, hearth 44.3 |
| `hearth/47-digests-budgets.md` | hearth 47: render's digests in the joint worlds, and the budgets | hearth 28.4, hearth 25.1, hearth 30.1 | |

51 sessions: skein 4, hearth 47; 150 increments: skein 11, hearth 139. Then
the closing repins (Repin). The increments of each session are in its file.

### How many at once, and in what order

- **As many sessions at once as dependencies and the machine allow,**
  no fixed cap below the eight Codex threads (development.md, 7.2),
  held only by development.md 11.4's memory and disk bounds, which the
  coordinator checks before each start; on debian-8gb-hel1-1, with
  ruling 101's memory bounds (decisions.md).
- **At the start,** the eleven that need nothing: skein 02, 03, 04 and 05,
  hearth 01, 02, 05, 07, 08, 10 and 26;
  the coordinator starts eight, the longest chains' first (below), and
  each later slot goes to the first session whose Needs land.
- **The longest chain** (45 increments, from the Needs): skein 02 →
  hearth 03 → hearth 04 → hearth 35 → hearth 13 → hearth 15 → hearth 16 →
  hearth 18 → hearth 19 → hearth 21 → hearth 29 → hearth 30 (47 waits for
  30.1). Beside it: hearth 03 → hearth 11 (with hearth 01, 02.3, 03.3 and
  10.2) → hearth 32.1 → hearth 12 before 13; hearth 22 starts once
  hearth 12 has landed (22.1 needs nothing else; 22.2 to 22.4 wait for
  hearth 17, 18 and 19, so 29 still waits for them); hearth 05 → 37 → 06 before hearth 15 (06 → 36.2 before 15.1 and 16.2); hearth 14 → 43 → 15 before
  hearth 16 (13.4 before 15.2); hearth 12 → 42 before 15, 18, 20, 21 and 24.2; skein 04 and 05 → hearth 23 → hearth 33 → hearth 24 → hearth 31 → hearth 28;
  hearth 34.1 and hearth 41.1 before hearth 24.2 and 31.1 (33.3 before 31.1);
  hearth 07.2 → hearth 38.1 before hearth 24.3 and 46.1, hearth 08 → hearth 41 before
  hearth 38.2, 46.1 and 46.2; hearth 26 → hearth 44 (44.2 before 28.3,
  44.3 after hearth 34 before 46.2); hearth 26 → hearth 27 → hearth 46.1 before 28.1; hearth 10 → hearth 40 before 12.2, 25 and
  32.2; hearth 25 → hearth 45 (45.1 after hearth 22.1, 32.2 and 42.1, 45.2
  after hearth 24 and 31.1) before 28.3 and 28.4; hearth 28.3 before 29.1
  and 29.4, hearth 46.1 before 29.4, hearth 24.1 before 29.5; hearth 09 →
  hearth 39; hearth 28.4 and 30.1 before hearth 47.
- **When slots are short, start first:** skein 02, hearth 01, hearth 02,
  hearth 34, hearth 10, hearth 40, hearth 05, hearth 37, hearth 07, skein 04, skein 05, hearth 03, hearth
  11, hearth 32, hearth 04, hearth 35, hearth 06, hearth 36, hearth 14, hearth 43, hearth 12, hearth 42, hearth 13, hearth 15,
  hearth 16, hearth 22, hearth 17, hearth 18, hearth 19, hearth 09, hearth 39, hearth 23, hearth 33,
  hearth 08, hearth 41, hearth 38, hearth 24, hearth 31, hearth 21, hearth 20, hearth 25, hearth 26,
  hearth 44, hearth 27, hearth 46, skein 03, hearth 45, hearth 28, hearth 29, hearth 30, hearth 47.

## Rules of this plan

- **Every session reads,** besides its "Read first": its repository's
  `AGENTS.md` and `docs/development/workflow.md`, this README,
  `code-organization.md`, and the rulings of `decisions.md` that name its
  session. Nothing else is read whole.
- **The code graph:** the codebase-memory projects
  `home-free-src-rust-skein` and `home-free-src-rust-hearth`, as
  implementer.md, 8.1, says: never call `list_projects`, `index_status`,
  `check_index_coverage` or `index_repository`. The graph indexes each
  repository's `main`, never the session's worktree: check each name it
  gives with `rg` in the worktree before relying on it.
- **skein's gate** (decisions.md, 1): each skein increment lands with
  `land --gate` and the gate below, whatever the machine, and its commit
  body says so in one line. It is check.sh's commands at `f981746`
  (unchanged since `7e8fb70`) with one browser test filtered,
  `a_page_of_another_origin_is_opened_by_its_step`, which failed on x1 at
  the base (`findings/skein.md`, 4); the plan's skein changes touch no
  `skein-ui-web` code. Run once at the base: `findings/measurements.md`.

  ```sh
  land --gate 'cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo check --target wasm32-unknown-unknown -p skein-lib -p skein-codec -p skein-http && cargo clippy --target wasm32-unknown-unknown -p skein-ui -p skein-ui-web -p skein-ui-example-view -p skein-ui-example-shell-web -- -D warnings && scripts/taste.sh crates/skein-tls tests/tls && cargo check -p skein-tls && export NEXTEST_DOUBLE_SPAWN=0 && cargo nextest run --workspace && cargo nextest run --workspace --profile fuzzy && cargo build --target wasm32-unknown-unknown -p skein-ui-example-shell-web && cargo run -p skein-ui-bundle -- --module target/wasm32-unknown-unknown/debug/skein_ui_example_shell_web.wasm --name skein_ui_example_shell_web --prefix /bundle/ --stylesheet examples/ui/shell-web/notes.css --out target/ui-example-bundle --noscript "Notes needs WebAssembly and scripts." && cargo build -p skein-ui-example-server && export SKEIN_UI_EXAMPLE_BUNDLE="$(pwd)/target/ui-example-bundle" && cargo nextest run --workspace --profile browser -E "not test(=a_page_of_another_origin_is_opened_by_its_step)"'
  ```

  The browser suite needs `SKEIN_TEST_CHROMIUM` in the environment; the
  coordinator checks it at the start (Prerequisites). Where
  skein's `scripts/check.sh` differs from these commands once the run
  starts, check.sh's commands win, with the same filter.
- **Upstream first** (development.md, 5.3 and 9.5): a hearth increment
  whose Needs name a skein capability starts with a repin commit of its
  own, after `git -C ~/src/rust/skein fetch origin`, `heavy cargo update
  -p skein-lib`, which moves hearth's whole
  skein source to skein's `origin/main`; then `grep -o
  'skein.git#[0-9a-f]*' Cargo.lock | sort -u` prints one commit, which
  `git -C ~/src/rust/skein merge-base --is-ancestor <the needed
  landing> <that commit>` accepts. hearth names skein as its `AGENTS.md`
  says, by URL with no rev.
- **hearth's first landings** (workflow.md, 1):
  - a landing that adds a crate lands with `land`; a landing that changes
    only `.config/`, the root `Cargo.toml` or `scripts/` while no crate
    is on `main` lands with `LAND_PREBUILD= land`. Several sessions add
    their first crate at once; whichever lands first ends the case;
  - **the members' globs** (decisions.md, 16): the landing that adds the
    first crate under `crates/`, `testing/` or `tests/` adds that root's
    glob to the root `Cargo.toml`'s `members` (`"crates/*"`,
    `"testing/hearth-*"`, `"tests/*"`), and no landing adds a glob whose
    root holds no crate: cargo refuses one that matches nothing;
  - **the first fuzzy test:** the landing that adds the first
    `tests/*/tests/fuzzy_*.rs` replaces both profiles' filters in
    `.config/nextest.toml` with the two lines its comments give, and
    removes those two comments; check.sh fails it otherwise. A session
    that rebases onto a `main` where the filters are already switched
    drops its own switch.
- **Profiles outside the gate:** hearth 34.1 adds the `conformance`
  profile (testing.md, 9.2: a 2-minute ceiling) with its first test, and
  its binaries join the default and fuzzy profiles' exclusions in the
  same landing (workflow.md, 2). It runs as `heavy cargo nextest run
  --workspace --profile conformance`, never in the gate.
- **One unsafe exception:** `crates/hearth/src/lib.rs` holds one
  `#[expect(unsafe_code, reason = …)]`, `hearth::umask`'s `libc::umask`, with its
  `// SAFETY:` comment (hearth 32.1); no other lint exception.
- **Live services:** snapshot.debian.org, by hearth 08.2's fetch script,
  and crates.io twice (Live services, below). No other step reaches beyond loopback;
  every test, conformance included, runs on loopback.
- **Format versions** (development.md, 5.1): each is version 1 and starts
  in this plan, so none moves within it:
  - the encoding's tag `hearth/1` (store.md, 3.2): owner hearth 01.2;
  - the records' frame, kinds 1, 4 and 5 at version 1 (store.md, 6.1):
    owner hearth 09.1;
  - the client protocol, version 1 (`client/README.md`, 3.2): owner hearth 10.1.
    A later session adds records and fields and renames none.
- **The meeting point** (development.md, 5.3): the builder's protocol
  layer and render's domain meet in hearth 11's thin story, `hearth
  render` printing a spec from a scratch repository through real git and
  the real decoders. From hearth 11 on, each render increment proves on
  that path (the real loop or the process tier: code-organization.md, 5),
  never only in render's domain world. Each increment that adds a
  variant to a vocabulary that crosses the protocol layer extends
  `hearth-protocol-builder`'s translation in the same increment
  (code-organization.md, 3).
- **Edits outside an increment's Files** that are no question (they
  replace implementer.md, 8.2's "it is in your session's own files" for
  these cases only):
  - an arm each exhaustive match needs for a variant the increment adds,
    in any crate, at most 5 lines per match;
  - the users of a type the Shape changes, edited to its new form with no
    other change, every assertion, input, expected value and name kept;
  - a fix the change, its Proof or the gate needs, at most 20 lines, in
    any session's files, every test's name and expected value kept;
  - a golden repository's expected files, rewritten with `HEARTH_BLESS=1`
    where the increment changes what render prints, each changed file
    named in the report;
  - code that moved since the plan was written, followed to its file;
  - hearth 17.1's edit of every fixture repository's `fleet.json` (format.md
    makes `debian` required), and the re-paste of `THIN_TEXT` and the spec
    digests that a later session's part moves;
  - hearth 21.4's edit of every fixture repository that has no `hearth`
    task: `tasks/hearth/task.json`, `{}`, in `THIN`, in the scripted
    `x1_prod` and in every repository not built on `THIN` (like 17.1's
    `fleet.json`).

  These limits are guides: each landing's report lists its lines outside
  its Files (`git diff --numstat origin/main...HEAD -- <those paths>`),
  and a limit passed is a question before landing.
- **A Proof's "Bound: N s"** is an aim for the test's time as nextest
  prints it, building aside; a bound missed is noted in the report, and
  the landing stands.
- **Test time** (`findings/measurements.md`):
  - hearth has no test at the base. The run may add, as aims, at most
    8 s to the focused suite (of its 15) and 30 s to the fuzzy suite (of
    its 60), so that M2 starts with room. These are `Summary` times, over
    the machine's threads; a session's bounds are each test's own time
    (decisions.md, 21: the fuzzy bounds sum to about 12 s over 4 threads).
  - skein: focused 6.46 s, fuzzy 41.84 s, one gate run at the base. The
    run may add at most 1.5 s to the focused suite and 5 s to the fuzzy
    suite.
  - Each landing's report names its suites' times from its
    `Gate-Result` line, and nothing else is measured. An aim exceeded is
    a guide missed: the landing stands, and the session's next increment
    first makes the slowest tests cheaper, found once by the measure run
    of the suite that missed (`.config/nextest.toml`'s two commands,
    through `heavy --gate --`), until its own gate is back under the aim;
    after a session's last increment the supervisor decides. Never:
    raising a `global-timeout`, a slow period or a per-test ceiling, or
    moving a focused test to the fuzzy suite.
  - A test's own time in a session file ("under 20 ms", "under 0.5 s")
    is an aim, development.md 5.6's ceiling: a test the gate's log flags
    `SLOW` is made cheaper before its increment lands.
- **One land at a time in each repository:** a session runs `land` only
  under its repository's lock, held for the whole of `land`, its retries
  included: `flock ~/src/rust/tmp/hearth/render/land-hearth.lock land …`
  in hearth, `land-skein.lock` in skein.
- **Shared files,** which parallel sessions may both change; the session
  that lands second rebases onto the first and follows its change:
  - hearth's root `Cargo.toml` and `Cargo.lock` (every session that adds a
    crate or a dependency), `.config/nextest.toml` (whichever session
    lands the first fuzzy test: 02.2, 03.2, 07.2, ...);
  - `crates/hearth-protocol-builder/src/translate/`, a file a vocabulary
    (hearth 11's "Where later sessions add"): `listing.rs`,
    `documents.rs` (hearth 11, 12.1, 17.1, 32.1, 42.2), `answers.rs`
    (hearth 11, 12.2, 22.1, 24.1, 25.1, 32.1); `parts.rs` and
    `parts/<name>.rs` (11, 32.1, then each part's session, 17 to 20; 22.2
    `packages.rs`);
    `errors.rs` (12, 13, 15, 16, 17, 19, 20, 21, 24, 31, 33, 42, 45; 18 and
    22 add kinds through their rules, below); `declarations.rs` (13, 16,
    20.2); `operations.rs` (43);
    `instructions.rs` (15); `store.rs` (23, 33, 24, 31); `resolution.rs` (24);
    `hunks.rs` (25); `changes.rs` (25, 17.5, 18, 19.5, 20.4, 45);
  - `crates/hearth-domain-render/src/{lib.rs, vocabulary.rs}` and
    `src/parts/` (hearth 11 to 25; 28 the `lib.rs`);
  - `crates/hearth-client-protocol-records/src/{records.rs, errors.rs,
    form/text.rs, form/json.rs, tests/fixtures.rs}` (hearth 10, 40, 12, 16,
    17 to 25, 29, 31, 33, 45);
  - `tests/render/src/fixtures.rs` and `tests/builder/src/fixtures.rs`
    (each session that tests there), and the step crates' own
    `src/tests/fixtures.rs`: render's and `hearth-protocol-builder`'s
    (with its `src/tests/translate.rs`) (hearth 11 to 25);
  - `crates/hearth-domain-render/src/limits.rs` (hearth 11 to 22, 24, 25, 28);
  - the builder's crates' roots: `hearth-domain-builder`'s `src/lib.rs`
    and `src/vocabulary.rs`, `hearth-protocol-builder`'s and
    `hearth-service-builder`'s `src/lib.rs`, `crates/hearth/src/lib.rs`
    (hearth 11, 32, 12, 15, 23, 33, 24, 31, 25, 45, 28);
  - `hearth-domain-store`'s `src/vocabulary.rs` (hearth 23, 33, 24, 31), the
    protocol layer's `src/walk.rs` (33, 24), the records' `src/progress.rs`
    (33, 31); `hearth-domain-store`'s `src/lib.rs` (hearth 23, 33, 24, 31, 28);
  - `testing/hearth-fake-machine/src/hearth.rs` (hearth 26, 27, 28.1, 46.1),
    `src/hearth/{universe,mirror}.rs` (27, 28.1), `src/linux.rs` and
    `src/linux/*.rs` (26, 28.1);
  - the error kinds: render's `src/errors.rs` (hearth 12.2 creates it) and
    the records' `src/errors.rs` (hearth 40.2 creates it, with `Guard`; 12.2
    adds `Decode`); 42, 13, 15, 16, 17, 18, 19, 20, 21, 22, 24, 31, 33 and 45 add
    their kinds;
  - `tests/clippy.toml`, the worlds' lints: the first world crate to land
    copies skein's `tests/clippy.toml` (at `f981746`) unchanged; a later
    one keeps it;
  - `testing/repos/` (hearth 08, 29);
  - `testing/recordings/` (hearth 34, 41, 44);
  - `testing/mirror/` (hearth 08; 46.2 adds the committed `universe` cut);
  - `tests/conformance/` (hearth 34, 41, 44, 46: each its scenario);
  - skein: `crates/skein-lib/src/lib.rs` and `src/tests.rs` (skein 03),
    `crates/skein-io/src/file_layer.rs` (skein 04, 05),
    `testing/skein-world/src/` (skein 03),
    `testing/skein-fake-machine/src/fs.rs`, `tests/io/src/files.rs` and
    `crates/skein-sim/src/trace.rs` (skein 03, 04, 05),
    `crates/skein-io/src/file_layer/store.rs` (skein 05),
    `testing/skein-world/src/files.rs` and `src/real.rs` (skein 04, 05).

## Prerequisites the run checks at its start

The coordinator runs each check once, before the first session, and
writes `blocked.md` (kind: process) on a failure:

1. `cargo nextest --version` prints a version.
2. `git --version` prints `git version 2.` and `mmdebstrap --version`
   prints `mmdebstrap 1.` (hearth 34's and 41's recordings, the
   conformance profile).
3. skein's browser suite runs on this machine:
   `ls "$(rustc --print sysroot)/lib/rustlib"` lists `wasm32-unknown-unknown`, and
   `"$SKEIN_TEST_CHROMIUM" --version` prints `Chromium 147` (skein's
   gate; on debian-8gb-hel1-1, `~/.config/environment.d/60-plan.conf`
   sets the variable).
4. hearth 08.2's fetch tools: `command -v curl sqv xz sha256sum unshare`
   prints five paths, `unshare --map-root-user --net true` exits 0, and
   `test -r /usr/share/keyrings/debian-archive-keyring.pgp` exits 0.
5. hearth 41.1's `command -v apt-get` prints `/usr/bin/apt-get`.

## One test of a suite that needs a built artifact

None: no Proof of this plan runs a test that needs what the gate builds
first. skein's browser suite runs only inside skein's gate.

## Live services

- **snapshot.debian.org,** read only, anonymous: no credential and no
  sign-in. hearth 08.2's script `testing/mirror/fetch` fetches the test
  mirror's indexes at the pinned timestamp (testing.md, 6.2: forky,
  `main`, amd64, no pockets): the suite's `InRelease` and its `Packages`
  index for `main/binary-amd64`, whole, as the archive serves it
  (`Packages.xz`, about 10 MB at x1's 2026-09-28 lists; a test that needs
  its stanzas decompresses it with `/usr/bin/xz`, no crate). **Budget:** one run of the script,
  two files (about 10 MB); a run that fails may be repeated twice, after
  a change aimed at the cause its output names; a fourth run is a
  question. Its runs and bytes are in hearth 08.2's report, and each commit
  whose work ran the script carries `Fetch-Runs: <n>` and `Fetch-Bytes:
  <n>` trailers, the failed runs counted. A check of the script under
  `unshare --map-root-user --net`, which reaches nothing, is no run. Every later
  use reads the fetched files from the cache directory outside the
  repository that the script names; nothing else fetches.
- **crates.io,** twice: hearth 44.1's first landing, `sha1` `=0.10.7`
  (decisions.md, 13; with cfg-if, cpufeatures and digest 0.10); and hearth 30.4's
  first landing: one `heavy cargo fetch` of minijinja-2.20.0, memo-map-0.3.4 and serde_json-1.0.154, with
  serde-1.0.229, serde_core, itoa, memchr and zmij that they need, which
  its `Cargo.lock` pins from the registry.

## Repin

The plan lands in skein, which smith, jig and temper pin. It ends with
their closing repins (development.md, 5.7), which have no files:

- **`smith-repin`** starts once every skein step of the plan has landed:
  smith moves skein to skein's `origin/main`, by 5.7's steps.
- **`jig-repin`, then `temper-repin`,** start only once temper's
  client-api row in jig's and temper's `docs/design/STATUS.md` reads
  `done` (decisions.md, 3): that plan is active in both, and pins skein.
  jig moves smith and skein together (5.7, step 1); temper after it.
  Until then they wait; the run ends with them `waiting`, and its
  `done.md` names them, if client-api is still active when every other
  step has landed.
- hearth's own pin of skein moves with its sessions (Rules, "Upstream
  first").
- **What the repins meet** (searched at the plan's base: smith
  `f4f7f675`, jig `ad61bcfc`, temper `118f7fb70`): skein 02 and 03 change
  no public signature downstream (`Host` and `Machine` gain default
  methods; the tokenizer gains a method). skein
  04 and 05's new variants need match arms: in smith 15 matches in 13 files
  (`file::Event`, `file::Request`, `kernel::Op`, `kernel::Error`,
  `skein_sim::Ask`), in jig 9 in 8 files, in temper 13 in 8 files; and
  jig's `testing/jig-test-store-files/src/files.rs` destructures
  `Rename`'s fields (about line 104), which take K5's `from_root` and
  `to_root`. hearth has no code at the base, so the users the sessions
  change are skein's own: `git -C ~/src/rust/skein grep -l -E
  'Op::ReadSignal|Error::NotAFile' f981746 -- crates testing tests
  examples` lists them, 34 files (skein 05.1's Files).

The coordinator adds the three steps at setup: `plan set smith-repin
repo=smith state=waiting`, and likewise `jig-repin` and `temper-repin`.

## Pulse

```
words: legacy|compat|hack|workaround|actual|genuine|minijinja
types: ^(Legacy|Old|Compat|Raw)[A-Z]|V2$
file-lines: 800
budgets: 15 60
```

`minijinja` is forbidden in hearth's code outside its one oracle test
file (code-organization.md, 6); the pulse's word lists it so that a use
anywhere shows.

## Done when

Each command runs once, after `done.md`, reading `origin/main`.

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked, but `jig-repin` and `temper-repin` when
   client-api is not `done` (Repin).
2. hearth's M1 crates exist: `git -C ~/src/rust/hearth ls-tree -d
   --name-only origin/main crates/hearth-lib crates/hearth-protocol-config
   crates/hearth-protocol-template crates/hearth-protocol-git
   crates/hearth-protocol-mmdebstrap crates/hearth-protocol-tree
   crates/hearth-client-protocol-records crates/hearth-protocol-builder
   crates/hearth-domain-builder crates/hearth-domain-render
   crates/hearth-domain-store crates/hearth-service-builder crates/hearth
   testing/hearth-fake-machine testing/hearth-fake-git
   testing/hearth-fake-mmdebstrap testing/hearth-recordings
   testing/hearth-test-mirror tests/render tests/builder tests/conformance
   | wc -l` prints 21.
3. skein's two asks are public: `git -C ~/src/rust/skein grep -n -E
   'pub fn position\(|fn state_digest\(' origin/main --
   crates testing` prints a line for each of the two patterns.
4. The client's render is the binary's: in a worktree of hearth at
   `origin/main`, `heavy cargo nextest run -p hearth -E
   'test(/^end_to_end_/)'` passes, its `Summary` line naming at least 2
   tests (render and diff, testing.md, 2.7).
5. Conformance passes: in that worktree, `heavy cargo nextest run
   --workspace --profile conformance` passes, its `Summary` line naming
   at least the git and resolve scenarios' tests; its one run within the
   profile's 2-minute ceiling.
6. One definition of the client protocol's version: `git -C
   ~/src/rust/hearth grep -n -E 'CLIENT_VERSION: u32 = 1' origin/main --
   crates` prints one line, in `crates/hearth-client-protocol-records/src/`.
7. The test-time aims, measured once in a worktree of hearth and of skein
   at `origin/main`:
   `heavy --gate -- sh -c 'export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run --workspace 2>&1 | grep Summary; cargo nextest run --workspace --profile fuzzy 2>&1 | grep Summary; done'`.
   The medians of the three `Summary` times per suite, against
   Rules' aims. An aim missed is recorded with the three slowest tests by
   name in decisions.md's rulings, for the user; the run is done either
   way.
8. No budget was raised: `git -C ~/src/rust/hearth diff 175e92f
   origin/main -- .config/nextest.toml | grep -E
   '^[-+].*(global-timeout|period)'` prints only the conformance
   profile's added lines; `git -C ~/src/rust/skein diff f981746
   origin/main -- .config/nextest.toml | grep -E
   '^[-+].*(global-timeout|period)'` prints nothing.
9. smith pins skein's last landing of this plan: `git -C ~/src/rust/smith
   show origin/main:Cargo.lock | grep -o 'skein.git#[0-9a-f]*' | sort -u`
   prints one commit, which is `git -C ~/src/rust/skein log origin/main
   --grep='^Plan: render$' --format=%H -1`'s or a later one.
10. The plan's commits, in each repository (`<repository>` is skein,
    hearth or smith):
    `git -C ~/src/rust/<repository> log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u`
    lists each planned step: in skein `skein-02.1` to
    `skein-05.<n>`, in hearth `hearth-01.1` to `hearth-47.<n>`, as the
    session files number them, and `smith-repin` in smith. A supplementary
    step (`.s<k>`) adds a line; a step dropped by a ruling is named in
    decisions.md's rulings instead.
11. The aims stated above are these items; none other is checked.
