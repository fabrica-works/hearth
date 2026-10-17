# render: code organization

The crates, modules, names, shared interfaces and their owners, test
layout and fixtures, the plan's "Don't" list and its size limits. The
roles are skein's `programming-model.md`, section 4, and hearth's
`README.md`, 4.2; a session's Shape gives the module paths and
signatures it needs.

## 1. Crates

### hearth (all new)

| Crate | Path | Role | Holds | First |
|---|---|---|---|---|
| `hearth-lib` | `crates/hearth-lib` | lib: no_std, skein-lib only | the canonical encoding and `H` (store.md, 3.1, 3.2) | hearth 01 |
| `hearth-protocol-config` | `crates/hearth-protocol-config` | machine: lib, skein-json | strict JSON to typed documents and declarations, with positions; schemas' form | hearth 03 |
| `hearth-protocol-template` | `crates/hearth-protocol-template` | machine: lib | expression and template text to postfix operations and instruction sequences | hearth 05 |
| `hearth-protocol-git` | `crates/hearth-protocol-git` | machine: lib | git's argument vectors and environment; `rev-parse`, `ls-tree`, `cat-file --batch` and `diff --no-index` outputs | hearth 02, 34, 42 |
| `hearth-protocol-mmdebstrap` | `crates/hearth-protocol-mmdebstrap` | machine: lib | simulate's vector; `Inst` lines; `Packages` stanzas; failure tails and their classes | hearth 07, 38 |
| `hearth-protocol-tree` | `crates/hearth-protocol-tree` | machine: lib, hearth-lib | M1: the records' frame, 3.1's decoder, kinds 1, 4 and 5 | hearth 09 |
| `hearth-client-protocol-records` | `crates/hearth-client-protocol-records` | machine: lib, skein-json | the client protocol's records of `render` and `diff`, version 1; the printer, as text and as JSON lines; the outcome's exit code | hearth 10, 40 |
| `hearth-domain-render` | `crates/hearth-domain-render` | domain: lib, hearth-lib | configuration to spec; resolution's decisions; the diff (render.md, 8.1) | hearth 11 |
| `hearth-domain-store` | `crates/hearth-domain-store` | domain: lib, hearth-lib | M1: resolutions and cache files' records; the scan's decisions; the index | hearth 23, 33 |
| `hearth-domain-builder` | `crates/hearth-domain-builder` | root domain: lib, hearth-lib, its two children | routes between render and store; the client's calls in, the answers out | hearth 11 |
| `hearth-protocol-builder` | `crates/hearth-protocol-builder` | protocol layer: lib, io, its machines, the builder's root | routes its machines; translates to and from the root's vocabulary; streams blobs and rendered files into the cache; the store's io; routes the answers to the printer when the client hosts it (decisions.md, 2) | hearth 11, 32 |
| `hearth-service-builder` | `crates/hearth-service-builder` | service: lib, io, the protocol layer, the root | the builder's `iterate` | hearth 11, 32 |
| `hearth` | `crates/hearth` | shell: ordinary Rust | the binary: in M1 the client's `render` and `diff`, in-process: argv, startup, the loop, the exit code | hearth 11, 32 |
| `hearth-fake-machine` | `testing/hearth-fake-machine` | test kit held to the step crates' lints (testing.md, 3.1): `[lints] workspace = true` and its own `clippy.toml` | M1: `linux` (files with owners and modes, processes, pipes, spawns with argument vectors) and `hearth` (the program table: hearth 26; the universe's index at snapshot timestamps and the fake mirror: hearth 27) halves (testing.md, 3.1, 3.2, 3.5, 3.6); from hearth 28, `behind.rs`: `impl skein_world::Machine`, with `skein-io`, `skein-sim` and `skein-world` as dependencies | hearth 26 |
| `hearth-fake-git` | `testing/hearth-fake-git` | test kit | git's fake: a domain on skein-fake-checkout's mechanics, and its argument-vector face (testing.md, 4.1, 4.4) | hearth 44 |
| `hearth-fake-mmdebstrap` | `testing/hearth-fake-mmdebstrap` | test kit | `--simulate --mode=chrootless` against the universe: `Inst` lines and `Packages` indices (testing.md, 4.3) | hearth 27, 46 |
| `hearth-recordings` | `testing/hearth-recordings` | test kit: ordinary Rust, own `clippy.toml` | the recordings' layout, read and written: replayed by the machine worlds, written by conformance under `HEARTH_RECORD=1` | hearth 34 |
| `hearth-test-mirror` | `testing/hearth-test-mirror` | test kit: ordinary Rust, own `clippy.toml` (which admits `std::thread::spawn`, for the proxy) | the mirror's pin, the fetched files' cache by the mirror's digest, the loopback HTTP proxy (testing.md, 6.2) | hearth 08 |
| `hearth-config-world` | `tests/config` | machine world | config's recordings and generated documents, cut and mutated; the goldens as corpora: `tests/fuzzy_goldens.rs` (hearth 30.6) | hearth 03 |
| `hearth-template-world` | `tests/template` | machine world | the template machine's inputs, cut and mutated; the goldens as corpora: `tests/fuzzy_goldens.rs` (hearth 30.6) | hearth 05 |
| `hearth-git-world` | `tests/git` | machine world | git's recordings, cut and mutated; hearth 34.1 adds `src/{expect, mutate}.rs` and `tests/recordings.rs` to hearth 02's crate, hearth 30.6 `tests/fuzzy_revisions.rs` | hearth 02, 34 |
| `hearth-mmdebstrap-world` | `tests/mmdebstrap` | machine world | mmdebstrap's recordings, cut and mutated | hearth 07, 38 |
| `hearth-tree-world` | `tests/tree` | machine world | records' frames, cut and mutated | hearth 39 |
| `hearth-render-world` | `tests/render` | domain world | render's domain under a scripted parent (from hearth 43.2 it also depends on `hearth-protocol-template` and `hearth-protocol-builder`, to send its expressions through the real compiler and translation; from hearth 16 it also has `hearth-protocol-config` as a dev-dependency, for declarations); the golden repositories (`tests/render/<name>/`, render.md, 9.2) with their harness, `src/golden.rs` and `tests/goldens.rs`, and each golden's `expected/` and `rendered/` (hearth 29); properties (`src/{repositories, json, spacing}.rs`: hearth 30); the MiniJinja oracle, whose engine is `tests/oracle/engine.rs`, reached by `#[path]` from `oracle.rs` and `fuzzy_oracle.rs` (hearth 30) | hearth 11 |
| `hearth-builder-world` | `tests/builder` | domain, protocol and simulated worlds; the real loop | the builder's root with render and store; the protocol layer with the tools' fakes; the service on skein's simulator; real git; from hearth 24, `hearth-test-mirror` as a dev-dependency, from hearth 31 `hearth-recordings`; from hearth 29, `hearth-render-world` as a dev-dependency, for the goldens; `tests/fuzzy_inputs.rs` (hearth 30.6) | hearth 11 (the real loop); hearth 28 (the rest) |
| `hearth-conformance-world` | `tests/conformance` | conformance, outside the gate | the scenarios of testing.md, 5.2's M1 rows (git, resolve); recording on with `HEARTH_RECORD=1` | hearth 34 |

