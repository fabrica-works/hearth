# Session 04: `hearth-protocol-config`: declarations, entries, expressions as text

hearth 03's machine gains the second thing it decodes, a task's
`task.json`. `declaration::declaration` takes the `Value` that 03's
`json::Decoder` gives and makes it a `Declaration` of format.md, 5.1's
fixed shape: first its plain keys (`reads`, `requires`, `scripts`, `bless`,
`doc` checked and dropped; `checks` refused until M5: 04.1), then the 14
entry maps of M1, each entry with its key, `if`, `for_each` and fields
(04.2). hearth 35 takes `settings` (the schema, the 13 types of 6.2),
`exports` and the sweep. The machine never reads an expression: every
string crosses as a `Text` at its position, and `hearth-protocol-builder`
passes each through `hearth-protocol-template` (render.md, 8.1). Each
increment proves in the machine world `tests/config`; nothing calls
`declaration` until hearth 13 (schemas) and hearth 16 (entries, guards)
route a task's `task.json` through the thin story's protocol layer.

## Read first

- hearth's `format.md`: 2.2, 2.4, 5.1, 5.2, 5.9 whole; 4.3's `order`; of
  5.3 to 5.8 and 5.10 to 5.14, each map's fields and the JSON blocks; 6.4;
  9.1, 9.3; 10.1's table, 10.2's last bullet; 11's last bullet.
  `render.md`: 2.3, 3.1, 7.1 to 7.3, 8.1.
- This plan: `hearth/03-json.md` (its Shapes, sketches, fixtures and world,
  whose names this session takes as written); `hearth/05-expressions.md`,
  Choices ("A function, not a stream machine", "One error per text");
  `code-organization.md`, 3, 4, 6; `decisions.md`, 9, 10;
  `findings/skein-testing.md`, 3 and 7.
- skein's `crates/skein-lib/src/list.rs` (`with_capacity` :21, `push` :55,
  `into_boxed` :104); `programming-model.md`: 3, 6.3, 7, 8, 10;
  `testing-strategy.md`: 2.1, 2.4, 3, 8 (by number only).

## What the design asks

- **The shape** (5.1): "`tasks/<task>/task.json` is one object. Every key
  is optional and an unknown key is an error"; 22 keys, `checks` [M5];
  "Until M5, render refuses a task that declares checks" (11).
- **Entries** (5.2): "Every declaration but `doc`, `settings`, `reads`,
  `requires`, `scripts`, `exports` and `bless` is a map of entries", each
  with "`if`: an expression" and "`for_each`: an expression giving a map or
  a list"; "A field of another type ... may be written as one `{{ expr }}`".
  Each map's fields: 5.3 to 5.7, 5.14, 9.1, 9.3, 10.1.
- **Guards** (5.9): "a map of guards, each named as 2.4 says and with
  `if` and `message`"; 2.4: "guard (5.9) | `[a-z]`, then `[a-z0-9_]`",
  and "A name out of form is a decode error at its key": `Name` at the
  key, as a read's task.
  **Reads** (6.4): `settings`, `exports: true`, `optional`. **Scripts**
  (5.8): `customize.sh`, `software.sh`, `order`, "an integer from 0 to 999,
  default 500" (4.3). **Bless** (5.13): `ignore_failed`.
- **Text with positions** (render.md, 8.1): "`hearth-protocol-config`
  cannot call the template machine, so it hands up a declaration's
  expression strings as text with their positions"; an error's `at` is
  `{ path, line, column }` (7.1).

## Choices this plan makes

- **A function over 03's `Value`,** as `document::document` is:
  `declaration::declaration(value, &Limits)`, called by the protocol layer
  on a task.json's `Decoded(value)`. Strings move out of the value into
  `Text`s; nothing is copied or read twice.
- **Every string crosses as `Text`,** its place saying what it is:
  expressions (`Entry::when`, `Entry::for_each`, `Guard::when`) for hearth
  05's `expression::compile`; interpolated (`Entry::key`, `Literal::Text`,
  `Literal::Typed`, `Export::value`, and each of `Literal::Strings`'s `each`,
  one by one: format.md 5.2) for hearth 36's; as written (names, a
  guard's message, which hearth 16 refuses when it holds `{{`, `{%` or `{#`
  (format.md 5.2; 16-entries.md's
  `a_message_holding_a_tag_or_a_comment_is_refused_at_its_string`), bless's units, and a `files` entry's `template`, which
  is never interpolated: format.md 5.5). The crate never depends on
  `hearth-protocol-template`.
