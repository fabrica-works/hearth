# Session 27: the package universe, the fake mirror and the closure

The fake machine's `hearth` half gains the package universe (testing.md,
3.6) at M1's grain: packages with their relations and archive fields,
read from and written as a `Packages` index, and a fake mirror holding
them at snapshot timestamps. A new kit, `testing/hearth-fake-mmdebstrap`,
resolves for real (decisions.md, 14). Its face and resolve's conformance
scenario are hearth 46.

## Read first

- hearth's testing.md: 3.1, 3.5, 3.6, 6.2; decisions.md, 14;
  code-organization.md, 1 and 3.
- `hearth/26-fake-git.md` and `hearth/44-fake-git.md` whole;
  `hearth/38-stanzas-recordings.md`, the `lists` sketch.
- Debian Policy 5.6.12 (versions), 7.1 (relations), 7.4 (Conflicts), 7.5
  (Provides); mmdebstrap(1), `--variant`.
- skein: `crates/skein-lib/src/rng.rs`, `src/sha256.rs`
  (`skein_lib::sha256`).

## What the design asks

- **The universe** (3.6): "Drawn from the seed, and shaped like Debian's
  and like the recording of the local mirror (6.2). A fake mirror serves
  it at snapshot timestamps"; "package name, version, architecture,
  priority, Essential, section / relations Depends, Pre-Depends, Provides,
  Conflicts, with their version relations / archive Installed-Size; the .deb's size and SHA-256, as Packages lists
  them"; "A timestamp between two snapshots answers as the earlier one";
  "a bump changes some versions"; "Variants as mmdebstrap defines them".
- **For real** (decisions.md, 14): "computes the closure from Essential,
  priorities, Depends, Pre-Depends, Provides, alternatives, version
  relations and Conflicts ..., which must equal apt's for `test-min` and
  `test-boot`".

## Choices this plan makes

- **Settled by the plan:** decisions.md 14; code-organization.md 1 and 3:
  the universe and the mirror in `hearth-fake-machine`'s `src/hearth/`;
  resolution in `hearth-fake-mmdebstrap` (its face is hearth 46), with its own
  `[lints]` as `skein/tests/json/Cargo.toml:17-33` and a copy of the
  machine's `clippy.toml`, as hearth 44's fake git, and `missing_errors_doc
  = "allow"` (`closure` returns a `Result`).
