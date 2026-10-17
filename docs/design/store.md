# The store

Provisional, 2026-10-09, revised 2026-10-16. The builder's store on
btrfs: its layout, the canonical encoding every digest uses, layer keys
and tree digests, a layer's life, the tree helper, records, pins and
collection, how each builder reads the store back, the steward's records
beside it, what the machine must provide, and how peers exchange layers.
M2 builds it, but for peers (section 10, M6). M1 builds the encoding and
`H` (3.1, 3.2), and a store without layers, which holds render's cache
and resolutions (2.4). How a layer's contents are made is build.md.
What is open is in section 12.

## 1. In one page

- **The store is a btrfs subvolume of the builder's own, mode 0700.**
  It holds layers with their records and tree files, resolutions, a
  download cache by SHA-256 and the scratch of steps in flight. Every
  name in it is a digest in lowercase hex.
- **One encoding, one hash.** Every digest is SHA-256 over the
  canonical encoding of 3.1: typed, length-prefixed, maps sorted by key.
  `H(tag, values)` hashes a list that starts with a version and a tag.
- **Layer keys are cumulative:** packages, software and config are each
  `H` of the key below and their own part's digest; the config key
  names the output. The packages key takes the digest of what the
  resolution resolved to, not of what was asked.
- **A tree digest is what a tree is, as the namespace sees it:** paths,
  kinds, permission bits, owners as namespace ids, capabilities, ACLs,
  `user.*` attributes, contents, link targets and hard links. No times,
  no inode numbers.
- **A layer is one read-only subvolume, `/var` included:** a writable
  snapshot of its parent, changed by build steps, walked, frozen,
  recorded and published by a rename. What a crash leaves half-made is
  deleted at startup.
- **What changed is found by a walk.** An entry whose path, inode number
  and change time match the parent's record is unchanged; only the rest
  is read and hashed. A snapshot keeps inode numbers and change times.
- **The tree helper does what the builder cannot.** Root of a user
  namespace mapped onto the builder's subordinate range, it walks,
  hashes, applies and empties layers, and snapshots and freezes them,
  which btrfs allows only to the owner of a subvolume's root.
- **Records sit beside what they describe,** in one framed, versioned
  format, each written to a temporary name and renamed.
- **Pins keep layers and their ancestors; collection deletes the
  rest:** unrecord, clear the read-only flag, empty inside the
  namespace, `rmdir`. Builders and test filesystems may delete whole
  with `user_subvol_rm_allowed`.
- **The filesystem is hearth's only state.** At each builder's start
  three unprivileged btrfs ioctls and the records rebuild what it holds
  in memory. No kv store.
- **Peers exchange change sets [M6]** by layer key over HTTP, applied to
  a snapshot of a parent the receiver holds and checked by tree digest,
  within one plane.

## 2. On disk

### 2.1 The layout

```
<store>/                           a subvolume, 0700, the builder's (@hearth.store on a machine)
├── store.record                   the store's own record (6.6)
├── layers/                        0700
│   ├── <key>                      a published layer: a read-only subvolume
│   ├── <key>.record               its record (6.2), 0644
│   ├── <key>.tree                 its tree file: entries and change set (6.3), 0644
│   ├── <key>.staging              a layer being built: a writable subvolume (4.2)
│   ├── <key>.trash                a layer being deleted (7.3)
│   └── <key>.tree.tmp             a tree file being written (6.8)
├── resolutions/<key>.record       a resolution (6.4)
├── cache/<sha256>                 bytes by content: downloads, kept indices, render's blobs; 0644, never modified (2.2)
├── cache/<sha256>.record          where they came from (6.5)
├── cache/<random>.tmp             a file being written, whose digest is known at its end: renamed to <sha256> (6.8)
├── <record>.skein-<16 hex>        beside any record, in any directory: skein's Store replacing it (6.8)
├── targets/<H("target", name)>.record   a target's last builds (6.6)
├── state/                         the steward's records and rings (8.3)
├── artifacts/                     [M6] artifacts, as layers/ holds layers: <key>, .record, .tree, .staging, .trash
├── sources/, caches/              [M6] artifacts' sources and compile caches, as subvolumes (artifacts.md, 3.2, 6.1)
├── repo/                          the builder's clone of the IaC repository (render.md, 2.2)
├── credentials/                   0700: the builder's own read credential for it (render.md, 2.2)
└── staging/<key>/                 scratch of a layer being built: fetched files, the local repository, inputs;
                                   or of a resolution being made, by its resolution key (render.md, 5.2)
```

- **Every name beneath the top is a digest,** 64 lowercase hex digits,
  then a suffix for its kind; nothing is named by a target, a path or a
  time. The one exception is a cache file being written, whose digest is
  known only at its end: it is named by 64 random hex digits, drawn from
  the builder's generator, and `.tmp`. The other is a record being
  replaced: skein's `Store` writes it as `<record's name>.skein-<16 hex>`
  beside it (6.8). The top's names are the fixed ones above, and
  `store.record`'s temporary. A name of no known
  shape is not hearth's: the startup scan reports it and leaves it alone
  (8.2).
- **Directories are 0700; records, tree files and cache files 0644,**
  all the builder's: user `hearth` on a machine, the developer on a
  developer's machine. The builder runs with the umask 022, and skein's
  `Store` makes a file `0o666` less it, with no mode of its own: the
  store's directories keep everyone else out (2.2).
- **Two writers, apart:** on a machine the steward writes only
  `state/`, and the builder writes the rest and never `state/`
  (steward.md, 2.2). Both run as `hearth`.
- **Artifacts [M6]** are read-only subvolumes named by key, with records
  and tree files beside them, built, published and deleted as layers are
  (artifacts.md, section 4); they live in `artifacts/` so that their
  renames, too, stay within one directory (2.3).

### 2.2 Owners and modes

- **A layer's files are owned by the builder's subordinate range.** Its
  root is a directory, 0755, owner and group 0 in the namespace (the
  range's base on disk), so the builder, as "other", can always open it
  to ask btrfs about it (8.2).
- **Cache files are 0644 in a 0700 directory.** A sandbox reads them
  through a bind mount (build.md, 5.3), where the builder's own uid is
  unmapped and only the "other" bits apply; the directory keeps everyone
  else out.
- **The cache and staging share the store's subvolume,** so a packages
  step's local repository is hard links to cache files: `link(2)` across
  subvolumes fails with `EXDEV`.

### 2.3 Why staging and trash live in `layers/`

- **A rename into another directory needs write permission on the
  directory moved,** to rewrite its `..` (`vfs_rename`, the kernel's
  `fs/namei.c`). A layer's root is the namespace root's, not the
  builder's, and once frozen btrfs refuses write permission on it
  (`EROFS`).
- **A rename within one directory needs only that directory's
  permissions,** which are the builder's. So a layer is built as
  `<key>.staging`, published as `<key>` and retired as `<key>.trash`,
  all within `layers/`.

### 2.4 A store without layers

A store whose filesystem is not btrfs, or whose builder has built no
layer, holds what render keeps: its cache and its resolutions. M1's
`render` and `diff` use one (client/README.md, 3.5), and so may a builder that
only renders.

- **Any filesystem,** in a directory of the user's own, 0700: btrfs and
  the subordinate range are prerequisites of layers alone (section 9).
  `build` refuses a store that is not a btrfs subvolume [M2], naming what
  is missing (9.1).
- **What it holds:** `store.record` (6.6), `cache/` with its records
  (6.5), `resolutions/` (6.4) and `staging/`, whose only scratch is a
  resolution's (render.md, 5.2). The layout and the records are those of
  a store with layers, so a builder of M2 reads it as its own.
- **The lock and the scan hold:** a builder takes 8.2's lock on the
  store's root, and its startup scan runs steps 1 and 2, step 6 for its
  resolutions and its cache alone, and step 7, settling 4.4's rows for
  temporaries, staging, resolutions and cache files. Steps 3 to 5, the
  subvolumes', belong to a builder that builds [M2], which runs them on
  btrfs and skips them elsewhere, telling the two by `statfs`'s
  filesystem type (README.md, 8.1). M1's builder builds no layer and
  runs none of them: a store it makes has no `layers/`.
- **Everything in it can be made again:** a resolution is resolved
  again, a cache file fetched or rendered again. A store without layers
  may be deleted whole, and made again as a btrfs subvolume when its user
  starts to build.

## 3. Digests and keys

### 3.1 The canonical encoding

Every value that is hashed is one of seven kinds:

```
value          bytes
none           00
false          01
true           02
uint n         03  n as 8 bytes, big-endian
bytes b        04  the length of b as 8 bytes, big-endian; then b
digest d       05  the 32 bytes of d
list [v…]      06  the count as 8 bytes, big-endian; then each v, encoded
map {k: v…}    07  the count as 8 bytes, big-endian; then, for each entry, k encoded as bytes (04, its length, k), then v
```

- **Map keys are byte strings, strictly increasing bytewise;** a
  duplicate is not a map. A record's field names are map keys. The order
  is that of the keys' own bytes, not of their encodings, whose lengths
  come first.
- **An optional field is left out when absent.** A record's map holds the
  fields it has: `gid?` without a gid, `security` with the pocket off, a
  container's packages part without `initramfs`. A field a later
  milestone adds (`[M5]`, `[M6]`) is optional, and absent where a record
  has none of it, so a record that does not use it keeps its digest
  across the release that adds it.
- **`none` is a value, not an absence:** it is for a field that is always
  present and may hold nothing, written `| none` (a layer's `parent`,
  6.2).
- **An enumeration is its variant's name,** as `bytes`, in the words the
  design gives it: `file`, `link` and `directory` in a config part's
  tree; `enable` and `disable`; `core`, `data` and `cache`; `none` and
  `generic` for `microcode`, which are names, never the `none` kind. A
  variant that carries fields is the map of its fields with one more,
  `kind`, its name. Booleans are `false` and `true`. The tree digest's
  entries (3.4) keep their kinds as numbers, fixed there.
- **A set is a list,** without duplicates, in its members' order: text
  bytewise, a pair by its first member and then its second, unless the
  part names another order, as a packages part's components do
  (render.md, 4.2).
- **Text is its UTF-8 bytes as given,** with no normalisation and no
  terminator.
- **No signed integers, no floats.** A value that needs one enters as
  text (render.md, section 4).
- **Kinds stay apart:** a digest is not 32 bytes of `bytes`, a boolean
  is not 0 or 1, none is not an empty list. One value has one encoding,
  and a decoder needs no lookahead.
- **Decoding is bounded:** every length and count is checked against a
  limit before anything is set aside, and nesting uses an explicit stack
  (`programming-model.md`, section 8).
- **The encoder and `H` live in `hearth-lib`,** a crate in the lib role:
  `no_std`, depending on skein-lib alone. A domain depends only on lib
  and its children (`programming-model.md`, 4.5), and six of hearth's
  domains encode and hash, so every hearth domain may depend on it.
  Decoding is parsing, and stays a protocol machine's:
  `hearth-protocol-tree` (5.3).

### 3.2 H

```
H(tag, v1, …, vn) = SHA-256( encode( list [ bytes "hearth/1", bytes tag, v1, …, vn ] ) )
```

- **The tag says what is hashed,** so two kinds of digest never collide.
- **`hearth/1` versions the encoding and hearth's own constants.** A
  change to 3.1 or 3.4, or to a fixed choice of how hearth builds (apt's
  and dpkg's fixed options, `force-confold` among them; the cleanup a
  step runs: build.md, section 5), changes it, and every key with it: a
  rebuild, never a migration.
- **A file's content digest is plain SHA-256 of its bytes,** without
  `H`, so it matches `sha256sum`, the `SHA256` of Debian's `Packages`
  and format.md's software digests (format.md, section 10).

| Tag | Values | Defined in |
|---|---|---|
| `empty` | none | 3.3 |
| `packages`, `software`, `config` | the key below, then a part's digest | 3.3 |
| `closure` | a resolution's closure | 6.4 |
| `resolution` | a resolution's inputs | render.md, section 5 |
| `part:<name>`, `spec` | a part's record; a spec's digests | render.md, 4.5 |
| `tree`, `changes` | a tree's entries; two trees and their changes | 3.4, 3.5 |
| `target`, `state` | a name | 6.6 |
| `artifact`, `inputs`, `command`, `outputs` | an artifact's | artifacts.md, section 3 |
| `environment` | a resolution key, then the packages, software and config part digests | artifacts.md, 3.1 |

Test vectors:

```
encode(list [bytes "hearth/1", bytes "empty"])
  = 06 0000000000000002 04 0000000000000008 6865617274682f31 04 0000000000000005 656d707479
