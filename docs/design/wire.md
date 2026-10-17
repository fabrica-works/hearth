# The faces and the wire

Provisional, 2026-10-09, revised 2026-10-16. hearth's faces:
standalone mode, the wire to an engine, the facts and events hearth
reports, telemetry read on demand, and bytes fetched by digest. The
command-line client, which M1 starts with `render` and `diff`, is
`client/README.md`. M3 builds the steward's face to the client and
standalone mode, the MVP's only faces (sections 2 and 4); M4 builds the
wire (sections 5 and 9); M5 telemetry (section 7); M6 bytes by digest
(section 8). The catalogue of section 6 says which milestone first
reports each fact and event. What is open is in section 11.

## 1. In one page

- **Three faces, one domain.** The daemon (the activator, its steward,
  and the builder the steward spawns for work; steward.md) serves a
  local client and, from M4, an engine. The steward is the face. The
  client is the same binary, run by any user: it talks to the steward
  over a local Unix socket, or runs the builder's domain in its own
  process when no daemon runs. Root-only commands run in-process as
  root.
- **The client prints typed records and ends with an exit code**
  (`client/README.md`, 3.2 and 3.3): a table for people or JSON lines
  for scripts.
- **Every effect is keyed or conditional, even from the client.** A
  prepare is keyed by target and revision. A trial and a roll back carry
  the facts the client decided from, and the activator refuses them if
  those facts have moved (`client/README.md`, 3.1; 5.4).
- **Standalone mode converges to a desired revision set locally**
  (section 4): a commit, or a branch polled on an interval, kept in the
  steward's state. A pass fetches, renders, compares, builds, prepares
  and tries, one target at a time, the fetch, the render and the build
  as a builder's jobs. The new slot's hearth blesses it once a builder
  of its own has rendered the slot's revision. A slot that fell back is
  never tried again on its own, and a revision that crashes a builder
  twice is reported and waited out, never retried in a loop. A local
  hold stops new passes.
- **The wire is skein's channel, dialled out by hearth** (section 5).
  The hello comes first and carries the facts: slots, the four facts,
  versions, costs, and the answers the engine has yet to acknowledge. A
  reboot is a redial, and the hello after it is how the engine learns
  whether an activation held or fell back; a live activation respawns
  the steward, which redials too.
- **Orders follow jig's recovery classes** (5.4). prepare is keyed;
  activate, bless and roll back are conditional on the facts decided
  from; restart a unit is keyed by the order's key, which hearth keeps.
  hearth keeps every order's outcome until the engine acknowledges it.
- **Facts are state, events are hints** (section 6). One typed catalogue
  says what each carries and which milestone first reports it.
- **Agents only read.** Facts, events and telemetry are reads. The
  engine's orders are the only writes, and the engine's policy grants
  them. `shell` is a local, read-only command, never on the wire.
- **Generic at the wire, Debian inside** (section 9). An engine sees
  targets, revisions, slots, keys, parts by name and digest, orders,
  checks and units; never a package, a layer or a subvolume. Another
  backend is another daemon speaking the same wire.
- **Its worlds** run standalone mode and the wire on the fake machine,
  against a fake engine (section 10); the client's is
  `client/README.md`, section 4.

## 2. Faces

```
                                   engine [M4]: the run side's or the lab's
                                      ▲ skein's channel over TLS, dialled out
a machine                             │
├── hearth activator   root  ◄─link─► hearth steward   user hearth, persistent
│                                       ▲      └─jobs─► hearth builder   user hearth, spawned for work
│                                       │ skein's channel over /run/hearth/client.sock
├── hearth (the client) any user ───────┘
│                       with no daemon: the builder's domain, in-process
└── hearth (as root)    root-only commands: the activator's code, in-process
```

- **The daemon** is activator.md's activator, started by systemd, and
  steward.md's two processes: the steward it starts, and the builder
  the steward spawns for work. The steward is the face. It serves the
  client's socket and, from M4, dials the engine;
  `hearth-domain-converge`, its child, holds the desired revision and
  the orders, whichever face they came from. Whatever reads the
  repository or builds is a builder's job (steward.md, 3.3).
- **The client** is the `hearth` binary's command-line face. It holds
  no state. It sends one request, prints the records that answer it,
  and exits.
- **In-process** is the client hosting a domain in its own loop: the
  builder's, on a developer's machine, in CI, wherever no daemon runs,
  and for any local repository (`client/README.md`, 3.5); and the
  activator's code, as root, for the root-only commands
  (`client/README.md`, 3.6).
- **The activator has no face.** It hears only its steward, over the
  link (activator.md, section 3). The client never reaches it. A
  root-only command runs the activator's own code in its own process.
- **One vocabulary behind every face.** A command from the client and an
  order from the engine reach `hearth-domain-converge` as the same
  domain records. The client's orders are keyed or conditional like an
  engine's (README.md, section 3), so the MVP already proves what the
  wire will rely on.

## 3. The client

The command-line client is `client/README.md`, whose section 3 keeps
this section's numbers: commands (3.1), output (3.2), exit codes (3.3),
the socket and peer credentials (3.4), in-process (3.5), root-only
commands (3.6), `shell` (3.7) and bootstrap's arguments (3.8). This
document keeps the faces the client reaches (section 2), standalone
mode and the wire.

## 4. Standalone mode

### 4.1 The desired revision

- **What a machine converges to, set locally.** Either a commit, or a
  branch to follow, polled every interval. It lives in the steward's
  state (store.md, 8.3) and survives restarts and reboots:

  ```
  desired
    target       x1-prod                      the activator's machine record, in its facts
    repository   the IaC repository's URL     written at bootstrap (client/README.md, 3.8)
    revision     commit 3f1c2ab0…  |  branch main, every 10 min  |  commit c07a11e9…, from a bundle
    trial        auto | manual
    containers   all  |  mail-dev, chat-dev  |  none    [M5] a development machine's selection (4.7)
    set by       uid 1000, 2 days ago
    generation   7                            bumped at every change
  ```

- **Set by the client:** `hearth revision set <commit>`, or `hearth
  revision follow <branch> --every <d>`. The interval is at least a
  minute, so a fleet of followers does not hammer the forge.
- **A commit from a bundle** (development.md, 6.2): `hearth revision set
  <commit> --bundle <file>` has a builder fetch a git bundle into its
  clone as `refs/hearth-dev/<commit>`, a fetch job (steward.md, 3.3),
  then sets the commit. The pass finds it in the clone and never fetches
  it from the repository's URL; render resolves it from that ref. The
  builder keeps the last 16 such refs, never the desired one's. This is
  how a working tree, committed on no branch, becomes a machine's
  desired revision.
- **`trial: manual`** stops each pass at a prepared slot and waits for
  `hearth trial`, live changes included. A workstation such as
  `x1-prod`, the MVP's machine, reboots, or changes under its user's
  feet, when its user chooses, not when a commit lands. `auto` tries at
  once, live when eligible, and suits servers.
- **The target is the activator's,** in its machine record
  (activator.md, 7.1), and the repository the steward's, whose
  builders keep its clone; both are written at bootstrap (client/README.md, 3.8). The
  client changes the revision, never which target the machine is.
