# Operating

Provisional, 2026-10-09. What hearth does on a machine after a rollout,
for as long as a target runs: the checks that say what healthy means, a
slot blessed by them, the facts it sends up, the telemetry it keeps and
serves, the operational effects it carries out, snapshots and backups
of its data, and the faults a lab's pool host injects. It is the
machine's end of the run side (temper's `factory.md`, section 9); the
rest is the engine's. M5 builds it, local snapshots and restores
included; backups and faults are M6's and say so. What is open is in
section 10.

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
  the journal, in a data entry a task declares, a container's in its
  host's; hearth's own samples of the machine's signals; each
  check's history. It is served when asked (protocol/wire.md, section 7). There
  is no central store to start.
- **Three operational effects, keyed or conditional** (section 6): roll
  back, restart a unit, and restore data. Policy is the engine's. Any
  other waits until it is needed.
- **Snapshots and backups are the activator's, by class** (section 7):
  read-only snapshots of each class of a target's data, `core` and
  `data`, on the period the `hearth` task sets, kept and pruned on the
  machine [M5], and sent incrementally with `btrfs send` to a backup
  peer, encrypted for the peer before any byte reaches the steward
  [M6]. A snapshot is atomic across every service in its class.
  Restoring one entry or a whole class is an effect; drills are
  experiments on the run side's own throwaway machines.
- **Faults are a pool host's effects** [M6] (section 8): a partition,
  latency, a killed process, a full disk, each with an end, granted only
  by the lab's policy and refused by any hearth that is not a lab pool
  host.
