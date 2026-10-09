# hearth: brainstorm

Draft, 2026-10-09. It records the direction of a design discussion
on 2026-10-09, the first about hearth. It starts from temper's
`factory.md` (a draft vision of the whole software factory) and from os
(`~/src/rust/os`), whose tools hearth replaces. Nothing here is adopted.
Section 12 is the order of building; section 13 lists what is open.

## 1. In one page

- **hearth is the factory's build daemon** (temper's `factory.md`,
  section 7). It makes outputs from inputs, keyed by digest, and runs
  them in slots. It runs on every machine: hosts, the lab's pool hosts
  and its builders.
- **It replaces os.** os's four tools (`manifest`, `deploy`,
  `provision`, `ek`) become one domain in one daemon. os stays a
  reference for its features and its golden files, not for its code.
- **os's IaC format stays, with tweaks** for agents and digests: a
  schema per task, every input in the repository, merging that can
  remove, data shared between targets through exports, and the rendered
  diff on every change (section 7).
- **It is built on skein** and follows `programming-model.md`: one
  loop, step functions, and a domain complete enough to run whole on a
  fake machine. The heavy work stays in child processes.
- **Two processes, split by privilege.** An unprivileged builder speaks
  the wire and runs everything that comes from an IaC repository; a
  small activator, as root, places outputs into slots, writes boot
  entries and restarts units (4.1).
- **Efficient everywhere, tests included, from day 0:** realistic fakes
  at every boundary, test tiers with budgets, real builds as a regular
  user, and hearth's own costs reported as facts (4.3).
- **The engine decides, hearth carries out.** The run side's engine, or
  the lab's, orders a target prepared or activated, and hearth does it.
  With no engine, hearth converges to a pinned revision on its own.
- **One wire:** skein's channel, dialed out by hearth to its plane's
  engine. Bytes move by digest, outside the channel.
- **Outputs are layers keyed by digest:** packages, then software, then
  etc, each a read-only btrfs subvolume (4.6). A change rebuilds only the
  layers above the lowest one it touched, on a snapshot of the layer
  beneath.
- **Slots try before they commit.** A new slot boots on trial. The boot
  default moves only when the slot is blessed, so a failure falls back
  to the old slot on its own.
- **An output holds nothing unique and nothing secret,** so it can be
  shared between machines and served by digest.
- **After a rollout, hearth is the machine's end of operating it.** It
  runs the target's checks, sends facts up, keeps logs and metrics on
  the machine for whoever asks, and carries out a small set of
  operational effects (section 10).
- **Reproducible builds are deferred.** They are a level of trust, and
  can be retrofitted without changing the store, the wire or the slots
  (section 11).

## 2. Where it comes from

- **The factory's daemon.** temper's `factory.md` (section 7) wants one
  daemon on every machine that builds outputs from inputs, keyed by
  digest, with two recipes: a root filesystem from a target at a
  revision, and an artifact from a build environment, sources and a
  command. It knows nothing of jig; to the lab and the run side, it is a
  system behind the infrastructure connector.
- **What the factory asks of os** (`factory.md`, section 12): each
  part's digest reported upward; slots made by snapshot; activation by
  soft reboot; boots counted, so a slot that does not come back healthy
  falls back; the artifact recipe; and a remote interface. hearth is
  that interface, and the rest grows inside it.
- **os today:** four command-line tools, ported from POSIX shell, that
  shell out to mmdebstrap, apt, btrfs, mount, parted, GRUB and
  efibootmgr through sudo. The pipeline is: configuration, then
  `manifest` renders a spec directory, then `deploy` builds a root
  filesystem from it, then `provision` or `ek` turns it into a bootable
  A/B slot and activates it.

## 3. The name

**hearth:** the bed of the forge, where the fire burns and the stock
is heated and shaped. Every machine has one; it does the heavy work on
that machine and keeps the warm state: caches and earlier outputs.

## 4. The daemon

### 4.1 Two processes per machine

- **The builder, unprivileged.** It speaks the wire, renders, builds,
  keeps the store, runs the checks and serves telemetry. Everything that
  comes from an IaC repository runs here, each in a user namespace:
  mmdebstrap, apt, maintainer scripts, `customize.sh`. In the lab, that
  includes code nobody has reviewed.
- **The activator, as root, and small.** It does a fixed set of
  operations on outputs named by digest: place one into a slot, write
  boot entries, soft reboot or reboot, restart a unit, partition and
  format a disk, snapshot data for backups. It runs no IaC code, and
  listens only to its builder, on a local socket.
