use crate::{
    errors::{Place, Rule},
    reserved::{self, Shape},
    revision::declaration::{Map, Name},
    tests::fixtures::{self, reserved as f},
};
const MODES: &[(&[u8], &[Rule])] = &[
    (b"0600", &[]),
    (b"0640", &[]),
    (b"0770", &[]),
    (b"0604", &[Rule::SecretMode]),
    (b"0644", &[Rule::SecretMode]),
    (b"2770", &[Rule::SecretMode]),
    (b"4700", &[Rule::SecretMode]),
    (b"1770", &[Rule::SecretMode]),
];
const SECRET_KINDS: &[Map] = &[Map::Secrets, Map::Identity];
#[test]
fn secret_modes_give_others_nothing_and_have_no_special_bits() {
    for map in SECRET_KINDS {
        for (row, &(mode, expected)) in MODES.iter().enumerate() {
            assert_eq!(
                f::word_rules(*map, b"key", &[(Name::Mode, mode)], false).as_ref(),
                expected,
                "{map:?} row {row}"
            );
        }
    }
}
const OWNERS: &[(Name, &[u8], &[Rule])] = &[
    (Name::Owner, b"hearth", &[Rule::HearthOwner]),
    (Name::Group, b"hearth", &[Rule::HearthOwner]),
    (Name::Group, b"systemd-journal", &[Rule::HearthOwner]),
    (Name::Owner, b"root", &[]),
    (Name::Group, b"root", &[]),
];
#[test]
fn hearth_and_its_constant_groups_own_no_secret_or_identity() {
    for map in SECRET_KINDS {
        for (row, &(field, value, expected)) in OWNERS.iter().enumerate() {
            assert_eq!(f::word_rules(*map, b"key", &[(field, value)], false).as_ref(), expected, "{map:?} row {row}");
        }
    }
}
const GROUPS: &[f::HearthGroupCase<'_>] =
    &[(b"stewards", &[], &[Rule::HearthOwner]), (b"other", &[b"stewards"], &[Rule::HearthOwner])];
#[test]
fn declared_hearth_groups_point_to_the_users_own_declaration() {
    for (row, &(group, groups, expected)) in GROUPS.iter().enumerate() {
        let refused = f::check(&mut f::hearth_groups(group, groups), false);
        assert_eq!(f::rules(&refused).as_ref(), expected, "row {row}");
        assert_eq!(
            fixtures::only(&refused).also,
            Some(Place {
                path: Box::from(b"tasks/hearth/task.json".as_slice()),
                position: Some(fixtures::kinds::FILE_AT)
            }),
            "row {row}"
        );
    }
}
const NONE_UNITS: &[f::LiveUnitCase<'_>] = &[(Some(b"x.service"), b"none", &[Rule::LiveNone]), (None, b"none", &[])];
#[test]
fn action_none_names_no_unit() {
    for (row, &(unit, action, expected)) in NONE_UNITS.iter().enumerate() {
        assert_eq!(f::live_unit_rules(action, unit).as_ref(), expected, "row {row}");
    }
}
const UNITS: &[f::LiveUnitCase<'_>] = &[
    (Some(b"dbus.service"), b"reload", &[Rule::NeverLiveUnit]),
    (Some(b"dbus"), b"reload", &[Rule::NeverLiveUnit]),
    (Some(b"dbus.socket"), b"reload", &[Rule::NeverLiveUnit]),
    (Some(b"systemd-journald.service"), b"restart", &[Rule::NeverLiveUnit]),
    (Some(b"systemd-custom"), b"restart", &[Rule::NeverLiveUnit]),
    (Some(b"hearth-activator.service"), b"restart", &[Rule::NeverLiveUnit]),
    (Some(b"NetworkManager.service"), b"restart", &[Rule::NeverLiveUnit]),
    (Some(b"getty@tty1.service"), b"restart", &[Rule::NeverLiveUnit]),
    (Some(b"serial-getty@tty1"), b"restart", &[Rule::NeverLiveUnit]),
    (Some(b"console-getty"), b"restart", &[Rule::NeverLiveUnit]),
    (Some(b"apparmor.service"), b"restart", &[Rule::NeverLiveUnit]),
    (Some(b"sshd.service"), b"restart", &[]),
];
#[test]
fn live_actions_leave_core_units_alone() {
    for (row, &(unit, action, expected)) in UNITS.iter().enumerate() {
        assert_eq!(f::live_unit_rules(action, unit).as_ref(), expected, "row {row}");
    }
}
const PATHS: &[(&[u8], &[Rule])] = &[
    (b"/etc/hostname", &[Rule::NeverLivePath]),
    (b"/etc/hostname/", &[Rule::NeverLivePath]),
    (b"/etc/apparmor.d/x", &[Rule::NeverLivePath]),
    (b"/etc/udev/rules.d/x", &[Rule::NeverLivePath]),
    (b"/etc/ssl/private/key", &[Rule::NeverLivePath]),
    (b"/etc/systemd/journald.conf", &[Rule::NeverLivePath]),
    (b"/etc/systemd/journald.conf.d/x", &[]),
    (b"/etc/systemd/x/y.conf", &[]),
    (b"/etc/systemd/", &[]),
    (b"/usr/local/lib/systemd/journald.conf.d/x", &[Rule::NeverLivePath]),
    (b"/usr/local/lib/systemd/x/y.conf.d/z", &[]),
    (b"/etc/nginx/nginx.conf", &[]),
    (b"/usr/local/bin/tool", &[]),
    (b"/dev/x", &[]),
    (b"/etc", &[]),
];
#[test]
fn only_equal_or_descendant_never_live_paths_are_refused() {
    for (row, &(path, expected)) in PATHS.iter().enumerate() {
        assert_eq!(f::live_rules(b"none", None, &[path]).as_ref(), expected, "row {row}");
    }
}
#[test]
fn a_live_path_beneath_a_hearth_install_is_refused() {
    for (row, &(task, expected)) in f::HEARTH_TASKS.iter().enumerate() {
        let installed = f::installed(task, f::TOOL_PATH, None, Shape::Directory);
        assert_eq!(
            f::rules(&f::live(b"none", None, &[f::TOOL_BINARY], core::slice::from_ref(&installed))).as_ref(),
            expected,
            "row {row}"
        );
    }
}

#[test]
fn a_live_path_beneath_a_hearth_download_is_refused() {
    for (row, &(task, expected)) in f::HEARTH_TASKS.iter().enumerate() {
        assert_eq!(f::rules(&reserved::live(&f::live_with_download(task), &[])).as_ref(), expected, "row {row}");
    }
}
const FIRST_FAULT: &[f::EntryFieldCase<'_>] = &[
    (&[(Name::Path, b"/etc/shadow"), (Name::Mode, b"0644")], &[Rule::Accounts]),
    (&[(Name::Mode, b"0644"), (Name::Path, b"/etc/shadow")], &[Rule::SecretMode]),
    (&[(Name::Owner, b"hearth"), (Name::Path, b"/etc/shadow")], &[Rule::HearthOwner]),
];
#[test]
fn owner_and_path_faults_keep_the_declared_field_order() {
    for (row, &(fields, expected)) in FIRST_FAULT.iter().enumerate() {
        assert_eq!(f::rules(&f::check(&mut f::ordered_secret(fields), false)).as_ref(), expected, "row {row}");
    }
}

const DEFAULT_GROUPS: &[(&[u8], &[Rule])] = &[(b"root", &[Rule::HearthOwner]), (b"stewards", &[])];
#[test]
fn the_default_secret_group_also_excludes_hearths_groups() {
    for (row, &(group, expected)) in DEFAULT_GROUPS.iter().enumerate() {
        assert_eq!(f::rules(&f::check(&mut f::default_hearth_group(group), false)).as_ref(), expected, "row {row}");
    }
}