- **The steward's state,** a record each in the store's `state/`,
  which only the steward writes, read back at startup (store.md, 8.3;
  steward.md, 2.2):
  - the desired revision, above;
  - the mode, the operators' uids and, under an engine, its address,
    its trust and the machine's credential [M4] (client/README.md, 3.8);
  - the hold (4.5) and the refused prepare keys (4.4);
  - the last pass: its generation, commit, stage and outcome, and the
    output and prepare keys it made; the bless's render reads its
    commit (steward.md, section 6);
  - orders with their outcomes, until acknowledged [M4] (5.5);
  - the rings of signals and check history [M5]
    (operating.md, section 5).

  The repository's read credential, if any, is the builder's, in the
  store's `credentials/` beside `state/` (client/README.md, 3.8; render.md, 2.2).
- **Every pass names the generation it serves.** A pass whose
  generation is no longer the desired one stops at its next stage, and a
  new pass starts.

### 4.2 A pass

The convergence loop is a procedure of `hearth-domain-converge`:
level-triggered, bounded, and stepped by events (jig's `connectors.md`,
section 6, is the model). It compares what the desired revision makes
with what runs, and does the next thing that closes the gap. Its
stages that read the repository or build, `fetching`, `rendering` and
`building`, and the live plan in `trying`, are a builder's jobs
(steward.md, 3.3); the steward compares, asks the activator, and
blesses.

```
stage        on                                          to           does
waiting      the interval elapsed (a branch), the        fetching     a fetch job: the branch, or the
             desired revision changed, the backoff                    commit if the clone lacks it
             elapsed, startup
fetching     the commit C                                rendering    a render job: the target at C
                                                                      (render.md)
             git failed, or the job crashed (steward.md, waiting      back off, or wait (4.4)
             5.1)
rendering    the spec S                                  comparing    the output key: resolution reused
                                                                      by its key, or resolved
             a configuration error (render.md, 7.1), or  waiting      wait for another commit
             the job crashed
             an operational failure (render.md, 7.1)     waiting      back off
comparing    the running slot is blessed but is not      re-blessing  ask the activator to bless it again
             the default: a cut bless, or the firmware                (slots.md, 2.3 and section 7)
             fell through to it
             S equals what runs: the running slot, or    waiting      converged
             the live state's prepare key
             (slots.md, section 3)
             S's prepare key is refused (4.4)            waiting      reported; wait for another key
             S equals the prepared, untried slot         trying
             otherwise                                   building     a build job: K, from S
re-blessing  blessed: the default is the running slot    comparing
             refused                                     waiting      back off
building     the output, keyed K                         preparing    ask the activator to place K with S's
                                                                      slot parts and trial window
             a failure, the job crashed included         waiting      back off, refuse, or wait (4.4)
preparing    prepared                                    trying       or waiting, if trials are manual
             refused or failed                           waiting      back off, refuse, wait or rebuild (4.4)
trying       a plan live job: eligible (slots.md, 6.7),  (respawn)    ask the activator to try live with the
             how auto or live                                         plan: delta, swap, units (slots.md, 6.8);
                                                                      a new steward is spawned over them
             not eligible, or live refused or failed     trying       ask an ordinary trial, the plan's
             before the swap                                          reasons kept as a fact
             the activator took the trial                (reboot)     a reboot or a soft reboot into the
                                                                      slot (slots.md, section 6)

over the live state, after the respawn
blessing     settled, the minimal bless holds            waiting      ask the activator to bless live: the
             (slots.md, 6.8, L7), the bless's render                  one-shot set to the slot's trial entry
             answered (steward.md, section 6)
             a touched unit failed, the criterion        waiting      ask the live fall back (slots.md, 8.5);
             failed, the bless's render failed or                     refuse b's prepare key
             crashed, or the window nearly over
             the activator fell back live at L6          waiting      refuse b's prepare key

in the new slot, after the reboot
blessing     the minimal bless holds (slots.md, 7.1),    waiting      ask the activator to bless
             the bless's render answered (steward.md,
             section 6)
             a unit failed that the target's             (reboot)     ask the activator to fall back
             `bless.ignore_failed` does not list
             the bless's render failed, or crashed       (reboot)     ask the activator to fall back
             neither, within the trial's window          (reboot)     the dead-man reboots into the default

in the old slot, after a fall back
waiting      the activator reports "fell back" from b    waiting      refuse b's prepare key
```

- **"S equals a slot"** means the same prepare key (slots.md, 2.2):
  the same output key, the same digests of the slot parts (`data`,
  `secrets`, `boot`; render.md, 4.3), the identity's and the secret
  store's current generations (4.6), and, from M5, the same trial window
  (operating.md, section 3). converge computes it as the activator does,
  from S and the generations in the activator's facts, and compares it
  with the slot records there (slots.md, 2.3). A change to the kernel's
  command line alone changes no output and still needs a new slot.
- **Re-blessing comes first.** While the running slot is not the
  default, prepare is refused (slots.md, 2.3), so the pass first asks
  the activator to bless the running slot again, which moves the
  default to it (slots.md, section 4 and 7.3). It runs even under a hold
  (4.5): it changes nothing but makes the boot default agree with what
  runs.
- **The desired output is pinned** while it is desired, and the
  outputs the activator reports for its slots are pinned too, so
  collection never takes what a slot runs or a pass is about to place
  (store.md, 7.1).
- **Compare before building.** Most commits change nothing for most
  targets: their output key is the running slot's, and the pass ends in
  milliseconds after the fetch (temper's `factory.md`, 6.2).
- **A prepared slot is reused.** When the inactive slot already holds
  S, untried (a manual trial, or a pass interrupted by a restart), the
  pass goes straight to trying.
- **The pass ends at the reboot.** The trial slot's own steward starts
  afresh, reads its state and the activator's facts, and its pass begins
  in `blessing`, with a render job of the slot's own revision, offline
  (steward.md, section 6). A broken hearth release in the new slot
  cannot bless itself, whether its steward never asks or its builder
  cannot render the revision that built the slot, so it cannot strand
  the machine.
- **A failed unit the target expects does not fail a trial.** A
  desktop often carries one that fails routinely; the target lists it
  in `bless.ignore_failed`, and the minimal bless counts only the
  others (format.md; slots.md, 7.1).
- **Activation is the cheapest that covers the change.** Live first,
  when slots.md, 6.7's rules hold: a builder plans it, as a job, from
  the two outputs' tree files and the live parts of the running and the
  new spec (render.md, 4.4; steward.md, 3.3), and the activator checks
  again. Otherwise the activator chooses a soft reboot or a one-shot
  reboot by slots.md, 6.1's rule (the kernel's digests and its command
  line). `status` shows which a prepared slot needs, and why it is not
  live. A reboot is a clean `systemctl reboot` with
  `SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1` (slots.md, 6.3); only the dead-man
  and a failed live fall back use `systemctl reboot -ff`.
