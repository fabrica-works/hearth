# Session 16: declarations to concrete entries: `for_each`, `if`, `requires`

A task's `task.json` becomes, per target, the concrete entries the kind
sessions turn into parts (render.md, 3.5; format.md, 5.2, 5.9). 16.1:
`requires` and its guards; 16.2: `if`, `for_each` and unique keys; 16.3:
string and typed fields, the kinds' accessors; 16.4: references checked
(3.3; 6.4). Each proves on `hearth render`, by the fault it prints.

## Read first

- `format.md`: 5.1, 5.2, 5.9, 6.4, 7.4, 7.6, 7.7; `render.md`: 2.3, 3.1, 3.3,
  3.5, 7.1 to 7.3, 8.3, 8.4.
- This plan: hearth 04 (Choices, 04.1, 04.2), 36 (36.2), 13 (Choices, 13.2,
  13.4), 14 (Choices), 15 (Choices, 15.1, 15.2), 11, 12 (`Error`), 40 (40.2).
- skein at `841f487` (hearth's pin): `skein-lib`'s `stack.rs` (`with_capacity`:22, `push`:50),
  `list.rs` (`push`:55, `into_boxed`:104), `map.rs` (`worst_case`:41,
  `contains_key`:61, `get`:70, `insert`:90), `decimal.rs` (`of`:18,
  `as_bytes`:36); `programming-model.md`: 3, 6.3, 8, 10.3.

## What the design asks

- **Guards** (5.9): "after validation and before any entry". **Entries**
  (5.2): "Keys are unique after expansion". **Strings** (5.2): "a string or an
  integer". **References** (6.4). **Bounds** (2.3): 65,536.

## Choices this plan makes

- **Which kinds.** None concrete into a part: every map's entries are expanded
  and typed generically (`Name::takes`); 17 to 20 read `Expansion::of(map)`.
- **Compile at translation:** a syntax error is `Decoded::Refused`,
  `Decode::Syntax` (13.1); `declaration` is fallible and takes the `Limits`.
- **Placed at the string** (render.md, 7.1): its opening quote, a template's
  fault at its line and column; a guard at its key.
- **`Parts` run here, not by 15's `run`** (it writes a boolean `true`, 14.2).
- **Order.** Per target, after 13.4: each task's guards (all evaluated, each
  false one printed), then, if none failed, its entries, then 15.2's template
  runs (render.md, 7.1: a false guard stops its target's entries and files).
- **Bounds.** `Limits::{expansion, expanded}` (64 MiB a target): `evaluation`,
  `Limit`, at the entry (render.md, 2.3, 7.2). `expansion` counts entries a
  `task.json` expands to for a target, `if`-false ones not.
- **Messages and `also`** are the Shapes' words, one an arm (render.md, 7.3);
  `also` prints as `path:line:col`, the later task in name order (7.1).
- **Not here:** scripts (18; `order` is 04.1's), exports' values (20), a
  field's presence, enum, form or path (kinds'), cross-task collisions (22).

## Rules that bind it

- **The meeting point** (README, Rules): every increment proves on the real
  loop; a variant added to render's `errors` kinds adds the records' and both
  arms of `translate/errors.rs`; `translate/declarations.rs` stays exhaustive,
  no wildcard arm, no `..` (hearth 13's `requires: _`, `entries: _` go).
- **No recursion, the shipped limits** (render.md 8.3; code-organization.md,
  4): every walk a `for`; no `Vec`, `String`, closure or `_` arm.
- **Shared files:** the error kinds, `translate/`, both fixtures modules.

## Increments

### 16.1 Guards: `requires` compiled, evaluated, printed
Scope: format.md 5.9; render.md 3.5's first sentence; `Kind::Guard` (40.2).
Needs: nothing beyond the session's start (hearth 13, 15).
Files: render new `src/entries.rs`, `src/revision/declaration.rs`,
`src/{call, errors, lib}.rs`, new `src/tests/entries.rs`,
`src/tests/fixtures.rs`; the `Declaration` literals that gain `requires` and
`entries`: render `src/schema/debian.rs:186`, `src/tests/fixtures/schemas.rs:32`,
`tests/render/src/schemas.rs:30`; protocol `src/translate/{declarations, errors}.rs`,
`src/read.rs`, `src/tests/{read, fixtures}.rs`, `src/tests/translate/declarations.rs`
(the reverse, `schema_back`:70);
`tests/render/src/{fixtures, parent}.rs`;
`tests/builder/{src/fixtures.rs, tests/real.rs}`.
Shape: the sketches. `declaration` compiles each guard's `if` (`Condition`),
keeps `message` as written, but compiles it too and refuses one that is not
`Plain`: it holds no `{{`, `{%` or `{#` (format.md, 5.2); a compile error is
`Refusal { at: the Text, fault: Decode::Syntax }`, which `read.rs` answers as
13.1's `Refused`. `entries::guards(declaration, context, limits)` evaluates each guard
in name order: false (a bare undefined is false, 7.6) is `Kind::Guard` at the
guard's key (`guard.name`'s `at`); an evaluation fault is `Kind::Evaluation` at the condition. A
target with a schema fault evaluates no guard (13.4 first); one with a fault
gets no spec, no entries and no template run.
Test shape: render unit (`src/tests/entries.rs`, under 20 ms, on
`Roots::x1_prod()` at `SHIPPED`; fixtures gain `guard(name, at, Condition,
message)`, `requires([Guard; N])`, `port_is(value, at)`):
```rust
// crates/hearth-domain-render/src/tests/entries.rs
#[test]
fn a_true_guard_passes_and_a_false_one_errs_at_its_key_with_its_message() {
    let roots = Roots::x1_prod(); // port is 2222
    let declaration = fixtures::requires([
        fixtures::guard(b"ssh", at(3, 5), fixtures::port_is(22, at(3, 14)), b"ssh must listen on 22"),
        fixtures::guard(b"web", at(4, 5), fixtures::port_is(2222, at(4, 14)), b"web needs 2222"),
    ]);
    let guard = Guard { name: Box::from(&b"ssh"[..]), message: Box::from(&b"ssh must listen on 22"[..]) };
    let fault = Fault { kind: Kind::Guard(guard), at: at(3, 5), also: None };
    assert_eq!(&entries::guards(&declaration, &roots.context(), &limits::SHIPPED)[..], &[fault], "format.md, 5.9: the false one only");
}
```
Then `every_false_guard_of_every_present_task_errs`,
`a_guard_reads_settings_tasks_and_target`,
`a_guard_that_cannot_be_evaluated_errs_at_its_condition`
(`settings.wifi == 1`),
`a_guard_over_an_undefined_optional_setting_is_false_not_an_error`,
`a_target_with_a_schema_fault_and_a_false_guard_prints_the_schema_fault_only`;
protocol `a_guards_condition_reaches_render_compiled_and_its_message_as_written`,
`a_condition_that_does_not_compile_refuses_its_declaration_at_the_string`,
`a_message_holding_a_tag_or_a_comment_is_refused_at_its_string` (`{{ x }}`,
`{% if %}`, `{# #}`);
translate `a_declarations_requires_cross_to_render_and_back`.
Proof: process, bound 0.5 s, on new `fixtures::GUARDED` (`tasks/motd/task.json`
`{"settings": {"greeting": {"type": "string", "default": "hello", "doc": "The
greeting."}},` LF `"requires": {` LF `    "greeting": {"if":
"settings.greeting != 'hello'", "message": "set a greeting of your own"}}}`):
`a_false_guard_prints_its_message_at_its_key_and_leaves_its_target_out` expects
stdout exactly `tasks/motd/task.json:3:5: x1-prod: motd: requires greeting:
set a greeting of your own` LF and `Exit::Code(2)` (client/README.md, 3.3).
Beside it `a_guard_the_settings_satisfy_leaves_the_thin_spec_as_it_was`
(x1.json `{"motd": {"greeting": "hi"}}`: `THIN_TEXT`). Real git, config, 05's
compiler, 14's evaluator and 40.2's printer.
Don't: expand an entry (16.2); interpolate `message`; run a guard per element.
Size: at most 220 lines.
Removes: hearth 13's `requires: _` in the translation.

### 16.2 `if` and `for_each`: entries expanded, keys interpolated and unique
Scope: format.md 5.2's `if`, `for_each`, keys; render.md 3.5; `entries::expand`
for keys alone; `Limits::{expansion, expanded}`; `Kind::Collision`.
Needs: 16.1; hearth 36.2 (`interpolation::compile`).
Files: render `src/entries.rs`, `src/revision/declaration.rs`,
`src/{call, errors, limits}.rs`, `src/tests/{entries, fixtures}.rs`; protocol
`src/translate/{declarations, errors}.rs`, `src/tests/translate/{declarations,
errors}.rs`; records `src/errors.rs`, `src/tests/errors.rs`; both
fixtures modules; `tests/builder/tests/real.rs`.
Shape: the sketches. The entry maps translate whole but `fields: _` (16.3).
`expand` walks each map's entries in key order. Without `for_each`, `if` is
evaluated once, with no `key`, `value` (`Unknown`, format.md, 7.7); undefined is
false (the entry is absent, no error). With it, a context or made list (each
element in turn, `key` the index) or a context map (its present entries in key
order) is walked; undefined is `Undefined`, anything else `Type`, at the
`for_each`. Per element: bind `key`, `value` (14's `Bindings`), `if`
(`truthy`), the key's `interpolate`; a key already made in its map (format.md
5.2: "within their declaration"; 21.4 moves main's index across a task's maps
to the map's own) is `Collision::Key` at the entry, `also` the first's; an
`if`-false element makes no key and counts nothing; past `expansion` entries,
counted across a task's maps, `Limit` at the `for_each` (exactly `expansion`
passes). `Concrete::element` is the `for_each` index;
`entries::element` rebinds `key`, `value` from it, for a kind that runs a
template per element (hearth 18). Message `the key "ops" is made twice
(path:line:col)`; JSON `"collision":{"key":"ops","kind":"key"}`.
Test shape: render unit, on `x1_prod()`'s `tags`, `peers`:
`a_for_each_entry_stands_for_one_entry_per_list_element_from_zero`
(`{"/etc/{{ key }}-{{ value }}": {"for_each": "settings.tags"}}`: `/etc/0-sudo`,
`/etc/1-wheel`),
`a_for_each_over_a_map_binds_key_and_value_in_key_order_skipping_absent`,
`an_if_false_for_an_element_leaves_that_element_out`,
`an_entry_if_over_an_undefined_optional_setting_leaves_the_entry_out`,
`key_and_value_in_a_plain_entrys_field_or_if_are_unknown`,
`a_for_each_over_undefined_or_a_scalar_fails_at_its_expression`,
`two_elements_making_one_key_are_a_collision_naming_the_first`,
`two_entries_making_one_key_collide_but_an_if_false_element_makes_none`
(`a{{x}}` and `{{y}}`, one `if` false: no collision),
`expanding_past_the_entry_limit_fails_at_the_for_each` (`expansion: 3`, beside
`SHIPPED`), `exactly_the_entry_limit_passes`,
`the_entry_count_runs_across_a_tasks_maps_and_skips_if_false_elements`,
`an_elements_index_rebinds_its_key_and_value`; translate
`each_entry_and_condition_translates_to_render_and_back`; records
`a_collision_prints_its_key_and_both_places`.
Proof: process, bound 0.5 s, on new `fixtures::TWICE` (`tasks/motd/task.json`
`{"settings": {"teams": {"type": {"list": "string"}, "default": ["ops", "dev",
"ops"], "doc": "Teams."}},` LF `"groups": {` LF `    "{{ value }}":
{"for_each": "settings.teams"}}}`):
`a_for_each_making_a_key_twice_prints_a_collision_at_its_entry` expects
`tasks/motd/task.json:3:5: x1-prod: motd: the key "ops" is made twice
(tasks/motd/task.json:3:5)` and exit 2;
`distinct_for_each_keys_render_with_no_error` (teams `["ops", "dev"]` in
x1.json) exits 0 with no `twice`. Real git, both translations.
Don't: check a key's form (kinds'); evaluate a field (16.3); compare tasks (22).
Size: at most 260 lines.
Removes: hearth 13's `entries: _` in the translation.

### 16.3 Fields: interpolated strings, typed fields, absent optional values
Scope: format.md 5.2's strings and "A field of another type"; `Name`, `Literal`
translated; `Concrete`'s fields and accessors; `Limits::expanded`; the worst
case; `kinds::required`.
Needs: nothing beyond 16.2.
Files: render `src/entries.rs`, new `src/kinds.rs`,
`src/revision/declaration.rs`, `src/{errors, limits}.rs`,
`src/tests/{entries, fixtures}.rs`; protocol `src/translate/declarations.rs`,
`src/tests/translate/declarations.rs`; world `tests/render/`: `Cargo.toml`
(`hearth-protocol-config`), new `src/declarations.rs`, `src/lib.rs` (`pub mod
declarations;`), `src/{reference, fixtures}.rs` (`fixtures::limits(rng)`, `:84`,
varies `made` and `nesting` only: it gains `expansion` and `expanded`), new `tests/{entries, fuzzy_entries}.rs`,
`tests/memory.rs`; `tests/builder/tests/real.rs`.
Shape: the sketches. Every field translates (`Text`, `Typed` and `Strings`
elements through `interpolation::compile`; a `template` field is
`Literal::Plain`, as written and never interpolated, so that every template
is used exactly: format.md, 5.5, 2.1; it makes a `Fielded::Text` charged
as is). `interpolate`: `Plain` as
written; `Parts` in order, each `Output` written by a function of its own
(`evaluate::output`, `operators.rs:35`, writes a boolean as `true`, which 5.2 refuses), an `Output` a string or an integer (`Decimal`), a
boolean `Type`, a list or map `Output`, undefined `Undefined` (format.md,
5.2). A `Text` field of exactly one `Output` that is undefined is absent (a key:
`Undefined`); a `Typed` field must be exactly one `Output` (else
`Schema::Wrong { expected }`, `expected` the `TypeName` of `name.takes()`: `Takes`,
render `errors.rs:269`, maps `Text` to `String`, `Int` to `Int`, `Bool` to `Bool`,
`Strings` to `List`, `Range` to `Object`) of that type (`Int`, `Bool`, a list of
strings, a map of integers `base`, `count`), undefined absent. A `Range`'s
members and a `Strings` element are never absent; `key`, `value` outside a
`for_each` entry are `Unknown` here too. Each string charges `left`.
`worst_case`: the evaluator, bindings, `expanded` and
`Map::worst_case(expansion)`. `kinds::required` is `MissingField` at the entry.
Test shape: render unit, under 20 ms:
`a_key_and_a_field_interpolate_text_and_integers_in_order`,
`a_boolean_in_a_string_fails_with_type_wherever_it_stands` (a table: a key, a
field with text, a lone tag, a list element),
`a_list_or_map_in_a_string_fails_with_output`,
`a_field_written_as_one_tag_is_absent_when_undefined_and_a_key_is_not`,
`a_typed_field_must_have_its_fields_type` (a table: `uid`, `directory`,
`groups`, `subordinate`; a plain `"1000"` is `Wrong`),
`a_list_fields_elements_interpolate_one_by_one`,
`key_and_value_in_a_plain_entrys_string_field_are_unknown`,
`a_required_field_the_entry_lacks_is_a_missing_field_at_the_entry`,
`expanded_text_past_the_targets_budget_fails_at_the_field` (`expanded: 64`,
beside `SHIPPED`); translate
`each_name_and_literal_translates_to_render_and_back` (37 names, 7 literals, `Plain` among them),
`a_template_field_with_a_tag_is_kept_as_written` (`{{ key }}.j2` stays text).
World (`tests/render/tests/entries.rs`, as 14.1's):
`generated_declarations_expand_as_the_reference_expands_them` (200 seeds from
`0x0016_0000`; `declarations::tree` makes `task.json` text of every `Takes`,
`declarations::expand` decodes, translates and expands it, `reference::expand`
recurses), under 0.5 s; `fuzzy_entries.rs`
`generated_declarations_and_contexts_expand_as_the_reference_does` (5,000
rounds at `fixtures::limits(rng)`; its time is `fuzzy_entries`'s nextest line
under `--profile fuzzy`, aim 2 s); `memory.rs` (43.3's `Meter`):
`expanding_at_the_limits_holds_no_more_than_the_worst_case` (`expansion: 4096`,
beside `SHIPPED`), 1 s.
Proof: process, bound 0.5 s, on new `fixtures::UNSET`: `tasks/motd/task.json`
`{"settings": {"origin": {"type": "string", "optional": true, "doc": "Where
from."}},` LF `"groups": {` LF `    "ops": {"from": "{{ settings.origin }}"},`
LF `    "dev": {"from": "x{{ settings.origin }}"}}}`:
`an_undefined_value_beside_text_fails_while_a_lone_tag_leaves_the_field_absent`
expects exactly `tasks/motd/task.json:4:21: x1-prod: motd: an undefined value
used` and exit 2: `ops` raised nothing, `dev` did. Beside it
`a_boolean_in_a_string_field_prints_an_evaluation_error_at_the_field`.
Don't: validate a field's presence, enum, form or path (the kinds'); call 15's
`run`; add a part.
Size: at most 300 lines.
Removes: hearth 16.2's `fields: _` (`Concrete` has no consumer until 17 to 20).

### 16.4 References checked against `settings` and `reads`
Scope: render.md 3.3's first bullet and format.md 6.4's last, in declarations
(once a revision) and in every template's expressions (`tree/*.j2`, hearth
15.1's `Template`, when read); `Schema::{Unlisted, Dynamic}`.
Needs: nothing beyond 16.3.
Files: render new `src/schema/references.rs`, `src/schema.rs`
(`declarations` calls it), `src/{call, errors, lib}.rs`,
`src/tests/{references, fixtures}.rs`; protocol `src/translate/errors.rs`,
`src/tests/translate/errors.rs`, `src/tests/fixtures.rs` (`schema_faults`:460, a
`[Schema; 19]` at 76c4653: one more per variant); records `src/errors.rs`,
`src/errors/schema.rs` (the `Schema` printer: `text`:28 and `json`:128 are at
clippy's 100-line limit, so each new arm is a function of its own),
`src/tests/errors.rs`, `src/tests/errors/validation.rs`, `src/tests/fixtures.rs`
(`SchemaInput`:147, `schema_fault`:168); `tests/render/src/schemas.rs` (its
exhaustive match over `Schema`); `tests/builder/{src/fixtures.rs,
tests/real.rs}`.
Shape: `references::check(&Expression, &Declaration)` reads the postfix
operations once: `settings` then a `Key` (or `String`, `Item`) must be a
declared setting (`Schema::Undeclared`, 13.4's words); `tasks` then two literal
names must be a read of that task listing that setting, `exports` then one a
read with `exports: true` (`Unlisted`); `tasks` then one name straight into
`is defined` or `is not defined` (`Op::NotDefined`) passes for an optional read of that task, else `Unlisted`
(`setting: None`); `tasks` or `exports` otherwise, or bare, is `Dynamic`;
`settings[x]` and `settings | tojson` pass, checked at evaluation, and so
does `exports.<task>` used whole (format.md, 6.4). `Missing` is
ignored: a typo under `is defined` is still a typo. `references::declaration`
walks the guards', conditions' and interpolations' expressions, each at its
`Text`'s position, as 13's `Fault { document, at }`, and makes the task
unsound. `references::template` checks every instruction's expression of a
template (`Output`, `Branch`, `Enter`'s iterable, `Set`), run or not; it runs
when 15.2 holds the template (the asked targets' tasks only: render.md, 3.3): an
`Error` at the operation's line and column, `task` set, no `target`. Scripts
are 18.4's, which calls `template`. Messages (render.md, 7.3's
`exports.mail is not in mail's reads`): `Unlisted` with a setting prints
`tasks.<task>.<setting> is not in <reader>'s reads`; with none, root `Exports`,
`exports.<task> is not in <reader>'s reads`; with none, root `Tasks`,
`tasks.<task> is not an optional read of <reader>`; `Dynamic` prints `tasks
takes only literal names` (or `exports …`, format.md, 6.4). JSON as the other
`Schema` arms, the fields by name.
Test shape: `src/tests/references.rs`, under 20 ms, on new
`fixtures::declared_for_references()` (built with `fixtures::declaration`,
render `src/tests/fixtures.rs:568`), with settings `greeting`, `port` and reads `core`
(`domain`), `mail` (exports), `wifi` (optional); new `fixtures::expression([..])`:
```rust
// crates/hearth-domain-render/src/tests/references.rs
#[test]
fn a_settings_reference_must_be_a_declared_setting() {
    let declaration = fixtures::declared_for_references();
    let typo = fixtures::expression([name(0, b"settings", Missing::Error), key(9, b"greting", Missing::Error)]);
    let declared = Box::new([Box::from(&b"greeting"[..]), Box::from(&b"port"[..])]);
    let expected = Schema::Undeclared { setting: Box::from(&b"greting"[..]), declared };
    assert_eq!(references::check(&typo, &declaration), Some((expected, 9)), "format.md, 6.4: at the key's offset");
}
```
Then `tasks_and_exports_references_must_be_in_reads` (a table:
`tasks.core.domain` passes, `tasks.core.port`, `tasks.wifi`, `exports.core`
do not; `exports.mail | tojson` passes; `tasks.core | tojson` and
`tasks.core[x]` are `Dynamic`), `tasks_and_exports_take_only_literal_names`,
`a_tasks_name_under_is_defined_passes_for_an_optional_read_only` (`tasks.wifi
is defined` and `tasks.wifi is not defined` pass; `tasks.core is defined`, `tasks.nope is defined` are
`Unlisted`), `a_reference_under_is_defined_is_checked_too`,
`a_dynamic_settings_key_is_not_checked`,
`a_typo_in_an_untaken_branch_of_a_template_is_found_at_its_offset`,
`every_instructions_expression_of_a_template_is_checked` (`Output`, `Branch`,
`Enter`, `Set`); records `an_unlisted_read_prints_the_reader_and_the_reference`.
Proof: process, bound 0.5 s, on new `fixtures::TYPO` (`tasks/motd/task.json`
`{"settings": {"greeting": {"type": "string", "default": "hello", "doc": "The
greeting."}},` LF `"requires": {` LF `    "known": {"if": "settings.greting
!= ''", "message": "set one"}}}`):
`a_typo_in_a_settings_reference_prints_the_declared_names` expects
`tasks/motd/task.json:3:21: motd: undeclared setting "greting"; motd declares
greeting` and exit 2. Beside it, on `fixtures::TYPO_BRANCH` (no `requires`;
`tasks/motd/tree/etc/issue.j2` `{% if false %}{{ settings.greting }}{% endif
%}` LF), `a_typo_in_a_branch_no_run_takes_prints_at_its_line` expects
`tasks/motd/tree/etc/issue.j2:1:27: motd: undeclared setting "greting"; motd
declares greeting`. Real git, 05's compiler and the printer.
Don't: check exports' values (hearth 20) or scripts (18.4); evaluate; check
`target` or `debian`.
Size: at most 180 lines.
Removes: hearth 13's "Not here" for expressions' references.

## Interface sketches

render owns these; the records mirror the errors variant for variant.

```rust
// crates/hearth-domain-render/src/limits.rs: `Limits` gains `expansion: u32` (16.2), `expanded: u64` (16.3); `SHIPPED`: 65_536, 67_108_864
// crates/hearth-domain-render/src/revision/declaration.rs: `Declaration` gains `requires`, `entries`
/// An expression of a declaration compiled, at its string's opening quote; and 5.9's guard, its `if` compiled.
pub struct Condition { pub at: Position, pub expression: Expression }
pub struct Guard { pub name: Text, pub at: Position, pub when: Condition, pub message: Text }
/// 5.2's string: mirrors hearth 36's `Interpolated`, the text kept for `Plain`.
pub enum Interpolated { Plain { at: Position, text: Box<[u8]> }, Parts { at: Position, parts: Box<[Instruction]> } }
/// Mirror config's `Entries`, `Entry`, `Field`, `Literal` (`declaration::Entry` is not the listing's).
pub struct Entries { pub map: Map, pub at: Position, pub entries: Box<[Entry]> }
pub struct Entry { pub key: Interpolated, pub at: Position, pub when: Option<Condition>, pub for_each: Option<Condition>, pub fields: Box<[Field]> }
pub struct Field { pub name: Name, pub at: Position, pub value: Literal }
pub enum Literal { Text(Interpolated), Plain { at: Position, text: Box<[u8]> } /* a `template`, as written: hearth 04's `Literal::Plain` */, Typed(Interpolated), Int(Number), Bool { at: Position, value: bool }, Strings { at: Position, each: Box<[Interpolated]> }, Range { at: Position, fields: Box<[Field]> } }
/// Mirrors config's `Name`, 04.2's 37 field names.
pub enum Name {
    Action, Age, Algorithm, Argument, Base, Class, Component, Copy, Count, Directory, From, Gecos, Gid, Group, Groups,
    Home, Kind, Link, Mode, Order, Owner, Param, Password, Path, Paths, PublicPath, Sha256, Shell, Strip, Subordinate,
    Template, Type, Uid, Unit, Unpack, Url, Value,
}
impl Name { #[must_use] pub const fn takes(self) -> Takes /* 04.2's table */; #[must_use] pub const fn key(self) -> &'static [u8] /* its JSON key */; }

// crates/hearth-domain-render/src/entries.rs (guards 16.1; expand 16.2; fields, accessors, worst_case 16.3)
/// Why a task has no concrete entries: the kind, where, and the other place.
pub struct Fault { pub kind: Kind, pub at: Position, pub also: Option<Position> }
/// A task's false guards, and those that did not evaluate, in name order.
/// `context` is `evaluate::Context`, as `evaluate::template::run` takes it; a test passes `&roots.context()`.
#[must_use] pub fn guards(declaration: &Declaration, context: &Context<'_>, limits: &Limits) -> Box<[Fault]>;
/// A task's entries expanded, charged to `left` (the target's `expanded`); the first fault.
pub fn expand(declaration: &Declaration, context: &Context<'_>, limits: &Limits, left: &mut u64) -> Result<Expanded, Fault>;
/// The `key` and `value` of the `index`th element of `entry`'s `for_each`.
pub fn element<'c>(entry: &Entry, index: u32, context: &Context<'c>, limits: &Limits) -> Result<(Operand<'c>, Operand<'c>), Fault>;
/// The most `expand` holds at `limits`; `None` past a u64.
#[must_use] pub fn worst_case(limits: &Limits) -> Option<u64>;
/// A target's concrete declarations: its present tasks' (name order), a task's maps' (`Group`: key order).
pub struct Expansion { pub tasks: Box<[Expanded]> }
pub struct Expanded { pub task: Box<[u8]>, pub groups: Box<[Group]> }
pub struct Group { pub map: Map, pub entries: Box<[Concrete]> }
/// A concrete entry: its key; the declaration's entry; the `for_each` index; the fields it kept.
pub struct Concrete { pub key: Box<[u8]>, pub at: Position, pub element: Option<u32>, pub fields: Box<[Made]> }
pub struct Made { pub name: Name, pub at: Position, pub value: Fielded }
/// A field's value by its `Takes`: text (integers in decimal), an integer, a boolean, strings, a range.
pub enum Fielded { Text(Box<[u8]>), Int(i64), Bool(bool), Strings(Box<[Box<[u8]>]>), Range { base: i64, count: i64 } }
/// An entry with its task, for a kind session: `of` gives a map's entries, task order then key order.
pub struct Located<'a> { pub task: &'a [u8], pub entry: &'a Concrete }
impl Expansion { #[must_use] pub fn of(&self, map: Map) -> Box<[Located<'_>]>; }
/// A field by name, `None` when absent (its type is `name.takes()`'s).
impl Concrete { #[must_use] pub fn text(&self, name: Name) -> Option<&[u8]>; #[must_use] pub fn int(&self, name: Name) -> Option<i64>;
    #[must_use] pub fn bool(&self, name: Name) -> Option<bool>; #[must_use] pub fn range(&self, name: Name) -> Option<(i64, i64)>;
    #[must_use] pub fn strings(&self, name: Name) -> Option<&[Box<[u8]>]>; }
// crates/hearth-domain-render/src/kinds.rs (16.3)
/// A field the kind needs: `Schema::MissingField` with `name.key()`, the entry's `at` at fault.
pub fn required(entry: &Concrete, name: Name) -> Result<(), Schema>;

// crates/hearth-domain-render/src/schema/references.rs (16.4)
/// The first reference `expression` makes that `declaration` does not declare, and its operation's `at`.
#[must_use] pub fn check(expression: &Expression, declaration: &Declaration) -> Option<(Schema, u32)>;
/// Every expression of `declaration`, each at its `Text`: faults for `document` (13.3's `Fault`).
pub fn declaration(declaration: &Declaration, document: u32, faults: &mut List<Fault>);
/// The first fault in any instruction's expression of `template` (15.1's), taken or not.
#[must_use] pub fn template(template: &Template, declaration: &Declaration) -> Option<(Schema, u32)>;

// crates/hearth-domain-render/src/errors.rs: `Kind` gains `Guard(Guard)` (16.1), `Collision(Collision)` (16.2)
/// format.md 5.9's false guard, as the records' 40.2; and 7.2's `collision` (hearth 22 adds variants).
pub struct Guard { pub name: Box<[u8]>, pub message: Box<[u8]> }
pub enum Collision { Key { key: Box<[u8]> } }
// `Schema` (13) gains, 16.4: `Unlisted { reader: Box<[u8]>, root: Root, task: Box<[u8]>, setting: Option<Box<[u8]>> }`, `Dynamic { root: Root }`
/// The roots that take literal names only (6.4).
pub enum Root { Tasks, Exports }

// crates/hearth-protocol-builder/src/translate/declarations.rs (13.2's; 16.1 to 16.3); reverses in src/tests/translate/{declarations, errors}.rs
/// A string that does not compile: where (its `Text`), and why.
pub struct Refusal { pub at: render::revision::Position, pub fault: render::errors::Decode }
pub fn declaration(declaration: config::declaration::Declaration, limits: &Limits) -> Result<render::revision::declaration::Declaration, Refusal>;
// and, each taking `&template::limits::Limits`: `guard`, `entries`, `entry`, `field`, `literal`, `condition`, `interpolated`
#[must_use] pub fn name(name: config::entries::Name) -> render::revision::declaration::Name;
```

## Questions to raise when reached

None.

## Done when

1. `git log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-16\.[1-4]'` prints 4.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run -p hearth-domain-render -p
   hearth-protocol-builder -p hearth-client-protocol-records -p hearth-render-world -p
   hearth-builder-world` passes, and `heavy cargo nextest run -p hearth-render-world --profile fuzzy -E
   'binary(=fuzzy_entries)'` passes, its `Summary` naming 1 test.
3. `git -C ~/src/rust/hearth ls-tree --name-only origin/main crates/hearth-domain-render/src/ | grep -c
   -x -E 'crates/hearth-domain-render/src/(entries|kinds)\.rs'` prints 2.
4. `git -C ~/src/rust/hearth grep -o -h -E 'expansion: 65_536|expanded: 67_108_864' origin/main --
   crates/hearth-domain-render/src/limits.rs | wc -l` prints 2.
5. `git -C ~/src/rust/hearth grep -n -i minijinja origin/main -- crates/hearth-domain-render tests/render/src` prints nothing.
6. `git -C ~/src/rust/hearth grep -n -E ', \.\. ?\}|\(\.\.\)' origin/main -- crates/hearth-protocol-builder/src/translate | wc -l` prints 0 (a struct update such as `..SHIPPED` is no match).
