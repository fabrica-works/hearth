# render: decisions

The decisions the plan rests on, each with its reason. The supervisor's
rulings during the run are appended under "Rulings"; the user's veto
points are in the last section.

## The plan's decisions

1. **skein's increments land through `land --gate`, on any machine**
   (planner-7, 2026-10-17). The user had chosen x1 for the run
   (2026-10-16); the director then moved it to debian-8gb-hel1-1. The gate
   is check.sh's commands but `a_page_of_another_origin_is_opened_by_its_step`,
   which failed on x1 at the base (`findings/skein.md`, 4) and passes on
   carbon-forge (`findings/skein.md`, 4); each commit body says so. The plan's
   skein changes touch no `skein-ui-web` code, so no machine needs a probe.
2. **`hearth-protocol-builder` routes the builder's answers to the
   printer** (render.md, 8.1 leaves it to the plan; the user chose it,
   2026-10-16). Hosted by the client, the builder's protocol layer
   routes `hearth-client-protocol-records` as one of its machines, and
   translates render's spec, diff and errors into the client's records,
   which the printer writes to standard output's stream. The binary's
   shell parses argv into the client's call before the loop, and exits
   with the code the answer's terminal record maps to (`client/README.md`, 3.3). In
   M3 the same layer, hosted by the steward's builder, routes the answers
   to `hearth-protocol-jobs` instead. Reason: a protocol layer routes its
   machines and translates between vocabularies (`programming-model.md`,
   4); the binary is a shell, and a service is its `iterate`.
3. **Closing repins** (the user, 2026-10-16): smith once every skein step
   has landed; jig and temper only once temper's client-api row in their
   `STATUS.md` reads `done`, since that plan is active in both and pins
   skein (README.md, Repin).
4. **One live service, snapshot.debian.org,** for the test mirror's
   indexes, fetched once by hearth 08.2's script (the user, 2026-10-16;
   testing.md, 6.2). Conformance then runs on loopback only. The budget
   is README.md's.
5. **As many sessions at once as dependencies and the machine allow,**
   no fixed cap below eight (the user's standing rule).
6. **The cut** (the user, 2026-10-16): skein's four sessions (a
   further one, lib's SHA-256, was landed by temper's forge-setup plan:
   skein `f981746`), then hearth's forty-seven; the thin story, hearth 11, as
   early as its parts allow (development.md, 5.3). Each render session
   after it proves on the thin story's path.
7. **The design's gaps went to the designer** in three rounds: the
   first (answered: skein `7e8fb70`, hearth `b3438da`), the second (K4 to
   K6, H9 to H13: skein's io for the store, its temporaries, the mirror's
   address, the client's configuration, the index's bound, the client's
   crate names; answered: skein `d21d90d`, hearth `175e92f`) and the third
   (K7 to K16, H14 to H178, every default the sessions kept; answered:
   skein `eadbe29`, hearth `31bed37`). The answers are in the designs;
   `~/src/rust/tmp/hearth-m1-design-answers.md` (README.md, Inputs)
   indexes them, each round's questions file beside it.
8. **Client crates carry the `hearth-client-` prefix** (the user,
   2026-10-16, after jig's `README.md`, 15; H13): the client's records
   and printer are `hearth-client-protocol-records`. Where the designer's
   answer to H13 gives another name, the plan follows it.
9. **Machine worlds are crates of their own,** `tests/<machine>`, named
   `hearth-<machine>-world`, as skein's `tests/json` is: config,
   template, git, mmdebstrap, tree. testing.md, 2.9's layout names
   `tests/<name>` for a domain's world only; it leaves a machine world's
   place open. testing.md, 10 puts the tree machine's world at M2; this
   plan brings it to M1 with `hearth-protocol-tree`'s first kinds, since
   a decoder M1 lands is fuzzed when it lands (render.md, 9.4).
10. **Fuzzing is seeded sweeps,** in each machine world's
    `fuzzy_*.rs` (generated inputs, recordings cut and mutated), as in
    skein and jig: no repository has a `cargo-fuzz` target
    (`findings/skein-testing.md`). testing.md, 2's "fuzz targets" and
    render.md, 9.4's "every decoder is fuzzed" are met by the sweeps;
    `cargo-fuzz` targets are `later.md`'s.
11. **Removing `staging/<resolution key>/`, and reading an index in
    chunks,** are `hearth-protocol-builder`'s: a walk with an explicit
    stack, bounded by render.md 2.3's limits and a count of entries, since
    skein's io never traverses; `ReadAt` at increasing offsets, since
    skein has no file read as a stream (`findings/skein-io.md`; listed for
    the designer in the second round, "The plan decides", which the
    designer did not object to). The walk is hearth 33's (the scan's
    `staging/` row), reused by hearth 24: at most 16 directories open,
    64 entries a page, 65,536 entries in all.
12. **Recordings are captured on the run's machine** (H7: "record the
    versions on the machine the plan runs on"), git's in hearth 34,
    mmdebstrap's in hearth 41, under `testing/recordings/<tool>/<version>/`.
13. **Fake git names objects as git does:** its blobs, trees and commits
    get git's SHA-1 object ids, computed in the fake (ordinary Rust) with
    the `sha1` crate pinned to an exact version, so that conformance
    compares listings and revisions whole (testing.md, 5.2, "git").
    skein-fake-checkout's commit ids are counters
    (`findings/skein-testing.md`).
14. **Fake mmdebstrap resolves for real:** for conformance's resolve
    scenario it loads the mirror's `Packages` index as its universe and
    computes the closure from Essential, priorities, Depends,
    Pre-Depends, Provides, alternatives, version relations and
    Conflicts (hearth 41.1's `unsatisfiable` case fails only through a
    `Conflicts`; testing.md 3.6 names the first relations alone: a third
    round question), which must equal apt's for
    `test-min` and `test-boot` (testing.md, 5.1 and 5.2). Differences go
    in the scenario's allowed differences with a reason each (5.3), or
    are fixed in the fake.
15. **Test-time aims:** hearth's suites may grow by at most 8 s focused
    and 30 s fuzzy in this run, half its budgets, so that M2 starts with
    room; skein's by at most 1.5 s and 5 s, above one gate's swing
    (development.md, 5.6).

16. **The workspace's members are globs,** `"crates/*"`,
    `"testing/hearth-*"` and `"tests/*"`, each added by the landing that
    puts the first crate under its root: some eight sessions add their
    first crate at the start, and a glob leaves only
    `[workspace.dependencies]` lines and `Cargo.lock` to rebase. cargo
    1.95 refuses a glob that matches no directory (checked at the base:
    `failed to load manifest for workspace member …/tests/*`), so no
    landing adds a glob before its root holds a crate; a file such as
    `tests/clippy.toml` beside the crates is skipped. `testing/` also
    holds data (`recordings/`, `repos/`, `mirror/`), hence
    `testing/hearth-*`.

17. **Fake git follows skein-fake-checkout's model without depending on
    it** (testing.md, 4.4: "takes its mechanics from
    `skein-fake-checkout`"): its objects are git's, by SHA-1 (decision
    13), where skein-fake-checkout's ids are counters. The fake machine
    answers two of M2's `openat2` flags already (an escape refused as
    `EXDEV`, a no-follow open of a link as `ELOOP`), since skein's io
    opens every path that way.

18. **Evaluation proves in render's domain world until hearth 15**
    (hearth 14 and 43): no expression reaches render before hearth 15 evaluates
    a template, so 14.1, 14.2 and 43.1 build render's values, operators
    and filters against a reference evaluator in `tests/render`, and
    43.2 lands the translation of hearth 05's operations whole, exhaustive
    both ways, once render has every `Op` and `Filter`. hearth 15's
    increment that evaluates, 15.2, connects evaluation to the real loop. This bends README's
    "The meeting point" for these three increments only. Likewise
    hearth 13.5's `schema::debian`, proved at the unit layer: hearth 17
    composes `debian` and connects it.
19. **The binary sets its umask to 022 at startup** (hearth 32.1, through
    `libc` in the shell, ordinary Rust): store.md 2.2's "Cache files are
    0644 in a 0700 directory" must hold whatever the user's umask, and
    skein's io makes files "`0o666` less the umask" and has no `chmod`.
    Directories keep their explicit 0700. The client's refusals before
    the loop print a `startup` record on standard error under `--json`
    (hearth 40.2, 32.2; client/README.md 3.2), a record of client
    protocol version 1.
20. **A session file that cannot reach 420 lines and 28,000 bytes without
    dropping an increment, a test, a boundary type or a question splits in
    two; the second part takes the next free number** (hearth 31 from 24,
    32 from 11, 33 from 23, 34 from 02, 35 from 04, 36 from 06, 37 from 05,
    38 from 07, 39 from 09, 40 from 10, 41 from 08, 42 from 12, 43 from 14,
    44 from 26, 45 from 25, 46 from 27, 47 from 30; skein 05 from skein 04), so no
    session is renumbered. Reason:
    a session file is read whole by its implementer (development.md, 5.2),
    and a shorter file that lost a test or a question would move the loss to the
    run; a renumbering would break every Needs and citation of the files
    written meanwhile, and the numbers are the step names (`hearth-<n>.<k>`).
21. **The test-time aims are the suites' `Summary` times; a session's bounds
    are its tests' own times** (planner-8, 2026-10-17). README.md's aims
    (Rules, "Test time") are what the gate's nextest prints, its tests run
    over the machine's threads (hearth's `.config/nextest.toml` orders them
    "so that the run ends close to its total over the threads"); a Test
    shape's bound is one test's time (`--profile measure -j 1`). The
    sessions' fuzzy bounds sum to about 47 s, about 12 s over 4 threads,
    within the 30 s aim. Reason: the aims are read from `Gate-Result`, and
    no cut of 29's and 30's rounds could bring the sum of bounds to 30 s
    without dropping the sweeps other sessions rest on.

## Rulings

None yet.

