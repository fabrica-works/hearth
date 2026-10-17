# Session 02: `hearth-protocol-git`; the vectors, and git's three stream machines

hearth's git machine (render.md, 2.2): 02.1 the vectors, environment,
revision check and rev-parse's answer; 02.2 and 02.3 stream machines for
`ls-tree`'s listing and `cat-file --batch`'s blobs, fed generated output
cut at random by the machine world `tests/git` (`hearth-git-world`).
hearth 11's thin story calls the machines. hearth 34 runs them on real git
(the conformance tier, git's recordings, replayed) and adds the hunks.

## Read first

- hearth's `render.md` (at `175e92f`): 1 (second bullet), 2.1 to 2.3, 6.1,
  7.1, 8.1, 8.2, 9.4; `client/README.md`, 3.5; `workflow.md`, 1 and 2.
- `testing.md`: 2 (table, 2.1, 2.4, 2.8, 2.9), 4.1, 4.2 (git's row), 4.5,
  5.1 to 5.3, 5.5, 9.2; `.config/nextest.toml`.
- `decisions.md`, 9, 10, 12, 13; `findings/skein-testing.md`, 3, 6, 7;
  `findings/skein-io.md`, 2, 6.
- skein's machine to copy: `crates/skein-json/src/tokenizer.rs` (doc 1-45;
  `MaxOut`, `Request`, `Event` 172-220; `up`, `down` 353-420),
  `crates/skein-lib/src/stream.rs:1-90`, `stream/held.rs:9-35`,
  `testing/skein-world/src/stream.rs:10-120` (`Wire`).
- temper's `crates/temper-worker-protocol-git/src/output.rs` (git read
  strictly); `tools/git-transcripts/src/{lib.rs,mask.rs}` (captures, masks).
- skein's `programming-model.md`: 3, 4.3, 7, 8, 10; `testing-strategy.md`:
  2.1, 2.4, 4.1, 8 (by number only).


## What the design asks

- **render.md 2.2:** four vectors as printed; "The environment is given
  whole"; the revision "checked before it reaches git".
- **2.3:** the listing "`<mode> <type> <oid> <size>\t<path>\0`", four
  modes, 65,536 entries, 4,096-byte paths, 16 MiB of paths in all; a blob's header size "must
  equal the listing's and fit the blob's kind" before the body.

## Choices this plan makes

- **Three increments:** 02.2 (listing) and 02.3 (blobs) are one README
  row's, split by decisions.md 20. The conformance tier and the hunks are hearth 34 (34.1,
  34.2). hearth 11 needs hearth 02.3.
- **skein's stream shape** (programming-model.md, 4.3; the tokenizer):
  one `lib::stream` below, `Request`/`Event` above, `up`/`down` emitting at
  most `MaxOut { above: 1, below: 1 }`, `Close` answered by `Closed` in any
  state. rev-parse (65 bytes) is one function. Up go one `Listed` and a
  blob in pieces of `Limits::chunk` (16 KiB); 34.2 adds `Hunks`.
- **A record or a line starts with `Fill(1)`:** `End` there is the
  output's end, `End` in the scan after it a truncation. An end on a
  boundary reads as git's: `Listed` (and 34.2's `Hunks`) stand only with
  `invocation::succeeded`'s exit.
