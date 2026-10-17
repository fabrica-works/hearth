# Session 07: `hearth-protocol-mmdebstrap`

The machine of render's resolution (render.md, 5.2 to 5.5): simulate's
argument vector, environment and `sources.list`, encoded from the
packages part's sources; mmdebstrap's two pipes read into `Inst` triples
and a failure tail with its cause. Its world, `tests/mmdebstrap`
(`hearth-mmdebstrap-world`), starts on generated output cut at random.
07.1 is the vector alone, since hearth 41 records with it. hearth 38
takes the rest of the crate: apt's `Packages` stanzas and lists, and the
world's end on hearth 41's recordings of real mmdebstrap. Nothing calls
the crate until hearth 24 routes it; hearth 46's fake answers the vector
it writes.

## Read first

- hearth's render.md: 2.3, 5 (5.1 to 5.5), 7.1, 7.2 (kinds `resolution`
  and `tool`), 8.1, 8.2; format.md, 5.3 (`debian`); store.md, 6.1; build.md,
  5.4 (`--include`'s split) and 10.1 (os's list, the line bound);
  testing.md, 2.4, 4.3, 5.2 (the resolve row), 6.2; client/README.md, 3.5.
- `findings/os-lists.md` (build.md 10.1's lists, which 5.5 holds to).
- This plan's `decisions.md`, 9, 10, 12; `findings/skein-testing.md`, 3, 7.
- skein's lib.md, 7 (streams); `programming-model.md`: 4, 4.3, 6.3, 7, 8,
  10 (by number only); `testing-strategy.md`: 2.1, 2.4, 4.1, 8 (by number
  only).

## What the design asks

- **The vector** (5.2): `mmdebstrap --verbose --simulate --mode=chrootless`
  ... `forky <scratch>/root <scratch>/sources.list`, an optional
  `--aptopt=Acquire::http::Proxy "<proxy>"`; the line `deb
  http://snapshot.debian.org/archive/debian/20261001T030405Z/ forky main
  contrib non-free-firmware`, plus `<suite>-updates` and `<suite>-security`
  with both pockets on; `/usr/bin/mmdebstrap`, "spawned by its path"; the
  environment "given whole" (`LC_ALL=C`, `PATH=/usr/sbin:/usr/bin:/sbin:/bin`,
  `TMPDIR` the scratch); the proxy "none in M1", the local mirror in the
  tests (testing.md, 6.2); no `--keyring`: apt checks each `InRelease`
  with the host's keys (5.2).
- **`Inst` lines** (5.3): "`Inst <name> [<old>] (<version> <origins>
  [<architecture>])` gives the name, the version and the architecture of
  each package apt would install"; they come on standard error under
  `--verbose` and both pipes are read; each triple comes twice, at most
  twice 8,192 lines are taken; output the tool should not give is a
  `tool` error, operational.
- **Tails** (5.5): at most 20 lines, each cut at 160 bytes with no
  marker, by build.md 10.1's lists and 5.5's additions (Choices); the
  cause by 5.5's table; "60 s without output ends a run".
  `made.costs.fetched` is apt's `Fetched <n> <unit> in <time>` (5.4).

## Choices this plan makes

- **Modules:** `limits`, `vector` (07.1), `output` (07.2), `tail` (07.3);
  hearth 38 adds `packages` and `lists`; `skein-lib` alone (render.md, 8.1).
- **Both readers are `lib::stream` sides above,** demanding
  `Read::Scan { until: Delimiter::LF, max }`, with no timer and no room.
  `Limits::idle` carries 5.5's 60 s for the owner (hearth 24), which arms it
  on each delivery; the 10-minute deadline is render's.
- **Names are refused, never quoted:** a `Name`, Debian Policy 5.6.1's
  form, at most 256 bytes (store.md, 6.1; format.md 5.3).
- **A malformed `Inst` line or a broken pipe is a `tool` error,
  operational** (render.md 5.3, 7.2). **A malformed `Inst` line fails the
  pipe at its end:** the reader stops
  sending triples and keeps reading, so the tail is whole and the pipe
  drains; a line starting `Inst ` is never skipped.
- **The tail is build.md 10.1's lists** (`findings/os-lists.md`), deny
  before allow, with indented continuation lines after a kept
  `dpkg: error` or `E: ` line; tab and CR become spaces; each line is cut
  at 160 bytes with no marker; the last 20 are kept (render.md 5.5). 5.5's
  additions: `The following packages have unmet dependencies:` and
  `However the following packages replace it:` are kept, with their
  continuation lines, and mmdebstrap's framing is denied. The lists and
  additions are the contract, held to recordings (hearth 41), with no
  comparison with os's source. The cause is taken over every kept line,
  not only the last 20, and over each line **before** the cut (up to
  `line`): the cut only shortens the kept text.
- **No marker within a run goes up** (5.5): hearth 24 reports a resolution's start and end.

## Rules that bind it

- **No hand-written recording, nothing beyond loopback**
  (code-organization.md, 6): the world's recordings are hearth 38.2's.
- **The first fuzzy test** (README, Rules, "hearth's first landings"): if
  `.config/nextest.toml` still selects `none()` when 07.2 lands, 07.2
  replaces both profiles' filters with the two lines its comments give;
  after a rebase onto a `main` that switched them, it drops its switch.
- **Limits from the shipped profile; no lint exception in tests**
  (code-organization.md, 4, 6): tests take `limits::SHIPPED`, and a test
  needing another value sets it beside the default with its reason. Step
  tests use no `Vec` and no closure-taking combinator (skein's
  `#![expect(clippy::disallowed_types)]` in its tests is not copied).