101. **Memory bounds on debian-8gb-hel1-1: starts held below 3 GB** (process, hearth 01.1's block of 2026-10-17T15:45:10Z).
     On debian-8gb-hel1-1 (8 GB, 4 threads), for the rest of this run, the
     coordinator holds new starts below 3 GB of available memory (`free -g`'s
     "available"), not 6 GB, and widens the build lane back to two tasks at
     4 GB or more, not above 6 GB. Unchanged: below 3 GB it narrows the build
     lane to one task (`pueue parallel 1 --group build`); below 1.5 GB it
     stops the line; the disk bounds (40 GB, 60 GB, 15 GB) and the order of
     starts stay as development.md 11.4 and the README say. Reason: 11.4's
     6 GB is three quarters of this machine; with no implementer running it
     has about 4.5 GB available, so the bound held every start. Builds go
     through the queue (two build tasks of `HEAVY_BUILD_JOBS=2` cargo jobs,
     the gate alone), whatever the number of sessions, and the planning's 502
     `heavy` tasks on this machine all ended with at least 4.0 GB available.
     The coordinator logs each memory check beside its disk check, as now.

102. **skein-02.s1: the replay bound's fixture, and skein's gate passed as written** (plan defect, skein 02's block of 2026-10-17T15:57:15Z).
     A plan defect: 02.1's Files say "`size_of::<tokenizer::Tokenizer>()`
     (`crates/skein-llm/src/openai/common.rs:69`) grows and is computed", and
     miss `tests/llm/tests/replay_total.rs:19-20`, whose literal totals
     include it. skein 089ed83 landed with that test failing: its gate script
     had no `set -e`, and the focused suite ran 543 of its 2484 tests. The
     repair is a step not in the plan, skein-02.s1
     (`skein/02.s1-replay-total.md`), skein 02's, landed before 02.2 and
     before any other skein landing:
     - each provider's `parent` and `fixed` advance by what
       `client::worst_case` now computes over the old value (32 for
       `OpenAiCodex`: 53,130,181 against 53,130,149); `removed`, `receiving`
       and the other assertions stay; the test keeps its name and inputs.
       Literals, as the file has them, not a `size_of` in the test.
     - it lands with `land --gate` and the README's skein gate passed as
       written, in single quotes, its commands joined by `&&`; never a script
       or file of the session's own (the corrected `land.sh` in `logs/`
       included). The README now says so for every skein session.
     - its commit body says, in one line, that 089ed83's `Gate-Result` is
       invalid (focused: 543/2484 run, 1 failed;
       `logs/20261017T155536-7.log`), and that this commit's gate ran the
       whole suite.
     - a further failure among the 1941 tests that did not run on 089ed83,
       where 02.1's Shape explains it, is fixed in the same landing under
       implementer.md's "A failure the Shape explains"; any other failure
       stops the step with a report.

     Refresh any further copy of a total that includes
     `size_of::<tokenizer::Tokenizer>()`, in skein or at a downstream repin,
     to what the code computes, and list it in the commit body.

103. **hearth 09's Shapes against main** (ready check of hearth-09.1).
     hearth 09's session file against hearth `29891d8`, `b62b498` and skein `089ed83` (its ready check): four text corrections in `hearth/09-records.md`, none in code.
     - 09.1's Files: `members` is `crates/*` already (`Cargo.toml:5`, `members = ["crates/*", "tests/*"]`; decisions.md, 16), so it gains nothing; `[workspace.dependencies]` still gains `hearth-lib` unless landed.
     - Kind 4's bounds are 6.1's table's (`store.md`, 6.1: "a URI, a path or a filename of 4,096 bytes; a suite, a component, a name, a version or an architecture of 256"), not bounds "6.1 leaves".
     - An index is `list [ path, size, sha256 ]` as store.md 6.4 and render.md 5.4 both give it (`render.md`, 5.4: "its indices in the cache, each [ path, size, sha256 ]"), not "render.md 5.4's map".
     - `a_resolution_field_one_past_its_limit_is_unreadable` and its writing twin gain a row: a component of 257 bytes.
     09.1's Size stays 280: the check's layout of the sketches is within implementer.md's tolerance, twice the Size. The sketches compile under clippy as written; hearth's default and fuzzy suites passed at `b62b498` with them applied.

104. **hearth 11's Shapes against main** (ready check of hearth-11.1).
     hearth 11's ready check, against hearth 5ead93f and skein 089ed83: three sketch paths corrected in `hearth/11-thin-story.md`, no suite failed.

     1. 11.2's `Limits` names `git::limits::Limits` and `records::limits::Limits`: neither `hearth-protocol-git` nor `hearth-client-protocol-records` re-exports `Limits` (`crates/hearth-protocol-git/src/limits.rs:3`, `crates/hearth-client-protocol-records/src/limits.rs:3`; their `lib.rs` declare `pub mod limits;` and no `pub use`).
     2. 11.3's translations return `records::records::Spec` and `records::outcome::Outcome` (`records.rs:28`, `outcome.rs:3`), `records` the crate.
     3. The records' `Spec` has `resolution: Option<Resolution>` since b62b498 (`records.rs:34`): 11.3's `translate::spec` writes `resolution: None` until hearth 24.1 fills it (`24-resolution.md:38`), which prints nothing, so 11's expected output is unchanged; the transitional list names it. A `Spec` literal without it fails with E0063.

     The check compiled the corrected sketches; hearth-lib's suite passed, 23 of 23, the plan's digests among them. Any further copy of these paths in 11's text takes the same form, and the commit body lists it.

105. **hearth 38's Shapes against main** (ready check of hearth-38.1).
     hearth 38's ready check, against hearth b62b498 (07.1's crate landed) and skein 089ed83: its Files corrected in `hearth/38-stanzas-recordings.md`, and the recordings' version given in `hearth/34-conformance.md`.

     1. 38.1's Files gain `src/vector.rs`: `Component::bytes` is private (`crates/hearth-protocol-mmdebstrap/src/vector.rs:59`, `fn bytes(self)`), and `lists::indices` builds `<component>/binary-<arch>/Packages` with it: it becomes `pub(crate)` (E0624 otherwise; clippy clean once it is).
     2. 38.1's Files gain `src/tests.rs`, which gains `mod packages;` (it reads `mod fixtures; mod output; mod vector;` on main).
     3. 38.2's world takes `hearth-recordings` and `hearth-test-mirror` as `[dependencies]`, not dev-dependencies: `tests/mmdebstrap/src/` is the world's library (`src/lib.rs`, and its `[dependencies]` already hold `skein-world`), which a dev-dependency does not reach (E0433). Its Files gain `src/lib.rs` (`pub mod recordings;`).
     4. The recordings' version: 38.2 finds `expect/<version>/<case>.expect`, and 34.1's `all(tool)` gave no version. 34.1's `Recording` gains `version`, its directory's own name, no file; `all(tool) -> Vec<(Box<str>, Recording)>`, each case's name and its recording, every version's, by version then case. 44 and 46 iterate `(case name, Recording)` as they did. testing.md 4.5 fixes the layout, not the kit's API.

     The check compiled 38.1's sketches against the landed crate under clippy once 1 was made; the workspace's suites passed but for the sketched test on a placeholder fixture (65 of 66), fuzzy 2 of 2. A further file a change reaches (the world's `src/fixtures.rs` for `tests/memory.rs`, say) joins the increment's Files, and the commit body lists it.

106. **hearth 37's Shapes against main** (ready check of hearth-37.1).
     hearth 37's ready check, against hearth de43be3 (05.1 and 05.2 landed) and skein 089ed83: `hearth/37-expression-operators.md` corrected.

     1. 37.1's Files: `src/tests/fixtures.rs` holds only `name` (:12-13); it gains `key` and `item`, each taking `Missing::Error`, and `lookups.rs`'s `Op::Key` and `Op::Item` literals (:13, 15, 17, 27, 30, 142, 156) call them (README, Edits outside Files: the users of a type the Shape changes).
     2. 37.1's world Files gain `src/check.rs`: `invariants` matches on every `Op` (:9-10), and an operator moves the depth (a binary one pops two and pushes one), more than README's five-line arm.
     3. 37.2's `If.start` is where a's operations begin: the start of the operand a stands in (the expression's, a group's, `Else`'s `start`), or the current element's, which `Pending::List` and `Pending::Arguments` gain as a mark set at each `,`. The list's `start` moved earlier elements: `[p, x if c else y]` compiled to `[c, if, p, x, else]`, not `[p, c, if, x, else]`. `values` falls back to the depth at that mark.
     4. 37.2's rotation sketch: `Stack::push` returns `Result<(), Pending>` (skein `stack.rs:48`) and `Error` has no `From<Pending>` (E0277): `pending.push(Pending::Else {..})` takes `.expect("the room of the If entry just popped")`; `push(out, ..)` stands for the parser's `emit` (`expression.rs:54`, no `Result`), which keeps `depth` and `values`.
     5. 37.3's worst case also counts the lexer's copy of the text (`copy_of(text)` in `compile`) and the buffer of the string being read (`lexer.rs:98`, the text's length): the longest string held 12,286 bytes at the peak, three times the text. The capacities `compile` allocates by are 05.2's (`expression.rs:33-34`): 37.3 counts them. `skein-heap` is in the root's `[workspace.dependencies]` already (`Cargo.toml:19`): only the world's `Cargo.toml` and `Cargo.lock` change.

     The check's suites at de43be3: clippy clean on the sketches, the workspace's 112 tests and fuzzy's 3 passed, the memory sketch held 12,286 and 8,224 of 266,240, the rotation's two tests passed once 3 was made. A further copy of these names takes the same correction, and the commit body lists it.

107. **hearth 11.1's size, by code-organization.md's count** (size, hearth 11's block of 2026-10-17T16:21:44Z).
     No defect: 11.1 is within its tolerance. code-organization.md, 7,
     counts an increment's size with "`git diff -M --numstat
     origin/main...HEAD -- crates testing ':!*/tests.rs' ':!*/tests/*'
     ':!testing/recordings/*' ':!testing/repos/*' ':!testing/mirror/*' | awk
     '{a+=$1} END {print a}'`", and says "Within twice its Size an increment
     lands without asking". On render/hearth-11-1-skeleton (42aae81) that
     command prints 584, the nine files of
     `logs/hearth-11/11.1-size.txt` (13 + 163 + 66 + 80 + 7 + 31 + 69 + 92 +
     63): within 600, twice 11.1's "at most 300 lines". The report's 684 is
     not that sum. Land 11.1 as committed, with the gate, and give 584 in
     the report's `Size:`, by that command. Every later increment counts its
     size with that command alone, written out as the document gives it.

108. **hearth 40's Shapes against main** (ready check of hearth-40.1).
     hearth 40's ready check, against hearth ff899e7 (hearth 10 landed) and skein 089ed83: `hearth/40-client-diff-errors.md` corrected against main's records crate and render.md 6.1, 6.3 and 7.3.

     1. Files: main's form is split, `src/form.rs`, `src/form/json.rs`, `src/form/text.rs` (`git ls-tree origin/main crates/hearth-client-protocol-records`): both increments' Files gain `src/form/{text,json}.rs`. `src/tests.rs` reads `mod fixtures; mod json; mod outcome; mod printer; mod text;`: 40.1 adds `mod diff;`, 40.2 `mod errors;`.
     2. `Terminal { outcome }` is built at `src/tests/json.rs:35`, `src/tests/printer.rs:90` and `src/tests/text.rs:65`, and nowhere else in hearth: 40.1 adds `diff: None` there, 40.2 `left_out: 0`.
     3. `left_out` (render.md 7.3: "JSON's `outcome` record carries `left_out`"; client/README.md 3.2: "one `outcome` line, which carries `from`, `to`, the counts and `left_out`"): always in the `outcome` line, last; 40.2's Files gain the five rows of `the_terminal_record_gives_its_outcome_by_name`, each ending `,"left_out":0}`. In text, `<n> more errors` right after the last error, then a diff's blank line and tally.
     4. The plan's text forms of render.md 6.1's rebuild names, 6.3 showing only `rebuild from config` and `nothing to build`: `rebuild from packages`, `rebuild from packages, resolution needed`, `rebuild from software`, `new slot only` (6.1: "a new slot of the same output"); the tally's singulars `target`, `changes`, `1 is unchanged`.
     5. Lints the check met: pedantic's `match_same_arms`; `<[_]>::concat` and `join` make a `Vec`, a disallowed type in step crates (`clippy.toml`).
     6. Done when 2: 20 tests on main (`git grep -c '#\[test\]'`: json 4, outcome 1, printer 6, text 9), and 12 + 11 named by 40.1 and 40.2: at least 43, not 44.

     A further `Terminal` literal, or a further outcome-line literal, that a later landing meets is refreshed the same way, and the commit body lists it.

109. **hearth 38's Shapes against main, re-checked after 07.2** (ready check of hearth-38.1).
     hearth 38's re-check after hearth 07.2 (5ead93f), against hearth f61c250: `hearth/38-stanzas-recordings.md` corrected.

     1. 38.1's sketched test drains `below` after each `up`: `UP_MAX_OUT.below` is 1 and each `Up::Bytes` queues the next demand, as `output.rs` does; undrained, the second line panics at skein `queue.rs:57` ("the loop reserves room for every output"). With `assert!(below.pop().is_some(), "each scan is followed by the next demand");` in the loop it passes (on a stub of the reader).
     2. The sketch's `Stanzas::new`, `largest_demand`, `worst_case`, `lists::indices` and `lists::listed` carry `#[must_use]`, as `output.rs` does (`Reader::new`, `tail`, `waiting`): pedantic's `must_use_candidate` fails five otherwise.
     3. 38.1's world Files gain `src/fixtures.rs` (a list at the limits; `fixtures::at_the_limits` builds only an output) and `read_index_metered` beside `read_index`, as `read_output_metered` (`tests/mmdebstrap/src/world.rs:77`) for `tests/memory.rs`.
     4. 38.2's Files gain `src/world.rs` (`read_recording`, which the sketch calls) and `src/reference.rs` (the `.expect`'s draft).
     5. `SHIPPED`'s new fields: `stanza: 1_048_576` (render.md 5.3, "at most 1 MiB a stanza"), `chunk: 16_384` and `filename: 4_096`, the plan's.

     The re-check's runs: the crate and world 28 of 28 with the corrected sketch on a stub, fuzzy 1 of 1, clippy clean after 2. A further sketch call that queues a demand drains it the same way.

110. **hearth 08.2's canonical timestamp from a listing** (plan defect, hearth 08's block of 2026-10-17T16:28:56Z).
     Ruling 110, plan defect: 08's Choices took the canonical timestamp from the URL that a file's request is redirected to, and snapshot.debian.org redirects a file to `/file/<sha1>/<name>`, which names none (build.md, 5.1: "apt follows its redirects to content-addressed `/file/<sha1>/` URLs"). What counts is the design's: "the canonical one snapshot.debian.org gives for the time the pin is made" (testing.md, 6.2), "the snapshot's own, since snapshot.debian.org redirects any other to the nearest earlier one" (format.md, 5.3). Where the script reads it is the plan's:

     1. `--pin` takes the timestamp first, from one of two listings: the URL that `…/archive/debian/<now>/dists/<suite>/` (a directory) is redirected to, `<now>` from `date -u +%Y%m%dT%H%M%SZ`; or, when that URL names no timestamp other than `<now>`, the greatest timestamp at or before `<now>` in the month's index, `…/archive/debian/?year=<YYYY>&month=<M>` (the month's number, no leading zero). It must match `[0-9]{8}T[0-9]{6}Z` and be at or before `<now>`; else the script fails, printing each listing's URL. Each listing's curl adds `--max-filesize 1048576` to the Shape's flags.
     2. Then `InRelease` from `…/archive/debian/<that timestamp>/dists/<suite>/InRelease`, the path the script takes without `--pin` from the pin's `archive`; the rest of 08.2's Shape is unchanged.
     3. The budget (README.md, Live services) gains, with `--pin`, one or two listings, each under 1 MB. The next run is the third, the budget's last; a run that fails after it is a question (blocked.md), with its output's lines.
     4. 08.2's commit carries `Fetch-Runs: 3` and `Fetch-Bytes:` 301546 plus the third run's total.

     Unchanged: the two files, the cache's layout, the pin's form, 08.2's tests, and the `unshare --net` check, which is no run.

