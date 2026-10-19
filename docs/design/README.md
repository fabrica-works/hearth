# hearth's design

Provisional, 2026-10-09, revised 2026-10-16. The design of hearth, the
factory's build daemon (temper's `factory.md`, section 7): what it
makes, how it makes it, how a machine runs what it made, and how all of
it is tested. It takes `draft/brainstorm.md` further and holds where the
two differ (section 7). The mechanics are those of skein's
`programming-model.md`; the tests follow `testing-strategy.md`. This
document is the overview: the reading order, the milestones, the
structure, the names, the conventions, the decisions taken, what hearth
asks of others and what is open. Each part has a document of its own
beside this one.

## 1. In one page

- **hearth makes outputs from inputs, keyed by digest, and runs them in
  slots.** One daemon on every machine: hosts, the lab's pool hosts and
  its builders. It replaces os's `manifest`, `deploy`, `provision` and
  `ek`.
- **The IaC format is os's, rebuilt for agents and digests**
  (format.md): strict JSON, inert Jinja templates, tasks that declare
  their settings, merging that can remove, exports between targets.
  Every input is in the repository.
- **Render is a pure function of a revision and a target** (render.md).
  It runs in full in milliseconds and gives a spec: parts, each with a
  digest. Any hearth renders any target, and the rendered diff of a
  change is what its reviewers read.
- **Outputs are layers** (store.md, build.md): packages, then software,
  then config, each a read-only btrfs subvolume named by its cumulative
  key. A change rebuilds from the lowest layer whose key it changed, on
  a snapshot of the layer beneath.
- **Three processes: two split by privilege, two by failure**
  (activator.md, steward.md). The activator, root and small, owns the
  machine's slots, boot state and secrets, and does a closed list of
  operations. It starts the steward and listens to nothing else. The
  steward, unprivileged and persistent, decides and faces: converge,
  the client, the engine, checks. It spawns the builder, unprivileged
  too, for work: render, builds, the store. A builder that crashes ends
  a job, never the process that reports it.
- **Immutable roots.** A slot's root (`/usr`, `/etc` and the rest) is
  mounted read-only. Only the slot's own `/var` and the data its tasks
  declare are writable: a directory per entry in one of the target's
  three class subvolumes, `core`, `data` and `cache`, bound at the
  entry's path and snapshotted by class. Nothing on a running machine
  is changed in place: a fix is a new slot, prepared beside the running
  one. Its configuration and in-house software may still reach the
  running machine with no reboot, as a delta of the new slot mounted
  over the old (the live changes below; slots.md, 6.6). A development
  machine may be mutable instead (development.md, section 4).
- **Slots try before they commit** (slots.md): prepare, trial, bless,
  fall back. A new slot runs once; the boot default moves only when the
  slot's own hearth blesses it (a container's, its host's), and any
  failure during the trial ends in a reboot into the old default.
- **Configuration and in-house software change live** (slots.md, 6.6):
  when a change touches only `/etc`, `/usr/local` and `/opt` over the
  same packages, and every changed path has a declared consumer, the
  activator mounts a delta of the new slot over the running one with an
  atomic swap and reloads or restarts the declared units, with no
  reboot; a failure puts the old files back. Its bless arms the new
  slot's trial for the next boot, so the default still moves only after
  a native boot.
- **Nothing unique and nothing secret in an output,** so outputs can be
  shared between machines. Identity and secrets are written into a slot
  when it is prepared, from a root-only store on the machine.
- **The engine decides, hearth carries out** (wire.md). With an engine,
  hearth prepares and activates on orders and reports facts. With none,
  it converges to a desired revision on its own: standalone mode, which
  is the MVP's.
- **Built on skein,** following `programming-model.md`: one loop, sans-io
  step functions, a domain complete enough to run whole on a fake
  machine. The heavy work runs in child processes: the builder, git,
  mmdebstrap, efibootmgr, and hearth's own tree helper.
- **Tested from day 0** (testing.md): on a fake machine, with a realistic
  fake for every tool, a conformance tier that holds the fakes to the
  real tools, real builds as a regular user, and VMs for what needs root
  until the lab exists.
- **Agents develop on throwaway machines** (development.md): Incus VMs
  cloned in seconds, roots they may change by hand, a capture of what
  they changed, and their working tree applied as a revision. Builds
  from scratch stay the lab's.
- **Built in milestones** (section 3): render, then builds, then slots
  on one EFI machine in standalone mode (the MVP), then BIOS, the wire,
  operating, and the rest of the vision.

## 2. Reading order

1. **README.md:** this overview.
2. **format.md:** the IaC format: the repository, targets, merging,
   tasks and their schemas, templates, exports, and secrets and data as
   the format names them.
3. **render.md:** from a revision and a target to a spec: the pipeline,
   parts and digests, package resolution, the rendered diff.
4. **store.md:** the store on btrfs: digests and keys, layers, the tree
   helper, records, pins and collection, hearth's own state.
5. **build.md:** how an output is made: the plan, the three layers,
   sandboxes, the tools as protocols, caches, and what an output must not
   hold.
6. **machine.md:** what a machine looks like: disks and partitions, the
   btrfs top level, how each kind of machine boots, the initramfs,
   bootstrap.
7. **slots.md:** a slot's life: the four facts, prepare, trial (live
   changes included), bless, fall back, roll back, drift, containers.
8. **activator.md:** the privilege boundary: the activator and its
   steward, the link, the activator's operations and what it verifies.
9. **steward.md:** the unprivileged side: the steward and the builder it
   spawns for work, the jobs between them, failures, the bless's render.
