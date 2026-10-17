# Session 36: `hearth-protocol-template`: whitespace, `raw`, interpolated strings, worst cases

hearth 06's crate gains MiniJinja 2's whitespace rules and `raw` in
`template::compile` (format.md, 7.2, 7.3), `interpolation::compile`, which
turns a declaration's key or string field (5.2) into text and outputs of
06's `Instruction` type, and `limits::worst_case` for both entry points.
The world `tests/template` gains whitespace-laden templates and strings,
cut and mutated, against its reference. Nothing calls `template::compile`
until hearth 15, nor `interpolation::compile` until hearth 16.

## Read first

hearth's format.md, 7.2, 7.3 (`raw`), 5.2 and 2.3; render.md, 2.3, 8.1,
9.4, 9.5; testing.md, 2.4, 2.9. This plan's `hearth/05-expressions.md` and
`hearth/06-templates.md`, whole, whose names this file takes as written,
and `hearth/03-json.md`'s sketches (`Value`, `Kind::String`: "UTF-8,
unescaped"). decisions.md, 9 and 10; `findings/skein-testing.md`, 2, 3
and 7. skein's `crates/skein-lib/src/bytes.rs` (`find_from`:40) and
`crates/skein-json/src/{tokenizer.rs:96, document.rs:196}` (a worst case
per entry point). The definition 7.2 names, in the cargo registry (the
first landing's `cargo fetch` puts it there):
`~/.cargo/registry/src/*/minijinja-2.20.0/src/compiler/lexer.rs`
(`lstrip_block`:220, `should_lstrip_block`:229, `skip_basic_tag`:254,
`handle_tail_ws`:583, `tokenize_root`:611, `handle_start_marker`:662,
`handle_raw_tag`:727).
Read skein's programming-model.md, 3, 6.3, 7 and 10, and
testing-strategy.md, 2.4 and 3, by number only.

## What the design asks

- **Whitespace** (7.2): `trim_blocks` and `lstrip_blocks` on, the `-` and
  `+` markers, "Trailing newlines are kept as written"; "MiniJinja 2's
  semantics"; render.md, 9.5: "byte for byte the same in both".
- **`raw`** (7.3): "the text between is output as written"; 7.8 keeps
  "os's inline `set`, `raw`, `-%}`, `+%}`".
- **Interpolation** (5.2): "each `{{ expr }}` is replaced by its value, a
  string or an integer. Statements (`{% %}`) are not allowed there"; "A
  field of another type ... may be written as one `{{ expr }}`".
- **Bounds** (2.3): "a template ... | 1 MiB"; the worst case per entry
  point (programming-model.md, 6.3; code-organization.md, 2).

## Choices this plan makes

- **Whitespace is MiniJinja 2's** (render.md, 9.5), by 36.1's rules, also
  (format.md 7.2, 7.3): Unicode `White_Space`
  (`char::is_whitespace`'s set), a table of its 25 code points' UTF-8
  forms, since step code may not call `core::str::from_utf8`.
- **An interpolated string's offsets are in the decoded string:** hearth 16
  places its errors at the string's `Position`, since JSON's escapes break
  offset-to-column. Its `Output`s are 06's, compiled as in a template.
- **Limits:** none new; 06's `template` and `statements`, and render.md 2.3 on
  which texts the bounds bind (an expression, wherever it stands), hold here.

## Rules that bind it

- **No recursion and no unbounded loop in step code** (render.md, 8.3;
  code-organization.md, 6), as 06's: no `Vec`, `_` arm or `matches!`; the
  world's reference recurses.
- **Extensions in place:** `ErrorKind` and `Decoder` grow, and their users
  are edited to the new form, every name and expected value kept.
- **The Pulse:** no `minijinja` in the crate or the world, comments
  included. No shared file, crate or dependency.

## Increments

### 36.1 Whitespace and `raw`

Scope: 7.2's rules, as MiniJinja 2's lexer applies them with
`trim_blocks`, `lstrip_blocks` and `keep_trailing_newline` on; `raw`.
Needs: hearth 06 (06.2).
Files: new `src/template/whitespace.rs` (the rules and the `White_Space`
table) and `src/template/raw.rs`; `src/{template, template/scan}.rs`; new
`src/tests/whitespace.rs`; the world's `src/{generate,
reference}/template.rs`, `src/fixtures.rs`.
Shape, the rules (whitespace: `White_Space`, newlines included):
- `-` after an opening removes the whitespace ending the text before it;
  `-` before a closing, the whitespace starting the text after it;
- **trim_blocks:** one `\n` directly after an unmarked `%}` or `#}` is
  removed; never after `}}`;