- **A live bless ends the pass converged** with the base still the
  default: the next boot, whenever it comes, is the new slot's ordinary
  trial, whose own pass begins in `blessing` (slots.md, 6.8, L9). The
  pass makes no reboot for it.
- **Effects are keyed or conditional.** prepare is keyed by target and
  revision; a trial is conditional on the facts the pass read. A pass
  that restarts in the middle finds what it made and goes on.

### 4.3 One target at a time

- **A pass converges one target, then the next.** In M3 a machine runs
  one target. In M5 a host runs its containers' targets too (4.7), and
  the loop takes them in a fixed order, one pass each.
- **One prepare and one trial per machine at a time.** A target has one
  inactive slot, the store builds within its own bounds
  (build.md, section 4), and a trial may reboot the whole machine.
- **A slot-changing command from the client while a pass is in
  `preparing` or `trying` is refused as busy** (exit 75). One that comes
  while the pass fetches, renders or builds stops the pass at its next
  stage, and the command goes first.
- **A client's `render`, `diff` or `build` is a job too:** it waits in
  the steward's queue, in the machine's own lane, behind the job in
  flight there (steward.md, 3.4).

### 4.4 Failures and backoff

Classes, as render.md, 7.1 and build.md, 4.3 type failures:
*operational* (build.md's transient): a tool or a resource failed, and
the same input may succeed later; *configuration*: the revision or the
secret store is wrong and must change; *refused*: the slot itself cannot
be made or run, so its prepare key is refused.

| Failure | Class | What follows |
|---|---|---|
| `git fetch`: the remote unreachable, a timeout | operational | back off; a new pass |
| the commit or the branch unknown | configuration | wait until the desired revision changes |
| render: a decode, schema, evaluation, guard, collision or resolution error, such as a package apt cannot locate, or a snapshot timestamp whose archive lacks the suite (render.md, 5.5 and 7.2) | configuration | wait for another commit: render is deterministic, so retrying cannot help |
| render: git or the resolution's mmdebstrap failed, timed out or could not start; fetching the indices failed with no answer or a broken one; a name did not resolve (render.md, 5.5 and 7.2's `tool`) | operational | back off |
| a build's fetch failed: the network, a 404 from the mirror, a digest mismatch (build.md, 4.3) | operational | back off |
| a build step failed: a maintainer script, `customize.sh`, a limit, the scan (build.md, 4.3) | refused | refuse the prepare key |
| out of space | operational | an alarm; collection (store.md, 7.2); back off |
| a secret the spec declares is missing at prepare (slots.md, 5.4) | configuration | wait for the secret store's generation to change: an import |
| the activator found a layer that does not match its record (activator.md, section 6) | none: the store is corrupt | never a refusal: a builder's verify job hashes the layer again and, if it is corrupt, deletes it and its descendants, and the pass builds again (store.md, 4.4; steward.md, 3.3) |
| a job lost twice alone: its builder died or hung under it (steward.md, 5.1) | `crashed`, classed by its job: a fetch's, a verify's or a collection's operational; a render's configuration; a build's refused | back off; wait for another commit; refuse the prepare key. Reported with the job, its stage and the exit status or signal |
| the trial fell back: no bless within the window, or a failed unit `bless.ignore_failed` does not list; or a live trial fell back (slots.md, 8.5) | refused | refuse the prepare key |
| a change not eligible for live, or a live trial whose delta failed to build or verify (slots.md, 6.7 and 6.8) | none: not an error | an ordinary trial; why it was not live is a fact |

- **Backoff** applies to operational failures only. It is jittered and
  doubles from 1 minute to a ceiling of an hour (starting values), and
  starts over when a pass converges or the desired revision changes. A
  branch's poll and the backoff are separate timers: a pass starts when
  both allow it. Render's own overall deadline for a resolution (10
  minutes, render.md, 5.5) bounds one attempt.
- **A refused prepare key is never tried again on its own.** Refusals
  are kept in the steward's state, bounded (the last 16), each with why
  and how long ago. A new commit that makes the same prepare key stays
  refused; one that makes another, by its output, its slot parts or a
  generation, is tried. This is what stops a boot loop.
- **A refusal never covers the running slot.** It names the slot that
  failed, prepare key and all, so a rotation that falls back (4.6)
  refuses only the new generation's slot, and the running one stays
  converged.
- **An operator clears a refusal:** `hearth retry <target>` clears the
  target's refusals, and the next pass tries again; `hearth trial` and
  `hearth prepare --at` act on a refused key when asked, and clear it if
  the trial is blessed. A failed unit that fails every boot belongs in
  `bless.ignore_failed`, not in a retry.
- **A hearth bug is not a crash loop.** A revision that crashes a
  builder ends, after one retry, in a wait or a refusal like any
  failure of its class, while the steward stays up, keeps its faces and
  reports the crash with the revision that caused it (steward.md, 5.1).
- **A failed pass is a fact and an event** (section 6), with its stage
  and its typed reason. `status --check` exits 1 while one stands.

### 4.5 The local hold

- **`hearth hold` stops convergence.** No new pass starts. A pass in
  flight stops at its next stage; a build stops after its current step,
  and what it built stays in the store.
- **It never stops a trial from ending.** A slot on trial is blessed or
  falls back whatever the hold says: a hold cannot keep a machine on an
  unblessed slot.
- **It stays until `hearth release`,** across restarts and reboots, in
  the steward's state, with who held it (the peer's uid), when, and the
  reason given.
- **It does not stop the operator.** `prepare`, `trial` and `rollback`
  still work while held. A manual change that convergence should not
  undo is made under a hold.
- **A roll back needs no hold.** It refuses the prepare key it rolled
  back from (4.4), so the loop does not try that slot again; a new
  commit with another key, a fix or a revert, converges as usual. This is the
  standalone form of the run side's rule that a roll back holds a
  machine until the desired state matches again (temper's `factory.md`,
  section 9).

### 4.6 Secrets rotated

- **Rotating a secret is a new slot of the same output** (README.md,
  section 7). `hearth secrets import` bumps the secret store's
  generation, which the activator reports to its steward as a fact.
- **The next pass sees the running slot's generation behind,** which
  changes the prepare key (slots.md, 2.2 and 5.4), and prepares a new
  slot from the output it already holds: no build, a snapshot and a
  shift (slots.md, 5.1 and 5.2), then a trial, which is a soft reboot
  when the kernel is unchanged; never live, since a rotated secret is
  not in the output's diff (slots.md, 6.7, rule 2).
- **A rotation that falls back** refuses only its own prepare key (4.4).
  The running slot, at the older generation, stays; the next import
  bumps the generation again and makes another key to try.
- **A generation is a number, never a digest of a value,** so no fact
  says anything about a secret's contents.

### 4.7 Containers [M5]

- **A host's spec names the containers it runs** (`target.containers`,
  format.md, 3.5). Each is a target with slots of its own on the host's
  top level, which the host's activator prepares (slots.md, 12.1).
- **The loop takes the containers first, one at a time, and the host's
  own target last,** since the host's trial may reboot the machine and
  interrupt the rest.