10. **wire.md:** hearth's faces: standalone mode, the wire to an
    engine, bytes by digest; and **client/README.md**, the command-line
    client, with the client's other designs under `client/`.
11. **operating.md:** after a rollout: checks, facts, telemetry,
    operational effects, snapshots and backups.
12. **testing.md:** how `testing-strategy.md` applies to hearth: the fake
    machine, the fakes of tools, the tiers and their budgets.
13. **development.md:** how an agent develops a machine: development
    machines on Incus, mutable roots, capture, applying a working tree.
14. **artifacts.md:** the second recipe: an artifact from a build
    environment, sources and a command.
15. **reproducible.md:** reproducible builds, deferred: what is kept from
    day one and how they are retrofitted.

`draft/brainstorm.md` is the record of the first discussion. These
documents take it further, and hold where the two differ.

## 3. Milestones

Each milestone leaves something usable on its own. Each document says
in its first paragraph which milestone builds it, and marks the sections
a later one builds (`[M5]` after the heading). The tests grow within
every milestone, from the first (testing.md, section 10).

| | Milestone | Gives | Builds |
|---|---|---|---|
| M1 | Render | specs with digests; the rendered diff of any change | format.md, but for checks and artifacts; render.md; the client's render and diff (client/README.md); store.md's encoding and a store without layers (2.4) |
| M2 | Build | outputs keyed by digest, built unprivileged, layer by layer | store.md, but for peers; build.md; the tree helper |
| M3 | Slots, standalone, on EFI: **the MVP** | one real EFI machine runs on hearth, installed from scratch | machine.md's EFI path and bootstrap; slots.md, but for containers; activator.md; steward.md, but for peers and artifacts; client/README.md's client and wire.md's standalone mode; a minimal bless; development.md's mutable machines, capture and working trees |
| M3b | BIOS; development machines | BIOS machines, which most IaaS providers offer; development machines on Incus | machine.md's BIOS path; slots.md's trial through GRUB; development.md's Incus driver |
| M4 | The wire | hearth as a system behind an engine | wire.md's wire, facts and events |
| M5 | Operating | the machine's end of the run side | operating.md, but for backups; checks as task data; local snapshots of each class, and restores from them (operating.md, section 7); containers and pool hosts (slots.md, section 12); clusters and containers in development (development.md) |
| M6 | The rest | the full vision | artifacts.md; peers; backups; faults; disks as an effect; development VMs from the host's store (development.md, section 9) |
| later | Reproducible builds | a level of trust | reproducible.md |

**The MVP needs no parity with os.** M3 leaves out on purpose: BIOS,
containers, the wire, task checks (a minimal bless stands in,
slots.md, section 7), peers, artifacts and backups.

**What it must get right anyway,** because changing it later is costly:

- outputs named by key, each recording its tree digest, in the ids seen
  inside the builder's namespace (store.md);
- nothing unique and nothing secret in an output (build.md, section 9);
- render a pure function of the revision and the target, exports
  included (render.md);
- the slots' four facts and their state machine (slots.md);
- every effect keyed or conditional, even while the client is the only
  one to ask (wire.md);
- the privilege split, from the first slot (activator.md), and the
  steward apart from its builder (steward.md).

## 4. Structure

### 4.1 Processes

```
a machine
├── hearth activator          root, started by systemd: the machine's slots, boot state, secrets
│   └── hearth steward        user hearth; persistent; started by the activator: converge, the faces, checks
│       └── hearth builder    user hearth, with a subordinate id range; started by the steward for work
│           ├── tools         git, apt-helper, mmdebstrap, dpkg-query: each build step in a sandbox of its own
│           └── tree helper   in the builder's user namespace: snapshots, freezes, walks, hashes, applies, empties
└── hearth (the client)       any user: talks to the steward over a local socket, or runs in-process
```

- **The link** joins the activator and its steward: a pipe pair made
  when the activator spawns the steward, so the activator hears no one
  else (activator.md, section 3).
- **The jobs** join the steward and its builder: the builder's standard
  input and output. The steward spawns a builder when it has work and
  lets it exit once idle, so a crash in render or a build loses a job,
  not the machine's face (steward.md).
- **The steward never reads the repository, runs a build tool or opens
  a layer;** whatever IaC steers runs in the builder or beneath it
  (steward.md, 2.2).
- **cgroups are delegated:** the activator's unit has `Delegate=yes`; the
  activator moves itself into a leaf and gives the steward a subtree of
  its own, in which the builder gets one with its own memory ceiling,
  for its sandboxes (steward.md, section 7).
- **A builder alone** serves tests, the lab's builders and a developer's
  machine: rendering, diffing and building need neither a steward, an
  activator nor root.
- **The client runs the builder's domain in-process** when no daemon
  runs, on a developer's machine (M1, M2), and whenever it renders a
  local repository: the daemon renders only its own clone
  (client/README.md, section 3).
- **Bootstrap runs the activator's code in-process,** as root from a
  live system, and it spawns a steward over a link as the daemon does
  (machine.md, section 8).

### 4.2 Crates

Crates are named by their layer (`programming-model.md`, section 4):
`hearth-domain-*`, `hearth-protocol-*`, `hearth-service-*`; the
client's are `hearth-client-*` (below). The crate graph is the role
graph.

