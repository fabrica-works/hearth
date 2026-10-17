# Session 40: `hearth-client-protocol-records`: the diff's records, the errors and the refusal

The rest of the client protocol's records of `render` and `diff`, version 1
(client/README.md, 3.2, 3.3), on hearth 10's crate: the diff's `change`
records, their text (render.md, 6.3) and the tally in the terminal record;
then the `error` record and its line (7.3), the count of errors left out, and
the `startup` record, the refusal printed when the client cannot answer at
all. Split from hearth 10 (decisions.md, 20). A machine (decisions.md, 2):
records in, bytes out, nothing parsed, no render type named. The other
parts' changes and error kinds are their sessions' (hearth 10, Choices, "Who
adds what").

## Read first

- hearth 10: Choices, Rules, its sketches and `src/tests/fixtures.rs`.
- hearth's `client/README.md`: 3.1 ("`render` and `diff` in M1"), 3.2, 3.3.
  `render.md`: 2.3 (its last row), 6.1, 6.3, 7.1 to 7.3, 8.1 (its last two
  bullets), 8.2, 10 (the bullet on "the printer's bounds"). `store.md`, 3.1;
  `format.md`, 4.3 and 5.9.
- This plan's `decisions.md`, 2 and 8.
- skein's `programming-model.md`: 3, 6.3, 7, 8, 10; `testing-strategy.md`:
  2.1 (by number only).

## What the design asks

- **Text** (render.md, 6.3, 7.3): the diff's target line, part-change line and
  hunks; errors are 7.3's lines, "aligned columns, keys and digests cut to 12
  hex digits".
- **Errors are the answer,** on standard output "after what did render";
  sorted and bounded by render (7.1; 2.3: "256, then a count"). "What keeps
  the client from answering at all (an unknown option, a repository or a
  store it cannot open) goes to standard error, as a record with `--json`,
  with 3.3's code" (3.2).
- **Exit codes** (3.3): "the worst outcome wins …: 2 if any error is a
  configuration error, else 75 if any is operational, else, for `diff`, 1 if
  anything differs, else 0"; a diff with a failed target "is then 2 or 75,
  never 1".

## Choices this plan makes

- **The refusal is a `startup` record** (client/README.md 3.2), not an error kind:
  the shell prints it alone on standard error through `form::print`, in the
  form `--json` asks wherever it stands in argv (hearth 32.2), JSON's
  version line first; it names its outcome, `usage` or `try_again`, and no
  `outcome` line follows.
- **Text the design shows none of** is the plan's, within 6.3's and 7.3's
  forms: `added spec <d>`, `removed`, `failed (from|to|both)`, `... <n>
  lines left out`, `1 more error`, `hearth: <message>`. **A hunk has three
  lines of context** (6.3 corrected to git's own). **`failed` names its
  side, `both` when each side has the target and neither rendered; it
  counts among the diff's targets, neither changed nor unchanged** (render.md
  6.1). **Errors left out** are counted once over a diff's two sides: `<n>
  more errors` after the last (7.1, 7.3); `left_out` in the `outcome` line.
- **The diff's JSON** follows hearth 10's layout: a `change` line, a change's
  fields nested under its name; the tally's fields in the `outcome` line,
  with no `diff` record (hearth 10, Choices; client/README.md 3.2).

## Rules that bind it

- **Version 1** (README, Rules, "Format versions"): a record, field or JSON
  name that lands is version 1's for good; later sessions add.
- **A machine in the strict subset** and **literals** as hearth 10's Rules:
  no `String`, `Vec` or `format!`; expected text and JSON are literals.

## Increments

### 40.1 The diff's records
Scope: a target's `change` (render.md, 6.1): `added { spec digest }`,
`removed`, `same`, `changed { parts, rebuild }`, `failed { side }`;
`rebuild`'s five; config's tree changes (added, removed, changed `{ path,
before, after }` with the two sizes and the hunks, 6.1 and 8.2); the
diff's tally in the terminal record, printed `2 of 11 targets change; 9
are unchanged.` after a blank line (6.3). Text: 6.3's target line, part
change line and hunks; a `same` target prints no line.
Needs: hearth 10.3.
Files: `src/lib.rs` (`mod diff`), `src/records.rs` (`Record::Change`,
`Type::Change`, `Terminal`'s `diff`), new `src/diff.rs`,
`src/parts/config.rs`, `src/form.rs`, `src/form/{text,json}.rs`;
`src/tests.rs` (`mod diff;`), `src/tests/{diff,fixtures}.rs`; the three
`Terminal` literals, `src/tests/json.rs:35`, `src/tests/printer.rs:90`
and `src/tests/text.rs:65`, which gain `diff: None`.
Shape: the sketches. JSON: `{"record":"change","target":…,"kind":
"changed","changed":{"parts":[…],"rebuild":{"kind":"from_config"}}}`;
the tally's fields in the `outcome` line, in client/README.md 3.2's
order (`from`, `to`, `targets`, `changed`, `unchanged`; `left_out` last,
40.2). Text, the plan's forms of render.md 6.1's names (6.3 shows two):
`rebuild from packages`, `rebuild from packages, resolution needed`,
`rebuild from software`, `rebuild from config`, `new slot only`,
`nothing to build`; the tally `<c> of <t> targets change; <u> are
unchanged.`, with `target` when `t` is 1, `changes` when `c` is 1, and
`1 is unchanged`. Lints: pedantic's `match_same_arms` joins arms of one
body with `|`; no `<[_]>::concat` or `join`, which make a `Vec`.
Test shape: `fixtures::wg0_changed()`, 6.3's carbon-prod change, its
hunk git's own (`--unified=3`) for a 13-line `wg0.netdev` whose line 12 is
the endpoint: 6.3's example, `@@ -9,5 +9,5 @@ ListenPort=51820`, from
format.md 8.6's template for carbon-prod. Reference:

```rust
#[test]
fn a_changed_file_prints_its_hunks_as_render_md_shows() {
    let printed = form::print(Form::Text, &Record::Change(fixtures::wg0_changed()), None, &limits::SHIPPED);
    let expected = concat!(
        "carbon-prod                                         rebuild from config\n",
        "  config   ~ /etc/systemd/network/wg0.netdev        0644 root:root\n",
        "               @@ -9,5 +9,5 @@ ListenPort=51820\n                [WireGuardPeer]\n",
        "                PublicKey=m3NY8…=\n                AllowedIPs=10.0.0.1/32\n",
        "               -Endpoint=194.164.162.157:51820\n               +Endpoint=194.164.162.158:51820\n                PersistentKeepalive=25\n",
    );
    assert_eq!(printed.as_deref(), Ok(expected.as_bytes()), "render.md, 6.3's carbon-prod block, three lines of context");
}
```

Then `nothing_to_build_prints_at_the_rebuild_column`,
`a_same_target_prints_no_line`,
`the_tally_ends_the_diff_after_a_blank_line`,
`a_binary_file_prints_no_hunk_line`,
`json_gives_a_change_with_its_rebuild_and_parts`,
`json_gives_the_tally_in_the_outcome_record`, and (render.md 6.3, 7.3: the plan's forms; the
target padded to column 52, as the reference) `an_added_target_prints_added_spec_and_its_twelve_digits`,
`a_removed_target_prints_removed`, `a_failed_target_prints_failed_and_its_side`
(`failed (to)`, `failed (both)`), `a_failed_target_counts_among_the_targets_only`
(the tally keeps `targets` above `changed + unchanged`), `hunk_lines_left_out_print_their_count` (`...
5 lines left out` at the hunk indent). Each under 20 ms.
Proof: unit layer: hearth 25's `hearth diff` connects it, at the process
layer. Run once: `heavy cargo nextest run -p hearth-client-protocol-records
-E 'test(=tests::diff::a_changed_file_prints_its_hunks_as_render_md_shows)'`.
Don't: compute a text diff or a tally; add another part's change.
Size: at most 280 lines.
Removes: nothing.

### 40.2 Errors, the count of those left out, and the refusal
Scope: the `error` record (render.md, 7.1): kind, `class`, `at`,
`also`, `target`, `task`; its line (7.3): `<path>:<line>:<column>: `,
`<target>: ` and `<task>: ` when present, the kind's message,
` (<also>)` last; the `guard` kind, 7.3's fourth line; `outcome::of`;
the count left out, in the terminal record, printed after the last
error (render.md 7.3); the `startup` record (client/README.md, 3.2:
"What keeps the client from answering at all"; Choices).
Needs: nothing beyond 40.1 (its `failed` change and these records meet
in a diff).
Files: `src/lib.rs` (`mod errors`, `mod startup`), `src/records.rs`
(`Record::Error`, `Record::Startup`, their `Type`s, `Terminal`'s
`left_out`), new `src/errors.rs`, new
`src/startup.rs`, `src/outcome.rs`, `src/form.rs`,
`src/form/{text,json}.rs`; `src/tests.rs` (`mod errors;`),
`src/tests/{errors,outcome,fixtures}.rs`; 40.1's three `Terminal`
literals, which gain `left_out: 0`, and `src/tests/json.rs`'s
`the_terminal_record_gives_its_outcome_by_name`, whose five rows end
`,"left_out":0}`.
Shape: the sketches. JSON: `{"record":"error","kind":"guard","guard":
{"message":…,"name":"modules"},"class":"configuration","at":{"column":7,
"line":22,"path":"tasks/boot/task.json"},"target":"carbon-lab",
"task":"boot"}`. `startup`: text `hearth: unknown option: --bogus`, or
`hearth: cannot open the store <path>: <error>`; JSON `{"record":
"startup","kind":"usage","outcome":"usage","usage":{"misuse":"option",
"word":"--bogus"}}`. JSON's `outcome` line always carries `left_out`,
last (render.md 7.3: "JSON's `outcome` record carries `left_out`"). In
text the terminal record prints `<n> more errors` (`1 more error`) right
after the last error, then, in a diff, the blank line and the tally.
Test shape: `fixtures::modules_guard()`, 7.3's fourth line. Reference:

```rust
#[test]
fn a_false_guard_prints_as_render_md_shows() {
    let printed = form::print(Form::Text, &Record::Error(fixtures::modules_guard()), Some(Type::Spec), &limits::SHIPPED);
    let expected = "\ntasks/boot/task.json:22:7: carbon-lab: boot: requires modules: no initramfs modules for this hardware\n";
    assert_eq!(printed.as_deref(), Ok(expected.as_bytes()), "render.md, 7.3, after a spec's blank line");
}
```

Then `an_error_without_a_target_leaves_out_its_prefix`,
`a_second_place_prints_in_parentheses_last`,
`a_place_without_a_position_prints_its_path_alone`,
`the_errors_left_out_print_their_count_after_the_last` (`44 more errors`),
`json_gives_an_error_with_its_class_and_place`,
`an_errors_class_gives_failed_or_try_again`,
`a_startup_refusal_prints_its_line_or_its_record_after_the_version`
(`Usage { Option, --bogus }`, both forms, the literals above), and
(render.md 7.3) `one_error_left_out_prints_1_more_error`,
`a_startup_refusal_for_the_store_or_repository_prints_its_line`
(`hearth: cannot open the store <path>: <error>`, `... the repository ...`;
without a path `hearth: cannot open the store: <error>`),
`a_startup_refusal_for_a_command_value_or_revision_prints_its_line`
(`hearth: unknown command: <word>`, `hearth: option needs a value: <word>`,
`hearth: malformed revision: <word>`). Each under 20 ms. That a diff with a failed target never exits 1 is render's
fold (hearth 25).
Proof: unit layer: hearth 12 connects its first kind on the thin story's
path (a broken document, printed by `hearth render`); hearth 16 the
guard; hearth 32.2's binary the `startup` record on standard error. Run
once: `heavy cargo nextest run -p hearth-client-protocol-records -E
'test(=tests::errors::a_false_guard_prints_as_render_md_shows)'`.
Don't: sort, bound or count errors (render does, 7.1); add a kind but
`guard`; print an `outcome` line after a `startup` record.
Size: at most 240 lines.
Removes: nothing.

## Interface sketches

Added to hearth 10's sketches (`Record` gains `Change` 40.1, `Error` and
`Startup` 40.2, and `Type` their types; `Terminal` gains `diff: Option<Tally>`
40.1 and `left_out: u32` 40.2). Every type derives `Clone, PartialEq, Eq,
Hash, Debug` (and `Copy` where it can); text is UTF-8; sequences keep
render's order (format.md, 4.3).

```rust
// src/outcome.rs (40.2; the rest is hearth 10.1's)
/// `Failed` for a configuration error, `TryAgain` for an operational one.
#[must_use] pub fn of(class: Class) -> Outcome;

