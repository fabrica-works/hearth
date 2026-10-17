# Session 26: the fake machine's M1 parts

`testing/hearth-fake-machine` starts with M1's parts (testing.md, 10):
a `linux` half of files, processes, pipes and a ledger, and a `hearth`
half holding the program table, behind the io face hearth 27, 46 and 28 act
on. Fake git, `testing/hearth-fake-git`, and git's conformance scenario
on it are hearth 44's.

## Read first

- hearth's testing.md: 3.1, 3.2, 3.5, 3.11, 10; decisions.md, 16, 17;
  code-organization.md, 1 and 3; later.md's rows for the fake.
- `findings/skein-testing.md`, 4, 5, Gaps; `findings/skein-io.md`, 2, 3, 5.
- skein: `testing/skein-fake-machine/src/fs.rs:40-160, 652-700`, its
  `Cargo.toml` and `clippy.toml`; `crates/skein-io/src/kernel.rs:520-660`.
- skein 05's session file, 05.1 (`Lock`). skein's `programming-model.md`:
  3, 10.2; `testing-strategy.md`: 4, 4.3, 5 (by number only).

## What the design asks

- testing.md 3.1: "io face | io's records: files by name, a full stat,
  ..., spawns with argument vectors ..., bytes on pipes, exits", which
  "grows as" hearth's records do; "asserts what its client guarantees: a
  handle it issued, a process it started"; "It keeps a ledger ... in a
  bounded queue the referee drains; and counts".
- 3.2: "Listings come back in an order drawn from the seed"; "a directory
  moved into another directory needs write permission on itself".
- 3.5: "A file no program claims exits 127; a missing one is ENOENT."

## Choices this plan makes

- **Settled by the plan:** decisions.md 16, 17; code-organization.md 1
  and 3 (hearth 28's face behind the simulator and io's file records);
  later.md's rows for the fake.
- **The io face is the machine's operations at the kernel's grain**
  (`linux::Op`), mirroring `skein_io::kernel`'s types by name and field;
  `linux` and `hearth` name no skein-io (3.11); hearth 28.1's `behind` does.
- **M1 keeps** files, directories, symbolic and hard links; owner, group,
  the mode's 0o7777 bits; contents by handle; a change counter per file;
  seeded listings; access by bits; renames' rule; `flock`; processes, pipes,
  signals, exits. No process runs as uid 0: `login` asserts it.

## Rules that bind it

- **Shared files:** the root `Cargo.toml` and `Cargo.lock` (26.1; hearth
  44.1 adds `hearth-fake-git` and `sha1`); `testing/hearth-fake-machine/`'s
  `src/hearth.rs` (26.2; hearth 27, 28.1, 46.1) and `src/linux.rs`,
  `src/linux/*.rs` (26; hearth 28.1).
- **A fake changed** (testing.md, 5.5): a landing that changes
  `testing/hearth-fake-machine/` also runs `heavy cargo nextest run
  --workspace --profile conformance` once, from hearth 44.3 on.
- **No network, no host file:** `clippy.toml` bans `std::fs`.

## Increments

