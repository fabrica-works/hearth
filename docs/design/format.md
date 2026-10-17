# The IaC format

Provisional, 2026-10-09, revised 2026-10-16. The format of an IaC
repository as hearth reads it: the repository's files, how targets are
made from bases and environments, how layers merge, what a task declares
and the schema of its settings, the template language, exports between
targets, and how secrets, identity and software are named. It takes os's
`config-format.md` and `task-format.md` further, for agents and digests
(draft/brainstorm.md, section 7), and is written fresh: there is no
converter from os's configurations. Milestone M1 builds it, but for
checks (M5) and artifacts (M6). How a revision becomes a spec is
render.md. Open questions are in section 13.

## 1. In one page

- **The repository is the whole input.** `fleet.json`, `envs/`, `hosts/`
  and `tasks/` at a revision decide every spec. Nothing on a command
  line, on the machine or in the process's environment changes one.
- **A target is a base in an environment.** `hosts/mail/mail-lab.json`
  makes the target `mail-lab`; a cluster's base makes one target per
  member, such as `stoa-1-lab`. Names are made from files and never
  parsed back.
- **Layers compose in a fixed order and merge by RFC 7386.** Task
  defaults, fleet, base, member, environment, base in the environment.
  Objects merge key by key, `null` removes, anything else replaces, lists
  included. Collections are maps keyed by name.
- **Defaults never add a task.** `fleet.json` and `envs/` set settings for
  the targets that have a task. Only a base and its environment files say
  which tasks a target has: `{}` adds a task with all its defaults, `null`
  removes it.
- **Placement is data of the environment.** Bare metal, a VM or a
  container on a named target, with its firmware and its hardware
  profile. A host's containers are the targets placed on it; the host
  keeps no list.
- **A task declares everything about one concern** in `task.json`: its
  settings and their schema, what it reads from other tasks, packages
  and preseeds, initramfs modules, software, files with modes and owners,
  users, presets, tmpfiles, persistent data, secret files, identity, boot
  parameters, exports, the units that consume its files when they change
  live, and its escape hatches with their order.
- **Every setting is declared:** a type, a default or `optional`, and one
  line of documentation. An undeclared setting, a wrong type, a missing
  required setting and an undeclared read are errors, with their file and
  line.
- **Templates are inert.** A file named `*.j2` is a template in a small
  Jinja subset: no macros, no includes, no calls, nothing that runs code.
  Every other file is copied as it is. An undefined name is an error,
  except an optional setting that is absent.
- **Exports are the only data that crosses targets.** A task declares
  them, computed from its settings alone; a reader names the task whose
  exports it reads; only targets of the same environment see them. Public
  halves of secrets and identity are recorded in the repository as data.
- **Secrets and identity are named, never held.** A task names each
  secret file by its name in the secret store, with a path, mode and
  owner; identity items (host keys, WireGuard keys) by kind; password
  hashes by secret. The activator writes them into a slot at prepare.
- **Software is fetched by digest.** A download is `{url, sha256}`,
  placed as one file or unpacked from a tar archive at a declared path;
  an artifact is pinned by its key and placed as a tree [M6].
- **hearth is a task too.** Every target but a container has it:
  hearth's binary, its activator's unit, its user and its subordinate
  range. No hearth runs in a container in M5; its host runs it.

## 2. The repository

### 2.1 Layout

```
<repository>/
├── fleet.json                          defaults for every target: the sources, fleet-wide constants
├── envs/
│   ├── lab.json                        defaults for every target in lab
│   └── prod.json                       defaults for every target in prod
├── hosts/
│   ├── x1/
│   │   ├── x1.json                     the base: a system described once
│   │   └── x1-prod.json                the base in prod: makes the target x1-prod
│   ├── chat/
│   │   └── chat-prod.json …
│   └── stoa/
│       ├── stoa.json                   a cluster's base, with its members (3.6)
│       └── stoa-lab.json               makes stoa-1-lab, stoa-2-lab, stoa-3-lab
└── tasks/
    └── ssh/
        ├── task.json                   the declaration (section 5)
        ├── tree/                       files installed at the same path under /
        │   └── etc/ssh/sshd_config.d/local.conf.j2
        ├── templates/                  templates rendered to computed paths (5.5)
        │   └── authorized_keys.j2
        ├── customize.sh                an escape hatch in the config layer (5.8)
        └── software.sh.j2              an escape hatch in the software layer (5.8)
```

- **Render reads `fleet.json`, `envs/`, `hosts/` and `tasks/`.** Any other
  file (a `README.md`, `lab-ca/lab.crt`) is read only when a setting of
  type `file` names it (6.2). Every path in the revision is UTF-8; one
  that is not is a `repository` error (render.md, 7.2).
