# Session 15: templates evaluated; the file tree rendered into the cache

A task's `tree/*.j2` becomes a file of its target's `config` part (format.md,
2.3, 5.5; render.md, 3.6). 15.1 reads and compiles each template (syntax
faults are `decode`); 15.2 runs them on hearth 14's evaluator into the tree
(`evaluation` errors); 15.3 keeps each file in the cache; 15.4 bounds a
target's rendered bytes and holds the runs to a reference.

## Read first

- hearth's `format.md`: 2.3, 5.5, 7.1 to 7.3, 7.6, 7.7; `render.md`: 2.3, 3.1,
  3.6, 4.2, 7.1 to 7.3, 8.2 to 8.4, 9.1, 9.5; `store.md`: 6.5, 6.8.
- This plan's drafts, whose names this file takes: `hearth/14`, `06`, `05`,
  `11`, `12` (12.1, 12.2), `42` (42.1), `43` (43.2, 43.3), `10`, `33` (33.1), `09`.
- skein: `skein-lib`'s `stack`, `list`, `map`, `writer`, `decimal`, `sha256`;
  `programming-model.md` 3, 4.4, 6.3, 10.3; `testing-strategy.md` 2, 6.

## What the design asks

- **Files and statements** (format.md, 2.3, 5.5, 7.3, 7.6): a `.j2` name is a
  template, rendered once less its suffix, mode 0755 if git records it
  executable, else 0644; `for`, `items`, `set`, `loop`.
- **Errors, cache, bounds** (render.md, 2.3, 3.1, 3.6, 8.2): decode "fails what
  depends on the file"; a file is kept once done, origin `rendered`; template
  1 MiB, file 16 MiB, target 256 MiB.

## Choices this plan makes

- **Templates are read for the targets asked,** in 32.1's second `Read`, each
  `As::Template`, its size checked against `Limits::template` and taken from
  12.1's `revision::Budget`. A decode error is the file's: once, `task` named.
  A task no asked target has is not read, so its broken template is silent
  (render.md, 3.1, 3.3). Whitespace, nesting, 4 KiB and the bytes between
  tokens are 06's, 36's, what 15 runs (render.md, 2.3).
- **One run a `resume`,** in name and path order. A fault fails its target; its
  other templates still run, each giving its first fault (render.md, 7.1).
