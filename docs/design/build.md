# Building outputs

Provisional, 2026-10-09. How an output is made from a spec: the recipe,
the three layers, the plan, each layer's steps, the sandboxes they run
in, what an output must not hold, the tools as protocols, caches and
costs. M2 builds it. Where layers live, how they are named and how a
layer is walked and published is store.md. What is open is in section
15.

## 1. In one page

- **An output is three layers:** packages, software and config, each a
  writable snapshot of the one beneath, changed by steps in sandboxes,
  then walked, frozen and published (store.md, 4.2).
- **The plan starts from the highest layer whose key is published.** A
  config change is a snapshot of the software layer and the config
  steps: seconds. The packages layer is always built from scratch in
  M2; apt's reconcile waits for the lab's comparison of patch and
  scratch.
- **Packages are fetched by digest, then installed offline.** A fetch
  step runs `apt-helper` for every `.deb` the resolution names by
  SHA-256, into the store's cache, where resolution kept the indices apt
  verified. mmdebstrap then installs from a local repository of hard
  links to both, with no network.
- **mmdebstrap runs inside hearth's sandbox, as its root.** It makes no
  user namespace of its own, and chroots from a root where only the
  layer is writable and the builder's tools are read-only.
- **The target's own tools make the config layer:** its
  `systemd-sysusers`, `systemd-tmpfiles` and `systemctl preset-all`,
  then each task's `customize.sh`. The tree helper applies the file
  tree with exact modes and owners.
- **Every build step runs in a sandbox of its own:** user, mount, pid,
  ipc, uts and network namespaces, pivoted into its root, inputs
  read-only, a minimal `/proc`, `/sys` and `/dev`, no network, cgroup
  limits. The fetch alone has the network, in a cgroup with no sandbox.
- **Nothing unique and nothing secret leaves a step.** mmdebstrap's
  cleanup stays on; the packages step removes what packages make (host
  keys, the snakeoil key, the sandbox's hostname, logs); a scan after
  every layer refuses an output that holds any offender.
- **Tools are protocols:** an argument vector down, typed events up.
  mmdebstrap's `I:` stage lines and failure tail, `apt-helper`'s
  `Get:` and `Err:` lines and `dpkg-query`'s fields are parsed below the
  domain, against recordings of real runs.
- **Everything slow is keyed:** layers, resolutions, downloads. What
  each step cost is a fact.
- **On a developer's machine the client builds in-process** (M2), as a
  regular user with a subordinate range and a btrfs store of its own
  (wire.md, 3.5; store.md, section 9).

## 2. Recipes

A recipe turns inputs into a chain of layers; the top one is the
output.

```
recipe rootfs                                                this document
  inputs   the spec's output parts: packages, software, config (render.md, section 4)
           the resolution the packages part names (store.md, 6.4)
  layers   packages on the empty layer; software on packages; config on software
  output   the config layer, named by its key (store.md, 3.3)

recipe artifact [M6]                                         artifacts.md
  inputs   an environment's output, sources, declared inputs, a command
  layers   one, collected from the command's outputs
```

- **One machinery for both:** keys (store.md, section 3), the fetch
  (5.1), sandboxes (section 8), the tree helper, records, pins.
- **Recipes are a closed set in `hearth-domain-build`,** each layer's
  steps fixed by its recipe; the parts give the data.
- **Slot parts never reach a recipe:** `data`, `secrets` and `boot` are
  the activator's at prepare (render.md, section 4; slots.md, section 5).
- **A host builds its containers' outputs [M5]** with the rootfs recipe,
  in its own store: no hearth runs in a container (slots.md, 12.5). A
  container's output differs only by its parts: the containers' variant
  (5.4), no initramfs configuration and no kernel, so its tree file's
  `boot` is empty (store.md, 6.3); nothing unique or secret enters it
  either (section 9), since its identity and secrets are written at
  prepare as a host's are.
- **The artifact recipe's sandbox is not a container target [M6]:** a
  build environment's output is only a sandbox's root, never run by
  nspawn nor placed in a slot (artifacts.md, 2.3).

## 3. The layers

| Layer | Parent | Key (store.md, 3.3) | Holds | Steps |
|---|---|---|---|---|
| packages | empty | `H("packages", resolution digest, packages part)` | the bootstrapped system: every package's files and what its maintainer scripts made, dpkg's database, the kernel and initramfs in `/boot`, `/var` as installed | fetch; mmdebstrap; verify (section 5) |
| software | packages | `H("software", packages key, software part)` | downloads placed; what each `software.sh` made (`/usr/local`, `/opt`) | fetch; place; `software.sh` (section 6) |
| config | software | `H("config", software key, config part)` | users; the file tree (`/etc`, `/usr/local/share` and more); tmpfiles' paths in the image; unit links from presets; what each `customize.sh` made | users; file tree; tmpfiles; presets; `customize.sh` (section 7) |

- **A layer is one staging subvolume;** its steps run on it in order,
  and it is walked once, after the last (store.md, 4.2).
- **An empty part still makes its layer:** a snapshot and a walk, about
  a second, so every output has three layers and keys stay uniform.
- **"config", not "etc":** the layer writes users, `/usr/local`, unit
  links and more (README.md, section 7).

## 4. The plan

### 4.1 From the highest published layer

```
plan(spec)
  resolution  ← the record named by the packages part's resolution key,
                or a resolution (render.md, section 5)
  keys        ← packages, software, config (store.md, 3.3)
  start       ← the highest of config, software, packages that is published
  build       ← each layer above start, bottom-up; from the empty layer if none is
```

- **A published config key is the output:** nothing is built.
- **A config change** builds the config layer on a snapshot of the
  published software layer. A software change builds software and
  config. A new closure or packages part builds all three.
- **The plan is data** in `hearth-domain-build`: layers, each with its
  steps. The builder's root routes its requests to the store (snapshots,
  walks, records) and to the protocol layer (tools).
- **A plan pins its chain** while it runs (store.md, 7.1).

### 4.2 The packages layer from scratch

- **In M2 the packages layer is always built on the empty layer,**
  never by apt's reconcile on an older packages layer.
- **Above packages, a patch and a scratch build agree by construction:**
  a config layer is always built on the same software layer, from that
  layer's tree, never on an older config layer.
- **A reconcile would not:** a purged package's leftovers, conffiles,
  debconf answers and maintainer scripts' state carry over (temper's
  `factory.md`, 6.4; os has no test that its reconcile matches a
  scratch build).
- **It is a later optimisation:** reconcile from the closest packages
  layer of the same suite, architecture and variant, adopted once the
  lab's comparison of patch and scratch holds it to the scratch build,
  which stays the proof.
