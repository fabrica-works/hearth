use super::kinds;
use crate::{
    collisions::{self, Declared},
    entries::Expansion,
    errors::{Class, Collector, Error, Kind},
    kinds::files::{self, Plan},
    limits::SHIPPED,
    revision::{
        Position,
        declaration::{Map, Name},
    },
};
use alloc::boxed::Box;
use skein_lib::List;
pub fn plan_of(task: &[u8], paths: &[(&[u8], Option<Position>)]) -> Plan {
    let mut names = List::with_capacity(u32::try_from(paths.len()).expect("fixture count"));
    for (path, _) in paths {
        names.push(*path).expect("fixture count");
    }
    let mut plan =
        files::plan(task, &kinds::tree(names.as_slice()), &[], &[], &mut List::with_capacity(EMPTY_CLAIM_COUNT));
    for install in &mut plan.installs {
        for (path, at) in paths {
            if install.path.as_ref() == *path {
                install.at = *at;
            }
        }
    }
    plan
}
pub fn no_expansion() -> Expansion {
    Expansion { tasks: Box::new([]) }
}
pub fn collide(plans: &[Plan], expansion: &Expansion) -> Box<[Error]> {
    collide_with_limits(plans, expansion, &SHIPPED)
}
pub fn collide_with_limits(plans: &[Plan], expansion: &Expansion, limits: &crate::limits::Limits) -> Box<[Error]> {
    let accounts = crate::parts::config::accounts(&no_expansion()).expect("no accounts");
    let mut errors = Collector::new(limits);
    let _valid =
        collisions::paths(&Declared { target: b"x1-prod", plans, expansion, accounts: &accounts }, limits, &mut errors);
    errors.finish().0
}
pub fn kinds(errors: &[Error]) -> Box<[(Kind, Class)]> {
    let mut kinds = List::with_capacity(u32::try_from(errors.len()).expect("error count"));
    for error in errors {
        kinds.push((error.kind.clone(), error.class)).expect("error count");
    }
    kinds.into_boxed()
}
pub fn tree_twice() -> Box<[Error]> {
    collide(
        &[
            plan_of(b"motd", &[(b"/etc/motd", None)]),
            plan_of(b"banner", &[(b"/etc/motd", Some(super::composition::at(1, 12)))]),
        ],
        &no_expansion(),
    )
}
pub fn data(task: &[u8], name: &[u8], path: &[u8]) -> Expansion {
    kinds::account_expansion(&[(task, Map::Data, &[kinds::file(name, [kinds::text(Name::Path, path)])])])
}
pub fn secret(task: &[u8], path: &[u8]) -> Expansion {
    kinds::account_expansion(&[(task, Map::Secrets, &[kinds::file(b"mail/tls.key", [kinds::text(Name::Path, path)])])])
}
pub fn identity(task: &[u8], path: &[u8]) -> Expansion {
    identity_fields(task, path, &[])
}
pub fn identity_with_public(task: &[u8], path: &[u8], public: &[u8]) -> Expansion {
    identity_fields(task, path, &[kinds::text(Name::PublicPath, public)])
}
fn identity_fields(task: &[u8], path: &[u8], extra: &[crate::entries::Made]) -> Expansion {
    let mut fields = List::with_capacity(
        u32::try_from(IDENTITY_FIELDS.len().saturating_add(extra.len()).saturating_add(1)).expect("fixture count"),
    );
    for (name, value) in IDENTITY_FIELDS {
        fields.push(kinds::text(*name, value)).expect("fixture count");
    }
    fields.push(kinds::text(Name::Path, path)).expect("fixture count");
    for field in extra {
        fields.push(field.clone()).expect("fixture count");
    }
    kinds::account_expansion(&[(
        task,
        Map::Identity,
        &[crate::entries::Concrete { fields: fields.into_boxed(), ..kinds::file(b"ssh", []) }],
    )])
}
const IDENTITY_FIELDS: &[(Name, &[u8])] = &[(Name::Kind, b"ssh-host-key"), (Name::Algorithm, b"ed25519")];
pub fn download(task: &[u8], path: &[u8], unpack: &[u8]) -> Expansion {
    kinds::account_expansion(&[(
        task,
        Map::Software,
        &[kinds::file(b"app", super::download_fields(b"https://example.org/app", unpack, path))],
    )])
}
pub fn joined(expansions: &[Expansion]) -> Expansion {
    let mut count = 0_u32;
    for expansion in expansions {
        count = count.saturating_add(u32::try_from(expansion.tasks.len()).expect("fixture count"));
    }
    let mut tasks = List::with_capacity(count);
    for expansion in expansions {
        for task in &expansion.tasks {
            tasks.push(task.clone()).expect("fixture count");
        }
    }
    Expansion { tasks: tasks.into_boxed() }
}
pub fn hidden(expansion: Expansion) -> Box<[Error]> {
    collide(&[], &joined(&[expansion, data(b"z-data", b"app", b"/opt/app")]))
}
pub fn tar_over(expansion: Expansion) -> Box<[Error]> {
    collide(&[], &joined(&[expansion, download(b"tar", b"/opt/app", b"tar")]))
}
pub fn two_data(first: &[u8], second: &[u8]) -> Expansion {
    joined(&[data(b"alpha", b"one", first), data(b"beta", b"two", second)])
}
// One pair of identical tree claims gives exactly one error.
pub const ONE_COLLISION: usize = 1;
// No claim fits in an empty table.
pub const EMPTY_CLAIMS: crate::limits::Limits = crate::limits::Limits { expansion: 0, ..SHIPPED };
// An empty fixture has no claims or faults.
const EMPTY_CLAIM_COUNT: u32 = 0;

