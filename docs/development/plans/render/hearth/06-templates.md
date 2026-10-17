# Session 06: `hearth-protocol-template`: templates to instructions, statements

hearth 05's and 37's crate gains `template::compile`, which turns a template
(format.md, 7.2, 7.3) into flat instructions: emit text, emit an expression,
branch, enter and advance a loop, assign (render.md, 8.3). It keeps a line
table for "the template's file and line" (7.6). Expressions go through 05's
`expression::compile`. The world `tests/template` gains templates, cut and
mutated, against a reference. Whitespace and `raw`, interpolated strings
and the worst cases are hearth 36's. Nothing calls `template::compile`
until hearth 15 (templates evaluated).

## Read first

hearth's format.md, 7 whole, 5.2 and 2.3 (UTF-8, "a `\r` in a template is
an error"); render.md, 2.3, 3.1, 7.2, 8.1 to 8.3, 9.4; testing.md, 2.4,
2.9. This plan's `hearth/05-expressions.md` and `hearth/37-expression-
operators.md`, whole, whose names this file takes as written.
decisions.md, 9 and 10; `findings/skein-testing.md`, 2, 3 and 7. skein's
`crates/skein-json/src/utf8.rs` (a byte-at-a-time check, `pub(crate)`
there: copy its shape), `crates/skein-lib/src/bytes.rs` (`copy_of`:12,
`find_from`:40, `count`:83), `stack.rs` (`push`:48, `pop`:57,
`top_mut`:69). The definition 7.2 names, in the cargo registry (the first
landing's `cargo fetch` puts it there):
`~/.cargo/registry/src/*/minijinja-2.20.0/src/compiler/lexer.rs`
(`tokenize_root`:611, `handle_start_marker`:662,
`tokenize_block_or_var`:771). Read skein's
programming-model.md, 3, 6.3, 7 and 10, and testing-strategy.md, 2.4 and
3, by number only.

## What the design asks

- **Delimiters** (7.2): "`{{ … }}` outputs an expression; `{% … %}` is a
  statement; `{# … #}` is a comment."
- **Statements** (7.3): `if`/`elif`/`else`/`endif`; `for x in e`, "over a
  list, or a map's keys in order"; `for k, v in e | items`, "two names
  only"; `set x = e`, "inside a `for`, lasts for one iteration; else to
  the end of the template"; `loop` "holds `index` (from 1), `index0`,
  `first`, `last` and `length`"; "Names may not shadow the context's roots
  (7.7)".
- **Refused** (7.3, "Nothing else"): seventeen forms, `macro` to
  `autoescape`; 7.8 keeps "os's inline `set`, `raw`, `-%}`, `+%}`".
- **Instructions** (render.md, 8.3), "evaluated by a `for` over a range
  bounded by the iteration limit, with a `lib::Stack` of loop frames";
  strict undefined is hearth 15's, "with the template's file and line"
  (7.6). **Bounds** (2.3): "a template ... | 1 MiB", "nesting of
  statements in a template | 32". A syntax error is `decode` (7.2).

## Choices this plan makes

- **Functions, as hearth 05's,** over the whole blob (render.md, 8.4).
  Pass 1 checks the length, then UTF-8 and `\r`, and builds the line
  table. Pass 2 is one `for` with a cursor, handling each tag where met.
- **One error per text:** pass 1's first, else the first met reading left
  to right, each found where its check completes.
- **Every `at` in a `Template` is an offset in the template:** an
  expression compiled from its slice has its operations' `at`s (and an
  error's) moved by the slice's start. hearth 15 places an evaluation
  error by `Template::lines` (`binary_search`), the protocol layer a
  refusal by `template::place`.
- **One instruction type** (code-organization.md, 6): jumps are relative,
  and only `Advance` goes back. There is one `Text` per non-empty run
  between tags, at its first kept byte. hearth 36.2 makes an interpolated
  string of `Text` and `Output`.
- **Inside tags, 05's separators** (format.md 7.4). A statement's head
  (word, names, `in`, `=`) is this session's scan; the rest, up to the
  tag's end, is an expression for 05's `compile`. A tag ends at the first
  `}}` or `%}` outside a string literal, which must be its own kind's
  (MiniJinja refuses `{{ a %}` too), a comment at its first `#}`.
- **What MiniJinja 2 reads and hearth refuses** (7.3, 7.8): a statement
  word outside 7.3's is `UnknownStatement`; `for … else` is `Unmatched`
  at the `else`. `for … if` and `recursive` fail as 05's expressions, and
  block `set` as `Expected(Assign)`. `loop.revindex` and the like compile
  as keys that hearth 15 finds missing. Valid generated text holds none of
  these, and 06.2's table test (`each_statement_seven_three_excludes`)
  lists them.
- **Shadowing is a decode error here:** `for` and `set` may not bind
  `target`, `debian`, `settings`, `tasks`, `exports`, `key`, `value` or
  `loop`, in every template (format.md 7.3, 7.7). 05's words (`and or
  not if else is in true false none None True False`) are `Expected(Name)`
  (format.md 7.4).
- **What follows `in`** is an expression with no conditional at its
  outermost level: `for x in a if c else b` is `Misplaced` at the `if`, and
  `for x in (a if c else b)` a loop (format.md 7.3, 7.7: as MiniJinja's
  parser has it). A pair loop is over two-element lists only, and what a
  pair's element must be is hearth 15's.
- **Limits:** `Limits` gains `template` and `statements`. A level is an
  open `if` or `for`, and every expression keeps 05's 4 KiB wherever it
  stands (render.md 2.3).

## Rules that bind it

- **No recursion and no unbounded loop in step code** (render.md, 8.3;
  code-organization.md, 6): pass 2's `for` runs at most `text.len() + 1`
  steps, each moving the cursor. Open blocks sit on a `lib::Stack<Block>`
  of `Limits::statements`, jumps to patch in a `List<u32>`, instructions
  in a `List` of `3 × openings + 1` (pass 1 counts them: an `elif` adds a
  `Jump`, a `Branch` and its text). There is no `Vec`, `_` arm or
  `matches!`. The world's reference recurses.
- **Extensions in place:** `Limits`, `ErrorKind` and `Expected` grow, and
  their users (05's `kind`, `limits(rng)`, `assert_fell`) are edited to
  the new form, every name and expected value kept (README, Rules).
- **The Pulse:** no `minijinja` in the crate or the world, comments
  included. No shared file, crate or dependency.

## Increments

### 06.1 Templates: bytes, lines, text, outputs and comments

Scope: `template::compile` for text, `{{ e }}` and `{# … #}`; pass 1;
`lines`; `place`; markers read, without effect until hearth 36.1; `{%` tags
found and refused as `UnknownStatement` until 06.2.
Needs: hearth 37, the crate and its world.
Files: new `src/{template, template/scan, instruction, utf8}.rs`;
`src/{lib, limits, error}.rs`; `src/tests.rs` (`mod templates;`),
`src/tests/fixtures.rs`, `src/tests/lookups.rs` (`ONE`, `:109`, becomes
`Limits { nesting: 1, ..SHIPPED }`), new `src/tests/templates.rs`. In `tests/template/`: new
`src/{generate, reference}/template.rs`; `src/{generate, reference,
check, fixtures}.rs`; new `tests/{templates, fuzzy_templates}.rs`;
`tests/fuzzy_expressions.rs` (`assert_fell`'s new form).
Shape:
- the sketches' `Template`, `Instruction`, `Does::{Text, Output}`,
  `compile`, `place`; `Limits::template`; `ErrorKind::{Utf8, Return,
  Unclosed, UnknownStatement}` and `Expected::TagEnd`;
- **pass 1** (`scan::bytes`): `TooLong` at `limits.template` if longer.
  Then, from the start, `Utf8` at the first byte RFC 3629 refuses (as
  skein-json's `Utf8::next`, `utf8.rs:21`) and `Return` at a `\r`.
  `lines` takes 0 and one past each `\n`, sized by `bytes::count(text,
  b"\n", u32::MAX) + 1`. It also counts the openings;
- **an opening** is `{` followed by `{`, `%` or `#`, found by
  `bytes::find_from`. A `-` directly after it, or directly before the
  closing delimiter, is a marker, and a `+` is one beside `{%`, `{#`, `%}`
  and `#}` only: beside `{{` or `}}` it stays in the expression, which 05
  refuses (format.md 7.2). A comment's closing marker is its body's last
  byte, so `{#-#}` has one only at its start;
- **an end:** for `{{` and `{%`, the first `}}` or `%}` outside a literal
  (a `'` or `"` runs to its unescaped twin, `\` skipping one byte). The
  end of the text inside a literal is 05's `Unterminated` at its quote.
  Otherwise no end is `Unclosed` at the opening's `{`. An end of the
  other kind (`{{ a %}`, `{% if a }}`) is `Expected(TagEnd)` at it, past
  any marker, found before the tag's inside is read. The expression is
  the slice between the markers, and `Output` sits at the `{{`.

Test shape: `src/tests/fixtures.rs` gains `template(text)` at `SHIPPED`,
`text(at, &[u8])`, `expr(values, [Operation; N])` and `output(at,
Expression)`. In `src/tests/templates.rs`, each under 20 ms:
`text_without_tags_compiles_to_one_text_as_written`,
`an_output_compiles_its_expression_at_offsets_in_the_template`,
`a_comment_leaves_nothing_and_splits_the_text_around_it`,
`closing_delimiters_inside_a_string_literal_do_not_end_the_tag`
(`{{ '}}' }}`, `{% if a == '%}' %}`),
`a_tag_without_its_end_is_refused_at_its_opening_brace`,
`an_end_of_the_other_kind_is_refused_where_it_stands` (`{{ a %}` at 5,
`{% if a }}` at 8, before 06.2 and after),
`a_carriage_return_is_refused_wherever_it_stands` (text, literal,
comment), `bytes_that_are_not_utf8_are_refused_at_the_first`,
`the_line_table_places_each_offset_at_its_line_and_column`,
`a_template_past_the_limit_is_refused_at_the_first_byte_past`.

```rust
// crates/hearth-protocol-template/src/tests/templates.rs
#[test]
fn an_output_compiles_its_expression_at_offsets_in_the_template() {
    let compiled = template(b"Port {{ settings.port }}\n").expect("an output compiles");
    assert_eq!(
        *compiled.instructions,
        [text(0, b"Port "), output(5, expr(1, [name(8, b"settings"), key(17, b"port")])), text(24, b"\n")]
    );
    assert_eq!(*compiled.lines, [0, 25]);
}
```
The world gives:
- `generate::template::text(rng, Shape)`, valid templates of what has
  landed: text runs holding `{`, `}}`, `%}`, `-`; literals holding `}}`,
  `%}`, `#}`; comments holding `{{` and quotes. `mutate` inserts,
  overwrites or removes delimiters, markers, statement words (7.3's
  refused ones too), `\r`, `0xFF` and a lone continuation byte; 05's `cut`
  is reused;
- `reference::template::compile(text, &Limits) -> Result<Template, u32>`,
  a recursive descent reading a tag at a time, its expressions by 05's
  `reference::compile`;
- `check::template`: every `at` in the text, each jump within it, no
  empty `Text`, `lines` 0 and one past each `\n`, and each expression
  passing 05's `check::invariants` against the template;
- `fixtures`: `SMALL_TEMPLATES`, `WIDE_TEMPLATES`, `TEMPLATE_KINDS` (05's
  and these), `limits(rng)`'s tiny `template` (64 to 4,096);
  `assert_fell(&fell, kinds)` takes its list, `EVERY_KIND` in 05's sweep.

`generated_templates_compile_as_the_reference_compiles_them`, in
`tests/templates.rs`, loops over 200 seeds of valid `SMALL_TEMPLATES` at
`LIMITS`, asserting `Ok`, under 0.5 s. The sweep,
`cut_and_mutated_templates_compile_as_the_reference_compiles_them` in
`tests/fuzzy_templates.rs`, follows 05's sweep line for line, with
`generate::template::{text, mutate}` at `WIDE_TEMPLATES`,
`template::compile` against `reference::template::compile`,
`check::template` on each `Ok`, seeds from `0x0006_F000`, and
`assert_fell(&fell, fixtures::TEMPLATE_KINDS)`: 5,000 rounds, about 2 s.
Proof: world, the highest this crate reaches: nothing calls
`template::compile` until hearth 15. `heavy cargo nextest run -p
hearth-template-world --profile fuzzy -E 'test(/templates/)'`, once,
shows each generated template compiling to the reference's instructions
at template offsets, and each refusal at its offset. Bound: 3 s.
Don't: evaluate or resolve a name; call `core::str::from_utf8`; place a
line by anything but `\n`.
Size: at most 290 lines.
Removes: nothing.

### 06.2 Statements: conditions, loops, assignments

Scope: 7.3's `if`, `elif`, `else`, `endif`, `for`, `endfor` and `set`;
their nesting against `Limits::statements`; shadowing.
Needs: nothing beyond 06.1.
Files: new `src/template/statement.rs` (the head and `Block`);
`src/{template, template/scan, instruction, limits, error}.rs`;
`src/expression.rs` (a `pub(crate)` entry that refuses a conditional at
the outermost level, for the iterable); `src/tests.rs` (`mod
statements;`), new `src/tests/statements.rs`; `src/tests/fixtures.rs`; the world's
`src/{generate, reference}/template.rs`, `src/{check, fixtures}.rs`.
Shape:
- `Does::{Branch, Jump, Enter, Advance, Set}` and `Names`;
  `ErrorKind::{Unmatched, Unended, Shadows}`; `Expected::{Statement,
  Assign}`, and `In` for `for`'s `in`. The layout that hearth 15
  evaluates (`over` counts the instructions skipped):

| Text | Instructions |
|---|---|
| `{% if c %}A{% endif %}` | `Branch { c, over: len(A) }`, A |
| `… A{% elif d %}B{% else %}C{% endif %}` | `Branch { c, over: len(A) + 1 }`, A, `Jump { over: to after C }`, `Branch { d, over: len(B) + 1 }`, B, `Jump { over: len(C) }`, C |
| `{% for x in e %}B{% endfor %}` | `Enter { One { x }, e, over: len(B) + 1 }`, B, `Advance { back: len(B) }` |
| `{% set x = e %}` | `Set { x, e }` |

- **The head:** whitespace, then a word. `if` and `elif` take the rest as
  the condition. `else`, `endif` and `endfor` take nothing more, and
  anything after them is `Expected(TagEnd)` where it starts. `for` takes
  a name, then optionally `,` and a name, then `in` (`Expected(In)`
  otherwise, so a third name is refused at its `,`: 7.3's "two names
  only"), then the iterable. `set` takes a name, then `=`
  (`Expected(Assign)`), then the value. An empty tag is
  `Expected(Statement)` at its end, any other word `UnknownStatement`. A
  name is 7.4's `name` and not one of 05's words; a root of 7.7 is
  `Shadows` at the name;
- `Block::{If { branch, jumps, at }, Else { jumps, at }, For { enter,
  at }}`: `branch` and `enter` are the indices to patch, and `jumps` is
  the length of the jump list when the `if` opened. `elif` and `else`
  push a `Jump`, then patch the open `Branch` to land after it; `else`
  turns the block to `Else`. `endif` patches the open `Branch`, if any,
  and the `Jump`s above `jumps`, `over = len - index - 1` each. A close
  the top block does not take (`endfor` in an `if`, `elif` after `else`,
  any close with none open) is `Unmatched` at its word. A block left open
  at the end is `Unended` at the innermost one's `{`. Pushing past
  `Limits::statements` is 05's `TooDeep` at the tag that opens the level.

Test shape: the fixtures gain `branch(at, Expression, over)`, `jump(at,
over)`, `enter(at, Names, Expression, over)`, `advance(at, back)`,
`set(at, name, Expression)`. In `src/tests/statements.rs`, each under 20 ms:
`a_condition_branches_over_its_body_to_after_endif`,
`elif_and_else_each_end_the_branch_before_with_a_jump`,
`a_loop_enters_over_its_body_and_advances_back_to_it`,
`a_loop_over_pairs_binds_both_names_in_order`,
`a_third_name_in_a_loop_is_refused_at_its_comma` (`{% for a, b, c in x |
items %}`, at 11), `set_binds_one_name_to_its_expression`,
`each_statement_seven_three_excludes_is_refused_at_the_offset_its_table_gives`
(a `const` of each form, its `ErrorKind` and offset: "Nothing else"'s
seventeen, `UnknownStatement` at the word; `for … else`, `Unmatched` at
the `else`; `for … if` and `recursive`, 05's errors at the token; block
`set` `{% set x %}`, `Expected(Assign)` at the tag's end; `{% set a, b = c
%}`, `Expected(Assign)` at the `,`),
`a_conditional_after_in_must_be_parenthesised` (`{% for x in a if c else b
%}`, `Misplaced` at the `if`, 14; `{% for x in (a if c else b) %}` loops),
`a_root_of_the_context_is_refused_as_a_name_bound`,
`a_close_no_open_block_takes_is_refused_at_its_word`,
`a_block_open_at_the_end_is_refused_at_its_tag`,
`statements_nested_past_the_limit_are_refused_where_the_level_opens`.

```rust
// crates/hearth-protocol-template/src/tests/statements.rs
#[test]
fn elif_and_else_each_end_the_branch_before_with_a_jump() {
    let compiled = template(b"{% if a %}x{% elif b %}y{% else %}z{% endif %}").expect("a condition compiles");
    assert_eq!(
        *compiled.instructions,
        [
            branch(0, expr(1, [name(6, b"a")]), 2),
            text(10, b"x"),
            jump(11, 4),
            branch(11, expr(1, [name(19, b"b")]), 2),
            text(23, b"y"),
            jump(24, 1),
            text(34, b"z"),
        ]
    );
}
```
The generator nests statements to the shape's depth, uses `loop.index`,
`index0`, `first`, `last` and `length` in loop bodies, draws names from a
pool that `mutate` widens with the roots, and never writes an
unparenthesized conditional after `in` (format.md 7.3). `check::template` adds: each
`Advance` goes back to just after its `Enter`, which skips just past it.
Proof: world; nothing calls `template::compile` until hearth 15. `heavy
cargo nextest run -p hearth-template-world --profile fuzzy`, once, shows
conditions, loops and assignments compiling as the reference compiles
them, each new refusal at its offset. Bound: 3 s.
Don't: check what a name is bound to, or what a loop iterates over
(hearth 15); jump back but by `Advance`.
Size: at most 300 lines.
Removes: `UnknownStatement` for the seven words this increment reads.

## Interface sketches

hearth 15 mirrors `Template`, `Instruction`, `Does` and `Names` in
render's evaluator, variant for variant. Its `translate/` maps them. 05's
sketches stand as written, but for the fields and variants added here;
hearth 36 adds `Decoder`'s variants, `Interpolated` and `Interpolation`.

```rust
// crates/hearth-protocol-template/src/limits.rs (05.1; `template` 06.1, `statements` 06.2)
/// The template machine's limits (render.md, 2.3), configured (programming-model.md, 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// 05: the longest expression, in bytes, wherever it stands (render.md 2.3); how deep one nests.
    pub expression: u32, pub nesting: u32,
    /// The longest template, in bytes (06.1); how deep its open `if` and `for` blocks nest (06.2; render.md 2.3).
    pub template: u32, pub statements: u32,
}
/// render.md, 2.3's defaults: the shipped profile's.
pub const SHIPPED: Limits = Limits { expression: 4096, nesting: 64, template: 1_048_576, statements: 32 };

// crates/hearth-protocol-template/src/template.rs (06.1)
/// A template's whole text (format.md, 7.2, 7.3) compiled, or the first error met.
pub fn compile(text: &[u8], limits: &Limits) -> Result<Template, Error>;
/// The line and column of `at` in `text`, each from 1, the column in bytes, as skein 02's `Position`.
#[must_use] pub fn place(text: &[u8], at: u32) -> (u32, u32);

// crates/hearth-protocol-template/src/instruction.rs (06.1; variants by increment)
/// A template compiled (render.md, 8.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Template {
    /// Run from the first; `Branch`, `Jump` and `Enter` skip forward, `Advance` goes back.
    pub instructions: Box<[Instruction]>,
    /// Each line's first offset: 0, then one past each `\n`.
    pub lines: Box<[u32]>,
}
/// One instruction, and where it starts in the text.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Instruction {
    /// Its tag's `{`; a text's first byte kept.
    pub at: u32,
    pub does: Does,
}
/// What an instruction does.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Does {
    /// Outputs its bytes (06.1).
    Text(Box<[u8]>),
    /// `{{ e }}`: outputs e's value (format.md, 7.4, "Output") (06.1).
    Output(Expression),
    /// `if c`, `elif c`: if c is false, undefined included (7.6), skips `over` instructions (06.2).
    Branch { condition: Expression, over: u32 },
    /// Before `elif` and `else`: the branch run skips `over` instructions, to after `endif` (06.2).
    Jump { over: u32 },
    /// `for`: if e has no element, skips `over`, past its `Advance`; else a frame binds the first (06.2).
    Enter { names: Names, iterable: Expression, over: u32 },
    /// `endfor`: binds the next element and goes back `back` to the body's first; after the last, drops the frame (06.2).
    Advance { back: u32 },
    /// `set x = e`: binds `name` until the innermost loop's iteration ends, or the template (06.2).
    Set { name: Box<[u8]>, value: Expression },
}
/// What a `for` binds (format.md, 7.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Names {
    /// `for x in e`: each element of a list, each key of a map in order.
    One { name: Box<[u8]> },
    /// `for k, v in e`: the two elements of each element (format.md 7.3: lists of two).
    Pair { first: Box<[u8]>, second: Box<[u8]> },
}

// crates/hearth-protocol-template/src/error.rs (05.1; these kinds 06.1, 06.2)
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ErrorKind {
    /// 05's, as its sketch gives them.
    TooLong, Character, Unterminated, Escape, Integer, Expected(Expected), TooDeep, Misplaced,
    UnknownFilter, Arity { filter: Filter, given: u32 }, UnknownTest,
    /// 06.1: not UTF-8 (`at`: the first byte); a `\r` (format.md, 2.3); a tag, comment or `raw` with no end (`at`: its `{`); a statement 7.3 lacks (`at`: its word).
    Utf8, Return, Unclosed, UnknownStatement,
    /// 06.2: a close no open block takes (`at`: its word); a block open at the end (`at`: its `{`); a root of 7.7 bound (`at`: the name).
    Unmatched, Unended, Shadows,
}
/// 05's; a tag's end, its own kind's (06.1); from 06.2, a statement after `{%`, `set`'s `=`, and `for`'s `in` as `In`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Expected { Operand, Operator, Name, CloseParen, CloseBracket, In, Else, Statement, Assign, TagEnd }
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-06\.[1-2]'` prints 2.
2. In a worktree of hearth at `origin/main`, `heavy cargo nextest run -p
   hearth-protocol-template -p hearth-template-world` and `heavy cargo
   nextest run -p hearth-template-world --profile fuzzy` each pass, once.
3. `git -C ~/src/rust/hearth grep -n 'pub fn compile' origin/main --
   crates/hearth-protocol-template/src ':!*/tests/*' ':!*/tests.rs' | wc -l`
   prints 2; `git -C
   ~/src/rust/hearth grep -n 'statements: 32' origin/main --
   crates/hearth-protocol-template/src/limits.rs` prints one line.
4. `git -C ~/src/rust/hearth grep -n -i minijinja origin/main --
   crates/hearth-protocol-template tests/template` prints nothing.
