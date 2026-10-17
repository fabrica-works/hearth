use super::{composition::at, only};
use crate::{
    errors::{Error, Owns},
    ids::{self, Owned},
    kinds::{
        groups::Group,
        users::{Accounts, User, UserKind},
    },
    revision::Position,
};
use skein_lib::bytes::copy_of;
pub fn id_accounts(users: &[User], groups: &[Group]) -> Accounts {
    Accounts { users: users.into(), groups: groups.into() }
}
pub fn sysusers(name: &[u8], uid: Option<u32>) -> User {
    User {
        name: copy_of(name),
        task: copy_of(b"motd"),
        at: at(1, 30),
        kind: UserKind::Sysusers {
            uid,
            group: copy_of(name),
            gecos: copy_of(b""),
            home: copy_of(b"/"),
            shell: copy_of(b"/usr/sbin/nologin"),
            subordinate: None,
            member_of: Box::new([]),
        },
    }
}
pub fn package_user(name: &[u8], uid: Option<u32>, gid: Option<u32>) -> User {
    let mut user = sysusers(name, uid);
    user.kind = UserKind::Package { uid, gid };
    user
}
pub fn id_group(name: &[u8], gid: Option<u32>, package: bool) -> Group {
    Group { name: copy_of(name), task: copy_of(b"motd"), at: at(1, 40), gid, package }
}
pub const OWNER_AT: Position = Position { line: 1, column: 71 };
pub fn owned_by<'a>(of: Owns, entry: &'a [u8], owner: &'a [u8], group: &'a [u8], at: Position) -> Owned<'a> {
    Owned { task: b"motd", of, entry, owner, owner_at: at, group, group_at: at }
}
pub fn ownership_fault(accounts: &Accounts, of: Owns, owner: &[u8], group: &[u8]) -> Error {
    only(
        &ids::check(accounts, &[owned_by(of, b"mail/tls/mx1.key", owner, group, OWNER_AT)])
            .expect_err("missing fixed id"),
    )
    .clone()
}
pub fn mail_accounts() -> Accounts {
    id_accounts(&[package_user(b"mail", Some(120), Some(121))], &[id_group(b"mail", Some(121), true)])
}

pub fn postfix_no_uid() -> Accounts {
    id_accounts(&[sysusers(b"postfix", None)], &[])
}
pub fn one_id_expansion(name: crate::revision::declaration::Name) -> crate::entries::Expansion {
    use crate::revision::declaration::{Map, Name};
    super::kinds::account_expansion(&[(
        b"motd",
        Map::Users,
        &[super::kinds::file(b"mail", [super::kinds::text(Name::From, b"package"), super::kinds::integer(name, 120)])],
    )])
}
