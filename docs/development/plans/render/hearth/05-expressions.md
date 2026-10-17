# Session 05: `hearth-protocol-template`: expressions to postfix operations

A new machine crate, `hearth-protocol-template`, compiles one expression's
text (format.md, 7.4) into a flat sequence of operations in postfix order
(render.md, 8.3), and checks 7.5's filters by name and arity. It parses in
one pass with no recursion, and its errors carry a byte offset. This
session puts in place the crate, its limits and errors, the lexer, the
primaries, lookups, lists, parentheses and filters, and the vocabulary the
sketches give. Its world, `tests/template` (`hearth-template-world`),
checks generated texts, cut and mutated, against a reference compiler
written from the grammar. hearth 37 adds the operators, chains, jumps and
worst case; hearth 06, templates. Nothing calls the crate until hearth 43.2
(it translates the operations into render's, in
`hearth-protocol-builder/src/translate/`) and hearth 15.

## Read first

hearth's format.md, 7 whole, 6.2 (`int` is i64), 2.3 and 5.2; render.md,
2.3, 3.1, 7.2, 8.1 to 8.3, 9.4; testing.md, 2.1, 2.4, 2.9. This plan's
decisions.md, 9 and 10; `findings/skein-testing.md`, 2 (37.3), 3 and 7.
skein's `crates/skein-lib/src/stack.rs` and `list.rs` (`with_capacity`:21,
`push`:55, `get_mut`:72, `truncate`:82, `into_boxed`:104). The model for
this world is skein's JSON world: `tests/json/src/reference.rs:25`,
`src/generate.rs:229`, `tests/fuzzy_documents.rs:142`, `tests/memory.rs:134`,
`Cargo.toml:17-33`, and `tests/clippy.toml`. Read skein's
programming-model.md, 3, 6.3, 7, 8 and 10, and testing-strategy.md, 2.1,
2.4, 3 and 8, by number only.

## What the design asks

- **The grammar** (7.4), from `expression = or [ "if" or "else"
  expression ]` to `postfix`; "No calls: no functions, no `range`, no
  methods, no slices, no map literals." Strings take "the escapes `\\ \'
  \" \n \t`"; `integer = [ "-" ] [0-9]+`.
- **Filters** (7.5): 17, and "The only test is `defined`". **Chains**
  (7.6): "A lookup chain that goes straight into `is defined` or `default`
  gives undefined at its first missing link instead of an error".
- **Postfix** (render.md, 8.3): "the domain evaluates it on a value stack
  whose depth the expression limit bounds". **Bounds** (2.3): 4 KiB for an
  expression, nesting 64. Errors are `decode` (7.2); **fuzzed** (9.4).

## Choices this plan makes

- **A function, not a stream machine:** `expression::compile(text,
  &limits)` takes one expression's whole text (render.md, 8.1). In the
  world, "cut" means the text is cut at an offset drawn from the seed.
- **One pass:** a `for` over the tokens, each lexed on demand, emits
  postfix and holds pending constructs on a `lib::Stack<Pending>`. Jumps
  are relative (`over`), so code that moves keeps its targets (37.2).
- **A flat vocabulary:** one `Op` per construct (`!=`, `not in` and `is
  not defined` each have their own). Each `Operation` carries `at`, its
  token's offset, so hearth 14 can name 7.6's line. `Expression::values`
  is the largest stack on any path.
- **Chains are marked in the vocabulary,** since they are syntax: `Name`,
  `Key` and `Item` carry `missing: Missing` (37.1 sets it); a `Name` is
  always `Error`, the name being no link (format.md 7.6: an unbound one
  fails even before `is defined` or `default`). Which names are bound
  (7.7) is the domain's question.
- **The sign follows the grammar:** a `-` directly before a digit starts
  a literal where an operand is expected, and subtracts elsewhere: `a-1`
  is `a - 1`, `[-1]` holds a literal, `-x` and `- 1` are refused.
