# Session 22: collisions, render's additions, errors collected, sorted, bounded

render stops being silent where tasks meet. 22.1 is hearth 12's held errors
made 7.1's collection: sorted, at most 256, then a count (ending the terminal
record's `left_out` 0). 22.2 writes what tasks may not (3.6): `/etc/subuid`,
`/etc/subgid`, data's `d` tmpfiles lines, `tiny-initramfs.conf`. 22.3 refuses
paths two tasks claim; 22.4 declarations that must agree, ids and ranges. Each
proves on the real loop's `hearth render` or `hearth diff` through real git.

## Read first

- hearth's `format.md`: 5.4, 5.5 (forbidden destinations), 5.6 (ids,
  `subordinate`), 5.7; `render.md`: 2.3 (last row), 3.6, 3.7, 4.2, 7.1 to 7.3;
  `machine.md`: 7.1; `build.md`: 5.5.
- This plan: `hearth/12-compose.md` (Choices; the `errors.rs` sketch);
  `16-entries.md` (Choices; `Expansion`, `Located`, `Collision::Key`);
  `17-packages.md` (17.3, 17.4); `18-tree-users.md` (Choices 8; 18.1, 18.3 to
  18.5; sketches); `19-secrets-software.md` (Choices 4; 19.1, 19.2);
  `40-client-diff-errors.md` (40.2); `11-thin-story.md` ("Transitional"); `15-files.md`
  (15.3); `24-resolution.md` (24.1); `45-diff-errors-resolve.md` (45.1).
- skein at `841f487`, hearth's pin: `skein-lib`'s `map.rs` (`Map`:26,
  `insert`:90, `remove`:105, `last`:121; its `Hash` by hand, :151).

## What the design asks

- **Errors** (7.1; 2.3): "Errors are sorted by path, line, column and target,
  and bounded"; "errors kept from one render | 256, then a count".
- **Additions** (3.6): "`/etc/subuid` and `/etc/subgid` from the users'
  subordinate ranges, a `tmpfiles` line of type `d` for each data entry's mode
  and owners ..., and the initramfs configuration (4.2)": `AUTO_MODULES=no`,
  `INCLUDE_MODULES`, `MICROCODE=no` or `generic`.
- **Collisions** (3.7): "A collision is an error that names both tasks and
  both lines." Tree path, data name and path, secret, identity and download
  paths: "one task only"; users, groups, presets, tmpfiles lines, preseeds, boot
  parameters: "several tasks only identically"; a uid or gid one user's;
  "ranges never overlap"; packages, modules, `live` paths: a union.

## Choices this plan makes

1. **One collector.** `errors::Collector` (a `skein_lib::Map` of the kept
   errors keyed by their order) replaces 12's list. A pushed error that
   sorts past the 256th kept is counted, not kept. The fold of an error's
   class into the outcome stays 12's, at the push: an error left out still
   decides it. `Rendered` and the root's `Reply` gain `left_out: u32`. A
   diff's two sides push into one collector (45.1): one bound, one count,
   printed `<n> more errors` (render.md 7.1, 7.3).
2. **The order** is 7.1's: errors with a place by path (bytes), line, column,
   target (none first); then those with none (`at: None`: 24.1), by
   target; ties in arrival order, which is the answers' order. 45.1 adds
   `side` after the target.
3. **Collisions run once a target,** after its kinds and before its parts, on
   rows every task gave (kinds keep all; the first-wins drops of 17 to 19 go)
   and return the surviving rows. A fault in one task's entry gives no rows to
   compare; templates running or failing change nothing here.
4. **`at` is the first claimant, `also` a later one:** tasks in name order,
   keys in key order, as 7.3's line (hardening, then ssh). Three claimants
   give two errors, each against the first. A claim by a tree file has the
   file's path and no position; by an entry, `task.json` at the entry's key.
   `Hidden` is `at` the data entry and `also` the claim beneath it, whatever
   the tasks' names: its `first` is the data entry's task.
5. **Kinds:** `Collision` gains `Given` (one name, path or id claimed twice),
   `Differs` (one key, differing rows), `Hidden` (under a data path),
   `Shared` (an id), `Overlap` (a range, a download, a data path); `Subject`
   says what. `Key` stays 16.2's.
