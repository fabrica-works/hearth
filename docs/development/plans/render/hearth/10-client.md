# Session 10: `hearth-client-protocol-records`, render's records and their printer

The client protocol's records of `render`, version 1, their printer (text
for people, JSON lines for scripts, client/README.md 3.2) and the outcome's
exit code (3.3). A machine (decisions.md, 2): records in, bytes out, nothing
parsed, no render type named; `hearth-protocol-builder` routes it from
hearth 11 on and translates render's answers into its records. First the
thin story's records as text; then their JSON lines and the printer on
standard output's stream, which hearth 11 prints through; then the
resolution and the layer keys. The diff's records, the errors and the
refusal printed when the client cannot answer at all are hearth 40's (split
from this session, decisions.md, 20). The parts' fields, the other part
changes and the other error kinds are their sessions' (Choices, "Who adds
what").

## Read first

- hearth's `client/README.md`: 3.1 (the `render` row, the bullet "`render`
  and `diff` in M1"), 3.2, 3.3, 3.5. `render.md`: 3.8, 4.1 to 4.5, 5.7, 8.1
  (its last two bullets), 9.2, 10 (the bullet on "the printer's bounds").
  `store.md`, 3.1; `format.md`, 3.4 and 4.3; `README.md`, 4.2.
- This plan's `decisions.md`, 2 and 8.
- skein's `json.md`, 4 (`skein_json::writer::Encoder`); `http.md`, 4.3 and
  4.4, with `crates/skein-http/src/sse/writer.rs`, the printer's model, and
  its tests' `Machine` (`src/tests/writer.rs:22`); `lib.md`, 7;
  `programming-model.md`: 3, 4, 4.3, 6.3, 7, 8, 10; `testing-strategy.md`:
  2.1 (by number only).

## What the design asks

- **A machine prints typed records** (client/README.md, 3.2: "The domain never
  formats"; render.md, 8.1: "a machine that writes the spec, the diff and
  the errors as text and as JSON lines").
- **Text:** "aligned columns, keys and digests cut to 12 hex digits"; the spec
  is 9.2's block, with its resolution `needed` or `known` and then "a last
  line gives the layer keys".
- **JSON lines:** "The first line is `{"record":"version","client":1}`; each
  line after is one record, its type in `record` (`spec`, `part`, `change`,
  `error`, `outcome`, …) and its fields under the names render.md gives
  them …, in snake case. Digests and keys are 64 lowercase hex digits, modes
  four octal digits as text"; "records and fields are only added".
- **Exit codes** (3.3): 0, 1, 2, 64, 75; "The code comes from the answer's
  terminal record, whose outcome the protocol layer maps to a code by one
  total function".

## Choices this plan makes

- **The crate is `hearth-client-protocol-records`** (decisions.md, 8;
  README.md 4.2's `hearth-protocol-client`
  renamed). Its root holds `CLIENT_VERSION`.
- **It depends on `skein-lib` and `skein-json`,** whose writer encodes
  each JSON line, as `hearth-protocol-config` composes its tokenizer: its
  role is "machine: lib, skein-json" (code-organization.md). Digests are
  `[u8; 32]`, `hearth_lib::encode::Digest`'s field, so the crate needs no
  `hearth-lib` and the session nothing at start.
- **A record is a printed unit:** a target's spec whole (meta and
  parts), a target's change, an error, the terminal record (the last
  three: hearth 40). JSON prints
  a spec as a `spec` line, then a `part` line a part, `meta` first. The
  printer prints in the order it is given; it sorts, bounds, folds and
  counts nothing.
- **JSON layout:** `record` first, then the frame (a spec's `target`,
  `commit`, `digest`; a part's `target`, `name`, `digest`; a change's
  `target`, `kind`; an error's `kind`, `class`, `at`, `also`, `target`,
  `task`); a part's own record, a change's or an error kind's fields
  nested under its name, so that an exports task named `digest` collides
  with nothing; inside, store.md 3.1's shape: a variant with fields is
  its fields and `kind`, maps objects, sets arrays, an absent optional
  field left out, fields in key order.
