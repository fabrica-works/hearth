use hearth_conformance_world::{fixtures::Scenario, git};
use hearth_git_world::world::{self, Settings};
use hearth_protocol_git::limits;
const REVISION_CASES: &[&str] = &[
    "rev-parse-branch",
    "rev-parse-commit",
    "rev-parse-tag",
    "rev-parse-missing",
    "rev-parse-tree",
    "rev-parse-sha256",
];
const LISTING_CASES: &[&str] = &["ls-tree-modes", "ls-tree-empty", "ls-tree-sha256"];
const BLOB_CASES: &[&str] = &["cat-file-blobs", "cat-file-missing"];
fn check(names: &[&str]) {
    let scenario = Scenario::new();
    for name in names {
        let case = git::CASES.iter().find(|case| case.name == *name).expect("named case");
        let run = scenario.run(case);
        let expected = scenario.expected(case);
        assert_eq!(
            world::replay(&run.recording(), &expected, &limits::SHIPPED, &Settings::whole()),
            Some(expected),
            "{name}: {}",
            run.stderr_text()
        );
    }
}
#[test]
fn real_git_resolves_each_revision_as_the_scenario_made_it() {
    check(REVISION_CASES);
}
#[test]
fn real_git_lists_each_scenario_tree_with_its_modes_and_sizes() {
    check(LISTING_CASES);
}
#[test]
fn real_git_s_batch_gives_each_blob_the_scenario_wrote() {
    check(BLOB_CASES);
}
#[test]
fn each_case_s_run_is_its_recording_for_this_git_version() {
    let scenario = Scenario::new();
    for case in git::CASES {
        let run = scenario.run(case);
        let recorded = run.recording();
        if hearth_recordings::recording_on() {
            hearth_recordings::write("git", &scenario.version, case.name, &recorded);
        } else {
            assert_eq!(
                recorded,
                hearth_recordings::read("git", &scenario.version, case.name),
                "{}: capture with HEARTH_RECORD=1",
                case.name
            );
        }
    }
}
const EXIT_CASES: &[(&str, hearth_protocol_git::invocation::Exit)] = &[
    ("rev-parse-branch", hearth_protocol_git::invocation::Exit::Code(0)),
    ("rev-parse-commit", hearth_protocol_git::invocation::Exit::Code(0)),
    ("rev-parse-tag", hearth_protocol_git::invocation::Exit::Code(0)),
    ("rev-parse-missing", hearth_protocol_git::invocation::Exit::Code(1)),
    ("rev-parse-tree", hearth_protocol_git::invocation::Exit::Code(1)),
    ("rev-parse-sha256", hearth_protocol_git::invocation::Exit::Code(0)),
    ("ls-tree-modes", hearth_protocol_git::invocation::Exit::Code(0)),
    ("ls-tree-empty", hearth_protocol_git::invocation::Exit::Code(0)),
    ("ls-tree-sha256", hearth_protocol_git::invocation::Exit::Code(0)),
    ("cat-file-blobs", hearth_protocol_git::invocation::Exit::Code(0)),
    ("cat-file-missing", hearth_protocol_git::invocation::Exit::Code(0)),
    ("diff-line-changed", hearth_protocol_git::invocation::Exit::Code(1)),
    ("diff-line-added", hearth_protocol_git::invocation::Exit::Code(1)),
    ("diff-line-removed", hearth_protocol_git::invocation::Exit::Code(1)),
    ("diff-lines-appended", hearth_protocol_git::invocation::Exit::Code(1)),
    ("diff-no-newline", hearth_protocol_git::invocation::Exit::Code(1)),
    ("diff-binary", hearth_protocol_git::invocation::Exit::Code(1)),
    ("diff-past-200-lines", hearth_protocol_git::invocation::Exit::Code(1)),
];
#[test]
fn each_real_git_case_exits_as_its_operation_requires() {
    let scenario = Scenario::new();
    for (name, expected) in EXIT_CASES {
        let case = git::CASES.iter().find(|case| case.name == *name).expect("named case");
        let run = scenario.run(case);
        assert_eq!(run.exit, *expected, "{name}: {}", run.stderr_text());
    }
}

#[test]
fn real_git_s_hunks_are_the_unique_shortest_diff_of_each_change() {
    let scenario = Scenario::new();
    for case in git::CASES {
        if let git::Kind::Diff(change) = case.kind {
            let run = scenario.run(case);
            let world::DiffOutcome::Done(hunks) =
                world::diff(&run.recording().stdout, &limits::SHIPPED, &Settings::whole())
            else {
                panic!("{}: {}", case.name, run.stderr_text())
            };
            let lines: Vec<_> = hunks.lines.iter().map(|line| (line.kind, line.text.as_ref())).collect();
            assert_eq!(lines.as_slice(), change.lines, "{}", case.name);
            assert_eq!(hunks.left_out, change.left_out, "{}", case.name);
            assert_eq!(hunks.binary, change.binary, "{}", case.name);
        }
    }
}
