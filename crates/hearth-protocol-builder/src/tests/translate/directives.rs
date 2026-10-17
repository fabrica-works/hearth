use crate::{records::parts::config as source, render::parts::config as target};
use alloc::boxed::Box;
use skein_lib::List;
pub(super) const fn action_back(value: source::Action) -> target::Action {
    match value {
        source::Action::Enable => target::Action::Enable,
        source::Action::Disable => target::Action::Disable,
    }
}
pub(super) fn line_back(line: source::Line) -> target::Line {
    let source::Line { kind, mode, owner, group, age, argument } = line;
    let kind = match kind {
        source::LineKind::D => target::LineKind::D,
        source::LineKind::CapitalD => target::LineKind::CapitalD,
        source::LineKind::E => target::LineKind::E,
        source::LineKind::F => target::LineKind::F,
        source::LineKind::FPlus => target::LineKind::FPlus,
        source::LineKind::W => target::LineKind::W,
        source::LineKind::WPlus => target::LineKind::WPlus,
        source::LineKind::CapitalL => target::LineKind::CapitalL,
        source::LineKind::CapitalLPlus => target::LineKind::CapitalLPlus,
        source::LineKind::CapitalC => target::LineKind::CapitalC,
        source::LineKind::Z => target::LineKind::Z,
        source::LineKind::CapitalZ => target::LineKind::CapitalZ,
        source::LineKind::R => target::LineKind::R,
        source::LineKind::CapitalR => target::LineKind::CapitalR,
    };
    target::Line { kind, mode, owner, group, age, argument }
}
pub(super) fn presets_back(entries: Box<[source::Preset]>) -> Box<[target::Preset]> {
    let mut rows = List::with_capacity(u32::try_from(entries.len()).expect("fixture count"));
    for source::Preset { unit, action } in entries {
        rows.push(target::Preset { unit, action: action_back(action) }).expect("fixture count");
    }
    rows.into_boxed()
}
pub(super) fn tmpfiles_back(entries: Box<[source::Tmpfile]>) -> Box<[target::Tmpfile]> {
    let mut rows = List::with_capacity(u32::try_from(entries.len()).expect("fixture count"));
    for source::Tmpfile { path, line } in entries {
        rows.push(target::Tmpfile { path, line: line_back(line) }).expect("fixture count");
    }
    rows.into_boxed()
}
pub(super) fn scripts_back(entries: Box<[source::Script]>) -> Box<[target::Script]> {
    let mut rows = List::with_capacity(u32::try_from(entries.len()).expect("fixture count"));
    for source::Script { task, order, content_digest } in entries {
        rows.push(target::Script { task, order, content_digest: hearth_lib::encode::Digest(content_digest) })
            .expect("fixture count");
    }
    rows.into_boxed()
}
use crate::{records::diff as records, render::parts::config as render};
pub(super) fn preset_changes_back(entries: Box<[records::PresetKeyed]>) -> Box<[render::PresetKeyed]> {
    let mut rows = List::with_capacity(u32::try_from(entries.len()).expect("fixture count"));
    for entry in entries {
        rows.push(match entry {
            records::PresetKeyed::Added { key, value } => render::PresetKeyed::Added { key, value: action_back(value) },
            records::PresetKeyed::Removed { key, value } => {
                render::PresetKeyed::Removed { key, value: action_back(value) }
            }
            records::PresetKeyed::Changed { key, before, after } => {
                render::PresetKeyed::Changed { key, before: action_back(before), after: action_back(after) }
            }
        })
        .expect("fixture count");
    }
    rows.into_boxed()
}
pub(super) fn line_changes_back(entries: Box<[records::LineKeyed]>) -> Box<[render::LineKeyed]> {
    let mut rows = List::with_capacity(u32::try_from(entries.len()).expect("fixture count"));
    for entry in entries {
        rows.push(match entry {
            records::LineKeyed::Added { key, value } => render::LineKeyed::Added { key, value: line_back(value) },
            records::LineKeyed::Removed { key, value } => render::LineKeyed::Removed { key, value: line_back(value) },
            records::LineKeyed::Changed { key, before, after } => {
                render::LineKeyed::Changed { key, before: line_back(before), after: line_back(after) }
            }
        })
        .expect("fixture count");
    }
    rows.into_boxed()
}
pub(super) fn script_change_back(change: Option<records::ScriptsChange>) -> Option<render::ScriptsChange> {
    let records::ScriptsChange { before, after } = change?;
    Some(render::ScriptsChange { before: scripts_back(before), after: scripts_back(after) })
}
#[test]
fn config_settings_translate_to_records_and_back() {
    for (row, input) in super::super::fixtures::directives_configs().into_iter().enumerate() {
        let back = super::config::config_back(crate::translate::parts::config::config(input.clone()));
        assert_eq!(back, input, "row {row}: every setting");
    }
}
#[test]
fn config_settings_changes_translate_to_records_and_back() {
    for (row, input) in super::super::fixtures::directives_changes().into_iter().enumerate() {
        let crate::records::diff::PartChange::Config(translated) =
            crate::translate::changes::part_change(crate::render::diff::PartChange::Config(input.clone()))
        else {
            panic!("config change")
        };
        let back = super::accounts::change_back(translated);
        assert_eq!(back, input, "row {row}: every setting change");
    }
}
