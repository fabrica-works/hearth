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

use crate::form::text::Lines;
use skein_json::writer::Encoder;
impl GroupKind {
    pub(crate) fn text(&self, lines: &mut Lines) {
        match self {
            Self::Package => lines.put(b"package"),
            Self::Sysusers { gid } => id_text(*gid, lines),
        }
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        json.object_start();
        match self {
            Self::Package => {
                json.key(b"kind");
                json.string(b"package");
            }
            Self::Sysusers { gid } => {
                if let Some(gid) = gid {
                    json.key(b"gid");
                    json.number(skein_lib::Decimal::of(u64::from(*gid)).as_bytes());
                }
                json.key(b"kind");
                json.string(b"sysusers");
            }
        }
        json.object_end();
    }
}
impl UserKind {
    pub(crate) fn text(&self, lines: &mut Lines) {
        match self {
            Self::Package => lines.put(b"package"),
            Self::Sysusers { uid, group, gecos, home, shell } => {
                id_text(*uid, lines);
                for value in [group, home, shell] {
                    lines.put(b" ");
                    lines.put(value);
                }
                if !gecos.is_empty() {
                    lines.put(b" ");
                    lines.put(gecos);
                }
            }
        }
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        json.object_start();
        match self {
            Self::Package => {
                json.key(b"kind");
                json.string(b"package");
            }
            Self::Sysusers { uid, group, gecos, home, shell } => {
                json.key(b"gecos");
                json.string(gecos);
                json.key(b"group");
                json.string(group);
                json.key(b"home");
                json.string(home);
                json.key(b"kind");
                json.string(b"sysusers");
                json.key(b"shell");
                json.string(shell);
                if let Some(uid) = uid {
                    json.key(b"uid");
                    json.number(skein_lib::Decimal::of(u64::from(*uid)).as_bytes());
                }
            }
        }
        json.object_end();
    }
}
fn id_text(id: Option<u32>, lines: &mut Lines) {
    match id {
        Some(id) => lines.decimal(u64::from(id)),
        None => lines.put(b"-"),
    }
}
impl super::Config {
    pub(super) fn accounts_text(&self, lines: &mut Lines) {
        for group in &self.groups {
            lines.put(b"  groups     ");
            lines.put(&group.name);
            lines.put(b" ");
            group.kind.text(lines);
            lines.put(b"\n");
        }
        for user in &self.users {
            lines.put(b"  users      ");
            lines.put(&user.name);
            lines.put(b" ");
            user.kind.text(lines);
            lines.put(b"\n");
        }
        for member in &self.members {
            lines.put(b"  members    ");
            lines.put(&member.user);
            lines.put(b" ");
            lines.put(&member.group);
            lines.put(b"\n");
        }
    }
    pub(super) fn accounts_json(&self, json: &mut Encoder) {
        json.key(b"groups");
        json.object_start();
        for group in &self.groups {
            json.key(&group.name);
            group.kind.json(json);
        }
        json.object_end();
        json.key(b"members");
        json.array_start();
        for member in &self.members {
            member.json(json);
        }
        json.array_end();
    }
    pub(super) fn users_json(&self, json: &mut Encoder) {
        json.key(b"users");
        json.object_start();
        for user in &self.users {
            json.key(&user.name);
            user.kind.json(json);
        }
        json.object_end();
    }
}
impl Member {
    pub(crate) fn json(&self, json: &mut Encoder) {
        json.object_start();
        json.key(b"group");
        json.string(&self.group);
        json.key(b"user");
        json.string(&self.user);
        json.object_end();
    }
}