- **The price:** a snapshot bump rebuilds packages in full, minutes
  (section 12).

### 4.3 Steps and failure

- **A step** is a tool or a tree helper request, with its inputs, its
  sandbox (section 8) and its limits, and ends in exactly one terminal
  event.
- **Failures are typed,** in wire.md's classes (wire.md, 4.4): a fetch
  is transient (the network, a 404, a digest mismatch); a step's exit,
  a limit, a failed verification or a scan finding fails the key.
- **A failed layer publishes nothing.** Its staging subvolume is
  deleted; the layers beneath stay published.

### 4.4 Concurrency

- **One build per key** (store.md, 4.6), different keys at once within
  the builder's limits: one packages step at a time, a configured
  number of other steps, one fetcher per source host.

## 5. The packages layer

### 5.1 The fetch

- **What:** each `.deb` of the closure (store.md, 6.4) whose SHA-256 the
  cache lacks. Never an index: Debian publishes no uncompressed
  `Packages` file, so the digest an `InRelease` gives for one names
  nothing a mirror serves. Resolution kept the indices apt verified in
  the cache instead (store.md, 6.4); a resolution whose kept indices the
  cache lacks is made again (render.md, section 5).
- **How:** `apt-helper download-file` with `<uri> <target>
  SHA256:<hex>` triples, at most 256 a run, into `staging/<key>/`; each
  file is then renamed into `cache/<sha256>` with its record (store.md,
  6.5).
- **URIs** join a source's URI with a package's `Filename`. Only `http` and `https` are allowed: with a `file:` URI
  `apt-helper` links the local file instead of fetching.
- **Checked before kept:** `apt-helper` verifies the SHA-256 and leaves
  nothing at the target on a mismatch (verified with apt 3.3.3: `Err:`,
  `Hash Sum mismatch`, exit 100). The builder does not hash again: apt
  in the build checks every `.deb` against its `Packages` entry, and the
  walk hashes what lands in the layer.
- **Polite:** one fetcher per source host, serially; snapshot.debian.org
  publishes no limits and has changed them. apt follows its redirects
  to content-addressed `/file/<sha1>/` URLs.
- **The fetch runs in a cgroup of its own, with no sandbox.** It is the
  one step with the network, and it has no namespaces: it runs as the
  builder, writes only the builder's own staging, and runs no code from
  the repository, since its inputs are data checked by digest. Its
  cgroup, a leaf of the builder's subtree (8.7), bounds its memory and
  processes, and stops it whole.
