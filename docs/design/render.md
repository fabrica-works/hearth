# Render

Provisional, 2026-10-09. How hearth turns a revision of an IaC repository
and a target into a spec: what render reads and how it reads it, its
pipeline, the spec's parts and their digests, package resolution, the
rendered diff of a change, its errors, its crates and how it is tested.
The format it reads is format.md. Milestone M1 builds it, with the
client's `render` and `diff` (wire.md, section 3). Open questions are in
section 10.

## 1. In one page

- **Render is a pure function of a revision and a target,** exports
  included. It reads nothing but the revision: no secrets, no machine, no
  clock, no network. Any hearth renders any target, in full, in
  milliseconds, so nothing about a spec is remembered but its digests.
- **The revision is read through git,** a tool behind a protocol machine
  (`hearth-protocol-git`): one listing, then the blobs render needs, every
  size bounded before a byte is read.
- **Eight steps:** decode, compose and merge, validate, evaluate exports,
  evaluate conditions, render files, detect collisions, assemble. The
  protocol layer decodes JSON and templates into typed values and
  instruction sequences; the domain never parses.
- **A spec is parts with digests.** Output parts (packages, software,
  config) enter layer keys; slot parts (data, secrets, boot) never do;
  checks, exports and live are the others; meta names the target. Each
  digest is SHA-256 over store.md's canonical encoding.
- **Resolution is the one slow step,** and it is remembered.
  `mmdebstrap --simulate --mode=chrootless` resolves a packages part to a
  pinned closure (each package's name, version, architecture, archive
  path, SHA-256 and size), unprivileged, once per key; the closure is a
  record in the store.
- **The rendered diff is what a change's reviewers read:** per target and
  per part, typed, then printed, with the targets it touches and the
  layers each must rebuild. Who posts it is open.
- **Errors are values,** each with its kind, its file, line and column in
  the revision, and its target and task. Render collects every
  independent error, not just the first.
- **Tested from day 0:** golden repositories of hearth's own, the merge
  laws and determinism as properties, every decoder fuzzed, the template
  engine held to MiniJinja, and fakes of git and mmdebstrap kept honest
  by conformance. os's goldens inspire cases; they are not an oracle.

## 2. Inputs

### 2.1 What render reads

- **A repository:** a git directory, of two kinds:
  - **a local one,** which `hearth render` and `hearth diff` name with
    `--repo` or a path: the client renders it in its own process, with
    its own user's access, and never hands the path to a daemon
    (wire.md, section 3);
  - **the builder's own clone,** `@hearth.store/repo/`, which the
    daemon's builder fetches and renders in jobs its steward gives it,
    and the only repository the daemon renders (wire.md, section 4;
    steward.md, 3.3).
- **A revision:** a commit id, or a ref name that render resolves to a
  commit id once, first. Everything after names the commit.
- **Targets:** one, several, or every target of the revision.
- **Nothing else.** No overlay, no workspace, no environment variable, no
  file outside the revision. Resolution alone reads more: the store's
  records and the archive (section 5).

### 2.2 Git as a tool

```
git --git-dir=<repository> rev-parse --verify --quiet <revision>^{commit}
git --git-dir=<repository> ls-tree -r -l -z --full-tree <commit>
git --git-dir=<repository> cat-file --batch
```

- **The environment is given whole:** `LC_ALL=C`, `GIT_CONFIG_NOSYSTEM=1`,
  `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_NO_REPLACE_OBJECTS=1`,
  `GIT_TERMINAL_PROMPT=0`, and `HOME` an empty directory. No user's or
  system's configuration changes what git prints, and a replace ref
  cannot change a blob behind a commit id.
- **The revision is checked before it reaches git:** a commit id of 40 or
  64 hex digits, or a ref name of `[A-Za-z0-9._/-]`, at most 255 bytes,
  never starting with `-`.
- **`ls-tree` gives the listing:** one record per blob, `<mode> <type>
  <oid> <size>\t<path>\0`. Modes `100644` and `100755` are files (the
  second executable), `120000` a symbolic link whose blob is its target,
  `160000` a submodule, which is an error under a path render reads.
- **`cat-file --batch` gives blobs,** one process per revision read:
  render writes `<oid>\n` and reads `<oid> <type> <size>\n`, the bytes,
  and `\n`. The header's size must equal the listing's and fit the
  blob's kind (2.3) before render demands the body.
- **`hearth-protocol-git` builds the vectors and parses the outputs** into
  typed records; the domain sees a listing, decoded documents and
  templates, and the digests of static files, never git's text (README,
  section 4.2).

### 2.3 Bounds

Render's limits are configured (`programming-model.md`, section 7); these
are the defaults.

| Limit | Default |
|---|---|
| entries in a revision's listing | 65,536 |
| a path | 4,096 bytes |
| a JSON document; its depth; keys in an object | 1 MiB; 32; 4,096 |
| a template, or a `task.json` string that is an expression | 1 MiB; 4 KiB |
| a static file, or a `file` setting's target: streamed to the cache, never held | 64 MiB |
| JSON documents and templates decoded from one revision, held in memory | 64 MiB |
| targets in a revision; tasks; settings in a task | 1,024; 1,024; 512 |
| nesting of statements in a template; of an expression | 32; 64 |
| entries a declaration expands to | 65,536 |
| loop iterations in one evaluation | 1,048,576 |
| a rendered file, held until it is in the cache; bytes rendered for one target | 16 MiB; 256 MiB |
| errors kept from one render | 256, then a count |

Exceeding a limit is an error of the revision, with the path at fault.

### 2.4 The builder's clone and its fetch

- **The clone is bare, at `@hearth.store/repo/`,** a name of no digest
  shape that store.md's scan knows (store.md, section 2). It is made once,
  `git init --bare --quiet <store>/repo`, and read with 2.2's commands.
- **The fetch** is the one git command that reaches the network, as
  `apt-helper` is build's (build.md, 10.2). The builder runs it in a
  `fetch` job from its steward (steward.md, 3.3). Its argument vector,
  with the repository's URL and the branch the machine follows, which
  the job carries from the steward's desired revision (wire.md, 4.1):

  ```
  git --git-dir=<store>/repo fetch --quiet --no-tags --no-recurse-submodules --prune
      <url> +refs/heads/<branch>:refs/hearth/<branch>
  ```

  A desired commit must then be reachable from `refs/hearth/<branch>`;
  render resolves it there, or a commit imported from a bundle from its
  own `refs/hearth-dev/<commit>` (wire.md, 4.1; development.md, 6.2).
