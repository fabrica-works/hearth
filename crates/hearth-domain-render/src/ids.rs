//! Fixed owners and package seeds (domain/render.md, 3.3 and 4.2).
use crate::{
    entries::Expansion,
    errors::{Error, Ids, Kind, Owns, Place, Why},
    kinds::users::{Accounts, User, UserKind},
    parts::packages::{SeedGroup, SeedUser, Seeds},
    revision::{
        Position,
        declaration::{Map, Name},
    },
    vocabulary::Class,
};
use alloc::boxed::Box;
use skein_lib::{List, Map as Names, Writer, bytes::copy_of};
/// An owner and group, retaining the concrete entry and each field's place.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Owned<'a> {
    pub task: &'a [u8],
    pub of: Owns,
    pub entry: &'a [u8],
    pub owner: &'a [u8],
    pub owner_at: Position,
    pub group: &'a [u8],
    pub group_at: Position,
}
pub(crate) fn expanded(accounts: &Accounts, expansion: &Expansion) -> Result<Seeds, Box<[Error]>> {
    let entries =
        [expansion.of(Map::Data), expansion.of(Map::Secrets), expansion.of(Map::Identity), expansion.of(Map::Software)];
    let mut capacity = 0_u32;
    for group in &entries {
        capacity = capacity.saturating_add(u32::try_from(group.len()).expect("bounded entries"));
    }
    let mut owned = List::with_capacity(capacity);
    for (group, of) in entries.iter().zip([Owns::Data, Owns::Secret, Owns::Identity, Owns::Download]) {
        for located in group {
            let entry = located.entry;
            let mut owner_at = entry.at;
            let mut group_at = entry.at;
            for field in &entry.fields {
                if field.name == Name::Owner {
                    owner_at = field.at;
                }
                if field.name == Name::Group {
                    group_at = field.at;
                }
            }
            owned
                .push(Owned {
                    task: located.task,
                    of,
                    entry: &entry.key,
                    owner: entry.text(Name::Owner).unwrap_or(b"root"),
                    owner_at,
                    group: entry.text(Name::Group).unwrap_or(b"root"),
                    group_at,
                })
                .expect("entry count");
        }
    }
    match check(accounts, owned.as_slice()) {
        Ok(seeds) => Ok(seeds),
        Err(mut faults) => {
            for fault in &mut faults {
                if let Kind::Ids(Ids { owner, of: Owns::User, entry: _, why: Why::OneId }) = &fault.kind {
                    for located in &expansion.of(Map::Users) {
                        if located.entry.key.as_ref() == owner.as_ref() && fault.task.as_deref() == Some(located.task) {
                            for field in &located.entry.fields {
                                if field.name == Name::Uid || field.name == Name::Gid {
                                    fault.at = Some(place(located.task, field.at));
                                }
                            }
                        }
                    }
                }
            }
            Err(faults)
        }
    }
}
pub fn check(accounts: &Accounts, owned: &[Owned<'_>]) -> Result<Seeds, Box<[Error]>> {
    let capacity = u32::try_from(
        accounts.users.len().saturating_add(accounts.groups.len()).saturating_add(owned.len().saturating_mul(2)),
    )
    .expect("bounded entries");
    let mut users = Names::with_capacity(capacity);
    let mut groups = Names::with_capacity(capacity);
    let mut faults = List::with_capacity(capacity);
    for user in &accounts.users {
        if let UserKind::Package { uid, gid } = user.kind
            && uid.is_some() != gid.is_some()
        {
            let item = Owned {
                task: &user.task,
                of: Owns::User,
                entry: &user.name,
                owner: &user.name,
                owner_at: user.at,
                group: b"root",
                group_at: user.at,
            };
            faults.push(fault(&item, &user.name, user.at, Why::OneId, None)).expect("account count");
        }
        if let UserKind::Package { uid: Some(uid), gid: Some(gid) } = user.kind {
            users
                .insert(user.name.clone(), SeedUser { name: user.name.clone(), uid, gid: Some(gid) })
                .expect("user count");
        }
    }
    for group in &accounts.groups {
        if group.package
            && let Some(gid) = group.gid
        {
            groups.insert(group.name.clone(), SeedGroup { name: group.name.clone(), gid }).expect("group count");
        }
    }
    for item in owned {
        if item.owner != b"root" {
            match user(accounts, item.owner, item.of == Owns::Download) {
                Ok(seed) => {
                    users.insert(seed.name.clone(), seed).expect("owner count");
                }
                Err((why, also)) => {
                    if why != Why::OneId {
                        faults.push(fault(item, item.owner, item.owner_at, why, also)).expect("fault count");
                    }
                }
            }
        }
        if item.group != b"root" {
            match group(accounts, item.group, item.of == Owns::Download) {
                Ok((gid, _)) => {
                    groups
                        .insert(copy_of(item.group), SeedGroup { name: copy_of(item.group), gid })
                        .expect("owner count");
                }
                Err((why, also)) => {
                    faults.push(fault(item, item.group, item.group_at, why, also)).expect("fault count");
                }
            }
        }
    }
    if !faults.is_empty() {
        return Err(faults.into_boxed());
    }
    let mut fixed_users = List::with_capacity(users.len());
    for (_, seed) in &users {
        fixed_users.push(seed.clone()).expect("user count");
    }
    let mut fixed_groups = List::with_capacity(groups.len());
    for (_, seed) in &groups {
        fixed_groups.push(seed.clone()).expect("group count");
    }
    Ok(Seeds { users: fixed_users.into_boxed(), groups: fixed_groups.into_boxed() })
}
fn user(accounts: &Accounts, name: &[u8], package: bool) -> Result<SeedUser, (Why, Option<Place>)> {
    let mut declared = None;
    for user in &accounts.users {
        if user.name.as_ref() == name {
            declared = Some(user);
            break;
        }
    }
    let Some(user) = declared else {
        return Err((Why::NoUser, None));
    };
    let also = Some(place(&user.task, user.at));
    let (uid, gid) = match &user.kind {
        UserKind::Package { uid, gid } => {
            if uid.is_some() != gid.is_some() {
                return Err((Why::OneId, also));
            }
            (*uid, *gid)
        }
        UserKind::Sysusers { uid, group: name, gecos: _, home: _, shell: _, subordinate: _, member_of: _ } => {
            if package {
                return Err((Why::NotPackageUser, also));
            }
            let gid = if name.as_ref() == b"root" {
                Some(0)
            } else {
                match group(accounts, name, false) {
                    Ok((gid, _)) => Some(gid),
                    Err(_) => None,
                }
            };
            (*uid, gid)
        }
    };
    match uid {
        Some(uid) => Ok(SeedUser { name: user.name.clone(), uid, gid }),
        None => Err((Why::NoUid, also)),
    }
}
fn group(accounts: &Accounts, name: &[u8], package: bool) -> Result<(u32, Place), (Why, Option<Place>)> {
    for group in &accounts.groups {
        if group.name.as_ref() != name {
            continue;
        }
        let at = place(&group.task, group.at);
        if package && !group.package {
            return Err((Why::NotPackageGroup, Some(at)));
        }
        return match group.gid {
            Some(gid) => Ok((gid, at)),
            None => Err((Why::NoGid, Some(at))),
        };
    }
    for User { name: user_name, task, at, kind } in &accounts.users {
        if let UserKind::Sysusers { uid, group, gecos: _, home: _, shell: _, subordinate: _, member_of: _ } = kind
            && user_name.as_ref() == name
            && group.as_ref() == name
        {
            let at = place(task, *at);
            if package {
                return Err((Why::NotPackageGroup, Some(at)));
            }
            return match uid {
                Some(gid) => Ok((*gid, at)),
                None => Err((Why::NoGid, Some(at))),
            };
        }
    }
    Err((Why::NoGroup, None))
}
fn place(task: &[u8], at: Position) -> Place {
    let mut path = Writer::new(task.len().saturating_add(16));
    path.put(b"tasks/").expect("path room");
    path.put(task).expect("path room");
    path.put(b"/task.json").expect("path room");
    Place { path: path.finish(), position: Some(at) }
}
fn fault(item: &Owned<'_>, owner: &[u8], at: Position, why: Why, also: Option<Place>) -> Error {
    Error {
        side: None,
        kind: Kind::Ids(Ids { owner: copy_of(owner), of: item.of, entry: copy_of(item.entry), why }),
        class: Class::Configuration,
        at: Some(place(item.task, at)),
        also,
        target: None,
        task: Some(copy_of(item.task)),
    }
}
