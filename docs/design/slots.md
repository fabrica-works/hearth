# Slots

Provisional, 2026-10-09. A slot's life on a machine: what a slot is and
what its record holds, the four facts, the state machine, prepare step
by step, trial (by a reboot, a soft reboot, or live, with no reboot),
bless, fall back, roll back, drift, what crashes and power cuts do at
every step, and containers. M3 builds it on EFI, with a minimal bless
and live changes; M3b adds trials through GRUB; containers and pool
hosts are M5, live changes in a container included. The layout slots
live on is machine.md's; the process that carries out every transition
is activator.md's. What is open is in section 15.

## 1. In one page

- **A slot is a read-only root and its own `/var`,** two subvolumes
  labelled `a` or `b`, holding one output prepared for this machine:
  shifted to real ids, with this machine's identity and secrets, fstab
  and units written in, then sealed.
- **Four facts, kept apart:** the slot running (the root subvolume's
  identity, never `/proc/cmdline`), the default (what the firmware or
  GRUB boots), the slot on trial (the activator's marker), and the live
  state (hearth's overlays on `/etc`, `/usr` and `/opt`, read from the
  mount table).
- **One state machine per slot,** written as a total table: empty,
  preparing, prepared, trying, on trial, on live trial, live-blessed,
  blessed, fell back. Its invariant: **the default always boots a
  blessed slot,** whatever crashes and power cuts happen, at any step.
- **Prepare is seven steps,** each in the activator: snapshot the
  output, shift it while hashing it, split `/var` off as a reflink copy,
  write identity and secrets, write the slot's files, write its boot
  files and entries, verify and seal. Every path in the slot is resolved
  from the snapshot's own descriptor, never through a link the output
  planted. A slot left preparing is deleted and made again.
- **A trial runs the new slot once.** A soft reboot into `/run/nextroot`
  when the kernel and its command line are unchanged; otherwise a
  one-shot reboot, `BootNext` or GRUB's `next_entry`. The default does
  not move.
- **Configuration and in-house software change live, with no reboot**
  (6.6 to 6.9). When a change touches only `/etc`, `/usr` and `/opt`
  over the same packages, and every changed path has a declared
  consumer, the activator builds a delta of the new slot, mounts it over
  the running slot's hierarchies with an atomic swap, and reloads,
  restarts or starts the declared units. Its bless sets the one-shot to
  the new slot's trial entry instead of moving the default, so the slot
  still boots natively on trial before it becomes the default.
- **Every trial ends:** in a bless, or in a reboot into the default,
  from `panic=10`, a hardware watchdog, a job timeout on the trial's
  target, or a dead-man timer the old activator wrote into the slot. The
  dead-man is armed by a kernel command-line token or a marker in `/run`,
  so no failed mount and no emergency mode can disarm it.
- **The new slot blesses itself:** its own builder asks its own
  activator, once the boot has reached its target with no failed unit.
  A broken hearth release cannot bless itself, so it cannot strand a
  machine.
- **Fall back is recorded, not decided:** the default slot's activator
  finds the marker naming a slot that is not running, and records that
  it fell back.
- **Roll back is a trial of the other blessed slot,** conditional on
  the facts it was decided from, never a blind move of the default.
- **Drift compares the live slot with its record,** under stable check
  ids, leaving out `/var`, the persistent subvolumes and other mounts.
  Drift is reported, never repaired: a fix is a new slot.
- **Containers have slots on their host [M5]:** prepared the same way,
  activated by a restart or live with the same mechanics, tried until
  their unit is stably active and their boot has no failed unit, and
  blessed by the host's hearth, since none runs inside.

## 2. Slots and targets

### 2.1 A slot

- **Two subvolumes at the top level,** `@<target>.<label>` (the root)
  and `@<target>.<label>.var` (its `/var`), with the label `a` or `b`
  (machine.md, section 3).
- **One output at a time,** prepared for one machine: the output's tree
  is the same on every machine that builds its key; the slot adds this
  machine's identity, secrets, fstab and units, and splits off `/var`.
  So a slot is not the output, and has a tree digest of its own.
- **Read-only once prepared.** Its root's subvolume is sealed with the
  read-only flag, and mounted `ro`; only its `/var` and the persistent
  subvolumes are written while it runs. Nothing is changed in place: a
  fix, a new secret or a new command line is a new prepare.
- **One machine runs one target,** so its slots are `@<target>.a` and
  `@<target>.b`. A host's containers have slots of their own on the
  host's top level [M5] (section 12).

### 2.2 The slot record

The activator keeps one record per slot in `@hearth.state`
(activator.md, section 7), written atomically:

```
slot record    slots/<target>.<label>
  state              empty | preparing | prepared | trying | on trial | on live trial | live-blessed
                     | blessed | fell back
  output             the output's key: its name in the store
  prepare key        H("prepare", output, data, secrets and boot digests, window, units allowed to fail,
                     the identity's generation, the secret store's generation)
  window, allowed    the trial's window and the units whose failure does not block a bless (7.1)
  backups [M6]       from the data part, each backed-up subvolume's peer and age recipient:
                     the only recipient a send may use (activator.md, 5.11; operating.md, 7.2)
  output tree        the output's tree digest, as its store record gives it and the shift found it
  slot tree          the slot's own tree digest: the sealed root after prepare; root-only
  reported tree      the same, each secret and identity file's content digest replaced by
                     H(name, generation): the one the facts carry (5.7)
  var tree           the slot's /var as copied, secrets beneath it included
  kernel             version; the four digests of 6.1; the initramfs file's digest, to check its ESP copy
  command line       as written for this slot (machine.md, section 6)
  subvolumes         root and var: id and uuid
  boot               EFI: its two entry numbers, normal and trial, and its ESP files' digests;
                     BIOS: its fragment's digest
  parts              digests of the data, secrets and boot parts it was prepared with
  generations        the identity's; the secret store's
  trial              last attempt: kind (soft, one-shot, live), phase reached, outcome, the failed
                     boot's id; for a live one, the step that failed and the units acted on
  times              prepared, tried, blessed, fell back (wall clock)
  costs              time per prepare step
```

- **The manifest** beside it, `slots/<target>.<label>.manifest`, is a
  tree file (store.md, 6.3) of the sealed root: one entry per path in
  tree order, with its local inode and change time as the seal left
  them. Drift compares against it (section 10).
- **The record inside the root,** `/.hearth/slot`, holds the target, the
  label, the output, the prepare key and the root subvolume's uuid. It
  lets a running slot confirm which slot it is (section 3), and it is
  covered by the slot's tree digest like any file.
- **Nothing secret leaves the state:** secrets appear by name, and the
  store by its generation. The slot tree digest covers each secret's
  content digest, so only the reported tree digest is ever sent.

### 2.3 Which slot

- **Prepare takes the slot that is neither running, nor the default,
  nor on trial.** On a machine at rest, running and default are the
  same slot, so the other one is free. When they differ (the firmware
  fell through to the other slot, or a bless was cut), prepare is refused
  until the facts agree again: the running slot's builder blesses it
  again, which moves the default to the slot that runs (7.5).
- **A `live-blessed` slot is neither running nor the default,** so a
  later change prepares over it while its delta, a subvolume of its own,
  keeps serving (6.9).
- **The builder chooses nothing:** it asks to place an output, and the
  activator answers with the slot it used. converge decides whether a
  new prepare is needed by comparing the desired output and parts with
  the slots' records in the facts (wire.md, section 4).

### 2.4 The live record

While a live state exists (6.6), the activator keeps one more record in
`@hearth.state`, written atomically, beside the slot records:

```
live record    live/<target>
  base               the running slot: its label and its root subvolume's uuid
  delta              @<target>.live.<n>: n, the subvolume's id and uuid
  previous           the delta it replaced, kept until the next live bless (6.9); or none
  carries            the output key and prepare key the delta was built from, and the slot that
                     holds them, while one does
  hierarchies        which of /etc, /usr and /opt it overlays
  swapped at         wall clock; the boot id and soft reboot count at the swap
```

- **The manifest beside it,** `live/<target>.manifest`, is the merged
  view's expected entries, made at L2 (6.8): the base's manifest under
  `etc/`, `usr/` and `opt/`, with the change's paths replaced or
  removed. Drift compares the live view against it (section 10), and it
  survives the slot being prepared over (6.9).
- **Written at every swap** (6.8, L5) and every swap back (8.5);
  removed when no overlay remains: by a live fall back to the bare
  base, or by the start that finds the overlays gone, after a reboot or
  a soft reboot into another root.
- **The live fact is read from the mount table** and checked against it
  (section 3); the record says what hearth made, the mount table what
  runs.

## 3. The four facts

| Fact | Read from | When it can be wrong |
|---|---|---|
| **running** | `BTRFS_IOC_GET_SUBVOL_INFO` on `/`: the root subvolume's uuid, matched with the slot records; cross-checked with `/.hearth/slot` | never; `none` on a live system or a root hearth did not prepare |
| **default** | EFI: the first of the target's entries in `BootOrder`; BIOS: `default` in the valid environment file (machine.md, 4.4 and 5.6) | a firmware that reorders or forgets entries: reported, and put right at start |
| **trial** | the trial marker in `@hearth.state` (6.4) | never: it is the activator's own record |
| **live** | `/proc/self/mountinfo`: an overlay on `/etc`, `/usr` or `/opt` with source `hearth-live`, whose layers are a delta `@<target>.live.<n>` over the running slot's directory; checked against the live record (2.4) and the marker | after a person's soft reboot, which keeps the overlays on `/etc` and `/usr` and drops `/opt`'s (to verify): the disagreement is found at start, and ends in a full reboot (section 11) |

- **Running stays the root subvolume.** A live state changes what
  `/etc`, `/usr` and `/opt` show, never which subvolume `/` is: prepare,
  the guards and `slot.root.*` still name the base.
- **What runs, derived:** the prepare key the machine's files come from,
  the live record's while a verified live state is in place, else the
  running slot's; and the effective slot, the slot that holds it, if
  any. converge compares the desired prepare key with it (wire.md, 4.2).
  After a later live change falls back, no slot may hold it (6.9).
- **Running is never read from `/proc/cmdline`.** After a soft reboot the
  kernel's command line still names the slot the kernel was booted for.
  The mount table would do on btrfs, but the subvolume's uuid is exact,
  unprivileged, and needs no parsing.
- **Observations beside the facts,** read at every start: `BootNext` and
  `BootCurrent` (EFI) or `next_entry` (BIOS), the boot id
  (`/proc/sys/kernel/random/boot_id`), the manager's `SoftRebootsCount`
  (`systemctl show --property=SoftRebootsCount`), and the booted kernel:
  the four digests of 6.1 of the slot running at the first start of
  each boot id, since a full boot always runs the kernel of the slot it
  boots.
- **Facts are read afresh** at every start of the activator and after
  every operation, and reported to the builder when they change
  (activator.md, section 4) and to an engine [M4] (wire.md, section 6).
  An event such as `fell back` is a hint; the facts are the truth.

## 4. The state machine

Each slot has one state; the machine adds guards across both slots.
States, as in `programming-model.md`, 5.4, each holding exactly what it
needs:

| State | Holds | Meaning |
|---|---|---|
| `empty` | nothing | no subvolumes, or none worth keeping |
| `preparing` | prepare key, step | prepare in flight; never booted, never bootable |
| `prepared` | record | complete, verified, sealed; not the default |
| `trying` | record, attempt, kind, phase | a trial armed; the machine still runs the default |
| `on trial` | record, attempt | the slot runs on trial |
| `on live trial` | record, attempt, phase, live: base, delta, previous delta, unit plan | armed, then its files run over the running slot (6.8); nothing boots it yet |
| `live-blessed` | record, live: base, delta | passed a live trial; the one-shot names its trial entry; never booted natively (7.6) |
| `blessed` | record | passed a bless; the default, or the previous default |
| `fell back` | record, attempt | tried and not blessed; must be prepared again |

Events:

| Event | What it is |
|---|---|
| `prepare K` | a place request for this slot, whose prepare key is K |
| `prepared` | prepare's last step passed (5.7) |
| `prepare failed` | a prepare step failed, typed |
| `try` | a trial request for this slot, or a roll back to it |
| `try live` | a live trial request for this slot (6.8) |
| `start here` | the activator starts (after a boot, a soft reboot or its own restart) and this slot runs |
| `start elsewhere, same boot` | the activator starts, another slot runs, and the boot id and soft reboot count equal the trial marker's |
| `start elsewhere, new boot` | the activator starts, another slot runs (or none), and they differ; with the marker's boot id and a higher soft reboot count, while a live state was recorded, it is a person's soft reboot (6.8) |
| `bless` | a bless request from the builder running in this slot |
| `bless live` | a live bless request from the builder running over this slot's live state (7.6) |
| `fall back` | a request to end this slot's trial now; for a live one, the live fall back (8.5) |
| `first bless` | bootstrap's bless: no slot is blessed and none runs |

The table, total: every cell a transition, `refuse` (a typed answer that
changes nothing), `ignore`, a `fault` (reported; every mutating
operation refused until a start finds a blessed slot running and the
facts agreeing), or `impossible` with the reason, only where the
activator's own order of steps excludes the cell.

| state ↓ event → | `prepare K` | `prepared` | `prepare failed` | `try` | `try live` | `start here` | `start elsewhere, same boot` | `start elsewhere, new boot` | `bless` | `bless live` | `fall back` | `first bless` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `empty` | `preparing` | impossible: no prepare runs | impossible: no prepare runs | refuse: not prepared | refuse: not prepared | fault: state lost; every operation refused | ignore | ignore | refuse | refuse | refuse | refuse |
| `preparing` | same K: answered when done; else refuse: busy | `prepared` | `empty`: subvolumes deleted | refuse | refuse | fault: a person booted its entry from the firmware's menu after a cut; it is never the default, in `BootOrder` or named by a one-shot | `empty`: deleted, the activator crashed | `empty`: deleted, the power was cut | refuse | refuse | refuse | refuse |
| `prepared` | same K: done; else `preparing`, old subvolumes deleted first | impossible | impossible | `trying` | eligible (6.7): the delta built and verified (6.8, L1 and L2), then `on live trial`; L1 or L2 failed: the delta deleted, failed; not eligible: refuse, naming the rules | fault: booted with no trial, by hand; reported | ignore | ignore | refuse: not on trial | refuse: not on trial | refuse | `blessed`, and the default |
| `trying` | refuse: busy | impossible | impossible | same attempt: done; else refuse | refuse: busy | `on trial`: marker set `running`; one-shot checked consumed | `prepared`: the reboot never came; one-shot cleared, `/run/nextroot` unmounted, marker removed | phase `arming`, or one-shot still set: `prepared`, one-shot cleared, reported; else `fell back` | refuse: not running | refuse | `prepared`: as `start elsewhere, same boot` | refuse |
| `on trial` | refuse: running | impossible | impossible | refuse | refuse | `on trial`: the activator restarted within the trial | fault: a person switched roots by hand | `fell back` | criterion holds: `blessed`; else refuse, saying which part failed | refuse: not a live trial | reboot into the default (8.1); the next start decides | refuse |
| `on live trial` | refuse: on trial | impossible: no prepare runs | impossible: no prepare runs | refuse: busy | same attempt: done; else refuse: busy | fault: booted natively with no trial, by hand; it was never armed | phase `arming`: `prepared`, disarmed, the delta deleted; later phases: `reboot -ff`, and the next start decides (section 11) | a person's soft reboot: `reboot -ff`; else phase `arming`: `prepared`; else `fell back`; the delta deleted | refuse: a live trial is blessed live | phase `running`, settled, criterion holds: `live-blessed` (7.6); else refuse, saying which part failed | the live fall back (8.5): `fell back`; a step that fails: `reboot -ff` | refuse |
| `live-blessed` | neither running nor default: `preparing`, its one-shot and marker cleared and old subvolumes deleted first; the delta keeps serving (6.9) | impossible | impossible | `trying`: an ordinary trial of it now, soft or one-shot (6.1) | refuse: already live | the one-shot booted it: `on trial`, marker set `running` (6.8, L9) | the live fact agrees with the live record: ignore; else `reboot -ff`, into its trial | a person's soft reboot: `reboot -ff`; the one-shot never set, or still set: `prepared`, one-shot cleared, reported; else `fell back`, never reached hearth | refuse: not running | same attempt: done; else refuse | the live fall back (8.5), one-shot cleared first: `fell back` | refuse |
| `blessed` | running or default: refuse; else `preparing`, old subvolumes deleted first | impossible | impossible | not the default: `trying` (a roll back); else refuse | refuse: a roll back is an ordinary trial | ignore; not the default: reported, and the builder blesses it again (7.5) | ignore | ignore | running and not the default, criterion holds: the default moves here (7.5); same attempt: done; else refuse | refuse | refuse | refuse |
| `fell back` | `preparing`, old subvolumes deleted first | impossible | impossible | refuse: prepare again | refuse: prepare again | fault: booted with no trial; reported | ignore | ignore | refuse | refuse | refuse | refuse |

Guards across both slots, checked before a cell is taken:

- **One mutating operation at a time** (activator.md, section 4), so at
  most one slot is `preparing`, `trying`, `on trial` or `on live trial`.
- **`prepare`** names a slot that is neither running nor the default.
- **`try`** names a slot whose other slot runs and is the default.
- **`try live`** names a `prepared` slot whose other slot runs, is the
  default and is blessed, with no trial; a live state, if one exists,
  has the running slot as its base; and the change is eligible (6.7).
- **`bless`** names the running slot, which is on trial, or blessed but
  not the default; it can only come from the builder of the running
  slot, the only one on the link.
- **`bless live`** names the slot on live trial; it comes from the
  builder on the link, the base's, spawned anew after the swap (6.8,
  L6).
