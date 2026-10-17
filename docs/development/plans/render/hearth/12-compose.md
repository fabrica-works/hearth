# Session 12: layers, RFC 7386, presence, provenance; the first error record

hearth 11's thin story composes for real, and says why when it cannot:
12.1 merges every document's layers by RFC 7386 with provenance; 12.2
prints the error record, `decode`. Each proves on the real loop's `hearth
render` through real git. hearth 42 takes the rest: the listing's
refusals, git's `tool`, members, names, the composition rules.

## Read first

- hearth's `format.md`: 2.3 (first and last bullets), 3.2, 4.1 to 4.4;
  `render.md`: 1 (the errors bullet), 2.3, 3 (first paragraph), 3.1, 3.2,
  7.1, 7.3, 8.3; `client/README.md`: 3.3.
- This plan: `hearth/11-thin-story.md` whole; `hearth/03-json.md`'s
  sketches, 03.3, 03.4; `hearth/10-client.md`'s "Who adds what";
  `hearth/40-client-diff-errors.md`'s 40.2 and `errors.rs` sketch.
- skein: `crates/skein-json/src/tokenizer.rs:232` (`Error`, which
  render's `Decode` mirrors); `crates/skein-lib/src/stack.rs:15`,
  `list.rs:14`. `programming-model.md`: 3, 4.4, 6.3, 7, 8;
  `testing-strategy.md`: 2.1, 2.2, 2.8 (by number only).

## What the design asks

- **Layers** (format.md 3.2, 3.3, 4.1; render.md 8.3): each a patch
  "but for `placement`, which a layer replaces whole"; "The result never
  holds `null`"; a value "carries the file, line and column of the layer
  that set it"; tasks "the keys left after merging its base, member and
  base-env layers alone"; a stack "bounded by JSON's depth".
- **Bounds and errors** (render.md 2.3, 3.1, 7.1, 8.2): "Exceeding a
  limit is an error of the revision, with the path at fault"; "A decode
  error fails what depends on the file"; render.md 7.1's record.

## Choices this plan makes

- **Merge first, so the session starts at once:** 12.1 needs what hearth
  11 and 32.1 need, 12.2 hearth 40.2's record. The README's row: Needs at
  start "hearth 11, hearth 32.1, hearth 03"; more "12.2: hearth 40.2".
- **Every configuration document, once a revision** (discovery needs
  every base's `members`; "Step 1 and discovery run once per revision",
  render.md 3): the first `Read` asks them all, in path order; their
  errors print with any command (render.md 3.1, 3.3).
- **The 64 MiB counts listed sizes** (`revision::Budget`) before each
  document or `task.json` is asked (hearth 15 adds templates); which
  limits fail the call is render.md 7.2's rows.
- **Errors are held as found, answered after the specs:** each whole, in
  a `List` starting at 16 and doubling, never past `Limits::errors` (22's;
  256), one a `resume` (`MAX_OUT_RESUME = 1`, render `lib.rs:16`: the
  output queue holds one), then `Rendered`; hearth
  22 sorts, keeps 256 and counts, changing none. render folds the
  outcome: configuration at least `Failed`, operational `TryAgain`.
- **A document's error is the file's:** once, no `target`, its targets
  left out (render.md 7.1: once, no `target`); a `task.json`'s names its
  `task`. Provenance
  holds a `revision::Documents` index, not a path.
- **One flat `Decode`** (the tokenizer's errors, config's faults and
  shapes) in render and the records: one message an arm, JSON one level
  deep in hearth 10's layout (`"decode":{"key":"motd","kind":
  "duplicate"}`, `"decode":"not_object"`); a shape fault is `decode`,
  one fault one error, config's first (render.md 7.1, 7.2).
- **Later sessions:** 13 adds `Layer::Defaults`, reads `Settings`; 17
  merges `debian` with `Merge`; hearth 42 the listing's `repository` and
  git's `tool`, members and the composition rules; 20 gives
  `target.containers`; 21 the `hearth` task; 24 mmdebstrap's `Command`.

## Rules that bind it

- **The meeting point** (README, Rules): every increment proves on the
  real loop (`tests/builder/tests/real.rs`); one that adds a variant to
  render's `errors` kinds, render's or the root's `Answer`, or the
  records' `Kind` extends `translate/errors.rs` and its reverse in
  `src/tests/translate.rs` in the same increment, no wildcard arm.
- **Shared files:** render's `lib.rs`, `vocabulary.rs` and, from 12.2,
  `src/errors.rs`; the records' `src/errors.rs` (both also hearth 13, 14,
  16, 19, 21, 22, 24, 42's) and `records.rs`; `hearth-protocol-builder`'s
  `src/translate/`; `tests/render/src/fixtures.rs`,
  `tests/builder/src/fixtures.rs`. Landing second, rebase, keep both.
- **No recursion, no computed expectation** (code-organization.md, 6):
  the merge and a value's translation walk `skein_lib::Stack`s of the
  JSON depth; positions, messages and digests in tests are literals
  (`THIN_TEXT` is 32.1's, `775324fc…` `sha256sum`'s). Every increment's
  Files include `tests/builder/{src/fixtures.rs, tests/real.rs}`.

## Increments

### 12.1 Every document, every layer: RFC 7386, provenance, presence
Scope: every configuration document read once, under 2.3's 64 MiB; a
task's `value` translated (hearth 11 left it `_`); fleet, base,
environment and base-env merged by RFC 7386 with provenance (format.md
3.2, 4.1; render.md 3.2, 8.3); presence as the merged base and base-env
keys; settings restricted to the present tasks.
Needs: hearth 11 (the thin story), hearth 32.1 (presence, the cache),
hearth 03 (`json::Value`, `Task`).
Files: render `src/{limits, revision, compose, call, lib}.rs`,
`src/tests.rs` (`mod compose;`), `src/tests/fixtures.rs`, new `src/tests/compose.rs`;
`tests/render/{src/fixtures.rs, tests/thin.rs}`, new
`tests/render/tests/compose.rs`; protocol `src/translate/documents.rs`,
`src/tests/translate.rs`; `tests/builder/{src/fixtures.rs, tests/real.rs}`.
Shape: the sketches. `limits::SHIPPED` gains `decoded: 67_108_864`
(`depth: 32` is 14.1's, `limits.rs:6` and `:15`); `translate::documents::value` walks on a `Stack`. After
`Listed`, one `Read` of every configuration document, each
`Budget::take`n first; a refusal answers `Rendered { Failed }` (12.2
adds its error). `compose::compose`: presence, the keys the base's and
base-env's task maps leave once merged; settings, fleet, base,
environment and base-env merged (a document's tasks one patch), kept for
the present tasks, held while the target renders. `Merge` keeps a frame
a level `{ target pairs, patch pairs, two cursors, pairs out }`: pairs
meet in key order (4.3), `null` drops its key, a map over a map pushes a
frame, a finished one joins its parent's pairs.
Test shape: unit (`src/tests/compose.rs`, under 20 ms each), values by
new `fixtures::{at, map, int, null, string}`, at `limits::SHIPPED`:
```rust
#[test]
fn a_later_layer_s_scalar_replaces_and_keeps_its_own_place() {
    let fleet = map(at(1, 1), &[(&b"port"[..], at(1, 2), int(at(1, 10), 22))]);
    let base = map(at(3, 3), &[(&b"port"[..], at(3, 4), int(at(3, 12), 10_127))]);
    let mut merge = Merge::new(&limits::SHIPPED);
    let composed = merge.merge(None, &fleet, compose::Layer::Fleet, 0);
    let composed = merge.merge(composed, &base, compose::Layer::Base, 2).expect("an object over an object");
    let Composite::Map(pairs) = &composed.kind else { panic!("objects merge into an object") };
    let [port] = &pairs[..] else { panic!("one key") };
    assert_eq!(port.value.kind, Composite::Int(10_127), "format.md, 4.1: scalars replace");
    let set = Provenance { layer: compose::Layer::Base, document: 2, at: at(3, 12) };
    assert_eq!(port.value.from, set, "render.md, 3.2: the layer, file, line and column that set it");
    assert_eq!(composed.from.layer, compose::Layer::Fleet, "an object keeps its maker (render.md 3.2)");
}
```
Then `null_removes_its_key_at_any_depth_and_never_stays`,
`a_list_or_a_scalar_patch_replaces_whole`,
`an_object_patch_over_a_scalar_makes_a_new_object`,
`a_merge_at_the_shipped_depth_fits_its_stack`. World
(`tests/render/tests/compose.rs`, `parent::run` as 11.1's, eight seeds,
under 0.5 s): `render_lists_once_then_reads_every_configuration_document_once`,
`fleet_and_environment_keys_never_add_a_task`,
`documents_past_the_revision_s_budget_fail_the_call` (`decoded: 64`
beside `SHIPPED`'s 64 MiB, so that three small documents pass it).
Proof: process,
`hearth_render_gives_each_target_the_tasks_its_presence_layers_leave`,
bound 0.5 s, on new `fixtures::LAYERS` (`tests/builder/src/fixtures.rs`): `fleet.json` `{"motd": {}}`,
`envs/prod.json` `{"issue": {}}`, `hosts/x1/x1.json` `{"motd": {},
"issue": {}}`, `x1-prod.json` `{"issue": null}` placed as THIN's,
`hosts/x2/x2.json` `{}`, `x2-prod.json` vm, bios, ionos; THIN's `motd`,
an `issue` with `tree/etc/issue`. The lines new `lines_starting(&stdout,
&["target ", "  /"])` keeps are exactly these, exit 0: real documents'
values cross the translation and merge into presence (settings print
first through hearth 13 and 14).
```text
target       x1-prod   base x1   environment prod
  /etc/motd                file 0644 root:root 775324fcea7f
  /usr/local/bin/motd      file 0755 root:root f487088ec8b1
