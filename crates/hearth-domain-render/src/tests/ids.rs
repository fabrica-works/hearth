use super::fixtures::{self, composition::at, only};
use crate::{
    errors::{Ids, Kind, Owns, Place, Why},
    ids,
    parts::packages::{SeedGroup, SeedUser, Seeds},
};
use skein_lib::bytes::copy_of;
#[test]
fn a_secrets_owner_without_a_uid_is_an_ids_fault_at_the_owner() {
    let accounts = fixtures::postfix_no_uid();
    let error = fixtures::ownership_fault(&accounts, Owns::Secret, b"postfix", b"root");
    assert_eq!(
        error.kind,
        Kind::Ids(Ids {
            owner: copy_of(b"postfix"),
            of: Owns::Secret,
            entry: copy_of(b"mail/tls/mx1.key"),
            why: Why::NoUid
        })
    );
    assert_eq!(error.at, Some(Place { path: copy_of(b"tasks/motd/task.json"), position: Some(fixtures::OWNER_AT) }));
}
#[test]
fn the_ids_fault_also_names_the_declaration() {
    let accounts = fixtures::postfix_no_uid();
    assert_eq!(
        fixtures::ownership_fault(&accounts, Owns::Secret, b"postfix", b"root").also,
        Some(Place { path: copy_of(b"tasks/motd/task.json"), position: Some(at(1, 30)) })
    );
}
#[test]
fn an_undeclared_owner_is_no_user() {
    assert_eq!(
        fixtures::ownership_fault(&fixtures::id_accounts(&[], &[]), Owns::Secret, b"postfix", b"root").kind,
        Kind::Ids(Ids {
            owner: copy_of(b"postfix"),
            of: Owns::Secret,
            entry: copy_of(b"mail/tls/mx1.key"),
            why: Why::NoUser
        })
    );
}
#[test]
fn a_package_user_with_both_ids_is_fixed() {
    let accounts = fixtures::mail_accounts();
    let seeds =
        ids::check(&accounts, &[fixtures::owned_by(Owns::Download, b"tool", b"mail", b"mail", fixtures::OWNER_AT)])
            .expect("fixed package ids");
    assert_eq!(
        seeds,
        Seeds {
            users: Box::new([SeedUser { name: copy_of(b"mail"), uid: 120, gid: Some(121) }]),
            groups: Box::new([SeedGroup { name: copy_of(b"mail"), gid: 121 }])
        }
    );
}
const ONE_ID: &[(Option<u32>, Option<u32>)] = &[(Some(120), None), (None, Some(121))];
#[test]
fn a_package_user_with_one_id_alone_is_an_ids_fault() {
    for (row, &(uid, gid)) in ONE_ID.iter().enumerate() {
        let accounts = fixtures::id_accounts(&[fixtures::package_user(b"mail", uid, gid)], &[]);
        let errors = ids::check(&accounts, &[]).expect_err("one id");
        assert_eq!(
            only(&errors).kind,
            Kind::Ids(Ids { owner: copy_of(b"mail"), of: Owns::User, entry: copy_of(b"mail"), why: Why::OneId }),
            "row {row}"
        );
    }
}
#[test]
fn a_package_user_with_neither_id_seeds_no_line() {
    let accounts = fixtures::id_accounts(&[fixtures::package_user(b"mail", None, None)], &[]);
    assert_eq!(ids::check(&accounts, &[]), Ok(Seeds { users: Box::new([]), groups: Box::new([]) }));
}
const GROUPS: &[(bool, u32)] = &[(true, 121), (false, 120)];
#[test]
fn a_group_is_fixed_by_its_gid_or_a_users_own_group() {
    for (row, &(declared, gid)) in GROUPS.iter().enumerate() {
        let group = fixtures::id_group(b"mail", Some(gid), false);
        let groups = if declared { core::slice::from_ref(&group) } else { &[] };
        let accounts = fixtures::id_accounts(&[fixtures::sysusers(b"mail", Some(120))], groups);
        let seeds =
            ids::check(&accounts, &[fixtures::owned_by(Owns::Secret, b"mail", b"root", b"mail", fixtures::OWNER_AT)])
                .expect("fixed group");
        assert_eq!(seeds.groups.as_ref(), &[SeedGroup { name: copy_of(b"mail"), gid }], "row {row}");
    }
}
#[test]
fn root_needs_no_declaration() {
    let seeds = ids::check(
        &fixtures::id_accounts(&[], &[]),
        &[fixtures::owned_by(Owns::Download, b"tool", b"root", b"root", fixtures::OWNER_AT)],
    );
    assert_eq!(seeds, Ok(Seeds { users: Box::new([]), groups: Box::new([]) }));
}
#[test]
fn a_download_owned_by_a_sysusers_user_is_not_a_package_user() {
    let accounts = fixtures::id_accounts(&[fixtures::sysusers(b"postfix", Some(120))], &[]);
    assert_eq!(
        fixtures::ownership_fault(&accounts, Owns::Download, b"postfix", b"root").kind,
        Kind::Ids(Ids {
            owner: copy_of(b"postfix"),
            of: Owns::Download,
            entry: copy_of(b"mail/tls/mx1.key"),
            why: Why::NotPackageUser
        })
    );
}
const OWNED_KINDS: &[Owns] = &[Owns::Data, Owns::Identity];
#[test]
fn a_data_path_and_an_identity_file_are_checked_as_owned() {
    for (row, &of) in OWNED_KINDS.iter().enumerate() {
        assert_eq!(
            fixtures::ownership_fault(&fixtures::id_accounts(&[], &[]), of, b"postfix", b"root").kind,
            Kind::Ids(Ids { owner: copy_of(b"postfix"), of, entry: copy_of(b"mail/tls/mx1.key"), why: Why::NoUser }),
            "row {row}"
        );
    }
}
#[test]
fn an_undeclared_group_is_no_group() {
    assert_eq!(
        fixtures::ownership_fault(&fixtures::id_accounts(&[], &[]), Owns::Secret, b"root", b"mail").kind,
        Kind::Ids(Ids {
            owner: copy_of(b"mail"),
            of: Owns::Secret,
            entry: copy_of(b"mail/tls/mx1.key"),
            why: Why::NoGroup
        })
    );
}
#[test]
fn a_group_with_no_gid_is_no_gid() {
    let accounts = fixtures::id_accounts(&[], &[fixtures::id_group(b"mail", None, false)]);
    assert_eq!(
        fixtures::ownership_fault(&accounts, Owns::Secret, b"root", b"mail").kind,
        Kind::Ids(Ids {
            owner: copy_of(b"mail"),
            of: Owns::Secret,
            entry: copy_of(b"mail/tls/mx1.key"),
            why: Why::NoGid
        })
    );
}
#[test]
fn a_download_group_that_is_no_package_group_is_refused() {
    let accounts = fixtures::id_accounts(&[], &[fixtures::id_group(b"mail", Some(120), false)]);
    assert_eq!(
        fixtures::ownership_fault(&accounts, Owns::Download, b"root", b"mail").kind,
        Kind::Ids(Ids {
            owner: copy_of(b"mail"),
            of: Owns::Download,
            entry: copy_of(b"mail/tls/mx1.key"),
            why: Why::NotPackageGroup
        })
    );
}
#[test]
fn the_seeds_are_the_package_ids_and_the_owners_sorted() {
    let accounts = fixtures::id_accounts(
        &[fixtures::package_user(b"z", Some(122), Some(123)), fixtures::sysusers(b"a", Some(120))],
        &[fixtures::id_group(b"z", Some(123), true), fixtures::id_group(b"a", Some(121), false)],
    );
    let owned = [
        fixtures::owned_by(Owns::Data, b"data", b"a", b"a", fixtures::OWNER_AT),
        fixtures::owned_by(Owns::Secret, b"key", b"a", b"a", fixtures::OWNER_AT),
    ];
    assert_eq!(
        ids::check(&accounts, &owned),
        Ok(Seeds {
            users: Box::new([
                SeedUser { name: copy_of(b"a"), uid: 120, gid: Some(121) },
                SeedUser { name: copy_of(b"z"), uid: 122, gid: Some(123) }
            ]),
            groups: Box::new([
                SeedGroup { name: copy_of(b"a"), gid: 121 },
                SeedGroup { name: copy_of(b"z"), gid: 123 }
            ])
        })
    );
}
#[test]
fn empty_seeds_are_a_field_of_the_packages_record() {
    assert_eq!(fixtures::thin_packages().seeds, Seeds { users: Box::new([]), groups: Box::new([]) });
}

