# The client

Provisional, 2026-10-16, revised 2026-10-19. hearth's command-line client: its crates, its
commands, its output and exit codes, the socket it reaches the steward
by, what it runs in-process and as root, `shell`, and bootstrap's
arguments. M1 builds `render` and `diff`, in-process (3.1 to 3.3, 3.5);
M3 builds the rest of the client, with the steward that serves it, but
for the commands a later milestone marks ([M3b], [M5], [M6]). The
steward's faces, standalone mode and the wire are wire.md's. File names name the documents of `docs/design/`, as
everywhere in hearth's design (README.md, section 6). What is open is in
section 5.

## 1. In one page

- **One binary, any user.** The client is the `hearth` binary's
  command-line face. It talks to the steward over a local Unix socket
  [M3] (3.4), or hosts the builder's domain in its own loop when no
  daemon answers, and for any local repository (3.5). Root-only
  commands run the activator's code in its own process, as root (3.6).
- **It prints typed records and ends with an exit code.** Every answer
  is a record of `hearth-client-protocol-records`, printed as a table
  for people or as JSON lines for scripts (3.2). The exit code says
  which of a few outcomes the command reached (3.3).
- **Every effect is keyed or conditional, even from the client.** A
  prepare is keyed by target and revision. A trial and a roll back carry
  the facts the client decided from, and the activator refuses them if
  those facts have moved (3.1; wire.md, 5.4).
- **It holds no state.** It sends one request, prints the records that
  answer it, and exits; in-process, the store it names is the builder's
  (3.5).
- **Its crates carry one prefix,** `hearth-client-` (section 2), and its
  designs live under `docs/design/client/`: this document, and any the
  client grows.

## 2. Its crates

A crate that exists for the client is named
`hearth-client-{domain,protocol,<kind>}-<name>` (README.md, 4.2):

| Crate | What it is | Milestone | Designed in |
|---|---|---|---|
| `hearth-client-protocol-records` | the client's records and their printer: `render`'s and `diff`'s from M1; the socket's requests and answers, and the other commands' records, from M3; tables and JSON lines | M1, M3 | 3.2, 3.3; render.md, 8.1 |
| `hearth-client-domain-dev` | the development loop's domain root: machines by handle, keyed effects, suggestions | M3 | development.md, 8.2 |
| `hearth-client-protocol-incus` | incus's argument vectors and JSON output | M3b | development.md, 8.3 |
| `hearth-client-fake-incus` | incus's fake, on the fake machine | M3b | testing.md, section 4 |
| `hearth-client-world` | the client's world (section 4) | M3 | section 4 |
| `hearth-client-dev-world` | the development loop's world | M3b | development.md, section 10 |

- **What the client hosts keeps its own name.** The builder's domain
  and protocol layer (`hearth-domain-builder`, `hearth-protocol-builder`
  and their children) run in the client's loop and in the daemon's
  builder alike; `hearth-protocol-git` serves render and the development
  loop; the activator's crates run as root for the root-only commands.
  None of them exists for the client.
- **The binary is `hearth`,** the project's name alone: each process,
  and the client (README.md, 4.2).

## 3. The command line

### 3.1 Commands