- **Its apt configuration is the builder's,** given whole with
  `APT_CONFIG`: retries, timeouts, a proxy where the builder has one
  (apt-cacher-ng on a pool's host). None enters a key or an output.

### 5.2 The fetcher, chosen

| | `apt-helper download-file` | apt in a fetch-only sandbox | skein's HTTP client |
|---|---|---|---|
| fetches | exactly the files named, each checked by SHA-256 | the closure again, from the sources | what the builder asks |
| needs | the list with digests, which the resolution holds | a scratch apt root; `apt-get update` every time; apt's text | DNS at run time, redirects, proxies, hashing gigabytes in step code |

**Chosen: `apt-helper`.**

- **The resolution already names every byte by digest,** so the fetch
  is a list, not a second resolution.
- **It is apt's acquire code:** HTTP and HTTPS, apt's proxy settings,
  snapshot's redirects, retries; it checks the digest before it keeps a
  file; it is on every Debian machine, in apt itself.
- **Its output is three line shapes** (10.2).
- **apt in a sandbox** would resolve again what resolution resolved,
  fetch the indices on every build, and speak more of apt's text.
- **skein's HTTP client** resolves names only at startup, and SHA-256
  in step code is unmeasured. It arrives with peers [M6]
  (`hearth-protocol-http`, wire.md, section 8), for bytes within a
  plane; the archive stays `apt-helper`'s.

### 5.3 The local repository

```
staging/<key>/repo/<a source URI's path>/                 one tree per source line
  dists/<suite>/InRelease                                 a hard link to cache/<sha256>
  dists/<suite>/<component>/binary-<arch>/Packages        the kept index, decompressed, at its plain path
  pool/…/<package>_<version>_<arch>.deb                   the closure's only
```

- **A sparse copy of the snapshot:** the genuine `InRelease`, the
  `Packages` files resolution kept (store.md, 6.4), and only the
  closure's packages. apt checks each index against `InRelease` and each
  `.deb` against its index, as it would online.
- **Indices at their plain paths:** `InRelease` lists the uncompressed
  `Packages` file with its SHA-256, so apt accepts it there. Debian's
  `InRelease` also announces `Acquire-By-Hash: yes`; whether apt asks a
  `file:` source for the by-hash path, or for a compressed file first,
  is to verify on recordings.
- **The sources mmdebstrap gets,** one line per source:

  ```
  deb [trusted=yes check-valid-until=no] file:///run/hearth/repo/<path> <suite> <components>
  ```

  `trusted=yes`: the signature was checked when resolution fetched the
  `InRelease` (render.md, section 5), and every byte since is checked
  by SHA-256; checking it again would add nothing, and could fail an
  old snapshot whose signing key has expired (to verify).
- **Bound read-only at `/run/hearth/repo`** in the sandbox (8.2), and
  into the chroot at the same path by mmdebstrap's
  `file-mirror-automount` hook, for apt inside. Directories are 0755,
  files 0644: their owner is unmapped there (store.md, 2.2).

### 5.4 mmdebstrap

```
mmdebstrap
  --mode=unshare                    run as the sandbox's root: no user namespace of its own (8.3)
  --format=directory
  --verbose                         apt's and dpkg's lines: progress and the failure tail (10.1)
  --variant=<variant>               the packages part: from placement
  --architectures=<arch>            the packages part
  --include=<name>=<version>,…      the closure, in arguments of at most 64 KiB
  --aptopt='APT::Install-Recommends "<install_recommends>"'     the packages part's one apt option
  --aptopt='Acquire::Check-Valid-Until "false"'                 hearth's, as at resolution (render.md, 5.2)
  --dpkgopt=force-confold                                       hearth's: keep what the setup hook placed
  --hook-dir=/usr/share/mmdebstrap/hooks/file-mirror-automount
  --setup-hook=<5.5>   --essential-hook=<5.5>   --customize-hook=<5.6>
  <suite> /target /run/hearth/input/sources.list
```

- **The whole closure is pinned** as `name=version`, so apt chooses
  nothing. Linux caps one argument at 128 KiB (`MAX_ARG_STRLEN`): a
  desktop's 3,000 names would pass it, so they are split, and
  mmdebstrap joins its `--include` options.
- **`--architectures` is given;** os's never was, so it built the
  host's.
- **apt's and dpkg's options stay in the output**
  (`/etc/apt/apt.conf.d/99mmdebstrap`, `/etc/dpkg/dpkg.cfg.d/99mmdebstrap`).
  `install_recommends` is the packages part's (render.md, 4.2). The
  others are hearth's constants, part of the encoding's version
  `hearth/1` (store.md, 3.2), so changing one changes every key:
  `Check-Valid-Until`, which resolution passes too, and `force-confold`,
  which keeps the configuration file the setup hook places before its
  package (5.5). No proxy is passed: the source is local.
- **No `--skip`:** every cleanup runs (5.6), and the target is an empty
  snapshot, so `check/empty` passes.
- **The environment is given whole:** `PATH=/usr/sbin:/usr/bin`,
  `HOME=/root`, `LC_ALL=C.UTF-8`, `DEBIAN_FRONTEND=noninteractive`,
  `DEBCONF_NONINTERACTIVE_SEEN=true`. No `SOURCE_DATE_EPOCH`
  (reproducible.md).
- **Inputs** sit read-only in `/run/hearth/input`: the local sources,
  the part's sources for the output, the seeded `passwd` and `group`
  lines, the preseeds, the initramfs configuration.

### 5.5 Hooks: seeds, preseeds, the initramfs and apt

```
--setup-hook='mkdir -p "$1/etc/tiny-initramfs" "$1/run/hearth" "$1/var/lib/machines" "$1/var/lib/portables"
  && cp /run/hearth/input/tiny-initramfs.conf "$1/etc/tiny-initramfs/"
  && cp /run/hearth/input/preseed "$1/run/hearth/preseed"'
--extract-hook='for f in passwd group; do
    [ -e "$1/etc/$f" ] || cp "$1/usr/share/base-passwd/$f.master" "$1/etc/$f"
    cat "/run/hearth/input/$f" >> "$1/etc/$f"; done'
--essential-hook='chroot "$1" debconf-set-selections /run/hearth/preseed'
```

- **Fixed text,** in `hearth-protocol-mmdebstrap`. Data from the
  repository reach hooks only as files, never as shell. A part with no
  initramfs configuration (a container) has no copy of it.
- **The setup hook runs before any package is unpacked:**
  - tiny-initramfs builds the initramfs from its kernel hook when the
    kernel package is configured, so its configuration must already be
    there (os does the same);
  - `/var/lib/machines` and `/var/lib/portables` are made directories
    first: systemd-container's postinst runs `systemd-tmpfiles --create`,
    whose `Q` lines make both subvolumes on btrfs where they are absent,
    and the walk refuses a nested subvolume (store.md, 3.4). Seen with
    sid's systemd 261; trixie's 257 is to confirm.
- **The extract hook seeds fixed ids,** after the essential packages are
  unpacked and before any is configured, in every variant. format.md
  requires a fixed uid and gid for each user and group that owns a data
  path, a secret or an identity file, package users included; the
  packages part carries the package users' lines (render.md, 4.2),
  since they change what this layer builds. A snapshot bump then cannot
  renumber them under their persistent subvolumes: `adduser --system`
  finds the user and keeps it, and the config layer's sysusers lines
  carry the same ids (7.2).
  - **Why the extract hook, not the setup hook:** base-passwd's preinst
    writes `/etc/passwd` and `/etc/group` only where they are absent, so
    files seeded before it would lack root and every static account. The
    hook copies base-passwd's own master files first, as its postinst
    would (read in base-passwd's maintainer scripts on sid), then adds
    the seeded lines.
- **The essential hook runs once the `Essential: yes` packages are
  configured and before the rest,** so the target's own
  `debconf-set-selections` loads the preseeds before tzdata, locales or
  postfix ask. mmdebstrap runs no essential hook for the `custom`
  variant, os's for containers and one candidate for hearth's
  (format.md, section 13): where a container's preseeds would load is
  open (section 15).
- **The initramfs configuration is the packages part's,** which render
  writes from its two fields, the modules and the microcode choice
  (render.md, 4.2; machine.md, 7.1). Nothing a task writes reaches it
  (format.md, 5.11), so `AUTO_MODULES=yes` and `MICROCODE=yes`, which
  read the build host, cannot occur.
- **apt's configuration** enters as `--aptopt` (5.4), written by
  mmdebstrap during its setup, permanent by design.

### 5.6 Cleanup

**mmdebstrap's own cleanup, all of it** (os skipped it all):

| Cleanup | Removes | Why it stays on |
|---|---|---|
| `cleanup/apt/lists` | the package lists | they name the local repository; no layer above runs apt |
| `cleanup/apt/cache` | downloaded `.deb`s and apt's caches | the store's cache has them |
| `cleanup/mmdebstrap` | its setup files: `00mmdebstrap`, the temporary apt configuration | the builder's details |
| `cleanup/reproducible` | `dpkg.log`, apt's `history.log`, `term.log`, `eipp.log.xz`, `alternatives.log`, ldconfig's `aux-cache`, `/var/lib/dbus/machine-id`; writes `/etc/machine-id` empty if it exists | logs and identity |
| `cleanup/run`, `cleanup/tmp` | everything in `/run` and `/tmp` | runtime only |

**hearth's customize hook,** fixed text, before mmdebstrap's cleanup:

```
--customize-hook='cp /run/hearth/input/sources.final "$1/etc/apt/sources.list"
  && rm -f "$1/etc/resolv.conf" "$1/etc/hostname"
  && rm -f "$1"/etc/ssh/ssh_host_*_key "$1"/etc/ssh/ssh_host_*_key.pub "$1"/etc/dropbear/dropbear_*_host_key
  && rm -f "$1/etc/ssl/private/ssl-cert-snakeoil.key" "$1/etc/ssl/certs/ssl-cert-snakeoil.pem"
  && find "$1/var/log" -type f -size +0 -exec truncate --size=0 {} +'
```

- **The sources** become the packages part's (the snapshot's URLs), in
  place of the local repository's.
- **`resolv.conf` and `hostname`:** mmdebstrap copies the host's, here
  the tool root's: `hostname` alone, `localhost` (8.2). Both are removed
  rather than given fixed bytes: an absent file is as reproducible
  (reproducible.md, 2.3), and holds nothing a target did not choose. A
  task that wants either writes it in the config layer.
- **Host keys:** openssh-server's postinst makes them, and dropbear's;
  the activator writes the machine's at prepare (slots.md, 5.4).
- **The snakeoil pair:** ssl-cert's postinst makes a private key, and
  dovecot-core and postgresql-common depend on ssl-cert. A task that
  serves TLS declares its key as a secret (format.md, section 9).
- **Logs** that mmdebstrap's list misses (fontconfig's trigger writes
  `/var/log/fontconfig.log`) are emptied, keeping modes and owners.