- **lstrip_blocks:** before an unmarked `{%` or `{#`, the run of
  whitespace but `\r` and `\n` directly before it is removed when a `\n` or the
  template's start precedes the run as written; never before `{{`;
- `+` after `{%` or `{#` stops lstrip_blocks, before `%}` or `#}` stops
  trim_blocks; `{{+` and `+}}` are errors, `+` not being a marker beside an
  output tag: 05 refuses it as `Expected(Operand)` (format.md 7.2);
- **`raw`** (7.3): a `{%` whose head is `raw` then the tag's end; anything
  else after `raw` is `Expected(TagEnd)`. Its body, one `Text`, runs to
  the first `{%` that opens `endraw` (a marker, whitespace, `endraw`,
  whitespace, a marker, `%}`), else `Unclosed` at the `raw`'s `{`. The
  body's start loses one `\n` (unmarked `%}`) or its leading whitespace
  (`-`). Its end loses its trailing whitespace (`{%-` endraw), or,
  unmarked, its trailing whitespace but `\n` when what remains is empty
  or ends in `\n` (`lstrip_block`). After endraw's `%}`, the rules above;
- nothing else removes a byte: a template's trailing newline is kept.

Test shape: in `src/tests/whitespace.rs`, each under 20 ms:
`a_statement_alone_on_its_line_leaves_nothing_behind`,
`trim_blocks_removes_one_newline_after_statements_and_comments_only`,
`lstrip_blocks_removes_indentation_only_at_the_start_of_a_line`,
`a_minus_marker_strips_all_whitespace_on_its_side` (U+00A0, U+3000, VT
and `\n` among it), `a_plus_marker_keeps_what_trim_and_lstrip_would_remove`
(7.2's `grub.cfg` line among it), `trailing_newlines_are_kept_as_written`,
`a_plus_beside_an_output_tag_is_an_error` (`{{+ a }}`, `Expected(Operand)`
at 2; `{{ a +}}`, at 6),
`raw_outputs_its_body_with_its_tags_whitespace_rules`,
`a_raw_without_endraw_is_refused_at_its_tag`,
`the_table_holds_the_25_white_space_code_points` (the table against
`char::is_whitespace` for every `char`). Expected texts are literals,
checked once against MiniJinja 2.20.0 with 9.5's settings: one run for
each expected text of these tests, in a scratch crate outside the
repository (`heavy cargo run --offline`, after the first landing's `cargo
fetch`); the report lists the test names checked.
```rust
// crates/hearth-protocol-template/src/tests/whitespace.rs
#[test]
fn a_statement_alone_on_its_line_leaves_nothing_behind() {
    let compiled = template(b"a\n  {% if x %}\n  b\n  {% endif %}\nc\n").expect("a condition compiles");
    assert_eq!(
        *compiled.instructions,
        [text(0, b"a\n"), branch(4, expr(1, [name(10, b"x")]), 1), text(15, b"  b\n"), text(33, b"c\n")]
    );
}
```
The generator draws whitespace around every tag (space, tab, `\n`, VT,
FF, U+00A0, U+3000 in text; 05's three inside tags), a marker on every
delimiter, and raw bodies holding `{{`, `{%`, `{% endif %}` and an
unclosed `{%endraw`. The reference strips with `str::trim_end_matches(
char::is_whitespace)`, not the table.
Proof: world; nothing calls `template::compile` until hearth 15. `heavy
cargo nextest run -p hearth-template-world --profile fuzzy`, once, shows
every rule giving the reference's texts, and each refusal at its offset.
Bound: 3 s.
Don't: apply trim_blocks after `}}`, or lstrip_blocks before `{{`; strip
inside a `raw` body beyond its edges.
Size: at most 220 lines.
Removes: `UnknownStatement` for `raw` and `endraw`.

### 36.2 Interpolated strings

Scope: `interpolation::compile` for a declaration's keys and string
fields (5.2), as hearth 03 decodes them: UTF-8 and unescaped.
Needs: nothing beyond 36.1.
Files: new `src/interpolation.rs` and `src/tests/interpolation.rs`;
`src/{lib, error}.rs`, `src/tests/fixtures.rs`. The world's
`src/generate/template.rs` (`string`), `src/reference/template.rs`
(`interpolate`), `src/fixtures.rs` (`STRING_KINDS`), `tests/{templates,
fuzzy_templates}.rs`.
Shape: `Interpolated`; `ErrorKind::Interpolation`.
- A text with no `{` followed by `{`, `%` or `#` is `Plain`. Otherwise
  it is `Parts`: `Text`s as written (no whitespace rule applies: there
  are no statements) and `Output`s, each found and compiled as in a
  template, its offsets in the string.
- There is no pass 1: the string is UTF-8 by 03, and a `\r` is text.
- `{%` is `Interpolation` at its `{` (5.2), and so are `{#`, at its `{`,
  and a `-` or `+` marker, at the marker; `${{ '{#' }}a[@]}` writes a
  shell's `${#a[@]}`. `{{` with no end is `Unclosed`.
  "One `{{ expr }}`" is `Parts` holding one `Output` alone, which the
  domain types by the field (hearth 16).

Test shape: in `src/tests/interpolation.rs`, each under 20 ms:
`a_string_without_tags_is_plain`, `one_output_alone_is_parts_of_one_output`,
`text_and_outputs_alternate_in_order`,
`a_statement_in_a_string_is_refused_at_its_brace`,
`a_comment_or_a_marker_in_a_string_is_refused`,
`the_escape_writes_a_comment_opener_as_text` (`${{ '{#' }}a[@]}`),
`an_unclosed_output_in_a_string_is_refused_at_its_braces`.
```rust
// crates/hearth-protocol-template/src/tests/interpolation.rs
#[test]
fn text_and_outputs_alternate_in_order() {
    let read = interpolation::compile(b"/home/{{ key }}/.ssh", &SHIPPED).expect("a key interpolates");
    let parts = [text(0, b"/home/"), output(6, expr(1, [name(9, b"key")])), text(15, b"/.ssh")];
    assert_eq!(read, Interpolated::Parts(Box::new(parts)));
}
```
In the world, `generated_strings_interpolate_as_the_reference_reads_them`
(200 seeds, under 0.5 s) and the sweep
`cut_and_mutated_strings_interpolate_as_the_reference_reads_them` (2,000
rounds, about 1 s, `assert_fell` with `STRING_KINDS`).
Proof: world; nothing calls `interpolation::compile` until hearth 16.
`heavy cargo nextest run -p hearth-template-world --profile fuzzy`, once,
shows generated strings, 5.2's examples' forms among them,
read as the reference reads them. Bound: 2 s.
Don't: allow statements; check a field's type (hearth 16).
Size: at most 100 lines.
Removes: nothing.

### 36.3 The worst cases, and texts at the limits

Scope: 37.3's `limits::worst_case` for templates and interpolated strings,
`Decoder::Template` and `Decoder::Interpolated { length }`, as skein-json
gives one for each entry point; the capacities `compile` sizes by.
Needs: nothing beyond 36.2 (and 37.3, which 06 needs).
Files: `src/limits.rs` (the two variants and their arms), `src/{template,
interpolation}.rs`; the world's `src/fixtures.rs`
(`templates_at_the_limits()`), `tests/{memory, templates, fuzzy_templates}.rs`.
Shape: the worst case counts these at once: the text's openings at three
instructions each (06's `3 × openings + 1`), one operation per byte of
expression, the bytes of texts, names and strings (at most the text's length), `lines` (one `u32`
per byte at most), the block stack and the jump list, and
`worst_case(Decoder::Expression, limits)` for the expression compiled.
`templates_at_the_limits()` gives 1 MiB of text, 1 MiB of `{{a}}`, 1 MiB
of `\n`, 32 nested `if`s and `for`s, a 4,096-byte expression in a tag,
and each of these one past.
Test shape: `compiling_templates_and_strings_at_the_limits_holds_no_more_than_the_worst_case`
in `tests/memory.rs` (37.3's binary and `Meter` loop, every case at
`SHIPPED`), under 1 s (37.3's test takes the other 1 s of the Bound below).
`templates_at_the_limits_compile_as_the_reference_does`
in `tests/templates.rs`, under 0.5 s, runs the 1 MiB cases scaled to
`template: 4096` (named beside `SHIPPED`, with the reason: two compilers
over 1 MiB in a debug build pass the focused aim), the rest at `SHIPPED`.
`templates_at_the_limits_cut_and_mutated_compile_as_the_reference_does`,
in the sweep's file, runs 500 rounds at tiny limits (`template` 64 to
4,096, `statements` 1 to 8) with the shapes scaled, about 1.5 s: the
file then takes about 4.5 s (06's 2 s, 36.2's 1 s and this), its Bound.
Proof: world; nothing calls the crate until hearth 43.2 and 15. `heavy
cargo nextest run -p hearth-template-world -E 'binary(memory)'`, once,
shows a template's and a string's compile at `SHIPPED` holding no more
than their worst cases. Bound: 2 s.
Don't: raise a limit or a ceiling to fit a test.
Size: at most 60 lines.
Removes: nothing.

## Interface sketches

hearth 15 mirrors 06's `Template`, `Instruction`, `Does` and `Names`;
hearth 16 maps `Interpolated`. 05's and 06's sketches stand as written, but
for what is added here.

```rust
// crates/hearth-protocol-template/src/limits.rs (`Decoder`'s two variants 36.3)
pub enum Decoder {
    /// `expression::compile` (37.3).
    Expression,
    /// `template::compile` (06.1).
    Template,
    /// `interpolation::compile`, for a text of at most `length` bytes (03's `Limits::document`; the memory test passes `SHIPPED`'s).
    Interpolated { length: u32 },
}

// crates/hearth-protocol-template/src/interpolation.rs (36.2)
/// A key or string field's decoded text (format.md, 5.2) read, or the first error met.
pub fn compile(text: &[u8], limits: &Limits) -> Result<Interpolated, Error>;
/// What an interpolated text is.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Interpolated {
    /// No tag: the text as written.
    Plain,
    /// `Text` and `Output` instructions only, in order; one `Output` alone is 5.2's "one `{{ expr }}`".
    Parts(Box<[Instruction]>),
}

// crates/hearth-protocol-template/src/error.rs (`ErrorKind` as 06's sketch gives it, and this)
pub enum ErrorKind {
    /// 36.2: a `{%`, a `{#` or a `-` or `+` marker in an interpolated text (5.2).
    Interpolation,
}
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-36\.[1-3]'` prints 3.
2. In a worktree of hearth at `origin/main`, `heavy cargo nextest run -p
   hearth-protocol-template -p hearth-template-world` and `heavy cargo
   nextest run -p hearth-template-world --profile fuzzy` each pass, once.
3. `git -C ~/src/rust/hearth grep -n 'pub fn compile' origin/main --
   crates/hearth-protocol-template/src ':!*/tests/*' ':!*/tests.rs' | wc -l`
   prints 3.
4. `git -C ~/src/rust/hearth grep -n -i minijinja origin/main --
   crates/hearth-protocol-template tests/template` prints nothing.
