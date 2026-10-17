# Session 33: `hearth-domain-store`: cache files enter, `resolutions/` scanned, what the store says

The rest of M1's store (store.md, 2.4), on hearth 23's index and io. 33.1:
every file enters the cache through the store, recorded after its rename,
refused past the limit (6.5, 6.8, 8.1); 33.2: `resolutions/` scanned, each
key recomputed, and `staging/` emptied; 33.3: the wait, the strays and the
store's errors said. It ends hearth 11's "cache file without its kind-5
record". hearth 24 and 31 write resolutions on this index and io (Choices,
"What hearth 24 calls").

## Read first

- hearth's `store.md`: 2.1, 3.1, 4.4, 6.1, 6.4, 6.5, 6.8, 7.5, 8.1, 8.2;
  `render.md`: 5.1 (the key), 7.1, 7.2, 8.1, 8.2 ("Resolution records");
  `client/README.md`: 3.2 ("Progress goes to standard error"), 3.3, 3.5.
- skein's `io.md`: 5 ("Removing", "Replacing a file, in order"), 9 ("Roots by
  token everywhere"); `programming-model.md`: 4.4, 6.3, 9.
- Drafts: `hearth/23-store.md` whole; `hearth/09-records.md` (kinds 4 and 5,
  09.3); `hearth/11-thin-story.md` (32.1's cache); `hearth/01-lib.md`
  (`H`, `Encoder::item`); `hearth/10-client.md` for 33.3 (`records.rs`, `form`,
  `printer`) and `40-client-diff-errors.md` (40.2's `errors.rs`);
  `findings/skein-io.md`, 3 and 4.
- The pattern for roots by token: smith's `Below::OpenDirectory`
  (`crates/smith-protocol-machine/src/boundary.rs:56`), resolved by its service
  with `FileIo::descriptor` (`crates/smith-agent-service/src/service.rs:2188`).

## What the design asks

- **A cache file** (6.8, 6.5): hashed as written, synced, renamed to
  `cache/<sha256>`, then its record `{ kind: "blob", oid }`; one present is not
  written again and "keeps its first record". A file that would pass 8.1's limit
  is refused and removed; nothing is evicted.
- **The scan** (8.2, 6 and 7; 4.4, 6.4): "each key recomputed, a resolution's
  among them", the index one per resolution by key (1,024); 4.4 deletes
  "anything in `staging/`" and "a record whose key does not [recompute]".
- **Saying** (8.2; 8.1): the wait said "once, after its first second", strays
  reported; a full store answers an operational error naming the store, the
  limit and what passed it (render.md, 7.2); client/README.md 3.2: progress and
  a store it cannot open go to standard error.

## Choices this plan makes

- **Every file enters the cache through the store** (33.1): the protocol layer
  syncs the temporary (32.1), sends `Arrived` and reads no further; the store
  answers `Keep` (rename, sync `cache/`, record) or `Discard`. The index, not a
  probe of `cache/<sha256>`, says what is present. `at` is the wall time of
  `Keep`; `took` runs from the blob's first piece asked to its temporary
  synced. hearth 15's rendered files are arrivals with `Origin::Rendered`:
  whichever of hearth 15 and 33.1 lands second routes them so.
- **A refused blob fails its read once:** the root withholds the read's later
  `Cached` and gives render `Failed { class: Operational }` at its last blob
  (`Arrived::last`; hearth 11 runs one read at a time). A refusal while running
  keeps the index: present files stay usable.
- **A resolution's record** (33.2) reaches the store as its `key`, `inputs`,
  `closure` (four fields a package) and `digest`; it is indexed when its name,
  its `key` and `H::new(b"resolution", 4)` over its inputs (each through
  `Encoder::item`: render.md, 5.1's order) agree and its `digest` is `H("closure",
  list [ list [ name, architecture, version, sha256 ] … ])` over its closure
  (store.md 6.4: the scan reads the record whole), else removed. One it cannot
  read is treated as absent (6.8): left, not indexed, not counted (6.1, 6.8).
- **The store's failures** (store.md 8.1; render.md 7.2's `store` kind): a
  page, removal, load, `staging/` walk past its bounds or keep that fails
  leaves no index, every request that reads or adds refused as when full,
  the error naming the store and the step; a page is not listed again (io.md
  5). The store's error is kind `store`, operational, once a call, with the
  wait and the strays on standard error (8.2; client/README.md 3.2).
- **The walk of `staging/`** (decisions.md, 11), `src/walk.rs`: a `lib::Stack`
  of open directories (depth 16; a resolution's deepest,
  `staging/<key>/root/var/lib/apt/lists/partial`, is 7), 64 entries a page,
  files and links removed, a directory removed from its parent after its last
  page, at most 65,536 entries. Subdirectories are open by token, so its
  `OpenDirectory` and `Remove` cross as `Asked::Beneath`, which the service
  resolves with `FileIo::descriptor` (io.md, 9's "Roots by token everywhere" is
  not built; smith's pattern, Read first).
- **What hearth 24 and 31 call:** 24.1 adds `Find { key }` (the index, then
  33.2's `Load`, whole); 31.1 `Record { resolution }` and `Room` (`Store`,
  refused past `resolutions`); 31.2 kept indices as arrivals with
  `Origin::Index` (renamed across by skein 04.1, then 33.1's sync and record);
  24 removes `staging/<key>/` with 33.2's walk, writes
  `translate/resolution.rs`.

## Rules that bind it

- **The meeting point** (README, Rules): each increment proves on the real loop,
  `hearth render` through real git in `tests/builder`; the store's own world is
  M2's (store.md, 11).
- **Shared files** (README, Rules): the root `Cargo.toml`, `Cargo.lock`;
  `crates/hearth-protocol-builder/src/translate/`; the records' `records.rs`
  (33.3); `tests/builder/src/fixtures.rs`; the root's `vocabulary.rs` and
  `lib.rs`, the protocol's and the service's `lib.rs`, `crates/hearth/src/lib.rs`
  (hearth 12, 15, 24, 25 too). Landing second, rebase and keep both.
- **Literals:** digests, object ids and keys in tests come from `sha256sum`,
  `git hash-object` or 3.1's table by hand (`python3 -I`), pasted once.

## Increments

### 33.1 Cache files enter through the store, with their records
Scope: each blob streamed into the cache (32.1) enters by the store: kept
(renamed, `cache/` synced, its kind-5 record written by `Store`), discarded
(indexed already), or refused (full, at the start or now), its read then
failed; `translate/store.rs`'s cache file.
Needs: hearth 23 (23.2's index and scan; 23.1's `Rename` form in `cache.rs`).
Files: `crates/hearth-domain-store/src/{lib, vocabulary, index, start,
scan}.rs` (`start.rs:9` and `scan.rs:61` match `Event` whole: each gains
`Arrived`'s arm), new `src/enter.rs`, `src/tests/{fixtures, enter}.rs`;
`crates/hearth-domain-builder/src/{lib, vocabulary}.rs`, `src/tests.rs`;
`crates/hearth-protocol-builder/src/{cache, read, store}.rs`,
`src/translate/store.rs`, `src/tests/{cache, store, translate}.rs`;
`tests/builder/tests/real.rs` (32.1's
`static_files_enter_the_cache_by_sha256_and_no_temporary_stays`: its listing
gains the two records), `tests/store.rs`.
Shape: the protocol, at a blob's temporary synced and closed, emits
`builder::Event::Arrived { request, oid, size, last, arrival, sha256, took }`
and reads no further; the root keeps `(request, oid, size, last)` for
`arrival` and steps the store's `Arrived { origin: Blob { oid: oid.0 } }`. The
store: no index (`Full`, `Unusable`) → `Discard`, then `Refused`; indexed →
`Discard`, then `Cached`; at `cache_files` → `Discard`, then `Refused`; else
inserted, `Keep { arrival, record }` with `at: env.wall`, then `Cached` on
`Done`, or, on `Failed`, the entry removed, `Refused`, and the store unusable
(store.md 8.1). `Keep`: `Rename { from_root: cache.fd, from:
<temporary>, to_root: cache.fd, to: <sha256 hex> }`, `SyncDirectory {
cache.fd }`, then `Store { root: cache.token, path: <hex>.record, expected:
Expect::Any }` of `cache_file::write`; `Discard`: `Remove` of the temporary,
except an `Origin::Index`'s, left for the walk (31.2).
Either's terminal frees the read for its next blob. The root gives render
`Cached` for a kept or discarded blob, holds a refused read's request, drops its
later answers, and gives `Failed { class: Operational }` at its `last`.
Test shape: unit as hearth 23's. Reference, in `src/tests/enter.rs`
(`fixtures::ready(limits, &[digests])` is a store whose index holds them;
`fixtures::blob(arrival, sha256)` an `Arrived` of `/etc/motd`'s 15-byte blob,
taking 3 ms; `ARRIVAL` is `Token::new(1)`; `MOTD` is a `Digest` of the 32 bytes
of THIN's `775324fc…3123` below, written once in `src/tests/fixtures.rs` as
bytes; the sketch as rustfmt wraps it at 120 columns, ruling 139):

```rust
#[test]
fn a_new_file_is_kept_with_its_record_and_answered_cached() {
    let mut store = fixtures::ready(limits::SHIPPED, &[]);
    let wall = Wall::from_nanos(1_760_000_000_000_000_000);
    let outputs = store.step_at(Time::ZERO, wall, fixtures::blob(ARRIVAL, MOTD));
    let record = CacheFile {
        at: wall,
        origin: Origin::Blob { oid: Box::from(&b"d23c59a84714a16a9597f6d92ce69390d6d13f9b"[..]) },
        sha256: MOTD,
        size: 15,
        took: Duration::from_millis(3),
    };
    assert_eq!(
        &*outputs,
        &[Output::Request(Request::Keep { arrival: ARRIVAL, record })],
        "6.8: renamed, then recorded"
    );
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Answer(Answer::Cached { arrival: ARRIVAL })]);
    assert!(store.indexed(&MOTD), "8.1: one entry per cache file, by digest");
}
```
Then `a_file_already_indexed_discards_its_temporary`,
`a_file_past_the_cache_limit_is_refused_and_discarded`,
`a_full_store_refuses_every_arrival`; root:
`a_refused_blob_fails_its_read_at_its_last_blob`; protocol:
`a_kept_arrival_is_renamed_then_cache_synced_then_recorded`,
`the_next_blob_waits_for_the_stores_answer`,
`each_cache_file_field_and_origin_translates_and_back`. Real loop, with THIN's
literals (`775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123`,
15 bytes, blob `d23c59a84714a16a9597f6d92ce69390d6d13f9b`;
`f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745`, 29 bytes,
blob `71318968f52ccfeabf6edd13442c868eaa35e49d`):
`each_static_file_enters_the_cache_with_its_record_naming_its_blob` (each
record read by `cache_file::read`: its sha256, size and origin equal the
literals, `at` and `took` non-zero),
`a_file_already_in_the_cache_keeps_its_first_record` (two renders: the
record's bytes equal), `a_store_past_its_cache_limit_refuses_every_new_file`
(`cache_files: 2`, three recorded files laid: exit 75, no spec, the laid files
as they were, neither of THIN's in `cache/`),
`a_cache_that_fills_while_rendering_removes_the_file_past_it` (`cache_files:
1`: `/etc/motd`'s file and record kept, the other's absent, exit 75).
Proof: process, `heavy cargo nextest run -p hearth-builder-world -E
'test(=each_static_file_enters_the_cache_with_its_record_naming_its_blob)'`:
through real git's `cat-file`, each static file of THIN is in `cache/` by its
SHA-256 beside a kind-5 record naming its blob. Bound: 0.5 s.
Don't: probe `cache/<sha256>`; write a record before the rename and the
directory's sync; hold a second blob while the store decides; evict.
Size: at most 240 lines (280 if it lands after 15.3: `Event::Produced` routed
as 15.3 says, `tests/store.rs`'s test).
Removes: hearth 11's "the cache file without its kind-5 record" (its
Choices, "Transitional"), and 32.1's probe of `cache/<sha256>`.

### 33.2 `resolutions/` in the scan, and `staging/` emptied
Scope: step 6 for `resolutions/`: each record loaded, its key recomputed,
indexed or removed, its temporaries removed, strays counted, past
`resolutions` full; 4.4's "anything in `staging/`" removed by the walk;
`Asked::Beneath` and the service's resolution of it.
Needs: nothing beyond 33.1 (hearth 09.3's `resolution::read` and `Inputs` are
in hearth 23's Needs).
Files: `crates/hearth-domain-store/src/{scan, index, vocabulary}.rs`, new
`src/resolutions.rs`, `src/tests/{fixtures, scan}.rs`;
`crates/hearth-protocol-builder/src/{lib, limits, store}.rs`,
`src/store/names.rs`, new `src/walk.rs`, `src/translate/store.rs`,
`src/tests/{store, translate, fixtures}.rs` (`fixtures.rs:288` matches
`Asked::Io(_) | Asked::File(_)`: it gains `Asked::Beneath`'s arm);
`crates/hearth-service-builder/src/lib.rs`
(`Asked::Beneath`), `src/tests.rs`; `tests/builder/{src/fixtures.rs,
tests/store.rs}`.
Shape: the scan's order is the top, `resolutions/`, `cache/` twice, `Empty`,
the settlement. In `resolutions/`: `Record(k)` → `Load { key: k }`:
`Resolution { key, inputs, closure, digest }` with `key == k`,
`resolutions::key(&inputs) == k` and `resolutions::digest(&closure) == digest`
→ indexed (past `resolutions`: `Full { limit: Resolutions }`), else `Remove {
Record(k) }`; `Unreadable` → nothing; `<64 hex>.record.skein-<16 hex>` a
`Temporary`. The protocol: `Load { root: resolutions.token, path, max:
u32::try_from(tree::frame::largest(tree::frame::Kind::Resolution,
&limits.tree)) }` (`largest` takes the kind and the tree's limits and returns
a `u64`, tree `frame.rs:113`; protocol `store.rs:58` converts `Kind::Store`'s
so), read by `resolution::read` (`Unreadable`
or `TooLarge` → `Unreadable`), its `inputs` translated. `Empty` → the walk from
`OpenDirectory { root: staging.fd, name: b"." }`, `staging/` itself kept; past
`depth` or `entries`, `Failed`.
Test shape: unit as before; `fixtures::inputs()` is `sources: map {}, request:
list [ bytes "hello" ], variant: bytes "apt", apt: map {}`, whose key, by 3.1's
table and `sha256sum` once, is
`01ea811afc99308a9fcc336c944966c7f9dd98857b27592fe00a1fb84bd49d00`:
`a_resolution_whose_key_and_digest_recompute_is_indexed`,
`a_resolution_whose_key_does_not_recompute_is_removed`,
`a_resolution_whose_digest_does_not_recompute_from_its_closure_is_removed`,
`the_closure_digest_is_h_over_each_packages_four_fields` (`fixtures::closure()`,
one package `apt` `amd64` `3.0.3` `01…01`: `f2fbea8ecfaf5c1ee1d15463c1acbc34d49ad2a0e572440f7cbf345fc0ae7ee3`;
the empty closure's `c1589a971c4ce5042b7a8c0f5c9af69b3d46fe64b8ab4cdaac585db0ea3ad530`),
`a_resolution_named_other_than_its_key_is_removed`,
`an_unreadable_resolution_is_left_and_not_counted`,
`a_resolution_past_the_limit_makes_the_store_full`; protocol:
`the_walk_removes_a_directorys_entries_then_the_directory`,
`a_staging_tree_past_the_walks_depth_fails_the_walk`,
`a_staging_tree_past_the_walks_entry_limit_fails_the_walk` (`entries: 4`),
`each_inputs_value_translates_and_back`; service:
`a_beneath_request_reaches_io_with_its_roots_descriptor`. Real loop:
`the_scan_removes_resolutions_whose_keys_or_digests_do_not_recompute` (laid by
`resolution::write`, which refuses a package whose `source` is not below
`sources.len()` (tree `resolution.rs:316`), so each record has one tree
`Line` in `sources` (one that `resolution::write` accepts) and the closure
of one tree `Package`, new in `tests/builder/src/fixtures.rs`: `apt`,
`amd64`, `3.0.3`, source 0, any filename and size, sha256 `01…01`, whose four
fields are `fixtures::closure()`'s, so its digest is `f2fbea8e…7ee3` above;
each with that closure and its digest unless said: `<K>.record` with `fixtures::inputs()`, K its key above; `<D2>.record`
(D2 `02…02`) of `variant: "minbase"` and key D2; `<D3>.record` (`03…03`) of
`inputs()` and key K; `<K2>.record` (K2 `a33fbba1e283ca942abc7b2376b18e4871d28e766ee1a0f1e90470d7b82e21bb`,
the key of `variant: "minbase"`) with digest `05…05`; `<D4>.record` (`04…04`)
a frame of version 2; `<K>.record.skein-0123456789abcdef`; `junk`; after a
render, `resolutions/` lists `<K>.record`, `<D4>.record` and `junk`),
`the_scan_empties_staging_and_keeps_it` (laid: `staging/<64 hex>/root/var/lib/apt/lists/{partial/, x}`, a symbolic
link; after a render `staging/` exists, 0700, and lists nothing).
Proof: process, `heavy cargo nextest run -p hearth-builder-world -E
'test(=the_scan_removes_resolutions_whose_keys_or_digests_do_not_recompute)'`:
the real loop keeps a resolution whose key and digest recompute and removes
those whose key or digest does not. Bound: 0.5 s. Nothing in M1 reads the
index of resolutions until hearth 24's `Find`.
Don't: hold more than one record; check the closure's order beyond hearth
09's reader; remove `staging/` itself; follow a link.
Size: at most 270 lines.
Removes: nothing.

### 33.3 What the store says: its wait, its strays, a full store's error
Scope: the wait said once, `said` after the first `Busy` (8.2, step 1); the
strays reported once a scan (8.2, step 7); both on standard error as records
(client/README.md, 3.2, 3.5); the store's path in an unopened store's error,
which hearth 23 already prints alone on standard error (3.2); a full or
unusable store's operational error, once a call, in the answer (8.1;
render.md, 7.2).
Needs: hearth 12 (the root's `Answer::Error`, `translate/errors.rs`, and 40.2's
error records through them).
Files: `crates/hearth-domain-store/src/{lib, start, scan, vocabulary,
limits}.rs` (`vocabulary.rs:25`: `Answer` drops `Copy`, which `Strays {
first: Stray }` cannot derive; `lib.rs:37`'s `Deadlines::with_capacity(1)`,
`start.rs:15`'s `.expect("one retry")` and `limits.rs:24`'s `worst_case(1)`
take two timers, `Retry` and `Said`), `src/tests/{start, scan, fixtures}.rs`;
`crates/hearth-domain-builder/src/{lib,
vocabulary}.rs`, `src/tests.rs`; `crates/hearth-domain-render/src/errors.rs`;
`crates/hearth-client-protocol-records/src/{records, errors}.rs`,
`src/form/{text, json}.rs` (shared), new `src/progress.rs`, new
`src/tests/progress.rs`; `crates/hearth-protocol-builder/src/{lib, client,
store}.rs` (`store.rs:155`'s `startup` takes the store's path),
`src/translate/{store, errors}.rs`, `src/tests/{client, translate, store,
fixtures}.rs` (`store.rs:54-58`'s Startup lines gain the path;
`fixtures.rs:70`'s `Protocol::new(.., roots(), 7)` gains `store`);
`crates/hearth-service-builder/src/lib.rs` (`adopt_error_pipe`,
`adopt_error_file`, `Configuration::store`; `Configuration` drops `Copy`,
`lib.rs:21`, which `store: Box<[u8]>` cannot derive), `src/tests/fixtures.rs`
(`:7`'s `Configuration { limits, uid }` gains `store`); `crates/hearth/src/lib.rs`
(standard error adopted; the store's path); `tests/builder/{src/fixtures.rs,
tests/store.rs}` (`Rendered::stderr`, `Release::OnNotice`).
Shape: the first `Busy` also arms `Timer::Said` at `now + said`, which fires
`Answer::Waiting` once and is cancelled by `Locked`; the settlement's last step
answers `Strays { count, first }` before `Ready` when `count > 0`. A step
may so emit two outputs: the store's `MAX_OUT` (`lib.rs:13`) and the root's
`MAX_OUT` and `MAX_OUT_RESUME` (builder `lib.rs:11-12`) become 2, their doc
comments say two, and their queues (builder `lib.rs:34`, `:36`) and the
store harness's (`tests/fixtures.rs:21`, `:26`) take that capacity, its
`collect` (`:41`) each output. The root
sends both as `Request::Progress`; the protocol prints each through a second
10.2 `Printer` on standard error, in the call's form, `Finish`ed with standard
output's. The root keeps the store's `Full` or `Unusable` and answers one
`Answer::Error` of the kind `store` (class operational; render's `errors.rs`
gains `Kind::Store`, mirrored below) before render's first answer; its arm in
`translate/errors.rs` names the store's path. An unopened store stays as
hearth 23 landed it (ruling 142, point 4): the root's `Request::Unopened { to,
why }` (builder `vocabulary.rs:20`) and `store::vocabulary::Why`, the
protocol's `Record::Startup` (protocol `lib.rs:195-206`), which the shell
writes to standard error (`take_startup`, `crates/hearth/src/lib.rs:115-116`),
exit 75 (`Outcome::TryAgain`). 33.3 only fills its `path: Some(store)`
(protocol `store.rs:175`, from `Protocol::new`'s `store`), so the line reads
`hearth: cannot open the store <store>: <error>` (records `startup.rs:45-50`),
its errors `lock failed`, `store.record failed` and `store.record is uid
<recorded>'s, not uid <builder>'s` as landed. Lines (store.md 8.2;
client/README.md 3.2: JSON on standard error has its own version line; strays
one notice, the count and the first):
`waiting for the store's lock: <store>`; `strays in the store <store>: <count>,
the first <directory>/<name>`; `the store <store> is full: <most> <cache files
| resolutions>`; `the store <store>: its <step> failed` (the steps of
`Unusable`: `List`, `Remove`, `Load`, `Empty`, `Keep`); JSON
`{"record":"waiting","store":…}`,
`{"record":"strays","store":…,"count":…,"first":…}`, and the error's
`"kind":"store"` with `store`, `limit`, `most`.
Test shape: unit as before; records', in `src/tests/progress.rs`:

```rust
#[test]
fn the_waiting_notice_prints_as_text_and_as_json_lines() {
    let record = Record::Waiting(Waiting { store: Box::from(&b"/home/u/.cache/hearth/store"[..]) });
    let text = form::print(Form::Text, &record, None, &limits::SHIPPED);
    assert_eq!(
        text.as_deref(),
        Ok(&b"waiting for the store's lock: /home/u/.cache/hearth/store\n"[..]),
        "8.2: said once"
    );
    let json = form::print(Form::JsonLines, &record, None, &limits::SHIPPED);
    let expected = concat!(
        r#"{"record":"version","client":1}"#,
        "\n",
        r#"{"record":"waiting","store":"/home/u/.cache/hearth/store"}"#,
        "\n"
    );
    assert_eq!(json.as_deref(), Ok(expected.as_bytes()), "client/README.md, 3.2: progress as records too");
}
```
Then `the_strays_notice_gives_the_count_and_the_first`,
`the_store_error_names_the_store_and_the_limit`; store:
`the_wait_is_said_once_a_second_after_the_first_busy`,
`a_lock_taken_within_the_second_says_nothing`,
`strays_are_said_once_with_their_count_and_the_first`; root:
`a_full_store_is_answered_once_before_renders_answers`; protocol:
`progress_prints_on_standard_error_and_answers_on_standard_output`,
`an_unopened_stores_startup_line_names_the_store` (`src/tests/store.rs:54-58`'s
rows, with the path). Real
loop (`said` 100 ms, `retry` 20 ms, each named so):
`a_render_waiting_past_its_notice_says_once_that_it_waits` (`Release::OnNotice`:
standard error holds the one line, standard output THIN's),
`a_lock_released_before_the_notice_says_nothing`,
`a_stray_in_the_cache_is_reported_once_on_standard_error`,
`a_full_store_answers_its_error_naming_the_store_and_the_limit`,
`a_store_recorded_by_another_uid_is_refused_on_standard_error` (its record
written with the test's uid plus one: standard output empty, standard error the
one line `hearth: cannot open the store <store>: store.record is uid <uid+1>'s,
not uid <uid>'s`, exit 75).
Proof: process, `heavy cargo nextest run -p hearth-builder-world -E
'test(=a_render_waiting_past_its_notice_says_once_that_it_waits)'`: a second
builder over a held store says on standard error, once, that it waits, then
prints THIN's spec on standard output. Bound: 0.5 s.
Don't: say the wait twice or before `said`; print a notice on standard output;
give the store's error per refused request; format in the domain.
Size: at most 250 lines.
Removes: hearth 23.2's strays counted and never said; the `path: None` of
23.1's Startup line for an unopened store (protocol `store.rs:175`).

