# Reproducible builds

Provisional, 2026-10-09. Reproducible builds for hearth: why two builds
of one key differ today, what is kept from day one so that they can be
retrofitted, what is given up meanwhile, how the retrofit goes step by
step, what Debian proves already, and the lab's gate that proves the
rest. It takes `draft/brainstorm.md` (section 11) further and corrects
it where research found otherwise. No milestone builds it yet: it comes
after M6 (README.md, section 3: "later"). Section 8 lists what is open.

## 1. In one page

- **Reproducible means one tree digest per key, on any builder.** Not
  the same bytes on disk: btrfs's inode numbers, UUIDs, generations and
  extents never match. Two builds of one key reproduce when their tree
  digests (domain/store.md, section 3) are equal.
- **Deferred, because it is trust, not function.** Nothing in the
  store's naming, the wire or the slots changes when it comes. Every key
  changes once, with the encoding's version (5.4).
- **Three things are kept from day one** (section 3), because sharing,
  drift and peers need them anyway: every output records its tree
  digest, which leaves mtimes out; nothing unique and nothing secret in
  an output; outputs trusted per plane.
- **Meanwhile, the lab proves a build of the inputs production runs, not
  its bytes** (section 4). Patch and scratch compare noisily, the
  initramfs is judged by its inputs rather than its bytes, and warm
  compile caches are trusted, not checked.
- **`SOURCE_DATE_EPOCH` alone does not do it.** mmdebstrap uses it to
  clamp tar output only; for directory output its code says `# nothing
  to do`. hearth clamps mtimes itself, per layer, to a time derived from
  the layer's key (5.1).
- **Retrofitting is four steps** (section 5): the time per layer and the
  clamp; sealed steps; a list of normalised files kept as data, each
  with its reason and its Debian bug; then the gate.
- **Debian proves the machinery, not the packages hearth installs**
  (section 6). Its live images are rebuilt and compared; the dpkg,
  debconf, triggers and byte-compilation every installation runs are
  covered by that. A mail server's postinst is not.
- **The lab's gate builds each new key twice, from scratch, on two
  builders,** varying what should not matter (section 7). diffoscope's
  report on the differing files is the finding. A layer that cannot be
  fixed degrades alone, to "built, not reproduced", and the layers above
  it are still judged on an identical parent.
- **The payoff is comparison.** Once a key reproduces, the lab's and
  the run side's tree digests for it must be equal, and comparing two
  facts proves that production runs the bytes the lab tested.

## 2. Why builds differ

### 2.1 What "the same" means

- **The tree, not the image.** The tree digest covers paths, types,
  modes, owners, extended attributes, contents and link targets, as seen
  inside the builder's namespace. It leaves out mtimes, ctimes, inode
  numbers and everything btrfs keeps below the tree.
- **Ids as seen inside the namespace,** so builders with different
  subordinate id bases agree (README.md, section 7). Capabilities read as
  v2 from inside, whatever rootid the disk holds.
- **Per layer.** Each layer's tree digest is compared on its own, with
  its parent identical on both sides (7.1).

### 2.2 The catalogue

Each source of difference, where it enters, and its fix. "Day one" means
domain/build.md already removes it for nothing unique and nothing secret
(domain/build.md, section 9); the other fixes are the retrofit's (section 5).