111. **skein 02.3's retry, hearth-09.s1 and skein-04.s1** (process and plan defect, skein 02's, hearth 09's and skein 04's blocks of 2026-10-17T16:32:44Z to 16:36:34Z).
     Ruling 111, three answers:

     1. **skein 02.3, process: one gate retry.** `terminal_is_controlling_and_the_persons_interrupt_reaches_only_the_child` (skein `tests/world/tests/end_to_end.rs:168`) failed in 0.005 s in `logs/20261017T163049-21.log` and passed in the six other gate logs of this run that ran it, `logs/20261017T163427-9.log` the latest. skein 02.3 runs it once alone (`heavy cargo nextest run -p skein-world-tests --test end_to_end -E 'test(=terminal_is_controlling_and_the_persons_interrupt_reaches_only_the_child)'`), then, if it passes, retries `land --gate` once with the README's skein gate as written. A failure of either is a block naming both logs; no fix to skein's terminal code in 02.3. A second failure of this test in any gate of this run is a block too, whichever session meets it.
     2. **hearth 09.s1, plan defect: the step is authorized** as `hearth/09.s1-source-count.md` says, hearth 09's, after 09.3: `length(record.sources.len(), l.lines, b"sources")` moves above `validate`'s depth scan, as 09's Rules ask ("each `for` runs over a count already checked against a limit", `hearth/09-records.md:73`).
     3. **skein 04.1, process: the repair is skein-04.s1,** as `skein/04.s1-directory-parent.md` says, skein 04's, from its commit 4baf858. Its gate failed on a `target/` the coordinator removed while it ran (`logs/20261017T163427-9.log`, ENOENT under `target/wasm32-unknown-unknown/debug/deps`), an infrastructure failure that counts against no retry of the step: it retries `land --gate` once, with the README's skein gate as written.

     The coordinator adds `hearth-09.s1` and `skein-04.s1` to the ledger, each its session's, and removes a session's `target/` only after the session's final report and the end of its turn, until the run's end.

112. **hearth 06's Shapes against main** (ready check of hearth-06.1).
     hearth 06's Shapes against main (ready check of hearth-06.1), against hearth 0fa1b84 and skein b888da9: `hearth/06-templates.md` corrected.

     1. 06.1's Files gain `src/tests/lookups.rs`: its `ONE` (`:109`, `Limits { expression: SHIPPED.expression, nesting: 1 }`) stops compiling once `Limits` has 06.1's `template`; it becomes `Limits { nesting: 1, ..SHIPPED }`. `src/tests.rs` gains `mod templates;`.
     2. 06.2's Files gain `src/tests.rs` (`mod statements;`).
     3. `a_conditional_after_in_must_be_parenthesised`: `Misplaced` at the `if` is at byte 14 of `{% for x in a if c else b %}`, not 12.

     The check's runs: clippy over the workspace with the sketches on stubs, clean; the crate and its world 42 passed (4 sketch tests stubbed), fuzzy 1 of 1. A further `Limits` literal with named fields takes `..SHIPPED` the same way, and the commit body lists it.

113. **hearth 26.1's size, 747 against 290** (plan defect, hearth 26's block of 2026-10-17T16:49:59Z).
     A plan defect, in 26.1's Size: "Size: at most 290 lines (the ledger is
     26.2's)" (`hearth/26-fake-git.md`, 26.1) undercounts the Shape it gives,
     which the prepared commit implements in full. 26.1 lands as prepared
     (0af8772), at 747 lines by code-organization.md 7's command, with the
     gate, and its report's `Size:` gives 747 and this ruling.

     - implementer.md's tolerance ("An increment lands within twice its Size
       without asking … Past that, stop on a question before landing, with the
       count by file") was followed: 747 is 2.58 times 290.
     - Splitting it now would land the same lines in two commits, behind a
       second gate, with nothing between them: 26.1's Shape is one face
       (`Op::Open` to `Op::Close`), and only 26.2 needs it within the session.
     - code-organization.md 7's aims hold: its largest file, `files.rs`, is
       456 lines, under 800, and the report gives every test under 40.

     26.2 keeps its Size, 340, and implementer.md's tolerance: past 680, it
     stops with the count by file, as 26.1 did.

114. **hearth 25's Shapes against main** (ready check of hearth-25.1).
     hearth 25's Shapes against main (ready check of hearth-25.1), against hearth cea230d and skein 303c829: `hearth/25-diff.md` corrected.

     1. 25.1's Needs gain hearth 32.1: `src/parts/config.rs` is new in 32.1 (`32-static-binary.md`, its Files), and 25.1 changes it.
     2. 25.1's Files gain `src/tests/outcome.rs`: `OUTCOMES: [Outcome; 4]` and `WORST: [[Outcome; 4]; 4]` (`:2`, `:3`) grow to 5 with `Negative`, between `TryAgain` and `Done`.
     3. 25.1's Files gain the render world: `tests/render/src/parent.rs:43` (`Answer::Rendered { outcome }`, which takes `diff`, and explicit arms for `Answer::Change`, `Request::Hunks`, `Event::Diff` and `Hunks`), and `tests/render/tests/thin.rs:102`'s match (an `Answer::Change` arm): `tests/render/Cargo.toml:22` denies `wildcard_enum_match_arm`.
     4. `fixtures::spec(target, parts)` goes beside `meta()` (`crates/hearth-domain-render/src/tests/fixtures.rs:45`); `x1_prod` is the world's (`tests/render/src/fixtures.rs:33`).
     5. render.md is read at 31bed37, its last change on main; the sections 25 names are there.
     6. The real-git reference: render.md 6.3's example is a 3-line-context hunk (`@@ -9,5 +9,5 @@`), so "not 6.3's `--unified=0` example" goes.

     The check's runs at cea230d: fmt, clippy and nextest 210 of 210, fuzzy 4 of 4. Done when 3 waits on 11.2 and 11.3, as planned. A further exhaustive match on the enums 25.1 grows gains its arms the same way, and the commit body lists it.

115. **hearth 32's Shapes against main** (ready check of hearth-32.1).
     hearth 32's Shapes against main (ready check of hearth-32.1), against hearth cea230d (and 46e5285, 0fa1b84, dce2e8b, which touch nothing 32 names) and skein b888da9: `hearth/32-static-binary.md` corrected.

     1. 32.1's Files gain render `src/lib.rs` (`:38`'s `match ev` needs `Event::Cached`) and `src/limits.rs`: main's `Limits` is `{ targets, document }` (`limits.rs:3` to `:7`), and 12-compose.md:181 and :195 use `static_file`; the sketch gains `static_file: u32`, `SHIPPED`'s 67_108_864 (render.md 2.3's 64 MiB for a static file). `SHIPPED` is its only literal with named fields; `tests/render/tests/thin.rs:128` takes `..limits::SHIPPED`.
     2. 32.1's Files gain the render world's `src/parent.rs`: `:87`'s event match gains `Event::Cached`, and its `Read` answers (`:49` to `:69`) answer a blob asked `As::Cached` with `Event::Cached` (size, `skein_lib::sha256` of its bytes) and one asked `As::Declaration` with its bytes.
     3. The Shape names the new variants' match sites: `call.rs:101`, `compose.rs:22` and `:31`.
     4. `tests/render/src/fixtures.rs:19`'s `Layer { placement }` becomes `Layer { placement, tasks: Box::new([]) }`.
     5. `src/tests/revision.rs:11`'s `(b"tasks/motd/task.json", Class::Other)` becomes `Class::Declaration { task: b"motd" }`; `tasks/motd/tree/etc/motd` is `Class::Tree`.
     6. `parts.rs:24`'s `assert!(parts.is_empty(), "no part records before config")` goes: `spec_digest` is H over the map of parts and the meta.

     The check's runs, with its 13 code corrections applied: fmt clean; clippy clean (4 E0004 before the arms, and 2 in `tests/render`); nextest 210 of 210, among them `the_config_digest_is_h_of_part_config_over_its_tree` and `the_spec_digest_is_h_of_spec_over_its_parts_and_meta`, so both digests reproduce; fuzzy 4 of 4. 32.2 was checked by names only: its suites need 11.2, 11.3 and 40.2. A further exhaustive match on a grown enum gains its arm the same way, and the commit body lists it.

116. **hearth 39's Shapes against main** (ready check of hearth-39.1).
     hearth 39's Shapes against main (ready check of hearth-39.1), against hearth b9b8e06 and skein 2112fce (moved since to hearth c884571 and skein b888da9, in no file 39 reads): `hearth/39-tree-world.md` corrected.

     1. Shared: the root `Cargo.toml` has `members = ["crates/*", "tests/*", "testing/hearth-*"]` (`:5`) and `skein-heap` (`:22`), not `hearth-protocol-tree`, without which `tests/tree`'s manifest fails to load: 39.1 adds it to `[workspace.dependencies]`, and `Cargo.lock` changes. `tests/clippy.toml` and `.config/nextest.toml` are on main.
     2. `worst_case` counts `held(kind)` too: a read that fails holds the record it built so far. Without it, `reading_generated_cut_and_mutated_frames_holds_no_more_than_the_worst_case` fails, 340,414 bytes held against 331,072; with it, it passes.
     3. `None` past a `u64` holds for a resolution only (`lines`, `indices`, `inputs` at `u32::MAX`): at its limits' maxima `worst_case(Store)` is `Some(832)`. The test becomes `worst_case_and_held_are_none_past_a_u64_for_a_resolution_only`; the sketches' docs say so.
     4. `check::read` has a `# Errors` section: the world's own `[lints]` deny `missing_errors_doc` (the root's `Cargo.toml:61` allows it).

     The check's runs, with these applied on a scratch world: clippy clean (it failed on 4 before); default 229 of 229, 9 of them hearth-tree-world; fuzzy 6 of 6, 2 of them tree. The reference memory test took 0.82 to 0.89 s against the 0.5 s aim, 0.3 s of it the check's own filename search. A further test or sketch that names the old bound takes the new one, and the commit body lists it.

117. **hearth 14's Shapes against main** (ready check of hearth-14.1).
     hearth 14's Shapes against main (ready check of hearth-14.1), against hearth cea230d and skein 2112fce (since moved by hearth 37.1 and 37.2, whose `Op` and `Missing` match the Interface sketch variant for variant): `hearth/14-evaluate.md` corrected.

     1. `a_lookup_of_the_wrong_type_fails_even_in_a_chain` takes `settings.peers[0]`, an integer into a map, in place of `settings.tags[-1]`: format.md 7.5 says "A negative index is outside the list", which is `NoIndex` (`an_index_outside_the_list_fails_at_its_bracket`, 2 of 2 and -1), not `Type`.

     The check's runs, with 14.1 and 14.2 built from the sketches: fmt and clippy clean; default 222 of 222; fuzzy 4 of 4. Its search found no user of `Limits`, `SHIPPED` or `Missing` that the Files miss. A further case listed under the wrong kind of fault moves to its kind's test, and the commit body lists it.

118. **hearth 27's Shapes against main** (ready check of hearth-27.1).
     hearth 27's Shapes against main (ready check of hearth-27.1), against hearth b961a12 and skein 2112fce: `hearth/27-fake-mmdebstrap.md` corrected. No hearth 26 or 27 step had landed, so its names were checked against 26's plan.

     1. The quote of testing.md 3.6 takes its whole relations row (`:416`): "Depends, Pre-Depends, Provides, Conflicts, with their version relations".
     2. The fixtures' index is the pin's, `20261017T143929Z` (`testing/mirror/pin`: `archive 20261017T143929Z`), which 08's cache holds, not `20260928T022309Z`; the pin's index has 78,830 relation fields. 41-resolve-recordings.md:58 still names x1's index of `20260928T022309Z`, with counts the 41 check reads against the pin.
     3. 27.1's Files gain `src/lib.rs` (`pub mod hearth;`, unless 26.2 made it public): 26.2's lib.rs re-exports `hearth::Machine` (26-fake-git.md:129), and 27's tests name `hearth_fake_machine::hearth::universe::Universe`.
     4. 27.2's Files gain the `pub mod`s `mirror` and `draw`, and `src/tests.rs` (`mod mirror;`).
     5. The sketches' `pub fn`s that return a value carry `#[must_use]`, elided: 18 fail the workspace's pedantic `must_use_candidate` without it.
     6. `DEBIAN` is invented in the index's shape: Debian keeps the initramfs alternation on `linux-image-<version>-amd64`, and the pin has dracut 112-7, initramfs-tools 0.151 and tiny-initramfs 0.1-8.

     The check's runs, on stubs of 27's sketches: clippy clean once `#[must_use]` was added; default 224 of 224 landed tests (3 stubs `unimplemented!()`); fuzzy 4 of 4. The closure counts 66 and 157 were not checked. A further fixture or comment that names the old timestamp takes the pin's, and the commit body lists it.

119. **hearth 34's Shapes against main** (ready check of hearth-34.1).
     hearth 34's Shapes against main (ready check of hearth-34.1), against hearth b9b8e06 and skein 2112fce (since moved, in no file 34 names): `hearth/34-conformance.md` corrected. The first two were run against real git 2.47.3.

     1. `run::git` takes `env` pairs beside `environment()`'s: under `env_clear` alone `commit-tree` exits 128, "Author identity unknown", and the `1700000000 +0000` date cannot be set. The Scenario passes `GIT_{AUTHOR,COMMITTER}_{NAME,EMAIL,DATE}`; a case passes none.
     2. The 4 KiB stdin bound is `cat-file`'s: the Scenario's `hash-object -w --stdin` takes the 10,000,000-byte file whole.
     3. The Shape names what 34.1's test sketch uses: `Settings::whole()` beside `cut(seed)` (`tests/git/src/world.rs:19` has only `cut`), `Scenario::run` and `expected`, `Case::is_listing`, `Run::stderr_text` and `recording`.
     4. The root `Cargo.toml` gains `hearth-recordings` and `hearth-git-world`; `skein-scratch` is there (`:19`).
     5. 34.2's Files name their crate, and gain `crates/hearth-protocol-git/src/tests/fixtures.rs`, whose drivers `listed` (`:9`) and `batched` (`:31`) get a `hunked` beside them.
     6. 34.2: `SMALL` (`tests/git/src/fixtures.rs:4`) gains `hunk_lines: 8` and `hunk_file: 64`, else E0063; `world.rs:232`'s and `:256`'s `match self.decoder` gain `Decoder::Diff`, else E0004.
     7. `succeeded` holds for a diff's exit 1 (`invocation.rs:131`: `Command::Diff { .. } => exit == Exit::Code(1)`): an exit 1 with nothing on stdout is the decoder's `Malformed`; exit 0 and an exit past 1 fail `succeeded`. Each is a tool error (render.md 2.2).
     8. This machine's git is 2.47.3: it prints the same `a/./cache/<64 hex>`, exits and lines, and its recordings go in `2.47.3/`, which Done when 2 needs whole.

     The check's runs, with 34.2's types and stubs applied and a stub `conformance_git`: `scripts/check.sh` passed (fmt, clippy, default 221, fuzzy 4); `--profile conformance` 1 of 1. The nextest edit and `conformance_git` land together, as the plan says. A further limits literal or exhaustive match the new fields and variants reach takes them the same way, and the commit body lists it.

