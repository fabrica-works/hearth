# Session 03: `hearth-protocol-config`, strict JSON with positions and the configuration documents

hearth's configuration machine. `json::Decoder` composes skein-json's
tokenizer as skein's collector does, demands one token at a time and
builds one `Value` per document, each value and key at the `Position`
skein 02 gives its token, refusing what format.md, 2.2 refuses and
dropping `_note`. Over it, `document::document` gives a configuration
document's shape (format.md, 3.1 to 3.4, M1's): tasks with their settings
or `null` and `debian` (03.1; hearth 11.2 needs through 03.3,
11-thin-story.md:175), then `placement` (03.3) and `members` (03.4). 03.2 opens the
machine world `tests/config` (`hearth-config-world`), its sweep and the
worst case. hearth 04 adds declarations after this session. Nothing
calls the machine until hearth 11's protocol layer routes it.

## Read first

- hearth's `format.md`: 2.1, 2.2, 2.4, 3.1 to 3.4, 3.6, 4.1, 4.3, 5.3
  (the `debian` bullet); `render.md`: 2.3, 3.1, 7.1 to 7.3, 8.1, 8.2,
  9.4; `README.md`: 4.2, 8.1 (the JSON row); `testing.md`: 2.1, 2.4, 2.9.
- skein's `json.md`: 3.1 to 3.6, 6, 7; `tokenizer.rs` (`Limits` :66,
  `Tokenizer::new` :290, `restart` :323, `up` :379, `down` :414) and
  `collector.rs` (`Collector` :179, `up` :328, `tokenizer_env` :406) of
  `crates/skein-json/src/`; skein 02's `Position`.
- `findings/skein.md`, 2; `findings/skein-testing.md`, 2, 3, 7;
  `decisions.md`, 9, 10. skein's `programming-model.md`: 3, 4.3, 6.3, 7,
  8, 10; `testing-strategy.md`: 2.1, 2.4, 3, 6, 8 (by number only).

## What the design asks

- **Strict JSON** (format.md, 2.2): "Duplicate keys are an error at every
  depth"; integers within i64; no U+0000; `_note`, "dropped when the
  document is decoded"; "1 MiB a document, depth 32, 4,096 keys an
  object" (render.md, 2.3).
- **Positions** (render.md, 3.1): typed values "each with its line and
  column"; an error's record holds `at { path, line, column }`, `also`,
  `target`, `task` (7.1).
- **Documents** (format.md, 3.3): `"ssh": {}` adds, `"wifi": null`
  removes; reserved `debian`, `placement` (3.4: `kind`, `firmware`,
  `hardware`, `on`, `mutable`), `members` (3.6: "a layer of its own"),
  `_note`; names per 2.4.

## Choices this plan makes

- **One `Decoder` per document,** one event above and one demand below a
  call as the collector's `UP_MAX_OUT`: the tokenizer restarts only after
  its own outcome (`tokenizer.rs:323`).
- **One owned tree:** objects as `Box<[Entry]>` in their keys' byte order
  (format.md, 4.3); open, an object's keys sit in a `skein_lib::Map` of
  `Limits::keys`, an array's elements in a `List` regrown by doubling.
- **A fault is where what is at fault starts** (json.md, 7): a key's
  opening quote, a value's first byte, a missing field at its object's
  `{`; a duplicate names the first key's position (7.1's `also`).
- **Order:** a token's fault is decided once the tokenizer has read it
  (json.md, 3.2); a key: U+0000, duplicate, `Limits::keys`; a number:
  fraction or exponent, then i64.
- **One error a document: its first fault** (render.md 7.1); `_note` is a
  key of its object for the 4,096 (format.md 2.2); a shape fault is
  `decode` (render.md 7.2).
- **The tokenizer's `string` and `number` are the document's length**
  (`tokenizer.rs:603`); `chunk` is 4 KiB. A non-string `_note` is refused.
- **One `Document` for the four configuration documents:** what each place
  may hold and what `on` names are composition's (render.md, 7.2; hearth
  12). `debian` stays a `Value` (hearth 17 types it).
