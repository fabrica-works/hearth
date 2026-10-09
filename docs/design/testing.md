# Testing hearth

Provisional, 2026-10-09. How hearth is tested. The strategy is
`testing-strategy.md`: the tiers, faults, what every fake shares, what
every world checks, scenarios and the referee, the two suites, and where
a failure is fixed. This document keeps what is hearth's: its tiers and
the three it adds outside the gate (section 2), the fake machine
(section 3), the fakes of its tools (section 4), conformance (section 5),
real builds (section 6), the VM tier (section 7), its worlds and their
invariants (section 8), budgets and costs (section 9), and what each
milestone adds (section 10). It applies from M1 and grows with every
milestone. What is open is in section 11.

## 1. In one page

- **Efficiency is a rule of the tests too.** Every test runs as a
  regular user, and the gate needs Linux and git, nothing else. A world
  settles in milliseconds; the focused suite takes 15 seconds and the
  fuzzy one 60. Each crate lands with its world, and each tool's
  protocol machine with its fake and its recordings.
- **The domain runs whole on a fake machine** (section 3): files with
  owners, modes, extended attributes, inode numbers and change times;
  btrfs under the kernel's unprivileged rules; user namespaces and id
  maps; processes running the tools' fakes; a Debian-shaped package
  universe; firmware variables and GRUB's env; reboots, soft reboots,
  panics and a watchdog; power cuts with a crash model; full disks.
- **Every tool has a realistic fake with one face** (section 4): an
  argument vector in; progress, output and an exit code out; effects on
  the fake machine. Output follows recordings of real runs, and the
  seed draws the faults real runs meet.
- **Conformance keeps the fakes honest** (section 5): one scenario runs
  against the real tool and its fake, and what each leaves behind is
  compared. The fake machine's own rules are held to the kernel too.
- **Real builds run as a regular user** (section 6), against a small
  pinned local mirror fetched once, through hearth's own store: the base
  layer is built once per mirror digest, and each test pays only for its
  own step.
- **What needs root runs on Incus VMs** (section 7) with their own disk,
  OVMF, Secure Boot off and an iTCO watchdog: bootstrap, trials, every
  fallback, power cuts. It stays outside the gate until the lab exists.
- **Every world checks hearth's promises** (section 8): a crash leaves
  the default booting a blessed slot; the three facts agree with the
  machine; no secret and no identity in an output; a sealed layer never
  changes; nothing unverified is placed; every order gets one answer.
- **Costs are behaviour** (section 9): worlds hold hearth's cost facts
  to what changed and to the fake machine's own count; the real tier
  measures them; a regression in efficiency is a finding.
- **The fake machine is temper's too** (3.11). It starts in hearth's
  `testing/`, its generic half knowing nothing of Debian, boot or
  hearth, and that half moves into skein when temper builds its machine.

## 2. The tiers

Each tier makes one more layer real (testing-strategy.md, section 2).
io worlds are skein's: hearth does not retest io.

| Tier | Real | Faked | Replays | Budget | From |
|---|---|---|---|---|---|
| step tests | one step function, or one protocol machine's step | its events, by hand | yes | focused | M1 |
| domain worlds | one domain: a child with the world as its parent, or a root with its children | the machine and the tools at their domain faces; the other process, scripted | yes | focused; sweeps fuzzy | M1 |
| system worlds | the builder's, the tree helper's and the activator's domains together | the machine and the tools at their domain faces; the world carries the link's and the tree helper's records | yes | focused; sweeps fuzzy | M2 |
| machine worlds | one protocol machine | the stream below; the user above; the tool's bytes: recordings, generated, mutated | yes | focused; fuzz targets | M1 |
| protocol worlds | the protocol layers, their machines and the domains above, at both ends | the tools' fakes at their io face; the bytes between two hearth processes | yes, in plaintext | focused | M1 |
| simulated worlds | every process's `iterate`: builder, tree helper, activator, client | the kernel, by skein's simulator; the machine and the tools behind it | yes, in plaintext | focused, a few; fuzzy | M1 |
| real loop | hearth as it ships, in one loop over one ring; real git | nothing: it runs only what needs no prerequisite | no | focused | M1 |
| end to end | the `hearth` binary, as processes | as the real loop | no | focused, one per path through `main` | M1 |
| conformance | each real tool as a regular user; real btrfs; OVMF and SeaBIOS in a VM | nothing: the same scenario also runs on the fake | no | `conformance` profile | M1 |
| real builds | the builder, the tree helper and the real tools on real btrfs, as a regular user | the network: a pinned local mirror | no | `real` profile | M2 |
| VM tier | everything, as root, from the firmware up | the hardware: an Incus VM | no | `vm` profile | M3 |

### 2.1 Step tests

Every step crate tests its step functions with events by hand: render's
merge of a variant that removes a key, a key's canonical encoding, a
slot's transition, the activator refusing an operation outside its
closed list. Each protocol machine's step meets one delivery, one
demand, one refusal.

### 2.2 Domain worlds

Most of hearth's tests. Each child domain has a world with the world as
its parent; each root (`hearth-domain-builder`,
`hearth-domain-activator`) has one with its children beneath it.

- **The machine and the tools sit at their domain faces** (3.1, 4.1):
  the world turns the domain's requests into the fakes' typed calls, and
  their outcomes into the domain's events, as the protocol layer would.
  The domain never sees a tool's text here or anywhere.
- **The other process is scripted:** the builder's world plays the tree
  helper and the activator; the activator's world plays the builder,
  including a builder that lies (8.3).
- **A restart is cold.** A reboot or a power cut ends every domain; the
  world builds each again from what the fake machine holds, as its
  startup reads it back (store.md, section 8; activator.md, section 7).

### 2.3 System worlds [M2]

`hearth-system-world`, under `tests/system`, is the crate of hearth's
system worlds. It begins at M2 with the builder and the tree helper; the
activator joins it at M3. Its two worlds are each joined on one fake
machine:

- **The builder and its tree helper** [M2]: the layers the helper
  snapshotted, froze and emptied are the ones the builder named; the
  change set it computed is the one the builder records.
- **The process pair and the tree helper** [M3]: the activator, the
  builder it starts, and the helper, across reboots and power cuts. The
  output the activator placed is the one the builder built; the slot the
  builder asked to bless is the one running.

The world carries the link's and the tree helper's records between the
domains, as their protocol layers would (testing-strategy.md, 2.3).

### 2.4 Machine worlds

Each protocol machine (README.md, 4.2) is fed recordings of real runs
(4.5), each with what it must decode to, output generated from the
seed, and both mutated: mmdebstrap's `I:`, `W:` and `E:` lines and
failure tails; apt's `Inst` lines and `Packages` stanzas; `apt-helper`'s
`Get:` and `Err:` lines; `dpkg-query`'s fields; efivarfs's variables as
OVMF and x1-prod's firmware write them; GRUB's environment blocks
[M3b]; strict JSON from the test repositories. Each machine is fuzzed
alone.

### 2.5 Protocol worlds

Both ends of each connection, joined by streams cut at random: the
builder and the tools' fakes at their io face (3.1) [M1]; the builder
and its tree helper [M2]; the builder and the activator over the link,
and the client and the builder [M3]; the builder and the engine's fake
over skein's channel [M4]; two builders, bytes by digest [M6]. They run
the system worlds' scenarios through bytes: the argument vectors hearth
makes, a tool's output cut mid-line, a pipe that fills, a tool killed
mid-write.

### 2.6 Simulated worlds

