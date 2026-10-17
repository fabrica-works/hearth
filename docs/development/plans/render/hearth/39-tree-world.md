# Session 39: `hearth-tree-world` (M1): the machine world and the worst cases of `hearth-protocol-tree`

`tests/tree` (`hearth-tree-world`), the machine world of hearth 09's
`hearth-protocol-tree` (decisions.md, 9 and 10): seeded generators of valid
records of its three kinds, cuts and mutations, an oracle, and the sweeps
that fuzz the decoder when it lands (render.md, 9.4); with
`limits::worst_case` and `limits::held`, checked against skein-heap's
counting allocator. Split from hearth 09 (decisions.md, 20). Nothing calls
the machine until hearth 23; hearth 28.5 sums `held` into the protocol
layer's worst case.

## Read first

hearth's store.md 3.1, 6.1, 6.8; render.md 8.3, 9.4; testing.md 2.4. hearth
09: Choices, its sketches and `src/tests/fixtures.rs`. code-organization.md
4, 6, 7; README, Rules ("Shared files", "Test time"); decisions.md 9, 10;
`findings/skein-testing.md` 2, 3, 7. skein's programming-model.md 3, 6.3, 7
and testing-strategy.md 2.1, 6, by number only.

## What the design asks

- **Fuzzing** (render.md, 9.4): "every decoder is fuzzed"; decisions.md, 10
  meets it with seeded sweeps in `fuzzy_*.rs`, and 9 brings the tree world to M1.
- **Memory** (programming-model.md, 6.3): a machine's worst case is a bound
  its tests hold it to, by a counting allocator.

## Choices this plan makes

- **The world is ordinary Rust,** a crate of its own, `hearth-tree-world`
  (decisions.md, 9), with its own `[lints]`; it writes records through
  hearth 09's writers and never by hand (hearth 09, Choices).
- **Sweeps are seeded** (decisions.md, 10): the focused suite holds none.

## Rules that bind it

- **Test time** (README, Rules): the world at most about 1 s focused and 5 s
  fuzzy (development.md, 5.3); each focused test under 0.5 s.
- **Limits** come from `fixtures::limits()`, which is hearth 09's `SHIPPED`;
  a test that needs another names it beside it, with its reason.

## Increments

### 39.1 The machine world, and the worst cases

Scope: `tests/tree`, ordinary Rust: seeded generators of valid records of
the three kinds within the shipped limits; cuts at every prefix and at
random; mutations (a byte changed, dropped or inserted; the length field one
more or one less; the kind or version changed); an oracle. And
`limits::worst_case(kind)` (through a crate-private `decode::worst_case`) and
`limits::held(kind)` against skein-heap's counting allocator.
Needs: hearth 09.3.
Files: `crates/hearth-protocol-tree/src/{decode.rs, limits.rs}`; new
`tests/tree/Cargo.toml`, as `findings/skein-testing.md` 7's world (its own
`[lints]`; `hearth-protocol-tree`, `hearth-lib`, `skein-lib`;
dev-dependency `skein-heap`); new `tests/tree/src/{lib.rs, fixtures.rs,
generate.rs, mutate.rs, check.rs}` and `tests/tree/tests/{frames.rs,
memory.rs, fuzzy_tree.rs}`. Shared (README, Rules): the root `Cargo.toml`
(`hearth-protocol-tree = { path = "crates/hearth-protocol-tree" }` in
`[workspace.dependencies]`; `members` globs `tests/*`, and `skein-heap` is
there) and `Cargo.lock`; `tests/clippy.toml` and `.config/nextest.toml`
are on main, as tests/git landed them.
Shape:
- `limits::worst_case` is what a read holds of its own beside the record it
  hands out: the decoder's stack and previous keys, the inputs'
  `List<Item>` before it is boxed, and `held(kind)`: a read that fails
  holds the record it built so far and hands none out. `limits::held` is the most a record holds
  once read, for hearth 28.5's worst case: each list's items at the kind's
  limits, and box bytes up to the body's bound in all. Both use checked
  arithmetic: `None` past `encode::DEPTH` for every kind; past a `u64`
  for a resolution only (`lines`, `indices` and `inputs` at `u32::MAX`).