## Interface sketches

Every type derives `Clone, PartialEq, Eq, Hash, Debug` unless said (`Copy`
where it can). These add to hearth 23's sketches, which they name.

```rust
// crates/hearth-domain-store/src/vocabulary.rs (33.1: Arrived to Origin; 33.2: Load, Resolution, Unreadable, Empty, Inputs; 33.3: Waiting, Strays)
pub enum Event {                                // hearth 23's, these added
    Resolution { key: Digest, inputs: Inputs, closure: Box<[Closed]>, digest: Digest }, // Load: key and digest as written, the key's values, the closure
    Unreadable,                                 // Load: a version this release cannot read (6.8)
    /// A file in its temporary, hashed and synced (6.8), named by the protocol layer's token.
    Arrived { arrival: Token, sha256: Digest, size: u64, origin: Origin, took: Duration },
}                                               // `Done` also ends Empty, Keep and Discard
pub enum Request {                              // hearth 23's, these added
    Load { key: Digest },                       // resolutions/<key>.record: Resolution or Unreadable
    Empty,                                      // all beneath staging/, kept itself: Done
    Keep { arrival: Token, record: CacheFile }, // rename to cache/<sha256>, sync cache/, record: Done
    Discard { arrival: Token },                 // remove the arrival's temporary: Done
}
pub enum Answer {                               // hearth 23's, these added
    Cached { arrival: Token },                  // in the cache, recorded now or before
    Refused { arrival: Token },                 // refused, its file removed
    Waiting,                                    // waited `said` for the lock: said once
    Strays { count: u64, first: Stray },        // names of no known shape, left alone
}
// `Directory` gains `Resolutions`; `Step` gains `Load`, `Empty`, `Keep`.
/// `cache/<sha256>.record` (6.5) and its origin; mirror `cache_file::{CacheFile, Origin}`.
pub struct CacheFile { pub at: Wall, pub origin: Origin, pub sha256: Digest, pub size: u64, pub took: Duration }
pub enum Origin { Fetch { uri: Box<[u8]> }, Index { resolution: Digest }, Blob { oid: Box<[u8]> }, Rendered { path: Box<[u8]> } }
/// A closure's package as its digest takes it (6.4): the four fields, in order.
pub struct Closed { pub name: Box<[u8]>, pub architecture: Box<[u8]>, pub version: Box<[u8]>, pub sha256: Digest }
/// A resolution's key values as read (6.4); mirrors `resolution::Inputs`.
pub struct Inputs { pub apt: Box<[Item]>, pub request: Box<[Item]>, pub sources: Box<[Item]>, pub variant: Box<[Item]> }

// crates/hearth-domain-store/src/resolutions.rs (33.2)
/// H("resolution", sources, request, variant, apt) (render.md, 5.1).
#[must_use] pub fn key(inputs: &Inputs) -> Digest;
/// H("closure", list [ list [ name, architecture, version, sha256 ] … ]) (render.md, 4.5).
#[must_use] pub fn digest(closure: &[Closed]) -> Digest;
```

