# Session 35: `hearth-protocol-config`: the schema, exports, the declarations' sweep

hearth 04's `declaration::declaration` gains the two keys it refuses as
`Declared::Undecoded`: `settings`, the schema of a task's settings, the 13
types of format.md, 6.2, laid out in an arena `Declaration::types` (35.1),
and `exports`, each a spec, a type and a `value` as `Text` (35.2), with the
world's generated declarations and their sweep. The machine never reads an
expression, as in 04; `Limits` gains `settings`. Each increment proves in
the machine world `tests/config`; nothing calls `declaration` until hearth
13 (`types`, `settings`) and hearth 20 (`exports`) route a task's
`task.json` through the thin story's protocol layer.

## Read first

- hearth's `format.md`: 2.4, 5.1; 6.1 to 6.6; 8.1, 8.4's first bullet,
  8.6, 8.7. `render.md`: 2.3, 3.1, 3.3, 7.1 to 7.3, 8.1 to 8.3.
- This plan: `hearth/04-declarations.md` (Choices, Rules, 04.1, 04.2,
  sketches: its names and fixtures are taken as written);
  `hearth/03-json.md` (its world); `code-organization.md`, 3, 4, 6;
  `decisions.md`, 9, 10; `findings/skein-testing.md`, 3 and 7.
- skein's `crates/skein-lib/src/stack.rs` (`with_capacity` :20, `push`
  :48) and `list.rs` (`with_capacity` :21, `push` :55, `into_boxed` :104);
  `programming-model.md`: 3, 6.3, 7, 8, 10; `testing-strategy.md`: 2.1,
  2.4, 3, 8 (by number only).

## What the design asks

- **The schema** (6.1, 6.2): `spec = { "type": type, "doc": string,
  "default"?: value, "optional"?: true, "min"?: int, "max"?: int,
  "max_length"?: int }`; 13 types; "`doc` is required: one line, at most
  200 bytes"; "a default that does not have its type, `default` with
  `optional`, or `min` above `max` is an error in the task". Limits (2.3):
  "settings in a task | 512".
- **Exports** (8.1): "a spec like a setting's (`type`, `doc`, `optional`)
  and a `value`, written as one `{{ expr }}`"; "never of type `secret`".
- **Text with positions** (render.md, 8.1): as 04; an error's `at` is
  `{ path, line, column }` (7.1).

## Choices this plan makes

- **Types are an arena,** `Declaration::types`, each type before its
  elements (pre-order), built by one `for` over a `skein_lib::Stack` of
  `Limits::depth` frames (render.md, 8.3) and walked the same way by
  render. Exports' types share it.
- **Defaults stay 03's `Value`,** checked against their types by render's
  validation (hearth 13: render.md, 3.3). The machine checks the rest of
  6.1's bullet: `default` with `optional`, `min` above `max`.
- **`optional` is `true` or absent** in a spec and an export, as 6.1's
  grammar writes it (`"optional"?: true`).
- **Docs are checked and dropped,** as in 04.
- **Limits:** 03's `Limits` gains `settings`, 512 (35.1). "A `task.json`
  string that is an expression" (4 KiB) is hearth 05's `compile`'s, and
  "entries a declaration expands to" hearth 16's: neither is repeated.
- **The world's examples** `boot`, `hearth`, `locale`, `users` (35.1) and
  `vpn`, `backup-peer` (35.2) join 04's, with 04's other 17: the 23.

## Rules that bind it

- **No recursion; the shipped limits** (programming-model.md, 3 and 8;
  code-organization.md, 4): the type walk is a `for` on a `Stack` of
  `Limits::depth`; no `Vec`, `String` or closure in `src/`; tests take
  `limits::SHIPPED`, or name another value with the reason.
- **One owner each** (code-organization.md, 3): hearth 03 owns `Limits`,
  and 04 `Declaration` and `Declared`; this session adds `Limits::settings`
  and their fields and variants (sketched) and changes nothing else.
  hearth 13 and 20 mirror this session's types field for field.
- **The meeting point:** as 04's: an increment that adds a `Fault` variant
  adds its arm in `crates/hearth-protocol-builder/src/translate/`, in its
  report.

## Increments

### 35.1 The schema: settings' specs and the 13 types