H("empty")
  = 8f43cea6a4bf9d3aec62c882bdafeab30cc6d71acc9b0b984b7f2fd8f70975d3
H("packages", digest 00…00, digest 01…01)
  = 22d5156ffefadb1f0df4eb2c4d405b2c8937215e044486b06b1725e3506b238d
H("tree", list [ list [bytes "", 1, 0o755, 0, 0, map {}, none] ])        the empty layer's tree
  = 6022d1fbc6ad38d08d2ae02cefc21e7e6af5a35e6be9d115f0ba267fdb0d005a
encode(map { "a": uint 1 })                                               a key as bytes
  = 07 0000000000000001 04 0000000000000001 61 03 0000000000000001
H("test", map { "adm": map { "gid": uint 4 }, "staff": map {} })          staff's gid absent
  = 8291ea66351528fd3979b84aec0397019d25dae6f8d8aa9a0318331d2fd164f5
H("test", map { "kind": bytes "file", "mode": uint 0o644, "owner": bytes "root" })   a variant by name
  = 72653b9931df15f292b4f5f17e091263cd9fb738b0f2c06cce1cd74242b5b211
```

`test` is the vectors' tag alone; no record of hearth's uses it. The
last three pin 3.1's rules for keys, absent fields and enumerations:
with the key's `04` left out, the second would be `3f4edef7…`, and with
`gid: none` for staff, `ab3793df…`.

### 3.3 Keys

```
empty key          = H("empty")
packages key       = H("packages", resolution digest, packages part digest)
software key       = H("software", packages key, software part digest [, heads])         heads [M6], below
config key         = H("config",   software key, config part digest)       the output's name
resolution digest  = H("closure",  closure)                               6.4
```

- **Each key covers everything beneath it.** A change to a part changes
  its layer's key and every key above, nothing below.
- **A head artifact's resolved key enters the software key [M6].** When
  the software part names an artifact by its head form (a recipe and its
  listing's digest), the plan resolves it to an artifact key before the
  software layer (artifacts.md, 7.2), and `heads` is a map from each
  such entry's path to that key, appended as a third value. It is absent
  when the part has no head entry, so keys without one are as above.
  It plays the closure digest's part in the packages key: the part holds
  what the revision says, the key what was resolved (render.md, 4.5).
- **A resolution's key and its digest differ.** The key, over its
  inputs (render.md, section 5), names its record. The digest, over the
  closure it gave, enters the packages key, so the packages layer is
  named by what is installed: a resolution made again (its record
  collected, a newer apt) that gives another closure makes another
  layer.
- **Part digests are render's** (render.md, 4.5), made with this `H`.
  The packages part carries what the packages step reads besides the
  closure: sources, the request, the variant, preseeds, the initramfs's
  modules, and `install_recommends`, its one apt option (render.md,
  4.2). hearth's fixed apt and dpkg options are constants of
  `hearth/1` (3.2), not fields of the part.
- **The empty layer** is every packages layer's parent (4.1).
- **Nothing else enters a key:** not the builder, its subordinate base,
  its store's path, the time, the target's name, the revision, nor the
  builder's tools' versions, which the record keeps (6.2). A step is
  given nothing its key does not cover (build.md, 8.6).
- **A key names what was asked for; the tree digest records what was
  built.** Until builds are reproducible, two builds of one key differ,
  and each records its own (reproducible.md).

### 3.4 The tree digest

**Covered,** per entry, as the tree helper reads it in the namespace:

- the path, relative to the layer's root;
- the kind: directory, file, symbolic link, hard link, character or
  block device, fifo, socket;
- the permission bits with setuid, setgid and sticky (`mode & 0o7777`);
- owner and group as namespace ids, 0 to 65535;
- extended attributes, by name and value, of four families:
  `security.capability` as the namespace reads it (version 2: a set
  made in the namespace is stored as version 3 with the base as root id
  and reads back so); `system.posix_acl_access` and
  `system.posix_acl_default`, their ids translated by the kernel;
  `user.*`;
- for a file, its size and content digest; for a link, its target's
  bytes; for a device, major and minor; for a hard link, the path it
  shares an inode with.

**Left out:**

- every time: mmdebstrap clamps none in a directory, and reproducible
  builds are deferred;
- inode numbers, link counts, the device, the generation, holes;
- inode flags: of those a user may set, only no-copy-on-write (`C`,
  which systemd's tmpfiles sets on `/var/log/journal`), a hint;
- btrfs properties (`btrfs.compression`) and the subvolume's identity;
- `trusted.*` and every other `security.*`: setting them needs privilege
  in the initial namespace, so no layer built in the builder's holds
  any, and a change set carrying one is refused.

```
entry    = list [ path: bytes, kind: uint, mode: uint, uid: uint, gid: uint, xattrs: map, payload ]

