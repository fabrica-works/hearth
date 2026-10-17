# Session 21: what hearth reserves; the `hearth` task

Render refuses at its line what a later step would refuse (render.md, 7.2,
7.3; format.md, 5.11), and gives every target the `hearth` task (5.12).
21.1 refuses reserved words: command-line keys and length, a `password`
preseed, trial and overlay units, tmpfiles types; 21.2 reserved paths, in
entries and in the file tree; 21.3 secrets hearth could read and `live`
entries on never-live units and paths; 21.4 the `hearth` task: present in
every target but a container, declared by the repository, and passing all of
it. Each proves on the real loop's `hearth render`, by the error it prints.

## Read first

- hearth's `format.md`: 5.2, 5.4 to 5.7, 5.11 (whole), 5.12, 5.14, 9.1, 9.3;
  `render.md`: 3.2, 3.3, 7.1 to 7.3; `machine.md`: 6.1, 6.2; `slots.md`: 5.5.
- This plan: `hearth/16-entries.md` (Choices, 16.3); `17-packages.md`
  (17.3, 17.4); `18-tree-users.md` (18.1, 18.3, 18.4, sketches);
  `19-secrets-software.md` (19.1 to 19.4); `12-compose.md` (12.1,
  sketches), `42-listing-members-rules.md` (42.3); `08-mirror.md` (08.2); `40-client-diff-errors.md` (40.2).
- skein at `f981746`: `crates/skein-lib/src/list.rs` (`push`:55).

## What the design asks

- **5.11:** "What a later step would refuse, render refuses first, at the line
  that declares it", in a table of words (command-line keys, `password`
  preseeds, tmpfiles types), paths, units, owners and modes of secrets, and
  `live` entries. render.md 7.2's `reserved`: "a secret or identity file
  `hearth` could read, or a secret readable by others".
- **5.12:** "Every target has the `hearth` task, but a container (below),
  without naming it: render adds it, and `"hearth": null` is an error";
  "naming it there [a container] is an error". Its files "are in the
  repository like any task's, `tasks/hearth/`". render.md 3.2: "the `hearth`
  task is added to every target but a container".
- **9.1:** a secret's mode "gives others nothing (at most `0770`, no setuid,
  setgid or sticky bit)", its owner "is neither user `hearth` nor a group
  `hearth` belongs to, such as `systemd-journal`". **5.14:** a `live` entry
  that names a never-live path or unit is refused (5.11).

## Choices this plan makes

1. **Three passes, one module.** After 16's expansion, `reserved::entries`
   checks every concrete entry's words and paths (21.1, 21.2, 21.3) and
   *refuses* each entry it faults: `Expansion::refuse` hides it from `of`, so
   no kind reports it a second time (a `password` preseed is an `Enum` fault
   to 17.3's `entry`, tmpfiles `v` to 18.4's: unit-level, they stand). After
   the kinds and `files::plan`, `reserved::tree` checks the installed paths
   (an install is not an entry), and `reserved::live` the `live` entries, which
   need the `hearth` task's installs. A target with an error has no spec
   (render.md 7.1; hearth 16.1). An entry's first fault only, in its map's field order.
2. **Closed rules.** `Kind::Reserved(Reserved { rule, what })`: `Rule` is a
   closed enum of 30 variants, `what` the key, word, unit or path at fault; the
   printer (records) owns the words, one phrase a rule, as 7.3 shows two. Each
   increment adds its rules, the records' and the translation's arms.
3. **A place is a position or a tree file.** `Spot::Declared(Position)` is an
   error at `tasks/<task>/task.json`; `Spot::Tree` a file only `tree/` gives,
   at `tasks/<task>/tree<path>`, no position (40.2's "path alone"), as 18.1's
   `Install::at: Option<Position>` says.
4. **Rows, not code** (21.2): the paths of 5.11 are one `const ROWS` of
   (rule, match, except). A match is `Exact`, `Beneath` (the path itself and
   below, at a `/`), or a named shape. Every row holds for every kind placed
   at a path (tree file, `files` entry or link, download, secret or identity
   file, tmpfiles line, data path; format.md 5.11) but `except`: `SecretPath`
   for `Secret` and `Identity`, `Runtime`'s `/run`, `/tmp` and `/var/tmp`
   for `Tmpfiles`. A strict ancestor of a row's path, or the directory its
   pattern matches in (`/etc/ssh`), takes only a directory, under that row: a
   tree directory, a `directory` entry, a tmpfiles `d`, `e` or `z` with no
   `age`. Paths are 13's `Format::Path` (absolute, no `.`, `..`, `//`,
   trailing `/`), so a prefix test is exact.
