# Session 43: the filters, hearth 05's operations translated, what one evaluation holds

hearth 14 evaluates lookups and operators; this session finishes
`evaluate`. 43.1 adds the 17 filters (format.md 7.5); 43.2 translates
hearth 05's operations into render's, exhaustive both ways, and sends the
world's expressions through the real compiler; 43.3 bounds what one
evaluation holds. Needs hearth 14; 43.2 also hearth 37.2
(`expression::compile`), 43.3 hearth 37.3. Nothing in the thin story evaluates an
expression: each increment proves in render's domain world against hearth
14's reference evaluator, and hearth 15.2 connects the evaluator to the
real loop. hearth 14's Choices and Rules bind here as there.

## Read first

- hearth's `format.md`: 7.5, 7.8, 6.2; `render.md`: 8.3, 8.4, 9.5.
- This plan: `hearth/14-evaluate.md` whole; `hearth/05-expressions.md`'s
  sketch; `hearth/37-expression-operators.md`'s 37.1, 37.2 and sketch;
  `findings/skein-testing.md`, 1, 2, 7.
- skein at `f981746`: `crates/skein-lib/src/list.rs` (`with_capacity`,
  `into_boxed`:104), `bytes.rs` (`find_from`:40);
  `crates/skein-json/src/writer.rs:530` (`escaped`);
  `tests/json/tests/memory.rs:134`. `programming-model.md`: 6.3, 8, 10.3;
  `testing-strategy.md`: 2.4, 3, 6 (by number only).

## What the design asks

- **Filters** (7.5): 17, each with what it "Takes" and "Gives"; "The only
  test is `defined`"; the systemd ones "as `systemd-escape`"; 7.8's
  tools.
- **The worst case** (render.md, 8.4; programming-model.md, 6.3): what an
  evaluation holds is bounded and checked at the shipped limits.
- **The oracle** (9.5): differences from MiniJinja "listed".

## Choices this plan makes

