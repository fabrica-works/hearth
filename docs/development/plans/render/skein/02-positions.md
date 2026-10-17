# Session 02: positions on skein-json's tokenizer

The tokenizer says where its last answer is (skein's `json.md`, 3.6):
`Tokenizer::position() -> Position { offset, line, column }`, all `u32`,
offset from 0, line and column from 1, a line feed alone ending a line.
First each token's first byte, the counters, the start and `restart`;
then each failure outside a string, by 3.6's table; last a string's or a
key's failures, at the character or escape at fault. `Token`, `Event`,
`Error` and the collector are unchanged, so no user in skein, smith, jig
or temper changes. In each increment the world's reference parser counts
the same positions on its own and every checked run compares them; the
`.expect` transcripts stay as they are. hearth 03 (strict JSON with a
position on every value) needs the whole session.

## Read first

- skein's `docs/design/json.md`: 1, 2, 3.1 to 3.4, 3.6, 6, 7; hearth's
  `docs/design/README.md`, 8.1, its JSON row.
- This plan: README.md (Rules: "The run's machine", "Test time", "One
  land at a time"); code-organization.md, 1, 6, 7; decisions.md, 1, 15;
  `findings/skein.md`, 2 and 4; `findings/skein-testing.md`, 3;
  `findings/measurements.md`.
- skein's `programming-model.md`: 3, 4.3, 5.4, 6.3, 10;
  `testing-strategy.md`: 2.1, 2.4, 3, 4.1, 6, 8 (by number only).

## What the design asks

- **An accessor** (3.6, 7): `position()` says where the last answer is;
  three counters, no heap, `worst_case` unchanged; no collector positions.
- **Its fields and lines:** offset "from 0", line "one more than the
  line feeds before it", column in bytes; "Only a line feed ends a line."
- **Where and when:** the first byte of a `Token`, `Long` or `Skipped`;
  after `Failed`, where what is at fault starts (3.6's table); held "from
  an answer until the next demand"; offset 0, line 1, column 1 before the
  first answer and after `restart`.
- **The reference** (6) counts them on its own and every run is checked
  against it; expectations record no positions; step tests write each
  position out by hand.

## Choices this plan makes

- **Three increments,** tokens, failures outside strings, a string's
  failures.
- **A `length` of `u32::MAX` has no `worst_case`** (`None`, as a `chunk`
  of 0) **and `new` asserts against it** (json.md, 3.3): the end's line
  and column, at most `length + 1`, stay within `u32` (3.6).
- **`Position` and the line counter are in
  `crates/skein-json/src/tokenizer/position.rs`,** re-exported as
  `skein_json::tokenizer::Position` (json.md, 2), not at the root;
  `tokenizer.rs` (988 lines at the base) gains call sites and
  `Step::Fail`'s offset.
- **What is held:** `read` (exists, :508); `Lines { line, start }`, the
  line being read and its first byte's offset; `pending`, where the token
  being read starts (a `Skip`'s value keeps its first byte); `answer`,
  what `position()` returns, set only as an answer goes up.
- **A line feed is counted where whitespace is consumed:** `significant`'s
  whitespace path (:792), and where a number's whitespace terminator is
  dropped (`number_byte`, :926). Strings and literals count none.
- **Each failure states its offset where it is decided**
  (`Step::Fail(Error, u32)`) and takes line and column from the line
  being read: no token spans two lines (3.6).
- **The reference keeps `parse`** (`reference.rs:25`, 8 calls) and
  `Decoded`; `reference::reading` adds the positions from offsets its
  descent records and one forward count of line feeds. **The world**
  records `position()` after each answer and asserts at each demand that
  it has not moved; the transcripts' whole-run test moves from
  `world::run` to `world::check`; `Stream`'s place is checked against the
  bytes the side below delivered.

## Rules that bind it

- **The gate** (README, Rules; decisions.md, 1): each increment lands
  with `flock ~/src/rust/tmp/hearth/render/land-skein.lock land --gate
  '<the README's command, whole>'`, its commit body saying so in one
  line; no probe of the machine.
- **Test time** (README, Rules): this session's allotment is 0.4 s of
  skein's focused suite (6.46 s) and 1 s of its fuzzy suite (41.84 s)
  (skein 02: 0.4 s / 1 s; 03: 0.3 s / 0.5 s; 04: 0.4 s / 1.5 s; 05:
  0.4 s / 2 s); each landing's report gives both from its `Gate-Result`
  line. The json sweeps that call `world::check` took 15.04 s, 6.97 s and
  6.33 s at the base (`check_demands`' 1.36 s): add no sweep or second
  parse, only an offset per token, one line-feed count per run, one
  `position()` per answer; the new world tests run 16 seeds at most. A
  sweep flagged `SLOW` (past 20 s) is made cheaper before its increment
  lands; no budget is raised.
- **Literals** (code-organization.md, 6): each position in a test is
  counted by hand; the reference shares no code with the tokenizer but
  the `Position` type.

## Increments

### 02.1 Where each token is

Scope: `Position`, `Tokenizer::position`, the line counter; the first
byte of each `Token`, `Long` and `Skipped`; the start before the first
answer and after `restart`; held until the next demand (3.6 but its
`Failed` table). After `Failed` and `Done` it stays the last token's
until 02.2.
Needs: nothing.
Files: `crates/skein-json/src/tokenizer/position.rs` (new); `tokenizer.rs`:
the module and re-export, `Document` (:499), `new` (:279), `reset`
(:323), `position`, the sites in `significant` (:791), `value` (:828),
`begin_string` (:855), `end` (:882), `number_byte` (:908),
`complete_token` (:668), `settle` (:655), `next` (:600), `worst_case`
(:96), `new` (:279), and a "# Positions" module doc. Tests: `src/tests.rs` (`mod position;`,
`placed`), `src/tests/position.rs` (new); world: `tests/json/src/`
`reference.rs` and `world.rs`, `tests/json/tests/transcripts.rs` (:53),
`tests/json/tests/positions.rs` (new). None of the README's shared files.
`Token`, `Event`, `Error`, `Request`, `Limits`, `new`, `up`, `down`,
`worst_case` and `largest_demand` keep their signatures, so skein's 23
other users and smith's 3, jig's 3 and temper's 5 (`findings/skein.md`, 2)
compile unchanged; `size_of::<tokenizer::Tokenizer>()`
(`crates/skein-llm/src/openai/common.rs:69`) grows and is computed: `rg -n
'size_of::<[A-Za-z_:]*(Tokenizer|Collector)>' crates tests testing`
prints that one line at the base.
Shape: the sketches. `value`, `begin_string` (keys) and `end` set
`pending` at a token's first byte unless a `Skip` has started
(`Mode::Skip { start: Some(_) }`); `answer` takes `pending` where a
`Token` or `Skipped` (`complete_token`) or a `Long` (`settle`, `next`'s
`ByteThenEnd` arm) goes up; `reset` sets `Lines::FIRST` and
`Position::START`. The reference records each token's offset as it
pushes it; `demands` (:393) gives a position for each answer before its
`Done`. The world: `route` (:503) records `position()` after each answer
in `Run.positions`; `down` (:494) asserts it unchanged before a `Next`,
`Text` or `Skip`; `check` (:271) compares the received tokens' positions
with `reading.at`; `check_demands` (:615) compares all of them.
Test shape: `src/tests/position.rs`, on `src/tests.rs`'s `LIMITS` (:24),
`Machine` (:47), and `placed(document, limits, requests) ->
Vec<(Event, Position)>` beside `read` (:96): each answer with
`position()` right after it, the demands `requests` then `Next`; `read`
keeps its signature over it. Each under 20 ms. Also
`a_length_of_u32_max_has_no_worst_case_and_no_tokenizer` (`worst_case`
`None`; `Tokenizer::new` `#[should_panic]`). A row's document is a
Rust byte string's contents. Reference test:

```rust
//! Positions (json.md, 3.6): where each answer is, counted by hand.
use super::{LIMITS, number, placed};
use crate::tokenizer::{Event, Position};
const fn at(offset: u32, line: u32, column: u32) -> Position { Position { offset, line, column } }
const AFTER_WHITESPACE: [(&[u8], Position); 5] = [(b"1", at(0, 1, 1)), (b"   1", at(3, 1, 4)),
    (b"\t\n\n 1", at(4, 3, 2)), (b"\r\n\r\n1", at(4, 3, 1)), (b" \r 1", at(3, 1, 4))];
#[test]
fn a_token_is_at_its_first_byte_after_any_whitespace() {
    for (document, expected) in AFTER_WHITESPACE {
        assert_eq!(placed(document, LIMITS, &[]).first(), Some(&(Event::Token(number(b"1")), expected)));
    }
}
```
Then `a_column_counts_each_byte_of_a_wide_character` (the `1` of
`["\xc3\xa9",1]` (6, 1, 7), `["\xe2\x82\xac",1]` (7, 1, 8),
`["\xf0\x9f\x98\x80",1]` (8, 1, 9));
`a_number_ended_by_a_line_feed_stays_on_its_line` (`[1\n,2]`: (1, 1, 2),
(4, 2, 2), `]` (5, 2, 3)); `long_and_skipped_are_at_their_value_s_first_byte`
(`[\n"abcdef", {"k":\n[1]}]` by `Next`, `Text(2)`, `Skip`: `Long(6)` (2,
2, 1), `Skipped(10)` (12, 2, 11), `]` (22, 3, 5));
`the_position_holds_from_an_answer_until_the_next_demand` (`\n[`, then
an end while idle: (1, 2, 1) both times);
`before_the_first_answer_and_after_restart_it_is_the_start` (made; a
demand and a space unanswered; `restart` after `Truncated`). World,
focused, under 0.5 s, `tests/json/tests/positions.rs`, reference test:

```rust
//! Where the reference and the world place answers (json.md, 3.6 and 6).
use skein_json::tokenizer::{Limits, Position};
use skein_json_world::{reference, world::{self, Settings}};
const LIMITS: Limits = Limits { depth: 4, string: 16, number: 8, chunk: 4, length: 1 << 16 };
const DOCUMENT: &[u8] = b"{\r\n  \"caf\xc3\xa9\": [1,\n true]}"; // a CRLF, a LF, a two-byte key character
const fn at(offset: u32, line: u32, column: u32) -> Position { Position { offset, line, column } }
const TOKENS: [Position; 7] = [at(0, 1, 1), at(5, 2, 3), at(14, 2, 12), at(15, 2, 13), at(19, 3, 2), at(23, 3, 6), at(24, 3, 7)];
#[test]
fn the_reference_and_the_world_place_each_token_where_counted() {
    assert_eq!(reference::reading(DOCUMENT, &LIMITS).at, TOKENS);
    for seed in 0..16 { let mut rng = skein_lib::Rng::new(seed); let run = world::check(DOCUMENT, &Settings::calm(&mut rng, LIMITS), seed);
        assert_eq!(run.positions[..TOKENS.len()], TOKENS, "seed {seed}"); }
}
```
Proof: world, the highest layer that reads positions (nothing above
skein-json asks for one until hearth 03's `hearth-protocol-config`):
`heavy cargo nextest run -p skein-json-world -E
'test(=every_transcript_decodes_to_its_expectation_whatever_the_cuts_and_scans)'`
(`transcripts.rs:43`), now through `world::check`: every token of the 39
transcripts, the Anthropic message's 41 lines and the OpenAI completion's
46 among them, is at the line and column the reference counts, under
scans of 1 to 4096 bytes and six seeds. Bound: 0.5 s (0.085 s at the base).
Don't: add a field to `Token`, `Event` or `Error`, or change `Request`,
`Limits`, `worst_case` or `largest_demand`; allocate
(`every_entry_point_holds_no_more_than_its_worst_case_at_its_limits`,
`tests/json/tests/memory.rs:134`, passes unchanged); touch the collector
or a `.expect` file; edit `docs/design/`; add a fuzzy test or seeds.
Size: at most 105 lines.
Removes: nothing.

### 02.2 Where a failure outside a string is

Scope: after `Failed`, `Unexpected`, `Trailing`, `TooDeep`, `Number`,
`NumberTooLong`, `TooLong`, `Truncated` and `Stream`, by 3.6's table,
and after `Done`, the end of what was read, as for `Truncated` (3.6); a
string's five failures stay at the end of what was read until 02.3, as
`position()`'s doc says.
Needs: 02.1.
Files: `tokenizer.rs`: `Step` (:561), `fail` (:709) and their sites in
`delivered` (:747), `significant`, `value`, `start` (:869), `end`,
`number_byte`, `number_more` (:940), `ended` (:716), `up` (:364), `next`
(:615-618), the `Done` answer; `tokenizer/position.rs`. Tests: `src/tests/position.rs`;
world: `reference.rs` (an offset at each `Err`), `world.rs` (`check`, a
count of the bytes delivered), `tests/positions.rs`.
Shape: `Step::Fail(Error, u32)`, the offset where what is at fault
starts; `fail(document, error, offset, above)` sets `answer` to
`lines.at(offset)`. Offsets: `read - 1` for the byte refused and the
bracket; a literal's first differing byte, not its last; `token_start`
for `Number` and `NumberTooLong`, in a skip too; `read` for `TooLong`
(before the refused delivery), `Truncated`, `Stream`, `Done`, and the
string arm until 02.3. `Reading` gains `failed_at`; `check` compares it
with the outcome's position, one arm per error, the five string errors'
arms comparing nothing until 02.3, and a `Done` with `reference::position(
document, document.len())`. At a `Failed(Stream)` the world asserts
`position() == reference::position(sent, delivered)`.
Test shape: `src/tests/position.rs`, each a table in a `const` beside
it, a row's limits other than `LIMITS` named in it:
`unexpected_and_trailing_are_at_the_byte_refused` (`[}` (1, 1, 2),
`{"a" 1}` (5, 1, 6), `nul!` (3, 1, 4), `t\nue` (1, 1, 2), `Trailing` in
`{}\n {}` (4, 2, 2)); `too_deep_is_at_the_bracket_that_opens_past_the_depth`
(`[\n[\n[\n[\n[` (8, 5, 1)); `a_bad_or_too_long_number_is_at_its_first_byte`
(`01` (0, 1, 1), ` [1.e5]` (2, 1, 3), `[-\n]` (1, 1, 2), `-` (0, 1, 1),
`NumberTooLong` in `[123456789]` (1, 1, 2));
`too_long_is_at_the_end_of_what_was_read` (`[1,2]` at `length: 4` (4, 1,
5), `"abcdef"` at `length: 6` (5, 1, 6));
`truncated_is_where_the_next_byte_would_be` (`[1` (2, 1, 3), `[1\n` (3,
2, 1), `tru` (1, 1, 2), `"abcdef` (5, 1, 6));
`a_stream_failure_is_at_the_end_of_what_was_read` (`[`, `\n`, then
`Up::Failed`: (2, 2, 1)); `done_is_at_the_end_of_what_was_read` (`1` (1,
1, 2), `[1] ` (4, 1, 5), `[1]\n\n` (5, 3, 1)). World: `positions.rs`'s
`the_reference_and_the_world_place_each_failure_by_the_table` (the
`Trailing`, `TooDeep`, `[-\n]`, `[1\n` and `nul!` rows, through
`failed_at` and 16 calm seeds). Ceilings as 02.1's.
Proof: world, `heavy cargo nextest run -p skein-json-world -E
'test(=the_stream_ending_after_any_byte_decodes_as_that_prefix)'`
(`tests/json/tests/world.rs:51`): the 46-byte document cut after each
byte, three seeds each, fails `Truncated` where its next byte would be,
as the reference counts it, scans cut in its strings included. Bound:
0.5 s (0.007 s at the base).
Don't: change the order of failures at one byte (json.md, 7); work an
offset out from the error's kind after the fact.
Size: at most 80 lines.
Removes: nothing.

### 02.3 Where a string's failure is

Scope: `Escape`, `Surrogate`, `Utf8`, `Control` and `StringTooLong`, in a
string or a key, at "the first byte of the character or the escape at
fault" (3.6; 3.4), across scans.
Needs: 02.2.
Files: `crates/skein-json/src/string.rs` (`Buffer` :24, `decode` :126);
`tokenizer.rs` (the string arm, :773-777); `tokenizer/position.rs`.
Tests: `src/tests/position.rs`; world: `reference.rs` (`string` :228,
`character` :265, `escape` :287, `unicode` :304), `world.rs`,
`tests/positions.rs`.
Shape: `Buffer` gains `begun: u32`, the offset of the character or
escape being decoded; `decode(text, piece, first: u32, out)` takes the
offset of `piece`'s first byte and sets `begun` at each byte read in
`Text::Plain`: a character's first byte, an escape's backslash, a high
surrogate's backslash kept through `Text::High`; a pair is one character
(3.4, 3.6), so a pair's second escape sets nothing. The string arm
fails at `document.text.begun`.
`string::next` (the writer's, `writer.rs:275`) and `Text` are unchanged.
The reference passes each escape's backslash and each character's first
byte to the failures it returns.
Test shape: `src/tests/position.rs`: `a_bad_escape_is_at_its_backslash`
(`"ab\\x"` (3, 1, 4), `"\\u12G4"` (1, 1, 2), the key of `{"k\\q":1}`
(3, 1, 4)); `a_lone_surrogate_is_at_its_half_s_backslash` (`"a\\uDC00"`
(2, 1, 3), `"\\uD800"` (1, 1, 2), `"x\\uD800\\u0041"` (2, 1, 3),
`"\\uDC00\\uD800"` (1, 1, 2)); `invalid_utf8_is_at_the_first_byte_of_its_character`
(`"ab\xff"` (3, 1, 4); `"abc\xe2\x28\xa1"`, its lead in the scan before
(4, 1, 5); `"\xc3"` (1, 1, 2)); `a_control_character_is_at_its_own_byte`
(`[\n"a\x01"]` (4, 2, 3)); `a_string_past_its_limit_is_at_the_character_or_escape`
(at `string: 4`: `"abcde"` (5, 1, 6), `"a\xF0\x9F\x98\x80"` (2, 1, 3),
`"abcd\\n"` (5, 1, 6), the key of `{"abcde":1}` (6, 1, 7));
`a_surrogate_pair_is_one_character_at_its_high_half_s_backslash`
(`"\\ud83d\\u12G4"` (1, 1, 2); at `string: 4`, `"a\\ud83d\\ude00"` (2, 1,
3)). World: these rows join `positions.rs`'s
table. Ceilings as 02.1's.
Proof: world, `heavy cargo nextest run -p skein-json-world -E
'test(=every_transcript_decodes_to_its_expectation_whatever_the_cuts_and_scans)'`
again: the hostile transcripts' bad and short escapes, lone and swapped
surrogates, invalid and overlong UTF-8, control character and strings
too long each fail at the first byte of the character or escape at
fault, as the reference counts it, under scans of 1, 2, 3, 7, 64 and
4096 bytes. Bound: 0.5 s.
Don't: change what a string decodes to, or the order of its failures.
Size: at most 40 lines.
Removes: 02.2's string arm passing `read`, and `check`'s five arms that
compare nothing.

## Interface sketches

`Position` crosses into hearth 03 (`hearth-protocol-config`), field for
field; the rest is skein-json's or its world's.

```rust
// crates/skein-json/src/tokenizer/position.rs, as skein_json::tokenizer::Position (02.1)
/// Where the tokenizer's last answer is in the document (json.md, 3.6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Position {
    /// The bytes delivered before it, from 0.
    pub offset: u32,
    /// From 1: one more than the line feeds before it.
    pub line: u32,
    /// From 1, in bytes: one more than the bytes since the last line feed.
    pub column: u32,
}
impl Position {
    /// Offset 0, line 1, column 1: before the first answer, and after `restart`.
    pub(crate) const START: Position;
}
/// The line being read: its number, from 1, and its first byte's offset.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Lines { line: u32, start: u32 }
impl Lines {
    pub(crate) const FIRST: Lines;
    /// The line feed at `offset` was read as whitespace: a line starts after it.
    pub(crate) fn feed(&mut self, offset: u32);
    /// Where the byte at `offset`, on the line being read, is.
    pub(crate) fn at(self, offset: u32) -> Position;
}

// crates/skein-json/src/tokenizer.rs (Document gains lines: Lines; pending, answer: Position)
impl Tokenizer {
    /// Where its last answer is (json.md, 3.6).
    #[must_use]
    pub fn position(&self) -> Position;
}

// tests/json/src/reference.rs
/// What the reference reads of a document, and where.
pub struct Reading {
    /// The tokens and the outcome, as `parse` gives them.
    pub decoded: Decoded,
    /// Where each token of `decoded` is, in order.
    pub at: Vec<Position>,
    /// Where the failure is, when the outcome is `Failed` (02.2).
    pub failed_at: Option<Position>,
}
pub fn reading(document: &[u8], limits: &Limits) -> Reading;
/// Where the byte at `offset` of `document` is, by its line feeds (02.2).
pub fn position(document: &[u8], offset: usize) -> Position;
pub fn demands(document: &[u8], limits: &Limits, seed: u64) -> (Vec<Request>, Vec<Event>, Vec<Position>);

// tests/json/src/world.rs: Run gains (02.1)
/// Where each answer is, as `position()` said right after it: one per entry of `answers`.
pub positions: Vec<Position>,
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/skein grep -n 'pub fn position(&self) -> Position'
   origin/main -- crates/skein-json/src/tokenizer.rs` prints one line.
2. This prints 3:
   `git -C ~/src/rust/skein log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'skein-02\.[1-3]'`
3. `git -C ~/src/rust/skein diff --stat 7e8fb70 origin/main --
   crates/skein-json/src/collector.rs crates/skein-json/src/collector
   tests/json/transcripts` prints nothing.
4. `git -C ~/src/rust/skein grep -n 'world::run(' origin/main --
   tests/json/tests/transcripts.rs` prints nothing.
