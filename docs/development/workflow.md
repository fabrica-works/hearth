# Development workflow

How work reaches main. Work happens on local branches; main moves only by
landing a branch that passed every check below, or that changes only
documentation.

## 1. Before merging to main

The gate is `scripts/check.sh`, run on the branch's tip rebased on main, so
that what is checked is what main becomes:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
NEXTEST_DOUBLE_SPAWN=0 cargo nextest run --workspace
NEXTEST_DOUBLE_SPAWN=0 cargo nextest run --workspace --profile fuzzy
```

skein's `land` (skein's `docs/foundation/development.md`, section 9) does
it all from a clean worktree on the branch: it rebases on `origin/main`,
runs the gate through the machine's queue, records its result in the tip
commit's trailers, and pushes the tip to `origin`'s `main`, which only
fast-forwards. The local `main` follows.

A branch that changes only Markdown files (the documents under `docs/`,
`AGENTS.md`) skips the gate, since nothing it touches is built or tested,
and lands with `land --docs`. Doc comments in `.rs` files are not
documentation in this sense: they are code, and clippy checks them.

Two things the gate does until the workspace grows:

- **With no crate,** cargo refuses every one of the four commands, so the
  gate says that there is nothing to check and passes. `land`'s build
  lane runs three of them before the gate, so a branch that lands
  something other than Markdown while no crate exists lands with
  `LAND_PREBUILD= land`. The first crate ends both.
- **Until the first world's fuzzy test lands** (`tests/*/tests/fuzzy_*.rs`),
  the fuzzy profile selects `none()` and the default profile every test:
  nextest refuses a filter that names a binary no crate has. The fuzzy
  run then passes with nextest's warning (`--no-tests=warn`). The landing
  that adds the first fuzzy test must also replace both profiles' filters
  with the two lines `.config/nextest.toml` gives in its comments; the
  gate fails a fuzzy test file while the fuzzy profile still selects
  `none()`. From then on a fuzzy run that selects nothing fails, as a
  default run always does: every crate lands with its tests (testing.md,
  section 1).

The gate needs Linux and git, nothing else: no root, no KVM, no Incus,
no network (testing.md, section 1).

## 2. The two test suites

The suites are those of skein's `docs/foundation/testing-strategy.md`
(section 8); `docs/design/testing.md` (sections 2 and 9) says what they
hold in hearth.

- **The default suite** is every crate's unit tests and each world's
  focused tests (`tests/<name>/tests/*.rs`): focused tests of expected
  behaviour (scenarios, referee tests, replay, facts changing nothing,
  memory against the worst case), and a cheap random world as a smoke
  test where one helps. It is what `cargo nextest run --workspace` runs,
  and it takes at most **15 seconds**.
- **The fuzzy suite** is the worlds' `tests/fuzzy_*.rs`: randomized
  tests, such as sweeps of random worlds and domains driven at random,
  over many seeds. It is not run by default, only with `--profile fuzzy`,
  as the gate before merging to main, and it takes at most **1 minute**.

Tests cost what they check:

- **Fixtures once.** Build a test repository, a package universe or a
  worst-case size once per test process or world when it is a fixture,
  then share it.
- **Checks on change.** Check what changed when it changes. Check the
  whole state at the end, unless waiting would lose the bug the check
  guards.
- **Traces on failure.** Keep typed observations and format them only on
  failure, or replay the failing seed with tracing enabled.
- **Test-sized limits.** Use limits as small as the assertions allow, so
  preallocation is proportionate.
- **Long runs are several tests.** Aim for each focused test to take at
  most about 0.5 s, and each fuzzy test about 5 s, unoptimized on the
  idle development machine. Split longer sweeps so the gate can share
  them across threads, keeping their assertions and coverage. The nextest
  slow periods flag these times; they do not fail a test.

The budgets are enforced, not advisory: `.config/nextest.toml` gives each
profile a `global-timeout`, so a suite that runs past its budget fails,
naming the tests it stopped, and a slow period past which a test is
flagged SLOW as it runs. A change that breaks a budget is fixed by making
tests cheaper, or by moving randomized ones to a world's fuzzy tests.
Raising a budget is a decision to take explicitly, not a fix. To see what
each test costs on its own, the `measure` profile has no budget:

```sh
NEXTEST_DOUBLE_SPAWN=0 cargo nextest run --workspace --profile measure -j 1
```

The budgets are wall time on the development machine (4 cores, 8 threads,
which nextest uses all of), with nothing else building. Run the checks on
an idle machine.

**Outside the gate** are hearth's own tiers: conformance, real builds and
the VM tier (testing.md, 2.8), each a profile of its own with a ceiling
(testing.md, 9.2). Each profile comes with its tier's first test, and
its binaries join the default and fuzzy profiles' exclusions in the same
landing.

## 3. When a fuzzy seed fails

A failure names its seed, and a seed replays to the same run. Fix the
bug, then keep its seed: as a scenario among its world's focused tests if
it shows behaviour worth naming, or among the sweep's pinned seeds. A
finding that cannot be fixed yet joins its world's `FINDINGS`, which an
ignored test replays until it is fixed.