```
lib                           no_std, skein-lib only
hearth-lib                    the canonical encoding and H (store.md, 3.1 and 3.2)        M1

domain                        no_std, skein-lib, hearth-lib and their own children only
hearth-domain-builder         the builder's root: routes between its children; jobs (steward.md)
├── hearth-domain-render      configuration to spec (format.md, render.md)                M1
├── hearth-domain-store       layers, keys, records, pins, collection (store.md)          M1, M2
├── hearth-domain-build       plans and steps (build.md)                                  M2
└── hearth-domain-artifacts   the second recipe (artifacts.md)                            M6
hearth-domain-steward         the steward's root: routes between its children; orders, facts, jobs  M3
├── hearth-domain-converge    the desired revision, orders, a slot's decisions (wire.md)  M3
└── hearth-domain-checks      checks, signals, windows (operating.md)                     M5
hearth-domain-activator       the activator's root (activator.md)                         M3
├── hearth-domain-slots       slot state, prepare, trial, bless, fall back, drift         M3
├── hearth-domain-boot        boot entries and files: EFI, then GRUB (machine.md)         M3
├── hearth-domain-disk        partitions, filesystems, the top level (machine.md)         M3
├── hearth-domain-backups     class snapshots, restores; from M6, streams (operating.md)  M5
└── hearth-domain-faults      a pool host's faults (operating.md)                         M6
hearth-domain-tree            walks, digests, change sets: the tree helper's, and the shift's (store.md)  M2

protocol
hearth-protocol-config        strict JSON to typed configuration and schemas              M1
hearth-protocol-template      the template language's parser (format.md, section 7)       M1
hearth-protocol-git           git's argument vectors and output                           M1
hearth-protocol-mmdebstrap    mmdebstrap: simulate, bootstrap, progress, failure tails    M1
hearth-protocol-dpkg          dpkg-query and apt                                          M2
hearth-protocol-tree          the tree helper's records; the store's records and tree files  M1, M2
hearth-protocol-accounts      passwd, group and shadow lines                              M2
hearth-protocol-tail          a tool's output tail, kept for a failure                    M2
hearth-protocol-xattr         file capabilities and POSIX ACLs                            M2
hearth-protocol-link          the steward–activator records, both ways                    M3
hearth-protocol-jobs          the steward–builder records, both ways (steward.md, 3.2)   M3
hearth-protocol-state         the activator's records, the trial marker, the slot record  M3
hearth-protocol-units         fstab, the trial's units, the watchdog's drop-in           M3
hearth-protocol-efi           EFI variables from efivarfs; efibootmgr's argument vectors  M3
hearth-protocol-systemd       systemctl, systemd-run and the journal's queries            M3
hearth-protocol-proc          /proc/<pid>/mountinfo and cgroup: mounts, overlays, units   M3
hearth-protocol-keygen        ssh-keygen and the other identity tools                     M3
hearth-protocol-grub          grub-editenv, GRUB's configuration and env                  M3b
hearth-protocol-wire          the wire to an engine, over skein's channel                 M4
hearth-protocol-probe         probes of a service as its users see it (operating.md, 2.5) M5
hearth-protocol-http          bytes by digest                                             M6
hearth-protocol-builder       the builder's protocol layer: routes its machines           M1
hearth-protocol-steward       the steward's protocol layer: routes its machines           M3
hearth-protocol-activator     the activator's protocol layer: routes its machines         M3

client                        what exists for the client, under one prefix, each crate held to its layer's rules (client/README.md, section 2)
hearth-client-protocol-records  the client's records and their printer                    M1, M3
hearth-client-domain-dev      the client's development loop; the Incus driver from M3b (development.md)  M3
hearth-client-protocol-incus  incus's argument vectors and JSON output (development.md, 8.3)  M3b

service and shell
hearth-service-builder        the builder's iterate                                       M1
hearth-service-steward        the steward's iterate                                       M3
hearth-service-activator      the activator's iterate                                     M3
hearth-service-tree           the tree helper's iterate                                   M2
hearth                        the binary: each process, and the client

testing
hearth-fake-machine           the fake machine (testing.md, section 3)                    M1
hearth-fake-<tool>            one fake per tool, acting on the fake machine               M1
hearth-fake-engine            an engine at the wire's far end (testing.md)                M4
hearth-client-fake-incus      incus's fake, for the client's development loop             M3b
hearth-system-world           the builder's and the tree helper's domains; the steward's and the activator's from M3     M2
hearth-<name>-world           the other worlds, under tests/
hearth-client-world           the client's world, under tests/client (client/README.md, section 4)  M3
hearth-client-dev-world       the development loop's world, under tests/dev (development.md, section 10)  M3b
```

- **Three crates start before their milestone.** M1 builds what render
  needs of them, and the later milestone grows each: `hearth-domain-store`
  holds resolutions and cache files' records, over a store without layers
  (store.md, 2.4); `hearth-protocol-tree` decodes the frame and records of
  kinds 1, 4 and 5 (store.md, 6.1); `hearth-client-protocol-records`
  holds the records of `render` and `diff` and prints them (render.md,
  8.1; client/README.md, 3.2).
- **Client crates live under one prefix,** as jig's do (jig's design
  `README.md`, section 15). A crate that exists for the client is
  `hearth-client-{domain,protocol,<kind>}-<name>`: after the prefix
  comes its layer, as for the daemon's crates, or its kind (a test kit's
  `-fake-`, a world's name ending in `-world`), so that everything the
  client alone needs lives under `hearth-client`. It is the client's
  domain, its protocol (the records it speaks, even where the steward
  reads them too), its tools' protocols, its fakes and its worlds. A
  crate the client hosts but the daemon runs too keeps the daemon's
  name: the builder's domain and protocol layer, `hearth-protocol-git`,
  the activator's crates for the root-only commands. The binary keeps
  the project's name alone, `hearth`, and composes the client; the
  prefix's base, `hearth-client`, is kept for a crate that composes it
  apart from the binary, should another program need one. The client's
  designs live under `client/` (client/README.md).