Data, not crates: `testing/recordings/<tool>/<version>/` (testing.md,
4.5: hearth 34, 41), `testing/repos/` (the test repositories, `test-min`
and `test-boot` among their targets: hearth 08), `testing/mirror/` (the
pin and its fetch script: hearth 08).

Every crate's `Cargo.toml` takes `version`, `edition`, `rust-version` and
`publish` from the workspace; a step crate takes `[lints] workspace =
true` and forbids `unsafe_code` at its root; an ordinary-Rust crate
declares its own `[lints]` table, as `findings/skein-testing.md`, 7's
example crate does.

### skein (changed)

| Crate | Change | Session |
|---|---|---|
| `skein-lib` | `StateHasher` and the containers' `Hash` (lib.md, 5.1), over the landed `sha256` module (lib.md, 6.2) | skein 03 |
| `skein-io` | the store's io: `Lock`, `Rename` across two directories, `OpenDirectory` and `ListPage` (K4 to K6) | skein 04, 05 |
| `skein-shell`, `skein-sim`, `skein-kv`, `skein-fake-machine`, `skein-conformance`, `tests/io` | the kernel's `Lock` (`flock`), the simulator's `Ask::Lock` and `Summary::Lock`, the fake machine's locks and moves, `skein-kv`'s one `Rename`, the lock's conformance scenario, io's worlds | skein 04, 05 |
| `skein-json`, `skein-json-world` (`tests/json`) | `Tokenizer::position` (json.md, 3.6); the reference parser's positions, checked on every run | skein 02 |
| `skein-shell`, `skein-world`, `skein-sim`, `skein-fake-machine`, the echo example's crates | `state_digest` on `Host` and `Machine`; `assert_replays` and the trace (testing.md, 5; shell.md, 12); skein-sim's `Event::Digest` and `Pid::MACHINE`; the fake machine's and the echo service's digests, so that a real service proves the harness | skein 03 |

