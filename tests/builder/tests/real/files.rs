use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository, files};
use skein_io::kernel::Exit;
#[test]
fn hearth_render_lists_a_link_a_directory_and_a_files_new_mode() {
    let repository = Repository::make("links", &fixtures::thin_with(files::LINKS));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  /"]),
        concat!(
            "  /etc/motd                file 0600 root:adm 775324fcea7f\n",
            "  /etc/mtab                link /proc/self/mounts root:root\n",
            "  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\n",
            "  /var/lib/motd            directory 0750 root:root\n"
        )
    );
}
#[test]
fn hearth_diff_shows_a_moved_link_and_a_directory_s_new_mode() {
    assert_eq!(
        fixtures::lines_starting(&files::diff_links().stdout, &["  config "]),
        concat!(
            "  config   ~ /etc/mtab                              link /proc/mounts root:root\n",
            "  config   ~ /var/lib/motd                          directory 0755 root:root\n"
        )
    );
}
#[test]
fn a_moved_link_and_directory_exit_with_a_difference() {
    assert_eq!(files::diff_links().exit, Exit::Code(1));
}
