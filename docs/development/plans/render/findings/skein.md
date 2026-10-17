# Findings: skein at the plan's base

The code audit of what the plan's skein increments touch (development.md,
5.4), at skein `de0f5ba`. Paths are skein's unless named. Line numbers
were read with grep and reads on 2026-10-16.

## 1. SHA-256

On skein's `main` already: temper's forge-setup plan landed lib.md, 6.2
(skein `0f093bb`, `8a51e10`, `f981746`), so this plan has no SHA-256
session. Read at `f981746` on 2026-10-17.

- **In `skein-lib`:** `skein_lib::sha256`, `crates/skein-lib/src/sha256.rs`:
  `Sha256` (`new`, `update(&[u8])`, `finish() -> [u8; 32]`, `Clone`),
  `digest(&[u8]) -> [u8; 32]`, `Hmac` and `equal`. `skein-lib` is
  `no_std`, `forbid(unsafe_code)` (`lib.rs:13-16`), depends on nothing.
- **`skein-io`'s `Digest` is over it,** `crates/skein-io/src/digest.rs`:
  `pub struct Digest(pub [u64; 4])` (:7), the digest's 32 bytes read as
  four big-endian words by `Digest::from_bytes` (:11), and `pub fn
  digest(&[u8]) -> Digest` (:22); the store's check holds a `Sha256`
  (`file_layer/store.rs`). Its users outside `skein-io` (`grep -rln
  'skein_io::digest\|digest::digest\|digest::Digest'` over `crates
  testing tests`): skein 5 files, smith 9, jig 11, temper 6. Several use
  `Digest`'s `.0` words (smith `smith-protocol-machine/src/files.rs:44`,
  jig `jig-protocol-engine-clients/src/sign_in.rs:72`).
- **PKCE and `skein-ui-bundle` use lib's** (`crates/skein-oauth/src/pkce.rs:16`,
  `crates/skein-ui-bundle/src/digest.rs:18`): skein holds one SHA-256.
- A domain may depend on lib only (programming-model.md, 4.5), so
  hearth's domains use `skein_lib::sha256`, not skein-io's.

## 2. skein-json's tokenizer

- **The crate:** `crates/skein-json`, `no_std` with alloc, depends on
  skein-lib only (`lib.rs:17-20`). Design: skein's `json.md`, section 3
  (the tokenizer; `Request`/`Event`/`Token` at :74-98, errors at :129),
  section 5.1 (the collector). json.md says nothing of positions, lines
  or columns.
- **A pull machine** (`src/tokenizer.rs`): `Tokenizer::new(&Limits)`
  (:279), `up(.., ev: stream::Up, above: &mut Queue<Event>, below: &mut
  Queue<Down>)` (:353), `down(.., rq: Request, ..)` (:388). Bytes arrive
  as `Up::Bytes`, each answering its own demand (`Fill(1)` between
  tokens, `Scan { until: '"', .. }` for string text).
- `Request` (:188): `Next`, `Text(u32)`, `Skip`, `Close`. `Event` (:202):
  `Token(Token)`, `Long(u64)`, `Skipped(u64)`, `Done`, `Failed(Error)`,
  `Closed`. `Token` (`src/token.rs:11`): `ObjectStart`, `ObjectEnd`,
  `ArrayStart`, `ArrayEnd`, `Key(Box<[u8]>)`, `String(Box<[u8]>)`,
  `Number(Box<[u8]>)`, `True`, `False`, `Null`; strings unescaped and
  checked UTF-8, numbers the validated text. `Error` (:221, `Copy`):
  `Unexpected`, `Trailing`, `TooLong`, `TooDeep`, `StringTooLong`,
  `NumberTooLong`, `Number`, `Escape`, `Surrogate`, `Utf8`, `Control`,
  `Truncated`, `Stream(Fault)`.
- **No public position.** `Document.read` (:508) counts delivered bytes
  (u32, from 0); `token_start`, `token_end` (:509-510) are exposed only
  as `pub(crate) token_wire()` (:308), which the collector uses
  (`collector.rs:786, 992, 1007`). No line or column is tracked. A raw
  newline occurs only in whitespace between tokens (strings refuse it as
  `Control`), read a byte at a time.
- **Duplicate keys** are refused only in a stored `Keep::Into` frame for
  the keys its filter names (`collector.rs:808-819`, `fn duplicate`
  :1098) and in a tagged object (`collector/tagged.rs:131-143`); under
  `Keep::Value` they pass (`tests/collector.rs:123-140`). jig refuses them
  itself (jig `crates/jig-protocol-config/src/read.rs:86`).
