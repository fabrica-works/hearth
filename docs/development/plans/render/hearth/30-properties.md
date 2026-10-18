# Session 30: render's properties, the MiniJinja oracle, the fuzzed decoders

render.md, 9.3 to 9.5 and 9.8, over what sessions 11 to 29 built. 30.1:
generated repositories, determinism; 30.2: locality, order, mutation; 30.3:
RFC 7386's laws, part digests; 30.4, 30.5: the oracle, MiniJinja 2.20.0
against hearth's expressions, statements and whitespace, every difference
listed; 30.6: each decoder has its sweep. The joint worlds and budgets are
hearth 47 (decisions.md, 20).

## Read first

- hearth's `render.md`: 3.2, 3.10, 6.2, 7.2, 8.3, 9.1, 9.3 to 9.5, 9.8;
  `format.md`: 2.2, 4.1, 7.2 to 7.6, 8.2, 8.3; `testing.md`: 2.4, 9.1, 9.3.
- This plan: `hearth/29-goldens.md`, `11`, `12`, `14`, `15`, `20`, `22`, `36`,
  `28`, `23`, `32`, `02`; the sweeps of `tests/{config,template,git,mmdebstrap,
  tree}` (03, 35, 05, 37, 06, 02, 34, 07, 38, 39); decisions.md 10, 15, 20.
  MiniJinja 2.20.0: `src/{utils,filters}.rs`, `src/value/ops.rs`,
  `src/compiler/{parser,lexer}.rs`.

## What the design asks

- **9.3:** "one revision renders to the same digests in every world, whatever
  the seed's order of blob arrival"; RFC 7386's laws, locality, order,
  digests; "valid by construction and then mutated". **9.4:** "Every decoder
  is fuzzed on its own". **9.5:** MiniJinja 2 with `trim_blocks`,
  `lstrip_blocks`, `keep_trailing_newline`, strict undefined; differences
  "listed". **9.8, 3.10:** a revision "decoded once".

## Choices this plan makes

