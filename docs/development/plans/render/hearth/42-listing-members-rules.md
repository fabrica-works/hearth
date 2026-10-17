# Session 42: the listing's refusals, git's failures, members, names, the composition rules

hearth 12 merges every document's layers and prints `decode`; this
session makes render say why a revision or a target cannot compose.
42.1 prints the listing's refusals (`repository`) and git's failures
(`tool`; its standard error leaves `/dev/null`); 42.2 discovers members,
names and duplicates; 42.3 the composition rules. Each proves on the real
loop's `hearth render` through real git. Needs hearth 12 (the merge, the
first `Read` of every document, the error record); hearth 12's Choices
and Rules bind here as there.

## Read first

- hearth's `format.md`: 2.1, 2.3 (first and last bullets), 2.4, 3.1,
  3.3, 3.4, 3.6; `render.md`: 2.2 (the `ls-tree` bullet), 2.3, 2.4 (last
  bullet), 3.1, 3.2, 4.4, 5.5 (first bullet), 7.2, 8.2;
  `client/README.md`: 3.2 (last bullet), 3.3.
- This plan: `hearth/12-compose.md` whole; `hearth/11-thin-story.md`
  (Choices, 11.2, 11.3); `hearth/03-json.md`'s 03.4; `hearth/02-git.md`'s
  02.1 to 02.3 and sketches; `hearth/40-client-diff-errors.md`'s 40.2.
- skein: `programming-model.md`: 3, 4.4, 6.3, 7; `testing-strategy.md`:
  2.1, 2.2, 2.8 (by number only).

## What the design asks

- **Discovery** (render.md 3.2; format.md 3.1, 3.6, 2.4): targets "from
  the listing and the bases' `members`"; members "named
  `<base>-<member>-<env>` ... A base with members has no target
  `<base>-<env>`", a base-env may "add one, remove one (`"3": null`), or
  set one's data there"; "Two targets of one name are an error"; a target
  "at most 63 bytes".
- **Rules** (3.3, 3.4): "`null` for a task there [a default] is an
  error"; "Every task named anywhere must exist"; placement "never in
  `fleet.json` or a base ... Every target must have a placement once
  composed"; `on`, "a target of the same environment that is not a
  container".
- **Bounds and errors** (render.md 2.3, 3.1, 7.2, 8.2): "A path under the
  four roots that format.md, section 2 does not allow is an error"; `tool`,
  "git ... failed, timed out or could not start"; printed "after what did
  render" (client/README.md 3.2).

## Choices this plan makes

- **The listing's limits are `decode`, configuration;** git's other
  failures `tool`, operational, ending the call `TryAgain` (hearth 11),
  replacing 11.2's "every `Failed` is operational". git's tail is its
  standard error's last lines, a `Tail` in `hearth-protocol-git`
  (render.md 2.2, 5.5: 20 lines of 160 bytes, no marker); mmdebstrap's chosen tail is hearth 07's.
- **Names:** 2.4's base, environment and member forms forbid a dot and
  every target holds a `-`, so a made name is never `hearth`: 42.2
  checks the 63 bytes alone. `tasks` is tasks in a revision, as
  "targets in a revision" before it. A container whose host failed is
  left out with no error of its own: it depends on the host's documents.
- **Later sessions:** 20 gives `target.containers`; 21 the `hearth` task;
  24 mmdebstrap's `Command` and `Ended::Timeout`; 45.1 git's diff
  `Command`.

## Rules that bind it

