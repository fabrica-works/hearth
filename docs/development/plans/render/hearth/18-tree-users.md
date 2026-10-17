# Session 18: files, users, groups, presets, tmpfiles, data, scripts

hearth 16's concrete entries become the rest of the `config` part and the
`data` part (format.md, 5.5 to 5.8; render.md, 4.2, 4.3). 18.1: links,
directories and `files` entries over `tree/`; 18.2: `template` and `copy`;
18.3: groups, users and members; 18.4: presets, tmpfiles and `customize.sh`;
18.5: the `data` part. Each adds its records (text, JSON), translation and
typed change (hearth 25.1's pattern), and proves itself through `hearth diff`.

## Read first

- hearth's `format.md`: 2.1, 2.3, 2.4, 4.3, 5.2, 5.5 to 5.8, 6.2 (`file`);
  `render.md`: 3.1, 3.3, 3.5 to 3.7, 4.2, 4.3, 6.1, 7.2, 8.2; `store.md`: 6.5.
- This plan's drafts, whose names this file takes as written: `hearth/
  16-entries.md` (Choices, 16.2, 16.3, sketches), `15-files.md`, `25-diff.md`
  (25.1), `10-client.md` (Choices, "Who adds what"), `11-thin-story.md`
  ("Where later sessions add"), `42-listing-members-rules.md` (42.1), `13-schema.md` (13.5).
- skein's `programming-model.md`: 3, 6.3, 8, 10.3; `testing-strategy.md`: 2.1.

## What the design asks

- **The tree** (format.md, 5.5): "A directory's entry governs its subtree: its
  `if` covers every file beneath it, its `owner` and `group` apply to entries
  beneath that have none of their own, and its `mode` is the directory's"; a
  path `tree/` lacks, or that `tree/` and a `template`, `copy` or `link` entry
  both give, is an error. 2.1: every file under `templates/` must be used.
- **Accounts** (5.6): the users table's defaults; a package user has "no other
  field but `uid`, `gid`, `if` and `for_each`". **Scripts** (5.8; render.md,
  3.3): a declared script must exist and a script file must be declared; 4.3
  sorts "by order, then key, then task name". **Data** (5.7; render.md, 4.3):
  "a name, a class and a path each"; mode and owners go to "the config part's
  tmpfiles lines". **The diff** (6.1): "added, removed, changed, by key".

## Choices this plan makes

1. **A kind is `Located` rows in, typed rows out.** `kinds/<map>.rs` reads
   `Expansion::of(map)` (16.3) and appends each entry's first fault to a
   `List`; a target with a fault has no spec (render.md 7.1; hearth 16.1).
   Forms reuse 13's
   `Schema::{Enum, Malformed, MissingField, UndeclaredField, Above, Below}`,
   `Format::{Path, Mode}`; faults against the repository are `repository`.
2. **Reads grow.** The second `Read` of a present task (32.1, 15.1) also asks
   its tree links (`As::Link`; render.md 8.2's `as:`), `templates/*.j2` and `customize.sh(.j2)`;
   a third, after expansion, asks each `copy` source once (`As::Cached`,
   origin `Blob`). A static file an `if` leaves out is cached all the same.
3. **A task installs by a `files::Plan`** of the tree and its `files` entries,
   one `Install` a path; the call runs its templates in path order (replacing
   15.2's list) and builds `Config::tree`. A path two sources give is 42.1's
   `Repository::Destination`, the first its `also`.
4. **Governing** (format.md 5.5): a Plain-keyed entry with no `for_each` that its `if`
   dropped leaves its tree path (or, ending `/`, its subtree and every entry
   beneath, kept or not) out; a directory entry makes a `Directory` node and
   gives each file beneath the owner and group it lacks, field by field, the
   nearest such entry first.
5. **Modes:** three or four octal digits; a tree file defaults to git's (0755
   executable, else 0644), `template` and `copy` 0644 unless given (5.5), directories 0755;
   owner and group `root`, never checked (build resolves them).
6. **Empty is present** (render.md 4.1; client/README.md 3.2): a config list
   with no row is a field of the record, empty, and prints no row; the `data`
   part is in every spec, its `entries` empty without an entry. THIN's config
   part is then `369415fd653c` (32.1's `e2475ba7…` without the empty fields), its
   data part `2d4765200bab`, its spec `af576004e1f8` (findings/digests.md).
   JSON: a node or a variant is an object with its `kind`, as `file`.
7. **Left to others:** hearth 19 reads `Accounts` (seeds, fixed ids) and
   `password`; `subordinate` waits for 22's `/etc/subuid`, data's `mode` and
   owners for 22's `d` tmpfiles line (render.md, 3.6); `software.sh` is
   checked here, digested by 19; `bless` is the M3 checks part's.
8. **Transitional, ended by hearth 22:** one path, user, group, preset or
   tmpfiles line from two tasks: equal rows collapse, a differing one is
   dropped (the first task wins), each drop marked `// transitional: hearth
   22`; 22's `collision` replaces it.
9. **Text** follows 9.2's `tmpfiles   /var/log/journal d 2755 root
   systemd-journal`: a label cell of 11, the list's name; users `<name> <uid|-> <group> <home> <shell> [gecos]`,
   groups `<name> <gid|package>`, members `<user> <group>`, presets `<unit>
   <action>`, scripts `<task> <order> <digest12>`, tmpfiles `<path> <type>
   <mode|-> <owner|-> <group|-> [<age> [<argument>]]`. A diff row is
   `<mark> <list> <key>` padded to 52 characters, then the entry's row less its key.
   Text no example shows (link, directory, lists, class, diff rows) is this
   Choice's (the plan decides).
10. **Package accounts:** `users` and `groups` both have a `package` variant
    (render.md 4.2); build writes no sysusers line for one (format.md 5.6).

## Rules that bind it

- **Every increment's Files also hold** protocol `src/translate/{parts/config,
  changes, errors}.rs`, `src/tests/translate.rs`, records `src/{errors,
  diff}.rs`, `src/parts/`, `src/form/{text, json}.rs` and tests, and
  `tests/builder/{src/fixtures.rs, tests/real.rs}` (shared). A variant added to
  render's `errors`, `Decoded`, `As`, `Node` or `PartChange` extends
  `translate/` both ways the same increment, no `_` arm.
- **No recursion, no `Vec`, `String` or `_` arm** (render.md, 8.3); rows in
  `skein_lib::List`, bounded by 2.3's listing and `expansion`. Digests are
  `sha256sum`'s literals; real tests use 12.1's `lines_starting`.

## Increments

### 18.1 Tree links, directories, modes and owners
Scope: format.md 5.5 beyond static files: git links in `tree/`; `files`
entries with no source (the tree's file, link, template or directory, with
`mode`, `owner`, `group`, `if`), `directory: true`, `link`; `Node::{Link,
Directory}`.
Needs: hearth 16, hearth 25.1, hearth 42.1.
Files: render new `src/kinds/files.rs`, `src/kinds.rs` (`mod` line),
`src/{call, revision, vocabulary, errors}.rs`, `src/parts/config.rs`,
`src/tests/fixtures.rs`, new `src/tests/kinds.rs`; protocol `src/read.rs`.
Shape: the sketches. `As::Link` blobs are gathered whole (at most `Limit::
Path`) and answered `Decoded::Link`. `files::plan` takes one source among
`template`, `copy`, `link`, `directory` (`Schema::Source` at the second
field); a key ending `/` takes no source field, `directory` included
(`UndeclaredField`; 5.5); `directory` is `true` or absent (`false` is `Wrong`);
a field the source lacks (`mode` on a link) is `UndeclaredField`; the key is
`Format::Path`. A link target (tree or `link`) is 1 to 4,095 bytes with no NUL
(`Format::Link`; a tree link's, `Repository::LinkTarget`). No source: the
tree path must exist, a directory exactly when the key ends in `/`, else
`Repository::NoTree` at the entry; `link` over a tree path is `Destination`. A tree template `NAME.j2.j2`
is stripped of one `.j2` (2.3; 15.2 owns the rule).
Test shape: `src/tests/kinds.rs` (new), `limits::SHIPPED`, each under 20 ms;
fixtures gain `tree`, `files`, `plan`, `owners`.
`a_directory_entry_gives_its_files_the_owner_and_group_they_lack`
(`{"/etc/app/": {"owner": "app", "group": "app", "mode": "0750"},
"/etc/app/secret": {"owner": "root"}}` over `tree/etc/app/{conf,secret}`: the
directory and `conf` are `app:app`, `secret` `root:app`; 5.5).
Then `a_directory_entrys_mode_is_the_directorys_only`,
`an_entry_for_a_path_the_tree_lacks_is_a_repository_error`,
`a_link_entry_over_a_tree_path_is_twice`,
`two_sources_on_one_entry_err_at_the_second`,
`an_entry_its_if_dropped_leaves_its_tree_path_out`,
`the_nearest_directory_entry_governs_owner_and_group`,
`a_dropped_directory_entry_drops_a_kept_entry_beneath_it`,
`a_directory_key_with_a_source_field_errs_undeclared`,
`a_key_ends_in_a_slash_exactly_when_it_names_a_tree_directory`,
`directory_false_is_wrong`,
`a_link_target_of_no_bytes_a_nul_or_4096_bytes_is_refused`,
`tree_etc_x_j2_j2_renders_to_etc_x_j2`; protocol
`a_tree_link_reaches_render_as_its_target`; translate
`each_node_and_fault_translates_to_the_records_and_back`; records
`a_link_and_a_directory_print_after_the_files_columns`. Real, each under 0.5 s,
on `LINKS` (`tasks/motd/task.json` `{"files": {"/etc/motd": {"mode": "0600",
"group": "adm"}, "/var/lib/motd": {"directory": true, "mode": "0750"}}}`, a git
link `tasks/motd/tree/etc/mtab` to `/proc/self/mounts` by new `File::link`):
`hearth_render_lists_a_link_a_directory_and_a_files_new_mode` expects
```text
  /etc/motd                file 0600 root:adm 775324fcea7f
  /etc/mtab                link /proc/self/mounts root:root
  /usr/local/bin/motd      file 0755 root:root f487088ec8b1
  /var/lib/motd            directory 0750 root:root
```
Proof: process, `hearth_diff_shows_a_moved_link_and_a_directory_s_new_mode`,
bound 0.5 s: 25.1's `diff` of `LINKS` and `LINKS_MOVED` (link to `/proc/mounts`,
mode `0755`) through real git; `lines_starting(&diffed.stdout, &["  config "])`
is exactly
```text
  config   ~ /etc/mtab                              link /proc/mounts root:root
  config   ~ /var/lib/motd                          directory 0755 root:root
```
and the exit is `Exit::Code(1)`: 25.1's change and 40.1's printer, for links.
Don't: ask hunks of a link (25.2); check reserved paths (21) or other
tasks' paths (22); read a link through a path.
Size: at most 300 lines.
Removes: 32.1's "links left out" (15.2 left links to 18).

### 18.2 Template entries and copies
Scope: `template` entries (format.md 5.5, 7.7's `key`, `value`), `templates/`
all used (2.1; render.md 3.3), `copy` entries (6.2 `file`; render.md 3.1).
Needs: 18.1's `files::plan`.
Files: render `src/kinds/files.rs`, `src/{call, revision, vocabulary,
errors}.rs`, `src/evaluate/template.rs`, `src/tests/{kinds, templates,
fixtures}.rs`; protocol `src/read.rs`.
Shape: the sketches. Every `templates/*.j2` of a present task is read
`As::Template` (15.1's compile and refusal at its line); `Make::Rendered`
names the template (`Repository::NoTemplate` if the task lacks it) and the
element: 15.2's `run` gains `element`, binding `key`, `value` from
`entries::element` (16.2). A `template` field is read as written, never
interpolated (format.md 5.5; a text with a `{{` names no template, `NoTemplate`),
so `Unused` is exact: each template no `template` field names. `copy` is checked as 13.5 checks a `file` setting (`Schema::File`,
`Large`), streamed once per path (`Make::Copied`); `Keep`'s `path` is the
template's. Rendered or copied over a tree path is `Destination`.
Test shape: `src/tests/kinds.rs` and `templates.rs`, under 20 ms:
`a_template_entry_runs_once_per_element_with_its_key_and_value`,
`a_template_no_entry_names_is_unused`,
`a_template_field_with_a_tag_is_not_interpolated_and_names_no_template`,
`a_template_entry_naming_no_template_is_a_repository_error`,
`a_copy_of_an_unlisted_file_is_a_missing_file`,
`a_copy_over_a_tree_path_is_twice`,
`a_template_entry_over_a_tree_path_is_twice`,
`a_template_only_a_dropped_entry_names_is_still_used`,
`a_copy_of_an_executable_file_is_0644_unless_given`; protocol
`a_templates_blob_reaches_render_compiled`. Real, on `FILES` (`task.json`
`{"files": {"/etc/motd.d/{{ value }}": {"for_each": "['dev', 'ops']",
"template": "templates/greeting.j2", "mode": "0640"}, "/etc/ssl/lab.crt":
{"copy": "lab-ca/lab.crt"}}}`; `templates/greeting.j2` `Hello {{ value }} on
{{ target.name }}.\n`; `lab-ca/lab.crt` `lab certificate\n`),
`hearth_render_lists_rendered_and_copied_files_by_their_sha256`:
```text
  /etc/motd                file 0644 root:root 775324fcea7f
  /etc/motd.d/dev          file 0640 root:root b0b0dd8b10a8
  /etc/motd.d/ops          file 0640 root:root e55838f39d99
  /etc/ssl/lab.crt         file 0644 root:root 4cb3bd448645
  /usr/local/bin/motd      file 0755 root:root f487088ec8b1
```
Proof: process, `hearth_diff_shows_each_element_s_changed_output`, bound 0.5 s:
`FILES_HI` (`Hi` for `Hello`), exit 1, prints exactly (`lines_starting`,
`  config `), rendered files crossing the cache and the diff:
```text
  config   ~ /etc/motd.d/dev                        0640 root:root
  config   ~ /etc/motd.d/ops                        0640 root:root
```
Don't: run a template for an entry its `if` dropped.
Size: at most 300 lines.
Removes: 15.2's run list of tree templates alone.

### 18.3 Groups, users and members
Scope: format.md 5.6 (`users`, `groups`, `from`, `groups` as members), 5.2's
example; render.md 4.2's `groups`, `users`, `members`; `Accounts`.
Needs: hearth 16, hearth 25.1, hearth 42.1.
Files: render new `src/kinds/{users, groups}.rs`, `src/kinds.rs`,
`src/parts/config.rs`, `src/{call, errors}.rs`, `src/tests/{kinds,
fixtures}.rs`.
Shape: the sketches. `from` is `sysusers` (default) or `package` (`Schema::
Enum`); a package account takes `uid`, `gid` alone (`UndeclaredField`). A
sysusers user: `group` its own name, `gecos` empty, `home` `/`, `shell`
`/usr/sbin/nologin` (`Format::Path`), none of the three with a `:`, `"`, `\` or
control byte (`Malformed`; 5.6), `uid` and `gid` within 1 to 65533 (`Below`,
`Above`), `subordinate` `{base, count}` carried (base from 65,536, count from
1, base plus count at most 4,294,967,295: `Below`, `Above` at the field),
`password` left to 19; each `groups` element a `Member`, sorted and once. The
part sorts by name (Choices 8).
Test shape: under 20 ms: `a_user_without_fields_gets_5_6s_defaults`,
`a_package_user_takes_only_its_ids`, `groups_become_sorted_members_once`,
`an_unknown_from_lists_the_two_values`,
`a_uid_or_gid_outside_1_to_65533_errs_below_or_above` (0, 65534, 65535),
`a_subordinate_range_out_of_bounds_errs_below_or_above` (base 65535, count 0,
base plus count 2^32),
`a_gecos_home_or_shell_with_a_colon_a_quote_a_backslash_or_a_control_byte_is_malformed`,
`equal_users_of_two_tasks_collapse`. Real, on `ACCOUNTS` (`task.json`
`{"groups": {"ops": {"gid": 1500}, "mail": {"from": "package", "gid": 8}},
"users": {"ci": {}, "free": {"uid": 1000, "home": "/home/free", "shell":
"/bin/bash", "groups": ["sudo", "ops"]}, "postfix": {"from": "package", "uid":
107, "gid": 113}}}`), `hearth_render_lists_groups_users_and_members`:
```text
  groups     mail package
  groups     ops 1500
  users      ci - ci / /usr/sbin/nologin
  users      free 1000 free /home/free /bin/bash
  users      postfix package
  members    free ops
  members    free sudo
```
Proof: process, `hearth_diff_names_each_account_group_and_member_moved`, bound
0.5 s: `ACCOUNTS_MOVED` (`ops` gone; `free` uid 1001, groups `sudo`,
`video`) prints exactly (`lines_starting`, `  config `), exit 1
```text
  config   - groups ops                             1500
  config   ~ users free                             1001 free /home/free /bin/bash
  config   - members free ops
  config   + members free video
```
Don't: write sysusers lines or `/etc/subuid` (22, build); read `password`;
check fixed ids (19) or two users of one id (22).
Size: at most 300 lines.
Removes: nothing.

### 18.4 Presets, tmpfiles and `customize.sh`
Scope: format.md 5.6's `presets` and `tmpfiles`; 5.8's `scripts` (render.md 3.3:
declared scripts exist, script files declared); `Config::{presets, tmpfiles,
scripts}`.
Needs: hearth 16, hearth 25.1, hearth 42.1.
Files: render new `src/kinds/{presets, tmpfiles, scripts}.rs`, `src/kinds.rs`,
`src/parts/config.rs`, `src/{call, revision, errors}.rs`, `src/tests/kinds.rs`.
Shape: `action` is `enable` or `disable`; `type` one of 14 letters (`Schema::
Enum`) and required (`MissingField`); `mode` `Format::Mode`; the rest as
written. `scripts`: each declared name must exist as `NAME` or `NAME.j2`
(`Repository::NoScript` at its key), each script file declared (42's
`Repository::Stray`, at its path), for `customize.sh` and `software.sh`;
`software.sh` is checked and its row dropped (19.2 digests it); a third name is `Schema::Enum` (5.8); `order`
default 500, 0 to 999 (`Below`, `Above`); `customize.sh` is read (`As::Cached`,
or `As::Template` then `Keep`, origin `Rendered`) and its row `{task, order,
content_digest}` sorts by order, then task (format.md 4.3). A template script
(`customize.sh.j2`, `software.sh.j2`) runs 16.4's `references::template`.
Test shape: under 20 ms: `a_preset_takes_enable_or_disable`,
`a_tmpfiles_line_needs_its_type_and_checks_its_mode`,
`a_declared_script_with_no_file_is_a_repository_error`,
`a_third_script_name_is_an_enum_fault`, `a_preset_without_action_is_missing`,
`script_order_defaults_to_500_and_ranges_over_0_to_999`,
`customize_sh_j2_is_digested_as_rendered`,
`a_script_file_nothing_declares_is_stray`, `scripts_sort_by_order_then_task`,
`a_software_script_is_checked_not_digested`,
`a_typo_in_a_script_template_is_found_at_its_line` (`customize.sh.j2`).
Real, on `PRESETS` (`task.json` `{"presets": {"ssh.service": {"action":
"enable"}, "getty@tty1.service": {"action": "disable"}}, "tmpfiles":
{"/var/log/journal": {"type": "d", "mode": "2755", "owner": "root", "group":
"systemd-journal"}}, "scripts": {"customize.sh": {"order": 300}}}`;
`tasks/motd/customize.sh` `update-ca-certificates\n`),
`hearth_render_lists_presets_tmpfiles_and_scripts`:
```text
  presets    getty@tty1.service disable
  presets    ssh.service enable
  tmpfiles   /var/log/journal d 2755 root systemd-journal
  scripts    motd 300 b40d08aa8c5c
```
Proof: process, `hearth_diff_names_a_preset_a_tmpfiles_line_and_a_script`, bound
0.5 s: `PRESETS_MOVED` (`ssh.service` disabled, mode `0755`, `customize.sh` now
`update-ca-certificates\nsystemctl daemon-reload\n`) prints exactly (`lines_starting`, `  config `), exit 1
```text
  config   ~ presets ssh.service                    disable
  config   ~ tmpfiles /var/log/journal              d 0755 root systemd-journal
  config   ~ scripts                                motd 300 4b1f03482366
```
Don't: refuse a reserved line (21) or digest `software.sh` (19).
Size: at most 250 lines.
Removes: nothing.

### 18.5 The `data` part
Scope: format.md 5.7, 2.4's data names; render.md 4.3's `entries`;
`PartRecord::Data` and its change (`Layer::Slot`, 6.2's `slot only`).
Needs: hearth 16, hearth 25.1, hearth 42.1.
Files: render new `src/kinds/data.rs`, `src/parts/data.rs`, `src/parts.rs`,
`src/diff.rs`, `src/{call, errors}.rs`, `src/tests/kinds.rs`; protocol new
`src/translate/parts/data.rs`, `src/translate/parts.rs`; records
`src/records.rs`.
Shape: the sketches. `path` required (`MissingField`, `Format::Path`), `class`
`core`, `data` or `cache` (default `data`), `mode` default `0755`, owner and
group `root`; the name `Format::DataName`. The record keeps `{class, path}`;
`data::Entry` carries `mode` and owners for 22's tmpfiles line. No entry, an
empty part (Choices 6; THIN_TEXT gains `data         part <d>`, the later
pass's); the 65th entry in name order is `Above { max: 64 }` at its key (5.7).
Test shape: under 20 ms: `a_data_entry_defaults_to_the_data_class_and_0755`,
`a_data_name_follows_2_4s_form`, `a_path_is_required`,
`no_entry_makes_an_empty_data_part`,
`a_65th_data_entry_errs_above_at_its_name`;
records `data_prints_its_names_and_paths_as_9_2_shows`. Real, on `DATA`
(`task.json` `{"data": {"journal": {"class": "core", "path":
"/var/log/journal", "mode": "2755", "group": "systemd-journal"}, "iwd":
{"class": "core", "path": "/var/lib/iwd", "mode": "0700"}}, "groups":
{"systemd-journal": {"from": "package", "gid": 999}}}`),
`hearth_render_lists_the_data_entries` (`lines_starting`, `  iwd `,
`  journal `):
```text
  iwd        /var/lib/iwd
  journal    /var/log/journal
```
Proof: process, `hearth_diff_names_a_data_entry_moved_to_another_class`, bound
0.5 s: `DATA_MOVED` (`iwd` in class `data`) prints (`lines_starting`, `  data `)
```text
  data     ~ iwd                                    data /var/lib/iwd
```
and exits 1. Unit `a_data_change_alone_rebuilds_slot_only` (6.2: no layer).
Don't: write the `d` tmpfiles line or policies (22, 21): until 21's [M5]/[M6]
the part has no policies until then though render.md 4.3 gives them; check
paths (21, 22).
Size: at most 150 lines.
Removes: nothing.

## Interface sketches

Types derive `Clone, PartialEq, Eq, Hash, Debug`. render's `parts/{config,
data}.rs` mirror the records' types; translation: a function a type, both ways.

```rust
// render src/kinds/files.rs (18.1; Rendered, Copied 18.2)
pub struct Plan { pub task: Box<[u8]>, pub installs: Box<[Install]> } // by path
pub struct Install { pub path: Box<[u8]>, pub at: Option<Position>, /* None: made by the tree */ pub make: Make, pub owner: Box<[u8]>, pub group: Box<[u8]> }
pub enum Make { Static { sha256: Digest, mode: u16 }, Link { target: Box<[u8]> }, Directory { mode: u16 }, Rendered { template: Box<[u8]>, via: Option<Via>, mode: u16 }, Copied { source: Box<[u8]>, mode: u16 } }
/// The `files` entry (its place in `Expansion::of(Map::Files)`) and `for_each` element a template runs for.
pub struct Via { pub entry: u32, pub element: Option<u32> }
pub struct TreeFile { pub path: Box<[u8]>, pub source: Source, pub mode: u16 }
pub enum Source { Static { sha256: Digest }, Template { oid: Oid, path: Box<[u8]> }, Link { target: Box<[u8]> } }
pub fn plan(task: &[u8], tree: &[TreeFile], files: &[Located<'_>], dropped: &[Box<[u8]>], faults: &mut List<Fault>) -> Plan;
#[must_use] pub fn dropped(declaration: &Declaration, kept: &[Located<'_>]) -> Box<[Box<[u8]>]>; // Plain keys their `if` removed
// render src/kinds/{users, groups, presets, tmpfiles, scripts, data}.rs: rows in, rows out, faults appended
pub fn accounts(users: &[Located<'_>], groups: &[Located<'_>], faults: &mut List<Fault>) -> Accounts;
pub struct Accounts { pub users: Box<[User]>, pub groups: Box<[Group]> }
pub struct User { pub name: Box<[u8]>, pub task: Box<[u8]>, pub at: Position, pub kind: UserKind }
pub enum UserKind { Sysusers { uid: Option<u32>, group: Box<[u8]>, gecos: Box<[u8]>, home: Box<[u8]>, shell: Box<[u8]>, subordinate: Option<(u32, u32)>, member_of: Box<[Box<[u8]>]> }, Package { uid: Option<u32>, gid: Option<u32> } }
pub struct Group { pub name: Box<[u8]>, pub task: Box<[u8]>, pub at: Position, pub gid: Option<u32>, pub package: bool }
pub struct Entry { pub name: Box<[u8]>, pub task: Box<[u8]>, pub at: Position, /* its key in the task.json, as User's: hearth 22.4 names it */ pub class: DataClass, pub path: Box<[u8]>, pub mode: u16, pub owner: Box<[u8]>, pub group: Box<[u8]> } // data.rs: the part keeps class, path
// render src/evaluate/template.rs: 15.2's `run` gains `element` (None: a tree template)
pub fn run<'c>(template: &'c Template, context: &Context<'c>, element: Option<(Operand<'c>, Operand<'c>)>, limits: &Limits, room: u64) -> Result<Box<[u8]>, Fault>;
// render errors.rs and the records': Repository gains NoTree, NoTemplate, Unused, NoScript, LinkTarget (`Stray` is
// 42's); Schema gains Source { first: Name, second: Name }; Format DataName, Link; vocabulary: As::Link, Decoded::Link(Box<[u8]>)

// records src/parts/config.rs: an empty list is an empty field of the record, with no line (Choices 6)
pub struct Config { pub tree: Box<[TreeEntry]>, pub groups: Box<[Group]>, pub users: Box<[User]>, pub members: Box<[Member]>, pub presets: Box<[Preset]>, pub tmpfiles: Box<[Tmpfile]>, pub scripts: Box<[Script]> }
pub enum Node { File { /* 10.1's */ }, Link { target: Box<[u8]>, owner: Box<[u8]>, group: Box<[u8]> }, Directory { mode: u16, owner: Box<[u8]>, group: Box<[u8]> } }
pub struct Group { pub name: Box<[u8]>, pub kind: GroupKind }  pub enum GroupKind { Sysusers { gid: Option<u32> }, Package }
pub struct User { pub name: Box<[u8]>, pub kind: UserKind }
pub enum UserKind { Sysusers { uid: Option<u32>, group: Box<[u8]>, gecos: Box<[u8]>, home: Box<[u8]>, shell: Box<[u8]> }, Package }
pub struct Member { pub user: Box<[u8]>, pub group: Box<[u8]> }
pub struct Preset { pub unit: Box<[u8]>, pub action: Action }  pub enum Action { Enable, Disable }
pub struct Tmpfile { pub path: Box<[u8]>, pub line: Line }
pub struct Line { pub kind: LineKind, pub mode: Option<u16>, pub owner: Option<Box<[u8]>>, pub group: Option<Box<[u8]>>, pub age: Option<Box<[u8]>>, pub argument: Option<Box<[u8]>> }
/// tmpfiles.d(5)'s letters: `Plus` a `+`, `Capital` an upper-case letter.
pub enum LineKind { D, CapitalD, E, F, FPlus, W, WPlus, CapitalL, CapitalLPlus, CapitalC, Z, CapitalZ, R, CapitalR }
pub struct Script { pub task: Box<[u8]>, pub order: u16, pub content_digest: [u8; 32] }
// src/parts/data.rs (18.5); src/diff.rs: ConfigChange gains the lists, PartChange Data(DataChange)
pub struct Data { pub entries: Box<[DataEntry]> }  pub struct DataEntry { pub name: Box<[u8]>, pub value: DataValue }
pub struct DataValue { pub class: DataClass, pub path: Box<[u8]> }  pub enum DataClass { Core, Data, Cache }
pub enum Keyed<T> { Added { key: Box<[u8]>, value: T }, Removed { key: Box<[u8]>, value: T }, Changed { key: Box<[u8]>, before: T, after: T } }
pub struct ConfigChange { pub tree: Box<[EntryChange]>, pub groups: Box<[Keyed<GroupKind>]>, pub users: Box<[Keyed<UserKind>]>, pub members: Box<[MemberChange]>, pub presets: Box<[Keyed<Action>]>, pub tmpfiles: Box<[Keyed<Line>]>, pub scripts: Option<ScriptsChange> }
pub enum MemberChange { Added(Member), Removed(Member) }  pub struct DataChange { pub entries: Box<[Keyed<DataValue>]> }
pub struct ScriptsChange { pub before: Box<[Script]>, pub after: Box<[Script]> }
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-18\.[1-5]'` prints 5.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run -p hearth-domain-render -p hearth-protocol-builder -p hearth-client-protocol-records -p hearth-builder-world` passes.
3. `git -C ~/src/rust/hearth ls-tree -r --name-only origin/main crates | grep -c -x -E '.*/(kinds/(files|users|groups|presets|tmpfiles|scripts|data)|translate/parts/data)\.rs'` prints 8.
4. `git -C ~/src/rust/hearth grep -n -i -E 'minijinja|legacy|workaround' origin/main -- crates/hearth-domain-render/src/kinds crates/hearth-domain-render/src/parts` prints nothing.
5. `git -C ~/src/rust/hearth grep -n -E '\bVec\b|String::|[^:]: *String\b|-> *String\b|<String\b|&String\b|^\s*_ =>' origin/main -- crates/hearth-domain-render/src crates/hearth-protocol-builder/src/translate ':!*/tests/*' ':!*/tests.rs'` prints nothing (code-organization.md, 6).