```rust
// crates/hearth-domain-builder/src/vocabulary.rs: hearth 23's enums gain these variants
pub enum Event {
    /// A blob of render's `Read` in its temporary, for the store to keep or refuse (33.1).
    Arrived { request: Token, oid: render::revision::Oid, size: u64, last: bool, arrival: Token, sha256: Digest, took: Duration },
}
pub enum Request {
    Progress(Progress),           // what the store says on standard error (33.3)
}                                 // `Unopened { to, why }` stays as hearth 23 landed it
/// client/README.md, 3.2's progress (33.3).
pub enum Progress { Waiting, Strays { count: u64, first: store::Stray } }

// crates/hearth-protocol-builder/src/lib.rs: hearth 23's, with Beneath and Walk (33.2) and `store` (33.3)
pub enum Asked { Io(io::Request), File(file::Request), Beneath(Beneath) }
/// A file request naming its directory by token, which the service resolves (`FileIo::descriptor`).
pub enum Beneath {
    OpenDirectory { owner: Token, root: Token, name: Box<[u8]>, no_follow: bool },
    Remove { owner: Token, root: Token, name: Box<[u8]>, directory: bool },
}
/// The walk's bounds (decisions.md, 11): 16 open directories, 64 entries a page, 65,536 entries.
pub struct Walk { pub depth: u32, pub page: u32, pub entries: u32 }
// `Limits` gains `walk: Walk` (33.2); `Protocol::new` gains `store: &[u8]` after `form`, the path the client named (33.3).
// src/translate/store.rs, as each is used
pub fn cache_file(record: store::CacheFile) -> tree::cache_file::CacheFile;    // 33.1
pub fn inputs(inputs: tree::resolution::Inputs) -> store::Inputs;              // 33.2
pub fn closed(closure: &[tree::resolution::Package]) -> Box<[store::Closed]>;  // 33.2
pub fn progress(progress: builder::Progress, store: &[u8]) -> records::Record; // 33.3

// crates/hearth-service-builder/src/lib.rs: `Configuration` gains `store: Box<[u8]>` (33.3)

// crates/hearth-client-protocol-records (33.3): src/progress.rs, `Record::{Waiting, Strays}` and their `Type`s
pub struct Waiting { pub store: Box<[u8]> }
pub struct Strays { pub store: Box<[u8]>, pub count: u64, pub first: Box<[u8]> }
// src/errors.rs: `Kind::Store(StoreError)`, render's the same (33.3); mirrors the store's `Full` and `Unusable`
pub struct StoreError { pub store: Box<[u8]>, pub fault: StoreFault }
pub enum StoreFault { Full { limit: StoreLimit, most: u32 }, Failed { step: Step } } // an unopened store is a Startup record
pub enum StoreLimit { Resolutions, CacheFiles } // not `Limit`: hearth 12's, 24's and 31's is in this file
pub enum Step { List, Remove, Load, Empty, Keep } // `Unusable`'s; Lock and Record are `Why`'s, a Startup record
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-33\.[1-3]'` prints 3.
2. `git -C ~/src/rust/hearth ls-tree --name-only origin/main
   crates/hearth-domain-store/src/` lists `enter.rs`, `index.rs`, `lib.rs`,
   `limits.rs`, `resolutions.rs`, `scan.rs`, `start.rs`, `tests`, `tests.rs`
   and `vocabulary.rs`.
3. In a worktree at `origin/main`, once: `heavy cargo nextest run -p
   hearth-domain-store -p hearth-builder-world
   -E 'package(hearth-domain-store) | binary(=store)'` passes, its `Summary` naming at least 46 tests (hearth
   23's 21 under that filter at hearth 3fd9fab, and these 25); and `heavy cargo nextest run -p
   hearth-domain-builder -p hearth-protocol-builder -p
   hearth-client-protocol-records -p hearth-builder-world -p hearth` passes.