kind 1   directory      payload none
kind 2   file           payload list [ size: uint, content: digest ]
kind 3   symbolic link  payload bytes: the target, as stored
kind 4   hard link      payload bytes: the path of the first entry with its inode
kind 5   char device    payload list [ major: uint, minor: uint ]
kind 6   block device   payload list [ major: uint, minor: uint ]
kind 7   fifo           payload none
kind 8   socket         payload none

tree digest = H("tree", list [ entry… ])        in tree order
```

- **A path** joins its components with `/`, with no leading or trailing
  `/`; the root's is empty. A component is not empty, `.` or `..`, and
  holds no `/` or NUL. At most 4096 bytes.
- **Tree order** compares paths component by component, each bytewise:
  a directory comes before everything beneath it, and its subtree before
  its next sibling (`a`, `a/b`, `a-b`). It is a depth-first walk with
  sorted names, so walks and tree files stream.
- **Hard links:** an inode's first path in tree order is its file; each
  later path is a kind 4 entry with the same mode, owners and
  attributes. Debian ships some (`perl-base`'s `/usr/bin/perl`).
- **Refused by the walk:** a root that is not a directory, 0755, owners
  0; an owner past 65535, which catches a file the builder wrote (its
  ids show as 65536 to the tree helper, 5.2); a nested subvolume (an
  entry on another device); a bound passed (5.6). systemd-container's
  postinst runs `systemd-tmpfiles --create`, whose `Q` lines make
  `/var/lib/machines` and `/var/lib/portables` subvolumes on btrfs, so
  the packages step makes both directories first (build.md, 5.5).
- **A subtree's digest** is the same formula over its entries, paths
  made relative to it, computed from the tree file without reading the
  layer. When the kernel is unchanged is slots.md's to decide (6.1),
  from digests the tree file records (6.3).
- **Its record format** is the tree file (6.3).

### 3.5 The change set

```
change            = list [ op: uint, path: bytes, entry or none ]
  op 1  put       the child's entry at path, whatever the parent had there
  op 2  delete    the parent had path; the child has nothing there
