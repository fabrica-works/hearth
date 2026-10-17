use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository};
use skein_io::kernel::Exit;
#[test]
fn hearth_render_seeds_the_owners_of_what_persists() {
    let repository =
        Repository::make("secrets-fixed", &fixtures::thin_with(core::slice::from_ref(&fixtures::ids::SECRETS_FIXED)));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(fixtures::lines_starting(&rendered.stdout, &["  seed "]), "  seed       group opendkim 120\n");
}
#[test]
fn an_owner_with_no_fixed_id_prints_its_fault_at_the_owner() {
    assert_eq!(fixtures::ids::no_id().stdout, b"tasks/motd/task.json:1:71: x1-prod: motd: postfix owns mail/tls/mx1.key and has no fixed id: no user postfix is declared\n");
}
#[test]
fn an_owner_with_no_fixed_id_exits_2() {
    assert_eq!(fixtures::ids::no_id().exit, Exit::Code(2));
}
