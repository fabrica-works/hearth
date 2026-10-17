# Session 05: skein's io for the store: the kernel's lock on a directory, and io's

hearth's store without layers needs of skein's io a directory's lock
(hearth's README.md, 8.1; the second round's K4, designed in skein
`d21d90d`); a rename across directories and a listing in pages are skein
04's. 05.1 puts the kernel's `Lock` (`flock`, `LOCK_EX | LOCK_NB`) in the
records, the ring, the simulator's seam and the minimal machine, held to
the real kernel by conformance; 05.2 gives io's owners `Lock`, answered
`Locked` or `Busy`, never waiting: hearth asks again on its own timer
(store.md, 8.2, step 1). hearth 23 waits for this session and skein 04.

## Read first

- What the design changed: `git -C ~/src/rust/skein show d21d90d`. At
  that commit: io.md 5 (the record block, "Locking", "One request at a
  time", "Memory"), 8 ("Files"), 9; kernel.md 3, 6 ("Some operations are
  synchronous"), 6.1 (on `Lock`, cancelling), 7, 8 ("Files beneath a
  root"); simulator.md 3.1, 5, 6, 7; testing.md 4; shell.md 3
  ("Synchronous operations run at submit").
- The caller, hearth at `175e92f`: store.md 8.2 (step 1); README.md 8.1's
  M1 row on a lock.
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

- **The kernel's lock** (kernel.md, 6.1, 7): "`flock` with `LOCK_EX |
  LOCK_NB` on a descriptor `Open` made of a file or a directory ..., never
  a path-only one", completing "with `Nothing` when the lock is taken, or
  already held through the same open; or ... `Busy` (`EWOULDBLOCK`)"; it
  "runs at submit and completes at the next reap, as `List` does"; its
  cancel, and "a lock on a socket, a pipe or a path-only descriptor", are
  broken invariants.
- **io's lock** (io.md, 5): "`Locked` when the lock is taken or already
  held through this root, or `Busy` when another open of the directory
  holds it ...; `Failed` with the kernel's error otherwise. io never waits
  for a lock and keeps no timer for one ... There is no unlock. A spawned
  child never holds it" (kernel.md, 6.1: "every descriptor being
  close-on-exec").
- **The machine's** (simulator.md, 3.1): "`Busy` while another handle of
  the same file holds it. A handle's lock goes when the handle does: at
  its `Close`, and when a process ends, killed or cut ... A power loss
  frees every lock."
- **Tests** (kernel.md, 8): "a directory locked through one open, `Busy`
  through a second open of it, locked again through the first, and taken
  through the second once the first is closed", on both backends; io.md
  9: "each with io's step tests and a world of files that meets it";
  simulator.md, 6: "each broken invariant of files" and "each fault of
  files".

## Choices this plan makes

- **Order:** 05.1 the kernel's `Lock`; 05.2 io's on it. Neither changes a
  variant at the base.
- **Tokens as io takes them today:** an unknown token is `Failed {
  NotFound }`, as `Stat`'s (`file_layer.rs:515`); io records no token's
  kind, so `Lock` takes any token, as kernel.md 6.1's takes "a file or a
  directory".
- **The machine keeps a lock on its handle** (`Handle::locked`): `close`,
  a power loss's `handles.clear()` (`fs.rs:472`) and a world's
  `Machine::cut`, which closes a cut process's handles
  (`tests/io/src/cuts.rs:272`), free it. Its refusal `Refusal::Held` is
  the face's `Error::Busy`. `locked` derives `Hash`, for skein 03.3's
  `state_digest`, whichever lands first.
- **Where code goes:** `kernel.rs`, `ring.rs`, `sim.rs`, `file_layer.rs`,
  the machine's `fs.rs` and conformance's `files.rs` are past 800 lines:
  they take variants and arms only. New:
  `testing/skein-fake-machine/src/fs/locks.rs`,
  `testing/skein-conformance/src/locks.rs`, `tests/io/src/locks.rs`; 05.2's
  step tests go in skein 04.1's `crates/skein-io/src/tests/directory.rs`
  and its world's tests in 04.1's binary `tests/io/tests/directory.rs`;
  its admission and answers in skein 04.2's
  `crates/skein-io/src/file_layer/directory.rs`; one real-ring test in
  `tests/io/tests/ring.rs`.
- **Limits:** skein ships no file profile: the world takes small ones of
  its own, as `tests/io/src/files.rs:19-21` (4 files, 4 entries).

## Rules that bind it

- **skein 03 and this session meet in the fake machine's digest.** At
  its rebase before landing, 05.1 runs `rg -n 'fn state_digest'
  testing/skein-fake-machine/src`: a match, it adds `Handle::locked` to
  that digest, with a test that a held lock changes it; none, skein 03.3
  digests it with the rest.
- **The gate** (README, Rules; decisions.md, 1): each increment lands with `flock
  ~/src/rust/tmp/hearth/render/land-skein.lock land --gate '<the README's
  command, whole>'`, its commit body saying so in one line; no probe of
  the machine. No `skein-ui-web` code changes.
- **Test time** (README, Rules; `findings/measurements.md`): focused
  6.46 s and fuzzy 41.84 s at the base; this session's allotment is 0.4 s
  and 2 s (skein 02: 0.4 s / 1 s; 03: 0.3 s / 0.5 s; 04: 0.4 s / 1.5 s;
  05: 0.4 s / 2 s). A unit test under 20 ms, a focused world or ring
  test under 0.5 s, a fuzzy sweep about 5 s; a test the gate flags `SLOW`
  is made cheaper (fewer seeds or cut points) before landing. No budget,
  period or ceiling is raised; no focused test moves to the fuzzy suite.
- **Users in skein change in the increment, none downstream:** Files
  name skein's users of what each Shape changes; smith's, jig's and
  temper's change at their closing repins (README, Repin). Shared with
  skein 03 and 04: `testing/skein-world/src/`,
  `testing/skein-fake-machine/src/fs.rs` and
  `crates/skein-sim/src/trace.rs`: landing second, rebase and keep both.

## Increments

### 05.1 The kernel's `Lock`
Scope: `Op::Lock` and `Error::Busy` (kernel.md, 3, 6.1, 7); the ring's
`flock` at submit (shell.md, 3); the simulator's seam (simulator.md, 3.1,
7); the minimal machine's locks (testing.md, 4); kernel.md 8's scenario
on both backends.
Needs: nothing.
Files: `crates/skein-io/src/kernel.rs` (`Op` :361, `Error` :781, `Files`
:903, `is_valid` :972, `shape` :1035, `files` :1069, `fails_on_files`
:1286 and the other error tables, `fits`, `binds`, `counts`; the module
documentation, :95-252); `crates/skein-shell/src/ring.rs` (`prepare`
:741 beside `List` :842, `error` :986, `file_error` :1111,
`check_path_access` :1227, the synchronous calls :46-52 and :1546);
`crates/skein-sim/src/machine.rs` (`Ask` :57), `src/sim.rs` (three `Op`
matches), `src/sim/files.rs` (`file` :100, `failure` :173, `ask` :233,
`answered` :309), `src/sim/access.rs:41`, `src/trace.rs` (shared;
`Summary` :54, `cancellable` :308, `fds` :345);
`testing/skein-fake-machine/src/fs.rs` (shared; `Refusal` :116, `Handle`
:241), new `src/fs/locks.rs`, `src/face.rs` (`step` :19, `kernel_error`
:169); `testing/skein-conformance/src/run.rs` (`lock` beside `sync` :610;
its `Op` matches), new `src/locks.rs`, `src/lib.rs`, `src/processes.rs`
(`command` :107 and `spawned` :98 made `pub(crate)`). Arms only, in the
files `rg -l 'Op::ReadSignal|Error::NotAFile' crates testing tests
examples` lists (the compiler names any other), `file_layer/store.rs` and
`testing/skein-world/src/real.rs` among them shared; `stream.rs`'s `decode`
(:648) is at clippy's 100 lines: it takes an `#[expect]` of
`clippy::too_many_lines`, with a reason. Tests:
`tests/sim/tests/{files,processes,fuzzy_files}.rs`,
`testing/skein-fake-machine/src/tests.rs`,
`tests/conformance/sim/tests/{conformance,fuzzy_conformance}.rs`,
`tests/conformance/ring/tests/conformance.rs`.
Shape:
- `Op::Lock { fd }` (sketch): valid as it stands, `Shape::Nothing`,
  `Files::Lock`. `Error::Busy` answers a `Lock` alone (`fails_on_files`,
  `names_a_file`); a `Lock` answers `Busy`, `NoBufferSpace`,
  `InvalidArgument` or `Other`, never `Cancelled`. The documentation's
  errors row `| Lock | Busy |`; a `Cancel` of a `Lock`, and a `Lock` on a
  socket's, a pipe's or a path-only descriptor, join its broken
  invariants.
