use crate::{
    errors::Rule,
    reserved::{Shape, Spot},
    revision::declaration::{Map, Name},
    tests::fixtures::{self, reserved as f},
};
const PATHS: &[(&[u8], &[Rule])] = &[
    (b"/usr", &[Rule::PackagesLayer]),
    (b"/usr/local/x", &[]),
    (b"/usr/localx", &[Rule::PackagesLayer]),
    (b"/var/lib/hearth", &[Rule::HearthState]),
    (b"/var/lib/hearthx", &[]),
    (b"/srv/core", &[Rule::ClassMount]),
    (b"/srv/www", &[]),
    (b"/etc/systemd/system.conf.d/zz-hearth-watchdog.conf", &[Rule::Dropin]),
    (b"/etc/systemd/system.conf.d/zz-hearth-a.conf", &[Rule::Dropin]),
    (b"/etc/systemd/system.conf.d/zz-z.conf", &[Rule::Dropin]),
    (b"/etc/systemd/system.conf.d/zz-a.conf", &[]),
    (b"/etc/systemd/system/hearth-trial.service", &[Rule::TrialName]),
    (b"/etc/systemd/system/hearth-check-a.service.d", &[Rule::TrialName]),
    (b"/opt/hearth-trial/x", &[]),
    (b"/etc/fstab", &[Rule::ActivatorFile]),
    (b"/.hearth/slot", &[Rule::ActivatorFile]),
    (b"/etc/shadow", &[Rule::Accounts]),
    (b"/var/lib/dbus/machine-id", &[Rule::Accounts]),
    (b"/etc/ssh/ssh_host_ed25519_key.pub", &[Rule::SecretPath]),
    (b"/etc/dropbear/dropbear_rsa_host_key", &[Rule::SecretPath]),
    (b"/etc/tiny-initramfs/tiny-initramfs.conf", &[Rule::InitramfsConf]),
    (b"/etc/extensions/a", &[Rule::Overlay]),
    (b"/usr/local/lib/confexts/a", &[Rule::Overlay]),
];
#[test]
fn a_table_of_paths_each_row_refuses_and_its_neighbour_does_not() {
    for (row, &(path, expected)) in PATHS.iter().enumerate() {
        assert_eq!(f::tree_rules(path, Shape::File { empty: false }).as_ref(), expected, "row {row}");
    }
}
#[test]
fn a_secrets_path_in_the_tree_is_refused_naming_the_secrets_map() {
    let refused = f::tree(b"/etc/ssl/private/mx1.key", Shape::File { empty: false }, Some(fixtures::FIELD_AT));
    assert_eq!(f::rules(&refused).as_ref(), &[Rule::SecretPath]);
    assert_eq!(fixtures::only(&refused).spot, Spot::Declared(fixtures::FIELD_AT));
}
#[test]
fn a_tree_file_with_no_entry_is_spotted_at_its_tree_path() {
    assert_eq!(fixtures::only(&f::tree(b"/etc/fstab", Shape::File { empty: false }, None)).spot, Spot::Tree);
}
#[test]
fn a_download_under_usr_reports_packages_layer_not_beneath() {
    assert_eq!(
        f::word_rules(Map::Software, b"x", &[(Name::Path, b"/usr/bin/x")], false).as_ref(),
        &[Rule::PackagesLayer]
    );
}
const LOGS: &[(Shape, &[Rule])] = &[
    (Shape::File { empty: true }, &[]),
    (Shape::File { empty: false }, &[Rule::LogFile]),
    (Shape::Directory, &[]),
    (Shape::Link, &[]),
];
#[test]
fn an_empty_log_file_is_allowed_and_a_non_empty_one_is_not() {
    for (row, &(shape, expected)) in LOGS.iter().enumerate() {
        assert_eq!(f::tree_rules(b"/var/log/x", shape).as_ref(), expected, "row {row}");
    }
}
const KINDS: &[f::EntryPathCase<'_>] = &[
    (Map::Software, b"x", b"/etc/shadow", &[Rule::Accounts]),
    (Map::Data, b"x", b"/etc/fstab", &[Rule::ActivatorFile]),
    (Map::Secrets, b"x", b"/etc/shadow", &[Rule::Accounts]),
    (Map::Identity, b"x", b"/etc/fstab", &[Rule::ActivatorFile]),
    (Map::Secrets, b"x", b"/etc/ssl/private/x", &[]),
    (Map::Identity, b"x", b"/etc/ssh/ssh_host_rsa_key", &[]),
    (Map::Data, b"x", b"/var/lib/dbus/machine-id", &[Rule::Accounts]),
    (Map::Software, b"x", b"/etc", &[Rule::ActivatorFile]),
    (Map::Secrets, b"x", b"/etc/ssh", &[Rule::SecretPath]),
];
#[test]
fn every_entry_kind_obeys_the_same_paths_but_secrets_use_secret_paths() {
    for (row, &(map, key, path, expected)) in KINDS.iter().enumerate() {
        assert_eq!(f::word_rules(map, key, &[(Name::Path, path)], false).as_ref(), expected, "row {row}");
    }
}
const DATA: &[f::DataPathCase<'_>] = &[
    (b"mail", b"data", b"/srv/data/mail", &[]),
    (b"mail", b"core", b"/srv/core/mail", &[]),
    (b"mail", b"data", b"/srv/data/spool", &[Rule::ClassMount]),
    (b"mail", b"core", b"/srv/data/mail", &[Rule::ClassMount]),
];
#[test]
fn a_data_path_may_be_its_classs_directory_and_no_other_beneath_a_mount() {
    for (row, &(key, class, path, expected)) in DATA.iter().enumerate() {
        assert_eq!(f::data_rules(key, class, path).as_ref(), expected, "row {row}");
    }
}
const DATA_ROOTS: &[f::DataPathCase<'_>] = &[
    (b"mail", b"data", b"/home", &[Rule::DataRoot]),
    (b"mail", b"data", b"/srv", &[Rule::DataRoot]),
    (b"mail", b"data", b"/var", &[Rule::DataRoot]),
    (b"mail", b"data", b"/home/mail", &[]),
    (b"mail", b"data", b"/srv/mail", &[]),
    (b"mail", b"data", b"/var/mail", &[]),
    (b"mail", b"core", b"/var/log/journal", &[]),
    (b"mail", b"data", b"/opt/mail", &[Rule::DataRoot]),
];
#[test]
fn data_roots_are_strictly_beneath_home_srv_or_var() {
    for (row, &(key, class, path, expected)) in DATA_ROOTS.iter().enumerate() {
        assert_eq!(f::data_rules(key, class, path).as_ref(), expected, "row {row}");
    }
}
const TMPFILES: &[f::TmpfilePathCase<'_>] = &[
    (b"/dev/x", &[(Name::Type, b"f")], &[Rule::Runtime]),
    (b"/proc/x", &[(Name::Type, b"f")], &[Rule::Runtime]),
    (b"/sys/x", &[(Name::Type, b"f")], &[Rule::Runtime]),
    (b"/run/x", &[(Name::Type, b"f")], &[]),
    (b"/tmp/x", &[(Name::Type, b"f")], &[]),
    (b"/var/tmp/x", &[(Name::Type, b"f")], &[]),
];
#[test]
fn tmpfiles_only_leave_runtime_rows_under_run_and_tmp_alone() {
    for (row, &(path, fields, expected)) in TMPFILES.iter().enumerate() {
        assert_eq!(f::word_rules(Map::Tmpfiles, path, fields, false).as_ref(), expected, "row {row}");
    }
}
const TMPFILE_DIRECTORIES: &[f::TmpfilePathCase<'_>] = &[
    (b"/etc/ssl", &[(Name::Type, b"D")], &[Rule::SecretPath]),
    (b"/etc/ssl", &[(Name::Type, b"d"), (Name::Age, b"1d")], &[Rule::SecretPath]),
    (b"/etc/ssl", &[(Name::Type, b"d")], &[]),
    (b"/etc/ssl", &[(Name::Type, b"e")], &[]),
    (b"/etc/ssl", &[(Name::Type, b"z")], &[]),
    (b"/var/lib/systemd/random-seed", &[(Name::Type, b"f")], &[Rule::SecretPath]),
];
#[test]
fn tmpfiles_only_leave_directories_above_reserved_paths() {
    for (row, &(path, fields, expected)) in TMPFILE_DIRECTORIES.iter().enumerate() {
        assert_eq!(f::word_rules(Map::Tmpfiles, path, fields, false).as_ref(), expected, "row {row}");
    }
}
const SHAPES: &[(Shape, &[u8], &[Rule])] = &[
    (Shape::Directory, b"/etc/ssh", &[]),
    (Shape::File { empty: true }, b"/etc/ssh", &[Rule::SecretPath]),
    (Shape::Link, b"/etc/systemd/system/hearth-trial.service", &[Rule::TrialName]),
    (Shape::Directory, b"/etc/systemd/system/hearth-trial.service.d", &[Rule::TrialName]),
    (Shape::Link, b"/etc/systemd/system/x.wants/systemd-sysext.service", &[Rule::Overlay]),
    (Shape::File { empty: true }, b"/etc/systemd/system/x.wants/systemd-sysext.service", &[]),
];
#[test]
fn installed_shapes_obey_the_rows_directory_and_link_rules() {
    for (row, &(shape, path, expected)) in SHAPES.iter().enumerate() {
        assert_eq!(f::tree_rules(path, shape).as_ref(), expected, "row {row}");
    }
}
