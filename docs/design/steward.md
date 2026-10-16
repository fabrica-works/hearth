# The steward and its builder

Provisional, 2026-10-14. hearth's unprivileged side, as two processes:
the steward, which runs for as long as the machine does, and the
builder, which the steward spawns for work and which exits once the work
is done. The activator above them is activator.md's. M3 builds both;
from M1 the client hosts the builder's domain in its own process
(client/README.md, 3.5). What the steward does is told where it belongs: the pass
and the faces in wire.md, checks and telemetry in operating.md. This
document holds the split itself: who does what, the jobs between them,
how a failed builder is handled, and the bless's render. What is open is
in section 10.

## 1. In one page

- **Split by failure, not by privilege.** The activator is root and
  closed (activator.md). Beneath it, user `hearth` runs two processes:
  the steward, persistent, and the builder, spawned for work. They share
  a uid, so the split is no boundary between them: it keeps a failure
  in building from taking down what watches the machine and answers for
  it.
- **The steward decides and faces.** It holds converge (the desired
  revision, the pass, refusals, holds, the bless), the client's socket,
  the wire to an engine [M4], facts and events, checks and telemetry
  [M5], and backup streams [M6]. It is the only process that talks to
  the activator.
- **The builder works.** It runs git, render, the resolution, the
  fetch, build steps, the store and its collection, live plans, and
  later artifacts and change sets for peers [M6]. It alone writes the
  store's layers, and it holds the store's lock while it runs.
- **The rule that keeps the steward small:** it reads no repository,
  runs no build tool and opens no layer. It decodes typed records from
  its builder, its activator, its clients and its engine, and the
  machine's own signals.
- **Jobs over a pipe pair** (section 3): the steward writes jobs to the
  builder's standard input and reads answers from its standard output,
  in sized records like the link's. One builder at a time, and the
  machine's own jobs one at a time (3.4).
- **The builder lives for its work** (section 4). The steward spawns
  one when it has a job and none runs, and closes its pipe once it has
  been idle for a while; the builder then exits. An idle machine runs
  the activator and the steward.
- **A crash is a failed job** (section 5). A builder that dies with a
  job in flight loses that job; a new builder tries it once more, and a
  second loss fails it as `crashed`, which the pass treats as its
  stage's own failure: render waits for another commit, a build is
  refused. The steward keeps running, answering and reporting why.
- **The bless proves both halves** (section 6). Before the new slot's
  steward asks to bless, one of its builders renders the slot's own
  revision offline, so a hearth release that cannot render what built
  it never becomes the default.

## 2. The processes

```
a machine
└── hearth activator          root, started by systemd                                activator.md
    └── hearth steward        user hearth, group systemd-journal; persistent; started by the activator
        └── hearth builder    user hearth, no supplementary group; for work; started by the steward
            ├── tools         git, apt-helper, mmdebstrap, dpkg-query: each build step in a sandbox of its own
            └── tree helper   in the builder's user namespace (store.md, section 5)
```

### 2.1 Who does what

| Concern | Steward | Builder | Defined in |
|---|---|---|---|
| the desired revision, the pass, refusals, the hold | ✓ | | wire.md, section 4 |
| requests to the activator, and its facts | ✓ | | activator.md, sections 4 and 5 |
| the client's socket | ✓ | | client/README.md, 3.4 |
| the wire, orders, facts and events [M4] | ✓ | | wire.md, sections 5 and 6 |
| checks, signals, the journal, telemetry on demand [M5] | ✓ | | operating.md, sections 2 and 5 |
| backup streams, between the activator and a peer [M6] | ✓ | | operating.md, 7.2 |
| the HTTP listener for peers [M6] | ✓ | a change set's bytes, as a job | wire.md, section 8 |
| `git fetch`, the clone, bundles | | ✓ | render.md, 2.2 and 2.4 |
| render, the resolution, the rendered diff | | ✓ | render.md |
| the fetch, steps, layers, records, collection | | ✓ | build.md, store.md |
| live plans: the eligibility rules and the units to act on | | ✓ | slots.md, 6.7 |
| artifacts [M6]; receiving a peer's change set [M6] | | ✓ | artifacts.md; store.md, 10.2 |

