# Testing hearth

Provisional, 2026-10-09, revised 2026-10-16. How hearth is tested. The
strategy is `testing-strategy.md`: the tiers, faults, what every fake
shares, what every world checks, scenarios and the referee, the two
suites, and where a failure is fixed. This document keeps what is
hearth's: its tiers and the three it adds outside the gate (section 2),
the fake machine (section 3), the fakes of its tools (section 4),
conformance (section 5), real builds (section 6), the VM tier (section
7), its worlds and their invariants (section 8), budgets and costs
(section 9), and what each milestone adds (section 10). It applies from
M1 and grows with every milestone. What is open is in section 11.

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
  fallback, power cuts, and live changes with real overlays and real
  services. It stays outside the gate until the lab exists.
- **Every world checks hearth's promises** (section 8): a crash leaves
  the default booting a blessed slot; the four facts agree with the
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
| system worlds | the builder's, the tree helper's, the steward's and the activator's domains together | the machine and the tools at their domain faces; the world carries the link's, the jobs' and the tree helper's records | yes | focused; sweeps fuzzy | M2 |
| machine worlds | one protocol machine | the stream below; the user above; the tool's bytes: recordings, generated, mutated | yes | focused; fuzz targets | M1 |
| protocol worlds | the protocol layers, their machines and the domains above, at both ends | the tools' fakes at their io face; the bytes between two hearth processes | yes, in plaintext | focused | M1 |
| simulated worlds | every process's `iterate`: steward, builder, tree helper, activator, client | the kernel, by skein's simulator; the machine and the tools behind it | yes, in plaintext | focused, a few; fuzzy | M1 |
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
its parent; each root (`hearth-domain-builder`, `hearth-domain-steward`,
`hearth-domain-activator`) has one with its children beneath it.

- **The machine and the tools sit at their domain faces** (3.1, 4.1):
  the world turns the domain's requests into the fakes' typed calls, and
  their outcomes into the domain's events, as the protocol layer would.
  The domain never sees a tool's text here or anywhere.
- **The other process is scripted:** the builder's world plays the tree
  helper and the steward; the steward's world plays the builder and the
  activator, including builders that die, hang or answer for a stale
  generation (steward.md, section 8); the activator's world plays the
  steward, including one whose store lies (8.3).
- **A restart is cold.** A reboot or a power cut ends every domain; the
  world builds each again from what the fake machine holds, as its
  startup reads it back (store.md, section 8; activator.md, section 7).

### 2.3 System worlds [M2]

`hearth-system-world`, under `tests/system`, is the crate of hearth's
system worlds. It begins at M2 with the builder and the tree helper; the
steward and the activator join it at M3. Its two worlds are each joined
on one fake machine:

- **The builder and its tree helper** [M2]: the layers the helper
  snapshotted, froze and emptied are the ones the builder named; the
  change set it computed is the one the builder records.
- **The daemon and the tree helper** [M3]: the activator, the steward
  it starts, the builders the steward spawns, and the helper, across
  reboots, power cuts and builders lost at any job. The output the
  activator placed is the one a builder built; the slot the steward
  asked to bless is the one running, after the bless's render
  (steward.md, section 6).

The world carries the link's, the jobs' and the tree helper's records
between the domains, as their protocol layers would
(testing-strategy.md, 2.3).

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
and its tree helper [M2]; the steward and the activator over the link,
the steward and its builder over the jobs, and the client and the
steward [M3]; the steward and the engine's fake over skein's channel
[M4]; two machines, bytes by digest [M6]. They run
the system worlds' scenarios through bytes: the argument vectors hearth
makes, a tool's output cut mid-line, a pipe that fills, a tool killed
mid-write.

### 2.6 Simulated worlds