- **`hearth-lib` is hearth's one crate in the lib role:** the canonical
  encoding and `H` are needed by domains in three processes, which may
  depend only on lib and their own children (`programming-model.md`,
  4.5). It moves into skein's lib if another service needs it.
- **The builder's children,** and the steward's, are siblings that
  share no types; each root translates between them
  (`programming-model.md`, 4.5).
- **Every format is its own machine crate,** lib-only and fuzzed alone;
  a process's protocol layer only routes between its machines
  (`programming-model.md`, section 4; activator.md, 8.1). JSON's machine
  is skein's tokenizer, which `hearth-protocol-config` composes
  (render.md, 8.1).
- **The activator's domain never depends on the steward's, nor the
  steward's on the builder's.** The link's records are the only
  vocabulary the first two share, through `hearth-protocol-link`; the
  jobs' are the only one the last two share, through
  `hearth-protocol-jobs`. The client hosts the builder's domain
  in-process (client/README.md, 3.5), and needs no steward for it.
- **Drift is the activator's** (slots.md, section 10): it wrote the slot
  record and alone can read every file of a slot.
- **`hearth-domain-tree` has two parents in two processes:** the tree
  helper's service, and the activator's slots domain, whose shift walks
  every inode and computes the slot's tree digest as it goes.

## 5. Names

| Name | What it is |
|---|---|
| IaC repository | a system's declarative inputs: fleet defaults, environments, bases, tasks (format.md) |
| revision | a commit of it |
| base | a system described once, `hosts/<base>/<base>.json`: the host's own data |
| environment | `envs/<env>.json`: defaults for every target in it, such as `lab` or `prod` |
| target | a base in an environment, such as `mail-prod`: what is rendered and built |
| placement | where a target runs: bare metal, a VM, a container on a named target; data of its environment |
| machine | where a target runs: a disk with slots, or a container's root on a host |
| development machine | a throwaway standalone machine an agent changes and applies its working tree to: an Incus VM cloned from a base image [M3b], which may be mutable (development.md) |
| mutable | a placement whose slots are not sealed and whose root is mounted read-write, for changes by hand; taken only by a machine bootstrapped to accept it, never by the run side (development.md, section 4) |
| task | everything about one concern: settings, files, packages, software, data, secrets, checks, exports |
| setting, schema | a value a task reads; the task's declaration of its settings: names, types, defaults, a line each |
| export | a fact a task publishes to the other targets of its environment |
| spec | what a target is at a revision, rendered: parts with digests |
| part | a piece of a spec with its own digest: meta, packages, software, config, data, secrets, boot, checks, exports, live |
| resolution | the pinned package closure a packages part resolves to, remembered by its key |
| digest | SHA-256 over a canonical encoding |
| key | what a layer or a resolution is named by: the digest of its inputs; a layer's is cumulative |
| layer | a read-only btrfs subvolume, named by its key: packages, software or config |
| output | a target's top layer: a build of its spec |
| tree digest | the digest of a tree as seen inside the builder's namespace: paths, types, modes, owners, extended attributes, contents, link targets |
| change set | what a layer changed from its parent: changed files and deletions |
| store | the builder's layers, records, resolutions and downloads (store.md) |
| pin | what keeps a layer from collection |
| tree helper | hearth's child process inside the builder's user namespace that walks, hashes, applies and empties trees |
| sandbox | a build step's own namespaces, pivoted into its layer (build.md, section 8) |
| slot | where a machine runs an output: a root and its own `/var`, labelled `a` or `b` |
| slot record | what a slot holds and how it was prepared: its output, its digests, its own tree digest after the shift |
| shift | rewriting a placed tree's owners from the builder's id range to real ids |
| class | a kind of persistent data, `core`, `data` or `cache`, with one subvolume per target, `@<target>.<class>`, mounted at `/srv/<class>`, and a snapshot and backup policy of its own (format.md, 5.7) |
| data entry | a target's data that outlives its slots: a directory in its class's subvolume, bound at the path a task declares |
| snapshot, backup | a read-only snapshot of a class, kept on the machine by its policy [M5]; one sent to the backup peer, encrypted [M6] (operating.md, section 7) |
| secret store | root-only, on the machine: the target's secrets and the machine's identity |
| identity | what makes a machine itself: its machine-id, its host keys |
| running, default, trial, live | the four facts: the slot running, the boot default, the slot on trial, and the live state over the running slot (slots.md, section 3); a container's, read by its host (slots.md, 12.6) |
| live change, delta, live bless | a trial with no reboot: a delta of the new slot (`@<target>.live.<n>`) mounted over the running slot's `/etc`, `/usr/local` and `/opt`, the declared units acted on; its bless arms the new slot's native trial (slots.md, 6.6 and 7.6) |
| prepare, trial, bless, fall back, roll back | a slot's transitions (slots.md, section 4) |
| boot partition | the FAT partition the firmware or GRUB writes: the ESP on EFI, GRUB's env on BIOS (machine.md) |
| activator, steward, builder | the three processes: root and closed; persistent, deciding and facing; spawned for work (activator.md, steward.md) |
| link, jobs | the pipe pairs between the activator and its steward, and between the steward and its builder |
| client | the command-line face (client/README.md, section 3) |
| engine | what orders a machine: the run side's or the lab's (temper's `factory.md`) |
| plane | the lab or the run side: outputs and credentials never cross between them |
| standalone | hearth with no engine, converging to its desired revision |
| desired revision | what a machine converges to: ordered by its engine, or set locally |
| order | an engine's request: prepare, activate, bless (releasing a held one), roll back, restart a unit; later, build an artifact |
| fact, event | what hearth reports: state, read afresh; what happened, as a hint |
| check | what healthy means for what a task installs, as data (operating.md) |
| drift | a slot that differs from its record |
| costs | hearth's own, as facts: time per step, bytes fetched, cache hits |

## 6. Conventions

- **A bare file name** names a document of this directory: `store.md`;
  with a section, `store.md, section 4` or `(store.md, 4.2)`. A
  document in a directory beneath it is named by its path from here,
  `client/README.md, 3.5`, and names the others as this directory's
  documents do. The
  brainstorm is `draft/brainstorm.md`.
- **skein's foundation documents** are named by their file names:
  `programming-model.md`, `testing-strategy.md`, `notes.md`. Other
  documents carry their project's name, never a path: skein's
  `channel.md`, temper's `factory.md`, jig's `connectors.md`, os's
  `deploy.md`.
- **Each document** says in one page what it is, then its parts, what it
  owes and is owed below the domain, its world, what it keeps or changes
  from os, and what is open.
- **Milestones:** a document's first paragraph names the milestone that
  builds it; a section a later milestone builds says so after its
  heading, as `[M5]`.
- **Code cites a section** at module or type level, as `slots.md, 5.2`,
  never on every field.
- **Examples** use the fleet's targets (`x1-prod`, `mail-prod`,
  `rhi-prod`) and the factory's (`mail-lab`, `stoa-1-lab`).

