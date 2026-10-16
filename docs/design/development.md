# Development

Provisional, 2026-10-09, revised 2026-10-16. How an agent, or a
person, develops and changes a machine or a container before anything
lands: a cheap
machine of its own, a root it may change by hand, a capture of what it
changed, and its working tree applied to the machine as a revision. M3
builds mutable machines, the capture and applying a working tree; M3b
the Incus driver that makes development machines; clusters and
containers in development are M5, VMs booted from the host's store M6.
What is open is in section 11.

## 1. In one page

- **One loop, from a working tree to a landing:** up, explore, capture,
  encode in IaC, apply from the working tree, land. Encode then apply is
  the inner loop; exploring and capturing are optional.
- **A development machine is an Incus VM cloned from a base image**
  (section 3) [M3b]: copy-on-write on a btrfs pool, booted in seconds,
  it takes its target once as bootstrap would, then converges as any
  standalone machine, fetching through the host's apt-cacher-ng.
- **`hearth dev` names machines, never containers** (3.4) [M5]: a
  container has no instance of its own. `hearth dev up rhi-dev` brings
  up the machine with the containers `dev` places on it, all of them or
  a selection; `shell` and `diff` reach one with `--container`; `apply`
  sends one revision to the machine and its containers, as they run.
- **A mutable machine may be changed by hand** (section 4): its
  environment marks the placement `mutable`, so the slot is not sealed
  and its root is read-write. The run side never deploys one; a
  standalone machine takes one only if its bootstrap said so.
- **Capture is drift, shown as a diff** (section 5): `hearth diff
  --running` lists packages, files under `/etc`, `/usr/local` and
  `/opt` with their contents, units and users; `--suggest` drafts the
  task fragments, and the agent decides what to keep.
- **A working tree applies without a branch** (section 6): a commit on
  no branch, shipped as a git bundle, becomes the machine's desired
  revision; the machine builds what changed and activates by the
  cheapest kind, live in seconds where it can.
- **No build from scratch in the loop** (section 7): every apply
  prepares a fresh slot from the IaC's output, so what was not captured
  is gone at once. A first boot from nothing, the upgrade from
  production and a sealed root are the lab's, at QA or at the gate.
- **The Incus driver is a client, outside the daemon** (section 8):
  `hearth dev`; nothing in the daemon depends on Incus.
- **Later, VMs boot from the host's store** (section 9) [M6]: no disk
  image, one build, a snapshot per VM.

## 2. The loop

```
up ──▶ explore (mutable) ──▶ capture ──▶ encode in IaC ──▶ apply from the working tree ──▶ land
                                               ▲                        │
                                               └──── the inner loop ────┘
```