- **The meeting point** (README, Rules; hearth 12's Rules): every
  increment proves on the real loop (`tests/builder/tests/real.rs`) and
  extends `translate/errors.rs` and its reverse in the same increment
  for each variant it adds, no wildcard arm.
- **Shared files:** render's `lib.rs`, `vocabulary.rs`, `src/errors.rs`;
  the records' `src/errors.rs` (both also hearth 13, 14, 16, 19, 21, 22,
  24's); `hearth-protocol-builder`'s `src/translate/`;
  `tests/render/src/fixtures.rs`, `tests/builder/src/fixtures.rs`.
  Landing second, rebase, keep both.
- **No recursion, no computed expectation** (code-organization.md, 6):
  positions, messages and digests in tests are literals. Every
  increment's Files include `tests/builder/{src/fixtures.rs,
  tests/real.rs}`.

## Increments

### 42.1 The listing's refusals and git's failures: `repository` and `tool`
Scope: render.md 3.1's refusals under the four roots (format.md 2.1;
2.3's `templates/`; 2.4's base, environment and task names), a
submodule (2.2), 2.3's `tasks`; 7.2's `tool` for git with its standard
error's tail (8.2, 2.4); the listing's limits as `decode`.
Needs: nothing beyond hearth 12.
Files: render `src/{revision, compose, call, errors, vocabulary}.rs`,
`src/tests/{revision, errors}.rs`; `hearth-protocol-git` `src/{limits,
lib}.rs`, new `src/tail.rs`, new `src/tests/tail.rs`; protocol `src/{git,
read}.rs`, `src/translate/{errors, listing}.rs`, `src/tests/{git,
translate}.rs`; records `src/errors.rs`, `src/form/{text, json}.rs`,
`src/tests/errors.rs`; `tests/render/tests/{errors, thin}.rs`;
`tests/git/{src/fixtures.rs, tests/memory.rs}` (a `Tail` case);
`tests/builder/{src/fixtures.rs, tests/real.rs}`.
Shape: the sketches. `revision::class` gains `Template`, `Script`,
`Readme` (accepted for hearth 15 and 18) and `Refused`; an environment
not listed and a task directory without `task.json` are found by
`compose::targets`' pass over the listing, as are two sources of one
destination in a task (`tree/etc/foo` and `tree/etc/foo.j2`,
`customize.sh` and `customize.sh.j2`), `Destination` at the second by
path with the first its `also` (format.md 2.3), and the 1,025th task,
`TooLarge { Tasks }`, failing the call. A repository error is at its
path, no position, with `task` under `tasks/<task>/`, and fails what its
path belongs to (render.md 3.1; format.md 2.1: `envs/` holds only `<env>.json`,
a `.json` directly in `hosts/<base>/` that is neither document is refused,
deeper it is another file; an absent `fleet.json` or `<base>.json` is an
empty layer). git: standard error piped, each piece
`Tail::admit`ted; once `Exited` and both pipes ended, a failure is
`Failed { class:
Operational, at: None, failure: Tool(..) }`, `ended` `Start` (a refused
`Spawn`), `Exit`/`Signal` (`invocation::Exit`), `Output` (a decoder's
`Malformed`, `Truncated`, `Size`, `Other`), `Missing { oid }` or
`Stream`; `listing::Error::{TooMany, PathTooLong, PathsTooLarge}` are `Failed { class:
Configuration, at: Some(path), failure: TooLarge { Entries | Path | Paths, max
} }`. Records: `git ls-tree exited 128` (`… was killed by signal 9`, `…
could not start`, …), each tail line below it indented two spaces
(render.md 7.3); JSON `"tool":{"command":"ls_tree","ended":{
"code":128,"kind":"exit"},"tail":["fatal: not a tree object"]}`.
Test shape: `hearth-protocol-git`'s `src/tests/tail.rs` (new, under 20 ms):
```rust
#[test]
fn the_tail_keeps_the_last_lines_each_cut_to_its_limit() {
    // 2 lines of 8 bytes, beside SHIPPED's 20 of 160, so that three short lines pass both.
    let limits = Limits { tail_lines: 2, tail_line: 8, ..limits::SHIPPED };
    let mut tail = Tail::new(&limits);
    tail.admit(b"error: one\nerr");
    tail.admit(b"or: two\nfatal: not a tree object\n");
    let expected: [&[u8]; 2] = [b"error: t", b"fatal: n"];
    let lines = tail.lines();
    assert_eq!(lines.len(), expected.len(), "the last two lines, one split across pieces kept whole");
    for (line, want) in lines.iter().zip(expected) {
        assert_eq!(&**line, want, "each cut to 8 bytes");
    }
}
```
Then `a_last_line_without_a_line_feed_is_kept`,
`a_run_that_exits_zero_holds_no_tail`; memory
(`tests/git/tests/memory.rs`): 02.1's test gains a `Decoder::Tail` case at
its limits. Render unit:
`paths_format_md_2_1_does_not_allow_are_refused` (a `const` table:
`envs/lab/x.json`, `envs/Lab.json`, `envs/lab-.json`,
`hosts/x1/notes.json` (but `hosts/x1/pub/a.json` is another file),
`hosts/X1/X1.json`, `tasks/ssh/tre/x`,
`tasks/ssh/templates/a.conf`, `tasks/Ssh/task.json`, a document listed
as a link, a submodule under `tasks/`),
`a_base_env_without_its_environment_is_a_repository_error`,
`a_task_directory_without_task_json_is_refused`,
`two_sources_of_one_destination_err_at_the_second`,
`the_task_past_the_limit_errs_and_fails_the_call` (`tasks: 2` beside
`SHIPPED`'s 1,024); protocol, at `SHIPPED`:
`git_s_failure_answers_its_exit_and_standard_error_s_tail`,
`a_listing_past_its_entries_fails_as_configuration_at_its_path`,
`a_4_097_byte_path_fails_as_configuration_at_its_prefix`,
`paths_past_the_total_fail_as_configuration_at_the_last_path` (`paths_total: 64`);
records: `a_tool_error_prints_its_tail_below_it`; real:
`a_stray_file_in_a_task_prints_a_repository_error` (THIN with
`tasks/motd/tre/etc/motd`; exit 2; the line `tasks/motd/tre/etc/motd: motd: a
path format.md 2.1 does not allow`).
Proof: process,
`a_tree_git_cannot_read_prints_git_s_tail_and_exits_75`, bound 0.5 s: THIN's repository, its root tree's loose object removed
(new `Repository::git` and `remove_object`), prints exactly `git ls-tree
exited 128` then `  fatal: not a tree object` (git 2's words, the
same on 2.47.3 and 2.53) and exits 75: git's failure reaches the record
through the piped standard error, the tail, render and the printer.
Don't: read git's standard error beyond its lines; keep a tail of a run
that succeeded; refuse a file under `hosts/<base>/` that is not `.json`
(a `file` setting's, hearth 18); check that `templates/` are used
(hearth 18.2).
Size: at most 300 lines.
Removes: git's standard error to `/dev/null` (11.2's transitional
piece); 11.2's "every `Failed` is operational"; 11.1's
`a_base_env_without_its_environment_makes_no_target` (now an error);
32.1's silent submodule failure (its test gains the error). 11.3's
`a_directory_that_is_no_repository_exits_75` keeps 75 and now reads
git's tool error on standard output (client/README.md 3.2, 3.3).

### 42.2 Members, names and duplicates
Scope: discovery from the bases' `members` (render.md 3.2; format.md
3.1, 3.6); a member's two layers in presence, settings and placement
(3.2; 3.4, "a base-env layer or its members"); meta's `member`
(render.md 4.4); 2.4's 63 bytes; two targets of one name (3.1);
`composition`'s `Duplicate` and `Long`.
Needs: nothing beyond 42.1 (hearth 03.4's `Members` is hearth 03's).
Files: render `src/{revision, compose, call, errors}.rs`,
`src/parts/meta.rs`, `src/tests/{compose, errors}.rs`; protocol
`src/translate/{documents, errors}.rs`, `src/tests/translate.rs`; records
`src/errors.rs`, `src/form/{text, json}.rs`, `src/tests/errors.rs`;
`tests/render/{src/fixtures.rs, tests/compose.rs}`;
`tests/builder/{src/fixtures.rs, tests/real.rs}`.
Shape: `compose::targets` runs after the first `Read`: for each base and
listed environment with a base-env document, a cluster (format.md 3.6: when the base's
or the base-env's document has `members`; none left, no target; a `null`
member in a base removes nothing) makes `<base>-<member>-<env>` for each of the base's members the
base-env does not remove and each it adds, in name order; any other base
`<base>-<env>`. A name past 63 bytes is `Long`; names made twice (sorted,
neighbours compared) one `Duplicate`, `target` the name, `at` the first
maker by path (a base-env's path, or a member's key at its position),
`also` the second, makers ordered by (name, maker's path, key position),
every target of the name left out: `hosts/stoa-1/
stoa-1-lab.json: stoa-1-lab: two targets are named stoa-1-lab
(hosts/stoa/stoa.json:5:5)`. A named target not discovered stays 11's
`Usage`, unless a `hosts/` document is refused: then it is left out
under that decode error, and the call ends `Failed`, exit 2 (render.md 3.1;
client/README.md 3.3). The
1,025th target is `TooLarge { Targets }`, failing the call. A member's
layers follow its base's and its base-env's; placement is the last of
the environment's, the base-env's and its member's.
Test shape: unit: `a_base_with_members_makes_one_target_per_member`,
`a_base_env_adds_removes_and_changes_members`,
`a_member_s_layers_follow_its_base_s_and_base_env_s`,
`two_targets_of_one_name_fail_both_with_one_error`,
`a_target_name_past_63_bytes_is_refused`; world:
`targets_are_discovered_from_every_base_s_members`,
`the_target_past_the_limit_errs_and_fails_the_call` (`targets: 2`
beside `SHIPPED`'s 1,024),
`a_named_target_a_refused_base_hides_is_no_usage`.
Proof: process,
`a_cluster_s_members_render_as_targets_with_their_own_layers`, bound
0.5 s, on new `fixtures::CLUSTER` (`tests/builder/src/fixtures.rs`): `envs/lab.json` placing vm, efi,
incus; `hosts/stoa/stoa.json` `{"motd": {}, "members": {"1": {}, "2":
{}, "3": {}}}`; `stoa-lab.json` `{"members": {"2": {"placement":
{"kind": "vm", "firmware": "bios", "hardware": "pool"}}, "3": null}}`;
THIN's `motd`. Its `target` and `placement` lines (`lines_starting`),
the `member` cell as 10.1's text prints it, are exactly these, exit 0:
```text
target       stoa-1-lab   base stoa   member 1   environment lab
placement    vm efi incus
target       stoa-2-lab   base stoa   member 2   environment lab
placement    vm bios pool
```
Don't: split a name on `-`; give a base with members a `<base>-<env>`;
refuse `placement` in a base's member (42.3).
Size: at most 220 lines.
Removes: 11.1's discovery from the listing alone.

### 42.3 The documents' composition rules
Scope: format.md 3.3 (`null` for a task in a default; a task that does
not exist), 3.4 (`placement` in `fleet.json`, a base or its member; a
target without placement; `on`) and 3.6 (`members` in `fleet.json` or
an environment); 7.2's `composition` rows of M1.
Needs: nothing beyond 42.2.
Files: render `src/{compose, call, errors}.rs`, `src/tests/compose.rs`;
protocol `src/translate/errors.rs`, `src/tests/translate.rs`; records
`src/errors.rs`, `src/form/{text, json}.rs`, `src/tests/errors.rs`;
`tests/render/tests/compose.rs`; `tests/builder/{src/fixtures.rs,
tests/real.rs}`.
Shape: once a revision, a document's `Null` at each `null` task key of
`fleet.json` or `envs/`, with its `task`, except the key `hearth` (a null
there is 21.4's `HearthNull`, reserved: one error, not two); `Placement` at `placement`'s
position in `fleet.json`, a base or a base's member (format.md 3.4, 3.6: a base-env's member's stands);
`Members` at `members`' position in `fleet.json` or `envs/` (format.md
3.6); `Task` at each key, in any document or member, naming no listed
`tasks/<task>/task.json`, with its `task`, failing the targets that have
it. A target's `Unplaced` at its base-env's path; then, every placement
composed (by `compose::placement(target, documents) -> Option<revision::Placement>`:
the last of the environment's, the base-env's and its member's; 20.1
reuses it), each container's `on` names a discovered target of its
environment that is not a container, else `On { on, host: Unknown |
Elsewhere { environment } | Container }` at `on`'s position, with its
`target`. Placements here are whole where written (format.md 3.4: `kind`; `firmware`,
`hardware` for metal and a VM; `on` for a container), as hearth 03.3
checks. Messages (render.md 7.3): `null for a task in a default`,
`placement's on names rhi-lab, a target of lab`, `no task motd under
tasks/`, `no placement once composed`.
Test shape: unit: `null_for_a_task_in_a_default_is_refused_at_its_key`,
`a_null_hearth_is_no_compose_error` (a `fleet.json` `"hearth": null`: no
`Null` row; 21.4 reports it),
`placement_in_fleet_json_or_a_base_is_refused`,
`members_in_fleet_json_or_an_environment_is_refused`,
`a_target_without_placement_fails_with_its_error`,
`a_container_s_on_names_a_non_container_of_its_environment` (a `const`
table: none, another environment's, a container),
`a_task_named_that_does_not_exist_is_refused_at_its_key`; world:
`a_container_whose_host_failed_is_left_out_without_an_error`.
Proof: process,
`a_container_on_another_environment_s_target_prints_its_error`, bound
0.5 s, on new `fixtures::CONTAINERS` (`tests/builder/src/fixtures.rs`): `envs/{prod, lab}.json` `{}`,
`hosts/rhi/rhi.json` `{}`, `rhi-prod.json` placed vm, bios, serverhino,
`rhi-lab.json` vm, efi, incus, `hosts/chat/chat.json` `{}`,
`chat-prod.json` `{"placement": {"kind": "container", "on":
"rhi-lab"}}`. Its `target` and `hosts/` lines are exactly these, exit 2:
a rule across targets fails one and spares the others.
```text
target       rhi-lab   base rhi   environment lab
target       rhi-prod   base rhi   environment prod
hosts/chat/chat-prod.json:1:43: chat-prod: placement's on names rhi-lab, a target of lab
```
Don't: check a container's M5 parameters; give a host its containers
(hearth 20); check settings against schemas (hearth 13).
Size: at most 160 lines.
Removes: hearth 11's silent failure of a target with no placement; the
silent ignoring of `placement` in `fleet.json`, a base or its member
(hearth 12.1, 12.2, 42.1 and 42.2).

## Interface sketches

Every type derives `Clone, PartialEq, Eq, Hash, Debug` (`Copy` where it
can). hearth 12's sketches are the base: render's `errors`, `compose`,
`revision`, the records' `Kind`; the records mirror render's kinds
variant for variant (an `Oid` as hex).

```rust
// crates/hearth-domain-render/src/revision.rs (hearth 12's, extended)
/// 42.2: mirror config's `Members`, `Member` (`layer` `None` for `null`); `Document` gains `members: Option<Members>`.
pub struct Members { pub at: Position, pub each: Box<[Member]> }
pub struct Member { pub name: Box<[u8]>, pub at: Position, pub layer: Option<Layer> }
// 42.1: `Class` gains `Template { task, path }` (`templates/*.j2`), `Script { task, name }`
// (`customize.sh`, `software.sh`, `.j2` or not), `Readme { task }`, `Refused { task: Option<&'a [u8]>, why: Repository }`.