- **Users of `Token`, `tokenizer::Event` or `tokenizer::Error`** that a
  change inside those types would break: in skein-json, 10 files; in
  skein's other crates and worlds, 23 (skein-llm 4, skein-oauth 2,
  skein-fake-oauth 1, `tests/http` 1, `tests/protocol` 4, `tests/json`
  11); smith 3 (`smith-events/src/read.rs`,
  `smith-protocol-machine/src/search.rs`,
  `tests/protocol-llm/src/translate.rs`); jig 3
  (`jig-protocol-tools/src/json.rs`, `jig-protocol-config/src/document.rs`,
  `tests/ops/src/protocol/json.rs`); temper 5
  (`temper-engine-smith/src/json.rs`, `temper-forge-forgejo/src/decode.rs`,
  `decode/walk.rs`, `document.rs`,
  `temper-engine-protocol-forge/src/connection.rs`).
- **Tests:** step tests `crates/skein-json/src/tests/{tokenizer,collector,document,text,writer}.rs`;
  the world `tests/json` (`skein-json-world`): `src/world.rs`,
  `src/reference.rs` (a reference parser, no positions),
  `src/generate.rs`, `src/transcript.rs`; focused
  `tests/{world,writer,transcripts,tagged,memory}.rs`; fuzzy
  `tests/fuzzy_{documents,writer,collector}.rs`; 39 transcript pairs in
  `tests/json/transcripts/`, which record no positions. No cargo-fuzz
  target (json.md lists `fuzz/` as not built).

## 3. A digest of a world's state

- **Specified in one line:** skein's `lib.md`, 13 (not built yet), "a
  fixed-key hasher over the state types' derived `Hash`, independent of
  their layout in memory"; `simulator.md`, 243 ("a state digest in the
  trace, beside the records"; it cites "lib.md, 12" for an item of
  section 13); `testing-strategy.md`, 390-391; skein's `testing.md`, 294
  ("No state digest yet") and 333; skein's `STATUS.md`, 86-88 ("waits for
  a plan"; it cites simulator.md, section 5 for what is in its section 7).
- **Today replay compares traces:** `Outcome.trace: Vec<Entry>`
  (`testing/skein-world/src/world.rs:113-121`); `skein_sim::trace::Entry`
  derives `Hash` (`crates/skein-sim/src/trace.rs:13-14`); domain worlds
  call `assert_replays(seed, other, run)`, comparing a `Vec<String>`
  trace and an end value `T: PartialEq`
  (`testing/skein-world/src/domain/trace.rs:36`); tests compare two runs
  of a seed (`tests/world/tests/world.rs:110`,
  `tests/json/tests/world.rs:146`).
- **lib's containers derive only `Debug`:** `Slab` (`slab.rs:20`), `Map`
  (`map.rs:23`), `Set` (`set.rs:16`), `Queue` (`queue.rs:11`), `Stack`
  (`stack.rs:12`), `Deadlines` (`deadlines.rs:20`). `Id` (`id.rs:86`),
  `List` (`list.rs:143`), the time types and `Token` derive `Hash`.
- **Where a digest could attach** (not specified): the `Host` trait
  (`crates/skein-shell/src/drive.rs:12`), `Machine`
  (`testing/skein-world/src/program.rs:90`), `Outcome`, `assert_replays`.

## 4. skein's gate on x1

- `scripts/check.sh` (31 lines): fmt; clippy; wasm32 checks and clippy
  of the ui crates; `taste.sh`; `cargo check -p skein-tls`; nextest
  focused, then `--profile fuzzy`; the wasm bundle and the example
  server; then `--profile browser`, whose filter is
  `(package(skein-browser-world) or package(skein-ui-web-tests)) and
  binary(/^browser_/)`, one thread (`.config/nextest.toml`).
- **check.sh takes no argument, variable or filter.** `land --gate
  COMMAND` runs `bash -c "$COMMAND"` from the repository's root
  (`scripts/dev/land:23,65,74-75,196`); the gate's cache key includes the
  command.
- `skein-ui-web-tests::browser_harness`
  `a_page_of_another_origin_is_opened_by_its_step`
  (`tests/ui-web/tests/browser_harness.rs:46`) fails on x1 with Chromium
  147 (an unexpected 404 logged during its visit) and passes on
  carbon-forge. de0f5ba landed with check.sh's commands but that test,
  from a copy of check.sh whose browser run adds `-E 'not
  test(=a_page_of_another_origin_is_opened_by_its_step)'` (its log:
  "37 passed, 1 skipped").