| Command | What it does | Served by | Who may | With an engine |
|---|---|---|---|---|
| `hearth render [<target>…] [--at <rev>] [--repo <path>] [--store <path>] [--resolve]` | renders each target named, or every target of the revision, at a revision; prints the spec's parts with their digests (render.md, section 4); with `--resolve`, resolves the keys the store lacks first (render.md, 5.7) | the steward, by a builder's job on the builder's clone (steward.md, 3.3) [M3]; in-process with `--repo`, or the repository around the current directory (3.5) | anyone | yes |
| `hearth diff <from> <to> [--target <t>]… [--repo <path>] [--store <path>] [--resolve]` | the rendered diff: per target and part, what changed (render.md, section 6) | as `render` | anyone | yes |
| `hearth diff --running [--target <t>] [--suggest]` | the running slot against its record: packages, files (with contents on a mutable machine), units, users; `--suggest` drafts task fragments (development.md, section 5) | the steward, from the activator's capture; the drafts in-process | operators | yes |
| `hearth build <target> [--at <rev>]` | builds the output; prints each layer's key, whether it was found or built, and its costs | the steward, by a builder's job; or in-process with a store of one's own | operators; anyone with their own store | yes |
| `hearth status [--check]` | each target's slots, the four facts, the last trial, drift, how a prepared slot would be activated and why not live, the desired revision, the pass and the hold | the steward, which reads the activator's facts afresh | anyone | yes |
| `hearth revision` | shows the desired revision | the steward | anyone | shows the engine's last order |
| `hearth revision set <commit> [--bundle <file>]` | sets the desired revision to a commit; with `--bundle`, imports it first from a git bundle into the builder's clone (wire.md, 4.1; development.md, 6.2) | the steward; the bundle by a builder's fetch job | operators | refused |
| `hearth revision bundle <file> [--repo <path>] [--since <commit>]` | commits a working tree to an object on no branch and writes it as a git bundle; prints the commit (development.md, 6.1) | in-process | anyone | yes |
| `hearth revision follow <branch> [--every <d>] [--trial auto\|manual]` | follows a branch, polled every `d` | the steward | operators | refused |
| `hearth containers [--only <c>… \| --all \| --none]` [M5] | shows, or on a development machine sets, which of its spec's containers the machine runs (wire.md, 4.7) | the steward | anyone to show; operators to set | refused |
| `hearth prepare [--at <rev>]` | prepares the inactive slot; keyed by target and revision | the steward, then the activator | operators | refused |
| `hearth trial [--how auto\|live\|full]` | tries the prepared slot: live when eligible, else by a soft or a one-shot reboot (slots.md, 6.1); conditional on the facts read | the steward, then the activator | operators | refused |
| `hearth rollback` | rolls back to the other slot (slots.md, section 9); conditional on the facts read | the steward, then the activator | operators | refused |
| `hearth hold [--reason <text>]` | holds convergence (wire.md, 4.5) | the steward | operators | refused |
| `hearth release` | releases the hold | the steward | operators | refused |
| `hearth retry <target>` | clears the target's refusals, so the pass may try those slots again (wire.md, 4.4) | the steward | operators | refused |
| `hearth credential <file>` | replaces the builder's read credential for the repository (3.8) | the steward; a builder's job writes it | operators | yes |
| `hearth bootstrap …` | partitions, formats and makes the first slot from a live system (machine.md, section 8); its arguments are 3.8's | in-process, as root | root | yes, before the daemon exists |
| `hearth retarget <target> --revision <commit> [--bundle <file>] [--secrets <dir>] [--containers <c>,… \| --no-containers]` [M3b] | a development machine cloned from a base image takes its target: its first slot built, placed and blessed as bootstrap's are, and from M5 its container selection written (development.md, 3.2 and 3.4) | in-process, as root | root | refused |
| `hearth secrets import <path>…` | imports secrets into the secret store (activator.md, 5.8), then asks the steward to read its facts afresh | in-process, as root | root | refused but at bootstrap |
| `hearth identity` | the public halves of the machine's identity, to record as data (format.md, section 8) | the steward, from the activator's facts | anyone | yes |
| `hearth logs [--unit <u>] [--since <d>] [-n <lines>]` | a bounded window of the journal, read locally (wire.md, section 7); M3 | the steward, in `systemd-journal` | operators | yes |
| `hearth shell [a\|b \| --snapshot <class>@<seq>] [-- <argv>]` | a read-only view of a slot, or of a class's snapshot [M5] (3.7) | in-process, started as root | root | yes, locally |
| `hearth snapshots [--target <t>]` [M5] | each class's snapshots: sequence, age, the buckets that keep it, and from M6 whether the peer holds it (operating.md, 7.1) | the steward, from the activator's facts | anyone | yes |
| `hearth restore <entry> \| --class <class> --from <seq> [--target <t>]` [M5] | restores one entry's directory, or a whole class, from a snapshot, then reboots (operating.md, 7.4); conditional on the facts read | the steward, then the activator | operators | refused |
| `hearth artifact <repository> <recipe> [--at <commit>] [--build]` [M6] | resolves a recipe at a commit to its key and prints the pin; with `--build`, builds it (artifacts.md, 5.7 and 7.1) | the steward, by a builder's job; or in-process | anyone; operators to build on the daemon | yes |
| `hearth dev up <target>… [--count <n>] [--containers <c>,… \| --no-containers] [--at <rev>] [--repo <path>] [--secrets <dir>]` [M3b] | development machines cloned from the base image, each retargeted to its target, a machine's never a container's; from M5 each runs the containers placed on it, all or a selection (development.md, section 3) | in-process, on the development host, driving `incus` | a user who may use Incus | — |
| `hearth dev apply <machine>… [--repo <path>] [--how auto\|live\|full]` [M3b] | the working tree as each machine's desired revision, followed until it converges or falls back (development.md, 6.3) | as `dev up` | as `dev up` | — |
| `hearth dev shell <machine> [-c <container>]`, `hearth dev diff <machine> [-c <container>… \| --all] [--suggest [--out <dir>]]` [M3b] | a root shell in the machine, or from M5 in one of its containers; their capture, the drafts written on the host (development.md, 3.4, 4.4 and section 5) | as `dev up` | as `dev up` | — |
| `hearth dev list`, `hearth dev down <machine>…`, `hearth dev image` [M3b] | the development machines; deleting them; making the base image (development.md, section 3) | as `dev up` | as `dev up` | — |

