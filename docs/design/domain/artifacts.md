# Artifacts

Provisional, 2026-10-09. hearth's second recipe (temper's `factory.md`,
section 7): an artifact made from a build environment, sources and a
command. It says what a recipe names, how its key is made, how an
artifact is kept and served, how the command runs, what compile caches
are and what trusting them costs, and how an IaC repository pins an
artifact. M6 builds it, in `hearth-domain-artifacts`. Section 9 lists
what is open.

## 1. In one page

- **An artifact is files made by a command, keyed by its inputs.** The
  inputs are a build environment, which is an ordinary rootfs output
  (`stoa-build-lab`); sources, which are paths of a git commit, by
  digest; declared inputs, by digest; a command; and the outputs it
  declares (section 2).
- **The key is the digest of those inputs** (section 3): the
  environment's identity (its resolution key and output part digests,
  not the closure a builder resolved), the sources' digest, the command,
  the declared inputs and outputs. A commit that changes nothing the recipe
  selects (a README) keeps the key. Until builds are reproducible, an
  artifact is built once per key and reused.
- **An artifact is a read-only subvolume named by its key,** recording
  its tree digest, exactly as a layer is (section 4): walked, pinned,
  collected and served to peers by the same mechanisms.
- **The command runs offline in a build step's sandbox whose root is the
  environment's output,** read-only, with the sources read-only beside
  it (section 5), as an unprivileged id with no capabilities, under
  limits. What it costs is reported as facts.
- **Compile caches are writable subvolumes per environment and
  project,** never part of the key (section 6). A new environment starts
  a new one. A warm cache's sources are patched from the ones it saw
  last, so tools that judge staleness by mtime rebuild exactly what
  changed.
- **A warm cache is trusted, not proven:** a tool's wrong judgement, or a
  build that poisons its cache, is invisible until builds reproduce. So
  unlanded heads get caches of their own, and builds are cold unless the
  order asks for warm: the lab asks, the run side does not (6.5).
- **IaC pins an artifact by key,** with the repository, commit and recipe
  that produce it, bumped by a change as a snapshot bump is (section 7).
  The key in the pin is a lock: render uses it as is, and the builder
  that builds it checks it. "Built from this head" means the revision
  being rendered: the software part names the recipe and its sources'
  listing, and the key resolved from them enters the software layer's
  key, as a closure's digest enters the packages key. The bash fleet's
  releases pinned by sha256 are the precedent (7.4).
- **Each plane builds its own copy of a key.** Outputs never cross
  planes, so the lab proves a build of the inputs production runs, not
  its bytes. Once builds are reproducible, the two planes' tree digests
  for a key are compared as facts (domain/reproducible.md, section 7).

## 2. The recipe

### 2.1 What a recipe names

```
recipe        a name, in a repository's artifacts.json
environment   a target, rendered at a commit                    → its identity (2.3)
sources       paths of this repository at the commit            → the sources' digest
inputs        named files by SHA-256: downloads, or artifacts   → each bound read-only
command       an argument vector, variables, a directory, a user
outputs       artifact path ← where the command left it
cache         a name, by default the repository's               (section 6; not in the key)
limits        memory, processes, time, disk                     (5.4; not in the key)
```

- **A recipe is data in the repository it builds,** read at a commit:
  product knowledge lives with the product (temper's `factory.md`,
  section 1). The same repository may hold the IaC of the product's
  clusters and of its build environment, as stoa's does.

### 2.2 The file

`artifacts.json` at the repository's root: strict JSON with `_note` for
annotations, as protocol/format.md (section 2) has it, and a map of recipes keyed
by name. stoa's:

```json
{
  "stoa": {
    "_note": "stoa's server: the binary its cluster runs",
    "environment": { "target": "stoa-build-lab" },
    "sources": ["Cargo.toml", "Cargo.lock", ".cargo", "crates", "vendor"],
    "command": ["cargo", "build", "--release", "--locked", "--offline", "--package", "stoa"],
    "variables": {
      "CARGO_HOME": "/cache/cargo",
      "CARGO_TARGET_DIR": "/cache/target"
    },
    "outputs": { "bin/stoa": "/cache/target/release/stoa" },
    "limits": { "memory_mib": 8192, "processes": 1024, "seconds": 1800, "output_mib": 512 }
  }
}
```

The protocol layer (`hearth-protocol-config`) parses it into a typed
recipe, refusing unknown keys and duplicates; the domain never sees the
text.

### 2.3 The environment