- **Its environment is 2.2's,** given whole, plus the credential's one
  variable. The URL never carries a credential.
- **The read credential is the builder's own,** never a secret of the
  secret store: a forge token or a deploy key kept in the store under
  `credentials/`, 0700, user `hearth`, imported at bootstrap
  (machine.md, section 8), and given to git by a fixed variable:
  - over HTTPS, `GIT_ASKPASS=/usr/local/lib/hearth/askpass`, a program the
    hearth task ships that prints `hearth` for a username and the token
    in `<store>/credentials/token` for a password;
  - over SSH, `GIT_SSH_COMMAND=ssh -F /dev/null -i <store>/credentials/deploy_key
    -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes
    -o UserKnownHostsFile=<store>/credentials/known_hosts`.
- **Bounded:** the deadline the steward sets on the `fetch` job
  (steward.md, 5.2), and the store's free space; git's progress is not
  parsed, only its exit and the tail of its errors.

## 3. The pipeline

```
revision ──git──▶ 1 decode ──▶ 2 compose ──▶ 3 validate ──▶ 4 exports ──▶ 5 conditions
                                                                              │
                  spec ◀── 8 assemble ◀── 7 collisions ◀── 6 files ◀──────────┘
```

Step 1 and discovery run once per revision; the rest once per target,
and step 4 reads other targets of the same environment.

### 3.1 Decode

- **The listing classifies every path** (format.md, section 2): a
  configuration document (`fleet.json`, `envs/*.json`,
  `hosts/<base>/*.json`), a declaration (`tasks/*/task.json`), a template
  (`*.j2`), a static file under `tasks/*/tree/`, or a file outside the
  four roots, read only if a `file` setting names it. A path under the
  four roots that format.md, section 2 does not allow is an error.
- **The domain asks for blobs by kind,** and the protocol layer decodes
  each before it goes up:
  - JSON through skein's tokenizer into `hearth-protocol-config`, which
    refuses duplicate keys, non-integer numbers and U+0000, drops
    `_note`, and gives typed values, each with its line and column;
  - a declaration into its fixed shape (format.md, 5.1), its `if` and
    `for_each` fields and its interpolated strings passed through
    `hearth-protocol-template`;
  - a template through `hearth-protocol-template` into a flat sequence of
    instructions (8.3);
  - a static file, or a file a `file` setting names, is never decoded
    and never held: the protocol layer streams its blob from `cat-file`
    into the store's cache by SHA-256 (store.md, section 2), hashing as
    the bytes pass with lib's SHA-256 (asked of skein: README, 8.1), and
    the domain receives its object id, size and digest.
- **A decode error fails what depends on the file,** not the revision:
  the targets that have a broken task, or that compose a broken document.

### 3.2 Compose and merge

- **Discovery** makes the targets from the listing and the bases'
  `members` (format.md, section 3): base, member, environment, name. Two
  targets of one name are an error.
- **Each target's layers** are merged in order (format.md, 3.2) by
  RFC 7386 (format.md, 4.1): task defaults, fleet, base, member,
  environment, base in the environment, its member.
- **Presence** comes from the base, member and base-env layers alone
  (format.md, 3.3), and the `hearth` task is added to every target but
  a container (format.md, 5.12); the composed settings are restricted to
  the present tasks.
- **Every value keeps its provenance:** the layer, file, line and column
  that set it, so a later error names the right file.

### 3.3 Validate

- **Once per revision, each declaration is checked against itself:** its
  schema is well formed and its defaults have their types; every
  `settings.<name>`, `tasks.<task>.<setting>` and `exports.<task>` that its
  templates and expressions name is declared (format.md, 6.4); its
  templates are used and its scripts declared.
- **Per target, the composed settings are checked against the schemas:**
  undeclared settings and fields, types, enums, ranges, required
  settings, `target` and `file` references (format.md, 6.5). Defaults are
  filled at every depth (format.md, 6.3). `debian` and `placement` are
  checked against their fixed schemas.
- **Owners of what persists have fixed ids.** Once the declarations are
  concrete (3.5), every owner and group of a data path, a secret file or
  an identity file must be a declared user or group with a fixed `uid`
  or `gid`, package users included; a download's must be `root` or such
  a package user or group (format.md, 5.6 and 10.1). Anything else is
  refused: the packages layer is rebuilt from scratch, and an id a
  postinst took could move under data that keeps the old one.

### 3.4 Exports

- **For each task whose exports a task of this target reads,** render
  composes every target of the environment that has that task, validates
  that task's settings there, and evaluates its exports
  (format.md, section 8). Each target's exports are computed once per
  revision and shared by all their readers.
- **Placements are read the same way:** `target.containers` holds the
  environment's targets placed on this one (format.md, 3.5). A
  container's `on` must name a target of its environment that is not a
  container; a build environment has none (format.md, 3.4). The
  containers placed on one host must have id ranges that overlap
  neither each other nor the builder's [M5] (format.md, 3.5).
- **No cycle is possible:** exports depend only on their own task's
  settings and on `target` (format.md, 8.1).
- **An error in another target's exports fails this render too,** naming
  both targets: a spec never reads a guess.

### 3.5 Conditions

- **`requires` first** (format.md, 5.9): a false guard is an error with
  its message.
- **Then every declaration entry:** `for_each` expanded, `if` evaluated
  for each element, keys and string fields interpolated, typed fields
  evaluated (format.md, 5.2). The result is each task's concrete
  declarations: packages, preseeds, modules, downloads, file entries,
  users, groups, presets, tmpfiles lines, data, secrets, identity, boot
  parameters, scripts.

### 3.6 Files

- **Every file of the tree is produced and digested:** a static file of
  `tree/` and a `copy` by the digest step 1 gave it; a template of
  `tree/` or `templates/` by evaluating it in the task's context
  (format.md, 7.7); a link by its target. `customize.sh` and
  `software.sh` fragments likewise.
