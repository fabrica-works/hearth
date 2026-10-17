use super::super::parts::config::{action, line, scripts as script_rows};
use crate::{records::diff as target, render::parts::config as source};
use alloc::boxed::Box;
use skein_lib::List;
pub(super) fn presets(entries: Box<[source::PresetKeyed]>) -> Box<[target::PresetKeyed]> {
    let mut translated = List::with_capacity(u32::try_from(entries.len()).expect("entry bound"));
    for entry in entries {
        translated
            .push(match entry {
                source::PresetKeyed::Added { key, value } => target::PresetKeyed::Added { key, value: action(value) },
                source::PresetKeyed::Removed { key, value } => {
                    target::PresetKeyed::Removed { key, value: action(value) }
                }
                source::PresetKeyed::Changed { key, before, after } => {
                    target::PresetKeyed::Changed { key, before: action(before), after: action(after) }
                }
            })
            .expect("entry bound");
    }
    translated.into_boxed()
}
pub(super) fn tmpfiles(entries: Box<[source::LineKeyed]>) -> Box<[target::LineKeyed]> {
    let mut translated = List::with_capacity(u32::try_from(entries.len()).expect("entry bound"));
    for entry in entries {
        translated
            .push(match entry {
                source::LineKeyed::Added { key, value } => target::LineKeyed::Added { key, value: line(value) },
                source::LineKeyed::Removed { key, value } => target::LineKeyed::Removed { key, value: line(value) },
                source::LineKeyed::Changed { key, before, after } => {
                    target::LineKeyed::Changed { key, before: line(before), after: line(after) }
                }
            })
            .expect("entry bound");
    }
    translated.into_boxed()
}
pub(super) fn scripts(change: Option<source::ScriptsChange>) -> Option<target::ScriptsChange> {
    let source::ScriptsChange { before, after } = change?;
    Some(target::ScriptsChange { before: script_rows(before), after: script_rows(after) })
}
