use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository};
use skein_io::kernel::Exit;

#[test]
fn hearth_diff_names_each_changed_target_and_its_rebuild() {
    let repository = Repository::make("diff", fixtures::MOTD);
    let from = repository.head();
    let to = repository.next(&[fixtures::MOTD_KIND], &[]);
    let rendered = fixtures::diff(&repository, &fixtures::Asked::new(&from, &to));
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), fixtures::MOTD_DIFF_TEXT, "domain/render.md 6.3");
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