- **A rendered file is written to the store's cache by digest** as soon
  as it is complete, and dropped from memory: a target holds one rendered
  file at a time.
- **Each output is bounded** (2.3), and an undefined value, a type error
  or an exceeded limit is an error at the template's line.
- **Render adds what tasks may not write themselves:** `/etc/subuid` and
  `/etc/subgid` from the users' subordinate ranges, a `tmpfiles` line of
  type `d` for each data entry's mode and owners (format.md, 5.6 and
  5.7), and the initramfs configuration (4.2).

### 3.7 Collisions

Render compares the concrete declarations of all the target's tasks:

| What | Rule |
|---|---|
| a path in the file tree | one task only, even with identical content |
| a data name, within the target and across classes; a data path | one task only |
| a secret's or identity's path | one task only, and never a path of the tree |
| a path in the tree under a data path | never: the mount would hide it |
| a download's path | one task only; never overlapping another's, nor a path of the tree |
| users, groups, presets, tmpfiles lines, preseeds, boot parameters | several tasks only identically |
| a path a `live` entry covers | several entries, of one task or several: their consumers are a union (format.md, 5.14) |
| a uid or gid; a subordinate range | one user or group; ranges never overlap |
| packages, initramfs modules | a union |

A collision is an error that names both tasks and both lines. os
overwrote silently, in alphabetical order of task.

### 3.8 Assemble

- **Each part's record** (section 4) is built from the concrete
  declarations and the produced files, sorted as format.md, 4.3 says,
  encoded canonically and digested.
- **Render also gives the resolution key** (5.1), and the layer keys when
  the store already holds that key's resolution (4.5).
- **The live part maps each covered path to its consumers,** from every
  task's `live` entries (format.md, 5.14): the eligibility input that
  only a spec holds. Render decides nothing about a change: the builder
  pairs two specs' live parts with two outputs' tree files when it plans
  a trial, in a `plan live` job (slots.md, 6.7; steward.md, 3.3).
- **The output is the spec.** Every file it names is in the store's
  cache by digest, static files from step 1 and rendered ones from
  step 6, where a build finds them. Specs are never stored: a builder
  renders again when it needs one, as a new builder does for a `build`
  job whose render an earlier one answered (steward.md, 3.3).

### 3.9 Checks and probes [M5]

- **A task's checks are evaluated as its other declarations** (step 5):
  their fields may read settings, and their rendered form, every value
  resolved, enters the checks part (operating.md, 2.2).
- **A probe with `from` is exported.** Render puts it into its task's
  exports, rendered, with the target it probes; in step 4 each target of
  the environment that has the task `from` names reads those probes and
  renders them into its own checks part (operating.md, 2.6). A change to
  one probe changes the probed target's and its probers' parts, and no
  other.
- **Refused at render:** a `blessing` check with `from`; a `from` that
  no target of the environment has; a command check whose `user` is
  `root`, or is neither `nobody`, a sysusers user nor a package user a
  task of the target declares (format.md, 5.6; operating.md, 2.2;
  activator.md, section 5).

### 3.10 Costs

- **A revision is decoded once** and shared by all its targets. Decoded
  documents, templates and static files' digests are kept by git object
  id, so the next revision decodes only the blobs that changed. Object
  ids name contents, so this memory changes no result.
- **A target's render** composes its layers, validates, evaluates and
  hashes its files: for the fleet's targets, milliseconds each.
  Rendering every target of two revisions, as a diff does (section 6), is
  cheap enough to do on every change.

### 3.11 What changes from os

- **Always in full.** os's `manifest` wrote a spec directory and reused
  its previous output as the resolution cache; hearth keeps only digests
  and records.
- **No overlay, no workspace, no `--resolved`:** every input is in the
  revision, and tests use fake mmdebstrap (9.6).
- **Collisions, undeclared settings and undefined values are errors**
  where os overwrote, ignored or printed nothing.
- **Every part has a digest;** os had a hash for packages alone, and
  deploy hashed the etc tree without modes or links.

## 4. The spec: parts and digests

### 4.1 The parts

| Part | Kind | Holds | Enters |
|---|---|---|---|
| `packages` | output | sources, the request, the variant, preseeds, the initramfs, apt's options | the packages key, and the resolution key |
| `software` | output | downloads by digest and where each is placed; artifacts by key [M6]; `software.sh` | the software key |
| `config` | output | the file tree; users; presets; tmpfiles; `customize.sh` | the config key |
| `data` | slot | data entries, each a directory in a class, bound at its path; each class's snapshot and backup policies | no key |
| `secrets` | slot | secret files, password secrets and identity items, by name | no key |
| `boot` | slot | the kernel's command-line parameters; a container's parameters instead [M5] | no key |
| `checks` [M5] | other | the target's checks (operating.md, section 2) | no key |
| `live` | other | the consumers of the target's paths, for live changes (format.md, 5.14) | no key; read by the builder to plan a live trial |
| `exports` | other | what this target publishes to its environment | no key; read by other targets' renders |
| `meta` | | target, base, member, environment, placement | no key |

### 4.2 Output parts

```
packages
  sources
    suite              string                          forky
    architecture       string                          amd64
    components         set of component, in fixed order main contrib non-free non-free-firmware
    archive            timestamp                       20261001T030405Z
    updates            bool
    security           timestamp; absent when the pocket is off
  request              set of package names
  variant              apt for metal and vm; containers' is settled in M5 (format.md, section 13)
  preseeds             map question → { owner, type, value }
  initramfs            { modules: set of names, microcode: none | generic }; absent for a container
  apt                  { install_recommends }
  seeds                users: map name → { uid, gid }; groups: map name → { gid }

software
  downloads            map path → { sha256, owner, group, file { mode } | tar { strip } }      format.md, 10.1
  artifacts [M6]       map path → { pinned { key } | head { recipe, listing digest } }
  scripts              list of { task, order, content digest }        software.sh, in order

config
  tree                 map path → one of
                         file       { content digest, mode, owner, group }
                         link       { target, owner, group }
                         directory  { mode, owner, group }
  groups               map name → { gid? }
  users                map name → { uid?, group, gecos, home, shell } | package
  members              set of (user, group)
  presets              map unit → enable | disable
  tmpfiles             map path → { type, mode?, owner?, group?, age?, argument? }
  scripts              list of { task, order, content digest }        customize.sh, in order
```

