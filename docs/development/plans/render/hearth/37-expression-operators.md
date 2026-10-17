# Session 37: `hearth-protocol-template`: operators, chains, forward jumps, worst case

hearth 05's crate gains the rest of 7.4's grammar: `+ - ~`, one comparison,
`not`, the test `is [not] defined` and 7.6's chains (render.md, 8.3), then
`and`, `or` and `… if … else …` as forward jumps, then `limits::worst_case`
for `expression::compile`. The world `tests/template` gains these forms,
cut and mutated, against its reference. Nothing calls the crate until
hearth 43.2 and 15. hearth 06 needs this whole session.

## Read first

hearth's format.md, 7.4, 7.6, 6.2 and 2.3; render.md, 2.3, 8.3, 9.4, 9.5;
testing.md, 2.1, 2.4, 2.9. This plan's `hearth/05-expressions.md`, whole,
whose names and sketches this file takes as written; decisions.md, 9 and
10; `findings/skein-testing.md`, 2, 3 and 7. skein's
`crates/skein-lib/src/{stack, list}.rs` (`push`:55, `get_mut`:72). The model
for the world is skein's JSON world, as 05's. Read skein's
programming-model.md, 3, 6.3, 7, 8 and 10, and testing-strategy.md, 2.1,
2.4, 3 and 8, by number only.

## What the design asks

- **Precedence** (7.4): the levels from `or` down to `filtered`; `test`
  and `comparison` stand at most once.
- **Chains** (7.6): "A lookup chain that goes straight into `is defined`
  or `default` gives undefined at its first missing link instead of an
  error".
- **Postfix** (render.md, 8.3): "with forward jumps for `and`, `or` and
  `… if … else …`; the domain evaluates it on a value stack whose depth
  the expression limit bounds". **Bounds** (2.3): 4 KiB, nesting 64.
  render.md, 9.5: the differences from MiniJinja are listed.

## Choices this plan makes

- **Chains are marked in 05's `Op`** (`Missing`),
  since they are syntax: the domain decides what a missing link means
  (7.7).
- **hearth keeps format.md 7.4's order** (`~` below `+ -`, `is` below the
  comparisons), where MiniJinja differs: both are listed (render.md 9.5).
- **The oracle** is 05's: a recursive descent, one function per production
  of 7.4, emitting 05's layout; the two agree on the operations when both
  are `Ok`, else on the error's offset; the kinds are the unit tests'.
- **The pass is 05's:** one `for`, a `lib::Stack<Pending>`, relative jumps;
  `values` is the deepest stack on any path.

## Rules that bind it

- **No recursion and no unbounded loop in step code** (render.md, 8.3;
  code-organization.md, 6), as 05's: no `Vec`, `String`, `format!`, `_` arm
  or `matches!`; unit tests compare `*expression.operations` with arrays of
  `Operation`s written out, offsets included.
- **Shared files:** root `Cargo.toml` and `Cargo.lock` (37.3: `skein-heap`).
  The world never names `minijinja`, a Pulse word.

## Increments

### 37.1 The operators, one comparison, the test, and chains

Scope: `+ - ~`; one comparison of `== != < <= > >= in not in`; `not`;
`is [not] defined`, with the precedence of 7.4's grammar; 7.6's chains,
whose lookups are marked `Missing::Undefined`, the root name never (it is
no link, 7.6).
Needs: hearth 05 (05.2).
Files: `src/expression.rs`, `src/expression/{pending, lexer}.rs`,
`src/expression/chain.rs` (new), `src/operation.rs` (`Missing`, a field of
`Name`, `Key` and `Item`), `src/error.rs`; `src/tests/operators.rs`
(new). The fixtures' `name` and `key`, and a new `item`, take
`Missing::Error`, which is all `literals.rs` and `lookups.rs` see of it
(`lookups.rs` writes `item` for `Op::Item`: README, Edits outside Files).
The world's
`src/{generate, reference, fixtures}.rs`.
Shape:
- **Levels,** 7.4's, from the loosest: conditional (37.2), `or`, `and`,
  `not`, test, comparison, `~`, `+ -`, filter, lookup (render.md 9.5). `or`, `and`,
  `~` and `+ -` associate to the left.
- **One test, one comparison:** each stands at most once, and only a
  looser level may follow it, `Misplaced` otherwise (`a < b < c`, `a is
  defined is defined`, `a == not b`).
