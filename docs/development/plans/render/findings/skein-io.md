# Findings: skein's io and shell for hearth's M1

Read at skein 7e8fb70 (main) on 2026-10-16; smith at f4f7f675 (main). Paths are relative to each checkout.

## 1. The service's loop and binary

- Exists. `pub trait Host` (crates/skein-shell/src/drive.rs:12): `fn iterate(&mut self, now: Time, wall: Wall)` (:18), `completions`, `submissions`, `work_pending`, `next_deadline`, `next_policy_deadline`, `is_empty`, `fn exit(&self) -> Option<Exit>` (:40, default `Code(0)` once empty), `worst_case`, `operations`, `drain`.
- Exists. `pub fn drive(kernel: &mut Kernel, clock: &Clock, host: &mut impl Host) -> Exit` (drive.rs:53): reap, clock, `iterate`, `drain`, then return `host.exit()` once `is_empty()`, else submit. A service's `iterate` is called from its `Host::iterate`; the host owns io's queues and runs io's `resume`/`up`/`fire`/`down` (crates/skein-io/src/lib.rs:11-24).
- Missing in code: `Host::state_digest()` is designed (docs/design/shell.md:424) but absent from drive.rs. It is one of hearth's asks of skein (README.md, 8.1).
- Startup kit: `Kernel::open(Config { operations })` (crates/skein-shell/src/ring.rs:435), `Clock::new` (clock.rs:28), `seed() -> Result<u64, i32>` (seed.rs:12), `crypto_seed() -> Result<Secret, i32>` (seed.rs:20), `open_root(path) -> Result<Fd, i32>` (ring.rs:1595), `make_root(path, mode) -> Result<Fd, i32>` (ring/roots.rs:11), `open_termination_signals` (ring.rs:97), `read_file` (startup.rs:37).
- argv and environment: no kit function. shell.md, section 6 (lines 123-154) names configuration but no argv or environment reader. Mains read them with std in shell code: `std::env::args()` (examples/ui/server/src/main.rs:69); `std::env::var_os` (smith: crates/smith-local-shell/src/compose.rs:408).
- Exit status: `drive` returns `kernel::Exit` (`Code(u8)` or `Signal(u32)`, crates/skein-io/src/kernel.rs:600); `main` maps it to `std::process::ExitCode` itself (examples/ui/server/src/main.rs:103-107). No kit helper.
- Own stdout/stderr as streams: `prepare_output(Fd) -> Result<OutputKind, Error>` (ring.rs:151; `Pipe` or `Append`, ring.rs:141), then `Io::adopt_write_pipe(fd, write_timeout: Option<Duration>) -> Result<Token, Fd>` (crates/skein-io/src/layer.rs:167) or `Io::adopt_append(fd, write_timeout)` (layer.rs:174). Bytes go as `Request::Stream { stream, down }` (records.rs:32) with lib's `Down::Send`/`Finish`. Used for fd 1 in smith: crates/smith-local-shell/src/compose.rs:327.
- The `drain` hook may write one short line to stderr outside io (shell.md:432-441).

## 2. Child processes

- Exists. `io::Request::Spawn { owner: Token, spawn: kernel::Spawn }` (crates/skein-io/src/records.rs:43). `kernel::Spawn { program: Box<[u8]>, args: Box<[Box<[u8]>]>, env: Box<[Box<[u8]>]>, root: Fd, dir: Box<[u8]>, pipes: Box<[Pipe]> }` (kernel.rs:523); `Pipe { child: u32, way: Way, parent: Option<Fd> }` (kernel.rs:534); `Way::{In, Out}` seen by the child (kernel.rs:543).
- The environment is exactly `env` (`NAME=value` entries), passed as envp to `pidfd_spawn`: nothing is inherited (crates/skein-shell/src/ring/process.rs:178, :265).
- `program` is exec'd by path: `pidfd_spawn`, not a `p` variant (ring/process.rs:17), so there is no PATH search. smith passes `/usr/bin/git` (smith: crates/smith-local-protocol/src/git_child.rs:228).
- The working directory is `dir`, opened beneath the root `root` with `RESOLVE_BENEATH` (ring/process.rs:101-107); empty means the root itself. The cwd must lie beneath a root the shell opened.
- Standard descriptors not given a pipe read/write `/dev/null` (ring/process.rs:153). The child leads its own process group (ring/process.rs:245).
- Events: `Event::Spawned { owner, child: Token, pipes: Box<[Token]> }` (records.rs:106), pipe tokens in request order; `Event::Exited { owner, exit: Exit }` (records.rs:111); `Failed`, then `Closed { owner }` last (records.rs:137, :143).
- Pipes are lib streams: `Request::Stream { stream, down: stream::Down }` with `Demand { read, room }`, `Send(Box<[u8]>)`, `Finish` (crates/skein-lib/src/stream.rs:51-67); `Event::Stream { owner, up }` with `Bytes`, `Room`, `End`, `Failed` (stream.rs:72-90). A stdin pipe (`Way::In`) is written with `Send` and ended with `Finish`.
- Signals: `Request::Signal { child, signal: Signal, to: Target }` (records.rs:45); `Signal::{Terminate, Kill}` (kernel.rs:550); `Target::{Child, Group}` (kernel.rs:559), through the pidfd. One signal in flight; a second is dropped silently (docs/design/io.md:850-854).
- `Request::Close { entity: child }` kills the child's group, then reaps (crates/skein-io/src/process.rs:1-12; io.md:855-860). Deadlines are the owner's (io.md:861-866).
- No production code in the four repositories writes to a child's stdin (`Way::In` appears only in skein's io, shell, sim and tests).