- **The variant is a function of placement,** never a task
  (format.md, 3.4): os chose it from a `container` task in render and
  from the target's kind in deploy, which disagreed. A container's
  variant changes its resolution key (5.1), so a host and its container
  never share a packages layer.
- **A container's packages part has no initramfs,** and render refuses
  a task's module on it, as it refuses a boot parameter (format.md, 3.4;
  7.2): with no kernel there is nothing to configure.
- **The initramfs configuration is render's,** written as
  `tiny-initramfs.conf` from the part's two fields: `AUTO_MODULES=no`
  always, `INCLUDE_MODULES` from `modules`, and `MICROCODE=no` or
  `generic` from `microcode` (machine.md, 7.1; build.md, 5.5). Render
  derives `microcode` from placement's kind, `none` on a VM and `generic`
  on metal; placement enters no key, so the part must carry the choice
  for the build to know it. Nothing a task writes can set `yes` for
  either.
- **`seeds` are the fixed ids build writes into `/etc/passwd` and
  `/etc/group`** in mmdebstrap's extract hook, before any package is
  configured (build.md, 5.5): every package user and group, and every
  owner of a data path, a secret or an identity file, with its uid and
  gid (3.3). They change what the packages layer builds (adduser keeps a
  user it finds), so they belong to its key; they do not change what apt
  resolves, so not to the resolution's.
- **`install_recommends` is the part's one option.** hearth's fixed apt
  and dpkg options (`Check-Valid-Until "false"`, `force-confold`, and the
  rest build.md, 5.4 passes) are constants of the encoding's version
  (store.md, 3.2): changing one changes every key, and no part carries
  them.
- **A download's owners** are `root`, or a package user or group with a
  fixed id: software is placed before any sysusers user exists, and
  render refuses any other owner (format.md, 10.1).
- **A pinned artifact is its key** (artifacts.md, 7.1); its repository,
  commit and recipe ride beside the part, outside its digest, as how to
  make it. **A head artifact is its recipe and its sources' listing**
  at the revision: the SHA-256 of the `ls-tree` records of the paths the
  recipe selects in `artifacts.json` (artifacts.md, 3.2), from render's
  own listing. Its resolved key never enters the part (4.5).
- **A download is what it is, not where it comes from.** Its URL stays
  outside the digest, as a fetch hint beside the part (map from SHA-256
  to URL), so moving a release to a mirror rebuilds nothing. Its name
  stays out too; the path keys it.
- **Owners are names** in every part; build resolves them in the layer
  (format.md, 5.5). Those of data, secrets, identity and software have
  fixed ids (3.3).
- **A package user** is a name only (format.md, 5.6): build writes no
  sysusers line for it, and the activator confirms it in the slot's
  `/etc/passwd` at prepare (slots.md, 5.7).
- **Each data entry adds a `tmpfiles` line** of type `d` with its mode
  and owners (format.md, 5.7), so they are applied at boot in the running
  slot (machine.md, 3.4).

### 4.3 Slot parts

```
data
  entries              map name → { class: core | data | cache, path }
  policies             map class (core, data) → { snapshots? { every, keep } [M5],
                         backup? { every, keep, peer, recipient } [M6] }
secrets
  files                map secret name → { path, mode, owner, group }
  passwords            map user → secret name
  identity             map item → { kind, algorithm?, path, public_path?, mode, owner, group }
boot                   list of parameters, in order; for a container [M5], its parameters instead:
                         { ids, binds: map path → { source, read_only }, capabilities, network, bridge? };
                         and mutable, from placement (development.md, section 4)
```

- **Slot parts never enter a layer key:** changing one makes a new slot of
  the same output (slots.md, section 5).
- **A container's slot parts are its own,** placed by its host's
  activator into the container's slot on the host [M5] (slots.md, 12.1).
  Its `boot` part holds no command line, since it has no kernel, but
  the parameters its placement gives nspawn (format.md, 3.5): what
  starts the slot, as a host's command line is, so a change to them
  prepares a new slot of the same output.
- **The data part's entries are the tasks' `data` entries,** a name, a
  class and a path each (format.md, 5.7); mode and owners go to the
  config part's tmpfiles lines instead (4.2). **Its policies are the
  `hearth` task's `snapshots` and `backups` settings** (format.md, 5.12),
  by class, a backup's peer and recipient read from the `backup_peer`
  task's exports (format.md, 8.7). A host carries every class's policy
  its settings give, used or not, since its containers' classes follow
  them; a container's data part carries none (slots.md, 12.1).
- **The secrets part holds names, never values.** Beside secret files it
  carries password secrets and identity items, the two other things the
  activator writes into a slot from the secret store
  (format.md, section 9; slots.md, 5.4).

### 4.4 Other parts and meta

```
exports                map task → map name → value; and [M5] the task's probes with `from`, rendered
live                   map path → list of { task, unit?, action }, in path order; a path ending in `/`
                       covers its subtree (format.md, 5.14)
checks                 from M3, the units whose failure does not block a bless (format.md, 5.13);
                       [M5] the target's checks, and the probes it runs for others (3.9; operating.md, section 2)
meta                   { target, base, member?, environment, placement { kind, firmware?, hardware?, on?, and a container's parameters [M5] } }
```

- **Exports have a digest** so a target's own diff shows what it
  publishes; a change to one reaches its readers through their own parts,
  whose content changes.
- **Meta enters no key:** two targets whose output parts agree share an
  output, whatever their names.
- **The live part enters no key, not even the prepare key:** how a
  change is applied is not what a slot holds, so a change to `live`
  entries alone builds and prepares nothing.

### 4.5 Keys

```
part digest        = H("part:<name>", the part's record)
resolution key     = H("resolution", sources, request, variant, apt)                 5.1
resolution digest  = H("closure", each package's name, architecture, version, SHA-256) 5.4
packages key       = H("packages", resolution digest, packages part digest)
software key       = H("software", packages key, software part digest
                         [, M6: the resolved key of each head artifact, by path])
config key         = H("config", software key, config part digest)                   the output's name
spec digest        = H("spec", the digests of meta and of every part)
```