6. **"Identically"** is field for field, once defaults are filled; positions
   and tasks are left out (render.md 3.7); a user's `groups` and `password`
   are part of its row; a boot parameter is keyed by its entry key, and its
   row is `(order, param)`.
7. **Additions are render's, not a task's:** `additions.rs`; the files go
   to the cache by 15.3's `Keep` with the destination (`/etc/subuid`: a
   leading `/` no revision path has) as `path`; tmpfiles lines join the
   tasks' before 22.4 compares them. A data line is its entry's task's.
8. **Messages** follow render.md 7.3's form: `<name> is given by A and by B`,
   with a subject word unless a tree path; `<subject> <name> differs between A
   and B`; `uid 1000 is shared by A and B`; `<path> is under the data path
   <data> (a mount would hide it)` (22.3's proof); `<subject> <first>
   overlaps <second>` for paths (`data path /srv overlaps /srv/www`), and
   `subordinate ids of alice overlap bob's` for a range (render.md 3.7:
   "paths collide when one is the other or lies beneath it", "ranges never
   overlap").

## Rules that bind it

- **The meeting point** (README, Rules): every increment proves on the real
  loop; a variant added to `errors` adds the records' and both arms of
  `translate/errors.rs` the same increment. No recursion, no `Vec`, `String`
  or `_` arm; tables are `Map`s of `Limits::expansion`, a full one 16.2's
  `Limit` at the entry; no expected value computed by the code (digests from
  `sha256sum`).
- **Shared files:** `errors.rs`, `vocabulary.rs`, `call.rs`, `limits.rs`, both
  fixtures modules, `translate/`, the records' `errors.rs` and `records.rs`.

## Increments

### 22.1 The collector: sorted, bounded, counted
Scope: render.md 7.1 (last bullet), 2.3 (last row); `Collector`,
`Limits::errors`, `left_out` through the root to the terminal record.
Needs: hearth 12 (its held errors; `Answer::Error`).
Files: render `src/{errors, call, limits, vocabulary, lib}.rs`,
`src/tests/{errors, fixtures}.rs` (`errors.rs:17`, `:40`, `:94` read
`call.errors.as_slice()`: they read `fixtures::kept(&call)`); root
`src/vocabulary.rs`, `src/lib.rs` (`:54-55` build and destructure
`Reply`); protocol `src/client.rs` (`:121`, the Terminal's `left_out:
0`), `src/tests/client.rs` (`:22`, `:43`, `:53`, `:67`: four `Reply`
literals), `src/translate/answers.rs`, `src/tests/translate.rs`;
`tests/render/src/parent.rs` (`:45`, the `Rendered` arm);
`tests/render/tests/errors.rs`; `tests/builder/{src/fixtures.rs,
tests/real.rs}`.
Shape: the sketches. `Call` pushes each error as 12 held it; `resume`
answers the kept ones in order after the last spec, then `Rendered {
outcome, diff, left_out }` (`diff` is 25's, `vocabulary.rs:31`). Each
`Side` holds its `Collector` where it holds `errors: List<errors::Error>`
(`call.rs:20`, made at `:47`), and `errors::push` (`call.rs:110`, `:121`,
`:164`, `:227`, `:234`) takes it. A diff's `Rendered` (`call.rs:479`,
`:543`; `:471` destructures it) carries its sides' counts, summed, the
`from` side's kept when the side is reused for `to` (`:487`): no `left_out:
0` stays (Done when 4), and 45.1 makes the two sides one collector
(Choice 1). The protocol layer sets `Terminal::left_out`, 40.2
prints `<n> more errors` after the last error and puts `left_out` in the
`outcome` line. A full map hands back the new key: it displaces `last()` when
smaller (`remove`, `insert`), else is counted; the count saturates.
Test shape: render unit (`src/tests/errors.rs`, under 20 ms; fixtures gain
`error_at(path, line, column, target)`, `unplaced(target)`, `collector(n)`,
`limits_with_errors(n)`, `kept(&call)`, and `places(&kept) ->
Box<[Spot<'_>]>` with `type Spot<'a> = (&'a [u8], u32, u32, &'a [u8])`:
render's lints forbid `str::from_utf8`, `Vec`, `collect` and `map`, and
clippy's `type_complexity` wants the alias; a `None` position, as
`Place.position` may be, reads `0, 0`):
```rust
#[test]
fn errors_sort_by_path_then_line_then_column_then_target() {
    let mut collector = Collector::new(&fixtures::limits_with_errors(8));
    for error in [fixtures::error_at(b"b.json", 1, 1, b"t"), fixtures::error_at(b"a.json", 2, 9, b"t"),
        fixtures::error_at(b"a.json", 2, 3, b"u"), fixtures::error_at(b"a.json", 2, 3, b"t")] { collector.push(error); }
    let (kept, left_out) = collector.finish();
    let expected: [Spot<'_>; 4] = [(b"a.json", 2, 3, b"t"), (b"a.json", 2, 3, b"u"), (b"a.json", 2, 9, b"t"), (b"b.json", 1, 1, b"t")];
    assert_eq!(*fixtures::places(&kept), expected, "7.1");
    assert_eq!(left_out, 0, "nothing past the bound");
}
```
Then `an_error_without_a_place_follows_the_placed_by_target`,
`equal_errors_keep_their_arrival_order`, `errors_past_the_bound_are_counted_
not_kept` (`errors: 3` beside `SHIPPED`'s 256), `a_smaller_error_displaces_the_
last_kept_one`, `a_left_out_operational_error_still_makes_the_call_try_again`,
`the_collector_never_holds_more_than_its_bound`; protocol `src/tests/client.rs`
(the Terminal is built in `client.rs`, not `translate/answers.rs`)
`rendered_left_out_reaches_the_terminal_record`; world (`tests/render/tests/errors.rs`, eight
seeds, under 0.5 s) `three_hundred_broken_documents_answer_256_errors_then_
the_count`.
Proof: process, `hearth_render_prints_256_errors_then_the_count`, bound 2 s:
`fixtures::broken_envs(300)` adds THIN's files and `envs/e000.json` to
`envs/e299.json`, each `{"motd": {}, "motd": {}}`. `lines_starting(&stdout,
&["envs/", "44 "])` has 257 lines: `envs/e000.json:1:14: duplicate key "motd"
(envs/e000.json:1:2)` first, `envs/e255.json:1:14: ...` the 256th, then
`44 more errors`; exit 2. `hearth_render_json_gives_left_out_44` finds
`"left_out":44`.
Don't: sort in the printer; drop an error from the outcome's fold; hold more
than the bound.
Size: at most 200 lines.
Removes: hearth 11's "Transitional" `left_out` 0 (its Choices; 40.2's field);
hearth 12's list "regrown by doubling" and its "changing none".

### 22.2 What render adds: subordinate ranges, data lines, the initramfs
Scope: render.md 3.6 (last bullet), 4.2; format.md 5.4, 5.6, 5.7;
machine.md 7.1. `additions.rs`.
Needs: hearth 17.3 (`Initramfs`), 18.3 (`Accounts`), 18.4 (`Line`), 18.5
(`data::Entry`), 25.1 (`diff`, for the last proof).
Files: render new `src/additions.rs`, `src/{lib, call, parts/config,
parts/packages}.rs`, new `src/tests/additions.rs`, `src/tests/fixtures.rs`;
protocol `src/translate/parts/packages.rs`, `src/tests/translate.rs`; records
`src/parts/packages.rs`, `src/form/json.rs`, `src/tests/packages.rs`;
`tests/builder/{src/fixtures.rs, tests/real.rs}` (`Keep`, its cache and
translation are 15.3's).
Shape: the sketches. `subordinate` is subuid(5): `<user>:<base>:<count>` LF
a user, by name; the same text is `/etc/subuid` and `/etc/subgid` (5.6: "a
subordinate uid and gid range"), tree files `0644 root:root`, none when no
user has a range. `data_lines` gives each entry a `d` line (its mode, owner,
group) at its path, merged into `Config::tmpfiles` in path order. `initramfs`
is `DEBUG=no`, `AUTO_MODULES=no`, `MICROCODE=<no|generic>`, `INCLUDE_MODULES=`
the modules in bytewise order, comma-joined (machine.md 7.1), the line
`INCLUDE_MODULES=` with no module, LF each; a container has none (17.3). Each
goes to `Keep` after the target's templates, one at a time. The config part
gains the subuid and subgid tree files and the data lines. The initramfs text
goes into the cache, and the spec names its content digest beside the packages
part, outside the part's digest (render.md 4.2): `Packages::initramfs_file:
Option<Digest>` (absent for a container), in neither `items` nor the digest, not
printed in text, JSON's `initramfs_file` beside the part's `digest`. So the part's
digest stands, and so does every spec's (findings/digests.md).
Test shape: render unit (`src/tests/additions.rs`, under 20 ms):
```rust
#[test]
fn subordinate_ranges_become_lines_by_user_name() {
    let accounts = fixtures::accounts_with_ranges([(b"free", 165_536, 65_536), (b"ci", 100_000, 65_536)]);
    let text = additions::subordinate(&accounts).expect("two ranges");
    assert_eq!(&*text, &b"ci:100000:65536\nfree:165536:65536\n"[..], "subuid(5), by name");
}
```
Then `no_subordinate_range_makes_no_file`, `a_data_entry_becomes_a_d_line_with_
its_mode_and_owners`, `the_initramfs_file_has_generic_microcode_on_metal`,
`the_initramfs_file_has_none_microcode_on_a_vm`, `a_container_has_no_initramfs_file`, `modules_are_sorted_
and_comma_joined`, `a_profile_with_no_module_writes_an_empty_include_modules_
line`, `the_initramfs_file_digest_is_beside_the_packages_part_not_in_its_digest`;
protocol `a_render_made_file_is_kept_under_its_destination`,
`the_initramfs_file_digest_translates_to_the_records_and_back`; records
`json_gives_the_initramfs_file_beside_the_packages_digest`.
Proof: process, bound 0.5 s, on `SUBORDINATE` (THIN's files;
`tasks/ci/task.json` `{"users": {"ci": {"subordinate": {"base": 100000,
"count": 65536}}, "free": {"uid": 1000, "subordinate": {"base": 165536,
"count": 65536}}}}`, `x1.json` `{"motd": {}, "ci": {}}`):
`hearth_render_lists_subuid_and_subgid`,
`lines_starting(&stdout, &["  /etc/sub"])` is exactly
```text
  /etc/subgid              file 0644 root:root 474a6f5bdf9c
  /etc/subuid              file 0644 root:root 474a6f5bdf9c
```
Beside it, on `DATA` (18.5's): `hearth_render_lists_a_d_line_for_each_data_
entry` (`lines_starting`, `  tmpfiles `: `  tmpfiles   /var/lib/iwd d 0700
root root`, then `  tmpfiles   /var/log/journal d 2755 root systemd-journal`),
and on `PRESEEDED` (17.3's: modules
`btrfs nvme`, metal) `the_initramfs_configuration_enters_the_cache`:
`<store>/cache/6cb7c9647dfa51a50f2cb619e3644f84e0f908044fb1aeb9187232a98528caa8`
holds the four lines (`MICROCODE=generic`, `INCLUDE_MODULES=btrfs,nvme`), 70
bytes, 0644, and `--json` gives `"initramfs_file":"6cb7c964…caa8"` (the same
64 digits) beside the packages part's `digest`. Process, `hearth_diff_shows_a_data_entrys_new_mode_as_a_tmpfiles_
change` (`DATA_MODE`: journal `0755`; `lines_starting`, `  config `, exit 1)
```text
  config   ~ tmpfiles /var/log/journal              d 0755 root systemd-journal
```
and its JSON holds `"kind":"from_config"` (5.7: mode enters the config key).
Don't: refuse a task's `/etc/subuid` (21); compare lines across tasks (22.4).
Size: at most 260 lines.
Removes: 18's "waits for 22" for `subordinate`, data's `mode` and owners;
17's Choices, "`tiny-initramfs.conf` (22 ...)".

### 22.3 Paths two tasks claim
Scope: render.md 3.7, rows 1 to 5 and "a path in the tree under a data path";
format.md 5.5, 5.7. `collisions::paths`.
Needs: 22.1; hearth 18.1, 18.2, 18.5, 19.1, 19.2.
Files: render new `src/collisions.rs`, `src/{errors, call, lib}.rs`, new
`src/tests/collisions.rs`, `src/tests/fixtures.rs`; protocol `src/translate/
errors.rs`, `src/tests/translate.rs`; records `src/errors.rs`, `src/form/
{text, json}.rs`, `src/tests/errors.rs`; `tests/builder/{src/fixtures.rs,
tests/real.rs}`; and where 18 and 19 drop a duplicate (`// transitional:
hearth 22`).
Shape: the sketches. Claims are tables keyed by path: the tasks' `Install`s
(any node: even identical content collides), data names (across classes) and
paths, secret and identity names and paths (`path`, `public_path`),
downloads. A tree, secret, identity or download path equal to or under a data path is
`Hidden`, `at` the data entry (a tar download under one too); a secret,
identity or download path equal to a tree path, or to another such path, is
`Given` (equal is always `Given`); two data paths equal or nested (5.7: "never
beneath another entry's path"), and a download with a claim strictly beneath
it (a tar covers its subtree: a tree path, a secret's or identity's path or
another download; a file only itself), are `Overlap` (render.md 3.7: paths
collide when one is the other or lies beneath it). Claims of one task collide
too, but two installs of one
task are 18.1's `Destination`: `Given` with `first` == `second`, printed "by A
and by A". Tree-made installs are placed by the tree
file (`tasks/<task>/tree/<path>`, no position), the rest by `Install::at`
(18's, `Option`: `None` for the tree).
Test shape: `src/tests/collisions.rs` (under 20 ms), with the reference:
```rust
#[test]
fn two_tasks_giving_one_tree_path_collide_naming_both() {
    let plans = [fixtures::plan_of(b"banner", &[(b"/etc/motd", Some(fixtures::at(1, 12)))]),
        fixtures::plan_of(b"motd", &[(b"/etc/motd", None)])];
    let errors = fixtures::collide(&plans, &fixtures::no_expansion());
    assert_eq!(fixtures::collisions(&errors), [("banner", "motd", "/etc/motd")], "7.3: the first by name at, the other also");
}
```
Then `identical_content_in_two_tasks_still_collides`, `three_tasks_on_one_path_
give_two_errors_against_the_first`, `a_data_name_in_two_classes_collides`, `a_
tree_path_under_a_data_path_is_hidden`, `a_secret_at_a_tree_path_collides`, `a_secret_identity_or_download_under_a_
data_path_is_hidden`, `a_tar_download_covering_a_tree_file_a_secret_or_an_identity_path_overlaps`, `two_data_
paths_equal_or_nested_overlap`, `a_secret_and_a_tree_file_of_one_task_collide_
naming_it_twice`,
`a_download_inside_a_tar_downloads_directory_overlaps`, `a_file_download_
covers_only_itself`; records `a_collision_prints_both_tasks_and_both_places`.
Proof: process, bound 0.5 s, `TREE_TWICE` (THIN's files;
`tasks/banner/task.json` `{"files": {"/etc/motd": {"copy": "banner.txt"}}}`
and `banner.txt` at the repository's root, where `copy` names its path
from (format.md, as 18's `lab-ca/lab.crt`); `x1.json` `{"motd": {}, "banner": {}}`):
`hearth_render_names_both_tasks_of_a_path`,
`lines_starting(&stdout, &["tasks/banner/"])` is exactly
```text
tasks/banner/task.json:1:12: x1-prod: /etc/motd is given by banner and by motd (tasks/motd/tree/etc/motd)
```
exit 2; `HIDDEN` (`tasks/app/task.json` `{"data": {"app": {"path": "/var/lib/
app"}}}`, `tasks/dump/tree/var/lib/app/seed`, `x1.json` `{"motd": {}, "app":
{}, "dump": {}}`): `hearth_render_refuses_a_file_under_a_data_path` finds

```text
tasks/app/task.json:1:11: x1-prod: /var/lib/app/seed is under the data path /var/lib/app (a mount would hide it) (tasks/dump/tree/var/lib/app/seed)
```
Don't: check reserved paths (21); report a path twice for one pair.
Size: at most 300 lines.
Removes: the first-task-wins drops of tree paths, secrets, identity items and
downloads (18 Choices 8; 19 Choices 4).

### 22.4 Declarations that must agree; ids and ranges
Scope: render.md 3.7, rows 6 and 8; format.md 5.6 (ids, ranges, package and
sysusers of one name). `collisions::{declarations, ids}`.
Needs: 22.2, 22.3; hearth 17.3, 17.4, 18.3, 18.4.
Files: render `src/collisions.rs`, `src/{errors, call}.rs`, `src/tests/
{collisions, fixtures}.rs`; protocol and records as 22.3's; `tests/builder/
{src/fixtures.rs, tests/real.rs}`; where 17 and 18 collapse their rows
(`// transitional: hearth 22`).
Shape: users and groups by name, presets by unit, tmpfiles by path (render's
data lines too), preseeds by question, boot parameters by entry key: rows of
several tasks must be equal (`Differs`, `at` the first, `also` the first that
differs); equal rows make one, and `accounts` returns the survivors, so 19.4
sees one row a name. A package and a sysusers declaration of one name differ.
A sysusers user's own group is a group row of the user's name with its `uid`
as `gid` (format.md 5.6): a `groups` entry of that name with another `gid`
`Differs` (subject `Group`, one task's two claims too), the same `gid` is one
row. A package user gives both ids or neither; with neither it takes none.
`ids`: two users with one uid, two groups with one gid (`Shared`); a user's
own group takes its uid in the gid space (format.md 5.6: "made with the same
id"), a package user's `gid` is a gid; an own group meets declared groups and
package gids only, since two own groups sharing an id is their uids' error
alone; subordinate ranges `[base, base + count)` of two users are
`Overlap` when they share any id, fine when they only touch.
Walks sort the ids and ranges once (a `Map`), then compare neighbours.
Test shape: `src/tests/collisions.rs`, under 20 ms: `equal_users_of_two_tasks_
make_one_row`, `a_user_two_tasks_declare_differently_collides_naming_both`,
`a_package_user_and_a_sysusers_user_of_one_name_collide`,
`a_groups_entry_of_a_users_own_name_with_another_gid_differs_and_with_its_uid_is_one_row`,
`two_users_with_one_
uid_collide`, `a_users_own_group_takes_its_uid_in_the_gid_space`, `overlapping_
subordinate_ranges_collide`, `adjacent_ranges_do_not`, `differing_preseeds_of_
a_question_collide`, `a_boot_key_with_two_parameters_collides`, `a_data_line_
and_a_tasks_differing_line_at_one_path_collide`; records `a_shared_uid_prints_
both_users`. Real, on `CLASH` (`tasks/a/task.json` `{"users": {"free": {"uid":
1000}}}`, `tasks/b/task.json` `{"users": {"free": {"uid": 1001}}}`, `x1.json`
`{"motd": {}, "a": {}, "b": {}}`), `hearth_render_names_the_two_tasks_of_a_
differing_user`, under 0.5 s, `lines_starting(&stdout, &["tasks/a/"])` is
exactly
```text
tasks/a/task.json:1:12: x1-prod: user "free" differs between a and b (tasks/b/task.json:1:12)
```
Proof: process, `hearth_render_names_the_two_users_sharing_a_uid` (`SHARED`:
`b` declares `"ci": {"uid": 1000}`), bound 0.5 s, the same filter, exit 2:
```text
tasks/a/task.json:1:12: x1-prod: uid 1000 is shared by free and ci (tasks/b/task.json:1:12)
```
Don't: refuse reserved ids or names (21); check fixed ids (19.4).
Size: at most 300 lines.
Removes: the first-wins drops of users, groups, presets, tmpfiles (18 Choices
8), preseeds and parameters (17 Choices).

## Interface sketches

Types derive `Clone, PartialEq, Eq, Hash, Debug`; render's mirror the records'.
`Collector` derives `Debug` alone (`Map` derives only `Debug`, and its `Hash`
is by hand); `Order` derives `PartialOrd, Ord` too, its first field
`unplaced: bool`, so that placed errors sort first.

```rust
// crates/hearth-domain-render/src/errors.rs (22.1; Collision 22.3, 22.4)
/// 7.1's order; private fields, in this order: unplaced, path, position, target, side (45.1), arrival.
pub struct Order { /* private */ }
/// The errors of one call: the first `Limits::errors` in order, and a count of the rest.
pub struct Collector { /* private: Map<Order, Error>, left_out, arrival */ }
impl Collector {
    #[must_use] pub fn new(limits: &Limits) -> Collector;
    /// Keeps `error` if it is among the first `errors` in order, else counts it.
    pub fn push(&mut self, error: Error);
    /// The kept errors in order, and how many were left out.
    #[must_use] pub fn finish(self) -> (Box<[Error]>, u32);
    /// `Map::worst_case` counts nodes only: this adds the kept errors' own bytes.
    #[must_use] pub fn worst_case(limits: &Limits) -> Option<u64>;
}
/// `limits.rs`: `Limits` gains `errors: u32`, `SHIPPED` 256 (render.md 2.3).
/// render.md 7.2's `collision`; `Key` is 16.2's.
pub enum Collision {
    Key { key: Box<[u8]> },
    /// A name, path or id two tasks give: `first` and `second` are tasks.
    Given { subject: Subject, name: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    /// One key, rows that differ: tasks.
    Differs { subject: Subject, name: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    /// `path` under the data path `data`, which a mount would hide: tasks.
    Hidden { path: Box<[u8]>, data: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    /// One id two accounts have: the accounts.
    Shared { id: Id, value: u32, first: Box<[u8]>, second: Box<[u8]> },
    /// Two paths or ranges that touch: paths, or accounts for a range.
    Overlap { subject: Subject, first: Box<[u8]>, second: Box<[u8]> },
}
pub enum Id { Uid, Gid }
pub enum Subject { Tree, DataName, DataPath, Secret, Identity, Download, User, Group, Preset, Tmpfiles, Preseed, Parameter, Range }

// crates/hearth-domain-render/src/vocabulary.rs: `Answer::Rendered` gains `left_out: u32`: `{ /* … */ left_out: u32 }`
// crates/hearth-domain-builder/src/vocabulary.rs: `Request::Reply` gains it: `{ /* … */ left_out: u32 }`

// crates/hearth-domain-render/src/additions.rs (22.2)
/// subuid(5) lines for every user with a range, by name; `None` when no user has one.
#[must_use] pub fn subordinate(accounts: &Accounts) -> Option<Box<[u8]>>;
/// A `d` line for each data entry, at its path with its mode, owner and group,
/// with the entry's task and place (18's `data::Entry` carries them), which 22.4 names.
pub struct DataLine { pub task: Box<[u8]>, pub at: Position, pub tmpfile: Tmpfile }
#[must_use] pub fn data_lines(entries: &[data::Entry]) -> Box<[DataLine]>;
/// `tiny-initramfs.conf` (machine.md, 7.1).
#[must_use] pub fn initramfs(initramfs: &Initramfs) -> Box<[u8]>;
// parts/packages.rs, render's and the records': `Packages` gains `initramfs_file: Option<Digest>`, outside `items`

// crates/hearth-domain-render/src/collisions.rs (22.3: paths; 22.4: the rest)
/// Everything a target's tasks declared, as the kinds built it.
pub struct Declared<'a> { pub target: &'a [u8], pub expansion: &'a Expansion, pub plans: &'a [files::Plan], pub accounts: &'a Accounts }
/// Tree, data, secret, identity and download claims.
pub fn paths(declared: &Declared<'_>, limits: &Limits, errors: &mut Collector);
/// Rows that must agree, by key; the survivors, one row a key.
pub fn declarations(declared: &Declared<'_>, limits: &Limits, errors: &mut Collector) -> Survivors;
/// Uids, gids and subordinate ranges.
pub fn ids(accounts: &Accounts, target: &[u8], limits: &Limits, errors: &mut Collector);
/// What agreed: users, groups, presets, tmpfiles, preseeds and parameters, first of each key.
pub struct Survivors { pub accounts: Accounts, pub presets: Box<[Preset]>, pub tmpfiles: Box<[Tmpfile]>, pub preseeds: Box<[Preseed]>, pub parameters: Box<[Parameter]> }
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-22\.[1-4]'` prints 4.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run -p hearth-domain-render -p hearth-protocol-builder -p hearth-client-protocol-records -p hearth-render-world -p hearth-builder-world` passes.
3. `git -C ~/src/rust/hearth ls-tree --name-only origin/main crates/hearth-domain-render/src/ | grep -c -x -E 'crates/hearth-domain-render/src/(collisions|additions)\.rs'` prints 2.
4. `git -C ~/src/rust/hearth grep -n -E 'left_out: 0' origin/main -- crates/hearth-domain-render/src crates/hearth-protocol-builder/src ':!*/tests/*' ':!*/tests.rs'` prints nothing.
5. `git -C ~/src/rust/hearth grep -n 'transitional: hearth 22' origin/main -- crates` prints nothing.
6. `git -C ~/src/rust/hearth grep -n -E '\bVec\b|String::|[^:]: *String\b|-> *String\b|<String\b|&String\b|^\s*_ =>' origin/main -- crates/hearth-domain-render/src crates/hearth-protocol-builder/src/translate ':!*/tests/*' ':!*/tests.rs'` prints nothing (code-organization.md, 6).
