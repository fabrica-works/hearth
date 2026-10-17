# Session 17: packages, `debian`, preseeds, the initramfs, the command line

The spec's first parts built from entries (render.md 4.2, 4.3), and the
document they read. 17.1 composes `debian` over format.md 3.2's layers, checks
it with hearth 13.5's `schema::debian` and fills the template context's root;
17.2 builds the `packages` part (sources, request, variant, apt), the inputs of
hearth 24's resolution key and the packages line; 17.3 reads `preseeds` and
`initramfs` into it; 17.4 builds the `boot` part from `cmdline` and placement;
17.5 gives both their change for `hearth diff`. Each proves on the real loop.

## Read first

- hearth's `format.md`: 3.2, 3.3 (last bullet), 4.3, 5.1 (table), 5.3, 5.4,
  7.7; `render.md`: 3.3, 4.1 to 4.3, 4.5, 4.6, 5.1, 5.6, 6.1, 6.2, 9.2;
  `store.md`: 3.1; `build.md`: 5.5.
- This plan: `hearth/16-entries.md` (Choices, 16.3, sketches); `13-schema.md`
  (13.5, sketches); `12-compose.md` (sketches); `15-files.md` (`Roots`);
  `14-evaluate.md` (`Value`); `03-json.md` (`Layer`); `10-client.md` (Choices,
  "Who adds what", 10.3); `11-thin-story.md` ("Where later sessions add");
  `25-diff.md` (25.1, sketches); `24-resolution.md` ("What hearth 24 needs of
  hearth 17"); `01-lib.md` (`Item`, `H`).
- skein at `841f487`, hearth's pin: `crates/skein-lib/src/map.rs` (`insert`:90,
  `get`:70, `iter`:126), `list.rs` (`push`:55). `programming-model.md`: 3, 6.3, 8.

## What the design asks

- **`debian`, packages** (5.3): "the one schema hearth fixes ... merges like
  any object", its root in templates (7.7); packages are "the union over the
  target's tasks", a component the target "does not enable is an error that
  names both".
- **Preseeds, modules** (5.3, 5.4): `owner` "default: the question's first
  segment"; render.md 4.2: `microcode` "`none` on a VM and `generic` on
  metal", a container has "no initramfs".
- **Command line** (4.3, 5.4): `param` "printable ASCII but a space, `"` and `'`", and GRUB's bytes;
  `order` 0 to 999, default 500, sorted "by order, then key, then task name";
  render.md 4.3: `boot` also holds `mutable`, "from placement".
- **Key, diff, spec** (render.md 5.1, 5.6, 6.1, 9.2): the key covers sources,
  request, variant and `install_recommends`, not "preseeds, the initramfs, the
  seeds, the proxy"; packages change "request: added, removed; ... before,
  after"; the packages line ends `resolution <key> needed|known`.

## Choices this plan makes

- **Entries.** Parts read `Expansion::of(map)` (16.3). One entry's check is
  `kinds/<map>.rs` (`Err`: a `Schema` and the field's position), the part
  `parts/<name>.rs`. Names are a sorted union; identical entries of several
  tasks make one; differing ones are hearth 22's (the first task's stands;
  `// transitional: hearth 22` on each drop), but packages are one a name, each declaration's `component` checked.
- **`debian`** is composed, checked and rooted once a target, after 13.4 and
  before 16's guards (which saw an empty root), and read back typed by
  `parts::packages::read` (`None`, only for a root 13.5 did not make, is
  `Schema::Missing`). Its faults are errors at the layer's path, no task.
- **Records** (4.6) are maps of their fields; sets sorted lists; components in
  fixed order; timestamps text; `security` left out with its pocket;
  `initramfs` left out for a container; `boot` is `{ mutable, params }`. Both
  parts are always present; the names are those hearth 24 needs of 17. Metal
  and VM are variant `Apt`, a container `Container`, a placeholder whose key
  stays needed, no error (render.md 4.2); its initramfs, module or parameter
  is 21's `reserved` refusal (7.2); metal's microcode is `generic`, a VM's
  `none`. Text no example shows (pockets, microcode, `apt`, owner, `boot`,
  `mutable`, changes) is the Shapes' (the plan decides).
