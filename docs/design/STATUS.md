# Status

Updated 2026-10-17. How far hearth's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|

No plan yet. The first takes the design to M1, render, alone
(README.md, section 3). The workspace is set up for it: an empty Cargo
workspace with the step crates' lints, its gate and the suites' budgets
(`docs/development/workflow.md`).

## 2. The design

| Document | Built | Left |
|---|---|---|
| `README.md` | no | the crates of 4.2, each in the milestone it names, and skein's part of each (section 8.1): not planned |
| `format.md` | no | all; M1 builds it but for checks (section 11, M5) and artifacts (10.2, M6): not planned |
| `render.md` | no | all, M1, with the client's `render` and `diff` (client/README.md, section 3): not planned |
| `store.md` | no | all, M2, but for peers (section 10, M6); M1 needs `hearth-lib`'s canonical encoding and `H` (3.1, 3.2), and a store without layers, its resolutions and cache files with their records, lock and scan (2.1, 2.4, 4.4, 6.1, 6.4 to 6.6, 6.8, 8.1, 8.2), in the first parts of `hearth-domain-store` and `hearth-protocol-tree`: not planned |
| `build.md` | no | all, M2, with the tree helper: not planned |
| `machine.md` | no | the EFI path and bootstrap, M3; the BIOS path, M3b; containers and pool hosts, M5: not planned |
| `slots.md` | no | all on EFI with a minimal bless and live changes, M3; trials through GRUB, M3b; containers and pool hosts, M5 (section 12): not planned |
| `activator.md` | no | the operations a slot needs, live changes included, M3; restarting a unit, data's snapshots and restores, a container's operations, M5; backups and disks as an effect, M6: not planned |
| `steward.md` | no | the steward and its builder, M3; from M1, the builder's domain hosted by the client (client/README.md, 3.5): not planned |
| `wire.md` | no | the faces and standalone mode (sections 2 and 4), M3; the wire (sections 5 and 9), M4; telemetry (section 7), M5; bytes by digest (section 8), M6: not planned |
| `client/README.md` | no | `render` and `diff`, in-process, M1 (3.1 to 3.3, 3.5), on built-in defaults; the rest of the client and its world, M3; `retarget` and `hearth dev`, M3b: not planned |
| `operating.md` | no | all, M5, local snapshots and restores included; backups and faults, M6: not planned |
| `testing.md` | no | each milestone's tiers, fakes and the fake machine's growth (section 10), from M1: not planned |
| `development.md` | no | mutable machines, the capture and applying a working tree, M3; the Incus driver, M3b; clusters and containers in development, M5; VMs from the host's store, M6: not planned |
| `artifacts.md` | no | all, M6: not planned |
| `reproducible.md` | no | all, after M6 (README.md, section 3, "later"): not planned |

## 3. Not planned

- **M1, render** (README.md, section 3): `format.md` but for checks and
  artifacts; `render.md`; the client's `render` and `diff`, with
  `hearth-client-protocol-records`'s printer (client/README.md);
  `hearth-lib`; the store without layers (store.md, 2.4);
  the builder's root, protocol layer and service; the fake machine's M1
  parts and the fakes of git and mmdebstrap (testing.md, section 10).
  Waits for its plan, the next.
- **skein's part of M1** (README.md, 8.1): a public, streaming SHA-256 in
  lib; positions on skein-json's tokens and errors; a digest of a world's
  state, for replay: skein's `lib.md`, 6.2 and 5.1, `json.md`, 3.6, and
  `testing.md`, section 5. For the store without layers: a lock taken
  without waiting, a rename between two directories, and a directory
  listed page by page: skein's `kernel.md`, 6.1, `io.md`, 5, and
  `simulator.md`, 3.1 and 3.3. Each lands in skein before its first
  hearth user, in M1's plan.
- **M2 to M6, and reproducible builds** (README.md, section 3): each
  waits for the milestone before it, and skein's part of it
  (README.md, 8.1).
- **The fake machine's generic half in skein** (testing.md, 3.11; README.md,
  8.2): waits for temper to build its machine.

## 4. Drafts

| Draft | About | Next |
|---|---|---|
| `draft/brainstorm.md` | the first discussion of hearth, which the design takes further | kept as its record; the design holds where the two differ (README.md, section 2) |