- `generate`: `small` (kind 1 or 5), `resolution(…, packages)`,
  `one_of_each` (records of at most 64 packages), and `resolution_at_its_limits` (8,192 packages, 8 lines of
  16 indices, filenames grown until the body is within 1 KiB of 4 MiB).
  `check`: `Record` (with `kind()`), `read` (with a `# Errors` doc
  section: the world's lints deny its lack), `write` (expects success),
  `canonical` (a frame that reads writes back to itself; a body the decoder
  takes re-encodes through `Encoder::item` byte for byte) and `sweep(rng,
  &record, limits, seed)` (it reads back, no cut reads, and each of 8
  mutations is unreadable or canonical).
Test shape: focused tests under 0.5 s each; limits from `fixtures::limits()`.
The reference test, in `tests/frames.rs`:

```rust
#[test]
fn no_cut_of_a_generated_frame_reads_as_a_record() {
    let limits = fixtures::limits();
    for seed in 0..16_u64 {
        for record in generate::one_of_each(&mut Rng::new(0x7EE0_0000 + seed), &limits) {
            let frame = check::write(&record, &limits);
            for cut in 0..frame.len() {
                assert!(matches!(check::read(record.kind(), &frame[..cut], &limits), Err(Unreadable::Short | Unreadable::Length { .. })), "{seed} {cut}");
            }
        }
    }
}
```

In `tests/memory.rs`, with `#[global_allocator] static HEAP: Counting =
Counting;` (skein's `testing/skein-heap/src/lib.rs:57`; `Meter` at :341):

```rust
#[test]
fn reading_a_resolution_at_its_limits_holds_no_more_than_its_worst_case() {
    let limits = fixtures::limits();
    let frame = check::write(&Record::Resolution(generate::resolution_at_its_limits(&limits)), &limits);
    let (own, held) = (limits::worst_case(Kind::Resolution, &limits).expect("fits a u64"), limits::held(Kind::Resolution, &limits).expect("fits a u64"));
    let meter = Meter::new(); meter.start();
    let read = resolution::read(&frame, &limits).expect("a record at its limits reads");
    let step = meter.end();
    assert!(step.held() <= held, "the record it hands out: {} of {held}", step.held());
    drop(read);
    meter.check(step, own, &"a resolution at its limits, besides its record");
}
```

`tests/fuzzy_tree.rs` follows jig's `tests/client/tests/fuzzy_joint.rs`,
lines 5 to 7: `FIRST_SEED = 0x7EE0_F000`, `SEEDS = 2_000`, `PINNED: &[u64] =
&[]`. Its reference, about 2.5 s:

```rust
#[test]
fn random_store_and_cache_file_frames_cut_and_mutated_read_as_the_oracle_says() {
    let limits = fixtures::limits();
    for seed in (FIRST_SEED..FIRST_SEED + SEEDS).chain(PINNED.iter().copied()) {
        let mut rng = Rng::new(seed);
        let record = generate::small(&mut rng, &limits);
        check::sweep(&mut rng, &record, &limits, seed);
    }
}
```

The others: in `frames.rs`, `every_generated_record_reads_back_as_it_was_generated`,
`a_mutated_frame_is_unreadable_or_its_records_one_encoding`,
`every_body_the_decoder_takes_the_encoder_writes_again_byte_for_byte` and
`a_frame_of_each_kind_is_unreadable_as_the_other_two`; in `memory.rs`,
`reading_generated_cut_and_mutated_frames_holds_no_more_than_the_worst_case`,
`worst_case_and_held_are_none_past_a_u64_for_a_resolution_only` (`None`
for both on `Kind::Resolution`, `Some` for both on `Store` and
`CacheFile`) and
`worst_case_and_held_are_none_past_the_encoders_depth` (limits set to
overflow, then `depth` one past `encode::DEPTH`);
in `fuzzy_tree.rs`, `random_resolution_frames_cut_and_mutated_read_as_the_oracle_says`
(200 seeds of up to 512 packages, about 2.5 s).
Proof: world, above hearth 09's unit tests. Run once: `heavy cargo
nextest run -p hearth-tree-world -E
'test(a_mutated_frame_is_unreadable_or_its_records_one_encoding)'`; it shows
that a frame of each kind changed at random is refused, or is the one
encoding of the record it reads as. Bound: 0.5 s. Nothing above calls the
machine until hearth 23.
Don't: hand-write a frame as a fixture; read `std::env::var`; lower a limit
to pass the memory test; put a sweep in the focused suite.
Size: at most 80 lines, the worst cases (the world, under `tests/`, is not
counted).
Removes: nothing.

## Interface sketches

Added to hearth 09's `limits.rs` sketch; the world's own items (`generate`,
`check`) are test code, not sketched.

```rust
// crates/hearth-protocol-tree/src/limits.rs (39.1; the rest is hearth 09.1's)
/// What one read of `kind` holds of its own beside its record, a failed read's partial record included; `None` past `encode::DEPTH`, and for a resolution past a u64.
#[must_use] pub fn worst_case(kind: Kind, limits: &Limits) -> Option<u64>;
/// The most a record of `kind` holds once read, for its holder's worst case (hearth 28.5); `None` past `encode::DEPTH`, and for a resolution past a u64.
#[must_use] pub fn held(kind: Kind, limits: &Limits) -> Option<u64>;
```

## Questions to raise when reached

None.

## Done when

Each runs once, reading hearth's `origin/main`:
1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-39\.[1-1]'` prints 1.
2. In a worktree at `origin/main`, `heavy cargo nextest run -p
   hearth-tree-world` passes, its `Summary` naming at least 9 tests, and
   `heavy cargo nextest run -p hearth-tree-world --profile fuzzy` passes,
   naming 2.
