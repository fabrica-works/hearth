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