## 7. Decisions this design takes

Questions the brainstorm left open (its section 13), and others met
while writing, with where each is answered.

**Decided with the user's input** (2026-10-09), each with consequences
that last:

- **Ownership across the user namespace.** The builder stays
  unprivileged. Its layers are owned on disk by its subordinate id range,
  aligned to 64 Ki; digests and change sets use the ids seen inside the
  namespace, so they agree across machines. When it prepares a slot, the
  activator snapshots the output and shifts it: owners, setuid and setgid
  bits, file capabilities and ACLs, at about 10 µs an inode, rewriting
  metadata only (slots.md, 5.2).
- **Secrets and identity are written into the slot** at prepare, by the
  activator, from a root-only secret store on the machine. Tasks declare
  each secret file's path, mode and owner. Rotating a secret is a new
  slot of the same output (format.md, section 9; slots.md, 5.4).
- **A slot is a read-only root and its own `/var`,** copied from the
  output at prepare, so dpkg's database travels with its slot. State that
  outlives a slot lives in data entries, at paths the tasks declare
  (format.md, section 5; machine.md, section 3).
- **Persistent data by class** (format.md, 5.7 and 5.12;
  machine.md, section 3; operating.md, section 7). A target has one
  subvolume per class, `@<target>.core`, `@<target>.data` and
  `@<target>.cache`, made only when an entry uses it and mounted at
  `/srv/core`, `/srv/data` and `/srv/cache`, os's and the bash fleet's
  layout. Each `data` entry is a directory in its class, named by the
  entry, bound at the path its task declares from the fstab the
  activator writes; tasks declare a path and a class, never a subvolume.
  Snapshots and backups are policies of the `hearth` task, by class:
  local snapshots [M5] and sends to the backup peer [M6], never of
  `cache`. A snapshot of a class is atomic across its services; one
  entry is restored by copying its directory out of a snapshot, a class
  by swapping its subvolume. A subvolume per entry is not in the design
  (machine.md, section 12).
- **Live changes, with hearth's own overlays** (slots.md, 6.6 to 6.9;
  format.md, 5.14), built in M3, a container's in M5. A change confined
  to `/etc`, `/usr/local` and `/opt`, over the same packages layer, with a
  declared consumer for every changed path, is applied with no reboot:
  a delta of the prepared slot, mounted over the running slot, the
  declared units reloaded, restarted or started. The activator mounts
  the overlays itself, with the new mount API and `MOVE_MOUNT_BENEATH`,
  not through `systemd-sysext` and `systemd-confext`, whose refresh is
  not atomic, whose mismatches are silent and whose search paths
  survive a soft reboot. The live bless arms the new slot's trial
  (`BootNext`, or `next_entry`) rather than moving the default, so every
  slot still boots natively on trial before it becomes the default.
- **The development loop** (development.md): up, explore, capture,
  encode in IaC, apply from the working tree, land. A development
  machine is an Incus VM cloned from a base image and driven by `hearth
  dev`, a client outside the daemon [M3b]. `hearth dev` names machines,
  never containers: a machine comes up with the containers placed on
  it, all or a selection, and a container is reached through its
  machine [M5]. An environment may mark its
  placement `mutable`: slots not sealed, roots read-write, taken only by
  a machine bootstrapped with `--accept-mutable` and never by the run
  side. `hearth diff --running` captures what was done by hand and
  drafts task fragments; a working tree becomes a machine's desired
  revision as a commit on no branch, shipped as a git bundle. Builds
  from scratch stay out of the loop: every apply prepares a fresh slot
  from the IaC, so what was not captured is gone at once, and a first
  boot, the upgrade from production and a sealed root are the lab's, at
  QA or at the landing gate (development.md, section 7).
- **Boot follows os, and adds the trial.**
  - EFI: the ESP holds each slot's kernel and initramfs, booted by a
    direct EFI-stub entry per slot. A trial is a `BootNext`; a bless
    moves `BootOrder`.
  - BIOS: GRUB, its core image after the MBR, reading kernels from the
    slot subvolumes. One change from os: a small FAT partition holds
    GRUB's files and its env, which GRUB cannot write on btrfs, so that
    `next_entry` is a one-shot; two env files, so a torn write never
    loses the default (machine.md, sections 4 and 5).