- **Where 7.5 is silent, MiniJinja 2's builtins** (9.5): `split` keeps
  empty pieces (`''` gives `['']`); `join` writes integers in decimal;
  `replace` goes left to right, without overlap. **Where it speaks,
  hearth's:** `first` of `[]` fails, `tojson` escapes no `<>&'`, and as
  skein-json's writer does (`\b \f \n \r \t`, other controls `\u00xx` in
  lower case, `/` and DEL as they are); `sort` by bytes as `<` compares
  (7.5; MiniJinja's ignores case): hearth 30's oracle lists these.
- **The systemd filters are systemd 261's** (7.5: "as `systemd-escape`"),
  lower-case hex, pinned by rows from one run of the tool.
  `systemd_escape_path` takes a path (6.2), `basename` and `dirname` a path
  or a relative path of a `file`'s form (6.2); any other value (7.5: `''`,
  `a/`, `/a/../b`, a relative path to `systemd_escape_path`) is `Path`, no
  coreutils behaviour. The edges are 7.5's: `/` gives `/` and `/`, `/a`'s
  `dirname` is `/`, a one-segment relative path's is `.`.
- **The translation lands whole, in 43.2:** a total match over hearth 05's
  26 `Op`s needs render's 26, which hearth 14.1, 14.2 and 43.1 add; no
  expression reaches render before hearth 15.

## Rules that bind it

- **No recursion and no unbounded loop in step code,** and hearth 14's
  list of banned forms (render.md, 8.3; code-organization.md, 6).
- **Expected values are literals,** from 7.5 by hand, the systemd and
  coreutils rows from one run of `systemd-escape` 261 and coreutils 9.10 on
  x1, pasted: the implementer runs neither tool; the planner rechecks the
  rows on this machine in the base runs (`findings/`). Limits are render's
  `limits::SHIPPED`, or named beside it.
- **Shared files:** hearth 14's, and `crates/hearth-protocol-builder/
  {Cargo.toml, src/translate.rs, src/tests/translate.rs,
  src/tests/fixtures.rs}`. Landing second, keep the other's lines.

## Increments

### 43.1 The filters
Scope: 7.5's 17 filters behind `Op::Filter`.
Needs: nothing beyond hearth 14.
Files: new `src/evaluate/{filter, escape}.rs` (`escape.rs`: the text
filters from `tojson` on), `src/evaluate.rs`, `src/evaluate/operation.rs`,
new `src/tests/filters.rs`, `src/tests/fixtures.rs`; the world's
`src/{expressions, reference, fixtures}.rs`.
Shape: `Op::Filter(Filter)`, `Filter`, `arity`.
- It pops the arguments, the last first, then the input (05.2). An
  undefined input or argument is `Undefined`, but `default`'s: undefined
  input gives the argument as it is (format.md, 7.6).
- Each result is measured and charged before it is made; `tojson` measures
  by the walk, then writes (render.md, 2.3).
- `first` or `last` of `[]`, `split('')`, `replace('', b)`: `Empty`; a
  value `basename`, `dirname` or `systemd_escape_path` refuses: `Path`.
  `items`, `keys`, `values` skip `None` entries (format.md, 7.6). `sort`: a
  `List` of (key, index) pairs, derived `Ord`, `sort_unstable` (strings by
  bytes or integers by value; equal keys by index), then the operands
  moved by index; a mixed list is `Type`.
Test shape: in `src/tests/filters.rs`, each under 20 ms:
`default_gives_its_argument_only_where_undefined`,
`a_filter_of_undefined_but_default_fails_with_undefined`
(`settings.wifi | length`),
`split_cuts_at_each_separator_keeping_empty_pieces`,
`join_writes_strings_and_integers_between_separators`,
`replace_replaces_every_occurrence_left_to_right` (`'aaa' | replace('aa',
'b')` is `ba`),
`first_or_last_of_an_empty_list_fails_with_empty`,
`an_empty_split_separator_or_replace_pattern_fails_with_empty`
(`split('')`, `replace('', 'b')`),
`length_counts_code_points_elements_or_present_entries` (`'é'` is 1),
`items_keys_and_values_follow_the_keys_byte_order`,
`sort_orders_by_bytes_and_refuses_a_mixed_list` (a tie row: `['b', 'a', 'b']`),
`tojson_is_compact_with_keys_in_order_and_controls_escaped`,
`shell_quote_wraps_in_single_quotes_and_rewrites_each_quote`,
`the_systemd_filters_match_systemd_escape_261` (`.x` to `\x2ex`, `a-b` to
`a\x2db`; `--path`: `/` to `-`, `/dev/nvme0n1p3` to `dev-nvme0n1p3`; `/a//b/`,
`/a/./b`, `/a/../b`, `a/b` and `''` are no path: `Path`),
`basename_and_dirname_give_7_5s_edges` (`/`: `/`, `/`; `/a`: `a`, `/`; `a`:
`a`, `.`; `a/b`: `b`, `a`; `''`, `a/`, `/a/../b`: `Path`).
```rust
// crates/hearth-domain-render/src/tests/filters.rs
use skein_lib::bytes;

#[test]
fn tojson_is_compact_with_keys_in_order_and_controls_escaped() {
    let roots = Roots::x1_prod();
    let peers =
        [name(0, b"settings", Missing::Error), key(9, b"peers", Missing::Error), op(17, Op::Filter(Filter::Tojson))];
    assert_eq!(fixtures::evaluate(&roots, &peers), Ok(Some(string(br#"{"a":{"port":51820},"b":{"port":51821}}"#))));
    let text = [op(0, Op::String(bytes::copy_of(b"\"\\\n\x01</>\xc3\xa9"))), op(9, Op::Filter(Filter::Tojson))];
    let escaped = b"\"\\\"\\\\\\n\\u0001</>\xc3\xa9\"";
    assert_eq!(fixtures::evaluate(&roots, &text), Ok(Some(string(escaped))), "format.md, 7.5: skein-json's forms");
}
```
The world's trees gain every filter at its arity; 14.1's sweep asserts each
fault kind fell at least once (`assert_fell` over `expressions::KINDS`, which
gains `"empty"` and `"path"`).
Proof: world, `generated_expressions_evaluate_as_the_reference_evaluates_them`,
now with filters: each of the 17 gives the reference's value on generated
input. Nothing calls `evaluate` until hearth 15.2. Bound 0.5 s.
Don't: follow MiniJinja where 7.4 or 7.5 differ (Choices); make a result
before charging it.
Size: at most 300 lines.
Removes: nothing.