## 2. Modules and names

- **Named by the design's words.** A type, module or function takes the
  word the design gives the thing: `Listing`, `Blob`, `Declaration`,
  `Part`, `Closure`, `Resolution`, `Change`, `Rebuild`, `Hunks`; render.md,
  8.2's requests and events keep their names (`List`, `Listed`, `Read`,
  `Cached`, `Keep`, `Kept`, `Resolve`, `Resolved`, `Hunks`, `Failed`).
- **`hearth-domain-render`'s modules** (render.md, 8.1): `revision`
  (the listing, its classes, decoded documents), `compose` (discovery,
  layers, RFC 7386, presence, provenance; the implicit `hearth` task:
  hearth 21.4), `schema` (validation and
  defaults; `schema/references`, the reference check of declarations and
  templates: hearth 16.4), `evaluate` (the value stack, the filters, the
  template instructions; `template.rs`'s `run` gains `element`: hearth
  18.2), `exports`, `entries` (conditions and concrete declarations:
  hearth 16; `Expansion::refuse`: hearth 21.1), `kinds/` (one module per declaration kind; `kinds.rs` and
  `required`: hearth 16.3; `packages` 17.2, `preseeds` 17.3, `cmdline`
  17.4, `files` 18.1, `users` and `groups` 18.3, `presets`, `tmpfiles`
  and `scripts` 18.4, `data` 18.5; `secrets` and `identity` 19.1,
  `software` 19.2, `live` 19.3), `ids.rs` (fixed ids:
  hearth 19.4), `reserved` (`reserved/{words,paths,owners,live}`: hearth
  21.1 to 21.3), `collisions`, `additions` (subordinate ranges, data lines
  and the initramfs's conf: hearth 22.2), `parts/` (one module per part,
  with its record and digest; `packages` 17.2, `boot` 17.4, `data` 18.5,
  `secrets` 19.1, `software` 19.2, `live` 19.3, `exports` 20.2), `keys`, `resolution`, `diff`, `errors`,
  `call` (a call's state), and `vocabulary` (the `Event` and `Request`
  that cross to the root).