// crates/hearth-domain-render/src/vocabulary.rs (42.1), beside hearth 11's and 12's
pub enum Event { /* hearth 11's others */ Failed { request: Token, class: Class, at: Option<Box<[u8]>>, failure: Failure } }
/// Why a request failed.
pub enum Failure { Tool(Tool), TooLarge { limit: Limit, max: u64 } }

// crates/hearth-domain-render/src/errors.rs (hearth 12's; Repository, Tool 42.1; Composition 42.2, 42.3)
/// `Kind` gains `Repository(Repository)`, `Composition(Composition)`, `Tool(Tool)`; `Limit` gains
/// `Entries`, `Path`, `Paths`, `Tasks` (42.1), `Targets` (42.2).
/// `Destination`: two sources of one destination in a task, `also` the first.
pub enum Repository { Stray, Environment { env: Box<[u8]> }, Name, Declaration, Static, Destination, Submodule, Link }
/// Which run, how it ended, its standard error's last lines.
pub struct Tool { pub command: Command, pub ended: Ended, pub tail: Box<[Box<[u8]>]> }
/// hearth 24 adds mmdebstrap's, hearth 45.1 git's diff.
pub enum Command { RevParse, LsTree, CatFile }
/// hearth 24 adds `Timeout`.
pub enum Ended { Start, Exit(u8), Signal(u32), Output, Missing { oid: Oid }, Stream }
/// 42.2: Duplicate, Long; 42.3: the rest.
pub enum Composition { Duplicate, Long, Placement, Members, Unplaced, On { on: Box<[u8]>, host: Host }, Null, Task }
/// What a container's `on` names instead of a host.
pub enum Host { Unknown, Elsewhere { environment: Box<[u8]> }, Container }