## 3. Files and directories

- Records: `skein_io::file::Request` and `file::Event` (crates/skein-io/src/file.rs:31, :192), run by `FileIo` (crates/skein-io/src/file_layer.rs:221): `FileIo::new(files, max_read, max_entries, timeout)` (:298), `adopt_root(Fd) -> Option<Token>` (:349), `descriptor(Token) -> Option<Fd>` (:355), `seed_randomness(u64)` (:360), `down(io, now, request, events, subs)` (:423), `down_until(.., deadline, ..)` (:428).
- One file request at a time per `FileIo`: `assert!(io.takes(), ...)` (file_layer.rs:435).
- Open for reading: `OpenRead { owner, root: Fd, name }` (file.rs:58) → `Opened { file: Token, len: u64 }` (file.rs:202); then `ReadAt { file, offset, max }` (file.rs:126) → `Read { bytes }`, each at most `max_read`. Whole file: `Load { root: Token, path, max, no_follow }` (file.rs:89). A file read as a stream does not exist (io.md:662-663, :1035).
- Create and write: `Create { owner, root: Fd, name, mode }` (file.rs:45); `WriteAt { file, offset, bytes }` (file.rs:120) → `Written`.
- fsync: `Sync { file }` (file.rs:138) → `Synced`; directory: `SyncDirectory { root: Fd }` (file.rs:146).
- Rename: `Rename { owner, root: Fd, from, to }` (file.rs:150). Both are single names in the same directory: `is_name` refuses `/` (file_layer.rs:576-582; kernel.rs:695). The kernel record `Op::Rename { from_dir, from, to_dir, to }` takes two directories, but no file request exposes it.
- Unlink: `Remove { owner, root: Fd, name, directory: bool }` (file.rs:157): one file, or one empty directory.
- mkdir: `MakeDirectory { owner, root: Token, path, mode: u32 }` (file.rs:33) → `Made`; the mode is masked by the umask, and the parent is synced before the answer (io.md:597-607). At startup: `make_root(path, mode)`.
- readdir: `List { owner, root: Fd }` (file.rs:164) → `Listed { entries: Box<[Entry { name, kind }]> }` (file.rs:261). It lists the whole directory, or fails with `NoBufferSpace` past `max_entries` (file_layer.rs:1260-1262). There is no paging. `Scan` (file.rs:97) keeps the first entries by name and counts the rest.
- stat: `Stat { owner, file: Token }` (file.rs:116) → `Stated { stat }` with `kernel::Stat { kind, size, mode, owner, links }` (kernel.rs:647). It works only on an open file; there is no stat by path.
- Remove a tree: missing by design: "io never traverses to remove a tree" (io.md:608-613). The owner composes `OpenDirectory`, `List` and `Remove`.
- `flock`: missing in the kernel records, the ring, io and the docs (grep of crates and docs/design: no match).
- `statfs`: missing (same grep).
- skein's `Store` (io.md, 5.2) is `file::Request::Store { owner, root: Token, path, bytes: Box<[u8]>, expected: Expect, no_follow }` (file.rs:107); `Expect::{Any, Absent, Digest(Digest)}` (file.rs:13) → `Stored { digest }` (file.rs:219), `Conflict { now }`, or `Failed { error, committed, residue }` (file.rs:265). The record is one whole buffer.
- Its temporary is named `<target>.skein-<16 hex>` (crates/skein-io/src/file_layer/store.rs:228-245), not `<name>.tmp`.
- `skein_kv::Store` (crates/skein-kv/src/store.rs:109) is kv's log store (kv.md), not io.md's replace.
- SHA-256: `skein_lib::sha256::{Sha256, digest}` (crates/skein-lib/src/sha256.rs), streaming and one-shot, landed by temper's forge-setup plan; `skein_io::digest::digest(&[u8]) -> Digest` (crates/skein-io/src/digest.rs:22) is a thin layer over it.

## 4. Randomness, the wall clock, timers

- `lib::Rng::new(seed: u64)` (crates/skein-lib/src/rng.rs:12), `next_u64()` (:16), SplitMix64, "not for secrets": 4 draws give 64 hex digits. `CryptoRng::new(&Secret, stream) -> Option<CryptoRng>` (crypto_rng.rs:16), `fill(&mut [u8])` (:33). Both are seeded at startup from `seed()` / `crypto_seed()`.
- The wall clock: `Env::wall: Wall` (crates/skein-lib/src/env.rs:7), read once per iteration from `CLOCK_REALTIME` (crates/skein-shell/src/clock.rs:33); `Wall::as_nanos() -> u64` (crates/skein-lib/src/time.rs:33).
- Timers: `lib::Deadlines<K>` (crates/skein-lib/src/deadlines.rs:21): `with_capacity`, `arm(key, at: Time) -> Result<(), K>` (:51), `cancel(key)` (:68), `next() -> Option<Time>` (:77), `expire(now) -> Option<K>` (:83). Each layer holds its own and reports `next_deadline`.

