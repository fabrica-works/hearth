# Session 31: the resolution recorded, the indices kept
render.md, section 5. 31.1: the closure is recorded by key (store.md 6.4) and
the spec prints `known`. 31.2: apt's indices are kept in `cache/` by a rename
(6.8), and `--resolve` reports its progress. Needs hearth 24 (24.3: the closure
matched), hearth 33.3, hearth 41.1. The gate runs no mmdebstrap (testing.md 2.7); both prove live.

## Read first
- `render.md` 5, 7.2, 8.2; `store.md` 2.1, 2.2, 6.1, 6.4, 6.5, 6.8, 8.1
- `client/README.md` 3.2; `testing.md` 2.7; `decisions.md` 11, 14;
  `findings/skein-io.md` 2 to 4
- Drafts: 08 (`Pin`, `Mirror`, `Proxy`); 41.1; 09 (kinds 4 and 5, 09.3); 10.3;
  12 (`errors`); 42.1 (`Failure`); 33 ("What hearth 24 calls"); 24 (all)
- skein: `04-store-io.md` (04.1's rename); `programming-model.md` 2, 4.5, 5.2

## What the design asks
- Recording (5.3, store.md 6.4): the closure kept by key, read back whole.
- Keeping (store.md 6.8, 2.2): indices kept in `cache/`, apt's mode, by rename.
- Progress (client/README.md 3.2): standard error, never standard output.
- Bounds (store.md 6.1, 8.1): limits "refused"; a record past 4 MiB.

## Choices this plan makes
- **`made`:** `at` when recorded; tools empty until M2 (later.md); `took`
  spawn to last index read; `costs.fetched` apt's `Fetched <n> <unit>`
  (`kB` 1,000 bytes), 0 when it prints none (render.md 5.4, store.md 6.4).
- **Store failures** (store.md 8.1): a lock or `store.record` that fails ends
  the call before render (75); a page, removal, load, `staging/` walk or keep
  that fails leaves no index and the store refused, naming the step; a
  declined `Record` or `Room` leaves the key `needed`.
- **Room and bound:** no resolution begins without room for one more and its
  indices (store.md 8.1); a record past its bound or a field's limit is
  declined, a `resolution` error, configuration (store.md 6.1, render.md 7.2).
- **Live inputs:** the mirror digest is the pinned `InRelease`'s SHA-256;
  `test-min` is the `apt` variant, no package; apt checks with the host's
  keys, no `--keyring` (render.md 5.2, testing.md 6.2); a pin bump re-pastes
  `TEST_MIN_KEY`.
- **Proxy:** as hearth 24's; `conformance_render.rs` runs mmdebstrap three
  times here, once on a dead port.

## Rules that bind it
- **Meeting point** (README): each increment proves on the real loop or, for
  what runs mmdebstrap, live; a variant added to a vocabulary crossing the
  protocol layer extends `translate/` and its reverse in
  `src/tests/translate.rs`.
- **No expected value computed by the code under test:** literals, by hand
  (`python3 -I`, once), from `sha256sum`, or from what hearth 41 recorded.
- **Shared files** (README): root `Cargo.toml`, `Cargo.lock`, and the files
  other sessions share named in Files below: landing second, rebase and keep
  both.

## Increments
### 31.1 The closure recorded, and printed `known`
Scope: render's `resolution::Record` (store.md 6.4); the root's translation both
ways; the store's `Record` and `Room`; the protocol's write of 09's record; the
spec `known`.
Needs: hearth 24 (24.3); hearth 33.3 (its `store` error, `Progress`); hearth
41.1 (the recorded `stanzas`, for the live test).
Files: render `src/{resolution, keys, call, vocabulary, errors}.rs`,
`src/tests/resolution.rs`; root `src/{resolution, vocabulary, lib}.rs`,
`src/tests.rs`; store `src/{resolutions, vocabulary, lib}.rs`,
`src/tests/{fixtures, find}.rs` (`record` tests there); protocol `src/store.rs`,
`src/translate/{store, errors}.rs`, `src/tests/{store, translate}.rs`; records
`src/errors.rs` (`Step::Write`); `tests/builder/{Cargo.toml (dev:
hearth-recordings), src/fixtures.rs, tests/conformance_render.rs}`.
Shape: the sketches. At `Resolved`, render builds `Record { key, inputs,
sources: lines, closure, digest: keys::closure, made }` and asks `Record`:
`Recorded` → `Known` for every target holding the key; `Failed { Store }` →
`Needed`, `TryAgain`; a closure past `closure` is `TooMany`, no `Record`. Store:
`Record` and `Room` decline `Unusable` with no index, `Full { Resolutions, most
}` at `resolutions`; `Record` also `Bound` when `Unwritable` (09's `Refused`:
Choices); else `Write` (`Store { resolutions.token, <hex>.record, Expect::Any
}`), indexed, `Recorded`; `Room` writes nothing. Root: `Room` before each
`Resolve` (Choices), `Record`'s check the backstop; `Declined { Full | Unusable
}` is 33.3's `store` error once a call and render `Failed { Store }`; `Bound` →
`Failed { TooLarge { limit: Record, max } }`, `resolution` `TooLarge`,
configuration.
Test shape: unit under 20 ms: render
`a_closed_closure_is_asked_recorded_with_its_digest_and_inputs`,
`once_recorded_every_target_holding_the_key_is_known`,
`a_closure_past_its_limit_is_never_recorded`; store
`a_record_is_written_then_indexed`,
`a_record_past_the_resolution_limit_is_declined_and_not_written` (`resolutions:
1`), `a_record_the_writer_refuses_declines_and_the_store_stays_usable`,
`room_follows_the_resolution_limit` (a table: below it `HasRoom`, at it `Full`,
no index `Unusable`); root
`a_declined_record_answers_the_stores_error_once_and_leaves_the_key_needed`,
`a_resolve_goes_down_only_when_the_store_has_room` (a full store: its error
once, nothing down); protocol `a_written_resolution_reads_back_as_the_stores`,
`each_resolution_field_translates_and_back`. Live:
`test_mins_closure_is_recorded_as_mmdebstraps_recording_resolved_it`: 08.3's
`Proxy` over `fixtures::mirror()`, `--resolve test-min`, exit 0; 09's
`resolution::read` of `TEST_MIN_KEY` equals `fixtures::recorded_closure` (41.1's
`stanzas`: `Package`, `Architecture`, `Version`, `Filename`, `Size`, `SHA256`;
`source` 0; sorted by name, architecture), its `sources` line the URI
`http://snapshot.debian.org/archive/debian/20261016T091523Z/` (08.1's pin),
`forky`, `main`; the packages line ends ` known`.
Proof: live, `binary(=conformance_render)` once: `--resolve test-min` records,
by key, the closure 41.1 recorded and prints `known`. Bound: 15 s a test, as
nextest prints it.
Don't: write a record past `resolutions` or its bound; run a `Resolve` without
`Room`.
Size: at most 280 lines.
Removes: 24.3's unrecorded closure and unasked `Room`; 24.1's `Found { digest }`
(now the record whole).
### 31.2 The indices kept, and the resolution's progress
Scope: each index handed to the store as an arrival (`Origin::Index`), kept by a
rename into `cache/` with its kind-5 record (6.8), or refused; `resolving` and
`resolved` progress on standard error (client/README.md 3.2).
Needs: nothing beyond 31.1.
Files: protocol `src/{indices, store, lib}.rs` (`Beneath::Rename`),
`src/translate/store.rs` (`progress` too), `src/tests/{indices, store}.rs`;
service `src/lib.rs` (`Beneath::Rename` by `FileIo::descriptor`),
`src/tests.rs`; root `src/{lib, vocabulary}.rs`, `src/tests.rs`; records
`src/{records, progress}.rs`, `src/form/{text, json}.rs`,
`src/tests/progress.rs`; `tests/builder/tests/conformance_render.rs`.
Shape: the sketches. At an index's `Ended`: `Sync { file }`, `Close`,
`builder::Event::Index`, the next index after the store's terminal; the root
steps `Arrived { origin: Index { resolution: key } }` (33.1). `Keep`:
`Beneath::Rename { from_root: <lists' token>, from, to_root: cache.fd, to: <hex>
}`, `SyncDirectory { cache.fd }`, its record by `Store`; `Discard`: the walk
takes it. A `Refused` index: the root withholds `Resolved`, gives render `Failed
{ Store }`, records nothing. `Progress::Resolving` at a `Resolve`, `Resolved {
key, took }` or `Unresolved` at its terminal; text `resolving <key12>`,
`resolved <key12> in <ms> ms` (whole ms, truncated), `resolution <key12>
failed`; JSON (`key` 64 hex) `{"record":"resolving","key":…}`,
`{"record":"resolved","key":…,"took_ms":…}`, `{"record":"unresolved","key":…}`
(the plan's wording, within render.md 7.3's forms).
Test shape: unit under 20 ms: protocol
`a_kept_index_is_synced_renamed_across_then_cache_synced_then_recorded`,
`a_discarded_index_is_left_for_the_walk`; service
`a_beneath_rename_reaches_io_with_both_descriptors`; root
`a_refused_index_fails_its_resolve_and_nothing_is_recorded`,
`a_resolve_says_it_starts_and_ends`; records
`resolution_progress_prints_as_text_and_json_lines`. Live:
`test_mins_indices_enter_the_cache_by_their_digests_with_their_records`: the
record's `InRelease` holds the pin's `digest`, `main/binary-amd64/Packages` the
`skein_lib::sha256::digest` of `Mirror::index("main")`; each is `cache/<hex>`,
mode 0644 (store.md 2.2: apt's, kept by the rename), beside a kind-5 record
(09's `cache_file::read`) of origin `Index { resolution: TEST_MIN_KEY }`; and
`a_resolution_says_on_standard_error_that_it_starts_and_ends` (dead proxy:
`resolving <key12>` then `resolution <key12> failed` on standard error only).
Proof: live, `binary(=conformance_render)` once: both indices are in `cache/` by
digest, 0644, each beside a kind-5 record, by rename. Bound: 15 s a test, as
nextest prints it.
Don't: copy an index; record one before the rename and `cache/`'s sync.
Size: at most 220 lines.
Removes: 24.3's indices removed with the scratch.

## Interface sketches
`B` is `Box<[u8]>`. Hearth 24's types keep their variants, these added.
```rust
// crates/hearth-domain-render/src/vocabulary.rs
pub enum Event { Recorded { request: Token } } // Record; `Found` carries the record whole
pub enum Request { Record { request: Token, resolution: resolution::Record } }
// crates/hearth-domain-render/src/resolution.rs (31.1)
pub struct Made { pub at: Wall, pub tools: Tools, pub costs: Costs }
pub struct Record { pub key: Digest, pub inputs: parts::packages::Inputs, pub sources: Box<[Line]>, pub closure: Box<[Package]>, pub digest: Digest, pub made: Made }
// crates/hearth-domain-render/src/errors.rs: `Resolution` gains `TooLarge { max: u64 }` (the record past kind 4's body); `Limit` gains `Record`
// crates/hearth-domain-store/src/vocabulary.rs: hearth 23's, 33's and 24's, these added
pub enum Event {
    Record { resolution: Resolution }, Room, // the root's
    Unwritable,                              // `Write`: 09's writer refused the record
}
pub enum Request { Write { resolution: Resolution } } // by `Store`, `Expect::Any`
pub enum Answer {
    Recorded, // Record
    HasRoom,  // Room: fewer than `resolutions` indexed
}
pub enum Decline { Bound } // `Declined` also when `Unwritable`
pub enum Step { Write }     // `Unusable`'s; the records' `Step` gains it too (31.1)
// crates/hearth-domain-builder/src/vocabulary.rs
pub enum Event { Index { request: Token, arrival: Token, sha256: Digest, size: u64, took: Duration } } // 31.2
pub enum Progress { Resolving { key: Digest }, Resolved { key: Digest, took: Duration }, Unresolved { key: Digest } } // 31.2
// crates/hearth-domain-builder/src/resolution.rs: `stored(Record) -> store::Resolution`; 24.1's `found` returns the `Record` (31.1)
// crates/hearth-protocol-builder/src/lib.rs (33.2's `Beneath`)
pub enum Beneath { Rename { owner: Token, from_root: Token, from: B, to_root: Fd, to: B } } // 31.2
// src/translate/store.rs: `tree_resolution`, the reverse of 24.1's `resolution` (31.1); `progress` (31.2)
// hearth-client-protocol-records: progress.rs's `Record::{Resolving, Resolved, Unresolved}` (31.2)
pub struct Resolving { pub key: [u8; 32] } pub struct Unresolved { pub key: [u8; 32] }
pub struct Resolved { pub key: [u8; 32], pub took: Duration }
```

## Questions to raise when reached
None.

## Done when
1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-31\.[1-2]'` prints 2.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run -p
   hearth-domain-render -p hearth-domain-store -p hearth-domain-builder -p
   hearth-protocol-builder -p hearth-client-protocol-records -p
   hearth-builder-world -p hearth` passes.
3. Once, on the run's machine, in that worktree: `heavy cargo nextest run -p
   hearth-builder-world --profile conformance -E 'binary(=conformance_render)'`
   passes, its `Summary` naming 5 tests, within the profile's 2 minutes.
