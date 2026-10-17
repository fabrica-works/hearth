# The activator

Provisional, 2026-10-09. The privilege boundary of a machine: what the
activator trusts and what it never does, the process pair it forms with
its steward, the link between them, its closed list of operations, how
it verifies what it is asked to place, its state, and the tools it
drives. M3 builds it, with the operations a slot needs, live changes
included; restarting a unit, snapshots and restores of data, and a
container's operations are M5; backups and disks as an effect are M6.
The slots it carries are slots.md's, the layout it writes is
machine.md's. What is
open is in section 10.

## 1. In one page

- **The activator is root, small and closed.** It owns the machine's
  slots, boot state, identity and secrets, and on a host its containers'
  too [M5], and carries out a closed list of operations on typed
  requests. It runs no IaC code, renders no template and parses no IaC.
- **It starts the steward and listens to nothing else.** systemd starts
  `hearth activator` as root; it spawns `hearth steward` as the user
  `hearth` and talks to it over a pipe pair. It has no socket and no
  port. The steward spawns the builder itself, for work (steward.md).
- **The link carries sized records:** requests, each with exactly one
  answer, and facts, pushed whenever they change. Every mutating request
  is keyed or conditional on the facts it was decided from, so a steward
  that restarts and asks again finds what its first request made.
- **What it places, it verifies.** The output must be a read-only layer
  of the store, owned by the builder's range, with a record. The
  activator snapshots it first and checks the snapshot, so the builder
  cannot swap it; the shift's walk recomputes the tree digest, which
  must equal the record's.
- **It carries live changes itself** (5.17 to 5.19): it builds a
  verified delta of a prepared slot, mounts it over the running slot's
  `/etc`, `/usr/local` and `/opt` with the new mount API, acts on the units
  the steward's plan names, and falls back without a reboot, or with
  `reboot -ff` when a step of the fall back fails.
- **Secrets never cross the link.** They enter the secret store from a
  root process (an import run in-process by the client) and leave it
  only into a slot. The steward sees names and the store's generation;
  its builder, the names render gives.
- **Its state is files in `@hearth.state`:** the machine record, slot
  records and manifests, the trial marker, identity, the secret store,
  and a journal of the operation in flight, so a crash or a power cut at
  any step is recovered at the next start.
- **It drives tools as protocols:** btrfs through ioctls (asks of skein),
  efibootmgr, grub-editenv, mount, systemctl and ssh-keygen, and at
  bootstrap parted and the mkfs tools. It never runs a program from a
  slot it prepares or from the store, and it resolves every path in a
  slot from the slot's own descriptor, following no link.
- **What it does not protect:** a compromised builder or steward is
  root at the next trial, since it can place any output the builder
  built, and that output's activator is root. The defence against a bad
  output is review and the lab; the activator guarantees that only the
  closed operations happen, on inputs that are what they claim, and
  that secrets never reach the steward or its builder.

## 2. The trust boundary

### 2.1 What runs as root

- **The activator,** hearth's own code, from the running slot: a sealed
  root that an activator prepared and that was blessed.
- **The running system's tools,** spawned with fixed argument vectors
  (section 8). At bootstrap the running system is the live system.
- **No task's check:** systemd runs a check [M5] as its declared user,
  never `root`, under fixed sandbox properties, from the running slot's
  own files (5.14).
- **Nothing else.** Never a program from the store, from a slot being
  prepared or from the repository; never a shell, a hook or a template.
  IaC code runs only in the builder's sandboxes (build.md, section 8).

### 2.2 What it accepts

- **Link records,** decoded by `hearth-protocol-link`, whose lengths are
  checked against the kind's limit before anything is allocated
  (`programming-model.md`, section 8).
- **Typed values only:** keys, labels, names, tokens and a few paths.
  The activator derives every other path itself: an output's from its
  key, a slot's from the machine's target and the label, a class's
  from the target and the class, an entry's directory from its class
  and name, a snapshot's from the class and the sequence, a secret's
  from the store's layout. No request names a store path, a subvolume
  or a device.

| Input | Rule |
|---|---|
| output key | a digest; a layer and a record of that name exist in the store (6.1) |
| label | `a` or `b` |
| target | the machine record's target; or [M5] a container whose placement's `on` is that target, by the request's `meta` |
| a container's parameters [M5], its `boot` part (slots.md, 12.1) | the id base 64 Ki aligned, its range clear of the builder's (store.md, 9.2) and of the host's other containers', and the owner of the roots of the container's existing class subvolumes; binds' paths absolute and normal, a source never the top level, `/run/hearth-activator`, `/var/lib/hearth` or anything beneath them, at most 32 binds; capabilities by name from the kernel's list; a bridge an interface name |
| `data` entries | machine.md, 3.4: a name, a class (`core`, `data` or `cache`) and a path; at most 64; a class's policies only for `core` and `data`, every period at least 10 minutes, every bucket's count from 1 to 1000 |
| secret and identity paths | absolute and normal; not beneath a `data` path, `/var/lib/hearth`, `/.hearth`, `/proc`, `/sys`, `/dev`, `/run` or `/tmp`; not a path the activator writes itself (`/etc/fstab`, `/etc/machine-id`, `/etc/shadow`) |
| secret modes | at most `0777`: no setuid, setgid or sticky bit; nothing for others to read |
| owners and groups | names found in the slot's own `/etc/passwd` and `/etc/group`; for a secret or identity file, never `hearth` nor a group `hearth` belongs to |
| secret names, identity kinds | format.md, section 9; kinds from the closed list of 5.7 |
| `boot` tokens | machine.md, 6.2 |
| `mutable`, in the `boot` part | only where the machine record accepts mutable targets (development.md, 4.3); the slot is then left unsealed and its `/` mounted `rw` |
| unit names [M5] | systemd's unit name syntax, at most 256 bytes; never hearth's own |
| a live plan's units (5.17) | systemd's unit name syntax, at most 256 bytes; at most 256 in all; never hearth's own nor a never-live one (format.md, 5.14) |
| a check [M5] | a user of the running slot, never `root`; capabilities from a closed list; an absolute program path and at most 64 arguments of at most 4 KiB in all (operating.md, 2.4) |
| a fault [M6] | a kind from 5.15's closed list, typed parameters within bounds, an end; only on a lab pool host |

A request that breaks a rule is refused whole, naming the field, and
changes nothing.

### 2.3 What it reads of an output

- **Through the snapshot's descriptor only.** Every path in a slot is
  resolved from the snapshot's root descriptor with
  `openat2(RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_XDEV)` (an
  ask of skein, README, 8.1), never by name in the activator's own
  namespace, so a link the output plants at `/var`, `/etc` or a
  kernel's path leads nowhere. What must be a directory or a regular
  file is checked as such (slots.md, section 5).
- **Every inode's metadata and contents,** to shift and hash them
  (slots.md, 5.2): bytes as data, never interpreted.
- **The kernel and the initramfs,** copied byte for byte to the ESP.
- **`/etc/passwd`, `/etc/group` and `/etc/shadow`,** through the bounded
  line parsers of `hearth-protocol-accounts` (8.1; fixed field counts,
  at most 1 MiB and 4096 lines each), to resolve owners by name, replace
  password hashes and confirm the package users (slots.md, 5.4 and
  5.7). They are files of an output built from IaC, so they are
  untrusted input: a line that does not parse fails the prepare.
- **No error carries a file's content,** only its path and what was
  wrong with it.
- **Nothing else:** no unit file, no configuration, no template.

### 2.4 Where it writes

- **The top level:** this target's slot subvolumes, its live deltas
  `@<target>.live.<n>` (slots.md, 6.6), and its class subvolumes and
  entries' directories when they are missing; its snapshots in
  `@hearth.snapshots` and a restore's renames [M5]; on a host, its
  containers' too [M5]. Nothing else there.
- **For containers [M5]:** their nspawn settings in
  `/run/systemd/nspawn/` and their links in `/run/machines/`
  (slots.md, 12.1).
- **Inside a slot being prepared,** beneath its root, never following a
  link out of it.
- **The boot partition:** this target's ESP files; on BIOS, the
  environment pair, and GRUB once at bootstrap.
- **NVRAM:** this target's four entries, `BootNext` and `BootOrder`.
- **`@hearth.state`, `/run/hearth-activator` and `/run/nextroot`;**
  and it makes `/run/hearth/` for the steward (3.2).