Each process's `iterate` on skein's simulator, the fake machine behind
its machine seam (skein's `simulator.md`, section 3). A spawn starts a
tool's fake, or one of hearth's own processes in the same world: the
activator spawning the builder, the builder its tree helper. A reboot
ends every process; the fake firmware boots a slot; its PID 1 starts the
activator, which reads its records from the fake's files.

### 2.7 The real loop and end to end

In the gate, these tiers run only what needs no prerequisite: no btrfs,
no subordinate ids, no mirror, no root.

- **The real loop:** the builder rendering with real git on a scratch
  repository, its resolutions already in the store by key so no
  mmdebstrap runs [M1]; the tree helper walking, hashing, applying and
  emptying a scratch tree, in a user namespace that maps only the test
  user's own id, which needs no subordinate range [M2]; a real program
  that never exits, stopped by signals to its process group, since a
  cgroup's kill needs a delegated subtree and is the real tier's [M2];
  the client on a Unix socket [M3].
- **End to end:** `hearth` as processes, one test per path through
  `main`: the client's render and diff in-process [M1]; the builder as a
  daemon and the client talking to it [M3]. The activator's `main`
  needs root, so the VM tier runs it.

### 2.8 Outside the gate

Three tiers are hearth's own: conformance (section 5), real builds
(section 6) and the VM tier (section 7). They do not replay. A failure
found there is rerun lower down as a scenario, where it replays, and
fixed there (testing-strategy.md, section 9).

### 2.9 Layout

```
crates/*/src/tests.rs               step tests
testing/hearth-fake-machine         the fake machine (section 3)
testing/hearth-fake-<tool>          a tool's fake: its domain and its face (section 4)
testing/hearth-fake-engine          the engine's fake [M4] (4.6)
testing/recordings/<tool>/<version> recordings of real runs (4.5)
testing/repos                       the test IaC repositories, cue tasks included (8.2)
testing/mirror                      the local mirror's pin, its extras, its fetch script (6.2)
tests/<name>                        a domain's world, hearth-<name>-world
tests/system                        hearth-system-world: the system worlds (2.3)
tests/*/tests/*.rs                  focused tests
tests/*/tests/fuzzy_*.rs            fuzzy tests
tests/conformance/tests/*.rs        conformance (section 5)
tests/real/tests/*.rs               real builds (section 6)
tests/vm/tests/*.rs                 the VM tier (section 7)
```

The world harness is skein's `skein-world`, and the counting allocator
`skein-heap` (skein's `testing.md`, section 5).

### 2.10 What changes from os

- **The machine's behaviour is tested at the gate.** os's 622 default
  tests never ran mmdebstrap, apt, dpkg, btrfs or sudo; its real tests
  sit behind cargo features and `#[ignore]`, need passwordless sudo, and
  skip themselves when a tool is missing. hearth runs all of it on the
  fake machine, and its real tiers need no sudo and refuse to start,
  rather than skip, without their prerequisites (6.4).
- **Recordings replace hand-written logs:** os's parser fixtures were
  approximations; hearth's are captured from real runs.
- **Boots are tested.** os has none: mocks of efibootmgr on `PATH`, and
  loopback images under sudo. The bash fleet's lab booted a virtiofs
  root with no firmware or bootloader. hearth's VMs boot from their own
  disk through OVMF, and SeaBIOS at M3b.
- **The dir backend and `--resolved` go:** the fake machine and its
  universe take their place. os's golden files inform render's tests but
  are not their oracle (README.md, section 7). The bash fleet's protocol
  fakes (efibootmgr, mmdebstrap, systemctl) become fakes on the fake
  machine; the three renderings of `efibootmgr -v` os parsed are no
  longer parsed, since the activator reads efivarfs (machine.md, 4.5).

## 3. The fake machine

### 3.1 Its parts and its faces

The fake machine is one set of state machines with no protocol or io
layers of its own (testing-strategy.md, 4.3). It shows a face to
whichever layer of hearth sits just above it:

| hearth is real down to | The machine's face | It answers |
|---|---|---|
| the domain | domain face | typed operations: a subvolume made, snapshotted, frozen, thawed or removed; a tree read or written; a tool run with typed arguments; a variable set; a reboot |
| the protocol layer | io face | io's records: files by name, a full stat, extended attributes, the btrfs ioctls, efivarfs, spawns with argument vectors into namespaces and cgroups, bytes on pipes, exits |
| io | behind the simulator | the operations the simulator passes on (skein's `simulator.md`, 3.1), and the machine's power |
| everything | none | the real kernel: a scratch directory, real btrfs, or a VM (sections 5 to 7) |

The io face answers the records hearth asks of skein (README.md, 8.1) and
grows as they do.

It is `testing/hearth-fake-machine`, in two halves:

```
hearth-fake-machine
├── linux     files, btrfs, ids and namespaces, processes, disks, power: no program, no package, no firmware
└── hearth    the package universe and its mirror, firmware, bootloaders, the kernel, PID 1, cues
```

- **Ordinary Rust in a step machine's shape,** as skein's minimal fake
  machine is: one call in, one answer out, held to the step crates'
  lints with a `clippy.toml` of its own. A scenario owns its sizes, tiny
  by default.
- **Its own vocabulary.** It depends on no hearth crate, and on skein's
  io and simulator only in its face behind the simulator, which
  translates.
- **It checks its client.** It refuses what Linux refuses, with Linux's
  error, and asserts what its client guarantees: a handle it issued, a
  process it started.
- **It keeps a ledger:** every operation, who made it (process, ids,
  view), what it changed and what it refused, in a bounded queue the
  referee drains; and counts a world reads at settle: bytes read and
  written per process, inodes changed, subvolumes made, programs run.

### 3.2 Files

- **Kinds:** regular files, directories, symbolic links, hard links (a
  link count), FIFOs and device nodes. A user namespace cannot make a
  device node, which is why mmdebstrap's unshare mode leaves `/dev`
  empty.
- **Metadata:** owner and group as on-disk ids (3.4); the full mode,
  setuid, setgid and sticky included; extended attributes as names and
  bytes (`security.capability`, `system.posix_acl_access`,
  `system.posix_acl_default`, `user.*`); size; inode number; ctime and
  mtime.
- **ctime** moves with every change to an inode: contents, owner, mode,
  attributes, link count, a rename. Its granularity is the scenario's:
  coarse as on trixie's 6.12, where btrfs has no fine-grained ctime yet,
  or fine.
- **Inode numbers** are per subvolume, and a subvolume's root is 256.
  After a remount, the numbers of deleted highest inodes can come back,
  as btrfs's next free id is the highest existing one plus one; a
  reused number comes with a new ctime.
- **A change counter** per inode and per subvolume: temper's file
  version (temper's `testing.md`, 4.3), and the ground truth that a
  sealed layer never changes (8.3).
- **Listings come back in an order drawn from the seed,** so nothing
  hearth computes may depend on it.
- **Contents are kept once, by handle,** and shared between snapshots,
  as btrfs shares extents: a snapshot copies metadata, never bytes.
- **Paths resolve beneath a root** as io resolves them, and access is
  checked by owner, group and other bits and by the capabilities a
  process holds over an inode in its namespace (3.4).
- **`openat2`'s flags are honoured as Linux answers them:**
  `RESOLVE_BENEATH` refuses an escape (EXDEV), `RESOLVE_NO_SYMLINKS` any
  link on the way (ELOOP), `RESOLVE_NO_XDEV` a mount crossed (EXDEV).
  The activator resolves every path of a slot so (activator.md,
  section 2), and the fake gives it the same refusals for a hostile
  tree (8.5).
- **Renames follow the kernel's rule:** a directory moved into another
  directory needs write permission on itself, to rewrite its `..`; a
  rename within one directory needs only that directory's. A frozen
  subvolume's root refuses write permission (EROFS). This is why
  `<key>.staging` and `<key>.trash` are names inside `layers/`
  (store.md, 2.3), and the fake keeps no exception that would hide it.

### 3.3 btrfs [M2]

One btrfs filesystem per disk, with its top level and the layout
machine.md, section 3 fixes. Mount options belong to the filesystem, not
to a subvolume. What a caller may do follows the kernel, as reproduced
unprivileged for this design:

| Operation | Allowed when | Otherwise |
|---|---|---|
| create a subvolume | the caller may write and search the parent directory | EACCES |
| snapshot | the caller owns the source's root: its fsuid is the owner, or it holds `CAP_FOWNER` in a user namespace that maps the owner (`inode_owner_or_capable`) | EPERM |
| set or clear read-only | the same rule, on the subvolume's root | EPERM |
| delete by ioctl | `CAP_SYS_ADMIN` in the initial namespace; or `user_subvol_rm_allowed`, write and search on its root, and rmdir allowed in the parent | EPERM; EROFS on a read-only subvolume, even with the option |
| rmdir an empty subvolume | rmdir allowed in the parent, read-only or not | ENOTEMPTY |
| `GET_SUBVOL_INFO` | anyone: id, uuid, parent uuid, read-only flag, generation | — |
| `GET_SUBVOL_ROOTREF` | anyone: the child subvolumes of a subvolume, 255 a call | — |
| `INO_LOOKUP_USER` | read and search on each component of the path | EACCES |
| list or show (tree search) | `CAP_SYS_ADMIN` | EPERM |

- **Ownership decides who acts on a layer.** A layer's root is the
  namespace root's, the range's base on disk (store.md, 2.2). So the
  builder is refused a snapshot, a freeze and a thaw (EPERM), and its
  tree helper, root of a namespace that maps the range, is allowed: it
  snapshots, freezes, thaws, empties and deletes, while the builder
  creates the empty layer, renames within `layers/`, `rmdir`s an empty
  subvolume and reads the three ioctls (store.md, 4.2, 5.5 and 7.3).
- **A snapshot keeps** every inode number, ctime and mtime, and its
  root's owner; only the device differs per subvolume. A nested
  subvolume shows in a snapshot as an empty directory with inode 2.
- **A write into a read-only subvolume** is EROFS.
- **The root subvolume's identity** is what `GET_SUBVOL_INFO` on `/`
  answers: the subvolume the machine booted (slots.md, section 3).

### 3.4 Ids, namespaces and the shift [M2]

- **Subordinate ranges** are `/etc/subuid` and `/etc/subgid` on the
  machine. `newuidmap` and `newgidmap` grant exactly the ranges listed,
  plus the caller's own id, and do nothing under `no_new_privs`, which
  disables their setuid bit. The `hearth` user's range is 65536 ids at
  262144 by default (store.md, 9.2).
- **A user namespace has an id map of one or more extents:** the tree
  helper's maps the range at 0 and the builder's own ids at 65536
  (store.md, 5.2); a sandbox's maps the range only (build.md, 8.1). A
  process sees on-disk ids through its map; an unmapped owner reads as
  65534, and a chown to an unmapped id is refused. The namespace's root
  holds capabilities only over inodes whose owner and group it maps.
- **chown clears setuid, and setgid with group execute,** even for
  root, and removes `security.capability`.
- **A capability set inside a namespace** is stored as version 3, with
  the namespace root's on-disk id as its root id. It reads as version 2
  from inside, and confers nothing outside.
- **ACL entries** are stored with on-disk ids.
- **The shift is the activator's** (slots.md, 5.2). The machine keeps
  only the kernel's rules, so a shift that forgets a setuid bit, a
  capability's root id or an ACL leaves the slot visibly wrong, and the
  world compares every prepared slot with a naive shift of its output
  that it computes on its own (8.3).

### 3.5 Processes, sandboxes and programs

- **Processes:** trees and process groups; credentials and their
  capabilities in their namespace; `no_new_privs`; user, mount, pid,
  ipc, uts and network namespaces; a mount view (a root and its binds,
  each writable or read-only); a cgroup with its memory and process
  limits; a working directory, an environment, three pipes; an exit
  status or a signal; time taken on the world's clock.
- **A sandbox is a view,** one of build.md's two (8.2): for the packages
  step, a tool root of the builder's `/usr` and `/etc`, read-only, with
  the staging layer at `/target`, the local repository at
  `/run/hearth/repo` and the inputs at `/run/hearth/input`; for every
  other step, the staging layer itself as root. A step reaches nothing
  else, and its network namespace has loopback only.
- **The fetch has no sandbox:** `apt-helper` runs as the builder in a
  cgroup of its own, in the machine's network namespace (build.md, 5.1).
  The fake mirror and the forge answer only that namespace, so a
  maintainer script or a `customize.sh` that downloads fails as it
  would offline.
- **Stopping is by signal and cgroup:** TERM to the first process, then
  to every process, then a kill of the cgroup; a step is over when its
  cgroup is empty (build.md, 8.7).
- **Programs run only where they are installed.** A spawn names a path,
  which must exist in the process's view and be executable for its ids:
  the packages step's mmdebstrap, apt and `dpkg-deb` come from the
  builder's `/usr` in the tool root; every other step's tools are the
  layer's own (build.md, 7.7); the activator's are the running slot's,
  put there by their packages (`/usr/bin/efibootmgr` from `efibootmgr`).
  A file no program claims exits 127; a missing one is ENOENT.
- **The generic half runs no program.** It reports each spawn to its
  owner, the hearth half, which hands it to a tool's fake (section 4) or,
  in a simulated world, to one of hearth's processes; as
  `skein-fake-checkout` leaves processes to its world.
- **Shell:** the escape hatches (`customize.sh`, `software.sh`) in the
  test repositories are written in temper's shell subset (temper's
  `testing.md`, 4.3), with the few file commands scenarios need.
  Maintainer scripts are not shell (3.6).

### 3.6 The package universe

Drawn from the seed, and shaped like Debian's and like the recording of
the local mirror (6.2). A fake mirror serves it at snapshot timestamps.
Its index serves resolution from M1; what installs it comes with builds
at M2.

```
package    name, version, architecture, priority, Essential, section
relations  Depends, Pre-Depends, Provides
files      path, kind, mode, owner and group by name, contents by handle; conffiles marked
scripts    typed effects, never shell: make a system user or group; set a file capability;
           run /etc/kernel/postinst.d; register an alternative; fail (a fault)
archive    Installed-Size; the .deb's size and SHA-256, as Packages lists them
```

- **Snapshots:** the mirror holds the universe at several timestamps,
  and a bump changes some versions. A timestamp between two snapshots
  answers as the earlier one, as snapshot.debian.org redirects.
- **Variants** as mmdebstrap defines them: essential, then each
  priority, then apt.
- **base-files and base-passwd:** `/usr/lib/os-release`,
  `/etc/debian_version`, and the static users and groups.
- **dpkg's database, of the real shape:** `/var/lib/dpkg/status` with
  each package's stanza (Package, Status, Priority, Section,
  Installed-Size, Maintainer, Architecture, Version, Depends, Conffiles
  with their md5sums, Description), and `info/<package>.list`,
  `.md5sums` and `.conffiles`.
- **Users made by maintainer scripts** take the next free system id, so
  their ids depend on the order of installation, as on Debian.
- **Conffiles** under `/etc`, recorded with their md5sums. One changed
  both locally and in the package is the question dpkg answers by its
  options (`--force-confold`).
- **Setuid files and capabilities,** as Debian ships them: `su` 4755,
  `unix_chkpwd` 2755, a capability set by a postinst as
  `iputils-arping`'s sets `cap_net_raw+ep`. The shift has something to
  get wrong.
- **A kernel package** installs `/boot/vmlinuz-<version>`, its config
  and `/usr/lib/modules/<version>`, and runs `/etc/kernel/postinst.d`.
  The initramfs package's hook there writes `/boot/initrd.img-<version>`
  from `/etc/tiny-initramfs/tiny-initramfs.conf`. Debian's `/vmlinuz`
  and `/initrd.img` links point at the pair; hearth reads the versioned
  files (machine.md, 4.1 and 5.4).
- **The fake initramfs** records which modules it holds, and the fake
  kernel mounts its root only if the disk's driver and btrfs are among
  them (3.7). Its bytes differ at every build while its inputs (the
  configuration, the modules, the generator) do not, so only slots.md,
  6.1's rule, which compares the inputs' digest, allows a soft reboot
  after a rebuild (8.3). With `AUTO_MODULES=yes` the hook inspects the
  running system, which in a sandbox is the wrong one: render refuses
  it and the build checks again (build.md, 5.5), and the fake keeps the
  behaviour so that both checks are tested.
- **The offenders of build.md, 9.2:** openssh-server's postinst makes
  host keys, ssl-cert's a snakeoil pair, systemd's a machine-id, and a
  trigger writes a log, so the customize hook and the scan have work.
- **Subvolumes made at build:** `systemd-container` ships
  `Q /var/lib/machines` in its tmpfiles, and its postinst applies it.
  The fake's tmpfiles makes a subvolume for a `v`, `q` or `Q` line whose
  path is absent on btrfs, and leaves an existing directory alone, as
  systemd-tmpfiles does. The setup hook pre-makes the directories
  (build.md, 5.5), so the layer holds a directory and no nested
  subvolume (8.5).
- **hearth's own tools are packages too:** mmdebstrap, apt, uidmap,
  efibootmgr, grub, git, systemd.
- **Small:** tens of packages and hundreds of files, so a fake essential
  bootstrap takes milliseconds.

### 3.7 Firmware and boot [M3]

A scenario picks the machine's kind:

| Kind | Firmware | Disk and boot | Watchdog |
|---|---|---|---|
| `efi` | UEFI as the specification says | GPT; ESP; direct stub entries (machine.md, section 4) | none |
| `efi-ovmf` | OVMF as Incus runs it | the same; prunes and reorders `BootOrder`'s options at every boot | iTCO |
| `efi-forgetful` | UEFI that loses its variables at a drawn boot | the same, and the removable path `\EFI\BOOT\BOOTX64.EFI` | none |
| `bios` [M3b] | SeaBIOS | msdos; GRUB's core in the post-MBR gap; all of GRUB on the FAT partition (machine.md, section 5) | iTCO on q35 |
| `os` | either firmware, as os left it | os's slots, ESP files, entries and `/.manifest/boot.json`, for adoption (machine.md, section 8) | as its kind |

**EFI variables:**

- **efivarfs is a filesystem of the fake machine:**
  `/sys/firmware/efi/efivars/<Name>-<GUID>`, each file 4 attribute
  bytes and then the variable, in the shapes machine.md (4.5) decodes.
  The activator reads them there; efibootmgr is its only writer.
- **A `Boot####` option** is an `EFI_LOAD_OPTION`: attributes, a
  description, a device path (the partition and the file) and optional
  data, the command line in UCS-2. Each slot has two at fixed numbers,
  its normal entry and its trial entry, whose command line carries the
  `hearth.trial=` token (machine.md, 4.2). The fake stub reads the command line
  up to its first NUL, so the arguments efibootmgr joins with NULs are
  lost, as on a real machine; x86 caps the line at 2048 bytes.
- **`BootNext` is deleted** by the boot manager before it starts the
  option it names; `BootCurrent` is set to the option booted. A load
  that fails, a missing file or Secure Boot, falls back to `BootOrder`.
- **OVMF,** at every boot and before it chooses, drops the options
  listed in `BootOrder` whose file cannot be loaded or whose disk has no
  bootindex, and reorders `BootOrder` by QEMU's device order; options
  outside `BootOrder`, such as a trial entry, it leaves alone. The other
  quirks of machine.md's section 9 (`BootNext` ignored, entries outside
  `BootOrder` dropped) are switches drawn from the seed.
- **The variable store fills.** It has a capacity (256 KiB in OVMF's
  4 MB build), and a deleted variable's space comes back only at the
  next boot, so cycles of create and delete without a reboot fill it.
- **Secure Boot** is off unless a scenario turns it on; then a Debian
  kernel loaded directly fails to load (machine.md, section 9).

**GRUB [M3b]:**

- **All of GRUB is on the FAT partition:** its modules, the fixed
  `grub.cfg` written at bootstrap, and the environment pair
  `hearth/env.0` and `hearth/env.1`, 1024-byte blocks (machine.md, 5.2
  and 5.5). Each slot carries its `linux` and `initrd` lines in its own
  fragment, `/.hearth/grub.cfg`, which GRUB sources from the slot's
  subvolume (machine.md, 5.4).
- **The fake GRUB runs that `grub.cfg`:** it loads `env.0`, or `env.1`
  if `env.0` is not a valid block; if `next_entry` is set, it becomes
  `default`, and is cleared and saved in both files before any kernel
  loads, each `save_env` writing the whole 1024-byte block, two sectors;
  `fallback` is the default, and covers only an entry whose loading
  returns. Its boot counter is the one machine.md's `grub.cfg` keeps
  (section 5).
- **A save can fail,** on a full or read-only partition: GRUB then boots
  the entry anyway, with `next_entry` still set.
- **On btrfs `save_env` fails** and GRUB boots the entry anyway, leaving
  `next_entry` set: the trap of trixie's GRUB 2.12. The fake keeps it,
  so a world shows that hearth never puts an environment there.

**The boot:**

- **The kernel** boots the image an option or a fragment names, with
  its command line. Its initramfs mounts `root=` with
  `rootflags=subvol=` only if its modules hold the disk's driver and
  btrfs, and the subvolume exists. Otherwise it prints, waits ten
  seconds and exits, and the kernel panics, as tiny-initramfs does.
- **PID 1** reads from the slot only the keys the fake models, from
  where systemd reads them: `RuntimeWatchdogSec=`,
  `hearth-trial.target`'s `JobTimeoutSec=` and `JobTimeoutAction=`, the
  units enabled, their conditions on a path or on the kernel's command
  line, `IgnoreOnIsolate=`, the default target, and fstab's mounts of
  `/var` and the persistent subvolumes. A unit's outcome is its
  package's, or its cue's (8.2). The boot reaches its target, or does
  not.
- **A mount that fails** isolates `emergency.target`, which stops every
  unit but those with `IgnoreOnIsolate=yes`, and leaves unmounted what
  the mount would have shown. So the trial's units, which arm from a
  marker in `/run` or the `hearth.trial=` token and never from a mount,
  keep running there (slots.md, section 6).
- **`is-system-running`** answers `degraded` while a unit has failed.

### 3.8 Reboots, soft reboots, panics and the watchdog [M3]

- **A reboot** stops every process (TERM, then KILL), syncs, unmounts,
  and hands over to the firmware.
- **`systemctl reboot -ff`** is `reboot(2)` at once: no unit stops.
  Whether it syncs first is to verify, so the fake does not.
- **`systemctl reboot`** turns into a soft reboot while `/run/nextroot`
  is a mount point, as systemd 257 does, unless
  `SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1` is in its environment. This is why
  the dead-man unit uses `-ff` and the activator sets the variable
  (slots.md, 6.3 and 6.5).
- **A soft reboot** keeps the kernel, its command line and `/run`, runs
  no firmware, bootloader or initramfs, and starts PID 1 in
  `/run/nextroot`; units marked to survive do. The new root needs
  `/usr/lib/modules/<running version>`, or every unit that loads a
  module fails.
- **A panic** reboots after `panic=N` seconds. Without `panic=` on the
  command line it hangs for ever, as Debian's kernel does.
- **Hangs come in three kinds:** the kernel's, which only a hardware
  watchdog ends; PID 1's, which the watchdog ends if
  `RuntimeWatchdogSec=` armed it; and a boot that never reaches its
  target, which `JobTimeoutAction=` or the dead-man unit ends. A hang
  nothing ends lasts until the referee's deadline fails the test (8.3).
- **The watchdog** resets the machine when PID 1 stops pinging it for
  its timeout. Whether it stays armed across a soft reboot is to verify,
  so the seed draws either.

### 3.9 Power cuts [M2]

A power cut falls between any two of the machine's operations, or inside
a write: the referee picks one, the seed draws one, or a sweep takes
each in turn. The model follows skein's for its fake disk (skein's
`kv.md`, section 8), and where a guarantee is still to verify it takes
the weaker one, so hearth is never written against a promise the kernel
may not keep:

| What | After a power cut |
|---|---|
| file data, synced | kept |
| a filesystem after `syncfs` | everything written to it before, kept |
| file data, not synced | any of its writes, in any order, the last torn at any byte |
| a name made, renamed or removed | kept once its directory is synced; a rename whole or not at all |
| a subvolume made or snapshotted | whole or absent; kept once the call returned (to verify) |
| a subvolume deleted | gone, or back whole (to verify when it is final) |
| the read-only flag | old or new (to verify) |
| a file on FAT: the ESP's kernels, an environment block | synced, kept; otherwise old, torn or absent; a file rewritten in place, as `grub-editenv set` truncates and rewrites one, may be found empty; a rename on FAT leaves either name, both or neither (to verify) |
| GRUB's own save | the 1024-byte block's two sectors, each old or new |
| an EFI variable | old or new, never torn (edk2's fault-tolerant writes: to verify); of several, any prefix |
| `BootNext` the firmware has read | gone |

The machine then powers on, and the firmware boots what it would (3.7).

### 3.10 A full disk, and the other faults [M2]

- **Capacities** per filesystem, tiny in worlds: btrfs data and
  metadata; the ESP; GRUB's env partition; the variable store. A
  snapshot needs metadata, and a shift unshares the metadata it touches
  (tens of MB on a real root), so a disk that held the output can fill
  while preparing a slot.
- **ENOSPC** falls at a drawn write.
- **The simulator's own disk faults** (skein's `simulator.md`, section
  4: an I/O error, a filesystem gone read-only, an operation that hangs)
  apply behind it. The machine adds its own: a variable store full,
  firmware that forgets, a disk missing at boot, a GRUB save that fails,
  a mount that fails at boot, and a published layer whose contents were
  lost (a file torn as an unsynced write would leave it).