- **Nothing else touches a machine:** no shell on the wire, no hand.
  Agents read; their only writes are the infrastructure connector's
  effects (temper's `factory.md`, section 9).

## 2. Checks

This section defines the checks' format and what each result means.
protocol/format.md, section 11 shows where a task declares them.

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
  (protocol/format.md, 5.6). Render refuses any other, and `root`; `nobody`
  needs no declaration (domain/render.md, 3.9; 2.4).

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
| `user` | command | a user a task declares (protocol/format.md, 5.6) | `nobody` | as whom it runs |
| `capabilities` | command | capability names | none | the only capabilities it keeps |
| `network` | command | bool | `false` | whether it may reach the network |
| `probe` | probe | `tcp`, `http`, `smtp`, `dns` | | the protocol spoken |
| `from` | probe | a task, which is to say a role | the target itself | which targets run it (2.6) |
| `host`, `port` | tcp, smtp | a name or an address; a number | | where to connect |
| `url`, `status` | http | an `http` or `https` URL; a status | `200` for status | what to GET, and the status expected |
| `starttls` | smtp | bool | `false` | the `EHLO` answer must offer `STARTTLS` |
| `tls` | tcp, http, smtp | `true`, or `{ "expires_within": duration }` | `false`; `true` for https | verify the certificate (for smtp, after `STARTTLS`, or from the start without it), and fail it if it expires within the duration |
| `server`, `name`, `type`, `expect` | dns | an address; a name; a record type; a value | | ask `server` for `name`'s `type` records; one must equal `expect` |

- **Durations** are the schema's `duration` type (protocol/format.md, 6.2): a
  positive integer and one unit, `s`, `m`, `h` or `d`, such as `5s`,
  `1m`, `14d`.
- **A field may take its value from the task's settings,** as the
  task's other declarations do (domain/render.md, 3.9). A check's rendered
  form, every value resolved, is what enters the `checks` part's
  digest.
- **Render refuses** an unknown field, a field of another kind, a
  `blessing` check with `from` (section 3), a command run as `root` or
  as a `user` no task declares, `nobody` aside (2.1), and a `from` that
  names a task no target of the environment has (2.6; domain/render.md, 3.9).

### 2.3 A unit active

- **Run by the steward,** which needs no privilege for it:
  `systemctl show --property=LoadState,ActiveState,SubState,Result
  <unit>`, parsed by `hearth-protocol-systemd`.
- **`pass`:** `ActiveState=active`. **`fail`:** `failed`, `inactive`,
  or still `activating` at the timeout. **`error`:** `LoadState` is
  `not-found`: the check names a unit the slot does not have, which is
  the check's own fault, not the service's.

### 2.4 A command's exit status, in a sandbox

- **The activator starts it as a transient unit,** with properties it
  fixes. It is one operation of the activator's closed list, *run a
  check* [M5] (domain/activator.md, 5.14): the activator builds the argument
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
  declares (2.1; protocol/format.md, 5.6); `nobody` by default. **Never `root`:**
  render refuses it and the activator refuses it again, so a check
  cannot read what the slot's secrets protect.
- **With which privileges:** none, unless the check lists them from the
  activator's closed list; no new privileges; the whole file system
  read-only (`ProtectSystem=strict` over a root that is read-only
  anyway, and `/var` and the data entries read-only too); the
  state, the activator's mounts and every declared secret and identity
  path inaccessible, whatever their owners; its own `/tmp`; no network
  unless it says so; bounded memory, tasks and time.
- **Only the outcome leaves the activator:** `pass` (exit status 0),
  `fail` (any other status, or the timeout) or `error` (the transient
  unit could not start: the user missing, systemd refusing), with the
  unit's `Result` (`exit-code`, `timeout`, `oom-kill`, `signal`). The
  command's output never reaches the steward or the engine: it stays in
  the journal, under the transient unit's name, for a person or a
  journal read (section 5) to find.
- **To verify against trixie's systemd (257):** the exit status
  `systemd-run --wait` propagates when `RuntimeMaxSec=` stops the
  command, and the summary it prints, which the activator reads for the
  unit's `Result` before `--collect` unloads it.

### 2.5 A probe of the service, as its users see it

- **Run by the steward of the target that runs it** (2.6), through
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
  role is a task, as exports are read by task (protocol/format.md, 8.2):
  `"from": "prober"` names the targets of the environment that have the
  `prober` task. Render exports such a probe with its task's exports,
  rendered, with the target it probes; each target of the environment
  that has the task `from` names reads those probes, as exports are read
  (domain/render.md, 3.4), and renders them into its own `checks` part
  (domain/render.md, 3.9).
- **So the coupling stays narrow and the digests follow.** A change to
  `mail-prod`'s public probe changes `mail-prod`'s and its probers'
  `checks` parts, and nothing else.
- **The prober's hearth reports the result** as its own fact, naming the
  probed target: `mail-prod/postfix.smtp-public` run by `ion-prod`. The
  engine pairs it with the probed machine; neither hearth needs the
  other.
- **Within one environment.** A lab target never probes production, and
  the reverse, as exports never cross an environment (protocol/format.md, 8.3).

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

- **Every result is a fact** (protocol/wire.md, section 6, `check`): the check's
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

No hearth runs in a container in M5 (domain/slots.md, 12.5): the host's hearth
runs its containers' checks, from the host, and reports them as its own
facts, naming the container's target.

- **A unit:** `systemctl --machine=<container> show --property=…
  <unit>`, through the container's systemd.
- **A command:** the same transient unit, started inside the container
  with `systemd-run --machine=<container>` by the host's activator
  (domain/activator.md, 5.14), under the same properties (2.4), with the
  container slot's secret and identity paths inaccessible.
- **A probe** the container's target runs itself: from the host, to the
  container's address, which is the host's own under the fleet's host
  networking.
- **Its failed units** are read the same way, for its trial's minimal
  criterion (domain/slots.md, 12.3) and as its `units` fact (section 4).
- **To verify:** that `--machine=` reaches a container that
  `systemd-nspawn@.service` started, as machined registers it, and
  whether it needs a system bus inside the container
  (domain/slots.md, section 15).

## 3. Blessing with checks

This extends domain/slots.md, section 7.

- **M3's minimal criterion stays the first condition** (domain/slots.md, 7.1):
  the boot reached its default target with no failed unit but those the
  target's `bless.ignore_failed` lists (protocol/format.md), the slot's own
  steward is up and has answered its activator, and a builder it spawned
  has rendered the slot's own revision (domain/steward.md, section 6).
- **Then every blessing check of the slot's target must reach
  `passing`.** While the slot is on trial its blessing checks run back
  to back, a `trial_interval` apart (5 s to start), not at their watching
  interval, so a healthy slot is blessed in seconds.
- **Blessing checks are local:** units, commands, and probes the target
  runs itself. A probe from another target cannot see a trial slot's
  hearth, so render refuses `blessing` with `from` (2.2). Under an
  engine, the engine may hold the bless while such probes bake
  (protocol/wire.md, 5.4).
- **`error` blocks the bless; `skip` does not.** A check that cannot be
  judged cannot prove health. A check that errors everywhere is a bug the
  lab finds first, since it runs the same checks.
- **The trial's window grows with the checks.** The dead-man's window is
  10 minutes in M3 (domain/slots.md, 6.4). From M5 the steward computes it
  before prepare, from the blessing checks its builder's render answered
  (domain/steward.md, 3.3), and passes it as an input of the place request
  (domain/activator.md, 5.1). It is part of the prepare key (domain/slots.md, 2.2) and
  sealed into the slot's trial units, so a slot carries the window it
  was prepared with, and a change to the checks' windows is a new slot:

  ```
  window = M3's 10 minutes, for the boot and the minimal criterion
         + max over blessing checks ( window.pass × trial_interval + timeout )
         + a margin (60 s to start)
  at most the environment's ceiling for a trial (30 minutes to start)
  ```

- **How a trial ends:**
  - every blessing check passing: the slot's steward asks its activator
    to bless, or to hold the bless under an engine that holds it
    (domain/slots.md, 7.4; protocol/wire.md, 5.4);
  - the window passes first: the dead-man reboots into the default, and
    the slot falls back;
  - a check that is failing is waited for until the window ends, since
    services start at their own pace. Only what the minimal criterion
    treats as final ends a trial early: a failed unit that
    `bless.ignore_failed` does not list (domain/slots.md, 7.1). A desktop's
    routinely failing unit belongs on that list, so it never fails a
    trial.
- **A container's trial** follows the same rule, with its own first
  condition: its unit stably active and no failed unit that counts,
  read from its host (domain/slots.md, 12.3). Its window is computed the same
  way and kept by its host's activator, which falls it back at the end,
  since no dead-man runs in a container.
- **A live trial's checks join its settle** (domain/slots.md, 6.8, L7). From
  the swap on, the new slot's blessing checks run back to back at
  `trial_interval`, as on any trial; the live bless waits for the settle
  period and for every one of them to pass (domain/slots.md, 7.6). Its window
  is the base's, whose dead-man ends it, so a change whose new window
  would be longer than the base's is not live (domain/slots.md, 6.7, rule 10).
  A failing check is waited for until a margin before that window ends;
  then the steward asks the live fall back (domain/slots.md, 8.5), which needs
  no reboot, and the dead-man stays the backstop.
- **A live bless is not the last word:** the slot's native boot is a
  trial of its own (domain/slots.md, 6.8, L9), and its blessing checks run
  again there before the default moves.
- **After the bless, checks only report.** Every check runs at its
  interval, blessing and watching alike. A check that starts failing on
  a blessed slot is a fact and an event; hearth rolls nothing back. In
  standalone mode a person reads it in `hearth status`; under an engine,
  the rollout decides.
- **In standalone mode** the same rule applies, with no engine to hold
  a bless: probes from elsewhere are watched and reported only.

## 4. Facts up

What decisions need, small and typed, sent when they change and only
while the channel has room (protocol/wire.md, 5.6). The catalogue is
protocol/wire.md, section 6.

- **Check results** (section 2): each check's state and last result.
- **Resource alarms,** sampled every 10 s, each with a level (`warning`,
  `critical`), the value and the threshold:

  | Alarm | Source | Default thresholds |
  |---|---|---|
  | disk | `df --block-size=1 --output=size,avail` on the store's file system and on each class's mount | free below 10 % or 5 GiB: warning; below 5 % or 1 GiB: critical |
  | memory | `MemAvailable` in `/proc/meminfo` | below 10 %: warning; below 5 %: critical |
  | pressure | `/proc/pressure/{cpu,memory,io}`, `avg60` | `some` above 20 %: warning; `full` above 10 % (memory, io): critical |

  - Thresholds are the `alarms` setting of the hearth task
    (protocol/format.md, 5.12), so a target tunes its own.
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
  trial, leaving out those `bless.ignore_failed` lists (domain/slots.md, 7.1);
  from M5 the whole list is a fact (`units`), ignored units included and
  marked, and each new one an event (`unit failed`). Which of
  systemctl's output forms is stable to parse is to verify.
- **A host's containers:** their failed units are read from the host
  (2.9) and reported under their own targets. They share the host's file
  system, memory and pressure, so the host's alarms cover them, with the
  host's thresholds.
- **Drift:** the activator's (domain/slots.md, section 10), checked at every
  start of the activator and on demand, and from M5 on a period (daily
  to start), reported by its stable ids.
- **Costs:** hearth's own, per build step: time, bytes fetched, cache
  hits; and the steward's memory, idle and at its peak, and the
  builder's (domain/build.md, section 12; domain/steward.md, 5.3). The lab benchmarks
  hearth with them, and a regression in its efficiency is a finding.
- **Liveness is redialling** (protocol/wire.md, 5.5). A machine that stops
  redialling is down; there is no heartbeat fact.

## 5. Telemetry

Logs and metrics stay on the machine, in bounded windows, served to
whoever asks (protocol/wire.md, section 7; `hearth logs`). A triage agent or a
person reads them through the engine.

- **The journal lives in a data entry a task declares,** so it outlives
  slot switches: the `core` task's `journal` entry, the directory
  `/srv/core/journal` of `@x1-prod.core` on x1-prod, bound at
  `/var/log/journal` with group `systemd-journal` (protocol/format.md, 5.7;
  domain/machine.md, 3.3).
  - The task sets journald's `Storage=persistent`, `SystemMaxUse=` and
    `MaxRetentionSec=` from its settings. The bash fleet relied on the
    directory's existence and capped it at 100 MB and a week.
  - journald waits for the mount with
    `RequiresMountsFor=/var/log/journal`, as os's host task does with a
    drop-in.
  - The journal's directory is `/var/log/journal/<machine-id>`. The
    machine-id is identity, written into every slot at prepare
    (domain/slots.md, 5.4), so every slot of the target writes one journal.
  - **A container's journal lives in its host's** [M5]. The host's
    activator writes `LinkJournal=try-host` into the container's nspawn
    settings (domain/slots.md, 12.1), so nspawn makes
    `/var/log/journal/<the container's machine-id>` in the host's
    journal, the `journal` entry's directory in its `core` class, and
    binds it into the container at the same path. The container's `core`
    task declares no `journal` entry (protocol/format.md, 5.7).
  - The container's machine-id is its identity, made once and kept by
    its host's activator (domain/slots.md, 5.4 and 12.1), so every slot of the
    container writes one journal. nspawn refuses to link a journal whose
    id is its host's, which a container's never is.
  - The host's steward reads it as it reads its own, being in
    `systemd-journal`, with `journalctl
    --directory=/var/log/journal/<machine-id>`, whether the container
    runs or not. That journald inside a container with its own id range
    can write the directory nspawn makes, and that the host's
    `systemd-journal` can read what it writes there, is to verify on the
    VM tier (domain/testing.md, 7.3).
- **hearth samples the machine's signals:** CPU (`/proc/stat`), memory
  (`/proc/meminfo`), pressure (`/proc/pressure/*`), load
  (`/proc/loadavg`), and disk use per mount (`df`), every 10 s.
  - They are kept in a ring of fixed size in the steward's records, in
    the store's `state/` (domain/store.md, 8.3; domain/steward.md, 2.2), at three
    resolutions: 10 s for 6 hours, 1 minute for 3 days, 15 minutes for
    60 days. The sizes are computed at startup and bounded.
  - The ring is on disk, so a reboot keeps what led up to it.
- **Each check's history:** its last 1,024 results, a ring per check in
  the steward's records, beside the signals.
- **Served on demand,** by bounded reads of a window
  (protocol/wire.md, section 7). Nothing is pushed. The steward reads the journal
  itself, as `hearth` in the group `systemd-journal` (domain/activator.md, 3.2;
  protocol/format.md, 5.12); its builder, spawned with no supplementary group,
  cannot (domain/steward.md, 2.3).
- **No central store for metrics or logs to start.** If history across
  the fleet is needed later, it is a service the factory deploys from
  its own IaC, behind an observability connector of its own (temper's
  `factory.md`, section 9).

## 6. Operational effects

Work after a rollout is a change through temper, an operational effect,
or one of the run side's standing procedures (convergence, certificate
renewal, snapshots and backups). The operational effects start with three
(protocol/wire.md, 5.4 defines their orders):