- **Entries are uniform:** an `Entry` holds `Field`s named by one `Name`
  enum, and a const table gives each map's names and what each takes
  (04.2). The machine checks names and JSON forms only; presence, defaults,
  enum values, modes, paths and owners are render's, on the concrete
  entries (hearth 16 to 20), since a field's presence and a string's value
  exist only after evaluation (5.2).
- **Docs are checked and dropped,** as `_note` is: no part holds one
  (render.md, 4), and no field crosses without its counterpart
  (code-organization.md, 6).
- **A read's `exports` and `optional` are booleans,** false when absent
  (6.4).
- **`users` take `gid`,** as 5.6's table (format.md:891) and its text give ("no
  other field but `uid`, `gid`, `if` and `for_each`"; the mail task's
  `"postfix": { "from": "package", "uid": 107, "gid": 113 }`).
- **`checks` and a download's `artifact` are refused at their key**
  (`Declared::Later`: a schema error at its key until M5 or M6,
  format.md 2.2); the keys a later increment
  decodes (the 14 maps until 04.2, then `settings` and `exports`, hearth
  35's) are refused as `Declared::Undecoded`, removed by hearth 35.2.
  Nothing calls `declaration` before then.
- **The world's examples** are format.md's task.json blocks at `175e92f`,
  their fences' indentation removed, as
  `tests/config/examples/tasks/<name>.json`: 23 that decode (8.7's [M6]
  peer among them), each joining the world's tables in the increment that
  decodes all its keys (04.1, 04.2, hearth 35), and 10.2's and 11's, which
  M1 refuses.

## Rules that bind it

- **No recursion; the shipped limits** (programming-model.md, 3 and 8;
  code-organization.md, 4): no `Vec`, `String` or closure in `src/`; tests
  take `limits::SHIPPED`, or name another value with the reason.
- **One owner each** (code-organization.md, 3): hearth 03 owns `Value`,
  `Error`, `Fault` and `Limits`; this session adds `Fault::Declaration` and
  changes nothing else of them. hearth 13 and 16 mirror this session's
  types field for field.
- **The meeting point:** with hearth 11's translation of config's errors on
  `main`, an increment that adds a `Fault` variant adds its arm in
  `crates/hearth-protocol-builder/src/translate/` (README, Rules, "Edits
  outside an increment's Files": at most 5 lines a match), in its report.

## Increments

### 04.1 The declaration's keys; reads, guards, scripts, bless

Scope: format.md, 5.1's keys and their unknown key; `doc`; `reads`
(6.4), `requires` (5.9), `scripts` (5.8), `bless` (5.13); `checks`
refused (11); `Text`; the world's first task examples.
Needs: hearth 03 (the session's start): `json::Value` with positions,
`Error`, `names::identifier`, `limits`, and the world `tests/config`.
Files: `crates/hearth-protocol-config/src/{lib,declaration,error}.rs`,
`src/tests.rs`, `src/tests/{fixtures,declaration}.rs`.
`tests/config/src/{examples,world,fixtures}.rs`,
`tests/config/examples/tasks/{reads,scripts,bless}.json`,
`tests/config/tests/declarations.rs`; where 03.2's world names every `Fault`
(`Seen::EVERY_FAULT_AND_ENDING`, `tests/fuzzy_documents.rs`,
`src/reference.rs`), `Fault::Declaration` joins it. hearth 11's translation as Rules.
Shape: `declaration::declaration(value, &Limits) -> Result<Declaration,
Error>`: the root a `Kind::Map` (else 03's `Fault::NotObject` at it); a
`for` over its entries, each key looked up in a const table of 5.1's 22
keys; each string becomes a `Text` at its opening quote (render.md 7.1). `doc`: a
string with no line feed (`Doc`; one line, unbounded), dropped. `reads`: task names
(`names::identifier`) to objects of `settings` (a list of identifiers),
`exports` and `optional` (booleans). `requires`: guards, each named by an
identifier (`names::identifier`, else `Name` at its key, before its value)
and an object of `if` and `message`, strings, both required. `scripts`: `customize.sh` and
`software.sh` only, each an object of an optional `order`, an integer from
0 to 999, 500 when absent (format.md, 4.3; else `Order` at the value).
`bless`: an object of `ignore_failed`, a list of strings. `checks`:
`Later`; `settings`, `exports` and the 14 maps: `Undecoded`; any other:
`Unknown { of: Object::Declaration }`. Lists are `skein_lib::List`s of
their source's length, boxed. One error, the first met in the value's
order (render.md 7.1).
Test shape: `src/tests/fixtures.rs` gains `declaration(&[u8])` (03's
`decode`, then `declaration::declaration`, both at `SHIPPED`),
`text(Position, &[u8]) -> Text` and `at(offset, line, column) -> Position`
(03.1's landed `fixtures.rs` has `decode`, `document`, `decode_under` and
`past_shipped` alone); under 20 ms each.
```rust
#[test]
fn a_guard_hands_up_its_condition_and_message_as_text() {
    let declaration = declaration(b"{\"doc\": \"The kernel.\",\n \"requires\": {\"kernel\": {\"if\": \"target.placement.kind != 'container'\", \"message\": \"a container has no kernel\"}}}").expect("a declaration");
    let [kernel] = &declaration.requires[..] else { panic!("one guard") };
    assert_eq!((kernel.name.clone(), kernel.at), (text(at(37, 2, 15), b"kernel"), at(47, 2, 25)));
    assert_eq!(kernel.when, text(at(54, 2, 32), b"target.placement.kind != 'container'"));
    assert_eq!(kernel.message, text(at(105, 2, 83), b"a container has no kernel"));
}
```
Then `an_empty_object_is_a_task_that_declares_nothing`,
`a_key_the_declaration_lacks_is_refused_at_the_key` (`pakages`,
`setting`), `checks_are_refused_at_their_key_until_m5`,
`a_guard_without_if_or_message_is_refused_at_its_brace`,
`a_guard_named_out_of_form_is_refused_at_its_key` (`Kernel`, `no-kernel`,
`""`),
`a_read_s_task_or_setting_out_of_form_is_refused_at_it` (`Core`, `a-b`),
`a_script_5_8_does_not_name_is_refused_at_its_key` (`build.sh`),
`a_script_without_order_is_at_4_3s_default_500`,
`a_script_order_is_an_integer_from_0_to_999` (0 and 999 decode; -1 and
1000: `Order` at the value),
`a_task_doc_of_two_lines_is_refused_at_its_value`,
`the_keys_decoded_later_are_refused_until_their_increment`.
World: `examples::task(name)` reads `examples/tasks/<name>.json`, as
03.2's `all()` reads its documents; `world::declare(document, &Settings,
seed) -> Result<Declaration, Error>` runs 03.2's `check`, held to its
reference, and declares the run's decoded value at the settings' limits
(03.2's `Run::value`), documented with `# Errors` unless 03.2's
`tests/config` `[lints]` allows `missing_errors_doc`; `fixtures::{at,
text}`. Under 0.5 s.
```rust
#[test]
fn the_reads_example_decodes_to_its_three_reads_cut_anywhere() {
    for seed in 0..8 {
        let settings = Settings::calm(&mut Rng::new(seed), limits::SHIPPED);
        let declaration = world::declare(&examples::task("reads").bytes, &settings, seed).expect("format.md, 6.4");
        let [core, mail, wifi] = &declaration.reads[..] else { panic!("three reads, in name order") };
        assert_eq!((core.task.clone(), &core.settings[..]), (text(at(19, 3, 5), b"core"), &[text(at(42, 3, 28), b"domain")][..]));
        assert_eq!((mail.task.at, mail.at, mail.settings.len(), mail.exports, mail.optional), (at(115, 5, 5), at(123, 5, 13), 0, true, false));
        assert_eq!((wifi.task.at, &wifi.settings[..], wifi.exports, wifi.optional), (at(59, 4, 5), &[text(at(82, 4, 28), b"ssid")][..], false, true));
    }
}
```
and `the_scripts_and_bless_examples_decode_cut_anywhere` (5.8's orders
300 and 500; 5.13's `foo.service`).
Proof: world,
`the_reads_example_decodes_to_its_three_reads_cut_anywhere`: 6.4's reads,
delivered in random pieces, give `core` reading `domain`, `mail` its
exports and `wifi` optional, each at its line and column. Nothing calls
`declaration` until hearth 13 and 16 route a task.json through the thin
story's protocol layer. Bound: 0.5 s.
Don't: parse or look into a string (render.md, 8.1); keep a `doc`; repeat
a check of 03's decoder.
Size: at most 220 lines.
Removes: nothing.

### 04.2 Entries: keys, `if`, `for_each` and fields as text

Scope: the 14 entry maps of M1 (5.2 to 5.7, 5.14, 9.1, 9.3, 10.1), each
entry's key, `if`, `for_each` and fields; a download's `artifact`
refused (10.2; format.md 2.2).
Needs: nothing beyond 04.1.
Files: `src/{entries,declaration,error,lib}.rs`, `src/tests.rs`,
`src/tests/{entries,declaration}.rs`; `tests/config/examples/tasks/`:
`users-accounts` (5.2), `ssh` (5.5), `mail-accounts` (5.6),
`core-journal`, `wifi`, `mail-data` (5.7), `firmware`, `core-gettys`
(5.10), `mail-live`, `stoa`, `ripgrep` (5.14), `dkim` (9.1),
`ssh-identity` (9.3), `software` (10.1), and `artifact-stoa` (10.2),
`checks-ssh` (11); `tests/config/tests/declarations.rs`.
Shape: `Declaration::entries`, one `Entries` per map written. A map is an
object of entries, any key a `Text`; an entry an object; `if` and
`for_each` strings; every other key found in the map's row below (else
`Unknown { of: Object::Entry(map) }`), its value by what it takes:
`Text` a string; `Int` an integer or a string (`Literal::Typed`); `Bool`
a boolean or a string; `Strings` a list of strings or a string; `Range`
an object of `base` and `count`, each an `Int`, or a string. Anything
else is `Wrong { expected: Expected::Field(takes) }` at the value, a
list's element that is not a string `Wrong { expected: String }` at it.

| Map | Its fields (what each takes) |
|---|---|
| `packages` | `component` |
| `preseeds` | `type`, `value`, `owner` |
| `initramfs` | none |
| `cmdline` | `param`; `order` (Int) |
| `files` | `template`, `copy`, `link`, `mode`, `owner`, `group`; `directory` (Bool) |
| `users` | `group`, `gecos`, `home`, `shell`, `password`, `from`; `uid`, `gid` (Int); `groups` (Strings); `subordinate` (Range) |
| `groups` | `from`; `gid` (Int) |
| `presets` | `action` |
| `tmpfiles` | `type`, `mode`, `owner`, `group`, `age`, `argument` |
| `data`, `secrets` | `path`, `mode`, `owner`, `group`; `data` also `class` |
| `identity` | `kind`, `algorithm`, `path`, `public_path`, `mode`, `owner`, `group` |
| `software` | `url`, `sha256`, `unpack`, `path`, `mode`, `owner`, `group`; `strip` (Int); `artifact`: `Later` |
| `live` | `unit`, `action`; `paths` (Strings) |

Every field not marked takes `Text`, interpolated but for `files`'
`template`, which is `Literal::Plain`: as written, never interpolated
(format.md 5.5; so the check that every template is used stays exact).
Test shape: `src/tests/entries.rs`, under 20 ms each:
```rust
#[test]
fn an_entry_hands_up_its_key_loop_condition_and_fields_as_text() {
    let declaration = declaration(b"{\"files\": {\"/etc/ssh/authorized_keys_{{ key }}\": {\n  \"for_each\": \"settings.authorized\", \"if\": \"key in settings.allow\", \"template\": \"templates/authorized_keys.j2\"}}}").expect("ssh's entry, 5.5");
    let [Entries { map: Map::Files, at: map_at, entries }] = &declaration.entries[..] else { panic!("one map") };
    let [entry] = &entries[..] else { panic!("one entry") };
    assert_eq!(*map_at, at(1, 1, 2));
    assert_eq!((entry.key.clone(), entry.at), (text(at(11, 1, 12), b"/etc/ssh/authorized_keys_{{ key }}"), at(49, 1, 50)));
    assert_eq!(entry.for_each, Some(text(at(65, 2, 15), b"settings.authorized")));
    assert_eq!(entry.when, Some(text(at(94, 2, 44), b"key in settings.allow")));
    let template = Literal::Plain(text(at(131, 2, 81), b"templates/authorized_keys.j2"));
    assert_eq!(&entry.fields[..], &[Field { name: Name::Template, at: at(119, 2, 69), value: template }]);
}
```
Then `every_map_of_m1_takes_entries` (a table of the 14 keys),
`a_field_of_another_type_is_its_literal_or_typed_text` (`uid` 59999 and
`"{{ value.uid }}"`, `directory`, `groups`, `subordinate`),
`a_field_its_map_lacks_is_refused_at_its_key` (`packages.mode`,
`initramfs.component`, `subordinate.size`),
`a_field_of_another_form_is_refused_at_its_value` (`mode` 640, `groups`
`[1]`, `if` true),
`an_entry_or_a_map_that_is_not_an_object_is_refused_at_its_value`,
`a_download_s_artifact_is_refused_at_its_key_until_m6`.
`src/tests/declaration.rs`: the maps' rows leave the `Undecoded` table.
World: `every_entry_example_decodes_to_its_entries_cut_anywhere`, over a
const table of each example's maps and entry counts, as literals (counted once
from each example by eye and pasted: `ssh` files 2, `stoa` files 2, live
1, presets 1, software 1, and so on for all 14), 8 seeds each, under 0.5
s; and `the_m5_and_m6_examples_are_refused_at_their_keys` (`checks` at
`at(2, 1, 3)`, `artifact` at `at(38, 4, 7)`; the implementer recomputes
both from the two files, whose indentation it removed).
Proof: world,
`every_entry_example_decodes_to_its_entries_cut_anywhere`: format.md's
14 entry examples, delivered in random pieces, give the maps and entries
the table lists. Nothing calls `declaration` until hearth 16. Bound:
0.5 s.
Don't: check a field's presence, value or default (render's, hearth 16 to
20); give a map a type of its own.
Size: at most 260 lines.
Removes: the 14 maps from `Undecoded`, and their rows.

## Interface sketches

The protocol layer (hearth 13, 16) calls `declaration` on a task.json's
`Decoded(value)`, places `Error::at` in `tasks/<task>/task.json`, passes
each `Text` through `hearth-protocol-template` by its place (Choices), and
translates the rest field for field: hearth 13 `types`, `settings` and
`reads` (the first two hearth 35's); hearth 16 `entries` and `requires`;
`scripts`, `bless` and `exports` with their kind sessions (18, 19, 20).
Every type derives `Clone, PartialEq, Eq, Hash, Debug` (`Copy` where it
can); an object's members are in their keys' byte order, a JSON list in its
own; an `at` is where its thing
starts.

```rust
// hearth_protocol_config::declaration (format.md, 5.1, 5.8, 5.9, 5.13, 6.4)
/// A string of the task.json, unescaped, at its opening quote: never read by this crate.
pub struct Text { pub at: Position, pub text: Box<[u8]> }
/// An integer of the task.json, at its first byte.
pub struct Number { pub at: Position, pub value: i64 }
/// A task.json in its fixed shape, at its `{`; an absent key is an empty list.
pub struct Declaration {
    pub at: Position,
    pub reads: Box<[Read]>,
    pub requires: Box<[Guard]>,
    pub scripts: Box<[Script]>,
    pub bless: Option<Bless>,
    pub entries: Box<[Entries]>, // 04.2: the maps written
    // hearth 35 adds `types: Box<[Type]>`, `settings: Box<[Spec]>`, `exports: Box<[Export]>`
}
/// A read (6.4): the task, at its key; its `{`; its settings; whether it reads the task's exports; whether the task may be absent.
pub struct Read { pub task: Text, pub at: Position, pub settings: Box<[Text]>, pub exports: bool, pub optional: bool }
/// A guard (5.9): its name, an identifier (2.4), at its key; its `{`; `if`, an expression; its message, as written.
pub struct Guard { pub name: Text, pub at: Position, pub when: Text, pub message: Text }
/// A fragment (5.8), at its key, and its order: 0 to 999, 500 when absent (4.3).
pub struct Script { pub fragment: Fragment, pub at: Position, pub order: u16 }
/// The fragments 5.8 names.
pub enum Fragment { Customize, Software } // `customize.sh`, in the config layer; `software.sh`, in the software layer
/// Units whose failure blocks no bless (5.13), at its `{`.
pub struct Bless { pub at: Position, pub ignore_failed: Box<[Text]> }
/// The declaration a decoded task.json is, or the first fault met in the value's order.
pub fn declaration(value: Value, limits: &Limits) -> Result<Declaration, Error>;

// hearth_protocol_config::entries (format.md, 5.2 to 5.7, 5.14, 9.1, 9.3, 10.1)
/// The entry maps of M1, named by their keys.
pub enum Map { Packages, Preseeds, Initramfs, Software, Files, Users, Groups, Presets, Tmpfiles, Data, Secrets, Identity, Cmdline, Live }
/// One map: which, at its key, and its entries.
pub struct Entries { pub map: Map, pub at: Position, pub entries: Box<[Entry]> }
/// An entry: its key, interpolated; its `{`; `if` and `for_each`, expressions; its other fields.
pub struct Entry { pub key: Text, pub at: Position, pub when: Option<Text>, pub for_each: Option<Text>, pub fields: Box<[Field]> }
/// A field of an entry or of a range: its name, at its key, and its value.
pub struct Field { pub name: Name, pub at: Position, pub value: Literal }
/// A field's value, as written.
pub enum Literal {
    Text(Text),                                   // a string field, interpolated
    Plain(Text),                                  // a `template` field: as written, never interpolated (format.md 5.5)
    Typed(Text),                                  // a field of another type as a string: one `{{ expr }}` of its type (5.2)
    Int(Number),                                  // an integer
    Bool { at: Position, value: bool },           // a boolean
    Strings { at: Position, each: Box<[Text]> },  // a list of strings, at its `[`; each interpolated (5.2)
    Range { at: Position, fields: Box<[Field]> }, // `subordinate` as an object of `base` and `count`, at its `{`
}
/// What a field takes besides a string, `Text` taking only a string.
pub enum Takes { Text, Int, Bool, Strings, Range }
/// Every field name of 04.2's table, each its key (`PublicPath` is `public_path`).
pub enum Name {
    Action, Age, Algorithm, Argument, Base, Class, Component, Copy, Count, Directory,
    From, Gecos, Gid, Group, Groups, Home, Kind, Link, Mode, Order, Owner, Param, Password,
    Path, Paths, PublicPath, Sha256, Shell, Strip, Subordinate, Template, Type, Uid, Unit,
    Unpack, Url, Value,
}

// hearth_protocol_config (src/error.rs): 03's `Fault` gains `Declaration(Declared)`
/// Why a decoded value is no declaration.
pub enum Declared {
    Unknown { of: Object },       // a key its object does not take, at the key
    Later,                        // `checks` (M5) or a download's `artifact` (M6), at the key
    Undecoded,                    // a key hearth 35 decodes (`settings`, `exports`): removed by 35.2
    Wrong { expected: Expected }, // a value of another form, at the value
    Missing { key: Required },    // a required key absent, at its object's `{`
    Name,                         // a name out of its form (2.4), or a script 5.8 does not name, at it
    Doc,                          // a task's doc of two lines, at the value
    Order,                        // a script's `order` outside 0 to 999 (4.3), at its value
    // hearth 35 adds Optional, Bound, Above, Enum, Secret, Settings
}
/// The object an unknown key is in, for its message.
pub enum Object { Declaration, Read, Guard, Script, Bless, Entry(Map), Range /* hearth 35 adds Spec, Type, Export */ }
/// The form a value must have: an object, a string, a list of strings, a boolean, an integer, an entry field's.
pub enum Expected { Object, String, Strings, Bool, Int, Field(Takes) /* hearth 35 adds True, Type */ }
/// The keys required where they stand: a guard's `if` and `message`.
pub enum Required { If, Message /* hearth 35 adds Type, Doc, Value */ }
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-04\.[1-2]'` prints 2.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest
   run -p hearth-protocol-config -p hearth-config-world` passes.
3. `git -C ~/src/rust/hearth grep -n hearth-protocol-template origin/main
   -- crates/hearth-protocol-config/Cargo.toml` prints nothing.
