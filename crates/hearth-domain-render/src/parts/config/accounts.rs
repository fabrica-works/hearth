use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Group {
    pub name: Box<[u8]>,
    pub kind: GroupKind,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GroupKind {
    Sysusers { gid: Option<u32> },
    Package,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct User {
    pub name: Box<[u8]>,
    pub kind: UserKind,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum UserKind {
    Sysusers { uid: Option<u32>, group: Box<[u8]>, gecos: Box<[u8]>, home: Box<[u8]>, shell: Box<[u8]> },
    Package,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Member {
    pub user: Box<[u8]>,
    pub group: Box<[u8]>,
}

use super::Config;
use crate::{entries::Expansion, errors::Error, kinds::users, revision::declaration::Map};
use skein_lib::{List, Map as Ordered};
pub fn build(expansion: &Expansion) -> Result<users::Accounts, Box<[Error]>> {
    let users = expansion.of(Map::Users);
    let groups = expansion.of(Map::Groups);
    let mut faults = List::with_capacity(u32::try_from(users.len().saturating_add(groups.len())).expect("entry count"));
    let accounts = users::accounts(&users, &groups, &mut faults);
    if faults.is_empty() { Ok(accounts) } else { Err(faults.into_boxed()) }
}
pub(crate) fn rows(accounts: &users::Accounts, tree: Box<[super::TreeEntry]>) -> Config {
    let mut groups = Ordered::with_capacity(u32::try_from(accounts.groups.len()).expect("entry count"));
    let mut users = Ordered::with_capacity(u32::try_from(accounts.users.len()).expect("entry count"));
    let mut capacity = 0_u32;
    for user in &accounts.users {
        if let users::UserKind::Sysusers { member_of, .. } = &user.kind {
            capacity = capacity.saturating_add(u32::try_from(member_of.len()).expect("member count"));
        }
    }
    let mut members = Ordered::with_capacity(capacity);
    for group in &accounts.groups {
        let kind = if group.package { GroupKind::Package } else { GroupKind::Sysusers { gid: group.gid } };
        groups.insert(group.name.clone(), Group { name: group.name.clone(), kind }).expect("entry count");
    }
    for user in &accounts.users {
        let kind = match &user.kind {
            users::UserKind::Package { .. } => UserKind::Package,
            users::UserKind::Sysusers { uid, group, gecos, home, shell, member_of, .. } => {
                for group in member_of {
                    members.insert(Member { user: user.name.clone(), group: group.clone() }, ()).expect("member count");
                }
                UserKind::Sysusers {
                    uid: *uid,
                    group: group.clone(),
                    gecos: gecos.clone(),
                    home: home.clone(),
                    shell: shell.clone(),
                }
            }
        };
        users.insert(user.name.clone(), User { name: user.name.clone(), kind }).expect("entry count");
    }
    let mut group_rows = List::with_capacity(groups.len());
    for (_, group) in &groups {
        group_rows.push(group.clone()).expect("entry count");
    }
    let mut user_rows = List::with_capacity(users.len());
    for (_, user) in &users {
        user_rows.push(user.clone()).expect("entry count");
    }
    let mut member_rows = List::with_capacity(members.len());
    for (member, ()) in &members {
        member_rows.push(member.clone()).expect("member count");
    }
    Config {
        tree,
        groups: group_rows.into_boxed(),
        users: user_rows.into_boxed(),
        members: member_rows.into_boxed(),
        presets: Box::new([]),
        tmpfiles: Box::new([]),
        scripts: Box::new([]),
    }
}
pub(super) fn encode(config: &Config, record: &mut hearth_lib::encode::Encoder) {
    record.key(b"groups");
    record.map(u64::try_from(config.groups.len()).expect("entry count"));
    for group in &config.groups {
        record.key(&group.name);
        group.kind.encode(record);
    }
    record.key(b"members");
    record.list(u64::try_from(config.members.len()).expect("member count"));
    for member in &config.members {
        record.map(2);
        record.key(b"group");
        record.bytes(&member.group);
        record.key(b"user");
        record.bytes(&member.user);
    }
}
impl GroupKind {
    fn encode(&self, record: &mut hearth_lib::encode::Encoder) {
        match self {
            Self::Package => {
                record.map(1);
                record.key(b"kind");
                record.bytes(b"package");
            }
            Self::Sysusers { gid } => {
                record.map(if gid.is_some() { 2 } else { 1 });
                if let Some(gid) = gid {
                    record.key(b"gid");
                    record.uint(u64::from(*gid));
                }
                record.key(b"kind");
                record.bytes(b"sysusers");
            }
        }
    }
}
pub(super) fn encode_users(config: &Config, record: &mut hearth_lib::encode::Encoder) {
    record.key(b"users");
    record.map(u64::try_from(config.users.len()).expect("entry count"));
    for user in &config.users {
        record.key(&user.name);
        match &user.kind {
            UserKind::Package => {
                record.map(1);
                record.key(b"kind");
                record.bytes(b"package");
            }
            UserKind::Sysusers { uid, group, gecos, home, shell } => {
                record.map(if uid.is_some() { 6 } else { 5 });
                record.key(b"gecos");
                record.bytes(gecos);
                record.key(b"group");
                record.bytes(group);
                record.key(b"home");
                record.bytes(home);
                record.key(b"kind");
                record.bytes(b"sysusers");
                record.key(b"shell");
                record.bytes(shell);
                if let Some(uid) = uid {
                    record.key(b"uid");
                    record.uint(u64::from(*uid));
                }
            }
        }
    }
}