- **The universe keeps `KEPT`:** 3.6's index fields, `Filename` (38.1's
  reader refuses a stanza without it) and `Conflicts` (testing.md 3.6: Conflicts and version relations kept; Breaks,
  Replaces and Recommends out until a scenario shows one changes a closure), in
  the order Debian's index writes them; reading drops the rest. Relations
  are Policy 7.1's: alternatives, version relations, an architecture
  qualifier (`perl:any` is in test-boot's closure) kept and matched by
  name, M1 having one architecture; `all` counts as the native one. In
  x1's forky index at `20260928T022309Z`, all 78,685 relation fields were
  in the form the writer prints, so kept fields round-trip byte for byte;
  the pin's, `20261017T143929Z` (`testing/mirror/pin`), has 78,830
  `Depends`, `Pre-Depends`, `Provides` and `Conflicts` fields.
- **Resolution, the plan's model of apt,** held to apt by hearth 46.2: the
  variant's packages, then `--include`'s, all marked before any
  dependency is followed (apt-get marks its requests first); then, in
  marking order, each Pre-Depends and Depends group is met by a marked
  package or provider (versioned only by a versioned provide), else takes
  its first alternative whose candidate (the highest version over the
  source lines) satisfies the relation and conflicts with nothing marked
  (its Conflicts against the marked, and the marked ones' against it),
  else that alternative's providers ranked Essential, then priority, then
  index order, else the next alternative; none free of conflicts, the
  first whose candidate satisfies the relation is taken all the same, as
  apt does (`exim4-config` under `postfix`: a Conflicts, not a Depends,
  fails 41.1's `unsatisfiable`). On x1's index the model gives
  66 packages for test-min (`awk` is mawk's, required, not gawk's,
  optional) and 157 for test-boot, no relation unmet, no conflict.
- **The mirror** holds the `debian` archive's snapshots, suites by
  codename; a `Suite` renders its indices and `InRelease` once (unsigned,
  undated; `SHA256:` entries by skein-lib: the size and SHA-256 of each index,
  paths relative to `dists/<suite>/`, render.md 5.4). `hearth::Machine::serve`
  puts it on the machine's network as snapshot.debian.org and, given a
  port, as `http://127.0.0.1:<port>/`; another port reaches nothing, as
  41.1's dead port. One owner: a world moves it in and takes it back.
- **A drawn universe** has a fixed core shaped like forky's (base-files
  pre-depending on `awk`, mawk and gawk providing it, base-passwd, dpkg,
  libc6, debconf providing `debconf-2.0`, apt), then the draw's packages
  and names, so every draw resolves the apt variant; hearth 28 takes it.
## Rules that bind it

- **No expected value computed by the code under test:** stanzas pasted
  from the pin's forky index, `20261017T143929Z` (08's cache, `checked/main/
  binary-amd64/Packages`; once, `awk -v RS= '$1=="Package:" && $2=="mawk"'`
  of it), digests and sizes from
  `sha256sum` and `wc -c`, version orders from `dpkg --compare-versions`.
- **Shared files:** root `Cargo.toml` and `Cargo.lock` (27.3);
  `testing/hearth-fake-machine/src/hearth.rs` (hearth 26, 28, 46);
  `testing/mirror/` (hearth 08).

## Increments

### 27.1 The universe's index: packages, relations, versions
Scope: 3.6's index fields as typed packages; Debian's version order; a
`Packages` index read and written.
Needs: hearth 26 (the machine and its `hearth` half).
Files: `testing/hearth-fake-machine/src/hearth.rs` (two `pub mod`s),
`src/lib.rs` (`pub mod hearth;`, unless 26.2 made it public); new
`src/hearth/universe.rs`, `src/hearth/version.rs`; tests `src/tests.rs`,
`src/tests/universe.rs` (new), `src/tests/fixtures.rs` (`MAWK`, mawk's
stanza whole; `MAWK_KEPT`, its kept fields, 306 bytes; `TINY_INITRAMFS`).
Shape: the sketch's `version` and `universe`. `compare` is Policy
5.6.12's: epoch (absent 0), upstream, revision (absent `0`); `~` first,
letters before other marks. `read`: stanzas to an empty line, `<name>:
<value>` fields, continuation lines skipped (no kept field has one); a
kept field at most once; Package, Version, Architecture, Section,
Priority, Filename, Size and SHA256 required; `Essential: yes` alone is
true (any other value, `Wrong::Form`); relations in 7.1's form; a name
twice refused (`Duplicate`); `Universe::new` asserts it too. `index` writes each
package's kept fields in `KEPT`'s order, then an empty line.
Test shape: `src/tests/universe.rs`, each under 20 ms:
`versions_compare_as_dpkg_compares_them` (a `const` table: `1.0~rc1` <
`1.0`, `2.41-12` > `2.41-9`, `1:5.3.2-1` > `1.3.4.20260302-2`,
`0.1-8+b+~` > `0.1-8`, `1.0` = `1.0-0` = `0:1.0`, `1.0+dfsg` > `1.0`),
`a_real_stanza_written_back_keeps_its_kept_fields_byte_for_byte`,
`relations_read_back_with_alternatives_versions_and_qualifiers`,
`a_provided_name_lists_its_providers_in_index_order`,
`a_stanza_missing_a_kept_field_is_refused_naming_its_line`,
`a_package_named_twice_is_refused_naming_the_second`,
`a_kept_field_given_twice_is_refused_as_twice`,
`a_value_out_of_form_is_refused_as_form`,
`essential_is_true_for_yes_alone`,
`a_universe_made_with_one_name_twice_asserts` (`#[should_panic]`).

```rust
#[test]
fn a_real_stanza_written_back_keeps_its_kept_fields_byte_for_byte() {
    // mawk's stanza, whole, from x1's forky main/binary-amd64/Packages at 20261017T143929Z, the pin's.
    let universe = Universe::read(fixtures::MAWK).expect("a real stanza reads");
    assert_eq!(universe.packages().len(), 1);
    assert_eq!(&*universe.index(), fixtures::MAWK_KEPT, "Maintainer, Description, Multi-Arch, Homepage, Tag and MD5sum dropped");
    assert_eq!(universe.package(b"mawk").map(|package| package.priority), Some(Priority::Required));
}
```
Proof: unit, that test: a real stanza read, and written back as apt's
lists will hold it. 27.2 serves it; hearth 46.2 loads the real index.
Bound: 0.02 s.
Don't: keep a field `KEPT` lacks; read `Multi-Arch`; name a hearth crate.
Size: at most 290 lines. Removes: nothing.

### 27.2 The fake mirror at snapshot timestamps; a drawn universe
Scope: suites, snapshots, bumps (3.6); `InRelease`; the mirror on the
machine's network (3.5); a universe drawn from the seed.
Needs: 27.1 (`Universe`); `skein_lib::sha256` is in hearth's pin.
Files: new `src/hearth/mirror.rs`, `src/hearth/draw.rs`; `src/hearth.rs`
(two more `pub mod`s, `mirror` and `draw`; `served`, `serve`, `unserve`,
`reach`); tests `src/tests.rs` (`mod mirror;`), `src/tests/mirror.rs`
(new), `src/tests/fixtures.rs` (`forky(universe) -> Suite`; `ARCHIVE`).
Shape: the sketch's `mirror` and `draw`. `Suite::new` renders each
component's index, then `InRelease`: `Origin`, `Label`, `Suite`,
`Codename`, `Version` if any, `Architectures`, `Components`, `SHA256:`
and ` <hex> <size> <component>/binary-<architecture>/Packages` each.
`Mirror::suite` answers the latest snapshot at or before the timestamp
(testing.md 6.2); none before the first (404), or for an archive but `debian`. `bump` copies the
latest snapshot to a later timestamp, appending `+b1` to a package's
version with probability `per_mille` in a thousand, with a new size and
digest. `draw`: the core, then `pkg-<k>` with 1 to 3 Depends groups on
earlier names (`>=` their versions; one group in five two alternatives),
one Pre-Depends on an earlier name for one package in four,
`virtual-<j>` provided by one package in six, `all` for one in ten; then
each of `names` as a package with no relation, `optional`.
Test shape: `src/tests/mirror.rs`, each under 20 ms:
`a_suite_s_inrelease_lists_its_index_by_sha256_and_size`,
`a_timestamp_between_two_snapshots_answers_as_the_earlier`,
`a_timestamp_before_the_first_snapshot_finds_no_suite` (41.1's
`20000101T000000Z`), `a_bump_moves_every_version_at_1000_none_at_0_and_keeps_the_earlier_snapshot`,
`a_draw_holds_its_core_and_every_name_it_asks_for`,
`one_seed_draws_one_universe`,
`only_the_proxy_port_the_mirror_is_served_on_reaches_it`.

```rust
#[test]
fn a_suite_s_inrelease_lists_its_index_by_sha256_and_size() {
    let universe = Universe::read(fixtures::MAWK_KEPT).expect("the kept fields read");
    let suite = Suite::new(Release::debian(b"testing", b"forky"), b"amd64", Box::new([(Box::from(&b"main"[..]), universe)]));
    // sha256sum and wc -c of MAWK_KEPT, once on x1.
    let expected: &[u8] = b"Origin: Debian\nLabel: Debian\nSuite: testing\nCodename: forky\nArchitectures: amd64\nComponents: main\nSHA256:\n \
        6e80ba38d7802345c39dc883a60851c8e1c9ddbc9911cffa6861d761f1253ed2 306 main/binary-amd64/Packages\n";
    assert_eq!(suite.in_release(), expected);
    assert_eq!(suite.index(b"main"), Some(fixtures::MAWK_KEPT), "the index served is the one listed");
}
```
Proof: unit, `a_timestamp_between_two_snapshots_answers_as_the_earlier`:
3.6's rule through `Mirror::suite`. Nothing above yet: hearth 46.1 reaches it.
Bound: 0.02 s.
Don't: sign or date `InRelease`; serve `Packages.xz`; keep a .deb (M2).
Size: at most 260 lines. Removes: nothing.

### 27.3 Fake mmdebstrap's domain: the closure
Scope: the closure of a variant and a request over a snapshot's
universes, and its failures (decisions.md, 14; Choices).
Needs: nothing beyond 27.2.
Files: root `Cargo.toml` (shared: `hearth-fake-mmdebstrap`), `Cargo.lock`
(shared); new `testing/hearth-fake-mmdebstrap/`: `Cargo.toml`
(`hearth-fake-machine`; dev `skein-lib`), `clippy.toml`, `src/lib.rs`
(`#![forbid(unsafe_code)]`, `extern crate alloc`), `src/resolve.rs`;
tests `src/tests.rs`, `src/tests/resolve.rs`, `src/tests/fixtures.rs`
(new: `DEBIAN`, 15 invented stanzas in `KEPT`'s form, shaped like the
pin's index, which keeps linux-image-amd64's initramfs alternation on
`linux-image-<version>-amd64`; the fixture folds it in: base-files, base-passwd,
dpkg, libc6, mawk, gawk, debconf, adduser, apt; exim4-daemon-light and
postfix, each providing and conflicting with `mail-transport-agent`;
linux-image-amd64 depending on `dracut (>= 110) | initramfs-tools (>=
0.120+deb8u2) | linux-initramfs-tool`, dracut 059-4, initramfs-tools
0.150, tiny-initramfs providing `linux-initramfs-tool` and conflicting
with both; `debian()`; `names(universes, selected)`, sorted).
Shape: the sketch's `resolve`. Marking order, mmdebstrap 1.5's variants
(testing.md 3.6: `essential`, `apt`, then `required`, `important`, `standard`
each with apt): the suite's Essential packages in index order; `apt`, in
every variant but `Essential`; for `Required`, `Important` and
`Standard`, then each package of that priority or above, in index order;
then `include`. A requested name nothing carries is `Unknown`; one only
provided takes its one provider, else `NoCandidate`. Groups as Choices;
a group left unmet is a `Broken`; after the walk, each Conflicts entry
matching another marked package (name and relation, or an unversioned
provide) is one. `Broken`s in marking order, at most `MOST_BROKEN`; the
walk is a worklist of at most the universes' package count.
Test shape: `src/tests/resolve.rs`, each under 20 ms:
`the_apt_variant_takes_the_essential_set_apt_and_their_dependencies`,
`a_virtual_dependency_takes_its_highest_priority_provider`,
`a_requested_provider_satisfies_a_group_before_its_first_alternative`
(tiny-initramfs requested: neither dracut nor initramfs-tools),
`a_versioned_dependency_passes_over_an_alternative_too_old`,
`a_request_no_package_carries_fails_as_unknown`,
`a_virtual_request_with_two_providers_has_no_candidate`,
`two_requests_that_conflict_fail_naming_both`,
`each_priority_variant_adds_its_priority_to_the_essential_set`,
`every_drawn_universe_resolves_its_apt_variant` (seeds 0 to 15, `Draw {
packages: 24, names: &[] }`),
`broken_relations_are_listed_at_most_most_broken` (65 unmet groups, 64
listed).

```rust
#[test]
fn the_apt_variant_takes_the_essential_set_apt_and_their_dependencies() {
    let universe = fixtures::debian();
    let request = Request { universes: &[&universe], variant: Variant::Apt, include: &[] };
    let closed = resolve::closure(&request).expect("the fixture resolves");
    // Essential: base-files, base-passwd, dpkg; then apt. awk is mawk's (required over gawk's optional);
    // base-passwd meets apt's `base-passwd (>= 3.6.1) | adduser`; debconf is its group's first alternative.
    assert_eq!(fixtures::names(&[&universe], &closed), ["apt", "base-files", "base-passwd", "debconf", "dpkg", "libc6", "mawk"]);
}
```
Proof: unit, that test: the apt variant's closure on Debian's own
relations. Nothing calls it yet: hearth 46.1's face; hearth 46.2 holds
it to apt. Bound: 0.02 s.
Don't: follow Recommends, Suggests or Breaks; recurse; depend on a hearth
crate but the machine.
Size: at most 230 lines. Removes: nothing.

## Interface sketches

Derives are elided where a line names none (`Clone`, `PartialEq`, `Eq`,
`Debug`; `Copy` where it holds); elided fields are private. Every `pub fn`
that returns a value and takes no `&mut` carries `#[must_use]`, elided like
the derives.

```rust
// testing/hearth-fake-machine/src/hearth/version.rs (27.1)
/// Debian's version order (Policy 5.6.12).
pub fn compare(a: &[u8], b: &[u8]) -> core::cmp::Ordering;

// testing/hearth-fake-machine/src/hearth/universe.rs (27.1)
use alloc::boxed::Box;
/// The fields kept, in the order the index writes them.
pub const KEPT: [&str; 14] = ["Package", "Version", "Essential", "Installed-Size", "Architecture", "Provides",
    "Depends", "Pre-Depends", "Conflicts", "Section", "Priority", "Filename", "Size", "SHA256"];
/// Debian's priorities, apt's preference first: required, important, standard, optional, extra.
pub enum Priority { Required, Important, Standard, Optional, Extra }
/// A version relation (Policy 7.1): `<<`, `<=`, `=`, `>=`, `>>`.
pub enum Relation { Earlier, EarlierOrEqual, Equal, LaterOrEqual, Later }
impl Relation { pub fn holds(self, have: &[u8], wanted: &[u8]) -> bool; }
/// One alternative: a name, its architecture qualifier (`any`), a version relation.
pub struct Alternative { pub name: Box<[u8]>, pub qualifier: Option<Box<[u8]>>, pub version: Option<(Relation, Box<[u8]>)> }
/// A name provided, with its version for a versioned provide (Policy 7.5).
pub struct Provide { pub name: Box<[u8]>, pub version: Option<Box<[u8]>> }
/// One package's kept fields; a group is `|`-joined alternatives.
pub struct Package {
    pub name: Box<[u8]>, pub version: Box<[u8]>, pub architecture: Box<[u8]>, pub priority: Priority,
    pub essential: bool, pub section: Box<[u8]>, pub installed_size: Option<u64>, pub provides: Box<[Provide]>,
    pub depends: Box<[Box<[Alternative]>]>, pub pre_depends: Box<[Box<[Alternative]>]>, pub conflicts: Box<[Alternative]>,
    pub filename: Box<[u8]>, pub size: u64, pub sha256: [u8; 32],
}
/// One component's packages at one snapshot, in index order, by name and by what they provide.
pub struct Universe { /* packages, names, providers */ }
/// An index refused: its line from 1, and what is wrong.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct IndexError { pub line: u32, pub wrong: Wrong }
/// A required field missing; a field twice; a value out of form (by field); a package named twice.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Wrong { Missing(&'static str), Twice(&'static str), Form(&'static str), Duplicate }
/// `new` asserts one package a name (an M1 snapshot holds one version); `providers`: places in `packages`, index order.
impl Universe { pub fn new(packages: Box<[Package]>) -> Universe; pub fn read(index: &[u8]) -> Result<Universe, IndexError>; }
impl Universe { pub fn index(&self) -> Box<[u8]>; pub fn packages(&self) -> &[Package]; pub fn package(&self, name: &[u8]) -> Option<&Package>; }
impl Universe { pub fn providers(&self, name: &[u8]) -> &[u32]; }

// testing/hearth-fake-machine/src/hearth/mirror.rs (27.2)
/// A snapshot.debian.org timestamp, `YYYYMMDDTHHMMSSZ`, ordered as time.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Timestamp([u8; 16]);
impl Timestamp { pub fn new(bytes: &[u8]) -> Option<Timestamp>; pub fn as_bytes(&self) -> &[u8; 16]; }
/// A suite's `InRelease` fields; `debian` gives Origin and Label `Debian` and no version.
pub struct Release { pub origin: Box<[u8]>, pub label: Box<[u8]>, pub suite: Box<[u8]>, pub codename: Box<[u8]>, pub version: Option<Box<[u8]>> }
impl Release { pub fn debian(suite: &[u8], codename: &[u8]) -> Release; }
/// One suite at one snapshot: release, architecture, components, their indices and `InRelease` rendered once.
pub struct Suite { /* release, architecture, components, indices, in_release */ }
impl Suite { pub fn new(release: Release, architecture: &[u8], components: Box<[(Box<[u8]>, Universe)]>) -> Suite; pub fn release(&self) -> &Release; }
impl Suite { pub fn universe(&self, component: &[u8]) -> Option<&Universe>; pub fn index(&self, component: &[u8]) -> Option<&[u8]>; pub fn in_release(&self) -> &[u8]; }
/// The `debian` archive: suites by codename at each snapshot.
#[derive(Clone, PartialEq, Eq, Debug, Default)] pub struct Mirror { snapshots: alloc::collections::BTreeMap<Timestamp, alloc::collections::BTreeMap<Box<[u8]>, Suite>> }
/// `suite`: the latest snapshot's at or before `at` (3.6); none before the first, or for another archive.
impl Mirror { pub fn new() -> Mirror; pub fn add(&mut self, at: Timestamp, suite: Suite); pub fn bump(&mut self, rng: &mut skein_lib::Rng, at: Timestamp, per_mille: u32); }
impl Mirror { pub fn suite(&self, archive: &[u8], at: Timestamp, codename: &[u8]) -> Option<&Suite>; }

// testing/hearth-fake-machine/src/hearth/draw.rs (27.2)
/// What a drawn universe holds besides its core: `packages` more, and each of `names`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Draw<'a> { pub packages: u32, pub names: &'a [&'a [u8]] }
impl Universe { pub fn draw(rng: &mut skein_lib::Rng, draw: &Draw<'_>) -> Universe; }


// testing/hearth-fake-machine/src/hearth.rs (hearth 26's; 27.2 adds `served`; hearth 46.1 adds `Tool::Mmdebstrap`)
#[derive(Debug)] #[expect(clippy::zero_sized_map_values, reason = "one tool until hearth 46.1")] pub struct Machine { linux: crate::linux::Machine, programs: alloc::collections::BTreeMap<crate::linux::Node, Tool>, served: Option<(Mirror, Option<u16>)> }
/// `serve`: as snapshot.debian.org on the machine's network, and as a proxy at `127.0.0.1:<port>` if given;
/// `reach`: the mirror a request reaches, directly with no proxy, through a proxy only at the port served.
impl Machine { pub fn serve(&mut self, mirror: Mirror, proxy: Option<u16>); pub fn unserve(&mut self) -> Option<Mirror>; }
impl Machine { pub fn reach(&self, proxy: Option<u16>) -> Option<&Mirror>; }

// testing/hearth-fake-mmdebstrap/src/resolve.rs (27.3)
use alloc::boxed::Box;
use hearth_fake_machine::hearth::universe::Universe;
/// mmdebstrap's variants (3.6): Essential packages; then apt; then required, important or standard and those above.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Variant { Essential, Apt, Required, Important, Standard }
/// One resolution: the source lines' universes in order, the variant, `--include`'s names in order.
#[derive(Debug)] pub struct Request<'a> { pub universes: &'a [&'a Universe], pub variant: Variant, pub include: &'a [Box<[u8]>] }
/// A package of the closure: its universe's place in the request, its own in that universe.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Selected { pub universe: u32, pub package: u32 }
/// The field a broken relation stands in: Pre-Depends, Depends, Conflicts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Field { PreDepends, Depends, Conflicts }
/// A relation the closure breaks: the package, the field, the relation as the index writes it, and the version of the marked package a Conflicts names (not a provided name).
pub struct Broken { pub package: Box<[u8]>, pub field: Field, pub relation: Box<[u8]>, pub against: Option<Box<[u8]>> }
/// No closure: a name nothing carries; a virtual name with several providers; the relations broken.
pub enum Failure { Unknown(Box<[u8]>), NoCandidate(Box<[u8]>), Unsatisfiable(Box<[Broken]>) }
pub const MOST_BROKEN: u32 = 64;
/// The closure in marking order (Choices).
pub fn closure(request: &Request<'_>) -> Result<Box<[Selected]>, Failure>;
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth ls-tree -d --name-only origin/main
   testing/hearth-fake-mmdebstrap | wc -l` prints 1.
2. In a worktree at `origin/main`, `heavy cargo nextest run -p
   hearth-fake-machine -p hearth-fake-mmdebstrap` passes, its `Summary`
   naming at least 51 tests (hearth 26's 24 there, this session's 27).
3. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-27\.[1-3]'` prints 3.