1. **One generator,** `tests/render/src/repositories.rs`: bases `b0` to
   `b2` (`Shape::SMALL`), environments `prod` and `lab`, so six targets;
   29.3's `fleet.json` `debian` and `tasks/hearth/task.json`; one task `node`
   with settings `mtu` (an int, read by `tree/etc/node/mtu.j2`), `name`
   (exported as `address`, read by `peers.j2`, as 20.3's `VPN`) and `unread`
   (read by nothing). Documents are a `Json` model printed by `json::print`,
   which draws member order, whitespace and `_note`s.
2. **Expected sets come from the model,** never from render (render.md 9.3,
   Locality): `touched` says `Own` (`mtu` in one base-env file) changes that
   target alone, `Exported` (`name`) the exporting target and the targets of
   its environment that read it (all do: `peers.j2`), `Task` every target,
   `Env` the environment's, `Unread` none.
3. **Determinism in four worlds** (render.md 9.3): render's world over seeds
   (the pace of answers), the protocol world's cut pipes (28.3), the simulated
   client (28.4), real git (11.3), by the whole digest the JSON form prints.
4. **Part digests:** per part name, records are equal iff digests are.
5. **The oracle is one engine file,** `tests/render/tests/oracle/engine.rs`,
   which `oracle.rs` and `fuzzy_oracle.rs` include by `#[path]` (a sweep must
   be a `fuzzy_*` binary). `minijinja` is `tests/render`'s dev-dependency,
   `=2.20.0`, with `json`. Settings: `set_trim_blocks`, `set_lstrip_blocks`,
   `set_keep_trailing_newline`, `UndefinedBehavior::Strict`, auto-escape off.
   Registered, from format.md 7.5: `keys`, `values`, `shell_quote`,
   `systemd_escape`, `systemd_escape_path`, `basename`, `dirname` (7.5's
   edges: `/` gives `/`; `dirname` of `/a` is `/`, of one relative segment
   `.`; any other value fails); the others are builtins. Equal bytes pass, two failures pass, nothing else.
6. **The listed differences** (`DIFFERENCES`: template, both outcomes as
   literals, reason; `each_listed_difference_still_differs_as_listed` runs
   both engines), hearth first: (1) `sort` of strings by bytes, against case
   folding; (2) `'a' + 'b'`, `true ~ 'x'`: `Type`, against joining; (3) `1 ==
   true`: false, against true; (4) `first`, `last` of `[]`: `Empty`, against
   undefined, and of a string, `Type`, against a character; (5) `tojson` of
   `<>&'`: as written, against four escapes (a backslash, `u` and four hex
   digits); (6) `~` below `+ -` (7.4), against above: `'v' ~ 1 + 1`; (7) `is`
   below the comparisons, against at the filters' level: `a == b is defined`;
   (8) a chain into `is defined` or `default` with a missing link before its
   last: undefined at the first (7.6), against a failure at the second
   lookup; (9) an absent optional as a condition (`if`, `elif`, `not`, `and`,
   `or`, `... if ... else`): false (7.6), against a failure under Strict;
   (10) `+ -` past i64: `Overflow`, against i128; (11) a list or map
   printed: `Output`, against its text; (12) `a[-1]`, a key into a list, an
   integer into a map: `NoIndex`, `Type`, against counting from the end,
   undefined; (13) `split('')`, `replace('', x)`: `Empty`, against the
   engine's. Rows 1 to 5 are hearth 14's and 43's, 6 and 7 render.md 9.5's. A
   difference the first run finds is added with its reason, or avoided.
7. **The generator avoids them by construction:** expressions are typed
   (operands both engines accept; output only of scalars, the rest through
   `join` or `tojson`); strings have no `<`, `>`, `&` or `'`; integers stay
   within 2^40; `~` beside `+ -`, and an operand of `is` that is not a
   lookup chain, are parenthesized (`Style::Oracle`); an absent optional
   appears only under `is defined` or `default`, a missing link only as a
   chain's last.
8. **Decoders:** a `const` table of the sweeps; one with no `fuzzy_*` file
   fails it. Store names and argument vectors are swept in the builder's world
   (render.md 9.4).

## Rules that bind it

- **No expected value computed by the code under test:** `touched` is the
  model's; `reference::merge` is format.md 4.1's pseudocode; the oracle is
  the other engine; a difference's outcomes are literals.
- **`minijinja` appears in `tests/render/Cargo.toml`, `Cargo.lock` and
  `engine.rs` only** (code-organization.md 6).
- **Shared files:** root `Cargo.toml`, `Cargo.lock`; `tests/render/{Cargo.toml,
  src/{lib, fixtures, expressions, templates, reference}.rs}`;
  `tests/builder/src/fixtures.rs`. Seeds from `0x0030_0000`.
- **Test time** (decisions.md 15; a Test shape's bound is for its whole line).
  Focused 3.5 s: 30.1 1, 30.2 1, 30.3 0.5, 30.4 0.5, 30.5 0.5 (30.6 20 ms);
  with hearth 47's 1 s, 4.5 s. Fuzzy 8 s: 30.1 1.5, 30.3 1, 30.5 3, 30.6 2.5.

## Increments

### 30.1 The generated repository; representation and determinism
Scope: `repositories`, `json`; render.md 9.3's first two properties.
Needs: hearth 29 (whole; 29.1's `golden::Leaf`, `fixtures::files_of`, the
world's play of leaves through `parent::run`), hearth 22.1 (errors sorted).
Files: new `tests/render/src/{repositories, json}.rs`, `src/{lib, fixtures}.rs`,
new `tests/render/tests/{properties, fuzzy_properties}.rs`; new `tests/builder/
tests/properties.rs`, `src/fixtures.rs` (`digests_of(&str) -> Vec<(String,
String)>`: each `spec` record's target and digest, client/README.md 3.2).
Shape: the sketches. `generate` draws values, `_note`s and documents' text;
`edit(Note | Reorder | Respace)` reprints with another order, whitespace or
note and nothing else; `Collide` is 30.2's. `fixtures::repository_of(&[Leaf])`
plays leaves as the world's repository, factored out of 29.1's
`Golden::repository` (`tests/render/src/golden/repository.rs:16`), which
calls it (`fixtures::files_of` is the builder world's, `Vec<File>`: ruling 190); `fixtures::digests(&Run)` is its
`spec` digests by target.
Test shape: world, 12 repositories, under 0.5 s:
`a_generated_repository_renders_without_errors`,
`a_reordered_object_changes_no_digest`,
`reformatted_whitespace_changes_no_digest`, `an_edited_note_changes_no_digest`,
`every_seed_gives_the_same_digests`; process, under 0.5 s:
`a_generated_repository_renders_the_worlds_digests_through_real_git` (two,
`Form::JsonLines`, `form.rs:12`: ruling 190). The sweep, `fuzzy_properties.rs`:
`generated_repositories_and_edits_keep_every_property` (750 rounds, about
1.5 s; each edit's changed set is `touched`'s; 30.2's edits and `mutate` join it).
```rust
// tests/render/tests/properties.rs
#[test]
fn every_seed_gives_the_same_digests() {
    for case in 0..12_u64 {
        let generated = repositories::generate(&mut Rng::new(0x0030_0000 + case), Shape::SMALL);
        let run = |seed| fixtures::digests(&parent::run(&fixtures::repository_of(&generated.leaves), &[], seed));
        for seed in 1..8 { assert_eq!(run(seed), run(0), "render.md, 9.3: case {case}, seed {seed}"); }
    }
}
```
Proof: world, `every_seed_gives_the_same_digests`: whatever the pace of
answers, one digest a target; process, the real-git test: the same digests
printed whole. Bound: 0.5 s; 0.5 s.
Don't: compute a digest in the generator; read `expected/`.
Size: no counted lines; the world about 280 lines. Removes: nothing.

### 30.2 Locality and order
Scope: 9.3's locality, order and mutation; `Edit::{Unread, Own, Exported,
Task, Env, Collide}`, `repositories::mutate`.
Needs: hearth 20.3 (`exports.<task>` across an environment), hearth 22.3
(collisions), hearth 30.1.
Files: `tests/render/src/repositories.rs`, `tests/render/tests/{properties,
fuzzy_properties}.rs`, `tests/builder/tests/properties.rs`.
Shape: `touched(generated, edit)`: `Own` the target; `Exported` every target
of its environment (its readers, itself included: 8.2); `Task` (`node`'s
`task.json` or tree) every target that has it; `Env` (`envs/<env>.json`)
every target of that environment; `Unread` and 30.1's three none.
`mutate(rng, &Generated) -> Generated` breaks one file of one base by
`generate::mutate`; the healthy targets are the other bases'. `Collide` adds
tasks `dup-a` and `dup-b`, each with a tree file `/etc/dup` (22.3's error
names both: `dup-a` at `at`, `dup-b` at `also`, render.md 9.3, Order). Order: `targets` reversed and shuffled; arrival is the seed's.
Test shape: world, 12 repositories, under 0.5 s:
`a_setting_only_its_own_template_reads_changes_its_target_alone`,
`an_unread_setting_changes_no_digest`,
`an_exported_setting_changes_every_target_of_its_environment`,
`a_production_edit_never_changes_a_lab_spec` (the three edits on `-prod`),
`a_task_edit_changes_every_target_that_has_it`,
`a_lab_environment_edit_leaves_every_production_spec` (render.md 6.2),
`a_mutated_repository_gives_sorted_bounded_errors_and_renders_its_healthy_targets`
(no panic; 22.1; healthy digests as before),
`targets_asked_in_any_order_give_the_same_specs`,
`a_collision_names_the_same_pair_whichever_task_arrives_first` (errors equal
over 8 seeds, sorted, 22.1); process, under 0.5 s:
`a_production_edit_leaves_the_lab_specs_through_real_git` (two repositories).
Proof: world, `a_setting_only_its_own_template_reads_changes_its_target_alone`:
the digests that differ after the edit are `touched`'s; process, the
production test. Bound: 0.5 s; 0.5 s.
Don't: edit a base file in a "production" edit (`b.json` is both's).
Size: no counted lines; about 120 lines of world. Removes: nothing.

### 30.3 RFC 7386's laws and the part digests
Scope: 9.3's laws and digests: `compose::Merge` on generated documents.
Needs: hearth 12.1 (`Merge`, `Composed`), hearth 30.1 (`json`).
Files: `tests/render/src/reference.rs` (gains `merge`); new `tests/render/
tests/laws.rs`; `tests/render/tests/fuzzy_properties.rs`.
Shape: `json::generate(rng, depth)` has `Null` at objects' members (never in
lists); `json::value(&Json) -> revision::Value` (positions from a counter);
`json::plain(&Composed) -> Json`. `reference::merge(Option<Json>, &Json)
-> Option<Json>` is format.md 4.1's pseudocode, recursive; a target is
`merge(None, document)`, patches objects, scalars, lists and `{}`.
`fixtures::merged(&[&Json]) -> Option<Json>` folds patches through render's
`Merge` and `json::plain`. The empty patch's targets are objects: `{}`
over a scalar or a list gives `{}` (format.md 4.1: "result = target if
target is an object, else {}"), which `merge_equals_format_md_4_1s_pseudocode`
covers (ruling 190).
Test shape: world, 200 seeds each, under 0.5 s:
`an_empty_patch_leaves_the_target_as_it_was`,
`merging_a_patch_twice_is_merging_it_once`,
`a_scalar_or_list_patch_replaces_whole_and_an_absent_key_keeps_its_value`,
`no_merge_result_holds_null`, `merge_equals_format_md_4_1s_pseudocode`,
`parts_with_different_records_have_different_digests` (200 generated runs'
parts by name: `record == record` iff `digest == digest`, every pair); the
sweep: `generated_patches_obey_the_laws_and_the_reference` (13,000 rounds,
depths to 8, about 1 s).
```rust
// tests/render/tests/laws.rs
#[test]
fn merging_a_patch_twice_is_merging_it_once() {
    for seed in 0..200_u64 {
        let mut rng = Rng::new(0x0030_3000 + seed);
        let (t, p) = (json::generate(&mut rng, 6), json::generate(&mut rng, 6));
        assert_eq!(fixtures::merged(&[&t, &p, &p]), fixtures::merged(&[&t, &p]), "seed {seed}");
    }
}
```
Proof: world, `merge_equals_format_md_4_1s_pseudocode`: render's `Merge` gives
the reference's value at every depth drawn. Bound: 0.5 s. Not process: 12.1's
real tests reach `Merge`; the laws need generated patches.
Don't: compare provenance, as `Composed`'s derived `PartialEq`
(`compose.rs:204`) does, its `from: Provenance` included (ruling 190).
Size: no counted lines; the world about 140 lines. Removes: nothing.

### 30.4 The oracle: expressions, and every difference listed
Scope: 9.5 for `{{ e }}` and conditions; the engine; `DIFFERENCES`.
Needs: hearth 15.4 (`templates::run`, the world's `Roots`, `expressions::
{roots, tree}`), hearth 30.3.
Files: `tests/render/Cargo.toml` (dev-dependency), root `Cargo.lock`;
new `tests/render/tests/oracle.rs`, `tests/render/tests/oracle/engine.rs`;
`tests/render/src/{expressions, templates, declarations}.rs` (`Style`,
`ORACLE`; `expressions::text` returns `Vec<u8>` on main, and its callers at
`declarations.rs:53, 117, 152, 155`, `templates.rs:75, 87, 93, 104, 110` and
`expressions.rs:318-326` take the new `String`: ruling 190).
Shape: the sketches. `context_of(&Roots)`, `Roots.bindings` included
(`expressions.rs:22`): maps with their present entries
only (`None` is absent), integers, strings, booleans, lists.
`expressions::text(&Tree, Style) -> String` writes 7.4 as 6 and 7 of the
Choices need; `templates::output_of(&str) -> String` makes a case of it:
`{{ e }}` for a scalar `e`, or `{% if e %}x{% else %}y{% endif %}`;
`fixtures::draw(seed) -> (Rng, Roots)`. The first landing's `Cargo.lock` pins
minijinja-2.20.0, memo-map-0.3.4 and serde_json-1.0.154 (and serde, itoa,
memchr, zmij) from the registry (README, Live services: one `heavy cargo
fetch`).
Test shape: world, under 0.5 s:
`generated_expressions_render_as_the_oracle_renders_them` (200 seeds),
`each_listed_difference_still_differs_as_listed` (`DIFFERENCES`, one
assertion shape), `every_filter_of_7_5_agrees_on_a_pinned_input` (17 rows; `basename` and
`dirname` at 7.5's edges),
`the_oracles_settings_are_the_ones_9_5_names` (a tag alone on its line leaves
nothing; a final newline stays; a missing key fails),
`a_difference_not_listed_fails_naming_both_outputs`.
```rust
// tests/render/tests/oracle.rs (`mod engine` by `#[path]`, Choices 5)
#[test]
fn generated_expressions_render_as_the_oracle_renders_them() {
    let oracle = engine::Oracle::new();
    for seed in 0..200_u64 {
        let (mut rng, roots) = fixtures::draw(0x0030_4000 + seed);
        let tree = expressions::tree(&mut rng, &roots, expressions::ORACLE);
        let template = templates::output_of(&expressions::text(&tree, Style::Oracle));
        if let Err(mismatch) = oracle.compare(&template, &roots) { panic!("seed {seed}: {mismatch}"); }
    }
}
```
Proof: world, that test: 200 generated expressions give MiniJinja's bytes, or
both fail. Not process: the oracle compares engines, without git. Bound: 0.5 s.
Don't: register a filter to hide a difference; name the engine in `src/`.
Size: no counted lines; `engine.rs` about 300 lines. Removes: nothing.

### 30.5 The oracle: statements, whitespace and `raw`
Scope: 9.5 and format.md 7.2, 7.3: `if`, `elif`, `for`, `set`, `loop`,
comments, markers, `raw`; the sweep.
Needs: hearth 36.1 (whitespace, `raw`), hearth 15.4, hearth 30.4.
Files: new `tests/render/src/spacing.rs`, `src/templates.rs`;
`tests/render/tests/oracle.rs`, new `tests/render/tests/fuzzy_oracle.rs`.
Shape: `templates::tree` (15.4's) draws the statements; `spacing::dress`
(`dressed`'s core, over a `Text`) puts spaces, tabs, `\n`, VT, FF, U+00A0 and U+3000 around and
inside tags, `-` on any delimiter, `+` on `{%` and `%}` (7.2's), comments
holding `{{` and quotes, and `raw` bodies holding `{{`, `{%` and `{%endraw`.
Names come from a pool outside the roots; no `\r`. `spacing::dressed(rng,
&Tree) -> String` is `templates::text` dressed; `templates::constructs(&Tree)`
lists the statements, markers and filters a tree uses, `CONSTRUCTS` all.
Test shape: world, 200 seeds each, under 0.5 s in all:
`generated_templates_render_as_the_oracle_renders_them`,
`markers_on_every_delimiter_render_as_the_oracle_renders_them`,
`raw_bodies_render_as_the_oracle_renders_them`,
`loops_and_assignments_scope_as_the_oracle_scopes_them` (nested `for`,
`set` in and after, `loop.*`),
`comments_and_trim_blocks_render_as_the_oracle_renders_them`. The sweep,
`fuzzy_oracle.rs`, each starting with the differences check:
`generated_expressions_at_wide_shapes_render_as_the_oracle_renders_them` and
`generated_templates_at_wide_shapes_render_as_the_oracle_renders_them`
(3,750 rounds each, about 1.5 s each, `assert_fell` of `CONSTRUCTS`).
```rust
// tests/render/tests/fuzzy_oracle.rs (`mod engine` by `#[path]`)
#[test]
fn generated_templates_at_wide_shapes_render_as_the_oracle_renders_them() {
    let oracle = engine::Oracle::new();
    oracle.check_differences().expect("each listed difference still differs");
    let mut drawn = BTreeSet::new();
    for seed in 0..3_750_u64 {
        let (mut rng, roots) = fixtures::draw(0x0030_5000 + seed);
        let tree = templates::tree(&mut rng, &roots, templates::WIDE); drawn.extend(templates::constructs(&tree));
        if let Err(mismatch) = oracle.compare(&spacing::dressed(&mut rng, &tree), &roots) { panic!("seed {seed}: {mismatch}"); }
    }
    fixtures::assert_fell(&drawn, templates::CONSTRUCTS);
}
```
Proof: world, `generated_templates_render_as_the_oracle_renders_them`: 200
dressed templates give MiniJinja's bytes. Not process (30.4). Bound: 0.5 s.
Don't: strip whitespace in the generator; mutate bytes (06's sweeps do).
Size: no counted lines; the generator about 160 lines. Removes: nothing.

### 30.6 Every decoder fuzzed: the audit, the goldens, the rest
Scope: 9.4 and decisions.md 10: an inventory of the sweeps, the goldens as
corpora, and the decoders with no sweep: `rev_parse::decode` and
`invocation::revision` (02.1), `names::shape` (23.2; `pub(crate)` at
`hearth-protocol-builder/src/store/names.rs:7`, the builder world's copy at
`tests/builder/src/domain/store.rs:226`: ruling 190), `argv::{form,
invocation}` (32.2).
Needs: hearth 02, 23.2, 32.2, 38.2 (the recordings corpus), 39 (`fuzzy_tree`),
hearth 29.
Files: new `tests/render/tests/decoders.rs`; new `tests/config/tests/
fuzzy_goldens.rs`, `tests/template/tests/fuzzy_goldens.rs`, `tests/git/tests/
fuzzy_revisions.rs`, `tests/builder/tests/fuzzy_inputs.rs`; owners'
`pub` for what a sweep calls, at most 20 lines (`names`, `argv`).
Shape: `ROWS` in `decoders.rs`: decoder, world, `fuzzy_*` file, corpus
(`golden`, `recordings` or none; 9.4), session: JSON documents (`config`,
`fuzzy_documents`, 03.2); declarations (`config`, `fuzzy_declarations`,
35.2); expressions (`template`, `fuzzy_expressions`, 05, 37); templates,
interpolated strings (`template`, `fuzzy_templates`, 06, 36); `ls-tree`,
`cat-file`, hunks (`git`, `fuzzy_git`, 02, 34); `Inst`, `Packages`, tails,
recordings (`mmdebstrap`, `fuzzy_mmdebstrap`, 07, 38); frames of kinds 1, 4,
5 (`tree`, `fuzzy_tree`, 39); and this session's four. The goldens: each
world reads `../render/<name>/` (`.json` outside `expected/` and
`rendered/`; `*.j2`) and feeds its `generate::mutate`, its `generate::cut`
where it has one (template; config has `mutate` alone,
`tests/config/src/generate.rs:56`: ruling 190), and its reference.
Revisions: `rev_parse::decode` on generated, cut, mutated output against a
recursive reading of 02.1's rules; `revision` on bytes in and out of the
alphabet. Names: 23.2's shapes, mutated by a byte, against a table. argv:
drawn calls round-trip; mutated argv gives `Ok` or `Startup`, in `form`'s form.
Test shape: world, under 20 ms in all:
`every_decoder_has_a_seeded_sweep_that_exists` (each row's file is in its
world, starts `fuzzy_` and holds a `#[test]`),
`every_fuzzy_file_is_a_row_or_a_domain_reference` (`tests/render`'s own are
references, exempt), `every_machine_crate_has_a_world_with_a_sweep` (the
records' crate parses nothing: hearth 10; the builder's inputs are rows),
`every_sweep_names_the_corpus_its_row_says` (its file names `golden::` or
`recordings::`, as `fuzzy_mmdebstrap.rs:36` reads
`hearth_mmdebstrap_world::recordings::all()`: ruling 190; the git and mmdebstrap rows say `recordings`). Sweeps
(about 0.5 s each):
`golden_documents_cut_and_mutated_read_as_the_reference_reads_them`,
`golden_templates_cut_and_mutated_compile_as_the_reference_compiles_them`,
`revisions_and_rev_parse_answers_cut_and_mutated_decode_as_the_reference_does`,
`store_names_mutated_by_a_byte_shape_as_the_table_says`,
`argv_drawn_and_mutated_gives_an_invocation_or_the_form_json_asks`.
```rust
// tests/render/tests/decoders.rs
#[test]
fn every_decoder_has_a_seeded_sweep_that_exists() {
    for row in ROWS {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(row.world).join("tests").join(row.file);
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(row.file.starts_with("fuzzy_") && source.contains("#[test]"), "{}: no sweep at {}", row.decoder, path.display());
    }
}
```
Proof: world, `every_decoder_has_a_seeded_sweep_that_exists`: 9.4's "every
decoder" is a table that fails when a sweep goes. Not process: it reads
source files. Bound: 20 ms.
Don't: add a `cargo-fuzz` target (decisions.md 10).
Size: no counted lines; at most 20 in owners. Removes: nothing.

## Interface sketches

```rust
// ordinary Rust (test code): `Vec`, `String` and `std` allowed
// tests/render/src/repositories.rs (30.1; Edit 30.2)
/// How many bases a repository has; `Shape::SMALL` is three (Choices, 1).
pub struct Shape { pub bases: u32 }
/// A repository's files (29.1's `Leaf`) and the targets the generator made.
pub struct Generated { pub leaves: Vec<golden::Leaf>, pub model: Model }
pub struct Model { /* each target and its environment */ }
/// A commit's change: the first three change no digest, the next five are 9.3's, `Collide` adds an error.
pub enum Edit { Note, Reorder, Respace, Unread(String), Own(String), Exported(String), Task, Env(String), Collide }
#[must_use] pub fn generate(rng: &mut Rng, shape: Shape) -> Generated;
#[must_use] pub fn edit(rng: &mut Rng, generated: &Generated, edit: &Edit) -> Generated;
/// The targets whose digests the edit must change, from the model alone.
#[must_use] pub fn touched(generated: &Generated, edit: &Edit) -> Vec<String>;
// tests/render/src/json.rs (30.1; merge 30.3)
/// A document as the generator holds it, `Null` allowed in objects; `print`: drawn member order, whitespace, `_note`s.
#[derive(Clone, PartialEq, Eq, Debug)] pub enum Json { Null, Bool(bool), Int(i64), String(String), List(Vec<Json>), Map(Vec<(String, Json)>) }
#[must_use] pub fn print(rng: &mut Rng, json: &Json) -> Vec<u8>;
// tests/render/src/{expressions, templates}.rs (14, 15 own; 30.4 adds)
/// The printer's parentheses: 7.4's grammar, or the oracle's (Choices, 7).
pub enum Style { Hearth, Oracle }
// tests/render/tests/oracle/engine.rs (30.4): the one file naming the engine
/// What one engine gave: its bytes (borrowed in `DIFFERENCES`), or that it failed; messages are not compared.
#[derive(Clone, PartialEq, Eq, Debug)] pub enum Outcome { Text(Cow<'static, str>), Fails }
/// One listed difference (Choices, 6).
pub struct Difference { pub id: &'static str, pub template: &'static str, pub hearth: Outcome, pub oracle: Outcome, pub reason: &'static str }
pub const DIFFERENCES: &[Difference];
pub struct Oracle { /* 9.5's environment */ }
impl Default for Oracle { fn default() -> Oracle; } // `new`'s, as clippy's new_without_default asks
impl Oracle { // `compare`: both engines on one case; `Err` names the template, roots, both outcomes
    #[must_use] pub fn new() -> Oracle;
    pub fn compare(&self, template: &str, roots: &Roots) -> Result<(), Mismatch>;
    pub fn check_differences(&self) -> Result<(), Mismatch>; // each `DIFFERENCES` row still differs
}
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-30\.[1-6]'` prints 6.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run
   -p hearth-render-world -p hearth-config-world -p hearth-template-world
   -p hearth-git-world -p hearth-builder-world` passes, and `heavy cargo nextest run --workspace --profile fuzzy -E 'binary(/^fuzzy_(oracle|properties|goldens|revisions|inputs)$/)'` passes, its `Summary` naming 9
   tests.
3. `git -C ~/src/rust/hearth grep -l -i minijinja origin/main -- crates testing tests ':!tests/render/tests/oracle/engine.rs' ':!tests/render/Cargo.toml' | wc -l` prints 0, and `git -C ~/src/rust/hearth grep -c '=2.20.0' origin/main -- tests/render/Cargo.toml | awk -F: '{print $NF}'` prints 1.
