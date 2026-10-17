# Session 38: `hearth-protocol-mmdebstrap`: `Packages` stanzas, apt's lists, and real runs

The rest of render's resolution machine (render.md, 5.3, 5.5): apt's
`Packages` lists read stanza by stanza into six fields, the lists' names
under the scratch, and the world's last step, hearth 41.1's recordings of
real mmdebstrap, cut and mutated, on which the readers and the tail's
patterns are held to what apt shows. Needs hearth 07 (07.2: the crate,
`Limits`, `Name`, `Line`, the world `tests/mmdebstrap`; 38.2 also 07.3's
`tail`) and, for 38.2, hearth 41.1's recordings. Nothing calls the crate
until hearth 24 routes it.

## Read first

- hearth's render.md: 5.3 to 5.5, 7.2 (kind `resolution`), 10; store.md,
  6.1 (kind 4's bounds), 6.4; testing.md, 2.4, 4.1, 4.3, 4.5, 6.2 (the
  mirror as a proxy).
- This plan's `hearth/07-mmdebstrap.md` whole (its sketches are this
  session's types; its Choices bind it) and
  `hearth/41-resolve-recordings.md`, 41.1 (the recordings, `Recording`'s
  `lists` and `stanzas`); `decisions.md`, 9, 10, 12;
  `findings/skein-testing.md`, 3, 7.
- skein's `programming-model.md`: 4, 6.3, 7 (by number only);
  `testing-strategy.md`: 2.4, 4.1, 8 (by number only).

## What the design asks

- **Stanzas** (5.3): it "reads every `*_Packages` file under
  `<scratch>/root/var/lib/apt/lists/` in bounded chunks, stanza by stanza
  (at most 1 MiB a stanza ...), and sends up each stanza's `Package`,
  `Version`, `Architecture`, `Filename`, `Size` and `SHA256`, holding no
  other field." The protocol layer hashes each file in that pass; the
  domain matches. apt keeps the lists decompressed: the reader takes the
  files ending `_InRelease` and `_Packages`, skips every other name
  (`lock`, `partial/`), and a compressed, unnamed or missing list is a
  `tool` error, operational.
- **Open** (10), for recordings: "The `Inst` line with several origins";
  failures with no recording yet.
- **The world** (testing.md, 2.4, 4.5): recordings "each with what it must
  decode to, output generated from the seed, and both mutated";
  "Captured, never written by hand."

## Choices this plan makes

- **Modules:** `packages` and `lists` (38.1), beside hearth 07's; no
  dependency is added (`skein-lib` alone).
- **Stanzas:** field names match whatever their case (deb822, Policy 5.1);
  hex digits in either case; a `Filename` is relative with no `.` or `..`
  segment, so the archive's URI and it stay in the archive.
- **Compressed lists are named, not read:** `lists::listed` names a
  `_Packages.<xz|gz|bz2|lzma|lz4|zst>` as `Compressed`, which hearth 24
  refuses as a `tool` error: M1 needs no decompressor (render.md 5.3; 10
  waits on an apt that keeps its indices compressed).
- **A refused list is a `tool` error, operational** (render.md 5.3, 7.2);
  the `InRelease` entry is the size and SHA-256 of the pass that reads the
  stanzas, no field of `InRelease` read (5.4).
- **The recordings' directory** is `<mmdebstrap>-apt<apt>` (testing.md
  4.5; hearth 41.1). **Both pipes' readers decode `Inst` lines**, each
  triple is sent as it comes, `insts` counts lines, and hearth 24 merges
  equal triples (render.md 5.3); the patterns follow the recordings, none
  compared with os's source (5.5); this crate decodes no tool's version
  (empty in M1, 5.4) and takes `Fetched` from hearth 07.2.
- **Expectations** are `tests/mmdebstrap/expect/<version>/<case>.expect`,
  drafted by the world's reference and checked by hand (skein's
  `tests/json/tests/transcripts.rs:24`), outside hearth 34's layout.
- **The test mirror is the proxy** (testing.md, 6.2): the sources line is
  the fixed URL in every tier, so apt's list names are the same in tests.

## Rules that bind it

- **No hand-written recording, nothing beyond loopback**
  (code-organization.md, 6): 38.2 reads `testing/recordings/mmdebstrap/`
  as hearth 41.1 captured it, never edits it, and runs no mmdebstrap.