- The ring: in `prepare`, as `List`, `flock(fd, LOCK_EX | LOCK_NB)`
  (`// SAFETY:` integers only) → `Prepared::Done(Ok(Done::Nothing))` or
  `Err(error(kind, errno, false))`; `file_error`: `EWOULDBLOCK` → `Busy`,
  else `None` (`Other`); `check_path_access` asserts its descriptor.
- The simulator: `Ask::Lock { file }` → `Reply::Done` or `Err(Busy)`;
  never hung; of the disk's faults `no_buffer` alone, as `Stat`'s;
  `Summary::Lock { fd }`, a file's, not cancellable: its `Cancel` fails
  the world ("which io never cancels").
- The machine: `Machine::lock` (sketch); the face: `Ask::Lock` →
  `done(machine.lock(opened(file)))`, `kernel_error(Refusal::Held) =
  Error::Busy`.
Test shape: the scenario, `testing/skein-conformance/src/locks.rs`; the
reference (`Locks`: its five answers, each a `pub` `Result<Done, Error>`,
deriving `Clone, PartialEq, Eq, Debug`; `NOTHING` as `files.rs:20`;
`ended` signals `Kill`, waits, reaps and closes, as `processes.rs:86-92`):

```rust
#[must_use]
pub fn lock<B: Backend>(backend: &mut B, executable: &[u8]) -> Locks {
    let mut run = Run::new(backend);
    let process = run.process();
    let root = run.root(process, &[Item::directory(b"store")]);
    let first = run.open(process, root, b"store", OpenHow::Directory).expect("a directory");
    let second = run.open(process, root, b"store", OpenHow::Directory).expect("a directory");
    let taken = run.lock(process, first);
    let through_a_second_open = run.lock(process, second);
    let again_through_the_first = run.lock(process, first);
    run.close(process, first);
    let once_the_first_is_closed = run.lock(process, second);
    let never = command(executable, root, &[b"never"], &[]);
    let (child, _) = spawned(run.call(process, Op::Spawn { spawn: Box::new(never) }));
    run.close(process, second);
    let third = run.open(process, root, b"store", OpenHow::Directory).expect("a directory");
    let beside_a_living_child = run.lock(process, third);
    ended(&mut run, process, child);
    run.close(process, third);
    run.close(process, root);
    run.finish();
    Locks { taken, through_a_second_open, again_through_the_first, once_the_first_is_closed, beside_a_living_child }
}
```

