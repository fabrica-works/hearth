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
    assert_eq!(fixtures::lines_starting(&rendered.stdout, &["git "]), "git rev-parse exited 128\n");
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
    let repository = Repository::make("miskeyed", &fixtures::thin_with(core::slice::from_ref(&fixtures::MISKEYED)));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        "tasks/motd/task.json:1:2: motd: no such key in a declaration\n"
    );
}
#[test]
fn a_task_json_out_of_shape_exits_with_configuration_failure() {
    let repository =
        Repository::make("miskeyed-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::MISKEYED)));
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

#[test]
fn a_default_without_its_type_prints_and_fails_its_targets() {
    let repository = Repository::make("mistyped", &fixtures::thin_with(&[fixtures::MISTYPED]));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        "tasks/motd/task.json:1:54: motd: not an int\n"
    );
}
#[test]
fn a_default_without_its_type_exits_with_configuration_failure() {
    let repository = Repository::make("mistyped-exit", &fixtures::thin_with(&[fixtures::MISTYPED]));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_tree_git_cannot_read_prints_git_s_tail_and_exits_75() {
    let repository = Repository::make("broken-tree", fixtures::THIN);
    repository.remove_object(&repository.git(&["rev-parse", "HEAD^{tree}"]));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), "git ls-tree exited 128\n  fatal: not a tree object\n");
    assert_eq!(rendered.exit, Exit::Code(75));
}
#[test]
fn a_stray_file_in_a_task_prints_a_repository_error() {
    let repository = Repository::make("stray", &fixtures::thin_with(core::slice::from_ref(&fixtures::STRAY)));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        "tasks/motd/tre/etc/motd: motd: a path format.md 2.1 does not allow\n"
    );
    assert_eq!(rendered.exit, Exit::Code(2));
}

#[test]
fn a_template_that_does_not_compile_prints_its_decode_error_at_its_line() {
    let repository =
        Repository::make("broken-template", &fixtures::thin_with(core::slice::from_ref(&fixtures::BROKEN_TEMPLATE)));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        "tasks/motd/tree/etc/issue.j2:2:1: motd: a tag with no end\n"
    );
}
#[test]
fn a_template_that_does_not_compile_leaves_its_target_out() {
    let repository = Repository::make(
        "broken-template-exit",
        &fixtures::thin_with(core::slice::from_ref(&fixtures::BROKEN_TEMPLATE)),
    );
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn the_diffs_errors_follow_its_changes_and_precede_the_outcome() {
    let repository = Repository::make("diff-error-order", fixtures::MOTD);
    let from = repository.head();
    let to = repository.next(&[fixtures::MOTD_KIND, fixtures::BAD_ION_LAB], &[]);
    let asked = fixtures::Asked { form: Form::JsonLines, ..fixtures::Asked::new(&from, &to) };
    let rendered = fixtures::diff(&repository, &asked);
    let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON text");
    let mut lines = stdout.lines();
    assert_eq!(lines.next_back().and_then(|line| line.split(',').next()), Some("{\"record\":\"outcome\""));
    assert_eq!(
        lines.next_back(),
        Some(
            "{\"record\":\"error\",\"side\":\"to\",\"kind\":\"decode\",\"decode\":\"unexpected\",\"class\":\"configuration\",\"at\":{\"column\":2,\"line\":1,\"path\":\"hosts/ion/ion-lab.json\"}}"
        )
    );
    assert_eq!(lines.filter(|line| line.starts_with("{\"record\":\"change\"")).count(), 4);
}
#[test]

fn a_cluster_s_members_render_as_targets_with_their_own_layers() {
    let repository = Repository::make("cluster", &fixtures::cluster());
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["target ", "placement "]),
        concat!(
            "target       stoa-1-lab   base stoa   member 1   environment lab\n",
            "placement    vm efi incus\n",
            "target       stoa-2-lab   base stoa   member 2   environment lab\n",
            "placement    vm bios pool\n",
        )
    );
    assert_eq!(rendered.exit, Exit::Code(0));
}

