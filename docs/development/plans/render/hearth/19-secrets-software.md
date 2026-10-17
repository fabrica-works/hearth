# Session 19: secrets, passwords, identity, downloads, live; fixed ids

The parts that name what the activator and the build place (format.md, 9,
10.1, 5.14; render.md, 4.2 to 4.4): `secrets` (19.1), `software` (19.2),
`live` (19.3), render.md 3.3's fixed ids and packages' `seeds` (19.4), their
change in `hearth diff` (19.5). Each proves on the real loop. M1 renders no
`checks` part (Choices 1).

## Read first

- hearth's `format.md`: 5.2, 5.6, 5.8, 5.11, 5.13, 5.14, 9, 10.1; `render.md`:
  3.3, 3.6, 3.7, 4.2 to 4.5, 6.1, 7.2; `build.md`: 5.5, 6.
- This plan: `hearth/16-entries.md` (Choices, 16.3, sketches); `17-packages.md`
  (17.2); `18-tree-users.md` (18.3, 18.4); `10-client.md` (Choices, "Who adds
  what"); `11-thin-story.md` ("Where later sessions add"); `13-schema.md`
  (13.5); `25-diff.md` (25.1); `24-resolution.md` (24.1).
- skein at `f981746`: `skein-lib`'s `list.rs` (`push`:55), `map.rs` (`get`:68).

## What the design asks

- **Secrets** (9.1 to 9.4): `path`, `mode` (default `0600`), `owner`, `group`
  (default `root`); "Names and places, never values."
- **Downloads** (10.1; render.md 4.2): `path` "under `/opt` or `/usr/local`",
  `mode` "at most `0777`"; the URL "stays outside the digest, as a fetch hint".
- **Live** (5.14; render.md 4.4): `paths` "under `/etc`, `/usr/local` or
  `/opt`"; "map path → list of { task, unit?, action }, in path order".
- **Fixed ids** (render.md 3.3, 4.2, 7.2): owners "must be a declared user or
  group with a fixed `uid` or `gid`, package users included"; `seeds` hold them.

## Choices this plan makes

1. **No `checks` part in M1.** 5.13 holds it "from M3"; 04.1 refuses `checks`
   until M5. `bless` is decoded, translated (13's `bless: _`) and placed in
   no part before M3 (format.md 5.13; `later.md` names M3).
2. **A part is present when empty** (render.md 4.1; 18's Choices 6): with no
   entry (for `software`: no download and no `software.sh`) `secrets`,
   `software` and `live` are their records with empty fields, and print
   their `part` line alone; THIN_TEXT gains the three lines (`secrets`
   `ac870b5c968e`, `software` `656d135b92c3`, `live` `6f1b7fda0c21`; with 19.5's
   `seeds`, THIN's spec is `405bf3bde392`: findings/digests.md). JSON: the record under the part's
   name (client/README.md 3.2).
3. **One file a part** (10's Choices): render `parts/<name>.rs` holds `build`,
   `digest` (`H("part:<name>", the record)`), `change`; `kinds/` an entry's
   check; protocol `translate/parts/<name>.rs`; records `parts/<name>.rs`.
4. **Reads.** `kinds::secrets` reads `password` from `Expansion::of(Map::Users)`
   (18 reads none). 19.2 keeps the `software.sh` row 18.4 drops. Entries of two
   tasks with one name: the first task's stands (18's Choices 8; each drop
   `// transitional: hearth 22`); 22 refuses.
5. **Errors** reuse `Schema::{Enum, Malformed, MissingField, UndeclaredField,
   Above, Below}` at the field's key (`Made::at`), task named; the sketches
   add `Beneath`, `Sha256` and `Ids`.
6. **Identity in M1** is `ssh-host-key` alone (format.md 9.3): the other
   kinds are `Enum` faults; `later.md` has their row.
7. **Text** (log 9), label cell 11 as 9.2: `secret <name> <path> <mode>
   <owner>:<group>`; `password <user> <secret>`; `identity <item> <kind>
   <algorithm> <path> <public_path|-> <mode> <owner>:<group>`; `download <path>
   file <mode>|tar <strip> <owner>:<group> <sha12>`; `scripts <task> <order>
   <digest12>`; `fetch <sha12> <url>`; `path <path> <task> <unit|-> <action>`;
   `seed user <name> <uid> <gid|->`; `seed group <name> <gid>` (after `apt`).
   `fetch` is the first path's URL of one sha256, as written (render.md 4.2);
   consumers of a path sort by task, unit (`-` first), action (4.4). Text no
   example shows (secrets, downloads, fetch, live, seeds, change rows) is
   these lines (the plan decides).
8. **The change** (6.1): secrets by name only, never a value; downloads by
   path, the row after; scripts as 18's; live by path; packages' `seeds`:
   users and groups added, removed, changed, by name. Layers: secrets `Slot`, software `Software`, live `Other` (`rebuild
   nothing`, 25.1).

## Rules that bind it

- **The meeting point** (README, Rules): each increment proves on the real
  loop; a part adds all three variants at once; no wildcard, no `..`.
- **No recursion, no `Vec`, `String` or `_` arm** (render.md, 8.3): every walk a
  `for`, rows in `skein_lib::List`. Proof digests are `sha256sum`'s; part
  digests stay out (`lines_starting`, 12.1's).
- **Shared files:** `records.rs`, `src/parts.rs`, `translate/parts.rs`, render's
  `lib.rs`, `kinds.rs`, `errors.rs`, `diff.rs`, `parts/packages.rs`, both
  fixtures modules.

## Increments

### 19.1 The `secrets` part: files, passwords, identity
Scope: format.md 9.1 to 9.4; render.md 4.3 `secrets`; `PartRecord::Secrets`.
Needs: nothing beyond the session's start (hearth 16).
Files: render new `src/parts/secrets.rs`, `src/kinds/{secrets, identity}.rs`,
`src/{parts, kinds, call, errors}.rs`, `src/tests/fixtures.rs`, new
`src/tests/secrets.rs`; protocol new `src/translate/parts/secrets.rs`,
`src/translate/{parts, errors}.rs`, `src/tests/translate.rs`; records new
`src/parts/secrets.rs`, `src/{parts, records, errors}.rs`, `src/form/{text,
json}.rs`, `src/tests/{fixtures, secrets}.rs`; `tests/builder/{src/fixtures.rs,
tests/real.rs}`.
Shape: the sketches. A secret: key `Format::Secret` (2.4), `path` required
(`Format::Path`), `mode` `Format::Mode` default `0600`, `owner` and `group`
default `root`, unchecked here. A password: each user entry's `password`
(`Format::Secret`), by user. Identity: `kind` `ssh-host-key` (`Enum` lists it),
`algorithm` required (`MissingField`), `path` required, `public_path`
optional, `mode` default `0600`.
Test shape: unit, `src/tests/secrets.rs`, under 20 ms; fixtures gain
`expansion(map, rows)`, `secret_file`. Reference:
```rust
// crates/hearth-domain-render/src/tests/secrets.rs
#[test]
fn a_secret_defaults_to_mode_0600_owned_by_root() {
    let rows = [(&b"mail/tls/mx1.key"[..], &[(Name::Path, &b"/etc/ssl/private/mx1.key"[..])][..])];
    let secrets = secrets::build(&fixtures::expansion(Map::Secrets, &rows)).expect("valid");
    let want = fixtures::secret_file(b"mail/tls/mx1.key", b"/etc/ssl/private/mx1.key", 0o600, b"root", b"root");
    assert_eq!(&*secrets.files, &[want], "format.md, 9.1: mode 0600, owner and group root");
}
```
Then `a_secret_name_out_of_2_4s_form_is_malformed` (table), `a_secret_without_
a_path_is_a_missing_field`, `a_mode_that_is_not_octal_is_malformed`,
`a_password_names_a_secret_by_user`, `an_ssh_host_key_needs_its_algorithm`,
`an_identity_without_a_kind_is_a_missing_field`, `an_algorithm_but_the_three_
is_an_enum_fault`, `an_identity_kind_but_ssh_host_key_is_an_enum_fault`,
`no_entry_makes_an_empty_part`; translate `each_secrets_field_translates_to_the_
records_and_back`; records `secrets_print_as_choices_7_shows`,
`json_nests_the_record_under_secrets`.
Proof: process, bound 0.5 s, on `SECRETS` (THIN's files but `tasks/motd/
task.json` `{"secrets": {"mail/dkim/example.org.key": {"path": "/etc/dkimkeys/
example.org.key", "mode": "0640", "group": "opendkim"}, "mail/tls/mx1.key":
{"path": "/etc/ssl/private/mx1.key"}}, "identity": {"ssh-ed25519": {"kind":
"ssh-host-key", "algorithm": "ed25519", "path": "/etc/ssh/ssh_host_ed25519_key",
"public_path": "/etc/ssh/ssh_host_ed25519_key.pub"}}, "users": {"free":
{"uid": 1000, "password": "users/free.passwd"}}}`):
```rust
// tests/builder/tests/real.rs
#[test]
fn hearth_render_lists_secrets_passwords_and_identity_by_name_and_place() {
    let repository = Repository::make("secrets", &fixtures::thin_with(fixtures::SECRETS));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    let lines = fixtures::lines_starting(&rendered.stdout, &["  secret ", "  password ", "  identity "]);
    let expected = concat!(
        "  secret     mail/dkim/example.org.key /etc/dkimkeys/example.org.key 0640 root:opendkim\n",
        "  secret     mail/tls/mx1.key /etc/ssl/private/mx1.key 0600 root:root\n",
        "  password   free users/free.passwd\n",
        "  identity   ssh-ed25519 ssh-host-key ed25519 /etc/ssh/ssh_host_ed25519_key /etc/ssh/ssh_host_ed25519_key.pub 0600 root:root\n");
    assert_eq!(lines, expected, "format.md, 9.4: names and places, never values");
}
```
Beside it `a_secret_without_a_path_prints_its_fault_at_the_entry` (`{"secrets":
{"mail/tls/mx1.key": {}}}`: stdout begins `tasks/motd/task.json:1:14: x1-prod:
motd: `, holds `path`; exit 2). Real git, 16's expansion, 40.2's printer.
Don't: refuse a mode past `0770`, a hearth-owned file or a path (21); compare
tasks' secrets (22); check owners (19.4).
Size: at most 300 lines.
Removes: nothing.

### 19.2 The `software` part: downloads and `software.sh`
Scope: format.md 10.1, 5.8; render.md 4.2 `software`; `PartRecord::Software`;
`Schema::Beneath`, `Format::Sha256` (13.5's `forms.rs`).
Needs: hearth 18.4 (`kinds::scripts` reads and checks `software.sh`).
Files: as 19.1's with `software`, `kinds/software.rs`, `src/tests/software.rs`,
and 18.4's `kinds/scripts.rs`, `src/tests/kinds.rs` (the removed test).
Shape: `url`, `sha256` (`Sha256`) and `unpack` (`file` or `tar`, `Enum`)
required; `path` required, `Format::Path`, beneath `/opt` or `/usr/local`, never
either itself (`Beneath`, on a segment; format.md 10.1). `file`: `mode` default
`0755`, at most `0777` (`Above`), `strip` an `UndeclaredField`. `tar`: `strip`
default 0, 0 to 255 (`Below`, `Above`), `mode` `UndeclaredField`. Owners default `root`, unchecked here (19.4). `build`
takes 18.4's `software.sh` rows (by order, then task) and one `fetch` a sha256,
the first path's; a task with `software.sh` and no download still makes the part
(scripts are in the digest). `digest` encodes `{downloads, scripts}` alone.
Test shape: unit, `src/tests/software.rs`, under 20 ms. Reference:
```rust
// crates/hearth-domain-render/src/tests/software.rs
#[test]
fn the_url_stays_outside_the_digest() {
    let (one, other) = (fixtures::software(b"https://a.example/rg"), fixtures::software(b"https://b.example/rg"));
    assert_eq!(software::digest(&one), software::digest(&other), "render.md, 4.2: a mirror rebuilds nothing");
    assert_ne!(one.fetch, other.fetch, "the hint is in the record");
}
```
Then `a_file_defaults_to_mode_0755_and_a_tar_to_strip_0`, `a_file_mode_past_
0777_is_above`, `a_path_outside_opt_and_usr_local_is_beneath` (table:
`/srv/tool`, `/optional`, `/usr/localx`, `/opt`, `/usr/local`),
`a_strip_past_255_is_above`, `strip_on_a_file_and_mode_on_a_tar_
are_undeclared_fields`, `a_sha256_that_is_not_64_lower_hex_is_malformed`,
`two_paths_of_one_sha256_keep_the_first_url`, `a_software_script_is_digested_
in_the_software_part`, `a_software_script_without_a_download_makes_a_
software_part`; translate and records as 19.1's.
Proof: process, bound 0.5 s, on `SOFTWARE` (THIN's files; `tasks/motd/software.sh`
`rg --version` LF; `tasks/motd/task.json` this one line, ripgrep's `sha256` a
fixture's, `printf ripgrep | sha256sum`, the design giving none):
```json
{"software": {"forgejo": {"url": "https://codeberg.org/forgejo/forgejo/releases/download/v15.0.0/forgejo-15.0.0-linux-amd64", "sha256": "7b36e8a38247d55d927349bd9083ffa10c819d2fedab7ce32be0943a760747ed", "unpack": "file", "path": "/opt/forgejo/bin/forgejo"}, "ripgrep": {"url": "https://github.com/BurntSushi/ripgrep/releases/download/15.1.0/ripgrep-15.1.0-x86_64-unknown-linux-musl.tar.gz", "sha256": "33165223a0a3c347557831c78e3780cafe5f22ef61a0dbf2a53ef9a3756eb1c9", "unpack": "tar", "strip": 1, "path": "/opt/ripgrep"}}, "scripts": {"software.sh": {"order": 500}}}
```
`hearth_render_lists_downloads_and_the_software_script` (`lines_starting`,
`  download `, `  scripts `, `  fetch `) is exactly
```text
  download   /opt/forgejo/bin/forgejo file 0755 root:root 7b36e8a38247
  download   /opt/ripgrep tar 1 root:root 33165223a0a3
  scripts    motd 500 1632d8753eb0
  fetch      33165223a0a3 https://github.com/BurntSushi/ripgrep/releases/download/15.1.0/ripgrep-15.1.0-x86_64-unknown-linux-musl.tar.gz
  fetch      7b36e8a38247 https://codeberg.org/forgejo/forgejo/releases/download/v15.0.0/forgejo-15.0.0-linux-amd64
```
Beside it `a_path_outside_opt_prints_its_fault_at_the_path_key` (`SOFTWARE_OUT`,
`{"software": {"tool": {` forgejo's `url`, `sha256`, `"unpack": "file",
"path": "/srv/tool"}}}` on one line, `path` at column 220): stdout begins
`tasks/motd/task.json:1:220: x1-prod: motd: `, holds `/opt`; exit 2.
Don't: refuse `/usr` or a setuid mode as reserved (21); overlaps (22); owners
(19.4); an `artifact` (04.2 refuses it, M6).
Size: at most 300 lines.
Removes: 18.4's drop of `software.sh`'s row, and its test
`a_software_script_is_checked_not_digested`.

### 19.3 The `live` part
Scope: format.md 5.14's declarations; render.md 4.4 `live`; `PartRecord::Live`.
Needs: nothing beyond the session's start.
Files: as 19.1's with `live`, `kinds/live.rs`, `src/tests/live.rs`.
Shape: `action` required, one of five (`Enum`); `unit` required unless `none`
(`MissingField`), a name (5.14 "unit name", `Malformed`); `paths` required, not
empty (`MissingField`), each absolute, one trailing `/` set aside (it covers a
subtree and is kept) before `Format::Path`, beneath `/etc`, `/usr/local` or
`/opt`, never one of them (`Beneath`, on a segment); a path twice in one entry
is `Malformed` at the second (5.14). `build` inverts the entries: a path to
its consumers `{task, unit?, action}`, equal ones once, paths in byte order,
consumers by task, unit (`-` first), action by name bytes (render.md 4.4).
Test shape: unit, under 20 ms: `two_entries_on_one_path_make_two_consumers`,
`equal_consumers_of_one_path_make_one`, `a_unit_is_required_but_with_none`,
`a_unit_that_is_no_unit_name_is_malformed`, `an_empty_paths_is_a_missing_
field`, `a_path_twice_in_one_entry_is_malformed`, `a_path_outside_the_three_
roots_or_one_itself_is_beneath` (table: `/etcetera`, `/etc/`, `/opt`), `a_trailing_slash_covers_a_
subtree_and_is_kept`, `no_entry_makes_an_empty_part`; translate and records as
19.1's.
Proof: process, bound 0.5 s, on `LIVE` (`tasks/motd/task.json` `{"live":
{"postfix": {"unit": "postfix.service", "action": "reload", "paths":
["/etc/postfix/", "/etc/mailname"]}, "ripgrep": {"action": "none", "paths":
["/opt/ripgrep/"]}}}`): `hearth_render_lists_each_path_with_its_consumers`
(`lines_starting`, `  path `) is exactly
```text
  path       /etc/mailname motd postfix.service reload
  path       /etc/postfix/ motd postfix.service reload
  path       /opt/ripgrep/ motd - none
```
Don't: refuse never-live units or paths, or `none` with a unit (21).
Size: at most 180 lines.
Removes: nothing.

### 19.4 Fixed ids: the `ids` kind and `seeds`
Scope: render.md 3.3, 4.2 `seeds`, 7.2 `ids`; format.md 5.6, 10.1.
Needs: hearth 18.3 (`Accounts`), 17.2 (`parts/packages.rs`), 19.1, 19.2.
Files: render new `src/ids.rs`, `src/parts/packages.rs` (17's), `src/{call,
errors, lib}.rs`, `src/tests/{fixtures, ids}.rs`; protocol `src/translate/
{parts/packages, errors}.rs`, `src/tests/translate.rs`; records
`src/parts/packages.rs`, `src/errors.rs`, `src/form/{text, json}.rs`,
`src/tests/{packages, errors}.rs`; `tests/builder/{src/fixtures.rs,
tests/real.rs}`.
Shape: once a target's entries are concrete, `ids::check` takes `Accounts` and
each owner and group of `Expansion::of` `Data`, `Secrets` and `Identity` (any
declared user, group) and `Software` (`root` or package ones), with their
`Made::at`. `root` is exempt (id 0). A user is fixed by its `uid`; a group by a
`groups` entry's `gid`, else by a sysusers user with a `uid` whose own group
it is, the group named as the user with the user's `uid` as `gid` (format.md
5.6; a `groups` entry of that name with another `gid` is 22's collision). A
package user gives both ids or neither: one alone is `Why::OneId`. A fault is
`Kind::Ids` at the field, `also` the declaration. `Seeds`: every package user
with both ids, every package group with a `gid` and every owner, by name; a
package user with neither seeds no line; empty, it is the record's empty
`seeds` (render.md 4.1): THIN's packages part is `64b1b99b813a`.
Test shape: unit, `src/tests/ids.rs`, under 20 ms. Reference:
```rust
// crates/hearth-domain-render/src/tests/ids.rs
#[test]
fn a_secrets_owner_without_a_uid_is_an_ids_fault_at_the_owner() {
    let accounts = fixtures::accounts(&[fixtures::sysusers(b"postfix", None)], &[]);
    let owned = [fixtures::owned(Owns::Secret, b"postfix", b"root", at(1, 71))];
    let faults = ids::check(&accounts, &owned).expect_err("postfix has no uid");
    let want = Kind::Ids(Ids { owner: Box::from(&b"postfix"[..]), of: Owns::Secret, why: Why::NoUid });
    assert_eq!((&faults[0].kind, faults[0].at), (&want, at(1, 71)), "render.md, 3.3: a fixed id");
}
```
Then `an_undeclared_owner_is_no_user`, `a_package_user_with_both_ids_is_fixed`,
`a_package_user_with_one_id_alone_is_an_ids_fault`,
`a_package_user_with_neither_id_seeds_no_line`,
`a_group_is_fixed_by_its_gid_or_a_users_own_group`, `root_needs_no_
declaration`, `a_download_owned_by_a_sysusers_user_is_not_a_package_user`,
`a_data_path_and_an_identity_file_are_checked_as_owned` (`Owns::Data`,
`Owns::Identity`), `an_undeclared_group_is_no_group`, `a_group_with_no_gid_is_
no_gid`, `a_download_group_that_is_no_package_group_is_refused`, `the_seeds_
are_the_package_ids_and_the_owners_sorted`, `the_seeds_leave_the_resolution_
inputs_alone` (as 17.3's), `empty_seeds_are_a_field_of_the_packages_record`,
`non_empty_seeds_move_the_packages_digest`; translate `an_ids_error_and_the_
seeds_translate_and_back`; records `seeds_print_as_choices_7_shows`.
Proof: process, bound 0.5 s, on `SECRETS_FIXED` (`SECRETS` plus `"groups":
{"opendkim": {"from": "package", "gid": 120}}`):
`hearth_render_seeds_the_owners_of_what_persists` (`lines_starting`, `  seed `)
is exactly `  seed       group opendkim 120`. Beside it
`an_owner_with_no_fixed_id_prints_its_fault_at_the_owner` (`SECRETS_NO_ID`:
`{"secrets": {"mail/tls/mx1.key": {"path": "/etc/ssl/private/mx1.key",
"owner": "postfix"}}}`) is exactly `tasks/motd/task.json:1:71: x1-prod: motd:
postfix owns mail/tls/mx1.key and has no fixed id: no user postfix is declared`,
exit 2.
Don't: refuse hearth's own users or modes (21); two users of one id (22).
Size: at most 260 lines.
Removes: nothing.

### 19.5 The parts' change, through `hearth diff`
Scope: render.md 6.1 (software, secrets; live per 25.1; packages'
`seeds`), 6.2; the `layer` and `PartChange` arms.
Needs: hearth 25.1 (`diff.rs`, `PartChange`, `Layer`), 17.5, 19.1 to 19.4.
Files: render `src/parts/{secrets, software, live, packages}.rs`, `src/diff.rs`,
`src/tests/diff.rs`; protocol `src/translate/{changes, parts/*}.rs`,
`src/tests/translate.rs`; records `src/diff.rs`, `src/parts/*.rs`,
`src/form/{text, json}.rs`, `src/tests/diff.rs`; `tests/builder/{src/
fixtures.rs, tests/real.rs}`.
Shape: the sketches. Both sides have every part, an empty one as its record.
Downloads and live paths are keyed as 18's `<Type>Keyed` (ruling 155) by path, a changed row its `after`;
secrets are `Named`, never a value; seeds are keyed by name (users
`<uid> <gid|->`, groups `<gid>`; list `seed user`, `seed group`); `layer` as
Choices 8. Text under the target's line:
`<mark> <list> <key>` padded to 52 characters, then the row (18's Choices 9).
Test shape: unit, under 20 ms: `a_changed_download_is_keyed_by_path`
(reference, as 25.1's), `secrets_change_by_name_only`,
`a_live_path_gaining_a_consumer_is_changed`,
`seeds_change_as_users_and_groups_added_removed_changed_by_name`, `the_three_layers_are_slot_software_other`;
translate `each_secrets_software_and_live_change_translates_back`; records the
JSON of each.
Proof: process, bound 0.5 s each, `fixtures::diff`, `Asked::new`, real git:
`hearth_diff_names_a_download_moved_and_rebuilds_from_software`:
`SOFTWARE_MOVED` (ripgrep at 15.1.1, `sha256`
`d2571cab39973c001a1734a8c432acb9111d135a97958989ba504dc8241d8bde`) prints
(`  software `) `  software ~ /opt/ripgrep` padded to 52 characters, then
`tar 1 root:root d2571cab3997`; exit 1, JSON `"kind":"from_software"`.
`hearth_diff_names_a_secret_moved_and_prepares_a_new_slot`: `SECRETS_MOVED`
(`mx1.key` mode `0640`, `free`'s password `users/free2.passwd`) prints
(`  secrets `) `  secrets  ~ secret mail/tls/mx1.key` and `  secrets  ~
password free`, `"kind":"slot_only"`.
`hearth_diff_a_live_path_added_rebuilds_nothing`: `LIVE_MOVED` (`/etc/nginx/`
for `motd`) prints `  live     + /etc/nginx/` padded to 52 characters, then `motd
nginx.service reload`; `"kind":"nothing"`, exit 1.
`hearth_diff_seeds_alone_rebuild_from_packages`: `SECRETS_FIXED_MOVED`
(`opendkim`'s gid 121; `lines_starting` over `x1-prod`, `  packages `) prints
exactly, 45.2 not landed (its resolution `Needed`, 25.1's line):
```text
x1-prod                                             rebuild from packages
  packages ~ seed group opendkim                    121
```
JSON `"kind":"from_packages"`, exit 1. Once 45.2 has landed the first line
ends in `known`, in 45.2's form, and this test follows it.
Don't: show a secret's before and after; ask hunks of parts.
Size: at most 260 lines.
Removes: nothing.

## Interface sketches

Types derive `Clone, PartialEq, Eq, Hash, Debug`; the records mirror render's.
```rust
// crates/hearth-domain-render/src/parts/secrets.rs (19.1; change 19.5)
/// render.md 4.3's `secrets` (9.1 files, 9.2 passwords, 9.3 identity); values never.
pub struct Secrets { pub files: Box<[SecretFile]>, pub passwords: Box<[Password]>, pub identity: Box<[Identity]> }
pub struct SecretFile { pub name: Box<[u8]>, pub path: Box<[u8]>, pub mode: u16, pub owner: Box<[u8]>, pub group: Box<[u8]> }
pub struct Password { pub user: Box<[u8]>, pub secret: Box<[u8]> }
pub struct Identity { pub name: Box<[u8]>, pub kind: IdentityKind, pub path: Box<[u8]>, pub public_path: Option<Box<[u8]>>, pub mode: u16, pub owner: Box<[u8]>, pub group: Box<[u8]> }
pub enum IdentityKind { SshHostKey { algorithm: Algorithm } }
pub enum Algorithm { Ed25519, Ecdsa, Rsa }
pub fn build(expansion: &Expansion) -> Result<Secrets, Box<[Error]>>;
#[must_use] pub fn digest(secrets: &Secrets) -> Digest;
pub struct Change { pub files: Box<[Named]>, pub passwords: Box<[Named]>, pub identity: Box<[Named]> }
pub enum Named { Added(Box<[u8]>), Removed(Box<[u8]>), Changed(Box<[u8]>) }
#[must_use] pub fn change(before: &Secrets, after: &Secrets) -> Change;
// crates/hearth-domain-render/src/parts/software.rs (19.2; change 19.5)
/// render.md 4.2's `software` (artifacts are M6); `fetch` rides outside the digest.
pub struct Software { pub downloads: Box<[Download]>, pub scripts: Box<[Script]>, pub fetch: Box<[Fetch]> }
pub struct Download { pub path: Box<[u8]>, pub sha256: Digest, pub owner: Box<[u8]>, pub group: Box<[u8]>, pub how: How }
pub enum How { File { mode: u16 }, Tar { strip: u32 } }
/// A fetch hint: a sha256 and where to get it; `Script` is 18.4's.
pub struct Fetch { pub sha256: Digest, pub url: Box<[u8]> }
pub fn build(expansion: &Expansion, scripts: Box<[Script]>) -> Result<Software, Box<[Error]>>;
#[must_use] pub fn digest(software: &Software) -> Digest; // H("part:software", { downloads, scripts })
pub struct Change { pub downloads: Box<[DownloadKeyed]>, pub scripts: Option<ScriptsChange> }
#[must_use] pub fn change(before: &Software, after: &Software) -> Change;
// crates/hearth-domain-render/src/parts/live.rs (19.3; change 19.5)
/// render.md 4.4's `live`, in path order.
pub struct Live { pub paths: Box<[Covered]> }
pub struct Covered { pub path: Box<[u8]>, pub consumers: Box<[Consumer]> }
pub struct Consumer { pub task: Box<[u8]>, pub unit: Option<Box<[u8]>>, pub action: Action }
pub enum Action { Reload, Restart, ReloadOrRestart, Start, None }
pub fn build(expansion: &Expansion) -> Result<Live, Box<[Error]>>;
pub struct Change { pub paths: Box<[ConsumersKeyed]> }
#[must_use] pub fn change(before: &Live, after: &Live) -> Change;
// crates/hearth-domain-render/src/ids.rs (19.4); parts/packages.rs: `Packages` gains `seeds: Seeds`, its `Change` `seeds: SeedsChange`
/// One owner and group asked for a fixed id, where each was written.
pub struct Owned<'a> { pub task: &'a [u8], pub of: Owns, pub owner: &'a [u8], pub owner_at: Position, pub group: &'a [u8], pub group_at: Position }
pub fn check(accounts: &Accounts, owned: &[Owned<'_>]) -> Result<Seeds, Box<[Error]>>;
/// render.md 4.2's `seeds`: what build writes into passwd and group.
pub struct Seeds { pub users: Box<[SeedUser]>, pub groups: Box<[SeedGroup]> }
pub struct SeedUser { pub name: Box<[u8]>, pub uid: u32, pub gid: Option<u32> }
pub struct SeedGroup { pub name: Box<[u8]>, pub gid: u32 }
/// 19.5: `Packages::Change::seeds`, by name; a user's row is `SeedIds`, a group's its `gid`.
pub struct SeedsChange { pub users: Box<[SeedUserKeyed]>, pub groups: Box<[SeedGroupKeyed]> }
// each `<Type>Keyed` as 18's `GroupKeyed`, one enum per type (ruling 155): `ConsumersKeyed` holds `Box<[Consumer]>`, `SeedGroupKeyed` a `u32`
pub struct SeedIds { pub uid: u32, pub gid: Option<u32> }
// errors.rs: `Kind` gains `Ids(Ids)`, `Schema` gains `Beneath { allowed: Box<[Box<[u8]>]> }`, `Format` gains `Sha256`
/// render.md 7.2's `ids`: whose id is missing, of what, and why.
pub struct Ids { pub owner: Box<[u8]>, pub of: Owns, pub why: Why }
pub enum Owns { Data, Secret, Identity, Download }
pub enum Why { NoUser, NoGroup, NoUid, NoGid, OneId, NotPackageUser, NotPackageGroup }
// diff.rs: `PartChange` gains `Secrets`, `Software`, `Live`; parts.rs: `PartRecord` too. Translation, both ways:
#[must_use] pub fn secrets(secrets: render::parts::secrets::Secrets) -> records::parts::secrets::Secrets; // and `software`, `live`
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-19\.[1-5]'` prints 5.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run -p hearth-domain-render -p hearth-protocol-builder -p hearth-client-protocol-records -p hearth-builder-world` passes.
3. `git -C ~/src/rust/hearth ls-tree -r --name-only origin/main crates | grep -c -x -E '.*/src/(parts|kinds|translate/parts)/(secrets|identity|software|live)\.rs'` prints 13.
4. `git -C ~/src/rust/hearth grep -l a_software_script_is_checked_not_digested origin/main -- crates/hearth-domain-render/src/tests` prints nothing.
5. `git -C ~/src/rust/hearth grep -n -E '\bVec\b|String::|[^:]: *String\b|-> *String\b|<String\b|&String\b|^\s*_ =>' origin/main -- crates/hearth-domain-render/src crates/hearth-protocol-builder/src/translate ':!*/tests/*' ':!*/tests.rs'` prints nothing (code-organization.md, 6).