- **`/usr` is the packages', but `/usr/local`** (2026-10-16; format.md,
  5.5; build.md, 9.3). Above the packages layer, tasks' files,
  downloads, scripts and hearth itself write `/usr` only under
  `/usr/local`, as os did (its helpers in `/usr/local/bin`, its shared
  tree in `/usr/local/share`): hearth's binary is
  `/usr/local/bin/hearth`, and its unit and its generated sysusers,
  tmpfiles and preset files are in `/etc`. `/usr/lib` is Debian
  packages' alone: should hearth become one, it moves there with the
  packages layer (format.md, 5.12). Render refuses a task's path
  elsewhere in `/usr`, and the build fails a step that writes there. So
  a live change overlays `/etc`, `/usr/local` and `/opt`, never the
  `/usr` every process maps (slots.md, 6.6).
- **The steward and its builder** (2026-10-14; steward.md). The
  unprivileged side is two processes, by failure rather than privilege:
  the steward, persistent, holds converge, the client's socket, the
  wire, checks and telemetry, and never reads the repository, runs a
  build tool or opens a layer; the builder, spawned by the steward for
  work and gone once idle, does git, render, builds and the store. A
  builder that crashes loses a job, which a second loss fails as
  `crashed`, so a revision that trips a hearth bug is reported by a
  machine that stays in view. The bless asks for one render more, of
  the slot's own revision, so a release that cannot render what built
  it never becomes the default.

**Others:**

- **SHA-256 for every digest:** skein has it, and Debian's archive and
  snapshot.debian.org speak it (store.md, section 3).
- **The filesystem is hearth's only state:** subvolumes and the records
  beside them, read back at startup; no kv store (store.md, section 8;
  activator.md, section 7).
- **The layers are packages, software and config.** The brainstorm's
  etc layer also writes users, `/usr/local`, unit links and more, so it
  is named for what it holds (build.md, section 3).
- **A layer is one subvolume, `/var` included;** a slot's `/var` is
  split from it at prepare (store.md, section 4; slots.md, 5.3).
