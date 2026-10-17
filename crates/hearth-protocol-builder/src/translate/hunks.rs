//! Git's unified lines translated without inspecting their text (render.md, 6.1).
use crate::{git_machine::hunks as source, render::diff as target};
use skein_lib::List;
#[must_use]
pub fn hunks(hunks: source::Hunks) -> target::Hunks {
    let mut lines = List::with_capacity(u32::try_from(hunks.lines.len()).expect("bounded lines"));
    for line in hunks.lines {
        lines.push(target::Line { kind: kind(line.kind), text: line.text }).expect("line capacity");
    }
    target::Hunks { lines: lines.into_boxed(), left_out: hunks.left_out, binary: hunks.binary }
}
#[must_use]
pub const fn kind(kind: source::Kind) -> target::LineKind {
    match kind {
        source::Kind::Hunk => target::LineKind::Hunk,
        source::Kind::Context => target::LineKind::Context,
        source::Kind::Removed => target::LineKind::Removed,
        source::Kind::Added => target::LineKind::Added,
        source::Kind::NoNewline => target::LineKind::NoNewline,
    }
}
