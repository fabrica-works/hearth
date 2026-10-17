//! Users retain their source and subordinate ranges for build (format.md, 5.6).
use super::{
    groups::{self, Group, bounded, id, package},
    secrets::{fault, formed, text},
};
use crate::{
    entries::Located,
    errors::{Error, Format, Schema},
    revision::{Position, declaration::Name},
};
use alloc::boxed::Box;
use skein_lib::{List, Map, bytes::copy_of};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Accounts {
    pub users: Box<[User]>,
    pub groups: Box<[Group]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct User {
    pub name: Box<[u8]>,
    pub task: Box<[u8]>,
    pub at: Position,
    pub kind: UserKind,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum UserKind {
    Sysusers {
        uid: Option<u32>,
        group: Box<[u8]>,
        gecos: Box<[u8]>,
        home: Box<[u8]>,
        shell: Box<[u8]>,
        subordinate: Option<(u32, u32)>,
        member_of: Box<[Box<[u8]>]>,
    },
    Package {
        uid: Option<u32>,
        gid: Option<u32>,
    },
}
pub fn accounts(users: &[Located<'_>], groups: &[Located<'_>], faults: &mut List<Error>) -> Accounts {
    let mut kept_users = List::with_capacity(u32::try_from(users.len()).expect("entry count"));
    let mut kept_groups = List::with_capacity(u32::try_from(groups.len()).expect("entry count"));
    for located in users {
        match check(located) {
            Ok(kind) => {
                kept_users
                    .push(User {
                        name: located.entry.key.clone(),
                        task: copy_of(located.task),
                        at: located.entry.at,
                        kind,
                    })
                    .expect("entry count");
            }
            Err((name, schema)) => {
                faults.push(fault(located, Some(name), schema)).expect("entry count");
            }
        }
    }
    for located in groups {
        match groups::entry(located) {
            Ok(group) => {
                kept_groups.push(group).expect("entry count");
            }
            Err(error) => {
                faults.push(*error).expect("entry count");
            }
        }
    }
    Accounts { users: kept_users.into_boxed(), groups: kept_groups.into_boxed() }
}
fn check(located: &Located<'_>) -> Result<UserKind, (Name, Schema)> {
    let entry = located.entry;
    let uid = id(entry, Name::Uid)?;
    if package(entry)? {
        for field in &entry.fields {
            if ![Name::From, Name::Uid, Name::Gid].contains(&field.name) {
                return Err((
                    field.name,
                    Schema::UndeclaredField {
                        field: copy_of(field.name.key()),
                        declared: Box::new([copy_of(b"from"), copy_of(b"uid"), copy_of(b"gid")]),
                    },
                ));
            }
        }
        return Ok(UserKind::Package { uid, gid: id(entry, Name::Gid)? });
    }
    if entry.int(Name::Gid).is_some() {
        return Err((Name::Gid, Schema::UndeclaredField { field: copy_of(b"gid"), declared: Box::new([]) }));
    }
    for name in [Name::Gecos, Name::Home, Name::Shell] {
        if let Some(value) = entry.text(name) {
            for byte in value {
                if [b':', b'"', b'\\'].contains(byte) || byte.is_ascii_control() {
                    return Err((name, Schema::Malformed { format: Format::Key }));
                }
            }
            if name != Name::Gecos
                && let Err(schema) = formed(value, Format::Path)
            {
                return Err((name, schema));
            }
        }
    }
    let mut members =
        Map::with_capacity(u32::try_from(entry.strings(Name::Groups).unwrap_or(&[]).len()).expect("member count"));
    for group in entry.strings(Name::Groups).unwrap_or(&[]) {
        members.insert(group.clone(), ()).expect("member count");
    }
    let mut member_of = List::with_capacity(members.len());
    for (group, ()) in &members {
        member_of.push(group.clone()).expect("member count");
    }
    let subordinate = match subordinate(entry.range(Name::Subordinate)) {
        Ok(range) => range,
        Err(schema) => return Err((Name::Subordinate, schema)),
    };
    Ok(UserKind::Sysusers {
        uid,
        group: text(entry, Name::Group, &entry.key),
        gecos: text(entry, Name::Gecos, b""),
        home: text(entry, Name::Home, b"/"),
        shell: text(entry, Name::Shell, b"/usr/sbin/nologin"),
        subordinate,
        member_of: member_of.into_boxed(),
    })
}
fn subordinate(range: Option<(i64, i64)>) -> Result<Option<(u32, u32)>, Schema> {
    let Some((base, count)) = range else { return Ok(None) };
    let base = bounded(base, 65536, i64::from(u32::MAX))?;
    let count = bounded(count, 1, i64::from(u32::MAX.checked_sub(base).expect("bounded base")))?;
    Ok(Some((base, count)))
}
