# Retrospective: render

2026-10-18. The run of hearth's `render` plan on debian-8gb-hel1-1, a
machine with 8 GB of memory and four cores. It was claimed at
2026-10-17T15:43:23Z. The last step landed at 05:24:59 on 10-18, and
the coordinator recorded `done.md` at 05:28:32 and released the run at
05:31:56. The supervisor's sub-agent ran the "Done when" commands from
05:32:36 to 05:41, and item 9 failed. The plan built hearth's M1 (the
render and diff of a fleet's configuration, with its store, client,
protocols, fakes and worlds) and skein's part of it. This
retrospective was written by a fresh session that took no part in the
run (development.md, section 15).

Fifteen read-only sub-agents, on Sonnet, one per source, read:
- the plan, its session files, `later.md`, `findings/` and its git
  history;
- `decisions.md`, rulings 101 to 205;
- `log.md`, in three parts, with the ledger, `done.md`, `context.md`,
  `sessions/` and `reports/`;
- `supervision.md`, `alerts.md`, `handed-over` and `claude.tsv`;
- the git history of hearth, with its trailers (the session's worktree
  guard kept that sub-agent out of skein and smith; this session read
  their histories itself);
- `logs/heavy.tsv`, the task logs and pueue's record;
- the coordinator's Codex rollout;
- the 131 implementer rollouts, in two groups;
- the sixteen supervisor transcripts with their 91 sub-agents, in two
  groups;
- the designer's nine sessions, the planner's sessions on this machine
  and the design's sessions;
- the earlier retrospectives, the metrics history, and skein's process
  from `11af042` to `main`.

This session read:
- development.md's sections 2.6, 3.4, 3.5, 4.3, 5.7, 7.6, 9, 10.5,
  10.6, 13 and 15 at `main`;
- the plan's README (Inputs, Repin, Done when);
- `done.md`, `status.md`, `claude.tsv`, `alerts.md`, `handed-over` and
  `threads`;
- the metrics' code that finds the planner's and the design's sessions
  (`numbers.sh`, 240 to 300), and `plan-check.sh`'s check of the last
  retrospective;
- the kit-batch retrospective's shape;
- skein's and smith's `Plan: render` commits and skein's process commits
  since `11af042`;
- hearth's code at `main`, for the debts (section 9), and the code
  graph's status for hearth.

Not read: four of the nine planner sessions (`4c4f5209`, `c704f410`,
`bdfde70d`, `038afdfe`) and three design sessions (`952e728b`,
`347b517e`, `bda27808`). Their transcripts are not on this machine.

Times are UTC. A time without a date is on the day last named: 10-17
until midnight, then 10-18. Costs are at the list prices of each
message's model, as `plan metrics` prices them. This document proposes;
the user, or the director, decides, and the designer applies what is
adopted.

**Which process.**
- The run's kit was skein `11af042`, the plan's own landing in skein
  (15:29:24), from the claim to the release. `kit/` never moved.
- The process documents did move. Every role read development.md and
  its card from the skein checkout, and that checkout followed `main`
  six times during the run: `b30cc95` at 20:05, `6926c2d` at 21:52, the
  two skein supplements, kit-batch's 29 steps at 02:35, and `9c57a6b`
  at 03:50. Section 5, item 8, counts what that cost.
