# Session 03: state digests: `StateHasher`, lib's `Hash`, replay and the trace

A seed replays to the same trace, but nothing digests a state
(`findings/skein.md`, 3; `findings/skein-testing.md`, 1 and 5). 03.1
lands lib.md, 5.1's keyless hasher over `skein_lib::sha256`; 03.2 a `Hash`
for each of lib's types over what it holds, never its capacity; 03.3
`Host::state_digest` and `Machine::state_digest`, `None` by default, read
from a run's `Outcome`; 03.4 the digests written into the trace after each
iteration under the world's `digests` option. Every change is additive for
smith, jig and temper. hearth 28 (the builder's simulated world, replay
with digests) is the first user downstream, and needs all four.

## Read first

- skein's `docs/design/lib.md`: 5, 5.1, 6.2, 10; `testing.md`, 5, 7;
  `shell.md`, 12; `simulator.md`, 3, 5, 7.
- This plan: README, Rules ("The gate", "Test time", "Shared files");
  `code-organization.md`, 6, 7; `decisions.md`, 1, 15; `findings/`:
  `skein.md` 3, 4, `skein-testing.md` 1, 5, `measurements.md`.
- skein-lib's `src/sha256.rs` (`Sha256`), `src/tests/sha256.rs` (`hex`, :21);
  `programming-model.md`: 2.1, 6.3, 10.2, 10.3.

## What the design asks

- **The hasher** (lib.md, 5.1): a keyless `Hasher` over SHA-256; integers
  little-endian at their width, `write_usize` and `write_isize` 8 bytes;
  `digest(self)`; `finish()` its first 8 bytes big-endian; `state_digest`.
- **lib's types:** 5.1's table; "Never hashed: a container's capacity ...
  and its layout in memory"; lib.md, 10's equalities (fill orders, wrapped
  queues, reused slots digest alike).
- **The traits** (testing.md, 5; shell.md, 12): `Host` and `Machine` give
  `state_digest() -> Option<[u8; 32]>`, `None` by default; `drive` never
  calls it. **A run's end:** a world reads them from its `Outcome`, so
  `assert_replays` compares them, its signature unchanged. **Each
  iteration:** with `digests`, the harness writes them into the trace
  (simulator.md, 7: "beside the records").

## Choices this plan makes

- **`StateHasher` and `state_digest` at lib's root,** from a private `mod
  digest`; `StateHasher` holds one `Sha256`.
- **A count is written as std writes a length,** `write_usize`. `Map`,
  `Set`, `Queue` hash through their std collection's `Hash`; `List`,
  `Stack`, a slab's lists loop item by item, never std's `hash_slice` of a
  `usize` (lib.md, 5.1).
- **The less obvious rows:** `Journal` and `RequestTable` every field but
  `limits`; `Intake` its bytes, not `searched` (a scan's memo); `CryptoRng`
  `stream`, the block counter (`words` 12 and 13), `used` and `exhausted`,
  never `words` 4 to 11 (the key) or `block`; `Id`, `Token`, the times,
  `Secret` and `Rng` already derive it.
- **Who gives a digest in skein:** the echo's shell, over its domain; the
  harness tests' scripts; `PipeHost`, its inner host's; the fake machine,
  every field. All others keep `None`.
- **The trace's entry** is `skein_sim::Event::Digest([u8; 32])` from
  `Sim::trace_digest`: a process's under its pid, the machine's under
  `Pid::MACHINE` (`u32::MAX`), since each `Entry` names a pid
  (`trace.rs:14-18`), printed `machine`; a `None` is not written
  (simulator.md, 7). A process's digest is taken after its `iterate` and
  drain, before its submit; the machine's once every process has had its
  turn (testing.md, 5).

## Rules that bind it

- **skein 04 and 05 add fields to the fake machine** (`pending_moves`,
  `Handle::locked`); the one that lands second digests them. At its
  rebase before landing, 03.3 runs `rg -n 'pending_moves|locked'
  testing/skein-fake-machine/src/fs.rs`: a match, it digests the field and
  adds a test that a pending move or a held lock changes the digest; none,
  skein 04.1 or 05.1 does the same once `fn state_digest` is there.
- **The gate** (README, Rules; decisions.md, 1): `flock
  ~/src/rust/tmp/hearth/render/land-skein.lock land --gate '<the README's
  command, whole>'`, its commit body saying so in one line; no probe of
  the machine.