### 2.2 The rule

- **The steward reads no repository, runs no build tool and opens no
  layer.** What the repository steers, and whatever a build produced,
  can be made to crash a parser or exhaust memory; in the builder, that
  ends one job.
- **What it decodes:** link records from its activator; job answers
  from its builder; the client's records; the engine's channel [M4];
  and the machine's own signals: `/proc`, the journal, systemd's units
  through `hearth-protocol-systemd`, checks' results, and the answers of
  probes, each bounded, with names resolved by `getent`: the one tool
  it runs, on a name a check carries, never a build tool
  (operating.md, 2.5) [M5].
- **What it writes:** its records in the store's `state/` (store.md,
  8.3), and nothing else of the store.
- **New work joins the side the rule says.** A responsibility that must
  read the repository or a layer, or run a tool on IaC's inputs, is a
  job; one that watches, decides or answers is the steward's. The
  steward's children in the crate graph are siblings under a root that
  only routes (README.md, 4.2), which is where its growth through M4 to
  M6 is kept in hand, not in more processes.

### 2.3 Both are `hearth`

- **One uid, two sets of groups.** The steward runs with `hearth`'s
  supplementary groups, `systemd-journal` among them, to read the
  journal (operating.md, section 5); the builder runs with none, since
  nothing it does reads the journal.
- **No boundary between them.** The builder can write the steward's
  state and signal it, so a compromised builder is a compromised
  steward, and both are root at the next trial (activator.md, 2.6).
  What the split gives is failure isolation: a bug, a hostile input or
  a runaway build ends a job, not the process that reports it.

## 3. The jobs

### 3.1 Spawning the builder

| | |
|---|---|
| program | `/proc/self/exe`, argument `builder`: the steward's own binary, which is its activator's |
| cgroup | `steward/builder/` (section 7), through skein's contained process trees (README.md, 8.1) |
| user | `hearth`'s uid and gid, no supplementary groups, no capabilities |
| environment | given whole: `LANG=C.UTF-8`, and the store's path |
| descriptors | stdin, the jobs; stdout, the answers; stderr, the steward's own, so the builder's traces reach the journal as the steward's do |
| working directory | `/var/lib/hearth/store` |
| process group | its own, so the steward can signal it and its children |

A builder never outlives its steward: the steward's shutdown stops it
(4.3), and the activator empties the steward's cgroup before it spawns a
new steward (activator.md, 3.3).

### 3.2 Records

Sized, as the link's are (activator.md, 4.1): a 12-byte header (magic
`HJOB`, version, kind, body length), the length checked against the
kind's limit before the body is read.

| Record | Way | Holds | Limit |
|---|---|---|---|
| `Hello` | builder to steward | version | 64 B |
| `Welcome` | steward to builder | version, the pins (store.md, 7.1) | 64 KiB |
| `Job` | steward to builder | job id (u64), the desired revision's generation, a job (3.3) | 64 KiB |
| `Progress` | builder to steward | job id, stage, done and total | 256 B |
| `Answer` | builder to steward | job id; done with its result, or failed with its typed error and class (wire.md, 4.4) | 4 MiB |
| `Pins` | steward to builder | the pins, whenever they change | 64 KiB |
| `Cancel` | steward to builder | job id | 64 B |
| `Bye` | either | reason | 64 B |

- **`hearth-protocol-jobs`** encodes and decodes them. It is the only
  vocabulary the steward's domain and the builder's share
  (README.md, 4.2), as the link is the steward's and the activator's.
- **Versions match by construction:** the builder is the steward's own
  binary. A mismatch or a framing error ends the builder, and its job is
  lost (5.1).
- **Pins come with the `Welcome`:** the outputs of the activator's
  slots and the desired revision's outputs, which only the steward
  knows. A builder collects nothing before it has them (store.md, 7.1).

### 3.3 The jobs

