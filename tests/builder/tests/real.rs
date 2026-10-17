use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository};
use skein_io::kernel::Exit;

#[test]
fn hearth_render_prints_the_thin_repositorys_spec_through_real_git() {
    let repository = Repository::make("thin", fixtures::THIN);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(&*rendered.stdout, b"target       x1-prod   base x1   environment prod\nplacement    metal efi 7th-gen\nspec         d5942fa1a047\n", "render.md 9.2's lines");
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
    assert_eq!(lines.len(), 4, "version, spec, meta and outcome");
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
