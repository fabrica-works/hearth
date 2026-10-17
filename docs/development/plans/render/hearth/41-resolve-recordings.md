# Session 41: resolve's real side: mmdebstrap through the mirror, and its recordings

The real side of conformance's resolve scenario (testing.md, 5.2, 4.5), on
hearth 08's kit. 41.1 runs real mmdebstrap with hearth 07.1's vector
through the harness's proxy for seven cases and test-boot split, and
records each under `testing/recordings/mmdebstrap/<version>/`, with the
`Packages` stanzas of test-min's closure cut from the list apt left.
Needs hearth 08 (08.2's fetched mirror and test repository, 08.3's
`Proxy`), hearth 34.1 (the conformance world, its profile, `Run`,
`hearth-recordings`) and hearth 07.1 (`vector::simulate`). hearth 38.2
replays the recordings; hearth 46 runs the same cases on its fake. Nothing
in the gate reads the cache or reaches past loopback.

## Read first

- hearth's testing.md: 2 (table, 2.4, 2.8, 2.9), 4.3, 4.5, 5.1 to 5.5, 6.2,
  9.2, 11; render.md: 5.1 to 5.5, 10 (its first three bullets); format.md:
  5.3 (`debian`); client/README.md, 3.5; `.config/nextest.toml` as hearth
  34.1 leaves it.