- **Text reproduces the design's lines byte for byte,** quoted in the
  tests; where it shows none, the nearest shown form (the plan decides within render.md 9.2's forms).
  Columns count bytes: labels 13 wide, a tree entry's kind at 27, a
  change's rebuild and a changed file's mode at 52; a column's width is
  max(its number, longest cell + 2), the entry's 2-space indent in the cell.
- **Version 1's layout is fixed once it lands** (client/README.md 3.2):
  `meta` a `part` line, a part's fields under its name, variants by
  `kind`, a diff's `from`, `to`, counts and `left_out` in its `outcome`
  line, no `diff` record. **Every part M1 builds has its `part` line, an
  empty one as its record with empty fields, in render.md 4.1's order.**
- **Text the design shows none of** is the plan's, within 9.2's forms: a
  blank line before a spec after any record and before a block of another
  type; `member <m>` between base and environment; `placement container on
  <host>`; columns widened past the examples'; no line for a render's
  terminal record. An unknown target is an error of the answer among the
  targets that rendered; `Usage` ranks worst (client/README.md 3.1, 3.3).
  A record `Refused` (past `record`, or a path not UTF-8 in JSON) is
  hearth 11's to print as an error (render.md 6.1, 7.2).
- **Outcomes take 3.3's words:** `Done`, `Negative`, `Failed`, `Usage`,
  `TryAgain` (JSON `done`, `negative`, `failed`, `usage`, `try_again`).
  `code`, 3.3's one total function, lives here; render decides the
  outcome, the worst it reached, and the protocol layer maps it to the
  terminal record (hearth 11's Choices, "render decides the outcome; the
  protocol layer maps it"; 11.1 tests the fold); the shell exits with `code`.
- **The printer is skein-http's SSE writer's shape:** `Request` down, one
  terminal `Event` a record, `up` and `down` with `MAX_OUT`, room demanded
  in chunks of at most `Limits::chunk`.
- **hearth 11 needs 10.2, not 10.1:** it prints through the printer; 10.1
  holds the records and their text, 10.2 the JSON lines and the printer.
- **A part is one variant and one file:** `PartRecord`'s methods in
  `records.rs` (`name`, `text`, `json`) hold the only matches over it, and
  10.1's forms print every part's `<name> part <digest>` line and frame
  from them; a session adds a part with its variant and those three arms
  in `records.rs`, a `mod` line in `src/parts.rs`, and its own
  `src/parts/<name>.rs` (its record, its lines, its JSON fields). No part
  record lands before the session that produces it.
- **Who adds what** (code-organization.md, 3): hearth 17 the packages part
  (its line ending with the spec's resolution, 9.2) and the boot part; 18
  config's links, directories, groups, users, members, presets, tmpfiles
  and scripts, and the data part; 19 the secrets, software and live parts,
  and the `ids` kind; 20 the exports part (and its values' JSON depth), and
  the `exported` kind; 12 the `decode` kind (it needs 40.2); 42 the
  `repository`, `composition` and `tool` kinds (`tool` for git's failure,
  with its standard error's tail); 13 `schema`; 15 and 16 `evaluation` (14
  emits none); 21 `reserved`; 22 `collision`; 24 `resolution`,
  mmdebstrap's failures as 42's `tool` kind, and render's resolution and
  keys into 10.3's field; 25 config's tree's change, meta's change and the packages change's
  `resolution`, and each part session after 25.1 (17 to 20) its own part's
  change; 29 what the goldens show missing.

## Rules that bind it

- **Version 1 is this session's** (README, Rules, "Format versions";
  code-organization.md, 3): a record, field or JSON name that lands is
  version 1's for good; later sessions add, and rename or remove none.
- **A machine in the strict subset:** `skein-lib` and `skein-json` only; no
  `String`, `Vec`, `format!` or `core::fmt::Write` (`clippy.toml`): text is
  UTF-8 in `Box<[u8]>`, numbers through `skein_lib::Decimal`, each box
  measured first and filled by a `skein_lib::Writer` (programming-model.md, 8).
- **Expected text and JSON are literals** (code-organization.md, 6);
  fixtures' digests are byte literals; tests gather bytes in a
  `skein_lib::Writer` or `List`, never a `Vec`.

## Increments

### 10.1 The thin story's records, as text
Scope: the crate; `CLIENT_VERSION`; `Spec` with its `meta` (render.md,
4.4) and a `config` part of `file` entries (4.2); the terminal record;
`Outcome`, `code` (client/README.md, 3.3); the text form (9.2): target,
placement and spec lines, `<name> part <digest>`, a tree entry a line,
a blank line before a spec that follows a record, nothing for a
render's terminal record (the plan's choice, Choices).
Needs: nothing.
Files: new `crates/hearth-client-protocol-records/`: `Cargo.toml`;
`src/lib.rs`, `limits.rs`, `records.rs` (README, Shared files),
`parts.rs`, `parts/meta.rs`, `parts/config.rs`, `outcome.rs`, `form.rs`,
`form/text.rs`; `src/tests.rs`, `src/tests/{fixtures,text,outcome}.rs`.
Shared: the root `Cargo.toml` (`members`; `skein-lib`, `skein-json` in
`[workspace.dependencies]`) and `Cargo.lock`.
Shape: the sketches. `form/text.rs`'s `Lines`: a sink measuring or
writing, the column so far, `pad_to`, `hex12`, `mode`, `decimal`; the
forms print a part's line from `PartRecord::name`, then its
`PartRecord::text` (Choices, "A part is one variant and one file").
Test shape: `src/tests/fixtures.rs` (new): `digest(prefix: [u8; 6]) ->
[u8; 32]` (the prefix, then zeros); `x1_prod() -> Box<Spec>` with 9.2's
values (spec `9a0c5e1f2b3d`, config `e7c2a0d95b13`, `/etc/hostname`
`3f1c2ab07e44`), meta `c4d3a0b1e2f5`, commit `1a2b3c4d` then 32 zeros;
`X1_PROD_TEXT`, the reference's literal. Limits are `limits::SHIPPED`.
Unit tests, each under 20 ms. Reference (`src/tests/text.rs`):

```rust
#[test]
fn a_spec_prints_its_meta_and_config_as_render_md_shows() {
    let printed = form::print(Form::Text, &Record::Spec(fixtures::x1_prod()), None, &limits::SHIPPED);
    let expected = concat!(
        "target       x1-prod   base x1   environment prod\n",
        "placement    metal efi 7th-gen\n",
        "spec         9a0c5e1f2b3d\n",
        "config       part e7c2a0d95b13\n",
        "  /etc/hostname            file 0644 root:root 3f1c2ab07e44\n",
    );
    assert_eq!(printed.as_deref(), Ok(expected.as_bytes()), "render.md, 9.2's lines");
}
```

Then `a_digest_prints_its_first_twelve_lowercase_hex_digits`,
`a_mode_prints_as_four_octal_digits` (0o644, 0o4755),
`a_path_past_the_column_widens_it_for_every_entry`,
`a_second_spec_follows_a_blank_line`,
`the_terminal_record_of_a_render_prints_nothing`,
`a_record_past_its_limit_is_refused_too_long` (with `record: 64` x1-prod is
refused; with SHIPPED it passes). `src/tests/outcome.rs`:

```rust
const CODES: [(Outcome, u8); 5] =
    [(Outcome::Done, 0), (Outcome::Negative, 1), (Outcome::Failed, 2), (Outcome::Usage, 64), (Outcome::TryAgain, 75)];

#[test]
fn each_outcome_exits_with_its_client_readme_code() {
    for (outcome, expected) in CODES {
        assert_eq!(outcome::code(outcome), expected, "client/README.md, 3.3's table");
    }
}
```

Proof: unit layer: nothing calls the crate until hearth 11's thin story,
which needs 10.2 (its binary prints through 10.2's printer), at the
process layer. Run once: `heavy cargo nextest run -p
hearth-client-protocol-records -E 'test(=tests::text::a_spec_prints_its_meta_and_config_as_render_md_shows)'`.
Bound 2 s.
Don't: name a type of render or of another machine; check the records'
order; print the commit in text (9.2 does not).
Size: at most 300 lines (a record's codec: within 2.5 times).
Removes: nothing.

### 10.2 The JSON lines, and the printer on standard output's stream
Scope: 10.1's records as JSON lines: the version line before the
answer's first record, a `spec` line, a `part` line for meta and each
part, the `outcome` line (client/README.md, 3.2), text not UTF-8 refused; the
printer, a step machine on a `lib::stream`'s write half (lib.md, 7): one
record at a time, in pieces of at most `Limits::chunk`; `Finish`;
`Close`.
Needs: nothing beyond 10.1.
Files: `src/form/json.rs`, `src/form.rs`, `src/parts/{meta,config}.rs`
(their `json`), `src/records.rs` (`PartRecord::json`), `src/printer.rs`,
`src/limits.rs` (`chunk`), `src/lib.rs`;
`src/tests/{json,printer}.rs`.
Shape: the sketches. Each line is one `Encoder` document, measured then
written (`skein_json::writer::Limits { depth: DEPTH, length: limits.record }`,
`const DEPTH: u32 = 16` in `src/form/json.rs`; hearth 20.2 sets it to 64), the
record's lines copied once into one box with their LFs. The printer's
states: idle (with what the stream told), printing (the box, how much
went down), finished, over, closed; a record that prints nothing is
`Printed` at once.
Test shape: fixtures reused. Reference (`src/tests/json.rs`):

```rust
#[test]
fn json_lines_start_with_the_version_and_give_digests_whole() {
    let printed = form::print(Form::JsonLines, &Record::Spec(fixtures::x1_prod()), None, &limits::SHIPPED);
    let expected = concat!(
        r#"{"record":"version","client":1}"#, "\n",
        r#"{"record":"spec","target":"x1-prod","commit":"1a2b3c4d00000000000000000000000000000000","digest":"9a0c5e1f2b3d0000000000000000000000000000000000000000000000000000"}"#, "\n",
        r#"{"record":"part","target":"x1-prod","name":"meta","digest":"c4d3a0b1e2f50000000000000000000000000000000000000000000000000000","meta":{"base":"x1","environment":"prod","placement":{"firmware":"efi","hardware":"7th-gen","kind":"metal"},"target":"x1-prod"}}"#, "\n",
        r#"{"record":"part","target":"x1-prod","name":"config","digest":"e7c2a0d95b130000000000000000000000000000000000000000000000000000","config":{"tree":{"/etc/hostname":{"content_digest":"3f1c2ab07e440000000000000000000000000000000000000000000000000000","group":"root","kind":"file","mode":"0644","owner":"root"}}}}"#, "\n",
    );
    assert_eq!(printed.as_deref(), Ok(expected.as_bytes()), "client/README.md, 3.2: version first, digests whole");
}
```

Then `the_terminal_record_gives_its_outcome_by_name`,
`a_path_that_is_not_utf8_is_refused_as_text`. `src/tests/printer.rs`
holds a `Machine` after skein-http's (queues sized from the `MAX_OUT`s,
asserted after each call; room granted as asked; pieces gathered in a
`Writer`). Reference:

```rust
#[test]
fn a_record_goes_down_in_pieces_of_at_most_a_chunk() {
    // 16 bytes, against SHIPPED's 64 KiB, so that x1-prod's text takes several pieces.
    let mut machine = Machine::new(Form::Text, Limits { chunk: 16, ..limits::SHIPPED });
    let printed = machine.print(Record::Spec(fixtures::x1_prod()));
    assert_eq!(&*printed.bytes, fixtures::X1_PROD_TEXT.as_bytes(), "the spec's lines, whole and in order");
    assert!(printed.largest <= 16, "no piece past the chunk");
    assert_eq!(&*printed.events, &[Event::Printed], "one Printed answers the record");
}
```

Then `only_the_answers_first_record_follows_the_version_line`,
`a_record_that_prints_nothing_is_printed_at_once`,
`finish_ends_the_stream_once_nothing_is_printing`,
`a_failed_stream_fails_the_record_being_printed`,
`close_withdraws_the_demand_and_answers_closed`. Each under 20 ms.
Proof: unit layer: hearth 11's thin story, which needs this increment,
connects it, its binary's
`hearth render` printing through `hearth-protocol-builder` and this
printer to standard output (the process layer). Run once: `heavy cargo
nextest run -p hearth-client-protocol-records -E 'test(=tests::printer::a_record_goes_down_in_pieces_of_at_most_a_chunk)'`.
Bound 2 s.
Don't: write JSON by hand; print a version line in text; demand room
before a record needs it; read the stream below.
Size: at most 290 lines.
Removes: nothing.

### 10.3 The resolution and the layer keys
Scope: the spec's resolution (render.md, 3.8, 5.7): its key, `needed`
or `known`, and when known the three layer keys (4.5); in text the
`keys` line last (9.2), in JSON `resolution` in the `spec` line, keys
whole. The packages line's `resolution <key> needed|known` is hearth
17's, with the packages part (Choices, "Who adds what").
Needs: nothing beyond 10.2.
Files: `src/records.rs` (`Spec`'s `resolution`, `Resolution`, `Keys`),
`src/form/text.rs`, `src/form/json.rs`; `src/tests/{text,json,fixtures}.rs`.
Shape: the sketches. The keys line: `keys` in the label column, then
`packages <k>   software <k>   config <k>`, cut to 12 hex digits
(the plan's choice).
Test shape: in 10.1's and 10.2's files, after their references;
`fixtures::with_resolution(spec, resolution)`, the key `0c7d9e41c0d2`
(9.2's). `a_known_resolution_prints_the_keys_line_last`,
`a_needed_resolution_prints_no_keys_line`,
`json_gives_the_resolution_and_its_keys_whole`. Each under 20 ms.
Proof: unit layer: nothing produces a resolution until hearth 24, which
proves it on the thin story's path (`--resolve`). Run once: `heavy cargo
nextest run -p hearth-client-protocol-records -E 'test(=tests::text::a_known_resolution_prints_the_keys_line_last)'`.
Don't: add a part's record; print the resolution on a line of its own.
Size: at most 90 lines.
Removes: nothing.

## Interface sketches

`hearth-protocol-builder` (hearth 11) owns a `Printer`, hands it one
`Request::Print` at a time, the terminal record's outcome the one render
decided (hearth 11's Choices); `Printed`, `Refused` or `Failed` ends each
print, `Closed` the printer. Every type derives `Clone, PartialEq, Eq,
Hash, Debug` (and `Copy` where it can); text is UTF-8; sequences keep
render's order (format.md, 4.3). hearth 40 adds to `Record`, `Type` and
`Terminal`.

```rust
// crates/hearth-client-protocol-records/src/lib.rs (10.1)
/// The client protocol's major version: the version line's `client` (client/README.md, 3.2).
pub const CLIENT_VERSION: u32 = 1;

// src/limits.rs (10.1; chunk 10.2)
pub struct Limits {
    /// The most bytes one record prints, LFs included: past it, `Refusal::TooLong`.
    pub record: u32,
    /// The most room the printer demands at once, at least 1.
    pub chunk: u32,
}
/// 64 MiB a record (render.md 10: a guide, set from measurements); a pipe's 64 KiB a chunk.
pub const SHIPPED: Limits = Limits { record: 67_108_864, chunk: 65_536 };

// src/records.rs (10.1; Change 40.1, Error and Startup 40.2): shared
pub enum Record {
    /// A target's spec, whole (render.md, 3.8): a `spec` line, then a `part` line each.
    Spec(Box<Spec>), // boxed: clippy's large_enum_variant, once 10.3 adds `resolution`
    /// The answer's last record (client/README.md, 3.3): an `outcome` line.
    Terminal(Terminal),
}
/// A record's type: JSON's `record`, and the blank lines between blocks.
pub enum Type { Spec, Terminal }
pub struct Spec {
    /// The meta part (render.md, 4.4); its `target` is the spec's.
    pub meta: Meta,
    pub meta_digest: [u8; 32],
    /// As git names it, 40 or 64 lowercase hex digits (render.md, 4.5: beside every digest).
    pub commit: Box<[u8]>,
    pub digest: [u8; 32],
    /// The other parts, in render.md 4.1's order, each once its session lands: every part M1 builds, an empty one too.
    pub parts: Box<[Part]>,
    // 10.3: pub resolution: Option<Resolution>, absent until render gives it (hearth 24)
}
pub struct Part { pub digest: [u8; 32], pub record: PartRecord }
/// A part's own record (render.md, 4.2 to 4.4): Config here; each of the others with
/// the session that produces it (hearth 17 to 20), as one variant and its arms below.
pub enum PartRecord { Config(Config) }
impl PartRecord {
    /// render.md, 4.1's name: the text line's label, JSON's `name` and nesting key.
    pub(crate) fn name(&self) -> &'static [u8];
    /// The part's lines below its `<name> part <digest>` line.
    pub(crate) fn text(&self, lines: &mut Lines);
    /// The part's own record, nested under its name.
    pub(crate) fn json(&self, json: &mut Encoder);
}
pub struct Terminal { pub outcome: Outcome /* 40.1: diff: Option<Tally>; 40.2: left_out: u32 */ }

// src/parts/meta.rs, src/parts/config.rs (10.1)
pub struct Meta { pub target: Box<[u8]>, pub base: Box<[u8]>, pub member: Option<Box<[u8]>>, pub environment: Box<[u8]>, pub placement: Placement }
pub struct Placement { pub kind: PlacementKind, pub firmware: Option<Firmware>, pub hardware: Option<Box<[u8]>>, pub on: Option<Box<[u8]>> }
pub enum PlacementKind { Metal, Vm, Container }
pub enum Firmware { Efi, Bios }
pub struct Config { pub tree: Box<[TreeEntry]> }
pub struct TreeEntry { pub path: Box<[u8]>, pub node: Node }
/// A tree entry (render.md, 4.2); `Link` and `Directory`: hearth 18. `mode` at most 0o7777.
pub enum Node { File { content_digest: [u8; 32], mode: u16, owner: Box<[u8]>, group: Box<[u8]> } }

// src/outcome.rs (10.1; `of`: 40.2)
/// What a command reached (client/README.md, 3.3): the terminal record's `outcome`.
pub enum Outcome { Done, Negative, Failed, Usage, TryAgain }
/// 0, 1, 2, 64, 75.
#[must_use] pub fn code(outcome: Outcome) -> u8;

// src/form.rs (10.1; JsonLines 10.2)
pub enum Form { Text, JsonLines }
pub enum Refusal { TooLong, Text }
/// `record`'s lines, after a record of type `after` (none: the answer's first).
pub fn print(form: Form, record: &Record, after: Option<Type>, limits: &Limits) -> Result<Box<[u8]>, Refusal>;

// src/printer.rs (10.2): skein-http's sse::writer, for records
pub enum Request { Print(Record), Finish, Close }
pub enum Event { Printed, Refused(Refusal), Failed(Fault), Closed }
pub enum Waiting { Above, Room, Close, Nothing }
pub struct Printer { /* the form, the type last printed, the state */ }
impl Printer {
    #[must_use] pub fn new(form: Form, limits: &Limits) -> Printer;
    #[must_use] pub fn waiting(&self) -> Waiting;
}
pub struct MaxOut { pub above: u32, pub below: u32 }
pub const UP_MAX_OUT: MaxOut = MaxOut { above: 1, below: 2 };
pub const DOWN_MAX_OUT: MaxOut = MaxOut { above: 1, below: 1 };
pub fn up(printer: &mut Printer, env: &Env<Limits>, ev: Up, above: &mut Queue<Event>, below: &mut Queue<Down>);
pub fn down(printer: &mut Printer, env: &Env<Limits>, rq: Request, above: &mut Queue<Event>, below: &mut Queue<Down>);
/// One record's bytes: `Limits::record`.
#[must_use] pub fn worst_case(limits: &Limits) -> Option<u64>;

// src/records.rs (10.3)
pub enum Resolution { Needed { key: [u8; 32] }, Known { key: [u8; 32], keys: Keys } }
pub struct Keys { pub packages: [u8; 32], pub software: [u8; 32], pub config: [u8; 32] }
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-10\.[1-3]'` prints 3.
2. In a worktree at `origin/main`, `heavy cargo nextest run -p
   hearth-client-protocol-records` passes, its `Summary` naming at least
   20 tests.
3. `git -C ~/src/rust/hearth grep -n -E 'CLIENT_VERSION: u32 = 1'
   origin/main -- crates` prints one line, in
   `crates/hearth-client-protocol-records/src/lib.rs`.