target       x2-prod   base x2   environment prod
```
Don't: recurse; merge `placement` (replaced whole) or `debian` (hearth
17); read `members` (hearth 42.2); add the `hearth` task (hearth 21).
Size: at most 260 lines.
Removes: 11.1's `render_lists_once_then_reads_each_targets_documents_once`;
32.1's presence by key sets and `revision::Task::removed` (now the
merge's keys and `Task::value`).

### 12.2 The error record: `decode` on the thin story's path
Scope: render.md 7.1's record from render to standard output: render's
`errors.rs`, `Answer::Error` in render's and the root's vocabularies,
`translate/errors.rs`, the records' `Kind::Decode`; what config refuses,
a document or `task.json` past 1 MiB, the documents past 64 MiB, a
static file past 64 MiB (2.3).
Needs: hearth 40.2 (the records' `Error`, `Place`, `Class`, its line).
Files: render new `src/errors.rs`, `src/{vocabulary, revision, call,
lib}.rs`, `src/tests.rs` (`mod errors;`), new `src/tests/errors.rs`; root
`src/vocabulary.rs`, `src/lib.rs` (`forward`'s match, `:35`, gains
`Answer::Error`); protocol `Cargo.toml` (`skein-json`, for the tokenizer's
variants), `src/translate.rs` (`pub mod errors;`), new
`src/translate/errors.rs`, `src/translate/answers.rs`, `src/read.rs`,
`src/client.rs` (`:107`'s match gains `Answer::Error`),
`src/tests/translate.rs`, `src/tests/read.rs` (`:84`'s `Decoded::Refused`
takes `fault: Missing { field: Kind }, also: None`); records `src/errors.rs`, `src/form/{text,
json}.rs`, `src/tests/{errors, fixtures}.rs`; `tests/render/{src/
parent.rs, tests/thin.rs}`, new `tests/render/tests/errors.rs`;
`tests/builder/{src/fixtures.rs, tests/real.rs}`.
Shape: the sketches. `translate::errors::fault` makes config's `Error`
`Decoded::Refused { at, fault, also }` (a duplicate's `first` its
`also`; the tokenizer's `TooLong`, `TooLarge { Document }`);
`Fault::Stream` stays 11.2's `Failed`. A refused document is `Error {
kind: Decode(fault), class: Configuration, at: Some(Place { path,
position: Some(at) }), also, target: None, task: None }`, a
`task.json`'s with its `task`. At the path, no position: past
`document`, `TooLarge { Document, max }`, its targets left out; the
budget's `TooLarge { Decoded, max }`, at the path that passed it; a
static file past `static_file`, never asked (32.1), `TooLarge { Static,
max }` with its `task`, its task's targets left out. Messages (the plan's,
within 7.3's forms): `duplicate key "motd"`, `not JSON: unexpected byte`, `a document
past 1048576 bytes`.
Test shape: render unit (`src/tests/errors.rs`, written as 12.1's
reference, under 20 ms):
`a_refused_document_s_error_names_its_path_and_position`,
`a_duplicate_key_s_error_gives_the_first_as_also`,
`a_refused_task_json_s_error_names_its_task`,
`errors_follow_the_last_spec_and_precede_rendered`; limits (each beside
`SHIPPED`'s, so that a few small files pass it):
`a_document_past_its_limit_errs_at_its_path_and_fails_its_targets`
(`document: 16`), `the_budget_s_error_names_the_path_that_passed_it`
(`decoded: 64`), `a_static_file_past_its_limit_errs_and_no_read_asks_it`
(`static_file: 8`); translate:
`each_config_fault_translates_to_its_decode_and_back`; records:
`a_decode_error_prints_its_place_message_and_also`; world
(`tests/render/tests/errors.rs`, as 11.1's reference):
`a_broken_document_answers_one_error_and_spares_other_targets` (two targets
over one broken document: one error). 11.1's
`a_refused_document_fails_its_targets_and_spares_the_rest` and 32.1's
`a_malformed_task_json_fails_the_targets_that_have_the_task` gain their
errors among the answers; in `tests/render/tests/thin.rs`, the
`Decoded::Refused` fixtures (`:87`, `:140`) take `fault` and `also`,
`targets_are_answered_in_name_order_each_once`'s match (`:104`) an
`Answer::Error` arm, and
`a_document_above_its_limit_fails_only_its_targets_before_reading`
(`:115`) answers `[Spec, Error]`. Lint: `Option::map` is a disallowed
method (`clippy.toml:106`), and a hand-written `match` that rewraps
`Some` trips `manual_map`: write a helper that uses `?`.
Proof: process, bound 0.5 s (and
`a_broken_task_json_prints_its_error_with_its_task`, on `BROKEN_TASK`,
`tasks/motd/task.json` `{"a": 1, "a": 2}`: stdout is exactly
`tasks/motd/task.json:1:10: motd: duplicate key "a"
(tasks/motd/task.json:1:2)` LF, exit 2); `BROKEN_LAB` (in
`tests/builder/src/fixtures.rs`): `envs/lab.json` `{"motd": {}, "motd": {}}`,
`hosts/x1/x1-lab.json` placed vm, efi, incus; new `thin_with` gives THIN's
files and these:
```rust
#[test]
fn a_broken_document_prints_its_error_after_the_specs_that_rendered() {
    let repository = Repository::make("broken-lab", &fixtures::thin_with(fixtures::BROKEN_LAB));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    let expected = [fixtures::THIN_TEXT, "\n", "envs/lab.json:1:14: duplicate key \"motd\" (envs/lab.json:1:2)\n"].concat();
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), expected, "render.md, 7.3's form, after x1-prod's spec");
    assert_eq!(rendered.exit, Exit::Code(2), "client/README.md, 3.3: a configuration error");
}
```
config's refusal crosses render, the root, the translation and 40.2's
printer through real git, and x1-prod still renders.
Don't: sort or bound errors (hearth 22); repeat a document's error per
target; format in render.
Size: at most 300 lines (a codec: within 2.5 times).
Removes: hearth 11's "No error record" for refused documents and
`task.json` (its Choices).

## Interface sketches

Every type derives `Clone, PartialEq, Eq, Hash, Debug` (`Copy` where it
can; `Merge`, `Budget` only `Debug`). render owns `errors` and `compose`;
the records mirror render's kinds variant for variant (an `Oid` as hex).

```rust
// crates/hearth-domain-render/src/revision.rs (12.1; hearth 42.2 adds Members, 42.1 Class's four)
/// A decoded value at its first byte: mirrors config's `json::Value`.
pub struct Value { pub at: Position, pub kind: Kind }
/// Mirrors config's `json::Kind`.
pub enum Kind { Null, Bool(bool), Int(i64), String(Box<[u8]>), List(Box<[Value]>), Map(Box<[Pair]>) }
/// Mirrors config's `json::Entry` (render's `Entry` is the listing's).
pub struct Pair { pub key: Box<[u8]>, pub at: Position, pub value: Value }
/// Mirrors config's `Task`: settings (`Kind::Map`) or `null` (`Kind::Null`).
pub struct Task { pub name: Box<[u8]>, pub at: Position, pub value: Value }
/// Mirrors config's `Document`; hearth 42.2 adds `members: Option<Members>`.
pub struct Document { pub at: Position, pub layer: Layer }
/// The configuration documents in path order: path, listed size, the document once decoded.
pub struct Documents { /* private */ }
impl Documents { #[must_use] pub fn path(&self, document: u32) -> &[u8]; #[must_use] pub fn get(&self, document: u32) -> Option<&Document>; }
/// `Limits::decoded` of decoded documents by listed size (templates with hearth 15); `take` refuses past it.
pub struct Budget { /* held, max */ }
impl Budget { #[must_use] pub fn new(limits: &Limits) -> Budget; #[must_use] pub fn take(&mut self, size: u64) -> bool; }

// crates/hearth-domain-render/src/compose.rs (12.1; targets hearth 42.2); limits gain decoded
/// format.md 3.2's layers in order; hearth 13 adds `Defaults` first.
pub enum Layer { Fleet, Base, Member, Environment, BaseEnv, BaseEnvMember }
/// render.md 3.2's provenance: layer, document (a `Documents` index), where the value starts.
pub struct Provenance { pub layer: Layer, pub document: u32, pub at: Position }
/// A composed value: never null (format.md 4.1).
pub struct Composed { pub from: Provenance, pub kind: Composite }
/// A map's pairs in key order.
pub enum Composite { Bool(bool), Int(i64), String(Box<[u8]>), List(Box<[Composed]>), Map(Box<[Pair]>) }
/// A composed key, where it was set, its value.
pub struct Pair { pub key: Box<[u8]>, pub from: Provenance, pub value: Composed }
/// RFC 7386's walk (render.md 8.3): a `Stack` of `Limits::depth` frames, made once a call. `merge`
/// lays `patch`, from `layer` of `document`, over `target`; `None` when `patch` is `null`.
pub struct Merge { /* private */ }
impl Merge {
    #[must_use] pub fn new(limits: &Limits) -> Merge;
    pub fn merge(&mut self, target: Option<Composed>, patch: &Value, layer: Layer, document: u32) -> Option<Composed>;
}
/// A discovered target: its names and its layers' documents (`Documents` indices).
pub struct Target {
    pub name: Box<[u8]>, pub base: Box<[u8]>, pub member: Option<Box<[u8]>>, pub environment: Box<[u8]>,
    pub fleet: Option<u32>, pub base_document: Option<u32>, pub environment_document: u32, pub base_env: u32,
}
/// A target's present tasks in name order, for hearth 13 on.
pub struct Settings { pub tasks: Box<[Present]> }
/// A present task, the presence key that kept it, its settings (a `Composite::Map`).
pub struct Present { pub name: Box<[u8]>, pub from: Provenance, pub settings: Composed }
pub fn compose(target: &Target, documents: &Documents, merge: &mut Merge) -> Settings;

// crates/hearth-domain-render/src/vocabulary.rs (12.2), beside hearth 11's
pub enum Answer { Spec(Spec), Error(Error), Rendered { outcome: Outcome } }
pub enum Decoded { Document(Document), Declaration, Refused { at: Position, fault: Decode, also: Option<Position> } }

// crates/hearth-domain-render/src/errors.rs (12.2; hearth 42 adds Repository, Composition, Tool)
/// render.md 7.1's record: mirrors the records' `Error` (hearth 40.2) field by field.
pub struct Error { pub kind: Kind, pub class: Class, pub at: Option<Place>, pub also: Option<Place>, pub target: Option<Box<[u8]>>, pub task: Option<Box<[u8]>> }
/// A path, with its line and column where there is one: mirrors the records' `Place`.
pub struct Place { pub path: Box<[u8]>, pub position: Option<Position> }
/// 7.2's kinds; hearth 42 adds `Repository`, `Composition`, `Tool`; hearth 13, 14, 16, 19, 21, 22 and 24 theirs.
pub enum Kind { Decode(Decode) }
/// The tokenizer's errors less `TooLong` and `Stream`; config's faults; its shapes; a limit passed.
pub enum Decode {
    Unexpected, Trailing, TooDeep, StringTooLong, NumberTooLong, Number, Escape, Surrogate, Utf8, Control, Truncated,
    NotObject, Duplicate { key: Box<[u8]> }, NotInteger, Range, Nul, Keys, Note,
    Name, Settings { task: Box<[u8]> }, Placement, Unknown { field: Box<[u8]> },
    Wrong { field: Field }, Missing { field: Field }, Misplaced { field: Field }, Members, Member,
    TooLarge { limit: Limit, max: u64 },
}
/// Mirrors config's `Field`.
pub enum Field { Kind, Firmware, Hardware, On, Mutable }
/// 12.2: Document, Decoded, Static; hearth 42 adds Entries, Path, Paths, Tasks, Targets.
pub enum Limit { Document, Decoded, Static }

// crates/hearth-domain-builder/src/vocabulary.rs (12.2)
pub enum Answer { Spec(render::Spec), Error(render::errors::Error) }

// crates/hearth-protocol-builder/src/translate/errors.rs (12.2; hearth 42 extends); reverses in src/tests/translate.rs
/// config's fault to render's, a duplicate's first key its `also`; `None` for `Fault::Stream`,
/// `Fault::Declaration` (13.1's `Decode::Declaration` replaces this arm) and
/// `Fault::Json(tokenizer::Error::Stream(_))` (config's `json.rs:146` makes it `Fault::Stream`).
#[must_use] pub fn fault(fault: config::Fault, limits: &config::limits::Limits) -> Option<(render::errors::Decode, Option<render::revision::Position>)>;
#[must_use] pub fn error(error: render::errors::Error) -> records::Error;

// crates/hearth-client-protocol-records/src/errors.rs (40.2's; 12.2)
pub enum Kind { Guard(Guard), Decode(Decode) }
// Decode, Field, Limit: render's, variant for variant.
```

## Questions to raise when reached

None.

## Done when

1. `git log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-12\.[1-2]'` prints 2.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest
   run -p hearth-domain-render -p hearth-domain-builder -p
   hearth-render-world -p hearth-builder-world -p hearth-protocol-builder -p
   hearth-client-protocol-records` passes.
3. `git -C ~/src/rust/hearth ls-tree --name-only origin/main
   crates/hearth-protocol-builder/src/translate/` lists `errors.rs`, and
   `git -C ~/src/rust/hearth grep -E
   '^[[:space:]]+Decode\(Decode\),' origin/main --
   crates/hearth-domain-render/src/errors.rs
   crates/hearth-client-protocol-records/src/errors.rs | wc -l` prints 2.