- **Limits and lints as hearth 07's Rules:** tests take `limits::SHIPPED`,
  another value set beside the default with its reason; step tests use no
  `Vec` and no closure-taking combinator.

## Increments

### 38.1 `Packages` stanzas and apt's lists
Scope: `packages::Stanzas`, one `Packages` list in bounded scans, stanza by
stanza, into the six fields (5.3); `lists`, the lists each source line
gives apt and their names under `<scratch>/root/var/lib/apt/lists/` (5.3;
store.md, 6.4).
Needs: hearth 07.2 (the world crate).
Files: `src/packages.rs`, `src/lists.rs`, `src/limits.rs` (`stanza`,
`chunk`, `filename`), `src/lib.rs`, `src/vector.rs` (`Component::bytes`
`pub(crate)`, for `lists::indices`), `src/tests.rs` (`mod packages;`),
`src/tests/{fixtures,packages}.rs` (fixtures gain `opened_list()`,
`apt_stanza()`); world:
`src/generate.rs` (`index`), `src/reference.rs`, `src/world.rs`
(`read_index`), `tests/mmdebstrap.rs`, `tests/fuzzy_mmdebstrap.rs`,
`tests/memory.rs`.
Shape: the sketch's `packages` and `lists`. Scans of at most `chunk`; a
longer line comes in pieces, and only the six values are held. A stanza
runs to an empty line (or one of spaces and tabs) or `End`, at most
`stanza` bytes with its line ends; empty lines between stanzas are
skipped. A field line is `<name>:<value>`, the value without leading and
trailing spaces and tabs; a line starting with a space or a tab continues
the field before it. Each of the six once, on one line: `Package` and
`Architecture` a `Name`; `Version` 1 to `name` bytes of `[A-Za-z0-9.+~:-]`;
`Filename` 1 to `filename` printable bytes but space, relative, no `.` or
`..` segment; `Size` decimal within `u64`; `SHA256` 64 hex digits. A list's
name is apt's for its URI (the line's URI without `http://`, then
`dists/<suite>/` and the path, each `/` a `_`): `InRelease`, then each
component's `<component>/binary-<architecture>/Packages`.
Test shape: `src/tests/packages.rs` on fixtures that gain `opened_list()`
and `apt_stanza()`, each under 20 ms:
`a_stanza_gives_its_six_fields_and_skips_the_rest`,
`field_names_match_whatever_their_case`,
`the_last_stanza_ends_at_the_end_of_the_list`,
`a_stanza_past_its_limit_fails_the_list` (`stanza: 1_024` beside the default),
`a_stanza_past_64_kib_with_one_long_provides_line_is_read`,
`a_missing_or_repeated_field_fails_naming_it`,
`a_size_digest_or_filename_outside_its_form_fails`,
`each_source_line_names_its_inrelease_and_packages_lists` (literal:
`snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_main_binary-amd64_Packages`),
`a_compressed_list_is_named_compressed_never_an_index`. World, focused:
`generated_lists_decode_as_the_reference_whatever_the_cuts`,
`a_list_cut_short_reads_as_the_reference_reads_its_prefix`; fuzzy:
`generated_and_mutated_lists_read_as_the_reference_reads_them`; memory:
`a_stanza_reader_at_its_limits_holds_no_more_than_its_worst_case`.
```rust
#[test]
fn a_stanza_gives_its_six_fields_and_skips_the_rest() {
    let (env, mut stanzas, mut above, mut below) = fixtures::opened_list();
    let lines: [&[u8]; 9] = [b"Package: apt\n", b"Description: package manager\n", b" for Debian\n", b"Version: 3.1.6\n",
        b"Architecture: amd64\n", b"Filename: pool/main/a/apt/apt_3.1.6_amd64.deb\n", b"Size: 1736\n",
        b"SHA256: 2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae\n", b"\n"];
    for line in lines { packages::up(&mut stanzas, &env, Up::Bytes(bytes::copy_of(line)), &mut above, &mut below); }
    assert_eq!(above.pop(), Some(Found::Stanza(fixtures::apt_stanza())));
    assert_eq!(above.pop(), None);
}
```
Proof: world, `generated_lists_decode_as_the_reference_whatever_the_cuts`
(bound 0.5 s); no caller until hearth 24. 38.2 runs the reader on the real
stanzas hearth 41.1 records.
Don't: hash (the protocol layer's, 5.3); decompress; read `InRelease`.
Size: at most 290 lines.
Removes: nothing.