- **The packages key takes the closure's digest,** not the resolution's
  key. Two builders that resolved one key to different closures (another
  apt, say) then name their layers differently, and a key never names two
  sets of installed packages.
- **A head artifact's resolved key enters the software key,** as the
  closure's digest enters the packages key: the part holds only what the
  revision says, so render stays a function of it, and the key is
  remembered by the listing's digest (artifacts.md, 7.2) [M6].
- **The output is the config layer,** and its key is the output's name
  (store.md, section 4).
- **The spec digest** answers "did anything change for this target";
  the revision's commit id rides beside it, outside every digest, so two
  commits that render the same give the same digests.

### 4.6 What render needs from the canonical encoding

store.md, section 3 defines the encoding and H. Render relies on it for:

- **one injective encoding of typed records:** integers at fixed width,
  byte strings and lists with their lengths, booleans and enumerations
  as tags, optional fields with a presence mark, maps sorted by key
  bytes, and a record's fields in a fixed order;
- **a tag per kind of record,** distinct between part digests and keys,
  with a version, so equal bytes of two kinds never share a digest and a
  change of encoding changes every key;
- **H as SHA-256 over the tag and the encoding,** computable in the
  domain (`no_std`), with a digest a fixed 32 bytes inside other records.

The encoding has no signed integers. An integer setting may be negative,
so an export's integer value enters as its decimal text; every other
integer in a part (ids, modes, orders, sizes) is unsigned.

## 5. Resolution

### 5.1 The key

```
resolution key = H("resolution", sources, request, variant, apt)
```

- **It covers everything that changes what apt resolves:** the sources
  (suite, architecture, components, snapshot timestamps, pockets), the
  requested packages, the bootstrap variant, and `install_recommends`;
  hearth's fixed options change it through the encoding's version (4.2).
- **It leaves out what does not:** preseeds, the initramfs, the seeds,
  the proxy and where the bytes come from.

### 5.2 The invocation

```
mmdebstrap --verbose --simulate --mode=chrootless
  --variant=<variant>               the part's: apt for metal and vm; a container's (4.2)
  --architectures=amd64
  --include=<the request, sorted, comma-joined>
  --aptopt=Acquire::Check-Valid-Until "false"
  --aptopt=APT::Install-Recommends "false"
  [--aptopt=Acquire::http::Proxy "<the builder's proxy>"]
  --skip=cleanup/apt/lists
  forky <scratch>/root <scratch>/sources.list
```

with `<scratch>/sources.list` encoded from the sources:

```
deb http://snapshot.debian.org/archive/debian/20261001T030405Z/ forky main contrib non-free-firmware
```

and, with both pockets on, `<suite>-updates` from the same archive and
`<suite>-security` from `http://snapshot.debian.org/archive/debian-security/<security>/`.

- **Unprivileged, with no namespace:** chrootless simulate needs no
  subordinate ids, where mmdebstrap's automatic mode would pick unshare
  and need them.
- **`--verbose`** makes apt print its `Inst` lines; **`--skip=cleanup/apt/lists`**
  keeps the indices apt fetched, which 5.3 reads.
- **The scratch directory** is new and empty for each run, in the
  builder's temporary area, and removed after.
- **The environment is given whole:** `LC_ALL=C`, a fixed `PATH`, and
  `TMPDIR` the scratch directory.

### 5.3 What it yields

- **`Inst` lines give the closure's names:** `Inst <name> [<old>]
  (<version> <origins> [<architecture>])` gives the name, the version and
  the architecture of each package apt would install.
- **The `Packages` indices give the rest.** `hearth-protocol-mmdebstrap`
  reads every `*_Packages` file under `<scratch>/root/var/lib/apt/lists/`
  in bounded chunks, stanza by stanza (at most 64 KiB a stanza), and
  sends up each stanza's `Package`, `Version`, `Architecture`,
  `Filename`, `Size` and `SHA256`. The indices are what apt verified
  against the signed `InRelease`, so their digests can be trusted.
- **The domain matches them:** each `Inst` triple must match a stanza;
  several stanzas may match (one per index that holds the package) and
  must agree on size and digest. The package's source is the first
  matching index's line, in the order of the sources; its URI is that
  line's archive followed by `Filename`.
- **The indices are kept.** Debian, and so snapshot.debian.org,
  publishes no uncompressed `Packages` file; `InRelease` lists the
  uncompressed file's size and SHA-256 so that apt can check it after
  decompressing. apt's lists hold the decompressed, verified files. Before
  the scratch directory is removed, the protocol layer streams each
  source line's `InRelease` and each `Packages` file into the store's
  cache by SHA-256 (store.md, section 2), hashing as the bytes pass, and
  the record names them by those digests. build.md's local repository
  serves them at plain paths (build.md, 5.3), so the build reads the
  indices apt verified, offline.
- **The closure** is, per package: name, architecture, version, source,
  archive filename, size and SHA-256, sorted by name, then architecture.
  It is what build.md's fetch step needs to fetch each `.deb` by digest
  into its cache.
- **Every requested package must be in the closure;** one that is not, or
  an `Inst` triple with no stanza, is an error.

### 5.4 The record

```
resolution record                                                            store.md, 6.4
  key           the resolution key                                           5.1
  inputs        sources, request, architecture, variant, options, as render gave them
  sources       per source line: uri, suite, components;
                  InRelease { sha256 }, in the cache;
                  each Packages index { path in the archive, sha256, size, its InRelease entry }, in the cache
  closure       per package: name, architecture, version, source, filename, size, sha256
  digest        H("closure", name, architecture, version, sha256 of each)    the resolution digest, 4.5
  made          wall time; apt's and mmdebstrap's versions; costs            outside the digest