- 35 commits changed skein's process during the run. The run took two
  by ruling or steer: `b30cc95` (wall-clock's decisions), points 2, 3,
  4 and 6, and `9c57a6b` (kit-batch's decisions), points 1, 6 and 9.
- Section 7 judges what was in force.

## 1. In one page

- **168 of 170 steps in 13.8 hours, and "Done when" item 9 failed.**
  - 151 planned steps and 17 added ones landed; none was dropped.
    jig-repin and temper-repin wait under the README's exception, while
    temper's client-api runs elsewhere.
  - Items 1 to 8 and 10 pass on `main`.
  - Item 9 fails: smith pins skein `841f487` (skein-03.4). smith-repin
    landed it at 17:43:55. Rulings 177 and 181 then added skein-03.s1
    and 03.s2, which landed at 00:28:44 and 01:01:41. Each ruling said
    "smith-repin stands", judging smith's build, never item 9.
  - Nobody checked item 9 until the release had passed. development.md
    5.7 has no case for an upstream step added after its closing repin
    (proposals 1 and 2).
- **The fastest large run on record.**
  - `step_h` 0.08, tied with kit-batch for the lowest of twelve runs.
    The runs of comparable size, jig-protocol, protocol and
    reliability, ran 0.14, 0.15 and 0.33.
  - `tokens_per_step` 14.4M, against protocol's 13.3M and
    jig-protocol's 39.8M.
  - 48 blocks for 168 steps (0.29 a step, against 0.62 and 1.18),
    answered in a median 2.7 minutes.
  - Up to six sessions at once, under the disk's cap, 3.9 on average
    before 20:00; one landing every 4.1 minutes, in median.
- **The plan was written ahead of its code, and the ready checks
  rewrote it.**
  - hearth was empty when the plan was written: 51 sessions and 150
    increments against code that did not yet exist.
  - 48 rulings corrected session files against `main` before they
    started; 27 refreshed line numbers. 30 more were plan defects.
  - The plan's Sizes ran 1.4 to 5 times low: ten Size blocks, each
    answered "land", while the README's rule climbed from twice to four
    times (proposals 5 and 13).
- **One bound took the run's last four hours.**
  - The 1 GiB worst case of the builder was first summed at 28.5, at
    01:14, 9.5 hours into the run: 1,240,426,175 bytes.
  - Nine supplements (28.s1 to 28.s9), four design answers and a change
    of the shipped defaults followed. The last landed at 05:24:59, at
    899,981,218 bytes.
  - From 04:16 the tail ran one session at a time, and none for 28
    minutes while the designer answered (proposal 4).
- **Test shape, steered again and again.**
  - About 19 of the 45 steer points were test shape.
  - "A table row holds its expected value" went out three times, and
    "a setup written once" six. Four became README rules (rulings 137,
    144, 159, 176), and each was broken again (proposal 6).
- **The focused suite stayed past its 8 s aim for 106 minutes.** The
  02:31 hourly check judged it against its 15 s budget. Ten landings
  ran over the aim before the 03:42 steer (proposal 7).
- **The code graph was stale for the whole run.** hearth's index dates
  from 09:41:04, six hours before the claim, with 890 nodes. About half
  of the implementers' graph searches returned nothing (proposal 8).
- **Supervision: $276, 16 sessions and 28 compactions.**
  - Its Opus sessions cost $113, its 91 Sonnet sub-agents $163, of
    which $143 went to checking session files.
  - $1.64 a landed step, against forgejo-17's $3.4.
  - It handed over at every second compaction, about once an hour, and
    the handovers stopped eight sub-agents mid-check.
  - By its own criteria, development.md 13's experiment of compaction
    in place does not hold: weighted tokens per hour rose to 2.7M, and
    the share in the first 40 requests to 63%. The share measures how
    short the windows were, not re-reading (section 7; proposals 10 and
    11).
- **The planning cost is not counted.** The plan's nine
  `Planner-Session:` lines are not trailers to git, and the metrics
  find no design document in its Inputs. $255.50 of planning and
  design is on this machine and uncounted, against $312 counted
  (proposal 9).
- **Thirteen proposals** (section 8):
  - a step added upstream reopens the closing repin;
  - "Done when" runs before the release;
  - every role reads the process from the run's kit;
  - a whole-system bound is summed early;
  - a Size miss is reported, not a block;
  - a test-shape rule steered twice becomes a check in the gate;
  - the pulse watches the test-time aims;
  - the graph's index is checked fresh;
  - every Claude session is counted;
  - the supervisor hands over only when confused;
  - the supervisor's metric counts its restores and its sub-agents;
  - implementers hand over past 100K;
  - session files cite symbols, not line numbers.

## 2. Outcome

"Done when", item by item, as the supervisor's sub-agent ran it from
05:33 against hearth `f21b5c8`, skein `c0662b5` and smith `3ae65b4`
(done.md):

1. **Every step landed but the two waiting repins: passes.** `plan
   status`: "168 of 170 steps landed; … 2 waiting", jig-repin and
   temper-repin, with client-api "active on carbon-forge since
   2026-10-16" in both repositories' `STATUS.md`.
2. **The 21 M1 crate paths exist on `origin/main`: passes.** 21 of 21.
3. **`pub fn position(` and `fn state_digest(` are public in skein:
   passes.**
4. **The end-to-end render and diff pass with at least two tests:
   passes.** 3 tests in 0.081 s.
5. **The conformance profile passes within its 2-minute ceiling:
   passes.** 24 tests in 43.5 s.
6. **One `CLIENT_VERSION: u32 = 1`, in the records crate: passes.**
7. **The suites' medians of three runs, against their aims: passes.**
   - hearth: focused 7.291 s (aim 8), fuzzy 16.004 s (aim 30).
   - skein: focused 7.632 s (aim 7.96), fuzzy 43.777 s (aim 46.84).
   - During the run, hearth's focused suite was past its aim from 02:30
     to 04:16 (section 5, item 5).
   - skein's aims rest on x1's base (6.46 s and 41.84 s), not on this
     machine's (6.27 s and 30.04 s, `findings/measurements.md`). Its
     fuzzy median passes by x1's base, 13.7 s over this machine's, while
     the last landing's gate read 29.5 s.
8. **No budget raised: passes.** Only the added conformance profile's
   `global-timeout` and `period`; nothing in skein.
9. **smith pins skein's last `Plan: render` landing, or a later one:
   fails.**
   - smith pins `841f487`, skein-03.4's. skein's last `Plan: render`
     commit is `0b08745`, skein-03.s2's.
   - The coordinator's `done.md` had called it covered: "rulings
     177/181 retain that landing".
   - The supervisor re-ran both commands at 05:41, alerted at 05:41:28,
     and its push reached no phone ("Mobile push not sent").
   - smith still pins `841f487`. A repin of smith to skein's `main`
     takes both supplements.
10. **Every planned step, and smith-repin, in the `Step:` trailers:
    passes.** skein 15 steps, hearth 155 (with three repin commits of
    their own), smith 1.
11. **These are the only checks.**

The plan's 17 added steps:
- **From the plan (5):** skein-02.s1 (a test file missing from 02.1's
  Files), hearth-09.s1 (a count checked after its scan, against the
  session's own rule), hearth-12.s1 (a decode error no step owned),
  hearth-45.s1 (generic items, which skein's programming-model.md
  forbids, in the plan's own sketches), skein-03.s1 (a world API the
  plan assumed).
- **From neither (3):** skein-04.s1 and skein-03.s2, faults in skein's
  fakes; hearth-45.s2, code that missed a Proof the plan had right.
- **From the 1 GiB bound (9):** hearth-28.s1 to 28.s9 (section 5, item
  2).

## 3. Numbers

`plan metrics` ran without error, from the kit at `main`, and wrote
`metrics.tsv`. The comparisons:
- forgejo-17, the last retrospective's run: temper, 11 steps, 2.4 h;
- protocol: temper, 115 steps, 18.5 h, the nearest in size;
- jig-protocol: jig, 217 steps, 29.8 h, the largest;
- kit-batch: skein, 29 steps, the only earlier run with the
  `supervisor` metric.

| Metric | render | Notes | forgejo-17 | protocol | jig-protocol | kit-batch |
|---|---|---|---|---|---|---|
| `wall_h`, `running_h` | 13.8, 13.4 | The claim to done.md's `Recorded` line. | 2.4, 2.0 | 18.5, 16.9 | 29.8, 29.5 | 3.8, 2.2 |
| `steps_planned`, `added`, `dropped` | 151, 17, 0 | 150 increments and smith-repin. | 9, 2, 0 | 112, 3, 22 | 166, 51, 0 | 25, 4, 0 |
| `step_h` | 0.08 | Tied for the lowest. 74 landings in the first 2 h 51 min; 4 to 8 an hour after midnight. | 0.18 | 0.15 | 0.14 | 0.08 |
| `escalations` | 48: plan defect 34, process 8, design gap 5, cross-session 1 | 0.29 a step. Ready checks fixed most plan defects before a session started, as rulings, which this metric does not count. | 3 | 71 | 256 | 3 |
| `escalation_wait_min` | 2.7 | 19 of the last 21 blocks were answered by adopting the implementer's proposal. | 10.5 | (−153) | 1.6 | 1.2 |
| `blocked_h` | 2.6 | | 0.6 | (−277.7) | 8.2 | 0.1 |
| `answers_reversed` | 2 | Ruling 141 by 142, 189's point 1 by 193 (section 6). | 0 | 5 | 38 | 0 |
| `steers` | 27 sent, 1 point refused | 45 points. The refused point repaired a step that had already landed. | 1, 0 | 5, 0 | 24, 0 | 4, 0 |
| `restarts` | none | | | the 1 | | loaded 1 |
| `tokens_coordinator` | 401.3M in (99% cached), 0.70M out | 2.39M a step. | 16.6M; 1.51M | 271.0M; 2.36M | 699.1M; 3.22M | 22.4M; 0.77M |
| `tokens_implementers` | 1,122.6M in (98%), 4.40M out | 6.68M a step. | 51.1M; 4.65M | 870.4M; 7.57M | 2,761.3M; 12.7M | 68.0M; 2.35M |
| `tokens_claude` | 880.9M in, 8.04M out: 0 before the claim, 876.1M in the run, 4.8M after | "0 before the claim" is a miss: 1,008M of planning and design input is on this machine (proposal 9). "After" is this retrospective's, at the time of the metrics. | 439.4M | 520.1M | 5,160.9M | 182.8M |
| `cost_claude` | $312: supervisor $276, designer $36, retrospective $1 | Uncounted on this machine: five planner sessions, $221.48, and the third design round, $34.02. | $174 | $192 | | $54 |
| `supervisor` | 15 sessions, 13.4 h, 2.7M weighted tokens an hour, 63% in the 40 requests after each start or compaction, 28 compactions | A sixteenth session, `d8cb9137`, is missing from `claude.tsv`. Section 7 reads these numbers. | | | | 1, 3.8 h, 0.7M, 36%, 1 |
| `tokens_per_step` | 14.4M | | 19.1M | 13.3M | 39.8M | 5.38M |
| `coordinator_requests` | 1,961, 32 after a wait ran out | 11.7 a step. The 32 timeouts cost 121 requests and 22.5M input. | 162, 9 | 1,404, 60 | 3,567, 67 | 188, 3 |
| `compactions` | 4 | At 354K to 384K of its 475K window. | 0 | 4 | 9 | 0 |
| `gates_per_step` | 1.3 | 219 gates: 189 passed, 30 failed. 13 passes (6.9%) went unused after `main` moved. | 1.4 | 2.3 | 2.2 | 2.1 |
| `gate_lane_other` | 21 | In truth 20: five measurements, fourteen supervisor scratch gates, one clippy run. The 21st is a line of a 25-line row (section 9). | 14 | 0 | | 1 |
| `queue_wait_ratio` | 0.74 | The build lane 0.99, the gate lane 0.06. 80% of the build lane's waiting fell while a gate held it. | 0.29 | 0.31 | 2.24 | 0.83 |
| `suites` | hearth: 14 tests / 0.007 s first; 1,945 / 7.142 s and 48 / 15.804 s last | skein's first row is 089ed83's invalid gate (section 5, item 11). smith: one landing. | | | | |
| `proofs` | unit 52, world 61, joint 11, process 71, live 10; 0 below plan | | process 2 | unit 67, world 28, joint 8, process 11, live 1, none 77 | unit 92, world 71, joint 15 | process 27, live 8 |
| `report_lines` | 12 median, 16 p90 | 282 reports. | 12, 15 | 13, 16 | 10, 11 | 11, 11 |
| `lines` | code +45,176 −3,776; tests +72,259 −3,985; data +10,260 −72 | Tests 1.6 times the code. Data is mostly real git and mmdebstrap recordings. | | | | |
| `contexts` | 172K median, 223K p90; 66 handovers | 6 handovers past 250K, the largest at 344K. | 113K, 182K; 2 | 225K, 262K; 45 | 238K, 319K; 70 | 113K, 160K; 3 |
| `pin_lag` | smith: skein 51; jig: skein 77, smith 193; hearth: skein 24; temper: skein 72, smith 193, jig 1 | smith is two `Plan: render` commits behind (item 9). | | | | |

**Not comparable, and why:**
- **`tokens_claude`, `cost_claude`.**
  - No planning or design is counted. The plan's landing, `87e03de`,
    writes its nine `Planner-Session:` lines in a paragraph apart from
    its trailers, so git reads none. Its Inputs name `format.md`, not
    `docs/design/format.md`, so `design_inputs` finds none.
  - On this machine: planners 5 to 9, $221.48 (5,661 requests, $191.58
    of it in 96 Sonnet sub-agents), and the third design round
    `1a9d02d8`, $34.02. Planners 1 to 4 ran on x1.
  - Earlier runs' planners cost $38 to $78. render's on this machine
    alone is about three times the largest.
  - `d8cb9137`, a supervisor session, is missing: 53 requests, $2.37.
  - render prices each message by its model; forgejo-17, protocol and
    jig-protocol priced Sonnet at Opus's rates.
- **`supervisor`.** Only kit-batch has it. Its bootstrap share is the
  share of requests that fall in the first 40 of a window, which a
  window of 52 to 58 requests makes about 70% by construction (section
  7). It leaves out the sub-agents, 59% of the supervisor's cost.
- **`gates_per_step`, `gate_lane_other`.** The claim kit's `heavy`
  wrote its row from the client, and row 7's 25-line command splits the
  metrics' line-based reading. 32 rows have no log file; one pueue task
  (26) has no row.
- **`escalation_wait_min`, `blocked_h`** of protocol are negative, a
  fault fixed after it.

**What explains the largest differences:**
- **Hours per step.**
  - Up to six sessions at once (eight, briefly), under the six-session
    cap that disk below 60 GB sets (development.md 11.4): 3.9 threads
    on average before 20:00, 1.9 after. Memory never bound: never below
    3.3 GB, above ruling 101's 3 GB floor.
  - hearth's gate was short: 18 s in median, 11 s of it alone.
  - Blocks were answered in minutes, and the ready checks had corrected
    each session file before its start.
  - After midnight the chain narrowed to sessions 28, 29, 30 and 47,
    and the 1 GiB tail ran alone: 4 to 8 landings an hour.
- **Tokens per step.**
  - The implementers' 6.7M a step is below protocol's 7.6M and half
    jig-protocol's 12.7M, with large increments: 792 lines changed per
    commit, in median.
  - Their contexts were high: 40% to 52% of their input was in requests
    past 150K. Polling took 16.5% of it, and before 20:00 the rule
    documents each thread read at its start carried 22.5% (section 5,
    item 10).
  - The coordinator spent 2.39M a step, as protocol's did. A third of
    its input was its own output read again: spawn prompts grew from
    1.1 KB to 10 KB, as the "Pass on" texts accumulated.
- **Queue waits.** The two build slots were 24.7% used. The waiting came
  from the gate's exclusivity: hearth's gates spent 57% of their time
  draining the build lane, behind skein's two-minute prebuilds and the
  hourly downstream builds. development.md 13's open question, "the
  build lane's width", has its answer for this machine: two slots
  suffice, and the gate's drain is what waits.

## 4. What worked

1. **Throughput on a new repository.**
   - 168 steps in 13.4 running hours: 117K lines of code and tests,
     nearly all in hearth, which held none of them at the claim.
   - The first session started 4 min 37 s after the claim, and the first
     step landed at 15:51:21.
   - The coordinator recorded a landing a median 13.3 s after its
     report, and started the next session a median 63 s after a
     landing.
2. **Fast answers, with evidence.**
   - Median 2.7 minutes from a block to its ruling. Rulings cite the
     command and the log, as ruling 111's grep of six passing gate logs
     against the one that failed.
   - Interim rulings let work go on while a question was with the
     designer (121, 124, 169, 183, 199). No code needed rework after
     the answer.
   - 19 of the last 21 blocks were answered by adopting the
     implementer's own proposal: the questions came with their options.
3. **The ready checks turned a plan ahead of its code into one that
   ran.**
   - 48 correction rulings before sessions started: Files added, sketch
     paths, line numbers, test counts.
   - Four design questions came from ready checks, before any code:
     03.1, 04.1, 21.4 and 47.1.
   - Every Proof reached its planned layer: 0 below plan.
4. **Implementers stopped where they should.**
   - Every increment past its Size tolerance stopped before landing.
   - 08.2 stopped after two live fetch runs, and asked before a third;
     skein-02.3 refused to touch an unrelated failing test.
   - Red-then-green proofs in 28.s1, s5, s6, s7, s8, 03.s2 and 45.s2.
5. **The design channel.**
   - Nine questions, nine design commits, each editing the affected
     session files in the same commit. Median 10.7 minutes from the
     question to the answer.
   - Ruling 192 foresaw that the defaults' question might come back,
     and said what would bring it back. It did, and decd06a answered
     it in 25 minutes.
6. **The machine held.**
   - Ruling 101 lowered development.md 11.4's 6 GB start floor to 3 GB
     for this 8 GB machine, four minutes into the run. It held: no task
     was killed, and the build lane never narrowed.
   - Disk was the bound: starts held below 40 GB for about 1.5 hours in
     all, never near the 15 GB stop.
7. **Wall-clock's card fixes reached the run in 21 minutes.** Until
   20:06, 49 implementer threads hit the refusal of `rm -rf target`,
   which the claim's card prescribed, and 38 lost and resent their
   report with it. `b30cc95` landed at 20:05; the 20:18 check took its
   points 2 and 6, and the 20:26:54 steer sent them. After it: no
   refusal, and no `python` call.
8. **Cheap handovers.** A successor started a median 0.4 to 0.7 minutes
   after its predecessor's report, read about 50K tokens of card,
   session file, checkpoint and code, and made its first edit within
   1.6 minutes.
9. **The end.** done.md 3.5 minutes after the last landing; the
   focused suite back to 7.1 s with no ceiling raised; items 1 to 8 and
   10 passing on their first run; a cleanup block answered in 2 min 14 s.

## 5. What did not

1. **"Done when" item 9: smith pins a skein older than the plan's last
   landing.**
   - **Cause.**
     - The README starts smith-repin "once every skein step of the plan
       has landed", as development.md 5.7 says of a closing repin.
       Neither says what a skein step added after the repin does. 5.7's
       `<repository>-repin.s<k>` is an earlier repin, for a broken
       build.
     - hearth's sessions could add skein steps until hearth's last
       step, by the plan's "Upstream first". Two did: 28.4 needed
       skein-03.s1 (ruling 177) and 03.s2 (ruling 181).
     - Both rulings said "smith-repin stands", because neither step
       changes a signature. They checked who calls `spawn_restartable`
       and read the README's Repin section. Neither read item 9. Ruling
       177 spoke of "smith's next pin"; smith-repin was the closing
       one, landed six hours earlier. Ruling 181 copied 177's shape.
     - The session that wrote ruling 177 had just lost its context
       (section 5, item 7), and handed over a minute later.
     - The hourly checks reported "smith builds". `plan downstream`
       builds smith against skein's `main` in a scratch worktree, so
       it cannot see smith's own pin.
     - The coordinator wrote done.md from the rulings, running nothing,
       as development.md 7.6 asks.
   - **First visible:** 00:10:50, ruling 177; certain at 00:28:44, when
     skein-03.s1 landed.
   - **Acted on:** after the release. The sub-agent's run at 05:41
     found it; the alert followed at 05:41:28. The run could no longer
     add a step (proposals 1 and 2).
2. **The 1 GiB bound was summed at the end.**
   - **Cause.**
     - The design asks that the builder's worst case at the shipped
       limits fit in 1 GiB. Each crate had its `worst_case`, and ten
       sessions tested their own layer's. Only 28.5, the last increment
       of session 28, summed them.
     - The bound's premise was answered without a measurement. The
       planner had asked it (H27): "At the shipped limits git's listing
       decoder alone holds more than 256 MiB". The answer, by a fork of
       the third design round, ran no probe. The planner then renamed
       the test to `the_shipped_limits_have_a_worst_case_within_1_gib`,
       unmeasured.
     - parity's adopted rule, "a bound's premise checked by a command
       in `findings/`", did not reach a bound that no single command
       could compute before the code existed.
   - **First visible:** 01:14:16, 28.5's report: 1,240,426,175 bytes.
   - **Acted on.**
     - Ruling 184 at 01:18 let 28.5 land without the test. The answer,
       36959af, said the defaults would be decided "once the peaks are
       measured"; ruling 192 at 01:59 added 28.s1 to s6.
     - Each step's tests then found another holder the formulas left
       out: 28.s7 (ruling 198), 28.s8 (200), 28.s9 (202, 203).
     - At 04:48, 28.s6 summed the peaks at 2,497,232,530 bytes. The
       designer lowered seven defaults (decd06a, 05:13), and 28.s6
       landed at 05:24:59 at 899,981,218.
   - **Cost:**
     - nine supplements, four design answers and 4 h 13 min;
     - at most two sessions from 03:25 and one from 04:16, and none for
       28 minutes while 28.s6 waited on the designer;
     - session 28 open for 11 h 19 min, with 14 threads and 11
       handovers (proposal 4).
3. **The plan's numbers were guesses, and its Sizes ran low.**
   - **Cause.** The plan was written before hearth had code: its session
     files cite line numbers, sketch types and guess test counts and
     Sizes. The base runs compiled the sketches, but could not run what
     did not exist.
   - **Evidence.**
     - 48 corrections at ready checks and re-checks; 27 of them
       refreshed line numbers, and 22 re-cited skein lines "at hearth's
       pin 841f487".
     - Six counts guessed before the code (rulings 182, 187, 188, 194,
       195, 201), and nine "Done when" counts gone stale.
     - Ten Size rulings (107, 113, 130, 136, 150, 154, 156, 161, 168,
       171), at 1.95 to 5.3 times the Size. Every one said "land":
       "Nothing is trimmed to fit". The README's rule went from twice to
       three times (130), then four (168), and 18.4 landed past four
       (171). Five of them also found Files missing.
     - The pattern recurs: jig-protocol, client-composition, skein-ui
       and forgejo-17 each reported Sizes below the work.
   - **First visible:** 16:02 (ruling 103) and 16:21 (107).
   - **Acted on:** per session; the Size rule by standing rulings
     (proposals 5 and 13).
4. **Test shape, steered every hour.**
   - **Cause.** The test-shape rules were read, not checked. A steer
     reaches the running sessions and, through its "Pass on" text, their
     successors; a README rule reaches later sessions. Nothing in the
     gate refuses a constant-index read, a one-row table, a computed
     label, or an expected value written twice.
   - **Evidence.**
     - About 19 of the 45 steer points.
     - Expected values computed in the test: first at 15:53:53 (02.1),
       steered at 16:19:42. The fix over-corrected into one-row tables
       and constant-index reads, steered at 17:12:49, then made a README
       rule by ruling 137.
     - "A table row holds its expected value": steered at 16:24 and
       17:12, then ruling 176 at 23:39, "the third such sighting". Seen
       again at 00:27, 01:30 and 04:32.
     - "A setup written once": steered at 17:12, 20:26, 21:29, 22:35,
       23:39 and 00:41.
     - Some landed and stay (section 9, items 2 to 5).
   - **First visible:** 15:53:53. **Acted on:** by a steer at nearly
     every hourly check, never by a check (proposal 6).
5. **The focused suite stayed past its aim for 106 minutes.**
   - **Cause.**
     - The README says the next increment of a session that misses the
       aim first makes the slowest tests cheaper. Session 30, whose 30.1
       added 15 fuzzy tests and crossed 8 s at 02:30:01, never reported
       a measure run.
     - The 02:31 hourly check judged 8.3 s against the 15 s budget, not
       the 8 s aim ("the 02:31 check measured the focused suite against
       its budget, not its aim").
     - The pulse prints the budgets (`budgets: 15 60`), not the aims.
   - **First visible:** 02:30:01, 30.1's gate at 8.307 s.
   - **Acted on:** 03:42:10, a steer, after ten landings at 8.3 to 10.7
     s. 47.1 brought it to 7.342 s at 04:16, keeping each memory case's
     peak and every ceiling (proposal 7).
6. **The code graph answered nothing.**
   - **Cause.** hearth's index is from 2026-10-17T09:41:04Z, with 890
     nodes, and never refreshed: it says so today, with 768 Rust files
     on `main`. The workspace's AGENTS.md says "the index refreshes
     itself". Implementers may not index (the same file), and nobody
     else did.
   - **Evidence.**
     - 59 of 128 graph calls returned nothing before 20:00, and 74 of
       121 searches after.
     - Reports mention empty graph results about 60 times.
     - The graph skill's `SKILL.md` was read 62 times, 172K tokens,
       before 20:00 alone.
     - The 04:43:07 steer fixed the project's name, not the index.
   - **First visible:** 16:35:37, 40.1's report. **Acted on:** never
     (proposal 8).
7. **Supervision handed over by the clock, and lost work at each
   handover.**
   - **Cause.** development.md 10.5 hands the supervisor over after its
     second compaction. Its context filled every 29 minutes in median:
     tool output (session files, git, the plan) and its own 1.44M
     output tokens. So it handed over about once an hour.
   - **Evidence.**
     - 16 sessions; 13 of 14 successions "at its second compaction".
     - Eight sub-agents stopped mid-check at a handover, $12.5, most
       relaunched by the successor for about $11.5.
     - Two hourly checks came 71.7 and 65.5 minutes after the one
       before, across handovers.
     - At 23:44:56 a `/resume` moved the supervisor's job from
       `bb3ca7c9` to a new session, `d8cb9137`, which woke at 00:06
       without context. `plan handover` refused it, knowing only the
       old id. It wrote ruling 177, declared itself confused, and handed
       over at 00:11:30. Who ran the `/resume` is not in the transcript.
     - Three re-checks lost the race to a short step: 23.1 (ruling 141,
       withdrawn by 142), 45.2 (45.s2) and 47.1.
   - **First visible:** 16:29, the first handover. **Acted on:** as the
     process says (proposals 10 and 11).
8. **The process documents followed `main`, while the kit stayed.**
   - **Cause.** Roles read development.md and their cards from
     `~/src/rust/skein`, which moved six times during the run. The kit
     in `kit/` stayed at `11af042`.
   - **Evidence.**
     - After its fourth compaction, the coordinator re-read section 7
       from the checkout, which had `plan landed` by then. At 02:51:13
       it called it, and the kit refused (exit 2).
     - The last supervisor re-read `main`'s supervisor.md after its
       compaction at 04:05, with a watch event its kit lacks.
     - Steers cited testing-strategy.md section 10, which the kit's
       copy lacks (the 02:31 check's own note).
   - **First visible:** 02:51:13. **Acted on:** the coordinator turned
     to `git show 11af042:` at 05:14 (proposal 3).
9. **The planning cost is invisible.** Section 3 has the facts. Five
   plan landings across the workspace have the same blank line before
   their trailers: hearth `87e03de`, skein `11af042` and `4e293b7`, and
   temper `05fe185c3` and `5cfc5a2ba`. Neither `land` nor `plan check`
   reads them. kit-batch's proposal 8, adopted, names a missing
   transcript; it cannot name a session it never finds (proposal 9).
10. **Implementers carried large contexts, and polled.**
    - **Handovers late.** All 66 were the coordinator's, after a
      landing, at a median 182K; 22 past 200K, 6 past 250K (hearth 20
      at 344K). 18 waited for an increment to end. A handover cost
      about 50K fresh tokens and 1.6 minutes (proposal 12).
    - **Polling.** 1,384 requests only waited on a running command:
      16.5% of the implementers' input. Before 20:00, 364 of 1,004
      `heavy` and `land` calls yielded after a second or less, against
      the card's 300 s.
    - **Lints.** 59% of the implementers' clippy runs failed, most on
      hearth's disallowed methods (`Option::map`, `Iterator::collect`,
      `Iterator::map`), `too_many_lines` and `type_complexity`. Each
      session learned them anew.
    - **Conflicts.** 65 `land` calls stopped on rebase conflicts, most
      in shared fixture files and the root `Cargo.toml`.
11. **Smaller faults.**
    - **skein-02.1 landed on an invalid gate** at 15:55:37. Its
      implementer passed its own script, without `set -e`, as `land
      --gate`: 543 of 2,484 tests ran, one failing. Ruling 102 added
      02.s1 2.8 minutes after the block, and ruled that the gate is
      passed only as the README writes it.
    - **The coordinator removed a running session's `target/`** at
      16:34:21, and skein 04's gate failed (ruling 111).
    - **`~/.local/bin/{heavy,land,plan}` pointed to a missing
      `/tmp/proto-fixes/kit/`** at about 21:39. Four
      implementers reported it; the coordinator relinked them. Who
      broke them is not visible.
    - **The downstream check worktrees blocked the release.** Their
      `Cargo.lock` had been rewritten by the hourly builds; `plan
      worktree --remove` refused at 05:28:36 (ruling 205). The third
      run with this, after skein-ui and forgejo-17. `main`'s kit
      removes `plan downstream`'s worktrees on every exit (`a5994d2`),
      which the run's kit lacked.
    - **The hourly downstream builds** of jig and temper failed in 21
      of 22 rounds, as expected while their repins wait. The 44
      downstream builds cost 2,129 s of the build lane.
    - **The coordinator's `context.md`** failed its own size asserts 12
      times, and ended with words run together ("ruling205") to fit 200
      characters a line.

## 6. Decisions taken during the run

105 rulings, 101 to 205: 37 ready checks and 11 re-checks, 32 plan
defects, 9 design answers, 4 interims, 7 steers made rules, 4 process
and 1 Size.

| Decision | Rulings | Reached the design? |
|---|---|---|
| A container's `mutable` is refused before M5; the tokenizer places string faults; the tree world keeps its name | 121, 122 | Yes, `3f16a56` |
| A guard's name is an identifier | 124, 128 | Yes, `180c4a5` |
| A container renders whatever else fails on its host | 162, 165 | Yes, `562994c` |
| A third script name, or an order out of range, is a decode error | 169, 170 | Yes, `a079343` |
| Keys are unique within their declaration | 183, 185 | Yes, `01ffde6` |
| Render's worst case is what it holds at once; each layer's formula is its peak; errors kept to 256 and 16 MiB | 184, 192 | Yes, `36959af` |
| A diff's second side decodes only the changed blobs | 196, 198 | Yes, `9a639f6` |
| A template counts 4 bytes a line | 199, 200 | Yes, `a28817a` |
| The shipped defaults change, to fit 1 GiB | 204 | Yes, `decd06a` |
| An expanded record counts as a map of its fields; a range field as its base and count | 202, 203 | render.md 2.3 as it stood; 203's byte counts are the plan's |
| `null` inside a list is a decode error, owned by 12.s1 | 145 | The design had it (format.md 4.1, render.md 7.2) |
| Client records' wordings and fields: forms, a store refusal, the resolution kind, `Ids` with its entry, `Owns::User` | 108, 139, 148, 149, 153, 157, 167, 174, 175 | No. A grep of `docs/design` finds none of their wordings; client/README 3.2 makes field names stable within a major version (inference) |
| No generic item in production code | 155 | skein's programming-model.md 1 and 10.3 already say it |
| Test shape: no constant-index read (137), a round trip's expected value is its input (144), written once (159), a row holds its expected value (176) | | The plan's README. testing-strategy.md 10 says the last two at `main`, not in the run's kit |
| An increment past twice its Size lands at its count, up to four times | 130, 168 | The plan's README; implementer.md 8.6 still says twice |
| Start floor 3 GB on this 8 GB machine | 101 | No: development.md 11.4 says 6 GB |
| One gate retry for an unrelated failure; `target/` removed only after a session's final report | 111 | Process; not in development.md |
| The skein gate is passed as the README writes it | 102 | The plan's README |
| smith-repin stands after a skein supplement | 177, 181 | In no document, and against "Done when" item 9 |
| Discard the downstream check worktrees' lockfiles | 205 | `main`'s kit (`a5994d2`) |

**The two reversed answers.**
- **Ruling 142 withdrew 141** at 18:52:55. 141's four corrections to
  23.1 came 52 s after 23.1 had landed, which met each another way. One
  process block of 1.7 minutes; no repair.
- **Ruling 193 reversed 189's point 1** at 02:06:16, for one golden:
  render.md 5.7 orders the targets by name, so `bad-prod` prints first.
  29.4 blocked for 9.4 minutes.

**For the user.** decisions.md's six veto points (decisions 2, 9, 10,
13, 19, and H169 with H174) have no `Decision:` line. The run built
each as written. kit-batch's proposal 3, adopted at 03:54, now refuses
such a plan at `plan check`; the claim's kit did not.

## 7. The proposals adopted after the last retrospective

The last retrospective before the claim is forgejo-17's, decided by the
director at 17:19 on 10-16 and applied in skein `ce02728` and `e6fd9fb`,
both in `11af042`. wall-clock's and kit-batch's landed during the run
(below).

| Proposal | Decision | Meant to move | forgejo-17 | render | Moved |
|---|---|---|---|---|---|
| 1. A plan lands finished: design questions answered, each command run at its base | adopted | steps added from a fault already on `main` | 2 | 17 added; base runs B1 to B8 ran every command. 9 came from one bound whose premise no command could check (section 5, item 2) | No |
| 2. A session file at most about 100 lines an increment, 400 a file | adopted | the planner's cost, implementers' tokens a step, context p90 | about 400 an increment; 4.65M; 182K | 127 an increment, 22 of 59 files past 400; 6.68M; 223K | In part |
| 3. A run takes its claim's kit | adopted | decision rows waiting on the kit; re-gates | 30; 1.4 | `heavy`, `plan`, `land` stayed `11af042`'s; 1.3. The documents did not (section 5, item 8) | Yes, for the kit |
| 4. A re-check never holds a start | adopted | holds by re-checks | 9.5 and 11 min | none; three re-checks lost the race instead | Yes |
| 5. The supervisor's sub-agents on Sonnet | adopted | supervisor cost, usage stops | $37, 1 stop | all 91 on Sonnet, $163; $1.64 a step against $3.4 (forgejo-17's priced at Opus's rates); no stop | Yes |
| 6. `plan downstream` removes its worktrees; the supervisor watches until the release | adopted | minutes to release; escalations at the end | 24; 1 | 3.4 minutes; 1 (ruling 205) | In part: its kit part was not in the run's kit |
| 7. A Size tolerance of twice; nothing trimmed to fit | adopted | Size escalations below twice | 1 | 0 below twice; ten above (section 5, item 3) | Yes, but the Sizes were wrong |
| 8. A Proof starts with its layer's word | adopted | proofs counted | 2 of 11 | 205 parts, each a layer's | Yes |
| 9. A usage notice changes nothing | adopted | false handovers | 1 | no notice | Not tested |

**The experiment of compaction in place** (development.md 13, skein
`71f6c38`, in the run's kit). Its criteria, against temper/client-api's
supervisors, which handed over by context (1.8M weighted tokens an hour
and 16% in their first 40 requests near 255K; 1.6M and 44% at 150K):
- **Weighted tokens an hour: 2.7M. Up.** Confounded: render's
  supervisor ran 77 checks of session files and wrote 105 rulings in
  13.4 hours. kit-batch, under the same rule, ran 0.7M.
- **The bootstrap share: 63%. Up, and not a measure of re-reading.**
  - A window averaged 52 to 58 requests, so 68% to 71% of all requests
    fall in some window's first 40.
  - A request cost about $0.05 at any point of a window.
  - What a restore read cost: 161 requests and $9.86 in the second half,
    about $8.7 in the first, together 16% of the Opus sessions' $112.9.
- **Missed watch events: none found.**
- **Process mistakes in the hour after a compaction: two.**
  - The 02:31 check, seven minutes after a compaction, judged the
    focused suite against its budget.
  - The last session, after its compaction at 04:05, read `main`'s card
    rather than the kit's.
  - The confused handover at 00:11 followed a session-id change, not a
    compaction.
- **Verdict.** By its own words, the experiment does not hold: neither
  measure fell. The measures cannot separate the rule from the run's
  density, and the cost the rule did cause is the handover at the
  second compaction: 14 handovers, eight stopped sub-agents, two late
  hourly checks. Compaction itself was cheap: about 7 requests and $0.4
  to restore (proposals 10 and 11).

**Changes to the process that landed during the run** (skein, 15:43 to
05:32):

| Commits | Landed | What | Did the run take it? |
|---|---|---|---|
| `b30cc95` | 20:05:17 | wall-clock's decisions, text | Points 2 and 6 by the 20:26:54 steer (`cargo clean`, `python3`); 3 and 4 as the supervisor's own (a chain of one-time cron jobs, a process listing at each check); 1 and 5 bind later runs. Taken 13 minutes after landing |
| `e28122c`, `9ffbcfd` | 23:48, 23:49 | kit-after-parity's rules and kit | No: "render keeps its process" |
| `dfd2c1c` to `c611973`, 27 commits | 23:51 to 01:58 | kit-batch's steps: `plan landed`, the landing turn, `heavy`'s rows inside the task, the ring that loads the coordinator, the watch's `started:` and `hourly:`, the metrics' fixes | No. The run's kit stayed `11af042`; its documents moved (section 5, item 8) |
| `9c57a6b` | 03:50:23 | kit-batch's decisions, text | Point 1's premise and 6 as the supervisor's own; 9, the graph project's name, by the 04:43:07 steer. Taken 42 minutes after landing |
| `2a9bcd3`, `5f26abb`, `92633c1`, `23cf08d` | 03:52 to 04:09 | kit: test-plan's budget, the "For the user" refusal, `plan status` on a released run, the shell kit counted as code | No. This retrospective's `plan metrics` ran with them |
| `05aa3c4`, `0b08745` | 00:28:44, 01:01:41 | skein-03.s1 and 03.s2, this run's | Not process |

## 8. Proposals

1. **A ruling that adds a step to a repository whose downstream
   closing repin has landed also adds that repin again, as
   `<downstream>-repin.s<k>`, to start once the added step lands.**
   - Evidence: section 5, item 1. Rulings 177 and 181 kept smith-repin
     on its build alone; "Done when" item 9 failed. hearth, which pins
     skein from its own sessions, repinned twice (c94ad33, c1f276e).
   - Changes: development.md 5.7 ("When it starts") and 10.3 (what a
     ruling that adds a step names); supervisor.md.
   - Moves: "Done when" repin items failing at the end, 1 to 0; the
     plan's repinned repositories' pin lag at the release, two `Plan:`
     commits to 0.
2. **The coordinator releases the run only after the supervisor's
   sub-agent has run every "Done when" command, and each passes or a
   ruling records the miss; STATUS.md says `done` only then.**
   - Evidence: the release at 05:31:56, the run at 05:32:36 to 05:41,
     STATUS.md marked done at 05:32:46, before the results. Item 9's
     failure could no longer become a step. done.md called it covered.
   - Changes: development.md 7.6; supervisor.md, section 8.
   - Moves: "Done when" items found failing after the release, 1 to 0.
3. **Every role reads development.md, its card and testing-strategy.md
   from the run's `kit/`, never from the skein checkout.**
   - Evidence: section 5, item 8: the coordinator's refused `plan
     landed`, the last supervisor's card, steers citing a section the
     kit lacked. The checkout moved six times.
   - Changes: development.md 3.6; the start and compaction sections of
     each card; the prompts `plan` gives each role.
   - Moves: commands a role calls that the kit lacks, 1 to 0; steers
     citing text the kit lacks, to 0.
4. **When a test or a "Done when" item bounds a sum over several
   sessions' parts, the plan's first increment that can compute the sum
   lands it as a reported number, and later increments keep it
   current.**
   - Evidence: section 5, item 2. The 1 GiB sum was first computed 9.5
     hours into the run, by 28.5; nine supplements and four design
     answers followed, with the run's last two hours at one or two
     sessions.
   - Changes: development.md 5.3 and 5.4; for the designer, 4.5 (a
     bound is answered with its first measure, or with "measure it
     first").
   - Moves: steps added for one bound in a run's last quarter, 9 to 0;
     hours at the tail with at most one session running.
5. **An increment past its Size lands without asking, as long as it
   keeps to its Files and the README's rule for edits outside them; its
   report gives the count by file.**
   - Evidence: section 5, item 3. Ten Size blocks, each answered "land";
     the rule climbed to four times and was passed anyway. The five
     that found Files missing were questions of Files, which stay
     questions. Four earlier runs reported Sizes below the work.
   - Changes: implementer.md 8.6; development.md 5.3.
   - Moves: Size escalations, 10 to 0; plan-defect escalations about
     Files, unchanged.
6. **A test-shape rule that an hourly check sends a second time, and
   that a script can test, becomes a check in the repository's gate,
   added as a step, instead of a third steer.**
   - Evidence: section 5, item 4. About 19 steer points; one rule sent
     six times; four README rules broken after they were written; parity,
     wall-clock and kit-batch each reported test shape steered late.
     Constant-index reads, one-row tables, `let [x] = … else {
     panic!(…) }`, `.first().expect` and the names `actual` and
     `observed` are each a grep.
   - Changes: development.md 10.3 and 14.4; testing-strategy.md 10.
   - Moves: test-shape steer points a run, 19 to at most 5; sightings
     after a rule is written, at least 4 to 0.
7. **`plan pulse` reads the plan's test-time aims beside its budgets,
   and names each landing whose focused or fuzzy time is past its
   aim.**
   - Evidence: section 5, item 5. Ten landings past the aim before the
     steer; the 02:31 check read the budget.
   - Changes: development.md 14.2, and the README's Pulse settings
     (5.2).
   - Moves: landings past an aim before a steer, 10 to at most 2.
8. **`plan check`, and each hourly check, compare each plan
   repository's graph index with its `origin/main`, and the supervisor
   re-indexes one that is older; until it does, the implementers'
   prompt says not to use the graph.**
   - Evidence: section 5, item 6. An index six hours older than the
     claim, never refreshed; half the searches empty; the skill read 62
     times in four hours.
   - Changes: the workspace's AGENTS.md ("the index refreshes itself");
     development.md 6 and 14.1; `plan-check.sh`.
   - Moves: the share of implementers' graph searches that return
     nothing, about 50% to under 10%.
9. **Every Claude session that worked on a plan is counted: `land`
   refuses a plan landing whose `Planner-Session:` lines git does not
   read as trailers, the metrics find the Inputs' design documents by
   name, and `plan` records each session id a role's job takes.**
   - Evidence: section 3. $255.50 of planning and design on this
     machine uncounted, against $312 counted; the same blank line in
     five plan landings; `design_inputs` matches only `docs/design/`
     paths; `d8cb9137` missing from `claude.tsv`, and refused by `plan
     handover`.
   - Changes: development.md 3.5, 9.1 and 15.2; the kit's `land`,
     `numbers.sh` and `plan-roles.sh`.
   - Moves: `tokens_claude` before the claim, from 0 to the planning's
     tokens for each plan whose sessions are on the machine; supervisor
     sessions missing from `claude.tsv`, 1 to 0.
10. **The supervisor hands over only when it finds itself confused, or
    when the watch says so, not at a count of compactions; its
    sub-agents run to their end across a handover, and the successor
    reads their reports.**
    - Evidence: section 7's verdict. Restoring after a compaction cost
      about 7 requests and $0.4; the handovers stopped eight sub-agents
      ($12.5, then relaunched), and preceded both late hourly checks.
      The one confused session followed an id change, not compactions.
    - Changes: development.md 10.5 and 13; supervisor.md, sections 1
      and 7.
    - Moves: supervisor sessions a run, 16 to at most 3; sub-agents
      stopped by a handover, 8 to 0; the supervisor's cost a running
      hour, $20.6 to less.
11. **The `supervisor` metric gives each session's restore cost, the
    requests and dollars from a start or compaction to its first watch,
    and counts its sub-agents, in place of the share in the first 40
    requests.**
    - Evidence: section 7. The share is about 70% by construction when
      windows run 52 to 58 requests; the sub-agents were 59% of the
      supervisor's cost and left out.
    - Changes: development.md 15.2 and 13; `metrics.sh`.
    - Moves: the comparability of the supervisor's cost across runs.
12. **The coordinator hands an implementer over after a landing once
    its context is past 100K, not 150K.**
    - Evidence: section 5, item 10. With increments of 500 to 1,000
      lines, a session past 150K at a landing reached 182K in median and
      344K at most; 40% to 52% of the implementers' input was in
      requests past 150K. A fresh thread costs about 50K tokens and 1.6
      minutes.
    - Changes: development.md 7.2; implementer.md.
    - Moves: `contexts`, 172K median and 223K p90 to under 140K and
      180K; the implementers' tokens a step, 6.7M to less.
13. **A session file names code by path and symbol, never by line
    number.**
    - Evidence: section 5, item 3. 27 of the 48 corrections refreshed
      line numbers, and five rulings corrected again sessions already
      corrected, after `main` moved (163, 164, 172, 180, 189). The
      checks of session files cost $143.
    - Changes: development.md 5.2.
    - Moves: corrections a session at ready checks and re-checks, 0.94
      to less; their cost.

## 9. Debts left in the code

hearth has no `docs/development/tech-debt.md`. These belong in one,
with what was seen, why it matters and what pays it back. Paths are at
hearth `main`.

1. **Eleven files within 100 lines of the 800-line aim.** The next
   increment that touches one splits it first.
   - Tests: `hearth-protocol-builder/src/tests/translate.rs` 799,
     `tests/builder/src/fixtures.rs` 785,
     `hearth-protocol-builder/src/tests/fixtures.rs` 785,
     `hearth-domain-render/src/tests/fixtures.rs` 764,
     `tests/builder/tests/real.rs` 735.
   - Production: `hearth-client-protocol-records/src/errors.rs` 787,
     `hearth-protocol-builder/src/lib.rs` 784,
     `hearth-domain-render/src/compose.rs` 743, `schema.rs` 721,
     `entries.rs` 717, `call.rs` 704.
2. **`RANGE_BASE` and `RANGE_COUNT` are defined twice,** with
   different values: `hearth-domain-render/src/tests/fixtures/entries.rs:300`
   and `fixtures/collisions.rs:182`. Small.
3. **Render's `fixtures/hearth.rs` `target_with` takes `[Option<&str>;
   6]`,** six unnamed slots with string labels. The 01:40 steer asked
   for named enums; no increment edited the file after. Small.
4. **The oracle's rows wrap 63 expected values in
   `Outcome::Text(Cow::Borrowed(..))`** (`tests/render/tests/oracle.rs`,
   `oracle/engine.rs`). Small.
5. **47.1's six numbered cut-pipe tests** are one table. Small.
6. **No test proves that 31.1's record survives the scan** (the
   supervisor's note). Medium.
7. **The three at-the-limit memory tests** of the render and template
   worlds take 1.3 to 1.8 s each, 6.7 s of focused test time, and were
   flagged slow 1,000 times. They keep their peaks by the steer's rule;
   a smaller limit with the same formula would keep the proof. Medium.
8. **Tests that failed more than one gate,** cause not established:
   `thin::a_template_and_a_link_are_left_out_of_the_tree` (three gates,
   sessions 13 and 15), `compose::fleet_and_environment_keys_never_add_a_task`
   and `a_dropped_template_entry_is_never_run` (sessions 24 and 22), and
   `fuzzy_simulation::a_builder_killed_before_each_submission_restarts_cold_and_settles`
   (two gates in session 28). Medium.
9. **`#[must_use]` is ruled per session** (rulings 109, 118, 121, 123,
   125, 140, 157), never by a lint. Clippy's `must_use_candidate` could
   hold it. Small.
10. **Client records' wordings and fields set by rulings, not by the
    design** (section 6). The designer's, not code's. Medium.

The kit's, for skein:
- `plan pulse` greps binary goldens ("binary file matches"), carried
  from wall-clock's and kit-batch's sections 9;
- the pulse counts other repositories' gates;
- `plan status` and another writer raced on `status.md.new` at 23:33;
- `tmp/machine.env` sets `PLAN_MIN_MEM_GB=4`, ruling 101 3 GB;
- the metrics read `heavy.tsv` by line, and a multi-line command
  splits a row.

## 10. Decisions

For the user, or the director.

2026-10-18. The director, acting for the user (skein's development.md,
2.7), adopted all thirteen proposals. The designer applied their text in
skein b1d2276, and built the kit parts that were small through the kit's
gate: 3 in b9c4b96, 7 in 171ca57, 8 in 094a27c, 9's unread session
trailers in 114db4d and its new session ids in 0cfe3b5, 10 in e4a754f,
and 12 in 1dfc098. Two parts go to the next kit batch: 9's design inputs
found by file name, and 11's restore cost and sub-agents in the
`supervisor` metric; until then, the metrics keep the bootstrap share.
Two kit fixes came with them, found by the gate on x1: `land` reads git
2.53's refused push on a locked ref as main moving (441c41a), and
`test-plan` snapshots pueue in UTC (867095d).

The runs in flight, temper's client-api (kit 841f487) and forge-setup
(kit c0268a8), keep their kits (development.md, section 12). Each takes
1, 6, 8 and 13 through its supervisor's rulings and checks, and 2, 3 and
12 as steers to its coordinator. Each takes 10: client-api's supervisor
ignores its watch's word to hand over at the second compaction, and
forge-setup's kit, older than skein 71f6c38, hands over by context as
before. Each keeps its process for 4, 5, 7, 9 and 11. The commits'
bodies say so.

| Proposal | Decision | Where |
|---|---|---|
| 1 | adopted | skein b1d2276: development.md, 5.7 and 10.3; supervisor.md, 3 |
| 2 | adopted | skein b1d2276: development.md, 7.4 and 7.6; supervisor.md, 8 |
| 3 | adopted | skein b1d2276: development.md, 3.6, 6, 7.1, 7.5 and 10.4; supervisor.md, 1 and 6; implementer.md; skein b9c4b96: the roles' prompts (`plan-roles.sh`) |
| 4 | adopted | skein b1d2276: development.md, 4.5, 5.3 and 5.4 |
| 5 | adopted | skein b1d2276: development.md, 5.2 and 5.3; implementer.md, 8.6 |
| 6 | adopted | skein b1d2276: development.md, 10.3 and 14.4; testing-strategy.md, 10 |
| 7 | adopted: an `aims:` line for each repository | skein b1d2276: development.md, 5.2 and 14.2; skein 171ca57: `plan pulse` (`numbers.sh`) |
| 8 | adopted: an older index is a warning in `plan check` | skein b1d2276: development.md, 3.1, 3.6, 6 and 14.1; supervisor.md, 1; implementer.md; the workspace `AGENTS.md`; skein 094a27c: `plan check` and `plan pulse` |
| 9 | adopted; the design inputs by file name in the next kit batch | skein b1d2276: development.md, 3.5, 5.4, 9.1 and 15.2; skein 114db4d: `land`; skein 0cfe3b5: `record_jobs` (`plan-roles.sh`) |
| 10 | adopted | skein b1d2276: development.md, 1, 2.1, 2.2, 2.4, 2.5, 3.6, 10.4, 10.5 and 13; supervisor.md, 1, 2, 4, 5 and 7; skein e4a754f: `plan context`, `plan watch` and `plan wake` |
| 11 | adopted; the metric's kit part in the next kit batch | skein b1d2276: development.md, 13 and 15.2 |
| 12 | adopted | skein b1d2276: development.md, 1, 2.4, 3.6, 5.3, 7.3 and 10.3; implementer.md, 8.6; skein 1dfc098: `PLAN_CONTEXT_LIMIT` (`numbers.sh`) |
| 13 | adopted | skein b1d2276: development.md, 5.2 and 5.6; supervisor.md, 5 |
