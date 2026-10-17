# Session 46: fake mmdebstrap's face; resolve's conformance scenario

The fake machine's `hearth` half and `hearth-fake-mmdebstrap`'s closure
(hearth 27) gain their face: a process on the fake machine that reads
render.md 5.2's vector, leaves apt's lists in the scratch root and prints
what real mmdebstrap prints. Last, hearth 41.1's seven resolve cases run
on the fake, against real mmdebstrap's recordings.

## Read first

- hearth's testing.md: 3.1, 3.5, 3.6, 4.1, 4.2 (mmdebstrap's row), 4.3,
  4.5, 5.1 to 5.3, 5.5, 6.2, 10; render.md: 5.2, 5.3, 9.6, 10 (first two
  bullets); decisions.md, 14; code-organization.md, 1 and 3.
- `hearth/27-fake-mmdebstrap.md` whole (`Universe`, `Mirror`, `closure`);
  `hearth/26-fake-git.md` and `hearth/44-fake-git.md` whole;
  `hearth/08-mirror.md` whole (`Mirror`);
  `hearth/41-resolve-recordings.md` whole (`resolve::{CASES, Case, Resolve,
  stanzas}`, `Mask::also`, `Recording`'s `lists` and `stanzas`);
  `hearth/07-mmdebstrap.md`, 07.1's reference test and the `vector` and
  `output` sketches; `hearth/38-stanzas-recordings.md`, the `lists` sketch;
  `hearth/34-conformance.md`, 34.1.
- skein: `crates/skein-lib/src/sha256.rs` (`skein_lib::sha256`).

## What the design asks

- **The fake** (4.2, 4.3): "`--simulate --mode=chrootless`: `Inst` lines
  and `Packages` indices"; 4.1: "Output follows recordings ... its varying
  fields (names, versions, counts) filled from the fake's state"; "Only
  the invocations hearth makes need a face".
- **Conformance** (5.1 to 5.3): "what each leaves behind is compared";
  resolve's "the closure: names, architectures, versions; the `Packages`
  stanzas it left"; "Left out: times, ..., the order of entries";
  "Allowed differences are data, each with its reason, and kept short".

## Choices this plan makes

- **Settled by the plan:** decisions.md 14; code-organization.md 1 and 3:
  the face in `hearth-fake-mmdebstrap` (hearth 27.3's crate, with its own
  `[lints]` and `clippy.toml`).
- **The face takes 07.1's vector alone** and asserts on anything else
  (testing.md 4.1: another vector asserts), `APT::Install-Recommends "true"` included: the
  fake asserts `"false"`, naming it (testing.md 3.6, 5.2). Lines, pipes, exits and the lists' fixed names (`lock`,
  `partial/`) are those 41.1's recordings show for each outcome, filled
  from the run (4.1: "Every line it prints has the shape of a recorded
  real run"): a failure of the apt pass prints the essential set's `Inst`
  lines first, as apt's first pass does (56 before the unknown package's
  error); `NoCandidate` as 41.1's `no-candidate` case
  (`mail-transport-agent` alone) prints it; a `Broken` of Conflicts in
  the shape of the recorded unmet-dependencies block (` <package> :
  <Field>: <relation>`, then ` but <against> is to be installed` where
  the relation names a marked package, not a provided name; sorted by
  package, as apt lists them); one of Depends or Pre-Depends asserts (no
  recording shows it). One fault, a stall, as fake git.
- **Time is hearth 28's** (4.1: "each takes the time its work takes, on
  the world's clock"): this session's kits keep no clock, and a time
  prints 0; hearth 28's worlds charge each run its time.
- **The real side is 41.1's run,** held to its recording by 41.1's
  `settle` in the same profile; 46.2 compares the fake with that
  recording live, and the gate replays every recording through the fake
  over a committed cut of the index (46.2), so real mmdebstrap runs once
  a case. The live universe is
  `Mirror::index("main")`, which 08.2's script decompressed with
  `/usr/bin/xz`: this session decompresses nothing.

## Rules that bind it