Scope: `settings` (6.1, 6.2): specs, types, `doc`, `default`,
`optional`, bounds; setting and field names (2.4); "settings in a task"
(render.md, 2.3).
Needs: hearth 04 (the session's start): `declaration`, `Text`, `Number`,
`Declared`, `fixtures::{at, text, declaration}`, `examples::task`,
`world::declare`.
Files: `src/{schema,declaration,error,limits,lib}.rs`,
`src/tests/{schema,declaration}.rs`; the users of `Limits`, found by `git
grep -n -e 'Limits {' -- crates tests`, run in the worktree and its
matches listed in the report (in 03's draft: `src/limits.rs`,
`tests/config/src/world.rs`'s `world::limits`,
`tests/config/tests/memory.rs`), each given `settings` and nothing else;
`tests/config/examples/tasks/{locale,boot,hearth,users}.json`,
`tests/config/tests/declarations.rs`.
Shape: `Limits::settings` (512); `Declaration::{types, settings}`. At most
`Limits::settings` settings (`Settings` at the next key), each an
identifier (`Name`) and a spec. Specs and types are read on one `Stack` of
`Limits::depth` frames by a `for` over at most `Limits::document` steps,
each placing one type or spec: a string, one of 6.1's nine type names
(else `Wrong { expected: Type }` at it), is pushed whole; `{"list": t}` or
`{"map": t}` is pushed with the next index, then `t` is read; `{"object":
{…}}` is pushed, its fields read in byte order, each name an identifier
(`Name`) and each spec's `ty` the index at which its type starts, and its
`Form::Object` set when its last field ends; `{"enum": […]}` holds at
least one string, each once (format.md 6.1). A type object of no key, or of two of
the four, is `Wrong { expected: Type }` at its `{`. A spec's keys are
6.1's (else `Unknown { of: Spec }`); `type` and `doc` required; `doc` one
line and at most 200 bytes, dropped; `optional` `true`, never beside
`default`; `min` and `max` on `int` only, `max_length` at least 0 and on
`string`, `list` or `map` only (`Bound`; format.md 6.1); `min` above `max`: `Above`.
Each check of a spec runs once its type is read.
Test shape: `src/tests/schema.rs`, under 20 ms each:
```rust
const PARAMS: &[u8] = b"{\"settings\": {\"params\": {\n  \"type\": {\"map\": {\"object\": {\n    \"param\": {\"type\": \"string\", \"doc\": \"One parameter.\"},\n    \"order\": {\"type\": \"int\", \"min\": 0, \"max\": 999, \"default\": 500, \"doc\": \"Its place.\"}}}},\n  \"default\": {}, \"doc\": \"Parameters, by name.\"}}}";
#[test]
fn nested_types_are_laid_out_each_before_its_elements() {
    let declaration = declaration(PARAMS).expect("boot's params, 5.4");
    let [params] = &declaration.settings[..] else { panic!("one setting") };
    assert_eq!((params.name.clone(), params.at, params.ty), (text(at(14, 1, 15), b"params"), at(24, 1, 25), 0));
    let [map, object, int, string] = &declaration.types[..] else { panic!("four types, in pre-order") };
    assert_eq!((map.at, &map.form, object.at), (at(36, 2, 11), &Form::Map(1), at(44, 2, 19)));
    let Form::Object(fields) = &object.form else { panic!("the map's values are objects") };
    let [order, param] = &fields[..] else { panic!("two fields, in name order") };
    assert_eq!((order.at, order.ty, order.min, order.max), (at(128, 4, 14), 2, Some(Number { at: at(151, 4, 37), value: 0 }), Some(Number { at: at(161, 4, 47), value: 999 })));
    assert_eq!((int.at, &int.form, string.at, &string.form, param.ty), (at(137, 4, 23), &Form::Int, at(79, 3, 23), &Form::String, 3));
}
```
Then `each_of_the_thirteen_types_decodes_from_its_name_or_object` (a
table), `a_type_name_6_1_lacks_is_refused_at_its_string` (`"integer"`,
`"float"`, `"null"`, and as a list's element),
`a_setting_or_field_name_out_of_2_4s_form_is_refused_at_its_key` (`Port`,
`a-b`, `1st`, each as a setting and as an object's field: `Name`),
`a_spec_without_type_or_doc_is_refused_at_its_brace`,
`a_spec_doc_of_two_lines_or_201_bytes_is_refused_at_its_value`,
`a_spec_value_of_another_form_is_refused_at_it` (`optional` false, `min`
`"1"`, `type` 5, `enum` `["a", 1]`),
`optional_beside_a_default_is_refused_at_its_key`,
`bounds_on_a_type_they_do_not_bound_are_refused_at_their_key`,
`min_above_max_is_refused_at_min_s_value`,
`an_enum_with_no_value_or_one_twice_is_refused`,
`the_513th_setting_is_refused_at_its_key` (at `SHIPPED`, `s000` to
`s512`, each `{"type": "bool", "doc": "d"}`). `world::limits(rng)` draws
`settings` too, tiny now and then.
World: `every_schema_example_decodes_to_its_settings_cut_anywhere`, over
a const table: `boot` (kernel, modules, params; 8 types), `hearth`
(alarms, backups, snapshots; 14), `locale` (locale, timezone; 4), `users`
(accounts; 7). Under 0.5 s.
Proof: world,
`every_schema_example_decodes_to_its_settings_cut_anywhere`: format.md's
four tasks with settings, delivered in random pieces, give their
settings by name and as many types as the table lists. Nothing calls
`declaration` until hearth 13. Bound: 0.5 s.
Don't: check a default against its type (hearth 13); recurse; keep a
`doc`.
Size: at most 280 lines.
Removes: `settings` from `Undecoded`, and its row.

### 35.2 Exports; the declarations' sweep

Scope: `exports` (8.1); the world's generated declarations and their
sweep (testing-strategy.md, 2.4, 3; decisions.md, 10).
Needs: nothing beyond 35.1.
Files: `src/{schema,declaration,error}.rs`,
`src/tests/{schema,declaration}.rs`; `tests/config/src/{generate,
fixtures}.rs`, `tests/config/examples/tasks/{vpn,backup-peer}.json` (8.6, 8.7),
`tests/config/tests/{declarations,fuzzy_declarations}.rs`.
Shape: `Declaration::exports`: export names identifiers (format.md 2.4, 8.1); keys `type`,
`doc`, `optional`, `value` (else `Unknown { of: Export }`); `type`, `doc`
and `value` required; the type read by 35.1's walk into `types`, then
refused (`Secret`, at that type) where it or any of its elements is
`Form::Secret` (format.md 8.1: never `secret` at any depth); `doc` as a spec's, dropped; `value` a string. Memory:
`declaration` moves 03's strings but boxes new lists while the decoded
`Value` is still held, so 03's `json::worst_case` does not cover it:
`declaration::worst_case(&Limits) -> Option<u64>` is `json::worst_case`'s
plus `document / 2 + 1` entries of the largest of `Type`, `Spec`, `Export`
(`size_of`, `checked_mul`), and `tests/memory.rs` meters `declaration` on
`at_the_limits`' cases against it.
Test shape: `src/tests/schema.rs`:
`an_export_hands_up_its_type_and_value_text`,
`an_export_whose_type_holds_secret_is_refused_at_that_type`,
`an_export_without_type_doc_or_value_is_refused_at_its_brace`,
`an_export_key_only_a_setting_takes_is_refused_at_it` (`default`, `min`),
`an_export_name_out_of_form_is_refused_at_its_key`. World:
`generate::declaration(rng, &Limits) -> Written { bytes, texts }` writes a
declaration of every key but `checks`, each `Text` recorded with the
position it wrote; `generate::break_once(rng, &Written) -> Broken { bytes,
at, fault }` breaks one place (it may write `checks` or an `artifact`, so
`Later` falls), its fault and position the generator's;
`fixtures::Tally` records the faults that fell. `tests/declarations.rs`
gains `every_task_example_decodes_as_a_declaration_cut_anywhere` (the 23)
and `generated_declarations_hand_up_each_string_where_it_was_written` (32
seeds, under 0.5 s); `tests/fuzzy_declarations.rs`, about 2 s: `const ROUNDS:
u64` is chosen once so that `heavy cargo nextest run -p hearth-config-world
--profile fuzzy` takes about 2 s, and `FIRST_SEED` is `0x0035_F000`:
```rust
#[test]
fn generated_declarations_broken_once_fail_at_the_break() {
    let mut tally = Tally::default();
    for seed in FIRST_SEED..FIRST_SEED + ROUNDS {
        let mut rng = Rng::new(seed); let limits = world::limits(&mut rng);
        let written = generate::declaration(&mut rng, &limits);
        let broken = generate::break_once(&mut rng, &written);
        let outcome = world::declare(&broken.bytes, &Settings::calm(&mut rng, limits), seed);
        assert_eq!(outcome.err(), Some(Error { at: broken.at, fault: Fault::Declaration(broken.fault) }), "seed {seed}");
        tally.record(broken.fault);
    }
    tally.assert_fell(Tally::EVERY_DECLARED);
}
```
`EVERY_DECLARED`: each `Declared` variant, with `Unknown` of each
`Object`.
Proof: world,
`every_task_example_decodes_as_a_declaration_cut_anywhere`: format.md's 23
task.json blocks, delivered in random pieces, are declarations, vpn's with
exports `address`, `endpoint`, `hub` and `public_key`. Nothing calls
`declaration` until hearth 13 and 16. Bound: 0.5 s.
Don't: export a `secret`; generate what a reference would judge (the
generator knows what it broke).
Size: at most 80 lines.
Removes: `Declared::Undecoded` and
`the_keys_decoded_later_are_refused_until_their_increment`.

## Interface sketches

The protocol layer (hearth 13, 20) translates `types`, `settings` and
`exports` field for field, as 04's sketches say; every `Text` passes
through `hearth-protocol-template` by its place (04, Choices). Every type
derives `Clone, PartialEq, Eq, Hash, Debug` (`Copy` where it can); an `at`
is where its thing starts. What 04 sketches grows only as marked.

```rust
// hearth_protocol_config::limits (03's, `settings` added by 35.1)
/// The machine's limits: a document's bytes; nesting; keys an object; settings in a task (render.md, 2.3).
pub struct Limits { pub document: u32, pub depth: u32, pub keys: u32, pub settings: u32 }
pub const SHIPPED: Limits = Limits { document: 1_048_576, depth: 32, keys: 4_096, settings: 512 };

// hearth_protocol_config::declaration (04's `Declaration` gains three fields)
pub struct Declaration {
    // ... 04's fields
    pub types: Box<[Type]>,    // 35.1: the types of settings and exports, each before its elements
    pub settings: Box<[Spec]>, // 35.1
    pub exports: Box<[Export]>, // 35.2
}

// hearth_protocol_config::schema (format.md, 6.1, 6.2, 8.1)
use crate::{declaration::{Number, Text}, json::Value};
/// A type, at its name's quote or its object's `{`.
pub struct Type { pub at: Position, pub form: Form }
/// 6.2's types; an index names a later type of `Declaration::types`.
pub enum Form {
    Bool, Int, String, Path, Mode, Duration, Secret, Target, File, // the nine named types
    Enum(Box<[Text]>),                                             // its values, as written
    List(u32),                                                     // its elements' type
    Map(u32),                                                      // its values' type, keyed by name
    Object(Box<[Spec]>),                                           // its fields' specs
}
/// A setting's or a field's spec: its name, at its key; its `{`; its type; its default as written; its bounds.
pub struct Spec {
    pub name: Text, pub at: Position, pub ty: u32, pub default: Option<Value>, pub optional: bool,
    pub min: Option<Number>, pub max: Option<Number>, pub max_length: Option<Number>,
}
/// An export: its name, at its key; its `{`; its type; whether it may be absent; its value, one `{{ expr }}`.
pub struct Export { pub name: Text, pub at: Position, pub ty: u32, pub optional: bool, pub value: Text }

// hearth_protocol_config (src/error.rs): 04's `Declared`, `Object`, `Expected` and `Required` grow
pub enum Declared {
    // ... 04's variants
    Doc,      // 04's, and a spec's or export's doc past 200 bytes, at the value
    Optional, // `optional` beside `default`, at `optional`'s key
    Bound,    // a bound on a type it does not bound, at the key; `max_length` below 0, at its value
    Above,    // `min` above `max`, at `min`'s value
    Enum,     // an enum with no value, at its `[`; a value twice, at the second
    Secret,   // an export's type that is or holds `secret`, at that type
    Settings, // a setting past `Limits::settings`, at its key
}
pub enum Object { /* ... 04's */ Spec, Type, Export }
pub enum Expected { /* ... 04's */ True, Type } // `true` (`optional`); a type
pub enum Required { /* ... 04's */ Type, Doc, Value } // a spec's or export's `type` and `doc`, an export's `value`
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-35\.[1-2]'` prints 2.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest
   run -p hearth-protocol-config -p hearth-config-world` passes, and
   `heavy cargo nextest run -p hearth-config-world --profile fuzzy -E
   'binary(=fuzzy_declarations)'` passes, its `Summary` naming 1 test.
3. `git -C ~/src/rust/hearth grep -n -w Undecoded origin/main --
   crates/hearth-protocol-config/src` prints nothing, and `git -C
   ~/src/rust/hearth grep -n hearth-protocol-template origin/main --
   crates/hearth-protocol-config/Cargo.toml` prints nothing.
4. `git -C ~/src/rust/hearth grep -n 'settings: 512' origin/main --
   crates/hearth-protocol-config/src/limits.rs` prints one line.