pub fn agreeing(expansion: &Expansion) -> Result<collisions::Survivors, Box<[Error]>> {
    let accounts = crate::parts::config::accounts(expansion).expect("valid fixture accounts");
    let mut errors = Collector::new(&SHIPPED);
    let survivors = collisions::declarations(
        &Declared { target: b"x1-prod", expansion, plans: &[], accounts: &accounts },
        &SHIPPED,
        &mut errors,
    );
    match survivors {
        Some(rows) => Ok(rows),
        None => Err(errors.finish().0),
    }
}
pub fn account_names(accounts: &crate::kinds::users::Accounts) -> Box<[&[u8]]> {
    let mut names = List::with_capacity(u32::try_from(accounts.users.len()).expect("fixture count"));
    for user in &accounts.users {
        names.push(user.name.as_ref()).expect("fixture count");
    }
    names.into_boxed()
}
pub fn user_rows(first: &[crate::entries::Made], second: &[crate::entries::Made]) -> Expansion {
    kinds::account_expansion(&[
        (b"a", Map::Users, &[crate::entries::Concrete { fields: first.into(), ..kinds::file(b"free", []) }]),
        (b"b", Map::Users, &[crate::entries::Concrete { fields: second.into(), ..kinds::file(b"free", []) }]),
    ])
}
pub fn group_with_user(gid: i64) -> Expansion {
    joined(&[
        kinds::account_expansion(&[(
            b"a",
            Map::Users,
            &[kinds::file(b"free", [kinds::integer(Name::Uid, ACCOUNT_ID)])],
        )]),
        kinds::account_expansion(&[(b"b", Map::Groups, &[kinds::file(b"free", [kinds::integer(Name::Gid, gid)])])]),
    ])
}
pub fn id_collisions(accounts: &crate::kinds::users::Accounts) -> Box<[Error]> {
    let mut errors = Collector::new(&SHIPPED);
    let _valid = collisions::ids(accounts, b"x1-prod", &SHIPPED, &mut errors);
    errors.finish().0
}
pub fn named_user(name: &[u8], task: &[u8], uid: Option<u32>) -> crate::kinds::users::User {
    let mut user = super::sysusers(name, uid);
    user.task = skein_lib::bytes::copy_of(task);
    user
}
pub fn ranged(name: &[u8], task: &[u8], base: i64, count: i64) -> crate::kinds::users::User {
    let mut user = named_user(name, task, None);
    let crate::kinds::users::UserKind::Sysusers { subordinate, .. } = &mut user.kind else { panic!("sysusers") };
    *subordinate = Some((u32::try_from(base).expect("fixture base"), u32::try_from(count).expect("fixture count")));
    user
}
// An account id inside format.md 5.6's bounds; its neighbour is the second account's id.
pub const ACCOUNT_ID: i64 = 1_000;
pub const ACCOUNT_UID: u32 = 1_000;
// Ranges begin at format.md 5.6's lower bound and span one ordinary subuid block.
pub const RANGE_BASE: i64 = kinds::MIN_SUBORDINATE;
pub const RANGE_COUNT: i64 = RANGE_BASE;

pub fn equal_users() -> Expansion {
    user_rows(&[kinds::integer(Name::Uid, ACCOUNT_ID)], &[kinds::integer(Name::Uid, ACCOUNT_ID)])
}
// The smallest count format.md 5.6 permits for a subordinate range.
pub const MIN_RANGE_COUNT: i64 = 1;

pub fn first_range() -> crate::kinds::users::User {
    ranged(b"free", b"a", RANGE_BASE, RANGE_COUNT)
}