### 38.2 Real runs: hearth 41's recordings, cut and mutated
Scope: the world reads hearth 41.1's recordings of simulate, each beside
its hand-checked expectation, with apt's lists and test-min's stanzas,
whole, cut at random, cut short and mutated (testing.md, 2.4, 4.5); the
readers follow what they show (render.md, 10), and the tail's patterns
are held to real failures (5.5).
Needs: hearth 07.3 (`tail`), 38.1; hearth 41.1's recordings under
`testing/recordings/mmdebstrap/<version>/` (argv, env, stdout, stderr,
exit: hearth 34's layout; `lists`; `<version>` as 41 names it):
`test-min` and `test-boot` resolved; the failures `unknown-package`,
`no-candidate` (a virtual package alone), `unsatisfiable`,
`unreachable-mirror` and `absent-snapshot` (its `InRelease` 404); and,
beside test-min's, `stanzas`, its closure's, cut from the list apt left.
Files: world: `Cargo.toml` (`[dependencies]` gain `hearth-recordings`, and
`hearth-test-mirror` for the pin: `src/` is the world's library), `Cargo.lock`,
`src/lib.rs` (`pub mod recordings;`), `src/recordings.rs` (new),
`src/fixtures.rs` (each case's
inputs, from `testing/mirror`'s pin and `testing/repos`' targets),
`expect/<version>/<case>.expect` (new), `tests/recordings.rs` (new),
`tests/fuzzy_mmdebstrap.rs`; the crate's `src/{output, tail, packages,
lists}.rs` where a recording disagrees with their Shape.
Shape: `recordings::all() -> Vec<Case>` wraps
`hearth_recordings::all("mmdebstrap")` (the kit's `Recording`, no second
reader; its `version` names `expect/<version>/`) with its expectation (`lists` among its files, its stanzas when it has them); an
`.expect` is `#` lines, then `inst <name> <version> <architecture>` lines,
`fetched <bytes>` (apt's `Fetched` figure, 0 when it prints none; 5.4),
then `ended`, or `cause <cause>` and `tail <line>` lines; when one is
missing the test fails with the reference's draft, which is checked
against the recording by hand: each `.expect` has as many `inst` lines as
`grep -c '^Inst '` finds in the recording's two pipes, and its causes
are the table below's. The stanzas need no `.expect`: the reference reads
them, and the closure's triples anchor them.
Test shape: `tests/recordings.rs`, each under 0.5 s:
`every_recording_has_an_expectation_and_the_reference_agrees`,
`every_recording_decodes_to_its_expectation_whatever_the_cuts`,
`the_recorded_vectors_are_the_ones_this_machine_encodes` (the recorded
`argv` and `env` equal hearth 07.1's encoding of the case's inputs, with
the scratch and the proxy the recording names: the harness's port varies),
`the_recorded_failures_have_the_causes_5_5_gives_them` (a `const` table:
`unknown-package` `Unknown(hearth-absent)`, `no-candidate`
`Unknown(mail-transport-agent)` and `unsatisfiable`
`Unsatisfiable(Some(<the first package apt names>))`, configuration;
`unreachable-mirror` `Fetch`, operational, kind `tool`; `absent-snapshot`
(its 404) `Absent`, configuration, kind `resolution`),
`the_recorded_lists_hold_each_index_and_no_other_list` (each name
`lists::indices` gives for the case's lines is a `file` of `lists` in
every case but the two that fail at the fetch; a `file` named
`…_Packages…` or `…_InRelease` that is not `Listed::Index` fails it,
named; `lock` and `partial/` are `Other`, skipped),
`the_recorded_stanzas_decode_as_the_reference_whatever_the_cuts`,
`every_inst_line_of_test_min_has_a_recorded_stanza` (more than 50
triples, each equal to exactly one stanza's `Package`, `Version` and
`Architecture`),
`a_recording_cut_short_reads_as_the_reference_reads_its_prefix`; fuzzy:
`recordings_cut_and_mutated_read_as_the_reference_reads_them`, output and
stanzas (mutations as temper's
`tests/worker/protocol-git/tests/transcripts.rs:111`).
```rust
#[test]
fn every_recording_decodes_to_its_expectation_whatever_the_cuts() {
    for recording in recordings::all() {
        let expected = recording.expected.as_ref().expect("an expectation");
        for seed in 0..4 {
            let run = world::read_recording(&recording, seed, &LIMITS);
            assert_eq!(&run.decoded, expected, "{}, seed {seed}", recording.case);
        }
    }
}
```
Proof: world, `every_recording_decodes_to_its_expectation_whatever_the_cuts`:
real mmdebstrap's output for `test-min` and `test-boot` decodes to the
hand-checked closure triples, and its failures to their causes (bound
0.5 s). Nothing above until hearth 24; conformance's resolve scenario
(hearth 46) then runs it live.
Don't: edit under `testing/recordings/`; run mmdebstrap; add a pattern no
recording shows.
Size: at most 80 lines (fixes).
Removes: nothing.

## Interface sketches

Derives (`Clone`, `PartialEq`, `Eq`, `Debug`; `Copy` where it holds) are
elided; an enum's doc line names what each variant is. `MaxOut`, `Name`,
`Component`, `Line`, `Limits`, `Fault` and `Up` are hearth 07's.

```rust
// crates/hearth-protocol-mmdebstrap/src/packages.rs (38.1)
/// A stanza's six fields (5.3).
pub struct Stanza { pub package: Name, pub version: Box<[u8]>, pub architecture: Name, pub filename: Box<[u8]>, pub size: u64, pub sha256: [u8; 32] }
/// One of the six fields.
pub enum Field { Package, Version, Architecture, Filename, Size, Sha256 }
/// Why a list was refused, nothing more of it read: past `stanza`; a line no field, or a continuation
/// first; one of the six missing, twice, or out of its form or continued; the stream broke.
pub enum StanzaError { TooLong, Malformed, Missing(Field), Repeated(Field), Value(Field), Stream(Fault) }
/// To the owner, at most two an entry point (the last stanza, then `Ended`); `Ended` or `Failed`, the terminal.
pub enum Found { Stanza(Stanza), Ended, Failed(StanzaError) }
pub struct Stanzas { /* private */ }
impl Stanzas { pub fn new(limits: &Limits) -> Stanzas; }
pub const UP_MAX_OUT: MaxOut = MaxOut { above: 2, below: 1 };
pub fn largest_demand(limits: &Limits) -> u32;
pub fn worst_case(limits: &Limits) -> Option<u64>;
pub fn open(stanzas: &mut Stanzas, env: &Env<Limits>, below: &mut Queue<Down>);
pub fn up(stanzas: &mut Stanzas, env: &Env<Limits>, event: Up, above: &mut Queue<Found>, below: &mut Queue<Down>);
pub fn close(stanzas: &mut Stanzas, below: &mut Queue<Down>);

// crates/hearth-protocol-mmdebstrap/src/lists.rs (38.1)
/// Where apt keeps its lists, below the scratch directory.
pub const LISTS: &[u8] = b"root/var/lib/apt/lists";
/// A list a source line gives (store.md, 6.4): the line's place; the component, none for `InRelease`;
/// its path in the archive below `dists/<suite>/`; apt's name for it.
pub struct Index { pub line: u8, pub component: Option<Component>, pub path: Box<[u8]>, pub name: Box<[u8]> }
/// A name under `LISTS`: one of the indices; a compressed copy of one, refused (render.md, 10); anything else.
pub enum Listed { Index(u32), Compressed(u32), Other }
pub fn indices(lines: &[Line], architecture: &Name) -> Box<[Index]>;
pub fn listed(name: &[u8], indices: &[Index]) -> Listed;
```

## Questions to raise when reached

None.

## Done when

- `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-38\.[1-2]'` prints 2.
- In a worktree of hearth at `origin/main`, once each: `heavy cargo nextest
  run -p hearth-protocol-mmdebstrap -p hearth-mmdebstrap-world` and `heavy
  cargo nextest run -p hearth-mmdebstrap-world --profile fuzzy` print a
  `Summary` line with no test failed.