- **`render` and `diff` in M1** run in-process only (3.5). `render`
  with no target renders every target of the revision, in name order,
  and the targets named in name order too, each once;
  `--at` defaults to `HEAD` of a local repository, and on the daemon's
  clone to the desired revision's commit [M3]. A revision is always a
  commit: what is not committed is not rendered. `diff` takes
  `--target` once per target, every changed target without it.
- **"With an engine"** is the machine's mode, set at bootstrap (3.8)
  and kept in the steward's state (wire.md, 4.1). A machine follows its engine or
  converges on its own, never both. Under an engine the commands that
  change slots or the desired revision exit 69 (3.3): the engine
  decides.
- **The conditional commands decide from what they read.** `trial` and
  `rollback` first read the four facts, then send the order with them
  as its condition: "try `b`, from `a` running and `a` the default".
  If anything moved in between, the activator refuses and the client
  exits 1, printing the facts as they now are.
- **`prepare` is keyed** by target and revision (wire.md, 5.4). Asked twice, it
  finds the slot the first made. `--at` defaults to the desired
  revision.
- **`trial` returns once the activator has taken the trial,** just
  before the machine reboots or soft reboots into the slot. The outcome
  is read after the reboot with `hearth status`: blessed, or fell back.
  The client cannot outlive the reboot it asked for. A live trial
  returns once the units are acted on, before the steward is spawned
  anew; `status` then shows it on live trial, live-blessed, or fell back
  (live). `--how live` asks for live only and exits 1, with the rules
  that fail, when the change is not eligible.
- **`rollback` while live** is the live fall back (slots.md, 8.5): back
  to the files that ran before, with no reboot.
- **`restore` is conditional and reboots,** as `rollback` is: it reads
  the class subvolume's UUID from the facts, asks the activator with it
  as the condition, and returns once the activator has swapped the
  names, just before the reboot (operating.md, 7.4).
- **`hearth dev` runs on a development host, not on a machine:** a
  client that drives `incus` and reaches each development machine's
  hearth through `incus exec` (development.md, section 8). Nothing in
  the daemon serves it.
- **There is no `bless` command.** The hearth in the new slot blesses
  it (slots.md, 7.2); only an engine releases a bless it held (wire.md,
  5.4). A
  person never blesses by hand.
- **A host's containers [M5]** are named with `--target <t>`:
  `prepare`, `trial`, `rollback`, `restore`, `snapshots`, `shell`,
  `secrets import` and `identity` act on the machine's own target by
  default, and on one of its containers with it; `status` lists every
  target the machine runs.
  A container's `trial` returns with its restart's outcome
  (activator.md, 5.16), and its host blesses it later (slots.md, 12.3).
  `logs --target <t>` reads a container's journal from the host's, by
  the container's machine-id (operating.md, section 5).
- **The command set is os's, folded into one tool.** `render` and `diff`
  replace `manifest`; `build` replaces `deploy install`; `status`
  replaces `deploy info`, `ek info` and `provision drift`; `prepare`,
  `trial` and `rollback` replace `ek boot`, which moved the persistent
  default with no trial; `shell` replaces `deploy shell` and `ek shell`,
  and is read-only where those were writable. `deploy patch` has no
  successor: a fix is a new slot.

### 3.2 Output

