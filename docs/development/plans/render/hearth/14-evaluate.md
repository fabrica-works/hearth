# Session 14: expressions evaluated: values, the context, undefined, the operators

render's `evaluate` module (code-organization.md, 2) runs an expression's
postfix operations (render.md, 8.3) on a value stack, with no recursion,
against the context's roots and the names bound above them (format.md,
7.7). 14.1 lays the values, the context, lookups and undefined (7.6);
14.2 the operators and jumps (7.4). hearth 43 takes the 17 filters (7.5),
the translation of hearth 05's operations, and what one evaluation holds.
Nothing in the thin story evaluates an expression: each increment proves
in render's domain world against a reference evaluator, and hearth 15.2
connects the evaluator to the real loop.

## Read first

- hearth's `format.md`: 7.3 to 7.7, 6.2 to 6.4, 8.2, 5.2, 3.4;
  `render.md`: 2.3, 3.6, 7.1, 7.2, 8.1, 8.3, 8.4, 9.1, 9.5.
- This plan: `hearth/05-expressions.md` and 37 whole; the sketches of
  `hearth/06-templates.md`, `hearth/03-json.md` and
  `hearth/11-thin-story.md` (with 11's Choices and "Where later sessions
  add"); `findings/skein-testing.md`, 1, 2, 7.
- skein at `f981746`: `crates/skein-lib/src/stack.rs` (`with_capacity`:20,
  `push`:48, `pop`:57), `list.rs` (`push`:55, `into_boxed`:104), `bytes.rs`
  (`copy_of`:12, `find_from`:40), `decimal.rs` (`of`:18), `writer.rs`
  (`new`:32, `put`:49, `finish`:60). `programming-model.md`: 3, 6.1, 6.3,
  8; `testing-strategy.md`: 2.4, 3, 6 (by number only).

## What the design asks

- **A value stack** (render.md, 8.3): "the domain evaluates it on a value
  stack whose depth the expression limit bounds".
- **The context** (7.7): `target` (`member` "undefined outside a
  cluster"), `debian`, `settings` ("defaults filled"),
  `tasks.<task>.<setting>`, `exports.<task>`, `key`, `value`, "`loop` and
  assigned names"; "Nothing else".
- **Values and operators** (7.4): "strings, integers, booleans, lists and
  maps"; "`and` and `or` short-circuit and give an operand"; "`false`, `0`,
  `""`, `[]`, `{}` and undefined are false"; "`==` ... deeply; values of
  two types are unequal"; "`<` ... two integers or two strings, compared
  by bytes"; "`in` finds a key in a map, an element in a list, a substring
  in a string"; "`+` and `-` take integers, checked"; "`~` joins strings
  and integers, written in decimal"; Output: "A list or a map is an error".
- **Undefined** (7.6): "A lookup that finds nothing is an error"; it
  "exists in four places only"; it "may be tested, defaulted or used as a
  condition"; "Printing it, looping over it, comparing it, looking into it
  or filtering it otherwise is an error"; a chain "that goes straight into
  `is defined` or `default` gives undefined at its first missing link".
- **Errors and bounds:** 3.6, "an undefined value, a type error or an
  exceeded limit is an error at the template's line"; 2.3, an expression
  of 4 KiB nested 64. **The oracle** (9.5): differences "listed".

## Choices this plan makes

- **render's own value,** `evaluate::Value`: 7.4's five kinds as validation
  gives them (6.2), no null, no position; an `Entry` holds `None` for 7.6's
  absent optional value, no key of its map but to a lookup (7.6). It is neither config's `json::Value` nor hearth
  12's composed value: hearth 13 makes `settings` and `tasks` of it, 17
  `debian`, 20 `exports` and `target.containers`; 14.1 makes `target`.
- **The stack holds `Operand`s:** undefined, a context value borrowed, or a
  made value; a made list holds operands, so `items` copies no context
  value. No `Cow` (clippy.toml): nothing is cloned to be changed.
- **One `for` over the operations,** a jump setting how many to skip. `==`,
  `in` and `tojson` walk on a `lib::Stack`, never through derived traits.
  The `Evaluator`'s stacks serve a template's expressions (hearth 15).
- **hearth 05's `Missing` decides:** a `Key` or `Item` marked `Undefined`
  that finds nothing gives undefined, one marked `Error` fails; a `Name`
  is always marked `Error` (7.6: the name is no link, so an unbound name
  fails before `is defined` too); an absent optional value is undefined
  either way. Names: the five roots, then `Bindings`, innermost first
  (hearth 06 refuses shadowing, log 52).
- **One fault, the first,** at the failing operation's `at`, where the
  fault is read (render.md, 7.1); hearth 15 and 16 place it. No error
  record: hearth 10's "Who adds what" gives `evaluation` to hearth 15 and
  16; nothing here emits one. A limit met is `Limit`, `evaluation` (7.2).
- **Where 7.4 is silent, MiniJinja 2's builtins** (9.5): `length` counts
  code points. **Where it speaks, hearth's:** `'a' + 'b'` and `true ~ 'x'`
  are `Type`, `1 == true` is false; hearth 30's oracle lists these (the
  filters', hearth 43's).
- **A `Budget` for made values** (render.md, 2.3: one evaluation's 16 MiB),
  passed by the caller, charged a string's bytes and 32 a list element, a
  fixed cost so that a fault is the same on every build; made lists nest
  at most 64 deep; hearth 15 decides its span. The value stack's room is
  `Limits::expression`, 4 KiB wherever an expression stands (2.3).
- **The world's oracle:** a recursive reference written from 7.4 to 7.7,
  on generated trees and contexts; the translation of hearth 05's
  operations lands whole in hearth 43.2, so until then the world
  flattens its trees itself (`flatten`).

## Rules that bind it

- **No recursion and no unbounded loop in step code** (render.md, 8.3;
  code-organization.md, 6): every walk a `for` bounded by `Limits` and the
  value's size, on a `lib::Stack`. No `Vec`, `String`, `format!`, `_` arm,
  `matches!`, `if let` on render's enums, `str::from_utf8`,
  `slice::split`, `sort_by` or `binary_search_by` (clippy.toml): strings
  are UTF-8 by construction, `bytes::find_from` searches, a key is found by
  bisection in a bounded `for`. The world's reference recurses.
- **Expected values are literals,** from 7.4 by hand. Limits are render's
  `limits::SHIPPED`, or named beside it.
- **Shared files:** root `Cargo.toml`, `Cargo.lock`; render's `src/lib.rs`,
  `src/limits.rs`, `src/tests.rs`, `src/tests/fixtures.rs`;
  `tests/render/{Cargo.toml, src/lib.rs, src/fixtures.rs}`;
  `.config/nextest.toml` if no `fuzzy_*.rs` is on `main`. Landing second,
  keep the other's lines.

## Increments

### 14.1 Values, the context, lookups and undefined
Scope: values, the stack, the `Budget` (render.md, 8.3; 2.3); roots and
bindings (7.7), `target` from meta; literals, names, `.key`, `[i]`, lists,
`is [not] defined`; chains and absent optional values (7.6).
Needs: hearth 11 (render's crate and `limits`, `parts::meta::Meta`,
`tests/render`).
Files: render `src/lib.rs`, `src/limits.rs` (`depth` is 12.1's), new `src/evaluate.rs`, `src/evaluate/{operation, value,
context, lookup}.rs`, `src/tests.rs`, new `src/tests/evaluate.rs`,
`src/tests/fixtures.rs`. World: `tests/render/{Cargo.toml, src/lib.rs,
src/fixtures.rs}`, new `src/expressions.rs`, `src/reference.rs`,
`tests/evaluate.rs`, `tests/fuzzy_evaluate.rs`.
Shape: the sketches; `Op`'s first nine variants.
- **Lookups:** a name is a root, else the innermost binding, else `Unknown`
  or undefined. `.key` or `[string]` into a map: the entry's value,
  undefined for `None`, else `NoKey` or undefined. `[integer]` into a list:
  the element, else (past the end, below 0) `NoIndex` or undefined
  (format.md, 7.4). Into undefined: undefined if marked, else `Undefined`.
  Any other pair is `Type`, in a chain too (7.4, 7.6).
- **Lists:** `List { length }` pops `length` operands; an undefined one is
  `Undefined` (7.6); made lists nested past `nesting` are `Limit`.
- **Charges:** a string literal its bytes, a list 32 an element, a copy out
  of a binding what it makes; past the budget, `Limit`.
- `context::target(&Meta)`: `name`, `base`, `environment`, `member`
  (`None` outside a cluster: 7.6's fourth place, and the root's only
  `None`), `placement { kind, firmware, hardware, on }` in format.md 3.4's
  words, and `containers`, an empty map (20.1 fills it). An absent `firmware`, `hardware` or `on` is left out of the map:
  7.6's "four places only" name none of them, so its lookup is `NoKey`,
  and a chain into `is defined` still gives false.
Test shape: `src/tests/fixtures.rs` gains `Roots`, owned, with `context()`
(15.2 moves it to `evaluate::context::Roots`), `Roots::x1_prod()` (x1-prod's
`target`; `settings` `{enabled: true, name: 'x1', peers: {a:
{port: 51820}, b: {port: 51821}}, port: 2222, tags: ['sudo', 'wheel'],
wifi: None}`; `tasks` `{core: {domain: 'example.org'}, wifi: None}`;
`exports`, `debian` small maps), `op`, `name`, `key`,
`string`, `list`, `map`, and `evaluate(&Roots, &[Operation]) ->
Result<Option<Value>, Fault>` at `limits::SHIPPED` (`None`: undefined).
Unit tests, each under 20 ms: `a_key_of_a_root_gives_the_maps_value`,
`an_unknown_name_fails_at_its_offset` (also before `is defined`, 7.6),
`a_lookup_of_the_wrong_type_fails_even_in_a_chain` (`settings.tags.x`,
`settings.port[0]`, `settings.tags[-1]`), `a_missing_key_fails_at_the_keys_offset`,
`a_key_named_like_a_method_is_only_a_key` (`settings.keys`: `NoKey`, 7.4's
`u.keys`), `absent_optional_values_read_as_undefined_however_marked`
(`settings.wifi`; `tasks.wifi`, an optional read's absent task, 6.4),
`an_undefined_list_element_fails_with_undefined` (`[settings.wifi]`),
`an_index_outside_the_list_fails_at_its_bracket` (2 of 2; -1),
`a_chain_into_is_defined_is_false_at_its_first_missing_link`,
`looking_into_undefined_outside_a_chain_fails`,
`target_holds_metas_names_an_undefined_member_and_no_containers`,
`an_absent_placement_field_is_no_key_not_undefined` (x1-prod's `on`:
`target.placement.on` is `NoKey`; marked into `is defined`, false),
`a_literal_past_the_budget_fails_with_limit` (`made: 4`, so that `'hello'`
passes it).
```rust
// crates/hearth-domain-render/src/tests/evaluate.rs
#[test]
fn a_chain_into_is_defined_is_false_at_its_first_missing_link() {
    let roots = Roots::x1_prod();
    // `settings.peers.c.port is defined`, as hearth 37.1 marks it (7.6: the name is no link)
    let operations = [
        name(0, b"settings", Missing::Error),
        key(9, b"peers", Missing::Undefined),
        key(15, b"c", Missing::Undefined),
        key(17, b"port", Missing::Undefined),
        op(22, Op::Defined),
    ];
    assert_eq!(fixtures::evaluate(&roots, &operations), Ok(Some(Value::Bool(false))), "format.md, 7.6: undefined at `c`, then false");
}
```
World (ordinary Rust): `expressions::roots(rng)` (the world's own `Roots`,
owned with `context()`, until 15.2; generated, depth at most 4, with `None`
entries; `key`, `value` bound in some rounds),
`tree(rng, &Roots, Shape)` (names and keys present and missing, chains
marked by 37.1's rule), `flatten` (37.2's table), `evaluate(&Tree, &Roots,
&Limits) -> Result<Option<Plain>, &'static str>` (render's; the fault by
`fixtures::kind`); `reference::evaluate`, alike, recursive;
`fixtures::limits(rng)`: `SHIPPED` three rounds in four, else `made` 16 to
4,096 and `nesting` 1 to 8 (testing-strategy.md, 3).
```rust
// tests/render/tests/evaluate.rs
#[test]
fn generated_expressions_evaluate_as_the_reference_evaluates_them() {
    for seed in 0..200_u64 {
        let mut rng = Rng::new(0x0014_0000 + seed);
        let roots = expressions::roots(&mut rng);
        let tree = expressions::tree(&mut rng, &roots, expressions::SMALL);
        let evaluated = expressions::evaluate(&tree, &roots, &limits::SHIPPED);
        assert_eq!(evaluated, reference::evaluate(&tree, &roots, &limits::SHIPPED), "seed {seed}: {tree:?}");
    }
}
```
Under 0.5 s. The sweep, `tests/fuzzy_evaluate.rs`:
`generated_expressions_and_contexts_evaluate_as_the_reference_does`, 10,000
rounds at `fixtures::limits(rng)` and `WIDE`, the kinds met checked by
`fixtures::assert_fell(&fell, kinds)` (new in `tests/render/src/fixtures.rs`,
taking its kinds list as 06's does); its time is `fuzzy_evaluate`'s nextest
line under `--profile fuzzy`, aim 2 s.
Proof: world, `generated_expressions_evaluate_as_the_reference_evaluates_them`:
render's stack gives the reference's value or fault on lookups, lists and
chains, for every seed. Nothing calls `evaluate` until hearth 15.2,
which evaluates a template of the thin repository on the real
loop. Bound 0.5 s.
Don't: build `debian`, `settings`,
`tasks` or `exports`; add an error record or a `vocabulary.rs` variant.
Size: at most 300 lines.
Removes: nothing.

### 14.2 Operators, jumps, truthiness and output
Scope: `not`, `== != < <= > >=`, `in`, `not in`, `~ + -`, 37.2's four
jumps; truthiness and output (7.4); the walk.
Needs: nothing beyond 14.1.
Files: `src/evaluate{.rs, /operation.rs}`, new `src/evaluate/{operators,
walk}.rs`, new `src/tests/operators.rs`, `src/tests/fixtures.rs`; the
world's `src/{expressions, reference, fixtures}.rs`.
Shape: `Op`'s next sixteen variants; `truthy`, `output`, `Written`.
- Undefined is false for `not` and the jumps (`and` and `or` keep it as
  their value), and `Undefined` for every other operator ("comparing it")
  and as a branch of `… if … else …` (7.6): the walk checks the top where
  a branch ends, at the `Jump`, and, when `JumpIfFalse` is taken, at the
  landing of the `Jump` before its own landing (37.2's layout).
- `==`, `!=`: one walk over both operands, a context value and a made one
  alike; `None` entries are no entries (7.6). `in`: a string needle in a
  map's keys or a string (`''` is in every string), any needle in a list;
  else `Type` (7.4).
- `+`, `-`: checked, `Overflow`. `~`: strings and integers (`Decimal::of`
  the magnitude, after a `-`), measured, charged, written by one `Writer`.
Test shape: in `src/tests/operators.rs`, each under 20 ms:
`and_keeps_a_false_left_operand_and_never_evaluates_the_right`,
`a_conditional_evaluates_its_condition_then_one_branch`,
`an_undefined_branch_of_a_conditional_fails_with_undefined` (either
branch), `or_gives_an_undefined_operand_where_both_are_undefined`,
`truthiness_is_false_for_the_six_values` (`false`, 0, `''`, `[]`, `{}`,
undefined),
`equality_is_deep_and_two_kinds_are_unequal` (`[1, [2]] == [1, [2]]`,
`1 == '1'`), `ordering_takes_two_integers_or_two_strings_by_bytes`
(`'B' < 'a'`; `1 < 'a'` is `Type`), `in_finds_a_key_an_element_or_a_substring`,
`sums_past_i64_fail_with_overflow_at_the_operator`,
`concat_writes_integers_in_decimal` (`'a' ~ -12 ~ 3` is `a-123`),
`undefined_compared_or_joined_fails_with_undefined` (`settings.wifi ==
1`, `settings.wifi ~ 'x'`),
`a_map_is_used_without_its_absent_entries` (`in`, `==`, truthiness),
`output_writes_booleans_as_words_and_integers_in_decimal` (`true`,
`false`, `-12`, `-9223372036854775808` through `output` and `Written`),
`output_refuses_lists_maps_and_undefined`.
```rust
// crates/hearth-domain-render/src/tests/operators.rs
#[test]
fn and_keeps_a_false_left_operand_and_never_evaluates_the_right() {
    let roots = Roots::x1_prod();
    // `0 and nope`: `nope` is bound to nothing, and is never looked up
    let operations = [op(0, Op::Integer(0)), op(2, Op::JumpIfFalseKeeping { over: 1 }), name(6, b"nope", Missing::Error)];
    assert_eq!(fixtures::evaluate(&roots, &operations), Ok(Some(Value::Int(0))), "format.md, 7.4: `and` gives an operand");
}
```
The world's trees gain these; half the rounds pass the value through
`output`, as `{{ e }}` would.
Proof: world, `generated_expressions_evaluate_as_the_reference_evaluates_them`,
now with operators and conditionals: every jump lands where the
reference's branch does. Nothing calls `evaluate` until hearth 15.2.
Bound 0.5 s.
Don't: compare through a derived `PartialEq`; coerce a kind; recurse.
Size: at most 220 lines.
Removes: nothing.

## Interface sketches

render owns these; its `Expression`, `Operation`, `Op`, `Missing` and
`Filter` mirror hearth 05's variant for variant (05's sketch gives each
`Op`'s pops). hearth 15 runs instructions with `Evaluator`, `Bindings`,
`truthy` and `output`. Types derive `Clone, PartialEq, Eq, Hash, Debug`
unless said. hearth 43 adds `Filter`, the translation, `worst_case`.

```rust
// crates/hearth-domain-render/src/limits.rs (11.1): `Limits` gains, 14.1, from 2.3:
// `expression: u32` (4 KiB, the value stack's room), `nesting: u32` (64),
// and `made: u64` (one evaluation's values, 16 MiB), in `SHIPPED` as
// `expression: 4_096, nesting: 64, made: 16_777_216`; it uses 12.1's `depth` (JSON's 32).

// crates/hearth-domain-render/src/evaluate/operation.rs (14.1; variants by increment)
/// Mirrors hearth-protocol-template's `Expression`.
pub struct Expression { pub operations: Box<[Operation]>, pub values: u32 }
/// Mirrors its `Operation`: `at`, the offset a fault names.
pub struct Operation { pub at: u32, pub op: Op }
/// Mirrors its `Op`.
pub enum Op {
    /// 14.1.
    String(Box<[u8]>), Integer(i64), Boolean(bool),
    Name { name: Box<[u8]>, missing: Missing }, Key { name: Box<[u8]>, missing: Missing },
    Item { missing: Missing }, List { length: u32 }, Defined, NotDefined,
    /// 14.2.
    Not, Equal, NotEqual, Less, LessOrEqual, Greater, GreaterOrEqual, In, NotIn, Concat, Add, Subtract,
    JumpIfFalseKeeping { over: u32 }, JumpIfTrueKeeping { over: u32 }, JumpIfFalse { over: u32 }, Jump { over: u32 },
    /// hearth 43.1.
    Filter(Filter),
}
/// Mirrors its `Missing`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Missing { Error, Undefined }

// crates/hearth-domain-render/src/evaluate/value.rs (14.1)
/// A context value (format.md, 6.2, 7.4): no null, no position; `Int` holds a `duration` in
/// seconds, `String` UTF-8 with no U+0000, `Map` a `map` or an `object`, keys in byte order, each once.
pub enum Value { Bool(bool), Int(i64), String(Box<[u8]>), List(Box<[Value]>), Map(Box<[Entry]>) }
/// A key and its value; `None`: 7.6's absent optional value.
pub struct Entry { pub key: Box<[u8]>, pub value: Option<Value> }
/// A value on the stack. No `PartialEq`: 7.4's `==` is the walk's.
#[derive(Clone, Debug)]
pub enum Operand<'c> {
    /// 7.6's undefined.
    Undefined,
    /// A context value or a binding's, borrowed.
    Value(&'c Value),
    /// Made by the expression; a list's `depth` counts the made lists it nests, itself included.
    Bool(bool), Int(i64), String(Box<[u8]>), List { depth: u32, items: Box<[Operand<'c>]> },
}

// crates/hearth-domain-render/src/evaluate/context.rs (14.1)
/// format.md, 7.7's roots, each a map: `target` below (`containers`: hearth 20); `debian`
/// (hearth 17); `settings`, `tasks` (an optional read's absent task `None`: hearth 13); `exports` (hearth 20).
#[derive(Clone, Copy, Debug)]
pub struct Context<'c> {
    pub target: &'c Value, pub debian: &'c Value, pub settings: &'c Value, pub tasks: &'c Value, pub exports: &'c Value,
}
/// The `target` root of a target's meta: `member` `None` outside a cluster; absent placement fields left out.
#[must_use] pub fn target(meta: &Meta) -> Value;
/// A name above the roots: `key`, `value` (hearth 16); a loop's names, `loop`, a `set` (hearth 15).
#[derive(Clone, Debug)]
pub struct Binding<'c> { pub name: &'c [u8], pub value: Operand<'c> }
/// The bindings, innermost last; `push` gives the binding back when full; `truncate` keeps the
/// first `length` when an iteration, a loop or an entry ends.
#[derive(Debug)]
pub struct Bindings<'c> { /* List<Binding<'c>> */ }
impl<'c> Bindings<'c> {
    #[must_use] pub fn with_capacity(capacity: u32) -> Bindings<'c>;
    pub fn push(&mut self, binding: Binding<'c>) -> Result<(), Binding<'c>>;
    pub fn truncate(&mut self, length: u32);
    #[must_use] pub fn len(&self) -> u32;
    #[must_use] pub fn is_empty(&self) -> bool;
}

// crates/hearth-domain-render/src/evaluate.rs (14.1; truthy, output 14.2; worst_case hearth 43.3)
/// The value stack (`Limits::expression`) and the walk's (`depth + nesting`); empty between expressions.
#[derive(Debug)]
pub struct Evaluator<'c> { /* values: Stack<Operand<'c>>, walk: Stack<…> */ }
impl<'c> Evaluator<'c> { #[must_use] pub fn new(limits: &Limits) -> Evaluator<'c>; }
/// What made values may still take (render.md, 2.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Budget { /* left: u64 */ }
impl Budget { #[must_use] pub fn new(limits: &Limits) -> Budget; }
/// A made list element's charge: at least an `Operand`'s size (hearth 43.3 checks it).
pub const ELEMENT: u64 = 32;
/// `expression`'s value in `context` under `bindings`, charged to `budget`; or the first fault.
pub fn expression<'c>(evaluator: &mut Evaluator<'c>, expression: &Expression, context: &Context<'c>, bindings: &Bindings<'c>, budget: &mut Budget) -> Result<Operand<'c>, Fault>;
/// 7.4's truthiness.
#[must_use] pub fn truthy(operand: &Operand<'_>) -> bool;
/// What `{{ e }}` writes (7.4, "Output"), or `Output`, `Undefined`.
pub fn output<'v>(operand: &'v Operand<'_>) -> Result<Written<'v>, FaultKind>;
/// A string, or a boolean's word, as bytes; an integer as a sign and its digits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Written<'v> { Bytes(&'v [u8]), Integer { negative: bool, digits: Decimal } }
/// Why an expression has no value (render.md, 7.2, `evaluation`), at the failing operation's `at`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Fault { pub at: u32, pub kind: FaultKind }
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FaultKind {
    /// 7.6: an unknown name, a missing key, an index outside the list.
    Unknown, NoKey, NoIndex,
    /// Undefined used but as 7.6 allows; a kind the operation does not take; `+` or `-` past i64.
    Undefined, Type, Overflow,
    /// hearth 43.1: `first`, `last` of `[]`, an empty `split` or `replace` pattern; a value `systemd_escape_path`, `basename` or `dirname` refuses.
    Empty, Path,
    /// A list or a map output; made values past the `Budget` or `nesting` (render.md, 2.3, 7.2).
    Output, Limit,
}
```

## Questions to raise when reached

None.

## Done when

1. `git log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-14\.[1-2]'` prints 2.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run
   -p hearth-domain-render -p hearth-render-world` passes, and `heavy
   cargo nextest run -p hearth-render-world --profile fuzzy` passes.
3. `git -C ~/src/rust/hearth grep -n -i minijinja origin/main --
   crates/hearth-domain-render tests/render/src` prints nothing.
