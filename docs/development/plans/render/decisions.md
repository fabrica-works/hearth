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
