# Session 28: the builder's worlds: domain, protocol and simulated; the fake machine behind the simulator; replay with digests

The builder's tiers above the real loop (testing.md, 2.2, 2.5, 2.6, 8.1 to
8.4); the joint layer opens here (code-organization.md, 5). 28.1: the fake
machine behind skein's simulator, with its digest. 28.2: the domain world and
io's file records made of the machine's operations. 28.3: the protocol world,
the tools' fakes at the io face, pipes cut at random; the failed `Hunks`. 28.4:
the simulated world, the shipped `Client` and hosted fakes with their time;
`diff --resolve`. 28.5: the builder's worst case summed, ending hearth 11's
`u64::MAX`; replay with state digests.

## Read first

- hearth's `testing.md`: 2.2, 2.5, 2.6, 3.1, 3.5, 4.1, 8.1 to 8.4; `render.md`:
  5.7, 8.2; `store.md`: 8.2, 11; `client/README.md`: 3.3.
- This plan: `code-organization.md`, 3 (last row), 5; `findings/skein-io.md`, 5;
  `findings/skein-testing.md`, 1 to 3, 5; the sessions `hearth/11`
  (Choices, sketches), `23` (23.1), `33` (33.3), `24`, `45` (45.1, 45.2),
  `26`, `44`, `27`, `46` (sketches), `skein/03`, `skein/05` (05.1).
- skein: `testing/skein-world/src/{program,world,stream}.rs`, `domain/trace.rs`,
  `testing/skein-fake-machine/src/face.rs:19`; `programming-model.md`: 2, 6.3,
  9; `testing-strategy.md`: 2.5, 2.7, 6, 7. temper: `tests/one-process/src/world.rs:613`,
  `testing/temper-fake-git/src/program.rs:13-62, :355` (a hosted tool),
  `tests/worker/protocol-git/`. jig: `tests/people/tests/simulation.rs:393`.

## What the design asks

- **Worlds** (testing.md, 2.2, 2.5, 2.6, 3.1, 4.1): "A restart is cold"; "a
  tool's output cut mid-line, a pipe that fills, a tool killed mid-write";
  each fake "takes the time its work takes, on the world's clock".
- **Checks** (8.4): memory against the worst case; "a seed replays to the same
  trace, the same fake machine ... and the same digest of hearth's own state".
  store.md, 11: "no staging, trash or temporary file remains". render.md,
  5.7: `diff --resolve` records "each closure before they print".

## Choices this plan makes