### 26.1 The `linux` half's files, and the io face for them
Scope: files as Choices keep them (3.2), their io face with skein 05's
`Lock` (3.1: it "grows as" hearth's records do).
Needs: nothing.
Files: root `Cargo.toml` (shared: `[workspace.dependencies]`'s
`hearth-fake-machine`, and `skein-lib` unless hearth 01 added it),
`Cargo.lock` (shared); new `testing/hearth-fake-machine/`: `Cargo.toml`
(`skein-lib` for `Rng`; `[lints] workspace = true`), `clippy.toml`,
`src/lib.rs` (`#![forbid(unsafe_code)]`, `extern crate alloc`),
`src/linux.rs`, `src/linux/files.rs`; tests
`src/tests.rs`, `src/tests/fixtures.rs`, `src/tests/files.rs`.
Shape: the sketch's `linux`, `Op::Open` to `Op::Close`. `lay` is the
world's, unchecked, parents first. `At::Beneath` resolves as io does:
each directory searched, `..` never above the handle (`Escape`), a link
followed only where `How` allows (`Loop`), at most 40; `Root` and `Cwd`
as a tool's paths. Refusals in skein-fake-machine's order: search, the
name, the call's own check. A rename needs write and search on both
parents, and write on a directory that changes parent; a sticky
directory lets only the entry's or its own owner remove or rename it.
`Create`'s mode is less the umask (0o022), 0o666 without one. Contents,
owner, mode, link count and a rename bump `changes`. A directory
handle's first `List` draws its order with `Rng::below`. `Lock` is skein
05.1's `flock` (`LOCK_EX | LOCK_NB`), which hearth 23 takes on the
store's root: `Done` when taken or already held through this handle,
`Busy` while another handle of the file holds it; it goes with its
handle (`Close`, and 26.2's exit); on a pipe or a `PathNoFollow` handle
it asserts. A handle the process does not hold, a call from an exited
process asserts, naming 3.1's "a handle it issued" or "a process it
started". `login` as uid 0 asserts. A sticky directory is as above;
`Loop` is the 41st link.
Test shape: `src/tests/files.rs` on `src/tests/fixtures.rs` (new:
`SEED`; `BUILDER` 1000:1000, `OTHER` 1001; `machine()` with
`Sizes::tiny()`, the scenario's own (3.1); `open(machine, pid, path,
how) -> Handle`); each under 20 ms:
`a_file_created_takes_its_mode_less_the_umask_and_its_creator`,
`a_path_that_climbs_above_its_handle_is_refused_as_an_escape`,
`a_link_on_the_way_is_refused_when_the_open_follows_none`,
`reading_a_file_whose_other_bits_deny_it_is_refused`,
`a_rename_into_another_directory_needs_write_on_the_moved_directory`,
`a_listing_comes_back_in_an_order_drawn_from_the_seed`,
`a_sticky_directory_lets_only_an_owner_remove_an_entry`,
`a_chain_of_forty_one_links_is_refused_as_a_loop`,
`a_login_as_uid_zero_asserts` (`#[should_panic]`),
`a_lock_is_busy_through_another_handle_until_the_first_closes` (skein
05.1's machine test), `a_handle_the_process_does_not_hold_asserts`
(`#[should_panic(expected = "a handle it issued")]`, as hearth 01's).

```rust
#[test]
fn a_rename_into_another_directory_needs_write_on_the_moved_directory() {
    let mut machine = fixtures::machine();
    let dir = |path: &[u8], mode| Item::directory(path, mode).owner(fixtures::BUILDER);
    machine.lay(&[dir(b"/srv", 0o755), dir(b"/srv/from", 0o755), dir(b"/srv/from/moved", 0o555), dir(b"/srv/to", 0o755)]);
    let builder = machine.login(fixtures::BUILDER, b"/srv");
    let srv = fixtures::open(&mut machine, builder, b"/srv", How::Directory);
    let rename = |from: &[u8], to: &[u8]| Op::Rename { from_dir: srv, from: from.into(), to_dir: srv, to: to.into() };
    assert_eq!(machine.call(builder, rename(b"from/moved", b"to/moved")), Err(Refusal::Permission), "a new parent rewrites `..`");
    assert_eq!(machine.call(builder, rename(b"from/moved", b"from/kept")), Ok(Reply::Done), "one parent needs only its write");
}
```
Proof: unit, that test: the kernel's rename rule, through a process's
call; nothing calls the crate until hearth 44.2, then hearth 46.1, 28.
Bound: 0.02 s.
Don't: name skein-io or skein-sim; keep M2's metadata (later.md); touch
a host file (`clippy.toml` bans `std::fs`); add an unlock or a shared lock.
Size: at most 290 lines (the ledger is 26.2's).
Removes: nothing.

### 26.2 Processes, pipes and exits; the program table
Scope: spawns, pipes, signals and exits (3.5); the program table; the
ledger (3.1), moved here from 26.1.
Needs: nothing beyond 26.1.
Files: `src/linux.rs`, new `src/linux/processes.rs`, new
`src/linux/ledger.rs`, new `src/hearth.rs`, `src/lib.rs` (re-exports
`hearth::Machine`); tests `src/tests/processes.rs` (new),
`src/tests/ledger.rs` (new), `src/tests/fixtures.rs`.
Shape: the sketch's `Op::Spawn` to `Op::Exit`, and `hearth`. Each call,
26.1's too, appends one `Entry`; a full ledger asserts, naming 3.1's
"bounded queue the referee drains". A spawn
checks `dir` beneath `root`, then `program` from the process's root:
search on the way (`Permission`), missing (`NotFound`), a directory or
no execute bit (`Permission`). The child inherits ids and umask, leads
its own group, and keeps `args` and `env` exactly
(`findings/skein-io.md`, 2); a descriptor given no pipe acts as
`/dev/null`. A pipe holds `sizes.pipe` bytes: `Send` takes what fits,
`BrokenPipe` once no reader holds it; `Take` is `Empty` while a writer
holds it, `Ended` once none does and it is drained. `Kill` ends the
child, or its group, at once (`Exit::Signal(9)`); `Terminate` stays
`pending` for its program, which ends by `Op::Exit` with a code or a
signal (`Exit::Signal(15)` on TERM, `Exit::Signal(13)` on a broken
stdout; testing.md, 3.5). An exit closes the process's handles; `Wait`
answers `Exited` once. A child whose program no tool claims exits 127
before `Spawned` returns. `counts(pid)`, read at settle (3.1): the bytes
its `Read`s and `Write`s moved, the files whose counter it moved, the
children it spawned; kept past its exit.
Test shape: `src/tests/processes.rs`; fixtures gain `whole() ->
hearth::Machine`, `builder(machine) -> Pid`, `install_git(machine)`,
`spawn(machine, parent, program, args, pipes) -> (Pid, Box<[Handle]>)`;
each under 20 ms: `a_spawn_of_a_file_no_program_claims_exits_127`,
`a_spawn_of_a_missing_program_is_refused_as_not_found`,
`a_program_its_ids_cannot_execute_is_refused_as_permission`,
`a_child_gets_its_arguments_and_exactly_its_environment`,
`a_full_pipe_takes_no_more_until_its_reader_takes_some`,
`a_kill_ends_the_child_with_signal_nine_and_ends_its_pipes`,
`counts_give_a_process_s_bytes_changes_and_spawns_after_its_exit`,
`a_terminate_stays_pending_for_its_program`,
`a_descriptor_given_no_pipe_acts_as_dev_null`,
`an_exit_releases_the_process_s_lock`,
`a_call_from_an_exited_process_asserts` (`#[should_panic(expected = "a
process it started")]`); `src/tests/ledger.rs`:
`each_change_to_a_file_moves_its_counter_and_lands_in_the_ledger`,
`a_full_ledger_asserts` (`#[should_panic(expected = "ledger")]`).

```rust
#[test]
fn a_spawn_of_a_file_no_program_claims_exits_127() {
    let mut machine = fixtures::whole();
    machine.linux_mut().lay(&[Item::file(b"/usr/bin/stray", 0o755, b"#!/bin/sh\n")]);
    let builder = fixtures::builder(&mut machine);
    let (child, pipes) = fixtures::spawn(&mut machine, builder, b"/usr/bin/stray", &[], &[Pipe { child: 1, way: Way::Out }]);
    assert_eq!(machine.tool(child), None, "no tool claims the file");
    assert_eq!(machine.call(builder, Op::Wait { child }), Ok(Reply::Exited(Exit::Code(127))), "testing.md, 3.5");
    assert_eq!(machine.call(builder, Op::Take { pipe: pipes[0], most: 64 }), Ok(Reply::Ended), "its stdout ends with it");
}
```
Proof: unit, that test: a spawn and its exit, seen by the parent;
hearth 44.2 spawns fake git through it. Bound: 0.02 s.
Don't: name a tool in `linux` (3.11); model namespaces, cgroups or mount
views (M2); keep a clock.
Size: at most 340 lines (the ledger's 40 moved here from 26.1, to keep
that one under 300; 340 stands: the ledger is one module, not split).
Removes: nothing.

## Interface sketches

The io face field by field, the rest as signatures; a process ends by
`Op::Exit` or a signal; elided fields are private.

```rust
// testing/hearth-fake-machine/src/linux.rs (26.1, 26.2)
use alloc::boxed::Box;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Pid(u32);  // A process the world logged in or the machine spawned; issued once.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Handle(u64);  // A file, directory or pipe end a process holds; issued once.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Node(u64);  // A file's identity, for the program table; issued once.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Ids { pub user: u32, pub group: u32 }  // The ids a process acts with, as on disk: one filesystem, no namespace in M1.
/// The scenario's sizes; `tiny()`: 64 files, 1 MiB, 8 processes, a 4,096-byte pipe, 256 ledger entries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Sizes { pub files: u32, pub bytes: u64, pub processes: u32, pub pipe: u32, pub ledger: u32 }
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Item { pub path: Box<[u8]>, pub made: Made, pub mode: u32, pub ids: Ids }  // What the world lays: an absolute path, what it is, its mode and owner (0:0 unless `owner`).
#[derive(Clone, PartialEq, Eq, Debug)] pub enum Made { File(Box<[u8]>), Directory, Link(Box<[u8]>), HardLink(Box<[u8]>) }  // A laid entry's kind: bytes; a directory; a target; an existing path.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum At { Root, Cwd, Beneath(Handle) }  // Where a path starts: a tool's absolute path; a tool's relative one; beneath a handle, as io resolves.
/// How a path opens: `skein_io::kernel::OpenHow`, variant for variant, with its meanings.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum How {
    PathNoFollow, Read, ReadNoFollow, Directory, DirectoryNoFollow,
    Create { mode: Option<u32> }, CreateNoFollow { mode: Option<u32> }, Write, WriteNoFollow,
}
/// One call of the io face, answered by the `Reply` its comment names, or a `Refusal`.
#[derive(PartialEq, Eq, Debug)] pub enum Op {
    Open { at: At, path: Box<[u8]>, how: How },                                  // Opened
    Read { file: Handle, at: u64, most: u32 },                                   // Read: empty at the end
    Write { file: Handle, at: u64, bytes: Box<[u8]> },                           // Done: every byte
    Truncate { file: Handle, length: u64 },                                      // Done
    Sync { file: Handle },                                                       // Done: no crash model in M1
    Stat { handle: Handle },                                                     // Stat
    Rename { from_dir: Handle, from: Box<[u8]>, to_dir: Handle, to: Box<[u8]> }, // Done
    Remove { dir: Handle, path: Box<[u8]>, directory: bool },                    // Done: a file, or an empty directory
    MakeDirectory { dir: Handle, path: Box<[u8]>, mode: u32 },                   // Done: less the umask
    List { dir: Handle, most: u32 },                                             // Listed: the seed's order, empty at the end
    Lock { file: Handle },                                                       // Done, or Busy: skein 05.1's flock
    Close { handle: Handle },                                                    // Done
    Spawn(Spawn),                                                                // Spawned (26.2)
    Send { pipe: Handle, bytes: Box<[u8]> },                                     // Sent: what fits
    Take { pipe: Handle, most: u32 },                                            // Took, Empty or Ended
    Signal { child: Pid, signal: Signal, to: Target },                           // Done: io's Request::Signal
    Wait { child: Pid },                                                         // Running, or Exited once
    Exit { exit: Exit },                                                         // Done: the caller ends, by a code or a signal
}
/// What a call answers; `Spawned` gives the parent's ends in the request's order.
#[derive(PartialEq, Eq, Debug)] pub enum Reply {
    Opened(Handle), Read(Box<[u8]>), Done, Stat(Facts), Listed(Box<[(Kind, Box<[u8]>)]>),
    Spawned { child: Pid, pipes: Box<[Handle]> }, Sent(u32), Took(Box<[u8]>), Empty, Ended, Running, Exited(Exit),
}
/// Linux's errors: ENOENT EEXIST ENOTDIR EISDIR ENOTEMPTY EACCES EPERM (sticky) EXDEV ELOOP ENAMETOOLONG EINVAL EPIPE EWOULDBLOCK.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Refusal { NotFound, Exists, NotADirectory, IsADirectory, NotEmpty, Permission, NotPermitted, Escape, Loop, NameTooLong, Invalid, BrokenPipe, Busy }
/// What `Stat` finds: M1's part of 3.2's metadata; `mode` holds the 0o7777 bits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Facts { pub kind: Kind, pub size: u64, pub mode: u32, pub user: u32, pub group: u32, pub links: u32, pub changes: u64 }
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub enum Kind { File, Directory, Link }  // What a name names in M1.
/// `skein_io::kernel::Spawn`, field for field; `args` exclude argv[0].
#[derive(PartialEq, Eq, Debug)] pub struct Spawn { pub program: Box<[u8]>, pub args: Box<[Box<[u8]>]>, pub env: Box<[Box<[u8]>]>, pub root: Handle, pub dir: Box<[u8]>, pub pipes: Box<[Pipe]> }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Pipe { pub child: u32, pub way: Way }  // `kernel::Pipe` less `parent`, which `Spawned` gives.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Way { In, Out }  // `kernel::Way`, `Signal` (TERM kept pending, KILL at once), `Target` and `Exit`, variant for variant.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Signal { Terminate, Kill }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Target { Child, Group }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Exit { Code(u8), Signal(u32) }
/// What a spawned child runs with; `dir` is its working directory's absolute path, `cwd` its handle.
#[derive(PartialEq, Eq, Debug)] pub struct Invocation { pub program: Box<[u8]>, pub args: Box<[Box<[u8]>]>, pub env: Box<[Box<[u8]>]>, pub dir: Box<[u8]>, pub cwd: Handle }
/// One ledger line: who, as whom, what it touched, the bytes moved, what was refused.
#[derive(PartialEq, Eq, Debug)] pub struct Entry { pub pid: Pid, pub ids: Ids, pub node: Option<Node>, pub bytes: u64, pub refused: Option<Refusal> }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Counts { pub read: u64, pub written: u64, pub changed: u64, pub spawned: u32 }  // What a world reads at settle, per process.
#[derive(Debug)] pub struct Machine { /* nodes, contents, handles, processes, pipes, rng, sizes, ledger */ }  // The generic half: files, processes, pipes, the ledger.
impl Item { pub fn file(path: &[u8], mode: u32, bytes: &[u8]) -> Item; pub fn directory(path: &[u8], mode: u32) -> Item; }
impl Item { pub fn link(path: &[u8], target: &[u8]) -> Item; pub fn hard_link(path: &[u8], existing: &[u8]) -> Item; pub fn owner(self, ids: Ids) -> Item; }
impl Machine { pub fn new(seed: u64, sizes: Sizes) -> Machine; pub fn lay(&mut self, items: &[Item]) -> Box<[Node]>; }
impl Machine { pub fn login(&mut self, ids: Ids, dir: &[u8]) -> Pid; pub fn call(&mut self, pid: Pid, op: Op) -> Result<Reply, Refusal>; }
impl Machine { pub fn invocation(&self, pid: Pid) -> &Invocation; pub fn descriptor(&self, pid: Pid, fd: u32) -> Handle; }
impl Machine { pub fn pending(&self, pid: Pid) -> Option<Signal>; pub fn program(&self, pid: Pid) -> Node; }
impl Machine { pub fn facts(&self, path: &[u8]) -> Option<Facts>; pub fn contents(&self, path: &[u8]) -> Option<&[u8]>; }
impl Machine { pub fn ledger(&mut self) -> alloc::collections::vec_deque::Drain<'_, Entry>; pub fn counts(&self, pid: Pid) -> Counts; }

// testing/hearth-fake-machine/src/hearth.rs (26.2)
/// The whole machine: the `linux` half, and which installed file is which tool.
#[derive(Debug)] #[expect(clippy::zero_sized_map_values, reason = "one tool until hearth 46.1")] pub struct Machine { linux: crate::linux::Machine, programs: alloc::collections::BTreeMap<crate::linux::Node, Tool> }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Tool { Git }  // A tool whose fake a world starts on a spawn of it; hearth 46.1 adds `Mmdebstrap`.
impl Machine { pub fn new(seed: u64, sizes: crate::linux::Sizes) -> Machine; pub fn install(&mut self, path: &[u8], tool: Tool) -> crate::linux::Node; }
impl Machine { pub fn call(&mut self, pid: crate::linux::Pid, op: crate::linux::Op) -> Result<crate::linux::Reply, crate::linux::Refusal>; }
impl Machine { pub fn tool(&self, pid: crate::linux::Pid) -> Option<Tool>; pub fn linux(&self) -> &crate::linux::Machine; pub fn linux_mut(&mut self) -> &mut crate::linux::Machine; }
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth ls-tree -d --name-only origin/main
   testing/hearth-fake-machine | wc -l` prints 1.
2. In a worktree at `origin/main`, `heavy cargo nextest run -p
   hearth-fake-machine` passes, naming at least 24 tests.
3. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-26\.[1-2]'` prints 2.