- **The world's examples are format.md's nine documents** (3.4's ion-prod,
  3.5's chat-prod, 3.6's stoa, 3.7's three, 4.2's two, 4.4's carbon-lab);
  the golden repositories join them in hearth 30.

## Rules that bind it

- **Upstream first:** 03.1 starts with a repin commit, `heavy
  cargo update -p skein-lib`, after `git -C ~/src/rust/skein fetch origin`; `grep -o 'skein.git#[0-9a-f]*' Cargo.lock | sort
  -u` then prints one commit, which `git -C ~/src/rust/skein merge-base
  --is-ancestor <L> <it>` accepts, `<L>` being `git -C ~/src/rust/skein
  log origin/main --all-match --grep='^Plan: render$'
  --grep='^Step: skein-02\.' --format=%H -1`. With no skein in `Cargo.lock` yet, no repin
  commit: the crate's first `heavy cargo check` pins it.
- **No recursion (a review guide: clippy lints only `Vec`, `String`); the
  shipped limits:** open containers live in a
  `skein_lib::Stack` of `Limits::depth`; tests take `limits::SHIPPED`, or
  name another value with the reason (code-organization.md, 4).

## Increments

### 03.1 Strict JSON with positions; a document's tasks

Scope: the crate, its limits (render.md, 2.3), the strict JSON machine
(format.md, 2.2; render.md, 3.1), a document's tasks and `debian` (2.4, 3.3).
Needs: skein 02's `Tokenizer::position() -> Position { offset, line,
column }` (json.md, 3.6): the session's start.
Files: root `Cargo.toml`, `Cargo.lock` (shared: README, Rules, "Shared
files"): the `crates/*` glob, if absent;
`[workspace.dependencies]` gains `skein-lib`, `skein-json` (`{ git =
"https://github.com/forgetic/skein.git" }`) where absent, and the crate
by path. `crates/hearth-protocol-config/`: `Cargo.toml` (`[lints]
workspace = true`), `src/{lib,limits,json,error,names,document}.rs`
(`no_std` with alloc, `forbid(unsafe_code)`), `src/tests.rs`,
`src/tests/{fixtures,json,document}.rs`.
Shape (sketched): `Decoder::new(&Limits)` holds a `Tokenizer` under
`limits::tokenizer`, its events, the open containers, a state. `Decode`
sends `Next`; each `Token` is placed at `tokenizer.position()` and `Next`
follows, until `Done` after the root: `Decoded(value)`. The tokenizer's
`Failed(e)` is `Failed(Error { at: position(), fault: Json(e) })`, or
`Stream(f)`: in a string or a key too, the tokenizer's position stands,
"the first byte of the character or the escape at fault" (render.md 7.1;
json.md 3.6), and only hearth's own faults in a string, `Nul`, are at its
opening quote; on a fault of its own the decoder answers `Failed` and closes
the tokenizer. `document::document` loops over the root's entries:
`debian`, a task name with an object or `null`, or `Shape::Reserved`.
Test shape: `src/tests/fixtures.rs`: `decode(&[u8])`, `decode_under(&[u8],
&Limits)` (each demand met from the bytes in one delivery, then `End`, as
jig's `crates/jig-protocol-config/src/document.rs:12` pumps a buffer),
`document(&[u8])`, `at(offset, line, column)`. Unit tests under 20 ms.
```rust
#[test]
fn a_value_carries_the_line_and_column_of_its_first_byte() {
    let value = decode(b"{\n  \"core\": {\"domain\": \"ekanayaka.io\"},\r\n  \"n\": [1, -2]\n}").expect("strict JSON");
    assert_eq!(value.at, at(0, 1, 1));
    let Kind::Map(entries) = &value.kind else { panic!("the document is an object") };
    let [core, n] = &entries[..] else { panic!("two keys, in byte order") };
    assert_eq!((&*core.key, core.at, core.value.at), (&b"core"[..], at(4, 2, 3), at(12, 2, 11)));
    assert_eq!((&*n.key, n.at, n.value.at), (&b"n"[..], at(43, 3, 3), at(48, 3, 8)));
}

#[test]
fn a_task_value_is_its_settings_object_or_null() {
    let document = document(b"{\"_note\": \"x1\", \"core\": {\"hostname\": \"x1\"}, \"wifi\": null}").expect("a document");
    let [core, wifi] = &document.layer.tasks[..] else { panic!("two tasks, the note dropped") };
    assert_eq!((&*core.name, core.at, core.value.at), (&b"core"[..], at(16, 1, 17), at(24, 1, 25)));
    assert_eq!(wifi.value, Value { at: at(52, 1, 53), kind: Kind::Null });
}
```
`src/tests/json.rs`, tables of `(document, Error)` literals:
`a_duplicate_key_fails_at_the_second_naming_the_first` (root, nested, in a list,
`_note` twice), `a_fraction_or_an_exponent_fails_at_the_number`,
`integers_decode_to_i64_s_edges_and_fail_one_past`,
`u0000_in_a_string_or_a_key_fails_at_its_opening_quote`,
`a_bad_escape_in_a_string_or_a_key_fails_at_its_backslash` (`"a\q"`, a
value and a key: `Json(Escape)` at the `\`, not the quote; render.md 7.1),
`a_note_is_dropped_from_every_object`,
`a_note_that_is_not_a_string_fails_at_its_value`,
`a_document_that_is_not_an_object_fails_at_its_first_byte`,
`the_shipped_limits_refuse_one_past_each` (depth 33, the 4,097th key, 1 MiB and
a byte). `src/tests/document.rs`:
`debian_is_kept_as_written_with_its_positions`,
`a_task_value_neither_object_nor_null_is_refused_naming_the_task`,
`a_key_that_is_no_task_name_is_refused_at_the_key`,
`placement_and_members_are_refused_until_they_are_decoded`.
Proof: unit, `a_value_carries_the_line_and_column_of_its_first_byte`:
a CRLF document's keys and values at their lines and columns. Bound:
0.02 s.
Don't: repeat a tokenizer check; read through the collector, which gives
no positions (json.md, 3.6); use `Vec`, `String` or recursion.
Size: at most 300 lines.
Removes: nothing.

### 03.2 The machine world, its sweep and the worst case

Scope: the machine world (testing.md, 2.4; testing-strategy.md, 2.4, 3):
one `Decoder` from a seed, a stream below it that cuts, delays, ends early
or fails, a user above it that asks late, stalls and closes in every
state; a reference; generated and mutated documents; the sweep;
`worst_case` (programming-model.md, 6.3).
Needs: nothing beyond the session's start.
Files: `src/json.rs`. `tests/config/` (ordinary Rust, its own `[lints]`,
`findings/skein-testing.md`, 7): `Cargo.toml` (`skein-lib`, `skein-json`,
`skein-world`, the crate; dev `skein-heap`), `src/{lib,world,reference,
generate,examples,fixtures}.rs`, `examples/*.json`,
`tests/{world,memory,fuzzy_documents}.rs`. Shared: `Cargo.lock`.
Shape: `json::worst_case(&Limits)`: the tokenizer's, the open containers
and their keys' maps, and the value being built: at most `document / 2 +
1` values, their text, an open array's doubling.
Test shape: `world::Settings::{calm, chaotic}`, `world::limits(rng)`
(tiny now and then, so each fault falls), `world::check(document,
&Settings, seed) -> Run`, whose `Run::value: Option<Value>` is the document decoded (03.3's and 03.4's proofs read it), after skein's `tests/json/src/world.rs` (:30 to
:119, :279), the side below `skein_world::stream::Wire`
(`testing/skein-world/src/stream.rs:16`), ended early or failed by the
world. `check` holds each run to `reference::decode(document, &Limits)`,
recursive, sharing no code with the machine (std judges UTF-8 and
integers; it counts bytes and line feeds itself): the same `Value` at each
position; the same hearth fault at the same position; `Fault::Json(_)`
where the reference first finds no JSON, whose error and position skein
02's reference checks. `generate::{document, mutate}` (duplicates,
fractions, i64's edges, `\u0000`, `_note`s, CRLF); `fixtures::{Seen,
at_the_limits, metered}`. Focused tests under 0.5 s, under 1 s in all
(development.md, 5.3); the sweep about 5 s, `ROUNDS` set once by the
measure profile: `heavy cargo nextest run -p hearth-config-world --profile
measure -j 1 --ignore-default-filter -E 'binary(fuzzy_documents)'` once,
`ROUNDS` the largest value that ran in 5 s or less, never re-measured.
```rust
#[test]
fn every_example_decodes_as_the_reference_reads_it_cut_anywhere() {
    assert_eq!(examples::all().len(), 9, "format.md's nine documents");
    for (example, seed) in examples::all().iter().flat_map(|e| (0..8).map(move |s| (e, s))) {
        let settings = Settings::calm(&mut Rng::new(seed), limits::SHIPPED);
        let run = world::check(&example.bytes, &settings, seed);
        assert_eq!(run.outcome, Some(Outcome::Decoded), "{} under seed {seed}", example.name);
    }
}
#[global_allocator]
static HEAP: Counting = Counting;
#[test]
fn each_step_holds_no_more_than_the_worst_case_under_tiny_limits() {
    let limits = Limits { document: 4_096, depth: 4, keys: 8 }; // SHIPPED's would run too long
    let bound = json::worst_case(&limits).expect("the limits are honoured");
    for (document, what) in fixtures::at_the_limits(&limits) { assert!(fixtures::metered(&document, &limits, bound, what) > 0, "{what}"); }
}
#[test]
fn generated_documents_mutated_and_cut_read_as_the_reference_reads_them() {
    let mut seen = Seen::default();
    for seed in FIRST_SEED..FIRST_SEED + ROUNDS {
        let mut rng = Rng::new(seed); let limits = world::limits(&mut rng);
        let mut document = generate::document(&mut rng, &limits);
        if rng.chance(700) { document = generate::mutate(&mut rng, &document); }
        let settings = Settings::chaotic(&mut rng, limits, document.len());
        seen.record(&world::check(&document, &settings, seed), &settings);
    }
    seen.assert_fell(Seen::EVERY_FAULT_AND_ENDING);
}
```
In order: `tests/world.rs` (the first test above), with
`generated_documents_decode_as_the_reference_reads_them`,
`a_document_cut_after_any_byte_fails_as_its_prefix_reads`,
`a_close_in_each_waiting_state_answers_closed_once_and_last`,
`a_failed_stream_fails_the_decode_with_its_fault`; `tests/memory.rs` (`metered`
makes each entry point's call a `skein_heap::Meter` step, `check`ed against
`bound`, after skein's `tests/json/tests/memory.rs:134`; `at_the_limits`: the
widest, the deepest, a string and an array of zeros filling the document, each
also failing); `tests/fuzzy_documents.rs` (`EVERY_FAULT_AND_ENDING`: each
`Fault` but `Shape`, `Decoded`, an early end, a failed stream, a close in each
`Waiting`).
Proof: world,
`every_example_decodes_as_the_reference_reads_it_cut_anywhere`:
format.md's nine documents, delivered in random pieces, decode to the
reference's values at its positions. Bound: 0.5 s.
Don't: depend on `skein-json-world` (`findings/skein-testing.md`, 3);
add a test-only API to the crate.
Size: at most 80 lines.
Removes: nothing.

### 03.3 Placement

Scope: `placement`'s M1 fields (format.md, 3.4), typed, checked alone.
Needs: nothing beyond the session's start.
Files: `src/{document,error}.rs`, `src/tests/document.rs`,
`tests/config/tests/world.rs`. hearth 11.2 (Needs 03.3) translates it.
Shape: `Layer.placement` (sketched). Refused: not an object, `null`
included, at the value (format.md 3.4: never a removal); a field M1 lacks,
at its key, a `schema` error (format.md 2.2; render.md 7.2), a
container's `mutable` among them, whatever its value (format.md 2.2, 3.4);
a wrong type, or a `kind` or `firmware` not listed, at the value; a placement is whole
where it is written (format.md 3.4): no `kind`, no `firmware` or
`hardware` on metal or a VM, no `on` on a container, at the `{`;
`firmware` or `hardware` on a container, `on` on anything else, at the key.
Test shape: `src/tests/document.rs`:
`a_vm_s_placement_decodes_its_kind_firmware_and_hardware` (ion-prod),
`a_container_s_placement_names_its_host_where_it_is_written` (chat-prod),
`mutable_is_false_unless_a_placement_sets_it_true`,
`a_placement_field_m1_lacks_is_refused_at_its_key` (`ids`, `binds`; a
container's `mutable`, `true` and `false`),
`firmware_or_hardware_on_a_container_is_refused_at_its_key`,
`on_for_anything_but_a_container_is_refused_at_its_key`,
`a_kind_or_firmware_not_listed_is_refused_at_its_value`,
`a_null_placement_is_refused_at_its_value`,
`a_placement_missing_what_its_kind_needs_is_refused_at_its_brace` (no
`kind`; a VM without `firmware`; metal without `hardware`; a container
without `on`).
Proof: world,
`the_placed_examples_decode_to_their_placements_cut_anywhere`: x1-prod,
ion-prod and chat-prod, delivered in random pieces, give metal, efi,
7th-gen; vm, bios, ionos; a container on `rhi-prod` at its line and
column. Bound: 0.5 s.
Don't: check what needs another document (`on`'s target, environment,
kind); type an M5 or M6 field.
Size: at most 150 lines.
Removes: `placement` from `Shape::Reserved`, and its test row.

### 03.4 Members

Scope: `members` (format.md, 3.6): member names (2.4) to a member's layer
or `null`.
Needs: nothing beyond the session's start.
Files: `src/{document,error,names}.rs`, `src/tests/document.rs`,
`tests/config/tests/world.rs`; hearth 11 destructures `members: _`.
Shape: `Document.members`, `names::member` (sketched); a member's layer is
a document's (tasks, `debian`, `placement`), with `members` refused (format.md 3.6).
Test shape: `src/tests/document.rs`:
`a_base_s_members_decode_in_name_order_each_a_layer` (stoa),
`a_member_set_to_null_is_removed`,
`a_member_name_out_of_form_is_refused_at_its_key` (`"a-"`, `"Stoa"`),
`members_inside_a_member_are_refused_at_the_key`,
`a_member_s_value_neither_object_nor_null_is_refused`,
`a_member_s_layer_holds_its_placement`.
Proof: world, `every_example_decodes_as_a_document_cut_anywhere`:
format.md's nine documents, delivered in random pieces, are documents,
stoa's with members `1`, `2`, `3`. Bound: 0.5 s.
Don't: make target names from members (render's discovery, hearth 42).
Size: at most 120 lines.
Removes: `Shape::Reserved`, and
`placement_and_members_are_refused_until_they_are_decoded`.

## Interface sketches

The protocol layer (hearth 11) owns each `Decoder`, its queues and the
stream below (one blob). Every type derives `Clone, PartialEq, Eq, Hash,
Debug` (`Copy` where it can), `Decoder` only `Debug`; an `at` is where its
thing starts.

```rust
// hearth_protocol_config::limits (render.md, 2.3)
/// The machine's limits (programming-model.md, 7): a document's bytes; nesting; keys an object, as written.
pub struct Limits { pub document: u32, pub depth: u32, pub keys: u32 }
pub const SHIPPED: Limits = Limits { document: 1_048_576, depth: 32, keys: 4_096 };
/// `depth`, `length`, `string` and `number` from `limits`, chunks of 4 KiB.
#[must_use] pub fn tokenizer(limits: &Limits) -> skein_json::tokenizer::Limits;
// hearth_protocol_config::json (format.md, 2.2; render.md, 3.1)
/// From the side above: a `Decode` gets one answer unless `Close` comes first.
pub enum Request { Decode, Close }
/// To the side above: the document (an object) or why it is refused; `Closed`, terminal.
pub enum Event { Decoded(Value), Failed(Error), Closed }
/// What the decoder waits for, a function of its state.
pub enum Waiting { Decode, Bytes, Close, Nothing }
/// One document's decoder: the tokenizer, the open containers, the state.
pub struct Decoder { /* private */ }
impl Decoder { #[must_use] pub fn new(limits: &Limits) -> Decoder; #[must_use] pub fn waiting(&self) -> Waiting; }
pub const UP_MAX_OUT: skein_json::tokenizer::MaxOut; pub const DOWN_MAX_OUT: skein_json::tokenizer::MaxOut; // each one above, one below
pub fn up(decoder: &mut Decoder, env: &Env<Limits>, ev: Up, above: &mut Queue<Event>, below: &mut Queue<Down>);
pub fn down(decoder: &mut Decoder, env: &Env<Limits>, rq: Request, above: &mut Queue<Event>, below: &mut Queue<Down>);
/// The most a decoder holds, its value being built included (03.2).
#[must_use] pub fn worst_case(limits: &Limits) -> Option<u64>;
/// A value, at its first byte.
pub struct Value { pub at: Position, pub kind: Kind }
/// What a value is.
pub enum Kind {
    Null, Bool(bool), Int(i64) /* within i64 */, List(Box<[Value]>),
    String(Box<[u8]>), // UTF-8, unescaped, no U+0000
    Map(Box<[Entry]>), // in the keys' byte order, each once, `_note` dropped
}
/// One key of an object, at its opening quote, and its value.
pub struct Entry { pub key: Box<[u8]>, pub at: Position, pub value: Value }
// hearth_protocol_config (src/error.rs; lib.rs: `pub use skein_json::tokenizer::Position;`)
/// Why a document is refused, at where what is at fault starts.
pub struct Error { pub at: Position, pub fault: Fault }
/// What is at fault.
pub enum Fault {
    Json(skein_json::tokenizer::Error), // not JSON, as the tokenizer says; never its `Stream`
    Stream(skein_lib::stream::Fault),   // the stream below failed: operational (render.md, 7.1)
    NotObject,                          // the document is not an object
    Duplicate { key: Box<[u8]>, first: Position }, // `first` is 7.1's `also`
    NotInteger,                         // a fraction or an exponent
    Range,                              // an integer outside i64
    Nul,                                // U+0000 in a string or a key
    Keys,                               // a key past `Limits::keys` in one object
    Note,                               // a `_note` that is not a string
    Shape(Shape),                       // a configuration document's form: `decode` (render.md 7.2)
}
/// Why a decoded value is no configuration document.
pub enum Shape {
    Name,                         // a key neither reserved nor a task name, at the key
    Settings { task: Box<[u8]> }, // a task's value neither an object nor null, at the value
    Reserved,                     // `placement`, `members` until 03.3, 03.4: removed by 03.4
    Placement,                    // 03.3: `placement` not an object
    Unknown { field: Box<[u8]> }, // 03.3: a field placement lacks in M1, a container's `mutable` included, at its key
    Wrong { field: Field },       // 03.3: a wrong type, or a `kind` or `firmware` not listed
    Missing { field: Field },     // 03.3: no `kind`, `firmware`, `hardware` or `on` its kind needs, at the placement's `{`
    Misplaced { field: Field },   // 03.3: `firmware`, `hardware` on a container; `on` elsewhere
    Members,                      // 03.4: `members` or a member's value of the wrong kind
    Member,                       // 03.4: no member name, or `members` in a member, at the key
}
/// 03.3: placement's M1 fields.
pub enum Field { Kind, Firmware, Hardware, On, Mutable }
// hearth_protocol_config::document (format.md, 3.1 to 3.4, 3.6)
/// `fleet.json`, `envs/<env>.json`, `hosts/<base>/<base>.json` or `<base>-<env>.json`; `members` from 03.4.
pub struct Document { pub at: Position, pub layer: Layer, pub members: Option<Members> }
/// A document's or a member's tasks, in name order, and reserved keys; `placement` from 03.3.
pub struct Layer { pub tasks: Box<[Task]>, pub debian: Option<Value>, pub placement: Option<Placement> }
/// A task's key and its value: a `Map` (its settings) or `Null`.
pub struct Task { pub name: Box<[u8]>, pub at: Position, pub value: Value }
/// 03.3: M1's fields, replaced whole by a later layer; `mutable` false unless set.
pub struct Placement {
    pub at: Position, pub kind: Runs, pub firmware: Option<Firmware>, pub hardware: Option<Box<[u8]>>,
    pub on: Option<On>, pub mutable: bool,
}
/// Where the target runs; how it boots.
pub enum Runs { Metal, Vm, Container }
pub enum Firmware { Efi, Bios }
/// A container's host, as written.
pub struct On { pub target: Box<[u8]>, pub at: Position }
/// 03.4: members in name order.
pub struct Members { pub at: Position, pub each: Box<[Member]> }
/// 03.4: a member's key and its layer; `None` for `null`, which removes it.
pub struct Member { pub name: Box<[u8]>, pub at: Position, pub layer: Option<Layer> }
/// The document a decoded value is, or why not.
pub fn document(value: Value) -> Result<Document, Error>;
// hearth_protocol_config::names (format.md, 2.4)
#[must_use] pub fn identifier(name: &[u8]) -> bool; // a task, a setting, an object field
#[must_use] pub fn member(name: &[u8]) -> bool;     // 03.4: a base, an environment, a member
```

## Questions to raise when reached

None.

## Done when

- `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-03\.[1-4]'` prints 4.
- Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest
  run -p hearth-protocol-config -p hearth-config-world` passes, and
  `heavy cargo nextest run -p hearth-config-world --profile fuzzy`
  passes, its `Summary` naming at least 1 test.
- `git -C ~/src/rust/hearth grep -n -w Reserved origin/main --
  crates/hearth-protocol-config/src` prints nothing; `git -C
  ~/src/rust/hearth grep -n 'pub const SHIPPED: Limits' origin/main --
  crates/hearth-protocol-config/src/limits.rs` prints one line.
