# Session 11: the thin story: `hearth render` on a repository through real git

The plan's meeting point (development.md, 5.3; decisions.md, 2 and 6):
`hearth render` prints a target's spec from a scratch repository through real
git, the real decoders and printer. 11.1 render's skeleton in `tests/render`;
11.2 the builder's root and the protocol layer's git and documents; 11.3 the
service, the printer and the real loop, `tests/builder`. Hearth 32 adds static
files (32.1) and the binary (32.2).

## Read first

- `render.md`: 1, 2.1 to 2.3, 3.1, 3.2, 3.6, 4.1, 4.4 to 4.6, 7.1 to 7.3, 8.1
  to 8.4, 9.1, 9.2; `format.md`: 2.1, 2.3, 2.4, 3.1 to 3.4; `client/README.md`:
  2, 3.1 to 3.3, 3.5; `store.md`: 2.1, 2.2, 2.4, 3.1, 3.2; `testing.md`: 2.2,
  2.7, 2.9.
- This plan: `decisions.md`, 2 and 6; `findings/skein-io.md`,
  `skein-testing.md` (1, 6, 7); the Shapes and sketches of `01-lib.md`,
  `02-git.md`, `03-json.md` (`Placement`) and `10-client.md`.
- skein: `examples/echo/service/src/lib.rs` (`iterate`), `examples/echo/shell/
  src/{lib.rs, main.rs}`, `crates/skein-shell/src/drive.rs:53`,
  `tests/echo/tests/real.rs`; smith's `crates/smith-local-protocol/src/
  git_child.rs`; `programming-model.md`: 2 to 4.5, 5.2, 6.3, 9;
  `testing-strategy.md`: 2.2, 2.8, 2.9, 6 (by number only).

## What the design asks

- **One listing, then the blobs** (render.md, 2.2, 3.1; format.md, 3.1): a
  target per `hosts/<base>/<base>-<env>.json` whose `envs/<env>.json` exists.
- **Parts and digests** (4.4 to 4.6); **real tiers refuse** (testing.md, 2.10).
- **The client** (client/README.md, 3.1, 3.3, 3.5): in-process; the code from
  the terminal record.

## Choices this plan makes

- **Answers leave by `resume`,** one an entry (`MAX_OUT`, programming-
  model.md, 2): each target's spec in name order, then `Rendered`. The
  root holds one call at a time; a second is a bug, asserted.
- **render.md 8.2's records, two fields more:** `Read` carries the
  `repository` and each blob its listed `size` (2.2), checked against
  render's limit before it asks (2.3). `List` ends in `Listed`,
  `Unresolved` (02.1) or `Failed`. A read's blobs answer in the order asked,
  each `(oid, kind)` once, the last its terminal; the field is `kind: As`.
- **Named targets and revisions** (client/README.md 3.1, 3.3): the known
  render, the unknown one is an error of the answer, `Usage` ranks worst,
  each target once in name order; a revision that names no commit is `Usage`
  (64). **Standard output that breaks** is 75 and a record refused
  leaves its spec out, `Failed` (render.md 6.1, 7.2).
- **render decides the outcome; the protocol layer maps it:** `Rendered
  { outcome }` is the worst reached (`Outcome::worst`); `outcome::code`
  of the terminal record is the exit (3.3).
- **No error record in hearth 11:** a failure ends the call with its
  outcome and leaves out the targets it fails (a refused document fails
  those that compose it; no placement fails, format.md, 3.4); `Failed`
  from git ends the call `TryAgain`. The error records are hearth 12's
  (42.1's `tool` kind for git, 24's for mmdebstrap); git's standard error
  goes to `/dev/null` until 42.1's `tool` kind keeps its tail.
- **Documents as the thin story reads them:** `placement` (11.1), each
  task's name and whether `null` removes it (32.1). A translation
  destructures config's structs whole, no `..`, naming what render takes
  later (`debian: _` until hearth 17, `value: _`, `members: _` until 12),
  and matches each source enum with no wildcard (Done when 4);
  `src/tests/translate.rs` holds each reverse, a total match (a variant
  with no counterpart yet `None`, naming its session).
- **git runs in the store's root** (`Spawn::root`, `dir` empty), its
  `HOME=/dev/null` (render.md 2.2: no home is made), its repository absolute; the shell
  checks `--at` with 02.1's `invocation::revision` (64), the protocol layer
  again (`Unresolved`). Scratch repositories are data in
  `tests/builder/src/fixtures.rs`, not `testing/repos/`.
- **Transitional, each with its end** (development.md, 5.3): the cache
  file without its kind-5 record, the store's lock or `store.record`
  (hearth 23 and 33); `Client::worst_case`, none until hearth 28; `records::
  Spec::parts` empty (32.1), and its `resolution` `None` (24.1); the
  terminal record's `diff: None` and `left_out: 0`, which 11.3's translation writes (or the later of 11.3 and 40.1, 40.2
  edits in), hearth 22 filling `left_out`.

## Rules that bind it

- **The meeting point and shared files** (README, Rules): from 11.3 every
  increment proves on the real loop's `hearth render`; a variant added to a
  vocabulary crossing `hearth-protocol-builder` extends its translation.
- **Literals** (code-organization.md, 6): a digest is a literal, a file's
  from `sha256sum`, a record's from its hand-written hex encoding (store.md
  3.1), once; a session that changes what `THIN` prints pastes its new one.

## Where later sessions add

| File | hearth 11 puts there | Who adds, and what |
|---|---|---|
| render `src/lib.rs` | `Render`, `step`, `resume`, `mod` lines; one match over `Event`, an arm a module call | each session: its `mod` line and arms |
| render `src/vocabulary.rs` | the enums (sketch) | 12 `Answer::Error`, 15 `Keep`/`Kept`, `As::Template`, 24 `Resolve`/`Resolved`, 22 `Rendered::left_out`, 25 `Hunks`, `Outcome::Negative` |
| render `src/revision.rs` | listing, classes, documents | 12 values and members, 13/16 declarations, 15 templates, 17 `debian` |
| render `src/compose.rs`, `src/call.rs` | discovery, placement, presence; one call's state: targets, reads, answers held | 12 layers, RFC 7386, provenance, names' forms and duplicates; 15, 24 their requests' bookkeeping |
| render `src/parts.rs`, `src/parts/<name>.rs` | `Spec`, `Part`, `PartRecord`, `spec_digest`; `meta.rs`; `config.rs` (32.1) | 17 `packages.rs`, `boot.rs`; 18 config's other fields, `data.rs`; 19 `secrets.rs`, `software.rs`, `live.rs`; 20 `exports.rs`: a variant and a file each |
| root `src/vocabulary.rs` | `Call::Render`, `Answer::Spec`, `Reply` | 12 `Answer::Error`, 22 `Reply::left_out`, 23 the store's child, 24 `--resolve`, 25 `Call::Diff`, `Answer::Change` |
| protocol `src/translate/` | `listing.rs`, `documents.rs`, `answers.rs`; `parts.rs` + `parts/config.rs` (32.1) | 12 `errors.rs`; 13, 16 `declarations.rs`; 14 `operations.rs`; 15 `instructions.rs`; each part's session `parts/<name>.rs` and its arm in `parts.rs`; 23, 33 `store.rs`; 24 `resolution.rs`; 25 `hunks.rs`, `changes.rs` |
| protocol `src/git.rs`, `read.rs`, `client.rs`, `cache.rs` (32.1) | git's children; a read's blobs; the printer on stdout; files into the cache | 15 rendered files into `cache.rs`; 23 the record and lock in new `store.rs`, `store/names.rs`; 33 the walk in `walk.rs`; 25 `diff --no-index` in `git.rs` |
| `tests/{builder,render}/src/fixtures.rs` | builder: `Repository::make`, `THIN`, `render` (`binary` 32.2); render: the scripted repository, `x1_prod` | each session its repositories, documents and literals |

## Increments

### 11.1 Render's skeleton: the listing, discovery, `meta`, the spec digest
Scope: `hearth-domain-render` (render.md, 8.1) and its world (9.1): the
call, `List`, the path classes (format.md, 2.1), discovery (3.1), `Read` of
each asked target's two documents, placement (the base-env layer's, else
the environment's: format.md, 3.4), `meta`, the spec digest, the answers.
Needs: hearth 01 (`hearth_lib::encode`, `hash::H`).
Files: root `Cargo.toml`, `Cargo.lock` (shared); new
`crates/hearth-domain-render/`: `Cargo.toml` (`skein-lib`, `hearth-lib`;
`[lints] workspace = true`), `src/{lib, limits, vocabulary, revision,
compose, call, parts}.rs`, `src/parts/meta.rs`, `src/tests.rs`,
`src/tests/{fixtures, revision, parts, outcome}.rs`; new `tests/render/`
(`hearth-render-world`, ordinary Rust, `findings/skein-testing.md` 7's
lints table): `Cargo.toml`, `src/{lib, fixtures, parent}.rs`,
`tests/thin.rs`; `tests/clippy.toml` (shared).
Shape: the sketches. `limits::SHIPPED` holds 2.3's `targets: 1_024`,
`document: 1_048_576`. `compose::targets(&[Entry]) -> Result<Box<[Target]>,
TooMany>`: `Class::Host { base, name }` with `name` equal to `base` is
the base; `<base>-<env>` with `envs/<env>.json` listed makes the target,
by prefix and suffix, never a split on `-`; `TooMany` ends the call
`Failed` (42.2 gives it its error). A named target not discovered is
`Usage`; a document past `document` or `Refused` fails its targets;
`Unresolved` ends the call `Usage`, `Failed` with its class's outcome.
`meta`'s record: `map { base, environment, member?, placement: map {
firmware?, hardware?, kind, on? }, target }`, kinds by name (store.md,
3.1). `Part` is declared here, `PartRecord` with no variant until 32.1.
Test shape: unit (`src/tests/`, under 20 ms):
`paths_are_classed_as_format_md_2_1_lays_them_out` (a `const` table),
`the_meta_record_encodes_as_store_md_3_1_writes_it` (x1-prod's, in hex),
`the_meta_digest_is_h_of_part_meta_over_its_record`
(`706777e0f8563a881bb75631bb73470298bc6d9f2f638d1dc526310adeb9b73d`),
`the_worst_outcome_wins_over_every_pair` (`src/tests/outcome.rs`: the
pairs of `[Usage, Failed, TryAgain, Done]` against a literal 4×4 table,
client/README.md 3.3 and 3.1; hearth 25 adds `Negative`).
World (`tests/render/tests/thin.rs`, each under 0.5 s, eight seeds):
`fixtures::x1_prod()` lists `envs/prod.json` (`{}`) and `hosts/x1/{x1,
x1-prod}.json` (metal, efi, 7th-gen); `parent::run(&Repository, targets,
seed)` plays the root and the machines, answering each request after a
`skein_world::domain::Schedule` delay, terminals held to one by `Ledger`.
```rust
// tests/render/tests/thin.rs
#[test]
fn a_targets_meta_is_its_names_and_the_base_envs_placement() {
    for seed in 0..8 {
        let run = parent::run(&fixtures::x1_prod(), &[], seed);
        let [Answer::Spec(spec)] = &run.answers[..] else { panic!("one spec: seed {seed}") };
        assert_eq!(spec.meta, fixtures::x1_prod_meta(), "render.md, 4.4: the target's meta");
        assert_eq!(spec.digest, fixtures::digest("d5942fa1a047fd65c5731c85cfba607e81d0fd16cb00f154aacfa79c44ed118c"), "seed {seed}");
        assert_eq!(run.outcome, Outcome::Done, "seed {seed}");
    }
}
```
Then, in `tests/thin.rs`, `render_lists_once_then_reads_each_targets_documents_once`,
`a_base_env_without_its_environment_makes_no_target`,
`the_base_envs_placement_replaces_the_environments_whole`,
`a_named_target_not_discovered_ends_the_call_with_usage`,
`a_refused_document_fails_its_targets_and_spares_the_rest`,
`targets_are_answered_in_name_order_each_once`.
Proof: world, `a_targets_meta_is_its_names_and_the_base_envs_placement`:
x1-prod's meta and spec digest, whatever the pace. Run once: `heavy cargo
nextest run -p hearth-render-world -E 'test(=a_targets_meta_is_its_names_and_the_base_envs_placement)'`.
Nothing above exists yet (11.3 connects it through real git). Bound 0.5 s.
Don't: read `fleet.json`, the base document or `tasks/` (32.1, 12); check
names or duplicate targets (hearth 42); format or parse.
Size: at most 300 lines. Removes: nothing.

### 11.2 The builder's root; the protocol layer's git and documents
Scope: `hearth-domain-builder` (render.md, 8.1): the call to render, its
requests out, its answers to the client; `hearth-protocol-builder`'s first
part: `List` as rev-parse then ls-tree, `Read` as one `cat-file --batch`
(render.md 2.2: one process per `Read`), a document's blob gathered whole (its listed size,
which render checked against `document` before it asked) and decoded
through `skein_lib::stream::Held`, `json::Decoder` and
`document::document`; `translate/{listing, documents}.rs`.
Needs: hearth 02.3 (`invocation`, `rev_parse`, `listing`, `batch`),
hearth 03.3 (`json`, `document`, `Placement`).
Files: root `Cargo.toml`, `Cargo.lock` (shared); new
`crates/hearth-domain-builder/` (`src/{lib, vocabulary}.rs`, `src/tests.rs`);
new `crates/hearth-protocol-builder/` (`skein-lib`, `skein-io`, the
three machines, the root): `src/{lib, limits, git, read, translate}.rs`,
`src/translate/{listing, documents}.rs`, `src/tests.rs`,
`src/tests/{fixtures, git, read, translate}.rs`.
Shape: the sketches. A child: `io::Request::Spawn` of `invocation::GIT`
with `invocation::arguments` and `environment()`, standard output
piped (and standard input for cat-file), the rest `/dev/null`; its
stdout's `Bytes` to its machine (rev-parse's kept to `MAX_STDOUT + 1`);
`Exited` checked by `invocation::succeeded`; then `Close`. cat-file's
request lines sent, then `Finish`; each blob `batch::Request::Read { oid,
size, max: size }`, its pieces into a `skein_lib::Writer` of `size`.
`Fault::Stream` from a decoder, a machine's `Failed`, `Missing` or a
failed exit is `Failed { class: Operational }`.
Test shape: step tests under the step crates' lints, by hand through
`fixtures::Layer` (`down`, `up`, `asked()`, `told()`), at
`limits::SHIPPED`, each under 20 ms.
Its first test, `a_list_spawns_rev_parse_with_render_md_2_2s_vector_and_environment`,
asserts `/usr/bin/git`, render.md 2.2's vector word for word and
`HOME=/dev/null` as the environment's last entry.
Then `rev_parse_exiting_1_answers_unresolved`,
`the_commit_rev_parse_names_is_listed_with_git_s_entries`,
`a_read_sends_each_request_line_then_ends_standard_input`,
`a_documents_blob_reaches_render_decoded_with_its_placement`,
`a_document_config_refuses_answers_refused_at_its_position`,
`git_exiting_past_its_success_fails_the_request_as_operational`;
`translate.rs`: `each_mode_and_placement_kind_translates_and_back`; root
(`src/tests.rs`): `a_render_call_reaches_render_and_its_spec_the_client`,
`a_second_call_while_one_is_held_panics` (`should_panic`).
Proof: unit, `a_documents_blob_reaches_render_decoded_with_its_placement`:
x1-prod's document, from `batch` pieces to render's `Document`. Run once:
`heavy cargo nextest run -p hearth-protocol-builder -E 'test(=tests::read::a_documents_blob_reaches_render_decoded_with_its_placement)'`.
No service runs the layer yet (11.3). Bound 0.02 s.
Don't: keep a deadline on git (render.md 2.2: none in M1); write render's limits into
the protocol layer (`max` is the listed size, checked by render); hold
more than one blob's bytes; name a child domain but through the root's
`pub use`.
Size: at most 450 lines (ruling 130: 896 landed as prepared). Removes: nothing.

### 11.3 The service, the printer, and the real loop
Scope: `hearth-service-builder` (its `iterate`, programming-model.md, 2);
the answers translated into `records::Record`s (`translate/answers.rs`) and
printed by 10.2's `Printer` on standard output, `Finish` after the terminal
record; `crates/hearth`'s `Client` Host (the store made 0700, the spawns'
root); `tests/builder`: the scratch repository and the real loop (testing.md, 2.7).
Needs: hearth 10.2 (`records`, `form`, `printer`, `outcome::code`).
Files: root `Cargo.toml`, `Cargo.lock` (shared); new
`crates/hearth-service-builder/` (`src/{lib, limits}.rs`, `src/tests.rs`);
`crates/hearth-protocol-builder/src/{lib, translate}.rs`, new `src/client.rs`,
`src/translate/answers.rs`, `src/tests/{client, translate}.rs`; new
`crates/hearth/` (ordinary Rust, its own `[lints]` and `clippy.toml`):
`Cargo.toml`, `src/lib.rs`; new `tests/builder/`: `Cargo.toml` (`hearth`,
`skein-shell`, `skein-scratch`, `skein-io`), `src/{lib, fixtures}.rs`,
`tests/real.rs`.
Shape: the sketches; `iterate` after echo's (`examples/echo/service/src/
lib.rs:316`), with `resume` drained first in each domain and protocol
stage. `Client::exit` is `Exit::Code(code)` once the service is empty.
Test shape: `fixtures::Repository::make(label, &[File])`: a `skein_scratch::
Scratch` holding `repo/` (`git init --quiet --initial-branch=main`, the
files, `git add -A`, `git commit -q -m thin` by `std::process::Command`,
the environment cleared and given: `PATH` as the process has it,
`GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`,
`HOME=/dev/null`, author and committer `hearth <hearth@hearth.invalid>`
at `1700000000 +0000`, as 34.1's) (`work_tree()`,
`scratch()`; `Repository::empty(label)`: no `repo/` committed);
`fixtures::render(&Repository, targets, Form)` is `render_at(.., "HEAD",
..)`, and `render_at(&Repository, at: &str, targets, Form) -> Rendered {
exit, stdout }` opens `Client` with `hearth::limits()`, the
store `<scratch>/store`, standard output a file
`<scratch>/stdout`, and runs `skein_shell::drive`.
`THIN`: `fleet.json` `{}`, `envs/prod.json` `{}`, `hosts/x1/x1.json`
`{"motd": {}}`, `hosts/x1/x1-prod.json` `{"placement": {"kind": "metal",
"firmware": "efi", "hardware": "7th-gen"}}`, `tasks/motd/task.json` `{}`,
`tasks/motd/tree/etc/motd` `Welcome to x1.\n`, `tasks/motd/tree/usr/local/
bin/motd` (100755) `#!/bin/sh\nexec cat /etc/motd\n`. Each test under 0.5 s:
```rust
// tests/builder/tests/real.rs
#[test]
fn hearth_render_prints_the_thin_repositorys_spec_through_real_git() {
    let repository = Repository::make("thin", fixtures::THIN);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    let expected = "target       x1-prod   base x1   environment prod\nplacement    metal efi 7th-gen\nspec         d5942fa1a047\n";
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), expected, "9.2's lines, meta's alone");
    assert_eq!(rendered.exit, Exit::Code(0), "client/README.md, 3.3: done");
}
```
Then `json_lines_give_the_version_first_and_the_outcome_last`,
`an_unknown_target_exits_64_after_the_listing`,
`a_revision_git_cannot_resolve_exits_64` (`render_at(.., "nope", ..)`),
`a_directory_that_is_no_repository_exits_75` (`Repository::empty`),
`a_scratch_repository_without_git_on_the_path_fails_by_name_not_by_skip`
(`Repository::make_on(path, ..)`, which `make` calls with `PATH`, on an empty
directory: `should_panic(expected = "git is required")`); unit:
`a_spec_translates_to_the_records_and_back`,
`the_terminal_record_follows_the_last_spec_then_the_stream_finishes`.
Proof: process, `hearth_render_prints_the_thin_repositorys_spec_through_real_git`:
the shipped service, real git and 10.2's printer give 9.2's lines and exit 0.
Run once: `heavy cargo
nextest run -p hearth-builder-world -E 'test(=hearth_render_prints_the_thin_repositorys_spec_through_real_git)'`.
Bound 0.5 s.
Don't: depend on `std::process` or `std::fs` in the service; print in
the shell what the printer prints; claim a worst case (Choices).
Size: at most 300 lines. Removes: nothing.