- **Forms** (format.md 5.3, 5.4): `Format::{Preseed, Module, Param}` (17.2
  to 17.4) are `Malformed`, refused where written; `main` stays on, merged
  under `debian`'s default (5.3).
- **Errors** reuse 13's `Schema::{Enum, Malformed, MissingField, Below,
  Above}` at the field (`Made::at`) in `tasks/<task>/task.json`, task named;
  17 adds `Schema::Component` (17.2) and `Format::{Preseed, Module, Param}`.
- **The ending** `   resolution <key12> needed|known` is `form/text.rs`'s, after
  the line of the part `PartRecord::name` calls `packages` (other parts add no
  arm). Nothing gives `Spec::resolution` before hearth 24.1, so until then
  the line ends at its digest: unit-proved here, connected by 24.1.
- **Key sensitivity** (5.1, 5.6). `inputs` is the one function; 24.1's
  `keys::resolution` hashes these four values alone, H is injective over 3.1,
  so equal `Inputs` is an equal key: tests compare `Inputs`, one per input that
  moves the key, one per input that does not. The proxy is no field (24.2).
- **Not here:** `seeds` (19 adds the field, empty in the record from then: THIN's packages part is then `64b1b99b813a`, findings/digests.md), `tiny-initramfs.conf`
  (22, render.md 3.6), reserved keys, modules, `password` preseeds (21; until
  then `password` is an unlisted type), collisions (22).
- **Ripple.** From 17.1 each fixture repository carries `debian`; from 17.2
  each spec prints a packages part, so `THIN_TEXT` and spec digests are
  re-pasted by the README's Rules (Edits outside Files), each named in the
  report
  (`git grep -n -w -e THIN_TEXT -e d5942fa1a047 -e 9970fb4d8f97 -e e0d142ee5fb0 -- tests crates`;
  README, "A new part").

## Rules that bind it

- **The meeting point** (README, Rules): each increment proves on the real
  loop; a part variant adds render's, the records' and the translation's arms
  in one increment; `translate/` stays exhaustive, no wildcard, no `..`.
- **No recursion, no computed expectation:** every walk a `for`; digests are
  literals from the encoding by hand (3.1; `python3 -I`, once), as 11's.
  **Shared files:** `records.rs`, `src/parts.rs`, `translate/parts.rs`, render's
  `lib.rs`, `kinds.rs`, `errors.rs`, `compose.rs`, both fixtures modules.

## Increments

### 17.1 `debian`: composed over six layers, checked, in the template context
Scope: format.md 3.2, 5.3, 7.7; render.md 3.3; `Layer::debian`; the root.
Needs: nothing beyond the session's start (hearth 16).
Files: render `src/{revision, compose, call, lib}.rs`, `src/evaluate/
context.rs`, `src/tests/{compose, fixtures}.rs`, new `src/tests/debian.rs`;
protocol `src/translate/documents.rs`, `src/tests/translate.rs`; `tests/render/
{src/fixtures.rs, tests/debian.rs (new)}`; `tests/builder/{src/fixtures.rs,
tests/real.rs}`; render `src/call/{templates, guards}.rs` (the empty
`debian` root at `templates.rs:61`, `guards.rs:24,45`: `call.rs` passes the
target's root to both); the `Layer` literals that gain `debian`: render
`src/tests/fixtures/discovery.rs` (seven), `src/tests/fixtures/schemas.rs:222`,
protocol `src/tests/read.rs:60,145`, `src/tests/translate/composition.rs:38`;
each fixture repository's `fleet.json` (builder world `src/fixtures.rs`: THIN
:25, LAYERS :173, MOTD :230; `git grep -n -F '"fleet.json"' -- tests crates`
for others) gains `DEBIAN`'s `debian` member beside what it holds (LAYERS
keeps `"motd": {}`), nothing else (past README's 20-line limit: a planner's
ruling). `DEBIAN` is THIN's:
`{"debian": {"suite": "forky", "architecture": "amd64", "snapshot":
{"archive": "20261001T030405Z"}, "pockets": {"updates": false, "security":
false}}}`.
Shape: the sketches. `compose::debian` walks 3.2's layers as `compose` does
(one shared walk), `Merge::merge`ing each. Per target the call composes it,
then `schema::debian(composed, base document)`; a `Fault` is an `Error` at
`Documents::path` and its position; a fault gives no spec. When a layer sets
`debian`, `{"components": {"main": true}}` is merged first, beneath it, at
`Layer::Defaults`, document 0 (`compose.rs:359`'s provenance), as
`schema::debian`'s doc says (`schema/debian.rs:12`); when none does, `None`
stays `Schema::Missing`. The translation follows `optional_layer`
(protocol `translate/documents.rs:154`): `Option::map` is a disallowed method
(`clippy.toml:106`). The root is cloned into each task's `Roots::debian`.
Test shape: unit, under 20 ms; new `fixtures::debian_layers([Option<&str>; 6])`
and `fixtures::suite(&Composed)` (the suite and the layer that set it):
```rust
// crates/hearth-domain-render/src/tests/debian.rs
#[test]
fn six_layers_compose_debian_in_format_md_3_2s_order() {
    let (documents, target) = fixtures::debian_layers([Some("fleet"), Some("base"), Some("member"), Some("env"), Some("base-env"), None]);
    let composed = compose::debian(&target, &documents, &mut Merge::new(&limits::SHIPPED)).expect("five set it");
    assert_eq!(fixtures::suite(&composed), Some((&b"base-env"[..], Layer::BaseEnv)), "3.2: base-env, last but its member");
}
```
Then `a_null_debian_removes_what_earlier_layers_set`, `the_debian_root_reaches_
every_tasks_roots`, `an_if_and_an_interpolated_key_read_debian_suite`;
translate `a_documents_debian_translates_to_render_and_back`; world (eight
seeds, 0.5 s) `a_target_without_debian_has_no_spec_and_one_error`.
Proof: process, bound 0.5 s, on `DEBIAN_LAYERS` (THIN's files with these):
`fleet.json` `{"debian": {"suite": "trixie", "architecture": "amd64",
"snapshot": {"archive": "20261001T030405Z"}, "pockets": {"updates": false,
"security": false}}}`, `x1.json` `{"motd": {}, "debian": {"suite": "forky",
"components": {"contrib": true}}}`, `x1-prod.json` THIN's placement and
`"debian": {"components": {"non-free-firmware": true}}`, and
`tasks/motd/tree/etc/debian.j2`: `{{ debian.suite }} {{ debian.architecture }}
{{ debian.snapshot.archive }} {{ debian.components | keys | join(',') }}` LF.
`a_template_reads_the_debian_root_composed_over_three_layers` renders it
through `fixtures::render` and finds, in the text, `  /etc/debian` padded to
the kind column, then `file 0644 root:root f745f2a642a2` (`forky amd64
20261001T030405Z contrib,main,non-free-firmware` LF). Beside it
`a_debian_the_schema_refuses_prints_the_layer_that_set_it` (`x1.json`
`"debian": {"components": {"main": false}}`: one line, from `hosts/x1/x1.json:`
to `: x1-prod: debian.components must keep main on`; exit 2).
Don't: read `debian` typed (17.2); change `placement`'s replace-whole merge.
Size: at most 140 lines.
Removes: `debian: _` in the translation; 15.2's empty `debian` root.

### 17.2 The `packages` part: sources, request, variant, apt; the key's inputs
Scope: render.md 4.2 `packages`, 5.1, 5.6, 9.2; format.md 5.3 `packages`,
`component`; `PartRecord::Packages`; the line's `resolution` ending.
Needs: nothing beyond 17.1.
Files: render new `src/parts/packages.rs`, `src/kinds/packages.rs`, `src/{parts,
kinds, call, errors}.rs`, `src/tests/fixtures.rs`, new `src/tests/packages.rs`;
protocol new `src/translate/parts/packages.rs`, `src/translate/{parts,
errors}.rs`, `src/tests/translate.rs`; records new `src/parts/packages.rs`,
`src/{parts, records, errors}.rs`, `src/form/{text, json}.rs`, `src/tests/
{fixtures, errors}.rs`, new `src/tests/packages.rs`; `tests/render/src/
fixtures.rs`; `tests/builder/{src/fixtures.rs, tests/real.rs}`; the ripple's.
Shape: the sketches. `kinds::packages::entry`: the name in Policy 5.6.1's form
(2 to 256 bytes, the first `a-z0-9`, then `a-z0-9+-.`: format.md 5.3,
`Malformed { Debian }`; hearth 07's `Name::new` takes every `Resolve`),
`component` default `main`, one of four (`Enum`). `build`: sources from `read`;
`components` the keys that are `true`, in fixed order; `security` only with its
pocket (set with it off, nothing moves); the request the union of the names; a component off is `Schema::
Component` at the field, `also` where `debian.components` was set; `preseeds`
and `modules` stay empty until 17.3. Text (9.2's, the rest log 9's): `sources`
suite, architecture, components, archive, then ` updates`, ` security <ts>` if
on; `request` (absent when empty); `variant`; `microcode`; `apt` and
`install_recommends <bool>`.
Test shape: unit, under 20 ms; `fixtures::thin_packages()` is THIN's part; new
`fixtures::bytes(&[u8]) -> Item`, imported as `bytes`:
```rust
// crates/hearth-domain-render/src/tests/packages.rs
#[test]
fn the_resolution_inputs_are_the_four_values_as_the_record_encodes_them() {
    let inputs = packages::inputs(&fixtures::thin_packages());
    let sources = [Item::Map(5), bytes(b"architecture"), bytes(b"amd64"), bytes(b"archive"), bytes(b"20261001T030405Z"),
        bytes(b"components"), Item::List(1), bytes(b"main"), bytes(b"suite"), bytes(b"forky"), bytes(b"updates"), Item::False];
    assert_eq!(&*inputs.sources, &sources[..], "store.md 3.1: keys in byte order, no security with its pocket off");
    assert_eq!((&*inputs.request, &*inputs.variant), (&[Item::List(0)][..], &[bytes(b"apt")][..]), "render.md 5.1");
    assert_eq!(&*inputs.apt, &[Item::Map(1), bytes(b"install_recommends"), Item::False][..], "apt's one option");
}
```
Then, the key's sensitivity (Choices), nine `the_<input>_changes_the_resolution_inputs` (suite,
architecture, components, archive, updates, security, request, variant,
install_recommends: `assert_ne!` of `inputs` from `thin_packages()` and one
field edited); `the_packages_digest_is_h_of_part_packages_over_its_record`
(`de38a7ed…fe35d`), `a_request_is_the_union_of_the_tasks_names_sorted_once`,
`all_four_components_print_in_fixed_order`,
`a_security_timestamp_with_its_pocket_off_changes_nothing`,
`one_package_of_two_tasks_with_two_components_is_one_name_each_checked`,
`a_name_out_of_policys_form_is_refused_at_its_entry` (table),
`a_component_the_target_does_not_enable_names_both`, `the_variant_follows_the_
placement`, `every_value_schema_debian_makes_reads_back`; translate
`each_packages_field_translates_to_the_records_and_back`. Records, with 10.3's
`fixtures::with_resolution`: `the_packages_line_ends_with_its_resolution_as_
render_md_shows` (the line `packages     part 4be1c0d2a7f1   resolution
0c7d9e41c0d2 needed`, 9.2),
`a_spec_without_a_resolution_ends_the_line_at_its_digest`, `a_known_
resolution_ends_the_line_known`, `json_gives_the_packages_record_in_key_order`,
`a_component_error_names_the_package_and_the_component`.
Proof: process, bound 0.5 s, on `PACKAGES` (THIN's files and
`tasks/base/task.json` `{"packages": {"curl": {}, "tzdata": {},
"firmware-linux": {"component": "non-free-firmware"}}}`; `x1.json` `{"motd":
{}, "base": {}, "debian": {"components": {"non-free-firmware": true}}}`):
`a_target_prints_its_packages_part_from_debian_and_its_tasks`; `lines_starting`
over `packages `,
`  sources`, `  request`, `  variant`, `  microcode`, `  apt` is exactly
```text
packages     part 51cab42f8d2b
  sources    forky amd64 main non-free-firmware 20261001T030405Z
  request    curl firmware-linux tzdata
  variant    apt
  microcode  generic
  apt        install_recommends false