- **One desired revision for the machine.** The host and its containers
  converge to the same commit, with the same trial policy (4.1): a
  container is a target of the host's revision, rendered and built by
  the host's builders, in the host's store (build.md, section 2), and
  placed by the host's activator at its steward's request.
- **A container's trial is live when eligible** (slots.md, 12.8), the
  host's activator swapping the overlays inside the container's mount
  namespace and acting on its units through `systemctl --machine=`.
  **Otherwise it is a restart on its new root,** watched, with a fall
  back to the old root if it does not come up healthy, as the bash
  fleet's `switch-container` does (slots.md, 12.2 and 12.3). The host's
  hearth judges both from outside; no hearth runs in the container in
  M5 (slots.md, 12.5).
- **Its bless is in the same pass.** Where a host's pass ends at its
  reboot, a container's goes on: once the restart or the live swap has
  answered, the pass
  waits for the minimal criterion and the container's blessing checks,
  read from the host, and asks the host's activator to bless, or to fall
  back on a failed unit that counts (slots.md, 12.3). A container's
  fell-back prepare key is refused as a host's is (4.4).
- **Each container's output key is compared on its own,** so a commit
  that changes only one container's target leaves the host and its
  other containers alone.
- **A container's first slot** is prepared and tried by its host's pass
  like any other, with nothing to fall back to (slots.md, 12.3); its
  secrets are imported for its own target (client/README.md, 3.6).
- **A container the spec no longer places on the host** is removed in
  the same pass, before the host's own target: the steward asks its
  activator to stop it and delete its slots, which unpins their outputs,
  and its data stays, reported as orphaned (slots.md, 12.7).
- **A development machine may run a selection** of its containers, set
  by `hearth containers`, by `retarget` or by `hearth dev up`
  (development.md, 3.4), and refused on a machine that does not accept
  mutable targets, so the run side's hosts always run their whole spec.
  The pass converges only the selected containers; one left out is
  never built, prepared or started, and one that ran is removed as
  above, its data kept, and reported as left out rather than orphaned.
  The host's own spec, output and prepare key are unchanged.

## 5. The wire [M4]

### 5.1 The channel

- **skein's channel** (skein's `channel.md`), the machine skein built
  for smith's and temper's channels, with hearth's schema: the magic
  `hrth`, one version to start, the kinds of 5.6. It gives framing, an
  opening that agrees a version, terms, bounds and flow control.
- **The domains own reliability,** as in temper's `channel.md`: the
  hello, fenced attempts, acknowledged answers and redialling are
  `hearth-domain-converge`'s and the engine's, never the channel's.
- **The stack, hearth's end:**

  ```
  hearth-domain-steward    routes orders to hearth-domain-converge, facts from every child
  hearth-protocol-wire     the connection, its timers, the translation, calls and their answers
  skein-channel            magic "hrth", versions, terms; hearth is the initiator
  TLS client               skein's; the engine terminates TLS
  io                       a TCP socket to the engine's address
  ```

- **TLS between hosts,** which the engine terminates with skein's TLS
  server side (the engine's need, not hearth's; temper pulls it too).
  Plaintext only to a loopback address, for tests and an engine on the
  same host.
- **The engine's address is configuration,** with what to trust it by,
  written at bootstrap (client/README.md, 3.8). io connects to addresses; the shell resolves a
  name at startup, so a moved engine needs a restart until skein's io
  resolves names (section 11).
- **The steward speaks the wire, never the activator nor the
  builder.** Orders reach the activator over the link, as the client's
  do, and what an order needs built is a builder's job.

### 5.2 The hello

The first application frame on every channel, sent by hearth once both
terms have passed. Nothing else goes before it. The engine's domain hears
of the channel only with it.

```
Hello
  machine       name; architecture; boot: efi | bios; soft reboot: yes | no; watchdog: yes | no
  release       hearth's version (the running slot's); the wire versions it speaks
  targets       for each target the machine runs, its containers' included [M5]:
    slots         per label: state; output key; prepare key; slot parts' digests;
                  the identity's and secret store's generations; slot tree digest,
                  secrets counted by H(name, generation) (slots.md, 2.2);
                  the revision, from the steward's records; drift; age
    boot          running, default, trial: a label or none; live: the delta's slot and prepare key,
                  or none (slots.md, section 3)
    last trial    slot; output key; kind; blessed | held | live-blessed | fell back | rolled back; age
    data          classes and entries, with orphans; per class, its snapshots [M5] and backups [M6]
  orders        in flight: key, attempt, stage
                answered, not yet acknowledged: key, attempt, outcome
  identity      per target, the public halves: SSH host keys, the machine-id
  costs         since the last acknowledged report (build.md, section 12)
  store         bytes used and free; outputs; layers
```

- **Facts, read afresh.** The steward asks its activator for the slots
  and the four facts when it composes the hello, never from a cache.
  The hello is a fresh read the engine's connector can decide on (jig's
  `connectors.md`, 5.1).
- **No time crosses the wire; only durations do** (temper's
  `channel.md`, 3.3). Ages are durations, and each side turns them into
  its own clock's times.
- **The answers it carries are resent on every channel** until the
  engine acknowledges them (5.5). The hello after a reboot is where the
  answer to an activation arrives.
- **How the engine tells the outcome of an activation:**

  | Facts in the hello | What happened |
  |---|---|
  | running `b`, default `a`, trial `b`, in phase `running` | `b` is on trial; its bless is pending |
  | running `b`, default `a`, trial `b`, in phase `held` | `b` held; its bless is held for the engine until the hold's end (slots.md, 7.4) |
  | running `b`, default `b`, no trial; `b` blessed | `b` held, and is blessed |
  | running `a`, default `a`, no trial; `b` fell back | `b` fell back; `a` runs |
  | running `a`, default `a`, no trial; `b` prepared, no record of its trial | the activation never began |
| running `a`, default `a`, live `b`, trial `b` in phase `running`; `b` on live trial | `b`'s files run live over `a`; its live bless is pending |
| running `a`, default `a`, live `b`; `b` live-blessed, the one-shot its trial entry | `b` passed live; its next boot is its trial |
| running `a`, default `a`, no live, no trial; `b` fell back, kind live | `b`'s live trial fell back; `a`'s files run, with no reboot |

- **A container's activation [M5] drops no channel:** its host stays
  up, so its outcome reaches the engine as fact updates while the
  channel is open, and the same table reads its facts in any later
  hello.
- **A host's live activation drops the channel once,** when the steward
  is spawned anew after the swap (slots.md, 6.8, L6); the new steward's
  hello carries the live fact, and the rest arrives as fact updates.
- **Bounded.** The hello's size follows from the limits: targets per
  machine, slots per target, orders kept, identity entries. Its largest
  body is in the terms.

### 5.3 Identity

How a machine proves who it is to its engine, and to its plane's secret
store, is open (README.md, section 9). What is fixed:

- **One machine, one credential,** presented in the channel's `Open`
  (an opaque credential, judged by the engine's owner) or by TLS. The
  engine knows machines by it, and keeps one channel per machine: an
  `Open` under a credential that already has one closes the older with
  `replaced` (temper's `channel.md`, 4.4).
- **Planes stay apart by wiring.** A machine's configuration names one
  engine and one trust. A lab machine holds no credential the run side's
  engine accepts, and the run side's engine refuses an unknown one as
  `unauthorized`, one reason for every failure so credentials cannot be
  probed.
- **The candidates:**
  - **a credential issued at bootstrap:** a name and a secret, entered
    with `hearth bootstrap` and kept in the steward's state. It works
    with skein's channel today. It is a bearer secret: whoever reads it
    is the machine.
  - **a client certificate:** a key made on the machine at bootstrap
    that never leaves it, its public half reported as an identity fact
    and recorded in the plane, by a change to the IaC repository or by
    an operator. It needs client certificates in skein's TLS client (an
    ask of skein, README.md, 8.1) and the engine's TLS server verifying
    them.
- **The lean is the certificate,** since identity's public halves are
  already facts (section 6) and nothing secret crosses.
- **Secrets under an engine** are fetched from the plane's store under
  the same identity. The lean: the store seals each secret to a key of
  the activator's (generated with the machine's identity, its public
  half a fact), and the steward fetches and relays the sealed bytes to
  the activator, so no secret value ever passes through the steward or
  its builder.

### 5.4 Orders

An order is the engine's request to change the machine. Each carries the
engine's key, derived from what it is for (jig's `connectors.md`, 4.4),
and an attempt (5.5). Each kind declares its recovery class (jig's
`connectors.md`, 4.3).

| Order | Fields | Form | Class |
|---|---|---|---|
| prepare | target; revision; the expected output key, optional; sources [M6] (section 8) | creation | keyed, by target and revision |
| activate | target; slot; how: `auto`, `live` or `full`; the running slot, the default and the live state decided from; bless: when healthy, or held for at most a duration | transition | conditional, on the running slot, the default and the live state |
| bless | target; slot | transition | conditional, on that slot running on trial, healthy, its bless held |
| roll back | target; the running slot and the default decided from | transition | conditional, on the running slot and the default |
| restart a unit [M5] | target; unit | transition | keyed, by the order's key, which hearth keeps; conditional at the activator on the unit's invocation (activator.md, 5.9) |
| restore [M5] | target; class; entry, optional; the snapshot's sequence; the class subvolume's UUID decided from | transition | keyed, by the order's key; conditional on the class's UUID (activator.md, 5.12) |
| build an artifact [M6] | its key, or repository, commit and recipe | creation | keyed, by the artifact's key (artifacts.md, 5.7) |

**prepare.**

- **Keyed by target and revision.** The steward keeps each prepare's
  target, revision and outcome (4.1; store.md, 8.3), and has a builder
  render the revision to an output and slot parts (steward.md, 3.3).
  The activator keys the place by its prepare key (slots.md, 2.2;
  activator.md, 4.4): a prepare whose output, parts and generations a
  slot already holds, prepared or running, answers with that slot and
  does nothing.
- **The expected output key** is what the engine rendered for the
  target (any hearth renders any target). A prepare that makes another
  key is refused as `mismatch`, which is a bug worth a finding: two
  hearths disagree on a render.
- **A newer prepare supersedes.** A prepare of the same target at
  another revision, while one builds, stops the first at its next step
  and answers it `superseded`. Once the activator has begun placing the
  slot, the first finishes, and the second is refused `busy`.
- **Outcomes:** prepared, with the slot, the output key, the slot parts'
  digests and the activation it needs (live, a soft reboot, a reboot,
  or a restart for a container; when not live, the rules that fail);
  superseded; refused (`busy`, `mismatch`); failed, with the stage and
  the typed reason.

**activate.**

- **Conditional on the running slot and the default** being those the
  engine decided from. The activator checks them as it applies the
  trial (activator.md, 4.4 and 5.2). A late or repeated copy finds the
  facts moved and is refused, with the facts as they are.
- **Its answer is `begun`,** recorded in the steward's state before the
  activator sets `BootNext` or `/run/nextroot`, and sent before the
  reboot when the channel allows, else in the next hello. The trial's
  outcome is told by the facts (5.2), never by an answer.
- **`bless: held for at most d`** holds the bless while a canary
  bakes: its semantics are slots.md, 7.4, and the activator's operation
  activator.md, 5.13. The engine then sends `bless` or `roll back`, and
  the hello shows the slot as held (5.2).
- **`how`** is slots.md, 6.1's: `auto` is live when eligible, else a
  soft or a one-shot reboot; `live` is live only, refused with the rules
  that fail when the change is not eligible, so an engine can keep
  reboots for a window of its own; `full` is a one-shot reboot.
- **A live activation's `begun`** is recorded before the swap and sent
  before the steward is spawned anew. A held live bless releases into
  the live bless; `roll back` while live is the live fall back
  (slots.md, 8.5). The live bless leaves the default where it was: the
  slot becomes the default only at its native boot's bless, which the
  hello after that boot shows, whenever the engine or a person reboots
  the machine.

**bless.**

- **Releases a held bless** (slots.md, 7.4), conditional on the slot
  running on trial, healthy and held. Outcomes: blessed; refused, with
  the facts.

**roll back.**

- **Switches to the other slot** (slots.md, section 9;
  activator.md, 5.5): a trial of the other blessed slot, then its own
  bless, conditional on the running slot and the default decided from.
  Its answer is `begun`, as for activate; the hello after the reboot
  shows it held. Whether to try the rolled-back output again is the
  engine's to decide.

**restart a unit.**