const ID_FIELDS: &[crate::revision::declaration::Name] =
    &[crate::revision::declaration::Name::Uid, crate::revision::declaration::Name::Gid];
#[test]
fn a_one_id_error_is_at_the_given_id_field() {
    for (row, &field) in ID_FIELDS.iter().enumerate() {
        let expansion = fixtures::one_id_expansion(field);
        let accounts = crate::parts::config::accounts(&expansion).expect("valid accounts");
        let errors = ids::expanded(&accounts, &expansion).expect_err("one id");
        assert_eq!(only(&errors).at.as_ref().expect("placed").position, Some(fixtures::kinds::FIELD_AT), "row {row}");
    }
}

const OWNED_ENTRIES: &[(crate::revision::declaration::Map, Owns)] = &[
    (crate::revision::declaration::Map::Data, Owns::Data),
    (crate::revision::declaration::Map::Secrets, Owns::Secret),
    (crate::revision::declaration::Map::Identity, Owns::Identity),
    (crate::revision::declaration::Map::Software, Owns::Download),
];
#[test]
fn each_owned_map_keeps_its_concrete_owner_field_place() {
    for (row, &(map, of)) in OWNED_ENTRIES.iter().enumerate() {
        let expansion =
            fixtures::secret_entry(map, b"entry", &[(crate::revision::declaration::Name::Owner, b"postfix")]);
        let faults = ids::expanded(&fixtures::id_accounts(&[], &[]), &expansion).expect_err("undeclared owner");
        assert_eq!(
            (&only(&faults).kind, only(&faults).at.as_ref().expect("placed").position),
            (
                &Kind::Ids(Ids { owner: copy_of(b"postfix"), of, entry: copy_of(b"entry"), why: Why::NoUser }),
                Some(fixtures::FIELD_AT)
            ),
            "row {row}"
        );
    }
}
#[test]
fn an_owned_package_user_with_one_id_has_one_fault_of_the_user_entry() {
    let accounts = fixtures::id_accounts(&[fixtures::package_user(b"mail", Some(120), None)], &[]);
    let faults =
        ids::check(&accounts, &[fixtures::owned_by(Owns::Secret, b"key", b"mail", b"root", fixtures::OWNER_AT)])
            .expect_err("one id");
    assert_eq!(
        only(&faults).kind,
        Kind::Ids(Ids { owner: copy_of(b"mail"), of: Owns::User, entry: copy_of(b"mail"), why: Why::OneId })
    );
}
