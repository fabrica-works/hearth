# hearth: brainstorm

Draft, 2026-10-09. It records the direction of a design discussion
on 2026-10-09, the first about hearth. It starts from temper's
`factory.md` (a draft vision of the whole software factory) and from os
(`~/src/rust/os`), whose tools hearth replaces. Nothing here is adopted.
Section 10 is the order of building; section 11 lists what is open.

## 1. In one page

- **hearth is the factory's build daemon** (temper's `factory.md`,
  section 7). It makes outputs from inputs, keyed by digest, and runs
  them in slots. One process runs on every machine: hosts, the lab's
  pool hosts and its builders.
- **It replaces os.** os's four tools (`manifest`, `deploy`,
  `provision`, `ek`) become one domain in one daemon. os stays a
  reference for its features and its golden files, not for its code.
- **It is built on skein** and follows `programming-model.md`: one
  loop, step functions, and a domain complete enough to run whole on a
  fake machine. The heavy work stays in child processes.
- **The engine decides, hearth carries out.** The run side's engine, or
  the lab's, orders a target prepared or activated, and hearth does it.
  With no engine, hearth converges to a pinned revision on its own.
- **One wire:** skein's channel, dialed out by hearth to its plane's
  engine. Bytes move by digest, outside the channel.
- **Outputs are layers keyed by digest:** packages, then software, then
  etc. A change rebuilds only the layers above the lowest one it
  touched, on a snapshot of the layer beneath.
- **Slots try before they commit.** A new slot boots on trial. The boot
  default moves only when the slot is blessed, so a failure falls back
  to the old slot on its own.
- **An output holds nothing unique and nothing secret,** so it can be
  shared between machines and served by digest.
- **Reproducible builds are deferred.** They are a level of trust, and
  can be retrofitted without changing the store, the wire or the slots
  (section 9).

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

### 4.1 A process per machine

- **One process per machine, as root, on skein's loop.** It manages
  disks, subvolumes and boot entries, so it needs no sudo.
- **The heavy work stays in child processes:** mmdebstrap, apt, btrfs,
  git, efibootmgr. hearth schedules them, bounds how many run at once,
  and keeps the store.

### 4.2 The layers

- **Domain:** pure step code, in modules that keep os's boundaries as
  crate boundaries rather than binaries:
  - *render:* configuration and tasks at a revision, to parts with
    digests (os's `manifest`);
  - *store:* outputs by digest, their layers, what pins them, and
    collecting what nothing pins;
  - *build:* plans how to make an output from a spec, from the closest
    layer it has;
  - *slots:* prepare, trial, bless, fall back (section 8);
  - *disk:* partitioning and formatting, for bootstrap and the lab's
    throwaway machines (os's `provision`);
  - *drift:* the live slot compared with the output it came from;
  - later, *artifacts* (the second recipe) and *faults* (section 5.2).
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

### 4.3 Tested on a fake machine

The domain runs whole in worlds of fakes (`testing-strategy.md`). Its
fake is a machine: subvolumes and snapshots, a package universe drawn
from the seed, firmware boot variables, reboots and power cuts. A world
can cut the power during a prepare, fail a boot on trial, fill the disk
mid-build, or drop the engine during an activation, and check that the
machine always comes back bootable.

This is the machine fake temper already plans (temper's `testing.md`,
section 3.2), so the two should share it. os's real tests need sudo and
feature gates; this is the largest gain in quality over it.

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
  a peer or a pool's cache with a plain HTTP GET, or as a btrfs send
  stream against a parent the receiver already holds. Content-addressed
  bytes need no session, and any cache can serve them.
- **Facts and events are typed.** os's tab-separated output and its
  exit codes go; the command-line client may still print a table.

## 6. From os

### 6.1 Kept

- **The configuration model:** hosts as a base, environments and an
  overlay, composed in that order; tasks; strict JSON; inert templates;
  `etc.json`'s declarative rules and `require` guards. It is the IaC
  language, and it is good.
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
4. **Slots get a state machine** (section 8).
5. **ek folds into hearth.** The helper on a live host is the daemon.
6. **Typed facts and events** replace the tab-separated output (5.3).
7. **The fleet leaves the code.** Four details of one fleet are in os's
   code and move into tasks:
   - `host.hardware == "7th-gen"` switching on non-free components;
   - the Europe/Lisbon timezone and en_US locale preseeds;
   - the persistent `@<target>.srv.core/journal` and `iwd` directories;
   - masking gettys in containers.
8. **Little of os's code ports.** Step crates are `no_std`, so
   MiniJinja, serde, clap and anyhow do not fit. Rendering needs a small
   inert template engine of its own, for the Jinja subset the tasks use.
   What ports is the design and the tests: os's golden files become
   render's conformance suite, and os serves as an oracle while hearth
   catches up, the same configuration giving the same parts.

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

## 7. Paying only for what changed

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

## 8. Slots

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
    GRUB are the awkward case (section 11).
- **Bless:** once the new slot is healthy, the boot default moves to
  it. hearth blesses its own slot, so a broken hearth release cannot
  bless itself and cannot strand a machine.
- **Fall back:** any failure during the trial ends in a full reboot,
  which boots the default, still the old slot.
- **Which slot runs is read from the mount table,** not from
  `/proc/cmdline`: after a soft reboot, the kernel's command line still
  names the old slot.
- **Three facts, kept apart:** the slot running, the boot default, and
  the slot on trial.

## 9. Reproducible builds: deferred

They are a level of trust, and can be retrofitted later.

### 9.1 Why builds differ today

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

### 9.2 Kept from day one

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

### 9.3 Given up meanwhile

- **The lab tests a build of the same inputs,** not the bytes production
  runs.
- **`factory.md`'s comparison of patch and scratch (6.4) is noisy:**
  two builds differ in mtimes and logs, so it compares modulo a list of
  known differences, or waits. Only the packages layer needs it (6.2,
  item 2).

### 9.4 Retrofitting

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

### 9.5 What Debian proves

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
  The lab's gate (9.4) proves those.
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

## 10. Order of building

1. **render,** checked against os's golden files;
2. **the store, with layers;**
3. **builds;**
4. **slots,** with trial and bless;
5. **the wire;**
6. **artifacts and peers.**

The fake machine grows alongside, from the first step.

## 11. Open questions

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
- **Health for blessing:** hearth's local check (systemd's
  `boot-complete.target`), or the engine's verdict from observability for
  a canary that bakes; probably both, the engine able to hold a bless.
- **Identity:** how a machine proves who it is to its engine and to its
  plane's secret store (`factory.md`, section 14).
- **The template engine:** exactly which part of Jinja the tasks use,
  and so what render's own engine takes.
- **The fake machine's home:** shared with temper, so in skein or a
  crate of its own.
- **Disk provisioning as an effect:** for the lab's throwaway machines
  (virtual disks), beside the local command for bootstrap.
- **Artifacts:** the second recipe (a build environment, sources and a
  command, run in a container of the environment's output, with its
  compile cache on a persistent volume), and how its outputs are stored.
- **Peers:** HTTP by digest or btrfs send streams, and a cache per pool
  or per site later (`factory.md`, section 14).
