# Session 13: schemas, validation and defaults

Declarations reach render, and settings meet their schemas: 13.1
reads `task.json`s; 13.2 translates and checks reads; 13.3 builds the validator
(12's `Composed` in, 14's `Value` out); 13.4 validates each target's settings,
builds the `settings` and `tasks` roots; 13.5 adds forms, references, `debian`.

## Read first

- `format.md`: 2.4, 3.2, 3.3, 4.1, 5.1, 5.3, 6.1 to 6.6, 7.6, 7.7.
- `render.md`: 1, 2.3, 3 to 3.3, 4.2, 7.1 to 7.3, 8.3, 8.4.
- `hearth/12-compose.md`: Choices, 12.1, 12.2, sketches.
- `hearth/04-declarations.md`, `35-schema-exports.md`: Choices, sketches.
- `hearth/14-evaluate.md`: Choices, `value.rs` and `context.rs` sketches.
- `hearth/11-thin-story.md`: Choices, "Where later sessions add", 11.3, 32.1.
- `hearth/40-client-diff-errors.md`: 40.2.
- skein `f981746`: `crates/skein-lib/src/{stack, list}.rs`.
- skein `programming-model.md` 3, 6.3, 7, 8; `testing-strategy.md` 2.1, 2.4, 3.

## What the design asks

- render.md 3.3, format.md 6.1, 3.3: a schema "well formed", defaults "have
  their types", "even where no target has it".
- render.md 3.3, format.md 6.3 to 6.5: per target, undeclared, types, enums,
  ranges, required, `target`, `file`, reads; defaults "filled at every depth".
- format.md 6.2, 2.4, 5.3, render.md 2.3, 7.3: forms, `debian`, a `file`'s
  64 MiB, "lists the declared ones".

## Choices this plan makes

- **A broken declaration is a broken task:** its errors print once, with its
  `task`, no `target`; its targets are left out; a task no target has fails
  nothing. Who checks 6.1 (render.md 3.3): config a `task.json`'s `default`
  beside `optional` and `min` above `max`; render a default's type, a
  `target` in one on each target (format.md 6.3).

## Rules that bind it

- **Meeting point:** each increment proves on the real loop; a variant
  added to `Decode` or `Schema` adds the records' and both `translate/errors.rs`
  arms at once; `translate/declarations.rs` is exhaustive, no `..`.
- **No recursion** (render.md 8.3): `for`s on `skein_lib::Stack`s of
  `Limits::depth`; tests take `limits::SHIPPED`.

## Increments

