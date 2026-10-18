# Session 29: render's golden repositories

render.md, 9.2: seven repositories under `tests/render/<name>/`, each its
files and the printed text it must give. 29.1 makes the harness and the
`merge` golden; 29.2 `templates`; 29.3 `fleet` and `cluster`; 29.4 `exports`
and `errors`; 29.5 `resolution`. Each golden runs on the real loop through
real git (the printed text is the client's, hearth 10's printer), again
through 28.3's protocol world, and its digests are cross-checked in render's
domain world. No production code is
planned: what a golden shows missing is fixed in the session that owns it
(Choices, 8).

## Read first

- hearth's `render.md`: 4.2, 7.1 to 7.3, 9.1, 9.2, 9.7; `format.md`: 3.1,
  3.4, 3.6, 3.7, 4, 7.2 to 7.7, 8.6; `client/README.md`: 3.2, 3.3.
- This plan: `hearth/11-thin-story.md` (`THIN`, `Repository::make`,
  `render`), `32-static-binary.md` (32.1's `THIN_TEXT`, the text of a
  `config` part), `40-client-diff-errors.md` (40.2, the errors' lines), `20-exports.md`
  (20.3's `VPN`), `21-reserved.md` (21.4's `hearth` task), `22-collisions.md`
  (22.1's order and bound), `24-resolution.md` (24.1's `fixtures::lay`,
  `test_min_recorded`); `08-mirror.md` (`testing/repos/test`).

## What the design asks

- **9.2:** "each a repository's files and the expected printed spec of each
  target (`expected/<target>`), or the expected errors"; "one test in the
  real tier builds a real repository from it and renders through real git";
  "`HEARTH_BLESS=1` rewrites the expected files and still fails, so a
  blessed change is always reviewed".
- **The seven:** `fleet` (x1-prod, rhi-prod, ion-prod, carbon-prod,
  carbon-lab, chat-prod), `merge`, `templates`, `exports` ("the VPN, the
  mail pair, two environments"), `cluster` ("stoa's members"), `errors`
  ("one case per kind of 7.2"), `resolution` ("closures from the fake
  universe").
- **9.7:** os's cases are "rewritten in hearth's format where the case still
  exists"; expected output "is hearth's, reviewed", never os's.

## Choices this plan makes

1. **Three tiers, one directory.** Process: `tests/builder/tests/goldens.rs`
   builds a real repository from the directory (`Repository::make`) and
   runs `fixtures::render(.., &[], Form::Text)`. Joint: the same test plays
   the files through 28.3's `protocol::run` (the real printer, fake git) and
   compares its text with `expected/`. World: `tests/render/tests/goldens.rs`
   plays the directory through `parent::run` for four seeds and checks each
   target's spec digest against the expected `spec` line (that world prints
   no text).
2. **Layout.** `tests/render/<name>/` holds the repository's files, plus
   `expected/` (never in the repository): `expected/<target>` the text of
   that target's spec; or `expected/output` the whole standard output
   (the `errors` cases); `expected/exit` the code when not 0. A name with a
   slash (`errors/decode`) is a golden of its own. `rendered/<target>/<path>`
   (an absolute path below it) holds the bytes the spec's file must have.
3. **Witnesses.** A spec shows a file by 12 hex digits; a reviewer cannot
   read a digest. The harness hashes each `rendered/` file with `skein_lib::
   sha256::digest`, and the file's line in that target's spec must end with
   those 12 digits. They are written by hand, from the design's rules, never
   blessed: the one expected value no hearth code computed. A link's line
   prints its target, so its witness is that target text; an empty file's
   prints `e3b0c44298fc` (render.md 9.2): `fleet` has one of each.
4. **Bless.** `HEARTH_BLESS` set to `1` alone (`asked`, by `var_os`) rewrites
   `expected/` from the run, then fails with a message naming each file
   changed. It never touches `rendered/`. This is the goldens' rewrite;
   `bless`, the declaration's key, is M3 (format.md, 5.13; hearth 19's Choices 1).
5. **A golden's targets are all of its repository's** (no names given): the
   run is the one a user makes. `expected/<target>` is the stdout cut at
   each line starting `target ` and named by its second word.
6. **Every repository has `tasks/hearth/task.json` `{}`** (hearth 21.4),
   but `fleet`'s, which declares format.md 5.12's `snapshots` setting alone,
   so that 3.7's `hearth.snapshots` defaults pass (ruling 191);
   `fleet.json` has `debian` (hearth 17.1); neither is counted below.
7. **A review, not a trust.** The first bless of each file is read against
   the design lines it cites (render.md 9.2 and 7.3's forms, the modes,
   owners and counts this file states, each `rendered/` file). The report
   lists the files blessed, one line each; a line the design shows is
   asserted verbatim (apart from digests, `mask12`) only in the tests below
   that name a `const`.
7a. **The errors' wording** is 21's, 22's and the others', within 7.3's forms;
   a change of it re-blesses. A repository error fails the targets of its task
   or base (`ok-prod` renders): render.md 3.1, 7.1.
8. **What a golden finds missing** is recorded as a question to the owner:
   the report names the session; a fix of under 20 lines goes in (README,
   "Edits outside Files") with a test in that session's file, else it asks.
9. **The resolution is recorded** (29.5): a laid record, as hearth 24.1's
   process tests lay one; the fake universe's closures meet apt's in
   conformance (hearth 46.2), not here (render.md 9.2: "from a recorded
   closure", printed with its `keys` line).
10. **Time:** 18 renders (merge, templates, fleet, cluster, exports; resolution's
   two variants; 11 error cases) at 0.15 s each is 2.7 s of the 3 s; a case's
   process, joint and world legs share its 0.15 s. `errors/tool` and
   `errors/resolution` (29.4) render through `protocol::run` alone, no
   process: their two joint legs share the 0.3 s left.

## Rules that bind it

- **The meeting point** (README): every golden proves at the process layer.
- **No expected value computed by the code under test:** `rendered/` is
  by hand; a digest in `expected/` is blessed and reviewed (Choices 7).
- **Shared files:** `testing/repos/` (hearth 08; 29.5 reads it, adds
  nothing), `tests/render/src/fixtures.rs`, `tests/builder/src/fixtures.rs`.
  Test time: all seven goldens add at most 3 s to the focused suite (Choices, 10).

## Increments

### 29.1 The harness; the `merge` golden
Scope: `golden` (load a directory, compare, bless) in both worlds; `merge`
(format.md, 4: presence, defaults, removal, maps, lists).
Needs: hearth 21 (21.4: every repository has `tasks/hearth`), hearth 15,
hearth 28.3 (`protocol::run`).
Files: new `tests/render/src/golden.rs`, `src/lib.rs`,
`tests/render/tests/goldens.rs`, `tests/render/merge/**`; the root
`Cargo.toml` (`hearth-render-world = { path = "tests/render" }` in
`[workspace.dependencies]`); `tests/builder/Cargo.toml` (hearth-render-world
in `[dependencies]`, not dev: `src/fixtures.rs`'s `files_of` names
`golden::Leaf`), `tests/builder/src/fixtures.rs`,
`tests/builder/src/fixtures/worlds.rs` (`Scenario`, `thin`, `Shared::build`,
`for_scenario`; its literals at :160 and :174),
`tests/builder/src/fixtures/worlds/protocol_world.rs` (the literal at :21),
new `tests/builder/tests/goldens.rs`.
Shape: the sketches. `golden::load(name) -> Golden` reads the directory
(`std::fs` is allowed in tests; symbolic links kept as links, the execute
bit as `Kind::Executable`); `check` and `bless` as the sketch read
`expected` (`expected-<variant>` for a variant). The fixtures' helpers
(builder's `src/fixtures.rs`): `exit_code(&Rendered) -> Option<u8>`, the
process's code (`fixtures::render`, :147, returns `Rendered`); `Laid::Resolution(..)`, a record laid in the store (24.1's
`fixtures::lay`); `files_of`, a golden's leaves as 11.3's `File`s;
`scenario_of(&[File], &[Laid], Option<Fault>) -> Scenario` (a `Scenario`
borrows its files, so it takes them, not the `Repository`), the call of
a golden through 28.3; `text(&protocol::Run) -> String`, its stdout. The
joint leg sees the laid records: `Scenario` gains `laid: &'a [Laid]`
(`&[]` in `thin()` and each literal without `..`), and `Shared::build`
takes it after `files` and lays each entry into the fake machine's
`/srv/store/<name>`, by a `laid_files` that `lay` (`:448`) shares;
`Shared::new` passes `&[]`, `for_scenario` `scenario.laid`. A witness's hex digits fold into a `String` with `write!`, not
`map` and `collect::<String>()` (clippy's `format_collect`).
`merge`: base `m`, environments `a`, `b`, `c`; `fleet.json`
holds defaults of the task `probe` (a map `limits` of two ints, a list
`ports`, a bool, a string) and a task `gone`; `hosts/m/m.json` overrides one
map key, replaces the list; `m-a.json`
removes `gone` with `null`; `m-b.json` sets a map key to `null` (the key
leaves the map); `m-c.json` replaces the list by `[]`. `tasks/probe`
renders `/etc/probe` as one `key=value` line a setting (`items`, sorted
keys), so that each merged value shows in the file; `rendered/m-<env>/etc/
probe` holds the expected lines. `tasks/gone` has a file `/etc/gone`.
Test shape: unit in `tests/render` (under 20 ms, no repository):
`bless_is_asked_by_the_variable_set_to_1_alone` (`asked` of
`Some(OsStr::new("1"))` true; `None`, `"0"`, `""`, `"11"` false),
`a_spec_text_is_cut_into_its_targets_at_each_target_line`,
`a_witness_must_match_its_files_line_to_twelve_digits`,
`a_golden_without_an_expected_file_is_a_failure_naming_it`. World
(`tests/render/tests/goldens.rs`, 4 seeds, under 0.15 s a golden):
`merge_digests_do_not_depend_on_arrival_order` (every seed gives the digest
of each `spec` line). Process (`tests/builder/tests/goldens.rs`, under 0.15 s
a golden), the reference:
```rust
// tests/builder/tests/goldens.rs
#[test]
fn merge_renders_what_its_expected_files_and_witnesses_say() {
    check("merge", "", &[]);
}
fn check(name: &str, variant: &str, laid: &[Laid]) {
    let golden = golden::load(name);
    let files = fixtures::files_of(&golden.tree);
    let repository = Repository::make(name, &files);
    fixtures::lay(&repository.scratch().path().join("store"), laid);
    let out = fixtures::render(&repository, &[], Form::Text);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let verdict = golden.check(variant, &stdout, exit_code(&out));
    if golden::bless() {
        golden.bless(variant, &stdout, exit_code(&out));
        panic!("blessed: {}", verdict.changed());
    }
    let joint = protocol::run(&fixtures::scenario_of(&files, laid, None), 0); // 28.3: its text equals `expected/`
    assert!(golden.check(variant, &fixtures::text(&joint), joint.code).is_clean(), "joint");
    assert!(verdict.is_clean(), "{}", verdict.report());
}
```
Then `merge_files_show_each_layers_value_in_its_witness` (the witness check
alone, so a failure names the file), `merge_removal_by_null_leaves_the_task_out_of_an_env`.
Proof: process, `merge_renders_what_its_expected_files_and_witnesses_say`:
three targets through real git print their `expected/` and their witnessed
files agree with the digests; joint, 28.3's protocol world prints the same text. Bound: 0.15 s.
Don't: bless in CI (the variable unset is the gate's); compute a witness
with hearth code; read the golden from `testing/`.
Size: no counted lines; the harness about 200 lines of test code.
Removes: nothing.

### 29.2 The `templates` golden
Scope: format.md 7.2 to 7.6 as bytes: each statement, filter and whitespace
rule (render.md, 9.7: os's cases inspire, they are not an oracle).
Needs: hearth 15 (15.2 evaluates), hearth 21.
Files: `tests/render/templates/**`, `tests/builder/tests/goldens.rs`.
Shape: one base `t`, environment `prod`, task `syntax`, a settings
declaration with a string, an int, a bool, a list of strings, a map of
ints, an optional string absent. `tasks/syntax/tree/etc/syntax/<case>.j2`,
one case a file, each with its `rendered/t-prod/etc/syntax/<case>` witness
(the `.j2` suffix leaves the destination, hearth 15). Cases: `if-elif-else`,
`for-list-loop-vars` (`index`, `index0`, `first`, `last`, `length`),
`for-items`, `set-in-for-and-after`, `raw`, `comment`, `trim-blocks` (a tag
alone on its line leaves nothing), `lstrip-blocks`, `dash-markers`,
`plus-markers` (7.2's `mynetworks` line), `trailing-newline-kept`,
`filters-text` (`join`, `split`, `replace`, `first`, `last`, `length`),
`filters-order` (`items`, `keys`, `values`, `sort`), `tojson`,
`shell-quote` (a value with `'`), `for-map-keys` (`for k in map`: keys in order, 7.3), `systemd-escape` (a path
with `-`, a non-ASCII byte and a leading `.`), `systemd-escape-path` (`/dev/nvme0n1p3`, `/` alone),
`basename-dirname` (`/`, `/a`, `a`, `a/b`: 7.5's edges, 43's), `precedence` (`and`, `or`, `not`, `in`, `~`, a
conditional expression), `undefined-tested` (`is defined`, `default`, as an
`if` condition). Expected values are written from 7.2 to 7.6. The spec lists
`config` only.
Test shape: process, under 0.15 s:
`templates_render_byte_for_byte_as_their_witnesses_say` (the golden),
`templates_cover_every_statement_and_filter_format_md_names` (a unit: a
`const` list of 7.3's five statements and 7.5's 17 filter names, `default`
to `dirname`; each occurs in some `.j2` of the golden); world:
`templates_digests_do_not_depend_on_arrival_order`.
Proof: process, the first test: every case's bytes are the witness's, by
the digest on its line. Bound: 0.15 s.
Don't: a case 7.3 or 7.4 forbids (that is `errors`); a witness copied from
a bless; MiniJinja (hearth 30's oracle compares generated templates).
Size: no counted lines. Removes: nothing.

### 29.3 The `fleet` and `cluster` goldens
Scope: the miniature fleet (format.md 3.7 and the fleet's six targets) and
stoa's members (3.6).
Needs: hearth 22 (collisions fail instead of dropping, so a blessed spec
holds what the design says), hearth 19, hearth 21, hearth 18.
Files: `tests/render/{fleet,cluster}/**`, `tests/builder/tests/goldens.rs`.
Shape: `fleet`: `fleet.json` and tasks as format.md 3.7 gives x1-prod
(forky, `20261001T030405Z`, contrib and non-free-firmware, no pockets;
tasks core, host, locale, boot, users, wifi, desktop, firmware, `hearth`;
`fleet.json`'s `hearth.snapshots` as 3.7 writes it, and `tasks/hearth/task.json`
`{"settings": {"snapshots": …}}`, the declaration 5.12 writes, copied from
`tests/builder/src/fixtures/reserved/hearth.json:18-25`: ruling 191),
each task a few entries, so that every part prints. Targets: `x1-prod`
(metal, efi, 7th-gen); `rhi-prod` (metal, bios; secrets, a download, a
unit that may fail, `live`); `ion-prod` (vm, bios, ionos; identity);
`carbon-prod` (metal, efi); `carbon-lab` (vm, efi, incus; the same base
file as carbon-prod, a different base-env file); `chat-prod` (a container
on `carbon-prod`: no `hearth` task, 21.4's exemption; variant `container`,
its packages `needed` with no error, `boot` with empty `params`: render.md
4.2, 5.7). Every part prints in every spec, an empty one with empty fields
(4.1). Some entries
carry `if` (one false, one true) and a `for_each` with interpolated keys
(hearth 16.2), and one task a true `requires`. Parts exercised: packages with preseed and initramfs, files (static, template,
executable, a link), users, groups, presets, tmpfiles, data, scripts,
secrets and identity (mode within 0770), downloads, live, boot
parameters, placement kinds. `x1-prod` carries 9.2's lines: its test
`x1_prod_prints_render_md_9_2s_lines` compares `expected/x1-prod` with a
`const` of that block (hostname file 0644, `/etc/localtime link root:root
/usr/share/zoneinfo/Europe/Lisbon`, `tmpfiles /var/log/journal d
2755 root systemd-journal`, `journal /var/log/journal`, the sources,
preseed and initramfs lines, request words as the fleet declares them),
digests masked by `mask12`. `cluster`: base `stoa` with members 1, 2, 3 as
3.6 shows, environments `lab` and `prod`; `stoa-prod.json` removes member
3 (`"3": null`); a task `stoa` exports `address` and `index`
and its `/etc/stoa/peers.j2` loops over `exports.stoa` and `target.member`.
Targets `stoa-1-lab`, `stoa-2-lab`, `stoa-3-lab`, `stoa-1-prod`,
`stoa-2-prod`; witnesses for the peers file.
Test shape: process, under 0.15 s:
`the_fleet_renders_its_six_targets_as_expected`,
`x1_prod_prints_render_md_9_2s_lines`,
`a_lab_spec_does_not_change_when_a_production_file_does` (a unit-style
process test: `repository.next` edits `hosts/carbon/carbon-prod.json`,
`carbon-lab`'s text is equal; 9.3's locality),
`the_container_has_no_hearth_task_and_its_host_has_one`,
`the_cluster_renders_its_members_with_their_names`,
`a_base_env_file_removes_a_member_from_its_environment`; world:
`fleet_and_cluster_digests_do_not_depend_on_arrival_order`.
Proof: process, `the_fleet_renders_its_six_targets_as_expected`: the six
specs through real git equal `expected/`, the witnessed files agree. Bound:
0.15 s.
Don't: an export in `fleet` (29.4's); a task whose output depends on the
clock; a secret value (none exists, format.md 9.1).
Size: no counted lines (a fix of a missing record field is hearth 10's or 40's,
at most 20 lines). Removes: nothing.

### 29.4 The `exports` and `errors` goldens
Scope: format.md 8 across targets; render.md 7.2's kinds, each a golden.
Needs: hearth 28.3 (`Fault::KilledAt`), hearth 46.1 (an unlocatable package),
hearth 20 (20.3: `exports.<task>`, `fixtures::VPN`), hearth 22
(22.1: collected, sorted, bounded; 22.3 for the collision), hearth 19
(`ids`), hearth 21 (`reserved`).
Files: `tests/render/exports/**`, `tests/render/errors/<case>/**`,
`tests/builder/tests/goldens.rs`, `tests/builder/src/fixtures.rs` (a
list of the error cases).
Shape: `exports`: hearth 20's `VPN` files, byte for byte (test
`the_exports_golden_holds_hearth_20s_vpn_files`), and the mail pair: bases
`mx1` and `mx2`, environments `prod` and `lab`, task `mail` reading
`mail` exports (`role`, `address`) and writing `main.cf` with 7.2's
`mynetworks` line (`+%}`); `rendered/` witnesses for `peers`, `mtu` and
`main.cf` of each target. A peer is a reader's of its own environment
alone (20.3): the lab and prod specs differ in their witnessed peers.
`errors/<case>`: a repository of a base `ok` (renders, so that the rest
goes on) and a base `bad` (`bad-prod` carries the fault). Cases, a kind
each: `decode` (a duplicate key and a fraction in `hosts/bad/bad.json`),
`repository` (a file in a task outside `task.json` and `tree/`), `composition`
(`null` for a task in `fleet.json`), `schema` (an undeclared setting
that lists the declared, 7.3), `evaluation` (an undefined value in a
template), `guard` (a false `requires`), `reserved` (the `rootwait` boot
parameter), `ids` (a data path whose owner has no fixed id), `collision`
(one tree path from two tasks), `exported` (hearth 20's `VPN_BAD`
reader and exporter); `sorted`: three faults in two files and two
targets, whose order is path, line, column, target (22.1). `tool` and
`resolution` are builder-world cases, git or mmdebstrap's fake failing
(render.md 9.2). `expected/output` holds the
spec of `ok-prod` then the errors; `expected/exit` is 2. `errors/tool` runs
only through `protocol::run` with `Fault::KilledAt`, `errors/resolution`
with `--resolve` and fake mmdebstrap asked for a package no index carries;
each has `expected/output`; `expected/exit` is 75 for `tool` and 2 for
`resolution`, whose output prints `ok-prod`'s spec first: apt that cannot
locate a package is a configuration cause (render.md 7.1, client/README.md
3.3). `errors/resolution`'s scenario is `fixtures::resolving(&files)`
(`worlds.rs:195`, `resolve: true`), as `scenario_of` sets no `--resolve`.
`ids`'s data path keeps format.md 5.11 (`/srv/<class>/<key>`, or outside
`/srv`): a reserved fault hides its entry from the `ids` reader. Ruling 189.
Test shape: process, under 0.15 s:
`the_vpn_and_mail_golden_render_with_their_peers_per_environment`,
`the_exports_golden_holds_hearth_20s_vpn_files`,
`each_error_case_prints_its_errors_after_the_healthy_spec` (a table over
`ERROR_CASES`, one assertion shape, 0.15 s a case: under 1.7 s),
`errors_are_sorted_by_path_line_column_and_target`,
`every_kind_of_7_2_but_checks_has_a_case` (a `const` list against
`errors/*`), `an_error_case_exits_2`,
`the_tool_and_resolution_errors_print_through_the_protocol_world`;
world: `exports_digests_do_not_depend_on_arrival_order`.
Proof: process, `each_error_case_prints_its_errors_after_the_healthy_spec`:
each fault is printed as 7.3 writes it, at its file, line and column, and
`ok-prod` is rendered whole. Bound: 0.15 s a case.
Don't: errors past the bound (22.1 tests 300); a case with two kinds; the
M5 kinds (`checks`).
Size: no counted lines. Removes: nothing.

### 29.5 The `resolution` golden
Scope: render.md 5.7 and 9.2: a spec `needed`, and `known` with its keys.
Needs: hearth 24.1 (keys, `Find`, `fixtures::lay`, `test_min_recorded`).
Files: `tests/render/resolution/{expected,expected-known}/**`,
`tests/builder/tests/goldens.rs`, `tests/builder/src/fixtures/goldens.rs`,
`tests/builder/src/fixtures/resolution.rs` (ruling 189).
Shape: no files of its own: the repository is `testing/repos/test` (hearth
08.2: base `test`, targets `test-min` and `test-boot`), read in place
(`golden::load_from`). Variant `""` renders against an empty store: each
packages line ends `needed`, no `keys` line. Variant `known` lays
`Laid::Resolution(fixtures::test_min_recorded())` first: `test-min` ends
`known` with its `keys` line (`TEST_MIN_KEYS_LINE`), `test-boot` is still
`needed`. The record is 24.1's `test_min_recorded()` (a closure of `apt`
alone), not the fake universe's (Choices, 9); the print does not show the
closure.
Test shape: process, under 0.15 s:
`an_empty_store_prints_every_resolution_needed`,
`a_laid_record_prints_known_for_its_targets_alone`,
`the_keys_line_is_the_one_hearth_24_pins` (the line equals
`TEST_MIN_KEYS_LINE`, a literal). `TEST_MIN_KEYS_LINE`
(`fixtures/resolution.rs:10`, unused on main) predates cce697a's keys:
29.5 refreshes it to the line `tests/builder/tests/resolution.rs:11` pins,
`keys         packages d018a0eea488   software f60610453970   config
8c9bf06f6262`. `fixtures::check_golden` (`goldens.rs:35`) loads by name:
29.5 splits its body into a function over a loaded `Golden`, which
`check_golden` calls, so that `load_from`'s golden takes the same checks
(ruling 189).
Proof: process, `a_laid_record_prints_known_for_its_targets_alone`: with
the record laid, `test-min` is `known` with its keys and `test-boot`
`needed`, exit 0. Bound: 0.15 s.
Don't: run mmdebstrap; compute a key.
Size: no counted lines. Removes: nothing.

## Interface sketches

```rust
// tests/render/src/golden.rs (29.1): ordinary Rust; `Vec`, `String` and std::fs allowed
/// One file of a golden's repository, as git stores it.
pub struct Leaf { pub path: Vec<u8>, pub bytes: Vec<u8>, pub kind: Kind }
/// 100644, 100755, 120000 (its bytes the link's target).
pub enum Kind { File, Executable, Link }
/// A golden's repository, its expected text and its witnesses.
pub struct Golden { pub name: String, pub tree: Vec<Leaf>, root: PathBuf }
/// How a run compared with its expected files.
pub struct Verdict { /* files differing, witnesses failing, files missing */ }
#[must_use] pub fn load(name: &str) -> Golden;               // under CARGO_MANIFEST_DIR; `expected/` and `rendered/` apart
#[must_use] pub fn load_from(name: &str, files: &Path) -> Golden; // another directory's files (29.5)
#[must_use] pub fn bless() -> bool;                          // `asked(std::env::var_os("HEARTH_BLESS").as_deref())`
#[must_use] pub fn asked(value: Option<&OsStr>) -> bool;     // true for "1" alone
#[must_use] pub fn mask12(text: &str) -> String;             // every run of 12 lowercase hex digits to twelve dots
impl Golden {
    #[must_use] pub fn check(&self, variant: &str, stdout: &str, exit: Option<u8>) -> Verdict; // `expected/exit` is 0 when absent
    pub fn bless(&self, variant: &str, stdout: &str, exit: Option<u8>);
    #[must_use] pub fn targets(stdout: &str) -> Vec<(String, String)>; // cut at each `target ` line
    #[must_use] pub fn spec_digest(&self, variant: &str, target: &str) -> Option<[u8; 6]>; // for the world's cross-check
}
impl Verdict { #[must_use] pub fn is_clean(&self) -> bool; #[must_use] pub fn report(&self) -> String; #[must_use] pub fn changed(&self) -> String; }
// tests/builder/src/fixtures.rs (29.1): `files_of(&[Leaf]) -> Vec<File>` by 11.3's `File` (hearth 18's `link`); `exit_code(&Rendered) -> Option<u8>`; `scenario_of(&[File], &[Laid], Option<Fault>) -> Scenario`; `text(&protocol::Run) -> String`; `ERROR_CASES: &[&str]` (29.4)
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-29\.[1-5]'` prints 5.
2. `git -C ~/src/rust/hearth ls-tree --name-only origin/main tests/render/ |
   grep -c -x -E 'tests/render/(fleet|merge|templates|exports|cluster|errors)'`
   prints 6, and `git -C ~/src/rust/hearth ls-tree -r --name-only origin/main
   tests/render/resolution | grep -c expected` prints at least 2.
3. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run
   -p hearth-render-world -p hearth-builder-world -E 'test(/golden|merge|templates|fleet|cluster|exports|errors|resolution/)'` passes, and
   `HEARTH_BLESS=1 heavy cargo nextest run -p hearth-builder-world -E
   'test(/^merge_/)'` fails with "blessed" while `git status --porcelain
   tests/render` prints nothing.
