use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository};
use skein_io::kernel::Exit;

#[test]
fn hearth_render_prints_the_thin_repositorys_spec_through_real_git() {
    let repository = Repository::make("thin", fixtures::THIN);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), fixtures::THIN_TEXT, "render.md 9.2's lines");
    assert_eq!(rendered.exit, Exit::Code(0), "client/README.md 3.3: done");
}
#[test]
fn json_lines_give_the_version_first_and_the_outcome_last() {
    let repository = Repository::make("json", fixtures::THIN);
    let rendered = fixtures::render(&repository, &[], Form::JsonLines);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON text");
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.first().copied(), Some("{\"record\":\"version\",\"client\":1}"), "version first");
    assert_eq!(
        lines.last().copied(),
        Some("{\"record\":\"outcome\",\"outcome\":\"done\",\"left_out\":0}"),
        "terminal last"
    );
    assert_eq!(lines.len(), 5, "version, spec, config, meta and outcome");
}
#[test]
fn an_unknown_target_exits_64_after_the_listing() {
    let repository = Repository::make("unknown", fixtures::THIN);
    let rendered = fixtures::render(&repository, &[b"no-such-target"], Form::Text);
    assert_eq!(rendered.exit, Exit::Code(64), "unknown target is Usage");
    assert_eq!(rendered.stdout, b"", "no unknown spec");
}
#[test]
fn a_revision_git_cannot_resolve_exits_64() {
    let repository = Repository::make("revision", fixtures::THIN);
    let rendered = fixtures::render_at(&repository, "nope", &[], Form::Text);
    assert_eq!(rendered.exit, Exit::Code(64), "unresolved revision is Usage");
}
#[test]
fn a_directory_that_is_no_repository_exits_75() {
    let repository = Repository::empty("no-repository");
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(rendered.exit, Exit::Code(75), "git failed operationally");
}
#[test]
#[should_panic(expected = "git is required")]
fn a_scratch_repository_without_git_on_the_path_fails_by_name_not_by_skip() {
    let scratch = skein_scratch::Scratch::new("without-git");
    let _repository = Repository::make_on(scratch.path().as_os_str(), "no-git", fixtures::THIN);
}

const CACHED: &[(&str, &[u8])] = &[
    ("775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123", b"Welcome to x1.\n"),
    ("f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745", b"#!/bin/sh\nexec cat /etc/motd\n"),
];
#[test]
fn static_files_enter_the_cache_by_sha256_and_no_temporary_stays() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let repository = Repository::make("cache", fixtures::THIN);
    hearth::umask(0o077);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(rendered.exit, Exit::Code(0));
    let cache = repository.scratch().path().join("store/cache");
    assert_eq!(fs::metadata(&cache).expect("cache").permissions().mode() & 0o7777, 0o700);
    let mut names: Vec<_> =
        fs::read_dir(&cache).expect("cache listing").map(|entry| entry.expect("cache entry").file_name()).collect();
    names.sort();
    assert_eq!(
        names,
        [
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123",
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123.record",
            "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745",
            "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745.record"
        ]
    );
    for &(name, body) in CACHED {
        assert_eq!(fs::read(cache.join(name)).expect("cached bytes"), body, "{name}");
        assert_eq!(fs::metadata(cache.join(name)).expect("cache file").permissions().mode() & 0o7777, 0o644, "{name}");
    }
}

#[test]
fn hearth_render_gives_each_target_the_tasks_its_presence_layers_leave() {
    let repository = Repository::make("layers", &fixtures::thin_with(fixtures::LAYERS));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["target ", "  /"]),
        "target       x1-prod   base x1   environment prod\n  /etc/motd                file 0644 root:root 775324fcea7f\n  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\ntarget       x2-prod   base x2   environment prod\n"
    );
}
#[test]
fn presence_layers_render_successfully_through_real_git() {
    let repository = Repository::make("layers-exit", &fixtures::thin_with(fixtures::LAYERS));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(0));
}