```

- **A record of the store,** found by its key (store.md, 6.4), and
  pinned by the layers whose keys use its digest (store.md, section 7).
  Before the store holds layers, in M1, it holds these records alone.
- **The digest covers what is installed,** not where it came from: a
  package's archive and filename stay outside it.
- **An index's InRelease entry** is the line that lists its path in the
  archive (`main/binary-amd64/Packages`) with the size and SHA-256 of
  the decompressed file, which apt checked and the record repeats; so a
  cached index is tied to the signed `InRelease` beside it.
- **Once recorded, a key's closure does not change** on that builder:
  render reuses it until the record is collected. Re-resolving the same
  key later, with another apt, may give another closure, and so another
  packages key (4.5).

### 5.5 Failures and costs

- **mmdebstrap's output is parsed as os parsed it:** progress markers,
  and a failure tail of at most 20 lines of 160 bytes, chosen by its
  allow and deny patterns (`E: `, `Err:`, `Failed to fetch`, `Hash Sum
  mismatch`, `404  Not Found`, and the rest).
- **Classified by pattern:** a fetch that failed, a name that did not
  resolve, a timeout or a spawn failure is operational, and the domain
  may retry it after a backoff; `Unable to locate package` and
  unsatisfiable dependencies are configuration errors, naming the
  package. Anything else defaults to configuration. The patterns are held
  to recordings of real runs (9.6).
- **Timeouts:** 60 s without output ends a run, as in os; the domain
  sets an overall deadline (10 minutes by default).
- **One resolution at a time per builder:** snapshot.debian.org is fetched
  serially. A run took 9 to 15 s, mostly `apt-get update`.
- **Its costs are facts:** each run's duration and bytes go up as costs
  (README, section 5).

### 5.6 What changes from os

- **The key covers the architecture, the variant and apt's options.**
  os's covered the sources text and the names alone, so changing the
  architecture or adding a `container` task reused a stale resolution.
- **`--architectures` is passed;** os resolved for the build host's.
- **It yields digests and URIs,** not only versions, so build fetches by
  digest.
- **The cache is a record in the store,** not the previous spec's
  `versions.json`, and there is no `--resolved` injection.

## 6. The rendered diff

### 6.1 Typed

```
diff
  from, to             commit ids
  targets              map target → change
change
  added { spec digest } | removed | same
  | changed { parts: list of part change, rebuild }
  | failed { side: from | to, errors }
part change
  packages             request: added, removed; sources, variant, preseeds, initramfs, apt: before, after;
                       resolution: same | new key | new key, resolved: added, removed, versions changed
  software             downloads and artifacts added, removed, changed, by path; scripts changed
  config               tree entries added, removed, changed { path, before, after };
                       groups, users, members, presets, tmpfiles: added, removed, changed, by key;
                       scripts changed
  data, secrets        entries added, removed, changed, by name; secrets by name only
  boot                 before, after
  exports              task.name: before, after
  checks [M5]
  meta                 placement: before, after
rebuild
  from packages { resolution: known | needed } | from software | from config | slot only | nothing
```

- **A changed file carries its two digests and sizes.** The printer adds
  hunks from the two contents in the store's cache (wire.md, section 3):
  line diffs of text files under 1 MiB a side, with three lines of
  context, at most 200 lines a file. Step code computes no text diff.
- **Secrets show names and places,** never values, since render has none.

### 6.2 Targets touched and costs

- **A change touches the targets whose spec digest differs,** and those
  it adds or removes. A change to a task touches every target that has
  it; a change to an export, every reader in that environment; a change
  to `envs/lab.json`, no production target.
- **Rebuild** is the lowest layer whose key changed: from packages (and
  whether the new resolution is known), from software, from config. A
  change to data, secrets or boot alone is a new slot of the same output.
  A change to exports alone builds nothing on the exporting target.
- **Time,** when the builder has built the target before, is its last
  recorded cost for each layer to rebuild. Whether activation needs a
  reboot is known only after the build, from the kernel's and the
  initramfs's digests (slots.md, section 6), and whether it can be live,
  from the two outputs' tree files and the live parts (slots.md, 6.7).

### 6.3 Printed

A commit changes ion-prod's public endpoint, in `hosts/ion/ion-prod.json`:

```
$ hearth diff main HEAD
carbon-prod                                         rebuild from config
  config   ~ /etc/systemd/network/wg0.netdev        0644 root:root
               @@ -13 +13 @@
               -Endpoint=194.164.162.157:51820
               +Endpoint=194.164.162.158:51820
ion-prod                                            nothing to build
  exports  ~ vpn.endpoint   "194.164.162.157" → "194.164.162.158"

2 of 11 targets change; 9 are unchanged.
```

- The client prints it as text, or as JSON for agents
  (wire.md, section 3). The typed diff is the same either way.
- The coupling is visible: carbon-prod changes because it reads ion-prod's
  exports, and carbon-lab does not, because exports stay in their
  environment.

### 6.4 Who posts it

- **Open** (README, section 9): the lab, CI or temper's gate could post it
  at a change's head, as a status is; and how a change that touches many
  targets is shown, folded by identical part changes or not, follows from
  who reads it.

## 7. Errors

### 7.1 The record

```
error
  kind        one of 7.2, with its own typed fields
  class       configuration | operational
  at          { path, line, column } in the revision, when there is one
  also        a second place: the other task of a collision, the layer that set a value
  target      the target, when the error is a target's
  task        the task, when it is a task's