- **git's text as git prints it:** lowercase ids; `-l`'s size in seven
  columns (`%(objectsize:padded)`), `-` and type `commit` for a submodule
  (`Entry::size` 0, the plan's reading of `-`);
  `<oid> missing`, a `tool` error of what needs that blob, the batch
  reading on (render.md 2.2). The revision check is a guard, not a parse
  (2.2): 1 to 255 bytes of `[A-Za-z0-9._/-]`, never a leading `-`; git
  resolves the rest. git runs with `HOME=/dev/null`: no home is made, and
  `environment()` takes no argument (2.2). git is `invocation::GIT`,
  spawned by its path, nothing configuring it in M1 (client/README.md, 3.5):
  `kernel::Spawn::program`; `arguments` are the rest, as `Spawn::args`
  takes them (`skein-io/src/kernel.rs:525`).

## Rules that bind it

- **hearth's first landings** (README; workflow.md, 1): 02.1 may be the
  first crate (it lands with `land`); 02.2's `tests/git/tests/fuzzy_git.rs`,
  if no fuzzy test is on `main`, replaces both profiles' filters with the
  two lines `.config/nextest.toml`'s comments give, and removes them.

## Increments

### 02.1 The vectors, the environment, the check, rev-parse's answer
Scope: render.md 2.2's vectors, environment, check; rev-parse decoded.
Needs: nothing.
Files: root `Cargo.toml`, `Cargo.lock` (shared: the `crates/*` glob, if absent; `skein-lib` by
GitHub URL, no rev; `hearth-protocol-git` by path);
`crates/hearth-protocol-git/`: `Cargo.toml` (`skein-lib` only, `[lints]
workspace = true`), `src/{lib.rs, oid.rs, invocation.rs, rev_parse.rs,
tests.rs}`, `src/tests/{invocation.rs, rev_parse.rs}`.
Shape: `oid::Oid` (`parse` of 40 or 64 lowercase digits, `hex`);
`invocation::{Revision, revision, Refused::{Empty, Long, Byte { at: u32 },
Dash}, GIT, Command, arguments, environment(), Exit, succeeded}` (0 or 1 for
rev-parse, 0 for ls-tree and cat-file, 1 for diff);
`rev_parse::{MAX_STDOUT = 65, decode(stdout, exit) -> Result<Resolved,
Error>}`, `Resolved::{Commit(Oid), Unresolved}` (exit 1, whatever it
printed), `Error::{Exit(Exit), Malformed}`; the caller reads stdout to its
end, keeping `MAX_STDOUT + 1` bytes. Vectors built with `lib::Writer`:
`--git-dir=<repository>`, `<revision>^{commit}` and `<store>/cache/<64
hex>` are one argument each.
Test shape: step tests under the step crates' lints (arrays, `zip`, no
`Vec`), each under 20 ms:
`each_command_s_vector_is_render_md_2_2_s_word_for_word`,
`the_environment_is_given_whole_with_home_dev_null` (render.md 2.2's six
variables in its order, `HOME=/dev/null` last, and no other),
`a_ref_name_of_the_alphabet_up_to_255_bytes_passes` (an abbreviated or
uppercase id and `a..b` among them: git resolves them),
`a_byte_outside_the_alphabet_or_a_256th_byte_is_refused` (the empty
revision too),
`a_revision_starting_with_a_dash_never_reaches_git`,
`rev_parse_s_one_line_decodes_to_the_commit_it_names`,
`rev_parse_exiting_1_is_unresolved_whatever_it_printed` (`a..a` prints two
lines and exits 1), `an_exit_past_1_or_a_signal_is_a_tool_failure`.
Proof: unit: `each_command_s_vector_is_render_md_2_2_s_word_for_word`, the four
vectors word for word; bound 1 s for the crate. Nothing calls them yet:
hearth 34.1 runs them on real git, hearth 11 in the real loop.
Don't: a `PATH` in git's environment (2.2 gives none); the fetch or `init
--bare` (render.md 2.4, the daemon's); an uppercase id read.
Size: at most 280 lines. Removes: nothing.

### 02.2 The listing; the git machine world
Scope: `ls-tree`'s records as a stream machine, with 2.3's two limits;
`tests/git` on generated listings (testing.md, 2.4).
Needs: nothing beyond 02.1.
Files: `src/{lib.rs (MaxOut), limits.rs, listing.rs}`, `src/tests/{fixtures.rs
(new: a machine over lib's Held), listing.rs}`; `tests/git/` (new,
findings/skein-testing.md 7's lints table): `Cargo.toml` (`skein-lib`,
`skein-world`, `hearth-protocol-git`), `src/{lib.rs, fixtures.rs,
generate.rs, world.rs}`, `tests/{listing.rs, fuzzy_git.rs}`;
`tests/clippy.toml` (shared: the first world to land makes it, as skein's
at `f981746`); root `Cargo.toml` (the `tests/*` glob, if absent),
`Cargo.lock`; `.config/nextest.toml` if
the first fuzzy test (Rules).
Shape: `limits::{Limits { entries, path, paths_total, chunk }, SHIPPED}`;
`listing::{Mode, Entry, Request::{List, Close}, Event, Listing, up, down,
UP_MAX_OUT, DOWN_MAX_OUT, largest_demand}`; `Error::{Malformed { entry:
u32 }, TooMany { path }, PathTooLong { prefix }, PathsTooLarge { path }, Truncated,
Stream(Fault)}`. A record: `Fill(1)`, then `Scan { until: NUL, max: RECORD
+ limits.path }` (`RECORD` 100): one of the four modes, `blob` (`commit`
for `160000`), `Oid::parse`, the size padded as git pads it, a tab, 1 to
`path` bytes. The entry past `entries` fails `TooMany` with its path;
`max` bytes and no NUL, `PathTooLong` with the first `path` bytes. The
entry whose path takes the running sum of the paths' bytes (no NULs) past
`paths_total` fails `PathsTooLarge` with its path (render.md 2.3).
Test shape: unit, through `fixtures::listed(stdout, &limits)`, under 20
ms: `a_listing_record_of_each_mode_decodes_to_its_class` (reference),
`a_size_not_padded_as_git_pads_it_is_malformed`,
`an_uppercase_or_short_object_id_is_malformed`,
`the_entry_past_the_limit_fails_naming_its_path`,
`the_entry_whose_path_passes_the_total_fails_naming_its_path` (`paths_total:
64`: two 30-byte paths list, a third fails),
`a_path_past_the_limit_fails_with_its_prefix`,
`an_end_within_a_record_is_truncated`. World: `generate::listing` prints
drawn entries with an encoder of its own; `world::list(stdout, &limits,
&Settings)` plays a `Wire` below (fragments of 1 to 30 bytes; an end, or
`Fault::Reset`, after a drawn byte) and above a `List` and a `Close` at a
drawn step. Limits: `fixtures::SHIPPED` (`limits::SHIPPED`) and `SMALL`
(`entries: 8, path: 32, paths_total: 256`). Focused, under 0.5 s:
`a_generated_listing_decodes_to_its_entries_whatever_the_cuts` (reference),
`a_close_in_every_state_answers_closed_and_demands_nothing`,
`a_stream_that_fails_mid_listing_fails_the_listing`. Fuzzy:
`random_listings_cut_ended_failed_and_closed_settle` (512 seeds, about 2
s; an end on a boundary lists what came before, else `Truncated`).

```rust
// crates/hearth-protocol-git/src/tests/listing.rs
#[test]
fn a_listing_record_of_each_mode_decodes_to_its_class() {
    let stdout = b"100644 blob ce013625030ba8dba906f756967f9e9ca394464a       6\ta b.json\0\
100755 blob 1a2485251c33a70432394c93fb89330ef214bfc9      10\tbin/x\0\
120000 blob d0f12b94eeb3c5301ed4468f5c26e689c113ae10      11\tlink\0\
160000 commit ce013625030ba8dba906f756967f9e9ca394464a       -\tsub\0";
    let Event::Listed(entries) = fixtures::listed(stdout, &limits::SHIPPED) else { panic!("git's own listing is listed") };
    let expected = [(Mode::File, 6_u64), (Mode::Executable, 10), (Mode::Link, 11), (Mode::Submodule, 0)];
    assert_eq!(entries.len(), expected.len(), "a record each");
    for (entry, (mode, size)) in entries.iter().zip(expected) { assert_eq!((entry.mode, entry.size), (mode, size), "its class, its size"); }
}

// tests/git/tests/listing.rs
#[test]
fn a_generated_listing_decodes_to_its_entries_whatever_the_cuts() {
    for seed in 0..16 {
        let listing = generate::listing(&mut Rng::new(seed), &fixtures::SMALL);
        let outcome = world::list(&listing.stdout, &fixtures::SMALL, &Settings::cut(seed));
        assert_eq!(outcome, Outcome::Done(listing.entries), "seed {seed}");
    }
}
```
Proof: world: `a_generated_listing_decodes_to_its_entries_whatever_the_cuts`,
drawn modes, sizes and paths listed whatever the fragments; bound 0.5 s.
No caller until hearth 11; real git's listings in hearth 34.1.
Don't: check git's order; map to render's errors (hearth 11); hold a
carry buffer (the side below's).
Size: at most 220 lines. Removes: nothing.

### 02.3 The blobs: `cat-file --batch`; the decoders' worst case
Scope: the request line; the header against the listing's size and the
kind's maximum, before the body; the body in pieces; its `\n`; `missing`;
what one listing or batch holds (programming-model.md, 6.3), so that
hearth 28's builder worst case can sum it.
Needs: nothing beyond 02.2 (`skein-heap` is in skein at hearth's pin).
Files: `src/{lib.rs, limits.rs, batch.rs}`, `src/tests/{fixtures.rs,
batch.rs}`; `tests/git/{Cargo.toml (skein-heap, dev), src/fixtures.rs,
src/generate.rs, src/world.rs}`, `tests/git/tests/{batch.rs, memory.rs
(new), fuzzy_git.rs}`; root `Cargo.toml` (shared: `skein-heap` by URL),
`Cargo.lock`.
Shape: `batch::{request_line, Request, Event, Batch, up, down, UP_MAX_OUT,
DOWN_MAX_OUT, largest_demand}`; `Error::{Malformed, Other { oid }
(another object, or not a blob), Size { oid, listed, header }, TooLarge {
oid, size, max }, Truncated, Stream(Fault)}`. The header: `Scan { until:
LF, max: HEADER }` (93); the body: `Arrived` of at most `chunk` and what is
left; then `Fill(1)`, a `\n`. The protocol layer sends `request_line` on
stdin, `Finish` after the last, and `max` from render's `as` (2.3: 1 MiB
a document or template, 64 MiB a static file).
`limits::worst_case(Decoder, &Limits) -> Option<u64>`, `Decoder::{Listing,
Batch}`: a listing's entries `List` at `entries`, their paths together at
`paths_total` bytes, and one delivery of `largest_demand`; a batch's one delivery of
`HEADER` or `chunk`; what goes up (`Listed`'s box, a `Piece`) is the
receiver's. `fixtures::at_the_limits(&limits)` gives each decoder's inputs
at and one past each limit.
Test shape: unit, each written as 02.2's reference is:
`a_blob_s_header_body_and_line_feed_read_whole` (`ce0136… blob
6\nhello\n\n` under `chunk: 4` answers `Piece(b"hell")`, `Piece(b"o\n")`,
`Whole`), `a_header_whose_size_is_not_the_listing_s_fails_first`,
`a_blob_past_its_kind_s_maximum_fails_before_its_body`,
`a_missing_object_answers_missing_and_the_batch_reads_on`,
`a_body_not_ended_by_a_line_feed_is_malformed`,
`the_request_line_is_the_id_and_a_line_feed`. World, as 02.2's:
`generate::batch` (empty blobs, NULs, absent ids), `world::read`, a `Next`
at drawn delays: `generated_blobs_arrive_whole_in_order_whatever_the_cuts`,
`a_close_between_pieces_answers_closed_and_withdraws_its_demand`; fuzzy
`random_batches_cut_ended_failed_and_closed_settle` (512 seeds, about 2
s). Memory, a binary of its own (findings/skein-testing.md, 2), at `SMALL`
(the shipped listing's paths alone are 16 MiB; hearth 28 sums
`worst_case` at `SHIPPED`), under 0.5 s, `tests/git/tests/memory.rs`
(`#[global_allocator] static HEAP: Counting`):
`each_decoder_at_its_limits_holds_no_more_than_its_worst_case` takes
`limits::worst_case` as `bound` for each `at_the_limits` case, makes
`world::Stepped::new(&case, &SMALL)` and a `Meter::new()`, calls
`run.open()` under the meter, then, while `run.pending()`: `meter.start()`,
`run.call()` (one entry point), `meter.end()`, `run.drop_events()`,
`meter.check(step, bound, &case.what)`.
Proof: world: `generated_blobs_arrive_whole_in_order_whatever_the_cuts`,
each blob's bytes in order and whole, absent ones `Missing`; bound 0.5 s;
and `heavy cargo nextest run -p hearth-git-world -E 'binary(memory)'` once,
bound 1 s. No caller until hearth 11.
Don't: write stdin (one stream below); hold a body whole; hash it (hearth
11, on `skein_lib::sha256`); lower a limit to pass the memory test.
Size: at most 250 lines. Removes: nothing.


## Interface sketches

This session owns these; hearth 11 translates the listing, the pieces and
the errors into render's `Listed`, `Read`, `Cached` and `Failed`. Each
`up`/`down` has the tokenizer's signature over its own state and
`Env<Limits>`. Every type derives `Clone, PartialEq, Eq, Hash, Debug`;
`Limits`, `Decoder`, `Command`, `Exit`, `Mode` also `Copy`; `Mode`, `Oid`
also `PartialOrd, Ord`. hearth 34.2 adds `Kind`, `Line` and `Hunks`.

```rust
// crates/hearth-protocol-git/src/limits.rs (02.2; Decoder and worst_case 02.3)
/// git's machines' limits (programming-model.md, 7); 34.2 adds `hunk_lines`, `hunk_file`; 42.1 `tail_lines`, `tail_line`.
pub struct Limits { pub entries: u32, pub path: u32, pub paths_total: u32, pub chunk: u32 }
/// render.md 2.3's defaults, and this plan's chunk: the shipped profile's.
pub const SHIPPED: Limits = Limits { entries: 65_536, path: 4_096, paths_total: 16_777_216, chunk: 16_384 };
/// One of the crate's stream machines: `listing::Listing`, `batch::Batch`; 34.2 adds `Diff`, 42.1 `Tail`.
pub enum Decoder { Listing, Batch }
/// The most heap one decoder holds of its own at once; `None` past a `u64`.
pub fn worst_case(decoder: Decoder, limits: &Limits) -> Option<u64>;

// crates/hearth-protocol-git/src/invocation.rs (02.1)
/// git's path: render.md 2.2, "git is `/usr/bin/git`".
pub const GIT: &[u8] = b"/usr/bin/git";
/// One of render.md 2.2's commands, with what its vector names.
pub enum Command<'a> {
    /// `--git-dir=<repository> rev-parse --verify --quiet <revision>^{commit}`.
    RevParse { repository: &'a [u8], revision: &'a Revision },
    /// `--git-dir=<repository> ls-tree -r -l -z --full-tree <commit>`.
    LsTree { repository: &'a [u8], commit: Oid },
    /// `--git-dir=<repository> cat-file --batch`, one process per revision read.
    CatFile { repository: &'a [u8] },
    /// `--git-dir=/dev/null diff --no-index … -- <store>/cache/<before> <store>/cache/<after>`: no repository, wherever it runs.
    Diff { store: &'a [u8], before: &'a [u8; 32], after: &'a [u8; 32] },
}
/// How git ended: mirrors skein-io's `kernel::Exit` (kernel.rs:600), which a machine cannot name.
pub enum Exit { Code(u8) /* its exit status */, Signal(u32) /* the signal that ended it */ }

// crates/hearth-protocol-git/src/listing.rs (02.2)
/// A blob's class by its mode (render.md, 2.2): `100644` File, `100755` Executable, `120000` Link (its blob
/// is the link's target), `160000` Submodule (a commit, no blob).
pub enum Mode { File, Executable, Link, Submodule }
/// One record: mirrors render.md 8.2's `Listed` entry (path, mode, oid, size); render's mirrors it (hearth 11).
pub struct Entry { pub path: Box<[u8]>, pub mode: Mode, pub oid: Oid, pub size: u64 }

// crates/hearth-protocol-git/src/batch.rs (02.3): `Request::{Read { oid, size, max }, Next, Close}`
/// To the side above, one for each `Read` or `Next`.
pub enum Event {
    /// The body's next bytes, at most `Limits::chunk`.
    Piece(Box<[u8]>),
    /// The blob is whole, its `\n` read; a `Read` may follow.
    Whole { oid: Oid, size: u64 },
    /// `<oid> missing`; a `Read` may follow.
    Missing { oid: Oid },
    /// Nothing follows but `Closed`.
    Failed(Error),
    /// Terminal.
    Closed,
}
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth ls-tree -d --name-only origin/main
   crates/hearth-protocol-git tests/git | wc -l` prints 2.
2. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-02\.[1-3]'` prints 3.