```
Beside it `a_component_debian_does_not_enable_prints_the_package_and_the_
component` (`x1.json` without `debian`: exactly `tasks/base/task.json:1:60:
x1-prod: base: firmware-linux needs component non-free-firmware, which
debian.components does not enable`, exit 2). THIN_TEXT gains after `spec` THIN's
part (`packages     part de38a7ed6276`, then `sources`, `variant`, `microcode`,
`apt`), `spec` `cf667f7c3bf8` (17 alone; the world's `x1_prod`: `db6a7238154a`).
The ending proves at the unit layer alone: no resolution exists before 24.1.
Don't: write a key (24.1); read preseeds or modules; write `seeds` (19).
Size: at most 300 lines (a document: within 2.5 times).
Removes: nothing.

### 17.3 Preseeds and the initramfs
Scope: format.md 5.3 `preseeds`, 5.4 `initramfs`; render.md 4.2 (`microcode`).
Needs: nothing beyond 17.2.
Files: render new `src/kinds/preseeds.rs`, `src/{kinds, parts/packages}.rs`,
`src/tests/{packages, fixtures}.rs`; records `src/parts/packages.rs`,
`src/tests/packages.rs`; `tests/builder/{src/fixtures.rs, tests/real.rs}`.
Shape: `kinds::preseeds::entry`: `type`, `value` required (`MissingField`),
`type` one of four (`Enum`), `owner` default the question's first segment. One
`debconf-set-selections` line (format.md 5.3; `Malformed { Preseed }`): a
question has two or more segments joined by `/`, an owner one, each of
`a-zA-Z0-9._+-`; a value no control byte and no final `\`; a `boolean`'s is
`true` or `false`. A module (`Malformed { Module }`) is `a-zA-Z0-9_-`, 1 to 55
bytes. `build` fills `preseeds` (by question) and `modules` (keys of
`Expansion::of(Map::Initramfs)`, sorted once; none for a container). Text:
`preseed    <question> <type> <value>` and ` owner <o>` unless the default, a
line each; `initramfs  <modules>` when any.
Test shape: unit, under 20 ms. The key's no-change half (Choices), three tests as 17.2's
`assert_ne!` ones, each `assert_eq!` on `inputs` and `assert_ne!` on `digest`
(they are in the part, so in the packages key, 4.5): `the_preseeds_leave_the_
resolution_inputs_alone`, `the_initramfs_modules_leave_the_resolution_inputs_
alone`, `the_microcode_choice_leaves_the_resolution_inputs_alone`. Then
`a_preseed_owner_defaults_to_the_questions_first_segment`,
`a_preseed_question_or_owner_out_of_form_is_malformed` (one segment; `a b`),
`a_preseed_value_with_a_control_byte_or_a_final_backslash_is_malformed`,
`a_boolean_preseed_is_true_or_false`,
`a_module_name_out_of_form_or_past_55_bytes_is_malformed`,
`a_preseed_without_type_or_value_is_a_missing_field`,
`an_unlisted_preseed_type_is_an_enum_fault` (`password`, `text`),
`modules_are_a_sorted_union_without_repeats`,
`a_container_has_no_initramfs_whatever_tasks_declare`,
`the_microcode_is_generic_on_metal_and_none_on_a_vm`; records
`preseeds_and_modules_print_as_render_md_9_2_shows`.
Proof: process, bound 0.5 s, on `PRESEEDED` (`PACKAGES`' files but `tasks/base/
task.json` `{"packages": {"curl": {}, "tzdata": {}}, "preseeds":
{"tzdata/Areas": {"type": "select", "value": "Europe"}}, "initramfs":
{"btrfs": {}, "nvme": {}}}`
and `x1.json` `{"motd": {}, "base": {}}`): `preseeds_and_modules_reach_the_
packages_part` prints `packages     part 552be03a9eec`, 17.2's lines for
`curl tzdata` and main alone, with `preseed    tzdata/Areas select Europe`
and `initramfs  btrfs nvme` between `variant` and `microcode`, as 9.2 shows.
Don't: refuse `password` as `reserved` (21); compare tasks' preseeds (22).
Size: at most 180 lines.
Removes: 17.2's empty `preseeds` and `modules`.

### 17.4 The `boot` part: the command line and `mutable`
Scope: format.md 4.3, 5.4 `cmdline`; render.md 4.3 `boot`; `PartRecord::Boot`.
Needs: nothing beyond 17.2.
Files: as 17.2's, but render new `src/parts/boot.rs`, `src/kinds/cmdline.rs`,
`src/tests/boot.rs`; protocol new `src/translate/parts/boot.rs`; records new
`src/parts/boot.rs`, `src/tests/boot.rs`.
Shape: `kinds::cmdline::entry`: `param` required, bytes `0x21` to `0x7e` but
`"`, `'`, `\ $ ; & | < > { } #` (`Malformed { Param }`: format.md 5.4, which
GRUB's script would read); a container's `params` are empty (render.md 4.3;
21 refuses an entry); `order` default 500, 0 to 999 (`Below`, `Above`) at
the `order` field. `build`: parameters by (order, key, task); a key two tasks
give once, the first task's (`// transitional: hearth 22`); `mutable` from the composed
placement. Text: `boot         part <d>`, `  params     <p> …` (absent when
none), `  mutable    <true|false>`.
Test shape: unit, under 20 ms: `parameters_sort_by_order_then_key_then_task`
(100, 500, 900), `order_defaults_to_500_and_ranges_over_0_to_999`,
`a_param_with_a_space_a_quote_a_shell_byte_or_a_control_byte_is_malformed`
(table: `"`, `'`, `\`, `$`, `;`, `&`, `|`, `<`, `>`, `{`, `}`, `#`),
`a_containers_boot_part_has_no_params`, `a_missing_param_is_a_missing_field`, `an_empty_param_is_malformed`,
`identical_entries_of_two_tasks_make_one_parameter`,
`a_key_two_tasks_give_differently_keeps_the_first_tasks`, `mutable_comes_from_the_
composed_placement`, `the_boot_digest_is_h_of_part_boot_over_its_record`
(`bc6cab41…a077`); translate and records as 17.2's.
Proof: process, bound 0.5 s, on `CMDLINE` (THIN's files and
`tasks/params/task.json` `{"cmdline": {"console": {"param":
"console=ttyS0,115200n8", "order": 900}, "audit": {"param": "audit=0"},
"quiet": {"param": "quiet", "order": 100}}}`; `x1.json` `{"motd": {}, "params":
{}}`): `the_boot_part_lists_the_parameters_in_order` prints `boot         part
68ef6aa14d25`, `  params     quiet audit=0 console=ttyS0,115200n8`, `  mutable
false`. Beside it `an_order_past_999_prints_its_fault_at_the_field`
(`{"cmdline": {"loud": {"param": "debug", "order": 1000}}}`: stdout begins
`tasks/params/task.json:1:41: x1-prod: params: `, holds `999`; exit 2) and
`placement_mutable_true_prints_mutable_true`. THIN_TEXT gains `boot         part
bc6cab417252` and `  mutable    false`, `spec` `dd576929cc4d` (17 alone; the
world's `a46334dcd355`).
Don't: refuse reserved keys or a 2048-byte line (21); collide tasks (22).
Size: at most 220 lines.
Removes: nothing.

### 17.5 The packages and boot parts' change, through `hearth diff`
Scope: render.md 6.1 (packages, boot), 6.2; the `layer` and `PartChange` arms.
Needs: hearth 25.1 (`diff.rs`, `PartChange`, `Layer`; 40.1's records).
Files: render `src/parts/{packages, boot}.rs`, `src/diff.rs`, `src/tests/diff.rs`;
protocol `src/translate/{changes, parts/packages, parts/boot}.rs`, `src/tests/
translate.rs`; records `src/diff.rs`, `src/parts/{packages, boot}.rs`,
`src/form/{text, json}.rs`, `src/tests/diff.rs`; `tests/builder/{src/fixtures.rs,
tests/real.rs}`.
Shape: the sketches. `request` has the names only `after` has (added) and only
`before` has (removed), sorted; a field equal on both sides is `None`, left
out of the JSON (`"packages":{"request":{"added":[…],"removed":[…]},…}`); so
is `request` (no `Option`) when both its lists are empty.
`layer`: packages `Packages`, boot `Slot`; 45.2 adds `resolution`. Text under
the target's line (log 9): `  packages + <names>`, `  packages - <names>`,
`  packages ~ <field> <before> → <after>`, `  boot     ~ <before> → <after>`.
Test shape: unit, under 20 ms: `the_packages_change_lists_added_and_removed_
names_sorted` (reference, as 25.1's), `a_field_equal_on_both_sides_is_left_out`,
`a_changed_architecture_is_a_sources_pair`, `boot_changes_as_its_before_and_
after`, `the_boot_part_is_a_slot_layer_and_packages_the_lowest`,
`a_preseed_only_change_prints_preseeds_and_rebuilds_from_packages`,
`an_initramfs_only_change_prints_initramfs_and_rebuilds_from_packages` (both
with `inputs` equal, so 45.2 prints `known`); translate
`each_packages_and_boot_change_translates_and_back`; records the JSON of both,
`an_equal_request_is_left_out_of_the_json`.
Proof: process, bound 0.5 s, on `PACKAGES` at two commits (`repository.next`
adds `"vim": {}` to `base`): `hearth_diff_lists_an_added_package_and_rebuilds_
from_packages`, `fixtures::diff` with `Asked { form: Form::JsonLines, .. }`,
holds `"added":["vim"],"removed":[]` and `"kind":"from_packages"`, exit 1;
beside it `a_changed_command_line_is_a_new_slot_of_the_same_output` (`CMDLINE`
without `quiet`: `"kind":"slot_only"`, exit 1): both specs, real git.
Don't: give a `resolution` (45.2); ask hunks of parts.
Size: at most 260 lines.
Removes: nothing.

## Interface sketches

render owns these; the records mirror them field for field. Types derive
`Clone, PartialEq, Eq, Hash, Debug`.

```rust
// crates/hearth-domain-render/src/compose.rs (17.1); revision.rs: `Layer` gains `pub debian: Option<Value>`
/// format.md 3.2's layers' `debian` merged in order, members' included; `None` when none sets it.
pub fn debian(target: &Target, documents: &Documents, merge: &mut Merge) -> Option<Composed>;
// crates/hearth-domain-render/src/parts/packages.rs (17.2; 17.3 fills; change 17.5)
/// render.md 4.2's `packages`; hearth 22.2 adds `initramfs_file: Option<Digest>` (here, and in the records').
pub struct Packages { pub sources: Sources, pub request: Box<[Box<[u8]>]>, pub variant: Variant, pub preseeds: Box<[Preseed]>, pub initramfs: Option<Initramfs>, pub apt: Apt }
/// Mirrors hearth 07's `Sources`, names and timestamps as text.
pub struct Sources { pub suite: Box<[u8]>, pub architecture: Box<[u8]>, pub components: Box<[Component]>, pub archive: Box<[u8]>, pub updates: bool, pub security: Option<Box<[u8]>> }
/// In format.md 5.3's order.
pub enum Component { Main, Contrib, NonFree, NonFreeFirmware }
/// `Apt` for metal and a VM; `Container`: a stand-in until M5 (render.md 4.2).
pub enum Variant { Apt, Container }
pub struct Apt { pub install_recommends: bool }
/// A debconf answer; `owner` the question's first segment unless given.
pub struct Preseed { pub question: Box<[u8]>, pub owner: Box<[u8]>, pub kind: PreseedKind, pub value: Box<[u8]> }
pub enum PreseedKind { Select, Multiselect, String, Boolean }
/// `INCLUDE_MODULES`, sorted, and `MICROCODE`.
pub struct Initramfs { pub modules: Box<[Box<[u8]>]>, pub microcode: Microcode }
pub enum Microcode { None, Generic }
/// The key's four values (5.1), items in pre-order as the record encodes them.
pub struct Inputs { pub sources: Box<[Item]>, pub request: Box<[Item]>, pub variant: Box<[Item]>, pub apt: Box<[Item]> }
#[must_use] pub fn inputs(packages: &Packages) -> Inputs;
#[must_use] pub fn digest(packages: &Packages) -> Digest; // H("part:packages", the record)
/// `debian`'s checked root, read back; `None` for a root 13.5 did not make.
#[must_use] pub fn read(root: &Value) -> Option<(Sources, Apt)>;
/// The part, or an `Error` per entry at fault; `set`: where `debian.components` was set.
pub fn build(root: &Value, placement: &revision::Placement, expansion: &Expansion, set: Option<&Place>) -> Result<Packages, Box<[Error]>>;
/// 17.5: `Some` where the field differs; 45.2 adds `resolution`.
pub struct Change { pub request: Names, pub sources: Option<SourcesPair>, pub variant: Option<VariantPair>, pub preseeds: Option<PreseedsPair>, pub initramfs: Option<InitramfsPair>, pub apt: Option<AptPair> }
pub struct Names { pub added: Box<[Box<[u8]>]>, pub removed: Box<[Box<[u8]>]> }
// and one `<Type>Pair { before, after }` per field's type, no generic `Pair<T>` (ruling 155):
// `PreseedsPair` holds `Box<[Preseed]>`, `InitramfsPair` `Option<Initramfs>`
#[must_use] pub fn change(before: &Packages, after: &Packages) -> Change;
// crates/hearth-domain-render/src/parts/boot.rs (17.4; change 17.5)
/// render.md 4.3's `boot`.
pub struct Boot { pub params: Box<[Box<[u8]>]>, pub mutable: bool }
#[must_use] pub fn digest(boot: &Boot) -> Digest;
pub fn build(expansion: &Expansion, placement: &revision::Placement) -> Result<Boot, Box<[Error]>>;
pub struct Change { pub before: Boot, pub after: Boot } // and `change(&Boot, &Boot) -> Change` (17.5)
// crates/hearth-domain-render/src/kinds/{packages, preseeds, cmdline}.rs: one entry's check, each `entry`
pub fn entry(entry: &Concrete) -> Result<Component, (Schema, Position)>; // or `Preseed`, or `Parameter { order: u16, param }`
// errors.rs: `Schema` gains `Component { package: Box<[u8]>, component: Box<[u8]> }` (17.2; `also`: where
// components were set), `Format` gains `Preseed`, `Module`, `Param`; diff.rs: `PartChange` gains `Packages`, `Boot`;
// parts.rs: `PartRecord` gains them. The translation (translate/parts/{packages, boot}.rs), both ways in tests:
#[must_use] pub fn packages(packages: render::parts::packages::Packages) -> records::parts::packages::Packages;
#[must_use] pub fn boot(boot: render::parts::boot::Boot) -> records::parts::boot::Boot;
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-17\.[1-5]'` prints 5.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run --workspace` and `heavy cargo nextest run --workspace --profile fuzzy` pass.
3. `git -C ~/src/rust/hearth ls-tree --name-only origin/main crates/hearth-domain-render/src/parts/ | grep -c -E '/(packages|boot)\.rs'` prints 2.
4. `git -C ~/src/rust/hearth grep -c -E 'pub fn inputs\(|debian: _' origin/main -- crates/hearth-domain-render/src/parts/packages.rs crates/hearth-protocol-builder/src/translate/documents.rs` prints one `…packages.rs:1` line and no `documents.rs` line.
5. `git -C ~/src/rust/hearth grep -n -E '\bVec\b|String::|[^:]: *String\b|-> *String\b|<String\b|&String\b|^\s*_ =>' origin/main -- crates/hearth-domain-render/src crates/hearth-protocol-builder/src/translate ':!*/tests/*' ':!*/tests.rs'` prints nothing (code-organization.md, 6).