- **The empty `/etc/machine-id`.** A maintainer script makes an id
  (systemd's, to verify); mmdebstrap empties it. An empty file means "not a first boot"
  (machine-id(5)); on a read-only root systemd would then use a new
  transient id at every boot. The activator writes the machine's id
  into the slot at prepare (slots.md, 5.4), so the output keeps the
  empty file and the scan refuses any other.

### 5.7 Verifying the installed set

After mmdebstrap exits 0, the target's own `dpkg-query`, in a sandbox
pivoted into the layer, read-only:

```
dpkg-query --show --showformat='${Package}\t${Architecture}\t${Version}\t${db:Status-Abbrev}\n'
```

- **It must equal the closure exactly:** each name, architecture and
  version once, each `ii ` (installed, no error flag), nothing more.
- **A difference fails the layer,** listing the missing, the extra and
  the wrong: a package half-configured (`iF`, `iU`), an essential one
  the resolution missed, a version apt chose otherwise.

### 5.8 Boot digests

After the packages layer's walk, the builder records, per kernel the
layer holds, what slots.md (6.1) needs to decide whether a slot's kernel
is the running one. They go in the tree file's end item (store.md, 6.3):

```
version     the <version> of usr/lib/modules/<version> and boot/vmlinuz-<version>
kernel      the content digest of boot/vmlinuz-<version>, from the walk
modules     the subtree digest of usr/lib/modules/<version> (store.md, 3.4)
initramfs   the initramfs's input digest, below
```

**The initramfs's input digest,** defined here alone (slots.md, 6.1
cites it):

```
H("initramfs",
  list [ the kernel package's name, its version ],          the package that installs boot/vmlinuz-<version>
  bytes tiny-initramfs-core's version,
  digest the content digest of etc/tiny-initramfs/tiny-initramfs.conf,   which lists the modules and MICROCODE
  bytes <version>,                                          the kernel version
  list [ list [ name, version ] … ])                        with MICROCODE=generic, each installed microcode
                                                            package by name; empty otherwise
```

- **The initramfs's inputs, not its bytes:** mktirfs packs its staging
  tree with `cpio -H newc`, whose headers carry inode numbers and the
  build's mtimes (reproducible.md, 2.2), so the image differs on every
  build, and comparing bytes would force a full reboot on every snapshot
  bump.
- **From the resolution and the walk:** versions come from the closure
  (store.md, 6.4), digests from the tree file; nothing is read again.
- **Carried up unchanged:** the software and config layers copy them.
  A change above packages to `boot/`, `usr/lib/modules/` or
  `etc/tiny-initramfs/` fails the scan (9.3), since nothing above
  packages makes the image again.

## 6. The software layer

### 6.1 Downloads

- **What a download is, and who owns what it gives, is defined once:**
  format.md, 10.1 and render.md, 4.2. The part maps each path to a
  SHA-256 and either a `file` (mode and owners) or a `tar` (leading
  segments to strip); software is root's, or a package user's with a
  fixed id, and render refuses any other owner. Its URL is a fetch hint
  beside the part, outside the digest.
- **Fetched as in 5.1,** into the same cache, unless the cache has it.
  An artifact by key [M6] comes from the store instead (artifacts.md,
  7.3).
- **Recorded:** the software layer's record lists the SHA-256 of each
  download it placed, and the tree digest of each artifact [M6], so
  collection keeps them while the layer is kept (store.md, 6.2 and 7.5).

### 6.2 Placing them

- **A `file`** is a put of its path, mode, owners and content digest,
  applied by the tree helper (store.md, 5.5), owners resolved by name in
  the layer; a package user is there with its fixed id (5.5). No tool
  runs.
- **A `tar`** is unpacked by the target's own `tar`, in a sandbox
  pivoted into the layer, the archive bound read-only at
  `/run/hearth/download`, its directory made first by a put:

  ```
  tar --extract --file=/run/hearth/download --directory=<path> --no-same-owner [--strip-components=<strip>]
  ```

  `--no-same-owner` makes every file the namespace root's, whatever the
  archive says, and modes are kept, as format.md (10.1) says. `tar`
  detects the compression, with the target's own decompressors.

### 6.3 `software.sh`

- **Each task's script, after every download is placed,** in the order
  the software part lists them (render.md, 4.2), each in a sandbox of
  its own: pivoted into the layer, offline, as the namespace's root,
  `/bin/sh -eu /run/hearth/script`, with the target's shell and tools.
- **It names a download by the path it was placed at** (format.md,
  5.8); nothing of the builder's is visible (8.6).
- **os ran it outside the chroot,** with the host's tools against
  `$ROOTDIR` and a writable host cache.

## 7. The config layer

### 7.1 The order

```
1  users          the part's users, through the target's systemd-sysusers
2  file tree      the part's files, applied by the tree helper with modes and owners
3  tmpfiles       the part's lines for paths in the image, through the target's systemd-tmpfiles
4  presets        the part's presets, through the target's systemctl preset-all
5  customize.sh   each task's, in the part's order
```

- **Users first,** so the file tree's owners resolve. **`customize.sh`
  last,** so it sees everything else in place.
- **Each runs in a sandbox of its own,** but the file tree, which the
  tree helper applies.

### 7.2 Users

- **The part's groups, users and memberships** (render.md, 4.2) become
  sysusers lines (`g`, `u`, `m`), written to
  `/usr/lib/sysusers.d/hearth.conf` (a put), then:

  ```
  systemd-sysusers /usr/lib/sysusers.d/hearth.conf
  ```

  Given a path, it runs that file alone (systemd-sysusers(8)).
- **Ids are fixed where format.md requires them:** a user or group that
  owns a data path, a secret or an identity file has its uid and gid in
  its line, the same a package user was seeded with (5.5), so a snapshot
  bump renumbers nothing a persistent subvolume holds. A package user
  gets no line: the package made it (render.md, 4.2).
- **Passwords are locked** (`!*`). A password hash is a secret: the
  activator applies it to the slot's shadow file at prepare (slots.md,
  5.4).
- **At boot** `systemd-sysusers.service` finds the users made, and with
  `/etc` read-only it writes nothing.

### 7.3 The file tree

- **Path to content digest, mode and owner** (render.md, 4.2). Each
  content is in the cache by digest: a static blob streamed there by
  render's protocol layer (render.md, 2.3), a rendered file written by
  the builder. The tree helper then applies the tree as a change set of
  puts (store.md, 5.5), with owner and group names resolved against the
  layer's own `/etc/passwd` and `/etc/group`, after 7.2.