- **Each machine's crate** has `src/<format>.rs` for its decoder or
  encoder, `src/limits.rs` for its configured limits, a `pub struct
  Limits` with the design's defaults as `pub const SHIPPED: Limits` (one
  name in every crate: the shipped profile, development.md 5.6) and
  `pub fn worst_case(...) -> Option<u64>` (in `limits.rs` or its machine's
  module; `hearth-protocol-tree`'s: 39.1), what one decoder holds at
  given limits (taking a `Decoder` enum first where the crate has
  several entry points, as hearth 02's does), and
  `src/tests.rs` or `src/tests/` for its step tests. `hearth-lib` has no
  configured limit, and so no `limits` module.
- **Tests** of a step crate are in `src/tests.rs` or `src/tests/<topic>.rs`
  (testing.md, 2.9); worlds are `tests/<name>`, named
  `hearth-<name>-world`, with focused tests in `tests/<name>/tests/*.rs`
  and fuzzy ones in `tests/<name>/tests/fuzzy_*.rs`.
- **End-to-end tests** are `crates/hearth/tests/end_to_end.rs`, each test
  named `end_to_end_<story>` (README.md, Done when 4).
- **Environment variables in tests** are read with `std::env::var_os`
  (skein's `tests/clippy.toml` bans `var`: `findings/skein-testing.md`):
  `HEARTH_BLESS=1` rewrites a golden repository's expected files and
  fails the test; `HEARTH_RECORD=1` makes the conformance profile
  rewrite the recordings.

## 3. Shared interfaces and their owners

One owner each: the session that defines it, which every other session
uses by name and extends only as its own Shape says.

| Interface | Owner | Extended by |
|---|---|---|
| `hearth_lib::encode`'s value model (`Item`, `Digest`, `Encoder`, the first-byte constants, `DEPTH`) and `hearth_lib::hash::{H, VERSION}` | hearth 01 | nobody |
| config's decoded values and errors (`hearth_protocol_config::{json::Value, Error}`, positions on every value) | hearth 03 | hearth 04 (declarations, entries), 35 (schemas, exports) |
| config's declarations, entries and schemas (`hearth-protocol-config`'s `declaration`, `entries`, `schema`: uniform `Entry`, `Field`, `Name`, `Map`, `Literal`; expressions as `Text` with positions) | hearth 04 (`schema` and `exports`: 35) | mirrored by `hearth-protocol-builder`'s translation: hearth 13 (`types`, `settings`, `reads`), 16 (`entries`, `requires`), 18 to 20 (`scripts`, `exports`) |
| render's values and evaluator (`hearth_domain_render::evaluate`: `Value`, with no null and no position, an absent optional entry `None`; `Evaluator`, `Bindings`, `Budget`, `truthy`, `output`; the filters, hearth 43) | hearth 14 | values built by hearth 13 (settings, tasks), 17 (`debian`), 20 (exports, containers); the evaluator used by hearth 15 (which adds `Operand::Loop { index0, length }` and `evaluate::context::Roots`), 16; each root of the template context filled by its session: 13 `settings`/`tasks`, 17 `debian`, 20 `exports`/`containers` |
| the store domain's vocabulary (`hearth-domain-store`: its calls and answers with the root) | hearth 23 | hearth 33 (arrivals with the four `Origin`s, `Load`, `Empty`, `Waiting`, `Strays`), hearth 15 (rendered files as arrivals with `Origin::Rendered`), hearth 24 (`Find`, `Found`, `Declined`, `Event::Resolution` with the whole record; the walk of `staging/<key>`), hearth 31 (`Record`, `Write`, `Room`, `Recorded`, `HasRoom`, `Unwritable`, `Bound`; kept indices as arrivals with `Origin::Index`) |
| render's errors and their kinds (`hearth_domain_render::errors`), and the records' (`hearth-client-protocol-records`'s `errors.rs`) | hearth 12 (render's file, `decode`; the records' `Decode`), 40.2 (the records' file, `Guard`), 42 (`repository`, `composition`, `tool` for git) | a kind each: hearth 13 `schema` (and `Decode::Declaration(Declared)`) and `evaluation` (13.4; 15.2 uses it), 16, 19 `ids`, 20 `exported` (and `Limits::exported`), 21 `reserved`, 22 `Collector` and `Collision` (the collection, sort and count, and `Limits::errors`), 24 `resolution` and `tool` for mmdebstrap (`Failure::{Run, Store}`, `Cause`, `Limit::Closure`), 31 (`Resolution::TooLarge`, `Limit::Record`, the records' `Step::Write`), 33.3 `store` (render's and the records'), 45.1 `side` on both |
| render's composition (`compose::{Merge, Provenance, Settings, Target}`; hearth 42.2 discovers the targets) | hearth 12 | hearth 13 (`Layer::Defaults`, `compose`'s `defaults` argument, `compose::present`); read by hearth 14, 17, 20 |
| the template machine's vocabulary (`hearth-protocol-template`: `Expression`, `Operation`, `Op`, `Missing`, `Filter`; then the instructions) | hearth 05 (expressions), 37 (operators, jumps) | hearth 06 (instructions), 36 (interpolated strings); mirrored by `hearth-protocol-builder`'s translation (hearth 43.2, 15, 16) |
| the frame and records of kinds 1, 4, 5 (`hearth-protocol-tree`), every field of store.md 6.4 to 6.6 | hearth 09 | nobody |
| the client protocol's records, version 1, and `CLIENT_VERSION` | hearth 10 (the diff's and the errors': 40) | hearth 11, 17 to 25, 29: records and fields added, none renamed |
| render's vocabulary (`hearth-domain-render`'s `vocabulary.rs`): `List`, `Listed`, `Unresolved`, `Read`, `Cached`, `Failed`; render.md 8.2's records, two fields more: `Read` carries the `repository` and each blob's listed `size`, and `List` ends in `Listed`, `Unresolved` or `Failed` (hearth 11's Choices) | hearth 11 | `Answer::Error`, `Decoded::Refused { fault, also }`: hearth 12; `Failed { at, failure }`: hearth 42.1; `Keep` (with the rendered file's `path`, store.md 6.5's origin) and `Kept`: hearth 15; `Resolve`/`Resolved`, `Find`, `Found`, `Absent`, `Inst`, `Stanza`: hearth 24; `Record`, `Recorded`: hearth 31; `Hunks`, `Diff`, `Change`, `Outcome::Negative`: hearth 25 |
| the builder root's vocabulary with its protocol layer: the client's call (`Render`, then `Diff`) and its answers (spec, change, error, outcome) | hearth 11 | hearth 23, 33 (the store's), hearth 24 (`--resolve`), hearth 31 (`Index`, `Progress::Resolving`), hearth 25 (`Diff`, `Answer::Change`, `Reply`'s diff tally), hearth 45.2 (`resolve` on `Diff`) |
| `hearth-protocol-builder`'s translations (`src/translate/`), exhaustive both ways: config's documents to render's, template's operations to render's, render's answers to the client's records | hearth 11 (documents, answers), hearth 32.1 (parts) | hearth 13, 16 (declarations), 43 (operations), 15 (instructions), each kind session (its part's records), 23 to 25, 33 |
| render's parts and their records (`parts/`) | hearth 11 (`meta`), hearth 32.1 (`config` with static files) | each declaration-kind session adds its part's fields; hearth 24 the keys |
| the test repositories' layout (`testing/repos/`) | hearth 08 | hearth 11, 24, 27, 29, 31 |
| the recordings' layout (`testing/recordings/<tool>/<version>/<case>/`: `argv`, `env`, `stdin`, `stdout`, `stderr`, `exit`, and an optional `expect`, which mmdebstrap's keep in `tests/mmdebstrap/expect/<version>/` (hearth 38); hearth 41 adds the optional `lists` and `stanzas`), read and written through `hearth-recordings` | hearth 34 | hearth 41, 44, 46 |
| the test mirror's kit (`hearth-test-mirror`: `Pin`, `Mirror`, `Proxy`) | hearth 08 | used by hearth 24, 27, 31, 46.2 |
| the fake machine's io face | hearth 26 | hearth 27, 46, 28 |
| the fake machine behind skein's simulator (`impl skein_world::Machine`, in `testing/hearth-fake-machine/src/behind.rs`, which gains `skein-io`, `skein-sim` and `skein-world`), and io's file records (`Load`, `Store`, `Scan`, `OpenDirectory`, `ListPage`, the two-directory `Rename`, and skein 05's `Lock` on hearth 26.1's `Op::Lock`) made of its operations | hearth 28 | nobody |

