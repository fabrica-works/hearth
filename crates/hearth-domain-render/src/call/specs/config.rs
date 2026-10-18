use super::{ELEMENT, bytes, optional, record};
use crate::parts::config::{Config, GroupKind, Node, Script, UserKind};
pub(super) fn config(config: &Config) -> u64 {
    let mut held = record(&[
        (b"tree", 0),
        (b"groups", 0),
        (b"users", 0),
        (b"members", 0),
        (b"presets", 0),
        (b"tmpfiles", 0),
        (b"scripts", scripts(&config.scripts)),
    ]);
    for entry in &config.tree {
        let node = match &entry.node {
            Node::Link { target, owner, group } => {
                record(&[(b"target", bytes(target)), (b"owner", bytes(owner)), (b"group", bytes(group))])
            }
            Node::Directory { owner, group, .. } => {
                record(&[(b"mode", 0), (b"owner", bytes(owner)), (b"group", bytes(group))])
            }
            Node::File { owner, group, .. } => {
                record(&[(b"content_digest", 32), (b"mode", 0), (b"owner", bytes(owner)), (b"group", bytes(group))])
            }
        };
        held = held.saturating_add(ELEMENT).saturating_add(record(&[(b"path", bytes(&entry.path)), (b"node", node)]));
    }
    for group in &config.groups {
        let kind = match group.kind {
            GroupKind::Package => bytes(b"package"),
            GroupKind::Sysusers { .. } => record(&[(b"gid", 0)]),
        };
        held = held.saturating_add(ELEMENT).saturating_add(record(&[(b"name", bytes(&group.name)), (b"kind", kind)]));
    }
    for user in &config.users {
        let kind = match &user.kind {
            UserKind::Package => bytes(b"package"),
            UserKind::Sysusers { group, gecos, home, shell, .. } => record(&[
                (b"uid", 0),
                (b"group", bytes(group)),
                (b"gecos", bytes(gecos)),
                (b"home", bytes(home)),
                (b"shell", bytes(shell)),
            ]),
        };
        held = held.saturating_add(ELEMENT).saturating_add(record(&[(b"name", bytes(&user.name)), (b"kind", kind)]));
    }
    for member in &config.members {
        held = held
            .saturating_add(ELEMENT)
            .saturating_add(record(&[(b"user", bytes(&member.user)), (b"group", bytes(&member.group))]));
    }
    for preset in &config.presets {
        held = held
            .saturating_add(ELEMENT)
            .saturating_add(record(&[(b"unit", bytes(&preset.unit)), (b"action", bytes(preset.action.name()))]));
    }
    for tmpfile in &config.tmpfiles {
        let line = &tmpfile.line;
        let line = record(&[
            (b"kind", bytes(line.kind.name())),
            (b"mode", 0),
            (b"owner", optional(line.owner.as_deref())),
            (b"group", optional(line.group.as_deref())),
            (b"age", optional(line.age.as_deref())),
            (b"argument", optional(line.argument.as_deref())),
        ]);
        held = held.saturating_add(ELEMENT).saturating_add(record(&[(b"path", bytes(&tmpfile.path)), (b"line", line)]));
    }
    held
}
pub(super) fn scripts(scripts: &[Script]) -> u64 {
    let mut held = 0_u64;
    for script in scripts {
        held = held.saturating_add(ELEMENT).saturating_add(record(&[
            (b"task", bytes(&script.task)),
            (b"order", 0),
            (b"content_digest", 32),
        ]));
    }
    held
}
