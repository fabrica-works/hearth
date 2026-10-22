# Agent guidance for hearth

- **main moves only through skein's `land`,** which runs the gate,
  `scripts/check.sh`, and pushes to `origin`'s `main`
  (`docs/development/workflow.md`, section 1). Do not run the gate
  yourself before `land`. A change to Markdown files only lands with
  `land --docs`. Run what compiles and the tests of the crates you touch
  through `heavy`.
- The default test suite (unit tests and the worlds' focused tests under
  `tests/`) takes at most 30 seconds (15 seconds before the user's doubling
  of 10-22, provisional); the fuzzy suite (the worlds' `tests/fuzzy_*.rs`,
  randomized tests) at most 2 minutes (1 minute before the user's doubling
  of 10-22, provisional). Keep new tests within these budgets: see the same
  document. Every test runs as a regular user; conformance, real builds and
  the VM tier are outside the gate (`domain/testing.md`, 2.8).
- **hearth is the factory's build daemon** (temper's `factory.md`,
  section 7): it renders, builds and runs a machine's outputs. Read
  [docs/design/README.md](docs/design/README.md) first: its section 2 is
  the reading order, its section 3 the milestones.
  [docs/design/STATUS.md](docs/design/STATUS.md) says how far the code
  is from the design.
- **hearth builds on skein alone,** and follows skein's foundation
  documents: `programming-model.md`, `testing-strategy.md` and
  `notes.md`. Read them before writing code: the first two are rules,
  and a departure needs its reason, stated in the design or the commit.
  Citations use those file
  names and their sections. hearth depends on no crate of smith, jig or
  temper.
- **hearth's design is `docs/design/`.** Inside hearth, a document is named
  by its path from there (`domain/render.md, 8.3`); other projects'
  documents carry their project's name, never a path (skein's
  `json.md`, temper's `factory.md`): `README.md`, section 6. Code cites a
  section at module or type level, never stamped on every field.
- **Every crate of hearth is step code under the strict subset** of
  programming-model.md, section 10, except the fakes and the test kits
  under `testing/`, the worlds under `tests/`, and the binary's shell,
  which are ordinary Rust. The root `Cargo.toml` and `clippy.toml` hold
  the step crates' lints; a crate of ordinary Rust declares its own.
- **Step code depends on skein's crates and hearth's own, and nothing
  else:** no serde, and no template engine; hearth parses its own
  templates (`domain/render.md`, 8.3). MiniJinja 2 is a dev-dependency only,
  pinned to an exact version, as the template world's oracle
  (`domain/render.md`, 9.5).
- **skein is named by its GitHub URL,**
  `{ git = "https://github.com/fabrig-dev/skein.git" }`, in the root
  `Cargo.toml`'s `[workspace.dependencies]`, with no `rev`, `branch` or
  `tag`: `Cargo.lock` pins it.
- **Work on a branch in a worktree of your own,** never in the primary
  checkout. `land` is the only push; do not push or publish otherwise.