- **The face behind the simulator is `behind`, in `hearth-fake-machine`**
  (3.1 admits skein's io and simulator there alone; not in `linux`, 3.11). It
  answers the seam's `Ask` with 26's `Op`s as one logged-in process, the
  builder (testing.md 3.1: per-process counts are [M3]). `Shared`, a newtype on `Rc<RefCell<Fake>>`, is the
  worlds' one owner of the machine and the repositories, read by the referee
  and the hosted fakes (27.2's `reach` is `&self`).
- **Tools twice.** The protocol world spawns them with `Op::Spawn` and steps
  44.2's `Git` and 46.1's `Mmdebstrap`; the simulated world hosts them
  (`HostedProgram`), wrapping the same pure faces (git's `face::{check, Run}`, mmdebstrap's
  `face::{check, print}` over its `Simulate`), over the
  repositories and served mirror that `Shared` lends. The domain and protocol
  worlds play io with `Files`; the simulated one runs the real `FileIo`.
- **Faults** are the seed's: a tool killed after n stdout bytes, a cache file
  gone before git reads it, git not installed, a stalled mmdebstrap (never git:
  no deadline in M1, render.md 2.2; fakes do not end their clients,
  testing-strategy.md, 6). A sweep asserts each fell (testing.md, 4.1).
- **The digest is the domain's,** as the echo's shell gives; `Held` and `Env`
  are not `Hash` (lib.md 5.1; testing.md 8.4); a process's digest is taken
  after its iterate and drain, before its submit, the machine's once every
  process has had its turn (skein testing.md 5). Hosted fakes give `None`.
- **Limits are `hearth::limits()`,** times as they ship, in simulated time
  (testing-strategy.md, 6). Named exceptions: a 64-byte pipe, and the memory
  test's `small_limits()`: the shipped limits but git's `entries` 64,
  `path` 256 and `paths_total` 16,384 (not 02.3's `SMALL`: 8, 32 and
  256); render's, tree's and store's stay as shipped.

## Rules that bind it

- **No expected value computed by the code under test:** text and digests are
  hearth 32.1's and 25's literals; replay compares two runs. From here a protocol
  increment proves at the joint layer (code-organization.md, 5).
- **Shared files:** `tests/builder/{Cargo.toml, src/fixtures.rs}`,
  `crates/hearth/src/lib.rs`, the service's, protocol's and root's `lib.rs`,
  render's `limits.rs`, `testing/hearth-fake-machine/src/` (26, 27, 46.1).

## Increments

### 28.1 The fake machine behind the simulator, and its digest
Scope: `impl skein_world::Machine` over 26's `Op`s (testing.md, 3.1);
`Hash` on the machine and its `state_digest` (skein 03.3).
Needs: hearth 27; hearth 46.1; skein 03; skein 05.1 (`Ask::Lock`, `Error::Busy`): `git -C
~/src/rust/skein grep -n 'Lock { file:' origin/main -- crates/skein-sim` prints a line.
Files: `testing/hearth-fake-machine/`: `Cargo.toml` (`skein-io`, `skein-sim`,
`skein-world`), new `src/behind.rs`, `src/lib.rs`, derive lines in
`src/{linux,hearth}.rs`, `src/linux/*.rs` where a type lacks `Hash`,
`src/hearth/{universe,mirror}.rs` (shared with 26, 27); tests new
`src/tests/behind.rs`, `src/tests.rs`, `src/tests/fixtures.rs` (`behind()`: the
builder logged in, `/srv/store` laid 0700 and the builder's, `.owner(BUILDER)`,
`linux.rs:70`; `directory(behind, path) -> Handle`); shared (ruling 132): the
root `Cargo.toml` (`skein-sim`, beside `skein-io` and `skein-world`) and
`Cargo.lock` (skein repinned to `main`, at least 841f487: the pinned 78dd8fa
has no `Machine::state_digest`, `StateDigests` or 03.4's `Digests::Traced`).
Shape: the sketch. `answer` is `step` less the ticket, so a unit test needs
no `Sim`. Seam handles are minted by `Behind`. `Open` is `Op::Open { at:
At::Beneath(root), .. }`, `OpenHow` to `How` variant for variant; `Read`,
`Write`, `Truncate`, `Sync`, `Stat`, `Rename`, `Remove`, `MakeDirectory`,
`Close`, `Lock` the same-named `Op`s; `Append` is `Stat` then `Write` at the
size; `List` is `Op::List { most: 1 }` per entry while 255 bytes of `room`
remain (an entry costs its name's length and 1), one at least while any is
left. `Refusal` to `kernel::Error` is a total match: the same names, but
`NotPermitted` is `Other(1)`, `Loop` `Other(40)`, `Invalid` `InvalidArgument`,
`Busy` skein 05.1's `Busy`. `Spawn` opens `dir` beneath
`root` and `program` `At::Root`: missing or denied as refused, else
`Program::Exit(127)` (3.5); hosted programs never reach the machine.
`cut(Kill)` closes the handles it is given, so a lock goes; `PowerLoss`
fails an `assert!` naming "M2" (the workspace denies `clippy::panic`). `Hash` is
derived on the parts, `hearth::Machine` included, and written by hand only for
a machine that holds the ledger: nodes, contents, handles, processes, pipes, rng, sizes, the served
mirror, never the ledger the referee drains.
Test shape: `src/tests/behind.rs`, each under 20 ms:
`a_file_made_through_the_seam_reads_back_and_stats_as_laid`,
`every_refusal_maps_to_the_kernel_error_of_its_name` (a `const` table),
`a_listing_resumes_where_it_stopped_and_keeps_to_the_room`,
`a_spawn_of_an_unclaimed_file_exits_127`, `a_spawn_of_a_missing_file_is_not_found`,
`a_write_changes_the_digest_of_machines_laid_alike`,
`the_ledger_does_not_move_the_digest`,
`a_lock_is_busy_through_a_second_handle_until_the_first_closes`,
`a_kill_cut_releases_the_lock_it_held`, `a_power_loss_cut_asserts_m2`
(`#[should_panic(expected = "M2")]`).
```rust
// testing/hearth-fake-machine/src/tests/behind.rs
#[test]
fn a_lock_is_busy_through_a_second_handle_until_the_first_closes() {
    let mut behind = fixtures::behind();
    let first = fixtures::directory(&mut behind, b"/srv/store");
    let second = fixtures::directory(&mut behind, b"/srv/store");
    assert_eq!(behind::answer(&mut behind, Ask::Lock { file: first }), Ok(Reply::Done), "kernel.md, 6.1: taken");
    assert_eq!(behind::answer(&mut behind, Ask::Lock { file: second }), Err(Error::Busy), "another open holds it");
    assert_eq!(behind::answer(&mut behind, Ask::Close { file: first }), Ok(Reply::Done), "closed");
    assert_eq!(behind::answer(&mut behind, Ask::Lock { file: second }), Ok(Reply::Done), "it went with its handle");
}
```
Proof: unit, that test: skein 05.1's lock through the seam, on 26.1's `Op::Lock`.
Nothing runs `Behind` in a world until 28.4. Bound: 0.02 s.
Don't: name a hearth crate or a tool in `behind`; model a power cut.
Size: at most 260 lines.
Removes: nothing.

### 28.2 The builder's domain world, and io's file records
Scope: testing.md, 2.2: the root with render and store, the world playing the
protocol layer at the fakes' domain faces; the store's requests made of
operations (`Files`); a cold restart; the referee's invariants.
Needs: hearth 24 and 31 (render, store and resolution through the root).
Files: `tests/builder/`: `Cargo.toml` (the fakes, the three domains,
`hearth-protocol-{config,tree}`, `skein-world`), new `src/{files,domain,referee}.rs`,
`src/{lib,fixtures}.rs` (shared: `Fake`, `Shared`, `Scenario`, `thin()`), new
`tests/domain.rs`.
Shape: `Files::answer` is a total match over `file::Request`; a record the layer
never asks asserts, naming this session. Each record is operations: `Load` is
Open, Stat (past `max` it fails), Read, Close; `Store` writes
`<path>.skein-<16 hex>` (the seed), syncs, tests `Expect` (`Conflict { now }`),
renames, syncs the directory; `Lock` is `Op::Lock`, `Locked` or `Busy`; the rest
as io.md, 5 answers them. `domain::Run` steps the root from a `Schedule` and
`Stage`s, answering each `Request` after `Span::millis(1, 20)`: `List` and `Read`
from `Repository::{resolve, list, object}` (documents decoded by
`hearth_protocol_config` and `translate::documents`), `Cached` and `Keep` by
`Files` and hearth 09's codecs, `Resolve` from `resolve::closure` on the drawn
universe with `Inst`, `Stanza` and `Index` events (24.3, 31.2), the store's
requests by `Files`. `cut_at(Cut)` leaves a request unanswered; `restart()`
makes root, render and store anew over the same machine. The referee holds one
terminal a request (`Ledger`), one lock holder at most, and, settled, no `.tmp`
or `.skein-` name, no `staging/` entry, a file for each record; in 28.3, 28.4
and after a `Kill` cut also testing.md 8.4's rest: every child reaped, every
pipe read to its end, no handle open.
Test shape: `tests/domain.rs`, each under 0.5 s, eight seeds:
`a_cold_restart_settles_what_a_cut_left_behind`,
`a_thin_repository_renders_through_the_three_domains` (the `spec` line's digest
in `fixtures::THIN_TEXT` as it stands), `a_busy_lock_is_asked_again_every_200_ms_until_released`,
`a_file_entering_twice_is_renamed_once`,
`a_resolved_key_is_recorded_and_the_next_call_finds_it_known`,
`every_request_gets_one_terminal_whatever_the_latencies`,
`resolution_keys_equal_a_naive_transitive_closure_of_the_fake_universe`
(testing.md 8.4: a closure over `Depends` and `Pre-Depends` computed in the
test, taking each group's first alternative and, for a name only provided, the
provider 27.3 ranks first, against the resolver's keys).
```rust
// tests/builder/tests/domain.rs
#[test]
fn a_cold_restart_settles_what_a_cut_left_behind() {
    for seed in 0..8 {
        let mut run = domain::Run::new(&fixtures::thin(), seed);
        run.cut_at(domain::Cut::BeforeKeep); // the first arrival's temporary synced, its rename never asked
        let left = run.finish();
        assert!(left.strays() > 0, "seed {seed}: the cut left a temporary in cache/");
        let after = left.restart().finish();
        assert_eq!(after.strays(), 0, "store.md, 11: no temporary, no staging, a file for each record");
        assert!(matches!(after.answers[..], [Answer::Spec(_)]) && after.outcome == Some(Outcome::Done), "answered");
    }
}
```
Proof: world, that test: a builder cut and started cold, the store settled and
the call answered. Not the highest layer (process, 23.1): the real loop cannot
cut a builder mid-write. Bound: 0.5 s.
Don't: let the domain see a tool's text; give `Files` a record the layer never asks.
Size: no counted lines (`tests/`); the world about 300.
Removes: nothing.

### 28.3 The protocol world: the tools at the io face, bytes cut at random
Scope: testing.md, 2.5: the real protocol layer, machines and domains, io played
by the world over the machine, the tools spawned with `Op::Spawn`; `Lock` on
`Op::Lock`; the failed `Hunks` (hearth 45.1's test, 44.2's missing side).
Needs: hearth 45.1 (the `tool` error of a failed `Hunks`); hearth 44.2.
Files: `tests/builder/`: new `src/{io,protocol}.rs`, `src/fixtures.rs` (shared:
`MOTD` pair, `diff_motd(Form) -> Scenario` (a diff of the pair), `Fault`,
`MOTD_KIND_SHA256` = `a603a55b…e8b2`, `sha256sum` of `Welcome, and be kind.\n`), new `tests/protocol.rs`, `tests/fuzzy_protocol.rs`.
Shape: `io::Below` takes `Asked`, gives `Told`. `Io(Spawn)`: `Op::Spawn` as the
builder, then `machine.tool(pid)` starts `Git` or `Mmdebstrap` (46.1), stepped
each turn; stdout and stderr by `Op::Take` with `most` drawn 1 to 30 (`Wire`'s
grain), as `Up::Bytes` after the layer's `Demand`; `Down::Send` to stdin by
`Op::Send` (what fits, then `Room`); `Finish` closes it; `Wait` gives `Exited`;
`Close` is `Op::Signal { signal: Kill, to: Group, .. }`. `File(request)` goes to `Files`.
`protocol::run` is `service::iterate`'s two passes with `Below` in io's place,
`next_deadline` and `fire` driven from a `Schedule`. `Fault::KilledAt` kills the
child after n stdout bytes; `CacheFileGone` removes `cache/<hex>` as it is renamed in;
`NotInstalled` removes or unmarks the tool's file. A failed `Hunks` is the
change bare and the target's `tool` error, 75 (render.md 2.2). Render's world
is its domain under a scripted parent (render.md 9.1), not this one.
Test shape: `tests/protocol.rs`, each under 0.5 s, eight seeds:
`render_prints_thin_whatever_the_cuts_of_every_pipe`, `a_full_pipe_holds_its_writer_until_read`
(64 bytes, 60 blobs), `a_tool_killed_mid_output_fails_the_call_with_75`,
`a_busy_lock_waits_and_says_so_once` (held by a second process until 2.5 s),
`a_resolve_over_cut_pipes_records_its_closure` (fake mmdebstrap, 31.2's index
renames), `a_tool_not_installed_fails_the_call_with_75` (`/usr/bin/git` absent,
then mode 0644: render.md 7.2 "could not start"),
`a_settled_run_leaves_no_child_unreaped_pipe_unread_or_handle_open`,
`a_diff_whose_git_fails_prints_the_change_bare_then_its_tool_error_and_exits_75`; fuzzy `fuzzy_protocol.rs`:
`random_cuts_kills_and_vanished_files_settle_and_each_fault_falls` (64 seeds, 4 s).
```rust
// tests/builder/tests/protocol.rs
#[test]
fn a_diff_whose_git_fails_prints_the_change_bare_then_its_tool_error_and_exits_75() {
    for seed in 0..8 {
        let scenario = Scenario { fault: Some(Fault::CacheFileGone { sha256: fixtures::MOTD_KIND_SHA256 }), ..fixtures::diff_motd(Form::JsonLines) };
        let run = protocol::run(&scenario, seed);
        let lines = fixtures::lines(&run.stdout);
        assert_eq!(lines.len(), 5, "version, two changes, the error, the outcome: seed {seed}");
        assert!(!lines[1].contains(r#""hunks""#), "git printed nothing (44.2: a missing side exits 1): the change is bare");
        assert!(lines[3].starts_with(r#"{"record":"error","kind":"tool""#), "45.1: the error follows the changes");
        assert_eq!(run.code, Some(75), "client/README.md, 3.3: operational");
    }
}
```
Proof: joint, `render_prints_thin_whatever_the_cuts_of_every_pipe`: the real
protocol layer, machines, domains and printer, through fake git on cut pipes,
print `fixtures::THIN_TEXT` (hearth 32.1's) and exit 0. Bound: 0.5 s. Not the
highest layer (process, 11.3): cut pipes and faults are this world's.
Don't: compute a tool's text in the world; stall git; let a fake end its client.
Size: no counted lines (`tests/`); the world about 300.
Removes: nothing.

### 28.4 The simulated world: the shipped `Client`, hosted fakes, their time
Scope: testing.md, 2.6: `Client` as a process on skein's simulator over `Behind`;
fake git and mmdebstrap as hosted `Host`s that take their time; `diff
--resolve`; the files compared with the protocol world's.
Needs: hearth 45.2; 28.1 and 28.3; skein-03.s1 (ruling 177), by a repin commit (Rules, "Upstream first").
Files: `crates/hearth/src/lib.rs` (shared: `Client::with_roots`); `tests/builder/`:
new `src/{simulated,pace,hosted}.rs`, `src/hosted/{git,mmdebstrap}.rs`,
`src/fixtures.rs` (shared: `Scenario::pace`, `FAKE_WORST_CASE`), new `tests/simulation.rs`.
Shape: `Client::with_roots` (a test constructor: a seed and the roots, 23.1's
`Roots`) is 11.3's `open` after the shell made and opened the
directories (`open` calls it with its own seed). `simulated::run`: `World::new_controlled`,
`with_machine(Shared)`, the client by `spawn_restartable_startup` (skein-03.s1) over startup roots
`store`, `cache`, `resolutions`, `staging` and an output
`StartupAppend`, the tools by `host_roots` at `/usr/bin/git` and
`/usr/bin/mmdebstrap` (roots `/` and the store as `cwd`; instances 1), the
scenario reaching the plain-`fn` factories through a thread-local, as temper's
`world.rs:613`. A factory asserts its program installed and executable. `Proc`
is an enum of the three, `Host` by delegation. A hosted fake checks its vector,
reads by kernel ops (the git-dir, the diff's two files, `sources.list`), runs the
pure `Run` or `print`, writes apt's lists, and sends its output as the pipes have
room after `now + pace` (`next_deadline`, costs drawn from the seed); its
`worst_case` is `fixtures::FAKE_WORST_CASE` (`1 << 20`). `Fault::Stalled` is
46.1's `stall_after`. `Shared` counts the bytes the served mirror gave
(`served_bytes()`); a spawn-to-exit span is from the trace's spawn event to the
program's exit.
Test shape: `tests/simulation.rs`, each under 0.5 s, `Config::calm()`:
`diff_resolve_resolves_each_changed_key_once_in_turn_before_printing` (45.2's
repository at two requests, one target's key changed, the other's not: the
referee sees two mmdebstrap spawns, one alive at a time; at each the client's
output is empty and the earlier runs' records exist; at the end exactly the
changed keys' records, `known`),
`each_resolved_costs_equal_the_bytes_served_and_the_spawn_to_exit_span`
(`Costs { fetched, took }`, testing.md 9.3),
`git_and_mmdebstrap_take_the_time_their_work_takes` (settle time at least the
sum of the spawned fakes' `Pace` minimums and at most their maximums), `a_stalled_mmdebstrap_meets_its_idle_timer` (60 s simulated,
exit 75), `two_builders_over_one_store_take_the_lock_in_turn` (the
second's `Lock` refused `Busy` in the ledger, both print `THIN_TEXT`),
`a_lock_held_past_the_resolve_limit_is_waited_out_then_the_call_renders` (held
700 s, past render's 600 s; store.md 8.2 sets no limit: says busy once),
`the_protocol_and_simulated_worlds_leave_the_same_store` (cache names and
payloads equal, and each decoded record's `origin`, `sha256` and `size`;
`at` and `took` are each world's clock, left out, and checked by the costs
and pace stories; no temporary; ruling 182).
```rust
// tests/builder/tests/simulation.rs
#[test]
fn hearth_render_through_the_simulator_prints_the_thin_spec() {
    let outcome = simulated::run(&fixtures::thin(), 7, Config::calm());
    assert_eq!(outcome.machine.contents(fixtures::OUTPUT).as_deref(), Some(fixtures::THIN_TEXT.as_bytes()), "the shipped Client, io and printer");
    assert_eq!(outcome.procs[0].exit(), Some(Exit::Code(0)), "client/README.md, 3.3: done");
}
```
Proof: joint, that test: the shipped `Client`, the real io layers and printer on
the simulator, `Behind` and fake git hosted print `THIN_TEXT` and exit 0, as the
real loop does. Bound: 0.5 s. Not the highest layer (process, 11.3): the
simulator is the subject.
Don't: end a hosted fake early; read the machine from a `Host` but through the
seam, bar the repositories and mirror; shorten a shipped time.
Size: at most 40 lines (`Client::with_roots`; the world is not counted).
Removes: nothing.

### 28.5 Replay with state digests, memory against the worst case, the sweeps
Scope: testing.md, 8.4's Replay and Memory; the builder's `worst_case` summed
(programming-model.md, 6.3); the sweeps.
Needs: skein 03 (03.4: `Digests::Traced`, `Outcome::state_digests`); 28.4;
hearth 39.1.
Files (shared): `crates/hearth-domain-render/src/{lib,limits,call}.rs`,
`hearth-domain-store/src/lib.rs`, `hearth-domain-builder/src/lib.rs`,
`hearth-protocol-builder/src/{lib,limits}.rs`, `hearth-service-builder/src/lib.rs`,
`crates/hearth/src/lib.rs`; `tests/builder/`: `src/simulated.rs`, new
`tests/{memory,fuzzy_simulation}.rs`, `tests/simulation.rs`.
Shape: `Hash` derived on the domains' state (`Render`, `lib.rs:17`, and its
`Call`, `call.rs:11`, whose `read` and `next` become `u32`; `Store`,
the root) and the private types under them that lack it: no `usize` (explicit
widths; log 7), no `Env` (log 8). `Service::domain()`; `Client`'s `state_digest`
is `Some(state_digest(domain))`. `worst_case` at each layer, as the echo's
`service::worst_case` sums: render's (render.md, 8.4, with `evaluate::worst_case`
and `evaluate::template::worst_case`), the root's with render's and store's, the
protocol layer's (each machine once: one git child, one decode, one resolution
at a time, hearth 11 and 24, and tree's `limits::{worst_case, held}`, 39.1), the service's (io, `FileIo::worst_case`, protocol,
domain, queues); `Client::worst_case` is the service's. `simulated::chaos()`
(ruling 186): `Config::chaos()` with `faults` that keep `Faults::CHAOS`'s latency,
short reads, writes, receives and sends, cancel races and late resets, and set
to 0 the faults that end an operation for good: `no_space`, `read_only`,
`io_error`, `hung` (a hung git has no deadline in M1, Choices), `no_buffer`,
`reset`, `refuse`, `timed_out`; the replay and the fuzzy story run under it.
Test shape: `tests/simulation.rs`, 0.5 s: ref below,
`a_digested_run_traces_the_domain_and_machine_digests` (more than one distinct
`Event::Digest` under the client's pid, the machine's under `Pid::MACHINE`, none
for hosted fakes). Unit, `hearth-service-builder`: `a_larger_limit_gives_a_larger_worst_case`,
`the_shipped_limits_have_a_worst_case_within_1_gib` (render.md 8.4: the sum at
the defaults, `Client::worst_case`, at most `1 << 30`; it waits for the
designer's answer on 8.4's bound, and 28.5 lands without it: ruling 184). `tests/memory.rs` (a
binary of its own, `Counting`): `the_builder_holds_no_more_than_its_worst_case_at_every_iteration`
(`Memory::Checked`, `small_limits()`; the store lists more than one 1,024
page, asserted from the ledger's `List` count, store.md 8.2). Fuzzy, `fuzzy_simulation.rs`:
`random_seeds_under_chaos_settle_with_the_calm_runs_spec` (64 seeds, 4 s),
`a_builder_killed_before_each_submission_restarts_cold_and_settles`
(`cut(0, n, Cut::Kill)`, n in 0..40, asserting 40 passes the run's last
submission: the output ends with `THIN_TEXT`).
```rust
// tests/builder/tests/simulation.rs
#[test]
fn a_seed_replays_to_the_same_trace_machine_and_state_digests() {
    let run = |seed| {
        let mut world = simulated::world(&fixtures::thin(), seed, simulated::chaos());
        world.digests(Digests::Traced);
        let outcome = world.run();
        (outcome.trace.iter().map(|entry| format!("{entry:?}")).collect(), outcome.state_digests())
    };
    let trace = assert_replays(11, 12, run);
    assert!(trace.iter().any(|line| line.contains("Digest")), "digests were traced");
}
```
Proof: joint, that test: two runs of seed 11 under chaos end with the same trace,
machine digest and domain digest; seed 12's trace differs. Bound: 0.5 s. The real
loop does not replay (testing.md, 2): this is the highest layer that does.
Don't: digest the protocol layer or an `Env`; raise the memory test's limits;
run the replay or the fuzzy story under the raw `Config::chaos()`.
Size: at most 200 lines.
Removes: hearth 11's `Client::worst_case` of `u64::MAX`.

## Interface sketches

The worlds are ordinary Rust; `behind` is held to the workspace lints, and
`Duration` is `core::time::Duration` (`std_instead_of_core`).

```rust
// testing/hearth-fake-machine/src/behind.rs (28.1)
/// The machine as the simulator's seam sees it: one logged-in process, a handle table.
pub struct Behind { /* hearth::Machine, the builder's Pid, seam handle to linux::Handle */ }
impl Behind { pub fn new(machine: crate::hearth::Machine, ids: crate::linux::Ids) -> Behind; pub fn machine(&self) -> &crate::hearth::Machine; }
/// One `Ask` answered, as `step` answers it less the ticket.
pub fn answer(behind: &mut Behind, ask: skein_sim::Ask) -> Result<skein_sim::Reply, skein_io::kernel::Error>;
impl skein_world::Machine for Behind { /* step, open_root, close_root, open_append, close_append, cut, state_digest */ }
// tests/builder/src/fixtures.rs (28.2 to 28.4)
/// The fake machine and its repositories.
pub struct Fake { pub behind: Behind, pub repositories: Repositories }
/// The worlds' one owner of it; `contents(path)` reads a file; implements `skein_world::Machine`.
pub struct Shared(Rc<RefCell<Fake>>);
/// What a run is told: the repository's files, the call, its form, a fault; 28.4 adds `pace: Pace` (`Pace::calm()` in `thin()`).
pub struct Scenario<'a> { pub files: &'a [File], pub call: Call, pub form: Form, pub fault: Option<Fault> }
/// A fault from the seed (testing.md, 4.1); a sweep asserts each fell.
pub enum Fault { KilledAt { stdout_bytes: u32 }, CacheFileGone { sha256: Digest }, NotInstalled { absent: bool }, Stalled { after: u64 } }
/// "Each takes the time its work takes" (4.1), in simulated time; `calm()`: git `Span::millis(1, 5)` and 2 ms a MiB, mmdebstrap `Span::millis(20, 80)` and 1 ms a package.
pub struct Pace { pub git: Span, pub per_mib: Duration, pub mmdebstrap: Span, pub per_package: Duration }
// tests/builder/src/{files,domain,protocol,simulated}.rs
impl Files { pub fn new(shared: Shared, fault: Option<Fault>, seed: u64) -> Files; pub fn answer(&mut self, request: file::Request) -> file::Event; }
/// domain world: what the root answered, and the call's outcome.
pub struct Run { pub answers: Vec<builder::Answer>, pub outcome: Option<render::Outcome> /* .. */ }
impl Run { pub fn new(scenario: &Scenario<'_>, seed: u64) -> Run; pub fn cut_at(&mut self, cut: Cut); pub fn finish(self) -> Run; pub fn restart(self) -> Run; pub fn strays(&self) -> usize; }
impl Run { pub fn requests(&self) -> &[builder::Request]; /* every request asked, in order (hearth 47.1) */ }
/// protocol world (`protocol::run`): stdout, stderr, the code, the vectors, the machine.
pub fn run(scenario: &Scenario<'_>, seed: u64) -> protocol::Run;
/// simulated world (`simulated::`): `world` for the options, then `run`; `Proc` is `Client`, `Git` or `Mmdebstrap`.
pub fn world(scenario: &Scenario<'_>, seed: u64, config: Config) -> World<Proc, Referee, Shared>;
pub fn run(scenario: &Scenario<'_>, seed: u64, config: Config) -> Outcome<Proc, Shared>;
// crates/hearth/src/lib.rs (28.4); `worst_case(&Limits) -> Option<u64>` joins render, builder, protocol-builder, service-builder (28.5)
impl Client { pub fn with_roots(configuration: Configuration, roots: Roots, call: Call, form: Form, output: Fd, seed: u64) -> Client; }
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-28\.[1-5]'` prints 5.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run -p
   hearth-fake-machine -p hearth-builder-world -p hearth-service-builder -p
   hearth` passes, and `heavy cargo nextest run -p hearth-builder-world
   --profile fuzzy -E 'binary(/^fuzzy_(protocol|simulation)$/)'` passes, its
   `Summary` naming 3 tests.
3. `git -C ~/src/rust/hearth grep -n -A3 'fn worst_case' origin/main -- crates/hearth/src | grep -c 'u64::MAX'` prints 0; `git -C ~/src/rust/hearth grep -n 'impl skein_world::Machine for Behind' origin/main -- testing/hearth-fake-machine/src` prints one line.
4. `heavy cargo nextest run -p hearth-builder-world -E 'test(=diff_resolve_resolves_each_changed_key_once_in_turn_before_printing) | test(=a_diff_whose_git_fails_prints_the_change_bare_then_its_tool_error_and_exits_75)'`
   passes, its `Summary` naming 2 tests.