## 4. Fixtures

- **One module per crate,** made by the first session that tests there,
  extended by the others, never redefined: a step crate's
  `src/tests/fixtures.rs`; a world's `tests/<name>/src/fixtures.rs`.
- **The scratch repository** of the real loop and end to end: a
  directory the test makes with real git (`git init`, a commit of files
  the test names), once per test process, in a directory of its own
  under `std::env::temp_dir()`, never inside hearth's checkout, whose git
  configuration a git run there would read (no `tempfile` crate); its
  helper lives in
  `tests/builder/src/fixtures.rs` (hearth 11; hearth 18 adds `File::link`,
  a symbolic link in the tree) and the end-to-end tests reuse it through
  a dev-dependency on `hearth-builder-world`.
- **Limits come from the shipped profile** (development.md, 5.6): each
  world and test takes render.md 2.3's and store.md 6.1's defaults from
  the crates' `limits` modules; a test that needs another value names it
  beside the default, with the reason.

## 5. Proof layers in this plan

| Layer | Here | From |
|---|---|---|
| unit | a step crate's `src/tests` | hearth 01 |
| world | a machine world; render's domain world | hearth 02, 11 |
| joint | the builder's worlds with the real protocol layer and the tools' fakes (`tests/builder`, protocol and simulated) | hearth 28 |
| process | the real loop with real git (`tests/builder/tests/real.rs`); the binary (`crates/hearth/tests/end_to_end.rs`) | hearth 11, 32 |
| live | conformance against real git, real mmdebstrap and the mirror's indexes on loopback | hearth 34 (git), hearth 41 (mmdebstrap), hearth 24, 31 (`render --resolve`, `tests/builder/tests/conformance_render.rs`) |

