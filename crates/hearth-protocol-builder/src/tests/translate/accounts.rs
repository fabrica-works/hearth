use super::config::{group_back, member_back, user_back};
use crate::{records::diff as target, render::parts::config as source};
pub(super) fn change_back(
    target::ConfigChange { tree, groups, users, members, presets, tmpfiles, scripts }: target::ConfigChange,
) -> source::Change {
    let mut entries = skein_lib::List::with_capacity(u32::try_from(tree.len()).expect("tree count"));
    for entry in tree {
        entries.push(super::entry_back(entry)).expect("tree count");
    }
    let mut changed_groups = skein_lib::List::with_capacity(u32::try_from(groups.len()).expect("entry count"));
    for row in groups {
        changed_groups
            .push(match row {
                target::GroupKeyed::Added { key, value } => source::GroupKeyed::Added { key, value: group_back(value) },
                target::GroupKeyed::Removed { key, value } => {
                    source::GroupKeyed::Removed { key, value: group_back(value) }
                }
                target::GroupKeyed::Changed { key, before, after } => {
                    source::GroupKeyed::Changed { key, before: group_back(before), after: group_back(after) }
                }
            })
            .expect("entry count");
    }
    let mut changed_users = skein_lib::List::with_capacity(u32::try_from(users.len()).expect("entry count"));
    for row in users {
        changed_users
            .push(match row {
                target::UserKeyed::Added { key, value } => source::UserKeyed::Added { key, value: user_back(value) },
                target::UserKeyed::Removed { key, value } => {
                    source::UserKeyed::Removed { key, value: user_back(value) }
                }
                target::UserKeyed::Changed { key, before, after } => {
                    source::UserKeyed::Changed { key, before: user_back(before), after: user_back(after) }
                }
            })
            .expect("entry count");
    }
    let mut changed_members = skein_lib::List::with_capacity(u32::try_from(members.len()).expect("member count"));
    for member in members {
        changed_members
            .push(match member {
                target::MemberChange::Added(value) => source::MemberChange::Added(member_back(value)),
                target::MemberChange::Removed(value) => source::MemberChange::Removed(member_back(value)),
            })
            .expect("member count");
    }
    source::Change {
        presets: super::directives::preset_changes_back(presets),
        tmpfiles: super::directives::line_changes_back(tmpfiles),
        scripts: super::directives::script_change_back(scripts),
        tree: entries.into_boxed(),
        groups: changed_groups.into_boxed(),
        users: changed_users.into_boxed(),
        members: changed_members.into_boxed(),
    }
}
#[test]
fn account_records_translate_to_records_and_back() {
    for (row, input) in super::super::fixtures::account_configs().into_iter().enumerate() {
        let back = super::config::config_back(crate::translate::parts::config::config(input.clone()));
        assert_eq!(back, input, "row {row}: every group, user and member");
    }
}
#[test]
fn account_changes_translate_to_records_and_back() {
    for (row, input) in super::super::fixtures::account_changes().into_iter().enumerate() {
        let records::diff::PartChange::Config(translated) =
            crate::translate::changes::part_change(render::diff::PartChange::Config(input.clone()))
        else {
            panic!("config change")
        };
        let back = change_back(translated);
        assert_eq!(back, input, "row {row}: every account change");
    }
}
use crate::{records, render};