- **Under `hosts/<base>/`** the documents are `<base>.json` and
  `<base>-<env>.json` for environments that exist in `envs/`. Any other
  `.json` directly in it is an error; deeper, a `.json` is another
  file. Other files (a base's own nginx site) may sit beside them, named
  by `file` settings.
- **A task directory holds `task.json` and only the entries above,** plus
  an optional `README.md`. Anything else is an error, so a misspelt
  `tre/` fails instead of being ignored. Every file under `templates/`
  must be used by a `files` entry.

### 2.2 Documents

- **Strict JSON (RFC 8259),** one object per file, in UTF-8. Duplicate
  keys are an error at every depth. Numbers are integers within i64: a
  fraction or an exponent is an error. No string may hold U+0000.
- **`_note` is the one annotation.** A string allowed in any object, it is
  dropped when the document is decoded: it never reaches merging, a
  schema, a template or a digest, so editing a note changes no spec.
- **No document is a template.** Values in `fleet.json`, `envs/` and
  `hosts/` are data, taken literally. Only the declarations of a
  `task.json` (5.2) and `*.j2` files are evaluated.
- **Bounds** are render's limits (render.md, 2.3): 1 MiB a document,
  depth 32, 4,096 keys an object. They count the document as written: a
  `_note` is one of its object's keys.
- **What a later milestone adds is refused before it.** A key or value
  this document marks [M5] or [M6] is an error at its key until that
  milestone builds it, as an undeclared field is: a placement's `ids`,
  `binds`, `capabilities`, `network` and `bridge` (3.4), a download's
  `artifact` (10.2), a task's `checks` (section 11). So nothing in a
  repository is silently ignored.

### 2.3 Templates and static files

- **A file whose name ends in `.j2` is a template; every other file is
  static.** `tree/etc/ssh/sshd_config.d/local.conf.j2` renders to
  `/etc/ssh/sshd_config.d/local.conf`; `tree/etc/issue.net` is copied byte
  for byte. The same rule holds for `customize.sh(.j2)`,
  `software.sh(.j2)` and everything under `templates/`, which must all be
  templates.
- **Why a suffix:**
  - *Inert by default.* A static file is never parsed, so it may hold any
    bytes: a certificate, a binary, a shell script with `${#a[@]}` (which
    a Jinja lexer reads as the start of a comment), a Prometheus rule
    with `{{ $labels }}`. os rendered every file, so a file that was not
    UTF-8 failed and a stray `{#` broke a script.
  - *Visible.* An agent, a diff and an editor see from the name alone
    which files are evaluated, and the path is still the real one, less
    a suffix.
  - *Cheaper.* Render hashes a static file and parses only templates.
  - *The alternatives are worse.* Two trees (`static/` and `templates/`)
    split one destination directory across two places; a list of
    templates in `task.json` drifts from the files.
- **A destination that itself ends in `.j2`** is written as
  `NAME.j2.j2`: the suffix is stripped once.
- **One source a destination:** a task that holds both `NAME` and
  `NAME.j2` (in `tree/`, or `customize.sh` or `software.sh` beside its
  `.j2`) is a `repository` error at the second in byte order, the first
  its `also` (render.md, 7.1). Across tasks, one path from two is a
  collision (5.2).
- **A template is a file with a name.** A symbolic link whose name ends
  in `.j2`, and a `.j2` alone, which would install at its directory's
  path, are `repository` errors.
- **Templates are UTF-8 with `\n` line ends;** a `\r` in a template is an
  error. Static files are bytes.

### 2.4 Names

| Name | Form | Notes |
|---|---|---|
| base, environment, member | `[a-z0-9]`, then `[a-z0-9-]`, not ending in `-` | no dot |
| target | made from them (3.1), at most 63 bytes | no dot; never `hearth` |
| task, setting, object field, export | `[a-z]`, then `[a-z0-9_]` | identifiers in expressions |
| data | `[a-z]`, then `[a-z0-9-]`, at most 32 bytes | names a directory in its class (5.7); no dot |
| secret | segments of `[a-z0-9._-]` joined by `/`, at most 255 bytes | no `.` or `..` segment |
| key of a map setting | any string of 1 to 255 bytes | |

- **No dots in target names.** The btrfs top level names a target's
  subvolumes `@<target>.<label>` and `@<target>.<class>`, and the ESP its
  files `vmlinuz.<target>.<label>` (machine.md, sections 3 and 4); a dot
  in a target would make them ambiguous. Data names have no dot either,
  so a restore's `<name>.restoring` beside an entry's directory is never
  an entry (machine.md, 3.4).
- **`hearth` is never a target,** since `@hearth.store` and `@hearth.state`
  share the top level.
- **Tasks and settings are identifiers,** so `tasks.core.domain` and
  `settings.public_key` parse; a hyphen would read as a minus.

## 3. Targets

### 3.1 Bases, environments and targets

- **A target is a base in an environment.** For each `hosts/<base>/` and
  each `<base>-<env>.json` beside it whose `envs/<env>.json` exists, there
  is one target `<base>-<env>`, or one per member of a cluster (3.6). The
  file may be `{}`. A base alone is not a target.
- **Names are made, never parsed.** Render knows each target's base,
  member and environment from the files that made it, and never splits a
  name on `-` (os inferred a target's kind and host from a hyphen). Two
  targets with the same name are an error: base `stoa-1` in `lab` and
  member `1` of base `stoa` in `lab` both make `stoa-1-lab`.

| Target | Base | Environment | Placement |
|---|---|---|---|
| `x1-prod` | x1 | prod | metal, efi, 7th-gen |
| `rhi-prod` | rhi | prod | vm, bios, serverhino |
| `ion-prod` | ion | prod | vm, bios, ionos |
| `mail-prod` | mail | prod | container on `rhi-prod` |
| `mail-lab` | mail | lab | vm, efi, incus; a container of a lab pool from M5 |
| `chat-prod` | chat | prod | container on `rhi-prod` |
| `stoa-1-lab` | stoa, member 1 | lab | the lab's |

### 3.2 Composition order

```
task defaults                      the defaults of every task the target has (section 6)
fleet.json                         defaults for every target
hosts/<base>/<base>.json           the base, without its members key
  members.<member>                   the member, in a cluster (3.6)
envs/<env>.json                    defaults for every target in <env>
hosts/<base>/<base>-<env>.json     the base in <env>, without its members key
  members.<member>                   the member in <env>
```

- **Each layer is merged onto the result of those above it** (section 4),
  so a later layer wins.
- **The environment comes after the base,** so its defaults (the lab's
  certificate authority, its DNS overrides) apply to every base in it;
  the base in the environment comes last, for what belongs to one target
  alone. This is draft/brainstorm.md's order (7.3), with task defaults
  and members added.

### 3.3 Which tasks a target has

- **A base, a member and a base-env layer decide presence.** A key naming
  a task adds it: `"ssh": {}` adds ssh with all its defaults, and
  `"ssh": {"allow": ["free"]}` adds it with a setting. `"wifi": null`
  removes it.
- **`fleet.json` and `envs/` never add or remove a task.** A task's key
  there sets settings for the targets that have the task; `null` for a
  task there is an error. This is what "defaults" means, and it lets
  `fleet.json` hold the SSH port without putting an SSH server on every
  target.
- **A target's tasks** are the keys left after merging its base, member
  and base-env layers alone, and `hearth`, which every target but a
  container has (5.12). **Its settings** are the merge of every layer,
  restricted to those tasks.
- **Every task named anywhere must exist** under `tasks/`, and its
  settings are validated even where no target has it, so a typo in a
  default fails. A layer's value of a task that none of the targets it
  composes has (`envs/lab.json` setting wifi, which only prod targets
  have; a base with no target) is checked alone, as a patch: each name
  declared, and each value but `null` of its type and form. What only a
  composed value shows is not checked there: a required setting, and
  whether a `target` names a target of the environment. Such an error
  names its task and no target (render.md, 7.1).
- **Reserved keys** of a document name no task: `debian` (5.3),
  `placement` (3.4), `members` (3.6) and `_note`.

### 3.4 Placement

`hosts/ion/ion-prod.json`:

```json
{ "placement": { "kind": "vm", "firmware": "bios", "hardware": "ionos" } }
```

| Field | Type | Meaning |
|---|---|---|
| `kind` | enum `metal`, `vm`, `container` | where the target runs |
| `firmware` | enum `efi`, `bios` | how it boots (machine.md, sections 4 and 5); never for a container |
| `hardware` | string | the profile tasks key hardware data by: `7th-gen`, `incus`, `ionos`, `serverhino`, `vultr`; never for a container |
| `on` | target | for a container only: its host, a target of the same environment that is not a container; absent for a build environment, which no host runs (artifacts.md, 2.3) [M6] |
| `ids` | int | for a container on a host [M5]: the base of its 64 Ki id range, nspawn's `PrivateUsers=` (3.5) |
| `binds` | map | for a container on a host [M5]: host paths bound into it, keyed by the path inside, each `{ "source": path, "read_only": bool }`, `read_only` true by default |
| `capabilities` | set of names | for a container on a host [M5]: capabilities added to nspawn's default set, such as `CAP_NET_ADMIN`; none by default |
| `network` | enum `host`, `veth` | for a container on a host [M5]: `host` by default, the host's own network; `veth`, a veth pair whose host end joins `bridge` [M6] |
| `bridge` | string | for `network: veth` only: the host's bridge [M6] |
| `mutable` | bool | `false` by default; `true` leaves the target's slots unsealed and their root read-write, for changes by hand on a development machine (development.md, section 4); a container's too [M5]; never a build environment's |

- **Placement is data of the environment** (draft/brainstorm.md, 7.7). It
  may appear in `envs/<env>.json` and in a base-env layer or its members,
  never in `fleet.json` or a base: a base describes a system apart from
  where it runs. Every target must have a placement once composed.
- **A placement is whole where it is written:** `kind` always; `firmware`
  and `hardware` for metal and a VM, which boot by one and are keyed by
  the other (5.4; machine.md, 2.7); `on` for a container until M6's
  build environments. One missing is an error at the placement's `{`,
  and a field the kind never takes is one at its key.
- **`placement` and `members` are objects:** `null` for either is an
  error at the value, never a removal. A placement is replaced, never
  cleared, and a base-env removes a member by its own key (3.6).
- **Placement is replaced whole,** the one exception to RFC 7386 (4.1):
  a layer that sets `placement` replaces it entirely, so `envs/lab.json`'s
  `firmware` and `hardware` never leak into a lab target that its own
  file places in a container.
- **What placement decides by itself:** the bootstrap variant
  (render.md, 4.2), and that a container has no kernel, so its initramfs
  modules and boot parameters must be empty: its packages part has no
  initramfs, its boot part holds its container parameters instead of a
  command line (3.5), and render refuses a module or a parameter on it
  (render.md, 7.2). The `boot` task, which installs the kernel, refuses a
  container by a guard of its own (5.4).
- **`mutable` is a slot's, never an output's.** Render puts it in the
  `boot` part (render.md, 4.3), so it enters the prepare key and no
  layer key: a mutable target builds the output its sealed twin would.
  Render accepts it anywhere placement may be; whether a machine takes
  it is the machine's configuration (machine.md, 8.1), and the run
  side's engine never orders one [M4] (development.md, 4.3). An
  environment such as `dev` sets it for all its targets.
- **Everything else that differs by placement is task data** that reads
  `target.placement`: the gettys masked in containers, the initramfs
  modules per hardware profile, EFI's tools (5.10).

### 3.5 Containers on a host

- **A container names its host** in `placement.on`. A host's containers
  are the targets of its environment placed on it; render hands them to
  the host's templates as `target.containers` (7.7), a map from target
  name to placement.
- **One source of truth.** The host keeps no list of its containers,
  where draft/brainstorm.md (7.7) kept one in the host's configuration.
  The bash fleet's `runner:600` user, declared once for carbon and again
  for its container, shows what a second copy invites.
- **The same environment only.** `chat-prod` runs on `rhi-prod`; a lab
  container runs on a lab host. So the bash fleet's error, a container
  file that cannot vary by environment and so carries production's forge
  host key in the lab, cannot be written. The Matrix container `rhi-chat`
  becomes the target `chat-prod`, with `hosts/chat/chat-prod.json`:

  ```json
  { "placement": { "kind": "container", "on": "rhi-prod" } }
  ```

- **What a container's target holds** [M5]: tasks like any target's,
  but no `hearth` task (5.12) and nothing that needs a kernel (3.4); the
  gettys masked by the `core` task (5.10). Its `data` entries, identity
  and secrets are its own target's, kept by its host's activator: its
  class subvolumes sit on the host's top level, and its machine-id is no
  longer its host's (slots.md, 12.1).
- **Users are not mirrored** [M5]. The bash fleet declared
  carbon-forge's `runner:600` on carbon too, with a home inside the
  container's `/srv`. A container's entries are bound into it alone,
  from its own class subvolumes, and never mounted on its host
  (slots.md, 12.1), so its host needs no such user; a host task that
  must know a container's user reads it from the container's exports
  (section 8).
- **A container's parameters are placement fields** [M5] (3.4): its id
  base, its binds, its capabilities and its network. A target's file
  sets them with the rest of its placement:

  ```json
  { "placement": { "kind": "container", "on": "rhi-prod", "ids": 327680 } }
  ```

  - **Render puts them in the container's `boot` part,** a slot part,
    which for a container holds them instead of a command line
    (render.md, 4.3). So a change to one changes the slot's prepare key
    and prepares a new slot of the same output, which its host tries by
    a restart, since nspawn reads them only when it starts
    (slots.md, 12.1 and 12.2).
  - **Render checks their form:** a base aligned to 64 Ki and clear of
    the builder's range (store.md, 9.2) and of the other containers'
    placed on the same host; binds whose two paths are absolute and
    normal; capability names from the kernel's list. The host's
    activator checks them again (activator.md, 2.2).
  - **They are refused on anything but a container on a host:** a host
    and a build environment have none.
- **How a host runs a container** (its slots, the shift into its ids,
  its nspawn settings) is slots.md, section 12 [M5].
- **A container's journal is its host's** [M5]: nspawn links it into
  the host's journal entry, `/srv/core/journal`
  (operating.md, section 5), so the `core` task declares no `journal`
  entry on a container (5.7).

### 3.6 Clusters

- **A base declares its members** under `members`, a map keyed by member
  name. Each member's value is a layer of its own (3.2): tasks and their
  settings, and `debian`; `placement` in a base-env's member alone, since
  a base's member is the base's (3.4); never `members`.
  `hosts/stoa/stoa.json`:

  ```json
  {
    "core": { "hostname": "stoa" },
    "stoa": { "port": 7000 },
    "members": {
      "1": { "stoa": { "index": 1 } },
      "2": { "stoa": { "index": 2 } },
      "3": { "stoa": { "index": 3 } }
    }
  }
  ```

- **Members are named `<base>-<member>-<env>`:** `stoa-1-lab`,
  `stoa-2-lab`, `stoa-3-lab`. The environment stays last, so every lab
  target ends in `-lab`. A base with members has no target `<base>-<env>`.
- **A base-env file may change the members** in its environment, under
  its own `members`: add one, remove one (`"3": null`), or set one's data
  there, such as its placement on a lab pool host. `"members": null` is
  an error (3.4).
- **A base is a cluster in an environment** when its document or its
  base-env's has `members`. It then makes one target per member left
  once the two are merged, and none when none is left. A `null` member
  in a base removes nothing. `members` in `fleet.json` or `envs/` is a
  `composition` error at its key, as `placement` in a base is.
- **Members see each other through exports** (section 8), like any
  targets of the environment: stoa exports its address and index, and
  each member's templates loop over `exports.stoa`. `target.member` holds
  the member's name.

### 3.7 Example: x1-prod

`fleet.json`:

```json
{
  "_note": "Defaults for every target of the fleet.",
  "debian": {
    "suite": "trixie",
    "architecture": "amd64",
    "snapshot": { "archive": "20261001T030405Z", "security": "20261001T031522Z" }
  },
  "core": { "domain": "ekanayaka.io" },
  "hearth": {
    "snapshots": {
      "core": { "every": "1d", "keep": { "daily": 7 } },
      "data": { "every": "1h", "keep": { "hourly": 24, "daily": 14, "weekly": 8 } }
    }
  },
  "locale": { "timezone": { "area": "Europe", "zone": "Lisbon" }, "locale": "en_US.UTF-8" },
  "ssh": {
    "port": 10127,
    "authorized": { "free": { "x1": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAA… free@x1" } }
  }
}
```

`hosts/x1/x1.json`:

```json
{
  "_note": "x1, the laptop: Debian testing and a desktop.",
  "debian": {
    "suite": "forky",
    "pockets": { "updates": false, "security": false },
    "components": { "contrib": true }
  },
  "core": { "hostname": "x1" },
  "host": {},
  "locale": {},
  "boot": { "params": { "audit": { "param": "audit=0" } } },
  "users": { "accounts": { "free": { "uid": 1000, "groups": ["sudo"], "password": "users/free.passwd" } } },
  "wifi": {},
  "desktop": { "user": "free" }
}
```

`hosts/x1/x1-prod.json`:

```json
{
  "placement": { "kind": "metal", "firmware": "efi", "hardware": "7th-gen" },
  "debian": { "components": { "non-free-firmware": true } },
  "firmware": {}
}
```

- **x1-prod's tasks:** core, host, locale, boot, users, wifi, desktop
  and firmware, and `hearth`, which render adds (5.12). Not ssh:
  `fleet.json` set its port and keys, but defaults add no task. No
  `snapper` task, as os had: `fleet.json`'s `hearth` settings snapshot
  its `core` and `data` classes (5.12).
- **Its sources:** forky, main, contrib and non-free-firmware, with no
  updates or security pocket, from the archive snapshot of `fleet.json`.

## 4. Merging

### 4.1 RFC 7386

```
merge(target, patch):
    if patch is not an object:      return patch                 scalars and lists replace
    result = target if target is an object, else {}
    for key, value in patch:
        if value is null:           remove key from result
        else:                       result[key] = merge(result[key], value)    a missing key is not an object
    return result
```

- **Each layer is a patch** over the result of the layers above it
  (3.2), but for `placement`, which a layer replaces whole (3.4). Objects merge key by key; `null` deletes; scalars and lists
  replace. The result never holds `null`, and no setting can: the schema
  has no null type, and an absent optional setting is simply absent.
- **A `null` a merge would carry is an error where it is written:** one
  inside a list (`[1, null]`, `[{"a": null}]`), which replaces whole, is
  a `decode` error in a document (render.md, 7.2) and a bad default in a
  `task.json` (6.5); one in a placement is a wrong type (3.4).
- **Lists replace.** os concatenated them, so a variant could add to a
  base's list but never drop or replace an entry, and thin variants grew
  into copies.
- **Values keep where they came from.** Each composed value carries the
  file, line and column of the layer that set it, so an error names the
  layer at fault (render.md, section 7).

### 4.2 Collections are maps keyed by name

- **A collection is a map keyed by name:** accounts, keys, peers, domains,
  modules, parameters. A later layer then adds, changes or removes one
  entry. `hosts/mail/mail.json`:

  ```json
  { "users": { "accounts": { "alice": { "uid": 1000 }, "bob": { "uid": 1001 } } } }
  ```

  and `hosts/mail/mail-lab.json`, which removes bob and adds a tester:

  ```json
  { "users": { "accounts": { "bob": null, "tester": { "uid": 1100 } } } }
  ```

- **A list is for values whose order is their meaning,** which a layer
  replaces whole: DNS servers in order of preference, a command's
  arguments. The bash fleet's positional `user:uid:home` strings and os's
  lists of objects become maps.

### 4.3 Order

- **Maps iterate in key order:** the bytes of the UTF-8 keys, compared
  lexicographically. Templates, `for_each` and the spec's parts see the
  same order on every machine. The order of members in a JSON source
  never matters.
- **Where order matters and keys do not give it, an `order` field says
  it:** an integer from 0 to 999, default 500. Entries sort by order,
  then key, then task name. It is used by `customize.sh` and
  `software.sh` fragments (5.8) and by boot parameters (5.4), where the
  last `console=` names the kernel's primary console.

### 4.4 Presence, defaults and removal

- **A task present with all its defaults:** `"locale": {}` in
  `hosts/x1/x1.json`.
- **A task removed in one environment:** carbon is a laptop whose lab
  variant is an Incus VM, which has no Wi-Fi. `hosts/carbon/carbon-lab.json`:

  ```json
  { "_note": "An Incus VM has no Wi-Fi.", "wifi": null }
  ```

  The bash fleet could not express this: task presence was the union of
  both files, so carbon-lab inherited the Wi-Fi task and its iwd mount.
- **One entry removed, one replaced:**
  `{"ssh": {"authorized": {"free": {"old-laptop": null}}}}` drops one key
  of one user; `{"dns": {"servers": ["10.0.0.1"]}}` replaces the whole
  list of servers.

## 5. Tasks

### 5.1 The declaration

`tasks/<task>/task.json` is one object. Every key is optional and an
unknown key is an error: `{"doc": "…"}` is a valid task that does
nothing.

| Key | Declares | Part (render.md, section 4) | Section |
|---|---|---|---|
| `doc` | one line about the task | | |
| `settings` | the schema of its settings | | 6 |
| `reads` | the settings and exports of other tasks it reads | | 6.4, 8.2 |
| `requires` | guards on its settings | | 5.9 |
| `packages` | packages to install, with their component | packages | 5.3 |
| `preseeds` | debconf answers | packages | 5.3 |
| `initramfs` | initramfs modules | packages | 5.4 |
| `software` | downloads and artifacts | software | 10 |
| `files` | entries of the file tree, with modes and owners | config | 5.5 |
| `users`, `groups` | accounts, as sysusers | config | 5.6 |
| `presets` | systemd presets | config | 5.6 |
| `tmpfiles` | tmpfiles.d lines | config | 5.6 |
| `scripts` | `customize.sh` and `software.sh`, and their order | config, software | 5.8 |
| `data` | persistent data: directories in class subvolumes, bound at their paths | data | 5.7 |
| `secrets` | secret files | secrets | 9.1 |
| `identity` | identity items | secrets | 9.3 |
| `cmdline` | the kernel's command-line parameters | boot | 5.4 |
| `exports` | facts for the environment's other targets | exports | 8 |
| `checks` [M5] | what healthy means | checks | 11 |
| `bless` | units whose failure does not block a bless | checks | 5.13 |
| `live` | the units that consume its paths, and what each does when they change with no reboot | live | 5.14 |

- **`doc` is one line,** a string with no line feed, bounded only by its
  document (2.2). It enters no part, so editing it changes no spec, as
  editing a `_note` does not.

### 5.2 Entries

- **Every declaration but `doc`, `settings`, `reads`, `requires`,
  `scripts`, `exports` and `bless` is a map of entries** keyed by a name or a
  path. Any such entry may carry:
  - `if`: an expression (7.4). The entry exists only when it is true.
  - `for_each`: an expression giving a map or a list. The entry stands
    for one entry per element, with `key` and `value` bound; for a list,
    `key` is the index from 0.
- **Keys and string fields are interpolated,** and a list's strings one
  by one: each `{{ expr }}` is replaced by its value, a string or an
  integer in decimal. Any other value, a boolean included, is an
  evaluation error. Statements (`{% %}`), comments (`{# #}`) and the `-`
  and `+` markers (7.2) are not allowed there: `${{ '{#' }}a[@]}` writes
  a shell's `${#a[@]}`. Strings outside entries are taken as written (a
  default, a `doc`, `bless`), but a guard's `message` holds no `{{`, `{%`
  or `{#`, so that a later milestone may interpolate it.
- **A field of another type** (an integer, a boolean, a list) may be
  written as one `{{ expr }}`, whose value must have the field's type.
- **A field that is one `{{ expr }}` alone,** a string field too, is
  absent when its value is undefined (7.6), so the field's default
  applies: `shell` below. Beside text, in a key and in a list's string,
  undefined is an evaluation error.
- **Keys are unique after expansion:** two elements that make the same
  key are an error.
- **Across tasks,** a file path, a data name and a secret's or identity's
  path belong to one task. Users, groups, presets, tmpfiles lines,
  preseeds and boot parameters may be declared by several tasks only
  identically. Packages and modules are a union. Anything else is a
  collision, an error that names both tasks (render.md, 3.7).
- Accounts from a setting, in `tasks/users/task.json`:

  ```json
  {
    "users": {
      "{{ key }}": {
        "for_each": "settings.accounts",
        "uid": "{{ value.uid }}",
        "home": "/home/{{ key }}",
        "shell": "{{ value.shell }}",
        "groups": "{{ value.groups }}",
        "password": "{{ value.password }}"
      }
    }
  }
  ```

### 5.3 Packages, components and preseeds

- **`packages`** is a map keyed by package name, with `if`, `for_each`
  and `component` (enum `main`, `contrib`, `non-free`,
  `non-free-firmware`; default `main`). The packages part requests the
  union over the target's tasks.
- **`component` is a check, never a switch.** A package whose component
  the target's `debian.components` does not enable is an error that names
  both, instead of apt's failure later. Components are a setting; no task
  name and no hardware value turns one on (os's `desktop` task and
  `7th-gen` did).
- **`debian`** is the one schema hearth fixes. It is reserved in every
  document and merges like any object:

  | Field | Type | Default | Meaning |
  |---|---|---|---|
  | `suite` | string | required | `trixie`, `forky` |
  | `architecture` | string | required | `amd64` |
  | `snapshot.archive` | timestamp | required | snapshot.debian.org's timestamp for `debian`, as `20261001T030405Z` |
  | `snapshot.security` | timestamp | required with the security pocket | the timestamp for `debian-security` |
  | `components` | map of component to bool | `{"main": true}` | `main` must stay on |
  | `pockets.updates`, `pockets.security` | bool | true | the `-updates` and `-security` suites |
  | `install_recommends` | bool | false | apt's `Install-Recommends` |

  The archives' URLs are hearth's, fixed:
  `http://snapshot.debian.org/archive/debian/<timestamp>/` and
  `…/debian-security/<timestamp>/`. Where the bytes come from (a proxy,
  the test tier's mirror) is the builder's own configuration, outside
  every spec. os's rule that `forky` has no updates or security lines
  becomes two booleans. Timestamps should be the snapshot's own, since
  snapshot.debian.org redirects any other to the nearest earlier one.
- **`debian`'s default is the bottom layer,** as a task's (6.3), so
  `{"contrib": true}` keeps `main` on; `main` written `false` or `null`
  is an error. A target whose layers set no `debian` is an error at its
  base's document. A `snapshot.security` beside a security pocket that
  is off is dropped, as the pocket's line is.
- **Names are Debian Policy's (5.6.1):** a package's name, `suite` and
  `architecture` are lowercase letters, digits, `+`, `-` and `.`, two
  bytes at least, starting with a letter or a digit, and at most 256
  bytes. A timestamp is `YYYYMMDDTHHMMSSZ`, its digits checked, not its
  date. `components`' keys are the four above. Render refuses any other
  where it is written, so no tool reads one.
- **`preseeds`** is a map keyed by debconf question, with `type` (enum
  `select`, `multiselect`, `string`, `boolean`) and `value`, both
  required, and `owner` (default: the question's first segment). There
  is no `password` type: a preseed is in the spec, and a spec holds no
  secret. They are answered before any package is configured (build.md,
  section 5). The timezone
  and locale, which os and the bash fleet set in code, are the `locale`
  task's:

  ```json
  {
    "doc": "The timezone and the locale, answered to debconf before packages are configured.",
    "settings": {
      "timezone": {
        "type": { "object": {
          "area": { "type": "string", "doc": "tzdata's area, such as Europe." },
          "zone": { "type": "string", "doc": "The zone in the area, such as Lisbon." }
        } },
        "doc": "The system's timezone."
      },
      "locale": { "type": "string", "default": "C.UTF-8", "doc": "The default locale, generated and set." }
    },
    "packages": { "locales": {}, "tzdata": {} },
    "preseeds": {
      "tzdata/Areas": { "type": "select", "value": "{{ settings.timezone.area }}" },
      "tzdata/Zones/{{ settings.timezone.area }}": { "type": "select", "value": "{{ settings.timezone.zone }}" },
      "locales/default_environment_locale": { "type": "select", "value": "{{ settings.locale }}" },
      "locales/locales_to_be_generated": { "type": "multiselect", "value": "{{ settings.locale }} UTF-8" }
    }
  }
  ```

- **A preseed is one line of `debconf-set-selections`** (build.md, 5.5),
  so render refuses what would break it: a question is two or more
  segments joined by `/`, and an owner one, each of letters, digits,
  `.`, `_`, `+` and `-`; a value holds no control character and does not
  end in `\`, which joins the next line; a `boolean`'s value is `true`
  or `false`. Other values are debconf's to read.

### 5.4 The initramfs and the command line

- **`initramfs`** is a map keyed by kernel module name, with `if` and
  `for_each`: letters, digits, `_` and `-`, at most 55 bytes, the
  kernel's `MODULE_NAME_LEN` less its NUL. Their union is the
  configuration's `INCLUDE_MODULES`. The
  rest of the configuration is no task's: the packages part carries
  `microcode`, `none` on a VM and `generic` on metal, which render
  derives from placement (machine.md, 7.1), and render writes
  `tiny-initramfs.conf` from the modules and that choice, with
  `AUTO_MODULES=no` always (render.md, 4.2). So neither `yes`, which
  would read the build host, can reach an output (5.11).
- **`cmdline`** is a map of entries with `param` (the parameter as the
  kernel reads it: printable ASCII but a space, `"` and `'`, and `\`,
  `$`, `;`, `&`, `|`, `<`, `>`, `{`, `}` and `#`, which GRUB's script
  would read in a BIOS slot's fragment, machine.md, 5.4) and `order`. The
  boot part is the parameters in order (4.3). Ten keys are the
  activator's or would change how the root is found (5.11). A line that
  would pass 2,048 bytes with the activator's tokens is refused at the
  `param` of the first entry, in order, that takes it past. Boot
  parameters are a slot part, so changing one builds nothing: it is a
  new slot of the same output.
- **Modules by hardware profile** are data, not branches. The bash
  fleet's table, from its `tiny-initramfs.conf` template, becomes the
  default of a setting of the `boot` task:

  ```json
  {
    "doc": "The kernel, its initramfs and its command line.",
    "settings": {
      "kernel": { "type": "string", "default": "linux-image-amd64", "doc": "The kernel package." },
      "params": {
        "type": { "map": { "object": {
          "param": { "type": "string", "doc": "One parameter, as the kernel reads it." },
          "order": { "type": "int", "min": 0, "max": 999, "default": 500, "doc": "Its place on the line." }
        } } },
        "default": {}, "doc": "Command-line parameters, by name."
      },
      "modules": {
        "type": { "map": { "list": "string" } },
        "default": {
          "7th-gen": ["btrfs", "nvme"],
          "ionos": ["btrfs", "virtio_blk", "virtio", "virtio_pci", "virtio_rng", "scsi_mod", "vmw_pvscsi"],
          "serverhino": ["btrfs", "scsi_mod", "sd_mod", "virtio_scsi", "virtio_pci", "virtio_mmio"],
          "vultr": ["btrfs", "virtio_blk", "virtio_pci"]
        },
        "doc": "Initramfs modules, by hardware profile."
      }
    },
    "requires": {
      "kernel": { "if": "target.placement.kind != 'container'", "message": "a container has no kernel: its host runs it" },
      "modules": { "if": "target.placement.kind == 'container' or target.placement.hardware in settings.modules", "message": "no initramfs modules for this hardware" }
    },
    "packages": {
      "{{ settings.kernel }}": {},
      "tiny-initramfs": {},
      "efibootmgr": { "if": "target.placement.firmware == 'efi'" }
    },
    "initramfs": { "{{ value }}": { "for_each": "settings.modules[target.placement.hardware]" } },
    "cmdline": {
      "{{ key }}": { "for_each": "settings.params", "param": "{{ value.param }}", "order": "{{ value.order }}" }
    }
  }
  ```

  The lists are the bash fleet's and machine.md's (7.2). The `incus`
  profile is not a default: the bash fleet's list served a virtiofs root,
  and hearth's test VMs boot from a disk of their own. `envs/lab.json`
  adds it, with the lab's placement and console:

  ```json
  {
    "_note": "Every lab target: an Incus VM with its own disk, trusting the lab's CA.",
    "placement": { "kind": "vm", "firmware": "efi", "hardware": "incus" },
    "boot": {
      "modules": { "incus": ["btrfs", "virtio_pci", "virtio_scsi", "sd_mod"] },
      "params": { "console": { "param": "console=ttyS0,115200n8", "order": 900 } }
    },
    "certificates": { "extra_ca": { "lab": "lab-ca/lab.crt" } }
  }
  ```

  The `incus` list is to verify on the VM tier (machine.md, 7.2;
  testing.md, section 7).

### 5.5 The file tree

- **`tree/` mirrors `/`.** A file at `tree/<path>` installs at `/<path>`,
  less a `.j2` suffix. Its mode is 0755 if git records it executable, and
  0644 otherwise; a symbolic link in git is a symbolic link, its target
  as written. Owner and group are root. A link's target, in `tree/` or a
  `link` entry, is 1 to 4,095 bytes with no NUL: Linux's `PATH_MAX`
  less its NUL.
- **`files` is a map keyed by absolute destination path.** An entry is
  one of:

  | Entry | Means |
  |---|---|
  | no source field | the file of `tree/` at this path, or with a key ending in `/`, the directory of `tree/` and its subtree |
  | `"template": "templates/NAME.j2"` | that template, rendered to this path |
  | `"copy": PATH` | a repository file, of type `file` (6.2), copied byte for byte |
  | `"link": TARGET` | a symbolic link |
  | `"directory": true` | an empty directory, which git cannot hold |

  with the fields `mode` (an octal string, `"0640"`), `owner` and `group`
  (names, default `root`), `if` and `for_each`.
- **Forms:** a key ends in `/` exactly when it names a directory of
  `tree/`, and then takes no source field, `directory` included.
  `directory` is `true` or absent. A `template` or `copy` entry's mode
  is 0644 unless it gives one, whatever git records of its source. A
  `template` is written as is, never interpolated, so that 2.1's check
  that every template is used stays exact.
- **A directory's entry governs its subtree:** its `if` covers every file
  beneath it; its `owner` and `group` apply, each alone, to entries
  beneath that do not give that field; and its `mode` is the
  directory's. A `tree/` path that an entry names is installed only
  where the entry is: a false `if` leaves it out, a file or a subtree.
- **An entry for a path that `tree/` lacks is an error,** as is a path
  that both `tree/` and a `template`, `copy` or `link` entry give.
- **Owners are names.** Build resolves them in the layer's own
  `/etc/passwd` and `/etc/group`, once the users are applied
  (build.md, section 7); a name it cannot resolve fails the build.
  `root` is always id 0.
- **A file replaces whatever the packages put at its path,** mode and
  owner included. os kept a packaged file's mode, so a task could not
  tighten a conffile.
- **Directories a path implies** are created 0755 and root-owned, only
  where the layers beneath lack them; a declared directory sets its own.
- **Under `/usr`, only `/usr/local`.** The rest of `/usr` is the
  packages layer's, as it was under os, whose tasks' helpers went to
  `/usr/local/bin` and their shared files to `/usr/local/share`. A unit
  a task ships goes in `/etc/systemd/system/`. So nothing above the
  packages changes what every process maps from `/usr`, and a live
  change overlays `/usr/local` alone of it (slots.md, 6.6). The build
  holds scripts to the same rule (build.md, 9.3).
- **Forbidden destinations:** anything under a data path or a class's
  mount point (5.7), which a mount would hide, and the paths hearth
  reserves (5.11).
- One file per user, from a map setting, in `tasks/ssh/task.json`:

  ```json
  {
    "files": {
      "/etc/ssh/sshd_config.d/local.conf": { "if": "settings.allow" },
      "/etc/ssh/authorized_keys_{{ key }}": {
        "for_each": "settings.authorized",
        "if": "key in settings.allow",
        "template": "templates/authorized_keys.j2"
      }
    }
  }
  ```

  with `templates/authorized_keys.j2`:

  ```jinja
  {% for name, line in value | items %}
  {{ line }}
  {% endfor %}
  ```

### 5.6 Users, groups, presets and tmpfiles

- **`users`** is a map keyed by user name. Build writes them as sysusers
  records and applies them in the config layer (build.md, section 7).

  | Field | Type | Default |
  |---|---|---|
  | `uid` | int | a system id that sysusers picks |
  | `gid` | int | none; a package user's alone: its primary group's id, in the line build seeds (a sysusers user's group is `group`) |
  | `group` | string | the user's own group, made with the same id |
  | `gecos` | string | empty |
  | `home` | path | `/` |
  | `shell` | path | `/usr/sbin/nologin` |
  | `groups` | list of strings | none: supplementary groups, as sysusers `m` lines |
  | `password` | secret | none: the account stays locked (9.2) |
  | `subordinate` | `{ "base": int, "count": int }` | none: a subordinate uid and gid range |
  | `from` | enum `sysusers`, `package` | `sysusers`: made by build; `package`: made by a package's maintainer script |

- **A user its packages create** is declared with `"from": "package"`,
  with no other field but `uid`, `gid`, `if` and `for_each`. Render knows
  the name and checks it as it checks a declared user, but never writes
  it: build writes no sysusers line for it, and the activator confirms it
  exists in the slot's `/etc/passwd` at prepare (slots.md, 5.7). A name
  declared by two tasks, once from a package and once for sysusers, is a
  collision.
- **`groups`** is a map keyed by group name, with `gid` and `from`, as
  for users: a group a package creates is declared with
  `"from": "package"`. Like a package user, build writes no sysusers
  line for it, and the activator confirms it exists in the slot's
  `/etc/group` at prepare (slots.md, 5.7).
- **Persistent owners have fixed ids.** A package's postinst, and
  sysusers for a user with no `uid`, take the next free id when they
  install, and the packages layer is rebuilt from scratch: a snapshot
  bump could renumber `postfix` while its data keeps the old ids. So
  every user or group that owns a data path (5.7), a secret file or an
  identity file (section 9) has a fixed id, package users included, and
  render refuses one that has none. Build seeds the package users' and
  groups' lines into `/etc/passwd` and `/etc/group` before any package
  installs, so adduser finds them (build.md, section 5). In the mail
  task:

  ```json
  {
    "users": { "postfix": { "from": "package", "uid": 107, "gid": 113 } },
    "groups": { "opendkim": { "from": "package", "gid": 120 } }
  }
  ```

  The ids are the task's, fixed once and the same on every machine.
  Every machine is installed fresh (README, section 7), so they need
  match no os machine's.
- **`subordinate` ranges** are the one thing sysusers cannot make. Render
  writes `/etc/subuid` and `/etc/subgid` from them into the file tree,
  and refuses two ranges that overlap; no task ships either file.
- **Ids are one user's:** two users or two groups with one id are an
  error.
- **A sysusers user's own group,** what an unset `group` names, has the
  user's name and the user's `uid` as its id. A `groups` entry of that
  name is that group, so its `gid` is the user's `uid`, and another is a
  collision (render.md, 3.7): user `free` with `uid` 1000 and group
  `free` with `gid` 1001 differ. A user whose primary group has another
  id names a group of another name in `group`.
- **A package user gives both `uid` and `gid`, or neither:** build seeds
  its passwd line from both (build.md, 5.5), and one alone is an `ids`
  error (render.md, 7.2). With neither, nothing is seeded, and the
  user may own nothing that needs a fixed id (above).
- **Ids and strings are bounded:** a `uid` or `gid` is 1 to 65533: 0 is
  root's, 65534 nobody's and 65535 no id, and the builder's namespace
  maps no more (store.md, 3.4). A `subordinate` range has a count of 1
  at least and a base of 65,536 at least, so that no id a target's
  files own is subordinate, and base plus count is at most
  4,294,967,295. `gecos`, `home` and `shell` hold no `:`, which splits a
  passwd line, no `"` or `\`, which sysusers reads as quoting, and no
  control character. Render refuses any other where it is written.
- **`presets`** is a map keyed by unit name, with `action`, `enable` or
  `disable`. Build applies them with the target's own systemctl
  (build.md, section 7). A unit is masked by a `link` to `/dev/null` in
  `files`.
- **`tmpfiles`** is a map keyed by path, with `type` (enum `d`, `D`, `e`,
  `f`, `f+`, `w`, `w+`, `L`, `L+`, `C`, `z`, `Z`, `r`, `R`), `mode`,
  `owner`, `group`, `age` and `argument`, as tmpfiles.d(5) reads them.
  `v`, `q` and `Q` are reserved, so a line of one is refused as such
  (5.11), not as an unknown type; any other letter is a schema error.
  Build applies them to the image and writes them for every boot
  (build.md, 7.4), which is what directories inside `/var` and the
  data entries need. Lines that make subvolumes, lines that write over
  what is there outside `/var`, and lines for a path under `/usr` but
  `/usr/local`, are refused (5.11).
- **Homes live on persistent data:** the users task declares each
  account's home as a `data` entry of its own (5.7, 6.6).

### 5.7 Data

- **Persistent data lives in classes.** A target has up to three class
  subvolumes at the btrfs top level, `@<target>.core`, `@<target>.data`
  and `@<target>.cache`, each made only when some entry uses it, and
  mounted at `/srv/core`, `/srv/data` and `/srv/cache`
  (machine.md, section 3). They outlive every slot. The classes are os's
  and the bash fleet's, whose reason was a snapshot policy per class:

  | Class | Holds | Snapshots and backups (5.12) |
  |---|---|---|
  | `core` | the machine's own state: the journal, iwd's networks | its own policy, typically daily |
  | `data` | services' data and people's homes: a mail spool, a database, `/home` | its own policy, typically hourly; the class backed up [M6] |
  | `cache` | what can be made again: downloads, build caches | never snapshotted, never backed up |

- **`data` is a map keyed by name; each entry is a directory** in its
  class's subvolume, named by the entry's name (`/srv/core/journal`,
  `/srv/data/postfix`), and bind-mounted at the path the task declares,
  from the fstab the activator writes (machine.md, 3.3). An entry never
  names a subvolume.

  | Field | Type | Default |
  |---|---|---|
  | `path` | path | required |
  | `class` | enum `core`, `data`, `cache` | `data` |
  | `mode` | mode | `0755` |
  | `owner`, `group` | name, with a fixed id (5.6) | `root` |

- **An entry whose path is its class's directory,** `/srv/<class>/<name>`,
  needs no bind: the class's mount shows it. That is its own class and
  name only: `/srv/core/journal` for an entry `journal` of class `core`.
  Any other path at or beneath `/srv/core`, `/srv/data` or `/srv/cache`,
  another entry's directory or a mount itself, is refused.
- **The activator makes the class subvolume** when the top level lacks
  it, with a root of `0:0 0755`, and **the entry's directory** when the
  class lacks it, seeded once from the output's content at the entry's
  path (slots.md, 5.3), so what packages put there (a database's
  cluster, a mail spool) is not hidden by an empty bind. The data part
  holds names, classes and paths, a slot part that enters no layer key.
- **Mode and owner are applied at every boot:** render turns them into a
  `tmpfiles` line of type `d` at the entry's path, which systemd applies
  in the running slot, through the bind, where names resolve. So they,
  unlike the path and the class, enter the config key.
- **Paths** are beneath `/srv`, `/var` or `/home`, never one of the three
  itself, whose bind would hide the rest of it, never `/var/lib/hearth`
  or beneath it, and never beneath another entry's path. A target has at
  most 64 entries (machine.md, 3.4): the 65th, in name order (4.3), is a
  schema error at its key. The rest of `/` stays immutable.
- **Names are per target, across classes:** two entries of one target
  with one name are a collision (render.md, 3.7), even in two classes.
  Moving an entry to another class moves no data, so the activator
  refuses it while the old directory exists (machine.md, 3.4).
- **Snapshots and backups are by class, never by entry:** their policies
  are the `hearth` task's settings (5.12). A snapshot is of a whole
  class subvolume, so it is atomic across every service in the class,
  postfix's spool and its mailboxes together; one entry is restored by
  copying its directory out of a snapshot (operating.md, 7.4).
- **Data keeps a program's default path.** An entry is bound where the
  program already looks, as Debian ships it, so no configuration changes
  for it: journald writes `/var/log/journal`, iwd keeps its networks in
  `/var/lib/iwd`. os did the same, binding the directories
  `/srv/core/journal` and `/srv/core/iwd` of its `@<target>.srv.core`
  subvolume onto those paths; hearth keeps the classes and the binds,
  and declares them per entry.
- The journal, which the bash fleet and os bound from a hard-coded
  `@<target>.srv.core/journal`, is the `core` task's:

  ```json
  {
    "data": {
      "journal": {
        "if": "target.placement.kind != 'container'",
        "class": "core", "path": "/var/log/journal", "mode": "2755", "group": "systemd-journal"
      }
    },
    "groups": { "systemd-journal": { "from": "package", "gid": 999 } }
  }
  ```

  The group owns persistent data, so its id is fixed (5.6); 999 is
  what systemd's sysusers gives it on the fleet's machines today, to
  verify. A container has no `journal` entry: its journal lives in its
  host's (3.5) [M5].

- iwd's state, its known networks and their passphrases, which os bound
  from `/srv/core/iwd` only when the Wi-Fi task was present, is the
  `wifi` task's, at iwd's own default path:

  ```json
  {
    "doc": "Wi-Fi through iwd, its known networks kept across slots.",
    "packages": { "iwd": {} },
    "data": { "iwd": { "class": "core", "path": "/var/lib/iwd", "mode": "0700" } }
  }
  ```

  iwd runs as root, so the root-owned default needs no fixed id; the
  mode keeps the passphrases from other users.
- A mail server's spool and mailboxes are the `mail` task's, in the
  `data` class, so one snapshot holds both as they were at one instant:

  ```json
  {
    "data": {
      "postfix": { "path": "/var/spool/postfix" },
      "mail": { "path": "/var/mail", "mode": "2775", "group": "mail" }
    },
    "groups": { "mail": { "from": "package", "gid": 8 } }
  }
  ```

  The spool's directory is root's, as Debian ships it, and the queues
  beneath keep the owners postfix gave them; `mail` is base-passwd's
  static group.
- **On x1-prod** the entries make `@x1-prod.core`, with `journal` and
  `iwd`, and `@x1-prod.data`, with `home-free` (6.6), bound at
  `/var/log/journal`, `/var/lib/iwd` and `/home/free`; neither journald
  nor iwd is configured to find them. Its fstab is machine.md, 3.3's
  example.

### 5.8 Escape hatches

- **`scripts` names the task's shell fragments and their order:**

  ```json
  { "scripts": { "customize.sh": { "order": 300 }, "software.sh": { "order": 500 } } }
  ```

  A declared script must exist in the task directory as `NAME` or
  `NAME.j2`, and a script file that is not declared is an error. Its
  keys are those two alone (2.1): any other is a schema error.
- **`customize.sh`** runs in the config layer, after its files and users
  are in place (build.md, section 7). **`software.sh`** runs in the
  software layer, after every download is placed at its declared path
  (build.md, section 6), so it names a download by that path instead of
  guessing a cache's layout, as os's examples did and got wrong.
- **Fragments run in order** (4.3), each as its own `sh -eu` script, so
  an error names its task and no `cd` or `set` leaks into the next one.
  os concatenated them in alphabetical order of task.
- **They run in the step's sandbox, offline** (build.md, section 8), and
  their output is part of the layer like any file.
- **They are the exception.** What the bash fleet's fragments did (users,
  modes, symbolic links, software unpacked into `/opt`) is declarative
  here. What remains for them: `postmap`, `update-ca-certificates`, and
  the like.

### 5.9 Requires

- **`requires`** is a map of guards, each with `if` and `message`. Render
  evaluates them after validation and before any entry; a false guard is
  an error carrying its message and its line in `task.json`.
- Most of os's guards are types now (`"type": {"enum": [...]}`).
  `requires` is for what spans settings or reads placement: the boot
  task refuses a container, and its module table must hold the target's
  hardware profile (5.4).

### 5.10 From the fleet's code to task data

| Detail in code | As data |
|---|---|
| `non-free` for `host.hardware == "7th-gen"` (os, bash `manifest`) | `debian.components` in `hosts/x1/x1-prod.json`, and the `firmware` task checks it (below) |
| Europe/Lisbon and en_US preseeds (os's `install.rs`, bash's `deploy`) | the `locale` task's preseeds (5.3); the values in `fleet.json` (3.7) |
| the persistent `journal` and `iwd` directories (os's `provision`) | `data` entries of `core` and `wifi` (5.7) |
| gettys masked in containers (os's and bash's `deploy`) | `files` links in `core`, conditional on placement (below) |
| `domain=ekanayaka.io` (bash's `manifest`) | the `core` task's `domain` setting, set in `fleet.json` (3.7) |
| SSH port 10127 (bash's ssh templates) | the `ssh` task's `port` setting, default 22, set in `fleet.json` (3.7), read as `Port {{ settings.port }}` in its sshd drop-in |

- **Firmware for x1's hardware.** `tasks/firmware/task.json`:

  ```json
  {
    "doc": "Firmware for Intel 7th-generation laptops.",
    "packages": {
      "firmware-intel-sound": { "component": "non-free-firmware" },
      "firmware-iwlwifi": { "component": "non-free-firmware" },
      "firmware-linux-free": {},
      "firmware-sof-signed": { "component": "non-free-firmware" }
    }
  }
  ```

  x1-prod has it, with `non-free-firmware` on, in its base-env file
  (3.7). The bash fleet also switched on `non-free`, which none of these
  packages needs.
- **Gettys in containers,** masked by links in `tasks/core/task.json`:

  ```json
  {
    "files": {
      "/etc/systemd/system/{{ value }}": {
        "for_each": "['getty@.service', 'serial-getty@.service', 'console-getty.service']",
        "if": "target.placement.kind == 'container'",
        "link": "/dev/null"
      }
    }
  }
  ```

### 5.11 What hearth reserves

What a later step would refuse, render refuses first, at the line that
declares it (render.md, 7.2):

| Reserved | Why | Defined in |
|---|---|---|
| command-line keys `root`, `rootflags`, `rootfstype`, `rootwait`, `rootdelay`, `ro`, `rw`, `init`, `initrd`, `panic`, and every `hearth.*` key (the trial's token, slots.md, 6.3); a line past 2048 bytes with the activator's tokens | the activator's, or they change how tiny-initramfs finds and starts the root | machine.md, 6.2 |
| task files at `/etc/fstab`, anything under `/.hearth/`; in `/etc/systemd/system.conf.d/`, a name beginning `zz-hearth` or sorting after `zz-hearth-watchdog.conf` | the activator writes them at prepare, and the watchdog's drop-in must sort last | slots.md, 5.5 |
| units, links and presets named `hearth-trial*`; `hearth-check-*` | the dead-man's; checks' transient units | slots.md, 5.5; operating.md, 2.4 |
| [M5] presets and links that enable `systemd-nspawn@<c>.service` for a container `c` placed on the target | the host's activator starts each container once its link and settings exist; one started before fails | slots.md, 12.2 |
| task files at `/etc/passwd`, `/etc/group`, `/etc/shadow`, `/etc/gshadow`, `/etc/subuid`, `/etc/subgid`, `/etc/machine-id`, `/var/lib/dbus/machine-id` | users are declared (5.6); identity is written at prepare | build.md, 7.3 and 9.2 |
| task files at `/etc/ssh/ssh_host_*_key` and their `.pub`, `/etc/dropbear/dropbear_*_host_key`, anything under `/etc/ssl/private/`, `/var/lib/systemd/random-seed`, `/var/lib/systemd/credential.secret` | identity and secrets: `identity` and `secrets` entries name these paths, files never do | build.md, 9.2 |
| anything under `/var/lib/hearth/` | the mount points of hearth's state and store | machine.md, 3.4 |
| task files, links, downloads and tmpfiles lines under `/usr` but `/usr/local` | the rest of `/usr` is the packages layer's: nothing above it may change what every process maps, and live changes overlay `/usr/local` alone | 5.5; 10.1; build.md, 9.3; slots.md, 6.6 |
| anything under `/srv/core`, `/srv/data` or `/srv/cache` but a data entry at its own `/srv/<class>/<name>` (5.7) | the class subvolumes' mounts hide what the slot holds there | 5.7; machine.md, 3.4 |
| anything under `/dev`, `/proc`, `/sys`; anything but a tmpfiles line under `/run`, `/tmp`, `/var/tmp`; a file under `/var/log` that is not empty; `/etc/.git`, `/etc/.etckeeper` | runtime, noise, history; a tmpfiles line there waits for boot (build.md, 7.4) | build.md, 9.2 |
| a task file at `/etc/tiny-initramfs/tiny-initramfs.conf`, so `AUTO_MODULES=yes` and `MICROCODE=yes` cannot reach an output | the packages part's, written before the kernel installs | build.md, 5.5; machine.md, 7.1 |
| tmpfiles `v`, `q`, `Q`; `w`, `w+`, `f+`, `L+` outside `/var`, `/run` and `/tmp` | a nested subvolume breaks a layer; a write on the read-only root fails at boot | build.md, 7.4 |
| preseeds of type `password` | they leave the password in `/var/cache/debconf/passwords.dat` | build.md, 9.2 |
| a secret or identity file owned by user `hearth`, or by a group `hearth` belongs to (`systemd-journal`); a secret's or identity item's `mode` with a bit outside `0770` | neither the steward nor its builder may ever read a secret or a private key | activator.md, section 2; 9.1; 9.3 |
| an owner of a data path, a secret or an identity file without a fixed id; a download's owner other than `root` or a package user with a fixed id | ids that move between builds; no sysusers user exists when software is placed | 5.6; 10.1 |
| `live` entries that name a never-live unit or path (5.14); a `unit` with action `none`; a path not beneath `/etc`, `/usr/local` or `/opt` | a never-live unit must not be acted on live, and a change elsewhere is never live | 5.14; slots.md, 6.7 |
| presets and links that enable `systemd-sysext.service` or `systemd-confext.service`; anything under `/etc/extensions`, `/usr/lib/confexts`, `/usr/local/lib/confexts`, `/var/lib/extensions` or `/var/lib/confexts` | nothing may merge extensions over hearth's live overlays, at boot or after a soft reboot, a container's included, which has no `hearth` task to disable them | 5.12; slots.md, 6.6 |

- **A row's paths hold for whatever is placed at a path:** a `tree/`
  file, a `files` entry or link, a download, a secret or identity file,
  a tmpfiles line and a data path, whatever the row's wording, "task
  files at" or "anything under". "Under" a path is that path and every
  path beneath it. Two rows leave a kind out: secrets and identity are
  what the identity and secrets row's paths are for, and a tmpfiles line
  may stand under `/run`, `/tmp` and `/var/tmp`. So a data path at
  `/var/lib/dbus/machine-id`, or a tmpfiles `f` line at
  `/var/lib/systemd/random-seed`, is refused.
- **Above a reserved path, only a directory may stand:** a `tree/`
  directory, a `directory` entry, or a tmpfiles line of type `d`, `e` or
  `z` without an `age`, which act on the directory alone. Anything else
  at a strict ancestor of a path a row names, or at the directory a
  row's pattern matches in (`/etc/ssh`, for `ssh_host_*_key`), is
  refused under that row: a data path, whose bind would hide it; a
  file, a link, a secret or an identity file, which would put something
  other than a directory there; a download, whose archive may hold it;
  and any other
  tmpfiles line, which may act beneath its path (`D`, `R`, `Z`, `C`, an
  `age`).
- **A tmpfiles line names one path:** its key holds no `*`, `?` or `[`,
  which tmpfiles reads as a glob, and no `%`, which it reads as a
  specifier, so that each row checks it as written.

### 5.12 The hearth task

- **Every target has the `hearth` task,** but a container (below),
  without naming it: render adds it, and `"hearth": null` is an error.
  In any layer, `fleet.json` and an environment's included, it is a
  `reserved` error at the key (render.md, 7.2), and no other kind
  reports it.
  Every such output then carries hearth's binary, its activator's unit
  (which starts the steward, which starts builders) and its user, which
  machine.md (8.3) and slots.md assume. Its files are in the repository
  like any task's, `tasks/hearth/`, so the release a fleet runs is
  pinned by a change.
- **But a container's:** render adds no `hearth` task to a
  container's target, and naming it there, in its base, member or
  base-env layer (3.3), is a `reserved` error at the key. The `hearth`
  settings of `fleet.json` and of an environment reach no container,
  since it lacks the task (3.3). No hearth runs in
  a container [M5] (slots.md, 12.5): its host's builder builds its output and
  its host's activator prepares and starts it, so an activator enabled
  inside would only fail, and the tools, the user and the range would
  serve nothing. Should a container run a hearth of its own, this
  exception goes.
- `tasks/hearth/task.json`:

  ```json
  {
    "doc": "hearth itself: its binary, its activator's unit, its user, its tools and its alarms.",
    "settings": {
      "alarms": {
        "type": { "map": { "object": {
          "warning": { "type": "int", "optional": true, "doc": "The threshold of a warning." },
          "critical": { "type": "int", "optional": true, "doc": "The threshold of a critical alarm." }
        } } },
        "default": {
          "disk_free_percent": { "warning": 10, "critical": 5 },
          "disk_free_gib": { "warning": 5, "critical": 1 },
          "memory_available_percent": { "warning": 10, "critical": 5 },
          "pressure_some_percent": { "warning": 20 },
          "pressure_full_percent": { "critical": 10 }
        },
        "doc": "Resource alarms' thresholds (operating.md, section 4)."
      },
      "snapshots": {
        "type": { "map": { "object": {
          "every": { "type": "duration", "doc": "How often the class is snapshotted." },
          "keep": { "type": { "map": "int" }, "doc": "How many to keep per bucket: hourly, daily, weekly, monthly." }
        } } },
        "default": {},
        "doc": "Local snapshots of each class's subvolume, by class: core or data [M5] (operating.md, 7.1)."
      },
      "backups": {
        "type": { "map": { "object": {
          "every": { "type": "duration", "doc": "How often the class is sent to the backup peer." },
          "keep": { "type": { "map": "int" }, "optional": true, "doc": "What the peer keeps, per bucket; the class's local keep if absent." }
        } } },
        "default": {},
        "doc": "Backups of each class to the environment's backup peer, by class [M6] (operating.md, 7.2)."
      }
    },
    "packages": { "git": {}, "mmdebstrap": {}, "uidmap": {}, "openssh-client": {} },
    "software": {
      "hearth": {
        "url": "https://git.ekanayaka.io/ai/hearth/releases/download/v0.1.0/hearth-x86_64-linux",
        "sha256": "…", "unpack": "file", "path": "/usr/local/bin/hearth", "mode": "0755"
      }
    },
    "users": {
      "hearth": {
        "uid": 59999, "home": "/nonexistent", "groups": ["systemd-journal"],
        "subordinate": { "base": 262144, "count": 65536 }
      }
    },
    "presets": {
      "hearth-activator.service": { "action": "enable" },
      "systemd-sysext.service": { "action": "disable" },
      "systemd-confext.service": { "action": "disable" }
    }
  }
  ```

  with `tree/etc/systemd/system/hearth-activator.service`, the unit of
  activator.md (3.1); `tree/usr/local/lib/hearth/askpass`, which gives
  git the fetch's credential (render.md, 2.4); and
  `tree/etc/hearth/alarms.json.j2`, which is
  `{{ settings.alarms | tojson }}`.
- **Where hearth's files live:** under `/usr/local` and `/etc`, as
  every download and every task file does (5.5, 10.1), since `/usr/lib`
  and the rest of `/usr` are Debian packages' alone. Should hearth
  become a Debian package, the task installs it as one, and it moves to
  `/usr` with the packages layer, its unit in `/usr/lib/systemd/system/`
  as any package's.
- **The uid, gid and range are constants of the task,** not settings:
  every slot of a machine mounts the same `@hearth.store`, owned by them
  (machine.md, 8.3; store.md, 9.2). The range is 64 Ki at base 262144.
  The uid 59999, with its own group of the same id, sits at the top of
  the ids adduser hands to people from 1000 up, and away from the system
  ids packages take below 1000 (to verify against the fleet's packages).
- **`systemd-journal`** lets the steward read the journal for telemetry
  (wire.md, section 7); its builder runs with no supplementary group
  (steward.md, 2.3).
- **Snapshots and backups are declared once, by class** (5.7), normally
  in `fleet.json` or an environment file:

  ```json
  {
    "hearth": {
      "snapshots": {
        "core": { "every": "1d", "keep": { "daily": 7 } },
        "data": { "every": "1h", "keep": { "hourly": 24, "daily": 14, "weekly": 8 } },
        "cache": null
      },
      "backups": { "data": { "every": "1h" } }
    }
  }
  ```

  - **`snapshots`** are local, snapper's role [M5]: the activator takes
    read-only snapshots of the class's subvolume on the period and
    prunes them by retention; they stay on the machine, readable by a
    person and restorable (operating.md, 7.1 and 7.4).
  - **`backups`** are what is sent to the backup peer, encrypted with
    `age` [M6] (operating.md, 7.2).
  - **`cache` is never snapshotted or backed up:** a key other than
    `core` and `data` is an error, and `"cache": null` only says so,
    since null deletes (4.1). Buckets are `hourly`, `daily`, `weekly`
    and `monthly`, each at least 1.
  - **Render puts them in the data part** (render.md, 4.3), so a change
    to one prepares a new slot of the same output, which the activator
    then follows (slots.md, 2.2).
  - **A container has no `hearth` task:** its classes follow its host's
    policies, which the host's activator applies to them under the
    container's target (slots.md, 12.1) [M5].
- **`systemd-sysext` and `systemd-confext` are disabled.** Debian's
  `90-systemd.preset` enables both; hearth makes its live overlays
  itself (slots.md, 6.6), and a boot service that merged an extension
  would stack it over them, or over a root it was not made for after a
  soft reboot.
- **The tools** hearth runs as child processes come from Debian: git
  for render, mmdebstrap and `newuidmap` for builds (store.md, 9.3), and
  `ssh-keygen` for the activator's host keys (9.3).

### 5.13 Units that may fail

- **A task may name units whose failure does not block a bless,** such as
  a desktop's flaky units:

  ```json
  { "bless": { "ignore_failed": ["foo.service"] } }
  ```

- **The target's list is the union over its tasks,** in the checks part
  (render.md, 4.4), which holds it from M3. The minimal bless counts only
  the failed units that are not on it (slots.md, 7.1).
- **Before M3, which brings the bless,** render checks each list and
  places it in no part. Unlike a check (section 11), the list only
  relaxes: with no bless, no failure is ignored, so nothing it declares
  is dropped silently.
- **A failed unit on the list is still reported** as a fact
  (operating.md, section 4); it only stops counting against a bless.
- **A container's list holds as a host's [M5]:** its host reads the
  container's failed units from outside during its trial
  (slots.md, 12.3).
- **A container never counts against its host [M5]:** the host's
  minimal bless leaves out its containers' nspawn units without any
  list, since each container's slot has its own trial (slots.md, 7.1).

### 5.14 Live changes

A change applies live, with no reboot, only if every path it changes
has a consumer: a unit to act on, or a declaration that none is needed
(slots.md, 6.7, rule 7). A task declares its own. Without one the
change takes an ordinary trial, a soft reboot or a reboot; that is never
an error.

- **`live` is a map of entries keyed by name,** with `if` and
  `for_each` (5.2):

  | Field | Type | Default | Meaning |
  |---|---|---|---|
  | `unit` | unit name | required, but with action `none`, which names none | the systemd unit that reads the paths |
  | `action` | enum `reload`, `restart`, `reload-or-restart`, `start`, `none` | required | what happens to the unit once the change is mounted |
  | `paths` | list of paths (6.2) beneath `/etc`, `/usr/local` or `/opt`, never one of the three itself; at least one, each once | required | what it consumes; one trailing `/`, set aside before 6.2's check, makes a path cover its subtree |

  | Action | For | Done |
  |---|---|---|
  | `reload` | a daemon that reads its configuration again on `ExecReload=`: postfix, nginx, sshd | `systemctl reload` after the swap; a reload that fails, as sshd's on a bad `sshd_config`, fails the live trial |
  | `restart` | a daemon whose binary, libraries or data change, or that reads its configuration only at start | for a changed path under `/usr/local` or `/opt`, stopped before the swap and started after, so the old binary never reads new files; for one under `/etc` only, restarted after |
  | `reload-or-restart` | a unit that may lack a reload | `systemctl reload-or-restart` after the swap |
  | `start` | a unit whose run applies the change: a oneshot, a timer | `systemctl start` after the swap |
  | `none` | inert files, read afresh at every use: a command-line tool, its manual, data a short-lived command reads | nothing |

- **Postfix,** whose configuration a reload applies, in the `mail` task:

  ```json
  {
    "live": {
      "postfix": { "unit": "postfix.service", "action": "reload", "paths": ["/etc/postfix/", "/etc/mailname"] }
    }
  }
  ```

- **An in-house service shipped as software under `/opt`,** whose new
  binary needs a restart, in a `stoa` task:

  ```json
  {
    "software": {
      "stoa": {
        "url": "https://git.ekanayaka.io/ai/stoa/releases/download/v0.4.0/stoa-x86_64-linux.tar.gz",
        "sha256": "…", "unpack": "tar", "strip": 1, "path": "/opt/stoa"
      }
    },
    "files": {
      "/etc/systemd/system/stoa.service": {},
      "/etc/stoa/stoa.json": { "template": "templates/stoa.json.j2" }
    },
    "presets": { "stoa.service": { "action": "enable" } },
    "live": {
      "stoa": { "unit": "stoa.service", "action": "restart", "paths": ["/opt/stoa/", "/etc/stoa/"] }
    }
  }
  ```

  A new release under `/opt/stoa` stops the service before the swap and
  starts it after; a change to `stoa.json` alone restarts it after. Its
  unit file needs no entry (below).
- **A command-line tool,** ripgrep from 10.1, which no unit runs:

  ```json
  { "live": { "ripgrep": { "action": "none", "paths": ["/opt/ripgrep/"] } } }
  ```

- **Several consumers may cover one path,** in one task or several: a
  certificate under `/etc/ssl/certs/` read by postfix and nginx runs
  both reloads. Render takes their union (render.md, 3.7).
- **A deleted path's consumer** is declared by the running slot's spec,
  where the path still exists; every other path's by the new spec.
- **Consumers hearth knows itself,** which no task declares:

  | Path | Consumer |
  |---|---|
  | a unit file or drop-in in `etc/systemd/system/` or `usr/local/lib/systemd/system/` | its unit: `daemon-reload`, then restarted if it runs, started if new and wanted, stopped before the swap if its file goes |
  | a `.wants/` or `.requires/` link | the unit it names: started when the link comes, stopped before the swap when it goes |
  | `etc/passwd`, `etc/group`, `etc/shadow`, `etc/gshadow`, `etc/subuid`, `etc/subgid` | the accounts, entries only added (slots.md, 6.7, rule 5); `nscd --invalidate` if nscd runs |
  | `etc/tmpfiles.d/hearth.conf` | `systemd-tmpfiles --create` on that file (rule 6) |
  | D-Bus policy in `etc/dbus-1/system.d/` | `systemctl reload dbus` |
  | `etc/sysusers.d/hearth.conf`, `etc/systemd/system-preset/00-hearth.preset` | none: their effects, account lines and unit links, are in the change already (build.md, 7.2 and 7.5) |

- **Never live.** A change to any of these takes an ordinary trial,
  whatever a task declares, and render refuses a `live` entry that names
  one (5.11). Boot-time and PID 1 configuration, read only at boot or
  at PID 1's re-exec:

  ```
  etc/systemd/system.conf  etc/systemd/system.conf.d/  usr/local/lib/systemd/system.conf.d/
  etc/systemd/system-generators/  usr/local/lib/systemd/system-generators/
  etc/systemd/system-environment-generators/  usr/local/lib/systemd/system-environment-generators/
  etc/udev/  usr/local/lib/udev/  etc/modprobe.d/  usr/local/lib/modprobe.d/  etc/modules
  etc/modules-load.d/  usr/local/lib/modules-load.d/  etc/sysctl.conf  etc/sysctl.d/  usr/local/lib/sysctl.d/
  etc/binfmt.d/  usr/local/lib/binfmt.d/  etc/kernel/  usr/local/lib/kernel/
  etc/initramfs-tools/  etc/tiny-initramfs/  etc/crypttab  etc/default/grub  etc/fstab
  etc/hostname  etc/localtime  etc/locale.conf  etc/vconsole.conf  etc/machine-id
  etc/os-release
  ```

  Their counterparts under `usr/lib/` are the packages' alone (5.5),
  which never change live (slots.md, 6.7, rule 1).

  and the services that must not restart live:

  | What | Units and paths | Why |
  |---|---|---|
  | systemd's own | every unit the `systemd` packages ship; `etc/systemd/*.conf`, their `.d/` directories and `usr/local/lib/systemd/*.conf.d/` (journald, logind, networkd, resolved, timesyncd and the rest) | PID 1's and its daemons' state does not survive a careless restart |
  | the core services | `dbus.service` and `dbus.socket` (their policy files are allowed, with a reload of dbus); `systemd-logind`, `systemd-journald`, `systemd-udevd`, `systemd-networkd`, `NetworkManager`; `getty@`, `serial-getty@`, `console-getty`; `apparmor.service` and `etc/apparmor.d/` | a restart ends sessions, loses logs, devices or the network; a profile needs `apparmor_parser -r` (slots.md, section 15) |
  | hearth itself, for now | every file of the `hearth` task (5.12) and every path 5.11 reserves | until live changes are proven (slots.md, section 15) |

- **Render refuses what it can tell is never live** (5.11): a `unit`
  among the core services, or named `systemd-*` or `hearth-*`, and a
  path that is one of the never-live paths above, a file of the
  `hearth` task or a path 5.11 reserves, or lies beneath one. A path that covers one, such
  as `/etc/systemd/`, is accepted, and a change to the never-live path
  beneath it still takes a trial. Render reads no package, so the
  builder and the activator find the units the `systemd` packages ship
  under other names (slots.md, 6.7, rule 8).
- **What the declarations cost:** nothing in any key. The `live` part
  enters no layer key and no prepare key (render.md, 4.4), so declaring
  consumers builds nothing and prepares nothing.

## 6. The schema

### 6.1 Form

`settings` in `task.json` is a map from setting name to its spec:

```
spec  = { "type": type, "doc": string,
          "default"?: value, "optional"?: true,
          "min"?: int, "max"?: int, "max_length"?: int }
type  = "bool" | "int" | "string" | "path" | "mode" | "duration" | "secret" | "target" | "file"
      | { "enum": [string, …] }
      | { "list": type }
      | { "map": type }                               keyed by name
      | { "object": { field: spec, … } }              fixed fields, each a spec
```

- **`doc` is required:** one line, at most 200 bytes, for every setting
  and every object field. It is what an agent reads to edit the setting.
- **`min` and `max`** bound an int; **`max_length`** bounds a string's
  bytes, a list's elements or a map's entries. Each is refused on any
  other type: `min` and `max` on all but `int`, `max_length`, 0 or more,
  on all but `string`, `list` and `map`, so on a `path`, a `secret` or an
  `enum` too.
- **An `enum` lists one value at least,** each once.
- A schema is checked when its `task.json` is decoded: a default that does
  not have its type, `default` with `optional`, or `min` above `max` is an
  error in the task, wherever it is used. Whether a `target` in a default
  exists depends on the environment, so it is checked on each target
  whose settings hold it (6.3).

### 6.2 Types

| Type | JSON | Notes |
|---|---|---|
| `bool` | `true`, `false` | |
| `int` | integer | i64 |
| `string` | string | UTF-8, no U+0000 |
| `enum` | string | one of the listed values |
| `list` | array | ordered; replaced whole on merge |
| `map` | object | keyed by name; merged per key; iterated in key order |
| `object` | object | only its declared fields; merged per field |
| `path` | string | absolute; no empty, `.` or `..` segment; no trailing `/`; `/` alone is one |
| `mode` | string | three or four octal digits, at most `"7777"` |
| `duration` | string | a positive integer with no leading zero and one unit, `s`, `m`, `h` or `d`: `5s`, `1m`, `14d`; a number of seconds once decoded, which fits an i64 |
| `secret` | string | a name in the secret store (2.4), never a value |
| `target` | string | a target of the same environment, which must exist |
| `file` | string | a path relative to the repository's root, with no empty, `.` or `..` segment and no leading or trailing `/`, of a file git records at the revision, mode `100644` or `100755`: never a link, a submodule or a directory |

- **No floats and no null.** Nothing in the fleet needs a fraction, and
  null is RFC 7386's deletion.
- **`secret` keeps values out of the repository.** A template that prints
  one prints a name.
- **`file` brings a repository file into a target,** such as the lab's
  certificate authority: `"extra_ca": {"lab": "lab-ca/lab.crt"}` with a
  `copy` entry in `files`. It replaces os's `copy` from a workspace
  outside the repository.

### 6.3 Defaults, optional and required

- **Defaults are the bottom layer** (3.2): the schema's defaults form a
  document that the composition merges first, so a layer can extend a
  default map or remove one of its entries with `null`.
- **Inside maps and lists,** whose entries the schema cannot name in
  advance, each object entry has its absent fields filled from the
  object type's defaults during validation: `"free": {"uid": 1000}` gets
  `/bin/bash` from the users task's `shell` default.
- **`optional`:** the setting may be absent, and reads as undefined
  (7.6). A setting with neither `default` nor `optional` is required.
- **Validation fills what is absent,** after the composition: a setting
  from its `default`, and an object's field, at any depth, from its
  field's. So `null` for a setting that has a default restores the
  default, and `null` for an entry of a default map removes the entry.
  A default that is a whole value, an object or a map or list of them, has
  its fields checked and filled like any value, and a `target` in it is
  checked on each target whose settings hold it.

### 6.4 Reads

```json
{
  "reads": {
    "core": { "settings": ["domain"] },
    "wifi": { "settings": ["ssid"], "optional": true },
    "mail": { "exports": true }
  }
}
```

- **A task reads another task's settings only if it declares them:**
  `tasks.core.domain` in its templates and expressions. The read value is
  the other task's setting with its default applied; its type is the
  other task's.
- **The other task must be on the target,** unless the read is
  `optional`; then `tasks.wifi` is undefined where wifi is absent, and
  `tasks.wifi is defined` tells. That is a settings read: a read of
  `exports` alone needs the task on no target, and `exports.<task>` is
  an empty map where no target of the environment has it (8.2).
- **`exports: true`** reads that task's exports from the environment's
  targets (8.2).
- **References are checked statically:** every `settings.<name>`,
  `tasks.<task>.<setting>` and `exports.<task>` in a task's templates and
  declarations must be declared. `tasks` and `exports` take only literal
  attribute names after them. So the typo `settings.alow` fails at
  render, where os dropped a file.
- **What may stand alone:** `tasks` is read only as
  `tasks.<task>.<setting>`, both names literal, so bare `tasks`,
  `tasks[x]`, `tasks.<task>[x]` and `tasks.<task>` used whole are
  errors, but `tasks.<task> is defined` for an optional read.
  `exports.<task>` may be used whole, the map of 8.2. `settings` may be
  used whole or looked into with `settings[x]`, which is checked when
  evaluated: a name the task does not declare is an error there, and an
  absent optional one is undefined (7.6).

### 6.5 What is an error

Each with its file and line (render.md, section 7):

- a setting the task does not declare, or an undeclared field at any
  depth of an object;
- a value of the wrong type, an enum value not listed, an int out of
  range, a malformed path, mode or secret name, a `target` that does not
  exist in the environment, a `file` absent from the revision;
- a required setting missing after composition;
- a read of a task or setting not declared in `reads`, or of a setting
  the other task does not declare;
- `null` for a task in `fleet.json` or `envs/`, `placement` in a base or
  in `fleet.json`, `members` in `fleet.json` or `envs/` (3.6), a task
  name that `tasks/` lacks.

### 6.6 Example: the users task

`tasks/users/task.json`, whose `users` declaration is 5.2's example:

```json
{
  "doc": "People's accounts: their ids, shells, groups and password hashes.",
  "settings": {
    "accounts": {
      "type": { "map": { "object": {
        "uid": { "type": "int", "min": 1000, "max": 59999, "doc": "The account's id." },
        "shell": { "type": "path", "default": "/bin/bash", "doc": "The login shell." },
        "groups": { "type": { "list": "string" }, "default": [], "doc": "Supplementary groups, such as sudo." },
        "password": { "type": "secret", "optional": true, "doc": "The secret holding the password's hash." }
      } } },
      "default": {},
      "doc": "Accounts, by user name."
    }
  },
  "data": {
    "home-{{ key }}": {
      "for_each": "settings.accounts",
      "path": "/home/{{ key }}", "mode": "0700", "owner": "{{ key }}", "group": "{{ key }}"
    }
  }
}
```

- `hosts/x1/x1.json`'s `"free": {"uid": 1000, "groups": ["sudo"], "password": "users/free.passwd"}`
  validates, and gets `/bin/bash`; its home is the directory
  `home-free` in `@x1-prod.data`, bound at `/home/free`.
- `"free": {"uid": "1000"}` is a wrong type; `"free": {"shel": "/bin/sh"}`
  an undeclared field; `"free": {}` misses the required `uid`. Each error
  names `hosts/x1/x1.json` and the line.

## 7. Templates

### 7.1 Where templates are

- **Files:** `*.j2` under `tree/` and `templates/`, `customize.sh.j2` and
  `software.sh.j2`.
- **Declarations:** in `task.json`, the `if` and `for_each` fields are
  expressions, and keys and string fields are interpolated (5.2).
- **Nowhere else.** Configuration documents are data (2.2).

### 7.2 Delimiters and whitespace

- `{{ … }}` outputs an expression; `{% … %}` is a statement; `{# … #}` is
  a comment.
- **`trim_blocks` and `lstrip_blocks` are on,** for statement and
  comment tags alike, never for `{{ … }}`. The first line break after
  such a tag, one `\n`, `\r\n` or `\r`, is removed; the whitespace
  before one at the start of a line is removed, which is every Unicode
  White_Space character but `\r` and `\n`: a space, a tab, U+000B,
  U+000C and U+00A0 among them. A statement or a comment alone on its
  line leaves nothing behind.
- **Markers:** `-` inside a delimiter (`{%-`, `-%}`, `{{-`, `-}}`, `{#-`,
  `-#}`) strips all Unicode White_Space, newlines included, on that
  side. `+%}` and `+#}` keep the line break after a tag that
  `trim_blocks` would remove; `{%+` and `{#+` keep the indentation that
  `lstrip_blocks` would remove. `{{+` and `+}}` are errors: an output
  tag has nothing for them to keep.
- **Trailing newlines are kept as written.** A file that ends in a
  newline renders ending in one (os stripped one, so `/etc/hostname`
  shipped without it).
- These are MiniJinja 2's semantics with `trim_blocks`, `lstrip_blocks`
  and `keep_trailing_newline` on, and render's tests hold hearth's engine
  to MiniJinja on generated templates (render.md, 9.5).
- **`trim_blocks` eats the newline after a statement at the end of a
  line too,** which os's `grub.cfg` met: write `+%}` there.

  ```jinja
  mynetworks = 127.0.0.0/8 [::1]/128{% for name, peer in exports.mail | items %}{% if peer.role == 'backup' %} {{ peer.address }}/32{% endif %}{% endfor +%}
  ```

### 7.3 Statements

| Statement | Form | Notes |
|---|---|---|
| condition | `{% if e %}` … `{% elif e %}` … `{% else %}` … `{% endif %}` | |
| loop | `{% for x in e %}` … `{% endfor %}` | over a list, or a map's keys in order |
| loop with pairs | `{% for k, v in e \| items %}` | two names only |
| assignment | `{% set x = e %}` | inside a `for`, lasts for one iteration; else to the end of the template |
| raw | `{% raw %}` … `{% endraw %}` | the text between is output as written, but for its tags' whitespace rules (7.2): the line break after `{% raw %}` goes, and the indentation before an `{% endraw %}` that starts its line |

- **Inside a `for`, `loop`** holds `index` (from 1), `index0`, `first`,
  `last` and `length`. It is read by those five keys alone: another key
  is an error, and so is `loop` used whole, but in `loop is defined` and
  as a condition, where it is true. Outside a `for`, `loop` is an
  unknown name (7.6).
- **Names may not shadow the context's roots** (7.7): `{% set settings = … %}`
  is an error. os's `for-each` element shadowed any task of the same name.
  The eight roots, `key`, `value` and `loop` among them, are refused as a
  `set` or `for` name in every template, bound there or not, when the
  template is read.
- **What follows `in`** is an expression without a conditional at its
  top, as in MiniJinja: `{% for x in (a if c else b) %}` loops over a
  conditional, and `{% for x in a if c %}` is the refused `for … if`. A
  pair loop takes any list whose elements are lists of exactly two, as
  `items` gives; another element is an evaluation error at the `for`.
- **Nothing else:** no `macro` or `call`, `include`, `import` or `from`,
  `extends` or `block`, `filter`, `with`, `do`, `break` or `continue`,
  block `set`, `for … else`, `for … if`, `recursive`, `autoescape`.

### 7.4 Expressions

```
expression = or [ "if" or "else" expression ]
or         = and { "or" and }
and        = not { "and" not }
not        = "not" not | test
test       = compare [ "is" [ "not" ] "defined" ]
compare    = concat [ ( "==" | "!=" | "<" | "<=" | ">" | ">=" | "in" | "not" "in" ) concat ]
concat     = sum { "~" sum }
sum        = filtered { ( "+" | "-" ) filtered }
filtered   = postfix { "|" name [ "(" [ expression { "," expression } ] ")" ] }
postfix    = primary { "." name | "[" expression "]" }
primary    = name | string | integer | "true" | "false" | list | "(" expression ")"
list       = "[" [ expression { "," expression } ] "]"
name       = [a-zA-Z_] [a-zA-Z0-9_]*
string     = '…' or "…", with the escapes \\ \' \" \n \t
integer    = [ "-" ] [0-9]+
```

- **Between tokens** stand spaces, tabs and line feeds only: any other
  byte outside a string literal is an error, a `\r` that a declaration's
  JSON string holds included. A string literal keeps the bytes between
  its quotes as written, its escapes read.

- **Grammar words are not names:** `and`, `or`, `not`, `if`, `else`,
  `is`, `in`, `true` and `false`, and `none`, `None`, `True` and
  `False`, which MiniJinja reads as constants, are refused where a name
  stands as an operand, and so as a `set` or `for` name. After `.` each
  is a key like any other: `settings.if`. `defined` is a name but after
  `is`.
- **Lookups:** `a.b` and `a['b']` find a map's key; `a[i]` a list's
  element. There are no methods, so `u.keys` is always the key `keys`
  (the trap os's survey found, where Jinja calls a method).
- **A lookup of the wrong type is an error,** in a chain too (7.6): a
  key into a list, an integer into a map, anything into a string, an
  integer or a boolean. A negative index is outside the list, where
  MiniJinja counts from the end. `in` takes a string to find in a map or
  a string; another value there is an error.
- **Values** are strings, integers, booleans, lists and maps, as the
  schema types them.
- **`and` and `or`** short-circuit and give an operand, as in Jinja.
  **Truthiness:** `false`, `0`, `""`, `[]`, `{}` and undefined are false.
- **`==` and `!=`** compare values of one type deeply; values of two types
  are unequal. **`<`, `<=`, `>`, `>=`** take two integers or two strings,
  compared by bytes. **`in`** finds a key in a map, an element in a list,
  a substring in a string.
- **`+` and `-`** take integers, checked: an overflow is an error. **`~`**
  joins strings and integers, written in decimal.
- **Output:** a string as it is, an integer in decimal, a boolean as
  `true` or `false`. A list or a map is an error: use `join` or `tojson`.
- **No calls:** no functions, no `range`, no methods, no slices, no map
  literals. Data belongs in settings.

### 7.5 Filters and tests

| Filter | Takes | Gives |
|---|---|---|
| `default(v)` | anything, or undefined | `v` where undefined, else the value |
| `join(sep)` | a list of strings or integers | a string |
| `split(sep)` | a string; `sep` not empty | a list of strings |
| `first`, `last` | a list that is not empty | an element |
| `length` | a string, list or map | its characters, elements or entries |
| `replace(a, b)` | a string; `a` not empty | every `a` replaced by `b` |
| `items` | a map | its `[key, value]` pairs, in key order |
| `keys`, `values` | a map | a list, in key order |
| `sort` | a list of strings or of integers | ascending: strings by bytes, as `<` compares them, integers by value; a list of both is an error |
| `tojson` | any value | compact JSON: keys in order; `"` and `\` as `\"` and `\\`; U+0008, U+000C, U+000A, U+000D and U+0009 as `\b`, `\f`, `\n`, `\r` and `\t`; the rest of U+0000 to U+001F as `\u00xx`, in lowercase hex; everything else, `/` and U+007F included, as it is, in UTF-8 |
| `shell_quote` | a string | single-quoted for sh, each `'` written `'\''` |
| `systemd_escape` | a string | as `systemd-escape`: `/` to `-`, every byte outside `[A-Za-z0-9:_.]` and a leading `.` to `\xNN` |
| `systemd_escape_path` | a path (6.2) | as `systemd-escape --path`: slashes collapsed and trimmed, then escaped; `/` alone is `-` |
| `basename`, `dirname` | a path (6.2), or a relative path of a `file`'s form | its last segment; the rest. Of `/`, both are `/`; `/a`'s `dirname` is `/`, and a one-segment relative path's is `.`, as coreutils gives them |

- **A value a filter does not take is an evaluation error:** an empty
  string, `a/` or `/a/../b` to `basename` or `dirname`, a relative path
  to `systemd_escape_path`.
- **The only test is `defined`:** `x is defined`, `x is not defined`.
- `tojson` is for JSON files (the bash fleet escaped its agents'
  `settings.json` with `sed`). `systemd_escape_path` turns
  `/dev/nvme0n1p3` into `dev-nvme0n1p3`, for unit names the bash fleet
  built with `cut -b 2- | tr / -`. `shell_quote` is for settings that go
  into `customize.sh.j2`.

### 7.6 Undefined

- **A lookup that finds nothing is an error,** with the template's file
  and line: an unknown name, a missing key, an index past a list's end.
- **Undefined exists in four places only:** an optional setting or object
  field that is absent; an optional read's task that is absent; an
  optional export field that is absent; `target.member` outside a
  cluster.
- **An absent optional value is no key of its map:** only a lookup of it
  gives undefined. `keys`, `values`, `items`, `length`, `tojson`, `in`,
  `==` and truthiness see the map without it.
- **Undefined may be tested, defaulted or used as a condition:** with
  `is defined`, `default(…)`, or as the condition of `if`, `elif`, `not`,
  `and`, `or` and `… if … else …`, where it is false. Printing it,
  looping over it, comparing it, looking into it or filtering it
  otherwise is an error.
- **Nor does it go further:** undefined as a list's element, as a
  filter's argument but `default`'s, as a `set` value, or as a branch of
  `… if … else …` is an error. `and` and `or` give an operand, which may
  be undefined: `settings.a or settings.b` is undefined where both are.
- **A lookup chain that goes straight into `is defined` or `default`**
  gives undefined at its first missing link instead of an error:
  `peer.endpoint is defined`.
- **A chain is a bound name and the `.name` and `[e]` lookups after
  it,** written straight before `is defined` or `| default`: never in
  parentheses, `(peer.endpoint) is defined`, and never from a literal,
  `[1][5] is defined`. The name is no link: an unknown name is refused
  when the template is read, and a `set` name read where no `set` has
  bound it is an error, before `is defined` too. A lookup that finds
  nothing makes the chain undefined; one of the wrong type (7.4) is
  still an error.
- So os's two rules (an undefined `a.b` printed empty in a file, and was
  false in a condition, while `a.b` on a missing `a` failed in a file)
  become one, and an agent's typo fails.

### 7.7 The context

| Name | Holds | Example |
|---|---|---|
| `target.name`, `.base`, `.environment` | the target | `x1-prod`, `x1`, `prod` |
| `target.member` | the member's name; undefined outside a cluster | `1` |
| `target.placement` | `kind`, `firmware`, `hardware`, `on` (3.4) | |
| `target.containers` | the targets placed on this one: name to placement | `{"chat-prod": {…}}` on `rhi-prod` |
| `debian` | the sources (5.3) | `debian.suite` |
| `settings` | the task's own settings, defaults filled | `settings.port` |
| `tasks.<task>.<setting>` | declared reads (6.4) | `tasks.core.domain` |
| `exports.<task>` | declared export reads: target name to exports (8.2) | `exports.vpn` |
| `key`, `value` | the element of a `for_each` entry, in its fields and its template; unknown elsewhere, in a `tree/` template too | |
| `loop` and assigned names | inside the template | |

- **Nothing else:** no process environment, no clock, no random, no
  other task's files, no secret values, no global functions. The context
  is a function of the revision and the target, which is what lets
  render be one (render.md, section 1).

### 7.8 From esh and from MiniJinja

The subset is what the two surveys found in use, less what schemas and
typed values make unneeded:

| The bash fleet's esh, or os's MiniJinja | hearth |
|---|---|
| `$(config_get Ssh Port)` | `{{ settings.port }}`, or a declared read |
| `if`/`elif`/`else`, `[ a = b ]`, `-n`, `-z` | `if`/`elif`/`else`, `==`, truthiness |
| `for x in $(…)`, `while read` | `for` over a list or a map |
| `cut -f N -d :` on `a:b:c` strings (33 sites) | gone: values are objects |
| `tail -n 1`, last value wins | gone: merging decides |
| `${v:-default}` | a schema default; `default` for an optional setting |
| `config_get --sections \| grep -q Wifi` | `tasks.wifi is defined`, an optional read |
| `s="$s $x"` | `join(' ')` |
| `case … esac` | `if`/`elif`, or a map setting indexed by the value |
| shell functions (5), the open question of macros | gone: defaults, objects and maps remove all five |
| `sed` as a JSON escape | `tojson` |
| `cut -b 2- \| tr / -` on a device path | `systemd_escape_path` |
| `dirname` | `dirname`, `basename` |
| `log_die`, os's `require` | types, and `requires` (5.9) |
| `$TARGET_KIND`, the `container` task | `target.placement.kind` |
| `systemd-detect-virt`, `awk` on another task's file | impossible: no calls, no files |
| os's `for-each` dotted path and `as` | `for_each`, an expression, binding `key` and `value` |
| os's inline `set`, `raw`, `-%}`, `+%}` | kept |
| os's slice `[1:]`, `map(attribute=…)` | gone: `systemd_escape_path`, `keys` |
| os's Lenient and Chainable undefined | one rule (7.6) |
| os's stripped final newline | kept as written |

## 8. Exports

### 8.1 Declaring

- **`exports` is a map from export name to a spec** like a setting's
  (`type`, `doc`, `optional`) and a `value`, written as one
  `{{ expr }}`.
- **The value is checked as a setting's is** (6.5), on each target that
  evaluates it: its type and form, an object's fields declared and its
  required ones present, a `target` that exists in the environment, a
  `file` the revision holds.
- **Computed from settings only:** the task's own settings and `target`.
  Never from another task's reads, never from another target's exports,
  so exports cannot form a cycle; never of type `secret`, at any depth of
  the type (8.4). `debian`, `tasks` and `exports` are refused in a
  value, and a `settings.<name>` the task does not declare is an error,
  as anywhere (6.4).
- **An optional export** whose expression is undefined is absent.
- The exports part holds them (render.md, 4.4); a change to an export
  changes exactly the specs that read it.

### 8.2 Reading

- **Exports are read by task, which is to say by role.** A reader
  declares `"reads": {"vpn": {"exports": true}}`, and its templates see
  `exports.vpn`: a map from target name to that target's vpn exports, for
  every target of the environment that has the task, itself included.
- **Heterogeneous pairs read the same way.** The bash fleet's pairs are
  roles, not identical members: mail's primary and backup, the VPN's hub
  and spokes. Each side exports its role, and the reader picks by it
  (7.2's `mynetworks`).

### 8.3 Scope

- **Exports never cross an environment.** A lab target reads only lab
  targets' exports, and production only production's. The bash fleet's
  recurring error, lab variants carrying production's public keys, and a
  container carrying production's forge host key in the lab, cannot be
  written.
- **Render evaluates the exports of the environment's targets** to render
  any one of them (render.md, 3.4): rendering stays a function of the
  revision and the target.

### 8.4 Public halves as data

- **Secrets are never exported, but their public halves often must be:**
  a WireGuard public key, an SSH host key for `known_hosts`, a Syncthing
  device id. Render never sees a secret, so it cannot derive them.
- **So the public half is recorded in the repository** as a setting of
  the target that owns it, in its base-env file. For identity, the
  activator generates the item and reports its public half as a fact
  (wire.md, section 6); a person or an agent records it, and the change's
  rendered diff shows every peer that picks it up. Until then the
  setting is absent, the optional export with it, and peers skip that
  target.

### 8.5 Fleet-wide constants

- **A value every target shares is a default in `fleet.json`, not an
  export:** the domain, the operator's SSH keys. Exports are for what
  differs by target and must agree between targets.

### 8.6 Example: the VPN

`tasks/vpn/task.json`:

```json
{
  "doc": "A WireGuard network between the targets of an environment, through a hub.",
  "settings": {
    "address": { "type": "string", "doc": "This target's address on the network, such as 10.0.0.2." },
    "hub": { "type": "bool", "default": false, "doc": "Whether this target relays for the others." },
    "endpoint": { "type": "string", "optional": true, "doc": "The public address peers dial, for a hub." },
    "port": { "type": "int", "min": 1, "max": 65535, "default": 51820, "doc": "The UDP port." },
    "public_key": { "type": "string", "optional": true, "doc": "This target's public key, as its activator reported it." }
  },
  "reads": { "vpn": { "exports": true } },
  "identity": {
    "wireguard": {
      "kind": "wireguard-key", "path": "/etc/wireguard/wg0.key",
      "mode": "0640", "owner": "root", "group": "systemd-network"
    }
  },
  "exports": {
    "address": { "type": "string", "value": "{{ settings.address }}", "doc": "Its address on the network." },
    "hub": { "type": "bool", "value": "{{ settings.hub }}", "doc": "Whether it relays." },
    "endpoint": { "type": "string", "optional": true, "value": "{{ settings.endpoint }}", "doc": "Where to dial it." },
    "public_key": { "type": "string", "optional": true, "value": "{{ settings.public_key }}", "doc": "Its public key." }
  }
}
```

`hosts/ion/ion.json` holds `"vpn": {"address": "10.0.0.1", "hub": true}`,
and `hosts/ion/ion-prod.json` what belongs to ion in production,
`"vpn": {"endpoint": "194.164.162.157", "public_key": "m3NY8…="}`, the
key as ion-prod's activator reported it.

`tasks/vpn/tree/etc/systemd/network/wg0.netdev.j2`:

```jinja
[NetDev]
Name=wg0
Kind=wireguard

[WireGuard]
PrivateKeyFile=/etc/wireguard/wg0.key
ListenPort={{ settings.port }}
{% for name, peer in exports.vpn | items %}
{% if name != target.name and peer.public_key is defined and (settings.hub or peer.hub) %}

[WireGuardPeer]
PublicKey={{ peer.public_key }}
AllowedIPs={{ peer.address }}/32
{% if peer.endpoint is defined %}
Endpoint={{ peer.endpoint }}:{{ settings.port }}
PersistentKeepalive=25
{% endif %}
{% endif %}
{% endfor %}
```

- carbon-prod, a spoke, gets ion-prod's key, address and endpoint from
  ion-prod's file; ion-prod gets carbon-prod's key from carbon-prod's.
  Nobody copies a key, and carbon-lab sees only lab peers, with lab keys.
- **`wireguard-key` is [later]** (9.3). Until it exists, the vpn task
  names the private key as a secret file at the same path, an operator
  imports it, and records its public half in the target's file just as
  above. The group `systemd-network` owns the key, so the task declares
  it from its package with a fixed id (5.6).

### 8.7 Example: the backup peer [M6]

- **Backups are encrypted to the peer's key** with `age` before any byte
  reaches a steward (operating.md, section 7). The peer's `backup_peer`
  task holds that key as identity and exports its public half, recorded
  as data like any other (8.4):

  ```json
  {
    "doc": "Receives the environment's backups, encrypted to its own key.",
    "settings": {
      "recipient": { "type": "string", "optional": true, "doc": "This peer's age recipient, as its activator reported it." }
    },
    "identity": { "backup": { "kind": "backup-key" } },
    "exports": {
      "recipient": { "type": "string", "optional": true, "value": "{{ settings.recipient }}", "doc": "Where backups are encrypted to." }
    }
  }
  ```

- **The backed-up target's render reads it.** A class that the `hearth`
  task's `backups` names (5.12) makes render read `backup_peer`'s
  exports in the target's environment and put the peer and its
  recipient into that class's policy in the data part (render.md, 4.3),
  where the activator takes it at prepare. A target with backups and no
  peer that exports a recipient is an error.

## 9. Secrets and identity

### 9.1 Secret files

- **`secrets` is a map keyed by the secret's name in the secret store**
  (2.4), with `path`, `mode` (default `0600`), `owner` and `group`
  (default `root`, names with fixed ids, 5.6, that the activator resolves
  in the slot), `if` and `for_each`:

  ```json
  {
    "secrets": {
      "mail/dkim/{{ key }}.key": {
        "for_each": "settings.domains",
        "path": "/etc/dkimkeys/{{ key }}.key",
        "mode": "0640", "group": "opendkim"
      }
    }
  }
  ```

- **The activator writes each into the slot at prepare,** from the
  root-only secret store (slots.md, 5.4). A missing secret fails the
  prepare, not the render: render never sees the store.
- **Paths:** a secret's path may not be one the file tree, another secret
  or identity gives, nor lie under a data path; a collision is an error
  (render.md, 3.7). A path beneath `/var` is allowed, and is written into
  the slot's own `/var` subvolume (slots.md, 5.4).
- **Never `hearth`'s to read,** the steward's nor its builder's: a
  secret's mode gives others nothing (at most `0770`, no setuid, setgid
  or sticky bit), and its owner is neither user `hearth` nor a group
  `hearth` belongs to, such as `systemd-journal` (5.11).
- **Rotating a secret changes no part.** The activator prepares a new
  slot of the same output (README, section 7).
- **TLS keys are secrets.** Build removes the snakeoil pair that
  ssl-cert's postinst makes (build.md, 5.6), so a task that serves TLS
  declares its private key as a secret file, such as
  `"mail/tls/mx1.key": {"path": "/etc/ssl/private/mx1.ekanayaka.io.key",
  "mode": "0640", "group": "ssl-cert"}`, with `ssl-cert` declared from
  its package with a fixed id (5.6). A certificate a machine obtains
  itself (ACME) lives on persistent data instead.

### 9.2 Password hashes

- **An account's `password` names a secret** that holds its hash, as
  crypt(3) writes it. The activator applies it to the slot's shadow file
  at prepare, the one edit it makes to a packaged file (slots.md, 5.4).
  An account without one stays locked.
- The bash fleet ran `chpasswd --encrypted` in `customize.sh`, so hashes
  ended up in every slot's `/etc/shadow` and in etckeeper's history.

### 9.3 Identity

- **Identity belongs to a target on its machine.** The activator
  generates each item once, when it is missing, keeps it in the secret
  store, writes it into every slot at prepare, and reports its public
  half as a fact (8.4).
- **`identity` is a map keyed by item name,** each with `kind`, its own
  fields, `path`, an optional `public_path`, `mode` (default `0600`),
  `owner` and `group` (fixed ids, 5.6; never `hearth`'s, 5.11):

  | Kind | Fields | Public half | Milestone |
  |---|---|---|---|
  | `ssh-host-key` | `algorithm`: `ed25519`, `ecdsa` or `rsa` | the `.pub` line | M3: made with the running slot's `ssh-keygen`, which the hearth task installs (5.12) |
  | `wireguard-key` | none | the base64 public key | [later]: a target's first slot cannot be prepared with a `wg` only that slot installs, so it waits for keys made in-process, an ask of skein |
  | `backup-key` | none; no `path`: the activator keeps it and writes it nowhere | the `age1…` recipient | [M6]: an X25519 key made with `age-keygen` (activator.md, 5.7; 8.7) |

  ```json
  {
    "identity": {
      "ssh-ed25519": {
        "if": "settings.allow", "kind": "ssh-host-key", "algorithm": "ed25519",
        "path": "/etc/ssh/ssh_host_ed25519_key", "public_path": "/etc/ssh/ssh_host_ed25519_key.pub"
      }
    }
  }
  ```

- **Never `hearth`'s to read,** as a secret (9.1): an item's `mode`, its
  private half's, has no bit outside `0770`, so it gives others nothing
  and has no setuid, setgid or sticky bit.
- **`wireguard-key` and `backup-key` wait for their milestones:** until
  then render refuses each as a kind outside the list. So M1 takes
  `ssh-host-key` alone, which the spec's identity part carries for M3's
  activator.
- **The machine-id is every target's,** never declared. The bash fleet
  fixed it in configuration so the journal had a stable path, and lab,
  production and a container shared one; here each target on each
  machine has its own, kept across its slots (slots.md, 5.4); a
  container's is kept by its host's activator (slots.md, 12.1) [M5].
- **The kinds are a closed list** that grows with the activator's
  operations (activator.md, section 5).

### 9.4 What the spec holds

- **Names and places, never values:** the secrets part holds secret
  names with their paths, modes and owners; accounts' password secret
  names; identity items' names, kinds and paths (render.md, 4.3). No
  byte of a secret or of identity is in a repository, a spec or an
  output.

## 10. Software

### 10.1 Downloads

This section defines how software is placed; build.md (section 6) and
artifacts.md (7.3) cite it.

- **`software` is a map keyed by name;** a download is fetched by its
  SHA-256 into the builder's content-addressed cache and placed in the
  software layer (build.md, section 6), in one of two ways:

  | Field | `file` | `tar` | Meaning |
  |---|---|---|---|
  | `url` | required | required | where to fetch it; outside the part's digest |
  | `sha256` | required | required | 64 lowercase hex digits: what it is |
  | `unpack` | `"file"` | `"tar"` | a single file, or an archive the target's `tar` unpacks, detecting its compression |
  | `path` | required | required | the file's path; the directory the archive unpacks into |
  | `strip` | | default 0; 0 to 255 | leading path segments dropped, as `tar --strip-components` |
  | `mode` | default `0755` | | the file's mode, at most `0777` |
  | `owner`, `group` | default `root` | default `root` | the owners of the file, or of every file and directory the archive gives |

  ```json
  {
    "software": {
      "forgejo": {
        "url": "https://codeberg.org/forgejo/forgejo/releases/download/v15.0.0/forgejo-15.0.0-linux-amd64",
        "sha256": "…", "unpack": "file", "path": "/opt/forgejo/bin/forgejo"
      },
      "ripgrep": {
        "url": "https://github.com/BurntSushi/ripgrep/releases/download/15.1.0/ripgrep-15.1.0-x86_64-unknown-linux-musl.tar.gz",
        "sha256": "…", "unpack": "tar", "strip": 1, "path": "/opt/ripgrep"
      }
    }
  }
  ```

- **Modes:** a `file` takes its `mode`; an archive's files keep the modes
  it records. `path`, and any parent missing, is made `0755`.
- **Owners:** software is placed in the software layer, before any
  sysusers user exists (build.md, 7.1). So an owner or group is `root`,
  or a package user or group with a fixed id (5.6), which build seeds
  before any package installs; render refuses any other. An archive is
  unpacked with `--no-same-owner`, whatever owners it records, and its
  files then take the entry's owners (build.md, 6.2).
- **No setuid, setgid or file capability:** a `file`'s mode has no such
  bit, and an archive whose members carry setuid or setgid fails the
  build. A program that needs privilege comes from a package.
- **Paths are beneath `/opt` or `/usr/local`,** never either itself,
  the bash fleet's and os's places for software (5.5), and outside every
  path of the file tree and every other download's: an overlap is an
  error. hearth's own binary is `/usr/local/bin/hearth` (5.12;
  activator.md, 3.1).
- **A version moves with its digest.** A task pins both in `task.json`; a
  bump edits both, in one change whose rendered diff shows it. os's
  `%VERSION%` in a URL goes.

### 10.2 Artifacts [M6]

- **An artifact is pinned by its key,** with how to make it, and its tree
  is placed at `path` (artifacts.md, 7.1):

  ```json
  {
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
  }
  ```

- **Built from this head,** a lab variant writes `"artifact": {"recipe":
  "stoa"}`: the recipe at the revision being rendered (artifacts.md, 7.2).
  The software part holds the recipe and the digest of its sources'
  listing at the revision, which render computes from its own listing and
  the repository's `artifacts.json`; never a resolved key. The key,
  resolved and remembered as packages are, enters the software layer's
  key instead (render.md, 4.5), so render stays a function of the
  revision.
- **An artifact is a tree, not an archive:** it has no `unpack`, `strip`
  or `mode`; its tree is placed under `path` from the artifact's
  subvolume, with the owners and modes 10.1 allows (artifacts.md, 7.3).
  An entry has either `url` and `sha256`, or `artifact`.

## 11. Checks [M5]

- **A task declares its checks** under `checks`, a map keyed by check
  name: what healthy means for what it installs. Their kinds, fields and
  semantics are operating.md, section 2; render places them in the
  checks part.
- **Checks are declarations like the others** (5.2): `if` and
  `for_each`, keys and string fields interpolated, typed fields written
  as one `{{ expr }}`. So a check reads the task's settings:

  ```json
  { "checks": { "ssh": { "kind": "probe", "probe": "tcp", "host": "127.0.0.1", "port": "{{ settings.port }}", "class": "blessing" } } }
  ```

- **A role is a task.** A probe's `from` names a task, and the targets of
  the environment that have it run the probe (operating.md, 2.6),
  reading it as they read exports (section 8).
- **Render refuses** a `blessing` check with `from`, a `from` that no
  target of the environment has, and a command check whose `user` is
  `root`, or is neither `nobody`, a sysusers user nor a package user some
  task declares (5.6; operating.md, 2.2; render.md, 3.9). The
  postfix task's `queue` check, as `postfix`, needs the postfix task to
  declare `"postfix": {"from": "package"}`.
- **Until M5, render refuses a task that declares checks,** so nothing in
  a repository is silently ignored. A declaration without effect would
  be ignored silently, which the format never does.

## 12. What changes from os

| os | hearth |
|---|---|
| `hosts/<host>/<host>-<env>.json` over a base | fleet, base, member, environment, base in the environment (3.2) |
| containers in `hosts/<host>/machines/`, alone, with no environment | containers are targets; placement names their host (3.5) |
| `--overlay` | gone: every input is in the repository |
| lists concatenate; nothing can be removed | RFC 7386; collections are maps; `null` removes (section 4) |
| a top-level key is a task, case-insensitively, in any file | presence from base and base-env layers only, exact names (3.3) |
| `_`-prefixed keys skipped for tasks, kept in the context | `_note` dropped when decoded (2.2) |
| reserved `suite`, `arch`, `snapshot` | `debian` (5.3) |
| `container`, `desktop`, `7th-gen` steer the build | placement, and `debian.components` (3.4, 5.3) |
| kind, host and env inferred from a hyphen | names made from files, never parsed (3.1) |
| `etc/` flat, with `___` for `/`; `usr-local-share/` | `tree/` mirrors `/` (5.5) |
| every file rendered; final newline stripped | `*.j2` only; newlines kept (2.3, 7.2) |
| `etc.json`: `require`, `tree`, `render`, `templates` | `requires`, `files` with `if` and `for_each` (5.2, 5.5) |
| templates completed at deploy or activation | none: the activator writes fstab from the data part (README, section 7) |
| settings undeclared; undefined is falsy | schemas; undefined only where optional (6, 7.6) |
| modes from the source file; owners root | modes and owners declared (5.5) |
| collisions overwrite in task order | collisions are errors (render.md, 3.7) |
| `customize.sh` concatenated in task order | ordered, each run alone (5.8) |
| `packages` and `secrets` as templates of lines | maps of entries (5.3, 9.1) |
| secrets copied into the output | written into the slot at prepare, with modes and owners (9.1) |
| machine-id from configuration | identity, per target and machine (9.3) |
| software in config as `{name, version, url, sha256}`, cached by name | in the task, by digest, placed as a file or unpacked from tar (10.1) |
| `copy` from a workspace | `file` settings, from the repository (6.2) |
| variant maps (never implemented) | maps of objects |
| hard-coded preseeds, journal and iwd, gettys, snapshot host | task data (5.10) |
| `@T.srv`, `@T.srv.core`, `@T.srv.data`, `@T.srv.cache` made at format time, with `journal` and `iwd` bound by fstab's template; snapper for snapshots | the classes `@<target>.core`, `.data` and `.cache`, made when an entry uses one; each entry a directory bound at its path; snapshots by class, the `hearth` task's (5.7, 5.12) |
| `deploy patch`, rewriting a running root's `/etc` | live changes: a prepared slot's delta mounted over the running one, its consumers declared (5.14; slots.md, 6.6) |

## 13. What is open

- **Identity kinds beyond SSH host keys:** the Syncthing
  certificate and DKIM keys could be identity or imported secrets; it
  depends on activator.md's operations and on whether DNS records follow
  from facts.
- **The bootstrap variant for containers:** os's `custom` with an
  essential list in a task, or mmdebstrap's `essential` variant; settled
  with containers in M5 (slots.md, section 12). It depends too on where
  preseeds load: mmdebstrap runs no essential hook for `custom`, and
  runs it for `essential` (build.md, 5.5).
- **A lab pool's containers** [M5], which its engine places rather than
  a repository: how their placement, parameters included (3.5), reaches
  the pool host; it depends on M5's pool design.
- **The `incus` module list** for a lab VM booting from its own disk, to
  verify on the VM tier (machine.md, section 7; testing.md, section 7).
- **Whether interpolation and `for_each` in declarations are enough,** or
  some declaration needs statements; it depends on converting the
  fleet's 27 tasks, which will show it.
- **Archive formats for software:** whether `zip` is needed beside
  `tar`; it depends on the downloads the fleet's tasks pin.
- **A download over a packaged file:** render cannot see what the
  packages layer holds, so a software path that replaces a packaged
  file is caught, if at all, by the build; it depends on build.md's
  scan.
- **A subvolume per entry,** as `"subvolume": true` on an entry that
  needs its own snapshot policy, quota or restore unit: not in the
  design, a possible later addition if a service needs one; it depends
  on what restoring a single entry and qgroups cost in practice (5.7).
- **Owners in the file tree:** render checks the owners of data,
  secrets, identity and software against declared users and groups with
  fixed ids (5.6, 5.11), but a file's owner in the config layer is still
  resolved only by build, which fails on an unknown name. Whether render
  should check those too depends on how often a wrong owner reaches a
  build.
- **`wireguard-key`** waits for keys made in-process, an ask of skein
  (9.3).
- **Several backup peers in one environment** [M6]: which one a target
  sends to, by a setting or by placement; until it is settled, more than
  one target with `backup_peer` is an error (8.7).
- **hearth's fixed uid, 59999,** to verify against the ids the fleet's
  packages and sysusers files take.
- **A static file whose name ends in `.j2`** must be written as a
  template wrapped in `raw`; whether that is enough depends on what
  tasks ship.
- **Live consumers by inference:** whether a unit's `ExecStart=` path
  or its `ConfigurationDirectory=` could name its consumers without a
  `live` entry (5.14); it depends on how many entries the fleet's tasks
  need once written.