From hearth 11 on, a render increment proves at the process layer (the
real loop's `hearth render` through real git), and a protocol increment
at the joint layer once hearth 28 opens it; before that, at the process
layer.

## 6. The plan's "Don't" list

- No type beside an existing one for the same thing: change the type in
  place, and its users in the same increment.
- No field or predicate that its counterpart across a boundary lacks.
- No test that checks several behaviours under one name. A table test
  whose rows are cases of one behaviour is one behaviour.
- No lint exception in tests.
- No expected value computed by the code under test: digests in tests
  are literals, from the design's vectors or from `sha256sum` run once
  and pasted.
- No recursion and no unbounded loop in step code (render.md, 8.3): a
  review rule (clippy cannot lint it); a report quotes `rg -n
  '\b(loop|while)\b' <crate>/src`, empty or each hit justified.
- No parsing in a domain; no formatting in a domain (`client/README.md`, 3.2).
- No `Vec`, `String` or `_` arm in render and its translation (render.md,
  8.3): `git grep -n -E '\bVec\b|\bString\b|^\s*_ =>' origin/main --
  crates/hearth-domain-render/src crates/hearth-protocol-builder/src/translate
  ':!*/tests/*' ':!*/tests.rs'` prints nothing (hearth 17 to 22's Done when).
- No crate outside skein's and hearth's in step code (hearth's
  `AGENTS.md`): no serde, no template engine. MiniJinja 2 only as a
  dev-dependency of `hearth-render-world`, pinned to an exact version,
  named in one test file, `tests/render/tests/oracle/engine.rs`, reached by
  `#[path]` from `oracle.rs` and `fuzzy_oracle.rs` (hearth 30).
- No hand-written recording (testing.md, 4.5): a recording is captured
  by the conformance profile with `HEARTH_RECORD=1`.
- No network beyond loopback in any test; no fetch from
  snapshot.debian.org but hearth 08.2's script.
- No name of a crate that is not M1's (README.md, 4.2's later rows).
- The words the pulse forbids, in code and comments (README.md, Pulse).

## 7. Size limits and the count

- **Aims:** a file of at most 800 lines and a test function of at most
  40; a miss is noted in the landing's report, and the landing stands. A
  table test keeps its rows in a `const` beside the function.
- **An increment's size** is its hand-written production lines, counted
  once before it lands, in its worktree:
  - hearth: `git diff -M --numstat origin/main...HEAD -- crates testing
    ':!*/tests.rs' ':!*/tests/*' ':!testing/recordings/*'
    ':!testing/repos/*' ':!testing/mirror/*' | awk '{a+=$1} END {print a}'`;
  - skein: `git diff -M --numstat origin/main...HEAD -- crates testing
    ':!*/tests.rs' ':!*/tests/*' | awk '{a+=$1} END {print a}'`.

  `tests/` (worlds), goldens, recordings and the test repositories are
  not counted. Each landing's report gives the number (implementer.md,
  8.6).
- **Within twice its Size** an increment lands without asking; a decoder
  or a document increment (a machine's decoder, a record's codec, a
  schema) within 2.5 times it; a split increment's parts count together.
  Past that, it asks before landing, with the count by file.
