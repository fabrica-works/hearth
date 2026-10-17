use crate::render::parts::config::*;
use alloc::boxed::Box;
#[must_use]
pub fn account_configs() -> [Config; 2] {
    [
        Config {
            presets: Box::new([]),
            tmpfiles: Box::new([]),
            scripts: Box::new([]),
            groups: Box::new([
                Group { name: b"ops".as_slice().into(), kind: GroupKind::Sysusers { gid: Some(1500) } },
                Group { name: b"mail".as_slice().into(), kind: GroupKind::Package },
            ]),
            users: Box::new([
                User { name: b"free".as_slice().into(), kind: sysuser() },
                User { name: b"postfix".as_slice().into(), kind: UserKind::Package },
            ]),
            members: Box::new([member()]),
            tree: Box::new([]),
        },
        Config {
            presets: Box::new([]),
            tmpfiles: Box::new([]),
            scripts: Box::new([]),
            groups: Box::new([Group { name: b"ci".as_slice().into(), kind: GroupKind::Sysusers { gid: None } }]),
            users: Box::new([]),
            members: Box::new([]),
            tree: Box::new([]),
        },
    ]
}
fn sysuser() -> UserKind {
    UserKind::Sysusers {
        uid: Some(1000),
        group: b"free".as_slice().into(),
        gecos: b"Free User".as_slice().into(),
        home: b"/home/free".as_slice().into(),
        shell: b"/bin/bash".as_slice().into(),
    }
}
fn member() -> Member {
    Member { user: b"free".as_slice().into(), group: b"ops".as_slice().into() }
}
#[must_use]
pub fn account_changes() -> [Change; 2] {
    [
        Change {
            presets: Box::new([]),
            tmpfiles: Box::new([]),
            scripts: None,
            tree: Box::new([]),
            groups: Box::new([
                GroupKeyed::Added { key: b"ops".as_slice().into(), value: GroupKind::Sysusers { gid: None } },
                GroupKeyed::Removed { key: b"mail".as_slice().into(), value: GroupKind::Package },
                GroupKeyed::Changed {
                    key: b"staff".as_slice().into(),
                    before: GroupKind::Sysusers { gid: Some(1500) },
                    after: GroupKind::Package,
                },
            ]),
            users: Box::new([
                UserKeyed::Added { key: b"free".as_slice().into(), value: sysuser() },
                UserKeyed::Removed { key: b"postfix".as_slice().into(), value: UserKind::Package },
                UserKeyed::Changed { key: b"ci".as_slice().into(), before: UserKind::Package, after: sysuser() },
            ]),
            members: Box::new([MemberChange::Added(member()), MemberChange::Removed(member())]),
        },
        Change {
            presets: Box::new([]),
            tmpfiles: Box::new([]),
            scripts: None,
            tree: Box::new([]),
            groups: Box::new([]),
            users: Box::new([]),
            members: Box::new([]),
        },
    ]
}