- **One binary, two processes,** each on skein's loop. Tests and the
  lab's builders need only the builder.
- **What the builder needs, in general:** a kernel that allows
  unprivileged user namespaces; the `newuidmap` and `newgidmap` helpers
  (Debian's `uidmap` package) with a subordinate uid and gid range for
  its user; and its store on btrfs (4.6).
- **The heavy work stays in child processes:** mmdebstrap, apt, btrfs,
  git, efibootmgr. Each process schedules its own, bounds how many run
  at once, and records what each cost.

### 4.2 The layers

- **Domain:** pure step code, in modules that keep os's boundaries as
  crate boundaries rather than binaries:
  - *render:* configuration and tasks at a revision, to parts with
    digests (os's `manifest`);
  - *store:* outputs by digest, their layers, what pins them, and
    collecting what nothing pins;
  - *build:* plans how to make an output from a spec, from the closest
    layer it has;
  - *slots:* prepare, trial, bless, fall back (section 9);
  - *disk:* partitioning and formatting, for bootstrap and the lab's
    throwaway machines (os's `provision`);
  - *drift:* the live slot compared with the output it came from;
  - *checks:* the target's health checks and the machine's own signals,
    in a bounded window (section 10);
  - later, *artifacts* (the second recipe), *backups* (10.4) and
    *faults* (section 5.2);
  - *slots*, *disk* and the backups' snapshots are the activator's work;
    the rest is the builder's.
- **Protocol:**
  - the wire (section 5.3);
  - **each external tool is a protocol:** an argument vector goes down;
    progress, output and an exit code come up and are parsed. os already
    has several such parsers: mmdebstrap's progress and failure tail,
    `dpkg-query`'s output, `efibootmgr -v` and its UTF-16 option data.
    They move here, and the domain never sees a tool's text.
- **io:** files beneath a root, and spawning programs. Nothing new is
  needed in skein's io to begin with: btrfs and mount are reached
  through their tools. Native records come later, only where they pay.

### 4.3 Testing, from day 0

Efficiency is a principle in every part of hearth, testing included:
fast tests, all of them runnable as a regular user, designed with the
code from the first step.

- **The fake machine.** The domain runs whole in worlds of fakes
  (`testing-strategy.md`). Its fake is a machine: subvolumes and
  snapshots, a package universe drawn from the seed, firmware boot
  variables, reboots and power cuts. A world can cut the power during a
  prepare, fail a boot on trial, fill the disk mid-build, or drop the
  engine during an activation, and check that the machine always comes
  back bootable. This is the machine fake temper already plans (temper's
  `testing.md`, section 3.2), so the two should share it. os's real
  tests need sudo and feature gates; this is the largest gain in quality
  over it.
- **A realistic fake at every boundary.** Each external tool is a
  protocol (4.2), so each gets a fake with the same face: an argument
  vector in; progress, output and an exit code out; effects on the fake
  machine's files.
  - Fake mmdebstrap installs from a small package universe modelled on
    Debian's: base-files, a dpkg status of the real shape, users made by
    maintainer scripts, conffiles, a kernel package. A fake build gives a
    small but realistic root filesystem.
  - Its output follows recordings of real runs (progress lines, failure
    tails), and the seed draws its faults: a stall, a 404 from the
    mirror, a maintainer script that fails, a full disk.
  - The same for apt, `dpkg-query`, btrfs, git, systemctl and
    efibootmgr.
- **Fakes kept honest.** A conformance tier runs the same scenarios
  against the real tool and against its fake, and compares what each
  leaves: the installed set, dpkg's status, files and their modes. skein
  holds its simulator and its io_uring backend to one kernel contract
  the same way.
- **Tiers, each with a budget** (temper's are 15 seconds for the default
  suite and a minute for the randomized one):
  - domain worlds on the fake machine, seeded: milliseconds a scenario,
    and the bulk of the tests;
  - protocol parsers against recorded output of the real tools:
    milliseconds;
  - render, with package resolution reused by its key: milliseconds.
    Resolution itself (`mmdebstrap --simulate`) needs no privilege;
  - real builds as a regular user, against a small pinned local mirror
    fetched once (mmdebstrap's own test suite runs offline this way),
    through hearth's own layer cache. The base layer is built once per
    mirror digest (an essential bootstrap takes 10 to 15 seconds); each
    test then pays only for its own step, seconds or less. The digest
    rule is tested on hearth itself.
- **What needs root runs in the lab:** real slots, boot entries and
  reboots, on throwaway machines; never in a developer's suite.
- **The real tier needs** what the builder needs (4.1), with a btrfs
  directory writable by the test user, mounted with
  `user_subvol_rm_allowed`.
- **hearth's costs are facts:** time per step, bytes fetched, cache
  hits. The lab benchmarks hearth as it does any product, and a
  regression in hearth's efficiency is a finding.

### 4.4 Two faces

- **The daemon,** speaking the wire to its engine.
- **A command-line client,** speaking the same channel over a local
  Unix socket. It can also run the domain in-process, to bootstrap a
  machine from a live system where no daemon runs yet.

`shell` (os's `deploy shell` and `ek shell`) stays a local command,
never part of the wire: agents only read.

### 4.5 Generic at the wire

The factory wants its concepts generic, with os as their first backend.
hearth keeps the genericity in its wire, not in traits (which the Rust
subset bans anyway): another backend is another daemon speaking the
same wire. Inside, hearth is Debian on btrfs, and says so.

### 4.6 The store on btrfs

- **btrfs only.** A layer is a read-only subvolume owned by the builder,
  made from a writable snapshot of its parent; a slot is a snapshot of
  an output; deleting a layer is immediate. One mechanism serves builds,
  slots and backups, as in os.
- **Unprivileged.** A user creates subvolumes, snapshots the subvolumes
  it owns, and makes them read-only. Deleting one needs the store's
  filesystem mounted with `user_subvol_rm_allowed`; without it, the
  builder empties the subvolume and removes its directory, one unlink
  per file.
- **What a step changed** comes from a walk of the new layer against
  its parent's record: a file whose path, inode and change time all
  match is unchanged, and only the others are hashed. The walk gives the
  layer's tree digest (11.2) and its change set.
- **No btrfs send and receive:** both need root. A layer moves to a peer
  as its change set (changed files and deletions), and the receiving
  builder applies it to a snapshot of the parent it already holds
  (5.3).
- **Overlay layers were considered:** overlayfs in a user namespace,
  each layer being an overlay's upper directory. They build on any
  filesystem and give the change set for free. But they add a second
  mechanism beside btrfs, which slots and backups need anyway; flattening
  a stack into a slot is a copy rather than a snapshot; deleting a layer
  unlinks every file; and a host running on overlay meets whole-file
  copy-up and hardlink quirks. The factory provisions its own builders,
  so building on any filesystem is not worth that.

## 5. Roles

### 5.1 Who drives

- **The engine decides; hearth carries out.** Rollouts order machines,
  so convergence cannot be each machine's own decision. The run side's
  engine, or the lab's, asks hearth to prepare a target at a revision,
  then to activate it; hearth fetches the revision, renders, builds and
  reports.
- **Effects are keyed or conditional** (jig's `connectors.md`, 4.3):
  - *prepare* is keyed by target and revision, and repeating it finds
    what the first made;
  - *activate* and *roll back* are conditional on the active slot being
    the one decided from.
- **Standalone mode:** with no engine, hearth converges to a revision
  pinned in its configuration. It serves machines with no run side,
  including the run side's own first machine.

### 5.2 Machines, containers and pools

- **A host's hearth owns its own slots and its containers' roots.** os
  already has container targets (`hosts/<host>/machines/<name>.json`);
  hearth prepares their roots and restarts them on a new one.
- **A lab pool's host is a host whose containers are the lab's
  machines.** One build per spec, then a copy-on-write snapshot per
  container (`factory.md`, 6.5).
- **Faults come later,** as effects of a pool's host (a partition,
  latency, a killed process, a full disk), granted only by the lab's
  policy (`factory.md`, 5.3).

### 5.3 The wire

- **Control is skein's channel,** as between temper's engine and its
  workers. It gives framing, a version agreed at opening, bounds and
  flow control; the domains own the hello, fenced attempts,
  acknowledged answers and redialling.
- **hearth dials out** to its plane's engine:
  - machines need no inbound port, and the engine no list of addresses;
  - a machine's identity is the credential it presents when the channel
    opens;
  - a lab machine holds no credential for the run side's engine, so the
    planes stay apart by wiring.
- **A reboot is a redial.** hearth's hello carries its facts: its
  slots, which one runs and which is the boot default. The hello is how
  the engine learns that an activation held or fell back; there is no
  other path for a machine coming back.
- **Bytes move by digest, outside the channel:** an output fetched from
  a peer or a pool's cache with a plain HTTP GET, as its change set
  against a parent the receiver already holds (4.6). Content-addressed
  bytes need no session, and any cache can serve them.
- **Facts and events are typed.** os's tab-separated output and its
  exit codes go; the command-line client may still print a table.

## 6. From os

### 6.1 Kept

- **The configuration model:** hosts as a base and environments,
  composed in that order; tasks; strict JSON; inert templates;
  `etc.json`'s declarative rules and `require` guards. It is the IaC
  language, and it stays, with section 7's tweaks.
- **The spec split into parts,** with package resolution pinned to a
  snapshot of snapshot.debian.org and reused while its hash is
  unchanged.
- **Secrets named by path,** never held by the spec.
- **A/B slots on btrfs:** a build, then a snapshot of it into a slot,
  with templates that only activation can complete (`fstab`).
- **Drift, with stable check ids.** The checks become facts.
- **The boundaries between render, build, disk and boot,** as crates of
  one domain (4.2).

### 6.2 Changed

1. **Specs are digests of parts, not a directory.** Rendering takes
   milliseconds, so it always runs in full; only the slow steps
   (resolving packages, building) are remembered by their input key. Any
   hearth can render any target, so the run side can ask any of them
   which parts a commit changes.
2. **Outputs are layers keyed by cumulative digest:** packages, then
   software, then etc (with the customize hook and systemd's presets).
   A layer's key is the digest of its parent's key and its own part's
   digest, and the layer is a read-only subvolume named by its key.
   - A one-line change to etc is a snapshot of the cached software layer
     with the etc layer replayed on it: seconds, with nothing left over
     from earlier configurations.
   - Above packages, a patch and a build from scratch agree by
     construction. apt's reconcile survives only for the packages layer,
     where `factory.md`'s comparison of patch and scratch (6.4) still
     applies.
   - This replaces etckeeper as the contract: the `pristine-debian` tag
     and the `Manifest config <hash>` commit go.
3. **Nothing unique and nothing secret in an output.** Outputs are
   shared, between a pool's containers and between peers. A machine-id or
   SSH host keys baked into one would be shared by every machine built
   from it. So machine-id is left uninitialized, host keys and the other
   secrets are delivered at activation (a separate mount, or systemd's
   credentials) under the machine's own identity, and a random seed is
   made at first boot.
4. **Slots get a state machine** (section 9).
5. **ek folds into hearth.** The helper on a live host is the daemon.
6. **Typed facts and events** replace the tab-separated output (5.3).
7. **The fleet leaves the code.** Four details of one fleet are in os's
   code and move into tasks (with the magic task names of 7.9):
   - `host.hardware == "7th-gen"` switching on non-free components;
   - the Europe/Lisbon timezone and en_US locale preseeds;
   - the persistent `@<target>.srv.core/journal` and `iwd` directories;
   - masking gettys in containers.
8. **Little of os's code ports.** Step crates are `no_std`, so
   MiniJinja, serde, clap and anyhow do not fit. Rendering needs a small
   inert template engine of its own, for the Jinja subset the tasks use.
   What ports is the design and the tests: os's golden files become
   render's conformance suite, and os serves as an oracle while hearth
   catches up, the same configuration giving the same parts. Where
   section 7 changes the format, a converter migrates os's configurations
   (lists to maps, `___` names to trees, the overlay to layers), and the
   comparison holds on the converted ones.

### 6.3 Dropped

- **`deploy patch`,** which repairs `/etc` in place on a live system.
  With immutable slots the fix is a new slot; drift is reported, never
  repaired in place.
- **The dir backend:** the fake machine takes its place in tests.
- **Legacy forms:** bare `@a`/`@b` slots, legacy kernel command-line
  tokens, the compatibility `label`.
- **Finding the target from `/proc/cmdline`:** hearth's configuration
  names its targets.
- **`--resolved`:** the fake machine's package universe takes its place.
- **Specs committed to git:** they are derived, never stored.
- **`--overlay`:** every input lives in the repository (7.3).
- **Templates completed at deploy** (`when: "deploy"`): render knows
  everything they read (7.9).

## 7. The IaC format

os's configuration and task formats (os's `config-format.md` and
`task-format.md`) fit the factory better than most IaC formats:
configuration is declarative data; templates cannot run anything;
rendering is deterministic and cheap; a task bundles everything about
one concern; variants are thin; secrets are named by path only. Their
best property for the factory comes first (7.1), then the tweaks they
need, most important first. The first four (7.2 to 7.5) matter most:
the schema and the merging make agents' edits safe, every input in the
repository makes the digests and GitOps hold, and exports make clusters
possible.

### 7.1 The rendered diff on every change

Determinism pays off here. Every change
to an IaC repository carries its rendered diff, per target and per part:
"this commit changes `/etc/postfix/main.cf` on mail-lab and mail-prod,
and nothing else". It is what agents and people review, posted at the
change's head as a status is. Rendering needs no secrets and no machine,
so any hearth can produce it.

### 7.2 A schema per task

- **Today a task's settings are whatever its templates read,** and an
  undefined name counts as false. An agent's typo (`ssh.alow`) silently
  drops a file instead of failing.
- **Each task declares its settings:** names, types, defaults, and a line
  of documentation each. Render rejects a setting the task does not
  declare and a value of the wrong type.
- **What it gives:** defaults in one place rather than in templates;
  most `require` guards become types (`syncthing.type` is one of a
  list); each task is documented for the agents that edit it.
- **Undefined stays falsy only for optional settings the schema
  declares.** A name no schema declares is an error.

### 7.3 Every input in the repository

- **A spec is a function of the revision and the target, nothing
  else.** That is what makes its digests mean something and the
  repository's main branch the desired state.
- **So os's `--overlay` goes.** An overlay passed when rendering is an
  input outside git. The original fleet repository had already dropped
  it.
- **Its legitimate uses become layers in the repository,** composed in
  this order:

  ```
  fleet.json                       defaults for every target
  hosts/<host>/<host>.json         the host's base
  envs/<env>.json                  defaults for every target in <env>
  hosts/<host>/<host>-<env>.json   the host in that environment
  ```

  An environment's defaults come after a host's base, so the lab's
  certificate authority, domain and mail sink apply to every lab
  variant; the host's own environment file comes last, for what is
  particular to it.

### 7.4 Merging that removes and replaces

- **Today lists concatenate:** a variant can add to a base's list, but
  cannot drop or replace an entry. Thin variants then turn into
  duplicated ones.
- **JSON Merge Patch's rules instead** (RFC 7386): objects merge key by
  key, `null` deletes a key, anything else replaces, lists included.
- **Collections become maps keyed by name** (users, keys, peers), so a
  variant adds, overrides or removes one entry. A base:

  ```json
  { "users": { "alice": { "uid": 1000 }, "bob": { "uid": 1001 } } }
  ```

  and a lab variant that removes bob and adds a tester:

  ```json
  { "users": { "bob": null, "tester": { "uid": 1100 } } }
  ```

- **Variant maps** (os's per-role overrides) keep working: they are
  objects.

### 7.5 Exports: data between targets

- **Today targets cannot read each other's data,** and the original
  fleet copies it by hand: each host's vpn configuration carries its
  peer's public key.
- **The factory needs it everywhere:** VPN meshes, DNS zones, a
  cluster's members knowing each other, checks run from another machine
  (10.1).
- **A task declares exports:** the facts other targets in the same
  environment may read, such as an address or a public key. A template
  reads its peers' exports; the vpn task loops over every target in its
  environment that has the vpn task.
- **The coupling stays narrow.** Only exported facts cross, so a change
  to one changes exactly the specs that read it, and the digests follow.
- **Secrets are never exported.** A public key may be; its private key
  stays a secret.

### 7.6 Clusters

- **One base, several members.** stoa-1, stoa-2 and stoa-3 share a base
  and each have their own data (an index, an address); each sees the
  others through exports.
- **The lab's experiments name clusters** (`factory.md`, 8.1), and a
  revision with several hosts maps to several targets (`factory.md`,
  5.3).

### 7.7 What a target is, apart from where it runs

- **Today a file's place decides the kind:**
  `hosts/<host>/<host>-<env>.json` is a host;
  `hosts/<host>/machines/<name>.json` is a container, which stands
  alone, with no base and no environments.
- **The factory runs one target in different places:** mail-lab in a
  container of a lab pool, mail-prod on a VM. So containers compose a
  base and environments as hosts do, and placement (bare metal, a VM, a
  container on a named host) is data of the environment.
- **Placement changes the build:** a container has no kernel or
  bootloader and bootstraps differently. That difference is part of the
  lab's gap, known and measured (`factory.md`, 5.3).
- **Which containers a host runs** is data in that host's
  configuration, as the original fleet's `machines` task already has
  it.

### 7.8 New kinds of task data

- **Checks** (10.1): what healthy means for what the task installs.
- **Software built in the factory.** Beside today's downloads
  (`{name, version, url, sha256}`), software may name an artifact: by
  digest, pinned by a change as a snapshot bump is; or "built from this
  head", which the lab resolves to a digest before rendering
  (`factory.md`, 8.5).

### 7.9 Smaller tweaks

- **Real directory trees.** `tasks/ssh/etc/ssh/sshd_config.d/local.conf`,
  not `tasks/ssh/etc/ssh___sshd_config.d___local.conf`: agents and
  diffs read real paths.
- **No templates completed at deploy.** What they read (the spec's
  metadata) is known when rendering. Templates completed at activation
  stay, limited to the slot's own variables (its label).
- **Explicit settings in place of magic task names.** Today the
  `container` task changes how the system is bootstrapped (now
  placement, 7.7), the `desktop` task adds apt's `contrib` component, and
  `host.hardware == "7th-gen"` adds `non-free`; apt's components become a
  setting.
- **Jinja's syntax stays,** since agents know it from Ansible and Salt,
  as a strict subset in hearth's own engine (6.2, item 8).
- **Shell fragments become the exception.** `customize.sh` and
  `software.sh` stay as escape hatches; the common cases become
  declarative: systemd presets, users (`sysusers.d`), software unpacked
  to a path.
- **Strict JSON stays,** with `_note` for annotations.

## 8. Paying only for what changed

`factory.md`'s rule (section 6) holds as it is: every piece of work is
keyed by the digests of its inputs, part by part, and redone only for
the parts whose digests changed. In hearth:

- **Resolution** is redone only when the sources or the requested
  packages change.
- **Layers** are rebuilt from the lowest one whose key changed, on a
  snapshot of the layer beneath it.
- **Activation** is the cheapest that covers what changed. If the
  kernel's and the initramfs's digests are unchanged, a soft reboot;
  otherwise a full reboot; for a container, a restart.
- **Caches:** apt-cacher-ng for packages; earlier outputs and their
  layers; software downloads by digest.

## 9. Slots

```
empty → preparing → prepared → trial → blessed
                                  └──→ fell back
```

- **Prepare:** snapshot an output into the inactive slot, complete the
  templates left to activation, deliver secrets, and record the slot's
  digests.
- **Trial:** the boot default stays on the old slot. The new one is
  tried once:
  - by a soft reboot into it (systemd's `/run/nextroot`) when the kernel
    is unchanged;
  - by a one-shot boot otherwise: EFI's `BootNext`, or systemd-boot with
    unified kernel images, which counts boots itself. BIOS machines with
    GRUB are the awkward case (section 13).
- **Bless:** once the new slot passes the target's checks (10.1), the
  boot default moves to it. hearth blesses its own slot, so a broken
  hearth release cannot bless itself and cannot strand a machine. The
  engine may hold a bless while a canary bakes.
- **Fall back:** any failure during the trial ends in a full reboot,
  which boots the default, still the old slot.
- **Which slot runs is read from the mount table,** not from
  `/proc/cmdline`: after a soft reboot, the kernel's command line still
  names the old slot.
- **Three facts, kept apart:** the slot running, the boot default, and
  the slot on trial.

## 10. Operating

What follows a rollout, for as long as a target runs. hearth is the
machine's end of it; the rest is the run side's (temper's `factory.md`,
section 9).

### 10.1 One definition of healthy

- **Each task declares its checks,** as data in the IaC repository:
  what healthy means for what the task installs. For postfix: its unit
  active, a message sent and delivered, its DKIM signature verified,
  TLS offered, the relay closed.
- **The same checks serve four uses.** Proving, deploying and operating
  are one activity at different stakes (`factory.md`, section 3):
  - the lab's experiments, on throwaway machines;
  - blessing a slot (section 9);
  - a rollout's bake;
  - watching the target for as long as it runs.
- **Where a check runs:** a check of the machine runs on it, by its
  hearth. A check of the service as its users see it (SMTP from
  elsewhere, DNS, a certificate close to expiry) runs from another
  machine, by that machine's hearth.

### 10.2 Facts go up; telemetry is fetched

- **Decisions get small typed facts,** over the channel: each check's
  result, resource alarms (disk, memory, pressure), units that fail,
  drift. Liveness costs nothing: a machine that stops redialling is
  down.
- **Logs and metrics stay on the machine.** The journal already lives
  on a persistent subvolume (`@<target>.srv.core/journal`), so it
  outlives slot switches. hearth keeps a bounded window of the
  machine's signals and of its checks' results, and serves both when
  asked: a triage agent or a person reads them through the engine.
- **No central store for metrics or logs, to start.** If history across
  the fleet is needed later, that store is a service the factory
  deploys from its own IaC, like any other.

### 10.3 What touches a machine

Work after a rollout is of three kinds:

- **Changes, through temper:** security updates (a bump of the Debian
  snapshot), configuration fixes, capacity.
- **Operational effects,** keyed and granted by policy. They start
  small, with two: *roll back* (a slot switch, section 9) and *restart a
  unit*. Any other waits until it is needed.
- **The run side's standing procedures:** convergence, certificate
  renewal, backups.

Nothing else touches a machine: no shell, and no hand.

### 10.4 Persistent data

os gives data a layout (`@<target>.srv.data` and its siblings) and
nothing more.

- **Backups:** read-only snapshots of a target's data subvolumes, on a
  period, sent to a backup peer as incremental streams against the
  previous one. The data belongs to the target's services, not to the
  builder, so this is the activator's work (4.1), and it may use btrfs
  send, which needs root.
- **Restore** becomes an effect when backups are built, granted by
  policy like the others.
- **Restore drills are experiments** on the run side's own throwaway
  machines: production's data never enters the lab.

## 11. Reproducible builds: deferred

They are a level of trust, and can be retrofitted later.

### 11.1 Why builds differ today

- **Time:** every file `deploy` writes gets the build's mtime, and
  nothing sets `SOURCE_DATE_EPOCH`.
- **Randomness and the build host:** os writes 8 KiB of the host's
  randomness into the root filesystem as a fake `/dev/urandom`;
  mmdebstrap copies the host's `resolv.conf` and `hostname`.
- **etckeeper:** `/etc/.git` holds commits with timestamps.
- **Logs and caches:** dpkg's and apt's logs, apt's lists, `.pyc`
  files, man-db's index.
- **Arbitrary shell:** maintainer scripts, `customize.sh` and
  `software.sh` can read the clock, the hostname or the processor count.
- **The filesystem:** btrfs's bytes (inode numbers, UUIDs, extents)
  never match, so "identical" has to mean the same tree, not the same
  image.

### 11.2 Kept from day one

These are not about trust:

- **Nothing unique and nothing secret in an output** (6.2, item 3):
  shared outputs would otherwise share identities.
- **Every output records a digest of its tree** (paths, types, modes,
  owners, extended attributes, contents, link targets) next to its
  input key. Two builds of one key differ for now; drift needs the
  digest anyway, and retrofitting becomes comparing two digests rather
  than changing how the store names outputs.
- **Outputs are trusted per plane:** a machine fetches only from its own
  plane's builders, so the lab's outputs never reach production.

### 11.3 Given up meanwhile

- **The lab tests a build of the same inputs,** not the bytes production
  runs.
- **`factory.md`'s comparison of patch and scratch (6.4) is noisy:**
  two builds differ in mtimes and logs, so it compares modulo a list of
  known differences, or waits. Only the packages layer needs it (6.2,
  item 2).

### 11.4 Retrofitting

- `SOURCE_DATE_EPOCH` per layer, derived from the layer's key (the
  Debian snapshot's timestamp), never from the commit, so an unchanged
  layer stays identical across commits. mtimes newer than it are
  clamped; packages keep their own.
- Sealed build steps: fixed hostname, `resolv.conf`, timezone, locale
  and umask; no network but the apt proxy, and only for the packages
  layer.
- A short list of files to remove or normalize, as data, each with its
  reason and, where there is one, its Debian bug.
- The lab's gate: every change to the packages layer (a snapshot bump,
  a change to the package set) built twice from scratch on two builders,
  varying what should not matter (hostname, umask, directory order, as
  reprotest does). A difference blocks the change, with diffoscope's
  report as the finding, and is reported to Debian. A layer that cannot
  be fixed degrades alone, to "built, not reproduced".

### 11.5 What Debian proves

- **Package builds** are reproducible for most of the archive, which
  does not matter here: hearth installs Debian's binaries, pinned and
  verified, and builds no package.
- **Installation is not tested archive-wide.** Debian's live images are
  built twice and compared on Jenkins for bookworm, trixie, forky and
  sid, and the installed system inside them has stayed reproducible;
  trixie 13.2's regression was in its boot screens and firmware patterns,
  fixed for 13.3. mmdebstrap promises reproducible output given
  `SOURCE_DATE_EPOCH` and the same mirror content, and removes the usual
  offenders (logs, ldconfig's `aux-cache`, machine-ids). Cloud images
  are not tested.
- **So Debian proves the machinery every installation runs** (dpkg,
  debconf, triggers, systemd's presets, ldconfig, ca-certificates,
  Python's byte-compilation), not the server packages hearth installs.
  The lab's gate (11.4) proves those.
- **Pinned inputs help:** Debian's live-image rebuilds must fit within a
  six-hour archive synchronisation; hearth's inputs are a snapshot, fixed
  for good.

References: the Debian wiki's
[ReproducibleInstalls](https://wiki.debian.org/ReproducibleInstalls) and
[live images](https://wiki.debian.org/ReproducibleInstalls/LiveImages)
pages; rb-general on
[trixie 13.2](https://lists.reproducible-builds.org/pipermail/rb-general/2025-November/003937.html)
and
[13.3](https://lists.reproducible-builds.org/pipermail/rb-general/2025-November/003947.html);
mmdebstrap's manual, on `SOURCE_DATE_EPOCH` and its cleanup.

## 12. Order of building

1. **render,** with section 7's format, checked against os's golden
   files on converted configurations;
2. **the store, with layers;**
3. **builds;**
4. **slots,** with trial and bless;
5. **the wire;**
6. **checks,** with facts up and telemetry served on demand;
7. **artifacts, peers and backups.**

The fakes, their conformance tier and the test budgets grow alongside,
from the first step (4.3).

## 13. Open questions

- **`/var` and layers.** dpkg's database lives in `/var`, and os splits
  `@<target>.var` off after the packages phase. How a layer pairs a root
  and its `/var`, and what of `/var` belongs to the slot rather than to
  persistent data.
- **hearth's own state:** the filesystem as the only truth (subvolumes
  and their records, read back at startup), or skein's kv as an index
  beside it.
- **Bootloaders:** systemd-boot with unified kernel images for EFI
  machines; for BIOS machines, whether GRUB can boot a slot once from
  btrfs, where it cannot write its environment block (to check).
- **The initramfs:** os uses tiny-initramfs; whether it stays, and how
  it fits unified kernel images.
- **Checks:** their format as task data; how a check of the service
  as users see it names the machine it runs from; what a check may do
  on a production machine (a message really sent and delivered).
- **Operational effects** beyond rolling back and restarting a unit,
  when one is needed.
- **Backups:** periods and retention, where the backup peer lives, and
  whether streams are encrypted for it.
- **Identity:** how a machine proves who it is to its engine and to its
  plane's secret store (`factory.md`, section 14).
- **The template engine:** exactly which part of Jinja the tasks use,
  and so what render's own engine takes.
- **The schema's form:** a file per task beside its templates, and how
  it states types (enumerations, maps keyed by name, references to
  secrets).
- **Exports:** how a task declares them, and whether any crosses an
  environment (the lab never reads production's, and the reverse
  seems as safe to forbid).
- **Clusters:** how a base declares its members, and how members are
  named as targets.
- **The rendered diff:** who posts it at a change's head (the lab, CI,
  or temper's gate), and how it is shown when a change touches many
  targets.
- **The fake machine's home:** shared with temper, so in skein or a
  crate of its own.
- **Disk provisioning as an effect:** for the lab's throwaway machines
  (virtual disks), beside the local command for bootstrap.
- **Artifacts:** the second recipe (a build environment, sources and a
  command, run in a container of the environment's output, with its
  compile cache on a persistent volume), and how its outputs are stored.
- **Peers:** change sets over HTTP (4.6), and a cache per pool or per
  site later (`factory.md`, section 14).
- **The activator's interface:** exactly its operations, and how it
  checks that a request names an output its builder's store holds.
- **Budgets:** each tier's ceiling, and the local mirror: which
  packages, pinned how, fetched by whom.