// crates/hearth-protocol-git/src/tail.rs (42.1); Limits gains tail_lines 20, tail_line 160; Decoder::Tail
/// The last `tail_lines` lines of a run's standard error, each cut to `tail_line` bytes; `lines` oldest
/// first, a last line without its LF included.
pub struct Tail { /* the lines kept, the line in hand */ }
impl Tail { #[must_use] pub fn new(limits: &Limits) -> Tail; pub fn admit(&mut self, bytes: &[u8]); #[must_use] pub fn lines(&self) -> Box<[Box<[u8]>]>; }

// crates/hearth-protocol-builder/src/translate/errors.rs (hearth 12's; 42.1 on); reverses in src/tests/translate.rs
// `error` gains an arm per variant above; `translate::listing` gains the listing's `TooMany`, `PathTooLong`, `PathsTooLarge`.

// crates/hearth-client-protocol-records/src/errors.rs (40.2's; 12.2)
// `Kind` gains Repository(Repository), Composition(Composition), Tool(Tool).
// Repository, Tool, Command, Ended, Composition, Host: render's, variant for variant.
```

## Questions to raise when reached

None.

## Done when

1. `git log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-42\.[1-3]'` prints 3.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest
   run -p hearth-domain-render -p hearth-render-world -p
   hearth-builder-world -p hearth-protocol-builder -p hearth-protocol-git
   -p hearth-client-protocol-records` passes.
3. `git -C ~/src/rust/hearth grep -E
   '^[[:space:]]+(Decode|Repository|Composition|Tool)\((Decode|Repository|Composition|Tool)\),'
   origin/main -- crates/hearth-domain-render/src/errors.rs
   crates/hearth-client-protocol-records/src/errors.rs | wc -l` prints 8.