- **Records, not text.** The steward answers with typed records of
  `hearth-client-protocol-records`, its builder's results among them: a
  spec's parts, a diff's entries, a build's layers, slot rows, identity
  rows, journal entries. The domain never formats. The client's protocol
  layer prints them.
- **Tables for people.** One table per record type, aligned columns,
  keys and digests cut to 12 hex digits, ages as durations:

  ```
  $ hearth status
  TARGET   MODE        DESIRED              PASS               HOLD
  x1-prod  standalone  main 3f1c2ab0 (10m)  waiting for trial  -

  TARGET   SLOT  STATE     RUNNING  DEFAULT  TRIAL  REVISION  OUTPUT        AGE
  x1-prod  a     blessed   yes      yes      -      1a2b3c4d  9e41c0d2a7f1  6d
  x1-prod  b     prepared  -        -        -      3f1c2ab0  c07a11e93b20  4m
  ```

- **JSON lines for scripts.** `--json` prints each record as one JSON
  object per line, with full keys and digests. CI reads `hearth diff
  --json` this way. The fields are the records' own; their names are
  stable within a major version of the client protocol. os's
  tab-separated lines go (`draft/brainstorm.md`, 6.2 item 6).
  - **Version 1 starts with M1's `render` and `diff`.** The first line is
    `{"record":"version","client":1}`; each line after is one record,
    its type in `record` (`spec`, `part`, `change`, `error`, `outcome`, …)
    and its fields under the names render.md gives them (4.2 to 4.4,
    6.1, 7.1), in snake case. Digests and keys are 64 lowercase hex
    digits, modes four octal digits as text, times RFC 3339 in UTC.
  - **Its layout:** a target's `spec` line, then a `part` line per part,
    `meta` among them, an empty one too, in render.md, 4.1's order;
    a part's fields nested under its name (`{"record":"part",
    "target":…,"name":"packages","digest":…,"packages":{…}}`); a
    variant as an object with its `kind`, as the canonical encoding has
    it (store.md, 3.1), a config tree's node as `{"kind":"file",…}`. A
    diff prints its `change` lines, then one `outcome` line, which
    carries `from`, `to`, the counts and `left_out`, the errors past the
    bound (render.md, 7.3): there is no `diff` record.
  - **Within a major version, records and fields are only added.** M3's
    commands add records to version 1 and rename none; a renamed or
    removed field is version 2.
- **Progress goes to standard error,** as records too: a build's steps
  as they start and end, a resolution as it starts and ends, the store's
  wait for its lock and its strays (store.md, 8.2). With `--json`,
  standard error is JSON lines as well, and starts with its own version
  line. Standard output holds only the answer.
- **A command's errors are part of its answer.** `render`'s and `diff`'s
  errors (render.md, 7.3), a target's or a revision's, go to standard
  output, as text or as JSON records, after what did render. What keeps
  the client from answering at all (an unknown option, a repository or a
  store it cannot open) goes to standard error alone: `hearth:
  <message>`, or with `--json` one `startup` record after the version
  line, naming its outcome, `usage` or `try_again`, with 3.3's code. A
  directory that is no git repository opens, and git's refusal is an
  error of the answer, of kind `tool` (render.md, 7.2).

### 3.3 Exit codes

| Code | Meaning | For example |
|---|---|---|
| 0 | done, or the answer asked for | `status`; `prepare` prepared or found; `diff` with no difference |
| 1 | a negative answer | `diff` found differences; a conditional order refused because the facts moved; `status --check` found a fall back, drift, a refusal or a hold |
| 2 | failed | a configuration error in the revision (render.md, 7.1); a build step that failed its key (build.md, 4.3); a prepare the activator refused on verification |
| 64 | usage | an unknown command, target or option; a malformed revision, or one that names no commit |
| 69 | unavailable | no daemon answers and the command does not run in-process; the machine follows an engine; the command's job crashed its builder twice (steward.md, 5.1) |
| 75 | try again | another order holds the target's slots; an operational failure that may pass: git, the resolution's mmdebstrap, a fetch, a timeout, a store full or broken (render.md, 7.1); a repository or a store the client cannot open; a standard output that breaks |
| 77 | not permitted | the peer's credentials do not allow it; a root-only command run by another user |

- **0, 1 and 2 keep os's and the bash fleet's meanings** (os's
  `deploy.md`, exit codes; the bash fleet's validation contract): 1 is
  an answer, 2 is a failure. 2 means the input must change; 75 means the
  same command may succeed later, as render.md, 7.1 separates
  configuration errors from operational ones.
- **64, 69, 75 and 77 are `sysexits.h`'s** `EX_USAGE`, `EX_UNAVAILABLE`,
  `EX_TEMPFAIL` and `EX_NOPERM`, so a script can tell a missing daemon
  from a failed build. os folded them all into 2.
- **The code comes from the answer's terminal record,** whose outcome
  the protocol layer maps to a code by one total function.
- **For `render` and `diff`, the worst outcome wins,** over every target
  of the answer and both sides of a diff: 64 if a target named is not
  the revision's (not while a refused `hosts/` document hides it:
  render.md, 3.1), an error of the answer after the targets that did
  render; else 2 if any error is a configuration error, else 75 if any
  is operational, else, for `diff`, 1 if anything differs, else 0. A diff with a failed target (render.md,
  6.1, `failed`) is then 2 or 75, never 1: what it answers is
  incomplete.

### 3.4 The socket and peer credentials

- **`/run/hearth/client.sock`,** a Unix stream socket bound by the
  steward. The activator makes `/run/hearth/` (owner `hearth`, mode
  0755) before it spawns the steward (activator.md, 3.2). The
  socket's mode is 0666: who may do what is decided by peer
  credentials, never by the file's mode.
- **skein's channel over it,** with the magic `hrtc`, the client as
  initiator and an empty credential (skein's `channel.md`, section 5).
  The client and the steward come from the same release, so the version
  range is one version wide; the opening still refuses a stale client
  cleanly.
- **Peer credentials decide.** The steward reads the peer's uid with
  `SO_PEERCRED` when it accepts the connection (an ask of skein,
  README.md, 8.1) and never trusts a uid a record names.
  - root, and the uids listed as operators in the steward's
    configuration, may run every command the socket serves;
  - any other uid may run `render`, `diff` (but `--running`, which
    reads the slot's files), `status`, `revision` (to read) and
    `identity`;
  - membership of a group as the test of an operator needs the peer's
    supplementary groups (`SO_PEERGROUPS`), which is open (section 5).
- **Bounded.** The steward admits at most `clients` connections (8 to
  start) and one request in flight on each. A connection beyond the
  bound is refused at the entrance, and the client exits 75.

### 3.5 In-process

- **When no daemon answers on the socket,** a command that needs no
  root and no slot runs in the client's own loop: `render`, `diff` and
  `build`. The client hosts `hearth-domain-builder` and its protocol
  layer, and routes the same records to it without bytes.
- **A local repository is always rendered in-process,** daemon or not:
  `render` and `diff` with `--repo`, or the repository around the
  current directory, run in the client's loop, as the user who asked. The daemon renders only its own
  clone of the machine's repository. It cannot read a user's checkout:
  a home is often mode 0700, and git refuses a repository owned by
  another user (its ownership check, `safe.directory`).
- **Its inputs are named on the command line:** `--repo <path>`, the
  only way to name a repository in M1 (positional words are targets),
  and `--store`, a directory of the user's own, by default
  `$XDG_CACHE_HOME/hearth/store` (`~/.cache/hearth/store` when it is
  unset), made 0700 on first use, with any parent it lacks.
  - **The repository** is `<path>/.git` where it exists, a directory or
    a gitfile, else `<path>`, given to git as its `--git-dir` (render.md,
    2.2). Without `--repo`, `<path>` is the nearest of the current
    directory and its parents that holds `.git`.
  - **The store's default:** an empty or relative `XDG_CACHE_HOME` is
    unset, as the XDG Base Directory specification has it, and so is an
    empty or relative `HOME`. With neither, and no `--store`, the store
    cannot be opened (3.2), code 75. For `render` and `diff` it may be on
  any filesystem: it holds the cache and the resolutions (store.md,
  2.4). For `build` it must be a btrfs subvolume [M2] (store.md, 9.1 and
  9.4).
- **Nothing else configures it in M1.** The in-process builder runs on
  built-in defaults: render's limits (render.md, 2.3), the store's
  bounds (store.md, 6.1 and 8.1), no proxy, and its tools at fixed
  paths, `/usr/bin/git` and `/usr/bin/mmdebstrap` (render.md, 2.2 and
  5.2). It reads no configuration file, and of its environment only
  `XDG_CACHE_HOME` and `HOME`, for the store's default. The builder's
  configuration is a value its host makes: the binary makes the
  defaults, and the test tiers and conformance make their own, the
  mirror's proxy among them (testing.md, 6.2). From M2, when it builds
  and so fetches, it takes one option more, `--proxy <url>`: the HTTP
  proxy (an apt-cacher-ng) its fetches go through, in the fetcher's apt
  configuration (build.md, 5.1), never in a sandbox's. A proxy changes
  no output, since every file fetched is checked by digest. Its bounds
  stay the defaults: larger bounds are the daemon's (M3), and a file
  beside the store waits for a need.
- **A second command over the same store waits** for the first to
  release the store's lock (store.md, 8.2, step 1), and says once, on
  standard error, that it waits (3.2). The person may interrupt it.
- **Its `main` is shell code:** it reads its arguments and those two
  variables with the standard library, opens the store as a root at
  startup (made 0700 if it is absent), adopts standard output and
  standard error as streams, and returns 3.3's code as the process's
  exit status, as skein's examples' mains do.
- **It never opens a daemon's store.** `@hearth.store` is owned by
  `hearth`, mode 0700 (machine.md, 3.1), and the client tries the
  socket first: if a daemon answers, the daemon serves.
- **M1's client never tries a socket.** There is no daemon before M3,
  and no Unix socket in skein before it (README.md, 8.1): `render` and
  `diff` always run in-process.
- **Uses:** a developer's machine (M1, M2); CI posting the rendered diff
  of a change (render.md, section 6); the factory's builders when they
  run without an activator (README.md, 4.1).

### 3.6 Root-only commands

- **`bootstrap`, `retarget`, `secrets import` and `shell` run in-process
  as root,** and `shell` drops root at once (3.7). The client checks its
  effective uid first and exits 77 otherwise.
  None of them asks the daemon to act: the activator listens only to its
  steward, and a secret must never pass through the steward or its
  builder.
- **`bootstrap`** runs the activator's code in the client's process,
  which spawns a steward over a link as the daemon's activator would,
  and the steward its builders (machine.md, 8.2). The privilege split
  holds from the first slot.
- **`retarget`** [M3b] is bootstrap's place and first bless on a
  development machine's own top level, with the daemon stopped, refused
  unless the machine accepts mutable targets (development.md, 3.2 and
  4.3).
- **`secrets import`** writes the secret store in `@hearth.state` with
  the activator's own code (activator.md, 5.8): each new value by
  rename, then the index, with the store's generation bumped once for
  the command and conditional on the index it read, so two imports
  never lose each other's writes. It reads files named on the command
  line, or a directory of them, by secret name. It imports no identity,
  which the activator makes (activator.md, 5.7). It prints the
  target's declared secrets with present or missing, never a value.
  Then, if a daemon answers, it asks the steward over the socket to
  read its facts afresh; that request carries nothing but itself.
- **Under an engine,** secrets come from the plane's store (wire.md, 5.3),
  and
  `secrets import` is refused except as part of `bootstrap`.

### 3.7 shell

- **A read-only view of a slot, for a person on the machine.** `hearth
  shell b` mounts slot `b` and shows it; with no slot, the slot not
  running. `hearth shell --snapshot data@412` shows a class's snapshot
  instead [M5], `@hearth.snapshots/<target>.data/412`, so a person can
  read a file as it was and copy it out (operating.md, 7.1); the view
  is the same, read-only, as `nobody`. It replaces os's `ek shell`,
  which gave a writable view of the running slot and was used to change
  slots in place.
- **How:**

  ```
  started as root, in new mount and PID namespaces (mount propagation private):
    mount -o ro,subvol=@<target>.<label>      UUID=<fs uuid>  <dir>
    mount -o ro,subvol=@<target>.<label>.var  UUID=<fs uuid>  <dir>/var
    or, for a snapshot [M5]:
    mount -o ro,subvol=@hearth.snapshots/<target>.<class>/<seq>  UUID=<fs uuid>  <dir>
    mount proc (the new PID namespace's) and sysfs, read-only, under <dir>
    mount a tmpfs at <dir>/dev holding only null, zero, tty and urandom
  then chroot into <dir> and drop root at once:
    setpriv --reuid=nobody --regid=nogroup --clear-groups
            --inh-caps=+dac_read_search --ambient-caps=+dac_read_search
            --bounding-set=-all,+dac_read_search --no-new-privs
            -- /bin/sh, or the argument vector given after --
  ```

- **Why each part:**
  - every mount is read-only, and without `CAP_SYS_ADMIN` nothing can
    be remounted;
  - `/dev` holds no block device, so the disk cannot be opened for
    write;
  - the shell runs as `nobody`, with the one ambient capability
    `CAP_DAC_READ_SEARCH`, so it reads every file and writes none it
    does not own, which is none;
  - its own PID namespace hides the machine's processes, and as
    `nobody` it could not signal the activator anyway.
- **The tools** are util-linux's `unshare`, `mount` and `setpriv`;
  util-linux is essential in Debian, so `setpriv` is in every slot, run
  inside the view after the `chroot`. Skein's contained process trees
  would do all of it at spawn (README.md, 8.1). That ambient
  capabilities survive `setpriv`'s `--no-new-privs` and the exec of the
  shell is to verify.
- **Refused on a slot being prepared:** the activator's state says so,
  and a half-placed tree is no view.
- **Never on the wire, never for agents.** Agents read facts and
  telemetry through the engine (wire.md, section 7). A shell is a person's tool,
  on the machine, started as root.

### 3.8 bootstrap's arguments

machine.md, 8.1 says what bootstrap checks before it starts and holds
the table of the machine's configuration it writes; 8.2 lists its
steps. Its syntax is this:

```
hearth bootstrap --disk <device> [--wipe]
                 --target <target>
                 --repository <url> [--repo-credential <file>]
                 --revision <commit | branch> [--every <d>] [--trial auto|manual]
                 [--secrets <directory>]
                 [--operator <uid>]…
                 [--accept-mutable]
                 [--engine <address> --trust <file> [--credential <file>]]     [M4]
```

- **`--disk`, `--wipe`:** the disk to partition, and leave to wipe one
  that is not blank (machine.md, 8.2, steps 1 to 3).
- **`--target`, `--repository`, `--revision`, `--every`, `--trial`,
  `--operator`:** the machine's configuration (machine.md, 8.1). The
  revision is a commit or a branch, as `hearth revision` takes them; a
  branch is polled every `--every` (10 minutes by default); `--trial`
  is `auto` by default, and `x1-prod` is bootstrapped with `--trial
  manual` (wire.md, 4.1).
- **`--accept-mutable`:** the machine takes targets whose placement is
  `mutable`, with unsealed, read-write roots: a development machine
  (development.md, 4.3). Without it the activator refuses them. It is
  kept in the activator's machine record, and no other command sets it.
- **`--repo-credential`:** the builder's own read credential for the
  repository, if the forge needs one. It is kept in the store's
  `credentials/` and given to git by render (render.md, 2.2); it is
  never a slot's secret. `hearth credential` replaces it later.
- **`--secrets`:** a directory imported into the secret store
  (activator.md, 5.8; machine.md, 8.2, step 5).
- **`--engine`, `--trust`, `--credential` [M4]:** the engine's address,
  what to trust it by, and the machine's credential if the identity
  chosen is one (wire.md, 5.3). Without `--engine` the machine is
  standalone; with it, the revision given builds the first slot only,
  and from the first boot the engine orders.

## 4. Its world

- **`hearth-client-world`** [M3] runs the client's protocol layer
  against the steward's domain on the fake machine, with the builder's
  behind its jobs: every command, from scripted peers with given uids,
  through records without bytes; then through the socket's bytes in a
  protocol world. It checks:
  - each command's records and exit code, against the table of 3.3;
  - peer credentials: a reader never changes state;
  - conditional commands refused when a scripted second client moved the
    facts in between;
  - in-process and over the socket give the same records.
- **M1's client** is checked by `hearth-client-protocol-records`'s
  machine tests, its records printed as tables and as JSON lines; by
  the builder's worlds beneath it (render.md, section 9); and end to
  end, `hearth render` and `hearth diff` run as processes (testing.md,
  2.7).

## 5. What is open

- **Operators by group:** admitting a group of operators needs the
  peer's supplementary groups (`SO_PEERGROUPS`, to verify in skein's
  Unix sockets); until then, operators are uids.