change set digest = H("changes", parent tree digest, child tree digest, list [ change… ])
```

- **In tree order, a path at most once.** A deleted directory lists
  every path beneath it; a put that changes a path's kind replaces what
  was there.
- **Never trusted alone:** applied to the parent and walked, it must
  give the child's tree digest (5.5, 10.2).
- **It is what a layer changed,** for peers (10.1) and for people and
  agents: the files a config change touched, from the tree file.

## 4. Layers

### 4.1 What a layer is

- **One read-only subvolume, `/var` included.** dpkg's database lives in
  `/var` and travels with the layer; a slot's `/var` is split from it at
  prepare (slots.md, 5.3).
- **Of four kinds:** `empty`, `packages`, `software`, `config`, each the
  parent of the next. Artifacts are layers of their own kind on the
  same machinery [M6] (artifacts.md, 4.2).
- **The empty layer** holds only its root, a directory, 0755, owners 0.
  The builder creates the subvolume (`BTRFS_IOC_SUBVOL_CREATE` needs no
  privilege); the tree helper applies the root's entry, since a new
  root is the builder's (5.5). It is made with the store and never
  collected.
- **No subvolume nests in a layer:** a snapshot does not descend into
  one, and it blocks the parent's deletion.
- **Never changed once published.** The read-only flag is cleared only
  to delete (7.3).

### 4.2 A layer's life

```
1  snapshot   tree helper   writable snapshot of the parent → layers/<key>.staging
2  steps      sandboxes     build.md, each in a sandbox over the staging subvolume
3  walk       tree helper   against the parent's tree file  → layers/<key>.tree.tmp
4  freeze     tree helper   the read-only flag set; then syncfs(2) on the store, by the builder
5  record     builder       <key>.tree.tmp → <key>.tree; <key>.record written by rename
6  publish    builder       rename layers/<key>.staging → layers/<key>
```

1. **Snapshot:** `BTRFS_IOC_SNAP_CREATE_V2`, writable, by the tree
   helper, since the parent's root is the namespace root's (5.1). The
   builder reads the new uuid and parent uuid
   (`BTRFS_IOC_GET_SUBVOL_INFO`).
2. **Steps** run in sandboxes (build.md, section 8). The last one's
   process tree is proved empty before the walk, and nothing else holds
   the staging subvolume, so nothing changes between walk and freeze.
3. **Walk:** the tree helper writes the tree file and answers the tree
   digest, the change set's digest, counts, and the scan's findings
   (build.md, section 9), any of which fails the layer.
4. **Freeze and sync:** `BTRFS_IOC_SUBVOL_SETFLAGS` with
   `BTRFS_SUBVOL_RDONLY`, then `syncfs(2)` on the store's filesystem.
   Records are synced as they are written (6.8), but the files the steps
   wrote are not; without the sync, a power cut could leave torn files
   in a published layer under a record that claims its tree digest.
   `syncfs` is one call for the whole layer, and an ask of skein
   (README.md, 8.1).
5. **Record:** the tree file renamed into place, then the record, which
   names the tree file's SHA-256 and the uuids (6.8).
6. **Publish:** a rename within `layers/` (2.3). A layer is published
   when `<key>` is a read-only subvolume whose uuid `<key>.record`
   names.

A failed step, a finding, a limit or a stop discards the staging
subvolume (7.3) and publishes nothing.

### 4.3 Unchanged: path, inode and ctime

- **The rule:** an entry whose path, inode number and change time (to
  the nanosecond) all equal the parent's record is unchanged, and keeps
  the parent's covered fields unread. Every other entry is read in full,
  and a file is hashed.
- **It holds** because a btrfs snapshot keeps every inode number and
  change time (verified: same `st_ino`, ctime and mtime after `btrfs
  subvolume snapshot`; btrfs-subvolume(8)), and because every change to
  a covered field moves the change time: a write, `chmod`, `chown`,
  `setxattr`, a link, a rename.
- **Caveats:** compare inode numbers, never `st_dev`, which differs per
  subvolume. On trixie's 6.12 the change time is coarse (btrfs has
  multigrain timestamps from 6.13), which is harmless: the parent was
  frozen before the child's steps began. A freed inode number may come
  back, with a new change time. A link added to an unchanged file moves
  its change time, so both paths are read again.
- **Nothing cheaper exists unprivileged:** `btrfs subvolume find-new`
  needs `BTRFS_IOC_TREE_SEARCH`, which needs root.
- **The local index** (inode, change and modification times) sits in
  the tree file beside each entry, outside the digest (6.3). It is this
  machine's: a received layer gets its own from its first walk (10.2).

### 4.4 Crash recovery

The startup scan (8.2) settles what a crash left:

| Found | What happened | The scan |
|---|---|---|
| `<key>.staging` | a build stopped | deletes it (7.3) |
| `<key>.trash` | a deletion stopped | finishes it |
| `<key>` with no record | a deletion began (7.3, step 1) | deletes it |
| a record with no `<key>` | a publish stopped before its rename | deletes the record and tree file |
| a record whose uuids are not the subvolume's | the subvolume is not the one recorded | deletes both |
| `cache/<64 hex>.tmp`, `layers/<key>.tree.tmp`; a record's name, then `.skein-` and 16 lowercase hex digits, skein's `Store`'s temporary (6.8); anything in `staging/` | a write or a step in flight | deletes it |
| `cache/<sha256>` with no record | a cut between a cache file's rename and its record, or before the file was durable (6.8) | deletes it |
| `cache/<sha256>.record` with no file | a deletion stopped (7.5) | deletes the record |

An interrupted build is built again when next asked for, and a cache
file deleted so is fetched or rendered again. A temporary is only one of
those shapes, in its own directory: any other name, `cache/notes.tmp`
among them, is a stray, reported and left alone (2.1).

- **A layer that no longer matches its record** is found by a check, not
  by the scan, which reads no file: the activator's verification of what
  it places (activator.md, section 6), a scrub, a peer's walk (10.2).
  It is never a lasting refusal of the key. The builder hashes the layer
  again (5.4), in a `verify` job (steward.md, 3.3); if the tree differs
  from its record, it deletes the layer and every layer built on it
  (7.3) and builds them again; if it does not, the mismatch is the
  activator's to report.

### 4.5 Lineage through `parent_uuid`

- **A snapshot's `parent_uuid` is its source's uuid,** and
  `BTRFS_IOC_GET_SUBVOL_INFO` reads both without privilege.
- **A layer's record keeps both** as read after its snapshot. The
  startup scan checks them against the live subvolume, and the parent's
  record, while it exists, must name the same uuid as the child's
  parent uuid. So the chain of records is the chain of snapshots, and a
  subvolume swapped or made by hand is never taken for a layer.
- **A slot inherits it:** its parent uuid names the output it was
  snapshotted from (slots.md, 5.1).

### 4.6 One build per key

- **A key is built once at a time.** A request for a key in progress
  waits for it; one for a published key finds it.
- **Different keys build at once** within the builder's limits
  (build.md, 8.7); a chain is built bottom-up. On a machine the steward
  gives its builder the pass's jobs one at a time, so one of its own
  outputs builds there at a time, with others' work beside it from M6
  (steward.md, 3.4).

## 5. The tree helper

### 5.1 Why it exists

The builder runs as itself; a layer's files belong to its subordinate
range. So the builder cannot:

- **read every file:** `/etc/shadow` is 0640 and `/root` 0700, both the
  namespace root's;
- **unlink** in the namespace root's directories, so cannot empty a
  layer;
- **change owners and modes,** so cannot apply a change set;
- **snapshot a layer or set its read-only flag:** btrfs allows both only
  to the owner of the subvolume's root (`inode_owner_or_capable`;
  verified, `EPERM` for a user on a subvolume its subordinate root
  owns).

Root of a user namespace that maps the range can do all of it. That is
the tree helper: hearth's own code, `hearth-domain-tree` on
`hearth-service-tree`, in the same binary.

### 5.2 How it starts

- **The builder spawns it,** it unshares a user namespace, and the
  builder maps it with the setuid helpers:

  ```
  newuidmap <pid> 0 <base> 65536  65536 <builder uid> 1
  newgidmap <pid> 0 <base> 65536  65536 <builder gid> 1
  ```

  Then it takes uid and gid 0 in its namespace. mmdebstrap does the
  same dance: a process left outside runs `newuidmap` on the one that
  unshared. In hearth it is skein's, as spawning with an id map
  (README.md, 8.1).
- **The builder's own ids are mapped too, at 65536:** to write the
  store's directories, read the parent's tree file, and give a new
  subvolume's root to the namespace's root. Build steps' sandboxes map
  the range only (build.md, 8.1).
- **It acts beneath an inherited descriptor of the store's root:**
  paths resolved with `openat2` and `RESOLVE_BENEATH |
  RESOLVE_NO_SYMLINKS`, stats with `statx` and `AT_SYMLINK_NOFOLLOW`.
  It follows no link.
- **Its pipes are its only channel.** If it dies, the request in flight
  fails, the staging layer it touched is discarded, and the builder
  starts another.

### 5.3 Its records

**The tree helper does seven things:** walk, hash, apply and empty
trees, and snapshot, freeze or thaw, and delete subvolumes. The last
three are the helper's, not the builder's, because btrfs allows them
only to the owner of a subvolume's root, which is the namespace's root
(5.1). Its records are `hearth-protocol-tree`'s: sized, each request
with a token that its one answer echoes.

| Request | Answer |
|---|---|
| `snapshot` a layer into a new name, writable | `snapshotted` |
| `freeze` or `thaw` a layer | `frozen`, `thawed` |
| `walk` a layer against its parent's tree file, with the scan's rules; write its tree file | `walked`: tree digest, change set digest, the tree file's SHA-256 and size, counts, findings |
| `hash` a layer with no parent: every file read | `hashed`, as `walked` |
| `apply` a change set from a file, contents by digest | `applied`: counts |
| `empty` a layer | `emptied`: entries removed |
| `delete` a layer whole, where `user_subvol_rm_allowed` is set | `deleted` |
| any | `failed`: the path and a typed cause: not found, permission, a bound, a refused entry, no space, a bad input |

- **One request at a time;** a builder may run more than one helper
  (one by default).
- **Big data stays in files:** tree files written by the helper, owned
  by the builder's ids, named in answers by SHA-256; change sets to
  apply written by the builder. The pipes carry summaries.
- **Formats are machines, each its own lib-only crate**
  (`programming-model.md`, section 4), so the helper's domain parses
  nothing:
  - `hearth-protocol-tree`: the decoder of 3.1's encoding, these
    records, and the store's record and tree files, which share the
    frame (6.1); it depends on skein-lib and `hearth-lib` only;
  - `hearth-protocol-accounts`: the lines of `/etc/passwd`, `/etc/group`,
    `/etc/shadow` and `/etc/gshadow`, read from a layer, as typed
    entries: names to ids for apply's owners (5.5), and whether each
    password field is locked, for the scan (build.md, 9.3).

### 5.4 The walk

- **Depth-first with an explicit stack, each directory's names
  sorted,** so entries come out in tree order, merged with the parent's
  tree file read as a stream: a path in both is compared (4.3), one
  only in the walk is a put, one only in the parent is a delete.
- **A changed entry is read whole:** `statx`, `listxattr` and `getxattr`
  for the covered families, the link target, and a file's contents,
  hashed as they stream. Files with more than one link are grouped by
  inode number.
- **The scan's rules come with the request** as data (build.md, 9.3).
  The helper checks them on what changed, since only it can read every
  file, and answers findings, never contents. A rule on a file's lines
  (a password field locked) is judged on the typed entries
  `hearth-protocol-accounts` gives the helper's domain (5.3).
- **`hash` is a walk with no parent.** It verifies a layer against its
  record (a scrub, a damaged tree file) and makes a tree file where
  none exists.
- **Later, the walk clamps modification times.** The reproducible
  retrofit has it set every mtime newer than the layer's time to that
  time as it visits changed inodes, and record the change time read
  after the clamp, so 4.3's rule stays sound (reproducible.md, 5.1). No
  digest changes: times are left out (3.4).

### 5.5 Apply, empty and the subvolume operations

- **Apply** takes a change set in tree order: deletes first, deepest
  first; then puts, parents first. A file is written to a temporary
  name in its directory and renamed over its path, its contents from
  `cache/<sha256>` (cloned with `FICLONE` where it can be, to verify) or
  from a peer's stream (10.2). For each put: owners first (a `chown`
  clears setuid, setgid and capabilities), then the mode, then
  `security.capability`, then ACLs and `user.*`; covered attributes the
  entry does not list are removed. It refuses a non-canonical path, an
  owner past 65535, a device (no `mknod` in a user namespace), a socket,
  an uncovered attribute. It does not hash: the walk after it does, and
  the tree digest is the check.
- **Uses in M2:** the empty layer's root; the config layer's file tree,
  owners resolved by name from the layer's own `/etc/passwd` and
  `/etc/group` as `hearth-protocol-accounts` decodes them (build.md, 7.3),
  and hearth's own sysusers, tmpfiles and preset files (build.md, 7.2 to
  7.5); single-file downloads (build.md, 6.2).
- **Empty** removes everything beneath a layer's root, deepest first, so
  the builder can `rmdir` it (7.3). Stopped half-way, it is done again.
- **Snapshot, freeze, thaw and delete** are `BTRFS_IOC_SNAP_CREATE_V2`,
  `BTRFS_IOC_SUBVOL_SETFLAGS` and `BTRFS_IOC_SNAP_DESTROY_V2`, the
  helper's because a layer's root is the namespace root's (5.1).

### 5.6 Bounds

| Bound | Default |
|---|---|
| entries in a layer | 2,000,000 |
| names in one directory | 262,144 |
| depth of the walk | 256 |
| names held across the walk's stack, in all | 1,048,576 |
| a path; a link target | 4096 bytes each |
| attributes per entry; one value | 64; 65,536 bytes (`XATTR_SIZE_MAX`) |
| a tree file | 1 GiB |
| requests in flight per helper | 1 |

Memory is the sorted names of each directory on the stack, bounded in
all and not only per directory and per level, and the frames in flight:
tree files stream in and out. A bound passed fails the request, naming
the path.

Contents never pass through a domain whole: the helper hashes files as
they stream, and render's static blobs reach `cache/<sha256>` streamed
by its protocol layer (render.md, 2.3), whence apply reads them.

### 5.7 Costs

- **An unchanged entry costs one `statx`;** a changed file, a read and a
  hash. A config layer over 100,000 entries is one pass of `statx` and a
  few files hashed: under a second warm (an estimate, to measure).
- **A packages layer is hashed whole,** at the helper's SHA-256 rate:
  6 seconds for 1.5 GB at 250 MB/s. skein's SHA-256 is byte at a time
  and unmeasured (README.md, 8.1 asks for a fast streaming one).
- **Emptying costs about 15 µs a path** (measured: 3,942 paths in
  0.062 s, unprivileged). A snapshot and a freeze cost constant time.
- **Each request's time, entries, bytes read and hashed** are costs
  (build.md, section 12).

### 5.8 The ioctls, and who makes them

| ioctl | Made by | For |
|---|---|---|
| `BTRFS_IOC_SUBVOL_CREATE` | the builder | the empty layer's subvolume (4.1) |
| `BTRFS_IOC_GET_SUBVOL_INFO` | the builder | a subvolume's uuids and flags (4.2, 8.2) |
| `BTRFS_IOC_GET_SUBVOL_ROOTREF` | the builder | the subvolumes under the store, and nested ones (8.2) |
| `BTRFS_IOC_INO_LOOKUP_USER` | the builder | each subvolume's path (8.2) |
| `BTRFS_IOC_SNAP_CREATE_V2` | the tree helper | a writable snapshot of a parent (4.2) |
| `BTRFS_IOC_SUBVOL_SETFLAGS` | the tree helper | freeze and thaw (4.2, 7.3) |
| `BTRFS_IOC_SNAP_DESTROY_V2` | the tree helper | delete whole, where `user_subvol_rm_allowed` is set (7.4) |
| `FICLONE` | the tree helper | contents cloned from the cache (5.5; to verify) |

All are unprivileged, the first four for any user, the rest for the
owner of a subvolume's root. They are synchronous records of skein's
ring adapter, a closed list (README.md, 8.1), made in two processes:
the builder and its tree helper. Beside them the builder calls
`syncfs(2)` once per layer (4.2), one more ask of the same kind.

## 6. Records

### 6.1 The frame

```
magic      8 bytes   "hearth\0r"
kind       u16 BE    1 store, 2 layer, 3 tree, 4 resolution, 5 cache file, 6 target, 7 state, 8 ring [M5], 9 artifact [M6]
version    u16 BE    the kind's format, from 1
reserved   u32       zero
length     u64 BE    the body's length, checked against the kind's bound first
body       a map of named fields (3.1); for a tree file, a sequence of items (6.3)
```

`hearth-protocol-tree`'s machines decode records; the domain gets typed
values.

**Bounds.** Each kind's bound on its body, and the limits within it, are
configured (`programming-model.md`, section 7), with these defaults:

| Kind | Body | Within it |
|---|---|---|
| 1 store | 4 KiB | — |
| 4 resolution | 4 MiB | 8,192 packages in a closure, 8 source lines, 16 indices a line; a URI, a path or a filename of 4,096 bytes; a suite, a component, a name, a version or an architecture of 256 |
| 5 cache file | 8 KiB | a URI or a path of 4,096 bytes; a git object id of 64 |

A body past its bound, or a field past its limit, is a record of a
version the builder cannot read (6.8). A resolution whose closure would
pass 8,192 packages is an error of kind resolution (render.md, 7.2),
never a record; so is one whose record would pass its bound or a
field's limit, which the store declines to write: it depends on the
inputs alone, a configuration error. The other kinds' bounds are set
with the milestone that writes them, M2 for layers, tree files and
targets.

- **The decoders' own bounds,** which no valid record reaches, hold
  for every kind: nesting 8 deep, a field's name of 64 bytes, a list or
  a map of 8,256 items.
- **A record that is not its version's** is one the builder cannot read
  (6.8): a reserved field that is not zero, a field its kind's version
  does not name, a field it requires missing, a value of another kind.
  A field a later release adds comes with a new version of its kind.

### 6.2 A layer's record

`layers/<key>.record`, small, read at startup:

```
key          digest                 the layer's key
kind         bytes                  empty | packages | software | config
parent       digest | none          the parent's key
inputs       map                    packages: { resolution, part }; software, config: { part }
tree         digest                 the tree digest (3.4)
changes      digest                 the change set's digest (3.5)
tree_file    list [ digest, uint ]  <key>.tree's SHA-256 and size
subvolume    map                    { uuid, parent_uuid | none }, as read after the snapshot
counts       map                    entries, files, bytes, puts, deletes, bytes hashed
scan         digest                 the scan's rules it passed (build.md, 9.3)
placed       list [ digest ]        software only: each download's SHA-256 it placed, and each artifact's tree digest [M6]
tools        list                   [ name, version ]: mmdebstrap, apt, dpkg, the target's tools it ran
costs        map                    per step: wall and CPU time, peak memory, bytes fetched, written, hashed
made         map                    { at: wall time in ns, by: built | received [M6], from [M6]; packages only: how: scratch | reconcile, base: key | none, depth: uint }
```

The key recomputes from `kind`, `parent` and `inputs`, and the startup
scan checks it. `placed` is what collection keeps in the cache for a
kept software layer (7.5), since its part digest names no download. The
activator may read records to check what it places (activator.md,
section 6).

### 6.3 A layer's tree file

`layers/<key>.tree`, large, read to build a child, serve a peer or
check a layer. After the header, items, each `tag: u8`, `length: u32
BE`, then a value:

```
tag 1  entry    list [ entry (3.4), local: list [ inode, ctime s, ctime ns, mtime s, mtime ns ] ]
tag 2  change   list [ op, path ]       a put's entry is the entry item of its path
tag 3  end      map { key, parent_tree, tree, changes, counts, boot }

