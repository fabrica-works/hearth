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
            "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745"
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
        stdout.lines().nth(1),
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
        stdout.lines().nth(1),
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
