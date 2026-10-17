use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository, data};
use skein_io::kernel::Exit;
#[test]
fn hearth_render_lists_the_data_entries() {
    let repository = Repository::make("data", &fixtures::thin_with(&[data::DATA]));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  iwd ", "  journal "]),
        "  iwd        /var/lib/iwd\n  journal    /var/log/journal\n"
    );
}
#[test]
fn hearth_diff_names_a_data_entry_moved_to_another_class() {
    assert_eq!(
        fixtures::lines_starting(&data::diff_data().stdout, &["  data "]),
        "  data     ~ iwd                                    data /var/lib/iwd\n"
    );
}
#[test]
fn moved_data_exits_with_a_difference() {
    assert_eq!(data::diff_data().exit, Exit::Code(1));
}
