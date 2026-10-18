use crate::{
    errors::{Class, Collision, Id, Kind, Subject},
    revision::declaration::{Map, Name},
    tests::fixtures::{self, collisions as f},
};
use skein_lib::bytes::copy_of;
#[test]
fn equal_users_of_two_tasks_make_one_row() {
    let expansion = f::equal_users();
    let survivors = f::agreeing(&expansion).expect("identical users");
    assert_eq!(f::account_names(&survivors.accounts).as_ref(), &[b"free".as_slice()]);
}
#[test]
fn a_user_two_tasks_declare_differently_collides_naming_both() {
    let expansion = f::user_rows(
        &[fixtures::kinds::integer(Name::Uid, f::ACCOUNT_ID)],
        &[fixtures::kinds::integer(Name::Uid, f::ACCOUNT_ID + 1)],
    );
    assert_eq!(
        f::kinds(&f::agreeing(&expansion).expect_err("different users")).as_ref(),
        &[(
            Kind::Collision(Collision::Differs {
                subject: Subject::User,
                name: copy_of(b"free"),
                first: copy_of(b"a"),
                second: copy_of(b"b"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn a_package_user_and_a_sysusers_user_of_one_name_collide() {
    let expansion = f::user_rows(&[fixtures::kinds::text(Name::From, b"package")], &[]);
    assert_eq!(
        f::kinds(&f::agreeing(&expansion).expect_err("different sources")).as_ref(),
        &[(
            Kind::Collision(Collision::Differs {
                subject: Subject::User,
                name: copy_of(b"free"),
                first: copy_of(b"a"),
                second: copy_of(b"b"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn a_groups_entry_of_a_users_own_name_with_another_gid_differs() {
    assert_eq!(
        f::kinds(&f::agreeing(&f::group_with_user(f::ACCOUNT_ID + 1)).expect_err("different gid")).as_ref(),
        &[(
            Kind::Collision(Collision::Differs {
                subject: Subject::Group,
                name: copy_of(b"free"),
                first: copy_of(b"a"),
                second: copy_of(b"b"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn a_groups_entry_of_a_users_own_name_with_its_uid_is_one_row() {
    let rows = f::agreeing(&f::group_with_user(f::ACCOUNT_ID)).expect("the own group agrees");
    assert_eq!(
        rows.accounts.groups.as_ref(),
        &[crate::kinds::groups::Group {
            name: copy_of(b"free"),
            task: copy_of(b"b"),
            at: fixtures::kinds::FILE_AT,
            gid: Some(f::ACCOUNT_UID),
            package: false,
        }]
    );
}
#[test]
fn two_users_with_one_uid_collide() {
    let accounts = fixtures::id_accounts(
        &[f::named_user(b"free", b"a", Some(f::ACCOUNT_UID)), f::named_user(b"ci", b"b", Some(f::ACCOUNT_UID))],
        &[],
    );
    assert_eq!(
        f::kinds(&f::id_collisions(&accounts)).as_ref(),
        &[(
            Kind::Collision(Collision::Shared {
                id: Id::Uid,
                value: f::ACCOUNT_UID,
                first: copy_of(b"free"),
                second: copy_of(b"ci"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn a_users_own_group_takes_its_uid_in_the_gid_space() {
    let accounts = fixtures::id_accounts(
        &[f::named_user(b"free", b"a", Some(f::ACCOUNT_UID))],
        &[fixtures::id_group(b"other", Some(f::ACCOUNT_UID), false)],
    );
    assert_eq!(
        f::kinds(&f::id_collisions(&accounts)).as_ref(),
        &[(
            Kind::Collision(Collision::Shared {
                id: Id::Gid,
                value: f::ACCOUNT_UID,
                first: copy_of(b"free"),
                second: copy_of(b"other"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn overlapping_subordinate_ranges_collide() {
    let accounts = fixtures::id_accounts(
        &[f::first_range(), f::ranged(b"ci", b"b", f::RANGE_BASE + f::RANGE_COUNT - 1, f::RANGE_COUNT)],
        &[],
    );
    assert_eq!(
        f::kinds(&f::id_collisions(&accounts)).as_ref(),
        &[(
            Kind::Collision(Collision::Overlap {
                subject: Subject::Range,
                first: copy_of(b"free"),
                second: copy_of(b"ci"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn adjacent_ranges_do_not_overlap() {
    let accounts = fixtures::id_accounts(
        &[f::first_range(), f::ranged(b"ci", b"b", f::RANGE_BASE + f::RANGE_COUNT, f::RANGE_COUNT)],
        &[],
    );
    assert_eq!(f::id_collisions(&accounts).as_ref(), &[]);
}
#[test]
fn differing_preseeds_of_a_question_collide() {
    let expansion = fixtures::kinds::account_expansion(&[
        (b"a", Map::Preseeds, &[fixtures::preseed(b"app/answer", &[(Name::Type, b"string"), (Name::Value, b"one")])]),
        (b"b", Map::Preseeds, &[fixtures::preseed(b"app/answer", &[(Name::Type, b"string"), (Name::Value, b"two")])]),
    ]);
    assert_eq!(
        f::kinds(&f::agreeing(&expansion).expect_err("different preseed")).as_ref(),
        &[(
            Kind::Collision(Collision::Differs {
                subject: Subject::Preseed,
                name: copy_of(b"app/answer"),
                first: copy_of(b"a"),
                second: copy_of(b"b"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn a_boot_key_with_two_parameters_collides() {
    let expansion = fixtures::kinds::account_expansion(&[
        (b"a", Map::Cmdline, &[fixtures::kinds::file(b"console", [fixtures::kinds::text(Name::Param, b"quiet")])]),
        (b"b", Map::Cmdline, &[fixtures::kinds::file(b"console", [fixtures::kinds::text(Name::Param, b"debug")])]),
    ]);
    assert_eq!(
        f::kinds(&f::agreeing(&expansion).expect_err("different parameter")).as_ref(),
        &[(
            Kind::Collision(Collision::Differs {
                subject: Subject::Parameter,
                name: copy_of(b"console"),
                first: copy_of(b"a"),
                second: copy_of(b"b"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn a_data_line_and_a_tasks_differing_line_at_one_path_collide() {
    let expansion = f::joined(&[
        f::data(b"a", b"app", b"/opt/app"),
        fixtures::kinds::account_expansion(&[(
            b"b",
            Map::Tmpfiles,
            &[fixtures::kinds::file(
                b"/opt/app",
                [fixtures::kinds::text(Name::Type, b"d"), fixtures::kinds::text(Name::Mode, b"0700")],
            )],
        )]),
    ]);
    assert_eq!(
        f::kinds(&f::agreeing(&expansion).expect_err("different line")).as_ref(),
        &[(
            Kind::Collision(Collision::Differs {
                subject: Subject::Tmpfiles,
                name: copy_of(b"/opt/app"),
                first: copy_of(b"a"),
                second: copy_of(b"b"),
            }),
            Class::Configuration
        )]
    );
}

#[test]
fn agreeing_users_keep_the_first_tasks_provenance() {
    let rows = f::agreeing(&f::equal_users()).expect("identical users");
    assert_eq!(fixtures::only(&rows.accounts.users).task.as_ref(), b"a");
}
#[test]
fn a_password_reference_is_part_of_a_users_declaration() {
    let expansion = f::user_rows(
        &[fixtures::kinds::text(Name::Password, b"users/free")],
        &[fixtures::kinds::text(Name::Password, b"users/other")],
    );
    assert_eq!(
        f::kinds(&f::agreeing(&expansion).expect_err("different passwords")).as_ref(),
        &[(
            Kind::Collision(Collision::Differs {
                subject: Subject::User,
                name: copy_of(b"free"),
                first: copy_of(b"a"),
                second: copy_of(b"b"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn an_enclosing_range_meets_each_later_range_inside_it() {
    let accounts = fixtures::id_accounts(
        &[
            f::first_range(),
            f::ranged(b"ci", b"b", f::RANGE_BASE + f::MIN_RANGE_COUNT, f::MIN_RANGE_COUNT),
            f::ranged(b"deploy", b"c", f::RANGE_BASE + f::MIN_RANGE_COUNT + f::MIN_RANGE_COUNT, f::MIN_RANGE_COUNT),
        ],
        &[],
    );
    assert_eq!(
        f::kinds(&f::id_collisions(&accounts)).as_ref(),
        &[
            (
                Kind::Collision(Collision::Overlap {
                    subject: Subject::Range,
                    first: copy_of(b"free"),
                    second: copy_of(b"ci")
                }),
                Class::Configuration
            ),
            (
                Kind::Collision(Collision::Overlap {
                    subject: Subject::Range,
                    first: copy_of(b"free"),
                    second: copy_of(b"deploy")
                }),
                Class::Configuration
            ),
        ]
    );
}
