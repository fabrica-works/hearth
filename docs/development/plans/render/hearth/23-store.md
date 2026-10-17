# Session 23: `hearth-domain-store` (M1) and the store's io: lock, record, scan of the cache

M1's store without layers (store.md, 2.4): `hearth-domain-store`, a child of
the builder's root, decides; `hearth-protocol-builder` does its io (skein 04's
`ListPage`, `Rename`, skein 05's `Lock`) and its records (hearth 09). 23.1: the crate,
the lock and `store.record` (8.2, steps 1 and 2); 23.2: the top and `cache/`
scanned by pages, settled (4.4) and indexed. It ends hearth 11's transitional
"the store's lock or `store.record`". hearth 33 builds on its index and io: cache files enter
through the store, `resolutions/` is scanned, `staging/` emptied, the store
says what it does (hearth 24 and 31 call it: hearth 33, "What hearth 24 calls").

## Read first

- hearth's `store.md`: 2.1, 2.4, 3.1, 4.4, 6.1, 6.4 to 6.6, 6.8, 7.5, 8.1, 8.2;
  `render.md`: 7.2, 8.1; `client/README.md`: 3.2, 3.3.
- skein's `io.md`: 5 (the record block, "Listing", "Renaming", "Locking",
  "Replacing a file, in order", "One request at a time"), 9 ("Roots by token
  everywhere"); `programming-model.md`: 4.4, 4.5, 6.3, 9 (by number only).
- Drafts: `hearth/09-records.md` whole; `hearth/11-thin-story.md` (Choices,
  "Where later sessions add", 11.2 to 32.1, sketches); `skein/04-store-io.md`
  and `skein/05-directory-lock.md` (Shape, sketches); `hearth/01-lib.md` (`H`); `findings/skein-io.md`, 3 and 4.

## What the design asks

- **The lock** (8.2, step 1): `flock` on the store's root, "taken without
  waiting ... and held until the builder exits", asked again every 200 ms; io
  "keeps no timer for one" (skein's io.md, 5).
- **What M1 runs** (2.4, 6.6, 6.8): steps 1 and 2, step 6 for resolutions and
  cache, step 7; `store.record` is `layout` 1, `uid`, `made`, no `range`; one it
  cannot read "is written again".
- **The scan** (8.2, 6 and 7; 4.4): listed "page by page", 1,024 a page, each
  entry counted against 8.1's limits; temporaries, a cache file with no record
  and a record with no file deleted; strays reported; the index built.
- **The index** (8.1): one per cache file, by digest (262,144), paired by name,
  no record read; a store at a limit is full: no index, every request that
  reads or adds is an operational error, a file passing it refused and removed.

## Choices this plan makes

- **The root opens the store at the call:** it steps `Open` and gives render its
  `Render` once the store answers `Ready`, `Full` or `Unusable`; `Unopened`
  ends the call before render, `TryAgain`, exit 75 (store.md 8.1: a lock or
  `store.record` that fails ends the call before render, standard error, 75).
- **One store request in flight** (`findings/skein-io.md`, 3): a terminal
  carries no token; an arrival (hearth 33.1) carries the protocol layer's.
- **A scan's page or removal that fails** leaves no index, the store refused
  as when full, naming the step (store.md 8.1); the page is not listed again
  (io.md 5: its position is past entries it never answered).
- **The protocol layer parses and formats names:** a page reaches the store as
  `Entry { name, shape }` (2.1's shapes); removal is `Removal::{File, Record}`
  or the name given (`Named`).
- **`cache/` is listed twice, records first:** pass one holds each record's
  digest (at most twice `cache_files`, the scan's memory bound) and removes
  temporaries; pass two pairs each file with its record, counting the pair
  against `cache_files`, removing a file with no record; each record left
  unpaired then goes; only a file whose record exists counts (store.md 8.2,
  step 6, and 8.1).
- **The directories are the shell's roots:** `Client::open` makes
  `resolutions/` and `staging/` 0700 and opens them as 32.1 does `cache/`
  (`skein_shell::make_root`, ring/roots.rs:11), records, tree files and cache
  files being 0644 as `Store` makes them under umask 022 (store.md 2.1); the
  service adopts each
  (`FileIo::adopt_root`, file_layer.rs:363) as `Root { fd, token }`. The lock
  is `Lock { root: <the store's token> }`. A listing is a cursor of its own,
  `OpenDirectory { root: <store fd>, name: b"cache" }` (`b"."` for the top).
- **`store.record`** is read with `Load` (at most `frame::largest(Kind::Store,
  &limits.tree)`, frame.rs:113, `-> u64`, narrowed by `u32::try_from`)
  and, absent or unreadable, written `{ layout: 1, made: env.wall, uid }` by
  `Store`, `Expect::Any`; a readable one is kept, its `range` too, unless its
  `uid` is not the builder's: `Unopened`, naming both (store.md 6.6: refused
  as another base is; render.md 7.2's `store` row: "no error of the answer:
  nothing renders, and the client says so at startup"). A field the version does
  not name, or a missing or mistyped one, makes the record unreadable, and
  one of another `layout` is written again (6.1, 6.8). The uid is
  `skein_shell::effective_user()` (ring.rs:1678), read at startup.
- **An unopened store is said at startup** (client/README.md 3.2: "a
  repository or a store it cannot open) goes to standard error alone:
  `hearth: <message>`, or with `--json` one `startup` record"): on
  `Unopened { why }` the root emits, in place of `Reply { TryAgain }`, its
  new `Request::Unopened { to: ReplyTo, why: store::vocabulary::Why }`; the
  protocol layer writes records' `Startup::Unopened { what: Opened::Store,
  path: None, error }` (startup.rs:11; hearth 33.3 gives the path) to
  standard error, its second output (`Service::adopt_error_pipe` and
  `adopt_error_file` beside `adopt_output_*`, service `lib.rs:72`, `:80`;
  `Client::open` takes `error: Fd`, which `main.rs:42` passes as
  `Fd::new(2)`; protocol `client.rs:19`'s `output` gains `error` beside it),
  and ends the call with 75. `error` is `lock failed`
  (`Why::Lock`), `store.record failed` (`Why::Record`), or `store.record is
  uid <recorded>'s, not uid <builder>'s` (`Why::Owner`): `hearth: cannot
  open the store: store.record is uid 1001's, not uid 1000's` LF.
- **The wait's pace is the store's `Limits`:** `retry` and `said`, `SHIPPED`
  200 ms and 1 s, tests 20 ms and 100 ms, naming why; the timers are
  `lib::Deadlines` (programming-model.md, 9). A `Lock` failing `TimedOut` is
  told `Busy` and asked again; hearth needs no answer to log 57.
## Rules that bind it

- **The meeting point** (README, Rules): each increment proves on the real
  loop, `hearth render` through real git in `tests/builder`; the store's own
  world is M2's (store.md, 11).
- **Upstream first:** 23.1 starts with its repin commit, `heavy cargo update -p
  skein-lib`, onto a skein `origin/main` holding skein 04 and 05; the repin's users
  of 04.1's `Rename` (hearth 32.1's `cache.rs`) take `from_root` and `to_root`
  in that commit.
- **Shared files** (README, Rules): the root `Cargo.toml` and `Cargo.lock`;
  `crates/hearth-protocol-builder/src/translate/`; `tests/builder/src/
  fixtures.rs`; and, beyond the README's list, the root's `vocabulary.rs` and
  `lib.rs`, the protocol's and the service's `lib.rs`, `crates/hearth/src/
  lib.rs`. Landing second, rebase and keep both.
- **Literals:** digests, object ids and keys in tests come from `sha256sum`,
  `git hash-object` or 3.1's table by hand (`python3 -I`), pasted once.

## Increments

### 23.1 The store's crate, its lock and `store.record`
Scope: `hearth-domain-store` (render.md, 8.1) and 8.2's steps 1 and 2: `Lock`,
asked again every `retry` while `Busy`; `store.record` loaded, written when
absent or unreadable (6.6, 6.8); the root's routing of the store; the protocol
layer's `src/store.rs` and `translate/store.rs`; the service's domain timers;
the shell's `resolutions/` and `staging/`.
Needs: hearth 09 (09.2's `store::{read, write}`, `frame::largest`), hearth 11
(the root, the protocol layer, the service, `Client`, the real loop), hearth
32.1 (the cache), skein 04, skein 05 (04.1's `Rename`, 04.2's `ListPage`, 05.2's
`Lock`); its repin first.
Files: root `Cargo.toml`, `Cargo.lock` (shared); new
`crates/hearth-domain-store/`: `Cargo.toml` (`skein-lib`, `hearth-lib`;
`[lints] workspace = true`), `src/{lib, limits, vocabulary, start}.rs`,
`src/tests.rs`, `src/tests/{fixtures, start}.rs`;
`crates/hearth-domain-builder/{Cargo.toml, src/lib.rs, src/vocabulary.rs,
src/tests.rs, src/tests/fixtures.rs}` (`fixtures.rs:4`'s `LIMITS` takes
`store`; `a_render_call_reaches_render`, `tests.rs:11`, sees the store's
`Open` first); `crates/hearth-protocol-builder/`: `Cargo.toml`
(`hearth-protocol-tree`), `src/{lib, limits, cache, client}.rs` (04.1's `Rename`
form), new `src/store.rs`, new `src/translate/store.rs`, `src/translate.rs`,
`src/tests/{fixtures, store, translate}.rs`;
`crates/hearth-service-builder/src/{lib, limits}.rs`, `src/tests.rs`
(`adopt_error_*`), `src/tests/fixtures.rs`
(`:7`'s `Configuration` takes `uid`, `:8`'s `Roots` `resolutions` and
`staging`); `crates/hearth/src/{lib, main}.rs`; `tests/builder/Cargo.toml`
(`hearth-protocol-tree`, `hearth-domain-store`, `skein-lib` for `Duration`,
`Time`, `Wall`, `Queue`), `src/fixtures.rs` (shared),
new `tests/store.rs`.
Shape: the sketches. Phases: `Closed`, `Locking`, `Recording`, `Writing`, then
`Ready` (23.2 inserts the scan before it); a `Failed` lock or record, or a
record whose `uid` is not the builder's, is `Unopened { why }`. `Busy` arms
`Timer::Retry` at `now + retry`; `fire` on it emits `Lock`; `Locked` cancels
it. The root: on `Call`, the call held and `store::vocabulary::Event::Open` stepped; on
`Ready`, `Full` or `Unusable`, render's `Event::Render`; on `Unopened { why }`,
`Request::Unopened { to, why }` and no render (Choices); `next_deadline` and `fire` forward to the
store, and the service fires the root's timers after its events and names them
in its `next_deadline`. The protocol: `Request::Lock` → `file::Request::Lock {
root: roots.store.token }`, `Locked` → `Locked`, `Busy` or `Failed { TimedOut
}` → `Busy`; `LoadRecord` → `Load { path: b"store.record", max:
frame::largest(Kind::Store, &limits.tree) as a `u32`, no_follow: true }`, read by `store::read` (Ok →
`Loaded { Some }`; `Unreadable`, `Failed { NotFound }` or `TooLarge` → `Loaded {
None }`); `WriteRecord` → `Store` of `store::write`'s bytes. The service:
`Configuration::uid`, `Files::whole` (`FileIo::with_whole_limit`'s `max_file`,
`frame::largest(Kind::Resolution, &limits.tree)` as a `u32`, 4,194,328, so that hearth 33.2 loads with
it), `Roots` with `resolutions` and `staging`, each adopted; `Files::files` 32 (`limits.rs:34`
holds 2, and the four roots, the lock, `Load` and `Store` need more), which
23.2 keeps; `files.seed_randomness(seed)` (skein `file_layer.rs:374`) in
`Service::new`, without which `Store` fails `InvalidArgument` at admission
(`store.rs:176`). At its terminal the protocol sends `file::Request::Close`
for each root's token: adopted roots count in `open_files`, which the
service's `is_empty` (`lib.rs:102`) waits on. So `Client` drops its `store`
and `cache` keepers, `is_empty`'s `&& self.store.is_none()` (`lib.rs:78`) and
`drain`'s closes; only `Client::open`'s error paths, before the service
exists, still close them. `Client::open` makes them 0700 and reads
`effective_user()`.
Test shape: unit (`src/tests/`, under 20 ms each, the step crates' lints), with
`fixtures::Harness` (a `Store` at given limits and uid; `step(now, Event)`,
`step_at(now, wall, Event)`, `fire(now)`, each `-> Box<[Output]>`;
`page(&[Entry], last)`, `next_deadline()`, `indexed(&Digest)`), which 23.2 and
hearth 33 extend. Reference, in `crates/hearth-domain-store/src/tests/start.rs`:

```rust
#[test]
fn a_busy_lock_is_asked_again_after_200_ms_and_no_sooner() {
    let mut store = Harness::new(limits::SHIPPED, 1000);
    assert_eq!(&*store.step(Time::ZERO, Event::Open), &[Output::Request(Request::Lock)], "8.2, step 1 first");
    assert!(store.step(Time::ZERO, Event::Busy).is_empty(), "io keeps no timer");
    let retry = Time::from_nanos(200_000_000);
    assert_eq!(store.next_deadline(), Some(retry), "store.md, 8.2: every 200 ms");
    assert!(store.fire(Time::from_nanos(199_999_999)).is_empty(), "no sooner");
    assert_eq!(&*store.fire(retry), &[Output::Request(Request::Lock)], "asked again");
    assert_eq!(&*store.step(retry, Event::Locked), &[Output::Request(Request::LoadRecord)], "then step 2");
}
```
Then `opening_the_store_asks_for_its_lock_first`,
`an_absent_or_unreadable_record_is_written_as_layout_1_by_its_user`,
`a_readable_record_of_layout_2_is_written_again_as_layout_1` (6.1, 6.8),
`a_readable_record_is_kept_with_its_range_and_the_store_is_ready`,
`a_failed_lock_or_record_leaves_the_store_unopened`,
`a_record_of_another_uid_leaves_the_store_unopened_naming_both`; root
(`src/tests.rs`): `the_call_reaches_render_only_once_the_store_is_open`,
`an_unopened_store_ends_the_call_before_render` (`Request::Unopened`, each
`Why`); protocol: `an_unopened_store_writes_its_startup_record` (the three
`error` texts, literal),
`a_lock_asks_io_through_the_stores_root_token`, `a_lock_timed_out_is_told_busy`,
`an_absent_store_record_is_none_and_a_written_one_is_framed_as_kind_1`,
`each_store_record_field_translates_and_back`. Real loop, each under 0.5 s,
with `fixtures::with_store(store::limits::Limits) -> Limits`
(`hearth::limits()`, the store's replaced) and `fixtures::render_holding(
&Repository, &Limits, Release) -> Held { rendered, printed_while_held }`: a
`Host` around `Client` that holds `std::fs::File::lock` on the store's
directory (made 0700 first) and drops it when `Release::After(span)` has passed
its first `now`. Reference, in `tests/builder/tests/store.rs`:

```rust
#[test]
fn a_render_over_a_locked_store_waits_then_renders_once_it_is_released() {
    let repository = Repository::make("locked", fixtures::THIN);
    // 20 ms beside the shipped 200 ms: five asks within the test's 0.5 s.
    let limits =
        fixtures::with_store(store::limits::Limits { retry: Duration::from_millis(20), ..store::limits::SHIPPED });
    let held = fixtures::render_holding(&repository, &limits, Release::After(Duration::from_millis(100)));
    assert!(held.printed_while_held.is_empty(), "store.md, 8.2: nothing renders while the lock is busy");
    assert_eq!(String::from_utf8_lossy(&held.rendered.stdout), fixtures::THIN_TEXT, "then the spec");
}
```
Then `the_first_render_records_the_store_as_layout_1_of_its_user` (read back by
`hearth_protocol_tree::store::read`: layout 1, no range, the uid of
`std::fs::metadata(store)`; the top lists exactly `cache`, `resolutions`,
`staging` and `store.record`, no `layers/` (2.4), each directory 0700 (2.1)),
`a_second_render_keeps_the_store_record_it_found` (the bytes equal),
`an_unreadable_store_record_is_written_again` (a frame of version 2, made by
changing one byte of a written record).
Proof: process, `heavy cargo nextest run -p hearth-builder-world -E
'test(=a_render_over_a_locked_store_waits_then_renders_once_it_is_released)'`:
the real loop asks for the store's lock, waits while another open of the
directory holds it, and renders THIN once it is free. Bound: 0.5 s.
Don't: wait or retry in the protocol layer; take the lock on `store.record`;
write a `range`; rewrite another user's record; arm a timer for io; scan
anything yet.
Size: at most 280 lines.
Removes: hearth 11's "the store's lock or `store.record`" (its Choices,
"Transitional").

### 23.2 The scan of the top and `cache/`, and the cache files' index
Scope: step 6 for `cache/` and the top, step 7 for them (4.4's temporaries,
files without records, records without files; strays counted), the index of
cache files, a scan past `cache_files` full.
Needs: hearth 23.1; skein 04.2's `ListPage` (in 23.1's repin).
Files: `crates/hearth-domain-store/src/{lib, limits, vocabulary}.rs`, new
`src/{scan, index}.rs`, `src/tests/{fixtures, scan}.rs`;
`crates/hearth-protocol-builder/src/store.rs`, new `src/store/names.rs`,
`src/tests/store.rs`; `crates/hearth-service-builder/src/limits.rs`
(`Files::entries: 1_024`; `files` is 23.1's 32); `tests/builder/{src/fixtures.rs,
tests/store.rs}`.
Shape: after the record, `List { directory: Top, max: page }`, then `Cache`
twice, `Next` until `last`, then the settlement, then `Ready`. Pass one:
`Temporary` → `Remove { Named }`; `Record(d)` → `index.insert(d, false)` (a
`Map` of twice `cache_files`; past it, `Full`); `Stray` → counted, the first
kept; `File` and `Fixed` skipped. Pass two: `File(d)` → paired (`true`) and
counted, past `cache_files` → `Close` if not `last`, then `Full { limit:
CacheFiles, most }`; or `Remove { File(d) }` when no record holds it.
Settlement: each unpaired digest, by `Map::range` past the last removed, →
`Remove { Record(d) }`, then out of the index. The page is held with its cursor
(`u32`) while a removal is in flight. Names (`names::shape(directory, name,
kind)`): at the top, 2.1's fixed names (`store.record`, `cache`, `resolutions`,
`staging`, `layers`, `targets`, `state`, `artifacts`, `sources`, `caches`,
`repo`, `credentials`) are `Fixed`, `store.record.skein-<16 hex>` a
`Temporary`; in `cache/`, `<64 hex>` a `File`, `<64 hex>.record` a `Record`,
`<64 hex>.tmp` and `<64 hex>.record.skein-<16 hex>` `Temporary`; each only as a
regular file (`Kind::File`), lowercase hex only (store.md 4.4: `cache/notes.tmp` is a stray, reported and
left); anything else `Stray`. The protocol: a cursor `OpenDirectory { root: store.fd, name: b"." or
b"cache", no_follow: true }`, `ListPage { max }`, `Page { last: len < max }`,
closed on `last` or `Close`; `Remove { root: cache.fd or store.fd }` with the
name formatted (`Removal`) or given.
Test shape: unit as 23.1's. Reference, in `src/tests/scan.rs`
(`fixtures::scanning(limits)` is a store past its record, at `List { Cache }`;
`D1`, `D2` are digests `01…01`, `02…02`; names written whole, 64 digits):

```rust
#[test]
fn a_cache_file_without_its_record_is_removed_in_the_second_pass() {
    let mut store = fixtures::scanning(limits::SHIPPED);
    let first = [fixtures::entry(b"0101…01.record", Shape::Record(D1))];
    assert_eq!(&*store.page(&first, true), &[Output::Request(Request::List { directory: Directory::Cache, max: 1_024 })]);
    let second = [fixtures::entry(b"0101…01", Shape::File(D1)), fixtures::entry(b"0202…02", Shape::File(D2))];
    let removal = Request::Remove { directory: Directory::Cache, name: Removal::File(D2) };
    assert_eq!(&*store.page(&second, true), &[Output::Request(removal)], "4.4: no record, so the file goes");
    assert_eq!(&*store.step(Time::ZERO, Event::Done), &[Output::Answer(Answer::Ready)], "D1 paired, alone");
    assert!(store.indexed(&D1) && !store.indexed(&D2), "the index holds what is recorded");
}
```
Then `temporaries_are_removed_as_they_come_and_strays_left`,
`a_record_without_its_file_is_removed`,
`the_recorded_file_past_the_cache_limit_stops_the_scan_full` (`cache_files: 2`
beside 262,144: three pairs show it),
`a_file_without_its_record_counts_toward_no_limit` (`cache_files: 2`: two pairs
and one bare file are `Ready`); protocol:
`a_page_of_the_cache_comes_up_as_typed_names` (a `const` table: each shape, an
uppercase digest, 63 digits, a `.tmp` that is a directory),
`the_top_names_store_md_2_1s_fixed_names_and_the_records_temporary`,
`a_cursor_closes_after_its_last_page`. Real loop, with
`fixtures::render_with(&Repository, &Limits)` and `fixtures::lay(&Path,
&[Laid])` (`Laid::{File, Record, Resolution}`; directories 0700, files, records
by hearth 09's writers):
`the_scan_settles_what_a_crash_left_in_the_cache` (laid, each name 64 hex: `a`
is D1 (`01…01`) with `a.record` written by `cache_file::write`, `b` D2 alone,
`c.record` D3 alone, `<D1>.tmp`, `<D1>.record.skein-0123456789abcdef`, `notes`;
after `hearth render` of THIN, `cache/` lists exactly `a`, `a.record`, `notes`
and THIN's two files),
`the_scan_removes_the_store_records_temporary`.
Proof: process, `heavy cargo nextest run -p hearth-builder-world -E
'test(=the_scan_settles_what_a_crash_left_in_the_cache)'`: on the real ring, a
store a crash left is settled as 4.4 says before render reads. Bound: 0.5 s.
Don't: read a cache file's record; sort a page or hold more than one; remove a
name of no known shape; list `resolutions/` (hearth 33.2); refuse anything yet.
Size: at most 270 lines.
Removes: nothing.

## Interface sketches

Every type derives `Clone, PartialEq, Eq, Hash, Debug` unless said (`Copy`
where it can); `Store`, the state, derives `Debug` alone, `Asked` and `Told` `PartialEq,
Eq, Debug` (skein's requests derive no `Clone`, `file::Request` no `Hash`). The
root re-exports the store (`pub use hearth_domain_store as store;`), through
which the protocol layer names it. hearth 33 adds the variants and fields it
names.

```rust
// crates/hearth-domain-store/src/limits.rs and src/lib.rs (23.1)
/// The store's configured limits (8.1, 8.2): 1,024 and 262,144; 1,024 a page; 200 ms; 1 s.
pub struct Limits { pub resolutions: u32, pub cache_files: u32, pub page: u32, pub retry: Duration, pub said: Duration }
pub const SHIPPED: Limits = Limits { resolutions: 1_024, cache_files: 262_144, page: 1_024,
    retry: Duration::from_millis(200), said: Duration::from_secs(1) };
/// The index and one page at `limits` (`Map::worst_case`, names of 255 bytes).
#[must_use] pub fn worst_case(limits: &Limits) -> Option<u64>;
/// The store's state: its phase, its index (23.2), its timers, the page in hand.
pub struct Store { /* private */ }
impl Store { #[must_use] pub fn new(limits: &Limits, uid: u32) -> Store; }
/// A step or a fire emits one output: one request in flight, an answer on a terminal alone (the root's is 1).
pub const MAX_OUT: u32 = 1;
pub fn step(store: &mut Store, env: &Env<Limits>, ev: Event, out: &mut Queue<Output>);
/// The timers due at `env.now`: the lock's retry (hearth 33.3 adds the wait said).
pub fn fire(store: &mut Store, env: &Env<Limits>, out: &mut Queue<Output>);
#[must_use] pub fn next_deadline(store: &Store) -> Option<Time>;

// crates/hearth-domain-store/src/vocabulary.rs (23.1; Page to Stray 23.2)
/// Into the store, through the root: `Open`, or the one request's terminal.
pub enum Event {
    Open,                                       // open for the call: 8.2's steps, then an Answer
    Locked,                                     // Lock: taken, or held already through this root
    Busy,                                       // Lock: another open holds it; asked again at `retry`
    Loaded { record: Option<StoreRecord> },     // LoadRecord: None when absent or unreadable
    Page { entries: Box<[Entry]>, last: bool }, // List or Next: at most `max`; `last` once the cursor closed
    Done,                                       // WriteRecord, Remove or Close done
    Failed,                                     // any request failed: unopened or unusable (store.md 8.1)
}
/// Out of the store, to the protocol layer through the root; one in flight.
pub enum Request {
    Lock,                                           // flock on the store's root, never waiting: Locked, Busy
    LoadRecord,                                     // read store.record: Record
    WriteRecord { record: StoreRecord },            // by skein's Store, Expect::Any: Done
    List { directory: Directory, max: u32 },        // open a cursor, its first page: Page
    Next { max: u32 },                              // the cursor's next page: Page
    Close,                                          // close the cursor before its last page: Done
    Remove { directory: Directory, name: Removal }, // one name: Done
}
/// The store to the root.
pub enum Answer {
    Ready,                            // locked, recorded, scanned, the index built
    Full { limit: Limit, most: u32 }, // a count past `most` (8.1): no index
    Unusable { step: Step },          // a scan's step failed: no index, as Full
    Unopened { why: Why },            // not opened: the call ends before render
}
/// What `step` and `fire` emit.
pub enum Output { Request(Request), Answer(Answer) }
/// The directories the scan lists (`Resolutions`: hearth 33.2); 8.1's two limits; where an unusable store failed.
pub enum Directory { Top, Cache }
pub enum Limit { Resolutions, CacheFiles }
pub enum Step { List, Remove }
/// Why the store is not opened: its lock or its record failed; another uid's record (6.6).
pub enum Why { Lock, Record, Owner { recorded: u32, builder: u32 } }
/// A name as the protocol layer shaped it (2.1, by name and kind), with its bytes for a removal.
pub struct Entry { pub name: Box<[u8]>, pub shape: Shape }
/// `File` and `Record` carry their digest; `Temporary` (`.tmp`, `.skein-<16 hex>`), `Fixed` (top), `Stray`.
pub enum Shape { File(Digest), Record(Digest), Temporary, Fixed, Stray }
/// A name to remove: formatted by the protocol layer, or as it was given.
pub enum Removal { File(Digest), Record(Digest), Named(Box<[u8]>) }
/// A stray: its directory and its name.
pub struct Stray { pub directory: Directory, pub name: Box<[u8]> }
/// `store.record` (6.6) and its range; mirror hearth-protocol-tree's `store::{Store, Range}`.
pub struct StoreRecord { pub layout: u64, pub made: Wall, pub range: Option<Range>, pub uid: u32 }
pub struct Range { pub base: u32, pub count: u32 }
```

```rust
// crates/hearth-domain-builder/src/vocabulary.rs: hearth 11's enums gain these variants
pub enum Event { Store(store::vocabulary::Event) }     // a store request's terminal (23.1)
pub enum Request {
    Store(store::vocabulary::Request),                 // the store's request of the protocol layer (23.1)
    Unopened { to: ReplyTo, why: store::vocabulary::Why }, // the call ends before render: said at startup (23.1)
}
// The root's state (11.2's) is made with the store's limits and `uid`; `fire`, `next_deadline` join `step`.

// crates/hearth-protocol-builder/src/lib.rs: hearth 11's, with Root and Roots (23.1); Beneath, Walk: hearth 33.2
/// A directory the shell opened, as io knows it; the store's four (2.4).
pub struct Root { pub fd: Fd, pub token: Token }
pub struct Roots { pub store: Root, pub cache: Root, pub resolutions: Root, pub staging: Root }
pub enum Asked { Io(io::Request), File(file::Request) }
// `Limits` gains `tree: hearth_protocol_tree::limits::Limits` (23.1).
impl Protocol {
    /// 11's `root` and 32.1's `cache` become `roots` (hearth 33.3 adds `store: &[u8]`, the path the client named).
    #[must_use] pub fn new(limits: &Limits, call: Call, form: Form, roots: Roots, seed: u64) -> Protocol;
}
// src/translate/store.rs: hearth-protocol-tree's records (`tree`) to the store's and back, as each is used
pub fn store_record(record: tree::store::Store) -> store::vocabulary::StoreRecord; // 23.1
pub fn tree_store(record: store::vocabulary::StoreRecord) -> tree::store::Store;   // 23.1

// crates/hearth-service-builder/src/lib.rs (`Configuration`, `Roots`) and src/limits.rs (`Files`, :14):
// hearth 11's and 32.1's, these fields added (`store`: hearth 33.3)
pub struct Configuration { pub limits: Limits, pub uid: u32 }
pub struct Files { pub files: u32, pub read: u32, pub entries: u32, pub whole: u32, pub timeout: Duration }
pub struct Roots { pub store: Fd, pub cache: Fd, pub resolutions: Fd, pub staging: Fd }
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-23\.[1-2]'` prints 2.
2. `git -C ~/src/rust/hearth ls-tree --name-only origin/main
   crates/hearth-domain-store/src/` lists `index.rs`, `lib.rs`, `limits.rs`,
   `scan.rs`, `start.rs`, `tests`, `tests.rs` and `vocabulary.rs`; in a
   worktree at `origin/main`, `heavy cargo tree -p hearth-domain-store -e
   normal --depth 1` prints three: `hearth-domain-store`, `hearth-lib`,
   `skein-lib`.
3. In that worktree, once: `heavy cargo nextest run -p hearth-domain-store -p
   hearth-builder-world -E 'package(hearth-domain-store) | binary(=store)'`
   passes, its `Summary` naming at least 17 tests; and `heavy cargo nextest run
   -p hearth-domain-builder -p hearth-protocol-builder -p
   hearth-client-protocol-records -p hearth-builder-world -p hearth` passes.