// src/diff.rs (40.1): render.md, 6.1
pub struct Change { pub target: Box<[u8]>, pub change: TargetChange }
pub enum TargetChange { Added { spec_digest: [u8; 32] }, Removed, Same, Changed { parts: Box<[PartChange]>, rebuild: Rebuild }, Failed { side: Side } }
pub enum Side { From, To, Both }
pub enum Rebuild { FromPackages { resolution: Needs }, FromSoftware, FromConfig, SlotOnly, Nothing }
pub enum Needs { Known, Needed }
/// One part's change: config's tree here; meta's hearth 45's; each other part's its session's.
pub enum PartChange { Config(ConfigChange) }
pub struct ConfigChange { pub tree: Box<[EntryChange]> }
pub enum EntryChange {
    Added { path: Box<[u8]>, node: Node },
    Removed { path: Box<[u8]>, node: Node },
    Changed { path: Box<[u8]>, before: Node, after: Node, sizes: Option<Sizes>, hunks: Option<Hunks> },
}
pub struct Sizes { pub before: u64, pub after: u64 }
/// render.md, 8.2's `Hunks { lines: (kind, text), left out, binary }`.
pub struct Hunks { pub lines: Box<[HunkLine]>, pub left_out: u32, pub binary: bool }
pub struct HunkLine { pub kind: LineKind, pub text: Box<[u8]> }
pub enum LineKind { Header, Context, Removed, Added }
pub struct Tally { pub from: Box<[u8]>, pub to: Box<[u8]>, pub targets: u32, pub changed: u32, pub unchanged: u32 }