boot             list, per kernel the layer holds; empty for a container:
  version        the kernel's version, as in usr/lib/modules/<version>
  kernel         the content digest of boot/vmlinuz-<version>
  modules        the subtree digest of usr/lib/modules/<version> (3.4)
  initramfs      the digest of the initramfs's inputs, not of its bytes (build.md, 5.8)
```

- **In tree order;** a change item follows its path's entry item, or
  stands alone for a delete. Both digests are computed from the items,
  and the end item must agree.
- **`boot` is the packages layer's,** computed after its walk (build.md,
  5.8), and copied unchanged by the layers above, which may not touch
  the kernel, its modules or the initramfs's configuration (build.md,
  9.3). slots.md (6.1) reads it to decide when the kernel is unchanged.
- **About 150 bytes an entry,** 15 MB for 100,000 entries (an
  estimate). A packages layer's change set is every entry; a config
  layer's, a few.

### 6.4 A resolution's record

`resolutions/<key>.record`. render.md (section 5) makes resolutions;
the store keeps them. What the packages layer needs from one:

```
key        digest   H("resolution", sources, request, variant, apt), over its inputs (render.md, 5.1)
inputs     map      { sources, request, variant, apt }: the key's values, as the packages part holds them
                      (render.md, 4.2); the architecture is the sources'
sources    list     per source line: list [ uri, suite, components,
                      indices: list [ path, size, sha256 ] ]      kept in cache/: its InRelease first, as
                                                                  [ "InRelease", size, sha256 ], then each Packages
                                                                  file, decompressed, at the path and with the size
                                                                  and SHA-256 of its InRelease entry; each path
                                                                  relative to the suite's directory, dists/<suite>/
closure    list     per package, by name then architecture:
                      list [ name, architecture, version, source: uint, filename, size, sha256 ]
digest     digest   H("closure", list [ list [ name, architecture, version, sha256 ] … ])
made       map      { at, apt, mmdebstrap, costs: { fetched, took } }: `at`, wall time in ns when
                      recorded; apt's and mmdebstrap's versions, empty in M1 (render.md, 5.4);
                      `fetched`, the bytes apt reports fetching; `took`, the run's duration in ns
```

- **The key recomputes from `inputs`,** as a layer's from its record
  (6.2), **and the digest from `closure`;** the startup scan checks
  both, since the scan reads each resolution's record whole anyway, and
  a packages key built on a digest its closure does not give would name
  bytes it does not install. A record whose key or digest does not
  recompute is deleted.
- **Every byte the packages step installs is named by SHA-256 here:**
  each `.deb` by its `Packages` entry, each `Packages` file by its
  `InRelease` entry, each `InRelease` by the digest it had when apt
  checked its signature.
- **The indices are kept, not fetched again.** Debian publishes no
  uncompressed `Packages` file, so the digest an `InRelease` gives for
  one names nothing a mirror serves. Resolution moves each `InRelease`
  and each decompressed `Packages` file, as apt verified them, from its
  scratch directory, `staging/<resolution key>/`, into `cache/<sha256>`
  before the directory is removed: hashed as it is read for its stanzas,
  then renamed, with no copy (render.md, 5.3). The fetch then gets only `.deb`s
  (build.md, 5.1), and the local repository serves the kept indices
  (build.md, 5.3).
- **The digest covers what is installed;** where it came from does not
  enter.

### 6.5 A cache file's record

`cache/<sha256>.record`, kind 5, version 1:

```
sha256     digest   the file's SHA-256: its name
size       uint     its length in bytes
origin     map      where it first came from, one of:
                      { kind: "fetch", uri }                  fetched (build.md, 5.1)
                      { kind: "index", resolution: digest }   an index a resolution kept (6.4), by its key
                      { kind: "blob", oid: bytes }            a static file render streamed from git, by its object
                                                              id as git prints it, lowercase hex
                      { kind: "rendered", path: bytes }       a file render produced, by its template's path in the revision