## 5. The simulator's machine seam and the fake machine

- The seam is data (docs/design/simulator.md:67-108). `Sim::calls(&mut Queue<Call>)` (crates/skein-sim/src/sim/files.rs:74), `Sim::answer(&mut Queue<Answer>)` (files.rs:92), `Sim::root(pid, Handle) -> Fd` (files.rs:53), `Sim::append` (files.rs:62).
- `skein_sim::Ask` (crates/skein-sim/src/machine.rs:57): `Spawn { root, program, args, env, dir, pipes }`, `Open`, `Read`, `Write`, `Append`, `Truncate`, `Sync`, `Stat`, `Rename`, `Remove`, `MakeDirectory`, `List`, `Close`. `Reply::{Program, Opened, Read, Stat, Listed, Done}` (machine.rs:108).
- The machine answers a spawn with a `Program` (machine.rs:121): `Echo { input, output }`, `Exit(u8)`, `Never`, `Fork { exit_leader }`, `Service`. There is no program with scripted output. The simulator keeps the pipes, the pidfd, the exit and the group.
- skein's minimal fake machine is `skein-fake-machine` (testing/skein-fake-machine/src/lib.rs). Its `step` (face.rs:19) answers every file `Ask` and maps the names `echo`, `exit`, `fork-exit`, `fork-live` and `never` to programs (face.rs:76-102). It ignores `env` (face.rs:22). `serve(&mut Machine, &mut Sim)` (face.rs:110).
- A world plugs a machine in with `World::with_machine(M: skein_world::Machine)` (testing/skein-world/src/world.rs:210). The trait is `skein_world::Machine` with `fn step(&mut self, call, answers)` (testing/skein-world/src/program.rs:90, :125), plus `open_root`, `open_append` and `cut`.
- A fake tool with real output is a hosted program: `World::host(HostedProgram { program, make: fn(&Spawn, &Inherited) -> P, instances, operations })` (program.rs:26; world.rs:239). A spawn whose `program` matches is answered `Program::Service` by the world, not by the machine (world.rs:710-734). The child then runs as a `Host` over its inherited pipes.
- `skein-fake-checkout` (testing/skein-fake-checkout; docs/design/fake-checkout.md:1-20) holds files, scripted commands and local git metadata for domain worlds. It has no simulator face.

## 6. A reference: spawning git and reading its output

- smith's `GitChild` (smith: crates/smith-local-protocol/src/git_child.rs) is a protocol-layer adapter. It spawns `/usr/bin/git` with stdout and stderr pipes at :225-239. It demands and reads each pipe's `Bytes` at :127-170, records `Exited` at :177, and on its deadline sends `Signal { Kill, Target::Child }` and aborts the pipes at :207-246.
- Its user: `GitChild::new` in smith: crates/smith-local-service/src/process.rs:600, run by the smith-local-shell binary.

## Gaps

- `flock` on a directory: missing. hearth needs it for the store's lock (store.md, 8.2, step 1; 2.4), at M1, since a client's in-process builder takes it.
- `statfs`: missing. hearth needs the filesystem magic to skip the subvolume steps of the scan (store.md, 2.4).
- Rename between two directories: missing from `file::Request` (the kernel record has it). hearth moves apt's indices from `staging/<key>/.../lists/` to `cache/<sha256>` by rename (render.md, 5.2-5.3; store.md, 6.4).
- Removing a tree: missing by design. Removing `staging/<key>/` after a resolution (render.md, 5.2) and the scan's "anything in `staging/`" (store.md, 4.4) need an owner-side walk, which hearth's design does not describe.
- Directory listing past `max_entries`: no paging. `List` fails on a larger directory, and the scan lists `cache/`, which grows without a bound (store.md, 8.2, step 6).
- A file read as a stream: missing; `ReadAt` at offsets is the substitute. hearth streams blobs and reads indices "in bounded chunks" (render.md, 5.3).
- Store's temporary name: `<target>.skein-<16 hex>`, while hearth's scan deletes `*.tmp` (store.md, 4.4; 6.8). As designed, the scan would report io's temporaries as strays.
- argv, environment and exit status: no kit; each main uses std (`std::env`, `ExitCode`). There is no gap in function, but hearth's design should say so.
- PATH lookup: `Spawn` takes a path. hearth's client must find `git` and `mmdebstrap` itself (render.md, 5.2 gives a fixed `PATH`).
- Fake tools: the minimal machine has no scripted program, so hearth's fake `git` and `mmdebstrap` are hosted programs (one `Host` each) in hearth's own fake machine crate.
- Already asked (README.md, 8.1): `Host::state_digest` (shell.md, 12, designed, not built); JSON positions (not audited here).