| Job | Holds | Answer | Asked by | Milestone |
|---|---|---|---|---|
| fetch | the repository's URL; a branch, a commit, or a bundle's file in `/run/hearth/` | the commit | the pass, `fetching` (wire.md, 4.2); `revision set --bundle` | M3 |
| render | the target, the commit; online or offline | the spec's part digests, the output key, the slot parts as the link carries them, the live part, the trial window's inputs [M5] | the pass, `rendering`; the bless (section 6) | M3 |
| build | the target, the commit, the output key render answered | the output: key, tree digest, kernel digests, costs | the pass, `building` | M3 |
| plan live | the running slot's output and the new one, with both specs' live parts | the plan, or the rules it breaks (slots.md, 6.7) | the pass, `trying`; after a prepare, for the `activation` fact and `status` (wire.md, section 6) | M3 |
| verify | a layer the activator found not to match its record | intact, or deleted with its descendants (store.md, 4.4) | a failed place | M3 |
| collect | — | what was deleted, and why (store.md, 7.2) | after a pass; the store's free-space floor | M3 |
| render, diff or build for a client | as the command names them | the command's records | the client (client/README.md, 3.1) | M3 |
| credential | the repository's read credential | stored under the store's `credentials/` (render.md, 2.2) | the client's `credential` (client/README.md, 3.1) | M3 |
| a change set's or a blob's bytes; a peer's change set received | a layer key or a digest; a peer | a stream, or the layer | a peer (wire.md, section 8); a prepare order's sources | M6 |
| an artifact | artifacts.md, section 4 | the artifact's record | an order | M6 |

- **A `build` names the output key its render answered.** A builder
  that no longer holds that spec, a new one after a loss, renders again,
  which is deterministic and reuses the resolution by key; a key that
  differs fails the job as a bug, never as a configuration error.
- **Every job carries the desired revision's generation.** A pass whose
  generation is no longer desired stops at its next stage
  (wire.md, 4.1), which is a job's end: no job is cut in the middle for
  it.
- **A job is cut only by `Cancel`,** when the order behind it is
  withdrawn [M6] (artifacts.md, 5.1): the builder stops it at its next
  step boundary, discards its staging and answers `cancelled`. The pass
  never cancels; it stops at a job's end.
- **Effects stay the steward's.** No job asks the activator for
  anything: the builder answers the steward, which decides and asks.
- **The builder reads nothing of the steward's:** never `state/`, so
  whatever a job needs, the repository's URL included, it carries.

### 3.4 One builder, two lanes

- **One builder per steward.** Jobs wait in the steward's queue, in
  memory, in two lanes:
  - **the machine's own,** one job in flight: the pass's next stage,
    then a client's job, then collection;
  - **others' work [M6],** artifacts and peers' change sets, as many at
    once as the builder's limits allow (build.md, 4.4; artifacts.md,
    5.4), so a long artifact never holds up the machine's own pass.
- **The store's lock:** the builder takes an exclusive `flock` on the
  store's root directory when it starts, and holds it until it exits; a
  directory no rename replaces. A second builder over the same store, a
  daemon's or a client's in-process on a developer's store, waits for it
  (store.md, 8.2).
- **Bounds:** at most 16 jobs queued per lane; a client's job beyond it
  is refused as busy (exit 75).

## 4. Its life

### 4.1 Spawned for work

- **When a job is queued and no builder runs,** the steward spawns one
  and gives it the job after the `Welcome`.
- **When it has been idle for the linger** (30 seconds to start), the
  steward sends `Bye` and closes its pipe. The builder finishes nothing
  more, since it holds no job, and exits.
- **So a pass is one builder,** its jobs back to back within the
  linger: fetch, render, build, and collection after it. An idle machine
  runs the activator and the steward, the builder's memory and the
  store's index included in neither.

### 4.2 Every start a recovery

- **The builder's start is the store's startup scan** (store.md, 8.2):
  what an earlier builder left half-made, after a crash, a kill or a
  power cut, is settled before the first job. The recovery path runs at
  every pass, not only after a failure.
- **What it costs:** the scan's ioctls and records, bounded by the
  store's 4,096 layers, and the index built from them: to measure
  (build.md, section 12). The linger keeps back-to-back jobs from paying
  for it twice.

