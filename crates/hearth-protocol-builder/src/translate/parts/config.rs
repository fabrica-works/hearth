//! Static trees translated without interpreting them (render.md, 4.2).
use crate::{records::parts::config as target, render::parts::config as source};
#[must_use]
pub fn config(config: source::Config) -> target::Config {
    let source::Config { tree, groups, users, members } = config;
    let mut translated = skein_lib::List::with_capacity(u32::try_from(tree.len()).expect("bounded tree"));
    for source::TreeEntry { path, node } in tree {
        let node = match node {
            source::Node::Link { target, owner, group } => target::Node::Link { target, owner, group },
            source::Node::Directory { mode, owner, group } => target::Node::Directory { mode, owner, group },
            source::Node::File { content_digest, mode, owner, group } => {
                target::Node::File { content_digest: content_digest.0, mode, owner, group }
            }
        };
        translated.push(target::TreeEntry { path, node }).expect("tree capacity");
    }
    let mut translated_groups = skein_lib::List::with_capacity(u32::try_from(groups.len()).expect("entry count"));
    for source::Group { name, kind } in groups {
        translated_groups.push(target::Group { name, kind: group(kind) }).expect("entry count");
    }
    let mut translated_users = skein_lib::List::with_capacity(u32::try_from(users.len()).expect("entry count"));
    for source::User { name, kind } in users {
        translated_users.push(target::User { name, kind: user(kind) }).expect("entry count");
    }
    let mut translated_members = skein_lib::List::with_capacity(u32::try_from(members.len()).expect("entry count"));
    for row in members {
        translated_members.push(member(row)).expect("entry count");
    }
    target::Config {
        tree: translated.into_boxed(),
        groups: translated_groups.into_boxed(),
        users: translated_users.into_boxed(),
        members: translated_members.into_boxed(),
    }
}

#[must_use]
pub fn group(kind: source::GroupKind) -> target::GroupKind {
    match kind {
        source::GroupKind::Package => target::GroupKind::Package,
        source::GroupKind::Sysusers { gid } => target::GroupKind::Sysusers { gid },
    }
}
#[must_use]
pub fn user(kind: source::UserKind) -> target::UserKind {
    match kind {
        source::UserKind::Package => target::UserKind::Package,
        source::UserKind::Sysusers { uid, group, gecos, home, shell } => {
            target::UserKind::Sysusers { uid, group, gecos, home, shell }
        }
    }
}
#[must_use]
pub fn member(source::Member { user, group }: source::Member) -> target::Member {
    target::Member { user, group }
}