## Increments

### 07.1 Simulate's vector, environment and `sources.list`
Scope: `vector::simulate`, 5.2's vector, environment and `sources.list`
from a resolution's inputs (5.1; render.md 4.2's packages part), the
scratch directory and an optional proxy; `limits.rs`.
Needs: nothing.
Files: new `crates/hearth-protocol-mmdebstrap/`: `Cargo.toml` (`skein-lib`
only, `[lints] workspace = true`), `src/lib.rs` (`#![cfg_attr(not(test),
no_std)]`, `#![forbid(unsafe_code)]`, `MaxOut` as skein-http's),
`src/limits.rs`, `src/vector.rs`, `src/tests.rs`, `src/tests/fixtures.rs`,
`src/tests/vector.rs`; shared: root `Cargo.toml` (the `crates/*` glob, if
absent, and `skein-lib` in `[workspace.dependencies]` if absent), `Cargo.lock`.
Shape: the sketch's `limits` and `vector`. `lines` refuses components that
are not a set in the fixed order with `Main`; `sources_list` writes `deb
<uri> <suite> <components>\n` a line; `simulate` refuses a request not in
strictly increasing byte order or past `request`, and a scratch not
absolute, with a NUL, ending in `/`, or whose `/sources.list` passes `path`;
then writes the reference test's arguments, a proxy's `--aptopt` after the
two others, and `LC_ALL=C`, `PATH=/usr/sbin:/usr/bin:/sbin:/bin`,
`TMPDIR=<scratch>`.
Test shape: `src/tests/vector.rs` on `src/tests/fixtures.rs` (new: `name`;
`forky_main()`: forky, amd64, `[Main]`, archive `20261001T030405Z`, no
pockets; `simulate(&[&[u8]])`), each under 20 ms:
`the_vector_is_render_s_for_a_forky_main_request`,
`the_environment_is_given_whole_with_tmpdir_the_scratch`,
`both_pockets_add_updates_and_security_lines_after_the_archive_s`,
`a_proxy_adds_its_apt_option_before_the_skip`,
`install_recommends_is_passed_as_apt_s_option`,
`a_request_past_one_argument_is_split_into_several_includes` (`include: 32`
beside the default: three names split without 64 KiB of them),
`an_empty_request_passes_no_include`,
`a_request_out_of_order_or_components_without_main_are_refused`.
```rust
#[test]
fn the_vector_is_render_s_for_a_forky_main_request() {
    let simulate = fixtures::simulate(&[b"apt", b"mmdebstrap"]);
    let made = vector::simulate(&simulate, b"/store/staging/k", None, &limits::SHIPPED).unwrap();
    let want: [&[u8]; 12] = [b"--verbose", b"--simulate", b"--mode=chrootless", b"--variant=apt", b"--architectures=amd64",
        b"--include=apt,mmdebstrap", b"--aptopt=Acquire::Check-Valid-Until \"false\"", b"--aptopt=APT::Install-Recommends \"false\"",
        b"--skip=cleanup/apt/lists", b"forky", b"/store/staging/k/root", b"/store/staging/k/sources.list"];
    assert_eq!(made.args.len(), want.len());
    for (arg, w) in made.args.iter().zip(want) { assert_eq!(&**arg, w); }
    assert_eq!(&*made.sources_list, b"deb http://snapshot.debian.org/archive/debian/20261001T030405Z/ forky main\n");
}
```
Proof: unit, `the_vector_is_render_s_for_a_forky_main_request`: 5.2's vector
and line, byte for byte (bound 0.02 s). Nothing above calls it yet:
hearth 41.1 runs real mmdebstrap with it (conformance, outside the
gate), and hearth 24 routes it.
Don't: spawn, open or write anything; quote a name (refuse it); search a
`PATH`; pass a `--keyring`.
Size: at most 230 lines.
Removes: nothing.

### 07.2 A pipe's reader and `Inst` lines; the world
Scope: `output::Reader`, one per pipe: lines of at most `line` bytes (the
rest cut, build.md 10.1), each `Inst` line decoded into its triple (5.3);
the world crate on generated output.
Needs: nothing beyond 07.1.
Files: `src/output.rs`, `src/limits.rs`, `src/lib.rs`, `src/tests.rs`,
`src/tests/{fixtures,output}.rs`;
new `tests/mmdebstrap/`: `Cargo.toml` (its own lints table, as skein's
`tests/json/Cargo.toml:17-33`; `skein-world`, and `skein-heap` for tests),
`src/{lib, fixtures, generate, reference, world}.rs`, `tests/mmdebstrap.rs`,
`tests/fuzzy_mmdebstrap.rs`, `tests/memory.rs`; shared: root `Cargo.toml`
(the `tests/*` glob, if absent), `Cargo.lock`, `.config/nextest.toml`
(Rules), and `tests/clippy.toml` (skein's, whole) if no world landed it.
Shape: the sketch's `output`. A delivery ending in LF, or the first of a
line past `line`, completes it (later pieces are dropped to the LF; the
cut line is one `Inst` candidate, as the last test pins). A
complete line starting `Inst `: the name to the next space; an optional
`[<installed>] `, dropped; `(`, the version to the next space, the origins
(at least one byte, to the last ` [`, never parsed), the architecture,
`])`, after which the rest of the line is ignored (apt's ` [<names>]` or
` []`). A line that fails, or the `insts + 1`-th, sets the held error;
`End` then gives `Failed(error)`, else `Ended`. Each triple is sent as it
comes, twice over: equal triples are one package, the owner's to merge
(hearth 24). A complete line `Fetched <n> <unit> in <time>` gives
`Seen::Fetched(bytes)`: `<n>` digits and an optional `.` and digits, the
unit `B`, `kB`, `MB`, `GB` or `TB` (1,000 a step), the bytes rounded
down; the last one counts; a line not of that form gives none.
Test shape: `src/tests/output.rs` (fixtures gain `opened()`: the env, a
reader opened, its first demand taken), each under 20 ms:
`an_inst_line_gives_its_name_version_and_architecture`,
`an_inst_line_with_several_origins_gives_the_same_triple` (10's line),
`an_installed_version_in_brackets_is_dropped`,
`an_inst_line_ignores_what_follows_its_closing_bracket` (` [libc6 :amd64]`),
`conf_remv_and_other_lines_give_no_inst`,
`a_malformed_inst_line_fails_the_pipe_once_it_ends`,
`inst_lines_past_the_limit_fail_the_pipe` (`insts: 2` beside the default:
three lines, not 16,385), `a_line_past_its_bound_is_cut_and_never_read_as_two`,
`a_fetched_line_gives_its_bytes_in_apt_s_units` (`Fetched 836 B`, `Fetched
1234 kB`, `Fetched 12.3 MB`: 836, 1,234,000, 12,300,000) and
`a_line_that_is_not_a_fetched_line_gives_no_bytes`.
```rust
#[test]
fn an_inst_line_gives_its_name_version_and_architecture() {
    let (env, mut reader, mut above, mut below) = fixtures::opened();
    let line = bytes::copy_of(b"Inst libc6 (2.41-12 Debian:testing [amd64])\n");
    output::up(&mut reader, &env, Up::Bytes(line), &mut above, &mut below);
    let inst = Inst { name: fixtures::name(b"libc6"), version: bytes::copy_of(b"2.41-12"), architecture: fixtures::name(b"amd64") };
    assert_eq!(above.pop(), Some(Seen::Inst(inst)));
    assert_eq!(below.pop(), Some(Down::Demand { read: Read::Scan { until: Delimiter::LF, max: 4_096 }, room: 0 }));
}
```
The world plays the side below with `skein_world::stream::Wire`
(testing/skein-world/src/stream.rs:16: fragments of 1 to 30 bytes), `eof`
once the bytes are written, and `Up::Failed(Fault::Reset)` in place of an
answer at a seeded point: `world::read_output(bytes, seed, &LIMITS, End)
-> Run`, `End::{Whole, Cut(usize), Break(usize)}`; `reference::output(bytes,
End, &LIMITS) -> Decoded`, plain std; `generate::output` (`I:`, `Get:`,
`Conf` and `Inst` lines, one to three origins, an installed version at
times, lines past `line`) and `generate::mutate` (a line dropped, cut or
doubled, a byte changed); `fixtures::LIMITS` is `limits::SHIPPED`.
Focused, each under 0.5 s:
`generated_output_decodes_as_the_reference_whatever_the_cuts`,
`output_cut_short_reads_as_the_reference_reads_its_prefix`,
`a_pipe_that_breaks_fails_the_reader_with_its_fault`; fuzzy, about 1.5 s:
`generated_and_mutated_output_reads_as_the_reference_reads_it`; memory:
`an_output_reader_at_its_limits_holds_no_more_than_its_worst_case`.
```rust
#[test]
fn generated_output_decodes_as_the_reference_whatever_the_cuts() {
    for seed in 0..32 {
        let mut rng = Rng::new(0x0702_0000 + seed);
        let output = generate::output(&mut rng, &LIMITS);
        let run = world::read_output(&output, seed, &LIMITS, End::Whole);
        assert_eq!(run.decoded, reference::output(&output, End::Whole, &LIMITS), "seed {seed}");
    }
}
```
The fuzzy test (`SEEDS = 1_000`; `PINNED: &[u64] = &[]`, chained first)
mutates a seed's output when `rng.chance(500)`, ends it by `world::end`,
and compares as above. The memory test (`#[global_allocator] static HEAP:
Counting`) holds `world::read_output_metered(&fixtures::at_the_limits(..))`
to `output::worst_case`, each entry point a `skein_heap::Meter` step, as
skein's `tests/json/tests/memory.rs:134`.
Proof: world, `generated_output_decodes_as_the_reference_whatever_the_cuts`:
the triples of output delivered in fragments of 1 to 30 bytes are the
reference's (bound 0.5 s). No caller until hearth 24.
Don't: arm a timer; read `I:` stage lines (build.md 10.1's, M2); skip a
malformed `Inst` line.
Size: at most 240 lines.
Removes: nothing.

### 07.3 The failure tail and its cause
Scope: `tail`: each reader keeps its pipe's tail, os's list chosen and cut
(5.5); `failure`, stderr's tail unless it is empty, with its cause and
class (5.5; render.md 7.1).
Needs: nothing beyond 07.2.
Files: `src/tail.rs`, `src/output.rs` (each complete line to `tail::admit`),
`src/limits.rs`, `src/lib.rs`, `src/tests/output.rs`; world:
`src/{generate,reference,world}.rs` (error, deny and continuation lines),
`tests/{mmdebstrap,fuzzy_mmdebstrap,memory}.rs`.
Shape: the sketch's `tail`. Deny, then allow: `findings/os-lists.md`'s
lists whole, in their order, then 5.5's additions: the unmet-dependencies
and `However the following packages replace it:` lines allowed, with
their continuation lines, and `E: setup failed: `, `E: apt-get update …
failed: process exited with` and `W: hooklistener errored out` denied.
Causes, by 5.5's table: `Resolve` (`Temporary failure resolving`, `Could
not resolve`) and `Fetch` (any other `Err:` or `Failed to fetch`: `Could
not connect`, `Unable to connect`, a timeout, a hash or size mismatch, a
5xx), `Space` (`No space left on device`), all operational; `Absent` (an
`Err:` or `Failed to fetch` answered `404  Not Found` or `NOT FOUND`, or
`does not have a Release file`), `Unknown` (`Unable to locate package
<name>` to the line's end, or `Package '<name>' has no installation
candidate`, a virtual package alone), `Unsatisfiable` (`Unable to correct
problems`, or the unmet-dependencies line, named by the first
continuation's ` <name> :`), configuration; `Other`, configuration. The
run's: the first operational cause of any kept line, else the first
configuration one, else `Other`. `Cause::kind` is `Resolution` for
`Absent`, `Unknown` and `Unsatisfiable`, `Tool` for the rest; naming
`Absent`'s source line is hearth 24's.
Test shape: in `src/tests/output.rs`, each under 20 ms:
`a_failed_fetch_keeps_its_err_and_e_lines_and_is_operational`,
`mmdebstrap_s_own_framing_is_never_in_the_tail` (`E: setup failed: `,
`E: apt-get update … failed: process exited with`, `W: hooklistener
errored out`), `the_unmet_dependencies_and_replace_blocks_are_kept_whole`,
`a_404_or_a_missing_release_file_is_absent_and_configuration`,
`a_5xx_a_mismatch_or_a_timeout_is_a_fetch_and_operational`,
`no_space_left_is_operational`,
`an_indented_line_after_an_e_line_stays_with_it`,
`only_the_last_twenty_lines_are_kept_each_cut_at_160_bytes`,
`unable_to_locate_package_is_configuration_naming_the_package`,
`no_installation_candidate_is_configuration_naming_the_package`,
`unmet_dependencies_name_the_first_package_apt_lists`,
`an_operational_cause_anywhere_outranks_a_configuration_one`,
`stderr_s_tail_is_taken_unless_it_is_empty`,
`a_200_byte_package_name_is_named_whole_though_its_line_is_cut`. World, focused:
`generated_failures_keep_the_reference_s_tail_and_cause`; the sweep
compares tails and causes from here on.
```rust
#[test]
fn a_failed_fetch_keeps_its_err_and_e_lines_and_is_operational() {
    let mut tail = Tail::new(&limits::SHIPPED);
    let lines: [&[u8]; 4] = [b"I: running apt-get update...", b"Err:1 http://snapshot.debian.org/archive/debian/20261001T030405Z forky InRelease",
        b"E: Failed to fetch http://snapshot.debian.org/archive/debian/20261001T030405Z/dists/forky/InRelease  Could not connect to snapshot.debian.org:80", b"E: mmdebstrap failed to run"];
    for line in lines { tail::admit(&mut tail, line, &limits::SHIPPED); }
    assert_eq!(tail.lines().len(), 2);
    for (line, want) in tail.lines().iter().zip([lines[1], lines[2]]) { assert_eq!(&**line, want); }
    assert_eq!((tail.cause(), tail.cause().class()), (&Cause::Fetch, Class::Operational));
}
```
Proof: world, `generated_failures_keep_the_reference_s_tail_and_cause`
(bound 0.5 s); no caller until hearth 24, which classes timeouts and spawn
failures itself (`tool`, operational: render.md 7.2).
Don't: a pattern beyond these; a cause for a timeout or a spawn.
Size: at most 215 lines.
Removes: nothing.
## Interface sketches

The crate is the machine's alone: `hearth-protocol-builder` (hearth 24)
translates render's `Resolve` into `Simulate`, and `Seen` and `Failure`
(and hearth 38's `Found` and `Index`) into render's events. Derives
(`Clone`, `PartialEq`, `Eq`, `Debug`; `Copy` where it holds) are elided; an
enum's doc line names what each variant is.

```rust
// crates/hearth-protocol-mmdebstrap/src/limits.rs (07.1; 07.2, 07.3 add theirs; hearth 38.1: `stanza`, `chunk`, `filename`)
/// The machine's configured limits (programming-model.md, 7); `SHIPPED`, the design's defaults (code-organization.md, 2).
pub struct Limits {
    pub path: u32, pub name: u32, pub request: u32, pub include: u32,          // 4,096 (2.3); 256, 8,192 (store.md 6.1); 65,536 (build.md 5.4)
    pub line: u32, pub insts: u32, pub tail_lines: u32, pub tail_line: u32,    // 4,096 (build.md 10.1); 16,384 (5.3: twice store.md 6.1's 8,192); 20 and 160 (5.5)
    pub stanza: u32, pub chunk: u32, pub filename: u32,                        // 1 MiB (5.3); 4,096; 4,096 (store.md 6.1)
    pub idle: skein_lib::Duration,                                              // 60 s (5.5), the owner's to arm
}
/// The design's defaults, the shipped profile's (module level, as every crate's).
pub const SHIPPED: Limits = Limits {
    path: 4_096, name: 256, request: 8_192, include: 65_536, line: 4_096, insts: 16_384, tail_lines: 20,
    tail_line: 160, stanza: 1_048_576, chunk: 4_096, filename: 4_096, idle: skein_lib::Duration::from_secs(60),
};
// crates/hearth-protocol-mmdebstrap/src/lib.rs (07.1)
/// The most an entry point emits into each of its two queues, which its caller reserves first.
pub struct MaxOut { pub above: u32, pub below: u32 }
```

```rust
// crates/hearth-protocol-mmdebstrap/src/vector.rs (07.1)
/// A package's, suite's or architecture's name: Policy 5.6.1's form, at most `Limits::name`.
pub struct Name(Box<[u8]>);
impl Name { pub fn new(bytes: &[u8], limits: &Limits) -> Option<Name>; pub fn as_bytes(&self) -> &[u8]; }
/// A snapshot.debian.org timestamp, `YYYYMMDDTHHMMSSZ` (format.md, 5.3).
pub struct Timestamp([u8; 16]);
impl Timestamp { pub fn new(bytes: &[u8]) -> Option<Timestamp>; }
/// A component, in format.md 5.3's order: `main` (always on), `contrib`, `non-free`, `non-free-firmware`.
pub enum Component { Main, Contrib, NonFree, NonFreeFirmware }
/// The packages part's sources (render.md, 4.2), field for field; `security` none with its pocket off.
pub struct Sources { pub suite: Name, pub architecture: Name, pub components: Box<[Component]>, pub archive: Timestamp, pub updates: bool, pub security: Option<Timestamp> }
/// The variant (4.2): `apt`, metal's and a VM's; a container's is a placeholder until M5, asking no resolution (5.7).
pub enum Variant { Apt }
/// The packages part's apt options.
pub struct Apt { pub install_recommends: bool }
/// One resolution's inputs, the key's values (5.1), as render's `Resolve` holds them.
pub struct Simulate { pub sources: Sources, pub request: Box<[Name]>, pub variant: Variant, pub apt: Apt }
/// The builder's proxy: `http://`, then printable bytes but `"`, `;`, `\`, `{`, `}`; at most `Limits::path`.
pub struct Proxy(Box<[u8]>);
impl Proxy { pub fn new(bytes: &[u8], limits: &Limits) -> Option<Proxy>; }
/// One source line, as the record names it (5.4): `ARCHIVE` or `SECURITY`, its timestamp and `/`.
pub struct Line { pub uri: Box<[u8]>, pub suite: Box<[u8]>, pub components: Box<[Component]> }
/// What the protocol layer writes, then spawns: `args` after argv[0] (skein's `kernel::Spawn::args`), `env` whole.
pub struct Invocation { pub args: Box<[Box<[u8]>]>, pub env: Box<[Box<[u8]>]>, pub sources_list: Box<[u8]> }
/// Why no vector was made: `Components`, `Request` or `Scratch` out of 07.1's forms.
pub enum VectorError { Components, Request, Scratch }
/// mmdebstrap, spawned by its path (render.md, 5.2).
pub const PROGRAM: &[u8] = b"/usr/bin/mmdebstrap";
pub const ARCHIVE: &[u8] = b"http://snapshot.debian.org/archive/debian/";
pub const SECURITY: &[u8] = b"http://snapshot.debian.org/archive/debian-security/";
pub fn lines(sources: &Sources) -> Result<Box<[Line]>, VectorError>;
pub fn sources_list(lines: &[Line]) -> Box<[u8]>;
pub fn simulate(simulate: &Simulate, scratch: &[u8], proxy: Option<&Proxy>, limits: &Limits) -> Result<Invocation, VectorError>;
```
```rust
// crates/hearth-protocol-mmdebstrap/src/output.rs (07.2; `tail` from 07.3)
/// One package apt would install: an `Inst` line's triple (5.3).
pub struct Inst { pub name: Name, pub version: Box<[u8]>, pub architecture: Name }
/// Why a pipe failed, held to its end: `Inst` (a line starting `Inst ` out of 5.3's form),
/// `TooMany` (past `Limits::insts`: kind `resolution`, store.md 6.1), `Stream` (the pipe broke).
pub enum OutputError { Inst, TooMany, Stream(Fault) }
/// To the owner, at most one an entry point: a decoded `Inst`; apt's `Fetched` bytes (5.4); `Ended` or `Failed`, the terminal.
pub enum Seen { Inst(Inst), Fetched(u64), Ended, Failed(OutputError) }
/// One pipe's reader: the line in hand, the count, the held error, the tail.
pub struct Reader { /* private */ }
impl Reader {
    pub fn new(limits: &Limits) -> Reader;
    /// A scan is outstanding, so a delivery is output for the owner's 60 s (5.5).
    pub fn waiting(&self) -> bool;
    pub fn tail(&self) -> &Tail;
}
pub const UP_MAX_OUT: MaxOut = MaxOut { above: 1, below: 1 };
pub fn largest_demand(limits: &Limits) -> u32;
pub fn worst_case(limits: &Limits) -> Option<u64>;
pub fn open(reader: &mut Reader, env: &Env<Limits>, below: &mut Queue<Down>);
pub fn up(reader: &mut Reader, env: &Env<Limits>, event: Up, above: &mut Queue<Seen>, below: &mut Queue<Down>);
pub fn close(reader: &mut Reader, below: &mut Queue<Down>);
pub fn inst(line: &[u8], limits: &Limits) -> Option<Inst>;
```

```rust
// crates/hearth-protocol-mmdebstrap/src/tail.rs (07.3)
/// Whether the same revision may succeed later (render.md, 7.1).
pub enum Class { Operational, Configuration }
/// What the patterns found (5.5's table): `Fetch`, `Resolve` (a name), `Space`, operational, kind `tool`; `Absent`
/// (the archive lacks the suite: 404, no Release file), `Unknown` (`Unable to locate package`, no installation
/// candidate) and `Unsatisfiable`, configuration, kind `resolution`, the last two naming the package; `Other`, configuration, `tool`.
pub enum Cause { Fetch, Resolve, Space, Absent, Unknown(Name), Unsatisfiable(Option<Name>), Other }
/// render.md 7.2's kind of the error a cause gives.
pub enum Kind { Resolution, Tool }
impl Cause { pub fn class(&self) -> Class; pub fn kind(&self) -> Kind; }
/// A pipe's last kept lines, each cut, and their cause.
pub struct Tail { /* private */ }
impl Tail { pub fn new(limits: &Limits) -> Tail; pub fn lines(&self) -> &Queue<Box<[u8]>>; pub fn cause(&self) -> &Cause; }
/// A failed run, for render's `Failed { class, tail }` (8.2).
pub struct Failure { pub cause: Cause, pub tail: Box<[Box<[u8]>]> }
pub fn admit(tail: &mut Tail, line: &[u8], limits: &Limits);
pub fn failure(stderr: &Reader, stdout: &Reader) -> Failure;
```
## Questions to raise when reached

None.

## Done when

- `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-07\.[1-3]'` prints 3.
- In a worktree of hearth at `origin/main`, once each: `heavy cargo nextest
  run -p hearth-protocol-mmdebstrap -p hearth-mmdebstrap-world` and `heavy
  cargo nextest run -p hearth-mmdebstrap-world --profile fuzzy` print a
  `Summary` line with no test failed.
- `git -C ~/src/rust/hearth show origin/main:crates/hearth-protocol-mmdebstrap/Cargo.toml
  | grep -E '^(skein|hearth)-'` prints one line, which names `skein-lib`.