- **Test time** (README, Rules; decisions.md, 15): this session's
  allotment is 0.3 s of skein's focused suite (6.46 s) and 0.5 s of its
  fuzzy suite (41.84 s) (skein 02: 0.4 s / 1 s; 03: 0.3 s / 0.5 s; 04:
  0.4 s / 1.5 s; 05: 0.4 s / 2 s); it adds no fuzzy test. Each landing's
  report names both times.
- **Additive downstream** (README, Repin): each trait method has a default
  body; the new `Event` variant meets no exhaustive match outside skein
  (smith's `tests/local-process/tests/terminal.rs:34` is a `matches!`);
  skein's own matches take the arm, at most 5 lines each.

## Increments

### 03.1 `StateHasher` and `state_digest`
Scope: lib.md, 5.1's hasher and `state_digest`.
Needs: nothing; `skein_lib::sha256::Sha256` (`new`, `update`, `finish`,
`Clone`) is on skein's `main`.
Files: `crates/skein-lib/src/digest.rs` (new); `src/lib.rs` (`mod digest;`
among :18-44, `pub use` among :46-68, the doc :1-8); tests:
`src/tests/digest.rs` (new), `src/tests.rs` (`mod digest;`),
`src/tests/sha256.rs` (`hex`, :21, made `pub(super)`).
Shape: as the sketch. `write` updates the `Sha256`; each `write_<int>`
writes `to_le_bytes()`, `write_usize` and `write_isize` those of
`u64::try_from(n)` and `i64::try_from(n)` (`expect`: no target's `usize`
passes 64 bits); `finish` ends a clone and reads 8 bytes big-endian.
Test shape: `src/tests/digest.rs`, unit, each under 20 ms; expected
digests are `sha256sum`'s of the bytes each comment prints, read by `hex`.
Tests: `a_u16_digests_as_its_two_little_endian_bytes` (below); `a_usize_digests_as_the_eight_bytes_of_its_u64` (`1_usize`; `printf
'\001\000\000\000\000\000\000\000'`);
`a_negative_isize_digests_as_the_eight_bytes_of_its_i64` (eight `\377`);
`bytes_written_in_pieces_digest_as_written_once` (`a`, then `bc`: `abc`'s
`ba7816bf…15ad`); `finish_gives_the_digest_s_first_eight_bytes_without_ending_it`
(after `write_u16(0x0102)`, `0x25df_d29c_0961_7dcc`; then `write_u8(3)`,
`printf '\002\001\003'`); `a_tuple_hashed_whole_digests_its_fields_in_order`
(`(1_u8, 2_u32)`, `printf '\001\002\000\000\000'`).
```rust
use core::hash::Hasher;
use super::sha256::hex;
use crate::{StateHasher, state_digest};
#[test]
fn a_u16_digests_as_its_two_little_endian_bytes() {
    // printf '\002\001' | sha256sum
    let printed = hex("25dfd29c09617dcc9852281c030e5b3037a338a4712a42a21c907f259c6412a0");
    let mut hasher = StateHasher::new();
    hasher.write_u16(0x0102);
    assert_eq!(hasher.digest(), printed, "low byte first, at the integer's own width");
    assert_eq!(state_digest(&0x0102_u16), printed, "a value hashed whole writes the same");
}
```
Proof: unit (lib has no world; 03.3 is the first user), `heavy cargo
nextest run -p skein-lib -E
'test(=tests::digest::a_usize_digests_as_the_eight_bytes_of_its_u64)'`: a `usize`
digests as `sha256sum` digests its 8 little-endian bytes. Bound: 0.02 s.
Don't: take a key or a seed; touch skein-io's `digest`.
Size: at most 80 lines.
Removes: nothing.

### 03.2 lib's types hash what they hold
Scope: lib.md, 5.1's table: `Hash` for `Slab`, `Map`, `Set`, `Queue`,
`Stack`, `Deadlines`, `Intake`, `CryptoRng`, `Journal`, `RequestTable`;
`List`'s made item by item.
Needs: nothing beyond 03.1.
Files: `crates/skein-lib/src/{slab,map,set,queue,list,stack,deadlines,intake,crypto_rng,journal,requests}.rs`;
tests: `src/tests/digest.rs`. `List`'s `Hash` has one user,
`src/tests/containers.rs:22`, unchanged; no other container has one.
Shape: an impl each, as the sketch. `Slab`: each slot in index order, the
free ones included (lib.md, 5.1), as a tag byte (0 vacant, 1 occupied, 2
spent) and its fields, no slot count (it is the capacity); then the free list's count and slots, then the retired
list's. `Deadlines`: `by_time` alone (`Set<(Time, K)>`, `expire`'s
order). The private types `Journal` and `RequestTable` hold (`Held`,
`Entry`, `State`, `Anchor`) and `Commit`, `JournalRoom`, `RequestRecord`,
`RequestProgress` derive `Hash`.
Test shape: `src/tests/digest.rs`, unit, each under 20 ms, as 03.1's;
capacities and items small and named (lib has no shipped profile). Tests:
`a_queue_digests_its_count_then_its_items` (one `0x0102_u16`; `printf
'\001\000\000\000\000\000\000\000\002\001'`);
`ordered_containers_filled_in_two_orders_digest_alike` (a `const` table:
`Map`, `Set`, `Deadlines`); `a_queue_that_wrapped_digests_as_one_filled_once`;
`slabs_whose_slots_were_reused_in_one_order_digest_alike` (capacity 2; one
inserts `x`, `z`, retires and reclaims `x`, inserts `y`; the other
inserts `x`, retires and reclaims it, inserts `y`, `z`: both end slot 0
occupied by `y` at generation 1, slot 1 by `z` at generation 0, no free
slot); `a_change_to_one_entry_changes_a_map_s_digest`;
`containers_of_any_capacity_holding_the_same_items_digest_alike` (`Queue`,
`List`, `Stack`, `Map`, `Set`, `Deadlines` at 2 and 8);
`an_intake_digests_the_bytes_it_holds_alone` (capacity 4, `zab` then `c`
appended, `Fill(1)` met; capacity 16, `abc`, `Line { max: 16 }` unmet, its
memo set: alike; `abd` otherwise);
`journals_at_one_commit_by_two_histories_digest_alike` (limits of 2: one
write and one held output, accepted, committed as 1 and released; limits
of 4: `from_durable(.., 1)`; `from_durable(.., 2)` otherwise);
`crypto_rngs_with_other_keys_digest_alike_after_the_same_draws`;
`a_request_table_s_digest_changes_with_each_request_asked` (and not with
its limits).
Proof: unit (lib has no world; 03.3 is the first user), `heavy cargo
nextest run -p skein-lib -E
'test(=tests::digest::slabs_whose_slots_were_reused_in_one_order_digest_alike)'`: two
slabs of different histories digest alike. Bound: 0.02 s.
Don't: hash a capacity, a `limits`, a B-tree's nodes, a ring's head,
spare room or an address; give `Hash` to a type outside the table, or
to `Env<L>` or `stream::Held` (lib.md, 5.1: not state); change a container's behaviour or `Debug`.
Size: at most 170 lines.
Removes: `List`'s slice hash (`list.rs:143-147`), replaced.