| # | Source | Where it enters | Fix | When |
|---|---|---|---|---|
| 1 | **mtimes of files a build writes** | every file a step creates gets the build's clock; packaged files keep their `.deb`'s | left out of the tree digest; clamped to the layer's time | day one; 5.1 |
| 2 | **the clock in contents** | tools that write today's date: shadow's last-change day in `/etc/shadow` (useradd, systemd-sysusers), generated headers | `SOURCE_DATE_EPOCH`, where the tool reads it | 5.1 |
| 3 | **mtimes copied into contents** | a `.pyc` records its source's mtime; an archive made during a build records its members' | `SOURCE_DATE_EPOCH` makes `py_compile` write hash-based `.pyc` files; the clamp covers the rest | 5.1 |
| 4 | **the initramfs** | mktirfs packs its staging tree with `cpio -H newc`, whose headers carry each file's inode number and mtime | patch mktirfs: clamp the staging tree, pass `cpio --reproducible` | 5.3 |
| 5 | **the build host** | mmdebstrap copies the host's `/etc/resolv.conf` and `/etc/hostname` (in hearth, the sandbox's); os wrote 8 KiB of the host's randomness as a regular file `/dev/urandom` | removed by a customize hook (`rm -f`, domain/build.md 5.6), the config layer writing the target's own; no fake `/dev/urandom` (the sandbox binds the real one) | day one |
| 6 | **identity made at install** | `/etc/machine-id`, `/var/lib/dbus/machine-id`, SSH host keys (openssh-server's postinst), the snakeoil key and certificate (ssl-cert's postinst) | emptied or removed; the activator writes the machine's own at prepare (domain/slots.md, 5.4) | day one |
| 7 | **logs and caches** | dpkg's, apt's and alternatives' logs, ldconfig's `aux-cache`, apt's lists and binary caches, mmdebstrap's setup files, `/run`, `/tmp`; logs its list misses (fontconfig's) | mmdebstrap's cleanup, all of it, and hearth's customize hook (2.3) | day one |
| 8 | **etckeeper** | os commits `/etc` with timestamps (`pristine-debian`, `Manifest config <hash>`) | gone: hearth has no etckeeper, and the scan refuses `/etc/.git` (domain/build.md, 9.2) | day one |
| 9 | **arbitrary shell** | maintainer scripts, `customize.sh`, `software.sh` read the clock, the hostname, the processor count, randomness | sealed steps; the gate finds the rest | 5.2; 7 |
| 10 | **directory order** | btrfs lists a directory in the order its entries were created (to verify); a tool that does not sort writes in that order | the tree digest sorts by path; the gate varies the order | 7.2 |
| 11 | **parallelism** | a tool whose output depends on how many processors it sees (multi-threaded compressors split their output into blocks) | the gate varies the count; a finding per tool | 7.2 |
| 12 | **ids allocated in order** | system users made by postinst take the next free id | the same if apt's install order is the same for one closure (to verify) | 7 |
| 13 | **the filesystem** | inode numbers, UUIDs, generations, extents, ctimes, otimes | never compared (2.1) | day one |

Notes on the rows that research corrected or confirmed:

- **Row 1.** mmdebstrap's `SOURCE_DATE_EPOCH` feeds its tar packer
  (`--mtime=@$SDE --clamp-mtime --sort=name --numeric-owner`) and seeds
  ext4 UUIDs; for directory and null output its code says `# nothing to
  do`. hearth's output is a directory, so mmdebstrap clamps nothing for
  it. Packaged files keep their own mtimes, set by dpkg from the
  package; those are older than the snapshot that holds the package.
- **Row 2.** systemd-sysusers (261) and shadow's useradd (4.19) both
  carry `SOURCE_DATE_EPOCH` handling, seen in their binaries on sid;
  trixie's 257 and 4.17.4 are to verify.
- **Row 3.** With `SOURCE_DATE_EPOCH` set, Python's `py_compile`
  defaults to `CHECKED_HASH` (PEP 552; reproduced with Python 3.14), and
  Debian's `py3compile` runs `python3 -m py_compile` (read on sid). That
  the variable reaches maintainer scripts through mmdebstrap is to
  verify.
- **Row 4.** Read in mktirfs (tiny-initramfs 0.1-8): `find . ! -name .
  -print0 | sort -z | cpio --null -o --quiet -R 0:0 -H newc | gzip -9`.
  Names are sorted and owners zero, and gzip reading a pipe stores no
  time (reproduced: MTIME 0). But GNU cpio 2.15 writes the file's inode
  number into the newc header (reproduced), and the staging tree's
  mtimes are the build's. cpio's `--reproducible` (`--ignore-devno
  --renumber-inodes`) fixes the first; clamping the staging tree, the
  second. It costs no reboot, since the initramfs is judged by its
  inputs (domain/build.md, 5.8), but two builds of one key differ in it, so the
  gate and every comparison of outputs meet it.
- **Containers [M5].** A container's output has no kernel and no
  initramfs (protocol/format.md, 3.4), so row 4 never reaches it, nor does
  section 4's initramfs judged by its inputs. Whatever its variant
  (protocol/format.md, section 13), it installs exactly its closure (domain/build.md,
  5.7) and meets the other rows as a host's output does.
- **Row 6.** mmdebstrap's `cleanup/reproducible` writes an empty
  `/etc/machine-id` if one exists and removes dbus's. An empty file
  means "not a first boot" (machine-id(5)): systemd then mounts a
  transient id and commits it only to a writable `/etc`, so on a
  read-only root the id would change at every boot. The output keeps the
  empty file, which is the same everywhere, and the activator writes the
  machine's id into the slot at prepare (README.md, section 7).

### 2.3 Cleanup's choices

- **hearth runs all of mmdebstrap's cleanup, and a customize hook of its
  own** (domain/build.md, 5.6, which lists what each removes). os skipped all
  of it (`--skip=cleanup`); the bash fleet did so on purpose, to keep a
  fixed machine-id, which hearth writes at prepare instead.
- **For reproducibility, `cleanup/reproducible` is the part that
  matters:** the logs carry the build's times, and ldconfig's
  `aux-cache` its inode numbers and times.
- **`resolv.conf` and `hostname`:** mmdebstrap's manual says cleanup
  keeps the host's copies, and suggests deleting them or filling them
  with fixed contents. Either is reproducible, and in hearth the copies
  are the sandbox's anyway (an empty file and `localhost`). domain/build.md
  removes them, and the config layer writes each target's own.

### 2.4 What os added

os's builds differ for reasons of its own, none of which hearth keeps:
the fake `/dev/urandom` with the host's randomness, never removed;
etckeeper's timestamped commits in `/etc/.git`; `--skip=cleanup`, which
keeps the logs, apt's caches, its proxy configuration and whatever
machine-id the bootstrap made; and `rsync -rltD`, which gives every
config file the spec directory's mtime, that is the time of the render.
The bash fleet's esh templates could also call programs at render
(`systemd-detect-virt`); hearth's render is pure (domain/render.md).

## 3. Kept from day one

None of these is about trust. Each is needed for sharing, drift or
peers, and each is what makes the retrofit cheap.

### 3.1 The tree digest

- **Every output records its tree digest next to its key** (domain/store.md,
  sections 3 and 6). Drift needs it, peers verify by it, and the
  activator checks what it places against it.
- **Its encoding is fixed once** and never changes meaning (domain/store.md,
  section 3): sorted by path, ids inside the namespace, mtimes out.
  Retrofitting is then comparing two digests already recorded, not
  changing how the store names outputs.
- **The walk already lists each file's digest** (domain/store.md, section 5),
  so two records of one key say which paths differ without reading a
  file again (7.3).

### 3.2 Nothing unique, nothing secret

- **Shared outputs would share identities** otherwise (domain/build.md,
  section 9). So host keys, machine-ids, snakeoil keys and every secret
  are kept out of outputs from the first build, and written into the
  slot at prepare.
- **These are also the largest irreproducible files.** A generated key
  differs on every build; removing it for sharing removes it for
  reproducibility too.

### 3.3 Trust per plane

- **A machine fetches only from its own plane's builders,** so the lab's
  outputs never reach production (README.md, section 5, "plane"). The run
  side builds its own copy of every key, artifacts included
  (domain/artifacts.md, 7.5).
- **This holds whether or not builds reproduce.** Reproducibility adds a
  check across planes; it never replaces the split.

### 3.4 Inputs fixed for good

- **Every input is pinned:** a Debian snapshot's timestamp, packages by
  the SHA-256 that `Packages` gives, downloads by digest, an artifact's
  sources by commit. Build steps run offline, and packages are installed
  from a local repository made from the content-addressed cache (domain/build.md,
  section 8). The network is not an input.
- **Rebuilding a key later builds the same inputs,** which Debian's own
  live-image rebuilds cannot assume (6.4).

## 4. Given up meanwhile

- **The lab tests a build of the same inputs, not the bytes production
  runs.** Each plane builds its own copy of a key, and two copies differ
  for now. The lab's verdicts are about its copy.
- **Patch and scratch compare noisily** (temper's `factory.md`, 6.4).
  Two builds differ in mtimes, logs and the initramfs, so the comparison
  runs modulo the catalogue's known differences, or waits. Only the
  packages layer needs it: above it, a layer is always replayed on its
  parent's snapshot, so patch and scratch agree by construction
  (domain/build.md, section 3).
- **The initramfs is judged by its inputs, not its bytes.** A new
  packages key rebuilds the layer, mktirfs runs again, and the image
  differs (2.2, row 4). So domain/slots.md (6.1) compares it by its inputs,
  which domain/build.md (5.8) defines. A snapshot bump that leaves them alone
  still allows a soft reboot, on trust that equal inputs make an
  equivalent image, which only a reproducible mktirfs would prove. `/usr/lib/modules/<version>` is
  still judged by its digest, so whether depmod's indexes reproduce is
  to verify.
- **No verification by rebuilding.** A peer, or the activator, cannot
  check a builder's output by building it again; each trusts its plane.
- **Warm compile caches are trusted, not checked** (domain/artifacts.md, 6.5),
  which is why production builds artifacts cold.
- **Not given up: evidence per key.** Keys are digests of inputs, so a
  verdict about a key carries over to every commit that keeps it
  (temper's `factory.md`, 8.2). That holds now.

## 5. Retrofitting

Four steps, in this order. The first three change what a build makes;
the fourth checks it. None changes the store's naming, the wire, the
slots or the records' shapes.

### 5.1 SOURCE_DATE_EPOCH per layer, from its key

- **The time is the Debian snapshot's timestamp,** taken from the
  packages part (domain/render.md, section 4), which every layer's cumulative
  key holds. With several archives pinned (`debian`, `debian-security`),
  the latest of their timestamps. An artifact takes its environment's
  (domain/artifacts.md, 5.2).
- **Never the commit's time.** A commit that changes nothing in a layer
  keeps its key, and must keep its bytes; a commit time would change
  them with every commit.
- **Set in every step's environment:** mmdebstrap's, and through it the
  maintainer scripts' (to verify that mmdebstrap passes it on; if not,
  hearth's apt configuration or a setup hook exports it), the software
  and config steps', `customize.sh` and `software.sh`. domain/build.md (5.4)
  leaves it out until then; artifacts set it from M6 (domain/artifacts.md,
  5.2).
- **Then the clamp, in the tree helper's walk, once per layer**
  (domain/store.md, 5.4). As the walk visits each changed inode, an mtime newer
  than the layer's time is set to it (`utimensat`,
  `AT_SYMLINK_NOFOLLOW`, the access time left as it is with
  `UTIME_OMIT`), and the ctime read after the clamp is recorded, so the
  next layer's change detection stays sound. This is mmdebstrap's
  `--clamp-mtime` for tar, done where mmdebstrap does nothing. Packaged
  files are older than the snapshot that holds them, and untouched files
  are not visited: both keep their own.
- **No digest changes.** The tree digest leaves mtimes out. What the
  clamp changes is what later steps read: a `.pyc` compiled from a file
  this step wrote, an archive made from the tree, diffoscope's noise.

### 5.2 Sealed steps

| What | Sealed to | How |
|---|---|---|
| hostname and domain | `localhost` | the step's uts namespace (domain/build.md, 8.1) |
| `/etc/hostname`, `/etc/resolv.conf` in the packages layer | absent | removed by a customize hook (2.3); the config layer writes the target's own |
| time zone | UTC | `TZ`; the target's own zone is config data |
| locale | `C.UTF-8` | `LC_ALL` (domain/build.md, 8.6) |
| umask | 022 | set by the sandbox before exec |
| variables | the step's fixed set | the environment given whole: none of the builder's crosses (domain/build.md, 8.6) |
| paths | the same in every build | `pivot_root`: a step sees its layer as `/`; the store's host path never appears |
| user | the namespace's root for build steps; id 1000 for artifacts | the sandbox |
| network | none, but the fetch step | loopback only (domain/build.md, 8.5); packages from a local repository |
| the clock | not sealable | time namespaces offset `CLOCK_MONOTONIC` and `CLOCK_BOOTTIME`, not `CLOCK_REALTIME` (time_namespaces(7)); `SOURCE_DATE_EPOCH` is the convention |
| randomness | not sealable | the real `/dev/urandom`; what a step makes from it is identity (removed, domain/build.md 5.6) or a finding |
| processors, kernel version | not sealed | the gate varies them (7.2): sealing the count would slow every build |

Most of this is in place from M2, since domain/build.md's sandboxes need it to
run IaC code safely. The retrofit adds what domain/build.md (8.6) does not set,
`TZ` and the umask, and checks the rest at the gate.

### 5.3 The normalised list

- **Data in `hearth-domain-build`, in the form of the scan's rules**
  (domain/build.md, 9.3): a path, a reason, the Debian bug where there is one,
  and an action besides. It describes Debian's packages, not a target.
  Whether a task may add entries for what it installs itself (protocol/format.md,
  section 10) is open.
- **A closed list of actions:** remove a path; empty a file; replace a
  file with fixed bytes from the list; refuse a package. No action edits
  a file's format: a fix that needs one is a fix to a tool, made with
  `SOURCE_DATE_EPOCH` or a patch, never an entry.
- **Each entry says why:**

  ```
  entry      /var/cache/man/**
  action     remove
  layer      packages
  made by    man-db's trigger (to verify)
  reason     an index of the build's pages; its timer makes it again in the slot's /var
  bug        none known
  since      the retrofit
  until      —
  ```

  A difference that comes from a bug carries the Debian bug's number
  and the version that fixes it; the entry goes once the pinned snapshot
  holds that version.
- **A constant of the encoding's version,** as every fixed choice of how
  hearth builds is (domain/store.md, 3.2): a change to the list is a new
  `hearth/N`, every key changes, and every layer is built again once.
  No digest of the list enters a key, and nothing else does either.
- **Its first entries are domain/build.md's day-one treatments and rules**
  (5.6, 9.2), which keep identity, secrets, the builder's details and
  logs out of every output. The retrofit adds only what reproducibility
  needs, from the gate's findings; the first candidate is man-db's
  index, above.
- **Fixes outside the list:** the initramfs (row 4), by a fix to mktirfs
  sent to Debian's tiny-initramfs (domain/machine.md, 7.5), or by hearth
  clamping the tree mktirfs packs; `.pyc` files and shadow's last-change
  day, by `SOURCE_DATE_EPOCH` (5.1).

### 5.4 The order of work

1. **The time and the clamp** (5.1).
2. **The sealed steps:** mostly a check of what M2's sandboxes already
   fix (5.2).
3. **The list** (5.3).
4. **The gate** (section 7), in the lab, on every new key from then on.

Each of the first three is a fixed choice of how hearth builds, so each
comes with a new version of the encoding (domain/store.md, 3.2): every key
changes and every layer is built again once. They may land together,
for one rebuild. Then comes the comparison across planes (7.6), once
enough keys reproduce to make a mismatch news.

## 6. What Debian proves

### 6.1 Package builds

- **Most of the archive builds reproducibly,** which matters little
  here: hearth installs Debian's binaries, pinned by the SHA-256 that
  `Packages` gives, and builds no package. Who built a `.deb` is
  Debian's trust, not hearth's.

### 6.2 Installations

- **Not tested archive-wide.** Debian's live images are built twice and
  compared on Jenkins, for bookworm, trixie, forky and sid, and the
  installed system inside them has stayed reproducible. trixie 13.2's
  regression was in its boot screens and firmware patterns, fixed for
  13.3.
- **mmdebstrap promises reproducible output** "across multiple runs with
  the same options and mirror content" given `SOURCE_DATE_EPOCH`, and
  its cleanup removes the usual offenders (2.3). The promise is about
  its packed formats: for a directory, it clamps nothing (2.2, row 1),
  and the host's `resolv.conf` and `hostname` are the caller's to fix.
- **Cloud images are not tested.**

### 6.3 So Debian proves the machinery

- **What every installation runs:** dpkg, debconf, triggers, systemd's
  presets, ldconfig, ca-certificates, Python's byte-compilation. A
  live image runs all of it, so its reproducibility covers it.
- **Not the server packages hearth installs:** postfix's postinst,
  dovecot's, a database's. The lab's gate (section 7) proves those, one
  key at a time, and reports what it finds.

### 6.4 Pinned inputs help

- **Debian's live-image rebuilds must fit within a six-hour archive
  synchronisation:** the second build must see the mirror the first
  saw. hearth's inputs are a snapshot, fixed for good, so a key can be
  built again a year later from the same bytes.

References: the Debian wiki's
[ReproducibleInstalls](https://wiki.debian.org/ReproducibleInstalls) and
[live images](https://wiki.debian.org/ReproducibleInstalls/LiveImages)
pages; rb-general on
[trixie 13.2](https://lists.reproducible-builds.org/pipermail/rb-general/2025-November/003937.html)
and
[13.3](https://lists.reproducible-builds.org/pipermail/rb-general/2025-November/003947.html);
mmdebstrap's manual
([trixie](https://manpages.debian.org/trixie/mmdebstrap/mmdebstrap.1.en.html)),
"cleanup" and "SOURCE_DATE_EPOCH"; PEP 552 for hash-based `.pyc` files.

## 7. The lab's gate

### 7.1 What is built twice

- **Every new layer key the lab builds,** once, on two builders:
  - a packages layer, from scratch on both;
  - a software or config layer, on the same parent: one builder fetches
    the other's parent (domain/store.md, section 10), so the layer is judged
    alone, whatever its parent's verdict;
  - an artifact, cold on both, and the lab's warm build compared with a
    cold one, which checks its compile cache (domain/artifacts.md, 6.5).
- **A key is judged once.** Evidence is per key (temper's `factory.md`,
  8.2): a commit that keeps a key gets its verdict by lookup, and a key
  is never built twice again for the gate.
- **Before landing,** as part of a change's lab gate: a snapshot bump or
  a change to a package set makes new packages keys, and every target
  the change touches makes new keys above them.

### 7.2 What varies

What should not matter, as reprotest varies it, but done by hearth, on
its own sandboxes:

- **across the two builders:** the machine, its kernel and its version
  string, the processor count, the subordinate id base, the store's
  path, the builder's hostname, its time zone and locale, the time of
  day the build runs;
- **inside one of the two builds:** directory order, through disorderfs
  (a FUSE filesystem that shuffles directory entries), mounted over the
  layer being built (to verify in the builder's user namespace); the
  processors the step may use;
- **not varied:** what 5.2 seals (it is the same by construction), and
  the clock beyond the builds' own distance in time. No namespace moves
  `CLOCK_REALTIME`, and an `LD_PRELOAD` clock does not reach static
  binaries, so a dependency on the day can escape the gate (section 8).

### 7.3 Comparing, and diffoscope

- **Equal tree digests: reproduced.** Nothing else is compared.
- **Different digests:** the two walks' records list each file's digest,
  so the differing paths are known at once, without reading the trees
  again.
- **diffoscope explains them.** It runs on the differing files only, in
  a sandbox, offline (it unpacks archives of content nobody reviewed),
  with its report bounded in size. The report is the finding's
  evidence.
- **The finding's signature** is the set of differing paths, with
  numbers and hashes normalised, so the lab's triager matches a second
  occurrence to an open finding (temper's `factory.md`, 8.3).

### 7.4 Verdicts, and degrading alone

Per layer key, a fact of the lab:

```
unchecked                  built once, not yet built twice
reproduced                 two builds on two builders, one tree digest
built, not reproduced      two digests; an open finding names the differing paths
```

- **A new difference blocks the change,** with the finding as its
  reason. A difference wholly covered by a finding already accepted
  does not: the layer degrades to "built, not reproduced".
- **A layer degrades alone.** The layers above it are built on one
  parent on both builders (7.1), so they are still judged, and still
  reproduce if they can. A target whose packages layer cannot reproduce
  can still have a reproduced config layer.
- **An accepted finding has an end:** an entry in the list (5.3), a fix
  in hearth, or a Debian version that fixes it.

### 7.5 Findings go upstream

- **A differing path that a Debian package owns** (dpkg's file lists say
  which) is reported to Debian's bug tracker against that package,
  tagged for Debian's reproducible builds team, with diffoscope's report.
  The list's entry, if one is needed meanwhile, carries the bug.
- **A difference that is hearth's own** (an unsealed variable, a step
  that writes the time) is a bug in hearth, fixed in hearth.
- **A difference that is a task's own** (`customize.sh`, `software.sh`,
  a recipe's command) is a finding for the repository that holds it.

### 7.6 What it buys

- **Comparison across planes.** The run side builds its own copy of a
  key and reports its tree digest as a fact; for a key the lab
  reproduced, the two must be equal. Equal means production runs the
  bytes the lab tested. Unequal means one builder misbehaved, and is a
  finding for both planes. Facts cross planes; outputs still do not.
- **Exact patch and scratch** (temper's `factory.md`, 6.4): a patched
  packages layer either has the scratch build's digest or a list of
  differing paths.
- **An initramfs judged by its bytes,** once it and the modules
  reproduce: domain/slots.md's rule (6.1) can then check the image itself, not
  only the inputs it trusts (section 4).
- **Verification by rebuilding,** for whoever wants it: a peer, or the
  activator for a builder that runs unreviewed code (README.md, section 9).
- **Warm equals cold** for artifacts, so compile caches stop being a
  matter of trust (domain/artifacts.md, 6.5).

### 7.7 Costs

- **One extra build per new key,** in the lab only: a packages layer
  from scratch (the bootstrap and the fetch from the local cache), or
  one step on an existing parent. Recorded as hearth's costs, like any
  build.
- **diffoscope only on differences,** bounded per finding.
- **Nothing on the run side** but a comparison of facts.

## 8. What is open

- **`SOURCE_DATE_EPOCH` through mmdebstrap:** whether mmdebstrap passes
  it to apt, dpkg and the maintainer scripts in directory mode, or
  hearth must export it itself (5.1). To verify on trixie's 1.5.7.
- **Which tools honour it on trixie:** systemd-sysusers 257 and shadow
  4.17.4 for the last-change day, dh-python's `py3compile` for hash-based
  `.pyc` files. Seen on sid; to verify on trixie.
- **The initramfs:** a fix to mktirfs sent to Debian, or hearth clamping
  the tree mktirfs packs before it runs (domain/machine.md, 7.5).
- **depmod's indexes:** whether `modules.dep.bin` and its siblings are
  the same for the same modules (to verify); a soft reboot after a
  snapshot bump depends on it while the modules are judged by digest
  (domain/slots.md, 6.1).
- **btrfs's directory order:** that readdir returns entries in creation
  order (to verify), and whether disorderfs runs over a layer in the
  builder's user namespace.
- **The clock beyond seconds:** no namespace moves `CLOCK_REALTIME`, and
  `LD_PRELOAD` clocks miss static binaries; whether two builders a day
  apart are worth the wait.
- **Who compares digests across planes,** and what the run side does on
  a mismatch: refuse to activate, or report and continue. It depends on
  the factory's engines (temper's `factory.md`, section 9).
- **Whether "built, not reproduced" may reach production,** or only
  after its finding is accepted. It is the run side's policy.
- **ids allocated by postinst:** that one closure installs in one order,
  so system users get the same ids on every build (to verify).
- **Tasks' own entries** in the normalised list (5.3): whether a task
  may add them, and how render shows them in the rendered diff.
