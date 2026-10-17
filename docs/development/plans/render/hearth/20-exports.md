# Session 20: exports, placements, a target's containers

What a target publishes to its environment, and what it reads of the others
(format.md, 8; render.md, 3.4, 4.4), with `target.containers` (format.md, 3.5).
20.1: `target.containers`; 20.2: the `exports` part, evaluated, checked,
printed; 20.3: `exports.<task>`, the environment's exports; 20.4: the part's
change in `hearth diff`. Each proves on the real loop.

## Read first

- hearth's `format.md`: 3.4, 3.5, 6.2, 6.4, 7.2, 7.7, 8 whole; `render.md`:
  2.3, 3.4, 4.4, 4.6, 6.1 to 6.3, 7.1, 7.2, 8.4.
- This plan: `hearth/16-entries.md` (Choices, 16.3, 16.4); `13-schema.md`
  (13.4, sketches); `42-listing-members-rules.md` (42.2, 42.3); `15-files.md`
  (`Roots`); `14-evaluate.md` (`Value`, `context`); `10-client.md` (Choices,
  "Who adds what", 10.2); `17-packages.md` (17.5); `25-diff.md` (25.1,
  sketches); `35-schema-exports.md` (35.2); `01-lib.md` (`Item`).
- skein at `f981746`: `crates/skein-lib/src/{stack, map, list}.rs`;
  `programming-model.md`: 3, 6.3, 8.

## What the design asks

- **Declared** (8.1): "a map from export name to a spec ... and a `value`,
  written as one `{{ expr }}`"; "Computed from settings only"; never `secret`.
- **Read** (8.2, 8.3, 6.4): `exports.vpn` is "a map from target name to that
  target's vpn exports, for every target of the environment that has the
  task, itself included"; "never cross an environment".
- **Resolved** (render.md 3.4): "An error in another target's exports fails
  this render too, naming both targets".
- **Part** (4.4, 4.6, 6.1, 6.2): "map task -> map name -> value", "an export's
  integer value enters as its decimal text", "empty when no task exports";
  "A change to exports alone builds nothing".
- **Containers** (3.5, 7.7): "`target.containers`, a map from target name to
  placement"; "The same environment only".

## Choices this plan makes

- **Placement's other M1 parts are built** (03.3, logs 30, 31; `on` 42.3;
  variant 17.2; `mutable` 17.4); a container's parameters and ranges are
  refused keys until M5 (format.md 2.2), its module, boot parameter and
  `hearth` are 21's (format.md 5.12).
- **`target.containers`** is always a map: the targets whose composed
  placement has `on` this target, in name order, each as
  `target.placement` (render.md 3.4), built by 14.1's placement function;
  empty where nothing is placed. A container whose placement cannot be
  composed fails its host with that fault (render.md 3.4).
- **Exports are a pure function** of a task's declaration, the target's
  validated settings of it and its `target` root (with `containers`)
  (format.md 8.1): a false guard, another task's fault or a file of the
  exporter leave them, and its readers render (render.md 3.4).
- **Computed once a revision for each (target, task)**, shared by the
  readers: `exports::Cache`, one pair a `resume` as 15.2's. No I/O (12.1).