- This plan's `hearth/34-conformance.md`, 34.1 (`hearth-recordings`, `run.rs`'s
  `Run`, `Scenario`, the profile); `hearth/08-mirror.md` (its sketches and
  Choices); `hearth/07-mmdebstrap.md` (07.1's sketch); README, "Live
  services"; decisions.md, 4, 12, 14, 16; `findings/skein-testing.md`, 6, 7.
- skein's `testing/skein-scratch/src/lib.rs:23-55` (`Scratch::new`, `path`).

## What the design asks

- **Resolve** (5.2): the real side is "`mmdebstrap --simulate
  --mode=chrootless` through the local mirror, as its proxy"; compared:
  "the closure: names, architectures, versions; the `Packages` stanzas it
  left"; 5.3: "Left out: times, ..., logs that carry a timestamp".
- **Recordings** (4.5): "each with its argument vector, its environment (no
  terminal), its stdout, its stderr and its exit"; "Captured, never written
  by hand"; "A new version is re-recorded". 5.5: in M1 the profile needs
  "git, mmdebstrap and the mirror's indexes", and "refuses to start,
  naming what is missing". render.md 10: what `--skip=cleanup/apt/lists`
  leaves "depends on a recording of each".
- **testing.md 2.10:** real tiers "refuse to start, rather than skip".
- **The vector** (render.md, 5.2): `/usr/bin/mmdebstrap`, "spawned by
  its path"; the environment "given whole"; the proxy as
  `--aptopt=Acquire::http::Proxy`; apt checks each `InRelease` with the
  host's keys (x1's `/etc/apt/trusted.gpg.d` holds trixie's), no
  `--keyring`: the mirror's digest is the pinned `InRelease`'s SHA-256, and
  test-min is the `apt` variant with no package (render.md 5.2; testing.md
  6.2).

## Choices this plan makes

- **Resolve's seven cases:** `test-min`; `test-boot`; `unknown-package`
  (test-min's inputs, request `hearth-absent`); `no-candidate` (request
  `mail-transport-agent` alone, virtual: "has no installation candidate");
  `unsatisfiable` (request `exim4-daemon-light` and `postfix`, each
  providing and conflicting with `mail-transport-agent`);
  `unreachable-mirror` (test-min's, the proxy a loopback port nothing
  listens on); `absent-snapshot` (test-min's, the timestamp
  `20000101T000000Z`, older than any snapshot.debian.org holds: the
  proxy's `InRelease` 404: render.md 5.5's `resolution`, configuration,
  since a timestamp's archive never changes). x1's local forky index of `20260928T022309Z` holds test-boot's
  ten names, both MTAs with those fields, twelve providers of
  `mail-transport-agent` and no package of that name, and no
  `hearth-absent`. Then test-boot split: under `Limits { include: 32, ..
  }`, several `--include`s, which mmdebstrap joins (build.md, 5.4),
  compared with test-boot's recording, recording nothing.
- **Recorded and compared:** hearth 34.1's files; the scratch masked as
  34.1 masks it, and the proxy's port as `PORT` in `127.0.0.1:<port>` and
  apt's `127.0.0.1 <port>`. Two files join the layout: `lists`, what apt
  left in `<scratch>/root/var/lib/apt/lists/` (`file <size> <name>` or
  `directory <name>`, by name), every case's; `stanzas`, each stanza of
  apt's own `Packages` list whose `Package`, `Version` and `Architecture`
  an `Inst` line names, whole, in the list's order, test-min's. That list
  must be the checked index byte for byte, so its size and SHA-256 are
  `InRelease`'s entry (render.md, 5.3, 5.4). Without `HEARTH_RECORD=1`,
  a run is compared with its recording by its exit, each pipe's lines
  beginning `Inst `, `E: `, `Err:` or `W: ` (apt 3.0.3 under mmdebstrap
  prints `E:` and `W:`, not `Error:`, render.md 5.5), `lists` and
  `stanzas`; the rest carries times (5.3).
- **`Inst` lines are read by the world, not by hearth 07.2's reader** (the
  name, the version after `(`, the architecture in the last `[…]`), into no
  type of its own: a real line 07.2 misreads cannot block the recordings
  hearth 38.2 corrects it by.
- **The proxy is the test's configuration** (client/README.md, 3.5): the
  world passes `Proxy::url()` to 07.1's `simulate`; no builder runs here.

## Rules that bind it

- **Live services** (README) as hearth 08's Rules say: 41.1 runs no fetch.
  A run it needs (a bump, a missing file) is named in the report and is a
  `blocked.md`; this commit carries no `Fetch-*` trailer.
- **No hand-written recording; no network beyond loopback**
  (code-organization.md, 6): each file under `testing/recordings/mmdebstrap/`
  comes from `HEARTH_RECORD=1`; nothing commits a fetched file.
- **A request apt cannot meet at the pin** (a name of test-boot's not
  locatable, a key apt refuses): 41.1 changes no request and no vector; it
  is named with apt's lines in `blocked.md`, and 41.1 waits.
- **Profiles outside the gate** (README): `conformance_resolve` is selected
  by 34.1's `binary(/^conformance_/)` and runs as `heavy cargo nextest run
  --workspace --profile conformance`, never in the gate;
  `.config/nextest.toml` does not change.

## Increments

### 41.1 Resolve's real side, and mmdebstrap's recordings
Scope: testing.md 5.2's resolve scenario, real side, the seven cases of
Choices and test-boot split; their recordings (4.5), with `lists` and
test-min's `stanzas`.
Needs: hearth 34.1 (the conformance world, its profile, `Run`,
`hearth-recordings`), hearth 07.1 (`vector::simulate`); hearth 08.2 (the
pin, the fetched mirror, the test repository), hearth 08.3 (`Proxy`).
Files: `tests/conformance/`: `Cargo.toml` (`hearth-protocol-mmdebstrap`,
`hearth-test-mirror`), `src/{lib.rs, run.rs (mmdebstrap), resolve.rs
(new)}`, `tests/conformance_resolve.rs` (new);
`testing/hearth-recordings/src/{lib.rs, tests.rs}`; users of `Recording {`
(`git grep -n -F 'Recording {' -- testing tests`), given `lists: None,
stanzas: None`; `testing/recordings/mmdebstrap/<version>/` (shared,
captured; `<version>` is `<mmdebstrap>-apt<apt>`, from each tool's
`--version`, x1's `1.5.7-apt3.3.3`; testing.md 4.5); root `Cargo.toml`,
`Cargo.lock`.
Shape: the sketch's `resolve` and `run::mmdebstrap`. `Resolve::simulate`:
`Sources { suite, architecture, components: [Main], archive, updates:
false, security: None }` from the pin, the target's request or the case's,
`Variant::Apt`, `install_recommends` false; `run` encodes it under
`limits::SHIPPED`, `run_under` under the limits given; the scratch is the
case's `Scratch` itself; 07.1's `vector::Proxy` is made from the kit's
`Proxy::url()`, or a dead port's for `Reach::Unreachable`; a case's
`archive` replaces the pin's. `Resolved::list` is the file apt left named
`…_<component>_binary-<architecture>_Packages`, whole.
`settle` follows 34.1: written under the flag, else compared (Choices); no
recording for this version fails naming the case and the flag. `compare`
is `settle` that never writes, and under the flag does nothing.
Test shape: `tests/conformance/tests/conformance_resolve.rs`, each under
60 s (34.1's `terminate-after`), aim 20 s:
`real_mmdebstrap_resolves_test_min_through_the_mirror` (reference),
`real_mmdebstrap_resolves_each_package_test_boot_requests` (its list too
the checked index),
`an_unknown_package_fails_naming_it`,
`a_virtual_package_alone_fails_with_no_installation_candidate`,
`conflicting_requests_fail_with_no_closure`,
`an_unreachable_mirror_fails_with_no_closure`,
`a_snapshot_the_mirror_lacks_fails_on_its_inrelease_not_found` (the proxy
logged a 404 for that timestamp's `InRelease`, and no `Inst` line); each
`settle`s its case. `several_includes_resolve_as_one` (test-boot
`run_under` `Limits { include: 32, ..limits::SHIPPED }`: more than one
`--include=` in its vector, then `compare`d with test-boot's recording).
Kit, under 20 ms: `a_recording_s_lists_and_stanzas_read_back_file_for_file`,
`a_further_mask_replaces_its_literal_after_the_scratch`.

```rust
#[test]
fn real_mmdebstrap_resolves_test_min_through_the_mirror() {
    let (scenario, case) = (Resolve::new(), resolve::case("test-min"));
    let resolved = scenario.run(case);
    assert_eq!(resolved.run.exit, Exit::Code(0), "{}", resolved.run.stderr_text());
    let insts = resolved.inst_lines();
    assert!(insts.len() > 50 && insts.iter().any(|line| line.starts_with(b"Inst apt ")), "the essential set and apt");
    let list = resolved.list("main").expect("apt left main's Packages list");
    assert!(list == scenario.index(), "apt's list is the index InRelease's entry names, byte for byte");
    let mut recording = resolved.recording();
    recording.stanzas = Some(resolve::stanzas(&list, &insts).expect("a stanza for each Inst line"));
    scenario.settle(case, &recording);
}
```
Proof: live, conformance on loopback: `HEARTH_RECORD=1 heavy cargo nextest
run -p hearth-conformance-world --profile conformance -E
'binary(=conformance_resolve)'` once (seven cases written; `git status
--short -uall testing/recordings | grep -vc '^?? testing/recordings/mmdebstrap/'`
prints 0 and `ls -d testing/recordings/mmdebstrap/*/*/ | wc -l` prints 7),
then the same without the
flag once: eight tests pass, each run as recorded. Real mmdebstrap, given
hearth's vector and the harness's proxy, resolves test-min's and
test-boot's closures from the pinned snapshot, test-boot's split request
as its one, each of test-min's packages with its stanza in the list apt
left, the checked index byte for byte, and fails as each failure case
should, apt's lines on the absent snapshot's `InRelease` 404 recorded for
hearth 38.2.
Bound: the profile's 2 minutes a run; aim 60 s.
Don't: edit a recording; a case reaching anything but the proxy; decode
with hearth 07's readers (38.2's); the fake side (hearth 46); mask the
timestamp; run the fetch beyond hearth 08's budget (Rules).
Size: at most 40 lines (the kit; the world is not counted). Removes: nothing.

## Interface sketches

The kit is hearth 08's (`Pin`, `Mirror`, `Proxy`, `Served`; its sketches);
hearth 46 runs `CASES` on its fake, and 38.2 reads the recordings.

```rust
// testing/hearth-recordings/src/lib.rs (hearth 34.1's; 41.1 adds two fields and `also`)
pub struct Recording {
    // 34.1's fields as it lands them: argv, env, dir, stdin, stdout, stderr, exit, expect.
    /// `lists`: what the run left in apt's lists directory, `file <size> <name>` or `directory <name>`, by name.
    pub lists: Option<Box<[u8]>>,
    /// `stanzas`: the `Packages` stanzas of the run's closure, cut from apt's own list.
    pub stanzas: Option<Box<[u8]>>,
}
impl Mask { /// One more literal masked, after the scratch's: the proxy's port.
    pub fn also(self, from: &[u8], to: &[u8]) -> Mask; }

// tests/conformance/src/resolve.rs (41.1)
/// One resolve case: a pinned target's inputs, its request or timestamp replaced or not, the mirror reached or not.
pub struct Case { pub name: &'static str, pub target: &'static str, pub request: Option<&'static [&'static str]>, pub archive: Option<&'static str>, pub reach: Reach }
/// The harness's proxy; or a loopback port nothing listens on.
pub enum Reach { Proxy, Unreachable }
pub const CASES: &[Case] = &[/* Choices' seven, in its order */];
pub fn case(name: &str) -> &'static Case;
/// The scenario's prerequisites: `/usr/bin/mmdebstrap` printing `mmdebstrap 1.`, `/usr/bin/apt-get`, the committed
/// pin, its fetched mirror; `version`, `<mmdebstrap>-apt<apt>` (testing.md 4.5).
pub struct Resolve { pin: Pin, mirror: Mirror, version: String }
impl Resolve {
    /// Refuses, naming what is missing (testing.md, 5.5): a tool, or the mirror with `testing/mirror/fetch`.
    pub fn new() -> Resolve;
    pub fn simulate(&self, case: &Case) -> Simulate;
    /// `run_under(case, &limits::SHIPPED)`.
    pub fn run(&self, case: &Case) -> Resolved;
    pub fn run_under(&self, case: &Case, limits: &Limits) -> Resolved;
    /// The checked `Packages` of `main`, whole.
    pub fn index(&self) -> Vec<u8>;
    /// Written under `HEARTH_RECORD=1`; else compared (Choices), failing with the case, the first difference and the flag.
    pub fn settle(&self, case: &Case, recording: &Recording);
    /// Compared with `case`'s recording as `settle` compares; never written; nothing under the flag.
    pub fn compare(&self, case: &Case, recording: &Recording);
}
/// One run, what the proxy answered, and what masks it.
pub struct Resolved { pub run: Run, pub served: Vec<Served>, scratch: Scratch, port: u16 }
impl Resolved {
    /// The lines beginning `Inst `, stderr's then stdout's.
    pub fn inst_lines(&self) -> Vec<&[u8]>;
    /// apt's list `…_<component>_binary-<architecture>_Packages` in the scratch, whole; none if apt left none.
    pub fn list(&self, component: &str) -> Option<Vec<u8>>;
    /// 34.1's recording of the run, masked, with `lists`.
    pub fn recording(&self) -> Recording;
}
/// Each stanza of `list` an `Inst` line names, whole, in the list's order; the first line with none is the error.
pub fn stanzas(list: &[u8], insts: &[&[u8]]) -> Result<Box<[u8]>, Box<[u8]>>;

// tests/conformance/src/run.rs (hearth 34.1's; 41.1 adds)
/// `sources_list` written to `<scratch>/sources.list`; `PROGRAM` spawned by its path, the environment cleared then `env`, stdin empty, both pipes read to their end.
pub fn mmdebstrap(invocation: &Invocation, scratch: &Path) -> Run;
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth ls-tree --name-only origin/main
   testing/recordings/mmdebstrap/ | wc -l` prints 1 (one version).
2. `git -C ~/src/rust/hearth ls-tree -r --name-only origin/main
   testing/recordings/mmdebstrap | grep -c -E '/(exit|stanzas)$'` prints 8:
   seven cases, one with stanzas.
3. In a worktree of hearth at `origin/main` on the run's machine, `heavy
   cargo nextest run --workspace --profile conformance -E
   'binary(=conformance_resolve)'` passes once, its `Summary` naming at
   least eight tests (hearth 46.2 adds its own), within the profile's 2
   minutes.
4. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-41\.1'` prints 1.