### 13.1 Every declaration read once a revision; config's faults printed
Scope: render.md 3.3's "each declaration" (every listed `task.json` in the first
`Read`, through 04's `declaration`); config's `Fault::Declaration` as render's
and the records' `decode`.
Needs: hearth 12 (the first `Read`, `Documents`, 12.2's error record,
`translate/errors.rs`), hearth 04 (`declaration`), hearth 35 (`Declared`'s
schema variants, `types`, `settings`).
Files: render `src/{revision, call, errors}.rs`, new
`src/revision/declaration.rs` (`Map`), `src/tests/{revision, errors}.rs`;
protocol `src/read.rs`, `src/translate/errors.rs`, `src/tests/{read,
translate}.rs`; records `src/errors.rs`, `src/form/{text, json}.rs`,
`src/tests/errors.rs`; `tests/render/{src/fixtures.rs, tests/errors.rs}`;
`tests/builder/{src/fixtures.rs, tests/real.rs}` (shared).
Shape: 12.1's first `Read` also asks each listed `tasks/<task>/task.json` in
path order, `Budget::take`n, indexed in `Documents`; its `Decoded(value)` goes
through `config::declaration::declaration(value, &limits.config)`: `Ok` answers
`Decoded::Declaration`, `Err` 12's `Refused` through `translate::errors::fault`,
whose `Fault::Declaration` arm gives `Decode::Declaration(declared(..))`
(replacing 12.2's, if 04 landed first). A declaration's fault is `decode`, config's first one error a
`task.json` (render.md 7.1, 7.2). Messages (the plan's, within 7.3's
forms), one an arm: `no such
key in a declaration` (`… in a files entry`), `not a string`, `optional beside a
default`; JSON
`"decode":{"declared":{"kind":"unknown","of":"declaration"},"kind":"declaration"}`.
Test shape: render unit `the_first_read_asks_every_listed_task_json_once`;
protocol `a_task_json_reaches_render_through_declaration`,
`a_declaration_config_refuses_answers_refused_with_its_fault`; translate
`each_declared_fault_translates_to_render_and_back`; records
`a_declaration_fault_prints_its_object_after_its_place`; world (eight seeds)
`a_broken_task_json_no_target_has_prints_once_and_fails_nothing`. Units under
20 ms, world under 0.5 s. `fixtures::MISKEYED`: `tasks/motd/task.json`
`{"setting": {}}`; 12.2's `thin_with` replaces a THIN file of that path.
Proof: process,
`a_task_json_out_of_shape_prints_its_fault_and_fails_its_targets`, 0.5 s, on
`thin_with(MISKEYED)`: the first message at `tasks/motd/task.json:1:2`, as
`motd: <message>`, exit 2.
Don't: translate a declaration's fields (13.2); read a `task.json` twice.
Size: at most 280 lines (a codec: within 2.5 times).
Removes: hearth 32.1's `task.json` in the second `Read`; 12's read of present
tasks' `task.json` alone; `Decoded::Declaration` from JSON alone.

### 13.2 Render's schemas, translated; reads checked against their tasks
Scope: `revision::declaration`, `translate/declarations.rs`;
`Decoded::Declaration(Declaration)`; each read's task listed and setting
declared (6.4, 6.5); the `schema` kind.
Needs: nothing beyond 13.1.
Files: render `src/revision/declaration.rs`, `src/{revision, vocabulary, call,
errors}.rs`, new `src/schema.rs`, new `src/tests/schema.rs`,
`src/tests/fixtures.rs`; protocol new `src/translate/declarations.rs`,
`src/{read, translate}.rs`, `src/translate/errors.rs`, `src/tests/{translate,
fixtures}.rs`; records as 13.1's; `tests/builder/{src/fixtures.rs,
tests/real.rs}` (`MISREAD`).
Shape: the translation moves each box, `Spec::default` through 12.1's
`translate::documents::value`, `Form::Object`'s specs by a `for`, naming
`requires: _`, `scripts: _`, `exports: _`, `bless: _`, `entries: _`, no `..`.
`schema::declarations` (13.3 grows it), in name order: a read of a task without
`task.json` is `Task` at the read's key, `optional` or `exports` or not (6.5); a
read setting its task lacks is `Unread` at it; a read of a refused task is not
checked; a fault makes its task unsound. Messages: `reads core.domian, which
core does not declare; core declares domain`, `reads mail, which tasks/ lacks`.
Test shape: `src/tests/schema.rs` (new, under 20 ms each), reference:
```rust
#[test]
fn a_read_of_a_setting_its_task_lacks_is_refused_at_it() {
    let core = fixtures::declaration(&[fixtures::spec(b"domain", Form::String)], &[]);
    let motd = fixtures::declaration(&[], &[fixtures::read(b"core", &[text(at(1, 34), b"domian")])]);
    let unread = Schema::Unread { task: name(b"core"), setting: name(b"domian"), declared: Box::new([name(b"domain")]) };
    let fault = Fault { schema: unread, document: 1, at: at(1, 34) };
    assert_eq!(&fixtures::check_declarations(&fixtures::declarations(&[(&b"core"[..], core), (&b"motd"[..], motd)]))[..], &[(name(b"motd"), fault)], "format.md, 6.5: the reader's, at its setting");
}
```
Then `a_read_of_a_task_tasks_lacks_is_refused_at_its_key`,
`an_optional_or_exporting_read_of_a_task_tasks_lacks_is_still_refused` (two
rows), `a_read_of_a_refused_task_is_not_checked`; translate
`each_form_translates_to_render_and_back` (13 rows),
`a_declaration_s_types_settings_and_reads_cross_unchanged` (04's `PARAMS` and
reads literals), `each_schema_fault_translates_and_back`; records
`each_schema_fault_prints_its_message` (a `const` of `Schema` variants and
lines; here `Task`, `Unread`; later increments add theirs).
Proof: process,
`a_read_of_a_setting_its_task_lacks_prints_and_fails_its_targets`, 0.5 s, on
`fixtures::MISREAD` (motd `{"reads": {"core": {"settings": ["domian"]}}}`, core
declaring `domain`): the first message at 1:34 of `tasks/motd/task.json`,
exit 2.
Don't: check a default (13.3), an expression's references or "templates used"
(16, 18); translate `entries`, `requires`, `scripts`, `bless` or `exports`.
Size: at most 260 lines.
Removes: `Decoded::Declaration` without its declaration.

### 13.3 The validator; defaults checked once a revision
Scope: 6.1, 6.2, 6.5: a value against its type, undeclared and required fields,
fills (6.3); each default checked once a revision.
Needs: hearth 14.1 (`evaluate::Value`, `Entry`, `ELEMENT`).
Files: render `src/schema.rs`, `src/{compose, call, errors, limits}.rs`,
`src/tests/{schema, fixtures}.rs`; protocol and records as 13.1's; world
`tests/render/{Cargo.toml, src/lib.rs}`, new `src/schemas.rs`, new
`tests/{schema, fuzzy_schema}.rs`; real as 13.1's.
Shape: a frame is `{ value, type or specs, cursors over pairs and specs, what it
made }`; an object's pairs and specs meet in key order: a pair without a spec is
`UndeclaredField` at its key; a spec without a pair is filled from
`TaskDefaults` (a `Composed`, walked as any value; so are list and map object
elements, 6.3), else `None` if optional, else `MissingField` at the object's
`{`; another kind is `Wrong`, not entered (one fault one error, render.md 7.1;
`max_length` bounds a string's bytes, a list's elements, a map's entries,
format.md 6.1). `schema::declarations` then scans
each sound declaration's defaults for `null` (`Null`), makes each a `Composed`
of `Layer::Defaults` (12's `Merge` over nothing) and walks it under
`Check::Default`. Messages: `not an int`, `"x" is not one of main, contrib`,
`below its min 1000`, `past its max_length 8`, `null in a default`, `undeclared
field "shel"; the object declares groups, password, shell, uid`, `missing
required field uid`.
Test shape: in `src/tests/schema.rs` (13.2's reference), with
`fixtures::users()` (6.6), `entry`, `composed` (document 2),
`validate(&Declaration, &Composed, &Check) -> Result<Value, Box<[Fault]>>` at
`SHIPPED`: `an_account_gets_the_shell_its_object_type_defaults` (`{"accounts":
{"free": {"uid": 1000}}}` fills `groups` `[]`, `shell` `/bin/bash`, `password`
`None`), `an_object_element_of_a_list_gets_its_defaults`,
`a_value_of_another_type_is_refused_at_it` (13 rows),
`an_enum_value_not_listed_is_refused_with_the_listed`,
`ints_outside_min_and_max_are_refused_at_them`,
`max_length_bounds_bytes_elements_and_entries`,
`an_undeclared_field_is_refused_and_its_siblings_still_checked`,
`a_null_in_a_default_is_refused_at_it`,
`a_walk_at_the_shipped_depth_fits_its_stack` (32 deep). World
(`tests/render/tests/schema.rs`, under 0.5 s), after 14.1's
`generated_expressions_evaluate_as_the_reference_evaluates_them`:
`generated_settings_validate_as_the_reference_validates_them`, 200 seeds from
`0x0013_0000`, `schemas::declaration(rng, SMALL)`, `schemas::settings(rng,
&Declaration)`, against `schemas::reference`; `tests/fuzzy_schema.rs`:
`generated_schemas_and_settings_validate_as_the_reference_does`, 10,000 rounds
at `WIDE`, kinds met checked by 14.1's `assert_fell`; its time is
`fuzzy_schema`'s nextest line under `--profile fuzzy`, aim 2 s.
Proof: process, `a_default_without_its_type_prints_and_fails_its_targets`,
0.5 s, on `fixtures::MISTYPED` (motd `{"settings": {"greeting": {"type": "int",
"default": "hi", "doc": "The greeting."}}}`): `not an int` at `tasks/motd/task.json:1:54`,
exit 2.
Don't: recurse; check `target` or `file` (13.5); validate a target's settings
(13.4).
Size: at most 300 lines.
Removes: nothing.

### 13.4 Each target's settings validated; the `settings` and `tasks` roots
Scope: 6.3's bottom layer; render.md 3.3's per-target check; reads (6.4); the
roots (7.7); format.md 3.3's "validated even where no target has it" (format.md 3.3).
Needs: nothing beyond 13.3.
Files: render `src/{schema, compose, call, errors, limits}.rs`,
`src/tests/{schema, compose}.rs`; protocol and records as 13.1's; `tests/render/{src/fixtures.rs,
tests/schema.rs}`; real as 13.1's.
Shape: `compose` starts each target's settings from `Defaults::layer` of each
present task (the bottom layer; a defaulted setting or field absent is filled
when validated, `null` restores it or removes a default map's entry:
format.md 6.3), then 12's layers; a target with an unsound
present task is left out, no error. 15.2 moves `Checked`'s `settings` and
`tasks` into `evaluate::context::Roots`. `schema::validate` walks each present task under
`Check::Target`: a setting without a spec is `Undeclared` at its key, a required
one absent `Missing` at the presence key; a read's task present gives
`tasks.<task>`, absent and optional `None`, else `Absent` at the read. Each made
value (a string its bytes, `evaluate::ELEMENT` an entry or element) is charged
to `Limits::validated` (64 MiB a call) and given back as the target's spec is
answered; past it the target fails with `Kind::Evaluation(FaultKind::Limit)`,
no position, its `target` (render.md 2.3, 7.2; 13.4 adds `Kind::Evaluation`,
the records' `Evaluation`, mirroring `FaultKind`, their translation and ten
messages, 15.2's list; 15.2 only uses them).
`schema::unclaimed`, once a revision: each layer's value of a task none of the
targets it composes has (`compose::present`; `envs/lab.json`'s wifi when only
prod targets have it; a base with no target) under `Check::Patch`, errors with
its `task`, no `target` (format.md 3.3). A missing setting is placed at the
presence key, a field at its object's `{`, a read at the read (render.md 7.1).
Messages (7.3): `undeclared setting "greting"; motd declares greeting`,
`motd declares no settings`, an echo quoted with `\"`, `\\`, `\n`, `\xNN`,
`missing required setting greeting`, `reads wifi, which this target lacks`.
Test shape: unit `the_defaults_lie_under_every_document_of_a_target` (compose),
`null_removes_a_default_map_s_entry_and_a_field_gets_its_default_back`,
`an_undeclared_setting_lists_the_declared_ones`,
`a_missing_setting_is_placed_at_the_task_s_key`,
`a_read_gives_the_other_task_s_setting_with_its_default`,
`an_optional_read_s_absent_task_is_none_and_a_required_one_refused`,
`validated_settings_past_their_budget_fail_the_target` (`validated: 64` beside
`SHIPPED`'s; the error is `Evaluation(Limit)`); records, rows for 7.3's own
`adress` line, `declares no settings`, an echo with `"`, LF and ESC, and
`Evaluation`'s ten messages; translate `each_evaluation_kind_translates_and_back`; world
`a_typo_in_a_default_for_a_task_no_target_has_is_refused`,
`an_environment_s_setting_of_a_task_only_other_targets_have_is_checked_alone`,
`one_target_s_schema_fault_spares_the_other_targets`. `fixtures::USERS`:
`tasks/users/task.json` as 6.6; `hosts/x1/x1.json` `{"motd": {}, "users":
{"accounts": {` LF `"ann": {"uid": "1000"},` LF `"bob": {"uid": 1001, "shel":
"/bin/sh"},` LF `"cy": {}}}}`. A template printing a default is 15.2's
`a_template_prints_the_default_its_setting_was_filled_with` (`GREET`).
Proof: process, `an_account_s_faults_print_at_their_lines_in_the_base`, 0.5 s,
on `fixtures::USERS`: 13.3's three messages at 2:16, 3:22 and 4:7 of
`hosts/x1/x1.json`, each `x1-prod: users: <message>`, exit 2.
Don't: evaluate; check owner ids (19); validate other targets' exported tasks
(20); merge `debian` (17).
Size: at most 240 lines.
Removes: 12's settings composed without their defaults.

### 13.5 Forms and references; `debian`'s fixed schema
Scope: 6.2 forms, a map's key (2.4), `target`, `file` (6.5) and its size
(render.md 2.3); 5.3's `debian`, for 17.
Needs: nothing beyond 13.4.
Files: render `src/{schema, errors}.rs`, new `src/schema/{forms, debian}.rs`,
new `src/tests/forms.rs`, `src/tests/fixtures.rs`; protocol and records as
13.1's; `tests/render/{src/schemas.rs, tests/schema.rs}`; real as 13.1's.
Shape: `forms::check` is a `for` over the bytes, under every `Check` (so a
default out of its form is refused); a `mode` stays as written, a `duration`
becomes its seconds (`Malformed` past i64); a `target` is found by bisection in
`Check::Target`'s targets, else `Target`; a `file` is in the call's listing
(byte order), a file or executable (format.md 6.2), else `File`, and a listed `size` past
`Limits::static_file` (render.md 2.3) is `Large` at the setting.
`schema::debian` walks a composed `debian` by 5.3's table: `suite`,
`architecture` (format.md 5.3), `snapshot { archive, security? }` timestamps, `components`
(keys 5.3's four, else `Enum` at the key; `{"main": true}` is the bottom
layer, so `main` is off only when written false or null: `Main`), `pockets { updates, security }` (true), `install_recommends`
(false); `security` absent with `pockets.security`, `Security` (one beside a pocket off
is dropped); `None` is `Missing { setting: debian }` at the base's document `{`
(format.md 5.3). Messages:
`malformed path`, `no target x9-prod in prod`, `no file lab-ca/lab.crt at this
revision`, `lab-ca/big.crt past 67108864 bytes`, `debian.components must keep
main on`.
Test shape: `src/tests/forms.rs` (new, under 20 ms each), reference:
```rust
const FORMS: &[(Format, &[&str])] = &[ // "+" accepted, "-" refused
    (Format::Path, &["+/etc/motd", "+/", "-", "-etc/motd", "-/a//b", "-/a/./b", "-/a/../b", "-/a/"]),
    (Format::Mode, &["+644", "+7777", "-64", "-0844", "-17777"]),
    (Format::Duration, &["+14d", "+106751991167300d", "-106751991167301d", "-5", "-05s", "-0s", "-5w"]),
    (Format::Secret, &["+users/free.passwd", "-users/../x", "-Users/x", "-a//b", "-."]),
];
#[test]
fn strings_follow_their_type_s_form() {
    for (format, rows) in FORMS { for (row, line) in rows.iter().enumerate() {
        let (sign, written) = line.split_at(1);
        assert_eq!(forms::check(*format, written.as_bytes()), sign == "+", "{format:?} row {row}");
    } }
}
```
Then `a_duration_is_its_seconds` (`14d` is `Int(1_209_600)` in
`Checked::settings`), `a_secret_name_is_at_most_255_bytes` (255 and 256),
`a_map_key_is_one_to_255_bytes`, `a_target_must_be_of_the_environment`,
`a_file_must_be_listed_as_a_file`,
`a_file_past_the_static_limit_is_refused_at_its_setting` (`static_file: 8`
beside `SHIPPED`'s), `a_default_failing_its_form_is_refused_once_a_revision` (a
`path` default `bin/sh`: `Malformed`),
`debian_fills_its_defaults_and_keeps_main_on`,
`an_absent_debian_is_missing_at_the_base_s_document`,
`debian_s_security_snapshot_is_required_with_its_pocket`,
`debian_s_timestamps_follow_snapshot_s_form`,
`debian_s_suite_and_architecture_follow_their_form`,
`an_undeclared_debian_field_is_refused_with_the_declared`,
`a_debian_field_of_another_type_is_refused`,
`an_unknown_component_is_refused_at_its_key` (`Enum`),
`components_with_main_off_are_refused` (rows `{"main": false}`,
`{"main": null}`: `Main`), `components_without_main_keep_it_on`
(`{"contrib": true}`, 3.7's x1.json); world
`a_default_target_is_checked_where_it_is_filled`; the world's generator and
reference gain the forms.
Proof: process, `a_missing_file_and_target_print_where_the_base_sets_them`,
0.5 s, on `fixtures::REFERENCES` (`net`: `file` setting `ca`, `target` setting
`peer`; `hosts/x1/x1.json` `{"motd": {}, "net": {"ca": "lab-ca/lab.crt", "peer":
"x9-prod"}}`): the `no file` message at 1:28 and the `no target` one at 1:54 of
`hosts/x1/x1.json`, each `x1-prod: net: <message>`, exit 2; unit, for
`schema::debian`: nothing composes it until 17, which calls it on the real loop.
Don't: merge `debian` or check a package's component (17); check `placement`
(03.3, 42.3).
Size: at most 240 lines.
Removes: nothing.

## Interface sketches

render owns these; the records mirror `Declared` to `Required`, `Map`, `Schema`,
`TypeName` and `Format` variant for variant. Types derive `Clone, PartialEq, Eq,
Hash, Debug`.

```rust
// hearth-domain-render src/revision/declaration.rs (Map 13.1, rest 13.2; 16 adds `entries`, `requires`; tests build one via `fixtures::declaration`)
pub struct Text { pub at: Position, pub text: Box<[u8]> } // config's `Text`, at its quote
pub struct Number { pub at: Position, pub value: i64 } // config's `Number`
pub struct Declaration { pub at: Position, pub types: Box<[Type]>, pub settings: Box<[Spec]>, pub reads: Box<[Read]> } // config's, at its `{`
pub struct Read { pub task: Text, pub at: Position, pub settings: Box<[Text]>, pub exports: bool, pub optional: bool } // config's `Read`
pub struct Type { pub at: Position, pub form: Form } // config's `Type`, `Form`, `Spec`; defaults as written
pub enum Form { Bool, Int, String, Path, Mode, Duration, Secret, Target, File, Enum(Box<[Text]>), List(u32), Map(u32), Object(Box<[Spec]>) }
pub struct Spec { pub name: Text, pub at: Position, pub ty: u32, pub default: Option<Value>, pub optional: bool, pub min: Option<Number>, pub max: Option<Number>, pub max_length: Option<Number> }
pub enum Map { Packages, Preseeds, Initramfs, Software, Files, Users, Groups, Presets, Tmpfiles, Data, Secrets, Identity, Cmdline, Live } // config's `entries::Map`
impl Documents { #[must_use] pub fn declaration(&self, task: &[u8]) -> Option<(u32, Option<&Declaration>)>; } // revision.rs: a task's index and declaration once decoded (13.1); `Decoded::Declaration` holds a `Declaration` (13.2)
pub fn compose(target: &Target, documents: &Documents, defaults: &schema::Defaults, merge: &mut Merge) -> Settings; // compose.rs (12's): `Layer` gains `Defaults` first (13.3); 13.4
#[must_use] pub fn present(target: &Target, documents: &Documents) -> Box<[Box<[u8]>]>; // a target's present tasks, name order
// errors.rs (12's): `Decode::Declaration(Declared)` (13.1), `Kind::Schema(Schema)` (13.2), `Kind::Evaluation(FaultKind)` (13.4; the records' `Kind::Evaluation(Evaluation)`, `Evaluation` mirroring `FaultKind`; 14.1's type)
pub enum Declared { Unknown { of: Object }, Later, Wrong { expected: Expected }, Missing { key: Required }, Name, Doc, Order, Optional, Bound, Above, Enum, Secret, Settings } // config's `Declared` (less `Undecoded`) and its parts
pub enum Object { Declaration, Read, Guard, Script, Bless, Entry(Map), Range, Spec, Type, Export }
pub enum Expected { Object, String, Strings, Bool, True, Int, Type, Field(Takes) }
pub enum Takes { Text, Int, Bool, Strings, Range }
pub enum Required { Type, Doc, Value, If, Message }
/// render.md 7.2's `schema`.
pub enum Schema {
    /// 13.2: a read's task or setting missing.
    Task { task: Box<[u8]> }, Unread { task: Box<[u8]>, setting: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    /// 13.3: types, enums, bounds, `null`, fields.
    Wrong { expected: TypeName }, Enum { value: Box<[u8]>, listed: Box<[Box<[u8]>]> },
    Below { min: i64 }, Above { max: i64 }, Length { max_length: i64 }, Null,
    UndeclaredField { field: Box<[u8]>, declared: Box<[Box<[u8]>]> }, MissingField { field: Box<[u8]> },
    /// 13.4: settings undeclared (7.3) or missing; a read's task absent.
    Undeclared { setting: Box<[u8]>, declared: Box<[Box<[u8]>]> }, Missing { setting: Box<[u8]> }, Absent { task: Box<[u8]> },
    /// 13.5: forms, `target`, `file`, `debian`.
    Malformed { format: Format }, Target { name: Box<[u8]>, environment: Box<[u8]> },
    File { path: Box<[u8]> }, Large { path: Box<[u8]>, max: u64 }, Main, Security,
}
pub enum TypeName { Bool, Int, String, Enum, List, Map, Object, Path, Mode, Duration, Secret, Target, File } // 6.2's types
pub enum Format { Path, Mode, Duration, Secret, File, Key, Timestamp, Debian } // 6.2's forms, keys, timestamps
// schema.rs (13.2 to 13.5); `Limits` gains `validated: 67_108_864` (13.4)
pub struct Fault { pub schema: Schema, pub document: u32, pub at: Position } // a walk's fault, by `Documents` index
/// What a walk checks beyond types: a default (required, fills, no `target`), a layer of a task no target has (none of these), a target's settings.
pub enum Check<'a> { Default, Patch, Target { environment: &'a [u8], targets: &'a [Box<[u8]>] } }
pub struct Against<'a> { pub declaration: &'a Declaration, pub defaults: &'a TaskDefaults, pub listing: &'a [Entry] } // what a walk reads beside the value
pub struct Revision<'a> { /* … */ } // what a revision's walks read
pub struct Validator { /* frames */ } // the walk: frames on a `Stack`
impl Validator {
    #[must_use] pub fn new(limits: &Limits) -> Validator;
    pub fn settings(&mut self, settings: &Composed, against: &Against<'_>, check: &Check<'_>, left: &mut u64, faults: &mut List<Fault>) -> Option<Value>; // charged to `left`; 20 calls it
}
pub struct TaskDefaults { /* private */ } // a sound task's defaults, `Composed`
pub struct Defaults { /* private */ } // `TaskDefaults` by task
impl Defaults { #[must_use] pub fn of(&self, task: &[u8]) -> Option<&TaskDefaults>; #[must_use] pub fn layer(&self, task: &[u8]) -> Option<&Composed>; }
pub fn declarations(documents: &Documents, validator: &mut Validator, merge: &mut Merge, faults: &mut List<(Box<[u8]>, Fault)>) -> Defaults; // once a revision: reads and defaults
pub struct Checked { pub name: Box<[u8]>, pub settings: Value, pub tasks: Value } // a task's roots (7.7), moved into 15's `context::Roots`; `None` for an absent optional setting or read
pub struct Validated { pub tasks: Box<[Checked]> } // a target's validated tasks
pub fn validate(settings: &Settings, revision: &Revision<'_>, check: &Check<'_>, validator: &mut Validator, left: &mut u64, faults: &mut List<(Box<[u8]>, Fault)>) -> Option<Validated>; // 13.4
pub fn unclaimed(targets: &[Target], revision: &Revision<'_>, validator: &mut Validator, merge: &mut Merge, faults: &mut List<(Box<[u8]>, Fault)>); // 13.4: tasks no discovered target has
pub fn debian(debian: Option<&Composed>, document: u32, faults: &mut List<Fault>) -> Option<Value>; // schema/debian.rs (13.5): `debian` filled, or its faults; 17 calls it
// hearth-protocol-builder translate/declarations.rs (13.2; 16 adds entries, requires; reversed in src/tests/translate.rs)
#[must_use] pub fn declaration(declaration: config::declaration::Declaration, limits: &Limits) -> render::revision::declaration::Declaration; // protocol `Limits` (`.config`; 15.1 adds `.template`); 16.1 makes it a `Result`
#[must_use] pub fn map(map: config::entries::Map) -> render::revision::declaration::Map;
#[must_use] pub fn declared(declared: config::Declared) -> render::errors::Declared; // translate/errors.rs (13.1), with `Schema`'s arms
```

## Questions to raise when reached

None.

## Done when

1. `git log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-13\.[1-5]'` prints 5.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run -p hearth-domain-render -p hearth-render-world -p hearth-builder-world -p hearth-protocol-builder -p hearth-client-protocol-records` passes,
   and `heavy cargo nextest run -p hearth-render-world --profile fuzzy -E 'binary(=fuzzy_schema)'` passes, its `Summary` naming 1 test.
3. `git -C ~/src/rust/hearth ls-tree --name-only origin/main crates/hearth-protocol-builder/src/translate/` lists `declarations.rs`,
   and `git -C ~/src/rust/hearth grep -n -E 'Schema\(Schema\),' origin/main -- crates/hearth-domain-render/src/errors.rs crates/hearth-client-protocol-records/src/errors.rs | wc -l` prints 2.
4. `git -C ~/src/rust/hearth grep -n -E ', \.\. ?\}|\(\.\.\)' origin/main -- crates/hearth-protocol-builder/src/translate | wc -l` prints 0 (a struct update such as `..SHIPPED` is no match).
