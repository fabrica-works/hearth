use crate::{records::parts::config as target, render::parts::config as source};
use alloc::boxed::Box;
use skein_lib::List;
#[must_use]
pub const fn action(action: source::Action) -> target::Action {
    match action {
        source::Action::Enable => target::Action::Enable,
        source::Action::Disable => target::Action::Disable,
    }
}
#[must_use]
pub fn line(line: source::Line) -> target::Line {
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
#[must_use]
pub fn presets(entries: Box<[source::Preset]>) -> Box<[target::Preset]> {
    let mut translated = List::with_capacity(u32::try_from(entries.len()).expect("entry bound"));
    for source::Preset { unit, action: value } in entries {
        translated.push(target::Preset { unit, action: action(value) }).expect("entry bound");
    }
    translated.into_boxed()
}
#[must_use]
pub fn tmpfiles(entries: Box<[source::Tmpfile]>) -> Box<[target::Tmpfile]> {
    let mut translated = List::with_capacity(u32::try_from(entries.len()).expect("entry bound"));
    for source::Tmpfile { path, line: value } in entries {
        translated.push(target::Tmpfile { path, line: line(value) }).expect("entry bound");
    }
    translated.into_boxed()
}
#[must_use]
pub fn scripts(entries: Box<[source::Script]>) -> Box<[target::Script]> {
    let mut translated = List::with_capacity(u32::try_from(entries.len()).expect("entry bound"));
    for source::Script { task, order, content_digest } in entries {
        translated.push(target::Script { task, order, content_digest: content_digest.0 }).expect("entry bound");
    }
    translated.into_boxed()
}