#[test]
fn an_account_s_faults_print_at_their_lines_in_the_base() {
    let repository = Repository::make("users-schema", &fixtures::thin_with(fixtures::USERS));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        concat!(
            "hosts/x1/x1.json:2:16: x1-prod: users: not an int\n",
            "hosts/x1/x1.json:3:22: x1-prod: users: undeclared field \"shel\"; the object declares groups, password, shell, uid\n",
            "hosts/x1/x1.json:4:7: x1-prod: users: missing required field uid\n",
        )
    );
}
#[test]
fn an_account_s_faults_exit_with_configuration_failure() {
    let repository = Repository::make("users-schema-exit", &fixtures::thin_with(fixtures::USERS));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn a_container_on_another_environment_s_target_prints_its_error() {
    let repository = Repository::make("containers", fixtures::CONTAINERS);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["target ", "hosts/"]),
        concat!(
            "target       rhi-lab   base rhi   environment lab\n",
            "target       rhi-prod   base rhi   environment prod\n",
            "hosts/chat/chat-prod.json:1:43: chat-prod: placement's on names rhi-lab, a target of lab\n",
        )
    );
}
#[test]
fn a_container_on_another_environment_s_target_exits_two() {
    let repository = Repository::make("containers-exit", fixtures::CONTAINERS);
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn hearth_render_lists_a_rendered_template_by_its_sha256() {
    let repository = Repository::make("templated", &fixtures::thin_with(fixtures::TEMPLATED));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  /etc/hostname", "  /etc/issue"]),
        "  /etc/hostname            file 0644 root:root 21c50adeb849\n  /etc/issue               file 0644 root:root e75f92b4b841\n"
    );
}
#[test]
fn a_rendered_template_leaves_the_call_successful() {
    let repository = Repository::make("templated-exit", &fixtures::thin_with(fixtures::TEMPLATED));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(0));
}
#[test]
fn a_missing_key_in_a_template_prints_its_evaluation_error() {
    let repository = Repository::make("template-key", &fixtures::thin_with(&[fixtures::MISSING_TEMPLATE_KEY]));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/tree/etc/issue.j2:1:11: x1-prod: motd: a missing key\n"
    );
}
#[test]
fn a_missing_key_in_a_template_fails_the_call() {
    let repository = Repository::make("template-key-exit", &fixtures::thin_with(&[fixtures::MISSING_TEMPLATE_KEY]));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_template_prints_the_default_its_setting_was_filled_with() {
    let repository = Repository::make("template-default", &fixtures::thin_with(fixtures::GREET));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  /etc/greeting"]),
        "  /etc/greeting            file 0644 root:root 5891b5b522d5\n"
    );
}
#[test]
fn a_template_with_its_default_setting_leaves_the_call_successful() {
    let repository = Repository::make("template-default-exit", &fixtures::thin_with(fixtures::GREET));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(0));
}

#[test]
fn a_missing_file_and_target_print_where_the_base_sets_them() {
    let repository = Repository::make("references-schema", &fixtures::thin_with(fixtures::REFERENCES));
    assert_eq!(
        String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout),
        concat!(
            "hosts/x1/x1.json:1:28: x1-prod: net: no file lab-ca/lab.crt at this revision\n",
            "hosts/x1/x1.json:1:54: x1-prod: net: no target x9-prod in prod\n",
        )
    );
}
#[test]
fn missing_file_and_target_references_exit_with_configuration_failure() {
    let repository = Repository::make("references-schema-exit", &fixtures::thin_with(fixtures::REFERENCES));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn rendered_files_enter_the_cache_by_sha256() {
    let repository = Repository::make("template-cache", &fixtures::thin_with(fixtures::TEMPLATED));
    drop(fixtures::render(&repository, &[], Form::Text));
    let cache = repository.scratch().path().join("store/cache");
    for (row, &(name, bytes)) in fixtures::RENDERED_FILES.iter().enumerate() {
        assert_eq!(std::fs::read(cache.join(name)).expect("cached rendering"), bytes, "row {row}");
    }
}
#[test]
fn rendered_cache_files_have_mode_0644() {
    use std::os::unix::fs::PermissionsExt;
    let repository = Repository::make("template-cache-modes", &fixtures::thin_with(fixtures::TEMPLATED));
    drop(fixtures::render(&repository, &[], Form::Text));
    let cache = repository.scratch().path().join("store/cache");
    for (row, &(name, _)) in fixtures::RENDERED_FILES.iter().enumerate() {
        assert_eq!(
            std::fs::metadata(cache.join(name)).expect("cache file").permissions().mode() & 0o7777,
            0o644,
            "row {row}"
        );
    }
}
#[test]
fn rendered_cache_files_leave_no_temporary() {
    let repository = Repository::make("template-cache-temporary", &fixtures::thin_with(fixtures::TEMPLATED));
    drop(fixtures::render(&repository, &[], Form::Text));
    let cache = repository.scratch().path().join("store/cache");
    assert_eq!(
        fixtures::names(&cache).into_iter().filter(|name| name.as_bytes().ends_with(b".tmp")).collect::<Vec<_>>(),
        Vec::<String>::new()
    );
}

#[test]
fn a_target_past_its_rendered_bytes_prints_a_limit_error() {
    let repository = Repository::make("template-target-limit", &fixtures::thin_with(fixtures::TEMPLATED));
    let rendered = fixtures::render_with(&repository, &fixtures::target_output_limit());
    assert_eq!(rendered.stdout, b"tasks/motd/tree/etc/issue.j2:1:39: x1-prod: motd: a limit exceeded\n");
}
#[test]
fn a_target_past_its_rendered_bytes_fails_the_call() {
    let repository = Repository::make("template-target-limit-exit", &fixtures::thin_with(fixtures::TEMPLATED));
    assert_eq!(fixtures::render_with(&repository, &fixtures::target_output_limit()).exit, Exit::Code(2));
}

#[test]
fn a_false_guard_prints_its_message_at_its_key_and_leaves_its_target_out() {
    let repository = Repository::make("guard", &fixtures::thin_with(core::slice::from_ref(&fixtures::GUARDED)));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/task.json:3:5: x1-prod: motd: requires greeting: set a greeting of your own\n"
    );
}
#[test]
fn a_false_guard_exits_with_configuration_failure() {
    let repository = Repository::make("guard-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::GUARDED)));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_guard_the_settings_satisfy_leaves_the_thin_spec_as_it_was() {
    let repository =
        Repository::make("guard-satisfied", &fixtures::thin_with(&[fixtures::GUARDED, fixtures::GREETING]));
    assert_eq!(String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout), fixtures::THIN_TEXT);
}
#[test]
fn a_target_with_a_schema_fault_and_a_false_guard_prints_the_schema_fault_only() {
    let repository =
        Repository::make("guard-schema", &fixtures::thin_with(&[fixtures::GUARDED, fixtures::WRONG_GREETING]));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"hosts/x1/x1.json:1:21: x1-prod: motd: not a string\n"
    );
}