- **The record** is 4.4's map, every integer at any depth its decimal text
  (4.6), the block in 4.1's order: always a part, the empty map when no task
  exports (hearth 17's `packages` alike). A broken declaration is a static
  fault.
- **An export's value** is checked as a setting's (format.md 8.1, 6.5) on
  each target that evaluates it: 13.5's forms, a `target` of the environment,
  a `file` of the revision; names are identifiers (2.4; 35.1); no `secret`
  at any depth (35.2); `debian`, `tasks`, `exports` refused.
- **Another target's fault** is the reader's error, kind `exported`
  (render.md 7.2): `target` the reader, the exporter in the kind, the class
  and place the cause's. Causes: a document the exporter composes broken, its
  settings failing validation, an export's expression failing, a refused
  `hosts/` document hiding which targets have the task (3.4).
- **The bound** is `Limits::exported` (64 MiB a call, a string its bytes, 32 an
  entry or element; render.md 2.3, 8.4); past it the reader fails with
  `Kind::Evaluation(FaultKind::Limit)`, no position (13.4), not `exported`.
- **JSON depth.** hearth 10.2 prints at its `const DEPTH: u32 = 16` in the
  records' `src/form/json.rs`; 20.2 sets that one to 64, and owns it. An
  export has its declared type, in a task.json at most 32 deep (render.md
  2.3): at most 29 lists (less root, `exports`, the entry), 32 with the
  record's three levels. A list an evaluation makes deeper (2.3 lets it 64)
  fails `conform` as `not a <type>`, so no export passes 64 (01's `DEPTH`,
  the digest's, also 64). The real loop prints 29, the unit test 32.
- **Not here:** probes `from`, backups' peer, id ranges (M5, M6);
  collisions, sorting and counting errors (hearth 22).

## Rules that bind it

- **The meeting point** (README, Rules): each increment proves on the real
  loop; a new render variant adds the records' and both translation arms
  at once, no `_`, no `..`.
- **No recursion** (render.md 8.3): values are walked on `Stack`s of
  `Limits::depth`; equality of two values is equality of their `items`.
- **Shared files:** `parts.rs`, `translate/{parts, errors, declarations}.rs`,
  `diff.rs`, the records' `records.rs`, `errors.rs`, `form/`, render's
  `lib.rs`, `errors.rs`, `limits.rs`, both fixtures modules.

## Increments

### 20.1 `target.containers`
Scope: format.md 3.5, 7.7; render.md 3.4's second bullet.
Needs: hearth 16, hearth 42.3.
Files: render `src/{compose, call}.rs`, `src/evaluate/context.rs`,
`src/tests/{compose, fixtures}.rs`; `tests/builder/` fixtures, `real.rs`;
the users of `context::target` and `templates::Target::new`: render
`src/call/{templates.rs:60, guards.rs:18}`, `src/call.rs:533`,
`src/tests/evaluate.rs:138`, `src/tests/fixtures/templates.rs:58`.
Shape: the sketches. `compose::containers` walks every discovered target of
the host's environment whose placement 42.3's `compose::placement` composes,
keeping those with `on` the host ("A host's containers are the targets of
its environment placed on it", format.md 3.5); `context::target` gains the
map, each placement as `target.placement` (absent fields no key). A target
of the host's environment whose placement is refused, its `on` the host,
fails the host with that error (`target` the host). A target of another
environment is no container of the host and fails no host: its own 42.3
error stands, so `real.rs`'s
`a_container_on_another_environment_s_target_prints_its_error` (:419) keeps
its lines. `Done::answer` (`call.rs:452`) is at 98 of clippy's 100 lines:
the lookup goes inside the call to `compose::placed` (`compose.rs:641`), or
into a function of its own.
Test shape: render unit, each under 20 ms:
`containers_are_the_targets_placed_on_a_host_in_name_order`,
`a_target_placed_on_another_host_is_not_listed`,
`a_target_nothing_is_placed_on_has_an_empty_containers_map`,
`a_containers_placement_map_leaves_absent_fields_out`,
`a_base_envs_placement_replaces_the_environments_for_containers`,
`a_container_with_a_refused_placement_fails_its_host`. Real, on new
`fixtures::CONTAINED`: `envs/{prod,lab}.json` `{}`; `hosts/rhi/rhi.json`
`{"inventory": {}}`, `rhi-prod.json` `{"placement": {"kind": "vm", "firmware":
"bios", "hardware": "serverhino"}}`, `rhi-lab.json` `{"placement": {"kind":
"vm", "firmware": "efi", "hardware": "incus"}}`; `hosts/chat/chat.json`
`{"inventory": {}}`, `chat-prod.json` and `hosts/mail/mail-prod.json`
`{"placement": {"kind": "container", "on": "rhi-prod"}}`, `mail-lab.json`
likewise `on` `rhi-lab`; `tasks/inventory/task.json` `{}`,
`tree/etc/containers.j2`: `{% for name, placement in target.containers | items
%}{{ name }}={{ placement.kind }}@{{ placement.on }};{% endfor +%}` LF.
```rust
// tests/builder/tests/real.rs
#[test]
fn a_hosts_template_lists_the_containers_placed_on_it() {
    let repository = Repository::make("contained", &fixtures::thin_with(fixtures::CONTAINED));
    let rendered = fixtures::render(&repository, &[b"chat-prod", b"rhi-lab", b"rhi-prod"], Form::Text);
    let expected = concat!(
        "  /etc/containers          file 0644 root:root 01ba4719c80b\n",
        "  /etc/containers          file 0644 root:root 9e40aaeae922\n",
        "  /etc/containers          file 0644 root:root 755c9769e418\n",
    );
    assert_eq!(fixtures::lines_starting(&rendered.stdout, &["  /etc/containers"]), expected, "3.5: chat none, lab mail, prod chat and mail");
    assert_eq!(rendered.exit, Exit::Code(0), "client/README.md, 3.3: done");
}
```
The digests are `sha256sum`'s (the first: LF alone).
Proof: process, bound 0.5 s, that test: real git and the evaluator put each
host's containers into a rendered file.
Don't: list a container of another environment (42.3 refuses it); check M5
parameters; give a host a list of its own (3.5).
Size: at most 150 lines.
Removes: 14.1's empty `containers` map.

### 20.2 The `exports` part: declared, evaluated, typed, printed
Scope: format.md 8.1; render.md 4.4, 4.6; `PartRecord::Exports`; the
`exports` translation; the JSON depth.
Needs: hearth 16, hearth 42.3.
Files: render new `src/{exports, parts/exports}.rs`, `src/{parts, call,
errors, limits, lib}.rs`, `src/revision/declaration.rs`,
`src/schema/references.rs`, new `src/tests/exports.rs`,
`src/tests/fixtures.rs`, and, as hearth 20.2 landed them (ruling 161),
new `src/exports/{conform, value}.rs` and `src/call/exports.rs`,
`src/call/{templates, validation}.rs`, `src/schema.rs`,
`src/schema/{debian, forms}.rs`; protocol new
`src/translate/parts/exports.rs`,
`src/translate/{declarations, parts, errors}.rs`, `src/tests/translate.rs`;
records new `src/parts/exports.rs`, `src/{parts, records, errors}.rs`,
`src/errors/schema.rs` (ruling 161), `src/form/{text, json}.rs`, `src/tests/{exports, errors, fixtures}.rs`;
`tests/builder/` fixtures, `real.rs`; `tests/render/` (`hearth-render-world`);
each `Declaration` literal, which gains `exports` as 16.1's gained
`requires` (`git grep -n 'requires: ' -- crates tests`); render
`src/diff.rs`'s arms (README, "A new part").
Shape: the sketches. `translate::declarations::export` replaces 13's
`exports: _`, compiling `value` with 36's `interpolation::compile`.
`exports::check`, once a revision, in `schema::declarations`: a value that
is not exactly one `Output` is `Export { Form }`; a `Name` operation of
`tasks`, `exports` or `debian` is `Export { Root }`, at the string; a
`settings.<name>` the task does not declare is `Schema::Undeclared` (13.4's),
there; a `secret` in an export's type is refused at decode (35.2). Per
target, after 13.4: `exports::evaluate` runs each present task's exports on
a `Roots` of `settings` and `target` alone, in name order: the value must
have the export's type; an optional undefined is absent, a required one is
`evaluation`. `conform` checks kind, enum members, 13.5's forms (`target` in
`Check::Target`'s targets, `file` in the listing), list and map elements,
object fields. `parts::exports::items` is the one
pre-order walk, digest and equality alike. Text under `exports      part <digest>`: one line a
`task.name`, the value as compact JSON, in a column the longest `task.name`
plus three wide (render.md 6.3, 7.3). The records' `json.rs` `DEPTH` becomes 64.
Test shape: render unit, under 20 ms: `an_integer_export_enters_the_record_as_
its_decimal_text` (below), `a_value_with_text_around_its_tag_is_refused`,
`an_export_names_settings_and_target_alone` (table: `tasks.core.domain`,
`exports.vpn`, `debian.suite` refused; `target.name` passes),
`an_export_naming_an_undeclared_setting_is_refused`, `a_negative_integer_enters_as_
its_decimal_text` (`-7`, `i64::MIN`), `items_walk_a_map_in_key_byte_order`
(input unsorted), `an_optional_export_undefined_is_absent_and_a_required_one_
errs`,
`an_exports_value_must_have_its_declared_type` (table, 8 rows; a list made
64 deep for a `list of int` is one),
`an_export_of_type_target_or_file_is_checked_on_each_target`,
`no_export_makes_an_empty_part`; translate `each_export_and_its_value_translates_
to_the_records_and_back`, `a_value_32_deep_translates_without_recursion`.
```rust
// crates/hearth-domain-render/src/tests/exports.rs
#[test]
fn an_integer_export_enters_the_record_as_its_decimal_text() {
    let exports = fixtures::exports(&[(b"vpn", &[(b"hub", Value::Bool(true)), (b"port", Value::Int(51820))])]);
    let items = [Item::Map(1), bytes(b"vpn"), Item::Map(2), bytes(b"hub"), Item::True, bytes(b"port"), bytes(b"51820")];
    assert_eq!(&*exports::items(&exports), &items[..], "render.md, 4.6: keys in byte order, 51820 as text");
}
```
Records, under 20 ms: `exports_print_as_task_dot_name_and_a_json_value`,
`json_nests_exports_under_their_task_and_name`, and
```rust
// crates/hearth-client-protocol-records/src/tests/exports.rs
#[test]
fn an_export_nested_32_lists_deep_prints_as_json() {
    let spec = fixtures::with_exports(*fixtures::x1_prod(), b"vpn", b"nest", fixtures::lists_around(32, b"x"));
    let printed = form::print(Form::JsonLines, &Record::Spec(Box::new(spec)), None, &limits::SHIPPED).expect("fits");
    let want: [&[u8]; 4] = [b"\"nest\":", &[b'['; 32], b"\"x\"", &[b']'; 32]];
    assert!(fixtures::has_run(&printed, &want), "32 lists and the part's three levels pass 16");
}
```
Real, on `fixtures::EXPORTS` (`tasks/vpn/task.json`, two lines, the second
starting with a space, `address`'s `value` at 2:53:
```json
{"settings": {"address": {"type": "string"}, "endpoint": {"type": "string", "optional": true}},
 "exports": {"address": {"type": "string", "value": "{{ settings.address }}"}, "endpoint": {"type": "string", "optional": true, "value": "{{ settings.endpoint }}"}}}
```
`x1.json` `{"motd": {}, "vpn": {"address": "10.0.0.2"}}`, `x1-prod.json` THIN's
placement and `"vpn": {"endpoint": "198.51.100.7"}`):
`hearth_render_prints_the_exports_a_target_publishes` (`lines_starting`,
`  vpn.`, text):
```text
  vpn.address    "10.0.0.2"
  vpn.endpoint   "198.51.100.7"
```
Beside it `an_optional_export_with_no_setting_prints_no_line`,
`json_prints_the_exports_part_nested_under_its_task` (the part line holds
`"exports":{"vpn":{"address":"10.0.0.2","endpoint":"198.51.100.7"}}`),
`an_export_nested_29_lists_deep_prints_as_json` (`fixtures::deep(29)`: type
29 nested lists of `int`, value `{{ [[...[1]...]] }}`; the line holds
`"nest":`, 29 `[`, `1`, 29 `]`); four change only `address`'s `value`:
`an_export_of_the_wrong_type_prints_at_its_value` (`{{ 7 }}`:
`tasks/vpn/task.json:2:53: x1-prod: vpn: not a string`, exit 2),
`a_value_with_text_around_its_tag_prints_once_without_a_target` (`v {{
settings.address }}`: `tasks/vpn/task.json:2:53: vpn: an export's value must be
one {{ }} tag`), `an_export_naming_an_undeclared_setting_prints_at_its_name`
(`{{ settings.adress }}`, 2:53: `vpn: undeclared setting "adress"; vpn declares
address, endpoint`) and `an_export_reading_tasks_is_refused` (`{{
tasks.core.domain }}`, 2:53: `an export reads settings and target alone, not
tasks`).
Proof: process, bound 0.5 s, the first real test: real git, config, 06's
compiler, 14's evaluator and 10's printer give a target's exports; unit
for the 32-deep print (no export nests past 29).
Don't: read another target (20.3); leave the part out when nothing is
exported; sort or bound errors (hearth 22).
Size: at most 300 lines (a codec: within 2.5 times).
Removes: 13's `exports: _`; 15.2's empty `exports` root stays until 20.3.

### 20.3 `exports.<task>`: the environment's exports, once a revision
Scope: format.md 8.2, 8.3, 6.4 (`exports: true`); render.md 3.4's first and
last bullets; `Roots::exports`; `Kind::Exported`; `Limits::exported`.
Needs: 20.2.
Files: render `src/exports.rs`, `src/{call, errors, limits, schema}.rs`,
`src/evaluate/context.rs`, `src/tests/{exports, fixtures}.rs`; world
`tests/render/{src/fixtures.rs, tests/exports.rs}`; protocol
`src/translate/errors.rs`, `src/tests/translate.rs`; records `src/errors.rs`,
`src/form/{text, json}.rs`, `src/tests/errors.rs`; `tests/builder/` fixtures, `real.rs`.
Shape: the sketches. For a task of the reader with `reads` `exports: true`
of task U, `exports.U` is a map from each target of the reader's environment
(`Target::environment`) having U present to its `Exported` (absent optionals
`None`, as 14's `Entry`); the reader included when it has U, `{}` when none
has. A pair (X, U) not in the `Cache`: `compose::compose` X, validate U's
settings alone (13.4's `Validator::settings`, `Check::Target` of X's
environment, charged to `exported`), evaluate as 20.2; a fault is cached as a
`Cause`. A reader meeting one fails (no spec, no entries) with `exported`,
`target` the reader, `task` its task, `at` the cause's own place (3.4: an
error in another target's exports fails this render). The causes are the
Choices'; a refused `hosts/` document gives its own fault, the `exporter` its
base. Nothing else on the exporter fails its readers.
13.4's `Absent` no longer applies to a read that only lists `exports`
(format.md 6.4: `exports.<task>` is then an empty map). Messages: `exports.vpn of ion-prod: not a string`; JSON
`"exported":{"cause":{"schema":…},"exporter":"ion-prod"}`.
Test shape: render unit, each under 20 ms: `a_reader_gets_every_target_of_
its_environment_that_has_the_task`, `the_reader_itself_is_included_when_it_
has_the_task`, `a_task_nobody_has_gives_an_empty_exports_map`, `an_exporters_
settings_fault_fails_the_reader_naming_the_exporter`,
`an_exporter_with_a_broken_document_fails_the_reader`,
`a_refused_hosts_document_fails_the_reader_with_its_fault`,
`an_exporters_other_faults_leave_its_readers_rendering`, `a_pair_is_evaluated_
once_for_all_its_readers`, `exports_held_past_their_limit_fail_the_reader_
with_an_evaluation_limit` (`exported: 64`, beside `SHIPPED`); translate `each_exported_cause_
translates_to_the_records_and_back`; records `an_exported_fault_names_the_
exporter_in_text_and_json`; world (`tests/render/tests/exports.rs`, 200
seeds from `0x0020_0000`, under 0.5 s) `generated_fleets_give_each_reader_
its_environments_exporters` (a reference recursing over the documents).
Real, on `fixtures::VPN`: `fleet.json` `{}`, `envs/{prod,lab}.json` `{}`;
`tasks/vpn/task.json` `{"settings": {"address": {"type": "string"}, "endpoint":
{"type": "string", "optional": true}, "mtu": {"type": "int", "default": 1420}},
"reads": {"vpn": {"exports": true}}, "exports": {"address": {"type": "string",
"value": "{{ settings.address }}"}, "endpoint": {"type": "string", "optional":
true, "value": "{{ settings.endpoint }}"}}}`; `tree/etc/vpn/peers.j2` `{% for
name, peer in exports.vpn | items %}{% if name != target.name %}{{ name }}={{
peer.address }}{% if peer.endpoint is defined %}@{{ peer.endpoint }}{% endif
%};{% endif %}{% endfor +%}` LF and `tree/etc/vpn/mtu.j2` `{{ settings.mtu }}`
LF; `hosts/ion/ion.json` `{"vpn": {"address": "10.0.0.1"}}`, `ion-prod.json`
`{"placement": {"kind": "vm", "firmware": "bios", "hardware": "ionos"}, "vpn":
{"endpoint": "194.164.162.157"}}`, `ion-lab.json` `{"placement": {"kind": "vm",
"firmware": "efi", "hardware": "incus"}, "vpn": {"address": "10.1.0.1"}}`;
`hosts/carbon/carbon.json` `{"vpn": {"address": "10.0.0.2"}}`,
`carbon-prod.json` THIN's placement, `carbon-lab.json` `{"placement": {"kind":
"vm", "firmware": "efi", "hardware": "incus"}}`.
`a_reader_sees_the_exports_of_its_own_environment_alone` (renders `carbon-lab`,
`carbon-prod`; `lines_starting`, `  /etc/vpn/peers`; exit 0):
```text
  /etc/vpn/peers           file 0644 root:root afd16377136a
  /etc/vpn/peers           file 0644 root:root 0ed834effddb
```
(`sha256sum` of `ion-lab=10.1.0.1;` and `ion-prod=10.0.0.1@194.164.162.157;`,
each plus LF.) Then `an_exporters_fault_prints_for_it_and_for_its_reader`
(`VPN_BAD`: `ion-prod.json` as above but `"vpn": {"endpoint": 7}`, the `7`
at 1:92; stdout holds `hosts/ion/ion-prod.json:1:92: ion-prod: vpn: not a string` and
`hosts/ion/ion-prod.json:1:92: carbon-prod: vpn: exports.vpn of ion-prod:
not a string`, the lab specs untouched, exit 2).
Proof: process, bound 0.5 s, the first test: peers cross targets through
real git and the evaluator, and stop at the environment.
Don't: read another environment; evaluate a pair twice; validate the
exporter's other tasks.
Size: at most 300 lines.
Removes: 15.2's empty `exports` root; 13.4's `Absent` for an exports-only read.

### 20.4 The exports part's change, through `hearth diff`
Scope: render.md 6.1 (`exports: task.name: before, after`), 6.2, 6.3's
ion-prod and carbon-prod blocks; the `layer` and `PartChange` arms.
Needs: 20.2, 20.3 (`exports.vpn`); hearth 25.1 (`diff.rs`, `PartChange`,
`Layer`; 40.1).
Files: render `src/parts/exports.rs`, `src/diff.rs`, `src/tests/diff.rs`;
protocol `src/translate/{changes, parts/exports}.rs`, `src/tests/translate.rs`;
records `src/diff.rs`, `src/parts/exports.rs`, `src/form/{text, json}.rs`,
`src/tests/diff.rs`; `tests/builder/` fixtures, `real.rs`.
Shape: the sketches. `change` pairs the two sides' (task, name) in order
(a side without the part is empty): `Some(before)` and `None` is removed,
the reverse added, differing `items` changed. `layer`: exports `Other`
(rebuild `nothing`, 6.2). Text under the target's line as 6.3 shows it
(`exports  ~ vpn.endpoint   "a" → "b"`), `+` and `-` for added and
removed; JSON `"exports":[{"after":...,"before":...,"name":"endpoint",
"task":"vpn"}]`, an absent side left out.
Test shape: unit, under 20 ms: `a_changed_export_lists_its_before_and_after`,
`an_added_or_removed_export_has_one_side`, `an_unchanged_export_is_left_out`,
`exports_alone_rebuild_nothing_and_with_config_from_config`; translate
`each_exports_change_translates_and_back`; records 6.3's line and the JSON.
Real, on `VPN`, `repository.next` setting `ion-prod`'s endpoint to
`194.164.162.158` (`VPN_MOVED`), or its `mtu` to 1280 (`VPN_MTU`):
`a_changed_export_rebuilds_its_readers_and_builds_nothing_where_it_is_made`
(`lines_starting`, `carbon-prod`, `ion-prod`, `  exports `; exit 1):
```text
carbon-prod                                         rebuild from config
ion-prod                                            nothing to build
  exports  ~ vpn.endpoint   "194.164.162.157" → "194.164.162.158"
```
The lab targets do not appear (6.3). Then `a_setting_no_export_reads_changes_
its_target_alone` (`VPN_MTU`: ion-prod `rebuild from config`, carbon-prod
absent) and `json_gives_an_exports_change_with_one_side_absent` (the endpoint
removed).
Proof: process, bound 0.5 s, the first test: two commits through real git
show 6.3's coupling: the exporter builds nothing, its reader rebuilds, the
other environment is untouched.
Don't: ask hunks of exports; compare values with derived equality.
Size: at most 180 lines.
Removes: nothing.

## Interface sketches

render owns these; the records mirror them; types derive as 17's do, but
`Cache` derives `Debug` alone (skein's `Map` has no `Clone` or `PartialEq`).

```rust
// crates/hearth-domain-render/src/compose.rs (20.1); evaluate/context.rs: `target` gains `containers`
pub struct Container { pub name: Box<[u8]>, pub placement: parts::meta::Placement }
/// `host`'s containers in name order; a container whose placement is refused fails the host with its error.
pub fn containers(host: &Target, targets: &[Target], documents: &Documents) -> Result<Box<[Container]>, Box<Error>>; // as `placed`, clippy's result_large_err
#[must_use] pub fn target(meta: &Meta, containers: &[Container]) -> Value;
// crates/hearth-domain-render/src/exports.rs (20.2; Cache, root 20.3); declaration.rs: `Export`
pub struct Export { pub name: Text, pub at: Position, pub ty: u32, pub optional: bool, pub value: Interpolated }
/// One task's exports on one target, in name order; an absent optional is `None`.
pub struct Exported { pub task: Box<[u8]>, pub values: Box<[Entry]> }
pub fn check(declaration: &Declaration, faults: &mut List<Fault>);
/// The task's exports on a target of `settings` and `target`; `left` is the target's `expanded`.
pub fn evaluate(against: &Against<'_>, check: &Check<'_>, settings: &Value, target: &Value, limits: &Limits, left: &mut u64) -> Result<Exported, Fault>;
/// 13.5's forms on an export's value, `Check::Target`'s `target` and the listing's `file` (format.md 8.1).
pub fn conform(value: &Value, against: &Against<'_>, check: &Check<'_>, ty: u32, limits: &Limits) -> Result<(), Schema>;
/// Each (target, task) once a revision, a fault kept as its cause.
pub struct Cache { /* Map<Box<[u8]>, Result<Exported, Cause>> */ }
/// The `exports` root of `reader`'s task: a map from target to exports, per read with `exports: true`.
pub fn root(reader: &Target, declaration: &Declaration, revision: &Revision<'_>, cache: &mut Cache, left: &mut u64) -> Result<Value, Exporting>;
/// Why a reader has no root: another target's fault, or the bound (`Kind::Evaluation(FaultKind::Limit)`).
pub enum Exporting { Exported(errors::Exported), TooLarge }
// crates/hearth-domain-render/src/parts/exports.rs (20.2; change 20.4)
/// render.md 4.4's `exports`: tasks in name order.
pub struct Exports { pub tasks: Box<[Exported]> }
/// The record in pre-order, ints as decimal text: digest, equality and tests.
#[must_use] pub fn items(exports: &Exports) -> Box<[Item]>;
#[must_use] pub fn digest(exports: &Exports) -> Digest; // H("part:exports", the record)
/// The part: the empty map when no present task exports.
#[must_use] pub fn build(exported: Box<[Exported]>) -> Exports;
pub struct Change { pub entries: Box<[EntryChange]> } // by (task, name)
pub struct EntryChange { pub task: Box<[u8]>, pub name: Box<[u8]>, pub before: Option<Value>, pub after: Option<Value> }
#[must_use] pub fn change(before: Option<&Exports>, after: Option<&Exports>) -> Change;
// errors.rs (20.2, 20.3): `Schema` gains `Export { fault: ExportFault }`, `Kind` gains `Exported`,
// limits.rs `Limits` gains `exported: u64`, `SHIPPED` 67_108_864 (no `Limit` variant: 13.4's `Evaluation(Limit)`)
pub enum ExportFault { Form, Root { root: Root } } // 16.4's `Root` gains `Debian`
/// render.md 3.4: another target's fault, naming it.
pub struct Exported { pub exporter: Box<[u8]>, pub cause: Cause }
/// What kept the exports from being computed (render.md 3.4), as 12's, 42's, 13's and 14's kinds.
pub enum Cause { Decode(Decode), Repository(Repository), Composition(Composition), Schema(Schema), Evaluation(FaultKind) }
// diff.rs: `PartChange` gains `Exports(parts::exports::Change)`; `layer` gives `Other`.
// translate/parts/exports.rs: `exports(render::parts::exports::Exports, &Limits) -> records::parts::exports::Exports`
// crates/hearth-client-protocol-records/src/parts/exports.rs (20.2; change in src/diff.rs 20.4)
pub struct Exports { pub tasks: Box<[TaskExports]> }
pub struct TaskExports { pub task: Box<[u8]>, pub values: Box<[Named]> }
pub struct Named { pub name: Box<[u8]>, pub value: Value }
pub enum Value { Bool(bool), Int(i64), String(Box<[u8]>), List(Box<[Value]>), Map(Box<[(Box<[u8]>, Value)]>) }
// src/diff.rs (20.4): `PartChange::Exports(ExportsChange { entries })`, entries as render's `EntryChange`
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-20\.[1-4]'` prints 4.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run --workspace` and `heavy cargo nextest run --workspace --profile fuzzy` pass.
3. `git -C ~/src/rust/hearth ls-tree --name-only origin/main crates/hearth-domain-render/src/ crates/hearth-domain-render/src/parts/ | grep -c -x -E 'crates/hearth-domain-render/src/(parts/)?exports\.rs'` prints 2.
4. `git -C ~/src/rust/hearth grep -n -E 'exports: _' origin/main -- crates/hearth-protocol-builder/src/translate/declarations.rs | wc -l` prints 0.
5. `git -C ~/src/rust/hearth grep -n -E 'const DEPTH: u32 = 64;' origin/main -- crates/hearth-client-protocol-records/src/form/json.rs | wc -l` prints 1.
6. `git -C ~/src/rust/hearth grep -n -E '\bVec\b|String::|[^:]: *String\b|-> *String\b|<String\b|&String\b|^\s*_ =>' origin/main -- crates/hearth-domain-render/src crates/hearth-protocol-builder/src/translate ':!*/tests/*' ':!*/tests.rs'` prints nothing (code-organization.md, 6).