```

- **Configuration errors** mean the revision is wrong and must change.
  **Operational errors** mean a tool or a resource failed (git, mmdebstrap,
  the network, a timeout) and the same revision may succeed later.
- **Render collects every independent error:** a broken document fails
  the targets that compose it, a broken task the targets that have it,
  and the rest goes on. Errors are sorted by path, line, column and
  target, and bounded (2.3).

### 7.2 Kinds

| Kind | Examples |
|---|---|
| decode | not JSON, a duplicate key, a fraction, invalid UTF-8, a template's syntax, an unknown filter, a limit exceeded |
| repository | a document out of place, an environment that does not exist, a submodule, a stray file in a task |
| composition | two targets of one name, `null` for a task in a default, `placement` in a base, a container's `on` that names a container or a target of another environment; [M5] a container's parameters on another kind, or an id range that overlaps another's (format.md, 3.5) |
| schema | an undeclared setting, a wrong type, a missing required setting, an undeclared read, a bad default |
| evaluation | an undefined value used, a type error in an expression, an overflow, a `for_each` that is not a map or list |
| guard | a false `requires`, with its message |
| reserved | a command-line key, a path, a unit name, a drop-in that sorts after `zz-hearth-watchdog.conf`, a tmpfiles line, an initramfs setting or a preseed that hearth reserves; a secret or identity file `hearth` could read, or a secret readable by others; [M5] an initramfs module, a boot parameter or the `hearth` task on a container, a host's preset that enables one of its containers' nspawn units (format.md, 3.4, 5.11 and 5.12; build.md, 7.3, 7.4 and 9.2; slots.md, 5.5; machine.md, 6.2) |
| ids | an owner of a data path, a secret or an identity file without a fixed id; a download owned by anyone but `root` or a package user with a fixed id (3.3; format.md, 5.6 and 10.1) |
| checks [M5] | a `blessing` check with `from`, a `from` no target has, a command's `user` that is `root`, or neither `nobody`, a sysusers user nor a declared package user (3.9) |
| collision | one path from two tasks, two users that differ |
| resolution | a package apt cannot locate, unsatisfiable dependencies, an `Inst` line with no stanza |
| tool | git or mmdebstrap failed, timed out or could not start: operational |

### 7.3 Printed

```
hosts/ion/ion-prod.json:4:5: ion-prod: vpn: undeclared setting "adress"; vpn declares address, endpoint, hub, port, public_key
tasks/mail/tree/etc/postfix/main.cf.j2:60:42: mail: exports.mail is not in mail's reads
tasks/hardening/task.json:9:7: rhi-prod: /etc/ssh/sshd_config.d/local.conf is given by hardening and by ssh (tasks/ssh/tree/etc/ssh/sshd_config.d/local.conf.j2)
tasks/boot/task.json:22:7: carbon-lab: boot: requires modules: no initramfs modules for this hardware
```

- **An undeclared setting lists the declared ones,** so an agent fixes a
  typo in one step.
- **What would fail later fails here.** Every path, key and line that a
  build's scan, the activator or the boot would refuse, and that render
  can see in the declarations, render refuses first, at its line:

  ```
  tasks/serial/task.json:12:7: rhi-prod: boot: "rootwait" is reserved: tiny-initramfs would wait for ever for a missing root (machine.md, 6.2)
  tasks/web/task.json:30:9: mail-prod: web: /etc/ssl/private/mx1.key is a secret's path; declare it under secrets (format.md, 9.1)
  ```

## 8. Layers and crates

### 8.1 The crates

```
hearth-domain-builder            the builder's root: routes between its children
└── hearth-domain-render         configuration to spec; resolution's decisions; the diff        M1

hearth-protocol-builder          the builder's protocol layer: routes its machines, translates to render's types
├── hearth-protocol-git          git's argument vectors; ls-tree and cat-file records             M1
├── hearth-protocol-config       strict JSON to typed documents and declarations, with positions  M1
├── hearth-protocol-template     template and expression text to instruction sequences            M1
└── hearth-protocol-mmdebstrap   simulate's argument vector; Inst lines; Packages stanzas; tails  M1
```

- **`hearth-domain-render`** holds, as modules: the revision's decoded
  documents; composition and RFC 7386; validation; evaluation; exports;
  collisions; parts and digests; resolution's keys, records and closures;
  the diff. It depends on lib and nothing else.
- **The machines depend on lib only** (`programming-model.md`, 4).
  `hearth-protocol-config` cannot call the template machine, so it hands
  up a declaration's expression strings as text with their positions,
  and `hearth-protocol-builder` passes each through
  `hearth-protocol-template` before translating the whole declaration
  into render's types.
- **The client runs the same domain in-process** when no daemon
  answers, or for a local repository (README, 4.1; wire.md, 3.5); in
  the daemon, the builder's root takes the steward's jobs through
  `hearth-protocol-jobs` (steward.md, section 3).

### 8.2 What crosses

```
render → protocol                                      protocol → render
List { repository, revision }                          Listed { commit, entries: path, mode, oid, size }
Read { commit, blobs: (oid, as) }                      Read { oid, a document | a declaration | a template }
  as: document | declaration | template | cached       Cached { oid, size, sha256 }: streamed into the cache