| Effect | Class | Condition or key |
|---|---|---|
| roll back | conditional | the running slot and the default the engine decided from (domain/slots.md, section 9; domain/activator.md, 5.5) |
| restart a unit | keyed by the order's key, kept by hearth; conditional at the activator | the unit's `InvocationID` read when the order was taken (domain/activator.md, 5.9); only a unit the running slot's spec ships or enables |
| restore data [M5] | keyed by the order's key; conditional | the class subvolume's UUID the engine decided from; one entry or a whole class, from a snapshot by its sequence (7.4; domain/activator.md, 5.12) |

- **Policy is the engine's.** Who may roll back or restart, when, and
  whether a person must accept it first are authority's (jig's
  `authority.md`). A roll back may be allowed ahead of time, since it is
  cheap and conditional (temper's `factory.md`, section 9).
- **hearth checks only mechanics:** the key, the condition, that the
  unit belongs to the slot. It never refuses on judgement.
- **For a container's target,** a roll back is a trial of its other
  blessed slot through its host (domain/slots.md, 12.3), and a restart names a
  unit inside the container, restarted through `systemctl --machine=`
  (domain/activator.md, 5.9).
- **In standalone mode,** `hearth rollback` and `hearth restore` are
  the client's (client/README.md, 3.1). A person on the machine restarts a unit
  with `systemctl` as root; hearth does not wrap it.
- **Any other effect waits until one is needed,** such as a reload or
  draining a service.

## 7. Snapshots [M5] and backups [M6]

A target's data belongs to its services, not to hearth's unprivileged
side, so snapshotting it and backing it up is the activator's work, and
it may use `btrfs send`, which needs root. Data is kept by class
(protocol/format.md, 5.7): `core`, `data` and `cache`, one subvolume each, with a
policy per class, as os's and the bash fleet's layout intended. The bash
fleet relied on snapper, which missed `/srv/data` and was off on rhi for
lack of disk, and had no off-machine backup of service data at all.

### 7.1 Snapshots [M5]

- **What:** each class the `hearth` task's `snapshots` setting names
  (protocol/format.md, 5.12), whole: a snapshot of `@mail-prod.data` holds
  postfix's spool and the mailboxes as they were at one instant. `cache`
  never.
- **How:** on the class's period, from the running slot's data policies
  (domain/slots.md, 2.2), the activator itself takes a read-only snapshot,
  atomic and crash-consistent, as a power cut would leave the data, with
  `BTRFS_IOC_SNAP_CREATE_V2` and the read-only flag
  (domain/activator.md, 5.11), the equivalent of:

  ```
  btrfs subvolume snapshot -r <top>/@mail-prod.data <top>/@hearth.snapshots/mail-prod.data/<seq>
  ```

  `<top>` is the top level, mounted by the activator at
  `/run/hearth-activator/top` (domain/machine.md, 3.2). `@hearth.snapshots` is
  a top-level subvolume, root's, mode 0700 (domain/machine.md, 3.1). `<seq>`
  only grows, and keys the snapshot: one of that sequence means done.
  The activator's record of the class names each snapshot's sequence,
  time, which policies keep it, and its backup state [M6].
- **No steward is needed:** the activator keeps each class's deadline
  itself, from its records, after its restarts too, so snapshots go on
  while the steward is down or the machine is on trial.
- **On the machine, for a person:** `hearth snapshots` lists them, and
  `hearth shell --snapshot <class>@<seq>` shows one read-only, as any
  slot is shown (client/README.md, 3.1 and 3.7), so a file can be copied out by
  hand.
- **Crash-consistent only.** A service whose files are consistent only
  at rest needs more (a quiescing command before the snapshot); that is
  open.
- **A container's classes** sit on its host's top level
  (domain/slots.md, 12.1), so the host's activator snapshots them by its own
  policies, into its own `@hearth.snapshots`, under the container's
  target.

### 7.2 Sending [M6]

- **What:** each class the `hearth` task's `backups` setting names, on
  its period: the activator takes a snapshot then, unless the local
  policy took one at that moment, and sends it.
- **Incremental, against the parent the peer holds:**

  ```
  btrfs send -p <top>/@hearth.snapshots/mail-prod.data/<parent> <top>/@hearth.snapshots/mail-prod.data/<seq>
  ```

  A full send (no `-p`) only the first time, or when the peer has lost
  the parent. The parent is a local snapshot the backup keeps: the
  newest the peer has confirmed, whatever the local retention says.
- **To a backup peer:** a target of the same environment that has the
  `backup_peer` task, a role as exports read roles (protocol/format.md, 8.2).
  Render gives each target its peer through that task's exports, and
  the peer the targets it receives from, so a peer accepts streams only
  from the targets its own spec names.
- **Encrypted for the peer before any byte reaches the steward.**
  Persistent data holds private material, such as ACME account and
  certificate keys, so no steward or builder ever sees a stream in
  plaintext. The activator pipes `btrfs send` into `age`, encrypting to
  the peer's backup key:

  ```
  btrfs send -p <parent> <seq>  |  age --encrypt --recipient <the peer's age1… key>  →  a pipe to the steward
  ```

  - **The peer's backup key** is an identity item of the peer's target,
    an X25519 key made once with `age-keygen` (domain/activator.md, 5.7), whose
    public half is a fact, recorded in the repository and exported by
    the peer's `backup_peer` task (protocol/format.md, 8.4 and 8.7).
  - **The recipient reaches the activator at prepare,** in the class's
    policy in the sending slot's `data` part, rendered from that export,
    and is kept in the slot record: a send request names a class and a
    sequence, never a recipient, so the steward cannot redirect a stream
    to a key of its own.
- **The path of the ciphertext:** over a pipe made for the transfer
  (not the link) to the steward, which sends it to the peer's steward
  over HTTP (`hearth-protocol-http`); the peer's steward hands it to its
  activator, which runs `age --decrypt --identity <its backup key>`
  into `btrfs receive <top>/@hearth.backups/mail-prod.data/` and
  confirms. Only the two activators see plaintext.
- **To verify:** `age` on trixie, and that it streams a large input in
  bounded memory (its format encrypts in 64 KiB chunks).
- **Facts:** per class, the last snapshot's age, the last sent, the
  peer and the bytes (`backup`, protocol/wire.md, section 6). An alarm when the
  last confirmed send is older than two periods.

### 7.3 Retention

- **Local, by bucket [M5]:** a class's `keep` names how many to keep
  per bucket, `hourly`, `daily`, `weekly` and `monthly`; a snapshot is
  kept while it is the newest of one of the buckets kept. After every
  snapshot the activator prunes the rest, with `SNAP_DESTROY_V2`.
- **The backup keeps what sending needs [M6]:** a snapshot not yet
  confirmed by the peer, and the newest confirmed, the next send's
  parent, are kept whatever the local buckets say; a class with backups
  and no local policy keeps only those.
- **The peer keeps independent subvolumes [M6],** one per received
  snapshot, and deletes by the sender's buckets: the backup's `keep`,
  or the class's local one. Any snapshot can go without breaking
  another, since an incremental stream needs its parent only when it is
  received.
- **At rest the peer holds plaintext subvolumes,** trusted within its
  plane. Keeping the ciphertext streams as files instead, so the peer
  never sees the data, is the alternative in section 10.
- **Snapshots cost space as data changes:** the disk alarm (4) covers
  the class mounts and the top level, and a class whose snapshots fill
  the disk is a policy to change, not something hearth prunes on its
  own.

### 7.4 Restore [M5]

- **An effect, granted by policy** like the others (section 6), keyed
  by the order's key and conditional on the class subvolume being the
  one decided from (its UUID, reported in the `data` fact). It is
  an operation of the activator (domain/activator.md, 5.12). In standalone
  mode, `hearth restore` asks for it (client/README.md, 3.1).
- **From a local snapshot [M5], or from the peer [M6]:** a snapshot the
  machine no longer has is first received from the peer, which sends it
  encrypted to this target's own backup key, as in 7.2, into this
  machine's `@hearth.snapshots`.
- **One entry: its directory copied back out of a snapshot,** then a
  swap of names, then a reboot:

  ```
  1. copy, sharing extents:  <top>/@hearth.snapshots/mail-prod.data/<seq>/mail  →  <top>/@mail-prod.data/mail.restoring
  2. record the swap (the class's UUID, the entry, the step) in the activator's records
  3. rename  mail            →  mail.before.<seq>
     rename  mail.restoring  →  mail                  both within @mail-prod.data
  4. a clean full reboot
  ```

  The copy is domain/slots.md, 5.3's walk through descriptors, with `FICLONE`.
  The reboot is needed because the bind and every service hold the old
  directory until then; fstab names the source by path
  (domain/machine.md, 3.3), so the restored one is bound at the next boot.
- **A class: the swap of subvolumes:**

  ```
  1. make it writable:   snapshot <top>/@hearth.snapshots/mail-prod.data/<seq>  →  <top>/@mail-prod.data.restoring
  2. record the swap (old UUID, new UUID, step) in the activator's records
  3. rename  @mail-prod.data            →  @mail-prod.data.before.<seq>
     rename  @mail-prod.data.restoring  →  @mail-prod.data
  4. a clean full reboot, `systemctl reboot` with SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1
     (domain/slots.md, 6.3): fstab names the subvolume by name (domain/machine.md, 3.3),
     so the restored one is mounted, and every bind of the class with it
  ```

- **Either way, the services keep writing** to the old data until the
  reboot, and those writes are what a restore discards. A power cut
  between the two renames is finished or undone at startup from the
  record. For a container's data, the reboot is a restart of the
  container instead, which binds the entries by path again
  (domain/slots.md, 12.2).
- **The old data is kept** as `mail.before.<seq>` or
  `@mail-prod.data.before.<seq>` until an operator or the engine
  removes it. Both forms are in domain/machine.md, 3.1 and 3.4's layout, so
  drift expects them (domain/slots.md, section 10).
- **One entry from a class snapshot restores that entry only:** the
  other services of the class keep their present data, so a restore
  that must agree across services (postfix's queue and the mailboxes)
  restores the class.

### 7.5 Drills [M6]

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
  (domain/activator.md, 5.15). A pool host of the lab is one whose own target
  is in the lab's environment and has containers placed on it
  (protocol/format.md, 3.5; domain/slots.md, 12.4).
- **The kinds, each applied and removed by an operation of the
  activator [M6] (domain/activator.md, 5.15) with typed arguments, never a
  command line from the engine:**

  | Fault | Mechanism, on the host (to verify) |
  |---|---|
  | partition | nftables rules on the host's bridge dropping traffic between two containers' interfaces |
  | latency, loss | `tc qdisc replace dev <veth> root netem delay <d> loss <p>` on a container's host-side interface |
  | a killed process | `systemctl --machine=<container> kill --signal=SIGKILL <unit>` |
  | a full disk | a qgroup limit on the container's `data` class subvolume (`btrfs qgroup limit`), which needs quotas enabled on the file system |

- **The network faults need a pool's containers on networks of their
  own** (veth pairs and a bridge), where the fleet's containers share
  their host's (`VirtualEthernet=no`); a container's network is a
  placement field, `network: veth` with its `bridge` (protocol/format.md, 3.4).
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
  and commands are reached with `--machine=` (domain/testing.md, 3.12). The
  seed draws them. Its invariants:
  - a slot is blessed only once the minimal bless held and every
    blessing check reached `passing`, within the trial's window;
  - every trial ends: a bless or a fall back;
  - after the bless, no check result changes a slot;
  - every result names its check, slot and output, or its probed target
    and prober;
  - no command check runs as `root`, reaches a path `InaccessiblePaths=`
    hides, or sends a byte of its output to the steward;
  - alarms follow their hysteresis; the signals' and history's rings
    stay within their sizes; every telemetry read is bounded.
- **The backups' world** [M5] runs `hearth-domain-backups` on the fake
  machine, which models class subvolumes, entries' binds, read-only
  snapshots and, from M6, received UUIDs, send streams and a fake peer.
  Two services write into one class while snapshots are taken; the
  world cuts the power mid-snapshot, mid-prune, mid-copy of an entry's
  restore and between a restore's renames, and from M6 mid-send and
  mid-receive, and makes a peer lose a parent. Its invariants:
  - a snapshot holds both services' files as of one instant, and never
    a `cache` class;
  - retention keeps every bucket's newest, and nothing else but what
    sending needs;
  - a restore is finished or undone after any power cut, an entry's
    touching no other entry of its class;
  - no byte of a stream reaches a steward before `age` has encrypted
    it, and every stream's recipient is the one its slot record names
    [M6];
  - the snapshot the peer last confirmed is never deleted locally, and
    a lost parent leads to a full send, never a broken chain [M6].
- **The faults' world:** every fault ends at its end; none is applied
  outside a lab pool host; removing twice is harmless.
- **Conformance and the VM tier** (domain/testing.md, sections 5 and 7): the
  sandbox's properties, the probes against real postfix, nginx and a
  DNS server, `btrfs send` and `receive` between two VMs, and a restore
  that boots.

## 10. What is open

- **What a check may do on a production machine:** whether a check may
  send a real message and see it delivered, or write any state, and
  under which rule; it depends on the run side's policy and on the
  lab's experiments needing the same checks.
- **The quiescing of data before a snapshot:** whether a `data` entry
  may declare a command to run before its class is snapshotted, for
  services that are not crash-safe; it depends on the services hearth
  hosts.
- **Restoring an entry without a reboot:** stopping the units that use
  the entry, swapping its directory and remounting its bind; it depends
  on hearth knowing those units, as `live` entries name them for files
  (protocol/format.md, 5.14).
- **Backups at rest on the peer:** whether the peer keeps the
  ciphertext streams as files, needing no root and never seeing the
  data, at the cost of retention keeping whole chains between periodic
  full sends, rather than receiving plaintext subvolumes (7.3); it
  depends on how far a backup peer is trusted within its plane.
- **The activator sending to the peer itself,** the alternative to
  encrypting for the peer (7.2): no steward would carry the stream at
  all, but the activator would dial out and need an HTTP and TLS client
  of its own; it depends on how small the activator must stay
  (domain/activator.md, section 2).
- **Where backup peers live** and how many each target has; it depends
  on the run side's pools.
- **DNS probes over UDP:** a DNS machine over TCP needs nothing new;
  UDP needs datagram sockets in skein's io (to verify).
- **Certificates' expiry:** reading a peer certificate's `notAfter`
  through skein's TLS client (to verify what it exposes).
- **Thresholds and windows:** the alarms' and the checks' defaults are
  starting values, to tune on real machines.
- **Operational effects beyond the three,** when one is needed.
- **Faults' mechanisms** on the pool host: the exact nftables and `tc`
  recipes, and whether qgroups cost too much to leave enabled; it
  depends on the lab's pool hosts (domain/machine.md).
- **History across the fleet:** when a central store for metrics and
  logs is worth deploying, and which.
- **A container's journal under its own ids [M5]:** whether journald
  in a container with a private id range can write the directory
  `LinkJournal=try-host` makes on the host, and whether the host's
  `systemd-journal` reads its files (section 5); to verify on the VM
  tier (domain/testing.md, 7.3).