- **A build environment is a target,** with tasks, packages and a pinned
  Debian snapshot like any other. Its placement's kind is `container`
  with no host (no `on`, protocol/format.md, 3.4): it has no kernel and no
  bootloader, takes the containers' bootstrap variant, and is never
  prepared into a slot. Its output is used only as a sandbox's root
  (5.2), never run by nspawn as a host's containers are (domain/slots.md,
  section 12): no host's `target.containers` holds it, and no hearth
  ever activates it.
- **It is named by target,** in the recipe's repository at the recipe's
  commit by default: stoa's is `stoa-build-lab`, base `stoa-build` in
  environment `lab` (protocol/format.md, 3.1), which every plane builds. Another
  repository's is `{ "repository", "commit", "target" }`.
- **What enters the key is its identity,** what was asked for: its
  resolution key and its three output parts' digests (domain/render.md, 4.5).
  Not its output key, which takes the closure a builder resolved: two
  builders whose apt resolves one key differently, or a resolution made
  again after its record was collected, would name one artifact twice,
  refuse a correct pin (7.1) and never match across planes. The output
  key a build used is a fact of the artifact (4.3).
- **An environment may pin artifacts by key** (7.1), such as a toolchain
  built in the factory, but not "built from this head" (7.2). The plan
  bounds the depth of artifacts within environments (a constant, 4); a
  cycle cannot be written, since each key covers the keys beneath it.

### 2.4 The sources

- **Paths of a commit,** listed in `sources`: files or directories,
  relative to the repository's root. An empty list means the whole tree.
  What is not listed is not visible to the command: the sandbox binds
  exactly what was hashed, so the key misses no input.
- **Submodules are not followed** in M6: a gitlink in the selection is
  an error. Large-file pointers are content like any other.