Keep { sha256, bytes }: a rendered file                Kept { sha256 }
Resolve { key, sources, request, variant, apt }        Resolved { key, closure, indices, versions, cost }
every request                                          Failed { request, class, at, tail }
```

- **Resolution records** go through the builder's root to the store and
  back (store.md, section 6): render asks for a key and records a new
  closure; it never touches a file.
- **Every request gets one terminal event** (`programming-model.md`,
  5.2), and a resolution's deadline is render's to set.

### 8.3 Evaluation without recursion

- **Step code has no recursion and no unbounded loop**
  (`programming-model.md`, section 3), so nothing is evaluated as a tree:
  - `hearth-protocol-template` gives an expression as a flat sequence of
    operations in postfix order, with forward jumps for `and`, `or` and
    `… if … else …`; the domain evaluates it on a value stack whose depth
    the expression limit bounds;
  - a template is a flat sequence of instructions (emit text, emit an
    expression, branch, enter and advance a loop, assign), evaluated by a
    `for` over a range bounded by the iteration limit, with a
    `lib::Stack` of loop frames bounded by the nesting limit;
  - merging walks two values with an explicit stack bounded by JSON's
    depth.

### 8.4 Limits and the worst case

- **Render's limits are 2.3's,** and its worst case
  (`programming-model.md`, 6.3) is the revision's decoded JSON and
  templates (64 MiB), an object id, size and digest per static file, the
  exports of one environment, and one rendered file at a time (16 MiB).
  Static files and rendered files live in the store's cache, never in
  the domain, so a small VM's builder renders what a large one does.

## 9. Its world

### 9.1 Worlds

- **Render's domain world** runs `hearth-domain-render` under a scripted
  parent that plays the protocol layer: it delivers typed documents,
  declarations and templates as the machines would, in an order and at a
  pace drawn from the seed, and plays the store's records. It checks
  every world's invariants (`testing-strategy.md`, section 6).
- **Machine worlds** run each protocol machine against recordings of the
  real tool, cut at random: `ls-tree` and `cat-file` streams, `Inst`
  lines, `Packages` indices, failure tails (`testing-strategy.md`, 2.4).
- **The builder's system world** runs render with the fakes of git and
  mmdebstrap on the fake machine (testing.md, sections 3 and 4).

### 9.2 Golden repositories

- **hearth's own:** `tests/render/<name>/`, each a repository's files and
  the expected printed spec of each target (`expected/<target>`), or the
  expected errors. The fake git serves a directory; one test in the real
  tier builds a real repository from it and renders through real git.
- **The repositories:** `fleet` (a miniature of the fleet: x1-prod,
  rhi-prod, ion-prod, carbon-prod, carbon-lab, chat-prod), `merge`
  (presence, defaults, removal, maps, lists), `templates` (each
  statement, filter and whitespace rule), `exports` (the VPN, the mail
  pair, two environments), `cluster` (stoa's members), `errors` (one case
  per kind of 7.2), `resolution` (closures from the fake universe).
- **The printed spec** is the client's stable text form:

  ```
  target       x1-prod   base x1   environment prod
  placement    metal efi 7th-gen
  packages     part 4be1…   resolution key 0c7d…
    sources    forky amd64 main contrib non-free-firmware 20261001T030405Z
    request    btrfs-progs ca-certificates dbus efibootmgr firmware-intel-sound …
    variant    apt
    preseed    locales/default_environment_locale select en_US.UTF-8
    initramfs  btrfs nvme
  config       part e7c2…
    /etc/hostname            file 0644 root:root 3f1c…
    tmpfiles   /var/log/journal d 2755 root systemd-journal
  data         part 51aa…
    journal    /var/log/journal
  ```

- **`HEARTH_BLESS=1` rewrites the expected files and still fails,** so a
  blessed change is always reviewed (os's practice).

### 9.3 Properties

Over generated repositories, valid by construction and then mutated:

- **Determinism:** one revision renders to the same digests in every
  world, whatever the seed's order of blob arrival and chunking.
- **Representation:** reordering a JSON object's members, reformatting
  whitespace or editing a `_note` changes no digest.
- **RFC 7386's laws:** `merge(t, {}) = t`; `merge(merge(t, p), p) =
  merge(t, p)`; a non-object patch replaces; a key the patch lacks keeps
  its value; no result holds `null`.
- **Locality:** changing a setting no export reads changes that target's
  spec alone; changing a production file never changes a lab spec.
- **Order:** the order in which render takes tasks and targets changes
  nothing; a collision names the same pair whichever task comes first.
- **Digests:** two different records of a part give different digests
  over generated pairs; the encoding's own laws are store.md's
  (section 11).

### 9.4 Fuzzed decoders

- **Every decoder is fuzzed** on its own (`testing-strategy.md`, 2.4):
  configuration documents and declarations; templates and expressions;
  `ls-tree` and `cat-file` streams; `Inst` lines and `Packages` stanzas.
  Corpora start from the golden repositories and the recordings.

### 9.5 MiniJinja as an oracle

- **Test code is ordinary Rust,** so it may run MiniJinja 2 with
  `trim_blocks`, `lstrip_blocks` and `keep_trailing_newline` on, strict
  undefined, and hearth's own filters registered.
- **Generated templates of the subset** render byte for byte the same in
  both. Where hearth differs on purpose (undefined, `first` on an empty
  list), the generator avoids it and the difference is listed in the
  test.

### 9.6 Resolution's fakes and conformance

- **`hearth-fake-mmdebstrap`** resolves against the fake machine's
  package universe and writes `Inst` lines and `Packages` indices of the
  real shape (testing.md, section 4).
- **Conformance** runs real `mmdebstrap --simulate --mode=chrootless`
  against the test tier's local mirror, and the fake against the same
  universe, and compares closures (testing.md, section 5). Its runs are
  recorded, and the recordings replace os's hand-written logs.

### 9.7 os's goldens

- **Inspiration, not an oracle.** os's fixtures (conditions, fan-out,
  accumulation, secrets, components, forky's sources, the full host and
  container) are rewritten in hearth's format where the case still
  exists. Their expected output is hearth's, reviewed; it is never
  compared with os's, which strips final newlines and records no modes.

### 9.8 Budgets

- **Rendering every target of the miniature fleet, and a diff of two
  revisions,** are domain-world scenarios of milliseconds. The budgets
  are testing.md's (section 9).

## 10. What is open

- **The indices after a simulate run:** that `--skip=cleanup/apt/lists`
  leaves `InRelease` and the decompressed `Packages` files in the scratch
  directory in chrootless simulate mode is to verify; if apt keeps them
  compressed, the protocol layer must decompress them before they enter
  the cache. It depends on a recording of each.
- **The `Inst` line with several origins** (`(2.41-12 Debian:13.1/stable,
  Debian-Security:13/stable-security [amd64])`), to verify against
  recordings.
- **Chrootless or unshare for resolution:** os and the bash fleet
  resolved chrootless and built unshare; a reason for preferring unshare
  was found once, in experiments, and not recorded. A conformance
  scenario compares the closures of both modes (testing.md, 5.2), and
  build.md, 5.7 refuses any build whose installed set differs from its
  closure. If the modes ever disagree, resolution moves to unshare, and
  render then needs the builder's subordinate range. A lead, from
  mmdebstrap 1.5.7's code: chrootless installs the essential set with
  `dpkg --force-depends` from outside and the rest with an outside
  `apt-get install`, while unshare and root skip that last apt step for
  `--variant=custom`, so the two may diverge for containers' variant.
- **apt's version and solver:** recorded beside the closure, not keyed.
  Whether a new apt should re-resolve depends on how often closures
  differ in practice, which the costs and conformance will show.
- **A foreign architecture** with chrootless simulate, to verify; it
  depends on the first non-amd64 target.
- **Who posts the rendered diff,** and how a change that touches many
  targets is shown (README, section 9).
- **The defaults of 2.3 and the printer's bounds on hunks,** to set from
  the first measurements of the fleet.
- **Resolution records before the store holds layers** (M1): in the
  store's record directory alone, or in the client's cache until M2; it
  depends on store.md, section 6.
- **Classifying mmdebstrap's failures by pattern,** which recordings of
  real failures will confirm or correct.
- **Keeping decoded documents across revisions** within one builder's
  life, a pass and its linger (steward.md, 4.1): what a pass's later
  renders, a diff's two revisions or the bless's render, would save,
  against the builder's memory, bounded by a limit to set.
- **Fetching a single commit** rather than a branch, which needs the
  forge to allow fetching an object by id; it depends on the forges the
  fleet uses.