Exploring is for what the agent does not know yet. Development is one
activity at different stakes (temper's `factory.md`, section 3): run a
revision somewhere and watch it, here on one agent's throwaway machine
with a root it may change. A session, adding DKIM to `mail-dev`, the
mail stack, which `dev` places as a container on `rhi-dev` as `prod`
places `mail-prod` on `rhi-prod` (format.md, 3.1):

```
hearth dev up rhi-dev --containers mail-dev      rhi-dev's VM, cloned (3.2), running mail-dev alone of its containers (3.4)
hearth dev shell rhi-dev -c mail-dev             in the container: apt install opendkim; edit /etc/opendkim.conf; systemctl enable opendkim
hearth dev diff rhi-dev -c mail-dev --suggest --out d
                                                 the capture and its drafts (section 5): about a second
                                                 the drafts move into tasks/mail, the conf as a template
hearth dev apply rhi-dev                         a packages change in mail-dev (section 6): the container restarted on a new slot,
                                                 the hand-made changes gone; rhi-dev's own slot untouched
hearth dev apply rhi-dev                         a template tweak with a reload entry: live inside the container, in seconds
git commit && git push                           a change, its rendered diff, the lab's gate (section 7)
```

`rhi-dev`'s own target is developed with the same commands without
`-c`, where a packages change is a soft reboot rather than a
container's restart. Until M5, a target that runs as a container in
production is developed as a machine of its own (3.4).

## 3. Development machines [M3b]

### 3.1 The host and the base image

- **The development host** has KVM, Incus with a btrfs storage pool,
  apt-cacher-ng on the Incus bridge, the `hearth` binary and the
  agent's checkout. The driver needs no root, and keeps its instances in
  a project of their own, `hearth-dev`. They are the VM tier's machines
  (machine.md, 2.2; testing.md, 7.1), real EFI machines to hearth.
- **The base image is one stopped VM, `hearth-dev-seed`,** bootstrapped
  once by `hearth dev image` from a small target, `seed-dev` (the
  `core` and `hearth` tasks and `incus-agent`), as the VM tier makes
  its machines (testing.md, 7.3 and 7.4), with `--accept-mutable` (4.3)
  and `--trial auto`. Clones share its filesystem's UUID, which is
  harmless: no machine sees two.
- **Its store is warm** with the essential set every target shares, and
  its builder fetches through the host's apt-cacher-ng (build.md, 5.1),
  so a clone fetches only what its target adds. A hearth release needs
  no new image: adoption runs the release the target pins (3.2).

### 3.2 Up

`hearth dev up <target>… [--count <n>] [--containers <c>,… | --no-containers] [--at <rev>] [--repo <path>] [--secrets <dir>]`:

1. **Render each target in-process** from the checkout (client/README.md, 3.5),
   at the working tree's commit (6.1) or at `--at`. A container's
   target is refused, naming the machine it runs on (3.4), and a
   machine's not placed on an EFI VM; both before Incus is touched.
2. **Copy** `hearth-dev-seed` to `rhi-dev`, a snapshot on the btrfs
   pool, set the instance's keys (8.2), start it, and wait for its agent.
3. **Push** the commit's bundle (6.2), the secrets, and the hearth
   binary the target's `hearth` task pins (format.md, 5.12), fetched
   once by its digest.
4. **Retarget:** `incus exec rhi-dev -- <binary> retarget rhi-dev --revision
   <commit> --bundle <file> --secrets <dir> [--containers <c>,…]`; wait
   for slot `a`, blessed, then [M5] for each container it runs.

`hearth retarget` is how a clone takes its target. It is root-only and
in-process, as bootstrap is (client/README.md, 3.6), and refused unless the
configuration accepts mutable targets (4.3) and the placement has the
machine record's kind and firmware. It runs bootstrap's steps 5 to 7 and
9 on the machine's own top level (machine.md, 8.2): import the secrets;
stop the daemon; build the output as `hearth` unless the store holds it;
place it into `@rhi-dev.a`, the target's identity made anew; give it the
first bless (slots.md, section 4); write the target, the desired
revision and [M5] the container selection (3.4); reboot. It writes as
the release it places, checked as bootstrap checks its own
(activator.md, 7.2). The new slot's first start deletes the retired
seed's slots, entries, ESP files, records and identity (activator.md,
7.4). A failed first boot is a VM to throw away, as after a bootstrap.

### 3.3 Several machines, and down

- **`--count <n>`** makes `rhi-dev-1` to `rhi-dev-<n>`, one target
  with its own identity on each: agents in parallel, or replicas.
- **Several targets make a cluster** [M5]: `hearth dev up stoa-1-dev
  stoa-2-dev stoa-3-dev`, a machine per member (format.md, 3.6) on one
  bridge, which resolves `<instance>.incus`; the development
  environment's settings name peers that way, and members read each
  other's exports as anywhere (format.md, section 8). Containers placed
  `on` a development machine's target run in its VM, as on any host
  (3.4).
- **Built once:** the driver builds every output in the first clone
  (`hearth build`), stops it and copies it for the others, which retarget
  with the outputs in their store, each with its own identity.
- **`hearth dev down <machine>…`** runs `incus delete --force`,
  conditional on the instance's target; `hearth dev list` lists them.

### 3.4 Machines and their containers [M5]