- **From M6:** on a backup peer, `@hearth.backups` (5.11), and on a lab
  pool host its faults' rules (5.15).
- **The running slot:** nothing, since it is sealed; it changes systemd's
  state through `systemctl`, and, for a live change, mounts read-only
  overlays over its `/etc`, `/usr/local` and `/opt` (slots.md, 6.8), which
  write nothing either.

### 2.5 What the steward and its builder never get

Secret values; identity's private halves; root; the top level; the
state. Their uid, `hearth`'s, cannot open `@hearth.state` or
`/run/hearth-activator` (both `0700`, root), and no link record carries
a secret's value or digest.

### 2.6 What the boundary does not give

- **A compromised builder or steward is root at the next trial.** They
  share a uid, so either is both (steward.md, 2.3). The steward can
  place any output the builder built, and the activator verifies only
  that the output is intact and the builder's, not that it is good;
  that output's own activator is root once tried, and its hearth can
  bless it.
- **Slot parts are unchecked in M3.** The `data`, `secrets` and `boot`
  parts arrive from the steward as typed values the activator validates
  (2.2), but nothing tells it they are what the repository says. Under
  an engine [M4], the prepare order carries the slot parts' expected
  digests, which the activator recomputes from the typed values it
  received (render.md, 4.5) and checks: that catches a builder whose
  render went wrong, not a steward that forges the order, which would
  need orders signed for the activator.