- **A limit met** while a template runs (iterations, a file, a target's bytes)
  is `evaluation`, `Limit`, at the template's line; a template past 1 MiB is
  `decode` (render.md, 2.3, 7.2). Messages are the Shapes' words, one an
  arm, in the forms of render.md, 7.3; the goldens bless them.
- **The context** (format.md, 7.7) is `evaluate::context::Roots`, owned per
  target and task: `target` by 14.1's `context::target(&Meta)`; `settings` and
  `tasks` moved from 13.4's `Checked`; `debian`, `exports` empty maps until
  17, 20 fill them. It replaces 14.1's fixture `Roots`.
- **`loop` is an operand of its own,** `Operand::Loop { index0, length }`: 14's
  made values have no map. Whole, it is only a condition (true) or the operand
  of `is defined`; printed, `Output`; any other use `Type` (format.md, 7.3).
  14's matches gain its arm.
- **The `Budget`'s span** is one a run: `set` values and loop iterables charge
  it, a `{{ }}` or a condition a copy; a frame restores it per iteration.
- **Two files of a task at one installed path** (`etc/motd`, `etc/motd.j2`)
  are 42.1's `Destination` (render.md, 7.2), found on the listing before any
  template is read; 15 adds no error. A task's entries are gathered in a
  `skein_lib::Map` keyed by installed path, as the suffix reorders paths
  (`etc/a-b` lists before `etc/a.j2`; `/etc/a` sorts before `/etc/a-b`).
- **Left to others:** `templates/*.j2`, `customize.sh.j2`, `software.sh.j2`,
  `copy` and links (18, 19). 16 maps an interpolated string's `Parts` with
  `instruction` and runs them with its own `interpolate`, not `run`.

## Rules that bind it

- **The meeting point** (README): every increment proves on the real loop
  (`tests/builder/tests/real.rs`); a variant added to render's `Decoded`, `As`,
  `Request`, `Event`, `errors` or the records' `Kind` extends `translate/` in
  that increment, no `_` arm.
- **No recursion, no unbounded loop in step code** (render.md, 8.3): the run is
  one `for` over `0..steps`, the instructions times (`iterations` + 1). No
  `Vec`, `String`, `_` arm, `matches!`, `if let` on render's enums, `sort_by`
  or `binary_search_by`: the tree is a `Map`, `place` uses `binary_search`.
- **Expected values are literals:** digests from `sha256sum`, positions by hand.
- **Shared files:** the root `Cargo.toml`, `Cargo.lock`; render's `src/{lib,
  vocabulary, limits, errors}.rs`, `src/tests/fixtures.rs`; the protocol's
  `src/translate/{errors, instructions}.rs`, `src/tests/{fixtures,
  translate}.rs`; the records' `src/errors.rs`; `tests/render/src/{fixtures,
  parent}.rs`, `tests/builder/src/fixtures.rs`; with 33.1, the root's
  `vocabulary.rs` and the protocol's `src/{cache, store}.rs`.

## Increments

### 15.1 Templates read: compiled, translated, refused at their line
Scope: each present task's `tree/*.j2` asked `As::Template` (render.md, 3.1,
8.2), compiled (06), translated, or `Refused` at its line and column; `decode`'s
`Syntax`; a template past 1 MiB; a `.j2` link or empty name (render.md, 7.2).
Needs: hearth 43 (`evaluate::Expression`; 43.2's `translate::operations::
expression`), 06 (`template::compile`, `template::place`, `instruction::*`,
`ErrorKind`, `Expected`), 12.2 (`errors`, `Decoded::Refused`, `Answer::Error`,
`translate/errors.rs`, 12.1's `revision::Budget`), 42.1 (`Repository::Link`,
`Name`, `Destination`), 36.2 (`ErrorKind::Interpolation`).
Files: render `src/{vocabulary, revision, call, errors, limits}.rs`,
`src/evaluate.rs`, new `src/evaluate/template.rs`, `src/tests/{revision,
fixtures}.rs`; protocol `src/{lib, read}.rs`, `src/translate.rs`, new
`src/translate/instructions.rs`, `src/translate/errors.rs`,
`src/tests/{read, translate, fixtures}.rs`; records `src/errors.rs`,
`src/form/{text, json}.rs`, `src/tests/errors.rs`; `tests/render/src/parent.rs`
(answers a template through 06's compiler); `tests/builder/{src/fixtures.rs,
tests/real.rs}`. Shape: the sketches' types, translations.
- **render:** a tree path ending in `.j2` (`revision::Class::Tree`) is a blob
  `{ oid, size, kind: As::Template }` in the task's `Read`; past `template` it
  is not asked: `TooLarge { limit: Template, max }` at its path, no position,
  failing the task's targets; else `Budget::take(size)`. A `.j2` git records
  as a link is `Repository::Link`, and a name `.j2` alone `Repository::Name`
  (42.1's), neither asked. `Decoded::Template` is dropped until 15.2;
  `Refused` becomes 12.2's error with the template's path and `task`.
- **protocol:** the blob gathered into a `Writer` (11.2);
  `template::compile(&bytes, &limits.template)`: `Ok` is `Decoded::Template(
  translate::instructions::template(t))`; `Err(e)` is `Refused { at:
  template::place(&bytes, e.at), fault: Decode::Syntax(translate::errors::
  syntax(e.kind)), also: None }`.
- **records:** a message an arm: `a tag with no end`, `not
  UTF-8`, `a carriage return`, `an unknown statement`, `a close with no open
  block`, `a block left open`, `a root of the context bound`, `an unknown
  filter`, `"join" takes 1 argument, given 2`; JSON `"decode":{"kind":
  "syntax","fault":"unclosed"}`, `Arity`'s filter by its 7.5 name; the other
  ten kinds print their name in words (`too deep`, `an unknown test`).
Test shape: protocol unit, through 11.2's `fixtures::Layer`, under 20 ms:
`a_templates_blob_reaches_render_compiled_with_its_lines`,
`a_template_that_does_not_compile_is_refused_at_its_line_and_column`;
translate (43.2's round trip; `fixtures::every_does() -> [template::Does; 7]`,
`every_syntax() -> [ErrorKind; 19]`): `each_instruction_translates_to_render_
and_back`, `each_syntax_fault_translates_to_render_and_back`,
`an_arity_fault_reaches_its_record_with_its_filters_name`; render unit:
`a_tree_file_ending_in_j2_is_asked_as_a_template`,
`a_template_past_its_limit_fails_its_tasks_targets_unread`,
`a_j2_symlink_is_refused_as_a_link_unread`,
`a_file_named_dot_j2_is_refused_by_its_name`; records:
`a_syntax_error_prints_its_path_task_and_message`,
`each_syntax_fault_prints_its_message` (a `const` table of the 19 kinds). Real loop, `BROKEN_TEMPLATE`: `etc/issue.j2` = `Host:\n{{ target.name\n` (06's `Unclosed`
at the `{`, offset 6): stdout `tasks/motd/tree/etc/issue.j2:2:1: motd: a tag
with no end\n`.
Proof: process, `a_template_that_does_not_compile_prints_its_decode_
error_at_its_line`: through real git, 06's compiler, this translation and
40.2's printer, a fault prints at its line and leaves its target out. 0.5 s.
Don't: evaluate (15.2); read `templates/` (18); hold a compiled template.
Size: at most 300 lines (a codec: within 2.5 times).
Removes: nothing.

### 15.2 Templates evaluated on the real loop; the `evaluation` error
Scope: render's run of a template's instructions (format.md, 7.3; render.md,
8.3), its context, the iteration and file bounds; each rendered file's SHA-256
in `config`'s tree; render.md 7.2's `evaluation`.
Needs: nothing beyond 15.1, and hearth 13.4 (`Checked`).
Files: render new `src/evaluate/frame.rs`,
`src/evaluate/{template, context, value, lookup, operators, filter, walk}.rs`,
`src/{call, parts, errors, limits}.rs`, `src/parts/config.rs`, new
`src/tests/templates.rs`, `src/tests/{fixtures, parts}.rs`; protocol
`src/translate/errors.rs`, `src/tests/translate.rs`; records `src/errors.rs`,
`src/form/{text, json}.rs`, `src/tests/errors.rs`; `tests/render/{src/
fixtures.rs, tests/thin.rs}`; `tests/builder/{src/fixtures.rs, tests/real.rs}`.
Shape: the sketches.
- **Text** writes its bytes; **Output** 14.2's `output` (`Written::Integer` as
  `-` and `Decimal::as_bytes`), in a `Writer` started at 4 KiB, regrown by
  doubling, past `room` (`rendered_file` here) `Limit` at the instruction.
  **Branch** skips `over` unless `truthy` (undefined is false, 7.6); **Jump**
  skips `over`; **Set** evaluates, then pushes `Binding { name, value }`; an
  undefined value is `Undefined` at the `set` (format.md, 7.6).
- **Enter's iterable:** a context list (elements borrowed), a context map (its
  keys, `None` entries skipped, each copied and charged as bound), a made
  list (each element moved out once); undefined is `Undefined`, anything else,
  a string or an integer among it, `Type`. `Names::Pair` takes each element as
  a list of exactly two, else `Type` at the `for` (format.md, 7.3). Each element
  bound counts an iteration; past `iterations`, `Limit` at the `for`. `Enter`
  keeps `outer = budget`; empty, it restores it and skips `over`; else it
  pushes a `Frame` (`base: bindings.len()`, `outer`, `iteration: budget`) on a
  `lib::Stack` of `Limits::statements`.
  `Advance` truncates bindings to `base`, restores `iteration`, binds the next
  and goes back `back`, else restores `outer` and pops.
- **`loop`'s keys:** `index` (index0 + 1), `index0`, `first`, `last`
  (index0 + 1 = length), `length`; another key is `NoKey`, or undefined where
  37.1 marked it so; `loop` outside a `for` is `Unknown`.
- **call:** each target's present tasks' templates in path order under
  `Roots`; the SHA-256 makes `TreeEntry { path: / + the path after
  tasks/<t>/tree/ less one .j2, node: File { content_digest, mode: 0o755 for
  Mode::Executable else 0o644, owner: root, group: root } }`; a fault is an
  `Error { kind: Evaluation(kind), class: Configuration, at: Some(Place { path,
  position: Some(place(lines, at)) }), also: None, target, task }`.
- **records:** `Kind::Evaluation` is 13.4's (with the messages here and
  `translate::errors::evaluation`); 15.2 adds none. A message an arm: `an unknown name`, `a
  missing key`, `an index outside the list`, `an undefined value used`, `a
  value of the wrong type`, `an integer overflow`, `an empty list or pattern`,
  `a path systemd_escape_path refuses`, `a list or a map output`, `a limit
  exceeded`; JSON `"evaluation":"no_key"`.
Test shape: `src/tests/fixtures.rs` gains 06's builders (`template`, `text`,
`output`, `branch`, `jump`, `enter`, `advance`, `set`, `one`, `pair`, `expr`),
`run` at `SHIPPED` and `run_at`. `src/tests/templates.rs`, each under 20 ms:
```rust
#[test]
fn a_loop_binds_each_element_in_turn() {
    let template = fixtures::template(&[0], [
        enter(0, one(b"t"), expr(1, [name(12, b"settings", Missing::Error), key(21, b"tags", Missing::Error)]), 3),
        output(28, expr(1, [name(31, b"t", Missing::Error)])), text(36, b","), advance(37, 3)]);
    assert_eq!(fixtures::run(&Roots::x1_prod(), &template), Ok(Box::from(&b"sudo,wheel,"[..])), "format.md, 7.3");
}
```
Then `a_loop_over_a_map_takes_its_present_keys_in_order` (`settings` skips
`wifi`), `a_pair_loop_unpacks_two_element_lists_and_refuses_others`,
`a_for_over_a_string_or_an_integer_is_a_type_fault`,
`elif_and_else_take_exactly_one_branch`,
`loop_first_index0_and_length_are_their_literals`,
`loop_outside_a_for_is_unknown`,
`loop_whole_is_a_true_condition_and_defined_but_otherwise_an_error`,
`a_tree_template_has_no_key_or_value` (`{{ key }}` is `Unknown`, 7.7),
`set_lasts_one_iteration_in_a_loop_and_to_the_end_outside`,
`the_outer_binding_returns_after_a_nested_for_ends`,
`an_undefined_set_value_fails_at_the_set`,
`an_undefined_condition_is_false_and_an_undefined_iterable_fails`,
`loop_iterations_past_the_limit_fail_at_the_for` (`iterations: 3`, beside
`SHIPPED`), `a_file_past_its_limit_fails_at_the_instruction_that_passes_it`
(`rendered_file: 4`); `src/tests/parts.rs`:
`a_template_installs_less_one_j2_with_gits_mode`, `x_j2_j2_installs_at_x_j2`,
`the_tree_orders_by_installed_path_when_the_suffix_reorders` (`etc/a-b` beside
`etc/a.j2`); translate:
`each_evaluation_fault_translates_to_its_record_and_back`; records:
`an_evaluation_error_prints_its_target_task_and_message`. Real loop,
`TEMPLATED`: `tasks/motd/tree/etc/hostname.j2` = `{{ target.name }}\n`,
`etc/issue.j2` = `{% set kind = target.placement.kind %}{{ target.base }}
runs on {{ kind }}{% if target.member is defined %} as {{ target.member }}{%
endif %}.\n{% for word in ['one', 'two'] %}{{ loop.index }}: {{ word }}\n{%
endfor %}` (no newline after a block tag: 36.1's `trim_blocks` changes
nothing; it renders `x1 runs on metal.\n1: one\n2: two\n`, 32 bytes):
`hearth_render_lists_a_rendered_template_by_its_sha256` expects the tree lines
`/etc/hostname            file 0644 root:root 21c50adeb849` and `/etc/issue
file 0644 root:root e75f92b4b841`; and
`a_missing_key_in_a_template_prints_its_evaluation_error` (`issue.j2` = `{{
target.nam }}\n`: stdout `tasks/motd/tree/etc/issue.j2:1:11: x1-prod: motd: a
missing key\n`, exit 2); and
`a_template_prints_the_default_its_setting_was_filled_with` on `GREET` (task
`greet`: string `greeting` default `hello`, `tree/etc/greeting.j2` `{{
settings.greeting }}` LF; x1-prod has it): the tree line `/etc/greeting
file 0644 root:root 5891b5b522d5` (`hello` LF), exit 0. Each 0.5 s.
Proof: process, `hearth_render_lists_a_rendered_template_by_its_
sha256`: through real git, 06's compiler, both translations and 14's
evaluator, two templates reach x1-prod's tree by the SHA-256 of what they
render. 0.5 s.
Don't: recurse; format in render.
Size: at most 300 lines.
Removes: 32.1's "templates (`.j2`) ... left out". Adds, until 15.3: a
rendered file's digest in the spec whose bytes are not in the cache.

### 15.3 Rendered files into the cache, one at a time
Scope: render.md 3.6 and 8.2's `Keep`/`Kept`: each rendered file is kept before
the next runs, written by store.md 6.8; with 33.1, entered through the store.
Needs: nothing beyond 15.2.
Files: render `src/{vocabulary, call, lib}.rs`, `src/tests/templates.rs`;
protocol `src/{lib, cache}.rs`, `src/tests/cache.rs`; `tests/render/src/
parent.rs`; `tests/builder/tests/real.rs`.
**Whichever of 15.3 and 33.1 lands second** also changes: root
`crates/hearth-domain-builder/src/{lib, vocabulary}.rs` (`Event::Produced`),
`src/tests.rs`; protocol `src/{cache, store}.rs`, `src/tests/store.rs`;
`tests/builder/tests/store.rs`. Shape: the sketches.
- **render:** a run's bytes leave in `Keep` from the `resume` that ran it; no
  run starts until its `Kept` or `Failed` (8.4). `Kept` adds the tree entry;
  `Failed` fails the target, operational, the call at least `TryAgain` (33.1).
- **protocol, before 33.1:** 32.1's cache writer: `cache/<64 random hex>.tmp`,
  bytes written and hashed as they pass (6.8), synced; a digest other than
  `Keep`'s removes it and answers `Failed { class: Operational }`; then 32.1's
  probe of `cache/<sha256>`, rename and `cache/` synced, or the temporary
  removed; `Kept { request, sha256 }`.
- **with 33.1 (the second to land):** at the temporary synced, `Event::
  Produced` (`took` from `Keep` to the sync); the root keeps `(request,
  sha256)` for `arrival` and steps the store's `Arrived { arrival, sha256,
  size, origin: Origin::Rendered { path }, took }`, then 33.1's `Keep` or
  `Discard`; `Cached` becomes `Kept`, `Refused` `Failed { class: Operational }`.
Test shape: render unit: `a_rendered_file_is_kept_before_the_next_run_starts`,
`a_failed_keep_leaves_its_target_out_and_the_call_try_again`; protocol
(`src/tests/cache.rs`, 32.1's `fixtures::Layer`, under 20 ms):
`a_kept_file_is_written_hashed_synced_then_renamed`,
`a_kept_file_already_present_removes_its_temporary`,
`a_keep_whose_bytes_are_not_its_sha256_fails_and_leaves_no_temporary`. Real
loop: the proof, and, by the second to land, in `tests/builder/tests/store.rs`:
`a_rendered_files_cache_record_names_its_templates_path` (09's
`cache_file::read` of `cache/21c50ade….record`: `origin` `Rendered { path:
b"tasks/motd/tree/etc/hostname.j2" }`, `size` 8, `took` non-zero). 0.5 s.
Proof: process, `rendered_files_enter_the_cache_by_sha256`
(`TEMPLATED`: `<store>/cache/21c50adeb849aa6df1c779b5464f84f614f2b3775ef6d2ffb
2ad0f0a1d650011` holds `x1-prod\n`, `e75f92b4b8419984111db313564627b1ee90ecb31
2dbffd66a1a42f64e6e2334` the issue's 32 bytes, each 0644, no `.tmp` left):
each rendered file is in the cache by its digest after `hearth render`. 0.5 s.
Don't: hold two rendered files; record before the rename and sync (33.1).
Size: at most 160 lines (200 for the one that lands after 33.1).
Removes: 15.2's transitional digest without its cache file.

### 15.4 A target's rendered bytes; what a run holds; generated templates
Scope: 2.3's 256 MiB for one target; `evaluate::template::worst_case`
(programming-model.md, 6.3) at the shipped limits; render's world runs
generated templates against a recursive reference (render.md, 9.1).
Needs: nothing beyond 15.3.
Files: render `src/evaluate/template.rs`, `src/{call, limits}.rs`,
`src/tests/templates.rs`; world `tests/render/`: new `src/templates.rs`,
`src/{reference, fixtures}.rs`, new `tests/templates.rs`, new
`tests/fuzzy_templates.rs`, `tests/memory.rs`; `tests/builder/{src/fixtures.rs,
tests/real.rs}`.
Shape: `Limits::rendered_target`; `call` counts each target's rendered bytes
and runs each template with `room` = the least of `rendered_file` and what the
target has left, so a fault is the instruction's that passes either.
`worst_case`: 43.3's `evaluate::worst_case`, the frames (`Stack::worst_case
(statements)`), the bindings (`template / 11 + 3 × statements`), the output at
its last regrowth (1.5 × `rendered_file`), the made values (`made`). The
world: `templates::tree(rng, &Roots, Shape)` (no `\n` after `%}` nor
whitespace before `{%`, so 36's whitespace rules strip nothing; 14's
`expressions::tree` outputs; `if`/`elif`/`else`; `for` over lists, maps and
`| items`; `set`; `loop`), `templates::text(&Tree)`, `templates::run` (06's
compile, 15.1's translation, render's `run`), `reference::run(&Tree, &Roots,
&Limits)` recursive; both give bytes or the fault's kind.
Test shape: unit `a_targets_rendered_bytes_past_the_limit_fail_its_template`;
world, under 0.5 s: `generated_templates_render_as_the_reference_renders_them`
(200 seeds from `0x0015_0000`); sweep
`generated_templates_and_contexts_render_as_the_reference_does` (2,000 rounds
at `fixtures::limits(rng)`; its time is `fuzzy_templates`'s nextest line under
`--profile fuzzy`, aim 2 s); `tests/memory.rs` (43.3's `Meter`
loop): `running_templates_at_the_limits_holds_no_more_than_the_worst_case`
(`fixtures::templates_at_the_limits()`: 32 nested `for`s, 1 MiB of text output
16 times, a `set` a line, a `~` chain past `made`), under 1 s.
Proof: process, `a_target_past_its_rendered_bytes_prints_a_limit_
error`, by `fixtures::render_with(&Repository, &Limits)` (made here in
`tests/builder/src/fixtures.rs` if absent; 23.2 uses it) at `hearth::limits()`
with `rendered_target: 8`: `TEMPLATED`'s `hostname` renders its 8 bytes,
`issue`'s first output passes them: stdout `tasks/motd/tree/etc/issue.j2:1:39:
x1-prod: motd: a limit exceeded\n`, exit 2. Bound 0.5 s.
Don't: raise a limit to fit a test; count a failed target's bytes elsewhere.
Size: at most 60 lines.
Removes: nothing.

## Interface sketches

render mirrors hearth-protocol-template's types variant for variant. Types
derive `Clone, PartialEq, Eq, Hash, Debug` (`Copy` where they can).

```rust
// render limits.rs: `Limits` gains `template: u32` (15.1); `statements`, `iterations`,
// `rendered_file: u32` (15.2); `rendered_target: u64` (15.4). `SHIPPED`: `template: 1_048_576,
// statements: 32, iterations: 1_048_576, rendered_file: 16_777_216, rendered_target: 268_435_456`.
// render evaluate/template.rs (types 15.1; run, place 15.2; worst_case 15.4)
pub struct Template { pub instructions: Box<[Instruction]>, pub lines: Box<[u32]> } // run from the first
pub struct Instruction { pub at: u32, pub does: Does }                              // `at` an offset
pub enum Does {
    Text(Box<[u8]>),                                         // writes its bytes
    Output(Expression),                                      // writes the value (format.md, 7.4)
    Branch { condition: Expression, over: u32 },             // skips `over` when false or undefined
    Jump { over: u32 },                                      // skips `over`, to after `endif`
    Enter { names: Names, iterable: Expression, over: u32 }, // binds the first; none: skips past `Advance`
    Advance { back: u32 },                                   // binds the next, goes back; else drops the frame
    Set { name: Box<[u8]>, value: Expression },              // binds until the iteration or the run ends
}
pub enum Names { One { name: Box<[u8]> }, Pair { first: Box<[u8]>, second: Box<[u8]> } }
pub fn run<'c>(template: &'c Template, context: &Context<'c>, limits: &Limits, room: u64) -> Result<Box<[u8]>, Fault>; // at most `room` bytes, or the first fault
#[must_use] pub fn place(lines: &[u32], at: u32) -> Position;  // line, column from 1 in bytes (06's `place`)
#[must_use] pub fn worst_case(limits: &Limits) -> Option<u64>; // the most one `run` holds; `None` past a u64
// render evaluate/value.rs (15.2): 14.1's `Operand` gains `Loop { index0: u32, length: u32 }`
// render evaluate/context.rs (15.2): one task's roots, owned; `settings`, `tasks` from 13.4's `Checked`; `debian` (17), `exports` (20) start empty
pub struct Roots { pub target: Value, pub debian: Value, pub settings: Value, pub tasks: Value, pub exports: Value }
impl Roots { #[must_use] pub fn context(&self) -> Context<'_>; }
// render vocabulary.rs: 11's and 12's enums gain
pub enum As { /* … */ Template }                         // 15.1: render.md 8.2's `template`
pub enum Decoded { /* … */ Template(Template) }          // 15.1
pub enum Request { /* … */                               // 15.3: a rendered file for the cache; `path` its template's (store.md, 6.5)
    Keep { request: Token, path: Box<[u8]>, sha256: Digest, bytes: Box<[u8]> } }