- **Keyed by the order's key, which hearth keeps.** systemd keeps no
  operation id for a restart, so without hearth's record the effect
  would be unrecoverable (jig's `examples.md`, 3.2).
- **Before acting,** the steward records the key with the unit's
  `InvocationID` (`systemctl show --property=InvocationID --value
  <unit>`), and the activator restarts it only if the invocation is
  still that one (activator.md, 5.9). After a crash, a repeated order
  finds a new invocation (restarted) or the same one (not yet, so
  restart now).
- **Only a unit of the running slot's spec:** one its config part ships
  or enables. hearth's own units, and any other, are refused.
- **Outcomes:** restarted, with the new invocation; failed, with the
  unit's `Result`; refused.

**restore.**

- **One entry or a whole class, from a snapshot** (operating.md, 7.4):
  from M5 a snapshot on the machine, from M6 one the peer holds. Its
  answer is `begun`, as for activate, sent before the reboot, and the
  hello after it shows the restored class's new UUID in the `data`
  fact. A container's restore restarts the container
  instead.

**For a container's target [M5]:**

- **Orders reach its host's hearth,** which builds, prepares and
  activates the container (4.7): prepare, activate and roll back are
  keyed and conditional as above, on the container's own facts, and an
  activation is live when eligible (slots.md, 12.8), or a restart
  (slots.md, 12.2), answered `begun` before it.
- **A held bless** is a host's (slots.md, 7.4): `activate` with `bless:
  held for at most d`, released by `bless`; at the ceiling the host's
  activator falls the container back, a restart on its old slot, as at
  its window's end (slots.md, 12.3).
- **A restart of a unit** is the host's order with the container's
  target: the unit is one inside the container, keyed and conditional
  on its invocation as above, read and restarted through
  `systemctl --machine=<container>` (activator.md, 5.9).

**For every order:**

- **hearth keeps every order's key with its outcome** in the steward's
  state, so a repeat by key always gets the first outcome, whatever its
  class. The condition protects an order whose first copy hearth never
  recorded.
- **Recorded before it acts.** An order's key, attempt and stage are
  written before the effect is asked of the activator, and an order
  found begun at startup is resumed under its key or its condition.
- **One slot-changing order per target at a time.** Another arriving
  meanwhile is refused `busy`, except a prepare that supersedes.
- **Policy is the engine's.** Which orders a task may give, and when, is
  authority's (jig's `authority.md`). hearth checks only mechanics: the
  key, the condition, the unit, the plane (section 8).

### 5.5 Redial and attempts

- **hearth dials out,** at once on startup. Machines need no inbound
  port, and the engine no list of addresses.
- **It redials with a jittered backoff** that doubles from 1 second to a
  ceiling of 2 minutes (starting values), and starts over once a channel
  has proved itself: when the engine has acknowledged the hello. A fleet
  that comes back from a power cut spreads its dials.
- **The protocol layer never redials.** Its end of the link runs idle,
  connecting, opening, open, closing, and says `Lost` once per dial
  (temper's `channel.md`, 11.2). The domain decides when to dial again.

  ```
  state        event                                  next         does
  idle         Dial                                   connecting   connect; arm handshake
  connecting   connected                              opening      TLS, then Open
  opening      Accept, then Terms that fit            open         Terms; the domain's Hello; Connected
               Refuse, or Terms that do not fit       closing      (Refuse limits, for the second)
  open         a request                              open         its frame; facts while room is left
               a frame                                open         its event
               silence, stall                         closing
  closing      closed                                 idle         Lost
  ```

- **Timers, in `hearth-protocol-wire`** (the channel keeps none): the
  handshake within 10 s; a `Ping` after 15 s of sending nothing; lost
  after 45 s of hearing nothing (silence); lost after 60 s of queued
  output that does not drain (stall). These are temper's starting
  values, to tune on real networks.
- **Attempts are fenced.** Every order carries an attempt that only
  grows per key.
  - An order whose key has an outcome is answered with it, under the new
    attempt.
  - An order whose key is in flight under a lower attempt goes on as it
    is, and its answer names the newer attempt; a copy under the same or
    a lower attempt is dropped.
  - The engine drops an answer for an attempt it has finished with.
  - So a stale attempt can never act over a newer one.
- **Answers are kept until acknowledged.** Each outcome stays in the
  steward's state until the engine acknowledges its key, which it does
  once it has committed it, and is resent in every hello. At most
  `orders` outcomes wait (64 to start); past that, new orders are
  refused `busy` until the engine catches up.
- **Losing the channel is survivable.** Orders in flight run to their
  end, and their outcomes wait. Facts keep their latest values, which
  the next hello carries. Events are hints: while the channel is down
  they are dropped and counted.
- **A reboot is a redial.** An activation's reboot drops the channel;
  the new slot's steward dials; its hello carries the facts. There is no
  other path for a machine coming back.
- **Liveness is the dial.** A machine that stops redialling is down; the
  engine judges it after its silence. hearth sends no heartbeat beyond
  the channel's pings.

### 5.6 The messages

Application kinds of skein's channel; each belongs to one direction.

| Kind | Message | Fields | Shape |
|---|---|---|---|
| `0x0101` | `Hello` | 5.2 | first, once per channel |
| `0x0102` | `Facts` | fact updates: latest values (section 6) | notice, only with room |
| `0x0103` | `Event` | an event (section 6) | notice, only with room; else dropped and counted |
| `0x0104` | `Answer` | key; attempt; outcome (5.4) | answers `Order`; kept until acknowledged |
| `0x0105` | `ReadAnswer` | read id; a page; a cursor, if more | answers `Read` |
| `0x0181` | `Order` | key; attempt; one of the orders of 5.4 | call |
| `0x0182` | `Acknowledge` | keys of answers committed | notice; the first one after a hello proves the channel |
| `0x0183` | `Read` | read id; what: facts afresh (M4), or a window (section 7, M5) | call |

- **Refusal reasons,** the application's, from 256 (skein's
  `channel.md`, 5.1): 256 *unauthorized*, 257 *busy* (the engine turned
  the machine away), 258 *replaced*.
- **The output cap** is sized from the limits, as temper's (its
  `channel.md`, section 10): the hello, plus `orders` answers, plus read
  answers in flight, plus a reserve for facts. Facts and events go only
  while the reserve stays free, so the channel never refuses an answer.
- **Records are versioned by the agreed version,** with a Rust variant
  per version where a layout changes (temper's `channel.md`, section
  17), so a record cannot be encoded under the wrong layout.

## 6. Facts and events

- **A fact is state, read afresh;** a newer value replaces an older one.
  The engine may ask for any fact afresh with a `Read`, and the hello
  carries them all.
- **An event is a hint** that something may have changed: the engine
  reads the fact it needs. Events may be dropped; facts never lie.
- **From** names the milestone that first reports each: from M3 to the
  client (`status`, `identity`), from M4 also on the wire.

| Name | Kind | What it carries | Known by | From |
|---|---|---|---|---|
| `machine` | fact | target; architecture; firmware; `SecureBoot`; the booted kernel; boot id; soft reboots; watchdog; firmware quirks seen (activator.md, 4.3) | activator | M3 |
| `release` | fact | hearth's version; the wire versions spoken | steward | M3 |
| `slot` | fact | target; label; state; output key; prepare key; slot parts' digests; the identity's and secret store's generations; the trial window [M5]; the slot's tree digest, in which each secret's contents count as `H(name, generation)`, never the value; age (slots.md, 2.2); the revision, from the steward's records | activator | M3 |
| `boot` | fact | per target: running, default, trial, live (slots.md, section 3) | activator | M3 |
| `live` | fact | per target, the live state: base; delta; the slot and prepare key it carries, or none; hierarchies overlaid; whether the mount table agrees with the live record (slots.md, 2.4) | activator | M3 |
| `trial` | fact | the last trial: slot; attempt; kind (live, soft reboot, reboot, restart); phase reached; outcome; for a live one, the step that failed and the units acted on; age | activator | M3 |
| `activation` | fact | per target, how its prepared slot would be activated: live with its unit plan, or a soft reboot, a reboot or a restart, and when not live, why: the rules of slots.md, 6.7 that fail, with their first paths or units, bounded | steward, from a builder's plan | M3 |
| `data` | fact | per target, each class subvolume present and mounted, and its UUID; each entry's directory present and bound; orphans, by name (machine.md, 3.4) | activator | M3 |
| `snapshots` | fact | per class: its snapshots, each with its sequence and age; the newest's and oldest's age; the next due; the bytes the class's mount holds (operating.md, 7.1) | activator | M5 |
| `drift` | fact | per slot: clean, or the stable ids of what differs (slots.md, section 10) | activator | M3 |
| `identity` | fact | per target, public halves: SSH host keys (type, key), the machine-id | activator | M3 |
| `secrets` | fact | the secret store's generation; declared secrets missing, by name | activator | M3 |
| `desired` | fact | standalone: revision, trial policy, generation; on a development machine [M5], the containers it runs and those it leaves out (4.7) | steward | M3 |
| `pass` | fact | standalone: stage; the job in flight; last outcome; refused keys with why; backoff left | steward | M3 |
| `hold` | fact | standalone: who, reason, age | steward | M3 |
| `store` | fact | bytes used and free; outputs; layers; pins | builder | M3 |
| `costs` | fact | per build: each step's time, bytes fetched, cache hits; totals (build.md, section 12) | builder | M3 |
| `orders` | fact | in flight: key, attempt, stage; answered, not acknowledged | steward | M4 |
| `units` | fact | per target, the running slot's failed units, with their results; a container's read from its host (operating.md, 2.9) | steward | M5 |
| `check` | fact | per check: state, last result, reason, slot and output, run by (operating.md, section 2) | steward | M5 |
| `alarm` | fact | disk, memory, pressure: level, value, threshold (operating.md, section 4) | steward | M5 |
| `backup` | fact | per class: the last sent and the last confirmed, their ages, the peer, the bytes sent (operating.md, 7.2) | activator | M6 |
| `fault` | fact | a pool host's active faults, each with its end (operating.md, section 8) | activator | M6 |
| `artifact` | fact | an artifact's key: its tree digest, or failed; its costs (artifacts.md, 5.6 and 5.7) | builder | M6 |
| `prepared` | event | slot; output key | steward | M3 |
| `trial begun` | event | slot; soft reboot or reboot; why it was not live | activator | M3 |
| `live trial begun` | event | slot; base; delta; the units stopped, reloaded, restarted and started | activator | M3 |
| `live-blessed` | event | slot; the one-shot set to its trial entry; the default unmoved | activator | M3 |
| `fell back (live)` | event | slot; the step and the reason; whether the replay held or `reboot -ff` followed | activator | M3 |
| `bless held` | event | slot; the hold's end (slots.md, 7.4) | activator | M4 |
| `blessed` | event | slot | activator | M3 |
| `fell back` | event | from slot; to slot | activator | M3 |
| `rolled back` | event | from slot; to slot | activator | M3 |
| `drifted` | event | slot | activator | M3 |
| `pass failed` | event | standalone: stage; typed reason | steward | M3 |
| `builder lost` | event | the job; its last stage; the exit status or signal; whether it is tried again (steward.md, 5.1) | steward | M3 |
| `job crashed` | event | the job, lost twice; its target and revision; the class the pass gave it (steward.md, 5.1) | steward | M3 |
| `render changed` | event | slot; its output key; the output key the running release renders the slot's revision to (steward.md, section 6) | steward | M3 |
| `unit failed` | event | unit; result | steward | M5 |
| `check changed` | event | check; from state; to state | steward | M5 |
| `alarm raised`, `alarm cleared` | event | alarm; value | steward | M5 |
| `snapshot failed` | event | class; reason, such as a full disk | activator | M5 |
| `restored` | event | class; entry, if one; the snapshot's sequence | activator | M5 |
| `backup sent`, `backup failed` | event | class; reason | activator | M6 |
| `fault began`, `fault ended` | event | fault; target | activator | M6 |

- **"Known by"** is the process that owns the fact. The activator's
  reach the steward over the link (activator.md, 4.3), and the
  steward relays them. A builder's come in its jobs' answers
  (steward.md, 3.2), and the steward keeps their latest values, since
  no builder runs between jobs. Drift is the activator's
  (README.md, 4.2).
- **The slots' outputs are pins.** The output keys in the activator's
  `slot` facts pin those outputs in the store, with their ancestors,
  for as long as a slot holds them or is prepared from them
  (store.md, 7.1). The steward gives them to every builder with its
  `Welcome`, and a builder collects nothing before it has them
  (steward.md, 3.2).
- **Events after a reboot** are not replayed. `blessed` and `fell back`
  happen in a boot whose channel may not be open yet; the hello's facts
  say the same.

## 7. Telemetry on demand [M5]

- **Reads, bounded, answered by pages.** A `Read` names a window; the
  `ReadAnswer` holds at most `read_bytes` (256 KiB to start) and a
  cursor to continue. Reads are calls: not kept, not acknowledged, asked
  again if the channel drops.

  | Read | Window | From |
  |---|---|---|
  | `journal` | target; units; since and until, as durations ago; priority; at most N entries; after a cursor | `journalctl --output=json --no-pager --after-cursor=… --lines=…`, run by the steward, parsed by `hearth-protocol-systemd`; a container's [M5] with `--directory=/var/log/journal/<its machine-id>` (operating.md, section 5) |
  | `signals` | names; since; resolution | hearth's own ring of samples (operating.md, section 5) |
  | `checks` | check ids; since | each check's history (operating.md, section 5) |
  | `facts` | names | afresh, as the hello's |

- **Journal entries carry their wall-clock times as data.** No deadline
  crosses the wire; a log's time is what it says, not a deadline.
- **The steward reads the journal itself.** The activator spawns it
  with `hearth`'s supplementary groups (activator.md, 3.2), and the
  hearth task puts `hearth` in `systemd-journal` (format.md, 5.12), the
  group of `/var/log/journal` (format.md, 5.7). No root is involved in
  a read, and no builder: it runs with no supplementary group
  (steward.md, 2.3).
- **Agents read through the engine,** for triage, with the engine's
  authority deciding who may read what.
- **Locally from M3.** The client's `hearth logs` reads the same journal
  window through the steward on the machine itself (client/README.md, 3.1); only reads
  over the wire wait for M5, as do the signals and the checks' history.
- **Bounded at both ends.** One read in flight per kind, the page's
  bytes, the entries' count and the window's span are limits; a read
  past them is refused at the entrance.

## 8. Bytes by digest [M6]

- **Outputs move as change sets, by HTTP GET, outside the channel.**
  Content-addressed bytes need no session, and any cache can serve them
  (`draft/brainstorm.md`, 5.3). The wire carries intent; HTTP carries
  bytes.
- **What is served** (store.md, 10.1 fixes the formats, and 10.2 how a
  change set is received):

  ```
  GET /v1/change-set/<layer key>    the layer's record as its header, then its change set
                                    against its parent (an artifact's: against the empty tree)
  GET /v1/blob/sha256/<digest>      a download, or a .deb by the SHA-256 its Packages gives
  ```

  Ranges resume an interrupted transfer; a file-backed body serves it
  without reading it into memory (both asks of skein, README.md, 8.1).
- **Who serves:** every hearth of a plane, for the layers its store
  holds, and a pool's cache (temper's `factory.md`, 6.5). The steward
  holds the HTTP listener; a change set's bytes come from a builder's
  job, streamed through the steward (steward.md, 2.1 and 3.3).
  `hearth-protocol-http` holds the routing, on skein's HTTP client and
  server.
- **Where from.** A prepare order names its sources (5.4): peers and
  caches of the machine's plane, in order of preference, from what the
  engine knows of where outputs live. A machine with no sources builds.
- **Verified by digest, trusted per plane.**
  - A blob is checked against its digest as it arrives.
  - A change set is applied to a snapshot of a parent the receiver
    holds, and the result's tree digest checked against the one its own
    engine gave for the key, in the order (store.md, 10.2 and 10.4).
    The engine had it from the hearth whose builder made the layer,
    over that machine's authenticated wire.
  - A machine fetches only from its own plane's peers and caches, and
    takes tree digests only from its own engine, so a lab output never
    reaches production (`draft/brainstorm.md`, 11.2; store.md, 10.4).
    Artifacts follow the same rule (artifacts.md, 4.5).
- **Plain HTTP within a plane's network,** since integrity comes from
  the digests. Whether bytes need TLS for confidentiality (outputs hold
  nothing secret: README.md, section 1) is open.

## 9. Generic at the wire [M4]

The factory wants its concepts generic, with hearth as their first
backend (temper's `factory.md`, section 7). The genericity lives in the
wire, not in traits: another backend is another daemon speaking it.

| What an engine sees | What it is inside hearth, which the engine never sees |
|---|---|
| a target, a revision | a base in an environment; a commit of the IaC repository (format.md) |
| an output, by key | the config layer, on software and packages layers, on btrfs (store.md) |
| parts, by name and digest | packages, software, config, data, secrets, boot: names the engine's policy may match ("caution follows what changed", temper's `factory.md`, 6.3), never interprets |
| a slot, by label, and the four facts | `@<target>.<label>` and its `/var`; `BootNext`, `BootOrder`, GRUB's env (machine.md); a delta subvolume and overlays in the mount table; for a container, its link and the default its host records [M5] (slots.md, 12.6) |
| an activation: live, soft reboot, reboot, restart | overlays swapped beneath `/etc`, `/usr/local` and `/opt` with the units acted on (slots.md, 6.8); `systemctl soft-reboot` into `/run/nextroot`; `systemctl reboot` with `SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1` (slots.md, section 6); a container's restart |
| a unit, by name | a systemd unit of the running slot |
| a check, by id, with its result | a unit, a command or a probe (operating.md) |
| drift, by stable ids | a slot compared with its record |
| data by class and entry; snapshots by sequence | class subvolumes, each entry a directory bound at its path; read-only btrfs snapshots (machine.md, section 3) |
| costs | per step: time, bytes fetched, cache hits |

- **Never at the wire:** a package, a resolution, a layer's name, a
  subvolume, an EFI variable, a tool's text.
- **A part's name is a label.** The engine reads that `config` changed
  and `packages` did not, to expect a live change or a soft reboot and a
  short bake; it needs to know nothing of dpkg. Why a change is not
  live comes as rule numbers and paths, never as Debian's terms.

## 10. Its world

- **The client's world** is `client/README.md`, section 4.
- **The standalone world** runs `hearth-domain-steward`,
  `hearth-domain-builder` and `hearth-domain-activator` on the fake
  machine, with a fake forge (a git remote, from skein's fake checkout)
  whose branch moves under it. The seed draws: commits that change
  nothing, change config, or break render; build steps that fail;
  builders that crash or hang on a revision, at every job; power cuts at
  every stage of a pass, of a prepare and of a trial; trials that hang,
  panic or fail the minimal bless; imports of secrets; holds and
  releases; changes eligible for live and not, live trials whose reload
  fails, cut at every phase, and later changes while live; [M5] a host's
  containers, their trials failing or cut by a host reboot, live or not.
  Its invariants:
  - the machine always boots a blessed slot or a trial that ends, and
    each container starts on one [M5];
  - a refused prepare key is never tried by the loop, and a refusal
    never covers the running slot;
  - a slot that runs but is not the default is blessed again before
    any prepare;
  - a hold starts nothing new, and never keeps a trial alive;
  - a trial is live exactly when slots.md, 6.7 allows, and a live bless
    never moves the default;
  - at most one prepare and one trial per machine at a time;
  - backoff and polling stay within their bounds; a pass for an old
    generation never acts;
  - a revision that crashes a builder ends in a wait or a refusal,
    never a loop, and the steward answers the client throughout;
  - no bless is asked before the bless's render has answered.
- **The wire's world** puts a fake engine on the other side: a step
  machine speaking the wire's domain records (and, in a protocol world,
  its bytes through skein's channel). It accepts or refuses dials,
  checks that the hello comes first, gives orders with keys and
  attempts, resends them, delays and loses acknowledgements, drops
  channels at any moment, opens a second channel under the same
  credential, and reboots the fake machine mid-order. Its invariants:
  - each key gets the same outcome every time it is asked;
  - a conditional order never applies from facts other than its own;
  - every outcome reaches the engine, by an answer or in a hello, until
    acknowledged;
  - after any reboot, the hello's facts tell held, fell back or never
    began, and agree with the fake machine; after a live activation's
    respawn, they tell on live trial, live-blessed or fell back (live);
  - one channel per machine; no fact before the hello; facts only with
    room;
  - the dial's backoff stays jittered and bounded.
- **The fake engine is shaped for the factory's infrastructure
  connector:** its world (jig's `connectors.md`, section 15) can use the
  same fake hearth behind it, and the same fake machine below.

## 11. What is open

- **Machine identity:** a credential issued at bootstrap or a client
  certificate (5.3); it depends on skein's TLS client certificates and
  on how the plane registers machines.
- **Secrets under an engine:** sealed to the activator's key and relayed
  by the steward, or fetched by the activator itself; it depends on the
  plane's secret store, which the factory owns (README.md, 8.3).
- **Where the wire's credential lives:** in the steward's state, or held
  by the activator and lent to the steward; it depends on the identity
  chosen and on how much the steward is trusted, which is no more than
  its builder (steward.md, 2.3).
- **A break-glass hold under an engine:** whether a person on the
  machine may hold it against its engine's orders; it depends on the run
  side's rule that no hand touches a machine (temper's `factory.md`,
  section 9).
- **Names for the engine:** resolving the engine's name while running,
  rather than at startup; it depends on a resolver in skein's io.
- **The liveness timings and backoff ceilings:** starting values, to
  tune against real networks and fleets.
- **TLS for bytes by digest:** whether change sets need confidentiality
  within a plane; it depends on the plane's networks.
- **A credential for the repository under an engine** [M4]: in M3 the
  builder's own read credential is imported at bootstrap and replaced
  with `hearth credential` (client/README.md, 3.8); whether an engine later issues it with
  the machine's identity depends on 5.3.
- **Trials on a timetable:** whether `trial: manual` grows a maintenance
  window (a time of day), when a machine needs one.
- **Live changes on a manual machine:** a manual machine applies them
  only on `hearth trial` (4.1); whether it should apply them at once and
  wait only for reboots depends on how often a live change surprises
  the person at the machine.
- **Changing a machine's mode** between standalone and an engine without
  a new bootstrap; it depends on how identity is issued (5.3).