## Interface sketches

Every type derives `Clone, PartialEq, Eq, Hash, Debug` unless said (`Copy`
where it can; a type holding a `ReplyTo` drops `Clone`; `Asked` and `Told`
derive `PartialEq, Eq, Debug` (skein's io and file requests and events derive
no `Clone`, `file::Request` and `file::Event` no `Hash`); `Render`,
`Protocol`, `Service` and `Client` derive `Debug` alone). The root
re-exports render (`pub use hearth_domain_render as render;`), through
which the protocol layer names it.

```rust
// crates/hearth-domain-render/src/vocabulary.rs (11.1)
pub enum Event { // Into render: the root's call, or a machine's answer (render.md, 8.2).
    Render { repository: Box<[u8]>, revision: Box<[u8]>, targets: Box<[Box<[u8]>]> },
    Listed { request: Token, commit: Oid, entries: Box<[Entry]> }, // List's terminal: git's order
    Unresolved { request: Token }, // List's terminal: no such commit
    Read { request: Token, oid: Oid, read: Decoded }, // a blob in the order asked; the last ends the Read
    Failed { request: Token, class: Class },
}
pub enum Request { // render to a machine (render.md, 8.2).
    List { request: Token, repository: Box<[u8]>, revision: Box<[u8]> },
    Read { request: Token, repository: Box<[u8]>, commit: Oid, blobs: Box<[Blob]> }, // one cat-file
}
pub enum Answer { Spec(Spec), Rendered { outcome: Outcome } }  // each target's spec, whole; then the worst outcome reached
pub enum Output { Request(Request), Answer(Answer) }
pub struct Blob { pub oid: Oid, pub size: u64, pub kind: As }  // its listed size, how to read it (8.2's `as`)
pub enum As { Document }
pub enum Decoded { Document(Document), Refused { at: Position } } // format.md, 3, or refused where its fault starts
pub enum Class { Configuration, Operational }
pub enum Outcome { Usage, Failed, TryAgain, Done } // client/README.md, 3.3, worst first; `Negative` with hearth 25
impl Outcome { #[must_use] pub fn worst(self, other: Outcome) -> Outcome; }
// crates/hearth-domain-render/src/limits.rs (11.1): 2.3's; hearth 12 on add theirs
pub struct Limits { pub targets: u32, pub document: u32 } // SHIPPED: 1_024, 1_048_576
// crates/hearth-domain-render/src/lib.rs (11.1): `Render` holds the call it serves (`call.rs`)
pub const MAX_OUT: u32 = 1; pub const MAX_OUT_RESUME: u32 = 1;
pub fn step(render: &mut Render, env: &Env<Limits>, ev: Event, out: &mut Queue<Output>);
pub fn resume(render: &mut Render, env: &Env<Limits>, out: &mut Queue<Output>);
// crates/hearth-domain-render/src/revision.rs (11.1)
pub struct Oid(pub Box<[u8]>); // as `Oid::hex` writes it: 40 or 64 lowercase hex digits
pub enum Mode { File, Executable, Link, Submodule }
pub struct Entry { pub path: Box<[u8]>, pub mode: Mode, pub oid: Oid, pub size: u64 }
pub struct Position { pub line: u32, pub column: u32 }
pub enum Class<'a> { // format.md, 2.1, 2.3; its refusals are hearth 42's.
    Fleet, Environment { env: &'a [u8] },
    Host { base: &'a [u8], name: &'a [u8] }, // hosts/<base>/<name>.json: the base, or a base-env
    Other,
}
#[must_use] pub fn class(path: &[u8]) -> Class<'_>;
// Mirror config's `Document`, `Layer` and `Placement` (format.md, 3.4), as the thin story reads them:
pub struct Document { pub at: Position, pub layer: Layer }
pub struct Layer { pub placement: Option<Placement> }
pub struct Placement { pub at: Position, pub kind: Runs, pub firmware: Option<Firmware>, pub hardware: Option<Box<[u8]>>, pub on: Option<On>, pub mutable: bool }
pub enum Runs { Metal, Vm, Container }  pub enum Firmware { Efi, Bios }
pub struct On { pub target: Box<[u8]>, pub at: Position }
// crates/hearth-domain-render/src/parts.rs (11.1); each mirrors the records' type
pub struct Spec { pub meta: Meta, pub meta_digest: Digest, pub commit: Oid, pub digest: Digest, pub parts: Box<[Part]> }
pub struct Part { pub digest: Digest, pub record: PartRecord } // H("part:<name>", the record)
pub enum PartRecord {}
#[must_use] pub fn spec_digest(meta: &Digest, parts: &[Part]) -> Digest; // H("spec", map { name: digest }, `meta` among them) (render.md 4.5)
// parts/meta.rs (11.1): render.md 4.4's meta; `digest` is H("part:meta", the record)
pub struct Meta { pub target: Box<[u8]>, pub base: Box<[u8]>, pub member: Option<Box<[u8]>>, pub environment: Box<[u8]>, pub placement: Placement }
pub struct Placement { pub kind: Runs, pub firmware: Option<Firmware>, pub hardware: Option<Box<[u8]>>, pub on: Option<Box<[u8]>> }
#[must_use] pub fn digest(meta: &Meta) -> Digest;
// crates/hearth-domain-builder/src/vocabulary.rs (11.2); `Event` and `Request` derive `PartialEq, Eq, Debug`
pub enum Event { Call { reply_to: ReplyTo, call: Call }, Render(render::Event) } // the client's call: `Answer`s, then one `Reply`
pub enum Call { Render { repository: Box<[u8]>, revision: Box<[u8]>, targets: Box<[Box<[u8]>]> } } // client/README.md, 3.1; `Diff`: hearth 25; `repository` an absolute git directory.
pub enum Request { Render(render::Request), Answer(Answer), Reply { to: ReplyTo, outcome: render::Outcome } } // `Reply` ends the call
pub enum Answer { Spec(render::Spec) } // `Error` with hearth 12, `Change` with hearth 25
pub struct Limits { pub render: render::Limits } // hearth-domain-builder's `src/lib.rs` (11.2); 23 adds the store's
// crates/hearth-protocol-builder/src/lib.rs (11.2; the printer 11.3)
pub enum Asked { Io(io::Request) }  pub enum Told { Io(io::Event) } // processes and streams (32.1: files)
pub struct Limits { pub git: git::limits::Limits, pub config: config::Limits, pub printer: records::limits::Limits } // neither crate re-exports `Limits`
pub struct Protocol { /* private: one call's git child, blob, printer */ }
impl Protocol {
    #[must_use] pub fn new(limits: &Limits, call: Call, form: Form, root: Fd) -> Protocol; // git spawned beneath `root`, `HOME=/dev/null`.
    #[must_use] pub fn code(&self) -> Option<u8>;
}
pub fn up(protocol: &mut Protocol, env: &Env<Limits>, ev: Told, above: &mut Queue<builder::Event>, below: &mut Queue<Asked>);
pub fn down(protocol: &mut Protocol, env: &Env<Limits>, rq: builder::Request, above: &mut Queue<builder::Event>, below: &mut Queue<Asked>);
pub fn resume(protocol: &mut Protocol, env: &Env<Limits>, above: &mut Queue<builder::Event>, below: &mut Queue<Asked>);
// src/translate/*.rs: one function a type, both ways in src/tests/translate.rs
#[must_use] pub fn entry(entry: listing::Entry) -> render::revision::Entry; // listing.rs (11.2)
#[must_use] pub fn document(document: config::document::Document) -> render::revision::Document; // documents.rs (11.2)
#[must_use] pub fn spec(spec: render::parts::Spec) -> records::records::Spec; // answers.rs (11.3); `resolution: None` until hearth 24.1
#[must_use] pub fn outcome(outcome: render::Outcome) -> records::outcome::Outcome; // answers.rs (11.3)
// crates/hearth-service-builder/src/lib.rs (11.3)
pub struct Configuration { pub limits: Limits }
pub struct Limits { pub io: io::Limits, pub protocol: protocol::Limits, pub domain: builder::Limits, pub queue: u32 } // `files` with hearth 32.1
pub struct Roots { pub store: Fd } // the store, git's working directory (32.1: `cache/`)
pub struct Service { /* private: io, protocol layer, root, envs, queues */ }
impl Service {
    #[must_use] pub fn new(configuration: &Configuration, roots: Roots, call: Call, form: Form) -> Service;
    pub fn adopt_output_pipe(&mut self, fd: Fd) -> Result<(), Fd>; // standard output as a pipe,
    pub fn adopt_output_file(&mut self, fd: Fd) -> Result<(), Fd>; // or a file to append; refused: the caller keeps `fd`
    #[must_use] pub fn code(&self) -> Option<u8>;
}
pub fn iterate(service: &mut Service, now: Time, wall: Wall);
#[must_use] pub fn operations(limits: &Limits) -> Option<u32>;
// crates/hearth/src/lib.rs (11.3): ordinary Rust; 32.2 changes `open`'s error to records' `Startup` (40.2)
pub struct Invocation { pub call: Call, pub form: Form, pub store: PathBuf } // What the command line asks.
pub struct Client { /* service, limits */ }
impl Client {
    pub fn open(invocation: Invocation, limits: &Limits, output: Fd) -> Result<Client, std::io::Error>; // shell.md, 6: the store made 0700; output adopted.
}
#[must_use] pub fn limits() -> Limits;
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-11\.[1-3]'` prints 3.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest run
   -p hearth-render-world -p hearth-builder-world` passes, its `Summary`
   naming at least 13 tests.
3. `git -C ~/src/rust/hearth ls-tree --name-only origin/main
   crates/hearth-protocol-builder/src/translate/` lists `answers.rs`,
   `documents.rs` and `listing.rs`.
4. `git -C ~/src/rust/hearth grep -n -E -e '\.\.\s*\}' -e '^\s*_ =>' origin/main
   -- crates/hearth-protocol-builder/src/translate` prints nothing.
