use hearth_builder_world::fixtures::reserved as f;
use skein_io::kernel::Exit;
#[test]
fn a_reserved_command_line_key_prints_as_render_md_shows() {
    assert_eq!(f::rendered(&f::WAIT).stdout, b"tasks/motd/task.json:1:23: x1-prod: motd: \"rootwait\" is reserved: tiny-initramfs would wait for ever for a missing root (domain/machine.md, 6.2)\n");
}
#[test]
fn a_reserved_command_line_key_exits_2() {
    assert_eq!(f::rendered(&f::WAIT).exit, f::CONFIGURATION_EXIT);
}
#[test]
fn a_password_preseed_prints_reserved_and_not_an_unlisted_type() {
    assert_eq!(f::rendered(&f::PASSWORD).stdout, b"tasks/motd/task.json:1:23: x1-prod: motd: \"password\" is reserved: a password preseed leaves its value in /var/cache/debconf/passwords.dat (protocol/format.md, 5.11)\n");
}
#[test]
fn a_password_preseed_exits_2() {
    assert_eq!(f::rendered(&f::PASSWORD).exit, f::CONFIGURATION_EXIT);
}
#[test]
fn a_line_past_2048_bytes_names_the_parameter_that_crosses_it() {
    let rendered = f::long_line();
    assert!(rendered.stdout.starts_with(b"tasks/motd/task.json:1:1040: x1-prod: motd: "));
    assert!(String::from_utf8_lossy(&rendered.stdout).contains("the command line is 2141 bytes"));
}
#[test]
fn a_line_past_2048_bytes_exits_2() {
    assert_eq!(f::long_line().exit, f::CONFIGURATION_EXIT);
}
#[test]
fn hearth_render_refuses_a_secrets_path_in_the_tree() {
    assert_eq!(f::rendered_files(f::KEY_IN_TREE).stdout, b"tasks/motd/task.json:1:12: x1-prod: motd: /etc/ssl/private/mx1.key is a secret's path; declare it under secrets (protocol/format.md, 9.1)\n");
}
#[test]
fn a_file_only_the_tree_gives_is_refused_at_its_path() {
    assert_eq!(f::rendered(&f::FSTAB).stdout, b"tasks/motd/tree/etc/fstab: x1-prod: motd: \"/etc/fstab\" is reserved: the activator writes this path at prepare (protocol/format.md, 5.11)\n");
}
#[test]
fn a_data_path_beside_its_class_mount_is_refused_at_its_path() {
    assert_eq!(f::rendered(&f::DATA_PATH).stdout, b"tasks/motd/task.json:1:20: x1-prod: motd: \"/srv/data/spool\" is reserved: the class mount hides this path (protocol/format.md, 5.7)\n");
}
const PATH_EXITS: &[(&[hearth_builder_world::fixtures::File], Exit)] = &[
    (f::KEY_IN_TREE, f::CONFIGURATION_EXIT),
    (core::slice::from_ref(&f::FSTAB), f::CONFIGURATION_EXIT),
    (core::slice::from_ref(&f::DATA_PATH), f::CONFIGURATION_EXIT),
];
#[test]
fn reserved_paths_exit_2() {
    for (row, &(files, expected)) in PATH_EXITS.iter().enumerate() {
        assert_eq!(f::rendered_files(files).exit, expected, "row {row}");
    }
}
#[test]
fn a_secret_owned_by_hearth_is_refused_at_its_owner() {
    assert_eq!(f::rendered(&f::SECRET_HEARTH).stdout, b"tasks/motd/task.json:1:71: x1-prod: motd: \"mail/tls/mx1.key\" is reserved: hearth must not be able to read this secret or identity file (protocol/format.md, 9.1)\n");
}
#[test]
fn a_never_live_unit_is_refused_at_its_unit() {
    assert_eq!(f::rendered(&f::LIVE_NEVER).stdout, b"tasks/motd/task.json:1:19: x1-prod: motd: \"systemd-networkd.service\" is reserved: this core service must not restart live (protocol/format.md, 5.14)\n");
}
#[test]
fn a_unit_with_none_is_refused_at_its_unit() {
    assert_eq!(f::rendered(&f::LIVE_NONE).stdout, b"tasks/motd/task.json:1:18: x1-prod: motd: \"x.service\" is reserved: action none names no unit (protocol/format.md, 5.14)\n");
}
const SECRET_LIVE_EXITS: &[(&hearth_builder_world::fixtures::File, Exit)] = &[
    (&f::SECRET_HEARTH, f::CONFIGURATION_EXIT),
    (&f::LIVE_NEVER, f::CONFIGURATION_EXIT),
    (&f::LIVE_NONE, f::CONFIGURATION_EXIT),
];
#[test]
fn reserved_secrets_and_live_entries_exit_2() {
    for (row, &(file, expected)) in SECRET_LIVE_EXITS.iter().enumerate() {
        assert_eq!(f::rendered(file).exit, expected, "row {row}");
    }
}

#[path = "reserved/hearth.rs"]
mod hearth;
