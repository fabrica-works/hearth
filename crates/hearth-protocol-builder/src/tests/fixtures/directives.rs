use crate::render::parts::config::*;
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
pub const DEFAULT_SCRIPT_ORDER: u16 = 500;
const KINDS: &[LineKind] = &[
    LineKind::D,
    LineKind::CapitalD,
    LineKind::E,
    LineKind::F,
    LineKind::FPlus,
    LineKind::W,
    LineKind::WPlus,
    LineKind::CapitalL,
    LineKind::CapitalLPlus,
    LineKind::CapitalC,
    LineKind::Z,
    LineKind::CapitalZ,
    LineKind::R,
    LineKind::CapitalR,
];
fn line(kind: LineKind) -> Line {
    Line {
        kind,
        mode: Some(0o755),
        owner: Some(b"root".as_slice().into()),
        group: Some(b"adm".as_slice().into()),
        age: Some(b"1d".as_slice().into()),
        argument: Some(b"x".as_slice().into()),
    }
}
fn scripts() -> Box<[Script]> {
    Box::new([Script { task: b"motd".as_slice().into(), order: DEFAULT_SCRIPT_ORDER, content_digest: Digest([1; 32]) }])
}
#[must_use]
pub fn directives_configs() -> [Config; 2] {
    let [mut config, empty] = super::account_configs();
    config.presets = Box::new([
        Preset { unit: b"ssh.service".as_slice().into(), action: Action::Enable },
        Preset { unit: b"getty.service".as_slice().into(), action: Action::Disable },
    ]);
    let mut lines = skein_lib::List::with_capacity(u32::try_from(KINDS.len()).expect("fixture count"));
    for kind in KINDS {
        lines.push(Tmpfile { path: kind.name().into(), line: line(*kind) }).expect("fixture count");
    }
    config.tmpfiles = lines.into_boxed();
    config.scripts = scripts();
    [config, empty]
}
#[must_use]
pub fn directives_changes() -> [Change; 2] {
    let [mut change, empty] = super::account_changes();
    change.presets = Box::new([
        PresetKeyed::Added { key: b"ssh.service".as_slice().into(), value: Action::Enable },
        PresetKeyed::Removed { key: b"getty.service".as_slice().into(), value: Action::Disable },
        PresetKeyed::Changed {
            key: b"other.service".as_slice().into(),
            before: Action::Enable,
            after: Action::Disable,
        },
    ]);
    change.tmpfiles = Box::new([
        LineKeyed::Added { key: b"/a".as_slice().into(), value: line(LineKind::D) },
        LineKeyed::Removed { key: b"/b".as_slice().into(), value: line(LineKind::F) },
        LineKeyed::Changed { key: b"/c".as_slice().into(), before: line(LineKind::Z), after: line(LineKind::CapitalZ) },
    ]);
    change.scripts = Some(ScriptsChange { before: Box::new([]), after: scripts() });
    [change, empty]
}
