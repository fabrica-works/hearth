//! Secret access must exclude hearth and every group it belongs to (format.md, 9.1).
use super::{Refused, refused};
use crate::{
    entries::{Expansion, Located},
    errors::{Place, Rule},
    revision::declaration::{Map, Name},
};
fn hearth(expansion: &Expansion, group: &[u8]) -> (bool, Option<Place>) {
    let mut belongs = group == b"hearth" || group == b"systemd-journal";
    let mut at = None;
    for user in expansion.of(Map::Users) {
        if user.entry.key.as_ref() == b"hearth" {
            at = Some(Place { path: super::declaration(user.task), position: Some(user.entry.at) });
            belongs |= user.entry.text(Name::Group) == Some(group);
            for name in user.entry.strings(Name::Groups).unwrap_or(&[]) {
                belongs |= name.as_ref() == group;
            }
        }
    }
    (belongs, at)
}
pub(super) fn entry(row: &Located<'_>, map: Map, expansion: &Expansion) -> Option<Refused> {
    if map != Map::Secrets && map != Map::Identity {
        return None;
    }
    let mut declared_group = false;
    for field in &row.entry.fields {
        declared_group |= field.name == Name::Group;
    }
    if !declared_group {
        let (belongs, also) = hearth(expansion, b"root");
        if belongs {
            let mut fault = refused(row, row.entry.at, Rule::HearthOwner, &row.entry.key);
            fault.also = also;
            return Some(fault);
        }
    }
    for field in &row.entry.fields {
        let value = row.entry.text(field.name).unwrap_or(b"");
        let (group, also) = hearth(expansion, value);
        let rule = if field.name == Name::Mode
            && let Ok(mode) = crate::kinds::secrets::mode(row.entry)
            && mode & !0o770 != 0
        {
            Some(Rule::SecretMode)
        } else if (field.name == Name::Owner && value == b"hearth") || (field.name == Name::Group && group) {
            Some(Rule::HearthOwner)
        } else {
            None
        };
        if let Some(rule) = rule {
            let mut fault = refused(row, field.at, rule, &row.entry.key);
            if rule == Rule::HearthOwner {
                fault.also = also;
            }
            return Some(fault);
        }
    }
    None
}