- **One error per text:** the first met reading left to right, at the
  token that completes its check (an arity: at the filter's name). The
  caller places it in a file (hearth-protocol-builder, or hearth 06).
- **Limits:** `limits::SHIPPED` holds 2.3's `expression: 4096` and
  `nesting: 64`, and `compile` refuses text past them.
- **The oracle:** the world's `reference::compile` is a recursive descent,
  one function per production of 7.4, emitting this file's layout. The two
  agree on the operations when both are `Ok`, else on the error's offset;
  the kinds are the unit tests'.

## Rules that bind it

- **No recursion and no unbounded loop in step code** (render.md, 8.3;
  code-organization.md, 6): one `for` over at most `text.len() + 1`
  tokens, with lib's `Stack` and `List` sized from the text and `Limits`.
  There is no `Vec`, `String`, `format!`, `_` arm or `matches!`.
- **No lint exception in tests, and expected values are literals:** unit
  tests compare `*expression.operations` with arrays of `Operation`s
  written out, offsets included. Limits are `limits::SHIPPED`, or are
  named beside it with the reason.
- **Shared files:** root `Cargo.toml` and `Cargo.lock`;
  `.config/nextest.toml` (05.1 switches the fuzzy filters if no fuzzy test
  is on `main`); `tests/clippy.toml` (skein's, copied by 05.1 if no world
  has landed it). The world never names `minijinja`, a Pulse word.

## Increments

### 05.1 The crate, its limits and errors, the lexer, the primaries

Scope: the crate and its world; `limits.rs` (render.md, 2.3) and
`error.rs`. The lexer reads names, `true` and `false`, strings with the
five escapes, and integers in i64 with the sign rule, leading zeros
allowed (7.4's `[0-9]+`) and the range checked by value, never by digit
count. Space, tab and `\n` separate tokens (format.md 7.4: a `\r` is
`Character`). `compile` takes a text that is one literal or one name.
Needs: nothing.
Files: new `crates/hearth-protocol-template/`: `Cargo.toml` (`skein-lib`;
`[lints] workspace = true`), `src/lib.rs` (`no_std` but in tests;
`forbid(unsafe_code)`), `src/{limits, error, operation, expression}.rs`,
`src/expression/lexer.rs`, `src/tests.rs`, `src/tests/{fixtures,
literals}.rs`. New `tests/template/`: `Cargo.toml` (lints as skein's
`tests/json/Cargo.toml:17-33`), `src/{lib, fixtures, generate, reference,
check}.rs`, `tests/{expressions, fuzzy_expressions}.rs`. Shared: root
`Cargo.toml` (the globs `"crates/*"` and `"tests/*"` if absent,
decisions.md 16; `skein-lib` by URL if absent; this crate by path),
`Cargo.lock`, `tests/clippy.toml`, `.config/nextest.toml`.
Shape:
- the sketches' `Limits`, `SHIPPED`, `Error`, `Expression` and
  `Operation`; `Op::{String, Integer, Boolean, Name { name }}` (37.1 adds
  `missing` to `Name`, `Key` and `Item`);
  `ErrorKind::{TooLong, Character, Unterminated, Escape, Integer,
  Expected(Operand | Operator)}`;
- `pub(crate) Lexer::next(&mut self, operand: bool) -> Result<Lexed,
  Error>`, with `Lexed { token, at }`. The words `and or not if else is
  in none None True False` are `Token::Word` (format.md 7.4); the length
  is checked first, giving `TooLong` at the limit's offset.

Test shape: `src/tests/fixtures.rs` gives `compile(text)` at `SHIPPED`,
`op(at, Op)`, `bytes(&[u8])` and `name(at, text)`. Unit tests, each
under 20 ms: `a_name_compiles_to_one_lookup_of_that_name`,
`a_string_literal_decodes_each_of_the_five_escapes`,
`an_escape_outside_the_five_is_refused_at_its_backslash`,
`an_unterminated_string_is_refused_at_its_opening_quote`,
`integer_literals_hold_the_range_of_i64_and_no_more` (both ends, one past
each), `leading_zeros_change_neither_an_integer_nor_its_range` (`007`,
`-0`, a 25-digit `0…01`; `i64::MIN` and one past it, each zero-padded),
`a_byte_that_begins_no_token_is_refused_where_it_stands` (`@`, `é`, `\r`),
`a_word_is_no_operand` (`and`, `or`, `in`, `is`, `if`, `else`, `none`,
`None`, `True`, `False`: `Expected(Operand)` at 0; `true` compiles),
`a_text_past_the_limit_is_refused_at_the_first_byte_past`.
```rust
// crates/hearth-protocol-template/src/tests/literals.rs
#[test]
fn a_string_literal_decodes_each_of_the_five_escapes() {
    let expression = compile(br#"'a\\b\'c\"d\ne\tf'"#).expect("the five escapes compile");
    assert_eq!(*expression.operations, [op(0, Op::String(bytes(b"a\\b'c\"d\ne\tf")))]);
    assert_eq!(expression.values, 1);
}
```
The world gives:
- `generate::text(rng, Shape)`, valid text for the grammar landed so far
  with its whitespace, quotes and escapes drawn; `generate::cut`, a
  prefix; `generate::mutate`, bytes and words of 7.4 inserted,
  overwritten or removed and a span duplicated;
- `reference::compile(text, &Limits) -> Result<Expression, u32>`;
- `check::invariants(&Expression, text)`: every `at` is in the text and
  every jump lands at or before the end; each operation is reached at one
  depth on every path, never below what it pops; the stack ends at 1, and
  its deepest is `values`;
- `fixtures`: `LIMITS`; `limits(rng)`, `SHIPPED` three times in four and
  otherwise tiny (`expression` 1 to 256, `nesting` 1 to 8: testing-
  strategy.md, 3); the shapes `SMALL` and `WIDE`; `kind(&ErrorKind) ->
  &'static str`, an exhaustive match; `EVERY_KIND` and `assert_fell`.

`generated_expressions_compile_as_the_reference_compiles_them`, in
`tests/expressions.rs`, loops over 200 seeds of valid `SMALL` text at
`LIMITS`, asserting `Ok`, under 0.5 s. The sweep runs 10,000 rounds, in
about 1.5 s (a round count is a literal, fixed once by timing the sweep:
the rule of 06, 36 and 37 too):
```rust
// tests/template/tests/fuzzy_expressions.rs
#[test]
fn cut_and_mutated_texts_compile_as_the_reference_compiles_them() {
    let mut fell = BTreeSet::new();
    for round in 0..10_000_u64 {
        let mut rng = Rng::new(0x0005_F000 + round);
        let (limits, valid) = (fixtures::limits(&mut rng), generate::text(&mut rng, fixtures::WIDE));
        let text = match rng.below(4) { 0 => valid, 1 => generate::cut(&mut rng, &valid), _ => generate::mutate(&mut rng, &valid) };
        let (got, want) = (expression::compile(&text, &limits), reference::compile(&text, &limits));
        assert_eq!(got.as_ref().map_err(|e| e.at), want.as_ref().map_err(|at| *at), "round {round}");
        match &got { Ok(e) => check::invariants(e, &text), Err(e) => { fell.insert(fixtures::kind(&e.kind)); } }
    }
    fixtures::assert_fell(&fell);
}
```
Proof: world, the highest this crate reaches: nothing calls `compile`
until hearth 43.2 and 15. `heavy cargo nextest run -p
hearth-template-world --profile fuzzy`, once, shows each generated
literal and name compiling to the reference's operations, and each of
05.1's refusals at the reference's offset. Bound: 2 s.
Don't: resolve a name against 7.7's context; depend on any crate but
`skein-lib`.
Size: at most 300 lines.
Removes: nothing.

### 05.2 Lookups, lists, parentheses and filters

Scope: the parse loop and its stack (programming-model.md, 8): `.name`
(`Key`) and `[e]` (`Item`), never after a filter (7.4's `filtered`);
lists (`List`) and `( e )`; `| name` and `| name(e, …)` (`Filter`), with
7.5's names and arities; nesting counted against `Limits::nesting` (render.md 2.3: each place format.md 7.4 nests).
Needs: nothing beyond 05.1.
Files: `src/filter.rs` (new); `src/expression/pending.rs` (new: the
stack's entries); `src/expression.rs`; `src/expression/lexer.rs` (`. | [
] ( ) ,`); `src/{operation, error, lib}.rs`; `src/tests/lookups.rs`
(new); `src/tests/fixtures.rs`; the world's
`src/{generate, reference, fixtures}.rs`.
Shape:
- `Filter`; `Op::{Key { name }, Item, List { length }, Filter}`;
  `ErrorKind::{TooDeep, Misplaced, UnknownFilter, Arity}` and
  `Expected::{Name, CloseParen, CloseBracket}`;
- `Pending::{Group { start }, List { start, length, at }, Index { start,
  at }, Arguments { start, filter, given, at }}`, where `start` is
  `out.len()` when the entry opened;
- a filter's input is pushed first, then its arguments; `f()` has none.

Test shape: in `src/tests/lookups.rs`, each under 20 ms:
`a_chain_of_keys_and_items_compiles_left_to_right` (`a.b['c'][0]`),
`u_keys_is_always_the_key_keys` (7.4: `u.keys` is `Key { keys }`, and
`u.keys | keys` that `Key`, then the filter),
`a_list_literal_pops_its_elements_in_order` (`[1, 'x', []]` is
`Integer` at 1, `String` at 4, `List { length: 0 }` at 9, `List { length:
3 }` at 0; `values` 3),
`each_filter_compiles_with_the_arguments_seven_five_gives` (17 rows),
`a_filter_given_another_count_is_refused_at_its_name`,
`an_unknown_filter_is_refused_at_its_name`,
`nesting_past_the_limit_is_refused_where_the_level_opens` (`(`, `[`,
`a[`, `f(`),
`calls_methods_slices_and_map_literals_are_refused_where_they_start`
(`range(3)` at 5 and `u.keys()` at 6: `Expected(Operator)` at the `(`;
`a[1:2]` at 3 and `{'a': 1}` at 0: `Character`),
`a_word_after_a_dot_is_a_key` (`a.and`, `a.in`, `a.true`, `a.none`),
`a_lookup_after_a_filter_is_refused_at_its_dot`.
The generator gains these forms, and `EVERY_KIND` these kinds.
Proof: world; nothing calls the crate yet (05.1). `heavy cargo nextest run
-p hearth-template-world --profile fuzzy`, once, shows these forms compiling as the reference compiles them, and each new
refusal falling at its offset. Bound: 2 s.
Don't: check a filter's input or the types of its arguments (hearth 43.1).
Size: at most 300 lines.
Removes: nothing.

## Interface sketches

The crate owns these types. hearth 14's and 43's evaluator mirrors `Expression`,
`Operation`, `Op`, `Missing` and `Filter` variant for variant, and hearth
43.2's `translate/` maps them, exhaustive both ways. hearth 06 extends
`ErrorKind` and `Limits`; hearth 37.3 adds `Decoder`.

```rust
// crates/hearth-protocol-template/src/limits.rs (05.1)
/// The template machine's limits (render.md, 2.3), configured (programming-model.md, 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// The longest expression, in bytes of its text; how deep one nests (render.md 2.3).
    pub expression: u32, pub nesting: u32,
}
/// render.md, 2.3's defaults: the shipped profile's.
pub const SHIPPED: Limits = Limits { expression: 4096, nesting: 64 };
// crates/hearth-protocol-template/src/expression.rs (05.1)
/// One expression's whole text (format.md, 7.4) compiled, or the first error met reading it.
pub fn compile(text: &[u8], limits: &Limits) -> Result<Expression, Error>;

// crates/hearth-protocol-template/src/operation.rs (05.1; variants by increment)
/// An expression compiled: its operations in postfix order (render.md, 8.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Expression {
    /// Run first to last; a jump only skips forward.
    pub operations: Box<[Operation]>,
    /// The most values on the stack at once, on any path.
    pub values: u32,
}
/// One operation, and where its token starts in the text.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Operation {
    /// A name, a literal's first byte, an operator; the `[` of `List` and `Item`; a `Key`'s is the first byte of its name.
    pub at: u32,
    pub op: Op,
}
/// What an operation does to the value stack; "pops b, then a" is the order values leave it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Op {
    /// A string literal, escapes decoded; an integer, its sign included; a boolean: pushes it (05.1).
    String(Box<[u8]>), Integer(i64), Boolean(bool),
    /// A name: pushes what it is bound to (05.1; `missing` 37.1, always `Error`: the name is no link, format.md 7.6).
    Name { name: Box<[u8]>, missing: Missing },
    /// `.name`: pops a value, pushes its key `name` (05.2).
    Key { name: Box<[u8]>, missing: Missing },
    /// `[i]`: pops i, then a value, pushes its key or element i (05.2).
    Item { missing: Missing },
    /// `[…]`: pops `length` values, pushes them as a list, in the order written (05.2).
    List { length: u32 },
    /// `| f(…)`: pops the arguments, the last first, then the input; pushes the result (05.2).
    Filter(Filter),
    /// `not`: pops a value, pushes whether it is false (37.1).
    Not,
    /// `== != < <= > >=`: pop b, then a; push a boolean (37.1).
    Equal, NotEqual, Less, LessOrEqual, Greater, GreaterOrEqual,
    /// `in`, `not in` (`at`: the `not`): pop the container, then the needle (37.1).
    In, NotIn,
    /// `~ + -`: pop b, then a; push a ~ b, a + b, a - b (37.1).
    Concat, Add, Subtract,
    /// `is defined`, `is not defined` (`at`: the `is`): pop a value, push a boolean (37.1).
    Defined, NotDefined,
    /// `and`: if the top is false, skips `over` operations, keeping it; else pops it (37.2).
    JumpIfFalseKeeping { over: u32 },
    /// `or`: if the top is true, skips `over` operations, keeping it; else pops it (37.2).
    JumpIfTrueKeeping { over: u32 },
    /// The `if` of `a if c else b`: pops c; if it is false, skips `over` operations (37.2).
    JumpIfFalse { over: u32 },
    /// The `else` of `a if c else b`: skips `over` operations, b's (37.2).
    Jump { over: u32 },
}
/// What a lookup that finds nothing gives (format.md, 7.6) (37.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Missing {
    /// An error, but for 7.6's absent optional values, which the domain knows.
    Error,
    /// Undefined, as is a lookup into undefined: a link of a chain into `is defined` or `default`.
    Undefined,
}

// crates/hearth-protocol-template/src/filter.rs (05.2)
/// format.md, 7.5's filters, in its order: each the filter of its name in snake case (`ShellQuote` is `shell_quote`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Filter {
    Default, Join, Split, First, Last, Length, Replace, Items, Keys, Values,
    Sort, Tojson, ShellQuote, SystemdEscape, SystemdEscapePath, Basename, Dirname,
}
impl Filter {
    /// The filter 7.5 names so, or none.
    #[must_use] pub fn named(name: &[u8]) -> Option<Filter>;
    /// The arguments it takes: 1 for `default`, `join`, `split`; 2 for `replace`; else 0.
    #[must_use] pub const fn arity(self) -> u32;
}

// crates/hearth-protocol-template/src/error.rs (05.1; kinds by increment)
/// Why a text is not an expression: render.md, 7.2's `decode`, placed in its file by the caller.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Error {
    /// The byte offset in the text the kind names.
    pub at: u32,
    pub kind: ErrorKind,
}
/// What is wrong at `at`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ErrorKind {
    /// 05.1: past `Limits::expression` (`at`: the first byte past); a byte outside a literal that
    /// starts no token; no closing quote (`at`: the opening one); an escape outside the five
    /// (`at`: its backslash); an integer outside i64.
    TooLong, Character, Unterminated, Escape, Integer,
    /// A token where the grammar wants another (05.1).
    Expected(Expected),
    /// 05.2: past `Limits::nesting` (`at`: what opens the level); an operator 7.4 does not allow
    /// after what precedes it; a filter 7.5 does not name.
    TooDeep, Misplaced, UnknownFilter,
    /// Another count of arguments than the filter takes; `at`: its name (05.2).
    Arity { filter: Filter, given: u32 },
    /// `is` not followed by `defined` or `not defined` (37.1).
    UnknownTest,
}
/// What the grammar wants: an operand; an operator, a close or the end (05.1); a name after `.`
/// or `|`, a `)`, a `]` (05.2); the `in` of `not in` (37.1); the `else` of `a if c` (37.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Expected { Operand, Operator, Name, CloseParen, CloseBracket, In, Else }
```

## Questions to raise when reached

None.
## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-05\.[1-2]'` prints 2.
2. `git -C ~/src/rust/hearth ls-tree -d --name-only origin/main
   crates/hearth-protocol-template tests/template | wc -l` prints 2.
3. In a worktree of hearth at `origin/main`, `heavy cargo nextest run -p
   hearth-protocol-template -p hearth-template-world` and `heavy cargo
   nextest run -p hearth-template-world --profile fuzzy` each pass, once.
4. `git -C ~/src/rust/hearth grep -n -i minijinja origin/main --
   crates/hearth-protocol-template tests/template` prints nothing.