- **What the boundary does protect:** the machine's secrets and identity
  from the steward and its builder (no value on the link; no secret file
  a `hearth` group can read, 2.2), root from the IaC's code, and the
  machine from operations outside the closed list. Bad outputs are kept
  out upstream: by review, by the lab, and by outputs never crossing
  planes (temper's `factory.md`).
- **Open (section 10):** how far the activator trusts the builder's
  records (6.3; README, section 9), and whether it should place only a
  hearth binary whose digest it already holds.

## 3. The process pair

### 3.1 Started by systemd

Every output but a container's carries hearth's binary and its unit,
from the `hearth` task (format.md, 5.12); a container's activator is its
host's [M5]:

```
# /etc/systemd/system/hearth-activator.service
[Unit]
Description=hearth's activator
RequiresMountsFor=/var/lib/hearth/state /var/lib/hearth/store

[Service]
ExecStart=/usr/local/bin/hearth activator
Restart=always
RestartSec=1s
KillMode=control-group
Delegate=yes
TimeoutStopSec=60s

[Install]
WantedBy=multi-user.target
```

- **One process, one loop,** on skein. It panics to abort, and systemd
  restarts it.
- **At start** it recovers (7.4), then spawns the steward.
- **Its cgroups are delegated** (`Delegate=yes`). At start the activator
  moves itself into a leaf, `activator/`, makes `steward/` owned by
  `hearth` for the steward, its builder, the builder's tools, its tree
  helper and its sandboxes (steward.md, section 7; build.md, 8.7), and
  enables the controllers they need, `memory` and `pids`, in the unit's
  own cgroup. Bootstrap does the same in a transient scope
  (machine.md, 8.2).
- **`KillMode=control-group`:** stopping the activator stops everything
  in both, so no steward and no builder outlives the activator that
  spawned it.
- **No sandboxing,** no `PrivateMounts=` and no `Protect*=`: the
  activator must run in the host's mount namespace, or its live
  overlays would land in a namespace of its own (slots.md, 6.8).

### 3.2 Spawning the steward

| | |
|---|---|
| program | `/proc/self/exe`, argument `steward`: the activator's own binary, wherever it lives (on a live system, not in `/usr/local/bin`) |
| cgroup | `steward/` (3.1), through skein's contained process trees (README, 8.1) |
| user | `hearth`'s fixed uid and gid (format.md, 5.12; store.md, 9.2), and its supplementary groups as the running slot's `/etc/group` lists them, in practice `systemd-journal`, so the steward reads the journal for telemetry (wire.md, section 7); no capabilities. An ask of skein: spawning as another user (README, 8.1) |
| environment | given whole: `LANG=C.UTF-8`, and the store's path |
| descriptors | the link: fd 3 reads what the activator writes, fd 4 writes what it reads; stdin `/dev/null` |
| working directory | `/var/lib/hearth/store` |
| process group | its own, so the activator can signal it and its children: an ask of skein (README, 8.1) |

Before the spawn, the activator makes `/run/hearth/`, owner `hearth`,
mode 0755, where the steward binds the client's socket (client/README.md, 3.4);
its own mounts live apart, under `/run/hearth-activator/` (0700, root),
where neither the steward nor its builder can replace a directory under
them. The same pair runs at bootstrap, where the client process is the
activator (machine.md, 8.2).

The steward spawns its builder itself, for work, into `steward/builder/`,
with no supplementary group, its jobs on the builder's standard input
and output (steward.md, section 3). The activator never talks to a
builder.

### 3.3 Restarts with backoff

- **The steward exits** (a panic, a framing error on the link, an
  exit): its pending requests die with its link; an operation already
  under way runs to its end, and its outcome is in the facts. The
  activator waits, empties `steward/` with `cgroup.kill`, so no builder
  outlives its steward, then spawns a new steward on new pipes. The
  wait starts at 1 second, doubles up to 60, and returns to 1 after the
  steward has run 10 minutes.
- **A builder that exits is not the activator's:** its steward loses a
  job and goes on (steward.md, section 5).
- **The activator exits:** systemd restarts it after 1 second, its
  cgroup's steward and builder having been stopped; the new activator
  recovers (7.4), then spawns a new steward.
- **During a trial,** the slot's steward must come up to bless, once a
  builder it spawned has rendered the slot's own revision
  (steward.md, section 6). One that keeps crashing never blesses, and
  the dead-man ends the trial (slots.md, 6.5); one whose builder cannot
  render asks to fall back.
- **Once on purpose, in a live trial:** after the units are acted on
  (slots.md, 6.8, L6) the activator answers, closes the link and spawns
  a new steward with no wait, so the live bless comes from a steward
  started in the new files. A build in flight ends with its builder,
  and the next pass starts it again.

### 3.4 Shutdown

A termination signal arrives as skein's `Shutdown` event:

1. **new requests are refused,** as shutting down;
2. **the operation in flight** stops at its next step boundary: a
   prepare leaves its slot `preparing`, deleted at the next start; a
   trial whose reboot was asked for is that reboot, and its marker stays
   `armed`; a bless or a fall back finishes its last steps;
3. **the link is closed** and the steward's process group is sent
   `Terminate`, then `Kill` after 30 seconds; the steward stops its
   builder within them (steward.md, 4.3);
4. **it waits for `steward/` to be empty,** killing what is left with
   `cgroup.kill`, then exits.

`TimeoutStopSec=60s` leaves room for all of it.

## 4. The link

### 4.1 Records

Sized, in both directions: a fixed header carries the length, checked
against the kind's limit before the body is read.

```
header   12 bytes: magic "HLNK" (u32), version (u16), kind (u16), body length (u32)
body     the kind's fields, each of fixed size or length-prefixed
```

| Record | Way | Holds | Limit |
|---|---|---|---|
| `Hello` | steward to activator | version | 64 B |
| `Welcome` | activator to steward | version, epoch, facts | 64 KiB |
| `Request` | steward to activator | request id (u64), an operation (section 5) | 64 KiB |
| `Progress` | activator to steward | request id, step, done and total | 64 B |
| `Answer` | activator to steward | request id, outcome: done with its result, refused with its reason, or failed with its error | 64 KiB |
| `Facts` | activator to steward | generation, facts | 64 KiB |
| `Bye` | either | reason | 64 B |

- **`hearth-protocol-link`** encodes and decodes them. It is the only
  vocabulary the two domains share (README, 4.2).
- **Versions match by construction:** both ends are the same binary of
  the same slot. A mismatch closes the link.
- **A framing error closes the link,** and the activator respawns the
  steward after its backoff. An unknown kind with a valid length is
  answered as refused and its body skipped.

### 4.2 Requests and answers

- **Each request gets exactly one answer.** In the activator's domain a
  request arrives as a call with a `ReplyTo`, and the domain answers it
  once (`programming-model.md`, 4.4); request ids stay in the protocol
  layer. In the steward's domain each request ends in its answer, or in
  `link lost` if the link closes first.
- **Three outcomes:**
  - *done,* with the operation's result;
  - *refused,* with a reason (invalid, naming the field; stale, with the
    facts' generation; busy, naming the operation in flight): nothing
    happened, and nothing will;
  - *failed,* with a typed error: the operation ran and failed, and the
    facts describe what it left.
- **One mutating operation at a time.** A second is refused as busy. The
  steward keeps at most one mutating request and one facts request in
  flight.
- **`Progress`** reports prepare's steps, for the client's display.
- **A trial is answered before its reboot,** which then ends the link.

### 4.3 Facts

Pushed in `Welcome`, and whenever they change, each with a generation
one higher than the last:

- **the machine:** target, firmware, `SecureBoot`, the booted kernel,
  the boot id, soft reboots, firmware quirks seen;
- **the four facts:** running, default, trial (slot, attempt, kind,
  phase), and live (base, delta, the slot and prepare key it carries,
  hierarchies, whether the mount table agrees with the live record)
  (slots.md, section 3), per target: the machine's own, and on a host
  each container's [M5] (slots.md, 12.6);
- **each slot,** its containers' included: state, output, prepare key,
  reported tree digest (slots.md, 5.7; never the full one, which covers
  secrets' digests), kernel digests (slots.md, 6.1), part digests,
  generations, last trial and its outcome;
- **data:** each class subvolume present and mounted; each entry's
  directory present and bound; orphans; and [M5] per class its
  snapshots, newest and oldest, and the next one due;
- **identity:** per target, its generation and public halves;
- **secrets:** the names the spec declares, each present or missing,
  and the store's generation; never values or digests;
- **the operation in flight** and its step; drift's results by check id
  (slots.md, section 10); the last operation's costs, per step.

The steward passes them on: to the client, and to an engine [M4]
(wire.md, section 6).

**The slots' outputs are the builder's pins** (store.md, 7.1): each
slot's output key, and the output of a prepare in flight, so collection
never deletes a layer a slot holds or is being prepared from. The
steward passes them to its builder, in the jobs' `Welcome` and in `Pins`
whenever they change (steward.md, 3.2); no builder collects anything
before the steward's first `Welcome` has told it these.

### 4.4 Fencing and idempotency

- **One link per steward process.** A new steward gets new pipes and a
  new epoch in its `Welcome`; nothing an old steward wrote can reach the
  activator.
- **Conditional:** each mutating request carries what it was decided
  from (running, default, the slot's state and prepare key or record,
  the trial's attempt) and is refused as stale, with the facts, if any
  of it changed. This is jig's conditional class (jig's `connectors.md`,
  4.3), with the activator as the system.
- **Keyed:** a place is keyed by its prepare key, which the activator
  computes (slots.md, 2.2); asking again finds what the first made. A
  trial, a bless, a fall back and a roll back carry the steward's attempt
  id, and asking again with it gets the first answer.
- **So a steward that restarts mid-operation** reads the facts in its
  `Welcome` and asks only for what is still missing. An engine's orders
  get the same treatment one layer up [M4] (wire.md, 5.4).

## 5. Operations

| Operation | Asked by | Recovery class | Milestone |
|---|---|---|---|
| place (prepare) | the steward | keyed: the prepare key | M3 |
| trial | the steward | conditional | M3; through GRUB M3b |
| bless, and re-bless | the steward of the slot on trial, or of a blessed slot that is not the default | conditional | M3 |
| fall back | the steward | conditional | M3 |
| roll back | the steward, for a person or an engine | conditional | M3 |
| try live (5.17) | the steward | conditional, on the slot's prepare key, the running slot, the default and the live state; a repeat by attempt gets the first answer | M3; a container's M5 |
| bless live (5.18) | the steward running over the live state | conditional | M3 |
| the live fall back (5.19) | the steward; the activator itself when an action fails | conditional | M3 |
| report facts, and drift | the steward | read only | M3 |
| generate identity | the steward; prepare; bootstrap | idempotent | M3 |
| import secrets | the client as root, in-process | idempotent | M3 |
| hold, and release, a bless | the steward of the slot on trial, for an engine | conditional | M4 |
| run a check | the steward, for operating's checks | idempotent: a reading | M5 |
| restart a unit | the steward, for an engine | keyed by the order's key; conditional on the unit's `InvocationID` | M5 |
| place, trial, bless, hold, fall back and roll back of a container's slot, and removing a container (5.16) | the host's steward | as the machine's own: place keyed by the prepare key, the rest conditional; remove idempotent | M5 |
| partition and format | bootstrap in-process; the lab as an effect | conditional | M3; M6 as an effect |
| snapshot and prune a class (5.11) | the activator itself, on the class's period | keyed: the sequence; a prune is idempotent | M5 |
| send, receive (5.11) | the steward, on the backup's period, or for a peer | send idempotent; receive keyed by the snapshot | M6 |
| restore an entry or a class (5.12) | the steward, for a person or an engine | keyed by the order's key; conditional on the class subvolume's UUID | M5; from the peer M6 |
| apply, and remove, a fault | the steward of a lab pool host, for the lab's engine | apply keyed by the experiment; remove idempotent | M6 |

### 5.1 Place

```
Place {
  slot        a | b
  output      Key                          the output's name in the store
  tree        Digest                       its tree digest, as its record gives it
  kernel      { version, image, initramfs, modules, initramfs inputs: Digest }   slots.md, 6.1
  data        entries    [ { name, class, path } ]                                   at most 64
              policies   per class, core or data: { snapshots? { every, keep } [M5],
                         backup? { every, keep, peer, recipient } [M6] }   operating.md, section 7
  users       [ name ]                                        package users the tasks declare (format.md, 5.6), at most 256
  groups      [ name ]                                        package groups the tasks declare (format.md, 5.6), at most 256
  secrets     files       [ { name, path, mode, owner, group } ]                     at most 256
              passwords   [ { user, name } ]                                         at most 64
              identity    [ { item, kind, algorithm, path, public path, mode, owner, group } ]   at most 16
  boot        [ token ]                                       at most 2048 bytes in all
  window      Duration                                        the trial's, sealed into the units (slots.md, 5.5)
  allowed     [ unit ]                                        bless.ignore_failed (format.md, 5.13), at most 64
  meta        { kind, firmware }                              checked against the machine record
  parts       { data, secrets, boot: Digest }                 the spec's (render.md, 4.5)
  expect      { running, default, slot state }
}
```

For a container's slot [M5], `meta` is
`{ kind: container, target, on }`, `on` checked against the machine
record (2.2); `kernel` is absent, and `boot` holds the container's
parameters instead of tokens:
`{ ids, binds, capabilities, network, bridge? }`, at most 32 binds
(slots.md, 12.1).

- **Effects:** slots.md, section 5, steps 5.1 to 5.7: identity first,
  then the prepare key, which covers the window and the units allowed
  to fail.
- **`window`** is 10 minutes in M3; from M5 the steward computes it from
  the blessing checks (operating.md, section 3). It rides here, not in
  `Try`, because prepare seals it into the slot's units.
- **The part digests are the spec's,** not recomputed in M3: they name
  what the builder's render meant, for the prepare key and the facts; the
  shift's digest is the check (6.3). Under an engine [M4], an order's
  expected digests are checked against the typed values (2.6).
- **Done:** the slot, its prepare key, its reported tree digest
  (slots.md, 5.7).
- **Refused:** invalid, stale, busy, or the slot is running, the default
  or on trial.
- **Failed:** the output is missing, not read-only or not the builder's;
  the digest differs; an id is out of range; a path that must be a
  directory or a regular file is not; a secret is missing, or readable
  by `hearth`; an owner or a package user is unknown; a parent directory
  is missing; an entry's directory is orphaned in another of the
  target's classes, so the entry changed class (machine.md, 3.4); the
  disk is full; a tool failed (which, its exit, its tail). No error
  carries a file's content.
- **A crash** leaves the slot `preparing`; the next start deletes it,
  and the steward asks again.

### 5.2 Trial

```
Try { slot, attempt, how: auto | full, expect { prepare key, running, default } }
```

- **Effects:** slots.md, section 6: the activator chooses a soft or a
  one-shot reboot, writes the marker, arms, answers, reboots. The window
  is the slot's, from its place (5.1).
- **Done:** the kind chosen. The facts after the reboot tell the rest.
- **A crash** is resolved from the marker's phase (slots.md, section 11).

### 5.3 Bless

```
Bless { slot, attempt }
```

- **Effects:** slots.md, 7.3: the criterion, then the record says
  `blessed`, the marker goes, the default moves, the dead-man stops. For
  a blessed slot that runs but is not the default, the re-bless of
  slots.md, 7.5: the criterion, then the default moves.
- **Refused** while the criterion does not hold, saying which part; the
  steward asks again, or asks to fall back.
- **Moving the default where it already is writes nothing,** so a bless
  cut anywhere is finished by asking again.

### 5.4 Fall back

```
FallBack { slot, attempt }
```

- **Effects:** slots.md, section 8: before the reboot, an abort; on
  trial, a full reboot that is never a soft one.

### 5.5 Roll back

```
RollBack { to, attempt, expect { running, default, record of to } }
```

- **Effects:** slots.md, section 9: a trial of the other blessed slot,
  then its own bless. While a live state runs, it is the live fall back
  (5.19).

### 5.6 Report facts, and drift

```
Facts { drift: none | metadata | contents }
```

- **Done:** the facts (4.3), after running drift's checks at the depth
  asked (slots.md, section 10). Read only: it changes nothing but the
  results it reports.
- **The capture** (development.md, section 5) is the same walk, asked by
  `hearth diff --running`: what differs, by kind, with the contents of
  changed files under `/etc`, `/usr/local` and `/opt` on a mutable slot
  only, and never a secret's or an identity file's.

### 5.7 Generate identity

```
Identity { target, items: [ { item, kind, algorithm } ] }
```

- **Each item once per target on this machine.** An item that exists is
  answered as it is; a missing one is made in `identity/<target>/.new/`,
  then renamed into place, and the identity's generation is bumped. The
  kinds are a closed list (format.md, 9.3):

  | Kind | Made with | Public half |
  |---|---|---|
  | the machine-id, always | `/proc/sys/kernel/random/uuid`: a random v4 UUID's 32 hexadecimal digits | itself |
  | `ssh-host-key` | the running system's `ssh-keygen -q -t <algorithm> -N "" -C "" -f <.new>/<item>`: the running slot's, from the `hearth` task's `openssh-client` (format.md, 5.12), or the live system's at bootstrap, which checks for it | the `.pub` line |
  | `wireguard-key` [later] | in-process, from the kernel's random source and X25519, an ask of skein: no running slot need carry `wg` | the base64 public key |
  | `backup-key` [M6] | `age-keygen -o <.new>/<item>`: an X25519 key, kept in the identity store and written into no slot (format.md, 9.3 and 8.7) | the `age1…` recipient, reported as a fact |

- **Done:** the generation and the public halves, which the steward
  reports so that people and agents can record them as data in the
  repository (format.md, section 8).
- **A crash** leaves a `.new` directory, removed at the next start.
- **Prepare asks for it first,** before it computes its key
  (slots.md, section 5), so a new item never causes a second prepare;
  bootstrap does the same.

### 5.8 Import secrets

Run by the client as root, in its own process, with the activator's
domain (client/README.md, section 3), never over the link: no value ever passes
through the steward or its builder.

- **Input:** files named on the command line, or a directory of them,
  whose relative paths are secret names (format.md, section 9), each
  value at most 1 MiB. Identity is never imported: 5.7 makes it, fresh
  on every machine.
- **Effects:** each value whose digest is new is written as
  `secrets/<target>/<name>@<generation>`, `0600`, through a temporary
  file, synced and renamed; then the index (the store's generation, and
  each name's value file, digest and size) is written with the
  generation bumped once for the command, conditional on the digest of
  the index it read (skein's `Store`), so two imports never lose each
  other's writes. The previous generation's files stay until the next
  import, since a running activator may still be reading through the
  previous index; that import removes what neither index names.
  Removing a secret is an index without its name.
- **Idempotent:** an import that changes nothing bumps nothing.
- **Readers read the index, then the files it names,** so they always
  see one consistent generation. The running activator reads the index
  before every prepare and every facts report; the client then asks the
  steward to refresh its facts (client/README.md, section 3).
- **A crash** leaves value files no index names, removed by the next
  import.

### 5.9 Restart a unit [M5]

```
RestartUnit { unit, invocation }
```

- **Keyed by the order's key,** which the steward keeps with the unit's
  `InvocationID` before it asks (wire.md, 5.4), and **conditional on
  that invocation** at the activator: `systemctl show
  --property=InvocationID --value <unit>` must still equal it, then
  `systemctl restart <unit>`. Done when the invocation has changed, so
  an uncertain restart is resolved by reading it again.
- **Never hearth's own units** (`hearth-activator.service`,
  `hearth-trial*`, `hearth-check-*`); that the unit is one the running
  slot's spec ships or enables is the steward's check
  (operating.md, section 6).
- **For a container's target [M5],** the same operation with the
  container's target named, `RestartUnit { target, unit, invocation }`:
  the unit is one inside the container, its `InvocationID` read with
  `systemctl --machine=<container> show --property=InvocationID --value
  <unit>`, then `systemctl --machine=<container> restart <unit>`. The
  container's own nspawn unit is never named: only the host's activator
  starts and stops it (5.16).

### 5.10 Partition and format

- **M3: bootstrap only,** run in-process by the client as root
  (machine.md, 8.2): wipe, partition, format, make the top level.
- **M6: an effect for the lab** (machine.md, 8.5):

  ```
  Disk { device id, firmware, fs uuid, boot volume id }
  ```

  conditional on a blank disk (no partition table, no signature), or on
  one that already holds exactly these identifiers, which is then done.

### 5.11 Snapshots [M5] and backups [M6]

```
Snapshot { target, class, seq }      the activator's own, on the class's period
Send { target, class, seq, parent }      Receive { from, class, seq }      [M6]
```

- **Snapshot,** by the activator itself: when a class's deadline comes
  (operating.md, 7.1), a read-only snapshot of `@<target>.<class>` into
  `@hearth.snapshots/<target>.<class>/<seq>` (`BTRFS_IOC_SNAP_CREATE_V2`
  with the read-only flag), atomic and crash-consistent. The policy is
  the running slot's record's (slots.md, 2.2), never a request's; a
  container's class follows its host's (slots.md, 12.1). Keyed by
  `<seq>`, which only grows: a snapshot of that sequence means done.
  Then the class's record (7.1) gains it, and the next deadline is set.
- **Prune,** after every snapshot: each snapshot that no bucket keeps
  and the backup does not need (operating.md, 7.3) is deleted with
  `SNAP_DESTROY_V2`, then dropped from the record. Idempotent: a
  snapshot already gone is done. No request can prune: the steward
  never decides what a machine's history loses.
- **Send** [M6]: `btrfs send -p <parent> <seq>`, or without `-p` the
  first time, piped into `age --encrypt --recipient <recipient>` before
  any byte reaches the steward, then handed to it over a pipe made for
  the transfer, never the link. The recipient is the peer's backup key,
  from the class's policy in the sending slot's record (slots.md, 2.2),
  never from the request, so the steward cannot redirect a stream to a
  key of its own. Idempotent: a send the peer has not confirmed is sent
  again. The semantics are operating.md, 7.2's.
- **Receive** [M6], on the backup peer: the ciphertext from its steward,
  piped into `age --decrypt -i <its backup key>` and then `btrfs
  receive` into `@hearth.backups/<from>.<class>/`, accepted only from a
  target the peer's spec names; what the peer keeps at rest is
  operating.md, 7.3's. Keyed by the received snapshot: one already
  there, read-only, is confirmed again; a partial one is deleted and
  received again.

### 5.12 Restore [M5]

```
Restore { target, class, entry?, seq, expect { uuid } }
```

- **Keyed by the order's key and conditional on the class subvolume's
  UUID** being the one decided from. With `entry`, one entry's
  directory is copied back out of the snapshot and swapped in by name;
  without, the class's subvolume is swapped for a writable snapshot of
  it. Both then reboot, or restart a container. The steps are
  operating.md, 7.4's; the names, machine.md, 3.1 and 3.4's.
- **The copy is the activator's own walk** through descriptors, with
  `FICLONE` (slots.md, 5.3): no `cp`, and nothing resolved outside the
  snapshot and the class.
- **A snapshot the machine lacks** [M6] arrives from the peer encrypted
  to this target's own backup key; the activator pipes `age --decrypt -i
  <its backup key>` into `btrfs receive`, into its own
  `@hearth.snapshots`, so plaintext exists only between two activators.
- **A cut between the renames** is finished or undone at the next start
  from the recorded swap (7.3).

### 5.13 Hold, and release, a bless [M4]

```
HoldBless { slot, attempt, until }        Bless { slot, attempt }        releases it
```

- **Conditional** on the slot running on trial, with that attempt, and
  the minimal criterion holding. What a hold does (the marker `held`, a
  transient timer at `until`, the default unmoved) and how it ends are
  slots.md, 7.4's; a `Bless` releases it as in 5.3.
- **A crash** finds the marker `held` and its transient timer still
  armed: nothing to redo.
- **A container's [M5]** is held the same way, with no timer: its end is
  the activator's own deadline, as a container's window is (5.16).

### 5.14 Run a check [M5]

```
RunCheck { task, name, user, capabilities, network, timeout, argv }
```

- **systemd runs it, not the activator:** a transient unit,
  `hearth-check-<task>-<name>`, through `systemd-run --wait --collect`
  with the fixed properties of operating.md, 2.4, as the declared user
  (`nobody` by default, never `root`), the file system read-only, no
  network unless declared, and memory, tasks and time bounded. The
  activator builds the argument vector from the typed check and renders
  nothing.
- **Secrets stay out of reach:** `InaccessiblePaths=` names
  `/var/lib/hearth/state`, `/run/hearth-activator` and every secret and
  identity path the running slot's record holds, so a check cannot read
  them even as a user their owners would allow.
- **The program is the running slot's,** a system the machine already
  runs, never a program of a slot being prepared or of the store.
- **In a container [M5],** the same unit is started inside it with
  `systemd-run --machine=<container>`, its `InaccessiblePaths=` the
  container slot's secret and identity paths (operating.md, 2.9).
- **Done:** `pass` (exit 0), `fail` (another status, or the timeout) or
  `error` (the unit could not start), with the unit's `Result`, and
  nothing else: no output reaches the steward, which could otherwise
  read a file through a check.
- **Idempotent:** checks only read until operating.md, 2.8 says
  otherwise, so an uncertain run is run again.

### 5.15 Faults [M6]

```
Fault { experiment, kind, targets, parameters, until }        Unfault { experiment }
```

- **Only on a lab pool host,** as the target's own spec says: a target
  in the lab's environment with containers placed on it, whatever an
  engine says; refused everywhere else (operating.md, section 8).
- **Kinds, each a fixed operation with typed parameters,** never a
  command line from the engine; their mechanisms (nftables, `tc netem`,
  `systemctl --machine kill`, a qgroup limit) are
  operating.md, section 8's.
- **Apply is keyed by the experiment:** a fault already applied under
  that key is found. **Remove is idempotent.**
- **Every fault ends at `until`,** which the activator keeps and
  enforces itself, after a restart too, so a fault ends even when the
  engine is gone.

### 5.16 A container's slots [M5]

Place, trial, bless, hold, fall back and roll back (5.1 to 5.5, 5.13) on
a container's slot, and removing a container, asked by the host's
steward with the container's target (2.2). The effects are
slots.md, section 12's; none touches the boot partition or NVRAM.

| Operation | Effects | Recovery class |
|---|---|---|
| place | section 5's steps with a container's differences (slots.md, 12.1) | keyed: the prepare key |
| trial | the marker; the slot's nspawn settings and link, each by rename; `systemctl restart systemd-nspawn@<container>.service`; the poll (slots.md, 12.2 and 12.3) | conditional on the container's running slot, default and the slot's prepare key; a repeat by attempt gets the first answer |
| bless | the criterion, read from the host; the record `blessed`; the marker removed; the default recorded (7.1) | conditional |
| hold, and release | the marker `held`, with the hold's end as the activator's deadline in place of the window's; a `Bless` releases it (slots.md, 7.4) | conditional |
| fall back | the default's settings and link back, a restart, the record `fell back` | conditional |
| roll back | a trial of the other blessed slot; while live, the live fall back | conditional |
| try live, bless live, the live fall back | slots.md, 12.8: the delta on the host's top level, the swap by a child that enters the container's mount namespace, the units through `systemctl --machine=`, the live bless recording the container's next start as `b`'s trial | as the machine's own |
| remove | `systemctl stop` on its unit; its link and settings removed; its slots' subvolumes, records, manifests and marker deleted, then its record; its class subvolumes, their snapshots, its identity and secrets kept (slots.md, 12.7) | idempotent: a container with no record is done |

- **A trial is answered after its restart,** not before a reboot: done
  once the unit is stably active, the slot `on trial`; failed once it is
  not, the container back on its default.
- **The window is the activator's own deadline,** kept in the marker:
  at its end with no bless, the activator falls the container back
  itself, after its own restart too, since no dead-man runs in a
  container. A held bless's ceiling is kept and enforced the same way.
- **Remove names a container the host's spec no longer places on it;**
  the steward checks that against its builder's render, as it checks a
  unit against the spec (5.9), and the activator refuses the host's own
  target.

### 5.17 Try live

```
TryLive {
  slot, attempt
  change      Digest                         the change from the base's output to the new slot's (slots.md, 6.7), in store.md, 3.5's form
  plan        stop                [ unit ]   before the swap
              reload              [ unit ]
              restart             [ unit ]
              reload or restart   [ unit ]
              start               [ unit ]
              then                { daemon reload, tmpfiles, dbus, accounts: bool }
  expect      { prepare key, running, default, live: delta | none }
}
```

- **Effects:** slots.md, 6.8, L1 to L6: the change recomputed and
  checked (6.5), the delta built and verified, the base's dead-man
  armed, the units stopped, the hierarchies swapped, the units acted on.
  The rule-11 units are added to the restarts.
- **The plan is a builder's,** computed in a `plan live` job the steward
  asked (steward.md, 3.3); the steward sends it, and the activator
  checks again what it can read (6.5).
- **Done,** after L6: the plan as acted on and each job's result; then
  the activator spawns a new steward (3.3), whose `Welcome` shows the
  slot `on live trial`.
- **Refused:** invalid, stale, busy; not eligible, naming the rules of
  slots.md, 6.7 that fail, and the steward then asks `Try`.
- **Failed:** at L1 or L2, the delta deleted and nothing on the machine
  changed; at L3 to L5, the stopped units started again and the trial
  disarmed, the slot `prepared`; at L6, a job failed and the live fall
  back ran (5.19), the answer saying whether it held or ended in
  `reboot -ff`.
- **A crash** is resolved from the journal and the marker's phase
  (slots.md, section 11): before arming, the delta deleted; after,
  `reboot -ff`.

### 5.18 Bless live

```
BlessLive { slot, attempt }
```

- **Effects:** slots.md, 7.6: the checks, the record `live-blessed`, the
  one-shot to the slot's trial entry, the marker's phase, the base's
  dead-man stopped. The default does not move.
- **Refused** while the settle, a unit or the criterion does not hold,
  saying which part; the steward asks again, or asks to fall back.
- **Asking again** with the attempt finds it done; a cut is finished at
  start from the journal.

### 5.19 The live fall back

```
FallBack { slot, attempt }        on a slot on live trial, or live-blessed
```

- **Effects:** slots.md, 8.5: the units L6 started stopped, the swap
  back, the actions replayed under the old files, every job checked,
  then the dead-man stopped and the slot `fell back`. For a
  `live-blessed` slot, its one-shot is cleared and the base's dead-man
  armed again first.
- **Run by the activator itself** inside `TryLive` when an action of L6
  fails, and asked by the steward otherwise; a roll back while live is
  this (5.5).
- **Any failed step is `reboot -ff`,** and the next start records the
  outcome. Done once the files that ran before run again and every job
  held.

## 6. Verification

What the activator checks before a place request becomes a slot.

### 6.1 The record and the layer

1. **The record:** read `layers/<key>.record` beneath the store
   (store.md, 6.2), as the builder wrote it, decoded by
   `hearth-protocol-tree`'s record machine. It must decode, name the
   request's key, and give the request's tree digest; the kernel's
   digests come from the activator's own walk (6.3).
2. **The layer:** open `layers/<key>` beneath
   `/run/hearth-activator/top/@hearth.store/` (store.md, 2.1) with
   `openat2(RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_XDEV)`. It must be a subvolume's root (inode 256); `GET_SUBVOL_INFO`
   must show the read-only flag; `statx` must show a directory, `0755`,
   owner and group `B`, the base of the builder's range (store.md, 2.2).
   Its uuid and change generation are noted.
3. **In the store, by construction:** the activator resolved the path
   itself, beneath the store's subvolume, from a name that is a digest.

### 6.2 Snapshot first, then verify the snapshot

- **The snapshot is taken from the very descriptor checked in 6.1:**
  `BTRFS_IOC_SNAP_CREATE_V2` takes the source as a descriptor, so no name
  is resolved twice.
- **The snapshot's parent uuid is the layer's,** by `GET_SUBVOL_INFO`.
- **The layer is checked again:** still read-only, with the same change
  generation, so nothing changed it between the check and the snapshot.
  Whether flipping the read-only flag moves the change generation is to
  verify; 6.3 catches any change anyway.
- **From here the builder cannot touch the slot:** it lives at the top
  level, which only root reaches, through `/run/hearth-activator` (`0700`).

### 6.3 The tree digest, from the shift's walk

- **The shift's walk computes the digest of the shifted tree**
  (slots.md, 5.2), as the builder's tree helper computed the output's
  inside its namespace: the same entries, in the same order, in the same
  encoding (store.md, 3.4), because the shift puts every id where the
  namespace saw it.
- **Equal to the record's:** the slot holds exactly the output.
  **Different:** the place fails as a digest mismatch, the slot is
  deleted, and the facts report a store that lies or has rotted.
- **The kernel is checked the same way:** the digests of
  `boot/vmlinuz-<version>` and `boot/initrd.img-<version>` (the latter to
  check its ESP copy), and of the subtree `usr/lib/modules/<version>`,
  from the same walk. The initramfs inputs' digest (slots.md, 6.1) comes
  from the request, as the part digests do: its inputs are package
  versions the walk cannot see.
- **Contents are hashed again in M3:** the walk reads every byte of every
  regular file, once per prepare: hundreds of MB to a few GB, at the speed
  of skein's SHA-256. The cost is recorded per prepare.
- **Open (README, section 9):** trusting the builder's per-file content
  digests where the builder runs only landed code. The walk would then
  check metadata, take contents from the record by inode, and lean on
  6.2's identity checks.

### 6.4 Refused outright

- an owner, group or ACL id outside `[B, B + 65536)`, or a v3 capability
  with another root id;
- a block or character device; anything but a directory, a regular file
  or a link beneath `/var`;
- an output whose `/var`, `/etc`, `/boot`, `usr/lib/modules/<version>`
  or a mount point's parents are not directories, or whose kernel,
  initramfs, `/etc/passwd`, `/etc/group` or `/etc/shadow` are not regular
  files (slots.md, section 5);
- `/.hearth`, any of the trial's unit names, or a drop-in sorting after
  `zz-hearth-watchdog.conf` (slots.md, 5.5);
- a kernel version without its `boot/vmlinuz-`, `boot/initrd.img-` or
  `usr/lib/modules/` entry;
- a target or a firmware that is not the machine record's; [M5] a
  container whose `on` is not the machine record's target, or whose id
  range overlaps the builder's or another container's (2.2).

### 6.5 The merged view

What the activator checks before a live trial arms (slots.md, 6.8, L1
and L2):

- **The change is its own.** It diffs the two slots' manifests itself,
  leaving out each slot's own files (slots.md, 5.4 and 5.5), and the
  digest must equal the request's. The delta is built from its own
  diff, never from paths the steward sent.
- **The rules it can read:** slots.md, 6.7's rules 1 to 5 from the
  layer records, the slot records and the two slots' account files,
  through the parsers of 2.3; rule 8 for the plan's units; rule 10 from
  the kernel's release and the mount table; rule 11 from
  `/proc/<pid>/mountinfo` and `/proc/<pid>/cgroup` of every process.
  Rules 6, 7 and 9 read the specs and the store, and stay the
  builder's, in the steward's `plan live` job (steward.md, 3.3).
- **The delta is built through descriptors:** `b`'s paths resolved
  beneath `b`'s root with `openat2` and `RESOLVE_BENEATH`,
  `RESOLVE_NO_SYMLINKS` and `RESOLVE_NO_XDEV`, the delta's beneath the
  delta's; nothing made but directories, regular files, links and the
  whiteouts.
- **The merged view is walked** through the detached overlays'
  descriptors, before anything is mounted where a process sees it, and
  must equal the expected manifest field by field (slots.md, 6.8, L2).
  A difference fails the trial with the first paths, never a content.
- **The mount table is read back** after every swap and at every start,
  by `hearth-protocol-proc` (8.1): what runs is what the kernel says,
  not what the activator meant.

## 7. Its state

### 7.1 Layout

```
/var/lib/hearth/state                       @hearth.state: root, 0700
├── machine                                 target, firmware, fs uuid, boot partition's volume id,
│                                           the ESP's GPT GUID, entry numbers, layout version; mutable:
│                                           accept or refuse; [M3b] a retired target (development.md, 3.2)
├── boot                                    the booted kernel: boot id, its digests (slots.md, 6.1)
├── journal                                 the operation in flight, and its step (7.3)
├── facts                                   quirks seen, drift's last results, last costs
├── snapshots/ [M5]
│   └── x1-prod.data                        the class's snapshots: each sequence, its time, the buckets that keep it,
│                                           and [M6] sent, confirmed by the peer; the next deadline
├── slots/
│   ├── x1-prod.a                           slot record (slots.md, 2.2)
│   ├── x1-prod.a.manifest                  one entry per path of the sealed root
│   ├── x1-prod.b
│   └── x1-prod.b.manifest
├── trial/
│   └── x1-prod.b                           the trial marker, only while a trial runs (slots.md, 6.4);
│                                           the dead-man is armed from /run and the command line instead
├── live/
│   ├── x1-prod                             the live record, while a live state exists (slots.md, 2.4)
│   └── x1-prod.manifest                    the merged view's expected entries
├── identity/
│   └── x1-prod/                            generation, machine-id, SSH host keys, the backup key [M6], public halves
├── secrets/
│   └── x1-prod/
│       ├── index                           the store's generation; each name's value file, digest, size
│       └── <name>@<generation>             one value per file, 0600
└── containers/ [M5]
    └── chat-prod                           on rhi-prod: the container's default slot, its id base,
                                            and a live-blessed slot's "next start is its trial" (slots.md, 12.8)
```

- **A host keeps its containers' records beside its own [M5],** each by
  the container's target: `slots/chat-prod.a`, `trial/chat-prod.b`,
  `live/chat-prod`, `identity/chat-prod/`, `secrets/chat-prod/`. The
  default has a record of its own, since no firmware holds it
  (slots.md, 12.6).

### 7.2 Records

- **The filesystem is the only state** (README, section 7): subvolumes,
  and these records beside them, read back at every start. No kv store.
- **Each record is sized,** a header with its kind and format version
  then its fields, encoded by `hearth-protocol-state` (8.1). The domain
  never sees their bytes.
- **Each is written with skein's `Store`:** a temporary file, synced,
  renamed over the old one, the directory synced; conditional on the old
  one's digest where two writers can meet (the secret index).
- **Formats are read back across releases.** An activator reads every
  format version it ever wrote and refuses a newer one, reporting it. A
  newer activator writes no record in a format the older one cannot read
  while the older one's slot is still blessed, so a roll back always
  finds its state readable.
- **Bootstrap writes as the release it places.** It refuses to run when
  its own binary's digest differs from the one the output pins in its
  `hearth` task (machine.md, 8.1), since the records it writes are read
  next by that release.

### 7.3 The journal

One record: the operation in flight, its key or attempt, its slot, its
step and when it started. It is written before each step whose effects
leave the state (a subvolume, the boot partition, NVRAM, a mount, a
reboot), and cleared when the operation ends.

| Operation in the journal at start | Recovery |
|---|---|
| place | the slot is `preparing`: delete it, nested subvolumes first (slots.md, section 11) |
| trial, roll back | by the marker's phase and the facts (slots.md, section 4) |
| bless | the record says `blessed`: remove the marker, move the default (re-bless), stop the dead-man; otherwise the slot stays on trial |
| fall back | by the marker and the facts |
| try live | at L1 or L2: the delta deleted, the slot `prepared`; from L3, by the marker's phase: `arming` disarmed, later `reboot -ff` (slots.md, section 11) |
| bless live | the record says `live-blessed`: set the one-shot, set the marker's phase, stop the base's dead-man; otherwise the slot stays on live trial |
| the live fall back | `reboot -ff`, unless its last step was done |
| generate identity | remove `.new` |
| hold a bless [M4] | the marker `held` and its transient timer: nothing to redo |
| run a check [M5] | the transient unit, collected by systemd: run it again |
| restart a unit [M5] | read the invocation id: changed means done |
| a container's trial, bless or fall back [M5] | by the marker and the facts, with the link and settings in `/run` (slots.md, 12.3) |
| partition and format [M6] | look for the identifiers: found means done |
| snapshot, prune [M5] | look for the snapshot by its sequence: found means done; prune again |
| send, receive [M6] | look for the received snapshot: found means done; a send not confirmed is sent again |
| restore [M5] | before the swap is recorded, delete the copy or the writable snapshot (`.restoring`); after, the recorded swap: finish it, or undo it, then reboot |
| a fault [M6] | look for it by the experiment's key; its end still holds |

### 7.4 Start

At every start, after a boot, a soft reboot or a crash:

1. **mount the top level** at `/run/hearth-activator/top`, unless it is mounted;
   find the disk and the boot partition from
   `/sys/fs/btrfs/<fs uuid>/devices/`; then the live layers' source,
   `/run/hearth-activator/layers`: a clone of the top level, private,
   read-only, without `noexec` and `nosuid` (slots.md, 6.8);
2. **read** the machine record, the slot records, the marker, the
   journal and the boot record;
3. **read the facts:** running, default, the one-shot, `BootCurrent`,
   the boot id, soft reboots, `SecureBoot`, and the live fact from the
   mount table against the live record (slots.md, section 3);
4. **on a new boot id,** record the booted kernel: the running slot's
   kernel digests (slots.md, 6.1);
5. **recover the journal's operation** (7.3);
6. **apply the start events** to both slots (slots.md, section 4): a
   live trial past arming in this boot, or a live record whose base
   still runs but whose overlays disagree with it (a person's soft
   reboot), ends here in `reboot -ff`, before anything else; a live
   record whose base no longer runs, or whose overlays a reboot
   dropped, is removed;
7. **repair what a firmware or a cut left:** an environment pair that
   disagrees (machine.md, 5.5); hearth's entries out of order in
   `BootOrder`; a normal entry of a blessed slot that the firmware
   pruned, made again; the dead-man of a running slot that is blessed,
   stopped; on BIOS, `tries` reset when the running default meets the
   criterion (slots.md, 8.4); each delta subvolume that neither the live
   record nor the marker names, deleted; after an adoption [M3b], the
   retired target's slots, entries, ESP files and records, deleted
   (development.md, 3.2);
8. **start the containers** [M5]: each container's settings and link
   that `/run` lacks, from its recorded default, then `systemctl start`
   on its unit (slots.md, 12.2); and set each class's snapshot deadline
   from its record and the running slot's policies, a snapshot missed
   while the machine was down taken at once (operating.md, 7.1) [M5];
9. **spawn the steward;** its `Welcome` carries the facts.

Every step is idempotent, so a crash during the start repeats it.

## 8. The tools it drives

### 8.1 Formats, each its own machine

Every format the activator reads or writes is a lib-only protocol
machine crate, fuzzed alone (`programming-model.md`, section 4); the
domain sees only typed values.

| Crate | Format | Way |
|---|---|---|
| `hearth-protocol-link` | the link's records (4.1), `TryLive`'s plan included (5.17) | both |
| `hearth-protocol-state` (new) | the records of `@hearth.state` (7.1), the trial marker, the live record and its manifest (slots.md, 2.4), the classes' snapshot records [M5], the slot record inside the root (slots.md, 2.2) | both |
| `hearth-protocol-accounts` (new) | `/etc/passwd`, `/etc/group`, `/etc/shadow` lines | read; `/etc/shadow` written |
| `hearth-protocol-units` (new) | fstab (machine.md, 3.3), the trial's units and the watchdog's drop-in (slots.md, 5.5); a container's nspawn settings [M5] (slots.md, 12.1) | written |
| `hearth-protocol-xattr` (new) | `security.capability` v2 and v3, POSIX ACL values (slots.md, 5.2) | both |
| `hearth-protocol-tree` | the store's layer records and tree files, read; the slot manifest, a tree file (store.md, 6.2 and 6.3) | both |
| `hearth-protocol-efi` | efivarfs variables and `EFI_LOAD_OPTION` (machine.md, 4.5); efibootmgr's argument vectors | both |
| `hearth-protocol-grub` [M3b] | `grub.cfg`, a slot's fragment, the environment block (machine.md, section 5); grub-editenv's and grub-install's argument vectors | both |
| `hearth-protocol-systemd` | systemctl's and systemd-run's lines: states, counts, ids, failed units, `Result`; a job's outcome, and `ActiveState`, `InvocationID`, `NRestarts` and `MainPID` per unit (slots.md, 6.8) | read |
| `hearth-protocol-proc` (new) | `/proc/<pid>/mountinfo` lines: mount ids, parents, mount points, propagation, fstype, source and options, an overlay's layers included; `/proc/<pid>/cgroup`, a process's unit | read |
| `hearth-protocol-keygen` | ssh-keygen's argument vector and public line | both |
| `hearth-protocol-dpkg` | dpkg's status, for capturing a mutable machine's packages (development.md, section 5) | read |

`hearth-protocol-activator` is the protocol layer: it routes these
machines, and holds only the argument vectors of tools whose answer is
an exit status (mount, parted, the mkfs tools and the rest of 8.2).

### 8.2 Tools

| Tool | For | Protocol | What the domain gets |
|---|---|---|---|
| btrfs ioctls: `SNAP_CREATE_V2`, `SUBVOL_CREATE_V2`, `SNAP_DESTROY_V2`, `SUBVOL_SETFLAGS`, `GET_SUBVOL_INFO`, `GET_SUBVOL_ROOTREF`; `FICLONE` | snapshots, slots, sealing, the running slot; cloning `/var`, seeding entries and restoring one; classes, their snapshots and pruning [M5] | skein's ring adapter: a closed list of synchronous ioctls (an ask of skein, README, 8.1) | typed results |
| files: `openat2` with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS` and `RESOLVE_NO_XDEV`; stat without following, readlink, extended attributes, chown, chmod, mkdir, symlink, link, unlink, rmdir, `Store` | the shift, `/var`, the slot's files, the state | skein's io, with the records hearth asks for (README, 8.1) | typed results |
| mounts: `fsopen`, `fsconfig`, `fsmount`, `move_mount` with `MOVE_MOUNT_BENEATH`, `open_tree`, `mount_setattr`, `umount2` with `MNT_DETACH`; `mknodat` of a 0/0 character device; `setns` into a container's mount namespace [M5], in a child | live overlays: the layers' source, the swap and the swap back, the whiteouts (slots.md, 6.8 and 12.8) | skein's io, asks of skein (README, 8.1) | typed results; the mount table through `hearth-protocol-proc` |
| efivarfs reads | entries, `BootOrder`, `BootNext`, `BootCurrent`, `SecureBoot` | `hearth-protocol-efi` | typed variables |
| `/usr/bin/efibootmgr` | create, delete, `--bootnext`, `--delete-bootnext`, `--bootorder` | `hearth-protocol-efi` | success, or exit and tail |
| `/usr/bin/grub-editenv` [M3b] | `set` and `unset` in `env.0`, then `env.1` | `hearth-protocol-grub` | success, or exit and tail |
| `/usr/sbin/grub-install` [M3b] | bootstrap, once | `hearth-protocol-grub` | success, or exit and tail |
| `/usr/bin/mount`, `/usr/bin/umount` | the top level, the boot partition, `/run/nextroot` | `hearth-protocol-activator` | success, or exit and tail |
| `/usr/bin/systemctl` | `soft-reboot`, `reboot`, `is-system-running`, `list-units --state=failed`, `show`, `stop`; for a live trial, `start` and `stop` of the base's dead-man, `daemon-reload`, `reload`, `restart`, `reload-or-restart`, `start` and `stop` of the plan's units (slots.md, 6.8); `restart` [M5]; for a container [M5], `start`, `restart` and `is-active` of `systemd-nspawn@<container>.service`, and `is-system-running` and `list-units` with `--machine=<container>`; `--machine=<container> kill` [M6] | `hearth-protocol-systemd` | a state, a count, an id, a list of units |
| `/usr/bin/systemd-run` | the hold's timer [M4]; checks as transient units, `--machine=<container>` for a container's [M5] | `hearth-protocol-systemd` | the unit's exit status and `Result`, nothing of its output |
| `/usr/bin/systemd-tmpfiles` | `--create /etc/tmpfiles.d/hearth.conf` in a live trial (slots.md, 6.8) | `hearth-protocol-activator` | success, or exit and tail |
| `/usr/sbin/nscd` | `--invalidate passwd` and `group`, in a live trial, when nscd runs | `hearth-protocol-activator` | success, or exit and tail |
| `/usr/bin/ssh-keygen` | identity's host keys | `hearth-protocol-keygen` | success; the keys as files |
| `/usr/sbin/wipefs`, `/usr/sbin/parted`, `/usr/sbin/partprobe`, `/usr/bin/udevadm`, `/usr/sbin/mkfs.vfat`, `/usr/sbin/mkfs.btrfs`, `/usr/sbin/useradd`, `/usr/sbin/usermod` | bootstrap; disks as an effect [M6] | `hearth-protocol-activator` | success, or exit and tail |
| `/usr/bin/btrfs` `send`, `receive`, `qgroup limit` [M6] | backups, and a restore from the peer; a full-disk fault | `hearth-protocol-activator` | a stream, and success |
| `/usr/bin/age`, `/usr/bin/age-keygen` [M6] | encrypting a backup stream, decrypting one received; the backup key | `hearth-protocol-activator`; the recipient line, `hearth-protocol-keygen` | a stream, and success; the `age1…` recipient |
| `/usr/sbin/nft`, `/usr/sbin/tc` [M6] | partition, latency and loss faults on a pool host | `hearth-protocol-activator` | success, or exit and tail |

- **Spawned with an absolute path,** an argument vector and an
  environment given whole (skein spawns with no `PATH` search):
  `LANG=C.UTF-8`, `PATH=/usr/bin:/usr/sbin` for tools that run helpers,
  and `SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1` for every reboot it asks for
  (slots.md, 8.1). stdin is `/dev/null`; stdout and stderr are kept to
  their last 4 KiB, which never carry a slot's file.
- **Each with a deadline,** after which it is killed: 30 seconds for
  efibootmgr, grub-editenv, mount and systemctl; minutes for mkfs and
  grub-install.
- **No tool walks a slot.** `cp` and `rm` would resolve paths in the
  activator's own namespace; the activator copies and empties through
  descriptors itself (slots.md, 5.3).
- **Not the `btrfs` tool** for subvolumes: its `list` and `show` need a
  tree search, and its text would be one more protocol for what the
  ioctls give typed.
- **Each tool has a fake** on the fake machine with the same face
  (testing.md, section 4).

## 9. Its world

- **The root's world:** `hearth-domain-activator` with its children
  (`hearth-domain-slots`, `hearth-domain-boot`, `hearth-domain-disk`;
  `hearth-domain-backups` from M5) on the fake machine and the fakes of
  section 8, with a scripted steward on the link side: requests at every
  moment, stale conditions, repeated attempts, a steward that crashes,
  hangs or never blesses (testing.md, sections 3 and 8).
- **Faults from the seed:** the activator's crash at any step, power
  cuts, tools that fail or time out, firmware quirks, a store whose
  record lies, a layer swapped between check and snapshot.
- **Invariants,** checked after every event:
  - slots.md, section 4's;
  - exactly one answer per request, and a refusal changes nothing;
  - no link record holds a secret's value, nor a digest of one: the
    world marks every secret byte and scans every record, the reported
    tree digest included;
  - an output that plants a link at `/var`, `/etc`, a kernel's path or
    an account file never leads a prepare outside its snapshot;
  - every spawned program is in section 8's list, with an absolute path,
    and none comes from a slot or the store;
  - no path from a request reaches io without passing 2.2's rules;
  - a crash at any step recovers to a state the table allows.
- **The formats:** each crate of 8.1 has machine worlds and a fuzz
  target: cuts at every byte, oversized lengths, unknown kinds,
  truncated bodies; the account parsers also take hostile lines from an
  output.
- **The pair:** a system world runs the steward's and the builder's
  domains and the activator's together, through the link's and the
  jobs' vocabularies, from a desired revision to a blessed slot,
  crashing any of the three at any point (testing.md, section 8;
  steward.md, section 8).
- **The VM tier** runs the real pair on an Incus VM: bootstrap, a
  prepare, both kinds of trial, a bless, a dead-man fall back
  (testing.md, section 7).
- **Live changes** (M3): the fake machine's mount namespaces, overlays
  and PID 1's jobs (testing.md, 3.13), with a scripted steward that
  asks `TryLive` with plans that lie (a change digest that differs, a
  never-live unit), and faults at every phase: a job that fails, a unit
  that dies in the settle, a crash or a power cut from L1 to L9, a
  person's soft reboot. Invariants: slots.md, section 4's, the live ones
  included; no swap before L2's walk passed; every swap read back; the
  activator never leaves the host's mount namespace.
- **Later operations join with their milestones:** a held bless that
  ends at its deadline [M4]; checks whose transient units fail, time out
  or never start, and a host's containers prepared, tried, blessed and
  fallen back across crashes and host reboots, snapshots and prunes cut
  at every step, and a restore of an entry or a class cut between its
  renames [M5]; sends and receives cut mid-stream, and faults that
  outlive their engine [M6].

## 10. What is open

- **Hashing contents again at every prepare,** or trusting the builder's
  records where the builder runs only landed code; depends on the
  measured cost and on how the lab's builders differ from production's
  (README, section 9).
- **Placing only known hearth binaries:** whether the activator should
  refuse an output whose `/usr/local/bin/hearth` has a digest it does not
  already hold (its own, or one a person or the engine vouched for),
  since that binary is root at the next trial (2.6); depends on how
  hearth releases are vetted and distributed.
- **Signed orders [M4]:** whether an engine's prepare order should be
  signed for the activator, so a forged order's slot-part digests fail
  (2.6); depends on wire.md, 5.3's machine identity.
- **Resolving owners by parsing the slot's account files;** it could go
  if build pins the ids of every user a secret or data entry names
  (build.md, format.md).
- **The change generation and the read-only flag:** whether flipping the
  flag moves it (to verify), which decides whether 6.2's second check
  alone shows an unchanged layer.
- **Formats across releases:** how many versions back an activator reads
  its state; depends on how long a previous slot may stay blessed.
- **The steward's diagnostics:** whether its traces reach the journal
  through its own standard error, or through the activator; depends on
  skein's spawn. The builder's standard error is the steward's
  (steward.md, 3.1 and section 10).
- **One generation for the whole secret store:** any import, even of an
  unrelated secret, prepares every target's slot again. One generation
  per target, or per secret a slot names, would spare the others;
  depends on how often secrets rotate (wire.md, 4.6).
- **Telling the running activator of an import:** it listens to nothing,
  so the client asks the steward to refresh the facts; whether that is
  enough depends on client/README.md, section 3.
- **The shift's walk in the activator's loop:** skein's file driver runs
  one request at a time, so a long walk shares the loop with the link;
  whether facts stay timely during a prepare, or the walk moves to a
  child, depends on measurement. A live trial's L2 walk asks the same.
- **The child that enters a container's mount namespace [M5]:** the
  activator's own binary spawned with a fixed argument, as the steward
  is, or skein spawning straight into a namespace; depends on what
  skein's contained process trees offer (README, 8.1).