pub enum Event { /* … */ Kept { request: Token, sha256: Digest } } // 15.3: `Keep`'s terminal, as `cache/<sha256>`
// render errors.rs: 12's enums gain
pub enum Kind { /* … */ Evaluation(FaultKind) } // 13.4's (15.2 uses it): render.md 7.2's `evaluation`
pub enum Decode { /* … */ Syntax(Syntax) }      // 15.1 (16: an expression's)
pub enum Limit { /* … */ Template }             // 15.1
pub enum Syntax {
    TooLong, Character, Unterminated, Escape, Integer, Expected(Expected), TooDeep, Misplaced,
    UnknownFilter, Arity { filter: Filter, given: u32 }, UnknownTest,
    Utf8, Return, Unclosed, UnknownStatement, Unmatched, Unended, Shadows, Interpolation,
}
pub enum Expected { Operand, Operator, Name, CloseParen, CloseBracket, In, Else, Statement, Assign, TagEnd }
// records errors.rs: 40.2's and 12's enums gain, as render's
pub enum Kind { /* … */ Evaluation(Evaluation) }  // 13.4's
pub enum Decode { /* … */ Syntax(Syntax) }        // 15.1; `Limit` gains `Template`
pub enum Evaluation { Unknown, NoKey, NoIndex, Undefined, Type, Overflow, Empty, Path, Output, Limit } // mirrors render's `FaultKind`
// `Syntax`, `Expected` mirror render's, but `Arity { filter: Box<[u8]>, given: u32 }`: the filter's 7.5 name.
// protocol translate/instructions.rs (15.1); reverses in src/tests/translate.rs
#[must_use] pub fn template(template: template::instruction::Template) -> render::evaluate::template::Template;
#[must_use] pub fn instruction(instruction: template::instruction::Instruction) -> render::evaluate::template::Instruction;
#[must_use] pub fn does(does: template::instruction::Does) -> render::evaluate::template::Does;
#[must_use] pub fn names(names: template::instruction::Names) -> render::evaluate::template::Names;
// protocol translate/errors.rs: 12's `error` gains its arms, and
#[must_use] pub fn syntax(kind: template::error::ErrorKind) -> render::errors::Syntax;   // 15.1
#[must_use] pub fn evaluation(kind: render::evaluate::FaultKind) -> records::Evaluation; // 13.4's
// protocol lib.rs: 11.2's `Limits` gains `template: template::limits::Limits` (15.1).
// root vocabulary.rs: by whichever of 15.3 and 33.1 lands second; `Cached` becomes `Kept`, `Refused` `Failed`
pub enum Event { /* … */
    Produced { request: Token, path: Box<[u8]>, arrival: Token, sha256: Digest, size: u64, took: Duration } }
```

## Questions to raise when reached

None.

## Done when

1. `git log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-15\.[1-4]'` prints 4.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run -p hearth-domain-render -p
   hearth-protocol-builder -p hearth-client-protocol-records -p hearth-render-world -p
   hearth-builder-world` passes, and `heavy cargo nextest run -p hearth-render-world --profile fuzzy`
   passes.
3. `git -C ~/src/rust/hearth ls-tree --name-only origin/main
   crates/hearth-protocol-builder/src/translate/` lists `instructions.rs`.
4. `git -C ~/src/rust/hearth grep -n -E 'rendered_target: 268_435_456|iterations: 1_048_576'
   origin/main -- crates/hearth-domain-render/src/limits.rs | wc -l` prints 2.
5. `git -C ~/src/rust/hearth grep -n -i minijinja origin/main -- crates/hearth-domain-render
   tests/render/src` prints nothing.