### 4.3 Shutdown

The steward's `Shutdown` (activator.md, 3.4):

1. **new jobs are refused,** and the queue dropped;
2. **the builder is sent `Bye`,** stops its job at the next step
   boundary, as at any stop (build.md, 8.7), and exits;
3. **its process group is sent `Terminate`,** then `Kill` after 20
   seconds, within the activator's 30;
4. **the steward closes the link** and exits.

## 5. Failures

### 5.1 A builder that dies

- **A builder that exits with a job in flight loses it:** an exit, a
  signal, the kernel's OOM killer, a framing error on the pipes. The
  loss is a fact and an event, with the job, its last stage from
  `Progress`, and the exit status or signal.
- **A lost job is tried once more,** at once, by a new builder: the OOM
  killer and the machine's other pressures end a builder through no
  fault of its input.
- **A loss counts only when the job was alone in flight.** Jobs lost
  beside others [M6] are tried again one at a time, alone, uncounted,
  while the steward holds the rest back; a loss is then the job's own.
- **Lost twice alone, it fails as `crashed`.** The count is per job, in
  the steward's memory: a steward that restarts starts it again, at the
  cost of two more builders.
- **What `crashed` means to the pass** (wire.md, 4.4):

  | Job | Class | What follows |
  |---|---|---|
  | fetch | operational | back off |
  | render | configuration | wait for another commit: render is deterministic, so the same input crashes again |
  | build | refused | refuse the prepare key, which render's answer gave |
  | plan live | none | an ordinary trial, the reason kept as a fact |
  | verify, collect | operational | back off |
  | the bless's render (section 6) | — | ask the activator to fall back |
  | a client's job | — | the client exits 69 |

- **This is what keeps a healthy machine in view.** A hearth bug that
  panics on one revision ends that revision's jobs. The steward still
  holds the client's socket and the engine's channel, still checks and
  samples, and reports the crash with the revision that caused it.

### 5.2 A builder that hangs

- **Each job has a deadline in the steward:** fetch 10 minutes; render
  15, the resolution's own 10 (render.md, 5.5) and a margin; build, the
  sum of its steps' wall limits (build.md, 8.7) and 10 minutes; plan
  live, verify and collect 15; an artifact [M6], its recipe's limit and
  a margin (artifacts.md, 5.4); all starting values.
- **Past it,** the steward stops the builder's cgroup in skein's three
  steps, and the job is lost (5.1).

### 5.3 Memory

