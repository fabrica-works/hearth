use super::{Config, GroupKeyed, MemberChange, UserKeyed};
use alloc::boxed::Box;
use skein_lib::{List, Map};
pub(super) fn groups(before: Option<&Config>, after: Option<&Config>) -> Box<[GroupKeyed]> {
    let before = match before {
        Some(config) => config.groups.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.groups.as_ref(),
        None => &[],
    };
    let capacity = u32::try_from(before.len().saturating_add(after.len())).expect("entry count");
    let mut entries = Map::with_capacity(capacity);
    for row in before {
        entries.insert(row.name.clone(), (Some(&row.kind), None)).expect("entry count");
    }
    for row in after {
        if let Some((_, value)) = entries.get_mut(&row.name) {
            *value = Some(&row.kind);
        } else {
            entries.insert(row.name.clone(), (None, Some(&row.kind))).expect("entry count");
        }
    }
    let mut changes = List::with_capacity(capacity);
    for (key, (old, new)) in &entries {
        let change = match (old, new) {
            (None, Some(value)) => GroupKeyed::Added { key: key.clone(), value: (*value).clone() },
            (Some(value), None) => GroupKeyed::Removed { key: key.clone(), value: (*value).clone() },
            (Some(before), Some(after)) if before != after => {
                GroupKeyed::Changed { key: key.clone(), before: (*before).clone(), after: (*after).clone() }
            }
            (Some(_), Some(_)) => continue,
            (None, None) => unreachable!("union entry"),
        };
        changes.push(change).expect("entry count");
    }
    changes.into_boxed()
}
pub(super) fn users(before: Option<&Config>, after: Option<&Config>) -> Box<[UserKeyed]> {
    let before = match before {
        Some(config) => config.users.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.users.as_ref(),
        None => &[],
    };
    let capacity = u32::try_from(before.len().saturating_add(after.len())).expect("entry count");
    let mut entries = Map::with_capacity(capacity);
    for row in before {
        entries.insert(row.name.clone(), (Some(&row.kind), None)).expect("entry count");
    }
    for row in after {
        if let Some((_, value)) = entries.get_mut(&row.name) {
            *value = Some(&row.kind);
        } else {
            entries.insert(row.name.clone(), (None, Some(&row.kind))).expect("entry count");
        }
    }
    let mut changes = List::with_capacity(capacity);
    for (key, (old, new)) in &entries {
        let change = match (old, new) {
            (None, Some(value)) => UserKeyed::Added { key: key.clone(), value: (*value).clone() },
            (Some(value), None) => UserKeyed::Removed { key: key.clone(), value: (*value).clone() },
            (Some(before), Some(after)) if before != after => {
                UserKeyed::Changed { key: key.clone(), before: (*before).clone(), after: (*after).clone() }
            }
            (Some(_), Some(_)) => continue,
            (None, None) => unreachable!("union entry"),
        };
        changes.push(change).expect("entry count");
    }
    changes.into_boxed()
}
pub(super) fn members(before: Option<&Config>, after: Option<&Config>) -> Box<[MemberChange]> {
    let before = match before {
        Some(config) => config.members.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.members.as_ref(),
        None => &[],
    };
    let mut changes =
        List::with_capacity(u32::try_from(before.len().saturating_add(after.len())).expect("member count"));
    for member in before {
        if !after.contains(member) {
            changes.push(MemberChange::Removed(member.clone())).expect("member count");
        }
    }
    for member in after {
        if !before.contains(member) {
            changes.push(MemberChange::Added(member.clone())).expect("member count");
        }
    }
    changes.into_boxed()
}
