use super::fixtures::{self, reserved as f};
use crate::{
    errors::{Reserved, Rule},
    reserved::Spot,
    revision::declaration::{Map, Name},
};
#[test]
fn a_reserved_key_is_refused_at_its_param() {
    let refused = f::check(&mut f::command(b"rootwait"), false);
    assert_eq!(*f::reserved(&refused), [Reserved { rule: Rule::RootWait, what: Box::from(b"rootwait".as_slice()) }]);
    assert_eq!(fixtures::only(&refused).spot, Spot::Declared(fixtures::FIELD_AT));
}
#[test]
fn a_reserved_entry_leaves_the_kinds_nothing() {
    let mut expansion =
        fixtures::boot_expansion(&[(b"base", b"wait", b"rootwait", None), (b"base", b"quiet", b"quiet", None)]);
    drop(f::check(&mut expansion, false));
    assert_eq!(fixtures::only(&expansion.of(Map::Cmdline)).entry.key.as_ref(), b"quiet");
}
const KEYS: &[(&[u8], &[Rule])] = &[
    (b"root=UUID=x", &[Rule::ActivatorKey]),
    (b"rootflags=x", &[Rule::ActivatorKey]),
    (b"ro", &[Rule::ActivatorKey]),
    (b"rw", &[Rule::ActivatorKey]),
    (b"initrd=x", &[Rule::ActivatorKey]),
    (b"panic=10", &[Rule::ActivatorKey]),
    (b"hearth.trial=x", &[Rule::ActivatorKey]),
    (b"hearth.any=x", &[Rule::ActivatorKey]),
    (b"rootfstype=x", &[Rule::RootKey]),
    (b"rootdelay=1", &[Rule::RootKey]),
    (b"init=x", &[Rule::RootKey]),
    (b"rootwait", &[Rule::RootWait]),
    (b"console=ttyS0", &[]),
    (b"rootwaiting", &[]),
    (b"hearthx.any=x", &[]),
];
#[test]
fn every_key_of_5_11_and_every_hearth_key_is_refused_and_console_is_not() {
    for (row, &(param, expected)) in KEYS.iter().enumerate() {
        assert_eq!(f::rules(&f::check(&mut f::command(param), false)).as_ref(), expected, "row {row}");
    }
}
const LENGTHS: &[(usize, &[Rule])] =
    &[(f::FITTING_PARAMETER, &[]), (f::FITTING_PARAMETER + 1, &[Rule::CmdlineLength { bytes: 2049 }])];