- `Pending::{Not { at }, Binary { op, at }}`.
- **Chains** (format.md 7.6): a bound name with its `.name` and `[e]`
  lookups, never parenthesized or from a literal; a `List<u32>` holds the
  indices of its lookups. The name is no link, so an unbound one is an
  error even before `is defined` or `default`: `Name` stays `Error`. An
  index's own chain sits above them and is dropped at its `]`. At `|
  default`, each lookup the list holds is marked `Undefined`; at `is`,
  only if the chain is its whole operand (7.4's `test`), so `a ~ b is
  defined` marks nothing. Any other operator, filter, group or literal
  ends the chain: `(peer.endpoint) is defined` and `[1][5] is defined`
  mark nothing.
- `ErrorKind::UnknownTest`; `Expected::In`.

Test shape: in `src/tests/operators.rs`, each under 20 ms:
`operators_bind_as_seven_four_s_grammar_nests_them` (`a ~ b + c`, `not a
== b`, `a + b | length`, `'x' in a ~ b`, `not a is defined`),
`sums_and_joins_associate_to_the_left`,
`a_second_comparison_is_refused_at_its_operator`,
`a_test_other_than_defined_is_refused_at_its_name`,
`a_chain_into_is_defined_gives_undefined_at_each_link`,
`a_chain_into_default_gives_undefined_at_each_link`,
`a_lookup_used_otherwise_fails_when_it_finds_nothing` (`a.b | length is
defined`, `a ~ b is defined`; `x[a.b] is defined` marks its `Item`, not
`a.b`), `a_name_is_no_link_whatever_follows_it` (`wifi is defined`, `wifi |
default('x')`: `Name` marked `Error`), `a_parenthesized_or_literal_chain_marks_nothing`.
```rust
// crates/hearth-protocol-template/src/tests/operators.rs
#[test]
fn a_chain_into_is_defined_gives_undefined_at_each_link() {
    let expression = compile(b"peer.endpoint is defined").expect("a test compiles");
    assert_eq!(*expression.operations, [
        op(0, Op::Name { name: bytes(b"peer"), missing: Missing::Error }),
        op(5, Op::Key { name: bytes(b"endpoint"), missing: Missing::Undefined }),
        op(14, Op::Defined),
    ]);
}
```
The generator writes the parentheses that precedence needs, and others at
random, among them around a `~` beside `+ -` and an operand of `is` that
is not a lookup chain (render.md 9.5); the reference marks as the chain
rule above does.
Proof: world; nothing calls the crate yet (hearth 05.1). `heavy cargo
nextest run -p hearth-template-world --profile fuzzy`, once, shows operators, the
test and marked chains compiling as the reference compiles them. Bound:
2 s.
Don't: decide which types compare or add (7.4's rules, hearth 14).
Size: at most 250 lines.
Removes: nothing.

### 37.2 Short circuits and the conditional: forward jumps

Scope: `and`, `or` and `… if … else …`, nested to the right after `else`.
The condition is at `or`'s level, so `a if b if c else d else e` is
`Misplaced`. `values` holds across the jumps.
Needs: nothing beyond 37.1.
Files: `src/expression.rs`, `src/expression/pending.rs`,
`src/expression/jumps.rs` (new: the patches and the rotation),
`src/{operation, error}.rs`; `src/tests/jumps.rs` (new); the world's
`src/{generate, reference, check, fixtures}.rs`.
Shape: the layout that hearth 14 evaluates:

| Text | Operations |
|---|---|
| `a and b` | a, `JumpIfFalseKeeping { over: len(b) }`, b |
| `a or b` | a, `JumpIfTrueKeeping { over: len(b) }`, b |
| `a if c else b` | c, `JumpIfFalse { over: len(a) + 1 }`, a, `Jump { over: len(b) }`, b |

`Pending::{And { jump }, Or { jump }, If { start, mid, at }, Else { jump,
start }}`; each jump is patched when its entry is reduced, `over =
out.len() - jump - 1`. `If` and `Else` count one level each (render.md 2.3).
`Expected::Else` at the end of `a if c`. At `if`, the depth `values`
tracks falls back to its value before a.
Test shape: in `src/tests/jumps.rs`, each under 20 ms:
`and_keeps_a_false_left_operand_and_skips_the_right`,
`or_keeps_a_true_left_operand_and_skips_the_right`,
`a_conditional_runs_its_condition_first_then_one_branch` (`x if c else y`:
`c` at 5, `JumpIfFalse { over: 2 }` at 2, `x`, `Jump { over: 1 }` at 7,
`y`; `values` 1), `conditionals_nest_to_the_right_of_else`,
`jumps_inside_a_moved_branch_keep_their_targets` (`(a and b) if c or d
else e`), `a_conditional_without_else_is_refused_at_the_end`,
`values_counts_the_deepest_stack_on_any_path`.
Proof: world; nothing calls the crate until hearth 43.2. `heavy cargo
nextest run -p hearth-template-world --profile fuzzy`, once, shows all of 7.4
compiling as the reference compiles it, every jump going forward, and the
stack on every path as deep as `values` says. Bound: 2 s.
Don't: jump backward, or give absolute targets.
Size: at most 200 lines.
Removes: nothing.

### 37.3 The worst case, and texts at the limits

Scope: `limits::worst_case(Decoder::Expression, &Limits)`
(programming-model.md, 6.3), a `Decoder` first as hearth 02's
(code-organization.md, 2), hearth 36.3 adding its variants; the capacities
`compile` allocates by; the world at the shipped limits.
Needs: nothing beyond 37.2. `skein-heap` is in skein at hearth's pin.
Files: `src/{limits, expression}.rs`. In the world: `src/fixtures.rs`,
whose `at_the_limits()` gives a 4,096-byte list, 64 levels of each
bracket, of `not` and of `else`, the longest string and chain, and each of
these one past; `tests/memory.rs` (new); `tests/expressions.rs`;
`tests/fuzzy_expressions.rs`; `Cargo.toml` (`skein-heap` as a
dev-dependency). Shared: root `Cargo.toml` (`skein-heap` by URL),
`Cargo.lock`.
Shape: the worst case counts the output at one operation per byte, the
pending stack at its capacity, the chain's list, the bytes of names and
strings (at most the text's length), and the expression returned.
Test shape: `compiling_at_the_limits_holds_no_more_than_the_worst_case`,
in a binary of its own (findings 2), under 1 s (the Bound below);
`texts_at_the_limits_compile_as_the_reference_does` in
`tests/expressions.rs`, under 0.5 s; and in the sweep's file, 500 rounds
in about 1.5 s, `texts_at_the_limits_cut_and_mutated_compile_as_the_reference_does`:
the file then takes about 3 s with 05's 1.5 s, and its Bound is that sum.
```rust
// tests/template/tests/memory.rs
#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn compiling_at_the_limits_holds_no_more_than_the_worst_case() {
    let bound = limits::worst_case(Decoder::Expression, &fixtures::LIMITS).expect("a worst case");
    for (text, what) in fixtures::at_the_limits() {
        let meter = Meter::new();
        meter.start();
        let compiled = expression::compile(&text, &fixtures::LIMITS);
        meter.check(meter.end(), bound, &what);
        drop(compiled);
    }
}
```
Proof: world; nothing calls the crate until hearth 43.2. `heavy cargo
nextest run -p hearth-template-world -E 'binary(memory)'`, once, shows
that a compile at `SHIPPED` holds no more than `worst_case`. Bound: 1 s.
Don't: raise a limit or a ceiling to fit a test.
Size: at most 80 lines.
Removes: nothing.

## Interface sketches

05's sketches stand as written: `Op`'s variants, `Missing`, `UnknownTest`,
`Expected::{In, Else}` and the `Pending` entries are these increments'.

```rust
// crates/hearth-protocol-template/src/limits.rs (37.3)
/// One of the crate's entry points; hearth 36.3 adds its own.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Decoder {
    /// `expression::compile`.
    Expression,
}
/// The most heap one `compile` of `decoder` holds at once, what it returns included; `None` past a `u64`.
#[must_use] pub fn worst_case(decoder: Decoder, limits: &Limits) -> Option<u64>;
```

The rotation at `else`, in `src/expression/jumps.rs` (37.2), once
everything above `If { start, mid }` is reduced. `out[start..mid]` is a,
`out[mid..]` is c, and c runs first:
```rust
let a = mid.checked_sub(start).expect("a's start is not past its end");
let over = a.checked_add(1).expect("a and the jump fit a u32");
push(out, if_at, Op::JumpIfFalse { over })?;               // [a][c][J]
rotate_left(out, start, a)?;                                // [c][J][a]: three reversals, swapping
                                                            // by List::get_mut and mem::replace
let jump = out.len();
push(out, else_at, Op::Jump { over: 0 })?;                 // patched when b is reduced
pending.push(Pending::Else { jump, start: out.len() })?;   // b, or a conditional to its right
```


## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-37\.[1-3]'` prints 3.
2. In a worktree of hearth at `origin/main`, `heavy cargo nextest run -p
   hearth-protocol-template -p hearth-template-world` and `heavy cargo
   nextest run -p hearth-template-world --profile fuzzy` each pass, once.
3. `git -C ~/src/rust/hearth grep -n -i minijinja origin/main --
   crates/hearth-protocol-template tests/template` prints nothing.