`Check::check` asserts each answer, its message "the contract: …":
`taken`, `again_through_the_first`, `once_the_first_is_closed` and
`beside_a_living_child` are `NOTHING`, `through_a_second_open` is
`Err(Error::Busy)`. It runs as
`a_directory_lock_is_busy_through_a_second_open_until_the_first_closes`
with the process fixture, as `child_processes_and_pipes`: on the
simulator (`calm_and_chaos(|world| lock(world, b"process_fixture"))`), the
ring (`env!("CARGO_BIN_EXE_process_fixture")`) and in
`fuzzy_conformance.rs` (`chaos`, as `groups_and_usage_under_chaos`), named
`a_directory_lock_under_chaos_keeps_the_contract`; Done 3 counts 2 tests,
for the fuzzy profile is not run there.
Beside it, under 20 ms each: in `crates/skein-io/src/tests/kernel.rs`,
`busy_answers_a_lock_and_no_other_operation` and
`a_lock_answers_nothing_and_is_never_cancelled`; in
`tests/sim/tests/files.rs`, as :313 and :321,
`a_cancel_of_a_lock_fails_the_world`,
`a_lock_on_a_path_only_descriptor_fails_the_world`,
`a_lock_on_a_socket_fails_the_world` (as :275, "an operation on files
on"), `a_lock_goes_with_the_handles_of_a_cut_process` (two processes
holding handles of one directory; `Sim::cut` of the holder, its handles
closed on the machine; the other's `Lock` then answers `Nothing`); in
`tests/sim/tests/processes.rs`, `a_lock_on_a_pipe_fails_the_world` (its
`child` with an output pipe; "which is not open in this process", as any
operation on files on a pipe's descriptor at the base, `sim.rs:635`); in
the machine's tests, `a_lock_is_held_through_another_handle_until_it_closes`,
`a_power_loss_frees_every_lock`. Fuzzy: `fuzzy_files.rs`'s workload locks
`root` in each of its four rounds (800 `Lock`s over 200 seeds, `no_buffer`
5 in 1,000), and `files_under_chaos_keep_the_contract_and_every_fault_falls`
asserts some `Lock` completed `Err(NoBufferSpace)`; if none does, the
`Lock`'s `no_buffer` is raised to 20 in 1,000 for this workload only, and
the assert is never skipped.
Proof: process, the real kernel: `heavy cargo nextest run -p skein-conformance-ring
-E 'test(=a_directory_lock_is_busy_through_a_second_open_until_the_first_closes)'`,
once: `flock` through a second open of one directory answers `Busy`
until the first open's close, and a living child holds none. Bound: 1 s.
Nothing above io submits a `Lock` until 05.2 connects it.
Don't: wait, retry or keep a timer for a lock below the owner; add an
unlock or a shared lock; make `Lock` a ring operation or cancellable; map
any errno but `EWOULDBLOCK` to `Busy`.
Size: at most 255 lines.
Removes: nothing.

### 05.2 io's `Lock`
Scope: io.md 5, "Locking": `Lock` on a root, answered `Locked`, `Busy` or
`Failed`, with no wait, no timer and no unlock in io.
Needs: skein 05.1's `Op::Lock` and `Error::Busy`; skein 04 (the files
and tests its 04.1 and 04.2 create, and `Pending`'s matches).
Files: `crates/skein-io/src/file.rs` (`Request::Lock`, `Event::Locked`,
`Event::Busy`, `Event::owner` :283); `src/file_layer.rs` (shared:
`Pending::Lock` and its matches, `up` :724 and `stopped` :851, which give
a `Lock` past its deadline its own answer), `src/file_layer/directory.rs`; users
of `file::Request` whole, `testing/skein-world/src/files.rs:207` and
`crates/skein-oauth-accounts/src/private_tests.rs:111, :367`, and of
`file::Event` whole, `crates/skein-oauth-accounts/src/component/private.rs`
(four matches). Tests: `src/tests/directory.rs`, new
`tests/io/src/locks.rs`, `tests/io/src/lib.rs`,
`tests/io/tests/{directory,ring}.rs`.
Shape:
- `Request::Lock { owner, root: Token }`: an unknown token `Failed {
  NotFound }`; else `Pending::Lock { owner }` and `Op::Lock { fd }`.
  `Done::Nothing` → `Locked { owner }`; `Err(Busy)` → `Busy { owner }`;
  any other error `Failed { error }`.
- Not cancellable: an owner's cancel waits for the answer, which is the
  terminal; a `Lock` that completes past its deadline answers its own,
  `Locked` or `Busy`, not `TimedOut` (io.md, 5: an operation that
  completes without a `Cancel` finishes first). No terminal leaves a
  deadline (`FileIo::next_deadline()` is `None`).
- `Close` of the token frees the lock with its descriptor; a root the
  shell opened keeps it until the process ends.
Test shape:
- Step, `src/tests/directory.rs`, under 20 ms each:
  `a_lock_asks_the_kernel_lock_and_tells_locked`,
  `a_busy_answer_tells_the_owner_busy_and_arms_no_deadline` (then
  `io.next_deadline()` is `None`, `io.descriptor(root)` still the
  root's), `another_kernel_error_fails_the_lock_with_it`,
  `a_lock_of_an_unknown_token_fails_not_found`,
  `an_owners_cancel_of_a_lock_waits_for_its_own_answer`,
  `a_lock_answered_after_its_deadline_keeps_its_own_answer`.
- The world, `tests/io/src/locks.rs` after `cuts.rs`: two restartable
  processes with startup roots of `root-0` (two handles of one
  directory); the holder locks, then, once the referee's flag says the
  waiter heard `Busy`, stats its root, and is cut (`Cut::Kill`) before
  that stat, its fresh incarnation locking nothing; the waiter asks again
  every 200 ms on its own `next_deadline` until `Locked`, keeping
  `heard`; `FileSystem::cut` closes the held handles. Under 0.5 s each:
  `a_second_owner_hears_busy_until_the_first_is_killed` (the waiter's
  `heard` starts `Busy` and ends with its one `Locked`; no handle left
  open), `a_lock_asked_again_through_its_root_is_locked`.
- The real ring: `a_second_root_of_a_locked_directory_hears_busy_over_the_ring`.
Proof: process: `heavy cargo nextest run -p skein-io-world -E
'test(=a_second_root_of_a_locked_directory_hears_busy_over_the_ring)'`,
once: an owner over the real ring holding two roots of one scratch
directory hears `Locked` through the first, `Busy` through the second,
then `Locked` through the second once it has closed the first. Bound:
1 s. hearth 23 is its first caller (store.md 8.2, step 1).
Don't: wait, retry or arm a deadline for a lock in io; free a lock but by
`Close`; take a lock inside `Store`, `Scan` or `ListPage`.
Size: at most 80 lines.
Removes: nothing.

## Interface sketches

The variants at the base stand; these are the new ones. `file::Request`
and `file::Event` are what hearth 23 calls; the rest is skein's own.
Private, as the Shapes give them: `Pending::Lock { owner }`.

```rust
// skein_io::file (crates/skein-io/src/file.rs)
pub enum Request {
    /// An exclusive lock through `root`, taken without waiting: `Locked`, `Busy` or `Failed` (05.2).
    Lock { owner: Token, root: Token },
}
pub enum Event {
    /// The lock is taken, or already held through this root.
    Locked { owner: Token },
    /// Another open of the directory holds the lock; the owner asks again on its own timer.
    Busy { owner: Token },
}

// skein_io::kernel (05.1)
pub enum Op {
    /// `flock(LOCK_EX | LOCK_NB)` on the file or directory open on `fd`, at submit: `Done::Nothing`.
    Lock { fd: Fd },
}
pub enum Error {
    /// `Lock`: another open of the same file holds its lock, in this process or another (`EWOULDBLOCK`).
    Busy,
}

// skein_sim::machine (05.1)
pub enum Ask {
    /// A lock through `file`: `Reply::Done`, or `Error::Busy` while another handle of its file holds one.
    Lock { file: Handle },
}

// skein_fake_machine: fs/locks.rs (05.1)
impl Machine {
    /// Locks what `file` has open: Ok when it holds the lock already or no other handle of its node does.
    pub fn lock(&mut self, file: Opened) -> Result<(), Refusal>;
}
```

## Questions to raise when reached

None.

## Done when

Each command once, reading skein's `origin/main` (the last two in a
worktree of it, through `heavy`):

```sh
git -C ~/src/rust/skein log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'skein-05\.[12]'
git -C ~/src/rust/skein grep -n -E '^    (Lock|Locked|Busy) \{$|^    Busy,$' origin/main -- crates/skein-io/src/file.rs crates/skein-io/src/kernel.rs
heavy cargo nextest run -p skein-conformance-ring -p skein-conformance-sim -E 'test(=a_directory_lock_is_busy_through_a_second_open_until_the_first_closes)'
heavy cargo nextest run -p skein-io-world -E 'test(=a_second_root_of_a_locked_directory_hears_busy_over_the_ring)'
```

The first prints 2; the second 5 lines, file.rs's `Lock`, `Locked` and
`Busy`, and kernel.rs's `Lock` and `Busy`; the third passes with a
`Summary` naming 2 tests, the fourth 1.