- **No template is completed at activation.** What os completed there
  (fstab's slot label) the activator writes from typed slot data: fstab,
  the slot record, the machine-id. The privileged side renders no
  template (slots.md, 5.5; activator.md, section 2). This replaces the
  brainstorm's activation templates (its 7.9).
- **The device leaves the output.** `root=` and fstab name the
  filesystem by UUID, supplied at prepare (machine.md, section 6).
- **A tree helper runs inside the builder's user namespace,** as the
  namespace's root. The builder cannot read every file its layers hold,
  and only the owner of a subvolume's root may snapshot it or make it
  read-only, so the helper snapshots, freezes, walks, hashes, applies,
  thaws and empties layers (store.md, 5.3 and 5.8).
- **Each build step runs in a sandbox** of its own namespaces:
  mmdebstrap's chroot does not contain code nobody reviewed. The
  packages step runs in a tool root and mmdebstrap chroots into the
  layer; the steps above it run in the layer itself; none has a network
  (build.md, 8.1 to 8.5). Only the fetch, by SHA-256 through
  `apt-helper`, reaches the network, in a cgroup of its own
  (build.md, 5.1).
- **The builder's subordinate range starts at 262144,** 65536 ids long,
  clear of systemd-nspawn's automatic range and of the fleet's
  containers' ranges (store.md, 9.2). The `hearth` user is declared by
  the hearth task, which every target but a container has
  (format.md, 5.12).
- **The kernel is unchanged** when the kernel image, the modules' tree,
  the initramfs's inputs (not its bytes, which are not reproducible) and
  the command line without the slot's own tokens all are; then a soft
  reboot is enough (slots.md, 6.1; build.md, 5.8).
- **Render refuses early** whatever would fail at prepare: reserved
  command-line keys, files hearth or the activator own, units named for
  hearth (format.md, 5.11).
- **A host keeps no list of its containers:** a container's placement
  names its host (format.md, 3.5). This replaces the brainstorm's list in
  the host's configuration (its 7.7).
- **The activator starts the steward** and talks to it over a pipe pair;
  it listens to nothing else (activator.md, section 3).
- **Which slot runs** is read from the root subvolume's identity
  (`BTRFS_IOC_GET_SUBVOL_INFO` on `/`), cross-checked with the slot
  record inside the root; never from `/proc/cmdline`
  (slots.md, section 3).
- **Trials always end:** `panic=N` on a trial's command line, a hardware
  watchdog where there is one, and a dead-man timer in the trial slot
  that forces a reboot unless the slot's hearth blesses it first
  (slots.md, section 6); a container's, by its host's activator at its
  window's end (slots.md, 12.3); a live trial's, by the running slot's
  own dead-man, started from `/run` (slots.md, 6.8).
- **A minimal bless until checks exist:** the boot reaches its target
  with no failed unit, the slot's own steward is up and talking to its
  activator, and a builder it spawned has rendered the slot's own
  revision (slots.md, section 7; steward.md, section 6).
- **Secure Boot off** for now: a Debian kernel cannot boot from a direct
  stub entry under it (machine.md, section 9).
- **tiny-initramfs is the initramfs,** Debian's package, configured
  explicitly per placement: it is in trixie and in forky, and it has
  been stable for years (machine.md, section 7).
- **No converter from os's format:** configurations are written again.
  os's golden files inform render's tests but are not its oracle
  (render.md, section 9).
- **Everything starts from scratch:** configurations are written again
  and machines are installed fresh, `x1-prod` included, by `hearth
  bootstrap` on a wiped disk, with identity generated anew; no part of
  hearth migrates os's data or machines (machine.md, section 8).
- **Root tests run on Incus VMs** with their own disk and OVMF, until the
  lab exists (testing.md, section 7).
- **Standalone mode follows a desired revision set locally:** a commit,
  or a branch to follow, with `trial: auto|manual`, so a laptop such as
  `x1-prod` reboots into a new slot only when its user chooses
  (wire.md, 4.1).
- **A machine follows an engine or converges alone, never both**
  (client/README.md, 3.1 and 3.8).
- **A bless can be held** by the engine while a canary bakes; the
  `bless` order releases it, and a timer that does not depend on hearth
  ends a hold that outlives its ceiling (slots.md, 7.4;
  activator.md, 5.13; wire.md, 5.4).
- **Render failures fall into three classes:** a configuration error
  waits for another commit; an operational failure (git, the
  resolution, a fetch) is retried with backoff; an output that failed
  to build, verify or hold is refused (wire.md, 4.4; render.md, 7.1).
- **The steward reads the journal itself:** it runs with `hearth`'s
  supplementary groups, `systemd-journal` among them, and no
  capabilities; its builder runs with none (activator.md, 3.2;
  steward.md, 2.3).
- **Users a package makes** (`postfix`) are declared by the task that
  installs it, so render knows them and the activator checks they exist
  at prepare (format.md, 5.6; slots.md, 5.7).
- **Owners of persistent data have fixed ids.** Every user or group that
  owns a data path, a secret or an identity file has a fixed uid or gid,
  package users included, seeded before any package is configured, so a
  rebuild from scratch never renumbers what a data entry holds
  (format.md, 5.6; build.md, 5.5).
- **A data entry is seeded once,** from the output's tree at its path,
  when its directory is first made in its class, so its bind never hides
  what packages put there (slots.md, 5.3).
- **Trials end even in emergency mode:** the trial units survive
  isolation, and arm from a `/run` marker or a `hearth.trial=` token on
  the command line, never from a mount; each EFI slot has a normal and a
  trial entry (slots.md, section 6; machine.md, section 4).
- **A bless marks the slot blessed before it moves the default,** so a
  cut leaves a blessed slot that is not the default, and the steward
  blesses it again (slots.md, section 7; wire.md, 4.2).
- **Refusals are keyed by the prepare key** (the output, the slot parts
  and the generations), never by the output alone; a corrupt layer is
  deleted and built again, not refused; the client can clear a refusal
  (wire.md, 4.4).
- **Prepare never follows a link the output plants:** every slot path is
  resolved beneath the snapshot's root without symlinks, and an output
  of the wrong shape is refused (slots.md, section 5;
  activator.md, section 6).
- **The boundary, stated honestly:** the activator keeps secrets from
  the steward and its builder and confines them to closed operations,
  but a compromised builder or steward is root at the next trial, since
  it can place an output whose activator is root. The defence against a bad output is review and the
  lab (activator.md, section 2).
- **Identity in M3 is SSH host keys,** made with `ssh-keygen`; WireGuard
  keys wait for in-process generation (format.md, 9.3).
- **The repository's read credential is the builder's own,** imported at
  bootstrap and kept in the store, never in the secret store
  (render.md, 2.2).
- **A layer is durable before it is recorded:** `syncfs` on the store
  between its walk and its record (store.md, section 4).
- **Package indices come from the resolution:** apt's verified,
  decompressed `Packages` files are kept by digest and served from the
  local repository; the fetch gets only packages (build.md, section 5).
- **The packages layer is reconciled by apt** from the target's previous
  one, and built from scratch instead, never in addition, when there is
  no base, when a reconcile fails, and now and then to bound the history
  a line carries; comparing the two is the lab's (build.md, 4.2).
- **Failed units a target expects** are listed as task data
  (`bless.ignore_failed`); the minimal bless counts only the others
  (format.md, section 5; slots.md, 7.1).
- **A host runs its containers [M5]:** its builders build their outputs,
  its activator prepares their slots on its top level, starts each at
  every start from the default it recorded, and tries a new slot live
  when the change is eligible, inside the container's mount namespace
  (slots.md, 12.8), or else by a restart, judged from outside with the
  minimal bless read through `systemctl --machine=`; the host's steward
  blesses it. No hearth runs in a container, which has no `hearth`
  task, while whether one should stays open (slots.md, section 12;
  format.md, 5.12).
- **A container's `/var` is per slot,** as a host's: the bash fleet's
  persistent `.var` becomes data entries, and a container's identity and
  secrets are its own target's, kept by its host (slots.md, 12.1).
- **A container's parameters are placement fields [M5]:** its id range,
  binds, capabilities and network, rendered into its `boot` part, so a
  change prepares a new slot of the same output (format.md, 3.5;
  slots.md, 12.1).
- **A container never fails its host's bless [M5]:** the host's minimal
  criterion leaves out its containers' nspawn units; each container's
  slot has its own trial and facts (slots.md, 7.1).
- **A container's journal is in its host's [M5]:** `LinkJournal=try-host`,
  under the container's own machine-id, read by the host's steward as
  its own (operating.md, section 5).
- **A container removed from its host's spec [M5]** is stopped and its
  slots deleted; its data stays, orphaned, until a person deletes it
  (slots.md, 12.7).
- **A container's orders are a host's [M5]:** a held bless ends at its
  ceiling in a fall back run by the host's activator, and a unit's
  restart goes through `systemctl --machine=` (slots.md, 7.4;
  activator.md, 5.9).
- **A build environment is a container with no host** [M6]: its output
  is only a sandbox's root, never an nspawn container
  (artifacts.md, 2.3; format.md, 3.4).

## 8. What hearth asks of others

### 8.1 skein

Each of these is generic, so it belongs in skein with hearth as its first
user (`testing-strategy.md`, section 9). From the earliest:

| Need | For | Milestone |
|---|---|---|
| a public, streaming SHA-256 in lib, usable by the domain | digests | M1 |
| JSON: each of the tokenizer's tokens and errors with its position in the document, its byte offset, line and column (the column in bytes, both from 1). `hearth-protocol-config` reads every document through skein-json's tokenizer, and refuses duplicate keys itself, at every depth, as it decodes each object's keys | strict configuration (format.md, 2.2); the file, line and column of every render error (render.md, 3.1 and section 7; format.md, 6.5) | M1 |
| a full stat by name, without following links; readlink; extended attributes | the tree helper's walk | M2 |
| writing a tree's metadata: symlink, link, chmod, chown, utimes, mkdir, truncate | change sets; the shift | M2 |
| spawning as another user, with supplementary groups; signalling a process group | the steward under the activator, the builder under the steward; stopping a tool | M2 |
| contained process trees (skein's draft `process.md`), with id maps through `newuidmap`, a composed root and `pivot_root`, network and uts namespaces, fresh `/proc`, `/sys` and `/dev`, `no_new_privs` and limits (build.md, 8.8) | sandboxes | M2 |
| a closed list of synchronous ioctls in the ring adapter: `GET_SUBVOL_INFO`, `GET_SUBVOL_ROOTREF`, `INO_LOOKUP_USER`, `SUBVOL_CREATE_V2`, `SNAP_CREATE_V2`, `SNAP_DESTROY_V2`, `SUBVOL_SETFLAGS` (store.md, 5.8) | the store and the slots, unprivileged where they can be | M2 |
| a digest of a world's state, for replay (testing.md, 8.4) | replay checks in hearth's worlds | M1 |
| an exclusive lock on a directory, taken without waiting, which the owner asks for again on its own timer (skein's `kernel.md`, 6.1; `io.md`, 5) | the store's lock (store.md, 8.2, step 1) | M1 |
| a rename from one directory to another (skein's `io.md`, 5) | a resolution's indices moved from its scratch directory into the cache (render.md, 5.3; store.md, 6.4) | M1 |
| a directory listed page by page (skein's `io.md`, 5) | the startup scan of `cache/` and `resolutions/`, whatever their size (store.md, 8.2) | M1 |
| `statfs`'s filesystem type | the scan's subvolume steps, on btrfs alone (store.md, 2.4) | M2 |
| Unix sockets with peer credentials | the client | M3 |
| redialling and identity on the channel's TLS client | the wire | M4 |
| `syncfs` | a layer's data durable before its record | M2 |
| `openat2` with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS` and `RESOLVE_NO_XDEV`; `FICLONE` | prepare resolving and copying a slot's paths safely | M3 |
| the new mount API: `fsopen`, `fsconfig`, `fsmount`, `move_mount` with `MOVE_MOUNT_BENEATH`, `open_tree`, `mount_setattr`; `umount2` with `MNT_DETACH`; walking a detached mount through its descriptor | live changes: overlays built aside, verified, swapped beneath and back (slots.md, 6.8) | M3 |
| `mknodat` of a 0/0 character device | a live delta's whiteouts | M3 |
| `setns` into another process's mount namespace, from a child | a container's live swap (slots.md, 12.8) | M5 |
| cgroup v2: making, delegating and moving into subtrees, and spawning into one; `cgroup.kill` | sandboxes' limits; the steward's and the builder's subtrees | M2 |
| generating Ed25519 and X25519 keys in-process | identity without tools from a slot | later |
| HTTP ranges and file-backed bodies | bytes by digest | M6 |

### 8.2 temper

- **The fake machine is temper's too** (temper's `domain/testing.md`, 3.2).
  hearth builds it first, under `testing/`. Its generic half (files,
  processes, programs on an argument-vector face) moves into skein as a
  kit when temper needs it; hearth's half (btrfs, namespaces, firmware,
  the package universe) stays as `hearth-fake-machine`
  (testing.md, 3.11).

### 8.3 The factory

- **An infrastructure connector** speaking hearth's wire, in jig's terms
  (jig's `connectors.md`), for M4.
- **A plane's secret store, and machine identity** toward it and toward
  the engine (wire.md, 5.3), for M4.

## 9. What is open

Each document lists its own. The ones that reach across:

- machine identity, toward the engine and the plane's secret store
  (wire.md);
- who posts the rendered diff at a change's head, and how (render.md);
- the checks' format, and what a check may do on a production machine
  (operating.md);
- signing kernels for Secure Boot (machine.md);
- firmware that discards its variables, as Vultr's does (machine.md);
- whether the activator hashes every output it places again, or trusts
  the builder's records where the builder runs only landed code
  (activator.md);
- one generation for the whole secret store, so any import prepares
  every target's slot again, or one per target (activator.md);
- whether a container runs a hearth of its own (slots.md, 12.5);
- live changes on a real machine: real ids, PID 1, D-Bus and AppArmor
  around a swap, overlays across soft reboots, `BootNext` kept for days
  (slots.md, section 15; testing.md, 7.3);
- hearth's own release applied live, once live changes are proven
  (slots.md, section 15);
- whether a manual machine applies live changes at once and waits only
  for reboots (wire.md);
- a blessed EFI default that later stops booting panic-loops, since the
  firmware falls through only when a load fails: keeping the previous
  slot first in `BootOrder` and arming `BootNext` at each healthy start
  is one answer (slots.md); BIOS has a boot counter (machine.md, 5);
- whether the activator should place only hearth binaries whose digests
  it holds itself (activator.md);
- the fake machine's final home (testing.md);
- the local mirror for the real tier: which packages, pinned how,
  fetched by whom (testing.md).
