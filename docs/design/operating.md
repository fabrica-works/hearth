# Operating

Provisional, 2026-10-09. What hearth does on a machine after a rollout,
for as long as a target runs: the checks that say what healthy means, a
slot blessed by them, the facts it sends up, the telemetry it keeps and
serves, the operational effects it carries out, backups, and the faults
a lab's pool host injects. It is the machine's end of the run side
(temper's `factory.md`, section 9); the rest is the engine's. M5 builds
it; backups and faults are M6's and say so. What is open is in section
10.

## 1. In one page

- **One definition of healthy, as data.** Each task declares its checks
  (section 2): a unit active, a command's exit status in a sandbox, or a
  probe of the service as its users see it, run by the target itself or
  from another target named by role. The same checks serve the lab's
  experiments, blessing a slot, a rollout's bake and watching the target
  for as long as it runs (temper's `factory.md`, section 3).
- **Checks report facts; decisions are someone else's.** A result is
  `pass`, `fail`, `error` or `skip`, kept as a fact. The one decision
  hearth takes from checks is the trial's own, declared in advance: a
  slot whose blessing checks pass is blessed. Rolling back a blessed
  slot belongs to the rollout, as the bash fleet's validation contract
  already said.
- **Blessing waits for the checks** (section 3). M3's minimal bless
  stands, and from M5 a slot is blessed only once every blessing check
  of its target passes, within a trial window computed from them. A slot
  that cannot pass falls back.
- **Small typed facts go up** (section 4): check results, resource
  alarms (disk, memory, pressure), failed units, drift, costs. Liveness
  costs nothing: a machine that stops redialling is down.
- **Telemetry stays on the machine** (section 5), in bounded windows:
  the journal, on a persistent subvolume a task declares; hearth's own
  samples of the machine's signals; each check's history. It is served
  when asked (wire.md, section 7). There is no central store to start.
- **Two operational effects, keyed or conditional** (section 6): roll
  back, and restart a unit. Policy is the engine's. Any other waits until
  it is needed.
- **Backups are the activator's** [M6] (section 7): read-only snapshots
  of a target's data subvolumes on a period, sent incrementally with
  `btrfs send` to a backup peer, encrypted for the peer before any byte
  reaches the builder; restore is an effect; drills are experiments on
  the run side's own throwaway machines.
- **Faults are a pool host's effects** [M6] (section 8): a partition,
  latency, a killed process, a full disk, each with an end, granted only
  by the lab's policy and refused by any hearth that is not a lab pool
  host.
- **Nothing else touches a machine:** no shell on the wire, no hand.
  Agents read; their only writes are the infrastructure connector's
  effects (temper's `factory.md`, section 9).

## 2. Checks

This section defines the checks' format and what each result means.
format.md, section 11 shows where a task declares them.

### 2.1 A task's checks

A task's checks are a map from a name to a check. A check's id is
`<task>.<name>`, stable for its life, as os's drift ids are: facts,
history and the engine's policy name checks by it.

The postfix task, its checks and the user its command check runs as:

```json
{
  "users": {
    "postfix": { "from": "package" }
  },
  "checks": {
    "instance": {
      "kind": "unit",
      "unit": "postfix@-.service",
      "class": "blessing"
    },
    "smtp-local": {
      "kind": "probe",
      "probe": "smtp",
      "host": "127.0.0.1",
      "port": 25,
      "starttls": true,
      "class": "blessing"
    },
    "queue": {
      "kind": "command",
      "argv": ["/usr/local/lib/checks/postfix-queue-below", "500"],
      "user": "postfix",
      "interval": "5m"
    },
    "smtp-public": {
      "kind": "probe",
      "probe": "smtp",
      "from": "prober",
      "host": "mx1.ekanayaka.io",
      "port": 25,
      "starttls": true,
      "tls": { "expires_within": "14d" },
      "interval": "5m",
      "window": { "pass": 1, "fail": 3 }
    }
  }
}
```

- **A command check's `user` is one a task of the target declares:** a
  sysusers user, or a user a package makes, such as `postfix`, made by
  postfix's maintainer script and declared as the package's
  (format.md, 5.6). Render refuses any other, and `root`; `nobody`
  needs no declaration (render.md, 3.9; 2.4).

### 2.2 Fields

| Field | Kinds | Type | Default | Meaning |
|---|---|---|---|---|
| `kind` | all | `unit`, `command`, `probe` | | what is checked (2.3 to 2.5) |
| `class` | all | `blessing`, `watching` | `watching` | blessing: must pass before a slot is blessed, then watched; watching: watched only |
| `interval` | all | duration | `1m` | between two runs, while watching |
| `timeout` | all | duration | `10s` | the most one run may take; a run past it fails |
| `window` | all | `{ "pass": n, "fail": n }` | `{ 2, 3 }` | how many results in a row change the check's state (2.7) |
| `unit` | unit | a unit's name | | the unit that must be active |
| `argv` | command | strings; the first an absolute path | | what runs |
| `user` | command | a user a task declares (format.md, 5.6) | `nobody` | as whom it runs |
| `capabilities` | command | capability names | none | the only capabilities it keeps |
| `network` | command | bool | `false` | whether it may reach the network |
| `probe` | probe | `tcp`, `http`, `smtp`, `dns` | | the protocol spoken |
| `from` | probe | a task, which is to say a role | the target itself | which targets run it (2.6) |
| `host`, `port` | tcp, smtp | a name or an address; a number | | where to connect |
| `url`, `status` | http | an `http` or `https` URL; a status | `200` for status | what to GET, and the status expected |
| `starttls` | smtp | bool | `false` | the `EHLO` answer must offer `STARTTLS` |
| `tls` | tcp, http, smtp | `true`, or `{ "expires_within": duration }` | `false`; `true` for https | verify the certificate (for smtp, after `STARTTLS`, or from the start without it), and fail it if it expires within the duration |
| `server`, `name`, `type`, `expect` | dns | an address; a name; a record type; a value | | ask `server` for `name`'s `type` records; one must equal `expect` |

- **Durations** are the schema's `duration` type (format.md, 6.2): a
  positive integer and one unit, `s`, `m`, `h` or `d`, such as `5s`,
  `1m`, `14d`.
- **A field may take its value from the task's settings,** as the
  task's other declarations do (render.md, 3.9). A check's rendered
  form, every value resolved, is what enters the `checks` part's
  digest.
- **Render refuses** an unknown field, a field of another kind, a
  `blessing` check with `from` (section 3), a command run as `root` or
  as a `user` no task declares, `nobody` aside (2.1), and a `from` that
  names a task no target of the environment has (2.6; render.md,
  3.9).

### 2.3 A unit active

- **Run by the builder,** which needs no privilege for it:
  `systemctl show --property=LoadState,ActiveState,SubState,Result
  <unit>`, parsed by `hearth-protocol-systemd`.
- **`pass`:** `ActiveState=active`. **`fail`:** `failed`, `inactive`,
  or still `activating` at the timeout. **`error`:** `LoadState` is
  `not-found`: the check names a unit the slot does not have, which is
  the check's own fault, not the service's.

### 2.4 A command's exit status, in a sandbox

- **The activator starts it as a transient unit,** with properties it
  fixes. It is one operation of the activator's closed list, *run a
  check* [M5] (activator.md, 5.14): the activator builds the argument
  vector from the typed check, renders nothing and runs nothing itself;
  systemd runs the command, as the declared user, under these
  properties:

  ```
  systemd-run --unit=hearth-check-<task>-<name> --wait --collect
    --property=User=<user>                         never root
    --property=NoNewPrivileges=yes
    --property=CapabilityBoundingSet=<capabilities, or none>
    --property=AmbientCapabilities=<capabilities, or none>
    --property=ProtectSystem=strict
    --property=ProtectHome=read-only
    --property=PrivateTmp=yes
    --property=PrivateNetwork=yes                  unless network
    --property=InaccessiblePaths=/var/lib/hearth/state /run/hearth-activator
                                 <every secret and identity path the running slot declares>
    --property=RuntimeMaxSec=<timeout>
    --property=MemoryMax=64M  --property=TasksMax=16
    -- <argv>
  ```

- **As which user:** the one the check declares, a user a task
  declares (2.1; format.md, 5.6); `nobody` by default. **Never `root`:**
  render refuses it and the activator refuses it again, so a check
  cannot read what the slot's secrets protect.
- **With which privileges:** none, unless the check lists them from the
  activator's closed list; no new privileges; the whole file system
  read-only (`ProtectSystem=strict` over a root that is read-only
  anyway, and `/var` and the persistent subvolumes read-only too); the
  state, the activator's mounts and every declared secret and identity
  path inaccessible, whatever their owners; its own `/tmp`; no network
  unless it says so; bounded memory, tasks and time.
- **Only the outcome leaves the activator:** `pass` (exit status 0),
  `fail` (any other status, or the timeout) or `error` (the transient
  unit could not start: the user missing, systemd refusing), with the
  unit's `Result` (`exit-code`, `timeout`, `oom-kill`, `signal`). The
  command's output never reaches the builder or the engine: it stays in
  the journal, under the transient unit's name, for a person or a
  journal read (section 5) to find.
- **To verify against trixie's systemd (257):** the exit status
  `systemd-run --wait` propagates when `RuntimeMaxSec=` stops the
  command, and the summary it prints, which the activator reads for the
  unit's `Result` before `--collect` unloads it.

### 2.5 A probe of the service, as its users see it

- **Run by the builder of the target that runs it** (2.6), through
  skein's io: a TCP connection, skein's TLS client where asked, skein's
  HTTP client for `http`, and two small protocol machines of hearth's
  for `smtp` and `dns` (`hearth-protocol-probe`, M5). Probes speak; they
  never write.
- **`tcp`:** connect within the timeout; with `tls`, complete a
  handshake that verifies the certificate against the plane's trust and
  `host`.
- **`http`:** `GET` the URL; `pass` if the status is the one expected.
  No redirect is followed; the body is discarded.
- **`smtp`:** connect (with `tls` and no `starttls`, in TLS from the
  start, as on port 465); read the `220` greeting; send `EHLO` with the
  prober's name; with `starttls`, the `250` lines must offer `STARTTLS`,
  and with `tls` too, send it and verify the handshake; `QUIT`. No
  `MAIL FROM`: no message is sent.
- **`dns`:** ask `server`, over TCP to start (section 10), for `name`'s
  records of `type`; `pass` if one equals `expect`.
- **Names are resolved as users' programs resolve them,** with `getent
  ahosts <host>` as a tool, since skein's io connects to addresses only.
- **`pass`:** the dialogue went as declared. **`fail`:** refused, reset,
  timed out, a wrong status or answer, a certificate that does not
  verify or expires within the bound. **`error`:** the prober could not
  try (resolution failed for a local reason, no route from the prober,
  its own limits full).

### 2.6 Probes from another target

- **A probe with `from` runs on the targets that play that role.** A
  role is a task, as exports are read by task (format.md, 8.2): `"from":
  "prober"` names the targets of the environment that have the `prober`
  task. Render exports such a probe with its task's exports, rendered,
  with the target it probes; each target of the environment that has the
  task `from` names reads those probes, as exports are read (render.md,
  3.4), and renders them into its own `checks` part (render.md, 3.9).
- **So the coupling stays narrow and the digests follow.** A change to
  `mail-prod`'s public probe changes `mail-prod`'s and its probers'
  `checks` parts, and nothing else.
- **The prober's hearth reports the result** as its own fact, naming the
  probed target: `mail-prod/postfix.smtp-public` run by `rhi-prod`. The
  engine pairs it with the probed machine; neither hearth needs the
  other.
- **Within one environment.** A lab target never probes production, and
  the reverse, as exports never cross an environment (format.md, 8.3).

### 2.7 Results and states

| Result | Means | Counts toward |
|---|---|---|
| `pass` | the check ran and found what it declares | passing |
| `fail` | the check ran and found otherwise, timeouts included | failing |
| `error` | the check could not be judged: hearth or the check is at fault, not the service | neither: the state becomes unknown |
| `skip` | the check does not apply here: the placement lacks what it checks, such as a unit masked in a container | neither |

```
state      on                                   next
unknown    window.pass passes in a row          passing
           window.fail fails in a row           failing
passing    window.fail fails in a row           failing
failing    window.pass passes in a row          passing
any        window.fail errors in a row          unknown
any        skip                                 skipped
```

- **Every result is a fact** (wire.md, section 6, `check`): the check's
  id, its state, the last result with a typed cause (`timeout`,
  `refused`, `exit-code`, `oom-kill`, `status 503`, `inactive`,
  `unknown unit`, `sandbox failed`) and, for units and probes, a reason
  of at most 256 bytes for people that hearth writes itself, never a
  command's output (2.4); the
  slot's label and output key it ran against (or, for a probe from
  elsewhere, the probed target and the prober), its age, and how many
  results in a row agree.
- **A state change is an event** (`check changed`). The engine's policy
  matches on states and typed causes; the reason's text is for people.
- **The bash fleet's contract, kept:** `pass|fail|error|skip`; generic
  checks of the platform beside service checks owned by tasks; and
  "validation modules report facts. They do not deploy, mutate
  configuration, or silently repair failures. Rollback decisions belong
  to `rollout`". In hearth the rollout is the engine's.
- **Its exit codes are not kept:** results are typed facts, so nothing
  needs 0, 1 and 2. The client prints them (`hearth status`).

### 2.8 What a check may do on a production machine

Open (section 10). Until it is settled, checks only read: commands run
with a read-only file system and no network unless declared, and probes
stop before anything the service would keep (no message sent, no
request but `GET`). A check that sends a message and sees it delivered,
which the lab's experiments want, needs a rule for production first.

### 2.9 Checks of a container

No hearth runs in a container in M5 (slots.md, 12.5): the host's hearth
runs its containers' checks, from the host, and reports them as its own
facts, naming the container's target.

- **A unit:** `systemctl --machine=<container> show --property=…
  <unit>`, through the container's systemd.
- **A command:** the same transient unit, started inside the container
  with `systemd-run --machine=<container>` by the host's activator
  (activator.md, 5.14), under the same properties (2.4), with the
  container slot's secret and identity paths inaccessible.
- **A probe** the container's target runs itself: from the host, to the
  container's address, which is the host's own under the fleet's host
  networking.
- **Its failed units** are read the same way, for its trial's minimal
  criterion (slots.md, 12.3) and as its `units` fact (section 4).
- **To verify:** that `--machine=` reaches a container that
  `systemd-nspawn@.service` started, as machined registers it, and
  whether it needs a system bus inside the container (slots.md,
  section 15).

## 3. Blessing with checks

This extends slots.md, section 7.

- **M3's minimal criterion stays the first condition** (slots.md, 7.1):
  the boot reached its default target with no failed unit but those the
  target's `bless.ignore_failed` lists (format.md), and the slot's own
  builder is up and has answered its activator.
- **Then every blessing check of the slot's target must reach
  `passing`.** While the slot is on trial its blessing checks run back
  to back, a `trial_interval` apart (5 s to start), not at their watching
  interval, so a healthy slot is blessed in seconds.
- **Blessing checks are local:** units, commands, and probes the target
  runs itself. A probe from another target cannot see a trial slot's
  hearth, so render refuses `blessing` with `from` (2.2). Under an
  engine, the engine may hold the bless while such probes bake (wire.md,
  5.4).
- **`error` blocks the bless; `skip` does not.** A check that cannot be
  judged cannot prove health. A check that errors everywhere is a bug the
  lab finds first, since it runs the same checks.
- **The trial's window grows with the checks.** The dead-man's window
  is 10 minutes in M3 (slots.md, 6.4). From M5 the builder computes it
  from the blessing checks before prepare and passes it as an input of
  the place request (activator.md, 5.1). It is part of the prepare key
  (slots.md, 2.2) and sealed into the slot's trial units, so a slot
  carries the window it was prepared with, and a change to the checks'
  windows is a new slot:

  ```
  window = M3's 10 minutes, for the boot and the minimal criterion
         + max over blessing checks ( window.pass × trial_interval + timeout )
         + a margin (60 s to start)
  at most the environment's ceiling for a trial (30 minutes to start)
  ```

- **How a trial ends:**
  - every blessing check passing: the slot's builder asks its activator
    to bless, or to hold the bless under an engine that holds it
    (slots.md, 7.4; wire.md, 5.4);
  - the window passes first: the dead-man reboots into the default, and
    the slot falls back;
  - a check that is failing is waited for until the window ends, since
    services start at their own pace. Only what the minimal criterion
    treats as final ends a trial early: a failed unit that
    `bless.ignore_failed` does not list (slots.md, 7.1). A desktop's
    routinely failing unit belongs on that list, so it never fails a
    trial.
- **A container's trial** follows the same rule, with its own first
  condition: its unit stably active and no failed unit that counts,
  read from its host (slots.md, 12.3). Its window is computed the same
  way and kept by its host's activator, which falls it back at the end,
  since no dead-man runs in a container.
- **After the bless, checks only report.** Every check runs at its
  interval, blessing and watching alike. A check that starts failing on
  a blessed slot is a fact and an event; hearth rolls nothing back. In
  standalone mode a person reads it in `hearth status`; under an engine,
  the rollout decides.
- **In standalone mode** the same rule applies, with no engine to hold
  a bless: probes from elsewhere are watched and reported only.

## 4. Facts up

What decisions need, small and typed, sent when they change and only
while the channel has room (wire.md, 5.6). The catalogue is wire.md,
section 6.

- **Check results** (section 2): each check's state and last result.
- **Resource alarms,** sampled every 10 s, each with a level (`warning`,
  `critical`), the value and the threshold:

  | Alarm | Source | Default thresholds |
  |---|---|---|
  | disk | `df --block-size=1 --output=size,avail` on the store's file system and on each persistent subvolume's mount | free below 10 % or 5 GiB: warning; below 5 % or 1 GiB: critical |
  | memory | `MemAvailable` in `/proc/meminfo` | below 10 %: warning; below 5 %: critical |
  | pressure | `/proc/pressure/{cpu,memory,io}`, `avg60` | `some` above 20 %: warning; `full` above 10 % (memory, io): critical |

  - Thresholds are the `alarms` setting of the hearth task (format.md,
    5.12), so a target tunes its own.
  - **Hysteresis:** an alarm is raised after 3 samples past its
    threshold and cleared after 6 back within it, so a flapping value
    does not flood the channel.
  - **To verify:** whether Debian's kernel enables pressure stall
    information by default or needs `psi=1` on the command line (the
    `boot` part); and that skein's io reads size-0 procfs files, with a
    conformance case.
- **Failed units:** the running slot's units in the `failed` state,
  read every 30 s with `systemctl list-units --state=failed`, with each
  unit's `Result`. M3's minimal criterion already reads them during a
  trial, leaving out those `bless.ignore_failed` lists (slots.md, 7.1);
  from M5 the whole list is a fact (`units`), ignored units included and
  marked, and each new one an event (`unit failed`). Which of
  systemctl's output forms is stable to parse is to verify.
- **A host's containers:** their failed units are read from the host
  (2.9) and reported under their own targets. They share the host's file
  system, memory and pressure, so the host's alarms cover them, with the
  host's thresholds.
- **Drift:** the activator's (slots.md, section 10), checked at every
  start of the activator and on demand, and from M5 on a period (daily
  to start), reported by its stable ids.
- **Costs:** hearth's own, per build step: time, bytes fetched, cache
  hits (build.md, section 12). The lab benchmarks hearth with them, and a
  regression in its efficiency is a finding.
- **Liveness is redialling** (wire.md, 5.5). A machine that stops
  redialling is down; there is no heartbeat fact.

## 5. Telemetry

Logs and metrics stay on the machine, in bounded windows, served to
whoever asks (wire.md, section 7; `hearth logs`). A triage agent or a
person reads them through the engine.

- **The journal lives on a persistent subvolume a task declares,** so it
  outlives slot switches: the `core` task's `journal` entry, mounted at
  `/var/log/journal` with group `systemd-journal`, which is
  `@x1-prod.journal` on x1-prod (format.md, 5.7; machine.md, 3.1).
  - The task sets journald's `Storage=persistent`, `SystemMaxUse=` and
    `MaxRetentionSec=` from its settings. The bash fleet relied on the
    directory's existence and capped it at 100 MB and a week.
  - journald waits for the mount with
    `RequiresMountsFor=/var/log/journal`, as os's host task does with a
    drop-in.
  - The journal's directory is `/var/log/journal/<machine-id>`. The
    machine-id is identity, written into every slot at prepare (slots.md,
    5.4), so every slot of the target writes one journal.
  - **A container's journal** is the same `core` entry of its own
    target: a persistent subvolume on its host's top level, bound into
    the container alone, under its own machine-id. The host's builder,
    which serves journal reads, cannot reach it beneath the activator's
    mount, so how a container's journal is read is open (section 10).
- **hearth samples the machine's signals:** CPU (`/proc/stat`), memory
  (`/proc/meminfo`), pressure (`/proc/pressure/*`), load
  (`/proc/loadavg`), and disk use per mount (`df`), every 10 s.
  - They are kept in a ring of fixed size in the builder's state
    (store.md, 8.3), at three resolutions: 10 s for 6 hours, 1
    minute for 3 days, 15 minutes for 60 days. The sizes are computed at
    startup and bounded.
  - The ring is on disk, so a reboot keeps what led up to it.
- **Each check's history:** its last 1,024 results, a ring per check in
  the builder's state, beside the signals.
- **Served on demand,** by bounded reads of a window (wire.md, section
  7). Nothing is pushed. The builder reads the journal itself, as
  `hearth` in the group `systemd-journal` (activator.md, 3.2; format.md,
  5.12).
- **No central store for metrics or logs to start.** If history across
  the fleet is needed later, it is a service the factory deploys from
  its own IaC, behind an observability connector of its own (temper's
  `factory.md`, section 9).

## 6. Operational effects

Work after a rollout is a change through temper, an operational effect,
or one of the run side's standing procedures (convergence, certificate
renewal, backups). The operational effects start with two (wire.md, 5.4
defines their orders):

| Effect | Class | Condition or key |
|---|---|---|
| roll back | conditional | the running slot and the default the engine decided from (slots.md, section 9; activator.md, 5.5) |
| restart a unit | keyed by the order's key, kept by hearth; conditional at the activator | the unit's `InvocationID` read when the order was taken (activator.md, 5.9); only a unit the running slot's spec ships or enables |

- **Policy is the engine's.** Who may roll back or restart, when, and
  whether a person must accept it first are authority's (jig's
  `authority.md`). A roll back may be allowed ahead of time, since it is
  cheap and conditional (temper's `factory.md`, section 9).
- **hearth checks only mechanics:** the key, the condition, that the
  unit belongs to the slot. It never refuses on judgement.
- **For a container's target,** a roll back is a trial of its other
  blessed slot through its host (slots.md, 12.3); which unit a restart
  names, one inside the container or the container's own, is open
  (wire.md, section 11).
- **In standalone mode,** `hearth rollback` is the client's (wire.md,
  3.1). A person on the machine restarts a unit with `systemctl` as
  root; hearth does not wrap it.
- **Any other effect waits until one is needed,** such as a reload or
  draining a service.

## 7. Backups [M6]

A target's data belongs to its services, not to the builder, so backing
it up is the activator's work, and it may use `btrfs send`, which needs
root. os gives data a layout and nothing more; the bash fleet has no
off-machine backup of service data at all, and snapper missed
`/srv/data` and was off on rhi for lack of disk.

### 7.1 Snapshots

- **What:** each persistent subvolume whose `data` entry declares a
  `backup`, with its period and retention (format.md, 5.7):

  ```json
  "data": {
    "mail": {
      "path": "/srv/mail",
      "backup": { "every": "1h", "keep": { "hourly": 24, "daily": 14, "weekly": 8 } }
    }
  }
  ```

- **How:** on its period, the activator takes a read-only snapshot,
  atomic and crash-consistent, as a power cut would leave the data, with
  `BTRFS_IOC_SNAP_CREATE_V2` and the read-only flag (activator.md, 5.11),
  the equivalent of:

  ```
  btrfs subvolume snapshot -r <top>/@mail-prod.mail <top>/@hearth.backups/mail-prod.mail/<seq>
  ```

  `<top>` is the top level, mounted by the activator at
  `/run/hearth-activator/top` (machine.md, 3.2). `@hearth.backups` is a
  top-level subvolume, root's, mode 0700, from M6 (machine.md, 3.1).
  `<seq>` only grows, and keys the snapshot: one of that sequence means
  done (activator.md, 5.11). The activator's record names each
  snapshot's sequence, age and whether the peer holds it.
- **Locally, only what sending needs:** the newest snapshot, and the
  newest the peer has confirmed, which the next incremental send needs
  as its parent. The rest of the retention is the peer's.
- **Crash-consistent only.** A service whose files are consistent only
  at rest needs more (a quiescing command before the snapshot); that is
  open.
- **A container's data** sits on its host's top level (slots.md, 12.1),
  so the host's activator snapshots and sends it as its own, under the
  container's target, to the backup peer of the container's environment.

### 7.2 Sending

- **Incremental, against the parent the peer holds:**

  ```
  btrfs send -p <top>/@hearth.backups/mail-prod.mail/<parent> <top>/@hearth.backups/mail-prod.mail/<seq>
  ```

  A full send (no `-p`) only the first time, or when the peer has lost
  the parent.
- **To a backup peer:** a target of the same environment that has the
  `backup_peer` task, a role as exports read roles (format.md, 8.2).
  Render gives each target its peer through that task's exports, and
  the peer the targets it receives from, so a peer accepts streams only
  from the targets its own spec names.
- **Encrypted for the peer before any byte reaches the builder.**
  Persistent data holds private material, such as ACME account and
  certificate keys, so no builder ever sees a stream in plaintext. The
  activator pipes `btrfs send` into `age`, encrypting to the peer's
  backup key:

  ```
  btrfs send -p <parent> <seq>  |  age --encrypt --recipient <the peer's age1… key>  →  a pipe to the builder
  ```

  - **The peer's backup key** is an identity item of the peer's target,
    an X25519 key made once with `age-keygen` (activator.md, 5.7), whose
    public half is a fact, recorded in the repository and exported by
    the peer's `backup_peer` task (format.md, 8.4 and 8.7).
  - **The recipient reaches the activator at prepare,** in the sending
    slot's `data` part, rendered from that export, and is kept in the
    slot record: a send request names a data subvolume and a sequence,
    never a recipient, so the builder cannot redirect a stream to a key
    of its own.
- **The path of the ciphertext:** over a pipe made for the transfer
  (not the link) to the builder, which sends it to the peer's builder
  over HTTP (`hearth-protocol-http`); the peer's builder hands it to its
  activator, which runs `age --decrypt --identity <its backup key>`
  into `btrfs receive <top>/@hearth.backups/mail-prod.mail/` and
  confirms. Only the two activators see plaintext.
- **To verify:** `age` on trixie, and that it streams a large input in
  bounded memory (its format encrypts in 64 KiB chunks).
- **Facts:** per data subvolume, the last snapshot's age, the last sent,
  the peer and the bytes (`backup`, wire.md, section 6). An alarm when
  the last confirmed send is older than two periods.

### 7.3 Retention

- **The peer keeps independent subvolumes,** one per received snapshot,
  and deletes by its buckets (`hourly`, `daily`, `weekly`): any snapshot
  can go without breaking another, since an incremental stream needs its
  parent only when it is received.
- **At rest the peer holds plaintext subvolumes,** trusted within its
  plane. Keeping the ciphertext streams as files instead, so the peer
  never sees the data, is the alternative in section 10.

### 7.4 Restore

- **An effect, granted by policy** like the others, keyed by the order's
  key and conditional on the data subvolume being the one decided from
  (its UUID, reported in the `backup` fact). It is an operation of the
  activator [M6] (activator.md, 5.12).
- **The lean:** a swap of names, then a reboot.

  ```
  1. receive the snapshot from the peer, if the machine lacks it: the peer's activator
     sends it encrypted to this target's own backup key, as in 7.2   age, btrfs receive
  2. make it writable:   snapshot <seq>  →  <top>/@mail-prod.mail.restoring
  3. record the swap (old UUID, new UUID, step) in the activator's records
  4. rename  @mail-prod.mail            →  @mail-prod.mail.before.<seq>
     rename  @mail-prod.mail.restoring  →  @mail-prod.mail
  5. a clean full reboot, `systemctl reboot` with SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1
     (slots.md, 6.3): fstab names the subvolume by name (machine.md, 3.3),
     so the restored one is mounted
  ```

  The services keep writing to the old subvolume until the reboot, and
  those writes are what a restore discards. A power cut between the two
  renames is finished or undone at startup from the record. For a
  container's data, step 5 is a restart of the container instead, which
  binds the subvolume by name again (slots.md, 12.2).
- **The old data is kept** as `@mail-prod.mail.before.<seq>` until an
  operator or the engine removes it. Both names, `.restoring` and
  `.before.<seq>`, are in machine.md, 3.1's layout from M6, so drift's
  `machine.subvolumes` check expects them (slots.md, section 10).

### 7.5 Drills

- **Restore drills are experiments on the run side's own throwaway
  machines.** The run side's engine creates a machine in its own pool,
  prepares the target, restores the latest backup into it with the same
  effect, runs the target's checks, reports a verdict, and tears the
  machine down. Production's data never enters the lab.
- **A drill proves the chain end to end:** snapshot, send, retention,
  restore and the service's own recovery from a crash-consistent copy.

## 8. Faults [M6]

- **Effects of a pool host** (temper's `factory.md`, 5.3): the host owns
  the network between its containers and their processes, so a fault
  needs no cooperation from the system under test.
- **Granted only by the lab's policy,** and refused by the activator
  itself on any machine but a lab pool host, whatever its engine says
  (activator.md, 5.15). A pool host of the lab is one whose own target
  is in the lab's environment and has containers placed on it
  (format.md, 3.5; slots.md, 12.4).
- **The kinds, each applied and removed by an operation of the
  activator [M6] (activator.md, 5.15) with typed arguments, never a
  command line from the engine:**

  | Fault | Mechanism, on the host (to verify) |
  |---|---|
  | partition | nftables rules on the host's bridge dropping traffic between two containers' interfaces |
  | latency, loss | `tc qdisc replace dev <veth> root netem delay <d> loss <p>` on a container's host-side interface |
  | a killed process | `systemctl --machine=<container> kill --signal=SIGKILL <unit>` |
  | a full disk | a qgroup limit on the container's data subvolume (`btrfs qgroup limit`), which needs quotas enabled on the file system |

- **The network faults need a pool's containers on networks of their
  own** (veth pairs and a bridge), where the fleet's containers share
  their host's (`VirtualEthernet=no`); a container's network is one of
  the parameters format.md (section 13) leaves open.
- **Every fault has an end,** a duration in its order, after which the
  host's hearth removes it itself: a fault, like a trial, always ends,
  even if the engine is gone.
- **Keyed by the experiment's key.** Applying is a creation found by its
  key; removing is idempotent. Active faults are facts (`fault`), with
  their ends.

## 9. Its world

- **The checks' world** runs `hearth-domain-checks` with the slots'
  domains on the fake machine. Its fakes: units with states that start
  slowly, fail, restart and flap; fake `systemd-run` with the sandbox's
  outcomes; fake services that probes reach, as step machines speaking
  TCP, HTTP, SMTP and DNS, which answer slowly, refuse, reset, present a
  certificate close to expiry or offer no `STARTTLS`; probers that lose
  their route to the probed target; a host's containers, whose units
  and commands are reached with `--machine=` (testing.md, 3.12). The
  seed draws them. Its invariants:
  - a slot is blessed only once the minimal bless held and every
    blessing check reached `passing`, within the trial's window;
  - every trial ends: a bless or a fall back;
  - after the bless, no check result changes a slot;
  - every result names its check, slot and output, or its probed target
    and prober;
  - no command check runs as `root`, reaches a path `InaccessiblePaths=`
    hides, or sends a byte of its output to the builder;
  - alarms follow their hysteresis; the signals' and history's rings
    stay within their sizes; every telemetry read is bounded.
- **The backups' world** runs `hearth-domain-backups` on the fake
  machine, which models read-only snapshots, received UUIDs and send
  streams, and a fake peer. It cuts the power mid-snapshot, mid-send,
  mid-receive and between a restore's renames, and makes a peer lose a
  parent. Its invariants:
  - no byte of a stream reaches a builder before `age` has encrypted
    it, and every stream's recipient is the one its slot record names;
  - the snapshot the peer last confirmed is never deleted locally;
  - a lost parent leads to a full send, never a broken chain;
  - a restore is finished or undone after any power cut;
  - retention keeps every bucket's newest.
- **The faults' world:** every fault ends at its end; none is applied
  outside a lab pool host; removing twice is harmless.
- **Conformance and the VM tier** (testing.md, sections 5 and 7): the
  sandbox's properties, the probes against real postfix, nginx and a
  DNS server, `btrfs send` and `receive` between two VMs, and a restore
  that boots.

## 10. What is open

- **What a check may do on a production machine:** whether a check may
  send a real message and see it delivered, or write any state, and
  under which rule; it depends on the run side's policy and on the
  lab's experiments needing the same checks.
- **The quiescing of data before a snapshot:** whether a `data` entry
  may declare a command to run first, for services that are not
  crash-safe; it depends on the services hearth hosts.
- **Backups at rest on the peer:** whether the peer keeps the
  ciphertext streams as files, needing no root and never seeing the
  data, at the cost of retention keeping whole chains between periodic
  full sends, rather than receiving plaintext subvolumes (7.3); it
  depends on how far a backup peer is trusted within its plane.
- **The activator sending to the peer itself,** the alternative to
  encrypting for the peer (7.2): no builder would carry the stream at
  all, but the activator would dial out and need an HTTP and TLS client
  of its own; it depends on how small the activator must stay
  (activator.md, section 2).
- **Where backup peers live** and how many each target has; it depends
  on the run side's pools.
- **DNS probes over UDP:** a DNS machine over TCP needs nothing new;
  UDP needs datagram sockets in skein's io (to verify).
- **Certificates' expiry:** reading a peer certificate's `notAfter`
  through skein's TLS client (to verify what it exposes).
- **Thresholds and windows:** the alarms' and the checks' defaults are
  starting values, to tune on real machines.
- **Operational effects beyond the two,** when one is needed.
- **Faults' mechanisms** on the pool host: the exact nftables and `tc`
  recipes, and whether qgroups cost too much to leave enabled; it
  depends on the lab's pool hosts (machine.md).
- **History across the fleet:** when a central store for metrics and
  logs is worth deploying, and which.
- **A container's journal:** on its own persistent subvolume, read
  by the host's activator for its builder, or kept on the host's with
  nspawn's `LinkJournal=`, which the host's builder reads as its own; it
  depends on how `hearth logs` and the wire's journal reads name a
  container (wire.md, sections 3 and 7).
