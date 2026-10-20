# hearth

*Immutable machines, built from a revision.*

A daemon on every machine that builds its system from infrastructure as
code, as layers keyed by digest. It boots them in slots that fall back
if the new one fails.

## The idea

- **Render is a pure function.** A revision and a target give a spec,
  part by part, each with its digest, in milliseconds. Any hearth renders
  any target, and the rendered diff of a change is what its reviewers
  read.
- **Pay only for what changed.** Outputs are layers, packages, then
  software, then configuration, each a read-only snapshot named by its
  cumulative key. A change rebuilds from the lowest layer it touched.
- **Nothing changes in place.** A machine's root is mounted read-only; a
  fix is a new slot, prepared beside the running one. A slot runs once
  on trial, and becomes the default only once it has proved itself;
  any failure boots back into the old one. Configuration can still go
  live without a reboot, and come back out if it fails.
- **Nothing unique and nothing secret in an output,** so outputs are
  shared between machines. Identity and secrets are written into a slot
  as it is prepared.
- **Privilege kept small.** The part that runs as root does a closed list
  of operations and listens to nothing; deciding, building and talking
  run unprivileged, in processes that can fail without taking it down.
- **Standalone first.** On its own, hearth converges a machine to a
  desired revision; behind an engine, it carries out orders and reports
  facts.
- **Built on skein, tested from day 0,** on a fake machine with a
  realistic fake of every tool it drives, held to the real tools by
  conformance tests.

## Start here

- [`docs/design/README.md`](docs/design/README.md): the overview; its
  section 2 is the reading order, its section 3 the milestones.
- [`docs/design/STATUS.md`](docs/design/STATUS.md): how far the code is
  from the design.
- skein's foundation, which hearth follows: `programming-model.md`,
  `testing-strategy.md` and `notes.md`.

Before changing code, read [`AGENTS.md`](AGENTS.md). `main` moves only
through skein's `land`, which runs the gate, `scripts/check.sh`.

## The family

These are the repositories of [Fabrig](https://github.com/fabrica-works), an
agent-driven software factory: write, prove, run.

| Repository | | Builds on |
|---|---|---|
| [skein](https://github.com/fabrica-works/skein) | One thread, one loop, no async | |
| [smith](https://github.com/fabrica-works/smith) | Protocol-driven agent loop for AI applications | skein |
| [jig](https://github.com/fabrica-works/jig) | Engine for agentic applications | skein, smith |
| [temper](https://github.com/fabrica-works/temper) | Agent-driven software development on a forge | skein, smith, jig |
| [hearth](https://github.com/fabrica-works/hearth) | Immutable machines, built from a revision | skein |