### 3.11 Shared with temper

temper plans the same machine (temper's `testing.md`, 3.2 and 4.3):
files with versions, per-tree views, processes, and programs whose
protocol is their command line. hearth builds it first.

- **It starts in hearth,** as `testing/hearth-fake-machine` at M1.
- **Its generic half names nothing of hearth's:** no program, package,
  firmware, slot or store. It depends on nothing in the hearth half, so
  it can move unchanged.
- **No shared types** (testing-strategy.md, section 4). Its vocabulary is
  its own; hearth's worlds translate to and from it, and temper's will.
  It reuses no io record outside its face behind the simulator.
- **temper's needs are in it from the start:** the change counter is
  temper's version; mount views are temper's per-tree views. temper's
  programs stay temper's, and git's working trees stay
  `skein-fake-checkout`'s.
- **Only the generic half moves to skein,** when temper builds its
  machine (temper's `testing.md`, section 9): as a kit beside
  `skein-fake-checkout`, with its tests and hearth as its first user
  (testing-strategy.md, sections 7 and 9). The hearth half (the
  universe, firmware, bootloaders, the kernel, PID 1, the tools' fakes)
  stays hearth's, as `hearth-fake-machine`, on the kit, since a
  service's fake machine is the service's (testing-strategy.md, 4.3).
- **skein's minimal fake machine stays minimal,** as skein's
  `testing.md`, section 4 says; the kit does not grow it.

## 4. The fakes of tools

### 4.1 One face

```
in    an argument vector, an environment, a working directory;
      the process's ids, namespaces and view; bytes on stdin, where the tool reads any
out   bytes on stdout and stderr, in chunks over the world's time: progress;
      an exit status, or the signal that ended it;
      effects on the fake machine, made as that process
```

- **A domain and a face,** as temper's programs have: the domain is
  what the tool does, in the fake's own vocabulary; the face parses the
  argument vector and prints the outcome. At the domain face a world
  calls the domain directly; from the io face down, every call goes
  through the argument vector. Only the invocations hearth makes need a
  face.
- **It checks its client.** It refuses an option the real tool refuses,
  exits as the real tool does on a usage error, and works only where the
  real one would: efibootmgr writes variables only as root; mmdebstrap
  in unshare mode makes no user namespace when it runs as root, and as
  anyone else needs `newuidmap`, which `no_new_privs` disables.
- **Output follows recordings.** Every line it prints has the shape of a
  recorded real run (4.5), with its varying fields (names, versions,
  counts) filled from the fake's state. Recordings are of the tools
  Debian trixie ships, run as hearth runs them, not on a terminal.
- **Faults come from the seed,** each one that real runs meet, and a
  sweep asserts that each fell (testing-strategy.md, section 3).
- **Time:** each takes the time its work takes, on the world's clock,
  so a stall meets hearth's progress deadline.
- **A step machine,** since every tier from domain worlds up needs it,
  in `testing/hearth-fake-<tool>`, acting on the fake machine.

### 4.2 The catalogue

| Tool | Machine | Its fake | Faults | From |
|---|---|---|---|---|
| mmdebstrap | `hearth-protocol-mmdebstrap` | resolution's `--simulate --mode=chrootless`: `Inst` lines and `Packages` indices; build.md's bootstrap: the closure unpacked and configured into `/target`, chrooted, from the local repository | a stall; a maintainer script that fails; a full disk; a package missing from the repository; killed | M1 (simulate), M2 |
| `apt-helper download-file` | `hearth-protocol-dpkg` | each file named, by URI, into its target, checked by its SHA-256 | a stall; a 404; a hash or size mismatch; a reset; a full disk | M2 |
| dpkg-query | `hearth-protocol-dpkg` | build.md 5.7's four fields per package, from the layer's dpkg database | a package left half-configured by an earlier failure | M2 |
| the target's tools: `systemd-sysusers`, `systemd-tmpfiles`, `systemctl preset-all` with `SYSTEMD_OFFLINE=1`, `tar` | `hearth-protocol-builder`'s generic machine | users, files and preset links made in the layer, as its root, inside it | a tool missing from the target; a failure with a tail | M2 |
| git | `hearth-protocol-git` | render.md 2.2's `rev-parse`, `ls-tree` and `cat-file --batch` on a repository, and the fetch of wire.md, section 4 | a remote unreachable; a revision missing; a branch rewritten; a stall | M1 |
| efibootmgr | `hearth-protocol-efi` | writes only: creates at a fixed number, deletes, `--bootnext`, `--delete-bootnext`, `--bootorder`, on the variables of efivarfs | the variable store full; efivarfs not writable; a number already taken | M3 |
| systemctl, at run time | `hearth-protocol-systemd` | `soft-reboot`, `reboot`, `reboot -ff`, `is-system-running`, `show`, `stop`; `restart` [M5] | a unit that fails; a job that times out | M3 |
| ssh-keygen | `hearth-protocol-keygen` | activator.md 5.7's host key: a private and a `.pub` file, their bytes from the seed | a full disk | M3 |
| the activator's others: mount, umount, cp, and bootstrap's disk tools | `hearth-protocol-activator` | their effects on the fake machine's mounts, files and disks (activator.md, section 8) | a failure with a tail | M3 |
| wg | `hearth-protocol-activator` | a WireGuard key as identity | a failure with a tail | later: M3's identity is SSH host keys only |
| grub-editenv, grub-install | `hearth-protocol-grub` | `set` and `unset` on one environment file, truncated and rewritten in place; GRUB installed on the FAT partition | a full partition; a block that is not GRUB's | M3b |

### 4.3 mmdebstrap

- **Two invocations:** render's resolution (render.md, 5.2), with
  `--simulate --mode=chrootless`, which needs no namespace and, with
  `--skip=cleanup/apt/lists`, leaves the `Packages` indices it read
  under its scratch root; and build.md's
  bootstrap (5.4).
- **The bootstrap runs as build.md runs it** (8.3): as the root of the
  sandbox's user namespace, in the tool root, `--mode=unshare`, so it
  unshares mount, pid, uts and ipc and no user namespace, and never
  reads `/etc/subuid`. It chroots into `/target`, the staging layer, and
  runs maintainer scripts there with the target's binaries. The fake
  gives one tree for `--mode=unshare` and `--mode=root`; whether the
  real ones do is conformance's question (5.2; build.md, section 15).
- **It checks what the real one checks:** an empty target; one
  `--include` joined from several; the hooks run as the namespace's
  root, with the chroot as `$1`; the local repository mounted into the
  chroot by the `file-mirror-automount` hook.
- **It does what the real one does that hearth must undo:** it copies
  the tool root's `/etc/hostname` and `/etc/resolv.conf` into the
  target, the postinsts of 3.6 make keys and a machine-id, and its own
  cleanup removes the apt lists and logs and empties
  `/etc/machine-id`. The customize hook and the scan must catch the
  rest (build.md, 5.6 and 9.3).
- **Its output** off a terminal is `I:`, `W:` and `E:` lines; with
  `--verbose`, apt's and dpkg's lines; on failure, the tail of build.md,
  10.1.

### 4.4 The other tools

- **`apt-helper download-file`** takes `<uri> <target> SHA256:<hex>`
  triples (build.md, 5.1), from the machine's network namespace with no
  sandbox, and prints `Get:`, `Err:` and `E:` lines, exiting 0 or 100;
  on a mismatch it leaves nothing at the target. Given a `file:` URI it
  links the file instead of fetching it, as the real one does, which is
  why the builder passes only `http` and `https`.
- **git** takes its mechanics from `skein-fake-checkout` (commits, trees,
  a remote the world owns); its face is render.md's commands and the
  fetch standalone mode makes. Its remote holds the test repositories at
  their revisions.
- **The target's tools** run inside the layer, as its root, with
  `SYSTEMD_OFFLINE=1`: `systemd-sysusers` adds the part's users to the
  layer's `/etc/passwd` and `/etc/group` with the next free ids;
  `systemd-tmpfiles` makes what the part's lines say; `systemctl
  preset-all` makes the enablement links. Only their exit and tail are
  parsed (build.md, 10.4).
- **efibootmgr only writes.** The activator reads every variable from
  efivarfs (machine.md, 4.5); with `--quiet`, only the fake's exit and
  error lines matter. Its `-v` listing exists for conformance only
  (5.2). `--create-only --bootnum N` makes an entry outside
  `BootOrder`, and exits 40 if `N` exists, as the real one does, so the
  activator deletes before it creates. A slot's trial entry in
  `BootOrder` at any time, or its normal entry before its bless, fails a
  world. `--unicode` joins several positional arguments with NULs, the
  mistake machine.md (4.2) avoids.
- **systemctl at run time** acts on the fake PID 1 (3.7, 3.8).
- **ssh-keygen** writes the files and modes the real one writes, the
  key's bytes from the seed; the world records each key, so no private
  half hides in an output (8.3). It is only ever asked into a `.new`
  directory (activator.md, 5.7), and asserts it.
- **grub-editenv** keeps GRUB's 1024-byte block, byte for byte the real
  one's (5.2). `set` truncates the file and rewrites it in place, so a
  cut mid-write leaves it torn or empty (3.9).

### 4.5 Recordings

- **Where:** `testing/recordings/<tool>/<version>/`, each with its
  argument vector, its environment (no terminal), its stdout, its
  stderr and its exit.
- **One recording, two uses:** it is a machine world's transcript, with
  what it must decode to beside it (testing-strategy.md, 4.1), and the
  shape the fake's face prints.
- **Captured, never written by hand.** The conformance profile records
  them from real runs (section 5); firmware recordings come from the VM
  tier.
- **Faults are recorded too:** a 404 and a hash mismatch from the local
  mirror, a failing postinst from the extras (6.2), a full disk and a
  full variable store in a VM.
- **A new version is re-recorded.** A recording that changes is a diff
  to review, and a likely change for its parser.
- **Firmware is recorded as bytes:** efivarfs's variables after each
  step on OVMF, and, on SeaBIOS, the environment pair, are the
  transcripts of `hearth-protocol-efi` and `hearth-protocol-grub`.

### 4.6 The engine's fake [M4]

`testing/hearth-fake-engine` is the engine at the wire's far end
(wire.md, section 10), a step machine like the tools' fakes:

- **Its domain** speaks the wire's records: it accepts or refuses dials,
  checks that the hello comes first, gives orders with keys and
  attempts, resends them, delays and loses acknowledgements, drops the
  channel, and opens a second channel under the same credential.
- **Its faces:** at the domain face, the wire's domain records; in a
  protocol world, bytes through skein's channel, with
  `skein-fake-channel` beneath.
- **It is shaped for the factory's infrastructure connector,** whose
  world (jig's `connectors.md`, section 15) can put it, and the fake
  machine, behind a fake hearth.

## 5. Conformance

### 5.1 The rule

The same scenario runs against the real tool and against its fake, and
what each leaves behind is compared, as skein holds its simulator to the
kernel (testing-strategy.md, section 5).

- **A difference is a conformance bug:** the scenario first, then the
  fake.
- **A behaviour found in a real tool** that the fake lacks becomes a
  conformance scenario first, then a fake's feature.
- **Where the real tool draws** among outcomes (the order dpkg
  configures in, so the ids users get), the fake shows each over its
  seeds, and the real one's is among them.

### 5.2 Scenarios

| Scenario | The real side | What is compared |
|---|---|---|
| resolve | `mmdebstrap --simulate --mode=chrootless` on the local mirror (render.md, 5.2) | the closure: names, architectures, versions; the `Packages` stanzas it left |
| resolve, both modes | the same requests simulated with `--mode=chrootless` and with `--mode=unshare`, for every variant | the two closures, which must be equal: os and the bash fleet resolved chrootless and built unshare, and a reason once found for preferring unshare was lost (render.md, section 10) |
| fetch | `apt-helper download-file` on the local mirror, over loopback HTTP | files by digest; a 404's and a mismatch's exit, lines and leftovers |
| bootstrap | mmdebstrap as build.md (5.4, 8.3) runs it: as the root of a sandbox's user namespace, `--mode=unshare`, chrooted into a staging layer | the installed set; dpkg's status, stanza by stanza; the tree: paths, kinds, modes, owners as the namespace sees them, attributes, link targets, contents by digest |
| the same, `--mode=root` | the bootstrap again in root mode | the two trees, to settle build.md's open choice (section 15) |
| install the extras | the bootstrap with the extras repository | users made; conffiles recorded; setuid bits; version 3 capabilities and their root id; a failing postinst's status and tail |
| query | `dpkg-query` on the bootstrapped layer, in build.md 5.7's format | stdout, byte for byte |
| the target's tools | `systemd-sysusers`, `systemd-tmpfiles` and `SYSTEMD_OFFLINE=1 systemctl preset-all`, inside a bootstrapped layer | users and groups made, files made, links made |
| git | real git on scratch repositories | revisions resolved, listings, blobs |
| host keys | `ssh-keygen` as activator.md (5.7) runs it, into a scratch directory | file names, modes and formats; never the keys |
| the btrfs rules | the builder's and the tree helper's operations on the real tier's btrfs | each row of 3.3's table, as 5.4 says |
| env file [M3b] | `grub-editenv` on a scratch file | the file, byte for byte; `set` truncating it and rewriting it in place |
| firmware [M3] | efibootmgr's writes in an Incus VM on OVMF, for both of a slot's entries | efivarfs's variables after each write; exit 40 for `--create-only --bootnum N` over an existing `N`; after a reboot, `BootNext` gone and `BootCurrent`; what OVMF dropped and reordered, of `BootOrder`'s options only; `efibootmgr -v`, recorded |
| GRUB [M3b] | the fixed `grub.cfg` and the environment pair on a SeaBIOS VM | `next_entry` consumed in both files before the kernel, each save the whole block, two sectors; the boot counter; the default booted after a panic |

### 5.3 What is compared, and what is not

- **Trees are compared as namespace ids,** as tree digests are
  (store.md, 3.4), so the subordinate base on disk does not matter.
- **Left out:** times, inode numbers, the order of entries, keys' bytes,
  logs that carry a timestamp.
- **Allowed differences are data,** each with its reason, and kept
  short.

### 5.4 The fake machine against the kernel

The fake machine's own rules (3.2 to 3.4) are held to the kernel, as a
regular user on the real tier's btrfs directory (6.4): each row of 3.3's
table, the builder refused a snapshot and a freeze of a layer its range
owns and the tree helper allowed them; a frozen root that cannot be
renamed into another directory; a snapshot keeping inode numbers and
ctimes; chown clearing setuid and capabilities; a capability's root id
set in a namespace; ACL ids; a map of two extents. Each case names the
rule it checks. No running kernel can check
the crash model, as skein's `kv.md` says of its own: it rests on the
sources it cites, and the VM tier's power cuts (7.3) test hearth, not
the model.

### 5.5 When it runs

`cargo nextest run --profile conformance`, as a regular user, with the
real tier's prerequisites (6.4) and the mirror; firmware scenarios run
in the VM tier. It runs when a fake or a recording changes, when a tool
moves (a trixie point release, a mirror bump), and before a milestone
lands. With recording on, it rewrites the recordings (4.5).

## 6. Real builds [M2]

### 6.1 What they show

What only real tools on real btrfs show: mmdebstrap as the root of a
sandbox's namespace, chrooted into its layer; real maintainer scripts;
the target's systemd tools run offline inside a layer; the tree helper
snapshotting, freezing, walking and emptying layers the builder cannot
touch; layers removed with and without `user_subvol_rm_allowed`; the
real costs of each step.

### 6.2 The local mirror

- **The pin,** in `testing/mirror`: a snapshot timestamp (the canonical
  one snapshot.debian.org gives), the suite, components and
  architecture, and the requests of the test repository's targets:
  `test-min` (the essential variant and apt), and `test-boot` for the VM
  tier (a kernel, tiny-initramfs, systemd, systemd-container,
  openssh-server, incus-agent, and the tools hearth runs: mmdebstrap,
  uidmap, efibootmgr, git). The
  mirror digest is the digest of the pinned closure's index entries. A
  bump is a commit, as a snapshot bump in IaC is.
- **The extras:** a few test packages built from sources in the
  repository with `dpkg-deb`, as a regular user: a postinst that fails,
  a conffile, a user made by a postinst, a setuid file, a capability.
- **Fetched once, online,** by a script: indexes only at M1, then the
  closure's .debs through hearth's own fetch step [M2], into a cache
  directory named by the mirror digest, outside the repository, each
  file checked as any fetch is (`InRelease`, then SHA-256). A run whose
  digest is present fetches nothing, as mmdebstrap's `make_mirror.sh` is
  a no-op when nothing changed.
- **Served over HTTP on loopback,** at a fixed address, by the harness:
  the fetch takes only `http` and `https` (build.md, 5.1), and the
  address is in the test targets' sources, so in their keys. The build
  then makes its own local repository from its cache, as on any machine
  (build.md, 5.3). No test reaches beyond loopback.

### 6.3 The base layer, once per mirror digest

- **The real tier's store is a hearth store** that lives across runs in
  the test directory.
- **The base layer is keyed** by the resolution's digest (store.md,
  3.3), so it is built once per mirror digest: the first run
  builds it, later ones find it. An essential bootstrap from a local
  mirror takes about ten seconds; mmdebstrap's README quotes 9.5 on a
  tmpfs.
- **A setup step** builds or finds the shared layers before the tests
  start (nextest's setup scripts), pins them (store.md, section 7), and
  so tests in parallel never race for them. Each test's own layers are
  collected after it.
- **A second run builds nothing,** and its cost facts say so (9.3): the
  digest rule, tested on hearth itself.

### 6.4 Prerequisites

- **The builder's own** (store.md, section 9): Linux with unprivileged
  user namespaces, Debian's default; uidmap's `newuidmap` and
  `newgidmap`; cgroup v2 with a subtree delegated to the test user; and
  skein's contained process trees, which sandboxes wait for (build.md,
  8.8).
- **On a developer's machine** the profile runs under `systemd-run --user
  --scope -p Delegate=yes`, which delegates that subtree for the
  builder's cgroups; a runner's service needs `Delegate=yes` as well.
- **One subordinate uid range and one gid range for the test user,** as
  store.md (9.2) demands of the `hearth` user: 65536 ids, one entry
  each, overlapping no other. A builder alone accepts an unaligned base,
  since only the activator's shift needs alignment, so the range
  `useradd` gives at 100000 serves as it is.
- **A btrfs filesystem** mounted `nosuid,nodev,noatime` and
  `user_subvol_rm_allowed`, made once by an administrator, in which the
  test user makes the store as a subvolume of its own (store.md, 9.1).
  The option covers the whole filesystem, so a dedicated one (a
  partition, or a loop file) is better than `/home`.
- **The mirror,** fetched (6.2).
- **No prerequisite, no run:** the profile refuses to start, naming what
  is missing. It never passes by skipping, as os's gated tests did.

### 6.5 Each test pays only for its own step

- **A config change:** a snapshot of the shared software layer, then the
  config step: a second or less.
- **A software change:** a snapshot of the packages layer, then the
  software step.
- **The tree helper's walk** of the layer just built against its
  parent's record: only changed files are hashed.
- **Removing layers:** the tree helper's `delete` where the mount
  option is set, and thaw, empty and `rmdir` where it is not (store.md,
  7.3 and 7.4): both paths run.
- **A packages layer of its own** only where a test is about packages:
  the extras on the base; and `test-boot`'s, whose `/var/lib/machines`
  must be a directory, not a nested subvolume (8.5).

Every test asserts its own cost facts: which steps ran, which layers
were reused, what was fetched.

## 7. The VM tier [M3]

### 7.1 The VMs

- **Incus VMs** (q35), each with its own disk: a block volume holding a
  partition table, an ESP and a btrfs filesystem (machine.md,
  section 3). Not the old lab's virtiofs boot: firmware, ESP and disk
  are what the MVP's machine has.
- **OVMF, with `security.secureboot=false`.** Incus turns Secure Boot
  on by default, and a Debian kernel cannot boot from a stub entry under
  it (machine.md, section 9).
- **The iTCO watchdog** q35 provides; QEMU's default action for it is a
  reset (slots.md, 6.5).
- **A serial console,** logged by the harness.
- **BIOS [M3b]:** `security.csm=true`, which runs SeaBIOS and requires
  Secure Boot off, on an msdos disk laid out as machine.md (5.1) says.
- **Containers [M5] run inside a VM,** which is their host target: its
  hearth runs the container targets with systemd-nspawn, as `carbon`,
  `rhi` and `vul` do (slots.md, section 12). A VM needs no nesting
  settings for that, unlike an Incus container, and the real tier
  cannot run them: nspawn needs root.

### 7.2 Driving them

- **The harness runs as a regular user** who may use Incus. Root exists
  only inside the VMs.
- **It serves the mirror and the hearth binary under test** on the Incus
  bridge. The test repository's hearth task installs that binary as a
  software download pinned by digest (format.md, section 10), which the
  harness writes into the revision it commits for the run.
- **It drives a VM** with `incus exec` (the test targets install
  `incus-agent`), runs hearth's client there as an operator would, and
  reads the serial log for what comes before userspace: the firmware, a
  panic.
- **It checks hearth's facts** against what it reads independently:
  efivarfs's variables, the environment pair [M3b], the root subvolume,
  the slot records; and records `efibootmgr -v` for conformance (5.2).
- **A power cut** is `incus stop --force`, which kills QEMU.

### 7.3 Scenarios

| Scenario | What happens | What must hold |
|---|---|---|
| bootstrap | from a live system on a second disk, the client, as root, bootstraps the empty disk (machine.md, section 8); the VM boots it | slot a is blessed and the default; two entries per slot, normal and trial; only a's normal entry in `BootOrder` |
| adopting os | a VM that os installed (its `provision` and `deploy`, run once and snapshotted); `hearth bootstrap --adopt-os` (machine.md, section 8), then hearth's first trial, made to panic by `test-panic` | os's running slot stays the default throughout; the failed first trial falls back to os |
| prepare, soft reboot, bless | a config change: b prepared, a soft reboot into it, the minimal bless (slots.md, section 7) | b running, its normal entry first in `BootOrder`; no trial left |
| a packages change that keeps the kernel | a new version of an extras package: b's packages layer rebuilt, its initramfs rebuilt with the same inputs | slots.md 6.1's rule holds, so a soft reboot, then the bless |
| a kernel change | a mirror bump that changes the kernel: b prepared, `BootNext` to b's trial entry, a reboot, the bless | before the bless, `BootNext` gone and `BootCurrent` b's trial entry; then b the default |
| fallback after a panic | the revision adds the `test-panic` task (8.2) | the trial panics; `panic=N` reboots into a, which records that b fell back |
| fallback after a hang | `test-hang` blocks the trial's target | `JobTimeoutAction=` or the dead-man unit reboots with `-ff`; fell back |
| a mount fails during a trial | `test-mount-fails` adds a mount that fails, and the trial lands in emergency mode | the dead-man, armed from `/run` or `hearth.trial=` and kept by `IgnoreOnIsolate=yes`, still reboots with `-ff`; fell back |
| a failed unit | `test-unit-fails` fails a unit | the boot is `degraded`, and the slot's builder asks to fall back at once (slots.md, section 8), with no dead-man; fell back |
| a failed unit, allowed | the same unit, named in `bless.ignore_failed` (slots.md, 7.1) | the minimal bless passes; b blessed |
| a power cut during prepare | a kill at a drawn moment of prepare | a boots; the half-prepared slot is redone or discarded (slots.md, section 11) |
| a power cut in trial and bless | kills just after `BootNext` is set, and between the steps of the bless: journal, `blessed`, marker removed, default moved, dead-man stopped | the default boots a blessed slot |
| BIOS [M3b] | the bootstrap, a one-shot trial, a fallback after a panic, and a cut while the activator writes the environment pair, on SeaBIOS | `next_entry` consumed in both files before the kernel; the default boots a blessed slot |
| BIOS, a blessed default that panics [M3b] | after a bless, the harness arms `test-panic` through a flag in a persistent subvolume, so the default panics at every boot | GRUB's boot counter answers (machine.md, section 5), and the machine reaches the other blessed slot without a person |

Containers [M5], in a host VM:

| Scenario | What happens | What must hold |
|---|---|---|
| a container's slot | a container target placed on the VM's host target: its slot prepared on the host, then its nspawn unit restarted on it (slots.md, 12.1 and 12.2) | the unit stably active, the slot blessed as the container's default in the host's state |
| a container that fails | `test-unit-fails` inside the container | the trial fails at the first `failed`, and the host's activator restarts the container on its old slot (slots.md, 12.3) |
| a host reboot during a container's trial | `incus stop --force` mid-trial | the links in `/run` are gone; the host's activator restarts the container on its default |
| a container's checks | a check of a unit and a probe, run by the host's hearth (operating.md, 2.9) | reported as the host's facts, naming the container's target |
| a pool host | one output, several containers (slots.md, 12.4) | one build and one shift; a snapshot per container; each container's files owned as nspawn maps them |
| a host and its container in one revision | both specs change | the container's trial first, the host's last, since the host's may reboot the machine (wire.md, 4.7) |

A hang that only the watchdog ends needs a kernel that stops; how to
cause one in a VM is open (section 11).

### 7.4 Paying once

- **A VM is bootstrapped once** per mirror digest and hearth binary,
  then snapshotted; each scenario starts from a copy. Whether a copy
  carries the VM's variables (`qemu.nvram`) is to verify; if not, a
  stub-only disk has nothing to boot (machine.md, 9.4), and each
  scenario pays for a bootstrap of its own.
- **The builder's store inside the VM** already holds the base layers,
  so a scenario rebuilds only what its change touched.

### 7.5 Where it runs

- **On a host with KVM and Incus:** the development machine, or a
  runner labelled for it. `cargo nextest run --profile vm`, a VM per
  test, a few at a time.
- **Never at the gate.** It runs by choice, and before a hearth release
  reaches a real machine: x1-prod, the MVP's.
- **Until the lab exists.** The lab's throwaway machines then run the
  same scenarios (temper's `factory.md`).

## 8. Worlds and their invariants

### 8.1 The worlds

| World | Real | From |
|---|---|---|
| `hearth-render-world` | render, on fake git and the universe's index | M1 |
| `hearth-builder-world` | the builder's root and its children; tree helper and activator scripted | M1 |
| `hearth-store-world` | the store, on fake btrfs | M2 |
| `hearth-build-world` | plans and steps, on the fakes of `apt-helper`, mmdebstrap, dpkg-query and the target's tools, in sandboxes | M2 |
| `hearth-tree-world` | the tree helper's domain, on files with namespace ids | M2 |
| `hearth-system-world` | the system worlds (2.3) | M2 |
| `hearth-converge-world` | the desired revision and the orders | M3 |
| `hearth-slots-world`, `hearth-boot-world`, `hearth-disk-world` | the activator's children, on firmware, btrfs and disks | M3 |
| `hearth-activator-world` | the activator's root and its children; the builder scripted | M3 |
| `hearth-checks-world` | checks, on units and signals | M5 |
| `hearth-artifacts-world`, `hearth-backups-world` | the second recipe; data snapshots and streams | M6 |

Each document's own world section names its scenarios: render.md,
section 9; store.md, section 11; build.md, section 13; machine.md,
section 10; slots.md, section 13; activator.md, section 9; wire.md,
section 10; operating.md, section 9; artifacts.md, section 8.

### 8.2 Scenarios, cues and the referee

- **A scenario is set up in the fakes' terms:** the test repository at
  its revisions on git's remote; the universe and its mirror's
  timestamps; the machine's kind and disks; the secret store and the
  identities; the faults; the power cuts.
- **Boots follow content.** The test repositories have tasks that
  install real units: `test-panic` crashes the kernel through
  `/proc/sysrq-trigger`, at once or when a flag file exists in a
  persistent subvolume; `test-hang` blocks the boot's target;
  `test-mount-fails` adds a mount that cannot succeed; `test-unit-fails`
  fails. The VM tier meets the real effect; the fake PID 1 knows these
  units by name and plays the same one. A scenario is then one
  repository at every tier, as temper's CI follows content.
- **The referee** (testing-strategy.md, section 7) watches the machine's
  ledger and hearth's facts, events and answers, never hearth's state.
  Its stimuli: a power cut, a reboot, the builder killed, the client
  gone, the engine's channel dropping [M4]. Its deadlines are simulated
  time, so a trial's window costs nothing.

### 8.3 hearth's invariants

- **Every crash leaves the default booting a blessed slot.** After every
  power cut, reboot, panic and watchdog reset, the world asks the fake
  firmware what it would boot (`BootOrder`'s first option that loads,
  after OVMF's pruning; on BIOS, `default` of the environment file GRUB
  would read), and checks that it names a
  slot whose record says blessed and whose tree equals that record.
  This holds from the first bless; what a cut during bootstrap leaves is
  machine.md's (section 8).
- **The three facts stay consistent** (slots.md, section 3). At every
  settle, and with every fact hearth reports, the world compares them
  with the machine's ground truth: running is the subvolume the fake
  booted; default is what its firmware would boot; trial is the
  activator's marker.
- **No secret and no identity in an output.** Secrets, machine-ids and
  keys in worlds are drawn from the seed, and the world knows each. It
  searches every layer, record, fact, event, trace and build step's
  argument vector for any of them; only slots and the secret store may
  hold them. And two machines with different identities and secrets
  build one spec, with one tool seed, to equal tree digests.
- **A sealed layer never changes.** Once the tree helper freezes a
  layer and the builder publishes it (store.md, 4.2), the machine's
  change counter for it stays put until it is retired as `<key>.trash`
  (store.md, 7.3), and its tree digest equals its record.
- **The activator never places an output it has not verified.** The
  world injects a builder that lies: a layer changed after its freeze,
  changed between the activator's check and its snapshot, a record that
  does not match, an output of a hostile shape (8.5). No slot goes on trial whose tree is not the verified
  output shifted (activator.md, section 6), which the world checks
  against its own naive shift (3.4).
- **Every order gets exactly one answer:** from the client [M3] and from
  the engine [M4], across restarts of either side, with keyed orders
  answered once by key (wire.md, 5.4). The same holds for each
  operation the builder asks of its activator over the link.
- **Trials always end.** When a trial starts, the referee arms a
  deadline: the bless, or the fallback, by the trial's bound. A command
  line without `panic=` meets the fake's endless panic and fails here.
- **A soft reboot exactly when the kernel is unchanged.** The world
  computes slots.md 6.1's rule on its own, from the fake trees: the
  digests of the kernel image, of the modules subtree and of the
  initramfs's inputs (never its bytes), and the command line without
  the slot's tokens. Each trial's kind must follow it. The named
  scenario is a snapshot bump that keeps the kernel: the packages layer
  is rebuilt, the initramfs's bytes differ, and the trial is a soft
  reboot; a bump that changes a module takes a full one.
- **Build steps stay in their sandbox.** A step reaches only its view,
  and no network (3.5).

### 8.4 The strategy's checks

- **Contracts** as the world goes; **settled invariants:** no live
  entity, nothing in flight, every child gone and its pipes read to the
  end.
- **Memory:** `skein-heap` checks each domain against its worst case
  (programming-model.md, 6.3), in a test binary of its own.
- **Replay:** a seed replays to the same trace, the same fake machine,
  whose state the fake digests itself, and the same digest of hearth's
  own state: a digest over the state types' derived `Hash`, which
  hearth asks of skein (README.md, 8.1; skein's `lib.md` plans it).
  Until skein has it, replay compares the trace and the fake's digest.
- **Transition coverage:** `cargo llvm-cov` over both suites lists the
  handlers never reached; each is a missing scenario.
- **Fuzzing:** each machine and decoder alone; step functions fed
  recorded events.
- **Functions against naive ones:** the shift against a naive shift;
  resolution against a naive closure; the tree digest against a naive
  walk.

### 8.5 Named scenarios

Each is a focused test in the world named, and the referee's checks of
8.3 hold in every one; where the VM tier can cause the same, it runs the
same repository (7.3).

| Scenario | World | What happens | What must hold |
|---|---|---|---|
| a mount fails during a trial | slots, system | `test-mount-fails`: the trial's boot isolates `emergency.target` | the trial's units, armed from the `/run` marker or `hearth.trial=` and kept by `IgnoreOnIsolate=yes`, still end the trial; fell back |
| a failed unit | slots, system | `test-unit-fails`: the trial boots `degraded` | the slot's builder asks to fall back at once; the dead-man plays no part. With the unit on `bless.ignore_failed`, the bless passes |
| a cut in the bless | activator, system | a crash of the activator, and a power cut, between each step of the bless (slots.md, section 7): journal, `blessed`, marker removed, default moved, dead-man stopped | a power cut boots the default, a blessed slot; after a crash the running slot is blessed but not the default, and the builder blesses it again, which finishes the order |
| both GRUB saves fail [M3b] | boot, system | GRUB cannot save `env.0` nor `env.1`, so `next_entry` stays set | the activator in the trial's slot clears it and reports it (machine.md, 5.6); a trial that panics meanwhile is bounded by the boot counter; the default stays a blessed slot |
| a blessed default that panics [M3b] | boot, system | the default panics at every boot | the boot counter moves GRUB to the other blessed slot (machine.md, section 5) |
| hostile tree shapes | activator, slots | an output whose `/var`, `/etc` or kernel is a symbolic link, or with a FIFO where `/etc/passwd` should be | prepare refuses each, since every slot path is resolved with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS` and `RESOLVE_NO_XDEV` (3.2); nothing outside the slot is read or written |
| a cut between walk and record | store, system | a power cut after a layer's walk and before its record; then, later, a published layer found corrupt (3.10) | `syncfs` precedes the record (store.md, 4.2), so a recorded layer is whole; the startup scan deletes the staging layer (store.md, 4.4); a corrupt layer found later is deleted and rebuilt, not refused |
| `systemd-container` | build | the closure holds `systemd-container`, whose tmpfiles make `Q /var/lib/machines` | with the setup hook's directories, the layer holds a directory; without them, a nested subvolume, which the walk refuses (store.md, 4.1) |
| adopting os | disk, boot, system | an `os` machine (3.7) adopted with `hearth bootstrap --adopt-os` (machine.md, section 8); hearth's first trial fails | os's running slot stays the default until hearth's first bless; the failed trial falls back to os, and os's slots and entries are untouched |
| a snapshot bump that keeps the kernel | system | 8.3's soft-reboot rule | a soft reboot, then the bless |

## 9. Budgets and costs

### 9.1 The two suites

The focused suite takes at most **15 seconds** and the fuzzy suite
**60 seconds**, wall time on the development machine, idle, as skein's,
smith's and temper's are measured; nextest's global timeouts enforce
them, and raising one is an explicit decision (testing-strategy.md,
section 8). temper's gate recorded 1,092 focused tests in 5.2 seconds;
the budgets hold when the design is cheap from the first step:

- **The fakes are small:** a universe of tens of packages, trees of
  hundreds of files, snapshots that copy no bytes, a fake bootstrap in
  milliseconds.
- **Time is simulated:** a trial's window, a stall's deadline, a panic's
  ten seconds cost nothing.
- **Real processes are few:** one test per path through `main`, and
  real git on tiny repositories.
- **The fuzzy suite sweeps:** random worlds settled under every
  invariant, and a small prepare, trial and bless cut at every one of
  its operations in turn, as skein's kv world crashes at every
  operation.

### 9.2 Profiles outside the gate

| Profile | Runs | Needs | Ceiling |
|---|---|---|---|
| `conformance` | section 5 | 6.4 and the mirror | 2 minutes |
| `real` | section 6 | 6.4 and the mirror | 5 minutes |
| `vm` | section 7 | KVM and Incus | 30 minutes |
| `measure` | both suites with `-j 1`, timing each test | — | none |

Each is selected by its test binaries' names and kept out of the
default and fuzzy profiles, as `live` is in smith. The ceilings are
provisional, set from the first measured runs, and enforced as the
gate's budgets are: raising one is a decision. A cold run adds the base
layers once per mirror digest (6.3, 7.4).

### 9.3 Costs as facts

hearth reports its costs as facts: time per step, bytes fetched, cache
hits (README.md, section 5). Worlds check them two ways:

- **Against what changed.** A config change runs the config step and no
  other; a second build of one key runs nothing; an unchanged resolution
  key runs no `--simulate`; the walk hashes only files whose path, inode
  and ctime differ from the parent's record; a second fetch of one .deb
  is a hit.
- **Against the machine's own count** (3.1): bytes read and written,
  programs run, subvolumes made. A cost fact that disagrees with the
  ledger fails, so hearth's costs can be trusted.
- **Ceilings on counts,** as temper holds its engine's requests to the
  forge: programs run per build, bytes read per walk.

The real tier keeps each test's cost facts beside the mirror digest. A
step slower than its last run on the same digest by more than a margin
is reported as a finding, with both measurements: timing on a shared
machine is too noisy to fail on.

### 9.4 The lab

Once the lab exists, it benchmarks hearth as it does any product
(temper's `factory.md`): real targets built and activated, their cost
facts compared run over run. A regression in hearth's efficiency is a
finding, as a failing check is.

## 10. By milestone

| | Tiers added | The fake machine grows | Fakes of tools | Outside the gate |
|---|---|---|---|---|
| M1 | step; domain (render, builder); machine (config, template, git, mmdebstrap); protocol (builder and tools); simulated (builder); real loop (render, real git); end to end (render, diff) | files with owners and modes; processes; the universe's index at snapshot timestamps | git; mmdebstrap `--simulate` | conformance: resolve, git; the mirror's indexes |
| M2 | domain (store, build, tree); system (builder and tree helper); machine (dpkg, tree); protocol (tree helper); real loop (tree helper, process groups) | btrfs and its ownership rule; ids, maps of several extents, namespaces, cgroups, sandboxes; `openat2`'s flags; attributes, inodes, ctimes; the full universe, `systemd-container` included; full disks; power cuts on files and subvolumes, and `syncfs` | mmdebstrap's bootstrap; `apt-helper`; dpkg-query; the target's systemd tools and `tar` | conformance: bootstrap in both modes, extras, fetch, query, the target's tools, the btrfs rules; real builds |
| M3 | domain (converge, slots, boot, disk, activator); system (the pair joins); machine (link, efi, systemd, keygen, client); protocol (link, client); simulated (every process, across reboots); real loop (the client's socket); end to end (daemon and client) | GPT and the ESP; efivarfs, two entries per slot, and OVMF; the kernel, the initramfs, PID 1 with its isolation to emergency mode; reboots, soft reboots, panics, the watchdog; power cuts on FAT and variables; cues; an os machine to adopt | efibootmgr; systemctl at run time; ssh-keygen, for SSH host keys only; mount, cp and bootstrap's disk tools | VM tier on OVMF; conformance: firmware, host keys |
| M3b | machine (grub) | SeaBIOS; msdos and the post-MBR gap; GRUB on FAT, its environment pair, its boot counter, saves that fail, and the slots' fragments | grub-editenv; grub-install | VM tier on SeaBIOS; conformance: env file, GRUB |
| M4 | protocol (the wire, on `skein-fake-channel`); machine (wire) | — | `hearth-fake-engine` (4.6) | — |
| M5 | domain (checks); containers and pool hosts | units' states and signals; containers' roots on a host | systemctl restarting a unit | VM tier: bless with checks |
| M6 | domain (artifacts, backups); protocol (bytes by digest, between builders) | send streams and a backup peer; faults as effects; virtual disks | — | real builds: artifacts |
| later | identity beyond SSH host keys | — | wg, for WireGuard keys | — |

M1 sets the pattern: every tier up to end to end exists for render
before M2 begins. M2 is the largest step for the fakes, since a build
touches everything; M3, the MVP, adds the machine's lower half and the
VM tier its one real machine relies on.

## 11. What is open

- **When the generic half moves, and its name in skein** (3.11): it
  depends on when temper builds its machine.
- **The local mirror:** whether `test-min` and `test-boot` are the right
  requests, how large the kernel makes the mirror, and whether a runner
  keeps the fetched mirror between runs. It depends on the first
  measurements and on the runner's disk.
- **How much shell the machine interprets:** temper's subset, plus which
  file commands. It depends on temper's own open question, and on what
  the test repositories' escape hatches need.
- **The crash model's open guarantees:** whether btrfs's subvolume
  create, delete and read-only flag are final once the call returns;
  whether a rename on FAT is atomic; whether edk2's variable writes
  never tear. Until verified, the fake takes the weaker answer (3.9).
- **The watchdog:** whether it stays armed across a soft reboot; and how
  to cause, in a VM, a hang only the watchdog ends (a kernel test
  module such as `test_lockup`, if Debian builds it). It depends on a
  VM experiment.
- **Incus as a regular user:** whether a restricted per-user project
  allows `security.secureboot=false`, `security.csm=true` and the
  snapshots 7.4 relies on, and whether a copy carries `qemu.nvram`. It
  depends on Incus's configuration on the runner.
- **Firmware that forgets its variables:** how the VM tier causes it,
  since removing `qemu.nvram` needs the Incus host. It depends on
  machine.md's answer for such firmware.
- **`systemctl reboot -ff` and sync:** to verify before the fake
  settles (3.8).
- **The version matrix:** trixie's tools only, or forky's too (GRUB
  2.14's env block, systemd 262). It depends on which suites hearth's
  targets run.
- **The ceilings outside the gate** (9.2): provisional until measured.
- **Differential checks:** whether a scenario must end the same at every
  tier it runs at, as temper also asks. It depends on how much the
  referee can observe in the real tiers.