120. **hearth 44's Shapes against main** (ready check of hearth-44.1).
     hearth 44's ready check, against hearth b961a12 and skein 2112fce (fmt, clippy, nextest 226 and fuzzy 4 pass), corrects 44-fake-git.md:
     - render.md 2.2 is pinned at `31bed37`, not `175e92f`;
     - `function`'s 80-byte cut also drops a UTF-8 sequence it split (git 2.47.3);
     - a missing side exits 1 as git's answer when it cannot read a file (render.md 2.2), not as a tool error;
     - of the five cases that decode to no answer, four are 02.1's `succeeded` and `diff-missing-side` is 34.2's `Malformed`, exit 1;
     - `hearth_git_world::expect` gains the ten case names in `CASE_NAMES` as well as `Expected::Failed`.
     44.2's `step` ends a child by a pending TERM (`Exit::Signal(15)`) or a broken stdout (`Exit::Signal(13)`), as testing.md 3.5 lets a process end by a signal; 26.2's `Op::Exit` took only a code, so it becomes `Exit { exit: Exit }` (26-fake-git.md), and session 26 is steered to it.

121. **hearth 03's Shapes against main** (ready check of hearth-03.1).
     hearth 03's ready check, against hearth dce2e8b and skein b888da9 (fmt and clippy pass, nextest 261, fuzzy 5; skein 02.1, 02.2, 02.s1 and 02.3 landed), corrects 03-json.md; main moved since to hearth a6d270f and skein aaddf88, in no file 03 reads (`git diff --stat`: skein-lib's `digest.rs`, hearth's fake machine, template, tree and plan files):
     1. The sketches' `tokenizer`, `Decoder::new`, `Decoder::waiting`, `worst_case`, `identifier` and `member` take `#[must_use]` (the root's `pedantic`, `Cargo.toml:60`).
     2. 03.2's Shared becomes `Cargo.lock`: the `tests/*` glob (`Cargo.toml:3`), the switched `.config/nextest.toml`, `tests/clippy.toml` and four `tests/*/tests/fuzzy_*.rs` are on main, so the Rules' first-fuzzy-test bullet goes, with nothing in its place. 03.1 still adds `hearth-protocol-config` to `[workspace.dependencies]` by path.
     3. 03.2's `world::check` gives a `Run` whose `Run::value: Option<Value>` is the document decoded: 03.3's and 03.4's proofs read it.
     4. skein's lines: `tokenizer.rs` `Limits` :66, `Tokenizer::new` :290, `restart` :323, `up` :379, `down` :414, `text_capacity` :603; `tests/json/src/world.rs`'s `check` :279.
     5. The opening's 03.1 is what hearth 11.2 needs through 03.3 (11-thin-story.md:175), not 03.1.
     Three design questions go to the designer in one question.md: (a) a container's `mutable` [M5] in M1 (format.md 2.2 and 3.4); (b) where a tokenizer fault inside a JSON string is (render.md 7.1 against skein json.md 3.6); (c) whose world `hearth-tree-world` is (testing.md 8.1 against the plan's code-organization.md). Interim, until the answer to (b) reaches session 03: 03.1 starts; where a tokenizer fault inside a string or a key is waits for it, no test asserts it, and a 03.1 ready to land before it blocks, kind design, naming (b) (03-json.md, 03.1's Shape). (a) bears on 03.3 and (c) on no step of 03. A further line number or sketch these corrections move takes the new value, and the commit body lists it.

122. **hearth 03's three design questions answered** (design answer, hearth 03.1's ready check of 17:08).
     The designer's answer, landed as hearth 3f16a56:


     (a) **Refused.** Before M5, a container's `mutable` is refused at its
     key, whatever its value, as its `ids` is: format.md 2.2 now lists "a
     placement's `ids`, `binds`, `capabilities`, `network` and `bridge`, and
     a container's `mutable`, whatever its value (3.4)", and 3.4 says
     "Render accepts it in every layer placement may be in, on metal and a
     VM, and on a container from M5 (2.2)". The fault is a later
     milestone's key (render.md 7.2's `schema` row), not a field the kind
     never takes. So 03.3 refuses it as it refuses `ids` and `binds`
     (`Shape::Unknown`, in `a_placement_field_m1_lacks_is_refused_at_its_key`,
     with `true` and `false` rows), not as `Shape::Misplaced { field: Mutable }`.
     `Field::Mutable` stays, for a `mutable` that is not a bool.

     (b) **The tokenizer's position stands.** render.md 7.1 now says: "In a
     JSON string or key, a fault of what is written, which skein's tokenizer
     finds (`Escape`, `Surrogate`, `Utf8`, `Control`, `StringTooLong`), is
     where the tokenizer places it: the first byte of the character or the
     escape at fault (skein's json.md, 3.6). A fault of the text decoded,
     which hearth finds once the tokenizer has read the string (U+0000, a
     name out of form, an expression's syntax), is at its opening quote."
     In 03.1, the interim goes: `Failed(e)` is `Error { at: position(),
     fault: Json(e) }` in a string or a key too, and `Nul` stays at the
     opening quote. 03.1 gains
     `a_bad_escape_in_a_string_or_a_key_fails_at_its_backslash` (`"a\q"`,
     as a value and as a key). 03.2's reference is unchanged: skein 02's
     reference checks a `Json` fault's error and position.

     (c) **The landed crate keeps the name.** `hearth-tree-world` is the
     tree's world crate. testing.md 8.1's row is now
     "`hearth-protocol-tree`'s records, cut and mutated, its machine world
     (2.4); from M2, also the tree helper's domain, on files with namespace
     ids", from M1. testing.md 2.9's `tests/<name>` is
     "hearth-<name>-world: one subject's domain and machine worlds (2.4)".
     The plan's code-organization.md (:37) and 39-tree-world.md stand
     unchanged. No step of 03 changes.

123. **hearth 27's Shapes against main** (re-check of hearth-27.1).
     hearth 27's re-check after hearth 26, against hearth eb6b8e4 and skein 78dd8fa (fmt and clippy pass with the corrections; nextest 42 passed, 3 skipped; fuzzy 0 tests), corrects 27-fake-mmdebstrap.md for 27.2 and 27.3; 27.1 landed meanwhile (4bc26d0) with the same fixes in its own code (no `#[expect]` on `Machine`, `#[should_panic(expected = "one package a name")]` at `tests/universe.rs:127`), so its Files, its test and its line at :131 stay as written:
     1. The interface sketch's `Machine` takes no `#[expect(clippy::zero_sized_map_values)]`: `Tool` is `#[repr(u8)]` (`hearth.rs:9-13`), so the lint does not fire and the `expect` is unfulfilled; 27.2 adds `served` without it.
     2. The must-use rule is "Every `pub fn` that returns a value other than a `Result` and takes no `&mut` carries `#[must_use]`": a `Result` is must-use already (`clippy::double_must_use`, on `Universe::read` and `closure`).
     3. 27.2's `Suite::new` line and 27.3's `assert_eq!` of the closure's names are wrapped under max_width 120 (`let suite =` then `Suite::new(...)`; each `assert_eq!` argument on its own line).
     4. Done when 2 names at least 66 tests (hearth 26's 39 in hearth-fake-machine, `git grep -c '#\[test\]' 4bc26d0~1 -- testing/hearth-fake-machine`: files 15, ledger 4, processes 20; this session's 27).
     5. 27.3's root `Cargo.toml` edit is its `[workspace.dependencies]` entry alone: `members` takes the crate by `testing/hearth-*` (`Cargo.toml:5`).
     A further line or sketch these corrections move takes the new value, and the commit body lists it.

124. **hearth 04's Shapes against main** (ready check of hearth-04.1).
     hearth 04's ready check, against hearth d7e42fe to 4bc26d0 (03.1 landed as 9ec5519) and skein 78dd8fa to 6141021 (clippy over the crate's targets passes with the sketches against the landed 03.1; the two unit sketches fail by design against a stub; fuzzy 0 tests; 03.2's world is not landed, so its world sketch compiled against a stub alone), corrects 04-declarations.md:
     1. `users` take `gid` as format.md 5.6's table gives it (format.md:891 has the `gid` row): "which 5.6's table lacks" goes.
     2. Each string of a `Literal::Strings` is interpolated, one by one (format.md 5.2: "and a list's strings one by one"): it joins the interpolated `Text`s, in the Choices and the sketch.
     3. A guard's `message` crosses as written, and hearth 16 refuses one that holds `{{`, `{%` or `{#` (format.md 5.2; 16-entries.md's `a_message_holding_a_tag_or_a_comment_is_refused_at_its_string`): 04.1 does not look into it.
     4. 04.1's test fixtures gain `at(offset, line, column) -> Position` beside `text`: 03.1's landed `src/tests/fixtures.rs` has `decode`, `document`, `decode_under` and `past_shipped` alone.
     5. `world::declare` reads 03.2's `Run::value`, which 03-json.md gives (ruling 121), and is documented with `# Errors` unless 03.2's `tests/config` `[lints]` allows `missing_errors_doc`.
     6. 04.1's Files add, where 03.2's world names every `Fault` (`Seen::EVERY_FAULT_AND_ENDING`, `tests/fuzzy_documents.rs`, `src/reference.rs`), `Fault::Declaration`.
     One design question goes to the designer in question.md: the form of a guard's name (format.md 2.4 gives none; 5.9). Interim, until its answer reaches session 04: 04.1 decodes a guard's name as a `Text` at its key and checks no form, no test asserts one, and a 04.1 ready to land before the answer blocks, kind design, naming the question. A further line number or sketch these corrections move takes the new value, and the commit body lists it.

125. **hearth 41's Shapes against main** (ready check of hearth-41.1).
     hearth 41's ready check, against hearth a6d270f to c00ac09 and skein aaddf88 to 6141021, its suites on 9ec5519 with hearth 34.1 landed (`scripts/check.sh` through `heavy --gate` passes: default 358, fuzzy 10; conformance 13 of 13, resolve 8 of 8, under `HEARTH_RECORD=1` and again compared equal), corrects 41-resolve-recordings.md:
     1. The forky index the Choices describe is the pin's, `20261017T143929Z` (`testing/mirror/pin`), not `20260928T022309Z`; its facts on test-boot's names, both MTAs and `hearth-absent` hold there.
     2. What `--skip=cleanup/apt/lists` leaves is quoted from render.md 10 (:1366-1368): "leaves `InRelease` and the decompressed `Packages` files in the scratch directory, as the recordings show (9.6)".
     3. An `Inst` line's architecture is the `[…]` that closes its parenthesis, before the `)`: apt ends every `Inst` line with a further ` []`, so "the last `[…]`" fails test-min's stanza lookup.
     4. `unsatisfiable` gets past no closure: chrootless mmdebstrap installs the essential set first, so apt prints its `Inst` lines (56 at the pin), none naming postfix or exim4, then `E: Unable to correct problems`, exit 25; its test is `conflicting_requests_fail_after_the_essential_set_naming_neither_mta`, and asserts those.
     5. `Recording`'s `lists` and `stanzas` are `Option<Vec<u8>>`, as its other fields are, and `resolve::stanzas` returns `Result<Vec<u8>, Box<[u8]>>`.
     6. `Run` is 34.1's `{ exit: hearth_protocol_git::invocation::Exit, captured: Recording }` (`tests/conformance/src/run.rs:10`), `Exit` imported from there; `Recording::exit` is its text ("0", "signal 9"); `Run::recording()` is unmasked, so `Resolved::recording` masks it with `Mask::recording(&mut Recording)` (`testing/hearth-recordings/src/lib.rs:135`).
     7. The users of `Recording {` are `testing/hearth-recordings/src/lib.rs:48` (`read_at`, which reads the optional `lists` and `stanzas` files, as `write_at` writes them), `src/tests.rs:25` and `tests/conformance/src/run.rs:56`; `tests.rs:55`'s `..original` needs none.
     8. `tests/conformance`'s own `[lints]` keep pedantic (`Cargo.toml:22`), so each `pub fn` returning a `Result` has `# Errors`, and each other one returning a value `#[must_use]`.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

126. **hearth 44's Shapes against main** (re-check of hearth-44.3).
     hearth 44's re-check after hearth 26 and 34.1, against hearth 00604e8 (44.1, 44.2) and c00ac09 (44.3, with 44.1, 44.2 and 34.1 landed), skein 78dd8fa (clippy clean on `hearth-conformance-world --all-targets` with the corrected sketch; default 400, fuzzy 10, conformance 5 of 5), corrects 44-fake-git.md's 44.3:
     1. The gate test's sketch compares `a.case == &**case`, and `(&run.captured.stdout, &run.captured.stderr, &run.captured.exit)` with the recording's three fields: 34.1's `Run` is `{ exit: Exit, captured: Recording }` (`tests/conformance/src/run.rs:10`) and `Recording::exit` a `Box<str>` (`testing/hearth-recordings/src/lib.rs:15`), so `fake_git::run` fills `captured`, its exit as text ("0", "128", "signal N").
     2. `REPOSITORIES` holds files by path, mode, contents and length alone (`tests/conformance/src/git.rs:4-14`); the identity and date (`IDENTITY`), the commit messages ("scenario\n", "submodule\n"), the tag `v1` ("scenario tag") and `refs/heads/main` are code in `src/fixtures.rs:15-22` and `:77-110`, which 44.3 makes data that real git's side and the fake's both read (both files are in its Files).
     3. The real side is `run::git(args, dir, stdin, env) -> Run` (`run.rs:26`).
     4. The recording's `dir`, `/scratch/store`, is spawned as `scratch/store` under the root handle, as 44.2's `tests/fixtures.rs` does: an absolute one is `Refusal::Escape` (`testing/hearth-fake-machine/src/linux/files.rs:118`). `big` is 10,000,000 bytes of `x`, by `length`; the repositories are bare at `/scratch/<name>`, the store at `/scratch/store`.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

127. **hearth 46's Shapes against main** (ready check of hearth-46.1).
     hearth 46's ready check, against hearth eb6b8e4 and again eeca0d2 (27.1, 27.2, 34.2 and 44.2 landed; 27.3, 38.1, 41 and 44.3 not), skein 78dd8fa, hearth's skein pin 303c829 (fmt and clippy clean with the corrections; default 427 of 427, fuzzy 11 of 11; the conformance profile could not run 46's tests), corrects 46-fake-mmdebstrap-resolve.md:
     1. `programs` has no `expect` (`testing/hearth-fake-machine/src/hearth.rs`; ruling 123): 46.1 adds `Tool::Mmdebstrap`, and `Tool` keeps its `#[repr(u8)]`.
     2. `Spawned`'s `argv` and `env` are `&'a [Vec<u8>]`, as `Recording`'s are (`testing/hearth-recordings/src/lib.rs:9-10`): `&real.argv` fails E0308 against `&[Box<[u8]>]`.
     3. `Spawned::argv` is whole, `PROGRAM` first, as every recording holds it (`/usr/bin/git` first in each git recording): the live side puts `PROGRAM` before `Invocation::args`, and `run` splits it into `Spawn::program` and `Spawn::args`.
     4. A recording's `dir` is absolute (`/scratch`): `run` spawns it without its leading `/`, under the root handle, as an absolute one is `Refusal::Escape` (`linux/files.rs:118`).
     5. `CONFORMANCE`'s literal names `hearth_fake_machine::linux::Sizes` in full, as its type does.
     6. Done when 1 names at least 74 tests (hearth 26's 39 in hearth-fake-machine, ruling 123; hearth 27's 27; this session's 8).
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

128. **A guard's name is an identifier** (design answer, hearth-04.1's ready check of 17:26).
     The designer's answer, hearth 180c4a5 ("Name a guard as an identifier"), decides it; ruling 124's interim ends:


     A guard's name is an identifier. format.md 2.4's table gains the row:

     > | guard (5.9) | `[a-z]`, then `[a-z0-9_]` | printed in its error, never in an expression |

     and 2.4 gains the bullet:

     > **A guard's name has their form,** though no expression reads it: a
     > false guard prints it in its error, `boot: requires modules: no
     > initramfs modules for this hardware` (render.md, 7.3), and a key of any
     > bytes, a `: ` or a line feed in it, would break that line. A name out
     > of form is a decode error at its key.

     5.9 now reads: "**`requires`** is a map of guards, each named as 2.4
     says and with `if` and `message`."

     For hearth 04.1 (04-declarations.md, as landed):
     - The Shape says: "`requires`: guards, each named by an identifier
       (`names::identifier`, else `Name` at its key, before its value) and an
       object of `if` and `message`, strings, both required." So `"Kernel": 5`
       gives `Declared::Name` at the key, not `Wrong` at the value.
     - The tests gain `a_guard_named_out_of_form_is_refused_at_its_key`, with
       the rows `Kernel`, `no-kernel` and `""`. `names::identifier`, on main
       in `crates/hearth-protocol-config/src/names.rs`, returns false for each.
     - The `Guard` sketch's comment reads "its name, an identifier (2.4), at
       its key". `Declared::Name` stands as sketched: "a name out of its form
       (2.4), or a script 5.8 does not name, at it".

     This replaces ruling 124's interim ("04.1 decodes a guard's name as a
     `Text` at its key and checks no form, no test asserts one"): 04.1 checks
     the form and its test asserts it.

     What stays unchanged:
     - 16-entries.md: its guards `ssh`, `web` and `greeting` are identifiers.
     - hearth 40's `Guard { name: Box<[u8]>, message }`.
     - render.md 7.2's decode row ("a declaration not of its shape").

     Nothing is deferred, so `later.md` gains no row.

129. **The gate compares a recording's stanzas where it has them** (plan defect, hearth-46.2's block of 17:52).
     hearth 46.2's stanzas in the gate (a plan defect). In the gate, `differ` compares each `Inst` line's stanza, reduced to `KEPT`, against a recording's `stanzas` where the recording has them, and compares a recording without them on the rest: the exit, the `Inst` lines, the error lines, the 07.3 failure, `lists`. The plan says so in its own words: 41.1 records "their recordings (4.5), with `lists` and test-min's `stanzas`" (41-resolve-recordings.md, 41.1's Scope); its Done when 2 counts "seven cases, one with stanzas"; and 46.2's sketch passes `real.stanzas.as_deref()`, an `Option`. 46.2's "none skipping it" now reads so (46-fake-mmdebstrap-resolve.md, 46.2's Shape).
     - The live test, `fake_mmdebstrap_leaves_what_real_mmdebstrap_recorded_for_each_case`, compares every case's stanzas, cut from the index live, as testing.md 5.2 asks of the resolve scenario ("the `Packages` stanzas it left, in the fields the universe keeps (3.6)"). Unchanged.
     - The gate derives no stanzas from `testing/mirror/universe`: the fake reads that universe, so stanzas derived from it would compare the fake with its own input. No recording is enriched; 46.2's Don't stands ("write a recording; ... run real mmdebstrap again").
     - `the_fake_prints_each_mmdebstrap_recording_as_recorded` counts the recordings whose `stanzas` it compared, and asserts at least 1 (test-min's, at the pin), so that the gate's stanza comparison runs at every landing. A recording that gains `stanzas` later is compared with no change to the test.
     - Unchanged: `ALLOWED`'s two rows and its rule, the universe's cut and its test, 46.2's Size.

130. **hearth 11.2's size, 896 against 300** (plan defect, hearth-11.2's block of 17:54).
     hearth 11.2's size, 896 against 300 (a plan defect, in 11.2's "Size: at most 300 lines"). 11.2 lands as prepared (cea43e4), with the gate, at 896 lines by code-organization.md 7's command; its report's `Size:` gives 896 and this ruling. 11.2's Size becomes 450.
     - The block is ruling 113's again (26.1, 747 against 290): a Shape that two new crates implement in full, with nothing between it and the next increment that needs it (11.3), and code-organization.md 7's aims held: its counted files are 11.2's Files alone, the largest `crates/hearth-protocol-builder/src/lib.rs` at 368 lines, under 800.
     - The same block twice becomes a standing rule (development.md, 10.4), in the README's "Rules of this plan": "**An increment past twice its Size** (rulings 113 and 130; it replaces, for this plan, code-organization.md 7's "Past that, it asks before landing" up to three times the Size): it lands at its count without asking when each counted file is in its Files, each file is at most 800 lines, and each test its Test shape names is there. Its report's `Size:` gives the count by file and names this rule. Past three times its Size, or with a counted file outside its Files, it still stops on a question before landing, with the count by file."
     - Unchanged: code-organization.md 7's count and its aims (a file of at most 800 lines, a test function of at most 40); 11.3's Size and Shape.

131. **hearth 46's Shapes against main** (re-check of hearth-46.2).
     hearth 46's re-check after 46.1, 41.1 and 27.3 landed, against hearth 3cbd5d1 to 3b48d0e (46.1 landed at 266e6b1 meanwhile) and skein 841f487 (clippy clean with the corrections; default 513, fuzzy 13, conformance 16 against real mmdebstrap 1.5.7 and apt 3.0.3; 46.1's Done when 1 at 89 tests; 46.2's two test sketches compile with `fake_mmdebstrap` stubbed), corrects 46-fake-mmdebstrap-resolve.md:
     1. The recordings' version is `1.5.7-apt3.0.3`, as `testing/recordings/mmdebstrap/` holds it (`Resolve::version`'s comment); 3.3.3 is the pinned index's apt package.
     2. A failed apt pass leaves `auxfiles` in the lists beside `lock` and `partial/` (`testing/hearth-fake-mmdebstrap/src/process.rs:74`), and `differ` compares the lists' names and kinds with it.
     3. `tests/conformance/Cargo.toml` gains `hearth-fake-mmdebstrap` and `skein-lib`: `hearth-fake-machine` is there since hearth 44 (`Cargo.toml:12`).
     4. The root `Cargo.toml` holds `hearth-fake-mmdebstrap`'s entry already (`Cargo.toml:22`): of the shared files, 46.2 changes `Cargo.lock` alone.
     5. `Outcome::Failed` is `{ essential, providers, failure }`, as landed (`testing/hearth-fake-mmdebstrap/src/face.rs:35`).
     6. `UNIVERSE` is `concat!(env!("CARGO_MANIFEST_DIR"), "/../../testing/mirror/universe")`, anchored as `Pin::committed` anchors its path (`testing/hearth-test-mirror/src/pin.rs:105`): a bare relative path resolves under `tests/conformance`.
     7. `class()` is `Cause::class` (`crates/hearth-protocol-mmdebstrap/src/tail.rs:30`), not `Tail`'s; `Tail::lines` returns a `Queue`, which has no `PartialEq`, and `tail::failure` takes `Reader`s (`tail.rs:262`): `differ` compares the lines one by one, and applies 07.3's "the stderr one unless empty" to the two tails itself.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

132. **hearth 28's Shapes against main** (ready check of hearth-28.1).
     hearth 28's ready check, against hearth 3cbd5d1 to 3b48d0e (46.1 landed at 266e6b1 meanwhile) and skein 841f487 (fmt and clippy clean with the corrections; default 516 passed, 1 skipped; fuzzy 13), corrects 28-builder-worlds.md:
     1. 28.1 repins hearth's skein in `Cargo.lock` to `main`, at least 841f487: the pinned 78dd8fa has no `Machine::state_digest` or `StateDigests` (6141021) and no 03.4's `Digests::Traced` (841f487), which 28.5 needs. `Cargo.lock` joins 28.1's shared Files.
     2. The root `Cargo.toml` gains `skein-sim` beside `skein-io` (`Cargo.toml:18`) and `skein-world` (`:33`), for `behind.rs`: it joins 28.1's shared Files.
     3. `behind()` lays `/srv/store` 0700 and the builder's, `.owner(BUILDER)` (`testing/hearth-fake-machine/src/linux.rs:70`, `src/tests/fixtures.rs:3`): laid as root, the builder's opens fail with `Permission`.
     4. 28.3's sketch takes `Form::JsonLines` (`crates/hearth-client-protocol-records/src/form.rs:12`); there is no `Form::Json`.
     5. 28.5's Files gain `crates/hearth-domain-render/src/call.rs`: `Call` (`call.rs:11`) derives `Debug` alone, and its `read` and `next` (`call.rs:19-20`) are `usize`, which become `u32` under the Shape's "no `usize`"; `Render` (`lib.rs:17`) and `Call` derive `Hash`.
     6. `PowerLoss` fails an `assert!` naming "M2": the workspace denies `clippy::panic` (`Cargo.toml:102`). `Hash` is derived on `hearth::Machine` too, written by hand only for a machine that holds the ledger; `src/linux/*.rs` names the files whose types lack `Hash`, not each one.
     7. The worlds' `Duration` is `core::time::Duration` (`std_instead_of_core`, `Cargo.toml:118`).
     8. The hosted fakes wrap git's `face::{check, Run}` and mmdebstrap's `face::{check, print}` over its `Simulate` (`testing/hearth-fake-mmdebstrap/src/face.rs:7, 45, 145`); mmdebstrap's face has no `Run`.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

133. **hearth 36's Shapes against main** (ready check of hearth-36.1).
     hearth 36's ready check, with `check/findings-36.md`'s points, against hearth 3cbd5d1 to cc68d36 and skein 841f487, its expected texts against MiniJinja 2.20.0 with render.md 9.5's settings (clippy clean with the corrections; the template crate and its world 74 run, 72 passed, the 2 failing only for 36.1's and 36.2's code; fuzzy 3; memory 1), corrects 36-template-whitespace.md:
     1. skein's `worst_case` is `tokenizer.rs:107`.
     2. 36.1's Files gain `src/tests.rs` (`mod whitespace;`) and `src/tests/fixtures.rs`, with a new `branch(at, condition, over)` as the sketch calls it; 36.2's gain `src/tests.rs` (`mod interpolation;`), and its world's `kind()` (`tests/template/src/fixtures.rs:50`, an exhaustive match) gains `ErrorKind::Interpolation => "interpolation"`.
     3. Inside the `raw` and `endraw` tags, whitespace is `u8::is_ascii_whitespace`'s, as MiniJinja's `skip_basic_tag` (lexer.rs:254) takes it: `{%\x0craw\x0c%}x{%\x0cendraw\x0c%}` is `x`, and a VT before `raw`, or a U+00A0 before `endraw`, makes no raw tag. Elsewhere it stays `White_Space`. The oracle run checks `\r` there with the other expected texts.
     4. A raw body empty as written, or after its edges' rules (`x{% raw %}  {% endraw %}z` is `xz`), gives no `Text`: `check.rs:79` forbids an empty one.
     5. A template's trailing newline is kept after text or `}}`; after an unmarked `%}` or `#}`, trim_blocks takes it (`a{# c #}\n` is `a`), so `trailing_newlines_are_kept_as_written` ends its templates in text or `}}`.
     6. A lone `-` between an opening and its closing marks the opening alone: `{#-#}`'s closing is unmarked, and trim_blocks takes the `\n` after it; `{{-}}` is an error, an output with no expression.
     7. In an interpolated string, only a `-` beside `{{` or `}}` is a marker, and `Interpolation` at it; a `+` there is no marker (36.1's rule), so `{{+ a }}` is `Expected(Operand)` at 2 and `{{ a +}}` at 6, as in a template. `ErrorKind::Interpolation`'s comment drops the `+`.
     8. 36.3's `lines` holds one `u32` per newline plus one (`scan.rs:9`), at most the text's length plus one.
     9. `Decoder::Interpolated`'s `length` is at most 03's `Limits::document` (`hearth-protocol-config/src/limits.rs:4`, 1,048,576); neither the template crate nor its world depends on `hearth-protocol-config`, so the memory test passes `SHIPPED`'s `template`, the same value.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

134. **hearth 35's Shapes against main** (ready check of hearth-35.1).
     hearth 35's ready check, against hearth e22d0e9 to d14d0dc (04.1 at cc68d36) and skein 841f487 (clippy clean with `settings` given to the `Limits` literals; default 513, fuzzy 13; the schema, 35.1 test and 35.2 fuzzy sketches compiled under clippy at e22d0e9 with stand-ins for what 35 adds), corrects 35-schema-exports.md:
     1. skein's `Stack::with_capacity` is `stack.rs:22` and `push` `:50`. 14's and 16's lines are given at f981746, as they say, and stand.
     2. The `Limits {` literals on main are `src/limits.rs`, `world::limits`'s two (`tests/config/src/world.rs:59` and `:65`), `tests/config/tests/world.rs:27` and `tests/config/tests/memory.rs:10`: each takes `settings`, or fails with E0063.
     3. 35.1's Files gain `src/tests.rs` (`mod schema;`).
     4. `TASKS` (`tests/config/src/examples.rs:32`) is a static `include_bytes!` table: each new example takes a row, `locale`, `boot`, `hearth` and `users` in 35.1, `vpn` and `backup-peer` in 35.2, and `examples.rs` joins both Files.
     5. The schema sketch takes `alloc::boxed::Box` and `crate::Position` (`lib.rs:13`).
     6. `at_the_limits`' documents (`tests/config/src/fixtures.rs:42`) are refused at their first unknown key, before any `Type`, `Spec` or `Export`: 35.2 meters `declaration` on a new `fixtures::declarations_at_the_limits(&Limits)`, the most settings, types and exports the limits allow, with `metered.rs` (`tests/config/src/fixtures/metered.rs:12`) metering `declaration` on the decoded `Value`; `metered.rs` and `tests/memory.rs` join 35.2's Files. `declaration::worst_case` counts the walk's `Stack` of `depth` frames too, and each list at its capacity.
     7. No rule gives `Unknown { of: Type }`: a type object's unknown key is `Wrong { expected: Type }` at its `{` (35.1), so `EVERY_DECLARED` takes `Unknown` of each `Object` but `Type`.
     8. `world::limits`' tiny branch (`world.rs:59`: document below 200, depth and keys below 5) holds no declaration of every key, and its JSON faults come first; its roomy one (`:65`) has `keys: 32`, below 512 settings. The sweep draws a new `world::declaring(rng) -> Limits`, the roomy values with `settings` drawn below `keys`, so that `Settings` falls; `world.rs` joins 35.2's Files.
     9. `Declared` is `Copy` already (`src/error.rs:24`), as the fuzzy sketch needs.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

135. **hearth 35.2's Shapes after 04.2** (re-check of hearth-35.2).
     hearth 35's re-check after 04.2 landed (b08a390), against hearth 055fba9 and skein 841f487 (every name and line of 35 holds; clippy clean with the interface sketches, the `Limits` users and the 35.1 and 35.2 sketches stubbed; default 82, fuzzy `fuzzy_documents` passing), corrects 35.2 in 35-schema-exports.md (35.1 landed meanwhile, at 992ff01):
     1. `TASKS` (`tests/config/src/examples.rs:32`) holds 23 rows, 25 with 35.2's: the sweep of task examples decodes the 23 that decode, and asserts `checks-ssh` and `artifact-stoa` refused as `Later`.
     2. `EVERY_DECLARED` counts `Object::Entry` once for each of its 14 `Map`s, and `Object::Range` (`src/error.rs:48`) is among the objects; a variant no input reaches is named in the report, not in the constant; `Tally::record` matches every variant (`wildcard_enum_match_arm` is denied).
     3. 35.2 removes `Key::Undecoded` too (`src/declaration.rs:70`, the `exports` row `:81`, the arm `:137`): Done when 3 greps `-w Undecoded`.
     4. `ROUNDS` is chosen so that `fuzzy_declarations` alone takes about 2 s (`-E 'binary(fuzzy_declarations)'`); `fuzzy_documents` takes 0.5 to 0.8 s of the profile already.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

136. **hearth 32.1 lands at its count** (plan defect, hearth 32's block of 2026-10-17T18:27:09Z).
     hearth-32.1 lands as prepared at 84ce356, at 956 production additions against its Size of 285 (README, Rules: "Past three times its Size, or with a counted file outside its Files, it still stops on a question before landing"). The plan's count was low, not the increment's scope: the re-check of 32 at hearth 992ff01 found Files that 32.1's text missed, and no counted file passes 443 lines.
     1. 32.1's Files gain the two counted files they missed: protocol `src/translate.rs` (`pub mod parts;`) and `crates/hearth/clippy.toml` (its comment: `unsafe` in the ring adapter only).
     2. 32.2's test sketch reads the scratch's path: `repository.scratch().path().join("xdg")` and `repository.scratch().path().as_os_str()` (`Scratch::path`, skein's `testing/skein-scratch/src/lib.rs:52`).
     3. 32.1's commit body gives the count by file and names ruling 136.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

137. **Fields, not an array read by constant index** (steer of 18:24).
     The hourly check of 2026-10-17T18:24:28Z saw a fixed array read by a constant index a second time, after the 17:12 steer's Pass on text had reached the sessions: hearth 14.1's `tests/render/src/expressions.rs:19` (`pub values: [Value; 5]`, read as `values[0]` to `[4]` there and in `reference.rs:32-36`), and the one-row tables left in fake-machine `src/tests/files.rs` by 27.2 (`CREATED[0]` :11, `DENIED[0]` :54, `ROOT[0]` :138, `CONTENTS[0]` :182, `LOCKS[0]` to `[3]` :147-151). The plan's README, Rules, gains "No fixed array read by a constant index" (development.md, 14.4): a test or a world names fixed elements as struct fields, or asserts the value as a literal; an increment that edits a test file first fixes its one-row tables and constant-index reads. Landed files stay until an increment edits them.

138. **hearth 12's Shapes against main** (ready check of hearth-12.1).
     hearth 12's ready check, against hearth 992ff01 to d0df92f and skein 841f487 (clippy clean with the corrections; default 601 of 601, fuzzy 13 of 13; the six render, builder and records packages 95 of 95 once the fixtures below take the new shape; real git's `a_broken_document_prints_its_error_after_the_specs_that_rendered` gives the plan's stdout and exit 2), corrects 12-compose.md, its names grepped again at hearth 1dc5d6f:
     1. skein's `tokenizer::Error` is `tokenizer.rs:232`, and `Stack` is `stack.rs:15`.
     2. Errors are answered one a `resume`, then `Rendered`: render's `MAX_OUT_RESUME` is 1 (`lib.rs:16`), and the output queue holds one.
     3. `depth: 32` is on main already (render `limits.rs:6`, `SHIPPED` `:15`, 14.1's): 12.1 adds only `decoded`.
     4. 12.1's Files gain render `src/tests.rs` (`mod compose;`). 12.2's gain render `src/tests.rs` (`mod errors;`); the root's `src/lib.rs`, whose `forward` (`:35`) matches render's `Answer` exhaustively; protocol `Cargo.toml` (`skein-json`, for the tokenizer's variants), `src/translate.rs` (`pub mod errors;`), `src/client.rs` (`:107`'s match takes `Answer::Error`) and `src/tests/read.rs` (`:84`'s `Decoded::Refused` takes `fault: Missing { field: Kind }, also: None`).
     5. In `tests/render/tests/thin.rs`, the `Decoded::Refused` fixtures (`:87`, `:140`) take `fault` and `also`, the match at `:104` takes an `Answer::Error` arm, and `a_document_above_its_limit_fails_only_its_targets_before_reading` (`:115`) answers `[Spec, Error]`.
     6. `translate::errors::fault` takes `config::limits::Limits` (config `lib.rs:10`), and gives `None` for `Fault::Declaration` (`error.rs:21`; 13.1's `Decode::Declaration` replaces the arm) and for `Fault::Json(tokenizer::Error::Stream(_))`, which config's `json.rs:146` makes `Fault::Stream`, as well as for `Fault::Stream`.
     7. Done-when 2 runs `-p hearth-domain-builder` too.
     8. `Option::map` is a disallowed method (`clippy.toml:106`), and a hand-written `match` that rewraps `Some` trips `manual_map`: a helper that uses `?` passes.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

139. **hearth 23's Shapes against main** (ready check of hearth-23.1).
     hearth 23's ready check, against hearth 992ff01 (d0df92f touched none of its crates) and skein 841f487 (fmt and clippy clean with the corrections; default 603 of 604, the one failure the plan's real-loop test, which needs 23.2 and 32.1; fuzzy 13 of 13), corrects 23-store.md, its names grepped again at hearth 3ba7416:
     1. skein's `FileIo::adopt_root` is `file_layer.rs:363`, and `effective_user` is `ring.rs:1678`.
     2. `frame::largest` takes `(Kind, &Limits)` and gives a `u64` (hearth-protocol-tree `frame.rs:113`): the record's `Load` and `Files::whole` take `frame::largest(Kind::Store, &limits.tree)` and `frame::largest(Kind::Resolution, &limits.tree)`, narrowed by `u32::try_from`.
     3. 23.1's Files gain the root's `src/tests/fixtures.rs` (`:4`'s `LIMITS` takes `store`; `a_render_call_reaches_render`, `tests.rs:11`, sees the store's `Open` first), the service's `src/tests/fixtures.rs` (`:7`'s `Configuration` takes `uid`, `:8`'s `Roots` takes `resolutions` and `staging`), and `skein-lib` in `tests/builder/Cargo.toml`, for `Duration`, `Time`, `Wall` and `Queue`.
     4. The store's types are named by their module: `store::vocabulary::{Event, Request, StoreRecord, Why}`.
     5. `worst_case`'s doc names `Map::worst_case` and names of 255 bytes; there is no `Set`.
     6. The service's `Files` is in `src/limits.rs` (`:14`), as 32.1 put it; `Configuration` and `Roots` stay in `src/lib.rs` (`:21`, `:25`).
     7. The store's `MAX_OUT` is 1, as the root's (`hearth-domain-builder/src/lib.rs:10`): one store request is in flight, and each answer comes alone on a terminal, in each transition 23's Shapes give.
     8. An unopened store is said at startup. render.md 7.2's `store` row: "A store that cannot be opened, or is another uid's or another range's (store.md, 6.6), is no error of the answer: nothing renders, and the client says so at startup (client/README.md, 3.2)"; client/README.md: "What keeps the client from answering at all (an unknown option, a repository or a store it cannot open) goes to standard error alone: `hearth: <message>`, or with `--json` one `startup` record after the version line, naming its outcome, `usage` or `try_again`, with 3.3's code." On `Unopened { why }` the root emits its new `Request::Unopened { to: ReplyTo, why }` in place of `Reply { TryAgain }`; the protocol layer writes records' `Startup::Unopened { what: Opened::Store, path: None, error }` (`startup.rs:11`; hearth 33.3 gives the path) to standard error, and the call ends with 75. `error` is `lock failed`, `store.record failed`, or `store.record is uid <recorded>'s, not uid <builder>'s`. The root's test checks each `Why`, and a new protocol test, `an_unopened_store_writes_its_startup_record`, the three texts as literals.
     9. The opening paragraph quotes hearth 11's transitional entry as written: "the store's lock or `store.record`".
     10. A new unit test, `a_readable_record_of_layout_2_is_written_again_as_layout_1`, covers 6.1 and 6.8's "written again" for a readable record of another `layout`.
     11. The reference test's `let limits` line is wrapped at 120 columns, as rustfmt does.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

140. **hearth 43's Shapes against main** (ready check of hearth-43.1).
     hearth 43's ready check, against hearth 339ba63 to f725453 and skein 841f487 (clippy clean with the corrections; default 639 of 639 and fuzzy 17 of 17 at d0df92f, with a stand-in for 43.2's five functions), corrects 43-evaluate-translated.md, its names grepped again at hearth 76d7da7:
     1. 43.1's `tojson` sketch: `bytes` is the module `skein_lib::bytes`, so the string is `bytes::copy_of(..)`, with `use skein_lib::bytes;` (as render `src/tests/evaluate.rs:4`, `:199`); its `peers` line and 43.3's `expression` call are wrapped within 120 columns.
     2. 43.1's sweep asserts each fault kind fell at least once: `assert_fell` (world `src/fixtures.rs:102`) checks fault-kind names, the only ones `tests/fuzzy_evaluate.rs:15` records, and 43.1 adds `"empty"` and `"path"` to `expressions::KINDS` (`:16`).
     3. 43.2's world compiles at `Limits { expression: <the text's length>, ..SHIPPED }`: the template's `SHIPPED` limits an expression to 4,096 bytes (`limits.rs:12`), `compile` refuses past it (`expression.rs:44`, `TooLong`), and the generator's strings pass it in 25 of 200 SMALL seeds and 2,089 of 10,000 WIDE rounds, so "for every seed" holds only so. Its depth (`SMALL` 3, `WIDE` 6, world `expressions.rs:14-15`) stays under the nesting limit 64.
     4. 43.2's printer wraps by precedence: 37.1's generator (`tests/template/src/generate.rs:84-119`) wraps every operand but a chain before `is` or `default`, so it is no model for the rest.
     5. 43.3: `ELEMENT` is 14.1's (render `evaluate.rs:53`), as are `FaultKind::Empty` and `Path` (`:67-68`); `at_the_limits()` and `wide_roots()` are `#[must_use]` (pedantic `must_use_candidate`); `skein-heap` goes under `[dev-dependencies]` of `tests/render/Cargo.toml`, as `tests/template/Cargo.toml:12-13`.
     The `Path` rows of 43.1 stand: format.md's `path` is "absolute; no empty, `.` or `..` segment; no trailing `/`" (`format.md:1558`), though this machine's systemd 257 escapes `/a//b/`, `/a/./b`, `a/b` and `''`; the unit tests take literals, not the tool.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

141. **hearth 23.1's roots, seed and standard error** (steer of 2026-10-17T18:50:18Z).
     hearth 23's re-check after 32.1, against hearth 76d7da7 to a29cf9f and skein 841f487 (three tests in the service confirm 1 to 3; default 677 of 677, fuzzy 17 of 17), corrects 23.1 in 23-store.md:
     1. `Files::files` is 32 in 23.1, not 23.2: service `limits.rs:34` holds 2, so a third `adopt_root` (skein `file_layer.rs:363`) gives `None`; 23.2 changes only `entries`.
     2. At its terminal the protocol sends `file::Request::Close` for each adopted root's token: the service's `is_empty` (`lib.rs:102`) waits for `open_files() == 0`. `Client` (hearth `src/lib.rs`) drops its `store` and `cache` keepers, `is_empty`'s `&& self.store.is_none()` (`:78`) and `drain`'s closes, which would close them twice; `Client::open`'s error paths, before the service exists, still close them.
     3. `Service::new` calls `files.seed_randomness(seed)` (skein `file_layer.rs:374`): unseeded, `Store` fails `InvalidArgument` at admission (`file_layer/store.rs:176`), and `store.record` is never written.
     4. `Startup::Unopened` goes to standard error as the protocol's second output: `Service::adopt_error_pipe` and `adopt_error_file` beside `adopt_output_*` (service `lib.rs:72`, `:80`), `Client::open` taking `error: Fd`, which `main.rs:42` passes as `Fd::new(2)`, and protocol `client.rs:19`'s `output` gaining `error` beside it. 23.1's Files gain protocol `src/client.rs`, service `src/tests.rs` and the root's `src/main.rs`.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

142. **hearth 23.1's landing meets ruling 141** (process, hearth 23.1's block of 2026-10-17T18:51:12Z).
     Ruling 141 is withdrawn, and no step repairs 23.1: hearth 23.1 landed as d25571e at 18:49:29Z, before it, and meets each of its four points another way. 1. `Files::files` is 8, the four roots and the operations (service `limits.rs:35`); 23.2 raises it to 32, as its Files said before ruling 141. 2. The roots stay adopted until the shell's `drain` closes them: the service's `is_empty` counts `open_files() == 4` (service `lib.rs:130`), so nothing closes them twice. 3. `Service::new` seeds the file layer (`lib.rs:64`). 4. The protocol prints the startup record (protocol `lib.rs:182`), and the shell writes it to standard error (`take_startup`, `crates/hearth/src/lib.rs:115-116`). 23-store.md is ruling 139's text again, with these four as 23.1's "As landed". 23.2 builds on d25571e as it stands.

143. **hearth 42's Shapes against main** (ready check of hearth-42.1).
     hearth 42's ready check, against hearth 3ba7416 and skein 841f487 (clippy clean with the corrections, and hearth 12's additions stubbed; default 664 of 664, with a `Tail` case in the git world's memory test and three tail tests; fuzzy 17 of 17), corrects 42-listing-members-rules.md, its names grepped again at hearth bec2e47:
     1. 42.1: `Class` drops `Copy` (render `revision.rs:25`), as `Refused { why }` holds a box (E0204), so `src/tests/revision.rs:17`'s `for &(path, expected) in PATHS` iterates by reference (E0507); each match on `revision::class` in `call.rs` and `compose.rs` names `Template`, `Script`, `Readme` and `Refused`, with no wildcard.
     2. 42.1's Files gain `hearth-protocol-git` `src/tests.rs` (`mod tail;`), the git world's `src/world.rs` (`:238` and `:266` match `Decoder` whole, E0004 on `Decoder::Tail`; a `Tail` driver feeds it, `Action::Feed`), and protocol `src/tests/fixtures.rs` (`:190`'s `spawn.pipes.len() == 2` counts standard error's pipe).
     3. 42.1's git pipes: `Child::spawn` (protocol `git.rs:28`) builds `[stdout, stdin]`, and `spawned` (`:61`) takes `pipes.first()` and `pipes.get(1)`; once standard error is piped, each pipe is taken by its role, not its position.
     4. 42.2's Files gain render `src/tests/fixtures.rs` (the `Document` literals at `:240` and `:257` take `members: None`) and protocol `src/tests/read.rs` (`:57`, `:129`); `translate/documents.rs` (`:13`'s `members: _`), `src/tests/translate.rs` and the world's `src/fixtures.rs` (`:20`, `:112`, `:150`), listed already, change the same way.
     git 2.47.3 prints `fatal: not a tree object` and exits 128 for a missing tree, as the records' sketch says.
     A further line number or sketch these corrections move takes the new value, and the commit body lists it.

144. **hearth 43.2's round trips compare with their input** (process, hearth 43.2's block of 2026-10-17T18:56:12Z).
     Ruling 144. hearth 43.2 writes its round trips as 43-evaluate-translated.md, 43.2, prescribes: `fixtures::every_op() -> [template::Op; 26]` in `src/tests/fixtures.rs`, and `each_operation_translates_to_render_and_back` asserts `assert_eq!(back, op, "row {row}: hearth 05's Op through render's and back")`. `each_filter_translates_to_render_and_back` (17 rows) and `missing_translates_to_render_and_back` take the same form.

     A round trip's expected value is its input: the 17:12:49Z steer's "A test states its expected value as a literal in the assertion, or as a plain `const` of its type" holds for one-way tests, and a round trip's literal is its input row. `template::Op`'s `String(Box<[u8]>)`, `Name { name: Box<[u8]>, .. }` and `Key { name: Box<[u8]>, .. }` cannot be `const`, so the table is the session file's `#[must_use]` fn returning an array, which the one test loops over by value. It keeps the steer's other rules: two or more rows, no constant index, no `.first().expect(..)`.

     The plan's README gains this as a rule for every session's `…_and_back` tests (hearth 11, 12, 13, 15, 17, 21, 23, 25, 31 name such tests). Unchanged: one-way tests state literal expected values; 43.2's Files, Shape, Proof and Size.

145. **hearth-12.s1: a document's null inside a list** (steer of 2026-10-17T19:02:09Z).
     A supplement step, `hearth-12.s1` (hearth/12.s1-list-null.md), hearth 12's, needing 12.2 (landed, 0aa2743): a document's `null` inside a list becomes the decode error format.md 4.1 names ("one inside a list (`[1, null]`, `[{"a": null}]`), which replaces whole, is a `decode` error in a document (render.md, 7.2) and a bad default in a `task.json` (6.5)"). hearth 12's re-check (check/recheck-12.md, point 3) found no owner: config's `document` (`document.rs:27`) keeps it, render's `Decode` (`errors.rs:25`) has no variant, and builder `translate/documents.rs:73` maps a json `Null` to the document's `Null`. The `task.json` half stays hearth 13.3's (`Null`, "null in a default"). The step adds `Shape::ListNull` (config), `Decode::ListNull` (render and records: message `null inside a list`, code `list_null`) and their translations. Nothing else changes.

146. **hearth 13's Shapes against main** (ready check of hearth-13.1).
     hearth 13's ready check, against hearth 3ba7416 and skein 841f487, re-checked at hearth ca17d04 (clippy clean with the corrections; default 805 of 805, fuzzy 17 of 17), corrects 13-schema.md:
     1. 13.5: `Format` also derives `Copy`: the FORMS test's `forms::check(*format, ..)` is E0507 without it.
     2. 13.2's Files gain `tests/render/src/fixtures.rs`: `with_task()` (`:127`) pushes the unit `Decoded::Declaration`, which builds a `Declaration` once the variant carries one (E0308); `call.rs:159` and `read.rs:54`, the other two sites, are in Files already.
     3. 13.4's Files gain render `src/evaluate.rs`: `:52`'s `.saturating_add(5 * 67_108_864)` becomes `limits.validated.saturating_mul(5)`; `validated` is a field of its own, beside 12.1's `Limits::decoded` (`limits.rs:5`, `:16`).
     Done when 4 prints 0 at ca17d04 (ca17d04 named each field of `translate/documents.rs`'s two frames): no change.
     A further line number these corrections move takes the new value, and the commit body lists it.

147. **hearth 42's Shapes against main** (re-check of hearth-42.1).
     hearth 42's re-check against hearth 3fd9fab (check/recheck-42.md), after ruling 143: its four corrections hold, and 42-listing-members-rules.md now says five more. 42.1: `Class`'s seven matches (`revision.rs:152`, `call.rs:206` and `:289`, `compose.rs:28`, `:37` and `:80`, `errors.rs:72`), not two files'; `Event::Failed`'s other builders (render `lib.rs:74`, `tests/diff.rs:201`, builder `lib.rs:549`) in its Files; `Limits.tasks` (1,024, beside `targets`) for `TooLarge { Tasks }`; the new `Limit` rows in builder `src/tests/translate/errors.rs` (`:50`, `:230`), in the README's round-trip form (ruling 144). `Command::Diff` moves from 45.1 to 42.1: the hunk call spawns through `git.rs:28` like the others, so 42.1 gives its failure (`lib.rs:464`, `:549`) its `Tool`, `ended` and tail; 45.1 keeps a failed diff's error per target. 42.2: `read.rs:133` (was `:129`), and `tests/translate.rs:95`, `:102` take `members: None` too. 42.3's Removes: hearth 11 composes `placement` from `fleet.json` and a base (`call.rs:329`), which 42.3 refuses; it does not ignore it. Refresh any further copy of a moved line, and list it in the commit body.

148. **hearth 22's Shapes against main** (ready check of hearth-22.1).
     hearth 22's ready check (check/result-22.md, hearth 3ba7416) and its re-check (check/recheck-1322.md, hearth ca17d04; since then main moved render `errors.rs` by one line only): 22-collisions.md now says each correction. 22.1: skein's `Map` lines at hearth's pin 841f487; `Collector` derives `Debug` alone and `Order` is `Ord` with `unplaced` first; `worst_case` adds the errors' bytes; `Rendered { outcome, diff, left_out }` (25's `diff`); each `Side`'s collector where `call.rs:20` holds its list, the diff's two `Rendered` (`call.rs:479`, `:543`) carrying their sides' counts, so no `left_out: 0` stays; the Files it missed (protocol `client.rs:121` and `src/tests/client.rs`, root `lib.rs:54-55`, `tests/render/src/parent.rs:45`), the terminal test in `src/tests/client.rs`, `fixtures::kept` and the byte-slice `Spot` rows. 22.3's `TREE_TWICE` puts `banner.txt` at the repository's root. Done when 6's regex, and 17's to 21's copies of it, match no enum variant named `String`. Two plan defects: 22.4 names a data line's task, so 18's `data::Entry` carries `task` and `at` as its `User` does, and `data_lines` returns them; `Hidden` and `Overlap` get message forms in Choice 8, from 22.3's proof line and render.md 3.7's words ("paths collide when one is the other or lies beneath it", "ranges never overlap"), and `Hidden` is `at` the data entry. Refresh any further copy of a moved line, and list it in the commit body.

149. **hearth 15's Shapes against main** (ready check of hearth-15.1).
     hearth 15's ready check (check/result-15.md, hearth d0aeeaa, skein 841f487) found eight places where 15-files.md does not match main; 15-files.md now says each. (1) 15.1's Files gain protocol `src/limits.rs`, where `Limits` (`:4-12`) gains `template`, and the sketch's line names limits.rs, not lib.rs. (2) The errors' reverse arms are in builder `src/tests/translate/errors.rs` (`decode_back` `:5`, `decode_limit_back` `:50`), not `src/tests/translate.rs`: 15.1's Files gain it and the sketch says so. (3) 15.2's Files gain render `src/evaluate.rs` (`:278`), `src/evaluate/escape.rs` (`:65`) and `tests/render/src/expressions.rs` (`:329`), whose matches gain `Operand::Loop`'s and `View::Loop`'s arms. (4) records' `Arity` is `{ filter: Box<[u8]>, expected: u32, given: u32 }`, `expected` from render's `Filter::arity` (`evaluate/filter.rs:30`), so the message can say `"replace" takes 2 arguments`; `syntax` translates the filter with the landed `translate::operations::filter` (`operations.rs:58`), not a copy. (5) `template::place` (06's `template.rs:82`) returns `(u32, u32)`; 15.1 makes it a `Position { line, column }`. (6) Code takes `skein_lib::List` and `match`, never `map`, `collect`, `into_vec` or `and_then` (`clippy.toml:104-123`), and the records JSON arm names all 19 kinds, no `_`. (7) `call.rs:388`'s `contains` asks one blob for the paths of one content: 15.1 keeps a template's `Decoded` by oid, and a `Refused` one makes an error at each path holding it, each with its task, so that every task's targets fail. (8) 15.3's `Keep` meets protocol `lib.rs:209`'s assert (`mode.is_none() && child.is_none()`, one git request at a time), and 15.3's commit body says how. Nothing else in hearth 15 changes.

150. **hearth 42.1: a fresh diagnosis of the three-pipe hang, and its size** (process and plan defect, hearth 42.1's block of 2026-10-17T19:25:11Z).
     hearth 42.1's two blocks of 19:25:11Z (report hearth-42-20261017T192149Z.md). (1) The hang: yes. A fresh 42.1 implementer takes the prepared commit 2fb7b91, rebases it on origin/main, diagnoses why `a_stray_file_in_a_task_prints_a_repository_error` hangs, then completes ruling 147's hunk call, and lands only once the gate passes, that test included. The story is a required test of 42.1's Test shape (42-listing-members-rules.md:171); it stays as written: its 15 s timeout, its expectations, and the piped standard error of every git child ("Removes: git's standard error to `/dev/null`"). The first child with three pipes (cat-file's standard input, output and error) hangs, while ls-tree's two pass (the Proof, 0.017 s): the first check, while the story hangs, is `ls -l /proc/<pid>/fd` of the git child and of the test process, to see which pipe git's descriptors 0, 1 and 2 are and which process still holds a write end of git's standard input; the machine has no strace or gdb. Aim for 45 minutes of diagnosis; past them, or if the cause lies in skein (`skein-io`, its backend), stop with a question naming the file:line and the evidence, and change no skein file from this step. (2) The size: 42.1 lands at its count up to 1,100 lines, ruling 147 having moved the hunk call's failure into it after the plan sized it at 300; the README's other conditions hold (each counted file in 42.1's Files, each at most 800 lines, each test its Test shape names there), and its report's `Size:` gives the count by file and names this ruling. Past 1,100 lines, it stops on a question with the count by file. 42.2 and 42.3 keep their Sizes.

151. **hearth 33's Shapes against main** (ready check of hearth-33.1).
     hearth 33's ready check (check/result-33.md, hearth 674c5d6 to 3fd9fab, skein 841f487; re-read at hearth a54f617, where none of the files it names moved but protocol `lib.rs` and `store.rs`, whose lines below are a54f617's) found ten places where 33-store-entries.md does not match main; it now says each. (1) 33.1's Files gain store `start.rs` and `scan.rs`, whose matches of `Event` (`:9`, `:61`) gain `Arrived`'s arm. (2) 33.3's Files gain store `vocabulary.rs` (`Answer` drops `Copy`, `:25`) and `limits.rs`: `lib.rs:37`'s `Deadlines::with_capacity(1)`, `start.rs:15`'s `.expect("one retry")` and `limits.rs:24`'s `worst_case(1)` take two timers, `Retry` and `Said`. (3) A step may emit two outputs (`Strays` before `Ready`; the store's error before render's first answer): the store's `MAX_OUT` and the root's `MAX_OUT` and `MAX_OUT_RESUME` become 2, with their queues and the store harness's `collect`. (4) An unopened store stays as hearth 23 landed it (ruling 142, point 4): a `Startup` record on standard error, exit 75; 33.3 only fills its `path: Some(store)` (protocol `store.rs:175`), and drops its own `Unopened { why }`, `StoreFault::Owner`, the `Lock` and `Record` steps and the "recorded by uid" line; its Removes says so. (5) 33.3's Files gain service `src/tests/fixtures.rs:7` and protocol `src/tests/fixtures.rs:70` and `src/tests/store.rs:54-58`; `Configuration` drops `Copy` (service `lib.rs:21`). (6) 33.2's Files gain protocol `src/tests/fixtures.rs:288`, whose match gains `Asked::Beneath`. (7) `Load`'s `max` is `u32::try_from(tree::frame::largest(tree::frame::Kind::Resolution, &limits.tree))`, as `store.rs:58` does for `Kind::Store`. (8) The real loop's laid resolutions carry one tree `Line` in `sources` (tree `resolution.rs:316` refuses a `source` past them) and a tree `Package` fixture whose four fields are `fixtures::closure()`'s. (9) The two Rust sketches are as rustfmt wraps them at 120 columns; `ARRIVAL` is `Token::new(1)` and `MOTD` THIN's `775324fc…` as bytes. (10) Done when 3 names at least 46 tests: hearth 23's 21 under that filter, and these 25. Refresh any further copy of a moved line, and list it in the commit body.

152. **hearth 45's Shapes against main** (ready check of hearth-45.1).
     hearth 45's ready check (check/result-45.md, hearth e8227da, re-run at a606080; skein 841f487) found eight places where 45-diff-errors-resolve.md does not match main; it now says each. (1) `end_to_end_diff`'s `HOME` is `repository.scratch().path()`, as `crates/hearth/tests/end_to_end.rs:11` runs the binary: there is no `repository.home()`. (2) `PartChange::Meta` is 45.1's: hearth 25 landed `Config` alone (render `diff.rs:34`, records `diff.rs:43`). (3) 22.1's need is the collector and `Rendered`'s `left_out`; `Answer::Error` was landed before it (render `vocabulary.rs:29`). (4) 22.1 left one collector a side (render `call.rs:20`), the diff summing their counts (`call.rs:497`), so a diff could keep 512 errors: at `call.rs:513` `to`'s side takes over `from`'s collector, so the order, the 256 bound and the count run once over the answer, as 45's Choices say, and `from`'s errors, which `call.rs:499` answers at once, are answered after the last change. (5) `Order` (`errors.rs:99`) has no side and gains none: `from`'s errors arrive first, so `arrival` sorts `from` before `to`. (6) An error's `side` is `From` or `To`, never `Both`, which stays a failed target's. (7) 45.1's Files gain the literals of render's and records' `Error` (render `src/tests/errors.rs`, `call.rs:210`; `tests/render/tests/{errors, compose}.rs`; protocol `src/tests/translate/errors.rs`; records `src/tests/{errors, fixtures}.rs`), and the failed `Hunks`' path: render `vocabulary.rs:18` and `lib.rs:75-77`, protocol `lib.rs:557`, `Event::Failed`'s literals (the root's `src/lib.rs:101,155`, render `src/tests/diff.rs:201`, protocol `src/tests/{cache, git, read}.rs`) unless 42.1 has already given it the `Tool`, and `git.rs:36`'s standard-error pipe unless 42.1 has. (8) 45.2's Files gain the literals of `Event::Diff` and `Call::Diff` (the root's `src/lib.rs:137`, render `src/lib.rs:49` and `src/tests/fixtures.rs:383`). Refresh any further copy of a moved line, and list it in the commit body.

153. **hearth 42.1: a refused read's Failure** (plan defect, hearth 42.1's block of 2026-10-17T19:44:53Z).
     Yes: 42.1's `Failure` gains `Store`, with no fields, for a read the store refused, and the root's two refused-read `Failed` (`src/lib.rs:101` and `:155`) name it, `class` operational, `at` `None`. Render fails that side `TryAgain` and adds no error, as main does today for an operational `Failed`. This is not transitional, and the store's error vocabulary does not move into 42.1: 33's Choices say "A refused blob fails its read once: the root withholds the read's later `Cached` and gives render `Failed { class: Operational }` at its last blob", and 33.3's Shape says "The root keeps the store's `Full` or `Unusable` and answers one `Answer::Error` of the kind `store` ... before render's first answer", with the Don't "give the store's error per refused request". So 33.3 needs no change for it. 42.1's Files gain the root's `src/lib.rs`, and 42-listing-members-rules.md now says both (ruling 153). Ruling 150's size and gate conditions stand. Refresh any further `Event::Failed` literal (protocol `src/tests/cache.rs:44` among them) and list each one in the commit body.

154. **hearth 13.4's split modules under the size rule** (plan defect, hearth 13.4's block of 2026-10-17T20:04:21Z).
     Yes: hearth 13.4 lands as prepared, at 610 production additions against its Size of 240, under the README's rule "An increment past twice its Size": 610 is under three times its Size (720), each file is at most 800 lines, and its proofs pass. Records `src/errors/validation.rs` (110) and render `src/schema/targets.rs` (222) count as `src/errors.rs` and `src/schema.rs`, which 13.1's and 13.4's Files name: each takes code out of its parent so that the parent stays at most 800 lines, as 42.1's records `src/errors/repository.rs` did. The README's rule now says so: "A new child module that takes code out of a file in its Files, so that file stays at most 800 lines, counts as in its Files" (ruling 154). Its report's `Size:` gives the count by file and names this ruling. It lands through the gate, rebased on origin/main. Refresh any further copy of a moved line, and list it in the commit body.

155. **hearth-45.s1: meta's pairs concrete, no generic items** (steer of 2026-10-17T20:16:55Z).
     hearth 45.1 (e252de9) followed 45's sketch and added the run's first generic items in production code, which skein's programming-model.md, 1 ("No `async`, closures, trait objects, user-defined traits or generics") and 10.3 ("generic items", out of step code) rule out: render `src/parts/meta.rs:32` `Pair<T>` and `:50` `fn pair<T: Clone + Eq>`, records `src/diff.rs:56` `Pair<T>`, builder `src/translate/changes.rs:134` `fn optional_pair<T>` (`git grep -nE '<T\b' origin/main -- crates ':!*/tests/*'` at hearth c5d7e6b lists those four and nothing else). The plan's sketches wrote more: 17's `Pair<Sources>` and its siblings, 18's `Keyed<T>`, 19's `Keyed<Download>`, `Keyed<Box<[Consumer]>>`, `Keyed<SeedIds>`, `Keyed<u32>`; none of 17, 18 or 19 has landed. (1) A step not in the plan, `hearth-45.s1` (`hearth/45.s1-concrete-pairs.md`, needs 45.1, at most 90 lines), replaces the four with `NamePair`, `MemberPair` and `PlacementPair` in render and records, one helper per struct, and records' `form/meta.rs` pair of `Value<'a>` with a local `ValuePair<'a>`; no output changes. (2) 45's sketch names the three structs; 17's names one `<Type>Pair` per field; 18's and 19's one `<Type>Keyed` enum per type. (3) The README gains the rule "No generic item in production code": a sketch's `Pair<T>` or `Keyed<T>` is one concrete type per type it holds, and a helper over them one fn per type; a test's `only<T>` (render, builder and config `src/tests/fixtures.rs`) stays. The binary's closures (`crates/hearth/src/argv.rs`, `lib.rs`, `main.rs`) are the shell's, not step code, and were there before 45.1: unchanged. A sketch still naming a generic item follows the README's rule; refresh any further copy, and list it in the commit body.

## For the user

The long-term choices the user may veto:

- **Decision 2:** the builder's protocol layer, not a client crate, holds
  the client's routing, so that in M3 the same layer serves the jobs and
  the client hosts no crate of its own beyond the binary and the
  records' machine.
- **Decision 9:** five machine-world crates under `tests/`, beside
  `tests/render`, `tests/builder` and `tests/conformance`.
- **Decision 10:** no `cargo-fuzz` targets in M1.
- **Decision 13:** a crate from outside (`sha1`) in a test kit.
- **The designer's H169 and H174** (third round, hearth `31bed37`):
  resolution trusts the host's `debian-archive-keyring` (no
  `--keyring`; testing.md 6.2); the client's JSON layout, version 1, is
  fixed once landed (client/README.md 3.2).
- **Decision 19:** hearth's binary calls `libc::umask` in its one
  `unsafe` block (skein has no safe `umask`); the alternative is a skein
  ask, a safe `umask` in skein's shell, for a later plan.