### 03.3 `Host` and `Machine` digests, read at a run's end
Scope: testing.md, 5 and shell.md, 12: `state_digest` on both traits;
`Outcome::state_digests` for `assert_replays`'s `T`; the digests Choices names.
Needs: 03.2's `Hash` for `Slab` (the echo's domain).
Files: `crates/skein-shell/src/drive.rs` (`Host`, :12-49);
`testing/skein-world/src/` (shared): `program.rs` (`Machine`, :90-126),
`world.rs` (beside `Outcome`, :113-136), `writer_pipe.rs` (:279),
`lib.rs` (:56), `domain/trace.rs` (:31-35's doc: digests go in `T`);
`testing/skein-fake-machine/src/fs.rs` (`Hash` on `How`, `Is`, `Node`,
`Body`, `Handle`, `Machine`); `examples/echo/{domain,service,shell}/src/lib.rs`
(`Hash` on `Domain` :77 and `Session` :84; `Service::domain`; `Echo`'s
impl :26; lines named in the report).
Tests: `tests/world/src/lib.rs` (`Script` :198 gives a digest and
`noting`; `Stamped`, a machine of one fixed digest),
`tests/world/tests/{digests.rs (new), writer_pipe.rs}`,
`testing/skein-fake-machine/src/tests.rs`, `tests/io/src/files.rs`
(`FileSystem`, :270), `tests/io/tests/files.rs`, `tests/echo/src/proc.rs`
(:51), `tests/echo/tests/scenarios.rs`. Other implementors stay: `Host` 98
(skein 38, smith 24, jig 15, temper 21: `rg -n --type rust
'impl(<[^>]*>)? +(\w+::)?Host +for' <repo> -g '!target'`), `Machine` 22
(skein 11, smith 5, jig 5, temper 1); `SignalHost<H>` (jig, temper) gives
`None` until it forwards.
Shape: as the sketch. `state_digests` reads `procs` in order, then
`machine`. A script hashes `received`, `next` and a `noted: u64` its test
sets. The echo's shell gives `Some(state_digest(self.svc.domain()))`;
`Proc` forwards it.
Test shape: `tests/world/tests/digests.rs` (new) under
`skein_heap::Counting`, worlds made as `tests/world/tests/world.rs:26-45`
makes them, each under 0.5 s. Tests:
`an_outcome_gives_each_process_s_digest_and_the_machine_s` (below);
`the_default_digests_of_a_host_and_a_machine_are_none`
(`skein_world_tests::hosted::Script::new(&[])`, `NoMachine`);
`a_seed_replays_to_the_same_state_digests_through_assert_replays`;
`a_pipe_host_gives_the_digest_of_the_host_it_wraps` (`writer_pipe.rs`);
`machines_laid_with_the_same_items_digest_alike` and
`a_write_to_a_file_changes_the_machine_s_digest` (the fake machine);
`a_store_replays_to_the_same_files_digest_at_its_end` (tests/io, `world(7,
Config::calm(), Story::Replace, None, 3)`, `files.rs:13`'s call with no
step failed); the Proof.
```rust
use skein_world::{Memory, World};
use skein_world_tests::{Expect, Judge, Script, Stamped, client, ms, server};
#[global_allocator] static HEAP: skein_heap::Counting = skein_heap::Counting;
#[test]
fn an_outcome_gives_each_process_s_digest_and_the_machine_s() {
    let judge = Judge::new(7, vec![Expect::Receives { at: 0, by: ms(1_000) }]);
    let mut world = World::<Script, Judge>::new(7, skein_sim::Config::calm(), judge, Memory::Checked).with_machine(Stamped([7; 32]));
    world.spawn(server); world.spawn(|| client(ms(10), ms(20)).noting(3));
    let digests = world.run().state_digests();
    assert!(digests.procs.len() == 2 && digests.procs.iter().all(Option::is_some) && digests.procs[0] != digests.procs[1]);
    assert_eq!(digests.machine, Some([7; 32]), "the machine's, as it gives it");
}
```
Proof: world, `heavy cargo nextest run -p skein-echo-world -E
'test(=a_seed_replays_to_the_same_echo_digest_at_its_end)'`
(tests/echo's `scenarios.rs`): `assert_replays(11,
12, ..)` over `closes(seed, Config::chaos())`, the trace each entry's
`{:?}`, `T` its `state_digests()`: the echo's digest is `Some` and two
runs of a seed end with the same. Not `live`: the real loop does not
replay. Bound: 0.5 s.
Don't: change `assert_replays` or an existing trait method; call
`state_digest` from `drive` or step code; give skein-io's types `Hash`;
touch `real::World`.
Size: at most 80 lines.
Removes: nothing.

### 03.4 Digests in the trace after each iteration
Scope: testing.md, 5's `digests` option; simulator.md, 7.
Needs: 03.3's `Host::state_digest` and `Machine::state_digest`.
Files: `crates/skein-sim/src/trace.rs` (`Event`, :21-28), `src/sim.rs`
(`Pid` and its `Display`, :32-45; `spawn_process`, :204; beside `trace`,
:517); `testing/skein-world/src/world.rs` (shared: the field,
`with_machine` :210, `turn` :541-575, `run_inner`'s loop :398-409),
`lib.rs`. An arm for `Event::Digest(_)` in each exhaustive match:
`tests/oauth-accounts/tests/sign_in.rs:77`, `tests/io/src/census.rs:34,
94, 120`, `tests/sim/tests/fuzzy_files.rs:151`,
`tests/sim/tests/fuzzy_exchange.rs:16, 49`. Tests:
`tests/sim/tests/digests.rs` (new), `tests/world/tests/digests.rs`,
`tests/echo/tests/scenarios.rs`.
Shape: as the sketch. Under `Traced` a process's digest is written in its
turn after `iterate` and `drain`, before its submit; the machine's after
the iteration's turns (:409). `trace_digest` asserts its pid is a
process's or `Pid::MACHINE`; `spawn_process` a pid below `u32::MAX`.
Test shape: `tests/sim/tests/digests.rs` on `skein_sim_tests::World`
(`tests/sim/src/support.rs:16`), each under 20 ms; world tests as 03.3's.
Tests: `a_traced_digest_is_an_entry_at_the_present_time_under_its_pid`
(below); `a_machine_digest_renders_under_the_machine_not_a_pid`;
`untraced_a_world_s_trace_holds_no_digest`;
`traced_each_process_s_digest_comes_between_its_iterate_and_its_submit`
(each script's first entry a `Digest`, its second a `Submit`);
`traced_the_machine_s_digest_follows_each_iteration` (`Stamped`'s
entries number `outcome.iterations`);
`traced_two_runs_that_part_differ_first_at_a_digest` (`noting(1)`
against `noting(0)`); the Proof.
```rust
use skein_sim::{Event, Pid};
use skein_sim_tests::World;

#[test]
fn a_traced_digest_is_an_entry_at_the_present_time_under_its_pid() {
    let mut world = World::calm();
    let pid = world.spawn();
    world.sim.trace_digest(pid, [7; 32]);
    let last = world.sim.trace().last().expect("an entry");
    assert_eq!((last.at, last.pid, &last.event), (world.sim.now(), pid, &Event::Digest([7; 32])));
    assert_ne!(pid, Pid::MACHINE, "no process has the machine's pid");
}
```
Proof: world, `heavy cargo nextest run -p skein-echo-world -E
'test(=a_seed_replays_to_the_same_digests_after_every_iteration)'`
(tests/echo's `scenarios.rs`): `clients(1,
Config::calm())` under `digests(Digests::Traced)`, run twice: the traces
are equal and hold the echo's `Digest` entries (the clients and
`NoMachine` give none), more than one distinct digest among them as its
sessions come and go. Bound: 0.5 s.
Don't: add a field to `Entry` or change `Config`; write an entry for a
`None`; change an `Untraced` trace.
Size: at most 80 lines.
Removes: nothing.

## Interface sketches

lib owns the hasher and the `Hash`es; skein-shell `Host`; skein-world
`Machine`, the option and the digests; skein-sim the entry. Only a world
calls `state_digest`.

```rust
// crates/skein-lib/src/digest.rs (03.1; re-exported at lib's root)
/// A keyless `Hasher` over SHA-256, alike on every target (lib.md, 5.1).
#[derive(Clone, Debug)]
pub struct StateHasher { sha: crate::sha256::Sha256 }
impl StateHasher {
    #[expect(clippy::new_without_default, reason = "step code names no Default (programming-model.md, 10.2)")]
    #[must_use]
    pub const fn new() -> StateHasher;
    /// Ends it: the SHA-256 of every byte written, in order.
    #[must_use]
    pub fn digest(self) -> [u8; 32];
}
impl core::hash::Hasher for StateHasher {
    /// The digest's first 8 bytes, big-endian, without ending it.
    fn finish(&self) -> u64;
    fn write(&mut self, bytes: &[u8]);
    /// Little-endian at its width; likewise `write_u16` to `write_u128`, `write_i8` to `write_i128`.
    fn write_u8(&mut self, n: u8);
    /// 8 bytes, little-endian, on every target; likewise `write_isize`.
    fn write_usize(&mut self, n: usize);
}
/// `value`'s digest, hashed whole.
#[must_use]
pub fn state_digest<T: core::hash::Hash + ?Sized>(value: &T) -> [u8; 32];

// crates/skein-lib/src/{slab,map,...}.rs (03.2): `impl<T: Hash> Hash for Slab<T>`, likewise
// Map, Set, Queue, List, Stack, Deadlines, Intake, CryptoRng, Journal, RequestTable.
// crates/skein-shell/src/drive.rs and testing/skein-world/src/program.rs (03.3)
pub trait Host {
    /// A digest of what it holds, for replay (lib.md, 5.1): `None` unless it gives one; `drive` never calls it.
    fn state_digest(&self) -> Option<[u8; 32]> { None }
}
pub trait Machine {
    /// A digest of the fake's own state (testing.md, 5): `None` unless it gives one.
    fn state_digest(&self) -> Option<[u8; 32]> { None }
}
// testing/skein-fake-machine/src/fs.rs (03.3): the fake's inherent `Machine::state_digest(&self) -> [u8; 32]`, every
// field; tests/io's `FileSystem` implements `skein_world::Machine::state_digest` as `Some(..)` of it.

// testing/skein-world/src/world.rs (03.3, 03.4)
/// The digests a run ended with: a world's `T` for `assert_replays` (testing.md, 5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StateDigests {
    /// Each surviving process's, in admission order, as `Outcome::procs`.
    pub procs: Vec<Option<[u8; 32]>>,
    /// The fake machine's.
    pub machine: Option<[u8; 32]>,
}
impl<P: Host, M: Machine> Outcome<P, M> { #[must_use] pub fn state_digests(&self) -> StateDigests; }
/// The world's `digests` option (testing.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Digests {
    /// The trace holds the records alone: the default.
    Untraced,
    /// Each process's digest in its turn before its submit, the machine's after each iteration.
    Traced,
}
impl<P: Host, R: Referee<P>, M: Machine> World<P, R, M> {
    /// Sets the option, before `run`.
    pub fn digests(&mut self, digests: Digests);
}

