use crate::{records, render};
pub(super) fn config_back(config: records::parts::config::Config) -> render::parts::config::Config {
    let mut tree = skein_lib::List::with_capacity(u32::try_from(config.tree.len()).expect("bounded tree"));
    for records::parts::config::TreeEntry { path, node } in config.tree {
        let node = super::node_back(node);
        tree.push(render::parts::config::TreeEntry { path, node }).expect("tree room");
    }
    let mut groups = skein_lib::List::with_capacity(u32::try_from(config.groups.len()).expect("entry count"));
    for records::parts::config::Group { name, kind } in config.groups {
        groups.push(render::parts::config::Group { name, kind: group_back(kind) }).expect("entry count");
    }
    let mut users = skein_lib::List::with_capacity(u32::try_from(config.users.len()).expect("entry count"));
    for records::parts::config::User { name, kind } in config.users {
        users.push(render::parts::config::User { name, kind: user_back(kind) }).expect("entry count");
    }
    let mut members = skein_lib::List::with_capacity(u32::try_from(config.members.len()).expect("member count"));
    for member in config.members {
        members.push(member_back(member)).expect("member count");
    }
    render::parts::config::Config {
        presets: super::directives::presets_back(config.presets),
        tmpfiles: super::directives::tmpfiles_back(config.tmpfiles),
        scripts: super::directives::scripts_back(config.scripts),
        groups: groups.into_boxed(),
        users: users.into_boxed(),
        members: members.into_boxed(),
        tree: tree.into_boxed(),
    }
}

pub(super) fn part_back(value: records::parts::config::Config) -> render::parts::PartRecord {
    render::parts::PartRecord::Config(config_back(value))
}

pub(super) fn group_back(kind: records::parts::config::GroupKind) -> render::parts::config::GroupKind {
    use records::parts::config::GroupKind as From;
    use render::parts::config::GroupKind as To;
    match kind {
        From::Package => To::Package,
        From::Sysusers { gid } => To::Sysusers { gid },
    }
}
pub(super) fn user_back(kind: records::parts::config::UserKind) -> render::parts::config::UserKind {
    use records::parts::config::UserKind as From;
    use render::parts::config::UserKind as To;
    match kind {
        From::Package => To::Package,
        From::Sysusers { uid, group, gecos, home, shell } => To::Sysusers { uid, group, gecos, home, shell },
    }
}
pub(super) fn member_back(
    records::parts::config::Member { user, group }: records::parts::config::Member,
) -> render::parts::config::Member {
    render::parts::config::Member { user, group }
}
