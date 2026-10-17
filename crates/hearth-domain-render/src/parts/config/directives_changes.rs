use super::{Config, LineKeyed, PresetKeyed, ScriptsChange};
use alloc::boxed::Box;
use skein_lib::{List, Map};
pub(super) fn presets(before: Option<&Config>, after: Option<&Config>) -> Box<[PresetKeyed]> {
    let before = match before {
        Some(config) => config.presets.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.presets.as_ref(),
        None => &[],
    };
    let capacity = u32::try_from(before.len().saturating_add(after.len())).expect("entry bound");
    let mut entries = Map::with_capacity(capacity);
    for row in before {
        entries.insert(row.unit.clone(), (Some(&row.action), None)).expect("entry bound");
    }
    for row in after {
        if let Some((_, value)) = entries.get_mut(&row.unit) {
            *value = Some(&row.action);
        } else {
            entries.insert(row.unit.clone(), (None, Some(&row.action))).expect("entry bound");
        }
    }
    let mut changes = List::with_capacity(capacity);
    for (key, (before, after)) in &entries {
        let change = match (before, after) {
            (None, Some(value)) => PresetKeyed::Added { key: key.clone(), value: **value },
            (Some(value), None) => PresetKeyed::Removed { key: key.clone(), value: **value },
            (Some(before), Some(after)) if before != after => {
                PresetKeyed::Changed { key: key.clone(), before: **before, after: **after }
            }
            (Some(_), Some(_)) => continue,
            (None, None) => unreachable!("union entry"),
        };
        changes.push(change).expect("entry bound");
    }
    changes.into_boxed()
}
pub(super) fn tmpfiles(before: Option<&Config>, after: Option<&Config>) -> Box<[LineKeyed]> {
    let before = match before {
        Some(config) => config.tmpfiles.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.tmpfiles.as_ref(),
        None => &[],
    };
    let capacity = u32::try_from(before.len().saturating_add(after.len())).expect("entry bound");
    let mut entries = Map::with_capacity(capacity);
    for row in before {
        entries.insert(row.path.clone(), (Some(&row.line), None)).expect("entry bound");
    }
    for row in after {
        if let Some((_, value)) = entries.get_mut(&row.path) {
            *value = Some(&row.line);
        } else {
            entries.insert(row.path.clone(), (None, Some(&row.line))).expect("entry bound");
        }
    }
    let mut changes = List::with_capacity(capacity);
    for (key, (before, after)) in &entries {
        let change = match (before, after) {
            (None, Some(value)) => LineKeyed::Added { key: key.clone(), value: (*value).clone() },
            (Some(value), None) => LineKeyed::Removed { key: key.clone(), value: (*value).clone() },
            (Some(before), Some(after)) if before != after => {
                LineKeyed::Changed { key: key.clone(), before: (*before).clone(), after: (*after).clone() }
            }
            (Some(_), Some(_)) => continue,
            (None, None) => unreachable!("union entry"),
        };
        changes.push(change).expect("entry bound");
    }
    changes.into_boxed()
}
pub(super) fn scripts(before: Option<&Config>, after: Option<&Config>) -> Option<ScriptsChange> {
    let before = match before {
        Some(config) => config.scripts.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.scripts.as_ref(),
        None => &[],
    };
    if before == after { None } else { Some(ScriptsChange { before: Box::from(before), after: Box::from(after) }) }
}