#[test]
fn a_for_each_making_a_key_twice_prints_a_collision_at_its_entry() {
    let repository = Repository::make("twice", &fixtures::thin_with(core::slice::from_ref(&fixtures::TWICE)));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/task.json:3:5: x1-prod: motd: the key \"ops\" is made twice (tasks/motd/task.json:3:5)\n"
    );
}
#[test]
fn a_for_each_making_a_key_twice_exits_with_configuration_failure() {
    let repository = Repository::make("twice-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::TWICE)));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn distinct_for_each_keys_render_with_no_error() {
    let repository =
        Repository::make("distinct-keys", &fixtures::thin_with(&[fixtures::TWICE, fixtures::DISTINCT_TEAMS]));
    assert_eq!(String::from_utf8_lossy(&fixtures::render(&repository, &[], Form::Text).stdout), fixtures::THIN_TEXT);
}
#[test]
fn distinct_for_each_keys_leave_the_call_successful() {
    let repository =
        Repository::make("distinct-keys-exit", &fixtures::thin_with(&[fixtures::TWICE, fixtures::DISTINCT_TEAMS]));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(0));
}

#[test]
fn an_undefined_value_beside_text_fails_while_a_lone_tag_leaves_the_field_absent() {
    let repository = Repository::make("unset-field", &fixtures::thin_with(core::slice::from_ref(&fixtures::UNSET)));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/task.json:4:21: x1-prod: motd: an undefined value used\n"
    );
}
#[test]
fn an_undefined_value_beside_text_exits_with_configuration_failure() {
    let repository =
        Repository::make("unset-field-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::UNSET)));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_boolean_in_a_string_field_prints_an_evaluation_error_at_the_field() {
    let repository =
        Repository::make("boolean-field", &fixtures::thin_with(core::slice::from_ref(&fixtures::BOOLEAN_FIELD)));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/task.json:3:21: x1-prod: motd: a value of the wrong type\n"
    );
}

#[test]
fn a_typo_in_a_settings_reference_prints_the_declared_names() {
    let repository = Repository::make("typo-reference", &fixtures::thin_with(core::slice::from_ref(&fixtures::TYPO)));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/task.json:3:21: motd: undeclared setting \"greting\"; motd declares greeting\n"
    );
}
#[test]
fn a_typo_in_a_settings_reference_exits_with_configuration_failure() {
    let repository =
        Repository::make("typo-reference-exit", &fixtures::thin_with(core::slice::from_ref(&fixtures::TYPO)));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn a_typo_in_a_branch_no_run_takes_prints_at_its_line() {
    let repository = Repository::make("typo-branch", &fixtures::thin_with(fixtures::TYPO_BRANCH));
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/motd/tree/etc/issue.j2:1:27: motd: undeclared setting \"greting\"; motd declares greeting\n"
    );
}
#[test]
fn a_typo_in_an_untaken_branch_exits_with_configuration_failure() {
    let repository = Repository::make("typo-branch-exit", &fixtures::thin_with(fixtures::TYPO_BRANCH));
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}

#[test]
fn a_hosts_template_lists_the_containers_placed_on_it() {
    let rendered = fixtures::contained();
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  /etc/containers"]),
        concat!(
            "  /etc/containers          file 0644 root:root 01ba4719c80b\n",
            "  /etc/containers          file 0644 root:root 9e40aaeae922\n",
            "  /etc/containers          file 0644 root:root 755c9769e418\n",
        ),
        "format.md, 3.5: chat none, lab mail, prod chat and mail"
    );
}
#[test]
fn a_hosts_containers_leave_the_render_successful() {
    assert_eq!(fixtures::contained().exit, Exit::Code(0));
}
