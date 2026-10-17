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