// src/errors.rs (40.2): render.md, 7.1
pub struct Error { pub kind: Kind, pub class: Class, pub at: Option<Place>, pub also: Option<Place>, pub target: Option<Box<[u8]>>, pub task: Option<Box<[u8]>> }
/// render.md, 7.2's kinds, each with its fields: Guard here, the others their sessions'.
pub enum Kind { Guard(Guard) }
/// A false `requires` entry (format.md, 5.9): its name and message.
pub struct Guard { pub name: Box<[u8]>, pub message: Box<[u8]> }
pub enum Class { Configuration, Operational }
pub struct Place { pub path: Box<[u8]>, pub position: Option<Position> }
pub struct Position { pub line: u32, pub column: u32 }

// src/startup.rs (40.2): what keeps the client from answering at all (client/README.md, 3.2)
pub enum Startup {
    /// `usage`, 64: an unknown command or option, an option without its value, a malformed revision.
    Usage { misuse: Misuse, word: Box<[u8]> },
    /// `try_again`, 75: the store or the repository not found, made or opened;
    /// `path` absent when nothing names it (hearth 32.2), `error` the system's words.
    Unopened { what: Opened, path: Option<Box<[u8]>>, error: Box<[u8]> },
}
pub enum Misuse { Command, Option, Value, Revision }
pub enum Opened { Store, Repository }
impl Startup {
    #[must_use] pub fn outcome(&self) -> Outcome;
}
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-40\.[1-2]'` prints 2.
2. In a worktree at `origin/main`, `heavy cargo nextest run -p
   hearth-client-protocol-records` passes, its `Summary` naming at least
   43 tests.