- **`first bless`** only while running is `none` and neither slot is
  blessed: bootstrap (machine.md, 8.2).

The invariant, checked by every world (section 13):

- **the default is blessed;**
- **a slot that is `preparing` is never the default,** never in
  `BootOrder`, and never named by a one-shot or `/run/nextroot`;
- **a one-shot exists only while a slot is `trying` or
  `live-blessed`,** and names that slot (a `live-blessed` slot's trial
  entry);
- **a live state sits on the running slot,** which is blessed and the
  default, and the default never moves while one runs, its bless
  included;
- **a live state is what its record says:** the overlays in the mount
  table name the record's delta over its base, and the merged view
  equals the record's manifest;
- **a trial ends:** every `trying`, `on trial` or `on live trial` slot
  leaves that state within its window, or at the next start.

## 5. Prepare

Inputs, from the place request (activator.md, 5.1): the slot, the
output's key and tree digest, its kernel version and digests, the
`data`, `secrets` and `boot` parts, the package users, the trial's
window, the units allowed to fail, and the target's kind and firmware
from `meta`, or a container's host, with its parameters in its `boot`
part [M5] (12.1).

- **Identity first:** before anything else the activator makes every
  identity item the request declares and the store lacks (activator.md,
  5.7), then computes the prepare key (2.2), whose identity generation
  is then final. A request whose key the slot already holds is answered
  at once, as done.
- **Every path in the slot is resolved from the snapshot's root
  descriptor** with `openat2(RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS |
  RESOLVE_NO_XDEV)` (an ask of skein, README, 8.1), never by a path name
  in the activator's own namespace: a link the output plants at `/var`,
  `/etc` or a kernel's path cannot lead the activator to the running
  machine's files. An output whose `/var`, `/etc`, `/boot`,
  `usr/lib/modules/<version>` or a mount point's parents are not
  directories, or whose kernel, initramfs, `/etc/passwd`, `/etc/group`
  or `/etc/shadow` are not regular files, is refused; no error carries
  a file's content.
- **Each step starts by writing the activator's journal** (activator.md,
  7.3), so a crash finds the slot `preparing` and deletes it
  (section 11).

### 5.1 The snapshot

1. **Take the slot out of the firmware's fallbacks:** on EFI its normal
   entry leaves `BootOrder` if it is there (it is, when the slot holds
   the previous blessed output); on BIOS `previous` is cleared if it
   names the slot (8.4) [M3b]. A one-shot that names it, a
   `live-blessed` slot's (7.6), is cleared too: `BootNext` on EFI,
   `next_entry` on BIOS; then its marker. So nothing falls through to,
   or boots, a slot being written.
2. **Delete the slot's old subvolumes,** nested ones first: the children
   of its `.var` (systemd's `/var/lib/machines` and
   `/var/lib/portables`), found with `BTRFS_IOC_GET_SUBVOL_ROOTREF`; then
   the `.var`; then the root, read-only or not (the activator is root).
3. **Open the output by its key,** beneath
   `/run/hearth-activator/top/@hearth.store/` without following links
   (store.md, section 2), and check it (activator.md, 6.1).
4. **Snapshot it, writable,** with `BTRFS_IOC_SNAP_CREATE_V2` from the
   descriptor opened in step 3, into the top level as `@x1-prod.b`.
5. **Check the snapshot** (activator.md, 6.2): its parent uuid is the
   output's; the output is still read-only, with the same change
   generation. Record the new subvolume's id and uuid.

A snapshot is a metadata operation: milliseconds, and no data copied.

### 5.2 The shift

The output is owned on disk by the builder's subordinate range, base
`B` (store.md, section 9), and its extended attributes are in that
range's terms. The shift makes it owned by real ids, metadata only, in
one depth-first walk of the snapshot with a bounded stack
(`hearth-domain-tree`, a child of `hearth-domain-slots`). For each
inode, once (hard links are found by inode number):

1. **stat it without following links** (`statx`, `AT_SYMLINK_NOFOLLOW`);
   refuse any owner or group outside `[B, B + 65536)`, and any block or
   character device (none can be made in a user namespace);
2. **read `security.capability`** (regular files), and
   **`system.posix_acl_access`** and, for directories,
   **`system.posix_acl_default`**, before anything else: `chown(2)`
   clears capabilities;
3. **`lchown` to `uid - B`, `gid - B`;**
4. **restore setuid and setgid** with `chmod` if the mode had them (not
   on links): `chown(2)` clears them, for root too;