#[test]
fn the_line_is_counted_with_the_activators_tokens_and_the_nul() {
    for &(length, expected) in LENGTHS {
        assert_eq!(
            f::rules(&f::check(&mut f::command(&f::PARAM_BYTES[..length]), false)).as_ref(),
            expected,
            "length {length}"
        );
    }
}
#[test]
fn a_password_preseed_is_reserved_where_the_kind_alone_calls_it_unlisted() {
    assert_eq!(
        f::word_rules(Map::Preseeds, b"d/p", &[(Name::Type, b"password")], false).as_ref(),
        &[Rule::PasswordPreseed]
    );
}
const TRIALS: &[(&[u8], &[u8])] = &[
    (b"hearth-trial.service", b"enable"),
    (b"hearth-trial-other.timer", b"disable"),
    (b"hearth-check-x.service", b"enable"),
    (b"hearth-check-.service", b"disable"),
];
#[test]
fn a_trial_or_check_name_is_refused_in_a_preset_whatever_its_action() {
    for (row, &(unit, action)) in TRIALS.iter().enumerate() {
        assert_eq!(
            f::word_rules(Map::Presets, unit, &[(Name::Action, action)], false).as_ref(),
            &[Rule::TrialName],
            "row {row}"
        );
    }
}
const OVERLAYS: &[(&[u8], &[u8], &[Rule])] = &[
    (b"systemd-sysext.service", b"enable", &[Rule::Overlay]),
    (b"systemd-confext.service", b"enable", &[Rule::Overlay]),
    (b"systemd-sysext.service", b"disable", &[]),
    (b"systemd-confext.service", b"disable", &[]),
];
#[test]
fn only_an_enabling_preset_of_sysext_or_confext_is_refused() {
    for (row, &(unit, action, expected)) in OVERLAYS.iter().enumerate() {
        assert_eq!(f::word_rules(Map::Presets, unit, &[(Name::Action, action)], false).as_ref(), expected, "row {row}");
    }
}
#[test]
fn an_entry_with_two_reserved_fields_is_refused_once_at_the_first() {
    let refused = f::refused(Map::Tmpfiles, b"/etc/a*", &[(Name::Type, b"v")], false);
    assert_eq!(f::rules(&refused).as_ref(), &[Rule::TmpfilesPath]);
    assert_eq!(fixtures::only(&refused).spot, Spot::Declared(fixtures::composition::at(1, 14)));
}
const TMPFILES: &[(&[u8], &[u8], &[Rule])] = &[
    (b"/var/x", b"v", &[Rule::TmpfilesSubvolume]),
    (b"/var/x", b"q", &[Rule::TmpfilesSubvolume]),
    (b"/var/x", b"Q", &[Rule::TmpfilesSubvolume]),
    (b"/etc/x", b"w", &[Rule::TmpfilesWrite]),
    (b"/etc/x", b"w+", &[Rule::TmpfilesWrite]),
    (b"/etc/x", b"f+", &[Rule::TmpfilesWrite]),
    (b"/etc/x", b"L+", &[Rule::TmpfilesWrite]),
    (b"/variant/x", b"w", &[Rule::TmpfilesWrite]),
    (b"/var/x", b"w", &[]),
    (b"/run/x", b"w+", &[]),
    (b"/tmp/x", b"f+", &[]),
    (b"/etc/x", b"f", &[]),
];
#[test]
fn tmpfiles_v_q_and_capital_q_and_writes_outside_var_run_tmp_are_refused() {
    for (row, &(path, kind, expected)) in TMPFILES.iter().enumerate() {
        assert_eq!(f::word_rules(Map::Tmpfiles, path, &[(Name::Type, kind)], false).as_ref(), expected, "row {row}");
    }
}
const GLOBS: &[&[u8]] = &[b"/var/*", b"/var/?", b"/var/[a]", b"/var/%u"];
#[test]
fn a_tmpfiles_key_with_a_glob_or_a_specifier_is_refused() {
    for (row, path) in GLOBS.iter().enumerate() {
        assert_eq!(f::word_rules(Map::Tmpfiles, path, &[], false).as_ref(), &[Rule::TmpfilesPath], "row {row}");
    }
}
#[test]
fn the_line_length_error_is_at_the_first_param_past_2048() {
    let mut expansion = fixtures::boot_expansion(&[
        (b"b", b"b", &f::PARAM_BYTES[..f::LONG_PARAMETER], None),
        (b"a", b"a", &f::PARAM_BYTES[..f::LONG_PARAMETER], None),
    ]);
    let refused = f::check(&mut expansion, false);
    assert_eq!(fixtures::only(&refused).task.as_ref(), b"b");
    assert_eq!(fixtures::only(&refused).spot, Spot::Declared(fixtures::FIELD_AT));
    assert_eq!(f::rules(&refused).as_ref(), &[Rule::CmdlineLength { bytes: 2141 }]);
}
const CONTAINERS: &[(Map, bool, &[Rule])] = &[
    (Map::Initramfs, true, &[Rule::ContainerModule]),
    (Map::Cmdline, true, &[Rule::ContainerParam]),
    (Map::Initramfs, false, &[]),
    (Map::Cmdline, false, &[]),
];
#[test]
fn a_module_or_a_boot_parameter_on_a_container_is_refused_and_on_a_vm_is_not() {
    for (row, &(map, container, expected)) in CONTAINERS.iter().enumerate() {
        assert_eq!(f::word_rules(map, b"btrfs", &[], container).as_ref(), expected, "row {row}");
    }
}
