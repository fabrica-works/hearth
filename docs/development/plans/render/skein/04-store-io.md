# Session 04: skein's io for the store: a rename across directories, a listing in pages

hearth's store without layers needs of skein's io a rename across
directories and a listing in pages (hearth's README.md, 8.1; the second
round's K5 and K6, designed in skein `d21d90d`); the directory's lock is
skein 05's. 04.1 makes io's `Rename` name the directory an entry leaves
and the one it lands in, and gives the minimal fake machine that rename's
crash model. 04.2 lists a directory the owner opened in pages, the open
directory being the cursor. hearth 23 waits for this session and skein
05; 04.1's rename is first called by hearth 32.1 (the cache), then 33 and
31 (a resolution's indices into `cache/`).

## Read first

- What the design changed: `git -C ~/src/rust/skein show d21d90d`. At
  that commit: io.md 5 (the record block, "Listing", "Renaming", "One
  request at a time", "Memory"), 8 ("Files"), 9; kernel.md 3, 6 ("Some
  operations are synchronous"), 6.1 (on `Rename`, `List`, cancelling), 7,
  8 ("Files beneath a root"); simulator.md 3.3, 5, 6, 7; testing.md 4.
- The caller, hearth at `175e92f`: store.md 8.2 (step 6), 6.8 ("A cache
  file is durable before its record"); README.md 8.1's M1 rows on a
  rename and a listing by pages.
- `findings/skein-io.md`, 3 and 5; `findings/skein.md`, 4;
  `findings/measurements.md`; skein's `AGENTS.md`, `scripts/check.sh`;
  skein 03, 03.3 (`Hash` and `state_digest` on the fake machine's every
  field).
- The models: `tests/io/src/cuts.rs` and `tests/io/tests/cuts.rs` (a world
  cut at each operation), `tests/io/src/files.rs` (a file owner's world),
  `tests/io/tests/ring.rs:194` (an owner over the real ring in a scratch
  directory), `testing/skein-conformance/src/files.rs:525` (`list`).
- skein's programming-model.md: 2.1, 3, 6.3, 10; testing-strategy.md:
  2.1, 2.6, 4.3, 5, 6, 8 (by number only).

## What the design asks

- **Renaming** (io.md, 5): "`Rename` takes the directory the entry leaves
  and the one it lands in, each a root, the same one for a rename in
  place, and a single name in each"; across two filesystems "the kernel's
  error is `Other(EXDEV)`"; it "survives a crash once the owner has synced
  the directory it lands in", and until then "a crash keeps it in one or
  the other, never both or neither" (kernel.md, 6.1; simulator.md, 3.3).
- **Listing** (io.md, 5): "each `ListPage` answers `Listed` with at most
  `max` of its entries (at least 1 and at most `max_entries`: 0, or
  more, is `InvalidArgument`, before any record), from where the last
  page of that open directory stopped. io fills each page until `max` or
  the directory's end, so a page shorter than `max` is the last, and an
  empty one says the end was reached already"; "A page's entries are
  counted in `worst_case` as a `List`'s are." `List` is unchanged. A page
  that fails, by a kernel's error, its deadline or a cancel, "may have
  taken entries it never answered"; the owner lists again from a new
  `OpenDirectory`. An operation that completes without a `Cancel`
  finishes first: a page stops at the `List` in flight, "unless the
  completion finished it".
- **Tests** (io.md 9): "each with io's step tests and a world of files
  that meets it"; simulator.md, 6: "each broken invariant of files" and
  "each fault of files".

## Choices this plan makes

- **Order:** 04.1 and 04.2 change io alone (`Op::Rename` already takes
  two directories; a page is `List`s). **`Rename` changes in place**
  (code-organization.md, 6): `root` gives way to `from_root` and
  `to_root`.
- **A page never takes past its end:** each of its kernel `List`s asks
  for `min(32, max − held)` entries (8192 bytes of names, as
  `FileIo::list`), since what a `List` takes moves the position.
- **Tokens as io takes them today:** an unknown token is `Failed {
  NotFound }`, as `Stat`'s (`file_layer.rs:515`); io records no token's
  kind, so a page of a file answers the kernel's `NotADirectory`.
- **The machine's model of a move:** a rename between two directories
  stays pending until its destination's `sync`; a crash draws each
  pending one, landed or not, whole in both directories' surviving copies
  (sketch); a sync of the source alone changes nothing. A rename in place
  keeps today's model. An entry never durable in its source may be in
  neither, as an unsynced create is lost (3.3): the tests lay their file
  durable. `Move` derives `Hash`, for skein 03.3's `state_digest`,
  whichever lands first.
- **Where code goes:** `kernel.rs`, `ring.rs`, `sim.rs`, `file_layer.rs`,
  the machine's `fs.rs` and conformance's `files.rs` are past 800 lines:
  they take variants and arms only. New:
  `crates/skein-io/src/file_layer/directory.rs` (a page's admission and
  answers; skein 05.2 adds a lock's),
  `testing/skein-fake-machine/src/fs/moves.rs`; step tests in a new
  `crates/skein-io/src/tests/directory.rs`, with `complete` and `listed`
  (`tests/file.rs:34, :62`) made `pub(super)`; worlds in new
  `tests/io/src/{renames,pages}.rs`, their tests in one new binary,
  `tests/io/tests/directory.rs`; one real-ring test an increment in
  `tests/io/tests/ring.rs`.
- **Limits:** skein ships no file profile: each world takes small ones
  of its own, as `tests/io/src/files.rs:19-21` (4 files, 4 entries).
  hearth's 1,024 a page (store.md 8.2, step 6) is hearth 23's.
- **io.md 5's temporary name,** contract since `d21d90d` for hearth's
  scan, gets two tests in 04.1 and no code.

## Rules that bind it

- **skein 03 and this session meet in the fake machine's digest.** At
  its rebase before landing, 04.1 runs `rg -n 'fn state_digest'
  testing/skein-fake-machine/src`: a match, it adds `pending_moves` to
  that digest, with a test that a pending move changes it; none, skein
  03.3 digests it with the rest.
- **The gate** (README, Rules; decisions.md, 1): each increment lands with `flock
  ~/src/rust/tmp/hearth/render/land-skein.lock land --gate '<the README's
  command, whole>'`, its commit body saying so in one line; no probe of
  the machine. No `skein-ui-web` code changes.
- **Test time** (README, Rules; `findings/measurements.md`): focused
  6.46 s and fuzzy 41.84 s at the base; this session's allotment is 0.4 s
  and 1.5 s (skein 02: 0.4 s / 1 s; 03: 0.3 s / 0.5 s; 04: 0.4 s / 1.5 s;
  05: 0.4 s / 2 s). A unit test under 20 ms, a focused world or ring
  test under 0.5 s, a fuzzy sweep about 5 s; a test the gate flags `SLOW`
  is made cheaper (fewer seeds or cut points) before landing. No budget,
  period or ceiling is raised; no focused test moves to the fuzzy suite.
- **Users in skein change in the increment, none downstream** (hearth has
  no code at the base): Files name skein's users of what each Shape
  changes; smith's, jig's and temper's change at their closing repins
  (README, Repin). Shared with
  skein 03 and 05: `testing/skein-world/src/`,
  `testing/skein-fake-machine/src/fs.rs` and
  `crates/skein-sim/src/trace.rs`: landing second, rebase and keep both.

## Increments

### 04.1 A rename between two directories
Scope: io's `Rename` from one directory into another (io.md, 5); the
machine's crash model of it (simulator.md, 3.3; kernel.md, 6.1); io.md
5's temporary name pinned.
Needs: nothing.
Files: `crates/skein-io/src/file.rs` (`Request::Rename` :150),
`src/file_layer.rs` (shared; its arm :576-582);
`crates/skein-kv/src/store.rs:977`, the one constructor;
`testing/skein-fake-machine/src/fs.rs` (shared; `sync` :408, `crash`
:426, `rename` :509), new `src/fs/moves.rs`. Tests: new
`crates/skein-io/src/tests/directory.rs`, `src/tests.rs`,
`src/tests/{file,store}.rs`; `testing/skein-fake-machine/src/tests.rs`;
new `tests/io/src/renames.rs`, `tests/io/src/lib.rs`, new
`tests/io/tests/directory.rs`, `tests/io/tests/ring.rs`. Unchanged users
(`{ owner, .. }`, `{ .. }`): `testing/skein-world/src/files.rs:228`,
`crates/skein-oauth-accounts/src/private_tests.rs:119, :374`.
Shape:
- `Request::Rename { owner, from_root: Fd, from, to_root: Fd, to }`
  (sketch): both names `is_name` as now (`Failed { InvalidArgument }`, no
  record), then `Op::Rename { from_dir: from_root, from, to_dir: to_root,
  to }`, answered `Renamed` or `Failed` with the kernel's error. kv's
  snapshot passes `store.root()` twice.
- The machine: `rename` between two directories pushes a `Move` onto
  `Machine::pending_moves`; `sync` of its destination decides it (the
  source's durable copy loses `from`); `sync` of its source changes
  nothing in the `Move`; `crash` draws each pending move (sketch) before `recover_links`.
Test shape:
- Step, `src/tests/directory.rs`, under 20 ms each; the reference test
  (`OWNER`, `STAGING`, `CACHE`: the module's `Token` and `Fd` constants):

  ```rust
  #[test]
  fn a_rename_names_the_directory_it_leaves_and_the_one_it_lands_in() {
      let mut io = FileIo::new(3, 16, 4, Duration::from_secs(1));
      let (mut events, mut subs) = (Queue::with_capacity(2), Queue::with_capacity(2));
      let (from, to): (Box<[u8]>, Box<[u8]>) = (Box::from(&b"InRelease"[..]), Box::from(&b"index"[..]));
      let request = Request::Rename { owner: OWNER, from_root: STAGING, from: from.clone(), to_root: CACHE, to: to.clone() };
      file_layer::down(&mut io, Time::ZERO, request, &mut events, &mut subs);
      let submit = subs.pop().expect("one rename");
      assert_eq!(submit.kind, Op::Rename { from_dir: STAGING, from, to_dir: CACHE, to });
      complete(&mut io, submit, Ok(Done::Nothing), &mut events, &mut subs);
      assert_eq!(events.pop(), Some(Event::Renamed { owner: OWNER }));
  }
  ```

  and `a_rename_of_a_name_holding_a_slash_fails_before_any_record`;
  beside `tests/store.rs:92`,
  `a_stores_temporary_is_its_target_skein_and_sixteen_hex_digits` and
  `a_long_targets_temporary_keeps_its_first_230_bytes`, checking the
  name's bytes, none computed.
- The machine's tests, 16 seeds each as `tests.rs:331`'s:
  `an_unsynced_rename_across_lands_in_exactly_one_directory_after_power_loss`
  (both outcomes seen),
  `a_rename_across_survives_power_loss_once_its_destination_is_synced`,
  `a_synced_source_alone_leaves_a_rename_across_in_either_directory`.
- The world, `tests/io/src/renames.rs` after `cuts.rs`: the owner opens
  `staging` and `cache`, renames the laid `staging/index` to
  `cache/index`, syncs `cache`, closes both, noting the submission its
  sync's answer came after; its fresh incarnation lists both into
  `found: Option<Found>` (`Source`, `Destination`), panicking on both or
  neither; `FileSystem::cut` closes the held handles, then crashes on
  `PowerLoss`; `Memory::Checked`; under 0.5 s each. The binary
  `tests/io/tests/directory.rs` declares `skein_heap::Counting`, as
  `tests/cuts.rs` does; its reference test:

  ```rust
  #[test]
  fn a_rename_across_cut_at_each_operation_is_in_one_directory_whole() {
      let uncut = renames::world(0).run();
      assert_eq!(uncut.procs[0].found, Some(Found::Destination));
      let (mut source, mut destination) = (false, false);
      for before in 0..uncut.submissions[0] {
          for (seed, cut) in [(0, Cut::Kill), (0, Cut::PowerLoss), (17, Cut::PowerLoss)] {
              let outcome = { let mut scenario = renames::world(seed); scenario.cut(0, before, cut); scenario.run() };
              match outcome.procs[0].found { Some(Found::Source) => source = true, Some(Found::Destination) => destination = true, None => panic!("both listed") }
              assert_eq!(outcome.machine.files.open_handles(), 0);
          }
      }
      assert!(source && destination, "cuts leave the file in either directory");
  }
  ```

  and `a_rename_across_recovers_in_its_destination_once_that_is_synced`.
- The real ring, as `ring.rs:194`: `a_rename_across_over_the_ring_lands_the_file_in_its_destination`.
Proof: process, io on the real kernel: `heavy cargo nextest run
-p skein-io-world -E 'test(=a_rename_across_over_the_ring_lands_the_file_in_its_destination)'`,
once: the bytes std laid in a scratch directory's `staging/index` are
read back by std from `cache/index`, and `staging` is empty. Bound: 1 s.
hearth 32.1 is its first caller, then 33 and 31.
Don't: add a second rename request; change `Op::Rename`, `temp_name`
(`file_layer/store.rs:228`) or a rename in place's crash model.
Size: at most 90 lines.
Removes: `Request::Rename`'s `root` field.

### 04.2 A directory listed in pages
Scope: `ListPage` (io.md, 5, "Listing"), each page filled to `max` or
the directory's end, from where its last page stopped.
Needs: 04.1 (its `tests/directory.rs` files, and `complete` and `listed`
made `pub(super)`).
Files: `crates/skein-io/src/file.rs`; `src/file_layer.rs` (shared:
`Pending` :30 and its matches, `down_inner` :446, `cancel` :648,
`cancelled` :734, `stopped` :851, `up_inner` :993, beside `list` :410
the new `list_at_most`), `src/file_layer/directory.rs`; the users of
`file::Request` whole: `testing/skein-world/src/files.rs:207` (shared),
`crates/skein-oauth-accounts/src/private_tests.rs:111, :367`. Tests:
`src/tests/directory.rs`, new `tests/io/src/pages.rs`,
`tests/io/src/lib.rs`, `tests/io/tests/{directory,ring}.rs`.
Shape:
- `Request::ListPage { owner, directory: Token, max: u32 }` → the
  existing `Event::Listed { owner, entries }`, or `Failed`.
- `directory::page`: an unknown token `Failed { NotFound }`; `max >
  max_entries` or `max == 0` `Failed { InvalidArgument }` before any
  record, as `Scan`'s and `ReadAt`'s `max` 0 (`file_layer.rs:544`; io.md,
  5: `max` is 1 to `max_entries`). Then `Pending::Page { owner, fd, max, entries:
  Vec::with_capacity(max) }` and `list_at_most(fd, max)`.
- Each `List` answered: its entries appended (`Entry { name, kind }`);
  at `Count(0)` or `max` held, `Listed`; else `list_at_most(fd, max −
  held)`. A short `List` never ends a page: only `Count(0)` does.
- io closes nothing: the directory is the owner's, who `Close`s it. A
  `List` is not cancellable (kernel.md, 7), so a deadline or a cancel
  stops the page when the `List` in flight completes, as `Pending::List`
  is settled, with no descriptor of io's to close: `Failed { TimedOut }`
  or `Cancelled`, the entries taken lost, unless that `List` finished the
  page (`Count(0)` or `max` held), which answers `Listed` (io.md, 5).
- `worst_case` unchanged: at most `max ≤ max_entries` entries and one
  `List` record of 32 entries and 8192 bytes, within its entries and
  listing terms (`file_layer.rs:263-278`).
Test shape:
- Step, `src/tests/directory.rs`, under 20 ms each, answered with
  `listed`; expected pages are `Box<[Entry]>` literals:
  `a_page_fills_to_max_across_short_kernel_lists` (max 3: a `List`
  answered one entry, then one asking for 2 answered two: one `Listed` of
  three, and no `Close`), `a_page_shorter_than_max_is_the_directorys_end`,
  `a_page_after_the_end_is_empty`,
  `a_page_past_max_entries_fails_before_any_record`,
  `a_page_of_an_unknown_token_fails_not_found`,
  `a_page_of_zero_fails_invalid_argument`,
  `a_page_cut_short_by_a_deadline_fails_timed_out`,
  `a_page_cut_short_by_a_cancel_is_cancelled`,
  `a_page_the_list_in_flight_finishes_is_listed_past_its_deadline`.
- The world, `tests/io/src/pages.rs` after `files.rs`: the owner opens
  `cache`, laid with the files `pages::names(n)` names, lists it at `max`
  until a page shorter than `max`, closes it, keeping `pages`;
  `Memory::Checked`; seeds 0, 1 and 17, under 0.5 s each:
  `ten_entries_come_in_pages_of_four_four_and_two` (each name once),
  `eight_entries_end_with_an_empty_third_page`,
  `a_page_of_max_entries_stays_within_file_ios_worst_case` (`max` =
  `ENTRIES`: the bound held unchanged).
- The real ring: `a_directory_listed_by_pages_over_the_ring_gives_each_entry_once`.
Proof: process: `heavy cargo nextest run -p skein-io-world -E
'test(=a_directory_listed_by_pages_over_the_ring_gives_each_entry_once)'`,
once: ten files std made in a scratch directory reach the owner over the
real ring in pages of 4, 4 and 2, each name once. Bound: 1 s. hearth
23's scan is its first caller.
Don't: change `List` or `Scan`; sort a page; close, reopen or rewind the
owner's directory; keep a cursor of io's own.
Size: at most 100 lines.
Removes: nothing.

## Interface sketches

The variants at the base stand; these are the new and changed ones.
`file::Request` and `file::Event` are what hearth 23, 24, 31, 32 and 33
call (33 calls `Load` and `Remove`); the rest is skein's own. Private, as the
Shapes give them: `Pending::Page { owner, fd, max, entries: Vec<Entry> }`,
and `FileIo::list_at_most(&mut self, fd: Fd, most: u32, subs: &mut
Queue<Submit>)`, one `List` of `min(32, most)` entries and 8192 bytes.

```rust
// skein_io::file (crates/skein-io/src/file.rs)
pub enum Request {
    /// Moves `from` of the directory open on `from_root` to `to` in the one open on `to_root`: `Renamed` or `Failed` (04.1).
    Rename { owner: Token, from_root: Fd, from: Box<[u8]>, to_root: Fd, to: Box<[u8]> },
    /// At most `max` entries of a directory the owner opened, from where its last page stopped: `Listed` or `Failed` (04.2).
    ListPage { owner: Token, directory: Token, max: u32 },
}

// skein_fake_machine: fs/moves.rs (04.1)
/// A rename between two directories no sync of its destination has decided; `durable_*`: what each
/// name held in its directory's durable copy at the rename.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Move { node: NodeId, from_dir: NodeId, from: Box<[u8]>, to_dir: NodeId, to: Box<[u8]>,
    durable_from: Option<NodeId>, durable_to: Option<NodeId> }
```

An algorithm easy to get wrong: a crash's pending moves (04.1), after
the pending directories are drawn:

```rust
for moved in core::mem::take(&mut self.pending_moves) {
    rng = draw(rng);
    if rng & 1 != 0 {
        entries_of(&mut surviving, moved.to_dir).insert(moved.to, moved.node); // landed: in the destination alone
        entries_of(&mut surviving, moved.from_dir).remove(&moved.from);
    } else {
        restore(entries_of(&mut surviving, moved.to_dir), &moved.to, moved.durable_to); // not landed: both as durable before it
        restore(entries_of(&mut surviving, moved.from_dir), &moved.from, moved.durable_from);
    }
}
```

## Questions to raise when reached

None.

## Done when

Each command once, reading skein's `origin/main` (the last in a worktree
of it, through `heavy`):

```sh
git -C ~/src/rust/skein log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'skein-04\.[12]'
git -C ~/src/rust/skein grep -n -E '^    ListPage \{$|^        from_root: Fd,$' origin/main -- crates/skein-io/src/file.rs
heavy cargo nextest run -p skein-io-world -E 'test(=a_rename_across_over_the_ring_lands_the_file_in_its_destination) | test(=a_directory_listed_by_pages_over_the_ring_gives_each_entry_once)'
```

The first prints 2; the second 2 lines, file.rs's `ListPage` and
`from_root`; the third passes with a `Summary` naming 2 tests.