at         uint     when it entered the cache: wall time in ns
took       uint     how long it took to enter, in ns: the fetch, the stream or the write
```

It is provenance and cost; whether a file is kept depends on who names
it (7.5). A file that enters again, from another origin, keeps its first
record.

### 6.6 The store, targets and state

- **`store.record`,** kind 1, version 1: `layout`, the layout's version,
  1; `uid`, the builder's; `range`, `{ base, count }`, the subordinate
  range its layers are owned by; `made`, wall time in ns. A store without
  layers (2.4) has no `range`; the builder records it, by rename, before
  its first layer [M2]. Once it is recorded, a builder configured with
  another base refuses to start: every layer on disk is the recorded
  range's. So does a builder of another uid, which would write files the
  recorded one cannot replace (`sudo hearth render --store` on a
  developer's store): the store is not opened, and the client's startup
  error (8.1) names both uids.
- **`targets/<H("target", name)>.record`:** the name and the last N
  builds (7.1), newest first, each `[revision, output key, at]`,
  rewritten when a build for the target completes.
- **`state/`:** the steward's records and rings (8.3).

### 6.7 An artifact's record [M6]

`artifacts/<key>.record`, beside the artifact's subvolume and its tree
file, in the same frame (kind 9): its tree digest, environment, sources,
inputs, command, outputs, cache, how it was made and its costs, as
artifacts.md (4.3) lists them. Its key is artifacts.md's (3.1).

### 6.8 Written by rename; versioned

- **Atomic:** a record is replaced whole by skein's `Store` (skein's
  `io.md`, sections 5 and 5.2): its temporary, `<name>.skein-<16 hex>`,
  written beside it, synced, renamed over `<name>`, the directory
  synced. A reader sees the old record or the new. A tree file [M2] is
  written the same way by the builder itself, as `<key>.tree.tmp`.
- **Ordered:** the layer's files synced (`syncfs`) and its tree file
  in place before its record, a record before the publishing rename
  (4.2), an unrecording before the subvolume is touched (7.3). The
  startup scan repairs any prefix (4.4).
- **A cache file is durable before its record.** It is written to
  `cache/<random>.tmp` (2.1), hashed as it is written, synced, and
  renamed to `cache/<sha256>`, in its own directory, so a reader never
  sees half a file under a digest; then its record is written. Several
  files may be made durable with one `syncfs` before their records [M2].
  A file whose name is already present is not written again: its
  temporary is removed. An index a resolution keeps is synced where apt
  left it, renamed from its scratch directory into `cache/<sha256>`
  (skein's `io.md`, 5: a rename between two directories), and `cache/`
  synced, before its record (render.md, 5.3). A cut anywhere leaves a
  temporary, or a file with no record, which the startup scan deletes
  (4.4).
- **Versioned per kind, across releases** (activator.md, 7.2's rule): a
  release reads its own version of each kind and the previous one, and
  writes only versions the previous release can read. After a fall back,
  the blessed slot's older builder and steward read what the newer ones
  wrote.
- **An unknown version never stops the builder.** A layer, tree file,
  resolution, cache file or target record of a version it cannot read is
  treated as absent: the layer deleted (7.3), the rest made again, which
  costs time and breaks no slot, since slots are copies. A
  `store.record` it cannot read is written again, and every layer it
  holds is treated as unrecorded; so is one whose `layout` is not 1.
  A resolution it cannot read stays in place, unindexed, until a
  resolution of its key is made again over it. `state/` records follow
  8.3's table.

## 7. Pins and collection

### 7.1 What pins a layer

| Pin | From | Lasts |
|---|---|---|
| the empty layer | the store | always |
| the outputs of the machine's slots [M3], its containers' included [M5] | the activator's facts over the link (activator.md, 4.3), passed on by the steward in the jobs' `Welcome` and `Pins` (steward.md, 3.2) | while a slot holds it, or is prepared from it |
| the desired revision's outputs [M3]: the machine's target's, and its containers' [M5] | the steward's `hearth-domain-converge` (wire.md, section 4), in the same records | while it is desired |
| builds in flight | the plan (build.md, section 4) | until the build ends |
| a target's last N outputs | `targets/` (6.6); N is 3 by default | until N newer builds |
| a peer's request [M6] | a lease per transfer (10.3) | until it ends or its deadline |
| an artifact in use [M6] | artifacts.md, 4.4 | as it says |

- **A pin holds a layer's ancestors too.** A child does not need its
  parent's subvolume (snapshots share extents, not existence), but the
  target's next build starts from it and a peer's change set is against
  it. An ancestor costs only what its descendants changed or deleted.
- **A builder collects nothing before it has the steward's pins,** in
  its `Welcome` (steward.md, 3.2): the activator's slots and the desired
  revision's outputs. The steward spawns no builder that may collect
  before its activator's `Welcome` has given it the slots' outputs.

### 7.2 Collection

- **What no pin holds is collected:** after each publish, at a
  builder's start once the pins are known, and when the store's
  filesystem falls below its free-space floor (`statfs`), which the
  steward watches between builders and answers with a `collect` job
  (steward.md, 3.3).
- **Oldest first, bounded per run.** What a deletion frees is not known
  in advance: without quotas, which need root, btrfs does not say what a
  subvolume holds alone.
- **Each deletion is reported** with its reason (wire.md, section 6).

### 7.3 Deleting a layer

```
1  unrecord    builder       unlink layers/<key>.record; sync the directory
2  retire      builder       rename layers/<key> → layers/<key>.trash
3  thaw        tree helper   clear the read-only flag
4  empty       tree helper   remove every entry, deepest first
5  rmdir       builder       rmdir layers/<key>.trash: an empty subvolume, since Linux 4.18
6  tree file   builder       unlink layers/<key>.tree
```

- **Unrecorded first,** so a crash at any later step leaves an
  unrecorded subvolume or a `.trash`, which startup deletes.
- **Thawed first:** nothing in a read-only subvolume can be unlinked.
- **Emptied in the namespace:** the builder cannot unlink in its range's
  directories. `rmdir` of an empty subvolume checks only the parent
  directory, the builder's. A staging layer is deleted from step 3.

### 7.4 `user_subvol_rm_allowed`

- **With it, steps 4 and 5 are one `delete`** by the tree helper
  (`BTRFS_IOC_SNAP_DESTROY_V2`): constant time for the caller, the
  extents freed by btrfs's cleaner. The flag must be cleared first: a
  read-only subvolume refuses the write permission deletion checks
  (`EROFS`).
- **Only on builders and test filesystems.** The builder sees the option
  in its mount and uses it where set.
- **The caveat.** It is per filesystem (the first mount's options apply
  to every subvolume), and lets any user with write and search
  permission on a subvolume's root delete it whole, with others' files
  and immutable or append-only ones inside: the kernel's comment says
  it checks neither emptiness nor what could otherwise be deleted. On a
  machine whose slots and data share the filesystem with local users,
  that widens what a user can destroy. Incus sets it on its pools by
  default; hearth does not ask for it where slots run.

### 7.5 Resolutions and the cache

- **A resolution is kept** while a kept packages layer names its digest,
  and the newest per target for a configured time.
- **A cache file is kept** while a kept resolution names it (its
  `.deb`s and its kept indices), while a kept software layer's record
  lists it in `placed` (6.2), while a build in flight uses it (render's
  blobs among them), and for a configured time after its last use, so a
  rebuild soon after a collection fetches nothing.
- **Deleting one** unlinks its name and record; a staging link keeps the
  inode until its step ends.
- **Nothing is collected in M1.** A store without layers grows with
  each resolution and each rendered or streamed file, and is full at a
  limit of 8.1; its user deletes it whole (2.4).

## 8. hearth's own state

### 8.1 The filesystem is the only truth

- **Subvolumes and the records beside them are the builder's state,**
  read back at each builder's start: no kv store, no index file to
  disagree with the disk.
- **Only in memory:** builds in flight (their staging is deleted) and
  leases. The queue of jobs is the steward's (steward.md, 3.4). A build
  asked for again finds or redoes its layers.
- **The in-memory index** holds one entry per published layer (key,
  kind, parent, tree digest, uuid, counts, when), bounded by the store's
  layer limit (4,096 by default); one per resolution, by key, bounded by
  its resolution limit (1,024); and one per cache file, by digest,
  bounded by its cache limit (262,144 files). The three limits are
  configured (`programming-model.md`, section 7). Tree files are never
  held whole, and the scan reads no cache file's record: it pairs files
  and records by name, and a record is read when its file is used.
- **A store at a limit is full.** A scan that counts more resolutions or
  cache files than their limits stops there: the builder holds no index,
  and answers every request that reads or adds to the store with an
  operational error of kind `store` naming the store, the limit and what
  passed it (render.md, 7.2). Only a cache file with its record counts
  (8.2, step 6). A resolution is not begun without room for one more
  resolution and for every index its source lines may keep (128 cache
  files by default, 6.1), so a full store runs no mmdebstrap; a
  resolution or a cache file that would still pass its limit while the
  builder runs is refused the same way, and its file is removed. Nothing
  is evicted to make room: collection is M2's (7.5), and from M2 a
  builder collects before it calls its store full. A store without
  layers that is full is deleted whole and made again (2.4).
- **A store that cannot be opened ends the call.** A lock or a
  `store.record` that fails, or a `store.record` the builder refuses
  (6.6), keeps the builder from starting: nothing renders, and the
  client says so on standard error, with the code 75 (client/README.md,
  3.2 and 3.3).
- **A store whose upkeep fails is refused as a full one.** A page of a
  listing, a removal, a record's load, a `staging/` tree past the walk's
  bounds (5.6) or an index's keeping (6.8) that fails leaves the builder
  with no index: it answers every request that reads or adds to the
  store with an operational error of kind `store`, naming the store and
  the step that failed, and leaves the store as it is for the next
  start's scan.
- **A store's error is the call's,** given once, not each target's: a
  resolution the store would not find or keep leaves its key needed
  (render.md, 5.7), and the call's code is 75 (client/README.md, 3.3).

### 8.2 The startup scan

It runs at every builder's start, and the steward spawns a builder for
work (steward.md, 4.2), so what a crash, a kill or a power cut left is
settled before the first job of every pass.

1. **The lock:** an exclusive `flock` on the store's root directory,
   taken without waiting (skein's `io.md`, 5: `Lock`) and held until the
   builder exits. A second builder over the same store, a daemon's or a
   client's in-process on a developer's store, waits for it
   (steward.md, 3.4): while the lock is busy it asks again every 200 ms,
   and says once, after its first second of waiting, that it waits (on
   the client's standard error, client/README.md, 3.5; as an event of
   its job [M3]). It sets no limit of its own: the person at the client
   may interrupt it, and a daemon's job has its steward's deadline
   (steward.md, 5.2). The root, not `store.record`, which 6.8 may write
   again by rename onto another inode.
2. **The store's record** (6.6): version, layout, uid and range.
3. **Every subvolume under the store,** with
   `BTRFS_IOC_GET_SUBVOL_ROOTREF` on its root: the child subvolumes'
   tree ids and the directories holding them, 255 a call, continued from
   the last tree id. `btrfs subvolume list` needs root; this does not.
4. **Each one's path,** with `BTRFS_IOC_INO_LOOKUP_USER`, which checks
   read and search permission on each component: all the builder's.
5. **Each one's identity,** with `BTRFS_IOC_GET_SUBVOL_INFO`: uuid,
   parent uuid, read-only flag, generation.
6. **Matched against the records:** 4.4's cases, 4.5's uuids, each key
   recomputed, a resolution's among them (6.4), and each cache file with
   its record. A subvolume inside a layer (`GET_SUBVOL_ROOTREF` on the
   layer) is refused, and the layer deleted. A store without layers runs
   this step for its resolutions and its cache alone (2.4). `cache/`,
   `resolutions/` and `layers/` are listed page by page (skein's
   `io.md`, 5: `ListPage`), 1,024 entries a page, so a directory of any
   size is scanned in bounded memory. Resolutions count against their
   limit (8.1) as they come. `cache/` is listed twice: its records
   first, counted against the cache limit as they come, then its files,
   each paired with its record; a file with none is deleted (4.4) and
   never counts, so what a crash left cannot fill a store.
7. **Settled:** deletions done (7.3), strays (any name 2.1 does not
   list) reported once, by their count and the first found, and the
   index built. `state/` is the steward's, read at its own start (8.3).

What the store says while it opens, that it waits for the lock and that
it holds strays, goes to the client's standard error as progress does,
as text or as records with `--json` (client/README.md, 3.2; as events
of its job [M3]); its error is part of the answer, unless it kept the
store from opening (8.1).

The three ioctls have been unprivileged since Linux 4.18, and the
builder makes them itself (5.8).

### 8.3 The steward's records

What the steward must keep across restarts and reboots lives in
`state/`, which only the steward writes and no builder opens
(steward.md, 2.2), each file named by `H("state", name)`, each bounded:

| Record | Holds | Bound | Written | Of a version it cannot read | Defined in |
|---|---|---|---|---|---|
| desired | target, repository, a commit or a branch with its interval, `trial: auto \| manual`, who set it and when, a generation | one | by rename, when the client changes it | reported; the running slot stays until the client sets a revision again | wire.md, 4.1 |
| refused | output keys refused, each with why and when | the last 16 | by rename | discarded, reported; a fall back is refused again as the activator reports it | wire.md, 4.4 |
| hold | who held (a peer's uid), when, the reason; or none | one | by rename | held, reported, until `hearth release` | wire.md, 4.5 |
| pass | the last pass: its generation, commit, stage, output and prepare keys, and its outcome | one | by rename, at each stage | discarded; the bless's render then takes the desired revision, its commit or the branch's head in the clone | wire.md, 4.2; steward.md, section 6 |
| orders [M4] | each order's key, attempt and stage, then its outcome, until the engine acknowledges it; a restart's unit `InvocationID` | 64 outcomes waiting (to start) | by rename, before the effect is asked | discarded; the engine sends again what it has not seen answered, and orders are keyed or conditional | wire.md, 5.4 and 5.5 |
| signals [M5] | CPU, memory, pressure, load, disk use per mount | three rings: 10 s for 6 hours, 1 minute for 3 days, 15 minutes for 60 days | in place, slot by slot | discarded, started afresh | operating.md, section 5 |
| check history [M5] | each check's results | 1,024 per check, a ring each | in place, slot by slot | discarded, started afresh | operating.md, section 5 |

- **Records are written by rename** (6.8), as every record is. An order
  is recorded before the activator is asked for its effect, so a
  restart finds it begun and resumes it under its key or its condition.
- **Rings are written in place.** A ring is a file of fixed-size slots,
  its size computed at startup, with a header in the frame of 6.1 (kind
  8). Rewriting a whole ring every 10 seconds would cost more than the
  sample, so each slot carries its sequence number and the first 8 bytes
  of its SHA-256, and a torn slot is skipped on reading: a power cut
  loses at most the sample being written.
- **Nothing here can be made again,** unlike the rest of the store, so
  6.8's rule matters most here: a newer release writes these in versions
  the previous one reads, and an unreadable one is settled as the table
  says, never by stopping. The fall back that brings an older steward
  back is when convergence matters most.

## 9. Prerequisites

### 9.1 btrfs and the mount

- **The store is a btrfs subvolume of its own:** `@hearth.store` on a
  machine, mounted at `/var/lib/hearth/store` in every slot (machine.md,
  3.2 and 3.3), or one a developer makes with `btrfs subvolume create`,
  which needs no privilege, and names with `--store` (client/README.md, 3.5). The scan lists the store's child subvolumes, so the
  store must be their parent. Only layers need it: a store that only
  renders may be a plain directory (2.4).
- **Linux 4.18 or later,** for unprivileged `rmdir` of an empty
  subvolume (`/sys/fs/btrfs/features/rmdir_subvol`) and 8.2's ioctls.
  Trixie's 6.12 has them.
- **Mount options:** `nodev,nosuid,noatime` on the store's own mount
  (machine.md, 3.2), all per mount: no setuid file in a layer works from
  the store, and reading a layer writes nothing; `user_subvol_rm_allowed` (per filesystem) on
  builders and test filesystems only (7.4).

### 9.2 The builder's user and its subordinate range

- **User `hearth`,** a system user, the steward's and the builder's,
  with one subordinate uid range and one gid range of 65536 ids, at a
  base aligned to 64 Ki: the activator's shift is then a mask
  (slots.md, 5.2), as systemd's shifter assumes.
- **Made explicitly.** `useradd -r` gives a system user no range unless
  `-F` (`--add-subids-for-system`), and then from `SUB_UID_MIN`, 100000,
  which is not aligned. So: `usermod --add-subuids <base>-<base+65535>
  --add-subgids <base>-<base+65535> hearth`, which checks nothing; or
  `useradd -r -F -K SUB_UID_MIN=<base>` (to verify).
- **Base 262144 (0x40000) by default:** aligned, and below
  systemd-nspawn's automatic range (524288 to 1879048191, systemd's
  `UIDS-GIDS.md`), which hearth's containers may use [M5] only with
  idmapped mounts: a shift into a container's own range needs its base
  at prepare, before nspawn would pick one (slots.md, 12.1 and 12.4).
  machine.md (section 8) sets it at bootstrap.
- **The range is the builder's alone.** Whoever runs as its base owns
  every layer, so no other user, container or nspawn range may overlap
  it. hearth refuses a range missing, short, overlapping or in more
  than one entry, and a store made with another (6.6); a host's
  activator refuses a container whose range overlaps it [M5]. The
  fleet's containers clear it today: carbon-forge's base is 65536,
  rhi-chat's 327680, the first id past the builder's range, and
  vul-agent-control has no user namespace.
- **Aligned where there is an activator.** Only the shift needs the
  alignment, so a machine with an activator refuses an unaligned range,
  while a builder alone (a developer's machine, a lab builder, the real
  tier of testing.md, section 6) accepts the range `useradd` gave it.

### 9.3 Namespaces and tools

- **Unprivileged user namespaces:** Debian's default
  (`kernel.unprivileged_userns_clone=1`). Ubuntu since 23.10 needs an
  AppArmor profile with `userns,` or
  `kernel.apparmor_restrict_unprivileged_userns=0`. An Incus container
  needs `security.nesting=true` (to mount `/proc` and `/sys`) and
  `security.idmap.size` above 65536.
- **`newuidmap` and `newgidmap`** (Debian's `uidmap`), setuid root.
- **A cgroup v2 subtree the builder can write,** for its sandboxes'
  limits: on a machine, `steward/builder/`, which the steward makes in
  the subtree the activator gives it, `steward/`, under its own
  delegated cgroup (`Delegate=yes`; steward.md, section 7); on a
  developer's machine, the client's own scope, run with `systemd-run
  --user --scope -p Delegate=yes`. What build does with it is build.md,
  8.7.
- **The build's tools** (build.md, section 10): mmdebstrap, apt, dpkg.
  On a machine, hearth's own task installs them in every target that
  builds.

### 9.4 Where the store lives

- **Reachable by `hearth` alone,** and of its processes only the
  builder and its helpers open a layer (steward.md, 2.2). A layer holds
  setuid files owned by the range: `su` is 4755, owned by the base.
  Whoever could reach and run one would run as the base, which owns
  every layer, and could rewrite an output before it becomes a slot.
  Guix met this (CVE-2024-52867: setuid outputs reachable before they
  were fixed up), and systemd warns of its foreign range the same way.
- **So it is mounted in one place only,** `/var/lib/hearth/store` in
  every slot, from the fstab the activator writes (machine.md, 3.3),
  with `nodev,nosuid,noatime`; its root is 0700 and owned by `hearth`,
  so no other user of the machine reaches anything beneath it.
- **No task places anything beneath it:** `/var/lib/hearth/` is
  reserved (format.md, 5.11). It is never bound into a container, nor into a
  sandbox but for the one layer a step builds.
- **Read-only is no barrier to the builder,** which can clear the flag;
  the activator snapshots an output before verifying it (activator.md,
  section 6).

## 10. Peers [M6]

### 10.1 What moves

- **A layer moves as its change set against its parent,** by layer key:
  `GET /v1/change-set/<layer key>` (wire.md, section 8). Not btrfs send
  and receive: both need root.

  ```
  header    the layer's record (6.2): key, parent, tree digest
  items     in tree order:
              delete  path
              put     entry; for a file whose content digest is not the parent's at that path,
                      its contents, streamed; its modification time, advisory
  end       the change set's digest
  ```

- **Contents travel only where the parent lacks them;** the receiver's
  snapshot has the rest.
- **Modification times travel outside the digest,** so a received tree
  keeps them: Python's byte-compiled files record their sources' times.
- **Downloads move by SHA-256,** `GET /v1/blob/sha256/<digest>`
  (wire.md, section 8), checked as they arrive.

### 10.2 Receiving

1. **The parent first:** a receiver that lacks it fetches its change set
   the same way, down to a layer it holds; every chain ends at the empty
   layer, which every store has.
2. **Snapshot** the parent into `<key>.staging` (4.2).
3. **Apply** the stream with the tree helper as it arrives (5.5).
4. **Walk** against the parent's tree file, hashing every file written.
5. **Check** the tree digest against the one the receiver's own engine
   gave for the key (wire.md, section 8). A mismatch deletes the
   staging layer and is reported.
6. **Freeze, record** (`made.by` received, with the peer) **and
   publish.**

### 10.3 Serving

- **A machine serves the layers it holds,** and its cache by digest:
  the steward holds the HTTP listener, and the bytes come from a
  builder's job (steward.md, 2.1 and 3.3). The tree helper streams the
  change set from the tree file and the layer, since only it can read
  every file.
- **A request leases the layer and its parent** until it ends or its
  deadline passes (7.1), within a bound on transfers at once and per
  peer.

### 10.4 Trust per plane

- **A machine fetches only from its own plane's builders and caches,**
  and takes tree digests only from its own engine, over its
  authenticated wire: a lab output never reaches production.
- **A receiver cannot check a key against a tree** until builds are
  reproducible; it trusts its plane's builders for that, and checks the
  bytes against the digest (reproducible.md, section 7).

## 11. Its world

- **`hearth-domain-store`'s world** runs on the fake machine (testing.md,
  section 3): subvolumes and snapshots that keep inode numbers and
  change times, read-only flags, ownership checks that refuse the
  builder's snapshots and flags and allow the namespace root's, `rmdir`
  of an empty subvolume, the three ioctls, the mount option. The tree
  helper is a fake with the real one's records, acting on the fake
  machine's files as the namespace sees them.
- **Faults from the seed:** a power cut at every step of 4.2 and 7.3; a
  full disk mid-walk or mid-apply; the helper killed mid-request; a
  subvolume swapped behind its record; the steward's pins late; a
  second builder over the same store.
- **Invariants** (testing.md, section 8):
  - every published layer's key recomputes from its record, its uuids
    match its subvolume, and its tree digest is the one the world gets
    walking the fake tree on its own;
  - after any crash and a builder's start, no staging, trash or
    temporary file remains, and every record has its layer;
  - no two builders hold the store's lock at once;
  - collection never deletes a pinned layer or an ancestor of one, nor
    runs before the pins are known;
  - no layer holds a nested subvolume, an owner outside the range, or
    the builder's ids;
  - a change set applied to its parent gives the child's tree digest.
- **Step tests** hold the codec to 3.2's vectors and tree order to
  hand-made cases.
- **Real builds** (testing.md, section 6), as a regular user on btrfs:
  snapshots keep inodes and change times, the walk reads what the
  builder cannot, layers are deleted with and without the option.

## 12. What is open

- **Whether `useradd -r -F -K SUB_UID_MIN=262144` makes the default
  range,** or bootstrap keeps `usermod --add-subuids`; depends on
  machine.md's bootstrap (section 8).
- **`FICLONE` in apply,** one more ioctl for skein's ring adapter;
  depends on measuring copies of large files.
- **Hashing speed:** skein's SHA-256, or a tool, for whole packages
  layers; depends on README.md 8.1's first ask and a measurement.
- **Several `statx` in flight for the walk;** depends on measuring the
  walk on the real tier against skein's file machine, which runs one
  request at a time.
- **Finishing an interrupted publish** (a frozen staging layer whose
  record is written) rather than deleting it; depends on how often power
  cuts land there.
- **The pins' and collection's defaults:** N per target, the free-space
  floor, how long unused resolutions and cache files stay; depends on
  the real tier's and the lab's costs.
- **Inode flags in a change set:** whether no-copy-on-write should
  travel; depends on whether a received journal directory ever matters.
- **A per-directory digest** (a Merkle tree) for cheap comparisons of
  subtrees between machines; depends on peers [M6].
- **Rules of the scan that change:** a layer recorded with older rules
  is scanned again from its tree file, with the helper for content
  predicates, before it is used as an output; depends on build.md, 9.3.
- **Trusting a received key [M6]:** whether a receiver ever checks a
  layer by building it; depends on reproducible builds.