5. **Hearth's groups are constants.** User `hearth`, its group `hearth` and
   `systemd-journal`, plus the groups of the target's `hearth` user entry
   (`group`, `groups` of `Expansion::of(Map::Users)`): the check does not
   depend on the repository declaring the task well.
6. **The command line is counted as on EFI,** its longest form: `118 + 3 *
   target length` bytes for `initrd=`, `root=UUID=`, `rootflags=`, `ro`,
   `panic=10` and `hearth.trial=` with a one-letter slot, a space each, the NUL
   (machine.md, 6.1), then `1 + length` a parameter in 17.4's order. The
   first `param` that takes the total past 2048 is at fault, on BIOS too, the
   line counted as an EFI entry's, `initrd=` included (machine.md 6.2).
7. **Implicit presence** (21.4): `compose::compose` adds `hearth` to the
   present tasks (`Present::implicit`, new; its `from` the base-env document at
   1:1), so 13 validates its settings and every fleet key for it reaches it.
8. **Left to M5** (`later.md`): a preset or link enabling a container's
   `systemd-nspawn@` unit, `snapshots` keys but `core` and `data`. A
   container is built now (format.md 5.12; render.md 3.2, 4.2): it renders
   with variant `container`, its module, boot parameter or `hearth` refused.
9. **The test repository's `hearth` task** declares nothing (08.2; testing.md
   6.2; render.md 5.2), so `test-min` is the `apt` variant with no package;
   21.4 proves it. Messages are the Shapes' (render.md 7.3's forms).

## Rules that bind it

- **The meeting point** (README, Rules): each increment proves on the real
  loop; a rule adds render's variant, the records' and the translation's arms
  in one increment, exhaustive.
- **No recursion, no `Vec`, `String` or `_` arm** (render.md, 8.3): every walk a
  `for`, rows in `skein_lib::List`, no new limit. Positions in proofs are
  counted by hand (`python3 -I`, once); no expected value is computed.
- **`clippy.toml`'s list binds every crate:** no `mem::take` (`mem::replace`),
  no `sort_by` closure, `is_some_and`, `Iterator::filter` or `Iterator::find`
  (`:114`, `:132`, `:147`, `:165`, `:207`): a `for`, or a `match`.
- **Shared files:** render's `errors.rs`, `entries.rs`, `compose.rs`, `call.rs`;
  the records' `errors.rs`; `translate/errors.rs`; both fixtures modules.

## Increments