#[test]
fn hearth_diff_names_each_changed_target_and_its_rebuild() {
    let repository = Repository::make("diff", fixtures::MOTD);
    let from = repository.head();
    let to = repository.next(&[fixtures::MOTD_KIND], &[]);
    let rendered = fixtures::diff(&repository, &fixtures::Asked::new(&from, &to));
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), fixtures::MOTD_DIFF_TEXT, "render.md 6.3");
}
#[test]
fn a_diff_with_changed_targets_exits_1() {
    let repository = Repository::make("diff-exit", fixtures::MOTD);
    let from = repository.head();
    let to = repository.next(&[fixtures::MOTD_KIND], &[]);
    assert_eq!(
        fixtures::diff(&repository, &fixtures::Asked::new(&from, &to)).exit,
        Exit::Code(1),
        "client/README.md 3.3"
    );
}
#[test]
fn a_diff_of_a_commit_with_itself_exits_0() {
    let repository = Repository::make("same", fixtures::MOTD);
    let commit = repository.head();
    assert_eq!(
        fixtures::diff(&repository, &fixtures::Asked::new(&commit, &commit)).exit,
        Exit::Code(0),
        "same spec digests"
    );
}
#[test]
fn a_named_target_neither_revision_has_exits_64() {
    let repository = Repository::make("diff-unknown", fixtures::MOTD);
    let commit = repository.head();
    let asked = fixtures::Asked { targets: &[b"unknown"], ..fixtures::Asked::new(&commit, &commit) };
    assert_eq!(fixtures::diff(&repository, &asked).exit, Exit::Code(64), "target absent on both sides");
}
#[test]
fn a_document_fixed_in_the_later_revision_fails_its_target_from() {
    let repository = Repository::make("fixed", fixtures::MOTD);
    let from = repository.next(&[fixtures::BAD_X1_LAB], &[]);
    let to = repository.next(&[fixtures::FIXED_X1_LAB], &[]);
    let asked = fixtures::Asked { form: Form::JsonLines, targets: &[b"x1-lab"], ..fixtures::Asked::new(&from, &to) };
    let rendered = fixtures::diff(&repository, &asked);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON");
    assert_eq!(
        stdout.lines().find(|line| line.starts_with("{\"record\":\"change\"")),
        Some("{\"record\":\"change\",\"target\":\"x1-lab\",\"kind\":\"failed\",\"failed\":{\"side\":\"from\"}}"),
        "the failed earlier document"
    );
}
#[test]
fn a_failure_beats_every_difference_and_exits_2() {
    let repository = Repository::make("failed-diff", fixtures::MOTD);
    let from = repository.head();
    let to = repository.next(&[fixtures::MOTD_KIND, fixtures::BAD_ION_LAB], &[]);
    assert_eq!(
        fixtures::diff(&repository, &fixtures::Asked::new(&from, &to)).exit,
        Exit::Code(2),
        "configuration failure outranks differences"
    );
}
#[test]
fn a_target_failing_in_the_later_revision_names_side_to() {
    let repository = Repository::make("failed-side", fixtures::MOTD);
    let from = repository.head();
    let to = repository.next(&[fixtures::MOTD_KIND, fixtures::BAD_ION_LAB], &[]);
    let asked = fixtures::Asked { form: Form::JsonLines, ..fixtures::Asked::new(&from, &to) };
    let rendered = fixtures::diff(&repository, &asked);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON");
    assert_eq!(
        stdout.lines().find(|line| line.starts_with("{\"record\":\"change\"")),
        Some("{\"record\":\"change\",\"target\":\"ion-lab\",\"kind\":\"failed\",\"failed\":{\"side\":\"to\"}}"),
        "the later document failed"
    );
}
#[test]
fn json_lines_give_each_targets_change_then_the_tally_in_the_outcome() {
    let repository = Repository::make("diff-json", fixtures::MOTD);
    let from = repository.head();
    let to = repository.next(&[fixtures::MOTD_KIND], &[]);
    let asked = fixtures::Asked { form: Form::JsonLines, ..fixtures::Asked::new(&from, &to) };
    let rendered = fixtures::diff(&repository, &asked);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON").replace(&from, "FROM").replace(&to, "TO");
    assert_eq!(
        stdout.lines().last(),
        Some(
            "{\"record\":\"outcome\",\"outcome\":\"negative\",\"from\":\"FROM\",\"to\":\"TO\",\"targets\":4,\"changed\":2,\"unchanged\":2,\"left_out\":0}"
        ),
        "client/README.md 3.2"
    );
    assert_eq!(
        stdout.lines().filter(|line| line.starts_with("{\"record\":\"change\"")).count(),
        4,
        "one change per target including same"
    );
}