- **Modes and owners are exactly the part's,** a packaged conffile's
  included. os's `rsync -rltD` kept a packaged file's mode and its
  owner root, and tasks fixed them afterwards with tmpfiles `z` lines.
- **A missing parent directory** is made 0755, owned by root, unless the
  tree declares it.
- **A path through a link is refused:** `/lib` is `usr/lib` since the
  merged `/usr`, so tasks name canonical paths.
- **What hearth reserves is refused at render,** paths and kinds as
  format.md (5.11) lists them.

### 7.4 tmpfiles

- **The part's lines go to `/usr/lib/tmpfiles.d/hearth.conf`** for every
  boot, and at build are applied to paths in the image:

  ```
  systemd-tmpfiles --create -E --exclude-prefix=/tmp --exclude-prefix=/var/tmp /usr/lib/tmpfiles.d/hearth.conf
  ```

  `-E` excludes `/dev`, `/proc`, `/run` and `/sys`. A line for `/etc` or
  `/usr` takes effect in the root, read-only at boot, where the line
  then finds its work done; one for `/var` lands in the output's
  `/var`, which every slot copies; one for a persistent subvolume's path
  is applied again at boot, on the mounted subvolume; lines for `/run`
  and `/tmp` wait for boot.
- **The types are format.md's (5.6),** less what it reserves (5.11):
  lines that would make a nested subvolume in a layer (store.md, 4.1),
  or write into the read-only root at every boot and fail its bless
  (slots.md, section 7).
- **Only hearth's file runs at build;** the packages' own `tmpfiles.d`
  run at boot, as Debian intends.

### 7.5 Presets

- **The part's presets go to
  `/usr/lib/systemd/system-preset/00-hearth.preset`,** which sorts
  before every other preset file, so its lines win; then:

  ```
  SYSTEMD_OFFLINE=1 systemctl preset-all
  ```