Each process's `iterate` on skein's simulator, the fake machine behind
its machine seam (skein's `simulator.md`, section 3). A spawn starts a
tool's fake, or one of hearth's own processes in the same world: the
activator spawning the steward, the steward its builder, the builder
its tree helper. A reboot
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
  `main`: the client's render and diff in-process [M1]; the steward as a
  daemon, the builders it spawns, and the client talking to it [M3]. The
  activator's `main` needs root, so the VM tier runs it.

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
testing/hearth-client-fake-<tool>   a fake of a tool only the client runs: incus [M3b]
testing/recordings/<tool>/<version> recordings of real runs (4.5)
testing/repos                       the test IaC repositories, cue tasks included (8.2)
testing/mirror                      the local mirror's pin, its extras, its fetch script (6.2)
tests/<name>                        a domain's world, hearth-<name>-world
tests/system                        hearth-system-world: the system worlds (2.3)
tests/client, tests/dev             the client's worlds, hearth-client-world and hearth-client-dev-world [M3, M3b]
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
  view; one process until the steward and its builders run apart [M3],
  since skein's calls name no process), what it changed and what it
  refused, in a bounded queue the
  referee drains; and counts a world reads at settle: bytes read and
  written per process, inodes changed, subvolumes made, programs run.
  The steward's rule is read from it (steward.md, 2.2): no open beneath
  the store's `layers/` or `repo/`, and no build tool spawned, by the
  steward's process.

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
  The activator resolves every path of a slot so
  (activator.md, section 2), and the fake gives it the same refusals for
  a hostile tree (8.5).
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
relations  Depends, Pre-Depends, Provides, Conflicts, with their version relations
files      path, kind, mode, owner and group by name, contents by handle; conffiles marked
scripts    typed effects, never shell: make a system user or group; set a file capability;
           run /etc/kernel/postinst.d; register an alternative; fail (a fault)
archive    Installed-Size; the .deb's size and SHA-256, as Packages lists them
```

- **Snapshots:** the mirror holds the universe at several timestamps,
  and a bump changes some versions. A timestamp between two snapshots
  answers as the earlier one, as snapshot.debian.org redirects.
- **Variants** as mmdebstrap defines them: `essential`, the Essential
  packages; `apt`, those and apt; `required`, `important` and
  `standard`, each the packages of its priority and those above it, with
  apt.
- **No Breaks, Replaces or Recommends,** until a scenario shows that
  one changes a closure (5.1). The fake mmdebstrap asserts `APT::Install-Recommends "false"` (render.md, 5.2),
  naming the option where a world passes another.
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
  configuration, the modules, the generator) do not, so only
  slots.md, 6.1's rule, which compares the inputs' digest, allows a soft
  reboot after a rebuild (8.3). With `AUTO_MODULES=yes` the hook
  inspects the running system, which in a sandbox is the wrong one:
  render refuses it and the build checks again (build.md, 5.5), and the
  fake keeps the behaviour so that both checks are tested.
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
  `/var`, the class subvolumes at `/srv/<class>` and the entries' binds.
  A bind waits for the mount its source lies on and for its own
  parent's, as systemd orders it, and fails if either fails. A unit's
  outcome is its package's, or its cue's (8.2). The boot reaches its
  target, or does not.
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
- **Out of memory:** a cgroup past its `memory.max` loses a process of
  its own subtree, killed: a step's in its leaf (build.md, 8.7), or,
  drawn by the seed, the builder itself when `steward/builder/` reaches
  its ceiling, never a process outside it (steward.md, 5.3).

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

### 3.12 Containers [M5]

A host's containers are part of the hearth half, on the host's fake
kernel (slots.md, section 12):

- **nspawn as a unit:** `systemd-nspawn@<c>.service` starts a container
  only if `/run/machines/<c>` links to a directory and
  `/run/systemd/nspawn/<c>.nspawn` exists; it reads `PrivateUsers=`,
  `ReadOnly=`, `Bind=`, `BindReadOnly=`, `Capability=`,
  `VirtualEthernet=` and `LinkJournal=` from it, and fails otherwise,
  as one started at boot before its link would.
- **A container is a view and a PID 1:** the slot's root, read-only,
  with its binds, seen through an id map of the container's range (3.4);
  the fake PID 1 of 3.7 runs inside it, with its units, failed units and
  `is-system-running`, and no kernel, firmware or initramfs of its own.
- **Reached from the host** with `systemctl --machine=` and
  `systemd-run --machine=`, which the fake answers from that PID 1;
  whether the real ones need a bus inside is the VM tier's question
  (7.3).
- **A host reboot** empties `/run`, so no container starts until the
  host's activator makes its link again; a power cut stops every
  container with its host.

### 3.13 Mounts, overlays and live changes [M3]

What a live change (slots.md, 6.6 to 6.9) needs of the machine. The
mounts and overlayfs are the generic half's; PID 1's jobs are hearth's
half's.

- **Mount namespaces and propagation:** each namespace a tree of
  mounts, each `shared`, `private` or `slave`; a mount made in a shared
  one appears in its peers and its slaves. systemd's services hold slave
  copies with binds of their own (`ProtectSystem=`, `ReadOnlyPaths=`,
  `InaccessiblePaths=`), as the units in a scenario declare.
- **The new mount API:** `fsopen`, `fsconfig` and `fsmount` give a
  detached mount that no namespace shows, walkable through its
  descriptor; `move_mount` attaches it, with `MOVE_MOUNT_BENEATH` under
  the current top, refused on a scenario's kernel older than 6.5, as
  `lowerdir+` is before 6.8; `umount2(MNT_DETACH)` removes a mount at
  once while open files and working directories keep it; `open_tree`
  clones and `mount_setattr` changes a clone's flags; `setns` moves a
  process into another namespace [M5]. `/proc/<pid>/mountinfo` is
  rendered from the tree, as the kernel renders it.
- **overlayfs, read-only:** lower layers stacked, topmost first; a 0/0
  character device hides a path, a whole directory included; a
  directory takes the topmost layer's attributes; a kind change needs no
  marker; a file reports its layer's device and inode, a directory the
  overlay's. A write to a layer while it is merged is a finding the
  referee reports, since the kernel leaves it undefined.
- **Propagation's two traps,** as measured on the kernel: a lazy unmount
  of a shared peer of a layer's source tears the overlay down; an
  overlay propagated into a service's namespace shadows that service's
  own mount beneath it until the service restarts.
- **PID 1's jobs:** `daemon-reload`, `stop`, `start`, `restart`,
  `reload` and `reload-or-restart`, each job with a result; a unit's
  `ActiveState`, `InvocationID` and `NRestarts`. A reload whose
  `ExecReload=` fails leaves the unit active and the job failed; a new
  `.wants` link starts nothing until started; a unit whose file is gone
  stays loaded while active. dbus and nscd are units with a reload.
- **Soft reboots and overlays:** with `/run/nextroot`, the old root's
  overlays go with it; a person's soft reboot without it keeps the
  overlay on `/etc` and drops `/usr/local`'s and `/opt`'s. Both are to
  verify (7.3), so until then the seed draws either.
- **Cues** (8.2) that only a live trial meets: `test-reload-fails`,
  whose `ExecReload=` fails when its configuration says so, and
  `test-dies-later`, which dies a drawn time after it starts.

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
  answers each argument vector hearth makes as the real tool does, and
  asserts that it is given no other, since a real tool's usage errors
  differ between its commands (`git diff` exits 129, `git rev-parse
  --verify` 1); and it works only where the real one would: efibootmgr writes variables only as root; mmdebstrap
  in unshare mode makes no user namespace when it runs as root, and as
  anyone else needs `newuidmap`, which `no_new_privs` disables.
- **Output follows recordings.** Every line it prints has the shape of a
  recorded real run (4.5), with its varying fields (names, versions,
  counts) filled from the fake's state. Recordings are of real tools,
  run as hearth runs them, not on a terminal, each under the version
  that made it (4.5), and a parser holds to every version recorded. M1's
  come from the machine its plan runs on, whatever Debian it runs;
  trixie's and forky's join when a machine that has them records, as a
  tool that moves does (5.5). Which versions must be recorded is open
  (section 11).
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
| git | `hearth-protocol-git` | render.md 2.2's `rev-parse`, `ls-tree` and `cat-file --batch` on a repository, and its `diff --no-index` of two files, by a line diff of the fake's own; the fetch of wire.md, section 4 | a remote unreachable; a revision missing; a branch rewritten; a stall | M1 |
| efibootmgr | `hearth-protocol-efi` | writes only: creates at a fixed number, deletes, `--bootnext`, `--delete-bootnext`, `--bootorder`, on the variables of efivarfs | the variable store full; efivarfs not writable; a number already taken | M3 |
| systemctl, at run time | `hearth-protocol-systemd` | `soft-reboot`, `reboot`, `reboot -ff`, `is-system-running`, `show`, `stop`; for a live trial, `daemon-reload`, `start`, `reload`, `restart` and `reload-or-restart` on PID 1's jobs (3.13); `restart` [M5]; a container's nspawn unit and `--machine=` (3.12) [M5] | a unit that fails; a job that times out; a reload that fails; a container that never comes up | M3 |
| systemd-tmpfiles and nscd, at run time | `hearth-protocol-activator` | a live trial's `--create` of `hearth.conf` in the running `/var`; nscd's `--invalidate` | a line that fails; nscd absent | M3 |
| systemd-run | `hearth-protocol-systemd` | the hold's timer [M4]; a check's transient unit (operating.md, 2.4), and with `--machine=` in a container [M5] | the unit refused; a timeout; an out-of-memory kill | M4 |
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
  `--verbose`, apt's and dpkg's lines; on failure, the tail of
  build.md, 10.1.

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
  argument vector, its environment (no terminal), its working directory,
  its stdin where the tool reads one (`git cat-file --batch`), its
  stdout, its stderr and its exit. A tool whose lines another prints is
  recorded under both versions: mmdebstrap's runs under
  `mmdebstrap/<its version>-apt<apt's>/`, as a resolution's record
  keeps both (render.md, 5.4).
- **From the store's root:** every conformance case runs there, as the
  builder runs its tools, so a recorded path (`./cache/…`) is the one a
  run prints.
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
| resolve | `mmdebstrap --simulate --mode=chrootless` through the local mirror, as its proxy (6.2; render.md, 5.2) | the closure: names, architectures, versions; the `Packages` stanzas it left, in the fields the universe keeps (3.6), the others one allowed difference (5.3) |
| resolve, both modes [M2] | the same requests simulated with `--mode=chrootless` and with `--mode=unshare`, for every variant | the two closures, which must be equal: os and the bash fleet resolved chrootless and built unshare, and a reason once found for preferring unshare was lost (render.md, section 10). With M2's real builds, since unshare needs the subordinate range (6.4) |
| fetch | `apt-helper download-file` through the local mirror, as its proxy, over loopback | files by digest; a 404's and a mismatch's exit, lines and leftovers |
| bootstrap | mmdebstrap as build.md (5.4, 8.3) runs it: as the root of a sandbox's user namespace, `--mode=unshare`, chrooted into a staging layer | the installed set; dpkg's status, stanza by stanza; the tree: paths, kinds, modes, owners as the namespace sees them, attributes, link targets, contents by digest |
| the same, `--mode=root` | the bootstrap again in root mode | the two trees, to settle build.md's open choice (section 15) |
| install the extras | the bootstrap with the extras repository | users made; conffiles recorded; setuid bits; version 3 capabilities and their root id; a failing postinst's status and tail |
| query | `dpkg-query` on the bootstrapped layer, in build.md 5.7's format | stdout, byte for byte |
| the target's tools | `systemd-sysusers`, `systemd-tmpfiles` and `SYSTEMD_OFFLINE=1 systemctl preset-all`, inside a bootstrapped layer | users and groups made, files made, links made |
| git | real git on scratch repositories | revisions resolved, listings, blobs; hunks of `diff --no-index` on changes whose shortest diff is unique (a line changed, added or removed, lines appended), where any two line diffs agree |
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
set in a namespace; ACL ids; a map of two extents; and 3.13's mounts,
in a user and mount namespace of the regular user's own: a delta of
puts and 0/0 whiteouts merged over a base giving the new tree exactly,
for adds, changes, deletions, kind changes, links and modes; a
directory taking the topmost layer's attributes; a beneath-swap with
no window under readers in a loop; the propagation traps. Real owners,
setuid bits, capabilities and `trusted.*` attributes through an overlay
need root, and wait for the VM tier (7.3). Each case names the
rule it checks. No running kernel can check
the crash model, as skein's `kv.md` says of its own: it rests on the
sources it cites, and the VM tier's power cuts (7.3) test hearth, not
the model.

### 5.5 When it runs

`cargo nextest run --profile conformance`, as a regular user, with its
scenarios' prerequisites and the mirror; firmware scenarios run in the
VM tier. The profile needs what its scenarios need: in M1 (resolve and
git), git, mmdebstrap and the mirror's indexes, and none of 6.4's; from
M2, 6.4's too. Without them it refuses to start, naming what is missing
(6.4). It runs when a fake or a recording changes, when a tool
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
  `test-min` (the `apt` variant with no package of its own: the test
  repository's `tasks/hearth/task.json` declares none, format.md,
  5.12), and `test-boot` for the VM
  tier (a kernel, tiny-initramfs, systemd, systemd-container,
  openssh-server, incus-agent, and the tools hearth runs: mmdebstrap,
  uidmap, efibootmgr, git). The
  mirror digest is the SHA-256 of the pinned `InRelease`, kept in the
  pin: it names every index the mirror serves. A bump is a commit, as a
  snapshot bump in IaC is, and carries the keys and closures the tests
  hold with it.
- **The extras:** a few test packages built from sources in the
  repository with `dpkg-deb`, as a regular user: a postinst that fails,
  a conffile, a user made by a postinst, a setuid file, a capability.
- **At M1, indexes only:** a snapshot of forky, `main`, amd64, without
  the updates and security pockets; the suite's `InRelease` and its
  `Packages` index, whole and signed, so apt verifies them as it would
  snapshot.debian.org's, with the host's keys (render.md, 5.2), and any
  request resolves. `test-min` and
  `test-boot` are targets of the test repository (`testing/repos`) with
  the requests above, and the resolve scenario compares their closures
  (5.2). The pin's first timestamp is the canonical one
  snapshot.debian.org gives for the time the pin is made; a bump is a
  commit.
- **Fetched once, online,** by a script: indexes only at M1, then the
  closure's .debs through hearth's own fetch step [M2], into a cache
  directory named by the mirror digest, outside the repository, each
  file checked as any fetch is (`InRelease`, then SHA-256). A run whose
  digest is present fetches nothing, as mmdebstrap's `make_mirror.sh` is
  a no-op when nothing changed.
- **Served on loopback as an HTTP proxy,** by the harness, at a port it
  chooses: it answers each request for
  `http://snapshot.debian.org/archive/…` with the file the mirror holds
  at that path (`by-hash` paths included); a timestamp it does not hold
  as the latest earlier one it does (3.6), and one before them all, or
  any other request, with 404, so no test reaches beyond loopback. The builder under test takes
  it as its proxy (render.md, 5.2's `Acquire::http::Proxy`; the fetch's
  apt configuration [M2], build.md, 5.1), as a builder takes an
  apt-cacher-ng: the proxy is the builder's configuration, made by the
  test (client/README.md, 3.5), and never in a key (render.md, 5.1). So
  the test targets' sources are any target's at the pinned timestamp,
  with format.md, 5.3's fixed URLs, and their keys are those any
  builder computes for them. The build then makes its own local
  repository from its cache, as on any machine (build.md, 5.3).

### 6.3 The base layer, once per mirror digest

- **The real tier's store is a hearth store** that lives across runs in
  the test directory.
- **The base layer is keyed** by the resolution's digest
  (store.md, 3.3), so it is built once per mirror digest: the first run
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
  skein's contained process trees, which sandboxes wait for
  (build.md, 8.8).
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
- **Removing layers:** the tree helper's `delete` where the mount option
  is set, and thaw, empty and `rmdir` where it is not (store.md, 7.3 and
  7.4): both paths run.
- **A packages layer of its own** only where a test is about packages:
  the extras on the base; and `test-boot`'s, whose `/var/lib/machines`
  must be a directory, not a nested subvolume (8.5).
- **Both ways of making a packages layer** (build.md, 4.2): a closure
  that adds an extras package and drops another is reconciled from the
  base, and built from scratch when the test forces it. Each must pass
  the installed-set check, and the test records how the two trees
  differ, as the recordings the lab's comparison starts from.

Every test asserts its own cost facts: which steps ran, which layers
were reused, what was fetched.

## 7. The VM tier [M3]

### 7.1 The VMs

- **Incus VMs** (q35), each with its own disk: a block volume holding a
  partition table, an ESP and a btrfs filesystem
  (machine.md, section 3). Not the old lab's virtiofs boot: firmware,
  ESP and disk are what the MVP's machine has.
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
| prepare, soft reboot, bless | a change that cannot be live, such as a drop-in in `/etc/sysctl.d/` (format.md, 5.14): b prepared, a soft reboot into it, the minimal bless (slots.md, section 7) | b running, its normal entry first in `BootOrder`; no trial left |
| a packages change that keeps the kernel | a new version of an extras package: b's packages layer rebuilt, its initramfs rebuilt with the same inputs | slots.md 6.1's rule holds, so a soft reboot, then the bless |
| a kernel change | a mirror bump that changes the kernel: b prepared, `BootNext` to b's trial entry, a reboot, the bless | before the bless, `BootNext` gone and `BootCurrent` b's trial entry; then b the default |
| fallback after a panic | the revision adds the `test-panic` task (8.2) | the trial panics; `panic=N` reboots into a, which records that b fell back |
| fallback after a hang | `test-hang` blocks the trial's target | `JobTimeoutAction=` or the dead-man unit reboots with `-ff`; fell back |
| a mount fails during a trial | `test-mount-fails` adds a mount that fails, and the trial lands in emergency mode | the dead-man, armed from `/run` or `hearth.trial=` and kept by `IgnoreOnIsolate=yes`, still reboots with `-ff`; fell back |
| a failed unit | `test-unit-fails` fails a unit | the boot is `degraded`, and the slot's steward asks to fall back at once (slots.md, section 8), with no dead-man; fell back |
| a failed unit, allowed | the same unit, named in `bless.ignore_failed` (slots.md, 7.1) | the minimal bless passes; b blessed |
| a power cut during prepare | a kill at a drawn moment of prepare | a boots; the half-prepared slot is redone or discarded (slots.md, section 11) |
| a power cut in trial and bless | kills just after `BootNext` is set, and between the steps of the bless: journal, `blessed`, marker removed, default moved, dead-man stopped | the default boots a blessed slot |
| BIOS [M3b] | the bootstrap, a one-shot trial, a fallback after a panic, and a cut while the activator writes the environment pair, on SeaBIOS | `next_entry` consumed in both files before the kernel; the default boots a blessed slot |
| BIOS, a blessed default that panics [M3b] | after a bless, the harness arms `test-panic` through a flag in a data entry, so the default panics at every boot | GRUB's boot counter answers (machine.md, section 5), and the machine reaches the other blessed slot without a person |

The steward and its builder, on the same VMs (steward.md, section 5):

| Scenario | What happens | What must hold |
|---|---|---|
| a builder killed for memory | during a packages step, the harness, root in the VM, writes a `memory.max` on `steward/builder/` below the step's peak | the kernel kills within `steward/builder/` only; a step killed fails as that step, the builder killed loses its job, tried once more by a new builder (steward.md, 5.1 and 5.3); `hearth status` answers throughout, and the loss is a fact |
| a builder killed mid-pass | `kill -9` of the builder during a build | the job lost and tried again by a new builder, whose startup scan settles the staging layer (store.md, 4.4); converged; the steward never restarted |

Data by class, on the same VMs (format.md, 5.7; machine.md, section 3;
operating.md, section 7):

| Scenario | What happens | What must hold |
|---|---|---|
| binds after their class | a target with `journal` and `iwd` in `core` and `home-free` in `data` | `/srv/core` and `/srv/data` mounted from `@<target>.core` and `@<target>.data`; each bind up after its class's mount and after `/var`; journald writes `/srv/core/journal/<machine-id>` |
| seeding | an entry whose path the output fills, `/var/spool/postfix` | its directory made once and seeded by reflink, owners and modes as the output had them; the next prepare leaves it alone; the slot's own copy hidden under the bind |
| a class that will not mount | the harness renames `@<target>.data` before a boot | the class's mount and every bind of it fail, the boot isolates `emergency.target`, and no service starts on the slot's copy of a path |
| a snapshot across two services [M5] | two services in `data` append numbered records to their files, without pause, while snapshots are taken | each snapshot holds both files as of one instant: no record in one past what the other had at that instant, beyond what was in flight |
| retention [M5] | a day of hourly snapshots, with time sped up | each bucket's newest kept, every other snapshot pruned; `cache` never snapshotted |
| restoring one entry [M5] | `hearth restore mail --from <seq>` | after the reboot, `/var/mail` holds the snapshot's files and `/var/spool/postfix` today's; `mail.before.<seq>` kept beside it |
| restoring a class [M5] | `hearth restore --class data --from <seq>` | after the reboot, every entry of the class as of the snapshot; `@<target>.data.before.<seq>` kept |
| a power cut in a restore [M5] | `incus stop --force` between the two renames | the start finishes or undoes the swap from its record, then reboots |
| reading a snapshot [M5] | `hearth shell --snapshot data@<seq>` | the snapshot's files, read-only, as `nobody` with `CAP_DAC_READ_SEARCH` |

Live changes, on the same VMs (slots.md, 6.6 to 6.9):

| Scenario | What happens | What must hold |
|---|---|---|
| a config change applied live, with a reload | a new line in the `ssh` task's drop-in, its `live` entry `reload` on `ssh.service` | no reboot; `/etc` an overlay with source `hearth-live`; `ssh` reloaded, its main process the same; `b` live-blessed; `BootNext` `b`'s trial entry and `BootOrder` unchanged; drift's `slot.live.*` clean |
| an in-house binary under `/opt`, with a restart | a `test-service` task whose download under `/opt/test-service` is a new release, its `live` entry `restart` | the unit stopped before the swap and started after; the new binary answers with its version; the old one never ran with the new files |
| an added file and a deleted file | the revision adds a file under `/etc` and deletes one under `/opt`, both covered | the merged view equals `b`'s, field by field; the whiteout hides the deleted file; L2's walk and `slot.live.tree` pass |
| a failed reload | an `sshd_config` drop-in that `sshd -t` rejects | the reload job fails while the unit stays active; the live fall back: files back, `ssh` reloaded again, `fell back (live)`; no reboot; `b`'s prepare key refused |
| a crash at each live phase | `kill -9` of the activator at L1, L2, L3, L4, L5, L6, L7 and L8 in turn | before arming: `b` `prepared`, the delta deleted; from `armed` to `running`: `reboot -ff` into `a`, then `fell back`; after the live bless: `live-blessed` kept |
| a power cut at each live phase | `incus stop --force` at the same points, and after the live bless | `a` boots natively and `b` fell back, or `prepared` if it was not armed; after the live bless, `b`'s trial boots |
| a person's soft reboot while live | `systemctl soft-reboot` with no next root, after a live bless | the start finds the overlays disagreeing with the live record and reboots with `-ff`; recorded: which overlays survived the soft reboot |
| the next native boot as the trial | after a live bless, the harness reboots the VM | `BootCurrent` `b`'s trial entry; `b` blessed and the default; the delta deleted; no overlay |
| a later live change, and its fall back | `c` while `b` is live-blessed, then `c`'s reload fails | `c` prepared into `b`; its delta against `a`; the fall back swaps back to `b`'s delta; the next boot runs `a` |
| a sandboxed service | a unit with `InaccessiblePaths=` beneath `/etc`, unchanged by the change | restarted by slots.md, 6.7's rule 11, and seeing its sandbox's view after |
| not live | a change to `/etc/sysctl.d/`, a new package, a renumbered user | an ordinary trial; the `activation` fact names the rule that failed |

The VM tier also settles what slots.md, section 15 lists as to verify
for live changes, on trixie's systemd 257 and forky's 262: real ids,
capabilities and ACLs through the overlay; `MOVE_MOUNT_BENEATH` from
PID 1's shared root, with real sandboxed services; `daemon-reload` and
the dynamic `etc.mount`, `usr-local.mount` and `opt.mount` units around
a swap; overlays across soft reboots; the base's dead-man started after
boot; `BootNext` kept over days and consumed by a power cut; timing on
a real tree; Debian's
specifics (AppArmor, dbus's `ReloadConfig`, glibc rereading
`nsswitch.conf`, `hosts` and `resolv.conf`); and whether a lower mount's
`noexec` reaches the merged files.

Containers [M5], in a host VM:

| Scenario | What happens | What must hold |
|---|---|---|
| a container's slot | a container target placed on the VM's host target: its slot prepared on the host, then its nspawn unit restarted on it (slots.md, 12.1 and 12.2) | the unit stably active, the slot blessed as the container's default in the host's state |
| a container that fails | `test-unit-fails` inside the container | its unit stays active, but the host reads a failed unit through `systemctl --machine=`, asks to fall back, and the host's activator restarts the container on its old slot (slots.md, 12.3); with the unit on `bless.ignore_failed`, it is blessed |
| a container that never comes up | a slot whose nspawn unit fails | the trial fails at the first `failed`; the container runs its old slot |
| a host reboot during a container's trial | `incus stop --force` mid-trial | the links in `/run` are gone; the host's activator restarts the container on its default, and records the trial `fell back` |
| reading a container from its host | the facts and the criterion of a running container | recorded: whether `systemctl --machine=` needs a bus inside (slots.md, section 15); running reads as the subvolume of the container's root |
| a container's checks | a check of a unit and a probe, run by the host's hearth (operating.md, 2.9) | reported as the host's facts, naming the container's target |
| a pool host | one output, several containers (slots.md, 12.4's lean) | one build and one shift; a snapshot per container; each container's files owned as nspawn maps them |
| a host and its container in one revision | both specs change | the container's trial first, the host's last, since the host's may reboot the machine (wire.md, 4.7) |
| a container's parameters change | a bind added to its placement | a new slot of the same output, its settings with the bind, tried by a restart (slots.md, 12.1) |
| a container that fails on its host's trial | the host's trial boots and the container's unit fails | the host is blessed all the same; the container's facts report it (slots.md, 7.1) |
| a container's journal | entries written inside the container, read by the host's steward while it runs and after it stops | read from `/var/log/journal/<its machine-id>` in the host's journal entry, in its `core` class; recorded: whether journald inside writes there under its id range (operating.md, section 5) |
| a container removed | the revision drops the container's target | its unit stopped, its slots deleted and their outputs unpinned; its class subvolumes reported orphaned (slots.md, 12.7) |
| a container's change applied live | a config change with a `reload` entry, in a container target | the overlays in the container's mount namespace, none in the host's; the unit reloaded through `systemctl --machine=`; the host's state records the container's next start as `b`'s trial (slots.md, 12.8) |
| a container's live fall back | the container's reload fails, then a step of the fall back is made to fail | the files back and the units acted on again; then a restart of the container on its default |
| a container's next start after a live bless | the harness restarts the host VM | the container starts on `b`'s trial, which its host blesses; the delta deleted |

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

### 7.6 The development loop

development.md's loop runs here too, on the VM tier's machines and, from
M3b, on a development host with the Incus driver. Its worlds live with
the others: capture and the bundle in the slots', activator's and
standalone worlds [M3], and `hearth-client-dev-world` under `tests/dev`, with
`hearth-client-fake-incus` under `testing/`, for the driver [M3b]
(development.md, section 10).

| Scenario | What happens | What must hold |
|---|---|---|
| a mutable slot [M3] | a VM bootstrapped with `--accept-mutable`, its target placed `mutable`; and one bootstrapped without | the root writable, its read-only flag clear; the second refuses the place, naming the field |
| capture [M3] | by hand: `apt install`, an edit in `/etc`, a program in `/opt`, `systemctl enable`, `useradd` | `hearth diff --running` lists each once, packages without their files; recorded: what systemd writes in `/etc` at a mutable root's first boot |
| the round trip [M3] | the suggestions, unchanged, committed and applied | a soft reboot; the next capture empty |
| a working tree [M3] | `hearth revision bundle`, then `hearth revision set --bundle` on the VM | the commit under `refs/hearth-dev/`; converged; the same tree again changes nothing |
| explored, then a config change [M3] | an edit by hand, then a change with a `reload` entry | not live (slots.md, 6.7, rule 12): a soft reboot, the edit gone |
| up [M3b] | `hearth dev up` from the base image, timed | a copy in seconds; slot `a` of the target blessed, its identity not the seed's; whether the copy kept `qemu.nvram` recorded |
| several [M3b] | `hearth dev up --count 3` | one build; three identities |
| a cluster [M5] | three members of one base on one bridge | each resolves the others by `<instance>.incus` and reads their exports |
| a container in development [M5] | a host target with a mutable container placed on it | the container explored with `machinectl shell`, captured through its host |

## 8. Worlds and their invariants

### 8.1 The worlds

| World | Real | From |
|---|---|---|
| `hearth-render-world` | render's domain, under a scripted parent that plays the protocol layer and the store's records (render.md, 9.1) | M1 |
| `hearth-builder-world` | the builder's root and its children; tree helper and steward scripted | M1 |
| `hearth-store-world` | the store, on fake btrfs | M2 |
| `hearth-build-world` | plans and steps, on the fakes of `apt-helper`, mmdebstrap, dpkg-query and the target's tools, in sandboxes | M2 |
| `hearth-tree-world` | the tree helper's domain, on files with namespace ids | M2 |
| `hearth-system-world` | the system worlds (2.3) | M2 |
| `hearth-steward-world` | the steward's root and its children; builder and activator scripted (steward.md, section 8) | M3 |
| `hearth-converge-world` | the desired revision and the orders | M3 |
| `hearth-slots-world`, `hearth-boot-world`, `hearth-disk-world` | the activator's children, on firmware, btrfs and disks | M3 |
| `hearth-activator-world` | the activator's root and its children; the steward scripted | M3 |
| `hearth-checks-world` | checks, on units and signals | M5 |
| `hearth-backups-world` | class snapshots, their retention and restores; from M6, send streams and a backup peer | M5 |
| `hearth-artifacts-world` | the second recipe | M6 |
| `hearth-client-world` | the client's protocol layer, against the steward's domain (client/README.md, section 4) | M3 |
| `hearth-client-dev-world` | the development driver's domain, on `hearth-client-fake-incus` (development.md, section 10) | M3b |

Each document's own world section names its scenarios:
render.md, section 9; store.md, section 11; build.md, section 13;
machine.md, section 10; slots.md, section 13; activator.md, section 9;
steward.md, section 8; wire.md, section 10; client/README.md, section
4; operating.md, section 9; artifacts.md, section 8; development.md,
section 10.

### 8.2 Scenarios, cues and the referee

- **A scenario is set up in the fakes' terms:** the test repository at
  its revisions on git's remote; the universe and its mirror's
  timestamps; the machine's kind and disks; the secret store and the
  identities; the faults; the power cuts.
- **Boots follow content.** The test repositories have tasks that
  install real units: `test-panic` crashes the kernel through
  `/proc/sysrq-trigger`, at once or when a flag file exists in a data
  entry; `test-hang` blocks the boot's target;
  `test-mount-fails` adds a mount that cannot succeed; `test-unit-fails`
  fails; for live changes, `test-reload-fails` fails its reload when
  its configuration says so, `test-dies-later` dies a while after it
  starts, and `test-service` is an in-house service under `/opt` that
  answers with its version. The VM tier meets the real effect; the
  fake PID 1 knows these units by name and plays the same one. A
  scenario is then one repository at every tier, as temper's CI follows
  content.
- **The referee** (testing-strategy.md, section 7) watches the machine's
  ledger and hearth's facts, events and answers, never hearth's state.
  Its stimuli: a power cut, a reboot, a builder killed at any job, the
  steward killed, the client gone, the engine's channel dropping [M4].
  Its deadlines are simulated time, so a trial's window costs nothing.

### 8.3 hearth's invariants

- **Every crash leaves the default booting a blessed slot.** After every
  power cut, reboot, panic and watchdog reset, the world asks the fake
  firmware what it would boot (`BootOrder`'s first option that loads,
  after OVMF's pruning; on BIOS, `default` of the environment file GRUB
  would read), and checks that it names a
  slot whose record says blessed and whose tree equals that record.
  This holds from the first bless; what a cut during bootstrap leaves is
  machine.md's (section 8). **Its container form [M5]:** after every
  host start, each container's link names a slot whose record says
  blessed, or none before its first bless; a link to a slot on trial
  never outlives its trial (slots.md, 12.6).
- **The four facts stay consistent** (slots.md, section 3). At every
  settle, and with every fact hearth reports, the world compares them
  with the machine's ground truth: running is the subvolume the fake
  booted; default is what its firmware would boot; trial is the
  activator's marker; live is the fake's mount table. For a container
  [M5], running is the subvolume of its root, and default the record
  its host keeps.
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
  operation the steward asks of its activator over the link, and each
  job it gives its builder ends in one answer or one loss
  (steward.md, 3.2).
- **Trials always end.** When a trial starts, the referee arms a
  deadline: the bless, or the fallback, by the trial's bound. A command
  line without `panic=` meets the fake's endless panic and fails here.
  A container's trial ends by its host's activator alone, so a host's
  steward that never asks still meets the deadline [M5].
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
- **Live exactly when allowed.** The world computes slots.md, 6.7's
  rules on its own from the fake trees and specs, and each trial is live
  exactly when they hold; the merged view after every swap equals the
  live record's manifest, which equals the new slot's files under the
  three trees.
- **The steward stays in view** (steward.md, section 5). Whatever its
  builder does, dying at any job, hanging, or killed for memory, the
  steward keeps answering the client and, from M4, its channel; a job
  lost twice fails as `crashed`, and the pass follows steward.md, 5.1's
  table; nothing outside `steward/builder/` is killed for the builder's
  memory; and the ledger shows the steward opening nothing beneath
  `layers/` or `repo/` and spawning no build tool (3.1; steward.md,
  2.2).
- **No native bless before the bless's render.** No slot is blessed by
  its own steward before a builder of that steward has rendered the
  slot's revision; a render that fails ends the trial in a fall back
  (steward.md, section 6).
- **A live state never moves the default.** No live bless writes
  `BootOrder` or `default`; after every reboot or power cut no overlay
  remains, and the firmware boots a blessed slot or the trial entry a
  live bless armed; and every live trial ends within the base's window,
  by the live bless, the live fall back or a reboot.

### 8.4 The strategy's checks

- **Contracts** as the world goes; **settled invariants:** no live
  entity, nothing in flight, every child gone and its pipes read to the
  end.
- **Memory:** `skein-heap` checks each domain against its worst case
  (programming-model.md, 6.3), in a test binary of its own.
- **Replay:** a seed replays to the same trace, the same fake machine,
  whose state the fake digests itself, and the same digest of hearth's
  own state: a digest over the state types' derived `Hash`, which
  hearth asks of skein (README.md, 8.1; skein's `lib.md`, 5.1). Each
  process's `state_digest` and the fake machine's are read at the end of
  a run, and after every iteration when a world asks (skein's
  `testing.md`, section 5). Until skein has it, replay compares the
  trace and the fake's digest.
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
| a failed unit | slots, system | `test-unit-fails`: the trial boots `degraded` | the slot's steward asks to fall back at once; the dead-man plays no part. With the unit on `bless.ignore_failed`, the bless passes |
| a cut in the bless | activator, system | a crash of the activator, and a power cut, between each step of the bless (slots.md, section 7): journal, `blessed`, marker removed, default moved, dead-man stopped | a power cut boots the default, a blessed slot; after a crash the running slot is blessed but not the default, and the steward blesses it again, which finishes the order |
| both GRUB saves fail [M3b] | boot, system | GRUB cannot save `env.0` nor `env.1`, so `next_entry` stays set | the activator in the trial's slot clears it and reports it (machine.md, 5.6); a trial that panics meanwhile is bounded by the boot counter; the default stays a blessed slot |
| a blessed default that panics [M3b] | boot, system | the default panics at every boot | the boot counter moves GRUB to the other blessed slot (machine.md, section 5) |
| hostile tree shapes | activator, slots | an output whose `/var`, `/etc` or kernel is a symbolic link, or with a FIFO where `/etc/passwd` should be | prepare refuses each, since every slot path is resolved with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS` and `RESOLVE_NO_XDEV` (3.2); nothing outside the slot is read or written |
| a cut between walk and record | store, system | a power cut after a layer's walk and before its record; then, later, a published layer found corrupt (3.10) | `syncfs` precedes the record (store.md, 4.2), so a recorded layer is whole; the startup scan deletes the staging layer (store.md, 4.4); a corrupt layer found later is deleted and rebuilt, not refused |
| `systemd-container` | build | the closure holds `systemd-container`, whose tmpfiles make `Q /var/lib/machines` | with the setup hook's directories, the layer holds a directory; without them, a nested subvolume, which the walk refuses (store.md, 4.1) |
| a snapshot bump that keeps the kernel | system | 8.3's soft-reboot rule | a soft reboot, then the bless |
| a reload that fails | slots, system | `test-reload-fails` in a live trial | the live fall back: files back, units reloaded again, `fell back (live)`, no reboot |
| a cut at every live phase | activator, system | a crash of the activator, and a power cut, at each step from L1 to L9 (slots.md, 6.8) | before arming, `prepared`; after, `reboot -ff` and `fell back`; after the live bless, its trial at the next boot; the default always a blessed slot |
| a later live change that falls back | slots, system | `c` while `b` is live-blessed, its reload failing | the delta against `a`; the swap back to `b`'s delta, which no slot holds any more; the next boot runs `a` |
| a person's soft reboot while live | slots, system | a soft reboot with no next root, the seed drawing which overlays survive | the start runs `reboot -ff`; nothing runs a half-overlaid tree past it |
| an entry that changes class | slots, system | a revision moves `mail` from `data` to `core` | the place is refused, naming the orphaned directory in `data`; no empty `mail` is seeded in `core` |
| a cut while seeding | slots, system | a power cut while an entry's `<name>.new` is filled | the next prepare deletes the `.new` and seeds again; no half-seeded entry is ever bound |
| a snapshot across two services [M5] | backups, system | two writers in one class, snapshots and prunes cut at every step | each snapshot is both writers' state at one instant; retention keeps each bucket's newest |
| restoring one entry [M5] | backups, system | a restore of `mail` from a snapshot, cut between its renames | finished or undone at start; `postfix`, in the same class, untouched |
| a revision that crashes the builder | steward, system | every builder dies rendering one revision: scripted in the steward's world, killed by the referee at that job in the system world | the job lost, tried by a new builder, lost again: `crashed`; the pass waits for another commit, the crash a fact and an event naming the revision; the client's `status` answered throughout; the next commit converges (steward.md, 5.1) |
| a builder killed for memory | steward, system | `steward/builder/` reaches its ceiling in a packages step (3.10): the step's leaf killed in one seed, the builder's own in another | the step killed: that step's failure (build.md, 4.3); the builder killed: a lost job, tried once more; nothing outside `steward/builder/` killed; the steward answers throughout |
| a builder that hangs | steward | the scripted builder never answers a build | at the job's deadline the steward stops the builder's cgroup; the job lost and tried again (steward.md, 5.2) |
| the bless's render fails | steward, system | in the new slot, the render of the slot's revision fails: a configuration error in one seed, the builder lost twice in another | the steward asks to fall back, never to bless; the default never moved; `a` boots, and `b`'s prepare key is refused (steward.md, section 6) |
| a release that renders another output | steward, system | the new slot's builder renders the slot's revision to another output key | the bless is asked and given; a `render changed` fact; the next pass builds that output and tries it |
| a steward that dies with a builder running | activator, system | the steward killed in the middle of a build | the activator empties `steward/` with `cgroup.kill` before it spawns a new steward, so no builder outlives its steward; the next builder's startup scan settles the staging layer; the pass goes on (steward.md, 5.4) |

## 9. Budgets and costs

### 9.1 The two suites

The focused suite takes at most **15 seconds** and the fuzzy suite **60
seconds**, wall time on the development machine, idle, as skein's,
smith's and temper's are measured; nextest's global timeouts enforce
them, and raising one is an explicit decision
(testing-strategy.md, section 8). temper's gate recorded 1,092 focused
tests in 5.2 seconds; the budgets hold when the design is cheap from the
first step:

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
| `conformance` | section 5 | its scenarios' (5.5): in M1 git, mmdebstrap and the mirror's indexes; from M2, 6.4's too | 2 minutes |
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
| M1 | step; domain (render, builder, with the store's resolutions and cache, store.md, 2.4); machine (config, template, git, mmdebstrap); protocol (builder and tools); simulated (builder); real loop (render, real git); end to end (render, diff) | files with owners and modes; processes; the universe's index at snapshot timestamps | git; mmdebstrap `--simulate` | conformance: resolve, git; the mirror's indexes |
| M2 | domain (store, build, tree); system (builder and tree helper); machine (dpkg, tree); protocol (tree helper); real loop (tree helper, process groups) | btrfs and its ownership rule; ids, maps of several extents, namespaces, cgroups, sandboxes; `openat2`'s flags; attributes, inodes, ctimes; the full universe, `systemd-container` included; full disks; power cuts on files and subvolumes, and `syncfs` | mmdebstrap's bootstrap; `apt-helper`; dpkg-query; the target's systemd tools and `tar` | conformance: bootstrap in both modes, extras, fetch, query, the target's tools, the btrfs rules; real builds |
| M3 | domain (steward, converge, slots, boot, disk, activator); system (the steward and the activator join); machine (link, jobs, efi, systemd, keygen, client, proc), each fuzzed; protocol (link, jobs, client); simulated (every process, across reboots); real loop (the client's socket); end to end (daemon and client); builders lost at every job, and the bless's render (steward.md, section 8); capture and the bundle (development.md, section 10) | GPT and the ESP; efivarfs, two entries per slot, and OVMF; the kernel, the initramfs, PID 1 with its isolation to emergency mode and its jobs; reboots, soft reboots, panics, the watchdog; power cuts on FAT and variables; mount namespaces, the new mount API and overlayfs (3.13); out-of-memory kills within a cgroup (3.10); cues; a mutable root changed by hand, apt and dpkg on a running slot | efibootmgr; systemctl at run time, its unit verbs included; systemd-tmpfiles; ssh-keygen, for SSH host keys only; mount, cp and bootstrap's disk tools; git's bundles | VM tier on OVMF, live changes included, builders killed for memory and mid-pass, and mutable slots, capture and working trees (7.6); conformance: firmware, host keys, overlays in a user namespace |
| M3b | machine (grub, incus); domain (dev) | SeaBIOS; msdos and the post-MBR gap; GRUB on FAT, its environment pair, its boot counter, saves that fail, and the slots' fragments; Incus's instances and pool | grub-editenv; grub-install; incus | VM tier on SeaBIOS, and the development loop on a development host (7.6); conformance: env file, GRUB, incus |
| M4 | protocol (the wire, on `skein-fake-channel`); machine (wire) | — | `hearth-fake-engine` (4.6); systemd-run, for a held bless's timer | — |
| M5 | domain (checks, backups); containers and pool hosts | units' states and signals; containers on a host (3.12); `setns` into a container's mount namespace; read-only snapshots of class subvolumes and their pruning | systemctl restarting a unit, a container's nspawn unit and `--machine=`; systemd-run's checks | VM tier: bless with checks; containers in a host VM, live changes included (7.3); clusters and containers in development (7.6) |
| M6 | domain (artifacts; backups' sends); protocol (bytes by digest, between two machines) | send streams and a backup peer; faults as effects; virtual disks | — | real builds: artifacts; development VMs from the host's store (development.md, section 9) |
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
- **Live changes on a real machine:** every item slots.md, section 15
  lists as to verify, run as 7.3's scenarios. Until then the fake
  machine follows the kernel's and systemd's sources (257 to 262) as
  read for this design, and where a behaviour is unknown (overlays
  across a person's soft reboot) the seed draws each answer.
- **Differential checks:** whether a scenario must end the same at every
  tier it runs at, as temper also asks. It depends on how much the
  referee can observe in the real tiers.