// crates/skein-sim/src/trace.rs and sim.rs (03.4)
pub enum Event {
    // Submit, Complete and Fault as they are, then:
    /// A state's digest from the world harness: a process's under its pid, the machine's under `Pid::MACHINE`.
    Digest([u8; 32]),
}
impl Pid {
    /// The fake machine's, in the trace: no process's. `Display` prints `machine`.
    pub const MACHINE: Pid = Pid(u32::MAX);
}
impl Sim {
    /// Records `digest` under `pid` at the present time.
    pub fn trace_digest(&mut self, pid: Pid, digest: [u8; 32]);
}
```

## Questions to raise when reached

None.

## Done when

Each runs once, after 03.4 lands, reading `origin/main`.

1. This prints 4:
   `git -C ~/src/rust/skein log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'skein-03\.[1-4]'`
2. `git -C ~/src/rust/skein grep -n -E 'pub struct StateHasher|fn state_digest[(<]|Digest\(\[u8; 32\]\)|pub enum Digests|pub const MACHINE' origin/main -- crates testing`
   prints 9 lines, in `skein-lib/src/digest.rs` (2),
   `skein-shell/src/drive.rs`,
   `skein-world/src/{program,writer_pipe,world}.rs`,
   `skein-fake-machine/src/fs.rs` and `skein-sim/src/{trace,sim}.rs`.
3. Additive: `git -C ~/src/rust/skein diff 7e8fb70 origin/main --
   crates/skein-shell/src/drive.rs testing/skein-world/src/program.rs
   testing/skein-world/src/domain/trace.rs | grep -E '^-\s*(pub )?fn '`
   prints nothing.
4. In a worktree of skein at `origin/main`: `heavy cargo nextest run -p
   skein-lib -p skein-world-tests -p skein-sim-tests -p skein-fake-machine
   -p skein-io-world -p skein-echo-world -E 'test(/digest/)'` passes, its
   `Summary` naming 31 more tests than the same command at `f981746`,
   counted there once with `nextest list` in place of `run` (this
   session's 31, and one for each digest test of skein 04.1 or 05.1).
