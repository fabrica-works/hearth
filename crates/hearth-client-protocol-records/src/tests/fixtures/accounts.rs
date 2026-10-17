use crate::{
    parts::config::*,
    records::Record,
    records::{Part, PartRecord},
};
use alloc::boxed::Box;
pub fn accounts_spec() -> Record {
    let mut spec = super::x1_prod();
    spec.parts = Box::new([Part {
        digest: [0; 32],
        record: PartRecord::Config(Config {
            tree: Box::new([]),
            groups: Box::new([
                Group { name: b"mail".as_slice().into(), kind: GroupKind::Package },
                Group { name: b"ops".as_slice().into(), kind: ops() },
            ]),
            users: Box::new([
                User { name: b"free".as_slice().into(), kind: account_user(1000, b"Free User") },
                User { name: b"postfix".as_slice().into(), kind: UserKind::Package },
            ]),
            members: Box::new([member(b"ops")]),
        }),
    }]);
    Record::Spec(spec)
}
pub fn accounts_change() -> Record {
    use crate::diff::{Change, ConfigChange, GroupKeyed, MemberChange, PartChange, Rebuild, TargetChange, UserKeyed};
    Record::Change(Change {
        target: b"x1-prod".as_slice().into(),
        change: TargetChange::Changed {
            rebuild: Rebuild::FromConfig,
            parts: Box::new([PartChange::Config(ConfigChange {
                tree: Box::new([]),
                groups: Box::new([GroupKeyed::Removed { key: b"ops".as_slice().into(), value: ops() }]),
                users: Box::new([UserKeyed::Changed {
                    key: b"free".as_slice().into(),
                    before: UserKind::Package,
                    after: account_user(1001, b""),
                }]),
                members: Box::new([MemberChange::Removed(member(b"ops")), MemberChange::Added(member(b"video"))]),
            })]),
        },
    })
}

fn account_user(uid: u32, gecos: &[u8]) -> UserKind {
    UserKind::Sysusers {
        uid: Some(uid),
        group: b"free".as_slice().into(),
        gecos: gecos.into(),
        home: b"/home/free".as_slice().into(),
        shell: b"/bin/bash".as_slice().into(),
    }
}
fn ops() -> GroupKind {
    GroupKind::Sysusers { gid: Some(1500) }
}
fn member(group: &[u8]) -> Member {
    Member { user: b"free".as_slice().into(), group: group.into() }
}