### 43.2 hearth 05's operations translated; the world through the compiler
Scope: `hearth-protocol-builder`'s `translate/operations.rs`: hearth 05's
`Expression`, `Operation`, `Op`, `Missing`, `Filter` into render's,
exhaustive both ways (hearth 11's Choices); the world's trees printed as
text, compiled by hearth 05 and translated.
Needs: hearth 37.2 (`expression::compile`; `Op`'s 26 variants by 37.2).
Files: `crates/hearth-protocol-builder/Cargo.toml`
(`hearth-protocol-template`), `src/translate.rs`, new
`src/translate/operations.rs`, `src/tests/translate.rs` (`back::{expression,
op, missing, filter}`, total matches over render's types),
`src/tests/fixtures.rs` (`every_op() -> [template::Op; 26]`); root
`Cargo.toml`, `Cargo.lock`. World: `tests/render/Cargo.toml`
(`hearth-protocol-template`, `hearth-protocol-builder`),
`src/expressions.rs` (`text(&Tree)`; `evaluate` compiles at the template
machine's `Limits { expression: <the text's length>, ..SHIPPED }`, translates,
evaluates), `src/reference.rs` (05's
marking: a chain is a bound name and its lookups, the name marked `Error`,
format.md, 7.6).
Shape: the sketch's five functions, matches with no wildcard, patterns with
no `..`; boxes moved; the operations rebuilt through `List::with_capacity`
and `into_boxed`. The printer writes the parentheses precedence needs, none
around a chain before `is` or `default`, and wrapping none that a literal or
parentheses start (by precedence: 37.1's generator wraps every operand but a
chain before `is` or `default`; 7.6: no chain).
Test shape: in `src/tests/translate.rs`, each under 20 ms:
`each_operation_translates_to_render_and_back` (26 rows),
`each_filter_translates_to_render_and_back` (17 rows),
`missing_translates_to_render_and_back`,
`an_expressions_offsets_and_values_cross_unchanged`.
```rust
// crates/hearth-protocol-builder/src/tests/translate.rs
#[test]
fn each_operation_translates_to_render_and_back() {
    for (row, op) in fixtures::every_op().into_iter().enumerate() {
        let back = back::op(translate::operations::op(op.clone()));
        assert_eq!(back, op, "row {row}: hearth 05's Op through render's and back");
    }
}
```
The world's two tests keep their names and assertions.
Proof: world, `generated_expressions_evaluate_as_the_reference_evaluates_them`:
generated text through hearth 05's compiler, this translation and render's
evaluator gives the reference's value or fault, for every seed. Nothing
above runs an expression until hearth 15.2 (templates) and
hearth 16 (declarations). Bound 0.5 s.
Don't: translate instructions (hearth 15) or declarations (hearth 16).
Size: at most 150 lines.
Removes: the world's `flatten`, hearth 14.1's stand-in for hearth 05's compiler.

### 43.3 What one evaluation holds
Scope: `evaluate::worst_case` (programming-model.md, 6.3): both stacks at
capacity and a `Budget`'s `made`, checked at the shipped limits.
Needs: hearth 37.3 (`skein-heap` is a workspace dependency since it).
Files: `src/evaluate.rs`, `src/tests/evaluate.rs`; world
`tests/render/Cargo.toml` (`skein-heap` under `[dev-dependencies]`, as
`tests/template/Cargo.toml:12-13`), `src/fixtures.rs`
(`at_the_limits()`, `wide_roots()`, the world's `Roots` of 14.1), new
`tests/memory.rs`.
Shape: `worst_case` (`ELEMENT` is 14.1's, `evaluate.rs:53`); `at_the_limits()`
and `wide_roots()` are `#[must_use]`. `at_the_limits()` compiles: a 4,096-byte
list (2,048 values), 64 nested lists, a `~` chain and a `replace` past
`made`, `tojson` and `sort` of a 4,096-entry setting, `split` of a 1 MiB
string.
Test shape: unit `an_operand_costs_no_more_than_an_elements_charge`
(`size_of::<Operand<'_>>()` within `ELEMENT`, the worst case's premise),
under 20 ms; in a binary of its own, under 1 s:
```rust
// tests/render/tests/memory.rs
#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn evaluating_at_the_limits_holds_no_more_than_the_worst_case() {
    let bound = evaluate::worst_case(&limits::SHIPPED).expect("the shipped limits have a worst case");
    let roots = fixtures::wide_roots();
    for (expression, what) in fixtures::at_the_limits() {
        let meter = Meter::new();
        meter.start();
        let mut evaluator = Evaluator::new(&limits::SHIPPED);
        let mut budget = Budget::new(&limits::SHIPPED);
        let (context, bindings) = (roots.context(), Bindings::with_capacity(0));
        let evaluated = evaluate::expression(&mut evaluator, &expression, &context, &bindings, &mut budget);
        drop(evaluated);
        drop(evaluator);
        let step = meter.end();
        meter.check(step, bound, &what);
    }
}
```
Proof: world, `heavy cargo nextest run -p hearth-render-world -E
'binary(=memory)'`, once: an evaluation at `SHIPPED` holds no more than
`worst_case`, the share hearth 28 adds to render's. Bound 1 s.
Don't: raise a limit or a ceiling to fit a test.
Size: at most 60 lines.
Removes: nothing.

## Interface sketches

render owns these, beside hearth 14's `Op`, `Missing`, `FaultKind`
(`Empty` and `Path` are 14.1's, `evaluate.rs:67-68`); 43.2 maps hearth 05's types to them. Types
derive `Clone, PartialEq, Eq, Hash, Debug` unless said.

```rust
// crates/hearth-domain-render/src/evaluate/filter.rs (43.1)
/// Mirrors its `Filter`, 7.5's order; no `named`: render parses nothing.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Filter {
    Default, Join, Split, First, Last, Length, Replace, Items, Keys, Values,
    Sort, Tojson, ShellQuote, SystemdEscape, SystemdEscapePath, Basename, Dirname,
}
impl Filter {
    /// The arguments it pops, as the template machine's `arity`.
    #[must_use] pub const fn arity(self) -> u32;
}

// crates/hearth-domain-render/src/evaluate.rs (43.3), beside hearth 14's
/// The most an `Evaluator` and a spent `Budget` hold at `limits`; `None` past a u64.
#[must_use] pub fn worst_case(limits: &Limits) -> Option<u64>;

// crates/hearth-protocol-builder/src/translate/operations.rs (43.2); `template` is
// hearth-protocol-template, `render` the root's `pub use`.
#[must_use] pub fn expression(expression: template::Expression) -> render::evaluate::Expression;
#[must_use] pub fn operation(operation: template::Operation) -> render::evaluate::Operation;
#[must_use] pub fn op(op: template::Op) -> render::evaluate::Op;
#[must_use] pub fn missing(missing: template::Missing) -> render::evaluate::Missing;
#[must_use] pub fn filter(filter: template::Filter) -> render::evaluate::Filter;
```

## Questions to raise when reached

None.

## Done when

1. `git log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-43\.[1-3]'` prints 3.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run
   -p hearth-domain-render -p hearth-render-world -p hearth-protocol-builder`
   passes, and `heavy cargo nextest run -p hearth-render-world --profile
   fuzzy` passes.
3. `git -C ~/src/rust/hearth ls-tree --name-only origin/main
   crates/hearth-protocol-builder/src/translate/` lists `operations.rs`.
4. `git -C ~/src/rust/hearth grep -n -i minijinja origin/main --
   crates/hearth-domain-render tests/render/src` prints nothing.
5. `git -C ~/src/rust/hearth grep -n -E ', \.\. ?\}|\(\.\.\)' origin/main -- crates/hearth-protocol-builder/src/translate | wc -l` prints 0 (a struct update such as `..SHIPPED` is no match).
