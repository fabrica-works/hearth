use super::super::fixtures::{
    self,
    kinds::{
        self as fixture, MAX_SUBORDINATE_COUNT, MAX_UID, MIN_SUBORDINATE, MIN_UID, accounts, file, integer, range,
        strings, text,
    },
};
use crate::{
    errors::{Format, Kind, Schema},
    kinds::users::UserKind,
    revision::declaration::{Map, Name},
};
#[test]
fn a_user_without_fields_gets_5_6s_defaults() {
    let accounts = accounts([]).expect("defaults");
    assert_eq!(
        fixtures::only(&accounts.users).kind,
        UserKind::Sysusers {
            uid: None,
            group: b"free".as_slice().into(),
            gecos: Box::new([]),
            home: b"/".as_slice().into(),
            shell: b"/usr/sbin/nologin".as_slice().into(),
            subordinate: None,
            member_of: Box::new([])
        }
    );
}
#[test]
fn a_package_user_takes_only_its_ids() {
    let accounts = accounts([text(Name::From, b"package"), integer(Name::Uid, MIN_UID), integer(Name::Gid, MAX_UID)])
        .expect("package ids");
    assert_eq!(fixtures::only(&accounts.users).kind, fixture::PACKAGE_LIMITS);
}
const PACKAGE_FIELDS: &[(Name, &[u8], &[u8])] = &[
    (Name::Group, b"other", b"group"),
    (Name::Gecos, b"User", b"gecos"),
    (Name::Home, b"/home/free", b"home"),
    (Name::Shell, b"/bin/sh", b"shell"),
    (Name::Password, b"secret", b"password"),
];
#[test]
fn other_package_user_fields_are_undeclared() {
    for (row, &(name, value, field)) in PACKAGE_FIELDS.iter().enumerate() {
        let faults = accounts([text(Name::From, b"package"), text(name, value)]).expect_err("package field");
        assert_eq!(
            fixtures::only(&faults).kind,
            Kind::Schema(Schema::UndeclaredField {
                field: field.into(),
                declared: Box::new([b"from".as_slice().into(), b"uid".as_slice().into(), b"gid".as_slice().into()])
            }),
            "row {row}"
        );
    }
}
#[test]
fn groups_become_sorted_members_once() {
    let entries = [file(b"free", [strings(Name::Groups, &[b"sudo", b"ops", b"sudo"])])];
    let config = fixture::account_config(&fixture::account_expansion(&[(b"motd", Map::Users, &entries)]));
    assert_eq!(
        config.members.as_ref(),
        [
            crate::parts::config::Member { user: b"free".as_slice().into(), group: b"ops".as_slice().into() },
            crate::parts::config::Member { user: b"free".as_slice().into(), group: b"sudo".as_slice().into() }
        ]
    );
}
const MAPS: &[Map] = &[Map::Users, Map::Groups];
#[test]
fn an_unknown_from_lists_the_two_values() {
    for (row, &map) in MAPS.iter().enumerate() {
        let entries = [file(b"free", [text(Name::From, b"other")])];
        let faults = crate::parts::config::accounts(&fixture::account_expansion(&[(b"motd", map, &entries)]))
            .expect_err("unknown source");
        assert_eq!(
            fixtures::only(&faults).kind,
            Kind::Schema(Schema::Enum {
                value: b"other".as_slice().into(),
                listed: Box::new([b"sysusers".as_slice().into(), b"package".as_slice().into()])
            }),
            "row {row}"
        );
    }
}
const IDS: &[(Name, i64, Schema)] = &[
    (Name::Uid, MIN_UID - 1, Schema::Below { min: MIN_UID }),
    (Name::Gid, MIN_UID - 1, Schema::Below { min: MIN_UID }),
    (Name::Uid, MAX_UID + 1, Schema::Above { max: MAX_UID }),
    (Name::Gid, MAX_UID + 1, Schema::Above { max: MAX_UID }),
    (Name::Uid, MAX_UID + 2, Schema::Above { max: MAX_UID }),
    (Name::Gid, MAX_UID + 2, Schema::Above { max: MAX_UID }),
];
#[test]
fn a_uid_or_gid_outside_1_to_65533_errs_below_or_above() {
    for (row, (name, value, schema)) in IDS.iter().enumerate() {
        let faults = accounts([text(Name::From, b"package"), integer(*name, *value)]).expect_err("reserved id");
        assert_eq!(fixtures::only(&faults).kind, Kind::Schema(schema.clone()), "row {row}");
    }
}
const RANGES: &[(i64, i64, Schema)] = &[
    (MIN_SUBORDINATE - 1, MIN_UID, Schema::Below { min: MIN_SUBORDINATE }),
    (MIN_SUBORDINATE, MIN_UID - 1, Schema::Below { min: MIN_UID }),
    (MIN_SUBORDINATE, MAX_SUBORDINATE_COUNT + 1, Schema::Above { max: MAX_SUBORDINATE_COUNT }),
];
#[test]
fn a_subordinate_range_out_of_bounds_errs_below_or_above() {
    for (row, (base, count, schema)) in RANGES.iter().enumerate() {
        let faults = accounts([range(*base, *count)]).expect_err("range bounds");
        assert_eq!(fixtures::only(&faults).kind, Kind::Schema(schema.clone()), "row {row}");
    }
}
#[test]
fn a_subordinate_range_at_the_limits_is_carried() {
    let accounts = accounts([range(MIN_SUBORDINATE, MAX_SUBORDINATE_COUNT)]).expect("range");
    let UserKind::Sysusers { subordinate, .. } = &fixtures::only(&accounts.users).kind else {
        panic!("sysusers account")
    };
    assert_eq!(*subordinate, Some(fixture::SUBORDINATE_LIMITS));
}
const BAD_TEXT: &[&[u8]] = &[b"/a:b", b"/a\"b", b"/a\\b", b"/a\nb", b"/a\x7fb"];
const TEXT_FIELDS: &[Name] = &[Name::Gecos, Name::Home, Name::Shell];
#[test]
fn a_gecos_home_or_shell_with_a_colon_a_quote_a_backslash_or_a_control_byte_is_malformed() {
    for (row, &name) in TEXT_FIELDS.iter().enumerate() {
        for (case, &value) in BAD_TEXT.iter().enumerate() {
            let faults = accounts([text(name, value)]).expect_err("passwd quoting");
            assert_eq!(
                fixtures::only(&faults).kind,
                Kind::Schema(Schema::Malformed { format: Format::Key }),
                "row {row}, case {case}"
            );
        }
    }
}
#[test]
fn equal_users_of_two_tasks_collapse() {
    let users = [file(b"free", [])];
    let config = fixture::account_config(&fixture::account_expansion(&[
        (b"one", Map::Users, &users),
        (b"two", Map::Users, &users),
    ]));
    assert_eq!(
        config.users.as_ref(),
        [crate::parts::config::User {
            name: b"free".as_slice().into(),
            kind: crate::parts::config::UserKind::Sysusers {
                uid: None,
                group: b"free".as_slice().into(),
                gecos: Box::new([]),
                home: b"/".as_slice().into(),
                shell: b"/usr/sbin/nologin".as_slice().into()
            }
        }]
    );
}
const RELATIVE_PATHS: &[(Name, &[u8])] = &[(Name::Home, b"home/free"), (Name::Shell, b"bin/sh")];
#[test]
fn a_home_or_shell_must_be_an_absolute_path() {
    for (row, &(name, value)) in RELATIVE_PATHS.iter().enumerate() {
        let faults = accounts([text(name, value)]).expect_err("relative path");
        assert_eq!(fixtures::only(&faults).kind, Kind::Schema(Schema::Malformed { format: Format::Path }), "row {row}");
    }
}
