# Machines

Provisional, 2026-10-09. What a machine looks like to hearth: the kinds
of machine it runs on, the btrfs top level and its mounts, how each kind
boots a slot, the kernel's command line, the initramfs, bootstrap from a
live system, and the firmware's quirks. M3 builds the EFI path and
bootstrap; M3b builds the BIOS path; containers and pool hosts are M5
(slots.md, section 12). A slot's life on this layout is slots.md's, and
the process that writes the layout is activator.md's. What is open is
in section 12.

## 1. In one page

- **A machine is one disk: a FAT boot partition and one btrfs
  filesystem.** On EFI the boot partition is the ESP; on BIOS it holds
  GRUB and its environment. Everything else is a subvolume at the btrfs
  top level.
- **The top level holds only subvolumes, named by target:** two slots,
  each a root and a `/var`; up to three class subvolumes for persistent
  data, `core`, `data` and `cache`; the builder's store; the
  activator's state; local snapshots [M5]; received backups [M6]. A
  host's containers have their slots and classes there too, under their
  own targets' names [M5]. Slots and classes are siblings, never nested,
  and only the activator mounts the top level.
- **Persistent data is a directory per `data` entry in its class's
  subvolume,** bound at the path its task declares: one subvolume per
  class, as os and the bash fleet had, so a snapshot is a class's and
  atomic across its services.
- **The root is read-only twice over:** its subvolume is sealed
  read-only when prepare ends, and it is mounted `ro`. `/var`, the
  classes, the store and the state are writable mounts beside it. A
  mutable development machine is the one exception (2.8).
- **fstab is the activator's, written from typed data** at prepare: the
  filesystem by UUID, the slot's own subvolumes, the classes at
  `/srv/<class>`, and a bind per entry at the path its task declares.
  No template, and no device in an output.
- **EFI boots each slot's kernel directly.** The ESP holds
  `\vmlinuz.<target>.<label>` and `\initrd.img.<target>.<label>`; each
  slot has two `Boot####` entries at fixed numbers, normal and trial,
  rewritten only when their command line changes. The normal one enters
  `BootOrder` when the slot is blessed; the trial one never does, and a
  trial is `BootNext` to it, checked with `BootCurrent`.
- **BIOS boots GRUB from the post-MBR gap [M3b].** GRUB lives on the
  FAT partition with a fixed `grub.cfg` and a pair of environment files
  it can write, so `next_entry` is a real one-shot, cleared before the
  kernel loads, and a boot counter falls back to the previous blessed
  slot. It reads kernels from the slot subvolumes.
- **The command line is hearth's, then the target's:** `root=UUID=`,
  `rootflags=subvol=`, `ro`, `panic=10`, then the `boot` part's
  parameters, consoles among them, in at most 2048 bytes.
- **tiny-initramfs stays,** configured per hardware profile: no
  automatic modules, an explicit list, no build host's microcode. It finds the
  root by UUID and panics on any error, which `panic=10` turns into a
  reboot. hearth uses Debian's package, in trixie and in forky.
- **Bootstrap is one command, run as root from a live system:**
  partition, format, make the top level, build as the unprivileged
  `hearth` user, place the output into slot `a`, write its boot files,
  and make it the default.
- **Secure Boot is off:** a Debian kernel cannot boot from a direct
  entry under it. Firmware that prunes, reorders or ignores entries is
  met by keeping hearth's entries few, fixed and checked after every
  boot.

## 2. Kinds of machine

| Kind | Firmware | Disk, boot partition | How a slot is tried | Watchdog | Examples | Milestone |
|---|---|---|---|---|---|---|
| bare metal on EFI | the vendor's UEFI | NVMe; ESP | live, soft reboot, or `BootNext` | iTCO where the chipset has it (to verify per machine) | `x1-prod` | M3 |
| Incus VM with its own disk | OVMF | a virtio disk; ESP | live, soft reboot, or `BootNext` | iTCO on q35 | the VM tier's machines (testing.md, section 7) | M3 |
| IaaS VM on BIOS | the provider's BIOS | a virtio disk; FAT environment partition | live, soft reboot, or `next_entry` | none known | `rhi-prod`, `ion-prod` | M3b |
| IaaS VM on EFI, NVRAM unreliable | the provider's UEFI | a virtio disk; ESP | open (9.4) | none known | vul | later |
| container | none: the host's kernel | slots on the host's top level | live, or restart | the host's | `chat-prod` and `mail-prod` on `rhi-prod`; `mail-lab` in a lab pool | M5 |
| VM with a virtiofs root | Incus direct kernel boot | slots on the pool host's top level | stop, repoint, start | the host's | the bash fleet's lab | later |

### 2.1 Bare metal on EFI

The MVP's machine is `x1-prod`, a laptop with an NVMe disk. It runs os
today, with direct EFI-stub entries labelled `a` and `b`, and already
shows the facts apart: running slot `b`, `BootOrder` default `a`.
hearth installs it afresh on a wiped disk, as every machine (section 8):
nothing of os's is kept. A person is at hand, so a hang no watchdog
covers is recoverable; the machine is held to the same invariants as
the others all the same.

### 2.2 Incus VMs with their own disk

The MVP's test machines (testing.md, section 7). Unlike the bash
fleet's lab VMs they have a real disk, a real ESP and real firmware
variables, so bootstrap, entries, `BootNext` and fall back run as on
bare metal.

- **`security.secureboot=false`:** Incus enables Secure Boot by default,
  and a direct entry for a Debian kernel then fails to load (9.5).
- **OVMF keeps its variables** in `qemu.nvram` beside the instance
  volume, regenerated only when missing or invalid, or when the Secure
  Boot or CSM settings change; `incus publish` drops them. At every
  boot it prunes and reorders the options `BootOrder` lists (9.1).
- **q35 has the ICH9 TCO watchdog,** whose default action is a reset,
  driven by `iTCO_wdt` and `RuntimeWatchdogSec=`.
- **The console is `ttyS0`;** the disk is Incus's default bus,
  virtio-scsi (to verify), which sets the initramfs's modules (7.2).
  BIOS is tested with `security.csm=true`, which boots SeaBIOS (to
  verify) [M3b].

### 2.3 IaaS VMs on BIOS [M3b]

- **Two of three production VPSes are BIOS:** `rhi-prod` on ServeRhino
  (primary MX, Forgejo, Matrix) and `ion-prod` on IONOS (backup MX, VPN
  hub). No BIOS host has been provisioned by os's code, whose BIOS path
  has three mismatches and keeps GRUB's environment on btrfs.
- **Each provider has its own disk bus,** hence its own initramfs
  modules (7.2), and a serial console as its last recovery path, which
  needs a root password hash that is not locked: a secret applied to the
  slot's `/etc/shadow` (slots.md, 5.4).
- **Bootstrap runs from the provider's rescue system** (section 8).

### 2.4 IaaS VMs on EFI with unreliable NVRAM [later]

Vultr's firmware can discard its variables when a disk moves between
contexts; the bash fleet booted GRUB from the removable path
(`\EFI\BOOT\BOOTX64.EFI`) with no slot entries. A trial in NVRAM is not
reliable there; 9.4 gives the lean.

### 2.5 Containers [M5]

No kernel, no boot partition, no entries: a container's slots and
class subvolumes are subvolumes on its host's top level, prepared
by the host's activator, and activating one restarts its nspawn unit
(slots.md, section 12). Sections 4 to 7 and 9 do not apply to it; no
fstab mounts anything of it (3.3), and it is never bootstrapped (8.4).
The fleet runs three today, `carbon-forge`, `rhi-chat` and
`vul-agent-control`, on carbon, rhi and vul.

### 2.6 VMs with a virtiofs root [later]