- **No PID 1 answers in a sandbox;** `SYSTEMD_OFFLINE=1` makes
  `systemctl` act on files (systemd's `ENVIRONMENT.md`), with the layer
  as `/`, so no `--root`.

### 7.6 `customize.sh`

- **Each task's, last, in the part's order** (render.md, section 4),
  each in a sandbox of its own: `/bin/sh -eu /run/hearth/script`, with
  the target's shell, offline, as the namespace's root.
- **It sees the layer and nothing of the builder:** no secret (they are
  written at prepare), no target name, no network. What a script did
  with a secret (os's users task ran `chpasswd` from `/secrets`) moves
  to the activator or to boot.

### 7.7 The target's own binaries

- **Every step above packages is pivoted into its layer,** so
  `/usr/bin/systemd-sysusers` is the target's, at the target's systemd
  version, with its configuration. os ran the host's with `--root`.
- **A target without the tool** (no systemd, yet users declared) fails
  the step with "not found"; render may refuse it earlier (render.md,
  section 7).
- **The packages step is the one exception:** the builder's mmdebstrap,
  apt and `dpkg-deb` unpack the essential set, and every maintainer
  script then runs chrooted with the target's binaries. The
  `dpkg-query` that verifies is the target's.

## 8. Sandboxes

### 8.1 Namespaces

Each build step runs in namespaces of its own (README.md, section 7):

- **user:** uids and gids `0 <base> 65536`, written by `newuidmap` and
  `newgidmap`; the builder's own ids unmapped, so what it owns reads as
  `nobody` and through its "other" bits. The step runs as uid 0, with
  the namespace's capabilities.
- **mount:** a root of its own (8.2), `pivot_root`, propagation private.
- **pid:** every process dies when the namespace's first process ends.
- **ipc:** its own.
- **uts:** hostname `localhost`. Maintainer scripts write the hostname
  into configuration (postfix's `myhostname`), and a packages layer is
  shared by targets, so the name is fixed and none of theirs, and never
  the builder's.
- **network:** its own, with loopback only (8.5).
- **`no_new_privs`,** and no core dumps.

### 8.2 Two views

```
the packages step: a tool root                    every other step: the layer as root
/                  tmpfs, read-only once made     /                     the staging layer, writable
/usr               the builder's, read-only       /run, /tmp, /var/tmp  tmpfs: nothing there is kept
/bin /sbin /lib*   links into /usr                /run/hearth/…         the step's inputs, read-only
/etc               composed by hearth, below      /proc /sys /dev       8.4
/target            the staging layer, writable
/run/hearth/repo   the local repository, read-only
/run/hearth/input  sources, seeds, preseeds, initramfs configuration, read-only
/tmp               tmpfs
/proc /sys /dev    8.4
```

- **The packages step runs in a tool root and chroots into the layer.**
  Its target starts empty, and mmdebstrap runs the builder's tools
  (mmdebstrap, perl, apt, `dpkg-deb`) outside it. So that sandbox's
  root is a tmpfs holding the builder's `/usr` read-only and an `/etc`
  of hearth's own, with the layer at `/target`, and mmdebstrap chroots
  from it into the layer. Every other step is pivoted into its layer
  itself (README.md, section 7).
- **The tool root's `/etc` is composed, never the machine's.** On a
  machine, `/etc` holds the slot's machine-id and host keys and the
  secret files written at prepare; a maintainer script that left its
  chroot could copy any it can read into the layer. So `/etc` is a
  tmpfs, read-only once made, with only what mmdebstrap, apt and perl
  need:
  - `passwd` and `group`, written by hearth: `root`, `_apt` and
    `nogroup`;
  - `nsswitch.conf` (`files`); `hostname` (`localhost`), which
    mmdebstrap copies and the customize hook removes (5.6); no
    `resolv.conf`;
  - `mtab`, a link to `/proc/self/mounts`;
  - the builder's `ld.so.cache`, `ld.so.conf`, `ld.so.conf.d/` and
    `alternatives/`, bound read-only, which its binaries need to load;
  - no apt configuration and no keyring: mmdebstrap points apt at the
    target with a configuration of its own, and the sources are
    `trusted=yes` (5.3).

  Whether mmdebstrap 1.5.7 reads anything more is to verify on the
  conformance tier.
- **Nothing else is visible:** no other layer, no store, no home.
- **A maintainer script that leaves the chroot** lands in the tool
  root: the builder's `/usr` and hearth's `/etc`, read-only, with
  nothing of the machine's own.

### 8.3 mmdebstrap inside the sandbox

**Decided: hearth's sandbox around, mmdebstrap inside as its root,
`--mode=unshare`.**

- **As root, unshare mode makes no user namespace.** It unshares mount,
  pid, uts and ipc, nested in the sandbox's, and runs no `newuidmap`:
  1.5.7's `get_unshare_cmd` adds `CLONE_NEWUSER` only when not root. Its
  own pid namespace reaps what maintainer scripts start. Unable to
  `mknod`, it binds the sandbox's devices into the chroot.
- **Not mmdebstrap around, as the builder:**
  - its isolation is a chroot, which a process holding `CAP_SYS_CHROOT`
    in its namespace leaves, to see the builder's filesystem as the
    range's root sees it;
  - it never unshares the network (`if (0) { … CLONE_NEWNET }` in
    1.5.7), so maintainer scripts would reach it;
  - its limits, stop and proof of emptiness would differ from every
    other step's.
- **Not nested as a non-root user:** a second user namespace needs
  `newuidmap`, which is setuid and so disabled by `no_new_privs`, and
  could map only ids the sandbox maps already.
- **`--mode=root` is the alternative.** mmdebstrap's code allows for
  root mode in a user namespace ("we don't know whether we run in root
  mode inside an unshared user namespace or as real root"); the
  conformance tier runs both (testing.md, section 5).

### 8.4 `/proc`, `/sys` and `/dev`

- **`/proc`:** a new procfs for the sandbox's pid namespace, `nosuid`,
  `nodev`, `noexec`. The kernel allows it only where a fully visible
  `/proc` exists; in an Incus container that takes
  `security.nesting=true`.
- **`/sys`:** a new sysfs, read-only; the sandbox owns its network
  namespace, which the kernel requires for it.
- **`/dev`:** a tmpfs (`nosuid`, `noexec`) with the builder's `null`,
  `zero`, `full`, `random`, `urandom` and `tty` bound in; a new
  `devpts` instance with `ptmx` linked to it; a tmpfs at `shm`; `fd`,
  `stdin`, `stdout` and `stderr` linked into `/proc/self/fd`.
  bubblewrap's set.
- **os ran its chroots with none of them,** and wrote 8 KiB of the
  host's randomness into the rootfs as a fake `/dev/urandom`, never
  removed. Here the device is real and only in the sandbox; a layer's
  `/dev` stays empty, and the scan refuses anything in it.

### 8.5 Offline

- **A build step's network namespace has loopback only:** no route, no
  DNS, no proxy.
- **Only the fetch reaches the network** (5.1, 6.1), and it runs no
  code from the repository.
- **A maintainer script or `customize.sh` that downloads fails,** and
  the failure tail says so. What it wanted becomes a download of the
  software part.

### 8.6 The environment

- **Given whole, fixed, and covered by the key:** `PATH`,
  `HOME=/root`, `LC_ALL=C.UTF-8`, `DEBIAN_FRONTEND=noninteractive`,
  `SYSTEMD_OFFLINE=1`; then the step's own, as above.
- **Never:** the target's name, the revision, the builder's hostname,
  its variables, its proxy. Inputs are files at fixed paths. A step
  can still read the clock and count processors; that is
  reproducible.md's.

### 8.7 Limits and stopping

| Limit | Default | By |
|---|---|---|
| memory | 4 GiB for packages, 2 GiB otherwise | cgroup `memory.max` |
| processes | 4096 | `pids.max` |
| wall time | 60 min for packages, 15 min otherwise | the domain's deadline |
| progress | 10 min with no output | the domain's deadline on the pipes |
| disk | the store's free-space floor (store.md, 7.2) | `statfs` while the step runs |

- **Deadlines are the domain's;** the protocol layer runs the timers.
- **The builder's cgroups.** On a machine the activator's unit has
  `Delegate=yes`; the activator moves itself into a leaf, `activator/`,
  and makes `builder/`, owned by `hearth`, with `memory` and `pids`
  enabled, for the builder and everything it starts (activator.md,
  section 3). On a developer's machine the client runs under `systemd-run
  --user --scope -p Delegate=yes`. cgroup v2 lets no cgroup with
  controllers enabled for its children hold processes itself (the
  no-internal-processes rule), so the builder moves itself into a leaf,
  `builder/self/`, enables both controllers in `builder/`'s
  `cgroup.subtree_control`, and gives each sandbox, fetch and tree helper
  a leaf of its own beside it, named by a token and removed once empty.
  The limits above are written to those leaves.
- **Stopping is skein's three steps:** the termination signal to the
  first process, then to every process, then a kill of the cgroup. A
  step is over when its cgroup is empty (`cgroup.events`), and only
  then is its layer walked or discarded.

### 8.8 What it asks of skein

README.md (8.1) asks for contained process trees with id maps through
`newuidmap`. Beyond skein's draft `process.md`, sandboxes need:

- **a view that is only what is given:** a root the owner composes and
  `pivot_root`, rather than the host read-only with given directories
  writable;
- **user namespaces mapped by `newuidmap` and `newgidmap`;** network,
  pid, ipc and uts namespaces, and a hostname;
- **a new `/proc` and `/sys`, a minimal `/dev`;**
- **`no_new_privs` and rlimits.**

The domain and its worlds need none of this; real builds wait for it
(testing.md, section 6).

## 9. Nothing unique, nothing secret

### 9.1 Classes

- **Secret:** whoever reads it can act as the machine or a user.
- **Identity:** unique to a machine, so shared by every machine that
  runs the output.
- **The builder:** what tells which builder made it: its hostname,
  resolver, paths, proxy.
- **Noise:** the build's logs and caches, which carry its time.

All four are kept out of outputs (README.md, section 1), so outputs can
be shared between machines and peers.

### 9.2 The offenders

| Path | Class | Made by | Treatment |
|---|---|---|---|
| `/etc/machine-id` | identity | systemd's postinst | emptied by mmdebstrap (5.6); refused unless empty; written at prepare |
| `/var/lib/dbus/machine-id` | identity | dbus | removed by mmdebstrap; refused unless a link to `/etc/machine-id` |
| `/etc/ssh/ssh_host_*_key{,.pub}` | identity, secret | openssh-server's postinst | removed (5.6); refused; written at prepare |
| `/etc/dropbear/dropbear_*_host_key` | identity, secret | dropbear's postinst | the same |
| `/etc/ssl/private/*`, not empty | secret | ssl-cert's postinst (snakeoil) | snakeoil removed (5.6); refused |
| `/var/lib/systemd/random-seed`, `/var/lib/urandom/random-seed` | identity | a seed service run at build | refused; made at first boot |
| `/var/lib/systemd/credential.secret` | secret | `systemd-creds` run at build | refused |
| `/etc/shadow`, `/etc/gshadow` and their `-` backups: a password field but `*`, `!`, `!!`, `!*` | secret | `chpasswd`, `useradd -p` in a script | refused; hashes applied at prepare |
| `/etc/passwd`: a password field but `x` or `*` | secret | the same | refused |
| `/var/cache/debconf/passwords.dat`, not empty | secret | a password preseed | refused; render refuses `password` preseeds (format.md, 5.11) |
| `/etc/resolv.conf`, `/etc/hostname` | the builder | mmdebstrap copies the host's | removed (5.6); a task may write its own |
| `/etc/apt/sources.list` | the builder | mmdebstrap: the local repository | replaced by the part's sources (5.6) |
| an apt proxy setting | the builder | os's `--aptopt` | never passed: the build is offline |
| `/var/lib/apt/lists/*` but `lock`, `partial/` | the builder, noise | apt | removed by mmdebstrap; refused |
| `/var/cache/apt/archives/*.deb`, `/var/cache/apt/*.bin` | noise | apt | removed by mmdebstrap; refused |
| `/var/log/**`, a file not empty | noise | dpkg, apt, fontconfig | removed or emptied (5.6); refused |
| `/var/cache/ldconfig/aux-cache` | noise | ldconfig | removed by mmdebstrap |
| `/etc/.git`, `/etc/.etckeeper` | noise; secret (`/etc/shadow`'s history) | etckeeper | refused: no use on a read-only root |
| anything in `/dev` | the builder | os's fake `/dev/urandom` | never made (8.4); refused |
| anything in `/proc`, `/sys`, `/run`, `/tmp`, `/var/tmp` | noise | a step | emptied by mmdebstrap; tmpfs later (8.2); refused |
| a socket | noise | a daemon a script started | refused |
| an initramfs with the builder's microcode or modules | the builder | `MICROCODE=yes`, `AUTO_MODULES=yes` | never written: render writes the configuration (5.5) |
| an owner outside the range | the builder | a file the builder wrote | refused by the walk (store.md, 3.4) |

### 9.3 The scan

- **After every layer, in the tree helper's walk,** over what the layer
  changed (store.md, 5.4). A layer is its parent and its change set,
  and the parent passed, so checking the changes covers the output.
- **Rules are data** in `hearth-domain-build`: a path (exact, a prefix,
  or a pattern in its last component, such as `ssh_host_*_key`), a
  predicate (exists; not empty; not a link to a given path; password
  fields locked; a socket; anything beneath), the class, and the reason
  with its Debian bug where there is one.
- **The helper judges contents** (password fields, emptiness), since
  only it can read every file. A password field is read by a protocol
  machine, `hearth-protocol-accounts`, never by the helper's domain
  (store.md, 5.3). A finding names the path and the rule, never
  contents.
- **A finding fails the layer,** naming the step's layer, the path and
  the rule. Nothing is removed silently above the packages step: a
  `customize.sh` that makes a host key fails its build.
- **Above packages, the boot files are fixed.** A change in the
  software or config layer to `boot/`, `usr/lib/modules/` or
  `etc/tiny-initramfs/` is a finding: nothing above packages makes the
  initramfs again, and the boot digests are carried up unchanged (5.8).
- **The rules' digest is in the layer's record** (store.md, 6.2).

## 10. The tools as protocols

Each tool is a protocol machine: an argument vector goes down; stage,
progress, a result or a typed failure with a bounded tail comes up. The
domain never sees a tool's text.

### 10.1 mmdebstrap

`hearth-protocol-mmdebstrap`. Its argument vector is 5.4's, its hooks
5.5's and 5.6's.

- **On a pipe there is no progress bar:** mmdebstrap prints it only when
  stderr is a terminal. It prints `I:`, `W:` and `E:` lines and, with
  `--verbose`, apt's and dpkg's own lines between them.
- **Stages,** from 1.5.7's `I:` lines:

  ```
  I: running apt-get update...                              update
  I: downloading packages with apt...                       download
  I: extracting archives...                                 extract
  I: installing essential packages...                       essential
  I: installing remaining packages inside the chroot...     remaining
  I: running --<stage>-hook …                               a hook
  I: cleaning package lists and apt cache...                cleanup
  I: success in <seconds> seconds                           done
  ```

  An `I:` line it does not know is reported as other, not refused.
- **Progress** within essential and remaining counts dpkg's `Setting up
  <package> (<version>) ...` lines against the closure's size.
- **The failure tail** keeps the last 20 lines that match os's list
  (`dpkg: error`, `E: `, `Err:`, `No space left on device`, `Hash Sum
  mismatch`, `Failed to fetch`, `Unable to `, `Could not `) and none of
  mmdebstrap's own framing (`I: PID n exited with exit code`,
  `E: mmdebstrap failed to run`), each cut at 160 bytes. It is
  classified: a maintainer script failed (and whose), no space, a
  package missing from the repository, other.
- **Bounds:** a line at most 4096 bytes, the rest cut; the tail 20
  lines.

### 10.2 `apt-helper`

`hearth-protocol-dpkg`, which holds dpkg's and apt's machines.

- **Argument vector:** `/usr/lib/apt/apt-helper download-file <uri>
  <target> SHA256:<hex> …`, at most 256 triples; `APT_CONFIG` the
  builder's.
- **Output** (seen with apt 3.3.3; trixie's apt 3 is to record):
  `Get:<n> <uri> [<size>]` as each file starts; on a failure `Err:<n>
  <uri>`, indented reason lines (`Hash Sum mismatch`, the HTTP status),
  `E: Failed to fetch <uri>  <reason>`, then `E: Download Failed`; exit
  0 or 100.
- **Events:** each file started; each failure with its kind (not found,
  mismatch, network, other). After the exit the builder looks for each
  target: those present are kept.

### 10.3 `dpkg-query`

`hearth-protocol-dpkg`: 5.7's argument vector; lines of four
tab-separated fields, decoded to name, architecture, version and a
three-letter status; at most the closure's size and a margin of lines,
each at most 1 KiB.

### 10.4 The target's tools and the scripts

`systemd-sysusers`, `systemd-tmpfiles`, `systemctl`, `tar`,
`software.sh` and `customize.sh`: an exit status and the last 20 lines
of their output. The tail is kept by `hearth-protocol-tail`, a machine
crate of its own (lib-only, as `programming-model.md`, section 4 has
every machine), which `hearth-protocol-builder` routes to. Nothing more
is parsed. A status other than 0, a signal or a deadline fails the step
with its tail.

### 10.5 Recordings

- **Every machine is tested against real runs** (testing.md, section
  4): the argument vector, the environment, the bytes of stdout and
  stderr as they came, the exit status, recorded by the conformance tier
  on the local mirror (testing.md, section 5).
- **They replace os's hand-written fixtures** (a 17-line `Inst` sample,
  a 22-line failure log), and are recorded again when a tool moves.

## 11. Caches

- **Layers by key** (store.md, section 4): a config change reuses the
  software and packages layers.
- **Resolutions by key** (render.md, section 5): unchanged sources and
  requests resolve with no network.
- **Downloads by SHA-256** (store.md, 2.1): `.deb`s, indices and
  software, shared by every layer and target; a `.deb` is fetched once
  per store.
- **An apt proxy is optional,** the fetcher's from the builder's
  configuration (apt-cacher-ng on a pool's host, temper's `factory.md`,
  6.5), never in a key. snapshot.debian.org already redirects to
  content-addressed URLs.
- **Peers [M6]:** change sets and blobs by digest, within a plane
  (store.md, section 10).
- **Not cached in M2:** work inside a packages build (an essential base
  shared by closures); compile caches (artifacts.md, section 6).

## 12. Costs

- **Facts, per step and per build** (wire.md, section 6): wall and CPU
  time (cgroup `cpu.stat`), peak memory (`memory.peak`), bytes fetched,
  bytes written (from the walk), entries walked, files and bytes
  hashed, hits and misses per cache. The layer's record keeps them
  (store.md, 6.2).
- **What to expect:**

  | Work | Cost | Source |
  |---|---|---|
  | essential bootstrap, local mirror on tmpfs | 9.5 s; the `apt` variant 11 s | mmdebstrap's README |
  | essential trixie from deb.debian.org | 28.6 s, 3,940 paths | measured, unprivileged |
  | a host's packages layer, about 1,000 packages | minutes | an estimate, to measure |
  | snapshot, freeze, publish | constant | btrfs |
  | walking an unchanged root of 100,000 entries | under a second | an estimate (store.md, 5.7) |
  | hashing a packages layer | its bytes at the SHA-256 rate | to measure (store.md, 5.7) |
  | a config change, end to end | seconds | an estimate |
  | deleting a layer without the mount option | about 15 µs a path | measured |

- **A regression is a finding:** the lab benchmarks hearth as it does
  any product, and the real tier's second run builds nothing, which its
  costs show (testing.md, section 9).

## 13. Its world

- **`hearth-domain-build`'s world** runs the build's domain with the
  store's (or a scripted store) on the fake machine (testing.md,
  section 3). Its fakes: mmdebstrap, installing a seeded universe from a
  local repository, with users made by maintainer scripts, conffiles, a
  kernel, and output from recordings; the fetcher; `dpkg-query`; the
  target's systemd tools and `tar`; scripts in temper's shell subset;
  the tree helper.
- **Faults from the seed:** a 404, a mismatch or a stall in a fetch; a
  maintainer script that fails; a full disk; a step past its deadline;
  a step that leaves an offender; a power cut mid-step.
- **Invariants** (testing.md, section 8):
  - no published output holds an offender of 9.2, as the world checks
    on the fake tree on its own;
  - every step's processes saw only their view, and no network;
  - a failed step publishes nothing, and the layers beneath stay;
  - a packages layer's installed set is its closure;
  - keys follow store.md, 3.3: a second build of a spec builds nothing,
    a config change builds only config;
  - every step reports its costs.
- **Real builds** run as a regular user against the local mirror
  (testing.md, section 6).

## 14. What changes from os

- **Unprivileged:** os's deploy ran as root through sudo in practice;
  the builder runs as itself, and the activator shifts at prepare.
- **Layers by key, shared between targets,** replace `@T`, `@T.var` and
  the pristine `@T.install` snapshots named by target; `/var` stays in
  the layer until prepare.
- **No reconcile:** M2 builds packages from scratch, where os ran apt
  install and purge on its pristine snapshot.
- **Offline, from a local repository,** where os fetched through
  apt-cacher-ng with the proxy as an `--aptopt` left in the output.
- **mmdebstrap's cleanup kept,** where os passed `--skip=cleanup`.
- **Sandboxes with `/proc`, `/sys` and `/dev`,** where os used
  `mmdebstrap --unshare-helper` and `chroot` with none, and left a fake
  `/dev/urandom` in the rootfs.
- **The target's own systemd tools,** where os ran the host's with
  `--root`; **`software.sh` with the target's tools,** offline.
- **Modes and owners as data, applied exactly,** where `rsync -rltD`
  kept packaged modes and tmpfiles `z` lines fixed them later.
- **No etckeeper and no secrets in the build:** the `pristine-debian`
  tag and `Manifest config` commit go, and so does `/secrets`, which os
  copied into the output for `customize.sh`.
- **The fleet leaves the code:** os's Lisbon and en_US preseeds become
  part data; the variant comes from placement; the initramfs
  configuration enters the packages part with `MICROCODE` explicit.
- **Verification adds architecture and status;** no `/.manifest/` in the
  output, whose records live beside the layer (slots.md, 5.5 for the
  slot's).
- **Recordings replace hand-written fixtures;** pipes and deadlines
  replace `os-proc`'s one-second polling of files.

## 15. What is open

- **apt's reconcile for the packages layer:** when it is adopted;
  depends on the lab's comparison of patch and scratch (temper's
  `factory.md`, 6.4).
- **Preseeds where no essential hook runs:** the `custom` variant, if
  containers take it (format.md, section 13), and preseeds for
  `Essential: yes` packages; depends on whether the extract hook can
  load them with an unconfigured debconf.
- **`--mode=unshare` or `--mode=root`** inside the sandbox; depends on
  the conformance tier's comparison.
- **The local repository's details:** whether apt asks a `file:` source
  for a by-hash path or a compressed index before the plain `Packages`,
  and whether `trusted=yes` is needed; depends on recordings of apt on a
  `file:` source.
- **The builder's tool versions** (mmdebstrap, apt, `dpkg-deb`) are
  recorded but not in the packages key; depends on whether two versions
  ever build different trees from one closure.
- **The snakeoil pair:** removed, or made per machine by the activator
  as identity; depends on slots.md's identity (5.4).
- **setuid, setgid and capabilities added above packages,** by an
  archive's modes or a script: allowed, or refused unless declared;
  depends on format.md (5.8, 10.1).
- **Nested subvolumes made at build** beyond `/var/lib/machines` and
  `/var/lib/portables` (5.5), which trixie's systemd 257 is to confirm;
  depends on recordings of a bootstrap of the fleet's closures.
- **The sandboxes' hostname,** `localhost`, and whether it should be
  something a scan can find in files; depends on recordings of the
  packages that write it.
- **seccomp in sandboxes;** depends on skein's contained trees.
- **The limits' defaults;** depends on the real tier's costs.
- **`SOURCE_DATE_EPOCH` per layer;** depends on reproducible.md.