- **`hearth dev` names machines only.** An instance is a machine, an
  Incus VM with slots of its own. A container has none: it runs inside
  the machine its development environment places it on, as in
  production. `hosts/mail/mail-dev.json`:

  ```json
  { "placement": { "kind": "container", "on": "rhi-dev", "mutable": true } }
  ```

  A placement is replaced whole (format.md, 3.4), so it names `mutable`
  again, which `envs/dev.json`'s VM placement set. `hearth dev up
  mail-dev` is refused before Incus is touched: `mail-dev is a container
  on rhi-dev: hearth dev up rhi-dev`. What counts is the placement in
  `dev`, so a service `dev` places on a VM of its own is a machine
  there, as before M5 (the last bullet).
- **A machine comes up with its containers.** `hearth dev up rhi-dev`
  brings up its VM (3.2), whose hearth then builds and starts every
  container `dev` places on it, as any host's does (wire.md, 4.7;
  slots.md, section 12). `--containers mail-dev,chat-dev` runs those
  alone, `--no-containers` none. The selection is the machine's own
  setting, kept by its hearth (wire.md, 4.1 and 4.7) and reported as a
  fact; the host's spec stays whole, and nothing in any key changes. A
  second `up` of a running instance sets its selection again; a
  container it leaves out is stopped and its slots deleted, as a
  removed container's are (slots.md, 12.7), its data kept for the next
  time it runs.
- **One revision for the machine and its containers** (wire.md, 4.7),
  so `apply` names machines too. `hearth dev apply rhi-dev` ships the
  working tree; the host's pass converges each container it runs, then
  its own target; the report has a line per target: unchanged, live,
  restarted, rebooted or fell back. A commit that changes the host and
  a container is one apply. One that changes only `mail-dev` prepares a
  new slot of `mail-dev` alone, tried live inside the container when
  eligible (slots.md, 12.8), else by restarting it on the new slot
  (12.2): seconds, with the host's own slot untouched.
- **`shell` and `diff` take `--container`** (`-c`): `hearth dev shell
  rhi-dev -c mail-dev` enters the container (4.4), and `hearth dev diff
  rhi-dev -c mail-dev` captures it through the host's hearth (5.1).
  Without it they act on the machine's own root. `diff` takes several
  `-c`, or `--all` for the machine and every container it runs, each
  captured apart, its drafts under `<dir>/<target>/` (5.2).
- **Several at once:** several containers of one machine come up
  together, all or the ones `--containers` names; containers of several
  machines are those machines named together, `hearth dev up rhi-dev
  ion-dev`, each machine running the listed containers placed on it; a
  machine and one of its containers are one instance, developed
  together as they run. `--count <n>` gives each copy the same
  selection, each container with its own identity per copy.
- **`down` and `list` name machines:** a container goes down with its
  machine, and `list` shows each machine's containers, run or left out,
  with their states.
- **A machine of its own before M5,** with no containers anywhere: a
  target that runs as a container in production is placed on a VM in
  `dev`, its base-env file adding what a machine needs and a container
  lacks, the `hearth` and `boot` tasks, as `mail-lab` runs in the lab
  until M5 (format.md, 3.1). It is then a machine, and `up` takes it.
  What that leaves untried, the container's parameters and its host, is
  the lab's.

## 4. Mutable machines

### 4.1 The field

`envs/dev.json`:

```json
{ "placement": { "kind": "vm", "firmware": "efi", "hardware": "incus", "mutable": true } }
```

Placement is data of the environment (format.md, 3.4). Render puts
`mutable` in the `boot` part (render.md, 4.3): a slot part, so it
enters the prepare key and no layer key, and a mutable target builds the
very output its sealed twin would.

### 4.2 What changes in a slot

Prepare leaves out the read-only flag (slots.md, 5.7, step 1) and still
writes the manifest that capture compares against. fstab's `/` line
says `rw` (machine.md, 3.3); the command line keeps `ro`, since
tiny-initramfs mounts the root read-only and systemd remounts it from
fstab, so the line is the sealed twin's and the kernel's rule holds
(slots.md, 6.1). A container's [M5] nspawn settings say `ReadOnly=no`
(slots.md, 12.1).

### 4.3 Who accepts one

- **The run side never deploys one:** its engine's policy refuses a
  mutable target [M4]; the lab's may allow one on development pools.
- **A standalone machine's configuration says,** `mutable: accept` or
  `refuse`, `refuse` by default, set only by bootstrap's
  `--accept-mutable` (client/README.md, 3.8): no command on the socket makes a
  machine a development machine. Sealed targets run there too.
- **The activator enforces it:** the setting is in its machine record,
  and it refuses a place whose `boot` part is mutable on a machine that
  refuses, naming the field (activator.md, 2.2); it would be the one to
  leave a root unsealed. The steward refuses first, from the facts, as a
  configuration error that waits for another commit (wire.md, 4.4).

### 4.4 Exploring

- **`hearth dev shell rhi-dev`** is `incus exec rhi-dev -- bash -l`,
  root, with `http_proxy` naming the host's apt-cacher-ng, so apt
  fetches through it with nothing written to `/etc`; `hearth dev shell
  rhi-dev -c mail-dev` [M5] is `incus exec rhi-dev -- machinectl shell
  mail-dev` (3.4). There the
  agent installs from the slot's own sources, the pinned snapshot's
  (build.md, 5.6), edits `/etc`, fills `/opt`, enables units, adds
  users.
- **The experiment lives in the running slot,** its root and its
  `/var`, never in data entries. The next prepare leaves it behind: a
  new slot is a snapshot of the output (slots.md, 5.1), never of the
  running root. The explored slot stays on disk as the previous blessed
  one until a later prepare takes its label; `hearth rollback` would
  bring it back, and the loop never does.
- **Live changes need an unexplored root** (slots.md, 6.7, rule 12): on
  a mutable machine a change is live only while nothing under `/etc`,
  `/usr/local` and `/opt` differs from the slot's manifest; otherwise its
  trial is a soft reboot, which ends the experiment at once. After a
  live change those three are read-only overlays (slots.md, 6.6) until
  the slot boots natively, which `hearth trial` does by a soft reboot.

## 5. Capture

### 5.1 What it compares

`hearth diff --running [--target <t>] [--suggest] [--json]` runs on the
machine, for operators. The steward asks its activator, since drift is
the activator's (slots.md, section 10): capture is drift's walk with
contents, presented as a diff, and reaches the client as the link's
typed records, so the steward reads no file of the slot itself
(steward.md, 2.2).

| Kind | Compared | Read from | Shown as |
|---|---|---|---|
| packages | the installed set, by name | dpkg's status in the slot's `/var` against the output's, and apt's `extended_states`, through `hearth-protocol-dpkg` | `+ opendkim (manual)`, `+ libopendkim11`, `- nano` |
| files | added, changed, deleted under `/etc`, `/usr/local`, `/opt` | the root's metadata walk against the manifest, then the contents of what differs | path, type, mode, owners, content |
| units | enabled, disabled, masked | the links under `/etc/systemd/system` | `+ opendkim.service in multi-user.target.wants` |
| users, groups | added, removed, ids changed | `/etc/passwd` and `/etc/group` against the output's | name, ids, home, shell, groups |

- **Packages own their files,** and are named without versions, which
  the pinned snapshot fixes. What dpkg lists for an added package, its
  conffiles unless edited (dpkg's status keeps their digests), and the
  unit links its maintainer scripts made are the package's, so `apt
  install opendkim` shows one package and the agent's edits. What
  differs elsewhere in the root, unexplained, is listed by path as
  `other`.
- **Left out:** `/var` but dpkg's and apt's records; data entries; the
  files the activator writes (`/etc/fstab`, `/.hearth`, the trial
  units, the machine-id, `/etc/shadow`), reported, never suggested; the
  contents of secret and identity files, listed by path only.
- **Contents only on a mutable machine,** up to 1 MiB a file and 16 MiB
  in all. Elsewhere it is drift by path and digest, so a production
  machine's files never reach the client. The exit code is `diff`'s
  (client/README.md, 3.3); a container's [M5] is captured through its host.

### 5.2 Suggestions

`--suggest` turns each kind into a draft fragment of a task
(format.md, section 5): a pure function of the capture, in
`hearth-client-domain-dev`, run by the client.

| Captured | Suggested |
|---|---|
| a package installed by hand (apt's manual mark) | `"packages": { "opendkim": {} }`; its dependencies are the resolution's |
| a package removed | a note naming the task that requests it |
| a file added or changed | the file under `tree/`, with a `files` entry when its mode or owners are not the defaults (format.md, 5.5) |
| a file deleted | a note: a task's file to drop, or a removal in `customize.sh` (format.md, 5.8) |
| a unit enabled, disabled or masked | `"presets": { "opendkim.service": { "action": "enable" } }`; a mask, a `files` link to `/dev/null` |
| a user or a group added | a `users` or `groups` entry with its ids, which build writes as sysusers lines (format.md, 5.6); a system id, below 1000, as `"from": "package"`, to confirm |

`--out <dir>` writes them as a task directory, `task.json` and `tree/`,
on the development host with `hearth dev diff`, never into the
repository: whether a file becomes a template, a value a setting or a
unit a `live` entry is the agent's decision.

### 5.3 Drift on a mutable machine

The same checks run under the same ids (slots.md, section 10): what
`slot.root.tree` finds is the experiment, reported as `explored`, never
as a fault; `slot.root.subvolume` expects the read-only flag clear. A
mutable root hides a service that writes beneath `/etc` at run time,
which fails with `EROFS` when sealed: capture shows a change nobody made
by hand, and the lab's sealed machines fail on it (section 7).

## 6. Applying a working tree

### 6.1 The commit

`hearth revision bundle <file> [--repo <path>] [--since <commit>]`, on
the development host, in-process:

```
GIT_INDEX_FILE=<scratch>/index git read-tree HEAD
GIT_INDEX_FILE=<scratch>/index git add --all               the working tree, .gitignore honoured
GIT_INDEX_FILE=<scratch>/index git write-tree              → T
git commit-tree T -p HEAD -m 'hearth: working tree'        with HEAD's author, committer and dates → C
git update-ref refs/hearth/dev/<machine> C
git bundle create <file> refs/hearth/dev/<machine> ^<since>
```

The checkout's index, HEAD and branches are untouched, and
`refs/hearth/dev/` is listed by no `git branch` and sent by no plain
`git push`. Names, dates and message are fixed, so one tree on one HEAD
gives one commit, and applying it twice changes nothing. `--since` is
the commit last shipped to the machine; without it the bundle holds the
whole history.

### 6.2 Into the machine

- **Pushed** with `incus file push --uid 59999 --gid 59999 --mode 0600`
  to `/run/hearth/<commit>.bundle`, which only the builder reads, in a
  fetch job (steward.md, 3.3).
- **`hearth revision set <commit> --bundle <file>`** (wire.md, 4.1): the
  steward has a builder fetch the bundle into its clone (render.md, 2.4)
  as `refs/hearth-dev/<commit>` and delete the file, then sets the
  desired revision; a missing prerequisite is refused, and the driver
  sends the whole history. The pass never fetches the commit, which the
  clone holds (wire.md, 4.2). Any standalone machine accepts it from an
  operator, as `revision set` accepts any commit (client/README.md, 3.1).

### 6.3 What the machine does

The pass, as on any machine (wire.md, 4.2); a development machine's
trials are `auto`.

| The change | Built | Activation | Time |
|---|---|---|---|
| config: a template, a file, a preset, a user | the config layer (build.md, section 4) | live, with a declared consumer over an unexplored root (slots.md, 6.7); else a soft reboot | seconds |
| software under `/opt` | software and config | live, by its `restart` entry; else a soft reboot | seconds |
| a package added or removed | all three; packages reconciled from the previous layer (build.md, 4.2), through apt-cacher-ng | a soft reboot while the kernel is unchanged | a minute or less |
| the kernel, the initramfs, the command line | all three | a one-shot reboot | a minute or two |

In a container [M5] (3.4), a soft reboot or a reboot in this table is a
restart of the container on its new slot, and the host is untouched;
the kernel and its command line are the host's.

- **`hearth dev apply <machine>… [--repo <path>] [--how auto|live|full]`**
  makes the commit and its bundle, prints the rendered diff of the
  machine's target and each container it runs against the revision the
  machine runs (render.md, section 6), pushes, sets the revision, and
  follows `hearth status` until the pass converges, is refused or falls
  back, with a line per target (3.4); its exit code is `status
  --check`'s, and `--how` is `hearth trial`'s (client/README.md, 3.1). A trial that falls back refuses its prepare key, as
  anywhere (wire.md, 4.4).
- **Without the driver** [M3]: `hearth revision bundle`, any copy, and
  `hearth revision set --bundle` on any standalone machine.

## 7. From development to landing

- **No build from scratch in the loop.** This is the user's decision.
  Every apply prepares a fresh slot from the IaC's output, leaving the
  experiment's slot and its `/var` behind, and builds the layers
  incrementally from the store's cache. Whatever was not captured
  disappears at once, and the agent sees it at the next apply.
- **The build is not what differs:** above packages an incremental layer
  equals a scratch one by construction, and packages are reconciled,
  with a scratch build now and then (build.md, 4.2). What a development
  machine does not show is a machine's history:
  - a first boot from nothing: data entries are seeded and identity is
    made once, then kept (slots.md, 5.3 and 5.4);
  - the native boot after a live change, until the agent reboots;
  - the upgrade from production's revision and data;
  - a sealed root, on which a service that writes `/etc` fails (5.3);
  - several related changes together.
- **These are the lab's,** at QA or at the landing gate (temper's
  `factory.md`, section 8): a sealed throwaway machine bootstrapped at
  the change's head, possibly once for a stack of related changes, at
  its top. So is the comparison of patch and scratch for the packages
  layer (build.md, 4.2; temper's `factory.md`, 6.4).
- **What lands is the agent's own commit** of the same tree;
  `refs/hearth/dev/` never leaves the development host. Reviewers read
  the rendered diff (render.md, section 6) and the lab's verdicts gate
  it.

## 8. The Incus driver [M3b]

### 8.1 Outside the daemon

`hearth dev` is the `hearth` binary's client, run as the agent's user,
reaching each machine's hearth through `incus exec` as a person at the
machine would. Nothing in the daemon depends on Incus: a development
machine is a standalone machine, and only `mutable: accept` and `hearth
retarget` know its use. It is the lab's pool-host machinery used by hand
(temper's `factory.md`, 5.2 and section 8): create a throwaway machine,
keyed; apply a revision; read the facts; tear it down. The lab automates
the same loop, its engine in the agent's place.

### 8.2 Its domain

`hearth-client-domain-dev`, a domain root of its own, hosted by the client
(`programming-model.md`, section 4):

- **Machines are entities named by handles,** one per instance, in one
  state each: `copying`, `starting`, `retargeting`, `converging`, `ready`,
  `applying`, `deleting` or `failed`; at most 16 a command, each stage
  under a deadline. Containers are not entities of their own: a
  machine's report carries them, read from its hearth's facts (3.4).
- **Keyed effects, no state of its own:** `up` is keyed by the
  instance's name and target, `apply` by the commit, `down` conditional
  on the target, all read back from each instance's `user.hearth.target`,
  `user.hearth.commit` and `user.hearth.image` keys.
- **Its other parts are pure:** the working tree's commit (6.1), whose
  git commands go to `hearth-protocol-git`, and the suggestions (5.2).
  It renders through the builder's domain, in-process (client/README.md, 3.5).

### 8.3 incus as a tool

`hearth-client-protocol-incus` holds incus's argument vectors and decodes its
output; the domain never sees its text:

```
incus copy hearth-dev-seed rhi-dev --project hearth-dev
incus config set rhi-dev user.hearth.target=rhi-dev user.hearth.commit=<c> --project hearth-dev
incus start rhi-dev --project hearth-dev
incus file push --uid 59999 --gid 59999 --mode 0600 <bundle> rhi-dev/run/hearth/<c>.bundle --project hearth-dev
incus exec rhi-dev --project hearth-dev -- hearth status --json
incus list --format json --project hearth-dev
incus delete --force rhi-dev --project hearth-dev
```

`incus list --format json` is decoded strictly, unknown fields skipped
(README.md, 8.1); `incus exec` returns the command's exit code, and the
hearth inside answers in `--json` records, which
`hearth-client-protocol-records` decodes. Failures are typed: an instance
missing or already there, an agent not answering yet (retried with
backoff), a timeout, a full pool.

## 9. Later: VMs from the host's store [M6]

- **The development host runs hearth, as a pool host:** its builder
  builds each output once; its activator prepares a slot per VM on its
  own top level, a snapshot of the output, which the VM boots with no
  disk image, by Incus's direct kernel boot with its root over virtiofs
  (machine.md, 2.6).
- **One build, a snapshot per VM, a boot in seconds:** a cluster of one
  spec costs one build, as the lab's pools do (temper's `factory.md`,
  6.5; slots.md, 12.4), and the host's store stays warm across every
  `down`. No base image and no adoption: slots, records and identities
  are the host's activator's, as a container's are.
- **What it gives up** is an ESP, firmware variables and a bootloader,
  which stay the VM tier's to prove. The loop above is unchanged.

## 10. Its world

- **`hearth-client-dev-world`** [M3b], under `tests/dev`, runs
  `hearth-client-domain-dev` against `hearth-client-fake-incus` on the
  fake machine (testing.md, sections 3 and 4): a pool whose copy is a snapshot,
  agents that answer late or never, and `incus exec` running a scripted
  hearth, or hearth's domains on a fake machine of the instance's own.
  Scenarios: `up` of one, of `--count 3`, of a cluster built once; a
  copy that fails half-way; `down` during `up`; one tree applied twice;
  a bundle without its prerequisite; [M5] `up` of a container's target,
  refused with its machine named; a selection set, then changed by a
  second `up`; one apply that converges a host and one of its
  containers. Invariants: every instance carries
  its keys; an `up` ends with each machine converged or reported failed;
  an apply ships exactly the working tree's commit, and a second one
  nothing; `down` deletes only what it names.
- **Capture, in the slots' and the activator's worlds** [M3]: a mutable
  slot explored by changes drawn from the seed (packages from the fake
  universe, files, units, users) captures exactly them, and the
  suggestions, applied unchanged, give a slot whose capture against the
  explored root is empty. No machine that refuses holds a mutable slot,
  and a live trial on one runs only over an unexplored root.
- **The bundle, in the standalone world** [M3] (wire.md, section 10):
  imported, refused without its prerequisite, never fetched.
  `hearth-client-protocol-incus`'s machine world [M3b] is fed recordings of
  real runs (testing.md, 4.5), and the VM tier runs the whole loop on a
  real development host (testing.md, 7.6).

## 11. What is open

- **Whether `incus copy` carries the VM's variables** (`qemu.nvram`),
  without which a stub-only disk has nothing to boot (testing.md, 7.4);
  to verify, with the time a copy takes on a btrfs pool.
- **Adoption with a trial:** the seed's slot could stay the default
  while the target's first slot is tried, so a broken first build falls
  back instead of costing a VM; it depends on how often first builds
  fail, and on guards that span two targets.
- **How the builder's fetch proxy is set,** by a bootstrap argument or a
  file of hearth's configuration, which the steward passes to its
  builders; it depends on build.md's fetch.
- **Noise in capture,** to verify on the VM tier: whether systemd's
  update services (`ConditionNeedsUpdate=/etc`) write `/etc` at a
  mutable root's first boot, marking every root explored, and whether
  deb-systemd-helper's state records the unit links a package made.
- **Writes beneath a live overlay:** on a mutable base, a process that
  held a directory under `/etc` open before the swap can still write the
  lower layer, which overlayfs leaves undefined; sealing the base while
  a live state runs would prevent it, if worth the bookkeeping.