- **`steward/builder/` has a ceiling of its own** (`memory.max`, 6 GiB
  to start: the packages step's 4 GiB, the tree helper's and the
  builder's own), so a subtree that reaches it is killed within itself:
  the kernel's OOM killer picks from the builder's subtree, never the
  steward.
- **A step killed for memory is that step's failure** (build.md, 8.7);
  the builder itself killed for memory is a lost job.
- **The steward's own memory, idle and at its peak,** is a cost fact
  (build.md, section 12): its rings are fixed in size, its records
  bounded, and a steward that grows is a finding.

### 5.4 A steward that dies

- **The activator respawns it** after its backoff (activator.md, 3.3),
  once the steward's cgroup is empty: a builder still running when its
  steward died is killed with the subtree (`cgroup.kill`), and its
  staging is settled at the next builder's start.
- **The rule makes this rare:** the steward decodes only bounded, typed
  records and the machine's own signals (2.2).

## 6. The bless's render

- **The minimal criterion asks one thing more** (slots.md, 7.1): before
  the new slot's steward asks to bless, a builder it spawned has
  rendered the slot's own target at the revision the slot was prepared
  from.
- **Offline:** from the clone and the store only. No fetch, and no
  resolution looked up again: the store holds the slot's, since a
  builder of this machine made its output. What render would have to
  fetch, it does not, and its answer stops there, which still counts.
- **The revision** is the commit of the pass that prepared the slot,
  which the steward records in its state (store.md, 8.3); under an
  engine [M4], the prepare order's.
- **Answered with a spec:** the condition holds, whatever the output
  key. A key that differs from the slot's output is a fact: the new
  release renders the same revision to another output, and the next
  pass builds it.
- **Answered with an error, or lost twice:** the steward asks the
  activator to fall back. The new release cannot render the revision
  that built its slot; the old one could.
- **So a release renders what built it.** A format change that drops
  something spans two releases, the first accepting both.
- **What it proves:** the builder spawns, takes the store's lock, scans
  the store, reads the clone and renders. It does not prove that a build
  works: a build may need the network and minutes, and building from
  scratch is the lab's.
- **Where it runs:** at every native trial, and in the base's steward
  respawned after a live swap (slots.md, 7.6). Not for a re-bless, whose
  slot's hearth was proven when it was first blessed (slots.md, 7.5);
  nor for bootstrap's first slot, blessed with no trial by a release
  whose builder has just rendered and built it (machine.md, 8.2); nor
  for a container's trial [M5], whose host's hearth did not change.
- **Its cost:** the builder's start and a render of one target from
  local data, within seconds of the trial's minutes of window.

## 7. cgroups

```
hearth-activator.service      Delegate=yes; memory and pids enabled
├── activator/                the activator, a leaf
└── steward/                  owned by hearth; memory and pids enabled for its children
    ├── self/                 the steward, a leaf
    └── builder/              the builder's subtree: memory.max 6 GiB (5.3)
        ├── self/             the builder, a leaf
        └── <token>/          a sandbox, a fetch or a tree helper, one leaf each (build.md, 8.7)
```

- **The activator** makes `steward/`, gives it to `hearth`, and enables
  the controllers in its unit's cgroup (activator.md, 3.1).
- **The steward** moves itself into `steward/self/`, makes `builder/`,
  enables `memory` and `pids` in `steward/`, and spawns each builder
  into `builder/`: the no-internal-processes rule lets no cgroup with
  controllers enabled for its children hold processes itself.
- **The builder** moves itself into `builder/self/` and gives each
  sandbox, fetch and tree helper a leaf beside it, as build.md, 8.7
  says.
- **On a developer's machine** the client hosts the builder's domain
  in-process under `systemd-run --user --scope -p Delegate=yes`, with no
  steward (build.md, 8.7).

## 8. Its world

- **The steward's root world:** `hearth-domain-steward` with converge
  (and checks from M5) on the fake machine, with a scripted builder on
  the jobs' side and a scripted activator on the link: builders that
  die at every stage, hang, answer late or for a stale generation, and
  a builder killed for memory mid-step (testing.md, sections 3 and 8).
- **Invariants,** checked after every event:
  - every job ends in exactly one answer or one loss;
  - a job lost twice fails as `crashed`, and the pass follows 5.1's
    table;
  - the steward opens nothing under `layers/` or `repo/` and spawns no
    tool of build.md, section 10: the fake machine records every open
    and every spawn with the process that made it;
  - while a builder fails, the steward keeps answering the client and,
    from M4, keeps its channel;
  - no bless is asked before the bless's render has answered.
- **The system world** runs the steward, the builder with its tree
  helper, and the activator together, from a desired revision to a
  blessed slot, crashing any of them at any point (testing.md,
  section 8).
- **The VM tier** kills a builder for memory during a packages step,
  and a builder in the middle of a pass, on a real machine; the steward
  reports both and keeps serving (testing.md, section 7).

## 9. What changes from os

- **os ran one root process per command:** `deploy` rendered, built and
  activated in it, so a failure in rendering and one in activation were
  the same process's. hearth's three processes fail apart: a render or
  a build ends a job, the steward reports it, and the activator is
  reached only through typed requests.

## 10. What is open

- **The linger, the deadlines and the builder's ceiling,** to set from
  measurements of the builder's start, its passes and its peak memory
  (build.md, section 12).
- **A crash's tail:** whether the steward keeps the last lines of the
  builder's standard error with the loss, through a pipe of its own,
  rather than leaving them in the journal only.
- **The bless building something:** whether the bless's render should
  also build a small layer offline, to prove the sandboxes and the tree
  helper, and at what cost to the trial's window.