- **Fetched as the IaC repository is,** by the builder's git, into a
  local clone kept as a cache (temper's `factory.md`, section 7).

### 2.5 Declared inputs

- **Files the command needs that are not in the sources:** a toolchain
  archive, a dataset, a dependency, another artifact. Each has a name, a
  SHA-256 and a source: a URL, as a software download has (protocol/format.md,
  section 10), or an artifact key. The fetch step, the only one with
  network (domain/build.md, section 8), fetches and verifies them into the
  store's content-addressed download cache; each is bound read-only at
  `/inputs/<name>`.
- **A project's own dependencies** are either committed (stoa's
  `vendor`, with cargo's source replacement in `.cargo`) or declared
  inputs. A lockfile that names its dependencies by digest (cargo's
  `Cargo.lock` carries a SHA-256 per crate) could be turned into
  declared inputs; whether hearth reads lockfiles is open.

### 2.6 The command

- **An argument vector, run as is** (no shell unless it names one), with
  variables added to the sandbox's fixed set (5.2), a working directory
  (`/src` or a path inside it) and a user (id 1000 by default, 5.2).
- **Writable sources,** for build systems that write into their source
  tree: `/src` is then a writable snapshot given to id 1000, discarded
  after the build.
  Such builds keep nothing between runs but what a content-keyed cache
  keeps (ccache in `/cache`). Out-of-tree builds (cargo's target
  directory, a meson or cmake build directory in `/cache`) are
  incremental (section 6).

### 2.7 Declared outputs

- **A map from a path in the artifact to where the command left it,**
  under `/out` or `/cache`, the two places the command may write: a
  file, or a directory taken whole.
- **A declared output that is missing fails the build.** Anything not
  declared is dropped and counted. What is taken is normalised (4.1).

## 3. Keys

### 3.1 The key

```
environment's identity E = H("environment", resolution key, packages part digest,
                             software part digest, config part digest)              2.3
sources' digest        S = the tree digest of the selected paths at the commit       3.2
inputs digest            = H("inputs",  each input sorted by name: name, SHA-256, where bound)
command digest           = H("command", argv, variables sorted by name, directory, user, writable sources)
outputs digest           = H("outputs", each output sorted by path: path, from)
artifact key             = H("artifact", E, S, command digest, inputs digest, outputs digest)
```

H is SHA-256 over the canonical encoding domain/store.md (section 3) fixes,
the same as for layer keys.

- **The environment's identity** brings the architecture, the snapshot
  and the requested toolchain, not the closure resolved: the bytes may
  depend on what the key does not name, as an output's do until builds
  are reproducible. **The recipe's name is not in the key:** two
  recipes that agree on everything make one artifact.
- **The commit is not in the key:** only what the commit holds at the
  selected paths is. A head that edits only the README keeps S, the key
  and every key above it (temper's `factory.md`, 8.5).

### 3.2 The sources' digest

- **A tree digest, as a layer's** (domain/store.md, section 3): paths, types,
  modes, owners, contents and link targets, as seen inside the builder's
  namespace, mtimes left out.
- **The export is made from the commit's objects, not by `git
  archive`.** `git ls-tree -r -z` of the selection gives each path's
  mode and object id; `git cat-file --batch` gives each blob; the
  builder turns them into a change set against the empty tree, which the
  tree helper applies to a fresh subvolume and walks (domain/store.md, 5.4 and
  5.5). git's modes become
  0644, 0755 and links; every owner is root. `git archive` would apply
  `.gitattributes` (`export-subst`, `export-ignore`, line-ending
  conversion, `ident`), so its tree could depend on the commit, not
  only on the blobs, and would not match a tree patched from blobs
  (6.3).
- **Remembered by the listing.** The SHA-256 of that listing, with the
  repository, names a memo entry whose value is S. The tree is a
  function of the listing alone, so a head whose selected paths are
  unchanged is a lookup, with no export. The memo is a shortcut only:
  git's object ids are SHA-1 (hardened against known collisions), and
  the key always rests on the SHA-256 tree digest.

### 3.3 What stays out of the key

The compile cache: a cold and a warm build of one key are meant to be
equivalent (6.5 says what that trusts). Limits: a build that hits one
fails, and makes no other artifact. The builder: its machine, its
subordinate id base, its store's path; digests use the ids seen inside
the namespace. Time: `SOURCE_DATE_EPOCH` is derived from the key (5.2).

### 3.4 Once per key

- **Without reproducibility, an artifact is built once per key and
  reused.** A builder that holds the key never builds it again; a peer
  of its plane fetches it instead of building (4.5).
- **Two builds of one key differ** for now: within a plane, when two
  builders build it before either learns of the other's; between
  planes, always. Each copy records its tree digest; the key is the
  identity, the tree digest is the fact, until builds reproduce (7.5).

## 4. Outputs and storage

### 4.1 Collecting the outputs

After the command exits 0, the builder takes the declared outputs:

1. the tree helper walks each declared path under `/out` or `/cache`;
2. it applies them, normalised, to a fresh subvolume, as a change set
   against the empty tree (domain/store.md, section 5): owner and group root,
   permission bits kept, setuid, setgid and sticky bits cleared,
   extended attributes dropped; a device, fifo or socket fails the
   build; links are kept as links; hard links become separate files;
3. the subvolume is frozen read-only and walked for its tree digest;
4. its record is written, and only then does the artifact count.

Contents are cloned (`FICLONE`), so a large binary costs metadata only;
that the tree helper's apply clones is to verify (domain/store.md, section 5).

### 4.2 A read-only subvolume, not an archive

**Decided: an artifact is a read-only subvolume named by its key,
recording its tree digest,** as a layer is. A content-addressed archive
(a canonical tar named by its SHA-256) was the alternative.

- **One mechanism.** Layers are subvolumes with tree digests, change
  sets, records, pins and collection. An artifact reuses every one of
  them; an archive would be a second kind of object with its own
  digest, its own transfer and its own collection.
- **The digest does not depend on a format.** A tree digest is a
  function of the tree; an archive's is a function of how it was packed
  (header fields, ordering, the packer's version). One tree could carry
  two digests, and a comparison of builds would compare packers too.
- **It is consumed without unpacking.** A subvolume binds read-only into
  a sandbox (an artifact as another artifact's input, or as a toolchain
  in an environment) and is placed into a software layer by cloning
  (7.3). An archive is unpacked every time, a copy of its data.
- **Named by key, as layers are.** A builder holds one artifact per key,
  whichever build it came from; the tree digest is the fact a pin's
  consumer records (7.3).
- **What an archive would give,** one file any cache serves and a
  publication outside hearth, is the release path (7.4), and open.

### 4.3 The record

Beside the subvolume (domain/store.md, section 6):

```
artifact <key>
  tree digest     D
  environment     identity E; the output key and tree digest of the copy built in, a fact
  sources         S; repository, commit first seen, selection
  inputs          name → SHA-256
  command         argv, variables, directory, user
  outputs         path → kind, size; dropped: count, bytes
  cache           cold, or warm from sources S′; the cache's name and class (section 6)
  built           this builder or a peer; when; exit status
  costs           section 5.6
```

The output key says which closure the environment resolved to on this
builder, and its tree digest which copy of it the artifact was built
in, since two builds of one key differ for now.

### 4.4 Pins and collection

- **The store's pins apply** (domain/store.md, section 7). An artifact is
  pinned while a spec in the pins the steward gives its builder names it
  (its desired revision's, or an order's; domain/steward.md, 3.2), while a
  layer is being built from it, and for a window after its last use.
  Then it is collected like a layer.
- **A software layer holds its own copy** (7.3), so collecting an
  artifact never breaks an output. Sources' subvolumes and the memo of
  listings (3.2, 6.3) follow the same rules.

### 4.5 Served by digest

- **Within a plane only** (protocol/wire.md, section 8). A builder asks a peer for
  a key: first its record, then its tree as a change set against the
  empty tree. The receiver applies it to a fresh subvolume, walks it,
  and keeps it only if its tree digest is the one expected (below). A
  layer moves the same way, against its parent.
- **The receiver takes the expected tree digest from its own engine,**
  as it does for layers (domain/store.md, 10.4), never from the peer's record.
  It does not rebuild to check; once builds are reproducible, it can.

## 5. Running the command

### 5.1 The plan

```
requested ─→ resolving ─┬─→ present                       the key is held, or fetched from a peer
                        └─→ fetching ─→ waiting ─→ preparing ─→ running ─→ collecting ─→ built
                            inputs,     for the    the env's    the        4.1, then
                            by digest   cache      output, the  sandbox    the record
                                                   sources, /out
any state ─→ failed (typed, 5.5)          any state ─→ cancelled (the order withdrawn)
```

- **Resolving** finds the key. A pin gives it, and a key already held
  ends here. Otherwise the builder reads the recipe at the commit,
  renders the environment for its identity, finds S (the memo, or an
  export) and computes the key, which must equal the pin's (7.1).
- **Two orders for one key join one build** (a keyed effect, jig's
  `connectors.md`, 4.3): the steward holds the orders, and gives its
  builder one job per key (domain/steward.md, 3.3). **Preparing** builds the
  environment's output if it is missing (domain/build.md, section 4) and
  patches the sources for the cache (6.3).
- **After a restart,** a builder's loss included (domain/steward.md, 5.1), a
  build that had not written its record never happened: its `/out` and
  sandbox are removed, its cache is emptied (6.4), and an order still
  standing starts it again. The filesystem is the only state (domain/store.md,
  section 8).

`hearth-domain-artifacts` steps this machine: a child of
`hearth-domain-builder` sharing no types with its siblings, whose
requests for renders, layer builds, store operations and sandboxes the
root translates (`programming-model.md`, 4.5).

### 5.2 The sandbox

A build step's sandbox, as domain/build.md (section 8) defines it: its
namespaces (8.1), the network's included, its `/proc`, `/sys` and `/dev`
(8.4), its cgroup, limits and stop in three steps (8.7). What differs
for an artifact:

```
/                the environment's output     read-only bind of its subvolume
/src             the sources                  read-only (a writable snapshot, id 1000's, if asked, 2.6)
/inputs/<name>   each declared input          read-only
/cache           the compile cache            read-write (section 6)
/out             a fresh subvolume            read-write
/tmp, /var/tmp   tmpfs                        bounded, charged to the memory limit
```

- **The root is the environment's output, read-only,** not a layer
  being built: the command must leave its environment as the next build
  will find it.
- **The command runs as id 1000, not as the namespace's root** (domain/build.md,
  8.1, runs a step as root, since dpkg changes owners), with an empty
  capability bounding set besides `no_new_privs`. As root it could
  remount its read-only binds, or clear the read-only flag of the
  environment's subvolume, which its namespace owns
  (`BTRFS_IOC_SUBVOL_SETFLAGS` checks only ownership). As 1000 it owns
  only `/out`, `/cache` and its `/tmp`. The environment may name id 1000
  in its passwd (a sysusers line of its own tasks) for tools that look
  it up.
- **Its own environment, given whole** as a step's is (domain/build.md, 8.6):
  `PATH=/usr/local/bin:/usr/bin:/bin`, `HOME=/tmp/home`,
  `LC_ALL=C.UTF-8`, `TZ=UTC`, umask 022, and `SOURCE_DATE_EPOCH` set to
  the environment's Debian snapshot timestamp, which its key holds
  (domain/reproducible.md, 5.1). Setting the variable costs nothing, so
  artifacts have it from M6. The recipe's variables are added; it may
  not set `SOURCE_DATE_EPOCH`.

### 5.3 No network

- **Loopback only,** as for every build step (domain/build.md, 8.5): a test
  suite that binds `127.0.0.1` works; a fetch does not.
- **The caches are files, not services.** What the command would fetch
  reaches it as declared inputs (2.5), and what it keeps between runs
  lives in `/cache` (section 6). A cache reached over a socket (an
  sccache server, a registry mirror) is not given in M6; one bound into
  the sandbox as a Unix socket would be the way, and is open.

### 5.4 Limits

| Limit | Mechanism | When exceeded |
|---|---|---|
| memory, tmpfs included | the cgroup's `memory.max` | killed: `memory` |
| processes | `pids.max` | forks fail; the command's own failure follows |
| time | a deadline the domain arms (`programming-model.md`, section 9) | stopped: `time` |
| `/tmp` | tmpfs `size=` | writes fail with ENOSPC |
| outputs | the declared outputs' size, at collection | failed: `output too large` |
| the store's disk | the store's free-space floor (domain/store.md, 7.2), checked with `statfs` while the command runs | stopped: `disk` |
| concurrent builds | the steward's lane for others' work, as many as the builder's limits allow (domain/steward.md, 3.4; domain/build.md, 4.4) | the order waits, or is refused when the queue is full |

- **Limits are the recipe's,** within the builder's ceilings, in place
  of a step's defaults (domain/build.md, 8.7); a recipe that asks for more is
  refused before anything runs.
- **No per-build disk quota:** btrfs quota groups need root to enable
  (to verify for unprivileged use), so `/out` and `/cache` share the
  store's space, bounded by the floor and the caches' caps (6.4). **No
  progress deadline,** unlike a build step's (domain/build.md, 8.7): a link
  step may be silent for minutes. The steward's deadline for the job
  (domain/steward.md, 5.2) is the recipe's time and a margin.

### 5.5 Failures

Typed, and reported as an event and a fact under the key: the command
exited non-zero (its status, and the last 64 KiB of its output as opaque
bytes); stopped by a limit (which); a declared output missing, or not a
file, a directory or a link; an input's digest wrong; the pin's key not
its recipe's (7.1, both keys); the environment's build failed
(domain/build.md's failure, nested); the sources not fetched or exported; the
builder lost twice, `crashed` (domain/steward.md, 5.1).

- **A failure is not a verdict:** a new order builds again.
- **The command's output** goes to a file beside the record, capped (the
  first and last 4 MiB kept), served on demand as telemetry is
  (domain/operating.md, section 5). The domain never reads it.

### 5.6 Costs as facts

Every build records, and reports as facts (protocol/wire.md, section 6):

- wall time per state (5.1); the command's CPU time, user and system
  (`cpu.stat`); its peak memory (`memory.peak`, Linux 5.19 and later,
  to verify on trixie's 6.12);
- bytes fetched; files changed in the sources (6.3); the cache, cold or
  warm, its size before and after, whether it was emptied;
- output files and bytes, dropped files; hits, by a key held or a peer.

### 5.7 Who asks

- **The build plan of a target** whose software pins an artifact
  (section 7): its software layer waits for the artifact.
- **An engine,** over M4's wire, with one order and one fact M6 adds: an
  order "build this artifact", keyed by its key, or by repository,
  commit and recipe; and a fact "key → tree digest, or failed" (protocol/wire.md,
  sections 5.4 and 6): the one effect and the one fact the lab's
  connector needs (temper's `factory.md`, section 7). The steward takes
  the order and runs it as its builder's job (domain/steward.md, 3.3).
- **The client:** resolve a recipe at a commit to its key and pin
  (7.1), and build it locally (client/README.md, section 3).

## 6. Compile caches

### 6.1 A volume per environment and project

- **A writable subvolume in the store,** named by the output key of the
  environment it builds in (another closure is another toolchain), the
  recipe's cache name (by default its repository's, so
  recipes of one workspace share it) and a trust class (6.5). It is
  bound at `/cache`, owned by id 1000, and kept between builds; the
  recipe's variables point the tools at it (`CARGO_TARGET_DIR`,
  `CCACHE_DIR`, a meson build directory). hearth knows no compiler. It
  holds no secret: the sandbox has none to give.

### 6.2 Invalidated by the environment's digest

- **A new environment output key is a new cache,** cold. A toolchain
  bump changes the environment's identity and output key, so it starts
  afresh, as it changes every artifact key built with it (temper's
  `factory.md`, section 7).
- **The cache records the environment's tree digest** it was filled
  against. If the builder's copy of the environment changes under the
  same key (built again after collection, or fetched from a peer), its
  tree digest differs (for now), and the cache is emptied before the
  next build.
- **Never carried across environments,** not even as a seed: a tool
  that does not check its compiler's version would link another
  toolchain's objects.

### 6.3 Sources and mtimes

Incremental tools judge staleness by mtime: cargo compares a local
crate's files with the dep-info of its last build (a checksum mode,
`-Z checksum-freshness`, is still unstable in cargo 1.95), and make and
ninja compare files with their targets. Reproduced with cargo 1.95: a
changed file whose mtime is older than the last build is reported
`Fresh`, and the stale object is linked.

- **A fresh export would be safe and slow:** every file newer than every
  object, so every local crate rebuilds. Fixed mtimes would be fast and
  wrong: a changed file would look old.
- **So a warm cache's sources are patched from the sources it saw
  last.** The cache keeps, beside its data, the read-only subvolume of
  the sources its last build saw, with their commit. For the next build,
  the builder snapshots it, applies the difference between the two
  commits' listings of the selection (changed blobs written anew from
  `git cat-file`, deleted paths removed), walks the result, which must have the tree
  digest S, and freezes it. Files that changed are newer than every
  object; the others keep the mtime the cache last saw. This holds in
  both directions: a bisect that goes back rewrites the files that
  differ too. A cold cache starts from an export (3.2).
- **Sources are therefore per cache:** the same S in two caches carries
  different mtimes. Snapshots make this cheap.

### 6.4 One build at a time, bounded

- **One holder per cache:** a second build waits, and the wait is a cost.
- **A cap per cache,** from the builder's configuration: a cache over it
  after a build is emptied whole, since tools' own cleaners are
  tool-specific and a cold build is always correct.
- **After a failure of the command, the cache is kept:** incremental
  tools are built for failed builds. **After a stop** (a limit, a
  cancel, a crash of the builder) **it is emptied:** a tool stopped
  mid-write may leave state it cannot judge.
- **Collected** when unused for a period, within a total cap for all
  caches; a cache whose environment key no spec names any more goes
  first.

### 6.5 Never in the key: the trust it implies

The key names the inputs. A warm build's bytes also depend on the
cache's history: which builds ran in it, in which order, and whether
each tool judged staleness right. Until builds are reproducible, nothing
shows the difference.

- **A tool's mistake is silent.** A wrong freshness judgement gives an
  artifact that does not match its sources, under a key that says it
  does, and looks like any other build. 6.3 removes the known mtime
  trap, not a tool's bugs.
- **A build can poison its cache:** a command writes what it likes into
  `/cache`, and a later build links it. In the lab, that command may
  come from a head nobody has reviewed.
- **So the trust class is part of a cache's name.** A head not yet
  landed builds in a cache of its own class; a landed revision never
  reads it. The engine's order says which class a commit is in; in
  standalone mode every commit is landed.
- **And builds are cold unless the order asks for warm.** A cold build
  is the same key with an empty cache. The lab asks for warm, where
  speed matters and its verdicts are about its own builds. The run side
  does not, and a standalone host builds cold: production's artifacts
  never depend on a cache's history, and outputs never cross planes.
- **Once builds are reproducible,** a warm and a cold build of one key
  must have one tree digest. The lab's gate checks it (domain/reproducible.md,
  7.1), and a cache's mistake becomes a finding instead of a silent
  binary.

## 7. Pinning artifacts into IaC

### 7.1 By key, pinned by a change

A software entry names the artifact, and where its tree goes (protocol/format.md,
section 10, shows the entry's shape):

```json
"software": {
  "stoa": {
    "artifact": {
      "repository": "https://git.ekanayaka.io/ai/stoa.git",
      "commit": "4e1d0c3a…",
      "recipe": "stoa",
      "key": "9f2c71e0…"
    },
    "path": "/opt/stoa"
  }
}
```

- **An entry has either `url` and `sha256`, or `artifact`.** `path`,
  `owner` and `group` mean what they mean for a download (protocol/format.md,
  10.1); `unpack`, `strip` and `mode` do not apply, since an artifact
  is a tree.
- **The key is the artifact's digest in the IaC,** and the software
  part's digest covers it (domain/render.md, section 4). Render uses it as it
  stands: it needs neither the product's repository nor its build
  environment, so it stays local and fast on every machine.
- **The repository, commit and recipe say how to make it.** The key is a
  function of them alone: the environment is named by the recipe, at the
  recipe's commit (2.3), never by the consumer's IaC. So a pin never goes
  stale by itself.
- **The key is a lock.** A builder that builds the artifact resolves the
  recipe and refuses a key that does not match it (5.5); the lab's gate,
  which builds every changed target, catches a wrong pin before it
  lands. A builder that already holds the key uses it. Since the key
  takes the environment's identity, not its closure (2.3), every
  builder computes the same key from the same pin.
- **Bumped by a change, as a snapshot bump is.** An agent or a bot
  updates commit and key together; the client prints the pin for a
  commit (5.7). The change carries its rendered diff (domain/render.md,
  section 6): "software of stoa-1-prod: stoa 9f2c… → 31ab… (commit
  4e1d… → 77c0…)", and lands through the usual gates.

### 7.2 Built from this head

```json
"artifact": { "recipe": "stoa" }
```

- **"This head" is the revision being rendered.** With no repository and
  no commit, the recipe is read from the repository being rendered, at
  the same revision. This is temper's `factory.md` (8.5) case: an
  experiment and its cluster are data in the product's repository, so
  the cluster at head h names stoa built from h.
- **The software part holds the recipe and its sources' listing, never
  a resolved key:** the recipe's name and the digest of `git ls-tree -r
  -z` over its selection (3.2), both read from the revision. Render
  stays a pure function of the revision and the target.
- **The resolved key enters the software layer's key,** as a closure's
  digest enters the packages key (domain/render.md, 4.5; domain/store.md, 3.3). The
  plan resolves each head entry before the software layer (5.1): the
  recipe, the listing's S, the environment's identity at the same
  revision.
- **So the specs do not show what only the resolution sees.** A
  toolchain bump in `stoa-build-lab` changes the resolved key, and the
  software key, of every head entry built with it, with the consumers'
  parts unchanged; the bump's rendered diff shows it on the
  environment's target, and the lab's experiment key names the artifact
  beside the specs (temper's `factory.md`, 8.1).
- **Which environments use it is data.** A lab variant can override a
  base's pin with this form, through merging (protocol/format.md, section 4): the
  lab builds heads, production runs pins.
- **When the IaC is in another repository than the product,** there is
  no "this head": a pin is bumped by a change (7.1), including an
  unlanded one the lab makes to test a product head.

### 7.3 Placed in the software layer

- **The tree helper applies the artifact's tree under the entry's
  `path`,** as it applies a single download's put (domain/build.md, 6.2;
  domain/store.md, 5.5): no tool runs, owners are root unless declared, and
  contents are cloned from the artifact's subvolume.
- **The software layer's record names the artifact's tree digest,** and
  the plan (domain/build.md, section 4) resolves, fetches or builds each
  artifact before the layer.

### 7.4 Today's precedent: releases pinned by sha256

The bash fleet already pins software built outside Debian by digest:

- `tasks/agent/software:1` pins `pi 0.74.0`, a tarball among the forge's
  releases (`infra/ekanayaka.io-software`), by sha256 `5a403c6e…`;
  `tasks/hermes/software:1` pins `hermes 2026.4.30` the same way.
- `lib/build-pi-tarball` (`npm install` with the network, then `tar
  czf`) and `lib/build-hermes-tarball` (a clone and pip) make them, run
  by hand on a machine of the fleet.
- **What carries over:** pinning by digest, bumps as reviewed changes,
  a cache keyed by digest. **What a recipe adds:** how the bytes were
  made, offline, in a known environment, under a key.
- **Downloads stay** (protocol/format.md, 10.1), for software built outside the
  factory, and for bytes identical in both planes before builds
  reproduce.

### 7.5 Across planes

- **Why a key, not a tree digest.** Each plane builds its own copy of a
  key (3.4). A tree digest would name bytes only the lab's builders
  hold: the run side would have to fetch them from the lab, which the
  planes forbid, or build its own, which would not match. So the lab
  proves a build of the inputs production runs, not its bytes: the gap
  `draft/brainstorm.md` (11.3) accepts.
- **Once builds are reproducible,** any builder's copy can be checked by
  building it again, and each plane reports its tree digest per key as a
  fact for the factory to compare: equal digests mean production runs
  the bytes the lab proved (domain/reproducible.md, 7.6).

### 7.6 A head of stoa

temper's `factory.md` (8.5), in hearth's terms. The experiment and the
cluster's members `stoa-1-lab`, `stoa-2-lab` and `stoa-3-lab` are data
in stoa's repository, with the environment `stoa-build-lab`; the lab
variant's software entry is `{ "recipe": "stoa" }`.

```
head h: crates/raft/src/raft.rs changed; its change's gate asks for lab/chaos-smoke
 │
 ├─ 1. render stoa-1-lab, stoa-2-lab, stoa-3-lab at h: the software part names recipe stoa
 │     and the listing of Cargo.toml, Cargo.lock, .cargo, crates, vendor: L_h, new.
 │     The packages part is unchanged
 ├─ 2. resolve the head entry: L_h is not in the memo → objects to a tree, walk → S_h;
 │     stoa-build-lab renders to identity E, unchanged since h−1
 │     → K_h = H("artifact", E, S_h, …). Held? no
 ├─ 3. build K_h, warm, in the cache (stoa-build-lab's output, stoa, unlanded): its
 │     sources patched from h−1's, so raft.rs alone is newer; cargo rebuilds the raft
 │     crate and what depends on it, and links → K_h, tree digest D_h; 41 s, cache 3.2 GiB
 ├─ 4. software key = H("software", packages key, software part, K_h): the output is the
 │     cached packages layer with the software layer rebuilt (one binary swapped) and
 │     the config layer replayed; each member's slot is prepared from it
 └─ 5. chaos-smoke, seeds 1–3 → evidence under the specs' digests and K_h → status at h
```

A later head h′ that edits only the README has the same listing L_h:
the parts, the specs, the memo's S_h, K_h and the experiment's key are
unchanged, and the lab posts h′'s status by lookup, with nothing built
and nothing run. Landing h changes nothing in production: the prod
members' pin moves only when a change bumps it (7.1), and the run side
then builds the new key cold, from the same inputs, on its own builders.

## 8. Its world

`hearth-artifacts-world`, a domain world of `hearth-domain-artifacts`
under `testing-strategy.md` (2.2), with its parent scripted, and a
system world with render, store and build beside it (2.3).

- **The fakes act on the fake machine** (domain/testing.md, sections 3 and 4):
  git with repositories, commits, listings and blobs; the tree helper's
  walk and apply; resolutions that give two builders different closures
  for one key; peers serving records and trees, sometimes wrong.
- **A fake compiler is the command.** It reads `/src` and its mtimes,
  keeps one object per source file in `/cache` with the mtime it saw,
  rebuilds an object only when its file is newer, as cargo does, and
  writes into its output the digest of the sources its objects came
  from. The referee checks that this digest equals S. With 6.3's rule
  the check holds on every seed; with fresh exports given fixed mtimes,
  it fails, which is the world's proof of the rule.
- **Faults drawn from the seed:** the command fails, runs past its
  deadline, exceeds memory, writes outside its outputs, leaves an output
  missing, fills the disk; an input's digest is wrong; a peer serves a
  tree that does not match its record; the power is cut in every state
  of 5.1.
- **Invariants,** checked as the world settles: one build per key at a
  time, and none for a key already held; one terminal event per order;
  an artifact counts only with its record, and matches it; one holder
  per cache, whose recorded sources match its sources' subvolume; no
  cache outlives a change of its environment's tree digest; nothing
  built for an unlanded commit is read by a landed build; no pinned key
  that mismatches its recipe is built; one pin gives one key on builders
  whose closures differ; render's software part holds no resolved key;
  every build records its costs.
- **The real tier** (domain/testing.md, section 6) builds a small C project in
  an environment output from the local mirror, as a regular user, cold
  then warm, and checks that a one-file change rebuilds one object. A
  Rust case checks 6.3 against cargo itself; it is heavier (the
  toolchain's packages) and runs in the slower suite. Nothing here needs
  root, so nothing runs in VMs.

## 9. What is open

- **Credentials for private repositories:** it depends on protocol/wire.md (5.3)
  and the plane's secret store.
- **Lockfiles as declared inputs:** whether hearth reads cargo's
  `Cargo.lock` (SHA-256 per crate), or recipes list their inputs; npm's
  lockfile gives SHA-512. It depends on which projects come first.
- **Caches served over a socket** (sccache, a registry mirror) bound into
  the sandbox: none in M6; it depends on measured cold-build costs.
- **Per-build disk quotas:** whether btrfs quota groups can be used
  unprivileged, or the free-space floor stays the only bound.
- **Publishing an artifact outside hearth:** a canonical archive as a
  forge release, so both planes pin identical bytes as a download before
  builds reproduce; it depends on which plane builds the release.
- **"Built from this head" beyond the lab** (7.2): whether a landed
  revision may build its own product for its run-side targets, which is
  the run side's policy; and, for IaC in another repository, who makes
  the unlanded pin change that tests a product head.
- **Signing artifacts** (and kernels for Secure Boot, domain/machine.md,
  section 9): a signing key is a secret, and the sandbox holds none.
- **6.3's rule and its alternatives:** whether an in-tree build's
  working tree should persist in its cache (a tree to reconcile, not
  sources to patch); and, once cargo's checksum freshness is stable, a
  plain export for cargo. Other mtime-based tools still need the rule.
- **Artifacts on small standalone hosts,** which build the environment
  and the artifact themselves until peers (domain/store.md, section 10).