- **No hand-written recording; no network** (code-organization.md, 6):
  46.2 reads `testing/recordings/mmdebstrap/` and the fetched cache, and
  writes neither; its universe cut is written by the profile alone.
- **Shared files:** root `Cargo.toml` and `Cargo.lock` (46.2);
  `testing/hearth-fake-machine/src/hearth.rs` (hearth 26, 27, 28.1, 46.1);
  `tests/conformance/` (hearth 34, 41, 44); `testing/mirror/` (hearth 08).
- **A fake or a recording changed** (testing.md, 5.5: "It runs when a
  fake or a recording changes"): from 46.2, the gate's
  `the_fake_prints_each_mmdebstrap_recording_as_recorded` holds fake
  mmdebstrap to the recordings at every landing, and a landing that
  changes `testing/hearth-fake-{machine,mmdebstrap}/`,
  `testing/recordings/mmdebstrap/` or `testing/mirror/` also runs `heavy
  cargo nextest run --workspace --profile conformance` once.

## Increments

### 46.1 Fake mmdebstrap's face, as a process on the fake machine
Scope: 07.1's vector checked, `sources.list` read as the child, the
mirror reached, apt's lists left, mmdebstrap's lines printed (4.1, 4.3);
`Tool::Mmdebstrap`.
Needs: hearth 27.3 (the closure), hearth 38.1 (apt's list names), hearth
41 (41.1's recordings: the lines to follow).
Files: `testing/hearth-fake-machine/src/hearth.rs` (`Tool::Mmdebstrap`; `Tool`
keeps its `#[repr(u8)]`, and `programs` has no `expect` to drop);
`testing/hearth-fake-mmdebstrap/src/{lib.rs, face.rs, process.rs}`;
tests `src/tests/face.rs` (new), `src/tests/fixtures.rs` (hearth 27.3's; adds `args(proxy)`:
07.1's reference vector, empty request, scratch `/scratch`; `PORT` 3142;
`ARCHIVE` 20261001T030405Z; `BUILDER` and `builder(machine)` as hearth
26's; `on_machine(proxy)`: `debian()` served at `ARCHIVE`, mmdebstrap
installed, `/scratch` the builder's with 07.1's `sources.list`;
`spawn_mmdebstrap(..) -> (Pid, [Handle; 2])`; `drain_both`; `lines`).
Shape: the sketch's `face` and `process`. `check` takes 07.1's arguments
in order (`--include=` once or several; the proxy's `--aptopt` before
the skip) and exactly its three variables, `TMPDIR` the scratch.
`Mmdebstrap::start` asserts `Tool::Mmdebstrap` at `/usr/bin/mmdebstrap`
and no `<scratch>/root` (render.md 5.2: "new and empty for each run");
then, as the child: reads `sources.list`; makes `root/var/lib/apt/lists/`
and its fixed names; `reach(proxy)`, else `Unreachable`; per line, the
suite at its timestamp, else `NotFound`; writes `InRelease` and each
index under apt's names (38.1's rule); resolves; prints. An `Inst` line
is `Inst <name> (<version> <origins> [<architecture>])`, the origins
`<Origin>:<Version>/<Suite>` or `<Origin>:<Suite>` per line holding the
version, joined by `, ` (render.md 10's sample). Failures print as
Choices says. `step` ends the child on a pending TERM or a broken pipe,
and sends as the pipes have room, exactly `stall_after` bytes, then
nothing.
Test shape: `src/tests/face.rs`, each under 20 ms, failure lines and
exits pasted from 41.1's recordings:
`hearth_s_vector_reads_into_its_simulate_request`,
`a_spawned_mmdebstrap_leaves_the_lists_and_prints_each_inst_line`,
`an_unreachable_proxy_fails_with_no_index_in_the_lists`,
`an_absent_snapshot_s_inrelease_is_not_found_and_nothing_resolves`,
`an_unknown_package_fails_after_the_lists_are_written`,
`a_virtual_request_prints_its_providers_and_no_candidate`,
`a_conflict_prints_the_unmet_dependencies_block`,
`a_stalled_mmdebstrap_sends_nothing_more_until_terminated`.

```rust
#[test]
fn a_spawned_mmdebstrap_leaves_the_lists_and_prints_each_inst_line() {
    let mut machine = fixtures::on_machine(Some(fixtures::PORT));
    let builder = fixtures::builder(&mut machine);
    let (child, [stdout, stderr]) = fixtures::spawn_mmdebstrap(&mut machine, builder, &fixtures::args(Some(fixtures::PORT)));
    let mut mmdebstrap = Mmdebstrap::start(&mut machine, child, Faults { stall_after: None });
    while mmdebstrap.step(&mut machine) == Progress::Working {}
    let printed = fixtures::drain_both(&mut machine, builder, stdout, stderr);
    assert!(fixtures::lines(&printed).any(|line| line == b"Inst mawk (1.3.4.20260302-2 Debian:testing [amd64])"), "render.md 5.3's form");
    let packages = b"/scratch/root/var/lib/apt/lists/snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_main_binary-amd64_Packages";
    assert_eq!(machine.linux().contents(packages), Some(fixtures::DEBIAN), "apt's list is the index the mirror served");
    assert_eq!(machine.call(builder, Op::Wait { child }), Ok(Reply::Exited(Exit::Code(0))));
}
```
Proof: unit, that test: 5.2's vector spawned through the io face resolves
on the fake mirror, leaves the list hearth 24 reads and prints the lines
07.2 decodes. Nothing above: hearth 28 routes the builder to it; 46.2
holds it to real mmdebstrap. Bound: 0.02 s.
Don't: a face for a vector 07.1 does not write; read recordings at run
time; keep a clock; write outside `<scratch>/root`.
Size: at most 300 lines. Removes: nothing.

### 46.2 Resolve's conformance scenario: the fake against real mmdebstrap
Scope: 41.1's seven cases on the fake, compared with real mmdebstrap's
recordings live, and replayed from them in the gate (testing.md, 4.5,
5.1 to 5.3, 5.5; decisions.md, 14).
Needs: 46.1; hearth 41 (41.1's world, cases, recordings), hearth 08
(08.1's `Mirror`), hearth 44 (44.3's pattern), hearth 07.3 (`tail`).
Files: `tests/conformance/` (shared): `Cargo.toml` (the two fakes),
`src/lib.rs`, `src/fake_mmdebstrap.rs` (new), `src/resolve.rs`
(`Resolve::version`; `Resolve::simulate`'s body moved to `inputs`, which
it calls), `tests/conformance_resolve.rs`, `tests/recorded_resolve.rs`
(new: a gate binary, outside `binary(/^conformance_/)`);
`testing/mirror/universe` (new, written under `HEARTH_RECORD=1`);
`testing/hearth-fake-mmdebstrap/src/resolve.rs` (`read_by`), and the
fakes' `src/` where a case shows them wrong; `Cargo.lock`.
Shape: the sketch's `fake_mmdebstrap`. `mirror`: `Mirror::index("main")`
as the universe, and the pinned `InRelease`'s Origin, Label, Suite,
Codename and Version (08.1's `Mirror::file`), one suite at the pin's
timestamp; the mirror's digest is the pinned `InRelease`'s SHA-256, and the
vector has no `--keyring`: apt checks with the host's keys (render.md 5.2). `run`: a machine of `CONFORMANCE` sizes (64 files, 256 MiB, 8
processes, 64 KiB pipes, 4,096 ledger entries), the mirror served at
`PORT`, the case's argv (`PORT` for the proxy's port, 9 for
`Reach::Unreachable`), env and dir, and its `sources.list` (07.1's
encoding of `inputs(&Pin::committed(), case)`) at `/scratch`, stepped to
its exit, the mirror taken back, masked as 41.1 masks the real run
(`Mask::also` on `127.0.0.1:<port>` and `127.0.0.1 <port>`): a `Recording`
with `lists` and the stanzas its `Inst` lines name, cut by 41.1's
`resolve::stanzas` from the list it left. The live test takes the argv,
env and dir of 07.1's vector of `inputs`; the replay, the recording's.
`differ` compares: the exit; the `Inst` lines cut after their `])` (apt
ends some with ` []`), sorted, one of equal lines (order left out, 5.3;
--simulate's second phase names a package again: 83 lines name test-min's
66); per pipe, in order, the lines beginning `E: `, `Err:`, `W: `, `Error: `,
`Warning: `; both sides' 07.3 failure, each pipe's lines
admitted in order to a `tail::Tail` under `limits::SHIPPED` and the
stderr one taken unless empty (07.3's `failure` rule): its `cause()`,
`class()` and `lines()`, so that a fake naming another package fails;
`lists` names and kinds; each `Inst` line's stanza reduced to `KEPT`,
against the real ones given (cut from the index live; the recording's
`stanzas` in the gate, none skipping it). `ALLOWED` starts with two
rows, each `case: None` (every case): list sizes (the fake's `InRelease`
and index are its own), and stanza fields outside `KEPT` (3.6; hearth
reads six, render.md 5.3). A row is added only for a difference the fake cannot derive from
its state (apt's explanation of a conflict, if so), never for
test-min's or test-boot's closure.
- **The gate's universe:** `testing/mirror/universe`, beside the pin it
  is cut from: a stanza of the pinned `InRelease`'s release fields, then
  the `KEPT` stanzas, in index order, of every package `resolve::read_by`
  names over the seven cases (marked, candidates weighed, providers
  ranked), so that it resolves each case as the whole index does; under
  `HEARTH_RECORD=1` the live run writes it, else a cut unlike it fails
  naming the flag. A bump of the pin rewrites it: `HEARTH_RECORD=1 heavy cargo nextest run -p hearth-conformance-world --profile conformance -E 'test(=the_committed_universe_is_what_the_cases_read_of_the_index)'`, once.
- **The replay** (4.5: "One recording, two uses"; 5.5, "It runs when a
  fake or a recording changes"): each recording of every version under
  `testing/recordings/mmdebstrap/<mmdebstrap>-apt<apt>/`, through `run`
  over that universe, by
  `differ`; no real mmdebstrap, no cache.
Test shape: in `tests/conformance_resolve.rs` (live):
`fake_mmdebstrap_leaves_what_real_mmdebstrap_recorded_for_each_case`
(one table test over `CASES`, naming the case and the first difference;
the index loaded once, under 30 s),
`the_committed_universe_is_what_the_cases_read_of_the_index` (under 30
s); `no_allowed_difference_touches_test_min_or_test_boot_s_closure`
(each row's case in `CASES` or every case, none named `test-min` or
`test-boot`, its reason not empty, at most four rows; under 20 ms). In
the gate, `tests/recorded_resolve.rs`:
`the_fake_prints_each_mmdebstrap_recording_as_recorded` (under 0.5 s).

```rust
#[test] // tests/conformance_resolve.rs
fn the_committed_universe_is_what_the_cases_read_of_the_index() {
    let fetched = hearth_test_mirror::Mirror::find(&Pin::committed()).expect("fetched, per 08.2");
    let cut = fake_mmdebstrap::cut(&fetched, fetched.pin());
    if hearth_recordings::recording_on() { std::fs::write(UNIVERSE, &cut).expect("written under the flag"); }
    assert_eq!(&*std::fs::read(UNIVERSE).expect("committed"), &*cut, "unlike the cut: rerun with HEARTH_RECORD=1");
}

#[test] // tests/recorded_resolve.rs
fn the_fake_prints_each_mmdebstrap_recording_as_recorded() {
    let mut mirror = Some(fake_mmdebstrap::committed());
    for (name, real) in hearth_recordings::all("mmdebstrap") { // (case name, Recording), every version
        let (case, spawned) = (resolve::case(&name), Spawned { argv: &real.argv, env: &real.env, dir: &real.dir });
        let fake = fake_mmdebstrap::run(&mut mirror, case, &spawned);
        assert_eq!(fake_mmdebstrap::differ(case, &real, &fake, real.stanzas.as_deref()), None, "{name}");
    }
}
```
Proof: live, `heavy cargo nextest run -p hearth-conformance-world
--profile conformance -E
'test(=fake_mmdebstrap_leaves_what_real_mmdebstrap_recorded_for_each_case)'`:
on forky's pinned universe of about 70,000 packages, the fake leaves for
each of the seven cases what real mmdebstrap recorded: test-min's and
test-boot's closures triple for triple, their stanzas' kept fields, and
the unknown package's, `no-candidate`'s, the conflict's, the dead
port's and the absent snapshot's lines, exits, causes and packages.
Bound: 30 s of the profile's 2 minutes. Then the gate holds the fake to
the recordings at every landing.
Don't: write a recording; allow a difference without a reason; change
41.1's real side but the accessor and the move; run real mmdebstrap
again; read the cache in `recorded_resolve.rs`.
Size: at most 100 lines (`read_by` and fixes to the fakes; the world is
not counted). Removes: nothing.

## Interface sketches

Derives are elided where a line names none (`Clone`, `PartialEq`, `Eq`,
`Debug`; `Copy` where it holds); elided fields are private.

```rust
// testing/hearth-fake-mmdebstrap/src/resolve.rs (hearth 27.3's; 46.2 adds `read_by`)
/// What `closure` reads for `request` (marked, candidates weighed, providers ranked), in universe order (46.2).
pub fn read_by(request: &Request<'_>) -> Box<[Selected]>;

// testing/hearth-fake-machine/src/hearth.rs (hearth 26's, 27.2's; 46.1 adds `Tool::Mmdebstrap`)
/// A tool whose fake a world starts on a spawn of it: git (hearth 26), mmdebstrap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] #[repr(u8)] pub enum Tool { Git, Mmdebstrap }

// testing/hearth-fake-mmdebstrap/src/face.rs (46.1)
/// What 07.1's vector asks, in the fake's words; `include` split into names.
pub struct Simulate { pub variant: Variant, pub architecture: Box<[u8]>, pub include: Box<[Box<[u8]>]>, pub proxy: Option<u16>, pub suite: Box<[u8]>, pub scratch: Box<[u8]> }
/// One `sources.list` line: its URI, archive, timestamp, suite and components.
pub struct Source { pub uri: Box<[u8]>, pub archive: Box<[u8]>, pub at: Timestamp, pub suite: Box<[u8]>, pub components: Box<[Box<[u8]>]> }
/// One `Inst` line's fields.
pub struct Inst { pub name: Box<[u8]>, pub version: Box<[u8]>, pub architecture: Box<[u8]>, pub origins: Box<[u8]> }
/// How a run ended: resolved; the mirror unreachable; a line's `InRelease` not found; the apt pass failed after the essential set's lines.
pub enum Outcome { Resolved(Box<[Inst]>), Unreachable, NotFound { line: u32 }, Failed { essential: Box<[Inst]>, failure: Failure } }
/// What a run prints, and its exit status.
pub struct Printed { pub stdout: Box<[u8]>, pub stderr: Box<[u8]>, pub exit: u8 }
/// 07.1's vector and environment, and its `sources.list` lines; anything else asserts.
pub fn check(args: &[Box<[u8]>], env: &[Box<[u8]>]) -> Simulate;
pub fn sources(text: &[u8]) -> Box<[Source]>;
/// apt's name for a list: the URI less `http://`, `dists/<suite>/`, then `path`, each `/` a `_`.
pub fn list_name(source: &Source, path: &[u8]) -> Box<[u8]>;
/// The lines 41.1's recordings show for `outcome`, filled from the run.
pub fn print(simulate: &Simulate, sources: &[Source], outcome: &Outcome) -> Printed;

// testing/hearth-fake-mmdebstrap/src/process.rs (46.1)
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Faults { pub stall_after: Option<u64> }
/// Working: call again; Waiting: a pipe is full; Exited: done.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Progress { Working, Waiting, Exited }
#[derive(Debug)] pub struct Mmdebstrap { /* child, printed, sent, faults */ }
impl Mmdebstrap { pub fn start(machine: &mut hearth_fake_machine::Machine, child: hearth_fake_machine::linux::Pid, faults: Faults) -> Mmdebstrap; }
impl Mmdebstrap { pub fn step(&mut self, machine: &mut hearth_fake_machine::Machine) -> Progress; }

// tests/conformance/src/fake_mmdebstrap.rs (46.2)
use hearth_fake_machine::hearth::mirror::Mirror as Served; // 27.2's, the fake's
use hearth_test_mirror::Mirror as Fetched; // 08.1's, the files in the cache
/// The machine of the live and replayed runs.
pub const CONFORMANCE: hearth_fake_machine::linux::Sizes = hearth_fake_machine::linux::Sizes { files: 64, bytes: 256 << 20, processes: 8, pipe: 65_536, ledger: 4_096 };
/// The pinned snapshot as the fake's mirror.
pub fn mirror(scenario: &Resolve) -> Served;
/// The gate's mirror: `testing/mirror/universe`, at the committed pin's timestamp.
pub fn committed() -> Served;
/// The cut `testing/mirror/universe` holds, from the whole index and the seven cases.
pub fn cut(mirror: &Fetched, pin: &Pin) -> Box<[u8]>;
/// What a run spawns: the argv, env and dir of a live vector or of a recording. `argv` is whole, `PROGRAM` first,
/// as every recording holds it: the live side puts `PROGRAM` before `Invocation::args`, and `run` splits it into
/// `Spawn::program` and `Spawn::args`. `dir` is a recording's, absolute (`/scratch`): `run` spawns it without its
/// leading `/`, under the root handle, as an absolute one is `Refusal::Escape` (`linux/files.rs:118`).
pub struct Spawned<'a> { pub argv: &'a [Vec<u8>], pub env: &'a [Vec<u8>], pub dir: &'a [u8] }
/// One case on the fake, its mirror moved in and back, masked as 41.1 masks the real run.
pub fn run(mirror: &mut Option<Served>, case: &Case, spawned: &Spawned<'_>) -> Recording;
/// What an allowed difference covers: the lists' sizes; stanza fields outside `KEPT`; a case's error lines.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum What { ListSizes, StanzaFields, ErrorLines }
/// One allowed difference (testing.md, 5.3): its case (none: every case), what, why.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Allowed { pub case: Option<&'static str>, pub what: What, pub reason: &'static str }
pub const ALLOWED: &[Allowed] = &[/* 46.2's two rows */];
pub const PORT: u16 = 3142;
/// The first difference past `ALLOWED`, named; none when the fake left what the real run recorded.
pub fn differ(case: &Case, real: &Recording, fake: &Recording, stanzas: Option<&[u8]>) -> Option<String>;

// tests/conformance/src/resolve.rs (hearth 41.1's; 46.2 adds)
impl Resolve { /// The recordings' directory, `<mmdebstrap>-apt<apt>` (x1: `1.5.7-apt3.3.3`; hearth 41).
    pub fn version(&self) -> &str; }
/// A case's inputs from the pin, as `Resolve::simulate` made them; no tool or mirror needed.
pub fn inputs(pin: &Pin, case: &Case) -> Simulate;
```

## Questions to raise when reached

None.

## Done when

1. In a worktree at `origin/main`, `heavy cargo nextest run -p
   hearth-fake-machine -p hearth-fake-mmdebstrap` passes, its `Summary`
   naming at least 74 tests (hearth 26's 39 and hearth 27's 27 there,
   this session's 8).
2. There, 46.2's Proof command passes, once; and `XDG_CACHE_HOME=$(mktemp -d) heavy cargo nextest run -p hearth-conformance-world -E 'test(=the_fake_prints_each_mmdebstrap_recording_as_recorded)'` passes
   with no fetched mirror.
3. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-46\.[1-2]'` prints 2.
