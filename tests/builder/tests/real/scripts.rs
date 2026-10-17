use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository, scripts};
use skein_io::kernel::Exit;
#[test]
fn hearth_render_lists_presets_tmpfiles_and_scripts() {
    let repository = Repository::make("scripts", &fixtures::thin_with(scripts::PRESETS));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  presets ", "  tmpfiles ", "  scripts "]),
        concat!(
            "  presets    getty@tty1.service disable\n",
            "  presets    ssh.service enable\n",
            "  tmpfiles   /var/log/journal d 2755 root systemd-journal\n",
            "  scripts    motd 300 b40d08aa8c5c\n",
        )
    );
}
#[test]
fn hearth_diff_names_a_preset_a_tmpfiles_line_and_a_script() {
    assert_eq!(
        fixtures::lines_starting(&scripts::diff_scripts().stdout, &["  config "]),
        concat!(
            "  config   ~ presets ssh.service                    disable\n",
            "  config   ~ tmpfiles /var/log/journal              d 0755 root systemd-journal\n",
            "  config   ~ scripts                                motd 300 4b1f03482366\n",
        )
    );
}
#[test]
fn moved_scripts_exit_with_a_difference() {
    assert_eq!(scripts::diff_scripts().exit, Exit::Code(1));
}