The bash fleet's lab booted Incus VMs straight into a host subvolume
over virtiofs, with `root=/dev/null rootfstype=virtiofs` (tiny-initramfs
accepts only a `/dev/` name, so the root's tag was `/dev/null`) and
`io.cache=unsafe` on the root (`mmap(2)` otherwise fails with
`ENODEV`). virtiofs rejects `subvol=`, and the guest cannot choose its
slot. Such a VM is closer to a container: its slots live on the pool
host, whose activator tries one by stopping the VM, repointing its
devices and starting it. hearth's test VMs do not use it.

### 2.7 What placement says

Placement is data of the environment (format.md, 3.4), in the spec's
`meta`. What this document takes from it:

| Field | Values | Decides here |
|---|---|---|
| `kind` | `metal`, `vm`, `container` [M5] | section 2; the initramfs's microcode (7.1) |
| `firmware` | `efi`, `bios` | sections 4 and 5; checked against the machine record |
| `hardware` | `7th-gen`, `incus`, `ionos`, `serverhino`, `vultr` | nothing directly: the `boot` task keys its modules and parameters, consoles among them, by it (format.md, 5.4) |
| `on` | the host's target [M5] | slots.md, section 12 |
| `mutable` | `true`, `false` | 3.3's `/` line `rw` and no seal at prepare, on a machine that accepts it (2.8; development.md, section 4) |

- **The watchdog is the machine's own fact:** the activator looks for a
  device under `/sys/class/watchdog/` when it prepares a slot, on the
  machine the slot will run on (slots.md, 5.5).
- **The trial window** is a place input, 10 minutes in M3, sealed into
  the slot's units (activator.md, 5.1); a container's is kept by its
  host's activator instead (slots.md, 12.3) [M5].

### 2.8 Development machines [M3b]

- **An Incus VM cloned from a base image** (development.md, section 3):
  2.2's kind of machine, with its own disk, ESP and OVMF variables,
  made by `incus copy` of a VM bootstrapped once from a small target,
  then given its own target by `hearth retarget`, which places and blesses
  its first slot as bootstrap does (8.2).
- **It may be mutable:** bootstrapped with `--accept-mutable` (8.1), it
  takes targets whose placement says `mutable`, whose slots are not
  sealed and whose root is mounted `rw` (3.3). Every other rule of this
  document holds: the same top level, boot entries, trials and fall
  backs. A machine that does not accept is never mutable.
- **Every clone shares the base's filesystem UUID** and partition
  identifiers; no machine sees two disks of one image, so nothing
  confuses them.

## 3. The btrfs top level

### 3.1 Subvolumes

```
btrfs filesystem, UUID 0f3c9a6e-…, label hearth     top level: subvolid 5, mounted only by the activator
├── @x1-prod.a              slot a's root           a snapshot of an output, shifted, sealed read-only
├── @x1-prod.a.var          slot a's /var           a reflink copy of that output's /var
├── @x1-prod.b              slot b's root
├── @x1-prod.b.var          slot b's /var
├── @x1-prod.live.3         a live delta            only while a live state exists (slots.md, 6.6)
├── @x1-prod.core           the core class          one directory per entry: journal, iwd
├── @x1-prod.data           the data class          one directory per entry: home-free
├── @hearth.store           the builder's store     owned by hearth, 0700; its layers nested in it (store.md, section 2)
├── @hearth.state           the activator's state   root, 0700 (activator.md, section 7)
├── @hearth.snapshots [M5]  class snapshots         root, 0700: <target>.<class>/<seq>, read-only (operating.md, 7.1)
└── @hearth.backups [M6]    received backups        root, 0700, on a backup peer: <from>.<class>/<seq> (operating.md, 7.3)
```

| Subvolume | Its root inode | Made by | Removed by |
|---|---|---|---|
| `@<target>.<label>` | the output's `/`; read-only flag set at the end of prepare | prepare: a snapshot (slots.md, 5.1) | the next prepare of that slot |
| `@<target>.<label>.var` | the output's `/var`, `0:0 0755` | prepare (slots.md, 5.3) | the next prepare of that slot, nested subvolumes first |
| `@<target>.<class>`: `core`, `data`, `cache` | `0:0 0755`; one directory per `data` entry of the class, each seeded once from the output's content at the entry's path | the first prepare that declares an entry of the class (slots.md, 5.3) | never by hearth (3.4) |
| `@<target>.live.<n>` | holds `etc`, `usr` and `opt` with the new slot's metadata: its changed files as reflinks, and 0/0 whiteouts; read-only once verified | a live trial (slots.md, 6.8, L1) | the native bless of its slot, a live fall back, or the start that finds no record naming it |
| `@hearth.store` | `hearth:hearth 0700` | bootstrap | never |
| `@hearth.state` | `0:0 0700` | bootstrap | never |
| `@hearth.snapshots` [M5] | `0:0 0700`; a directory `<target>.<class>` per class snapshotted, holding read-only snapshots `<seq>`, this machine's own, for its targets and its containers' | the first snapshot | each snapshot by its class's retention, or once sent and no longer needed (operating.md, 7.1) |
| `@hearth.backups` [M6] | `0:0 0700`; `<from>.<class>/<seq>`, read-only snapshots received from the targets that send to this backup peer | a backup peer's first receive | each by the sender's retention (operating.md, 7.3) |
| `@<target>.<class>.restoring` [M5] | a writable snapshot of one of the class's snapshots | a class's restore (operating.md, 7.4) | renamed to `@<target>.<class>` by the restore's swap |
| `@<target>.<class>.before.<seq>` [M5] | the class a restore replaced | the restore's swap | an operator or the engine |

- **One tree for both snapshots and backups:** `@hearth.snapshots`
  holds this machine's own snapshots of its classes, whatever took
  them, and each snapshot's record says whether the local policy keeps
  it, whether it was sent, and whether the peer confirmed it
  (activator.md, 7.1). When the local and the backup period meet, one
  snapshot serves both; an incremental send's parent is a local
  snapshot the backup keeps; and one walk prunes them all. What a
  backup peer receives from others goes to `@hearth.backups` instead:
  another target's data, kept by its sender's retention, which no local
  policy may prune and no local restore lists.
- **A host's containers [M5]** add the same names under their own
  targets: on `rhi-prod`, `@chat-prod.a`, `@chat-prod.a.var`,
  `@chat-prod.b`, `@chat-prod.b.var`, and `@chat-prod.data` and the
  other classes its `data` entries use, beside the host's own
  (slots.md, 12.1), and their snapshots in the host's
  `@hearth.snapshots`. Its journal has no entry of its own: it lives in
  the host's (operating.md, section 5). The table above holds for them,
  the host's activator acting for the container; os's
  `@<target>.machines` under `/var/lib/machines` has no successor.
- **Slots exist once prepared:** os made empty placeholders at format
  time; hearth makes a slot by snapshot, so `@x1-prod.b` is absent until
  the first prepare into `b`. No build happens at the top level: os's
  `@T`, `@T.var` and `@T.install` are layers in the store now
  (store.md, section 4). Classes, too, exist once an entry uses them:
  os made `@T.srv`, `@T.srv.core`, `@T.srv.data` and `@T.srv.cache` at
  format time; hearth makes `@x1-prod.cache` only when an entry is
  `cache`, and has no `@T.srv`, since `/srv` is the root's.
- **In a slot, nested subvolumes appear only at runtime,** made by
  systemd in its `/var` (`/var/lib/machines`, `/var/lib/portables`), and
  are deleted before their `.var`, found with
  `BTRFS_IOC_GET_SUBVOL_ROOTREF`.
- **The default subvolume stays 5:** GRUB resolves its paths from it
  (5.4).
- **No filesystem-wide option is set.** Most btrfs options apply to the
  whole filesystem, from its first mount, the root's. hearth passes only
  `subvol=`, and never `user_subvol_rm_allowed`: the builder deletes a
  layer by emptying it through its tree helper and calling `rmdir(2)`
  (store.md, section 5).

### 3.2 Mounts

| Path | Source | Mounted by | Options |
|---|---|---|---|
| `/` | `@<target>.<label>` | tiny-initramfs, from `root=` and `rootflags=`; then systemd remounts it from fstab | `ro,subvol=/@x1-prod.b` |
| `/var` | `@<target>.<label>.var` | systemd, from fstab | `nodev,nosuid` |
| `/var/lib/hearth/state` | `@hearth.state` | systemd, from fstab | `nodev,nosuid,noexec` |
| `/var/lib/hearth/store` | `@hearth.store` | systemd, from fstab | `nodev,nosuid,noatime` (store.md, 9.1) |
| `/srv/core`, `/srv/data`, `/srv/cache`, each class some entry uses | `@<target>.<class>` | systemd, from fstab | `nodev,nosuid` |
| each `data` path but `/srv/<class>/<name>` | a bind of `/srv/<class>/<name>`, the entry's directory | systemd, from fstab, after its source's mount | `bind`; the class mount's `nodev,nosuid` carried over (to verify) |
| `/run/hearth-activator/top` | the top level, `subvolid=5` | the activator, when it starts | `nodev,nosuid,noexec`; `/run/hearth-activator` is `0700`, root's |
| `/run/hearth-activator/boot` | the boot partition | the activator, for each write, then unmounted | `vfat`, `umask=0077,nodev,nosuid,noexec` |
| `/run/nextroot` | a slot's root, for a soft reboot | the activator (slots.md, section 6) | `ro,subvol=/@x1-prod.a` |
| `/run/hearth-activator/layers` | a clone of the top level: the live overlays' layers | the activator, when it starts | `ro`, private; without `noexec` and `nosuid` (slots.md, 6.8) |
| `/etc`, `/usr/local`, `/opt`, while a live state runs | overlayfs, source `hearth-live`: a delta over the running slot's directory | the activator (slots.md, 6.8) | `ro,nodev`; `nosuid` on `/etc` only |

- **The store and the state hang below `/var`,** which is per slot, so
  each slot's `/var` gets their mount points at prepare (slots.md, 5.3).
  systemd orders a mount after its parent path's (systemd.mount(5)), so
  fstab needs no `x-systemd.requires-mounts-for=`.
- **A bind waits for its source:** systemd orders a bind mount after the
  mount of the path it binds from, as well as after its own parent's,
  so `/var/log/journal` comes up after both `/srv/core` and `/var`. A
  class mount that fails fails its binds, and the boot isolates
  `emergency.target` rather than start a service on the slot's own copy
  of the path (slots.md, 5.3).
- **The snapshots are not mounted:** a person reads one through
  `hearth shell`, read-only (wire.md, 3.7), and `@hearth.snapshots`
  stays the activator's.
- **The store is `nosuid`:** it holds setuid binaries owned by the
  builder's range, behind a `0700` root. Builds run as root inside their
  namespaces and need no setuid binary (build.md, section 8).
- **The boot partition is not in fstab:** only the activator uses it,
  mounted for one write at a time, so a power cut rarely finds it
  dirty.

### 3.3 fstab

The activator writes `/etc/fstab` into each slot at prepare
(slots.md, 5.5), from typed data: the filesystem's UUID from its machine
record, the target and label, and the `data` part's entries. An output
holds no fstab of its own; the activator replaces whatever is there.

```
# /etc/fstab of slot x1-prod.b, written by hearth's activator at prepare (machine.md, 3.3)
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /                      btrfs  subvol=/@x1-prod.b,ro                   0  0
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /srv/core              btrfs  subvol=/@x1-prod.core,nodev,nosuid     0  0
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /srv/data              btrfs  subvol=/@x1-prod.data,nodev,nosuid     0  0
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /var                   btrfs  subvol=/@x1-prod.b.var,nodev,nosuid     0  0
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /var/lib/hearth/state  btrfs  subvol=/@hearth.state,nodev,nosuid,noexec 0  0
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /var/lib/hearth/store  btrfs  subvol=/@hearth.store,nodev,nosuid,noatime 0  0
/srv/data/home-free                         /home/free             none   bind                                    0  0
/srv/core/iwd                               /var/lib/iwd           none   bind                                    0  0
/srv/core/journal                           /var/log/journal       none   bind                                    0  0
```

- **Lines in a fixed order:** the subvolumes' mounts in path order,
  then the binds in path order of their targets, so the file is a
  function of its inputs. The order of lines decides nothing: systemd
  orders the mounts by their paths and a bind by its source.
- **The filesystem by UUID,** never by device: the device name differs
  between a live system, a rescue system and the machine itself, and a
  provider may change it.
- **A bind by its source's path,** `/srv/<class>/<name>`, so a restore
  that swaps a directory's name or a class's subvolume is mounted at the
  next boot with no fstab to change (operating.md, 7.4).
- **Every mount point already exists** in the sealed root or in the
  slot's `/var`, because a read-only root cannot get one later, and
  systemd would otherwise try to create it (slots.md, 5.3 and 5.5):
  `/srv/<class>` for each class used, and each bind's path.
- **A container is in no fstab [M5].** Its host's fstab holds the
  host's own lines only; nspawn makes the container's root and binds its
  `/var` and its entries' directories from the activator's mount of the
  top level, as the host's nspawn settings say (slots.md, 12.1). The
  container's own `/etc/fstab` is written with no line, so its systemd
  mounts nothing.

### 3.4 Names and data entries

- **Target names** are lowercase letters, digits and `-`, with no dot,
  since `.` separates the parts of a subvolume's name; `hearth` is
  reserved for `@hearth.*`. After a target, `a`, `b`, `core`, `data`,
  `cache` and `live` are the only names, and after a class `restoring`
  and `before` are reserved for restores [M5].
- **A `data` entry** (format.md, 5.7) has a name, a class and a path.
  The name is one segment of lowercase letters, digits and `-`, at most
  32 bytes, unique within its target across classes, and names the
  entry's directory in its class's subvolume. The path is absolute and
  normal, beneath `/srv`, `/var` or `/home`, not `/var/lib/hearth` or
  beneath it, not beneath another entry's path, and beneath
  `/srv/core`, `/srv/data` or `/srv/cache` only as the entry's own
  `/srv/<class>/<name>`, which needs no bind. At most 64 entries a
  target, each a line of fstab at most. Render refuses an entry that
  breaks a rule, and the activator refuses it again (activator.md, 2.2).
- **A class's contents belong to its entries:** the subvolume is made
  with a root of `0:0 0755`, and each entry's directory when the class
  lacks it, seeded with what the output holds at the entry's path, so
  packages' files there are not hidden by an empty bind (slots.md, 5.3);
  the slot's own `tmpfiles.d` lines set owners and modes at boot, by
  name. A container's `0:0` is its own root, in the ids its slots are
  shifted to (slots.md, 12.1) [M5].
- **Beside an entry's directory,** only a prepare's `<name>.new` while
  it seeds (slots.md, 5.3), and a restore's `<name>.restoring` and
  `<name>.before.<seq>` (operating.md, 7.4); entry names hold no dot,
  so none of these is ever an entry.
- **An entry that disappears keeps its directory,** unbound and
  reported as orphaned; a class left with no entry keeps its subvolume,
  unmounted, orphaned too, and its snapshots. Deleting persistent data
  is a person's decision (section 12).
- **An entry that changes class moves no data:** its directory stays in
  the old class, and the new one would get a fresh, seeded directory.
  So the activator refuses a place whose entry has no directory in its
  class but an orphaned one of its name in another of the target's
  classes, and names it; a person moves it, a reflink copy within one
  filesystem (`cp --reflink`), or deletes it.
- **Why a subvolume per class and not per entry:** a class is a policy's
  unit, so a snapshot is atomic across every service in it; there are
  three subvolumes to snapshot, prune and mount instead of one per
  entry; and fstab grows by one bind per entry. A subvolume per entry,
  for one that needs a policy, a quota or a restore of its own, is open
  (section 12).

## 4. EFI boot

### 4.1 The ESP

```
ESP: GPT partition 1, 512 MiB, FAT32, flag esp
├── vmlinuz.x1-prod.a        slot a's kernel
├── initrd.img.x1-prod.a     slot a's initramfs
├── vmlinuz.x1-prod.b        slot b's kernel
├── initrd.img.x1-prod.b     slot b's initramfs
└── EFI/…                    other loaders, if any: never touched by hearth
```

- **Flat files, one pair per slot,** as os. The EFI stub loads `initrd=`
  from the volume the kernel came from, so both live on the ESP.
- **Written at prepare, for that slot only** (slots.md, 5.6): copied
  from the slot's `boot/vmlinuz-<version>` and
  `boot/initrd.img-<version>`, synced, read back and checked against the
  slot record's digests. The slot's entry left `BootOrder` first (4.4),
  so a cut leaves torn files that nothing boots.

### 4.2 Entries

Each slot has two direct EFI-stub entries on the same files, normal and
trial:

| Field | Normal | Trial |
|---|---|---|
| number | `4800` for `a`, `4801` for `b` (`0x48` is `H`) | `4810` for `a`, `4811` for `b` |
| label | `<target>.<label>`, such as `x1-prod.b` | `<target>.<label>.trial` |
| device | `HD()` of the ESP: partition 1 of the disk the btrfs filesystem is on | the same |
| loader | `\vmlinuz.<target>.<label>` | the same |
| optional data | the whole command line, UCS-2 (section 6) | the same, then `hearth.trial=<target>.<label>` |
| attributes | active | active |
| `BootOrder` | from the slot's bless until its next prepare | never: only `BootNext` names it |

The numbers are chosen at bootstrap, or the next free ones above if the
firmware uses these, and kept in the machine record. Made with
`efibootmgr`, one element per argument, never through a shell:

```
/usr/bin/efibootmgr --quiet --create-only --bootnum 4811
    --disk /dev/nvme0n1 --part 1 --label x1-prod.b.trial
    --loader \vmlinuz.x1-prod.b --unicode
    initrd=\initrd.img.x1-prod.b root=UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90 rootflags=subvol=@x1-prod.b ro panic=10 hearth.trial=x1-prod.b
```

- **Why a trial entry:** the dead-man must be armed before any mount
  (slots.md, 5.5), and the command line is what a one-shot boot carries
  that nothing on disk can lose.
- **`--create-only`** makes the entry without adding it to `BootOrder`;
  `--create` would add it.
- **The command line is one argument.** `--unicode` makes the
  positional arguments UCS-2 and joins them with a UCS-2 NUL, and the
  stub stops at the first NUL, so a command line split into arguments
  loses all but its first word.
- **`initrd=` is written with a backslash.** efibootmgr converts `/` in
  `--loader`, not in the command line.
- **The disk is found afresh** at each start, from
  `/sys/fs/btrfs/<fs uuid>/devices/` (activator.md, 7.4).
- **Other calls,** each an argument vector of `hearth-protocol-efi`:

  ```
  efibootmgr --quiet --bootnum 4801 --delete-bootnum      remove an entry, before re-creating it
  efibootmgr --quiet --bootnext 4811                      trial: the trial entry (4.4)
  efibootmgr --quiet --delete-bootnext                    clear a one-shot left behind
  efibootmgr --quiet --bootorder 4801,4800,0001,0002      bless (4.4)
  ```

### 4.3 Writing NVRAM

- **An entry is written only when it is missing or differs** in label,
  loader, partition or command line: its files change at every prepare,
  its variable does not. To change it, the activator deletes it, then
  creates it at the same number: `--create-only --bootnum` exits 40 when
  the number exists.
- **Only hearth's own numbers** are ever deleted or created; foreign
  entries keep their place in `BootOrder`, behind hearth's.
- **Writes per rollout:** one `BootOrder` write when prepare takes the
  slot out of it, if it was in; one `BootNext`, which the firmware
  deletes; one `BootOrder` write at bless; the two entries only if the
  command line changed. NVRAM is flash, and OVMF's store is 256 KiB.
- **The activator never writes variables itself;** efibootmgr clears
  efivarfs's immutable flag through libefivar.

### 4.4 Default, trial and bless

- **The default** is the first of the target's normal entries in
  `BootOrder`. With both slots blessed, `BootOrder` reads: the default,
  then the other slot, then every foreign entry in the order it had. A
  firmware that fails to load the default falls through to the other
  blessed slot.
- **A slot about to be prepared leaves `BootOrder` first,** so the
  firmware never falls through to a slot being written.
- **A trial is `BootNext` to the trial entry.** The boot manager deletes
  it before it transfers control to the entry (UEFI 2.10, 3.1.2; edk2's
  `BdsEntry()`), so any later reset boots `BootOrder`, and an entry that
  fails to load falls back to `BootOrder` at once. The trial's boot
  checks `BootCurrent` and `BootNext` (slots.md, 6.3).
- **A bless rewrites `BootOrder`** with the slot first, and reads it back
  (slots.md, 7.3).

### 4.5 Reading the firmware

- **The activator reads variables, not efibootmgr's text.** os parsed
  three shapes of `efibootmgr -v` output, the UTF-16 options glued to
  the path as hex among them. The variables are sized and stable:

  ```
  /sys/firmware/efi/efivars/<Name>-8be4df61-93ca-11d2-aa0d-00e098032b8c
  BootCurrent, BootNext      4 attribute bytes, then a u16
  BootOrder                  4 attribute bytes, then u16 entries
  Boot4801                   4 attribute bytes, then an EFI_LOAD_OPTION: attributes u32, path list length u16,
                             a NUL-terminated UCS-2 description, device path nodes, optional data (the command line)
  SecureBoot                 4 attribute bytes, then a u8
  ```

- **`hearth-protocol-efi` decodes them** into typed values: the entry's
  label, its `HD()` node (partition number, start, size, GPT GUID), its
  file path and its command line. It also builds efibootmgr's argument
  vectors. The domain sees neither bytes nor text.
- **The ESP's GPT GUID** is read back from the first entry bootstrap
  makes and kept in the machine record, so drift can tell an entry for
  another disk.

## 5. BIOS boot [M3b]

### 5.1 The disk

```
MBR: GRUB's boot.img
post-MBR gap, sectors 1 to 2047: GRUB's core.img
partition 1: 1 MiB to 65 MiB, FAT, flag boot      the boot partition: GRUB, its configuration, its environment
partition 2: 65 MiB to the end, btrfs             the top level (section 3)
```

- **An msdos table with the core image in the post-MBR gap,** as os and
  the bash fleet. A GPT `bios_grub` partition would also work; one
  layout per firmware is enough, and msdos's limit of 2 TiB is far above
  any VPS here.
- **The boot flag is on partition 1,** for BIOSes that boot only a disk
  with an active partition; GRUB's own code ignores it.
- **Partition numbers match EFI's:** the boot partition is 1 and btrfs
  is 2 on both.

### 5.2 The boot partition

```
/grub/i386-pc/…     GRUB's modules and core image source, from grub-install at bootstrap
/grub/grub.cfg      fixed: written once, at bootstrap, from typed data (5.3)
/hearth/env.0       a GRUB environment block, 1024 bytes
/hearth/env.1       the same variables, written second (5.5)
```

- **All of GRUB is on FAT,** its prefix included, so GRUB can write its
  environment there and the btrfs top level holds only subvolumes. os
  kept `grub.cfg` and `grubenv` on btrfs, where GRUB's read-only btrfs
  driver makes `save_env` fail.
- **GRUB is installed once, at bootstrap,** by the live system's
  `grub-install` (8.2). hearth does not reinstall it in M3b: GRUB is
  shared by both slots, so updating it cannot be tried in a slot
  (section 12).

### 5.3 grub.cfg

Written at bootstrap from typed data: the target, both labels, the
btrfs UUID and the serial console the `boot` part names. It never
changes after, so a power cut cannot tear it. The exact script is to
verify on GRUB 2.12, trixie's:

```
# written by hearth at bootstrap (machine.md, 5.3)
insmod part_msdos
insmod fat
insmod btrfs
insmod loadenv
insmod test
serial --speed=115200 --unit=0 --word=8 --parity=no --stop=1
terminal_input serial console
terminal_output serial console

set env=($root)/hearth
set loaded=
if load_env -f ${env}/env.0 default previous tries next_entry; then
    set loaded=env.0
    set other=env.1
elif load_env -f ${env}/env.1 default previous tries next_entry; then
    set loaded=env.1
    set other=env.0
fi
set fallback="${default}"
set hearth_trial=
export hearth_trial
if [ -n "${loaded}" -a -n "${next_entry}" ]; then
    set trial="${next_entry}"
    set next_entry=
    if save_env -f ${env}/${loaded} next_entry; then
        set default="${trial}"
        set hearth_trial="hearth.trial=${trial}"
        save_env -f ${env}/${other} next_entry
    fi
elif [ -n "${loaded}" -a -n "${previous}" ]; then
    if [ "${tries}" = 0 ]; then
        set default="${previous}"
    else
        if [ "${tries}" = 3 ]; then set tries=2
        elif [ "${tries}" = 2 ]; then set tries=1
        else set tries=0
        fi
        if save_env -f ${env}/${loaded} tries; then
            save_env -f ${env}/${other} tries
        fi
    fi
fi
set timeout=3

search --no-floppy --fs-uuid --set=hearth_fs 0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90
menuentry 'rhi-prod.a' --id rhi-prod.a {
    set root=${hearth_fs}
    source /@rhi-prod.a/.hearth/grub.cfg
}
menuentry 'rhi-prod.b' --id rhi-prod.b {
    set root=${hearth_fs}
    source /@rhi-prod.b/.hearth/grub.cfg
}
```

- **A trial is taken only once its one-shot is spent.** GRUB clears
  `next_entry` in the file it loaded first, and boots the trial only if
  that save succeeded, then clears it in the other. A failed save boots
  the default instead: a trial is never sticky, so a trial that panics
  cannot loop. On btrfs every save fails, which is why GRUB lives on FAT
  and `grub-reboot` is never used.
- **The trial boots with `hearth.trial=<target>.<label>`,** appended by
  the slot's fragment (5.4), which arms its dead-man (slots.md, 5.5).
- **A boot counter falls back to the previous blessed slot,** RAUC's
  pattern: each boot of the default spends one of `tries`, and at 0 GRUB
  boots `previous` (slots.md, 8.4). GRUB's script has no arithmetic,
  hence the chain of comparisons.
- **`fallback` is the default,** so a trial whose kernel fails to load
  returns to the default within the same boot. It covers only entries
  whose loading returns, not a kernel that panics: that is `panic=10`'s
  and the counter's job.
- **The serial lines** are written only when the `boot` part names a
  serial console (`console=ttyS0,115200n8` gives unit 0 at 115200).
  `timeout=3` leaves a person at the provider's console time to pick a
  slot by hand.

### 5.4 The slot's fragment

Each slot carries its own `linux` and `initrd` lines, in
`/.hearth/grub.cfg` inside its root, written at prepare
(slots.md, 5.5) and sealed with the slot:

```
# slot rhi-prod.b, written by hearth's activator at prepare (machine.md, 5.4)
linux /@rhi-prod.b/boot/vmlinuz-6.12.48+deb13-amd64 root=UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90 rootflags=subvol=@rhi-prod.b ro panic=10 console=tty0 console=ttyS0,115200n8 ${hearth_trial}
initrd /@rhi-prod.b/boot/initrd.img-6.12.48+deb13-amd64
```

- **Inside the slot,** so the command line changes with the slot it
  belongs to, a torn write harms only the slot being prepared, and
  `grub.cfg` stays fixed.
- **`${hearth_trial}` is empty** but on a trial's boot (5.3).
- **Paths are resolved by the activator,** to the kernel's own files,
  rather than through Debian's `/vmlinuz` link, so GRUB never follows a
  link on btrfs.
- **GRUB reads paths from the default subvolume,** the top level, which
  is why `/@rhi-prod.b/` leads every path (3.1).

### 5.5 The environment pair

| Variable | Meaning | Written by |
|---|---|---|
| `default` | the default slot's entry, such as `rhi-prod.a` | the activator: bootstrap, bless, re-bless |
| `previous` | the previous blessed slot, or empty | the activator: set at bless; cleared at re-bless and before that slot is prepared again |
| `tries` | boots of the default left before `previous`: 3 to 0 | the activator: 3 at bless and at every start that meets the minimal criterion; GRUB counts it down |
| `next_entry` | the slot on a one-shot trial, or empty | the activator at trial; GRUB clears it at boot |

- **Two files hold the same variables.** The activator writes `env.0`,
  then `env.1`, each with `grub-editenv` (`hearth-protocol-grub`):

  ```
  /usr/bin/grub-editenv /run/hearth-activator/boot/hearth/env.0 set next_entry=rhi-prod.b
  /usr/bin/grub-editenv /run/hearth-activator/boot/hearth/env.1 set next_entry=rhi-prod.b
  ```

  GRUB reads `env.0`, and `env.1` only if `env.0` is not a valid block.
  A cut while one file is written leaves the other holding a state the
  activator meant.
- **Why a pair:** `grub-editenv set` truncates the file and rewrites it
  in place (only `create` writes a new file and renames it), FAT has no
  journal, and GRUB's `save_env` rewrites the whole 1024-byte block in
  place, two sectors. Any of these writes can be torn by a power cut,
  which leaves that file invalid; one file alone could leave no default
  at all.
- **The activator reads both files directly** (a 1024-byte sized format
  in `hearth-protocol-grub`) and, at each start, makes a mismatch whole:
  the other file from a valid `env.0`, or `env.0` from a valid `env.1`.
- **Writes per boot:** GRUB's count, and the activator's reset, two
  small writes to each file at every boot; a FAT partition on a VPS's
  disk bears them.

### 5.6 Default, trial and bless

The default is `default` in the valid file GRUB would read; a trial sets
`next_entry` in both files; a bless sets `previous`, `default` and
`tries` in both (slots.md, 7.3). A `next_entry` found still set on the
trial's boot means GRUB did not save, and the activator clears it at
once (slots.md, 6.3).

## 6. The kernel's command line

### 6.1 Tokens

| Token | Value | Why |
|---|---|---|
| `initrd=` | `\initrd.img.<target>.<label>` | EFI only: the stub loads it from the ESP. GRUB's fragment has its own `initrd` line |
| `root=` | `UUID=<fs uuid>` | tiny-initramfs resolves `UUID=` for btrfs; the device never appears in an output |
| `rootflags=` | `subvol=@<target>.<label>` | passed as mount data by tiny-initramfs |
| `ro` | | tiny-initramfs mounts the root read-only, as fstab does after it |
| `panic=` | `10` | Debian builds with `CONFIG_PANIC_TIMEOUT=0`, which hangs on a panic; a trial must end in a reboot |
| the `boot` part | its parameters, in their order, such as `console=ttyS0,115200n8` or `audit=0` | the target's own, from task data keyed by hardware (format.md, 5.4; render.md, 4.3); the last `console=` is `/dev/console` |
| `hearth.trial=` | `<target>.<label>` | the trial entry's line only (4.2), or GRUB's on a trial boot (5.3): it arms the dead-man before any mount (slots.md, 5.5) |

An example, for `x1-prod`'s slot `b` on EFI, whose `boot` part is
`audit=0`, with no console:

```
initrd=\initrd.img.x1-prod.b root=UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90 rootflags=subvol=@x1-prod.b ro panic=10 audit=0
```

### 6.2 Rules

- **Reserved keys:** the `boot` part may not set `root`, `rootflags`,
  `ro`, `rw`, `initrd`, `panic` or anything beginning `hearth.`, which
  are the activator's, nor `rootfstype`, `rootwait`, `rootdelay` or
  `init`, which would change how tiny-initramfs finds and starts the
  root; `rootwait` in particular would make a missing root wait for ever
  instead of panicking. Render refuses them (format.md, 5.11), and the
  activator refuses them again.
- **Tokens** are printable ASCII without spaces or quotes; a value with
  a space cannot be expressed. Parameters are compared and written as
  tokens, never as one string.
- **Length:** the whole line, the trial's token included, is at most
  2048 bytes with its terminating NUL, x86's `COMMAND_LINE_SIZE`. The
  `boot` part is refused when the line would be longer.
- **Nothing in hearth reads `/proc/cmdline`** to learn which slot runs:
  after a soft reboot it still names the slot the kernel was booted for
  (slots.md, section 3).

### 6.3 Where it is written, and when it changes

- **Written at prepare only,** into the EFI entry or the BIOS fragment
  (5.4); a trial and a bless never touch it.
- **A changed line** costs one entry write on EFI (4.3) and forbids a
  soft reboot (slots.md, 6.1).

## 7. The initramfs

### 7.1 Configuration

tiny-initramfs's configuration, `/etc/tiny-initramfs/tiny-initramfs.conf`,
is part of the `packages` part (render.md, section 4), because the
kernel's postinst hook builds the image while packages install, and the
configuration must exist before that (build.md, section 5):

```
DEBUG=no
AUTO_MODULES=no
MICROCODE=no
INCLUDE_MODULES=btrfs,virtio_pci,virtio_scsi,sd_mod
```

- **`AUTO_MODULES=no`** always: `yes` inspects what mounts `/` where the
  hook runs, which inside a build is the builder's own root.
- **`MICROCODE=no`** on a VM (placement's kind), whose hypervisor loads
  microcode; `generic`, every installed microcode package, on bare
  metal. `yes` means the build host's processor.
- **`INCLUDE_MODULES`** is the packages part's `initramfs` set, which the
  `boot` task keys by hardware profile (7.2; format.md, 5.4); `mktirfs`
  resolves their dependencies when it makes the image, and nothing is
  resolved at boot.
- **The image is in the output,** at `boot/initrd.img-<version>`,
  independent of the build host. Its bytes differ between two builds of
  the same inputs, since mktirfs's cpio keeps inode numbers and mtimes
  (reproducible.md, 2.2), so the soft-reboot rule compares its inputs,
  never its bytes (slots.md, 6.1).

### 7.2 Modules per hardware profile

| Profile | Example | `INCLUDE_MODULES` | `MICROCODE` | Consoles, in the `boot` part |
|---|---|---|---|---|
| `7th-gen` | `x1-prod` | `btrfs,nvme` | `generic` | none |
| `incus`, own disk | the VM tier | `btrfs,virtio_pci,virtio_scsi,sd_mod` (to verify) | `no` | `console=ttyS0,115200n8` |
| `serverhino` | `rhi-prod` | `btrfs,scsi_mod,sd_mod,virtio_scsi,virtio_pci,virtio_mmio` | `no` | `console=tty0 console=ttyS0,115200n8` |
| `ionos` | `ion-prod` | `btrfs,virtio_blk,virtio,virtio_pci,virtio_rng,scsi_mod`, and `vmw_pvscsi` (needed in 2022) | `no` | `console=tty0 console=ttyS0,115200n8` |
| `vultr` [later] | vul | `btrfs,virtio_blk,virtio_pci` | `no` | `console=tty0 console=ttyS0,115200n8` |

These are the bash fleet's lists, learned by trial and error; `virtio_blk`
needs `virtio_pci` to work. They move from a template keyed by hardware
into the `boot` task's settings (format.md, 5.4); the `incus` list for a
VM with its own disk is new, and `envs/lab.json` adds it.

### 7.3 What it does, and what it cannot

- **It finds the root** by `root=` as a kernel device name, `UUID=` (for
  ext2, ext3, ext4, xfs and btrfs) or `0xMAJMIN`. It does not resolve
  `PARTUUID=`, `LABEL=` or any `/dev/disk/by-*` name: it has no udev.
  os accepted those and could not boot them; hearth writes `UUID=` only.
- **It mounts with `rootflags=`, `rootfstype=` and `ro`,** reads `/etc/fstab`
  only for a separate `/usr`, and runs `init=` or `/sbin/init`.
- **It cannot do** LVM, LUKS, FUSE, fsck, overlays, idmapped mounts, a
  virtiofs or 9p root, or a multi-device btrfs (it scans no devices). A
  machine's btrfs is one device.
- **Every failure panics.** On an error it prints, sleeps 10 seconds and
  exits, and init exiting panics the kernel. A missing root device
  panics after 180 seconds, a constant compiled in; `rootdelay=` only
  adds to it, and `rootwait` waits for ever, which is why both are
  reserved (6.2). With `panic=10`, a trial whose root is missing reboots
  into the default after about three minutes.

### 7.4 After a soft reboot

A soft reboot skips the initramfs, so the activator mounts
`/run/nextroot` as tiny-initramfs would have mounted the root
(slots.md, 6.2).

### 7.5 Debian's package

- **hearth uses Debian's `tiny-initramfs` as it is:** trixie has
  0.1-6.1 and forky 0.1-8. It has been stable for years, and it boots
  the fleet today with the lists above.
- **Its maintenance:** upstream's last commit is from 2016, and Debian
  orphaned the package on 2026-08-02 (#1143475); 0.1-8 is a QA upload.
  Its stability is why it is used, not a reason to replace it. Should
  Debian drop it, it is about 2.3k lines of C, and hearth could carry
  it.

## 8. Bootstrap and disks

### 8.1 The command

`hearth bootstrap` runs on a live or rescue system, as root, with the
client running the activator's domain in-process (wire.md, 3.6). It is
the only way a machine gets its first slot. Its arguments name the disk,
the machine's configuration and a directory of secrets to import;
wire.md, 3.8 owns their syntax:

```
hearth bootstrap --disk /dev/nvme0n1 --wipe --target x1-prod
    --repository https://git.ekanayaka.io/ai/fleet.git
    --revision main --every 10m --trial manual --operator 1000
    --secrets /root/x1-prod.secrets
```

The machine's configuration, which bootstrap writes once (8.2, step 4;
wire.md, 3.8):

| Field | Standalone | Following an engine [M4] |
|---|---|---|
| mode | `standalone` | `engine`: never both (wire.md, 3.1) |
| target | the machine's target, such as `x1-prod`; its containers come from that target's spec [M5] (wire.md, 4.7) | the same |
| repository | the IaC repository's URL (wire.md, 4.1) | the same |
| desired revision | a commit, or a branch to follow with its interval, and `trial: auto` or `manual` (wire.md, 4.1) | none: the engine orders revisions (wire.md, 5.4) |
| operators | the uids that may run every socket command (wire.md, 3.4) | the same |
| mutable | `refuse`, or `accept` with `--accept-mutable`: whether the activator places mutable targets (development.md, 4.3); in its machine record | `refuse`: the run side never deploys one |
| engine | none | its address, what to trust it by, and the machine's credential if the identity chosen is one (wire.md, 5.1 and 5.3) |

The target and `mutable` go into the activator's machine record; the
rest is the steward's, kept in its state (store.md, 8.3), from which the
client changes only the desired revision and its trial mode
(wire.md, 4.1).

**Before it touches the disk,** bootstrap renders the target: it makes
the `hearth` user (8.3) and spawns `/proc/self/exe render` as that user,
in-process render with no store (wire.md, 3.5), given the repository's
read credential (`--repo-credential`) through a `0600` file it owns. Then
it refuses to start, naming every reason at once, unless:

- it runs as root, on a kernel with btrfs and unprivileged user
  namespaces (Debian's default; Ubuntu's AppArmor restriction must be
  lifted first);
- the firmware is the rendered placement's `firmware`: on EFI, efivarfs
  writable and `SecureBoot` off (9.5);
- its own binary, `/proc/self/exe`, has the digest the `hearth` task's
  download pins in the spec (format.md, 5.12): the slot it makes runs
  that release, which must read the records bootstrap writes
  (activator.md, 7.2);
- the disk is a whole block device, none of it mounted, not the live
  system's own, and blank unless `--wipe` is given (Vultr's volumes come
  with an image installed, and `x1-prod`'s holds os);
- the tools of 8.2 are present, `ssh-keygen` (`openssh-client`) among
  them, and what the builder needs: `newuidmap` and `newgidmap`
  (Debian's `uidmap`), `mmdebstrap` and `git` (store.md, section 9).

### 8.2 The steps

Each step is an effect of `hearth-domain-disk`, `hearth-domain-slots` or
`hearth-domain-boot`, keyed so that running bootstrap again after a
failure finds what the first run made, up to the disk's wipe.

1. **Wipe**, with `--wipe` only: `wipefs --all /dev/nvme0n1`.
2. **Partition:**

   ```
   EFI   parted --script /dev/nvme0n1 mklabel gpt mkpart ESP fat32 1MiB 513MiB set 1 esp on mkpart hearth btrfs 513MiB 100%
   BIOS  parted --script /dev/vda mklabel msdos mkpart primary fat16 1MiB 65MiB set 1 boot on mkpart primary btrfs 65MiB 100%
   ```

   then `partprobe /dev/nvme0n1` and `udevadm settle`, and both
   partitions must exist under `/sys/class/block/`.
3. **Format,** with identifiers hearth draws from the kernel's random
   source and passes in, so it knows them without reading them back:

   ```
   mkfs.vfat -F 32 -n ESP -i 3e5a91c4 /dev/nvme0n1p1          EFI
   mkfs.vfat -n HEARTH -i 3e5a91c4 /dev/vda1                   BIOS [M3b]
   mkfs.btrfs --force --label hearth --uuid 0f3c9a6e-… /dev/nvme0n1p2
   ```

4. **Make the top level:** mount it at `/run/hearth-activator/top`; make
   `@hearth.state` (`0:0 0700`) and `@hearth.store` (`hearth`, `0700`);
   write the machine record (target, firmware, filesystem UUID, boot
   partition's volume id, entry numbers) into the state, and the
   machine's configuration (8.1) into the steward's state, owned by
   `hearth`, with the repository's read credential under the store's
   `credentials/` (render.md, 2.2), never in the secret store.
5. **Import the secrets** from `--secrets` (activator.md, 5.8). A
   secret the spec names and the store lacks fails the place in step 7,
   naming every missing one. Identity is never imported: step 7 makes
   it.
6. **Build:** bootstrap enters a transient scope with `Delegate=yes`
   (`systemd-run --scope --property=Delegate=yes`) and lays out its
   cgroups as the daemon does (activator.md, 3.1; steward.md,
   section 7); its activator's domain spawns the steward from
   `/proc/self/exe` (the live system has no `/usr/local/bin/hearth`) over a
   link, as on a running machine (activator.md, 3.2), with the store
   mounted at `/var/lib/hearth/store`. The steward spawns a builder for
   its jobs (steward.md, section 3), which renders the target at its
   desired revision and builds its output; the steward then asks to
   place it, as converge does on any machine (wire.md, section 4);
   under an engine [M4], the revision given builds this first slot
   only.
7. **Place into slot `a`:** prepare, from snapshot to verify
   (slots.md, section 5), including the boot files and both entries
   (EFI) or the slot's fragment (BIOS). Prepare first makes every
   identity item the spec declares and the store lacks, and the
   machine-id (activator.md, 5.7).
8. **Install the boot loader,** BIOS only [M3b]:

   ```
   grub-install --target=i386-pc --boot-directory=/run/hearth-activator/boot --locales= --fonts= /dev/vda
   ```

   then write `grub.cfg` (5.3) and both environment files with
   `default=<target>.a`, and `previous`, `tries` and `next_entry`
   empty.
9. **Make slot `a` the default:** `efibootmgr --bootorder 4800,…` on
   EFI; `default` is already `a` on BIOS. There is nothing to fall back
   to, so there is no trial: bootstrap blesses `a` with the one
   transition that needs no running slot (slots.md, section 4).
10. **Report and stop:** print the machine-id and each identity item's
    public half for the repository's exports (format.md, section 8);
    stop the steward, which stops its builder; unmount everything. The
    operator reboots.

The import comes before the build and the identity is made by the
place, though the task listed both last: prepare writes them into the
slot, so they must exist when it runs.

### 8.3 The build on a live system

- **The steward and its builder run as `hearth`, unprivileged, even
  here.** A live system has no `hearth` user, so bootstrap makes one,
  with the fixed uid, gid and subordinate range that every output also
  declares (format.md, 5.12; store.md, 9.2):

  ```
  useradd --system --uid 59999 --user-group --home-dir /nonexistent --shell /usr/sbin/nologin hearth
  usermod --add-subuids 262144-327679 --add-subgids 262144-327679 hearth
  ```

  System accounts get no subordinate range by default, so it is added
  explicitly, aligned to 64 Ki; a live system's `/etc` is writable.
- **The uid must be the outputs' own:** every slot mounts the same
  `@hearth.store`, owned by it, so a uid that differed between outputs
  would lose the store at the first slot switch.
- **The build writes to the target disk,** under `@hearth.store`; its
  layers are the first slot's store when it boots.
- **No program from the output runs as root:** the tools are the live
  system's, the system running the activator (activator.md, 2.1).

### 8.4 The first boot

The firmware or GRUB boots `a`. Its activator finds `a` running and
default, no trial marker and no journal; its steward finds the desired
revision built and placed, and converges from there.

**A host's containers are not bootstrapped [M5].** Bootstrap places the
host's own target only. From its first boot, the host's hearth builds
and prepares each container's first slot as any change (wire.md, 4.7),
and tries it with nothing to fall back to (slots.md, 12.3). A
container's secrets are imported for its own target (wire.md, 3.6),
before or after: a missing secret only makes its prepare wait
(wire.md, 4.4).

### 8.5 Disks as an effect [M6]

The lab's throwaway machines need disks made on order, not by a person
at a live system: the same partitioning and formatting as an effect of
the activator (activator.md, 5.10), on a disk named by its stable id
(`/dev/disk/by-id/…`), conditional on the disk being blank, and keyed
by the identifiers the order gives. The pool host's activator makes the
disk and bootstraps the machine onto it.

## 9. Firmware quirks and Secure Boot

| Quirk | Seen on | What it breaks | hearth's answer | Reported as |
|---|---|---|---|---|
| prunes the options `BootOrder` lists whose file cannot be loaded, or that sit on a disk without a bootindex | OVMF, every boot | a blessed slot's normal entry; never a slot being written, nor a trial entry, which are outside `BootOrder` | the entries are checked, and made again, before every trial and at every start | fact: entry missing |
| reorders the options `BootOrder` lists by device order | OVMF, every boot | the default | the relative order of two entries on one disk is kept (to verify); the activator restores it at start | fact: order changed |
| ignores `BootNext` | some vendors' firmware | the trial never runs | `BootNext` still set on the next boot: cleared, the slot stays prepared | fact: one-shot ignored |
| drops entries not in `BootOrder` | some vendors' firmware | the trial's entry | made again before each trial; if it is gone by boot, as above | fact: entry missing |
| discards all variables | Vultr | every entry, and the default | open (9.4) | fact: entries missing |
| no boot manager | old OVMF without NVRAM | entries | none: such firmware is out of scope | |

### 9.1 OVMF

At every boot OVMF refreshes the options `BootOrder` lists, removes
those it cannot load, and calls `SetBootOrderFromQemu()`, which reorders
them by QEMU's device order (Incus always gives disks a bootindex).
Options outside `BootOrder` are left alone. Hence both slots on one
disk, entries outside `BootOrder` until blessed and while written, and
fixed numbers reused (4.2 to 4.4).

### 9.2 Other firmware

Nothing public describes GCP's, Azure's or Hetzner's UEFI; AWS
publishes its own as edk2 patches. hearth assumes nothing beyond the
UEFI specification and checks every boot, so a quirk shows up as a
fact, never as a slot silently left on trial.

### 9.3 Hangs and resets

The UEFI watchdog (5 minutes) is disabled at `ExitBootServices()`, so it
covers the boot loader only; after that a hang needs the machine's own
watchdog (slots.md, 6.5).

### 9.4 Firmware that discards its variables [later]

- **A stub-only machine has nothing to boot** when its variables are
  gone. The removable path, `\EFI\BOOT\BOOTX64.EFI`, is what keeps such
  a machine bootable, and a bare Debian kernel there has no command line
  (trixie builds no built-in one).
- **The lean:** GRUB for EFI on the removable path, with the BIOS
  design's fixed `grub.cfg`, slot fragments and environment pair on the
  ESP (section 5). The trial then lives on the ESP and needs no NVRAM,
  and most of M3b's work is reused. systemd-boot with boot counting is
  the alternative, and needs unified kernel images.

### 9.5 Secure Boot

- **Off,** on every kind of machine. A Debian kernel loaded directly by
  the firmware fails with `EFI_SECURITY_VIOLATION` (Debian signs it for
  its own chain, shim then GRUB), and shim cannot load it directly
  because Debian kernels lack the `.sbat` section shim requires.
- **A failed load is safe** (the firmware moves on to `BootOrder`) but
  no trial ever runs, so bootstrap refuses a machine whose `SecureBoot`
  variable is 1, and the activator reports it as a fact.
- **Incus test VMs set `security.secureboot=false`** (2.2). Bare metal is
  switched off in the firmware's setup by the person bootstrapping it.
- **Later:** enrolling hearth's own key in `db` and signing kernels or
  unified kernel images (Incus 7.4 and later take
  `initial.secureboot.db`), or shim, then a signed loader, then the
  kernel (section 12; README, section 9). Lockdown does not block
  efivarfs, so hearth's writes would keep working.

## 10. Its world

`hearth-domain-disk` and `hearth-domain-boot` run in worlds of the fake
machine (testing.md, section 3), which models for them:

- **disks:** gpt and msdos tables, partitions with types and flags, the
  post-MBR gap, FAT volumes, one btrfs filesystem and its subvolumes;
- **firmware:** a bounded variable store, a boot manager that deletes
  `BootNext` before it starts an entry, and section 9's quirks as
  switches drawn from the seed;
- **GRUB:** a model that runs the fixed `grub.cfg` against the
  environment pair, its saves torn at any sector;
- **boots:** an entry picked as the firmware and GRUB would, models of
  tiny-initramfs and systemd, and reboots, soft reboots and power cuts
  at any instant, mid-write included.

The fakes of the tools in sections 4, 5 and 8 take their argument
vectors and act on that machine (testing.md, section 4); their
conformance cases run the real tools in the VM tier
(testing.md, sections 5 and 7): efibootmgr against OVMF, GRUB against
SeaBIOS [M3b].

The invariants every world checks (testing.md, section 8), beside
slots.md, section 4's, judged by booting the firmware and GRUB models:

- **no write touches** the default's boot files, its entry, a foreign
  entry, or a file outside the boot partition and the top level;
- **NVRAM writes per rollout stay within 4.3's count,** and an entry is
  written only when its command line changed;
- **a one-shot whose save fails is never taken,** and a default that
  panics at every boot reaches `previous` on BIOS (slots.md, 8.4);
- **bootstrap ends with one blessed default slot,** or with a typed
  failure and a disk that a second run can finish.

## 11. What changes from os

- **The device leaves the output:** os rendered `root=/dev/nvme0n1p2`
  and fstab's devices from configuration; hearth writes `UUID=` at
  prepare, from the machine record, and fstab from typed data rather
  than a template completed with MiniJinja.
- **Entries at fixed numbers, written only on change:** os re-created an
  entry at every install, at a new number, with labels `a` or
  `x1-prod.a`.
- **`panic=10` always:** os set none, so a missing root hung the
  machine (trials are slots.md, section 14's).
- **BIOS works:** os's BIOS path booted a kernel nothing copied, edited
  a `grubenv` GRUB did not read, and kept it on btrfs, where no
  one-shot is possible.
- **tiny-initramfs takes no build host's microcode,** and its module
  lists are task settings instead of a template branching on hardware.
- **The top level holds only subvolumes,** flat; and the root is
  sealed, not only mounted `ro`.
- **Persistent data keeps os's classes, declared per entry.** os made
  `@T.srv` and `@T.srv.{core,data,cache}` at format time and bound
  `journal` and `iwd` from a template; hearth makes `@T.core`, `@T.data`
  and `@T.cache` when an entry uses them, binds each entry from the
  fstab it writes, and snapshots by class itself instead of snapper.
- **Firmware state is read from efivarfs,** not from efibootmgr's text.
- **Bootstrap is one command** with an unprivileged build, instead of
  `format`, `deploy install` under sudo, `prepare-rootfs` and `boot`.
- **Gone:** the `rescue` entry, the `efi-removable` and `efi:grub`
  loaders (until 9.4), `startup.nsh`, and legacy `@a`/`@b` and labels.

## 12. What is open

- **Secure Boot:** an enrolled key signing kernels or unified kernel
  images, or shim's chain; depends on which machines require it
  (README, section 9).
- **Firmware that discards its variables (Vultr):** GRUB on the
  removable path with the environment pair on the ESP, or systemd-boot
  with boot counting; depends on M3b and on vul's priority (README,
  section 9).
- **OVMF's reorder:** whether it keeps the relative order of two entries
  on one disk (to verify), which decides how often the activator
  rewrites `BootOrder`.
- **The GRUB script on 2.12:** `load_env -f` with a whitelist, the
  status `save_env -f` returns on FAT, `fallback`, `source` and an
  exported variable in a menu entry (to verify on SeaBIOS).
- **GRUB 2.14's environment block in the btrfs header** (forky, sid):
  one 512-byte sector `grub-editenv` and `save_env` can write, which
  would spare the FAT partition on machines bootstrapped later; depends
  on when the fleet's BIOS machines move past trixie's 2.12.
- **Updating GRUB after bootstrap:** never in M3b, since GRUB is shared
  by both slots and cannot be tried; depends on how often trixie's GRUB
  changes in ways that matter.
- **The `hearth` user's fixed uid and gid,** 59999 (format.md, 5.12), to
  verify against the ids the fleet's packages take.
- **Owners of persistent data:** users made by maintainer scripts get
  dynamic uids, which may differ between outputs; depends on build.md
  and format.md pinning them.
- **Deleting orphaned data:** an entry's directory, or a class with no
  entry left, by an operational effect or a person's command; depends
  on operating.md, section 6.
- **A subvolume per entry:** `"subvolume": true` on an entry that needs
  its own snapshot policy, quota or restore unit, made `@<target>.<name>`
  and mounted at its path as before classes; not in the design, a
  possible later addition if a service needs one (3.4).
- **Re-targeting a machine,** or two targets on one disk: the layout
  allows it, bootstrap and entries assume one; depends on the factory's
  need.
- **Watchdogs on bare metal:** whether `x1-prod`'s chipset has an iTCO
  (to verify on the machine).
- **The Incus test VMs:** virtio-scsi as the default disk bus, and
  `security.csm=true` for BIOS (to verify).
- **Disks as an effect [M6]:** how the lab names the disks it orders;
  depends on the pool's design.
