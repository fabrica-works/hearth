# Session 47: render's digests in the joint worlds, and the budgets

render.md, 9.3's "every world" past the process tier, and 9.8 with 3.10's
cost facts, over what hearth 28 built. Split from hearth 30 (decisions.md,
20); it uses 30.1's generator.

## Read first

- hearth's `render.md`: 3.10, 9.1, 9.3, 9.8; `testing.md`: 2.2, 9.1, 9.3.
- This plan: `hearth/30-properties.md` (30.1's `repositories`,
  `fixtures::digests`), `28` (`Scenario`, `protocol::run`, `simulated::run`,
  `domain::Run`), `25` (25.1's `Call::Diff`), `29` (`golden::load`, `files_of`).

## What the design asks

- **9.3:** "one revision renders to the same digests in every world".
  **9.8, 3.10:** a revision "decoded once"; each blob read once.

## Choices this plan makes

1. **Digests in two more worlds:** the protocol world's cut pipes (28.3) and
   the simulated client (28.4), by the whole digest the JSON form prints
   (render.md 9.3; hearth 30, Choices, 3); render's world and real git are 30.1's.
2. **Budgets** (9.8) assert 3.10's cost facts as counts of a run's requests;
   time is an aim of 0.5 s a scenario, reported and never failed on (render.md
   9.8; testing.md 9.3).

## Rules that bind it

- **No expected value computed by the code under test:** the digests are the
  render world's; the counts are 3.10's facts.
- **Shared files:** `tests/builder/tests/properties.rs` (hearth 30.1, 30.2),
  `tests/builder/src/fixtures.rs`.
- **Test time** (decisions.md 15; a Test shape's bound is for its whole
  line): focused 0.5 s for the joint tests and 0.5 s for the world tests; no
  sweep.

## Increments

### 47.1 The digests in the joint worlds, and the budgets
Scope: 9.3's "every world" past the process tier; 9.8 and 3.10.
Needs: hearth 28.4 (28.2 to 28.4), hearth 25.1 (`Call::Diff`), hearth 30.1.
Files: `tests/builder/tests/properties.rs`, new `tests/builder/tests/
budgets.rs`, `tests/builder/src/fixtures.rs`, `tests/builder/src/domain.rs`
(`Run::new`, `:54`), `tests/builder/src/fixtures/worlds.rs` (`Shared::
for_scenario`, `:325`); each user of a changed `Scenario` or `domain::Run`
the compiler names: `tests/builder/src/simulated.rs`, and
`crates/hearth/tests/{refusals,end_to_end}.rs` (`hearth-builder-world` is a
dev-dependency, `crates/hearth/Cargo.toml:16`). No other repository uses the
builder world.
Shape: `protocol::run(&Scenario { files, form: Form::JsonLines,
..fixtures::thin() }, seed)` (`form.rs:12`; `thin()`, `fixtures/worlds.rs:179`,
renders) and `simulated::run(.., Config::calm())` print the `spec` records;
30.1's repositories are `Shape::SMALL`'s: the protocol world takes 1.66 s
over 8 seeds on the fleet, 148 ms on thin. The fleet is `golden::load("fleet")`.
Its diff's second revision is one `-prod` file edited: `domain::Run::new`
builds `Shared::new`, one revision, and `Shared::for_scenario` edits only
`MOTD_KIND`'s path (`tasks/motd/tree/etc/motd`, `fixtures/reserved.rs:55`),
which the fleet has not, or `x1-prod`'s with `resolve`; so the domain world
builds a diff's second revision as `for_scenario` does, with the fleet's
edit a fixture of its own. The counts read `domain::Run::requests()`
(`domain.rs:195`).
Test shape: joint, under 0.5 s in all:
`generated_repositories_have_the_render_worlds_digests_through_cut_pipes` (3
repositories, 8 seeds),
`generated_repositories_have_the_render_worlds_digests_through_the_simulated_client`
(3 repositories, calm),
`a_render_spawns_git_once_per_read_and_twice_more_and_a_known_resolution_no_mmdebstrap`
(the ledger's spawns, 28.4's world; testing.md 9.3's "programs run"; git:
`rev-parse`, `ls-tree`, and one `cat-file --batch` per `Read`, render.md 2.2,
"one process per `Read` (8.2)": thin and the fleet spawn 4, the
`resolution` golden 3; the known resolution: the `resolution` golden with
`test_min_recorded()` (`fixtures/resolution.rs:29`) laid and `targets:
["test-min"]`, as without them `test-boot` runs mmdebstrap); world, the same
0.5 s: `rendering_the_fleet_lists_once_and_reads_each_blob_once` (the fleet
with a second file in one of its task trees holding another's body, as
`DUPLICATE_BLOB`, `fixtures/worlds.rs:213`, is thin's: one `List`, no oid
twice among the `Read`s, fewer oids than the listing's entries),
`a_diff_reads_a_blob_the_two_revisions_share_once`,
`a_diffs_second_revision_reads_only_the_blobs_that_changed`.
```rust
// tests/builder/tests/budgets.rs
#[test]
fn rendering_the_fleet_lists_once_and_reads_each_blob_once() {
    // use hearth_domain_builder::vocabulary::Request;
    // use hearth_domain_render::vocabulary::Request as RenderRequest; (as tests/domain.rs:2-6)
    let mut files = fixtures::files_of(&golden::load("fleet").tree);
    files.push(/* a second file in one of the fleet's task trees, with another's body */);
    let scenario = Scenario { files: &files, ..fixtures::thin() };
    let run = domain::Run::new(&scenario, 0).finish();
    let lists = run.requests().iter().filter(|r| matches!(r, Request::Render(RenderRequest::List { .. }))).count();
    let mut reads = Vec::new();
    for request in run.requests() {
        // no `_` arm: wildcard_enum_match_arm
        if let Request::Render(RenderRequest::Read { blobs, .. }) = request {
            reads.extend(blobs.iter().map(|blob| blob.oid.clone()));
        }
    }
    let asked = reads.len();
    reads.sort();
    reads.dedup();
    assert_eq!((lists, reads.len()), (1, asked), "3.10: one listing, no blob twice");
    assert!(reads.len() < files.len(), "the shared blob is read once");
}
```
Proof: joint, the cut-pipes test: the real protocol layer over fake git, its
pipes cut, prints the render world's digests; process holds in 30.1; world,
the fleet test: counts of requests, highest in the domain world (testing.md
2.2). Bound: 0.5 s; 0.5 s.
Don't: assert a time (testing.md 9.3); raise a limit to fit a count.
Size: no counted lines. Removes: nothing.

## Interface sketches

None: the worlds' types are hearth 28's, the generator's hearth 30's.

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-47\.[1-1]'` prints 1.
2. In a worktree of hearth at `origin/main`: `heavy cargo nextest run -p hearth-builder-world -E 'binary(properties) | binary(budgets)'` passes, its `Summary` naming at least 8 tests.