#[test]
fn a_broken_document_prints_its_error_after_the_specs_that_rendered() {
    let repository = Repository::make("broken-lab", &fixtures::thin_with(fixtures::BROKEN_LAB));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        fixtures::BROKEN_LAB_TEXT
    );
}
#[test]
fn a_broken_document_exits_with_configuration_failure() {
    let repository = Repository::make("broken-lab-exit", &fixtures::thin_with(fixtures::BROKEN_LAB));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_broken_task_json_prints_its_error_with_its_task() {
    let repository = Repository::make("broken-task", &fixtures::broken_task());
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        "tasks/motd/task.json:1:10: motd: duplicate key \"a\" (tasks/motd/task.json:1:2)\n"
    );
}
#[test]
fn a_broken_task_json_exits_with_configuration_failure() {
    let repository = Repository::make("broken-task-exit", &fixtures::broken_task());
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_store_inside_a_work_tree_diffs_as_outside_any_repository() {
    let rendered = fixtures::diff_beneath_work_tree("diff-config", false);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["               @@"]),
        "               @@ -1,5 +1,5 @@\n               @@ -9,7 +9,7 @@ line 8\n"
    );
}
#[test]
fn a_work_trees_gitattributes_do_not_make_a_file_binary() {
    let rendered = fixtures::diff_beneath_work_tree("diff-attributes", true);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["               @@"]),
        "               @@ -1,5 +1,5 @@\n               @@ -9,7 +9,7 @@ line 8\n"
    );
}
#[test]
fn json_counts_the_hunk_lines_left_out_past_200() {
    let repository = Repository::make("diff-left-out", fixtures::MOTD);
    let from = repository.next(&[fixtures::File { body: b"old\n".repeat(300).leak(), ..fixtures::MOTD_KIND }], &[]);
    let to = repository.next(&[fixtures::File { body: b"new\n".repeat(300).leak(), ..fixtures::MOTD_KIND }], &[]);
    let asked = fixtures::Asked { form: Form::JsonLines, targets: &[b"x1-prod"], ..fixtures::Asked::new(&from, &to) };
    let rendered = fixtures::diff(&repository, &asked);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON");
    assert_eq!(stdout.matches("\"kind\":\"removed\"").count(), 199);
    assert!(stdout.contains("\"hunks\":{\"binary\":false,\"left_out\":401,\"lines\":["));
}
#[test]
fn a_binary_file_answers_binary_with_no_hunk_line() {
    let repository = Repository::make("diff-binary", fixtures::MOTD);
    let from = repository.next(&[fixtures::File { body: b"old\0\n", ..fixtures::MOTD_KIND }], &[]);
    let to = repository.next(&[fixtures::File { body: b"new\0\n", ..fixtures::MOTD_KIND }], &[]);
    let asked = fixtures::Asked { form: Form::JsonLines, targets: &[b"x1-prod"], ..fixtures::Asked::new(&from, &to) };
    let rendered = fixtures::diff(&repository, &asked);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON");
    assert!(stdout.contains("\"hunks\":{\"binary\":true,\"left_out\":0,\"lines\":[]}"));
}

#[test]
fn a_task_json_out_of_shape_prints_its_fault_and_fails_its_targets() {
    let repository = Repository::make("miskeyed", &fixtures::thin_with(&[fixtures::MISKEYED]));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        "tasks/motd/task.json:1:2: motd: no such key in a declaration\n"
    );
}
#[test]
fn a_task_json_out_of_shape_exits_with_configuration_failure() {
    let repository = Repository::make("miskeyed-exit", &fixtures::thin_with(&[fixtures::MISKEYED]));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn a_read_of_a_setting_its_task_lacks_prints_and_fails_its_targets() {
    let repository = Repository::make("misread", &fixtures::thin_with(fixtures::MISREAD));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        "tasks/motd/task.json:1:34: motd: reads core.domian, which core does not declare; core declares domain\n"
    );
}
#[test]
fn a_read_of_a_setting_its_task_lacks_exits_with_configuration_failure() {
    let repository = Repository::make("misread-exit", &fixtures::thin_with(fixtures::MISREAD));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn hearth_render_prints_256_errors_then_the_count() {
    let repository = fixtures::broken_envs(300);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    let stdout = fixtures::lines_starting(&rendered.stdout, &["envs/", "44 "]);
    let mut lines = stdout.lines();
    assert_eq!(lines.next(), Some("envs/e000.json:1:14: duplicate key \"motd\" (envs/e000.json:1:2)"));
    assert_eq!(lines.next_back(), Some("44 more errors"));
    assert_eq!(lines.next_back(), Some("envs/e255.json:1:14: duplicate key \"motd\" (envs/e255.json:1:2)"));
    assert_eq!(lines.count(), 254);
}
#[test]
fn hearth_render_with_300_broken_documents_exits_with_configuration_failure() {
    let repository = fixtures::broken_envs(300);
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn hearth_render_json_gives_left_out_44() {
    let repository = fixtures::broken_envs(300);
    let rendered = fixtures::render(&repository, &[], Form::JsonLines);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout).lines().last(),
        Some("{\"record\":\"outcome\",\"outcome\":\"failed\",\"left_out\":44}")
    );
}