### 21.1 Reserved words: command-line keys and length, preseeds, units, tmpfiles
Scope: 5.11's rows for `cmdline`, `preseeds`, `presets`, `tmpfiles`; machine.md
6.2; `Kind::Reserved`; `Expansion::refuse`; `reserved::entries`.
Needs: the session's start (hearth 17, 18, 19).
Files: render new `src/reserved.rs`, `src/reserved/words.rs`, `src/{entries,
call, errors, lib}.rs`, new `src/tests/reserved.rs`, `src/tests/fixtures.rs`;
protocol `src/translate/errors.rs`, new `src/tests/translate/reserved.rs` (a
rules fixture beside `composition_kinds`, `src/tests/fixtures.rs:740`);
records `src/errors.rs` (`Kind::Reserved`, its text and json arms beside
`Composition`'s, `:81` and `:137`, its `mod` and `pub use` beside
`composition`'s, `:657` to `:669`), new `src/errors/reserved.rs` (no_std: a
length prints with `lines.decimal(u64::from(*bytes))`), `src/tests/errors.rs`;
`tests/render/src/declarations.rs` (its exhaustive `Kind` match, `:223` to
`:232`, gains `| Kind::Reserved(_)`); `tests/builder/{src/fixtures.rs,
tests/real.rs}`.
Shape: the sketches. On a container (`entries`' `container`, from its
placement) an entry of `Map::Initramfs` is `ContainerModule` and one of
`Map::Cmdline` `ContainerParam`, at the entry's key (render.md 4.2, 7.2). A
tmpfiles key holding `*`, `?`, `[` or `%` is `TmpfilesPath`, at the key. A
key is the `param` up to its first `=`: `root`,
`rootflags`, `ro`, `rw`, `initrd`, `panic` and every key beginning `hearth.`
are `ActivatorKey`; `rootfstype`, `rootdelay`, `init` `RootKey`; `rootwait`
`RootWait` (7.3's words). `CmdlineLength { bytes }` is Choices 6, at the
`param` of the crossing entry, sorted by order (default 500), key, task as
17.4. A preseed whose `type` is `password`: `PasswordPreseed`, at `type`. A
preset whose unit begins `hearth-trial` or `hearth-check-`: `TrialName`, at
the key; a preset enabling
`systemd-sysext.service` or `systemd-confext.service`: `Overlay` (`disable` is
the hearth task's own). Tmpfiles `type` `v`, `q`, `Q`: `TmpfilesSubvolume`,
`w`, `w+`, `f+`, `L+` at a path not beneath `/var`, `/run`, `/tmp`:
`TmpfilesWrite`; both at `type`.
Test shape: `src/tests/reserved.rs`, under 20 ms, on 17.4's `fixtures::cmdline`
and new `fixtures::{preseeds, presets, tmpfiles, refused, reserved}`
(`reserved(&[Refused]) -> Box<[Reserved]>`, each row's `reserved` in order).
Reference:
```rust
// crates/hearth-domain-render/src/tests/reserved.rs
#[test]
fn a_reserved_key_is_refused_at_its_param_and_leaves_the_kinds_nothing() {
    let mut expansion = fixtures::cmdline([("base", "wait", "rootwait", 500), ("base", "quiet", "quiet", 100)]);
    let refused = reserved::entries(&mut expansion, b"x1-prod", false);
    let want = Reserved { rule: Rule::RootWait, what: Box::from(&b"rootwait"[..]) };
    assert_eq!(*fixtures::reserved(&refused), [want], "format.md, 5.11: rootwait waits for ever");
    assert_eq!(expansion.of(Map::Cmdline).len(), 1, "the refused entry is hidden: no kind reports it twice");
}
```
Then `every_key_of_5_11_and_every_hearth_key_is_refused_and_console_is_not` (a
table), `the_line_is_counted_with_the_activators_tokens_and_the_nul` (target
`x1-prod`: 139 bytes; 2048 passes, 2049 is refused at the crossing entry),
`a_password_preseed_is_reserved_where_the_kind_alone_calls_it_unlisted`,
`a_trial_or_check_name_is_refused_in_a_preset_whatever_its_action`,
`only_an_enabling_preset_of_sysext_or_confext_is_refused`,
`an_entry_with_two_reserved_fields_is_refused_once_at_the_first`,
`tmpfiles_v_q_and_capital_q_and_writes_outside_var_run_tmp_are_refused`
(reserved wins over 18.4's `Enum` fault), `a_tmpfiles_key_with_a_glob_or_a_
specifier_is_refused`, `the_line_length_error_is_at_the_first_param_past_2048`,
`a_module_or_a_boot_parameter_on_a_container_is_refused_and_on_a_vm_is_not`;
translate `each_rule_translates_to_the_records_and_back`; records
`a_reserved_error_prints_the_rules_words`, `json_gives_the_rule_and_what`.
Proof: process, bound 0.5 s, on `WAIT` (THIN's files but `tasks/motd/task.json`
`{"cmdline": {"wait": {"param": "rootwait"}}}`):
```rust
// tests/builder/tests/real.rs
#[test]
fn a_reserved_command_line_key_prints_as_render_md_shows() {
    let repository = Repository::make("wait", &fixtures::thin_with(fixtures::WAIT));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    let expected = "tasks/motd/task.json:1:23: x1-prod: motd: \"rootwait\" is reserved: tiny-initramfs would wait for ever for a missing root (machine.md, 6.2)\n";
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), expected, "render.md, 7.3's example");
    assert_eq!(rendered.exit, Exit::Code(2), "client/README.md, 3.3: a configuration error");
}
```
Beside it `a_password_preseed_prints_reserved_and_not_an_unlisted_type`
(`{"preseeds": {"d/p": {"type": "password", "value": "x"}}}`: begins
`tasks/motd/task.json:1:23: x1-prod: motd: `, holds `password`, once; exit 2)
and `a_line_past_2048_bytes_names_the_parameter_that_crosses_it` (`a` and `b`
of 1,000 bytes each, `fixtures::long_params()`: begins `tasks/motd/task.json:
1:1040: x1-prod: motd: `, holds `2141`; exit 2).
Don't: check a path (21.2); a secret (21.3); change 17.3's or 18.4's `entry`.
Size: at most 250 lines.
Removes: nothing.

### 21.2 Reserved paths: in entries and in the file tree
Scope: 5.11's path rows; 5.5's "Forbidden destinations"; 5.7's data paths;
`reserved::tree`.
Needs: nothing beyond 21.1.
Files: render new `src/reserved/paths.rs`, `src/reserved.rs`,
`src/{call, errors}.rs`, `src/tests/{reserved, fixtures}.rs`; protocol,
records and `tests/builder` as 21.1's.
Shape: `ROWS` (Choices 4), each by its rule. `ActivatorFile`: `/etc/fstab`,
`/.hearth`. `Dropin`: a child of `/etc/systemd/system.conf.d/`
whose name begins `zz-hearth` or sorts bytewise after `zz-hearth-watchdog.conf`.
`TrialName`: a path whose final name (a unit, link, preset or
drop-in directory) begins `hearth-trial` or `hearth-check-`. `Accounts`: `/etc/{passwd, group, shadow, gshadow, subuid, subgid,
machine-id}`, `/var/lib/dbus/machine-id`. `SecretPath` (but `Secret`,
`Identity`): `/etc/ssh/ssh_host_*_key` and `.pub`, `/etc/dropbear/dropbear_*_host_key`,
`/etc/ssl/private`, `/var/lib/systemd/{random-seed, credential.secret}` (7.3's
words). `HearthState`: `/var/lib/hearth`. `PackagesLayer`: `/usr`
but `/usr/local`. `ClassMount`: `/srv/{core, data, cache}`, but a `Data` path
at `/srv/<its class>/<its name>`. `DataRoot`: a data path not strictly beneath
`/srv`, `/var` or `/home` (never one itself; format.md 5.7). `Runtime`: `/dev`,
`/proc`, `/sys`, and but for Tmpfiles `/run`, `/tmp`, `/var/tmp`; `/etc/.git`,
`/etc/.etckeeper`. `LogFile`: a non-empty file beneath `/var/log` (`Shape::File {
empty: false }`; a rendered file's bytes are known at its `Keep`).
`InitramfsConf`: `/etc/tiny-initramfs/tiny-initramfs.conf`. `Overlay`:
the five extension directories, and a link named `systemd-sysext.service`
or `systemd-confext.service` in a `*.wants` or `*.requires` directory.
A non-directory above a row's path is refused under that row (Choices 4).
Entries' paths are checked in `entries`: the tmpfiles key, a data, software,
secret or identity `path` and `public_path`, each at its field; installs in
`tree`, from the plan and the produced files.
Test shape: under 20 ms; new `fixtures::installed(task, path, at, Shape)`.
Tests: `a_secrets_path_in_the_tree_is_refused_naming_the_secrets_map`
(`/etc/ssl/private/mx1.key` at 30:9: `SecretPath`, `Spot::Declared`),
`a_table_of_paths_each_row_refuses_and_its_neighbour_does_not` (`/usr`,
`/usr/local`, `/usr/localx`, `/var/lib/hearth`, `/var/lib/hearthx`, `/srv/core`,
`/srv/www`), `a_download_under_usr_reports_packages_layer_not_beneath` (a
`software` entry at `/usr/bin/x`: `PackagesLayer`, once, no 19.2 `Beneath`),
`a_dropin_after_the_watchdogs_name_is_refused_before_it_is_not`,
`a_hearth_trial_final_name_refuses_a_unit_a_link_and_a_dropin_directory`,
`a_hearth_trial_name_above_the_final_one_is_allowed` (`/opt/hearth-trial/x`),
`a_data_path_may_be_its_classs_directory_and_no_other_beneath_a_mount`,
`an_empty_log_file_is_allowed_and_a_non_empty_one_is_not`,
`a_tree_file_with_no_entry_is_spotted_at_its_tree_path`,
`every_kind_at_etc_shadow_or_fstab_is_refused_but_a_secret_at_a_secret_path`,
`a_data_path_at_a_machine_id_and_a_tmpfiles_f_at_random_seed_are_refused`,
`only_a_directory_may_stand_above_a_reserved_path` (table: a tar download at
`/etc`, a file at `/etc/ssh`, a tmpfiles `D` and a `d` with an `age` at
`/etc/ssl` are refused under their row; a tree directory, a `directory` entry
and a `d` with no `age` pass),
`home_srv_and_var_themselves_are_no_data_root_but_a_path_beneath_each_is`,
`tmpfiles_under_dev_proc_sys_are_refused_and_under_run_tmp_var_tmp_are_not`.
Proof: process, bound 0.5 s each: `hearth_render_refuses_a_secrets_path_in_the_
tree` on `KEY_IN_TREE` (`task.json` `{"files": {"/etc/ssl/private/mx1.key":
{}}}`, `tree/etc/ssl/private/mx1.key` `key` LF) is exactly
`tasks/motd/task.json:1:12: x1-prod: motd: /etc/ssl/private/mx1.key is a
secret's path; declare it under secrets (format.md, 9.1)` and exit 2, as 7.3
shows; `a_file_only_the_tree_
gives_is_refused_at_its_path` on `FSTAB` (`tree/etc/fstab`) begins
`tasks/motd/tree/etc/fstab: x1-prod: motd: `, holds `/etc/fstab`; and
`a_data_path_beside_its_class_mount_is_refused_at_its_path` (`{"data": {"mail":
{"path": "/srv/data/spool"}}}`: begins `tasks/motd/task.json:1:20: x1-prod:
motd: `, holds `/srv/data`).
Don't: compare paths of two tasks, or a path under a data path (22); refuse a
mode or owner (21.3). 19's `Beneath` stays for paths outside its roots.
Size: at most 300 lines.
Removes: nothing.

### 21.3 Secrets hearth could read; `live` on never-live units and paths
Scope: 5.11's rows for secrets and `live`; 9.1; 5.14's "Never live".
Needs: nothing beyond 21.2.
Files: render new `src/reserved/{owners, live}.rs`, `src/reserved.rs`,
`src/{call, errors}.rs`, `src/tests/{reserved, fixtures}.rs`; protocol, records
and `tests/builder` as 21.1's.
Shape: `owners`: a secret's or identity item's `mode` with any bit outside
`0770` set is `SecretMode` at `mode` (so `0604`, `2770`, `4700` and `1770`
too; format.md 9.3); a secret's
or identity's `owner` equal to `hearth`, or its `group` one of Choices 5's, is
`HearthOwner`, at the field, `also` the `hearth` user entry when declared.
`live` runs after `files::plan`: a `unit` with action `none` is `LiveNone` at
`unit` (one missing with another action is 19.3's missing field, format.md
5.14); a unit named in 5.14's
list (`dbus.service`, `dbus.socket`, `systemd-logind`, `systemd-journald`,
`systemd-udevd`, `systemd-networkd`, `NetworkManager`, `getty@*`,
`serial-getty@*`, `console-getty`, `apparmor`, each with or without `.service`,
and any `systemd-*` or `hearth-*` unit) is `NeverLiveUnit` at `unit`; a path
equal to or beneath a never-live path (5.14's list, 5.11's `ROWS`, or a path of
the `hearth` task's installs or downloads), its one trailing `/` set aside, is
`NeverLivePath` at `paths`, a path that covers one accepted (`/etc/systemd/`);
5.14's globs are named shapes of `ROWS`' `Match`: `etc/systemd/
*.conf` a child file, `usr/local/lib/systemd/*.conf.d/` a child directory and
below, and `etc/apparmor.d/` a `Beneath`.
Test shape: under 20 ms; `fixtures::{secret, live}`. Tests: `a_secret_readable_
by_others_or_with_a_special_bit_is_refused_at_its_mode` (`0600`, `0640`,
`0770` pass; `0604`, `0644`, `2770`, `4700`, `1770` are refused, for an
identity item too),
`hearth_the_user_and_its_groups_own_no_secret_nor_identity_file`,
`a_group_the_hearth_user_declares_is_refused_with_the_user_as_also`,
`a_unit_with_none_is_refused_and_none_without_a_unit_is_not`,
`a_never_live_unit_is_refused_with_or_without_its_suffix`,
`a_live_path_is_refused_when_equal_to_or_beneath_a_never_live_path`,
`a_live_path_beneath_a_hearth_task_install_is_refused`,
`a_live_path_matching_5_14s_globs_or_apparmor_dir_is_refused` (`/etc/systemd/
journald.conf` is; `/etc/systemd/journald.conf.d/x`, `/etc/systemd/x/y.conf` are
not), `a_live_path_covering_a_never_live_one_is_accepted_and_with_a_slash_equal_is_not`.
Proof: process, bound 0.5 s each: `a_secret_owned_by_hearth_is_refused_at_its_
owner` on `SECRET_HEARTH` (19.4's `SECRETS_NO_ID` with `"owner": "hearth"`):
begins `tasks/motd/task.json:1:71: x1-prod: motd: `, holds `hearth` and
`mail/tls/mx1.key`, nothing about ids, exit 2; `a_never_live_unit_is_refused_
at_its_unit` on `LIVE_NEVER` (`{"live": {"net": {"unit": "systemd-networkd.
service", "action": "restart", "paths": ["/etc/hostname"]}}}`): begins
`tasks/motd/task.json:1:19: x1-prod: motd: `, once; and `a_unit_with_none_is_
refused_at_its_unit` (`{"live": {"rg": {"unit": "x.service", "action": "none",
"paths": ["/opt/rg/"]}}}`: `1:18`).
Don't: refuse an owner for a missing id (19.4); a `live` path outside the three
roots or one of them, a repeat, an empty `paths` (19.3); two consumers of one path (22).
Size: at most 220 lines.
Removes: nothing.

### 21.4 The `hearth` task: present, declared, and passing every rule
Scope: format.md 5.12; render.md 3.2; hearth 12's presence; log 68.
Needs: nothing beyond 21.3; hearth 42.3 (`Composition::Task`, `CONTAINERS`).
Files: render `src/{compose, call, errors}.rs`, `src/tests/{compose,
fixtures, errors}.rs`, `src/tests/fixtures/schemas.rs`;
protocol, records and `tests/builder` as 21.1's; both fixtures modules: `THIN`
and the scripted `x1_prod` gain `tasks/hearth/task.json` `{}` (a ripple past
README's 20 lines, as 17.1's: a planner's ruling), and each repository that
is not built on them: `MOTD` and `CONTAINERS` (`tests/builder/src/fixtures.rs`,
`:237`, `:646`). `LAYERS` and `CLUSTER` need nothing, and `CONTAINED`
(`thin_with`) has THIN's; `VPN`, if hearth 20 lands it without `thin_with`.
The ripple, at hearth c8f830d:
- render's unit fixtures: `schemas.rs:176`'s `Present` gains `implicit:
  false`; `removed_task` (`fixtures.rs:203`) gains a `tasks/hearth/task.json`
  entry, its declaration (`declaration(&[], &[])`) and `call.defaults =
  schema::declarations(..)`; the list capacities at `fixtures.rs:692` and
  `:712` grow by one (4 to 5, 5 to 6), as `tests/errors.rs:116`'s count (4 to
  5). `tests/compose.rs:140`, `:219`, `:238` and `:251` read
  `only(&settings.tasks)`, which now holds two: they read a new
  `fixtures::task_of(&Settings) -> &Present`, the task not `implicit`;
- the render world: the scripted `x1_prod` (`tests/render/src/fixtures.rs:38`)
  gains the entry (oid `h`) and an empty `Declaration`;
  `tests/render/tests/compose.rs:21`'s reads gain its oid, `thin.rs:121`'s
  count goes from 4 to 5, and `thin.rs:142`'s reads gain `oid(b'h')`.
Shape: `compose::compose` adds `hearth` to the present tasks of a target whose
placement is not a container, in name order, `implicit`. Naming `hearth` in a
presence layer (base, member, base-env) of a container, with any value, is
`HearthContainer`; `"hearth": null` in any layer, `fleet.json` and an
environment's included, `HearthNull` (42.3's `Null` skips `hearth`: no other
kind reports it), both `reserved`, at the key, `task` `hearth`, `target` named
(`fleet.json`'s and an environment's once, without a target). A present
`hearth` with no `tasks/hearth/task.json` is 42.3's `Composition::Task` at that
path, no position, for each such target: 42.3 added the variant, and 21.4
emits it for the implicit task (nothing on main does). A non-null `hearth` in a base or member of
any other target is accepted: one `Present` (not two), `implicit` false.
`fleet.json` and `envs/` keys for `hearth` are settings, as for any task; a
container has none: a key for it on a container is dropped, with no error. Nothing else:
the task's packages, user, software and presets are the repository's.
Test shape: unit, `src/tests/compose.rs`, under 20 ms:
`the_hearth_task_is_present_in_every_target_but_a_container`,
`a_fleet_key_for_hearth_reaches_the_implicit_task_as_settings`,
`naming_hearth_in_a_containers_layer_is_reserved_with_any_value`,
`hearth_null_in_a_base_fleet_json_or_an_environment_is_reserved_alone`, `a_missing_declaration_fails_each_target_
with_no_position`, `a_container_needs_no_declaration`, `a_named_hearth_in_a_metal_base_is_one_
present_task_not_implicit`, `a_fleet_or_env_key_for_hearth_on_a_container_is_
dropped_silently`, on a pair from `fixtures::target_with(placement: &str,
layers: [Option<&str>; 6]) -> (Documents, Target)` (17.1's `debian_layers`
form; metal, container): names are `hearth` for the first and none for the
second (format.md, 5.12).
Proof: process, bound 0.5 s each. `the_hearth_task_of_5_12_passes_every_
reserved_rule` on `HEARTH` (THIN's files but `tasks/hearth/`: 5.12's `task.json`
whole, its `sha256` `ca2dff16…fc7fab9`, the digest of `hearth 0.1.0` LF;
`tree/etc/systemd/system/hearth-activator.service`, `tree/usr/local/lib/hearth/
askpass` (100755), `tree/etc/hearth/alarms.json.j2`): exit 0, and
`lines_starting` over `  request `, `  users `, `  members `, `  presets `,
`  download ` and `  fetch ` is exactly
```text
  request    git mmdebstrap openssh-client uidmap
  users      hearth 59999 hearth /nonexistent /usr/sbin/nologin
  members    hearth systemd-journal
  presets    hearth-activator.service enable
  presets    systemd-confext.service disable
  presets    systemd-sysext.service disable
  download   /usr/local/bin/hearth file 0755 root:root ca2dff161724
  fetch      ca2dff161724 https://git.ekanayaka.io/ai/hearth/releases/download/v0.1.0/hearth-x86_64-linux
```
`a_target_whose_hearth_task_declares_nothing_requests_no_package` (log 68, on
08.2's `testing/repos/test` as main has it: its `tasks/hearth/task.json`
holds a `doc` alone, and 21.4 leaves it so; a target with no other task):
`lines_starting` over `  request` is empty, exit 0.
`a_container_renders_without_a_hearth_declaration_and_a_host_does_not` waits
for the designer's answer to whether a container renders when its host
fails (ruling 162): main leaves such a container
out (42-listing-members-rules.md, Rules; `call/validation.rs:129`). Its host
half holds either way: on `NO_HEARTH` (42.3's `CONTAINERS` with
`chat-prod.json` overridden to `"on": "rhi-prod"`, as `CONTAINERS` places it
on `rhi-lab`; no `tasks/hearth`), asked `rhi-prod`, stdout is
`tasks/hearth/task.json: rhi-prod: no task hearth under tasks/` (the records
print no task for `Composition::Task`, `errors.rs:66`) and exit 2. `a_container_naming_hearth_is_refused_at_the_key` (`chat-prod.json`
`{"placement": {"kind": "container", "on": "rhi-prod"}, "hearth": {}}`): the
`hosts/` line begins `hosts/chat/chat-prod.json:1:56: chat-prod: hearth: `.
Don't: write the data part's `snapshots` and `backups` policies (M5, M6);
check `hearth`'s uid or range; refuse the task's own files.
Size: at most 220 lines.
Removes: 12.1's "Don't ... add the `hearth` task (hearth 21)".

## Interface sketches

render owns these; the records mirror them. Types derive `Clone, PartialEq,
Eq, Hash, Debug`.

```rust
// crates/hearth-domain-render/src/reserved.rs (21.1; paths 21.2; owners, live 21.3)
/// Where a reserved thing is declared: a position of `task.json`, or a tree file's own path.
pub enum Spot { Declared(Position), Tree }
/// One refusal: its task, place, second place, and what is reserved.
pub struct Refused { pub task: Box<[u8]>, pub spot: Spot, pub also: Option<Position>, pub reserved: Reserved }
/// A path installed, for `tree` and `live`: from 18's plan; `empty` of a rendered file from its bytes.
pub struct Installed<'a> { pub task: &'a [u8], pub path: &'a [u8], pub at: Option<Position>, pub shape: Shape }
pub enum Shape { File { empty: bool }, Link, Directory }
/// Words and paths of every entry (a container's modules and parameters too); each entry refused is hidden from `expansion.of`.
pub fn entries(expansion: &mut Expansion, target: &[u8], container: bool) -> Box<[Refused]>;
/// The installed paths against 5.11's rows.
#[must_use] pub fn tree(installed: &[Installed<'_>]) -> Box<[Refused]>;
/// `live` entries on never-live units and paths; `installed` holds the `hearth` task's.
#[must_use] pub fn live(expansion: &Expansion, installed: &[Installed<'_>]) -> Box<[Refused]>;
// src/entries.rs (16's): `Expansion` gains
impl Expansion { pub fn refuse(&mut self, map: Map, task: &[u8], key: &[u8]); } // `of` skips what it refused
// src/compose.rs (12's): `Present::implicit: bool` (`Install::at: Option<Position>` is 18.1's)

// crates/hearth-domain-render/src/errors.rs: `Kind` gains `Reserved(Reserved)`; `Composition` none
/// render.md 7.2's `reserved`: the rule, and the key, word, unit or path at fault.
pub struct Reserved { pub rule: Rule, pub what: Box<[u8]> }
pub enum Rule {
    ActivatorKey, RootKey, RootWait, CmdlineLength { bytes: u32 }, PasswordPreseed, TrialName, Overlay,
    TmpfilesSubvolume, TmpfilesWrite, TmpfilesPath, ContainerModule, ContainerParam, // 21.1
    ActivatorFile, Dropin, Accounts, SecretPath, HearthState, PackagesLayer, ClassMount, DataRoot, Runtime, LogFile,
    InitramfsConf, // 21.2
    SecretMode, HearthOwner, LiveNone, NeverLiveUnit, NeverLivePath, // 21.3
    HearthContainer, HearthNull, // 21.4
}
// translate/errors.rs: `error` gains the arm; the records' `Kind::Reserved(Reserved)` mirrors it variant for variant
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-21\.[1-4]'` prints 4.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run -p hearth-domain-render -p hearth-protocol-builder -p hearth-client-protocol-records -p hearth-render-world -p hearth-builder-world` passes.
3. `git -C ~/src/rust/hearth ls-tree -r --name-only origin/main crates/hearth-domain-render/src | grep -c -x -E '.*/src/reserved(/(words|paths|owners|live))?\.rs'` prints 5.
4. `git -C ~/src/rust/hearth grep -l -F tasks/hearth/task.json origin/main -- tests/builder/src/fixtures.rs tests/render/src/fixtures.rs | wc -l` prints 2.
5. `git -C ~/src/rust/hearth grep -n -E '\bVec\b|String::|[^:]: *String\b|-> *String\b|<String\b|&String\b|^\s*_ =>' origin/main -- crates/hearth-domain-render/src crates/hearth-protocol-builder/src/translate ':!*/tests/*' ':!*/tests.rs'` prints nothing (code-organization.md, 6).
