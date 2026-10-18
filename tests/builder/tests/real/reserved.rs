use hearth_builder_world::fixtures::reserved as f;
use skein_io::kernel::Exit;
#[test]
fn a_reserved_command_line_key_prints_as_render_md_shows() {
    assert_eq!(f::rendered(&f::WAIT).stdout, b"tasks/motd/task.json:1:23: x1-prod: motd: \"rootwait\" is reserved: tiny-initramfs would wait for ever for a missing root (machine.md, 6.2)\n");
}
#[test]
fn a_reserved_command_line_key_exits_2() {
    assert_eq!(f::rendered(&f::WAIT).exit, Exit::Code(2));
}
#[test]
fn a_password_preseed_prints_reserved_and_not_an_unlisted_type() {
    assert_eq!(f::rendered(&f::PASSWORD).stdout, b"tasks/motd/task.json:1:23: x1-prod: motd: \"password\" is reserved: a password preseed leaves its value in /var/cache/debconf/passwords.dat (format.md, 5.11)\n");
}
#[test]
fn a_password_preseed_exits_2() {
    assert_eq!(f::rendered(&f::PASSWORD).exit, Exit::Code(2));
}
#[test]
fn a_line_past_2048_bytes_names_the_parameter_that_crosses_it() {
    let rendered = f::long_line();
    assert!(rendered.stdout.starts_with(b"tasks/motd/task.json:1:1040: x1-prod: motd: "));
    assert!(String::from_utf8_lossy(&rendered.stdout).contains("the command line is 2141 bytes"));
}
#[test]
fn a_line_past_2048_bytes_exits_2() {
    assert_eq!(f::long_line().exit, Exit::Code(2));
}