5. **write the capability back as v2:** a v3 capability with root id `B`
   (what a namespace's root writes) becomes v2 with the same sets; a v2
   one is written back as it was; a v3 with any other root id is refused;
6. **rewrite each ACL's `ACL_USER` and `ACL_GROUP` qualifiers** from
   `q` to `q - B`, refusing any outside the range;
7. **hash its contents** if it is a regular file (activator.md, 6.3),
   and **write its manifest entry,** the same entry the builder's tree
   helper wrote inside its namespace.

- **The ids land where the namespace saw them.** With `B` aligned to
  64 Ki, `uid - B` is the id inside the builder's namespace, and a
  capability or ACL read by root now reads as the namespace read it. So
  the digest of the shifted tree, computed as the walk goes, must equal
  the output's tree digest: that is the shift's own verification
  (activator.md, 6.3).
- **What it costs:** about 10 µs an inode for the metadata, so half a
  second to two seconds for a root of 50k to 100k inodes, plus hashing
  contents when the activator hashes again. Data extents stay shared
  with the store; btrfs copies the metadata leaves it touches, tens of
  MB per slot. Each step's time is recorded as a cost.
- **A cut leaves a half-shifted tree,** which nothing can tell from a
  whole one by looking; the slot is still `preparing`, so it is deleted
  (section 11).
- **Precedents:** systemd's `shift-uid.c` shifts ACLs and restores modes
  but leaves capabilities alone; Incus's `shift_linux.go` rewrites
  capabilities with a new root id.

### 5.3 /var

A layer is one subvolume, `/var` included (store.md, section 4); a slot
splits it off, so that dpkg's database travels with its slot and the
root can stay read-only.

1. **Make `@x1-prod.b.var`** with `BTRFS_IOC_SUBVOL_CREATE_V2`; its root
   inode takes the owner and mode of the output's `/var`.
2. **Copy the output's `/var` into it, sharing extents,** in one walk
   through descriptors: directories made, regular files cloned with
   `FICLONE` (an ask of skein, README, 8.1), links and hard links made
   again, and owners, modes, times and extended attributes copied as
   the shift left them. Anything but a directory, a regular file or a
   link beneath `/var` is refused. No `cp`: it resolves paths in the
   activator's own namespace.
3. **Empty the root's `/var`,** depth first, by `unlinkat` on directory
   descriptors; `/var` itself stays, as the mount point.
4. **Seed and empty each `data` path.** The first time the activator
   makes a persistent subvolume, `@x1-prod.<name>` (machine.md, 3.4), it
   seeds it with the output's content at that path, cloned the same way,
   so what packages put there (a database's cluster, a mail spool) is
   not hidden by an empty mount. Then, made or not, the slot's copy at
   that path is emptied to a mount point. An existing persistent
   subvolume is never touched.
5. **Make the mount points** that a sealed root cannot make later:
   `lib/hearth/state` and `lib/hearth/store` in the slot's `/var`; each
   `data` path, in the slot's `/var` if it is beneath `/var`, in the root
   otherwise. Missing parents are made `0:0 0755`.
6. **Amend the manifest:** the entries beneath `/var` leave the root's
   manifest for the `/var` manifest, whose digest is the record's `var
   tree`; the mount points join; the emptied `data` paths leave.

A copy, rather than a second snapshot of the output trimmed down to its
`/var`: trimming would unlink every other file of the output, about
15 µs each, while `/var` holds a few thousand files.

### 5.4 Identity and secrets

Written from the secret store in `@hearth.state` (activator.md, 7.1)
into the slot, through descriptors (section 5): every parent must
already be a directory in the slot; the file is created anew, replacing
a file at its path and refusing a link there. A path beneath `/var` is
written into the slot's `.var` subvolume, which 5.3 has made, and its
entry joins the `/var` manifest.

- **The identity** (format.md, 9.3), made before the prepare key
  (section 5):
  - the machine-id, every target's and never declared, at
    `/etc/machine-id`, `0:0 0444`: 32 hexadecimal digits and a newline.
    The journal's directory is named by it, so it must be the same for
    every slot of the target, and it must be written: on a read-only
    root an empty file means a new id at every boot;
  - each item of the `secrets` part's `identity` map (render.md, 4.3),
    at its `path`, with its `public_path`, mode and owners: an SSH host
    key in M3, such as `/etc/ssh/ssh_host_ed25519_key`. An output holds
    none (build.md, section 9).
- **Secret files** from the `secrets` part: name, path, mode, owner,
  group. Owner and group are names, resolved against the slot's own
  `/etc/passwd` and `/etc/group` after the shift, through bounded parsers
  (activator.md, 2.3); an unknown name, or a missing secret, fails the
  prepare and names them all.
- **Never readable by the builder:** a secret or identity file owned by
  `hearth`, or by a group `hearth` belongs to, and a secret whose mode
  lets others read it, are refused (format.md, 5.11 refuses them first).
- **Password hashes,** secrets too, for the users the `secrets` part
  names: the activator replaces the second field of those users' lines
  in the slot's `/etc/shadow`, keeping the file's owner, group and mode.
  It is the one edit hearth makes to a file the output holds, and it is
  a fixed operation: no other field and no other file.
- **What leaves the activator:** the identity's public halves, the
  secrets' names with present or missing, and the store's generation.
  Never a value, nor a digest of one (5.7).
- **Rotation:** an import bumps the secret store's generation, which
  changes the prepare key, so converge prepares the same output again
  into the other slot (wire.md, 4.6), and the new secret arrives by trial
  and bless like any change.

### 5.5 The slot's files

Written from typed data into the root before it is sealed, each into the
manifest, by the format crates of activator.md, 8.1:

| Path | Owner, mode | Content |
|---|---|---|
| `/etc/fstab` | `0:0 0644` | machine.md, 3.3 |
| `/.hearth/slot` | `0:0 0444` | the record inside the root (2.2) |
| `/.hearth/grub.cfg` [M3b] | `0:0 0444` | the slot's `linux` and `initrd` lines (machine.md, 5.4) |
| `/etc/systemd/system/hearth-trial.timer` | `0:0 0644` | the dead-man's timer |
| `/etc/systemd/system/hearth-trial.service` | `0:0 0644` | the dead-man's reboot |
| `/etc/systemd/system/hearth-trial-arm.service` | `0:0 0644` | starts the trial's target when the slot is on trial |
| `/etc/systemd/system/hearth-trial.target` | `0:0 0644` | the trial boot's job timeout |
| `/etc/systemd/system/sysinit.target.wants/hearth-trial.timer`, `…/hearth-trial-arm.service` | links | evaluated at every boot |
| `/etc/systemd/system.conf.d/zz-hearth-watchdog.conf` | `0:0 0644` | `RuntimeWatchdogSec=`, when the machine has a watchdog device; named to sort last |

An output that holds `/.hearth`, any of these unit names, or a drop-in
sorting after `zz-hearth-watchdog.conf` is refused: they are hearth's
(format.md, 5.11). The units, for slot `x1-prod.b` with a 10-minute
window (their exact behaviour is to verify on the VM tier):

```
# hearth-trial.timer
[Unit]
Description=hearth: end the trial of x1-prod.b unless it is blessed
DefaultDependencies=no
IgnoreOnIsolate=yes
ConditionKernelCommandLine=|hearth.trial=x1-prod.b
ConditionPathExists=|/run/hearth-trial/x1-prod.b

[Timer]
OnActiveSec=10min
AccuracySec=1s

# hearth-trial.service
[Unit]
Description=hearth: the trial of x1-prod.b was not blessed in time
DefaultDependencies=no
IgnoreOnIsolate=yes

[Service]
Type=oneshot
ExecStart=/usr/bin/systemctl reboot -ff

# hearth-trial-arm.service
[Unit]
Description=hearth: arm the trial boot of x1-prod.b
DefaultDependencies=no
IgnoreOnIsolate=yes
ConditionKernelCommandLine=|hearth.trial=x1-prod.b
ConditionPathExists=|/run/hearth-trial/x1-prod.b

[Service]
Type=oneshot
ExecStart=/usr/bin/systemctl start --no-block hearth-trial.target

# hearth-trial.target
[Unit]
Description=hearth: the trial boot of x1-prod.b
DefaultDependencies=no
IgnoreOnIsolate=yes
After=multi-user.target
JobTimeoutSec=10min
JobTimeoutAction=reboot-force

# zz-hearth-watchdog.conf, when /sys/class/watchdog/ lists a device
[Manager]
RuntimeWatchdogSec=30s
RebootWatchdogSec=10min
```

- **Armed by what exists before any mount:** the kernel command line's
  `hearth.trial=<target>.<label>`, which a one-shot boots with (6.3), or
  `/run/hearth-trial/<target>.<label>`, which the activator writes before
  a soft reboot (6.2) and which `/run` carries across it. No trial unit
  depends on a mount, so a failed `/var` cannot leave a trial unarmed.
- **Inert elsewhere:** a blessed slot's own boot has neither, and a
  fallback boot of the old slot names another slot.
- **Emergency mode does not disarm it:** `IgnoreOnIsolate=yes` keeps the
  timer and the target when systemd isolates `emergency.target`, and the
  timer alone still reboots (to verify that the target's pending job
  survives an isolate too).
- **Only the arming starts the target:** were it enabled for every boot,
  a blessed slot that took longer than the window to boot would be
  rebooted by its timeout, again and again. Queued early in the boot,
  its job times out unless `multi-user.target` is reached within the
  window.
- **It depends on nothing of the new release:** the units are written by
  the activator that prepares the slot, the old blessed one, and they
  use only systemd. A slot whose hearth never starts still reboots.
- **`OnActiveSec=` counts from the timer's start,** which a soft reboot
  starts afresh; `OnBootSec=` would count from the kernel's boot, which
  a soft reboot keeps.
- **After a soft reboot the command line keeps its token,** so a person's
  soft reboot of a slot that was booted on trial arms it once more, and
  ends in one full reboot into the default; every soft reboot hearth
  makes is a trial anyway.

### 5.6 Boot files and entries

- **EFI:** mount the ESP at `/run/hearth-activator/boot`; write
  `vmlinuz.x1-prod.b` and `initrd.img.x1-prod.b` from the slot's
  `boot/vmlinuz-<version>` and `boot/initrd.img-<version>`, read
  through descriptors (section 5), each through a temporary file, synced
  and renamed; unmount. Then read the slot's two entries, normal and
  trial, from efivarfs and make again each one that is missing or
  differs, at its number (machine.md, 4.2 and 4.3). Both stay out of
  `BootOrder`.
- **BIOS [M3b]:** nothing on the boot partition changes at prepare. The
  slot's fragment is one of its files (5.5), and `grub.cfg` is fixed.
- **Containers [M5]:** no boot files; the host's nspawn settings instead
  (section 12).

### 5.7 Verifying

1. **Seal the root:** `BTRFS_IOC_SUBVOL_SETFLAGS` with
   `BTRFS_SUBVOL_RDONLY`.
2. **Check the package users:** every user a task declares as made by a
   package (format.md, 5.6), such as `postfix`, must be in the slot's
   `/etc/passwd`, read through the same bounded parser as 5.4's; a
   missing one refuses the slot, naming every one missing.
3. **Walk the sealed root's metadata** against the manifest, without
   contents, and record each entry's inode and change time as the seal
   left them. Anything else that differs (nothing should) fails the
   prepare. `H("tree", …)` over the manifest's entries (store.md, 3.4)
   is the slot's own tree digest; it stays in the state.
4. **Compute the reported tree digest,** the same formula with each
   secret and identity file's content digest replaced by
   `H("secret", name, generation)`: the slot's digest the facts carry to
   the builder and to an engine, which says when a secret changed but
   not what it holds.
5. **On EFI, read the boot files back** from the ESP and check their
   digests against the kernel's, and **read both entries back** and
   check them field by field.
6. **Write the slot record,** state `prepared`, then clear the journal.
   The answer carries the prepare key and the reported tree digest.

## 6. Trial

### 6.1 Which kind

Three kinds, the first that applies:

1. **Live,** with no reboot, when the change is eligible (6.7): the new
   slot's files are mounted over the running slot's (6.6 to 6.9).
2. **A soft reboot,** when the kernel is unchanged (below).
3. **A one-shot reboot** otherwise.

A container's trial is live when eligible, and otherwise a restart,
with no kernel to compare (12.2, 12.8) [M5]. For a machine's own slots:

**The kernel is unchanged,** and a soft reboot enough, when the new slot
and the booted kernel (section 3) agree on all four of:

| Digest | Of | From |
|---|---|---|
| image | the content of `boot/vmlinuz-<version>` | the output's tree file (store.md, 6.3); the shift's walk finds it again |
| modules | the subtree `usr/lib/modules/<version>` (store.md, 3.4) | the tree file; the walk finds it again |
| initramfs inputs | `H("initramfs", …)` over the versions of `tiny-initramfs-core` and of the kernel's package, its configuration (which lists the modules), the kernel version, and with `MICROCODE=generic` the microcode packages' versions | the resolution's closure (render.md, 5.4), which build.md, 5.7 checks is what was installed, and the `packages` part's initramfs configuration (render.md, 4.2); the builder computes it with the output (build.md, 5.8; store.md, 6.3) and the place request carries it (activator.md, 5.1) |
| command line | the line without the slot's own tokens, `initrd=` and `rootflags=` | the slot record (machine.md, section 6) |

- **The initramfs is compared by its inputs, never its bytes.** mktirfs's
  cpio carries inode numbers and mtimes, so two builds of the same inputs
  differ (reproducible.md, 2.2, row 4); comparing bytes would force a full
  reboot at every snapshot bump.
- **It counts at all because a soft reboot never runs it.** A changed
  initramfs would first run at the next full boot, after the bless, with
  no trial to catch it; a change to its inputs takes a one-shot reboot,
  which runs it.
- **The modules must match** because the running kernel loads its
  modules from the new root. Whether depmod's indexes beneath them
  reproduce across builds is to verify (reproducible.md, section 4);
  until then a snapshot bump that keeps the kernel may still cost a full
  reboot.
- **The command line must match** because the running kernel keeps its
  own, and systemd in the new root reads its `systemd.*` options.
- **A one-shot reboot** otherwise: `BootNext` (EFI), or `next_entry`
  (BIOS) [M3b].
- **The builder may ask for a full reboot** (`how: full`), to try the
  firmware's path as well; it may not ask for a soft reboot the rule
  forbids.
- **`how: auto`** takes the first kind that applies; **`how: live`**
  asks for live only, and is refused, with the rules that fail, when the
  change is not eligible; **`how: full`** a one-shot reboot
  (wire.md, 5.4).
- **The builder asks for live,** with `try live`, since half of 6.7's
  rules read the specs and the store, which are its own; the activator
  checks the rest and refuses otherwise, and the builder then asks an
  ordinary `try`.
- **The activator chooses between soft and one-shot,** since that rule
  reads only the records and facts it owns, and it answers with the
  kind it chose.

### 6.2 By soft reboot

1. **Write the marker,** phase `arming` (6.4).
2. **Mount the slot's root** as a real btrfs mount, read-only, so that
   the new root's mount table shows its own subvolume:

   ```
   /usr/bin/mount -t btrfs -o ro,subvol=/@x1-prod.b /dev/nvme0n1p2 /run/nextroot
   ```

   It is the mount tiny-initramfs would have made, since the initramfs
   does not run again (machine.md, 7.4).
3. **Write `/run/hearth-trial/x1-prod.b`,** which `/run` carries across
   the soft reboot and which arms the new root's dead-man (5.5).
4. **Set the marker's phase to `armed`.**
5. **Answer the request, then `systemctl soft-reboot`.**

What the new root finds:

- **the same kernel, the same boot id, and `/proc/cmdline` unchanged:**
  it still names the old slot, which is why nothing reads it;
- **`SoftRebootsCount` one higher,** which tells the new activator that
  the root switched;
- **its own fstab mounted afresh:** its `/var`, the persistent
  subvolumes, the store and the state; `/` remounted by systemd with
  fstab's `subvol=` (to verify);
- **`/run` carried over;** the activator mounts `/run/hearth-activator/top` again
  if the switch dropped it.
- **No live overlay:** a live state's overlays sit on the old root's
  directories and go with it (to verify); a `live-blessed` slot tried by
  a soft reboot comes up natively, its own files on its own root.

### 6.3 By one-shot reboot

1. **Check the slot's trial entry** (EFI): present and as written, or
   made again (machine.md, 4.2). Its command line is the normal entry's
   with `hearth.trial=x1-prod.b`, which arms the dead-man (5.5).
2. **Write the marker,** phase `arming`.
3. **Set the one-shot:** `efibootmgr --bootnext 4811`, the trial entry;
   or, on BIOS, `next_entry=x1-prod.b` in `env.0`, then `env.1`, from
   which GRUB adds the same token (machine.md, 5.3).
4. **Set the marker's phase to `armed`.**
5. **Make sure `/run/nextroot` is not a mount point,** unmounting a
   leftover, and answer the request.
6. **Reboot:** `systemctl reboot`, with
   `SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1` in its environment.

On the trial's boot, the new slot's activator checks that the one-shot
was consumed: `BootCurrent` is the slot's trial entry and `BootNext` is
gone (EFI), or `next_entry` is empty in both files (BIOS); a
`next_entry` still set is cleared at once (machine.md, 5.6).

### 6.4 The trial marker

```
/var/lib/hearth/state/trial/<target>.<label>
  slot, attempt          what is tried, and the builder's attempt id
  kind                   soft | one-shot | live
  phase                  arming | armed | running | held [M4]
                         live: arming | armed | swapped | running | live-blessed | held [M4]
  boot id, soft reboots  at arming: how a later start tells "same boot" from "new boot"
  armed at               wall clock
  window                 the dead-man's, from the place request, sealed into the units (5.5);
                         a live trial's is the base's (6.8, L3)
  live                   base, delta, previous delta; the unit plan; after L6, each unit's
                         InvocationID and NRestarts, which the settle compares
```

- **At most one marker exists.** It is written before the one-shot and
  removed only by a bless, an abort or the start that records the
  outcome.
- **`running`** is set by the trial slot's own activator when it starts
  there. It is the difference between a slot that never reached hearth
  (failed to load, panicked, hung early) and one that reached it and was
  not blessed in time; both fall back, and the record keeps which.
- **It does not arm the dead-man:** it lives on a mount, which may fail.
  The command line's token or `/run/hearth-trial/` does (5.5); the
  marker is the activator's record of the trial.
- **A live trial's marker** is written at L3 (6.8) and kept through the
  live bless, in phase `live-blessed`, until the slot's native boot sets
  `running` (kind `one-shot` from then) and its bless removes it.

### 6.5 How every trial ends

| What happens in the trial | What ends it | Where it lands |
|---|---|---|
| the minimal criterion holds, and the slot's builder asks | bless (section 7) | the new slot, now the default |
| the trial's kernel fails to load | the firmware moves on to `BootOrder`; GRUB's `fallback` | the default, in the same boot |
| the kernel or the initramfs panics (a missing root after 180 s) | `panic=10` | the default: the one-shot was consumed |
| the kernel or PID 1 hangs | the hardware watchdog (`RuntimeWatchdogSec=`), where the machine has one | the default |
| userspace never finishes booting | `JobTimeoutAction=reboot-force` on `hearth-trial.target` | the default |
| a mount fails and systemd isolates `emergency.target` | the dead-man's timer, kept by `IgnoreOnIsolate=yes`, armed without any mount | the default |
| it boots, but no bless comes: a failed unit, a broken hearth | the dead-man: `systemctl reboot -ff` after the window | the default |
| the slot's builder finds a definite failure (`degraded`) | it asks `fall back` (section 8) | the default |
| the power is cut | the next power-on | the default |
| live: a unit action fails, a touched unit dies in the settle, the criterion fails | the live fall back (8.5) | the base's files, with no reboot |
| live: the live fall back itself fails or hangs | `reboot -ff`, or the base's dead-man | the default |
| live: no live bless within the base's window | the base's dead-man, `reboot -ff`; no one-shot is set before the live bless | the default |
| live: the activator crashes after arming | `reboot -ff` at its restart (section 11) | the default |
| live: a power cut, a panic, a watchdog reset | the next power-on; the overlays were kernel state | the default |
| live: the live bless | the one-shot set to the slot's trial entry (7.6) | the base, still running; the next boot is the slot's trial |

Two gaps remain: a kernel that hangs with no hardware watchdog, which
needs a person or, with an engine, a power cycle from outside [M4]; and
a firmware that ignores `BootNext`, where the trial never runs and is
reported (machine.md, section 9). A hardware watchdog's behaviour across
a soft reboot is to verify.

### 6.6 Live

A live trial runs the new slot's files over the running slot, which
stays the root: no reboot and no soft reboot. With `x1-prod` running
`a` and `b` prepared:

- **The base, `a`,** stays the root subvolume, running, blessed and the
  default throughout. Its `/var` keeps running; `b`'s `/var` is not used
  until `b` boots.
- **The new slot, `b`,** is prepared in full first, exactly as for any
  trial (section 5): shifted, verified, sealed, with its record. The
  live trial reads it; nothing runs from it.
- **The delta, `@x1-prod.live.<n>`,** a read-only subvolume at the top
  level, holds only what differs between `a` and `b` under `etc/`,
  `usr/` and `opt/`: reflink clones of `b`'s changed and added files,
  with `b`'s owners, modes, extended attributes and capabilities, and an
  overlayfs whiteout, a 0/0 character device, at each deleted path. `n`
  only grows, from the live record (2.4).
- **The overlays:** one read-only overlayfs per hierarchy the delta
  changes, the delta over the base's directory, which the activator
  mounts on `/etc`, `/usr` and `/opt` itself. A process that opens a
  path afterwards gets `b`'s file; one already open stays `a`'s, and
  nothing on disk changes, so there is no `ETXTBSY` and no torn file.
- **The units** that read the changed files are stopped, reloaded,
  restarted or started, as the tasks declare (format.md, 5.14).
- **The live bless arms `b`'s trial, and does not move the default**
  (7.6). `a + delta` is not `b`: `b`'s early boot, generators, udev,
  sysctl and every unit run `b`'s configuration only at `b`'s first
  native boot, and the units acted on live are a small part of what a
  boot starts. So the live bless sets the one-shot to `b`'s trial entry,
  and whatever ends the uptime next (a person's reboot, a power cut, a
  panic) boots `b` on an ordinary trial, with its dead-man and its own
  bless. Only that bless moves the default.

**hearth mounts the overlays itself, not through `systemd-sysext` and
`systemd-confext`.** Their `refresh` is not atomic: it unmounts the old
overlay before it mounts the new one, so for about half a millisecond on
a small tree (measured at systemd 257 and 261) `/etc` shows the bare
base and files only the extension had are missing, and a refresh that
fails after the unmount leaves no overlay at all; the fix (systemd PR
#43680) is in no release. A mismatched extension makes them silent
no-ops: an `extension-release` that does not match is ignored with exit
status 0, `merge` mounting nothing and `refresh` unmerging.
`systemd-confext` mounts `/etc` `noexec` by default, and Debian runs
scripts from `/etc`. Their behaviour differs across the versions hearth
meets: 260 and later skip a refresh they judge unchanged, 257 follows no
symlink under `--root`, 262 adds unit restarts of its own. And their
search paths in `/run` survive a soft reboot, where the new root's boot
services would merge the delta over whatever root came up. hearth's own
mount, `fsopen("overlay")` then `move_mount` with `MOVE_MOUNT_BENEATH`,
is the kernel mechanism upstream's fix and NixOS's `/etc` use: it swaps
with no window, fails without touching the old overlay, and costs about
as much code as driving the tools and checking their output. The
`hearth` task disables both services (format.md, 5.12), so nothing
merges extensions behind hearth's back.

### 6.7 What may be live

An allowlist. The builder decides it when it plans the trial, from the
two outputs' tree files (a sorted diff, store.md, 6.3) and the specs'
`live` parts (render.md, 4.4); the activator checks again what its own
records and the machine show. A change that fails any rule takes an
ordinary trial (6.1). Not being live is never an error: the facts say
why (wire.md, section 6).

| # | Rule | Why | Checked by |
|---|---|---|---|
| 1 | **The same packages layer:** both outputs have one packages key | packages, the kernel, its modules, the initramfs's inputs, systemd, dbus and libc stay; a packages change is never live | builder; activator, from the layer records' parents (activator.md, 6.1) |
| 2 | **The prepare key equal but for the output,** the window and the units allowed to fail: the same `data`, `secrets` and `boot` digests, the same identity and secret store generations | a new mount changes fstab, a new command line changes the boot, a rotated secret is not in the output's diff | both, from the slot records |
| 3 | **Changes only under `etc/`, `usr/` and `opt/`,** none under `usr/lib/modules/` | `/var` content reaches a slot only at prepare; `/boot` and the modules are packages anyway | both |
| 4 | **No boot-time or PID 1 configuration** (format.md, 5.14) | read only at boot or at PID 1's re-exec; applied live it is not what a boot does (a removed sysctl is not reverted), so the trial would test the wrong thing | both |
| 5 | **Users and groups only added:** no line of `passwd`, `group`, `shadow`, `gshadow`, `subuid` or `subgid` removed, no id changed | a renumbering touches files in `/var` and data; a removal strands running processes | builder; activator, with its account parsers (activator.md, 2.3) |
| 6 | **tmpfiles only creatable:** the changed lines of `usr/lib/tmpfiles.d/hearth.conf` are of types `systemd-tmpfiles --create` applies whole; none is `r`, `R` or `D` | the output's `/var` got them at build; the running `/var` needs them now, and nothing removes live | builder |
| 7 | **Every changed path has a consumer:** a task's `live` entry, or one hearth knows (format.md, 5.14); a deleted path's from the running slot's spec, every other path's from the new one | hearth must know what to reload or restart; guessing from paths is how this fails | builder |
| 8 | **Nothing never-live** (format.md, 5.14): systemd's own units and configuration, dbus (its policy files are allowed, with a reload of dbus), logind, journald, udevd, networkd, NetworkManager, the gettys, AppArmor, and hearth itself for now | restarting them live is risky or wrong | builder; activator, for the plan's units |
| 9 | **No file dpkg owns** under `usr/` or `opt/` changed by the software layer, by the packages layer's dpkg database; conffiles in `etc/` are fine | a download over a packaged file is a packages change in disguise | builder |
| 10 | **The machine fits:** Linux 6.8 or later (`MOVE_MOUNT_BENEATH` since 6.5, overlayfs's `lowerdir+` since 6.8; trixie runs 6.12); `etc`, `usr` and `opt` are directories in the base; no mount on or beneath them but hearth's; running = default = blessed; no trial; at most one live state, whose base is the running slot; [M5] the new slot's window no longer than the base's | the swap's mechanisms; a base proven by a native boot; the base's dead-man ends the trial | activator |
| 11 | **Sandboxes beneath the hierarchies:** units whose mount namespace has a mount beneath `/etc`, `/usr` or `/opt` join the restarts, and if one is never-live the change is not eligible | the overlay, propagated into the unit's namespace, shadows that mount until the unit restarts | activator, as root, from `/proc/<pid>/mountinfo` of every process and its cgroup |

- **What the builder sends:** `try live`, with the digest of the change
  from the base's output to the new one, which the delta will hold, and
  the unit plan (activator.md, 5.17): the units to stop before the swap,
  to reload, restart or start after it, and whether `daemon-reload`,
  tmpfiles, D-Bus or the accounts need acting on.
- **What the activator recomputes:** the change between the two slots'
  manifests, leaving out each slot's own files (5.4, 5.5), whose digest
  (in store.md, 3.5's form) must equal the builder's; then rules 1 to 5,
  8 for the plan's units, 10 and 11. The delta is built from its own
  diff, never from paths the builder sent. A difference refuses the
  trial, naming the rule.
- **A later live change** is planned against what runs, the live
  state's files, and its delta built against the base (6.9).

### 6.8 The steps

Every step that leaves the state starts by writing the journal
(activator.md, 7.3); the marker carries the phase from L3.

| Step | What the activator does | Phase after |
|---|---|---|
| **L0. Prepare `b`** | section 5, unchanged: `b` sealed and verified, with its record | none: `b` is `prepared` |
| **L1. Build the delta** | make `@x1-prod.live.<n>` and fill it from the change (below) | none |
| **L2. Verify, then seal** | walk the merged view in no mount namespace anyone sees (below); then `BTRFS_SUBVOL_RDONLY` on the delta | none |
| **L3. Arm** | write the marker: kind `live`, phase `arming`, base `a`, the delta, the previous one, the boot id and soft reboot count. Write `/run/hearth-trial/x1-prod.a`, then `systemctl start hearth-trial.timer`: `a`'s own dead-man (5.5), whose condition the file meets and whose `OnActiveSec=` counts from now; check it is active | `armed`; `b` is `on live trial` |
| **L4. Stop** | `systemctl stop`, one call: the units being removed or disabled, and each `restart` unit with a changed path under `usr/` or `opt/`; every job's result checked | `armed` |
| **L5. Swap** | for `/etc`, then `/usr`, then `/opt`: the overlay built and moved beneath the current top (below); the mount table read back; the live record written | `swapped` |
| **L6. Act** | `systemctl daemon-reload` if a unit file, drop-in or `.wants` link changed; `systemd-tmpfiles --create /usr/lib/tmpfiles.d/hearth.conf` if it changed; `systemctl reload dbus` if D-Bus policy changed; `nscd --invalidate` for `passwd` and `group` if nscd runs and the accounts changed. Then the declared reloads, restarts and starts as one batch (below), every job checked. Then answer the request and spawn the builder anew (activator.md, 3.3), so the live bless comes from a builder started in the new files | `running` |
| **L7. Settle** | 60 seconds to start, within the base's window: every unit L4 to L6 touched stays `active` with the `InvocationID` and `NRestarts` L6 read; the minimal criterion holds (7.1); from M5, the blessing checks pass (operating.md, section 3) | `running` |
| **L8. Live bless** | asked by the builder: 7.6. The default does not move | `live-blessed`; `b` is `live-blessed` |
| **L9. The native boot** | the next boot consumes the one-shot and boots `b`'s trial entry: `b` runs on trial (6.3), its dead-man armed by `hearth.trial=`; its activator sets the marker `running`; its builder blesses it (7.3), the default moves to `b`, and the delta is deleted | the marker removed by the bless |

**L1, the delta.** `BTRFS_IOC_SUBVOL_CREATE_V2` makes it, with `etc`,
`usr` and `opt` carrying `b`'s metadata. The change is walked in tree
order through descriptors, `b`'s paths resolved beneath `b`'s root as
prepare resolves them (section 5), the delta's beneath the delta's:

- **A put:** missing parents made with `b`'s owner, mode, ACLs and
  extended attributes, since a merged directory takes the topmost
  layer's attributes and a default `0755` parent would loosen
  `/etc/sudoers.d`; a regular file cloned with `FICLONE`, then its
  owner, mode, extended attributes and v2 capability, in the shift's
  order (5.2), since `chown` clears setuid bits and capabilities; a
  link made again; two changed paths that share an inode in `b` share
  one in the delta. A changed path whose other link is unchanged
  becomes an inode of its own, with another link count: harmless, and
  neither L2's walk nor drift compares link counts.
- **A delete:** `mknodat(…, S_IFCHR, makedev(0, 0))` at each topmost
  deleted path, a whole directory included. A path whose kind changes
  needs no whiteout and no opaque directory.
- **Built from the prepared slot,** the delta holds real ids, setuid
  bits and capabilities as the shift left them; the overlay presents
  them unchanged.

**L2, the merged view.** The three overlays are built exactly as L5
will build them, with `fsopen` and `fsmount`, and never attached: a
detached mount is in an anonymous mount namespace no process sees, and
the activator walks it through its descriptor (activator.md, 6.5). The
walk must give the expected manifest: `a`'s manifest under `etc/`,
`usr/` and `opt/`, with each path of the change replaced by `b`'s entry
or removed. It compares type, mode, owners, size, extended attributes
and link targets for every entry, and contents for the delta's files;
not inode numbers or change times, which an overlay reports from its
layers or itself. Any difference fails the trial, and the delta is
deleted. The expected manifest becomes the live record's (2.4). It is
store.md, 3.5's rule for a change set, kept for a delta: applied and
walked, it must give the child's tree. Then the delta is sealed: nothing
writes a layer once it is merged, which overlayfs calls undefined.

**L4 and L6, the units.** Each changed path's consumers
(format.md, 5.14) decide:

- **Before the swap** stop what must not straddle it: a `restart` unit
  whose binary, library or tree under `usr/` or `opt/` changes, so it
  never runs new code with old files or the reverse (NixOS stops changed
  units first for the same reason), and a unit whose file is removed or
  whose `.wants` link goes, since `ExecStop=` may name a file the delta
  removes.
- **After the swap,** in this order: reloads, then restarts (a
  `restart` unit with changes under `etc/` only, a unit whose file or
  drop-in changed, and rule 11's units), then `reload-or-restart`, then
  starts (the units L4 stopped, new units and newly wanted ones, and
  `start` units). One `systemctl` call per verb, each enqueuing its
  units' jobs in one transaction and waiting for them. A new `.wants`
  link starts nothing by itself, and a swap fires no inotify event on
  the base's inodes, so every action is explicit.
- **Every job's result counts, not the unit's state:** each is read back
  (`ActiveState`, `Result`, `InvocationID`, `NRestarts`). A reload of
  `ssh` that fails `sshd -t` leaves the unit active and the job failed,
  which is the live fall back (8.5).
- **What is not acted on:** shells, cron jobs and timers already
  running, and people's processes, which see a mixed world until they
  exit.

**L5, the swap,** for each hierarchy the delta changes:

```
fs  = fsopen("overlay", FSOPEN_CLOEXEC)
fsconfig(fs, FSCONFIG_SET_STRING, "lowerdir+", "/run/hearth-activator/layers/@x1-prod.live.<n>/etc")
fsconfig(fs, FSCONFIG_SET_STRING, "lowerdir+", "/run/hearth-activator/layers/@x1-prod.a/etc")
fsconfig(fs, FSCONFIG_SET_STRING, "source", "hearth-live")
fsconfig(fs, FSCONFIG_CMD_CREATE)
mnt = fsmount(fs, FSMOUNT_CLOEXEC, MOUNT_ATTR_RDONLY | MOUNT_ATTR_NODEV | MOUNT_ATTR_NOSUID)   NOSUID on /etc only
a hearth overlay on /etc:  move_mount(mnt, "", AT_FDCWD, "/etc", MOVE_MOUNT_F_EMPTY_PATH | MOVE_MOUNT_BENEATH)
                           umount2("/etc", MNT_DETACH | UMOUNT_NOFOLLOW)                        the old top
otherwise:                 move_mount(mnt, "", AT_FDCWD, "/etc", MOVE_MOUNT_F_EMPTY_PATH)
```

- **No window:** readers go from the old view straight to the new, and
  never see the base nor a missing file, as measured with readers
  opening a file in a tight loop. A `move_mount` that fails leaves the
  old top in place, so a failed hierarchy changes nothing.
- **Exec is allowed** on all three; `nosuid` is set on `/etc` only,
  since `/usr` holds setuid programs.
- **A hierarchy the new delta does not change** loses a previous overlay
  by `umount2(MNT_DETACH)` alone, and shows the base.
- **The hierarchies are not atomic with each other:** `/etc`, then
  `/usr`, then `/opt`, which is why software stops first (L4).
- **Read back** `/proc/self/mountinfo`: each hierarchy an overlay with
  source `hearth-live`, its layers this delta over the base.
- **The layers' source,** `/run/hearth-activator/layers`, is a clone of
  the top level (`open_tree` with `OPEN_TREE_CLONE`), read-only,
  without the top level's `noexec` and `nosuid` (`mount_setattr`), and
  private, made at the activator's start: a lazy unmount of a shared
  peer of a layer's source propagates and tears the overlay down (as
  measured), and whether overlayfs carries a lower mount's `noexec` to
  the merged files is to verify.
- **The activator stays in the host's mount namespace,** with no
  sandboxing in its unit (activator.md, 3.1), or its overlays would land
  in a namespace of its own.

**A person's soft reboot while live** (`systemctl soft-reboot` with no
`/run/nextroot`) keeps the overlays on `/etc` and `/usr`, which systemd
treats as extrinsic, and unmounts `/opt`'s (to verify). The next start
finds the base still running, the boot id unchanged, the soft reboot
count higher and the mount table disagreeing with the live record, and
runs `reboot -ff`: the default, or the trial a one-shot names, boots
natively. hearth's own soft reboot into a `live-blessed` slot (a `try`,
6.2) is not this: the new root is that slot's, and the start removes
the live record whose base no longer runs.

### 6.9 A later live change

While `b` is `live-blessed` (or a live state outlives its slot, below):

- **Prepare `c` into `b`:** allowed, since `b` is neither running nor
  the default; 5.1 clears the one-shot that names `b`'s trial entry.
  The delta keeps serving: it is a subvolume of its own, not `b`.
- **The delta is always against the base:** `delta(a → c)`, built and
  verified as L1 and L2, never stacked on the previous one. The unit
  plan and rules 3 to 9 compare `c` with what runs, the live state's
  files, so only what changed since the last live change acts; rules 1
  and 2 compare `c` with `a`.
- **L5's beneath-swap replaces the previous overlay** atomically.
- **The previous delta is kept until `c` is live-blessed,** and the live
  fall back swaps back to it, not to the bare base (8.5). The live bless
  deletes it.
- **After such a fall back no slot holds what runs:** `b` holds `c`,
  fell back, and the previous delta's slot was prepared over. The live
  record keeps naming the previous delta, with no slot, and no one-shot
  is set: the next boot runs `a`, older than what ran, and the next live
  change replaces the delta.
- **A dead-man reboot or a power cut** in any of these states lands on
  `a`, natively: the invariant ("the default boots a blessed slot")
  holding, older than the live state. The facts say so.

## 7. Bless

### 7.1 The minimal criterion (M3)

- **The boot reached its default target with no failed unit that
  counts:** `systemctl is-system-running` answers `running`, or
  `degraded` with every failed unit (`systemctl list-units --state=failed`,
  parsed by `hearth-protocol-systemd`) on the target's
  `bless.ignore_failed` list (format.md, 5.13), which the place request
  carries. `starting` means not yet; a failed unit that counts means the
  builder asks to fall back rather than wait out the window. A desktop
  such as `x1-prod` often carries a routinely failed unit; comparing
  against a baseline of the previous slot is later, once measured there.
- **The slot's own builder is up and has answered its activator:** the
  bless request arrives over the link from the builder the slot's
  activator spawned, after its hello, and that is the answer.
- **A host's containers never count [M5]:** the failed units counted
  leave out the nspawn unit, `systemd-nspawn@<container>.service`, of
  every container the host's activator keeps records for (activator.md,
  7.1), with no list to write. Each container's slot has its own trial
  and its own facts (12.3); one bad container must not roll its host
  back.
- **Task checks join in M5** (operating.md, section 3).

### 7.2 Who asks

- **The new slot's hearth blesses itself:** its builder's converge asks
  its own activator, once the link is up and the criterion holds. The
  old slot's hearth never blesses the new one: it is not running.
- **A broken hearth release cannot bless itself.** If the new builder
  never starts, never connects or never asks, the dead-man ends the
  trial, and the default never moved. No outside probe judges the slot:
  the bash fleet once lost a good boot to an SSH probe that never
  reconnected.
- **A container is the exception [M5]:** no hearth runs in it, so its
  host's builder asks its host's activator, which reads the container
  from outside (12.3).
- **A live trial is blessed by the base's builder,** spawned anew after
  the swap so that it starts in the new files (7.6); the new slot
  blesses itself only at its native boot (6.8, L9).

### 7.3 What it does

1. **Check:** the marker names this slot and attempt; the slot runs; the
   criterion holds (the activator checks it itself).
2. **Journal** the bless (activator.md, 7.3).
3. **Write the slot record** as `blessed`.
4. **Remove the marker,** and `/run/hearth-trial/x1-prod.b` if a soft
   reboot left one; after a live trial, delete the delta it names
   (6.8, L9).
5. **Move the default:** `efibootmgr --bootorder 4801,4800,…`, read back
   (EFI); or, on BIOS, `previous` set to the old default, `default` to
   `x1-prod.b` and the boot counter reset, in `env.0`, then `env.1`
   (8.4).
6. **Stop the dead-man:** `systemctl stop hearth-trial.timer
   hearth-trial.target`.

- **The default moves last but one,** so a cut never leaves the default
  on a slot still `on trial`, which would re-arm at every boot. A cut
  before step 5 leaves a blessed slot that is not the default, which the
  re-bless finishes (7.5); a cut before step 6 leaves a dead-man that
  reboots once into a blessed default.
- **The other slot stays `blessed`,** second in `BootOrder` on EFI and
  `previous` on BIOS: the roll back's target, and the firmware's or
  GRUB's fallback, until the next prepare takes it.

### 7.4 Holding a bless [M4]

An engine may hold a bless while a canary bakes (wire.md, 5.4: an
`activate` with `bless: held for at most d`):

- **The slot's builder asks to hold** once the minimal criterion holds
  (operating.md's checks too, from M5). The activator moves the marker to
  phase `held`, with the hold's end, and replaces the dead-man's timer by
  a transient one ending at the hold's end, `systemd-run
  --on-active=<d> /usr/bin/systemctl reboot -ff`, so the end does not
  depend on hearth either. The default does not move.
- **The engine's `bless` releases it:** the bless of 7.3, conditional on
  the slot on trial, healthy and held. Its `roll back`, or the hold's
  end, falls back (section 8).
- **While held, the slot runs on trial,** not the default: any reboot or
  power cut lands on the old default, which is what an unbaked canary
  should do, and the next start records `fell back`.
- **The table gains one state,** `held`, between `on trial` and
  `blessed`, with `on trial`'s cells but for `bless` (released: `blessed`)
  and `hold` (same attempt: done).
- **A container's bless is held the same way [M5]:** the engine may hold
  it, and the `bless` order releases it. Its end is the host's
  activator's deadline, kept in the marker as the window is (12.3): at
  the hold's ceiling the activator falls the container back, a restart
  on its old slot.
- **A live trial's bless is held the same way:** the marker's phase
  `held`, the base's dead-man replaced by the hold's transient timer;
  the `bless` order releases it into the live bless (7.6), and the
  hold's end reboots into the default, which drops the overlays.

### 7.5 Re-bless

- **The running slot is blessed but not the default** after a bless cut
  before it moved the default, after a firmware fell through to it, or
  after GRUB's counter booted it (8.4). The facts say so.
- **Its builder asks to bless it again,** a stage of converge's pass
  (wire.md, section 4). The activator checks the criterion as for a
  trial, but needs no marker: the slot is already blessed, and it runs.
  Then it moves the default to it (7.3, step 5), and writes nothing else.
- **Until then, prepare is refused** (2.3), so no slot is written while
  the firmware would boot another.
- **A live state never calls for one:** the default never leaves the
  base, which runs, so "running and blessed but not the default" does
  not arise. Had the live bless moved the default, this rule would move
  it back to the base at the next start.

### 7.6 The live bless

The bless of a live trial (6.8, L8), asked by the builder on the link,
the base's, spawned after the swap:

1. **Check:** the marker names this slot and attempt, in phase
   `running`; the live fact agrees with the live record; every unit L4
   to L6 touched is active with the `InvocationID` and `NRestarts` L6
   read; the settle has passed; the criterion holds (7.1); from M5, the
   blessing checks pass (operating.md, section 3).
2. **Journal** the live bless (activator.md, 7.3).
3. **Write the slot record** as `live-blessed`.
4. **Set the one-shot to the slot's trial entry:** `efibootmgr
   --bootnext 4811`, read back (EFI); or `next_entry=x1-prod.b` in
   `env.0`, then `env.1`, from which GRUB adds the trial's token (BIOS,
   machine.md, 5.3) [M3b].
5. **Set the marker's phase to `live-blessed`,** and delete the previous
   delta, if any (6.9).
6. **Stop the base's dead-man:** `systemctl stop hearth-trial.timer`,
   and remove `/run/hearth-trial/x1-prod.a`.

- **The default does not move.** `a` stays first in `BootOrder`, or
  `default` on BIOS, until `b` has booted natively and blessed itself
  (6.8, L9). A live bless that moved it would make `b` the default with
  no native boot ever tried: should `b` then fail in user space, no
  token and no `/run` marker would arm its dead-man, and the firmware
  falls back only when a kernel fails to load.
- **Cuts:** a crash after step 3 is finished at start from the journal.
  A power cut before step 4 leaves `b` `live-blessed` with no one-shot,
  after a boot that dropped the overlays: the start makes it `prepared`
  again (section 4). After step 4 the power-on boots `b`'s trial. A
  crash before step 6 leaves the base's dead-man armed, which the
  journal's recovery stops.
- **The one-shot may wait for days** before a boot consumes it: that a
  firmware keeps `BootNext` across a long uptime is to verify.

## 8. Fall back

### 8.1 The reboot

- **A fall back is a full reboot into the default,** never a soft reboot
  back into the trial. A live trial's is 8.5's instead, which reboots
  only when one of its steps fails. While `/run/nextroot` is a mount
  point, `systemctl reboot` turns into a soft reboot (systemd 255 and
  later, trixie's 257 included; soft reboot itself came in 254), unless
  `SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1` is set.
- **So every reboot hearth makes avoids it:** the activator unmounts
  `/run/nextroot` and runs `systemctl reboot` with that variable set; the
  dead-man runs `systemctl reboot -ff`, which calls `reboot(2)` without
  a clean shutdown; `reboot-force` is PID 1's own.
- **In the trial slot `/run/nextroot` is not a mount point,** since the
  switch consumed it. The trap matters in the old slot, while a soft
  reboot is armed, which is why aborting one unmounts it first.
- **Asked for in two cases:** the trial slot's builder finds a definite
  failure; or the old slot's builder, before the reboot, gives up on a
  trial it armed. In the second case nothing reboots: the one-shot is
  cleared, `/run/nextroot` unmounted, `/run/hearth-trial/` emptied, and
  the slot is `prepared` again.

### 8.2 Recording it

At its start in the default slot, the activator finds a marker naming
another slot, in phase `armed` or `running`, and a boot id or soft
reboot count that changed:

1. **write that slot's record** as `fell back`, with the phase it reached
   (`never reached hearth`, or `not blessed in time`) and the failed
   boot's id;
2. **clear what the trial left:** a `BootNext` or `next_entry` still set,
   and `/run/hearth-trial/`;
3. **remove the marker;**
4. **report** the facts and a `fell back` event.

Each step is idempotent, so a cut between them is finished at the next
start.

### 8.3 What a fell-back slot keeps

- **Its subvolumes, until the next prepare,** including its `/var` with
  the failed boot's state, for triage [M5].
- **Its entries, outside `BootOrder`.**
- **No second trial without a new prepare:** its `/var` has run, so it is
  no longer what prepare made.

### 8.4 A blessed default that stops booting

A slot that passed its bless can later fail at every boot: a disk, a
firmware or a kernel that turns bad. A panic then reboots into the same
default, for ever: EFI falls through `BootOrder` only when an image
fails to load, and GRUB's `fallback` is the default itself.

- **BIOS [M3b]: a boot counter in the environment pair,** RAUC's pattern
  (machine.md, 5.3). At bless the activator writes `previous`, the old
  default, and `tries=3`. GRUB counts `tries` down at each boot of the
  default; at 0 it boots `previous` instead. The activator resets
  `tries` at every start that meets the minimal criterion. The previous
  slot, booted so, is blessed and not the default: its builder blesses it
  again (7.5), and the slot that stopped booting is recorded `fell back`.
  Prepare clears `previous` before it writes over that slot (5.1).
- **EFI (M3): no such fallback.** A blessed default that panics at every
  boot loops until a person or, with an engine, a power cycle and an
  order intervene. One option: keep the previous blessed slot first in
  `BootOrder` and set `BootNext` to the default at each start that meets
  the criterion. It costs one NVRAM write per boot, and a power cut
  during a boot rolls back to the previous slot. It is open
  (section 15).

### 8.5 The live fall back

How a live trial falls back with no reboot (6.8). The activator runs it
itself when an action of L6 fails; the builder asks for it when a
touched unit dies or the criterion fails during the settle, or, a margin
before the base's window ends (60 seconds to start), when the bless has
not come, so the dead-man stays the backstop; and a `fall back` or a
roll back of a `live-blessed` slot is the same, its one-shot cleared and
the base's dead-man armed again first. The dead-man stays armed
throughout:

1. **stop the units L6 started new;**
2. **swap back,** per hierarchy: beneath, onto the previous delta's
   overlay (6.9), or `umount2(MNT_DETACH)` down to the base; read the
   mount table back; write the live record, or remove it;
3. **`systemctl daemon-reload`,** if L6 ran it;
4. **replay the actions under the old files:** reload and restart the
   same units again, and start what L4 stopped;
5. **reload D-Bus and invalidate nscd again,** if L6 did;
6. **check every job,** as L6 does;
7. **only then** stop the dead-man and remove
   `/run/hearth-trial/x1-prod.a`, write the slot record `fell back` with
   kind `live`, the step and the reason, remove the marker, and delete
   the failed delta.

- **Any failure in steps 1 to 6 is `reboot -ff`:** the default boots
  natively, and a fall back that hangs meets the dead-man.
- **When it succeeds nothing reboots:** the files that ran before run
  again, and the facts report `fell back (live)` (wire.md, section 6).
- **It restores files, not state.** An open file or a working directory
  keeps the new files until closed, a process keeps what it loaded,
  users added keep their files in `/var`, tmpfiles' paths stay, and what
  was written to `/var` or a data subvolume stays: a data migration is
  not undone, as with any fall back (8.3). Only a reboot restores
  process state; nothing restores data.
- **A fell-back slot is refused by its prepare key** (wire.md, 4.4), as
  after any trial, though its `/var` never ran.

## 9. Roll back

- **The request names the other slot,** conditional on the facts it was
  decided from: the current slot runs and is the default, and the other
  slot is `blessed` with the record the decision saw. Anything else is
  refused as stale.
- **It is a trial of the other slot,** soft or one-shot by 6.1's rule,
  followed by that slot's own bless when the criterion holds. Its `/var`
  is as it was when it last ran.
- **It never moves the default blind.** If the old slot no longer boots,
  its trial falls back and the machine stays where it was, with both
  facts reported.
- **While a live state runs,** a roll back is its live fall back (8.5):
  back to the files that ran before it, with the one-shot cleared and no
  reboot. A
  `live-blessed` slot is not blessed, so no other slot is a roll back's
  target until it has booted natively.
- **Who asks:** an engine's order, an operational effect granted by
  policy [M5] (operating.md, section 6); standalone, a person through
  the client (wire.md, section 3).

## 10. Drift

The activator compares the running slot with its record, and the
machine with its layout, and reports each check under a stable id. The
ids are facts; drift is never repaired in place.

| Check id | Passes when |
|---|---|
| `slot.root.subvolume` | `/` is the running slot's root subvolume, by uuid, and its read-only flag is set |
| `slot.root.record` | `/.hearth/slot` equals the slot record's fields |
| `slot.root.tree` | a walk of the running slot's root, through `/run/hearth-activator/top/@<target>.<label>` and never through `/`, matches the manifest; the first paths that differ are listed, up to a bound |
| `slot.var.subvolume` | `/var` is the slot's var subvolume, by uuid |
| `slot.mounts` | every fstab line is mounted, from its subvolume, with its options |
| `slot.live.mounts` | with a live record: each hierarchy it names carries one overlay with source `hearth-live`, its layers the record's delta over the base, the delta read-only; the others none. Without one: no `hearth-live` overlay anywhere |
| `slot.live.tree` | with a live record: a walk of the merged `/etc`, `/usr` and `/opt` matches the record's manifest (2.4), comparing type, mode, owners, size, extended attributes and link targets |
| `slot.boot.files` | EFI: the slot's ESP files match their digests |
| `slot.boot.entry` | EFI: the slot's two entries match; BIOS: covered by `slot.root.tree` |
| `machine.default` | the default is the slot the records say, and blessed |
| `machine.subvolumes` | every subvolume the layout names exists, and no other `@<target>.*` but a restore's or the deltas the live record and the marker name (machine.md, 3.1) |
| `machine.data.<name>` | the persistent subvolume exists and is mounted at its path |

- **Left out:** the contents of `/var` and of the persistent subvolumes,
  which are meant to change, and anything mounted beneath `/`. The root's
  walk goes through the top level, so it stays on the root's subvolume
  and never sees a live state's overlays, which `slot.live.tree` walks.
- **Two depths:** a metadata walk compares type, mode, owners, size,
  change time and inode with the manifest; a sealed root changes none of
  them unless someone cleared its read-only flag. Contents are hashed
  again only on demand.
- **When:** on demand from the client, and at every start; on a period
  with operating's signals [M5] (operating.md, section 4).
- **Drift is the activator's:** it wrote the record, and only root can
  read every file of a slot (README, section 4.2).
- **A host's containers [M5]** are checked by the host's activator under
  the same ids, reported for the container's target: the running slot's
  root from the top level, against its record and manifest;
  `slot.mounts` against the nspawn settings it wrote (12.1), not fstab;
  no `slot.boot.*`; `machine.default` against the default it recorded.
  `machine.subvolumes` counts the containers' subvolumes as the host's
  layout.

## 11. Crashes and power cuts

The activator writes its journal before each step with effects outside
the state, and the trial marker before each one-shot (activator.md,
section 7). Every start reads them, reads the facts, and applies the
start events of section 4. What each cut leaves, with `a` running and
default and `b` the slot in question:

| Step | Activator crash | Power cut |
|---|---|---|
| prepare, any step from 5.1 to 5.7 | `b` `preparing`: deleted at start, nested subvolumes first; the builder asks again | the same after the reboot into `a`; ESP files of `b` may be torn, behind entries outside `BootOrder` that nothing boots but a person's choice (a fault cell, section 4) |
| prepare, record written, journal not cleared | `b` `prepared`; the journal is cleared | the same |
| trial, marker `arming` | `b` `trying`, same boot: `prepared`, one-shot cleared | new boot into `a`: `prepared`, one-shot cleared |
| trial, one-shot set, marker `armed`, before the reboot | same boot: `prepared`, one-shot cleared, `/run/nextroot` unmounted | power-on consumes `BootNext` or `next_entry`: `b` boots, `start here`: `on trial`. Soft kind: `a` boots, `fell back` (never reached hearth) |
| trial, the reboot under way | none: the activator is stopped by it | as above |
| on trial, before bless | restart in `b`: still `on trial`, the dead-man still running | `a` boots: `fell back` |
| bless, record written, default not moved | restart in `b`: `blessed`, not the default: the re-bless moves it (7.5) | `a` boots, the default: `b` is a blessed slot not the default, the roll back's target; converge tries it again |
| bless, default moved, dead-man not stopped | restart in `b`: stopped at start | `b` boots as the default, blessed, unarmed: nothing to do |
| fall back recorded, marker not removed | finished at start | finished at start |
| BIOS environment, a file torn | the other holds the old state; made whole at start | GRUB reads the other: the old state (machine.md, 5.5) |
| live, L1 or L2 | the journal: the delta deleted; `b` stays `prepared` | the same after the boot into `a` |
| live, marker `arming` | `prepared`: the dead-man stopped, `/run/hearth-trial/` emptied, the delta deleted | `a` boots: `prepared`, the delta deleted |
| live, `armed`, `swapped` or `running` | `reboot -ff` at start, before anything else; then `a` boots: `fell back` | `a` boots, the overlays gone: `fell back`, the delta deleted |
| live bless, record written, one-shot not set | finished at start: the one-shot set, the dead-man stopped | `a` boots: `b` `prepared` again, the delta deleted |
| live bless, one-shot set, dead-man not stopped | finished at start | the power-on boots `b`'s trial: `start here`, `on trial` |
| `live-blessed`, before the native boot | a restart: ignored while the live fact agrees with the record | the power-on boots `b`'s trial (6.8, L9) |
| the live fall back, any step | `reboot -ff` at start, unless the journal shows it finished | `a` boots: `fell back` |

- **The default is written once per operation,** at bless, after the
  slot's record says `blessed`, and only to a slot that is running and
  passed the criterion in that boot.
- **Firmware fall-through reaches only blessed slots:** a slot leaves
  `BootOrder` before it is prepared, and enters it only at bless.
- **A one-shot is consumed before the kernel loads,** by the firmware or
  by GRUB, so no cut makes a trial sticky.
- **Recovery is idempotent:** every start event's handling may itself be
  cut and done again.
- **A live state never outlives its boot:** overlays are kernel state
  and `/run` is a tmpfs, so any reboot comes back as the default, or as
  the trial a live bless armed; the start then deletes every delta no
  record names. Mid-trial, `reboot -ff` is the one recovery: simple, and
  always correct before the live bless.
- **A person's soft reboot while live** is neither a crash nor a cut:
  the start finds the overlays disagreeing with the live record and runs
  `reboot -ff` (6.8).
- **A container's slot [M5]** follows the same table for prepare. A cut
  in its trial or bless leaves a link in `/run` and a marker: a power
  cut drops the link, and the host's start links the recorded default
  (12.3); an activator crash finds the link and the marker, and goes on
  as 12.3 says.

## 12. Containers and pool hosts [M5]

### 12.1 Container slots

- **On the host's top level:** `@<container>.a`, `@<container>.a.var`,
  `@<container>.b`, `@<container>.b.var`, and its persistent
  subvolumes, where the container's name is its target's unless a pool
  runs several of one target.
- **Its `/var` is per slot,** as a host's. The bash fleet bound a
  persistent `.var` into each container; what a container keeps across
  slots is a `data` entry of its tasks, as on a host.
- **Prepared by the host's activator,** section 5's steps, with these
  differences:
  - the shift writes ids from the container's own range, its
    `PrivateUsers=` base, 64 Ki aligned and clear of the builder's
    range (store.md, 9.2), instead of from 0 (or from 0, with idmapped
    mounts: 12.4); its persistent subvolumes are owned in the same ids,
    their root the container's root;
  - 5.1 takes nothing out of a firmware's fallbacks, and 5.3 makes no
    mount point for hearth's state and store, which are never bound into
    a container (store.md, 9.4);
  - 5.5 writes its `/etc/fstab` with no line, and writes no dead-man
    unit and no watchdog drop-in: nspawn makes its root and binds the
    rest (machine.md, 3.3), and the host judges the trial (12.3);
  - 5.6 writes the host's nspawn settings instead of boot files (below),
    and 5.7 reads no boot file back;
  - the identity and secrets are the container's target's, made and
    kept by the host's activator, so a container no longer shares its
    host's machine-id.
- **Its parameters arrive in its `boot` part** (format.md, 3.5;
  render.md, 4.3): the id base, the binds, the capabilities and the
  network. Its slot record (2.2) holds them where a host's holds a
  kernel and a command line, and its nspawn settings are written from
  them. They enter the prepare key as any slot part does, so a change to
  one prepares a new slot of the same output, which the host tries by a
  restart (12.2), since nspawn reads its settings only when it starts.
- **A new id base needs fresh data.** The shift writes the new slot in
  the new range, but the container's persistent subvolumes stay owned
  in the old one, which the old slot needs to fall back to. So the
  activator refuses a place whose base is not the owner of the roots of
  the container's existing persistent subvolumes, each made owned by
  the container's root (machine.md, 3.4), and names them. A person
  deletes them, and the container starts again from scratch, as every
  machine does (README, section 7).
- **The host's nspawn settings,** written by the host's activator from
  the slot's parameters into `/run/systemd/nspawn/<container>.nspawn`:

  ```
  # /run/systemd/nspawn/chat-prod.nspawn, written by rhi-prod's activator for slot b (slots.md, 12.1)
  [Exec]
  PrivateUsers=327680
  LinkJournal=try-host
  Capability=<name>                                             one per capability
  [Files]
  ReadOnly=yes
  Bind=/run/hearth-activator/top/@chat-prod.b.var:/var
  Bind=/run/hearth-activator/top/@chat-prod.<name>:<path>       one per data entry
  BindReadOnly=<source>:<path>                                  one per bind; Bind= if not read-only
  [Network]
  VirtualEthernet=no                                            or Bridge=<bridge> for network veth [M6]
  ```

  A settings file in `/run/systemd/nspawn/` takes full effect, and
  `systemd-nspawn@.service` runs nspawn with `--settings=override`, so
  these lines win over the unit's own `--link-journal=try-guest` and
  `--network-veth`. `LinkJournal=try-host` keeps the container's journal
  in its host's (operating.md, section 5). The root is
  found at `/run/machines/<container>`, a link to the slot's root (that
  nspawn's image search includes `/run/machines` is to verify). That
  top-level mount is `nodev,nosuid,noexec` (machine.md, 3.2); whether
  nspawn's bind of a root and of `/var` from it keeps those flags is to
  verify (section 15).

### 12.2 Activation: a restart, or live

A container's trial is live when the change is eligible (12.8), and
otherwise a restart: write the slot's settings and repoint the link,
each by rename over the old, and run `systemctl restart
systemd-nspawn@<container>.service`. There is no kernel, no soft reboot
and no one-shot. A change to the container's parameters is never live
(6.7, rule 2): nspawn reads them only when it starts.

- **The host's activator starts its containers.** At every start it
  makes each container's settings and link that `/run` lacks, from the
  default it recorded (12.6), then runs `systemctl start` on its unit,
  which leaves one already active alone. So a host reboot starts every
  container on its default, and an activator restart within a boot
  leaves them as they are.
- **No output enables them:** a host's output enables no
  `systemd-nspawn@` unit of its containers, and render refuses one that
  does (format.md, 5.11). Started at boot before its link exists, it
  would fail.

### 12.3 Trial

- **The unit must be stably active:** two consecutive
  `systemctl is-active` answers of `active`, polled within 30 seconds;
  `failed` fails at once. This is the bash fleet's `vul-host-control
  switch-container`, its only working trial and fall back.
- **Then the minimal criterion, read from the host:** the container's
  boot reached its default target with no failed unit but those its
  target's `bless.ignore_failed` lists (7.1; format.md, 5.13), through
  `systemctl --machine=<container> is-system-running` and
  `list-units --state=failed`, as operating.md (2.9) reads a container's
  units. 7.1's second condition, a builder of the slot's own, has no
  counterpart (12.5).
- **The host's builder asks to bless** once the criterion holds, and the
  container's blessing checks too (operating.md, section 3), or to fall
  back on a failed unit that counts. The bless records the link's target
  as the container's default in the host's state (12.6).
- **The window is the host's activator's:** no dead-man runs in a
  container, so the activator falls the container back itself when the
  trial's window ends with no bless.
- **A held bless** is a host's (7.4), ended at its ceiling by the
  host's activator, as the window is.
- **Its trial and facts are its own,** never its host's: the host's
  minimal bless leaves out its containers' nspawn units (7.1), and a
  container that fails to start when its host boots is reported in its
  own facts, running none on its default, while the host's trial goes
  on.
- **On failure,** the host's activator points the link back and restarts
  the old slot.
- **A container's first slot** is tried with no other slot running, the
  one exception to `try`'s guard (section 4): a failed first trial
  leaves the container stopped, its slot `fell back`.
- **A host reboot mid-trial** falls back by itself: the links live in
  `/run`, the host's activator makes them again at start from the
  defaults it recorded, and records `fell back` from the marker of an
  older boot. An activator restart within the boot keeps the trial, its
  window counted from the marker's `armed at` (6.4).

### 12.4 Pool hosts

- **One build, then a snapshot per container** (temper's `factory.md`,
  6.5): a lab pool runs many containers of one output.
- **The lean:** shift once, into a base subvolume the host's activator
  keeps owned from 0; snapshot that base for each container; and let
  nspawn map ids with idmapped mounts (`PrivateUsersOwnership=map`). One
  shift then serves every container. The alternative shifts each
  container's snapshot into its own range, one walk each.
- **VMs with a virtiofs root [later]** have the same shape: their slots
  live on the pool host, and a trial stops the VM, repoints its virtiofs
  devices and starts it, judged from the host (machine.md, 2.6).

### 12.5 A hearth in the container

- **The lean, open:** none in M5. The host's hearth prepares, tries and
  judges its containers from outside, runs their checks through the host
  (operating.md, section 2), and reports their facts as its own.
- **What one would bring:** the container's own bless, as hosts have, and
  checks run inside it. What it would lack: an activator with anything
  to own, since the container has no boot and its root is the host's to
  place. This depends on how operating.md's checks reach a container,
  and on whether a pool's containers need to report to the lab's engine
  directly.

### 12.6 The facts and the state machine

A container's facts are its host's activator's, read from outside:

| Fact | Read from |
|---|---|
| **running** | the subvolume of the container's root, by uuid, while its unit is active (through its leader's root, to verify how); `none` otherwise |
| **default** | the slot the host's state records as the container's default (activator.md, 7.1), which its link names at every host start |
| **trial** | the trial marker, `trial/<container>.<label>` (6.4) |
| **live** | the container's leader's mount table, `/proc/<leader>/mountinfo`, against the container's live record, `live/<container>` (12.8) |

- **Section 4's states, table and guards hold,** with a container's
  events: `try` is 12.2's restart; `try live` and `bless live` are
  12.8's; the unit stably active on the slot is `start here`, and on
  the other slot `start elsewhere`; a host start, or a restart of the
  container, is a new boot; `bless` comes from the host's builder
  (12.3), not from one running in the slot; `first bless` never happens
  (12.3's first slot). A `live-blessed` slot's one-shot is the host's
  record that the container's next start is its trial (12.8).
- **The invariant reads:** the slot the host links at start, and the one
  nspawn runs outside a trial, is blessed; a link to a slot on trial
  lasts until its bless, its fall back or the host's next start.
- **One mutating operation at a time** (activator.md, 4.2) holds across
  the host and its containers, so a host prepares and tries one target
  at a time (wire.md, 4.3).
- **What does not apply:** boot files and entries (5.6), the dead-man
  and the watchdog (5.5; the window is the activator's, 12.3), the
  kernel's rule (6.1), and the BIOS boot counter (8.4).

### 12.7 A container removed from its host's spec

A container leaves its host's spec when its target is deleted from the
repository or placed on another host. The host's builder then finds,
in the activator's facts, a container whose target its spec no longer
names (`target.containers`, format.md, 3.5), and asks its activator to
remove it (activator.md, 5.16):

1. **Stop it:** `systemctl stop systemd-nspawn@<container>.service`;
   remove its link in `/run/machines` and its settings in
   `/run/systemd/nspawn/`.
2. **Delete its slots:** each slot's subvolumes, nested ones first
   (5.1), then their records, manifests and any marker, then the
   container's own record, so no host start links it again.
3. **Keep its data:** its persistent subvolumes stay, unmounted and
   reported as orphaned as any target's data is (machine.md, 3.4),
   until a person deletes them; its identity and secrets stay with them.

- **Its outputs are then unpinned:** no slot holds them, so the store
  may collect them (store.md, section 7).
- **Each step is idempotent,** and a removal cut anywhere is finished
  by asking again; a container already gone is done.
- **A container placed on another host starts there from scratch:**
  its new host prepares a first slot with no data and makes its
  identity anew; nothing moves between hosts.

### 12.8 Live changes in a container

A container gets live changes with the same mechanics (6.6 to 6.9), run
by its host's activator from outside, since no hearth runs inside:

- **Eligibility** is 6.7's, read inside the container: the never-live
  list (format.md, 5.14) applies to its units, the same as a host's;
  rule 10 reads the host's kernel and the container's own facts (12.6);
  rule 11 reads the mount namespaces of the container's processes.
- **The delta** is built from the container's new slot, in the
  container's ids, as `@<container>.live.<n>` on the host's top level.
- **The swap enters the container's mount namespace.** A short-lived
  child of the host's activator builds each overlay in the host's
  namespace, where the layers' paths resolve, then `setns` into the
  mount namespace of the container's leader (`/proc/<leader>/ns/mnt`),
  moves the overlay beneath `/etc`, `/usr` or `/opt` there as L5 does,
  and reads the container's mount table back. The activator itself
  never leaves the host's namespace.
- **The units' actions** go through `systemctl --machine=<container>`
  (`stop`, `daemon-reload`, `reload`, `restart`, `reload-or-restart`,
  `start`, `show`), and tmpfiles through `systemd-run
  --machine=<container> --wait`; there is no builder inside to spawn
  anew. The host's builder plans, waits out the settle and asks the
  live bless (12.3).
- **The dead-man is the host's:** the window is the activator's deadline
  in the marker, as for any container trial (12.3). At its end, or when
  a step of the live fall back fails, the activator restarts the
  container on its default slot: the container's `reboot -ff`. A restart
  drops the overlays, which live only in the container's mount
  namespace.
- **The live bless records "the next start of this container is `b`'s
  trial"** in the host's state (activator.md, 7.1): the container's
  one-shot. The next start the host's activator makes, at a host boot or
  a restart it asks for, links `b` and tries it as 12.3 does; that
  bless records `b` as the default and deletes the delta.
- **A restart nobody asked for** (the unit restarting on its own, a
  person's `systemctl restart`) starts the linked slot, the base, and
  drops the live state; the activator finds it at its next reading of
  the facts and makes the `live-blessed` slot `prepared` again, as for a
  one-shot a firmware ignored (section 4).
- **A host reboot** ends every container's live state with the
  container, as it ends their trials.
- **Pool hosts** (12.4) whose containers' roots are idmapped mounts
  would need idmapped layers under the overlay: to verify.

## 13. Its world

`hearth-domain-slots` runs in domain worlds on the fake machine
(testing.md, sections 3 and 8), under the activator's root or alone,
with:

- **the fake machine's top level, firmware and GRUB** (machine.md,
  section 10), and its models of tiny-initramfs and systemd: a boot
  reaches its target or not, units fail, `is-system-running` answers, a
  soft reboot switches roots and keeps the boot id;
- **a scripted builder** on the link, which asks to place, try, bless,
  fall back and roll back, at awkward moments and with stale conditions;
- **faults from the seed** at every step: a crash of the activator, a
  power cut, a kernel that panics or hangs, a unit that fails, a hearth
  that never blesses, a firmware quirk, a torn FAT or NVRAM write;
- **a host's containers [M5]** on the fake machine's nspawn (testing.md,
  3.12): units that start or fail, failed units inside, a host reboot
  or an activator crash mid-trial, a container removed from the spec
  (12.7), checked against 12.6's invariant;
- **live changes** on the fake machine's mounts, overlays and PID 1's
  jobs (testing.md, 3.13): eligible and ineligible changes drawn from
  the seed, a reload that fails, a unit that dies during the settle, a
  crash or a power cut at every phase from L1 to L9, a person's soft
  reboot, a later change while live and its fall back to the previous
  delta, the native boot after a live bless; and [M5] the same in a
  container.

The referee checks section 4's invariant after every event, every cell
of the table is reached (transition coverage), and every request gets
exactly one answer. It computes 6.7's rules on its own from the fake
trees and specs, and each trial's kind must follow them. The VM tier
runs the trial's ending paths for real: a panic, a dead-man reboot, a
soft reboot and its fall back, and every live scenario
(testing.md, section 7).

## 14. What changes from os

- **Trial, bless and fall back are new.** os had two states, prepared and
  default, and an activation that was permanent at once; nothing ended a
  bad slot's boot.
- **Prepare shifts and verifies.** os built as root and snapshotted;
  hearth snapshots an unprivileged output and shifts it, and the shift
  checks the output's digest.
- **`/var` splits at prepare,** not after the packages phase of a build:
  a layer is one subvolume.
- **No template is completed at activation.** os rendered `fstab` and any
  declared file with MiniJinja inside the root-owned slot; the activator
  writes fixed files from typed data.
- **Secrets and identity enter at prepare,** from a root-only store. os
  copied `/secrets` into every root filesystem and rendered the
  machine-id from configuration.
- **Which slot runs** comes from the root subvolume's uuid, not from
  `/proc/cmdline`.
- **Slots are sealed and drift is checked against the slot's record,**
  so fstab and secrets no longer show as drift; `ek shell`, which made
  the running slot writable, is gone.
- **Nested subvolumes are deleted first,** where os deleted the `.var`
  first and looked for them in the root.
- **Containers' slots** move from a symlink under `/var/lib/machines`,
  switched by a script on one host, to records and links the host's
  activator keeps [M5].
- **Live changes replace patching in place.** os's and the bash fleet's
  `deploy patch` rewrote a running root's `/etc` (reset to a tag,
  replayed, committed again), with no trial; the bash fleet's failed
  live cutover (its #603) came from raw copies into a live `/etc`.
  hearth writes nothing in a running root: it mounts a verified delta
  of a prepared slot, swaps it atomically, tries it under a dead-man,
  and still boots the slot natively on trial before it becomes the
  default.

## 15. What is open

- **The trial window:** 10 minutes in M3, a place input sealed into the
  units; from M5 the builder computes it from the blessing checks
  (operating.md, section 3). Its right length depends on measuring the
  slowest target.
- **A blessed EFI default that stops booting:** keeping the previous
  slot first in `BootOrder` and arming `BootNext` to the default at each
  healthy start (8.4); depends on what one NVRAM write per boot costs,
  and on whether a roll back on a power cut during boot is acceptable.
  It would compete with a live bless's `BootNext` (7.6).
- **Failed units against a baseline:** blessing when the trial fails no
  unit the previous slot did not; depends on measuring `x1-prod` (7.1).
- **A dead-man that survives the soft reboot:** a unit of the old slot,
  `SurviveFinalKillSignal=yes`, that reboots unless the new slot's hearth
  cancels it through `/run`, would not depend on the new slot's systemd
  at all; untested, and depends on the VM tier.
- **Hardware watchdogs across a soft reboot:** whether systemd keeps
  `RuntimeWatchdogSec=` armed through the switch (to verify).
- **`systemctl reboot -ff` and `/run/nextroot`:** that `-ff` calls
  `reboot(2)` without the automatic soft reboot (to verify); if not, the
  dead-man's service uses `SuccessAction=reboot-immediate`.
- **The units' behaviour:** the triggering conditions on the command
  line and `/run`; the arming service's `systemctl start --no-block`
  early in the boot and the target's job timeout counted from then; and
  whether `IgnoreOnIsolate=yes` keeps the target's pending job through
  an isolate into `emergency.target` (to verify on the VM tier).
- **Remounting `/` after a soft reboot** with fstab's `subvol=` (to
  verify).
- **Holding a bless [M4]:** the `held` state's exact cells, and whether a
  hold survives the activator's restart by its marker alone; depends on
  wire.md, 5.4.
- **What a fell-back slot leaves for triage:** its journal is in a
  persistent subvolume only if a task declares one; depends on
  operating.md's telemetry.
- **Hashing contents again at prepare,** or trusting the builder's
  records (README, section 9; activator.md, 6.3).
- **Drift's period and its full depth;** depends on operating.md's
  signals and on the cost of hashing a root, once measured.
- **Containers:** whether a container runs a hearth of its own; shifting
  to each container's range, or once with idmapped mounts; how a pool
  names several containers of one target; whether nspawn finds images in
  `/run/machines`. Depends on M5's pool design and operating.md's checks.
- **nspawn's binds from the activator's top-level mount [M5]:** whether
  a root and a `/var` bound from `/run/hearth-activator/top`, mounted
  `nodev,nosuid,noexec`, keep those flags (to verify); if they do,
  containers need a mount of the top level without them.
- **Reading a container from its host [M5]:** whether
  `systemctl --machine=` needs a system bus inside the container, and
  how the running slot's subvolume is read through the container's
  leader (to verify on the VM tier, testing.md, 7.3).
- **Live changes on a real machine,** to verify on the VM tier
  (testing.md, 7.3) on trixie's systemd 257 and forky's 262, Incus with
  OVMF:
  - real root and real ids: `trusted.overlay.*` whiteouts and opaque
    directories honoured as the `user.*` forms were; owners, setuid
    programs (`/usr/bin/passwd` in a delta), v2 capabilities and ACLs
    through the overlay; a delta built from a shifted slot exactly what
    L2 expects;
  - `MOVE_MOUNT_BENEATH` from the host's namespace, PID 1's shared `/`,
    with real services: `ProtectSystem=strict` and `full`,
    `ReadWritePaths=/etc`, `InaccessiblePaths=` and `BindPaths=` beneath
    `/etc` and `/usr`, `PrivateTmp=`, `DynamicUser=` and
    `RootDirectory=` units;
  - PID 1 around a swap: `daemon-reload` with new, changed and removed
    fragments of running units, new `.wants` links and masks;
    `NeedDaemonReload=`; whether dynamic `etc.mount`, `usr.mount` and
    `opt.mount` units appear, whether `daemon-reload` disturbs them, and
    what shutdown does with them;
  - soft reboots: the overlays gone with the old root when
    `/run/nextroot` is set; `/etc`'s and `/usr`'s kept and `/opt`'s
    dropped without it (6.8);
  - the base's dead-man started after boot from
    `/run/hearth-trial/<target>.a`; `reboot -ff` landing on the default;
    `BootNext` to a trial entry surviving days of uptime and consumed by
    a power cut, on OVMF and real firmware;
  - timing on a real tree of 100k inodes in `/usr`: L1's build, L2's
    walk, the swap under a load of opens;
  - Debian's specifics: `/opt` present in outputs; trixie's kernel
    configuration without overlayfs's `XINO_AUTO`, `INDEX` or `METACOPY`
    defaults; AppArmor profiles on overlay paths, exec transitions for
    `/usr/bin` and `/opt` programs; dbus-daemon's `ReloadConfig` after a
    policy change; glibc reading `nsswitch.conf`, `hosts` and
    `resolv.conf` afresh after a swap; `sshd -t` failing a reload
    reported as a failed job;
  - the unit actions end to end: stop, swap, start for a changed binary;
    a reload for a configuration change; newly wanted units started; a
    unit stopped whose file and binary the delta removes; every job's
    result reaching the builder;
  - the fall backs: 8.5 end to end, including a unit that fails only
    after the settle; an activator killed in each phase; the power
    pulled in each phase; a person's soft reboot;
  - whether overlayfs applies a lower mount's `noexec` or `nosuid` to
    the merged files, which decides whether the layers need their own
    source mount (6.8).
- **hearth itself, live:** its own binary and units are never live
  until live changes are proven; then the activator would restart into
  the new release and recover. Depends on the VM tier's results.
- **`/var` paths live:** whether a change may make paths beneath `/var`
  that hearth's own tmpfiles lines create, run live (6.7, rule 3);
  depends on the tasks that would use it.
- **AppArmor and networkd live:** a profile change with `apparmor_parser
  -r`, a `.network` change with `networkctl reload`, which may cut the
  machine off; depends on the fleet needing them.
- **The settle's length:** 60 seconds to start (6.8, L7); depends on
  measuring the slowest services' restarts.
- **A live roll back to the other blessed slot,** its files over the
  running one's, refused for now (section 4); depends on the need for
  undoing a change faster than a trial.
