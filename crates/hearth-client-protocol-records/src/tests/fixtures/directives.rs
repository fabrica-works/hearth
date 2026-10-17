use crate::{
    parts::config::*,
    records::{Part, PartRecord, Record},
};
fn line(mode: u16) -> Line {
    Line {
        kind: LineKind::D,
        mode: Some(mode),
        owner: Some(b"root".as_slice().into()),
        group: Some(b"systemd-journal".as_slice().into()),
        age: None,
        argument: None,
    }
}
fn script(digest: [u8; 32]) -> Script {
    Script { task: b"motd".as_slice().into(), order: 300, content_digest: digest }
}
pub fn directives_spec() -> Record {
    let mut spec = super::x1_prod();
    spec.parts = Box::new([Part {
        digest: [0; 32],
        record: PartRecord::Config(Config {
            tree: Box::new([]),
            groups: Box::new([]),
            users: Box::new([]),
            members: Box::new([]),
            presets: Box::new([
                Preset { unit: b"getty@tty1.service".as_slice().into(), action: Action::Disable },
                Preset { unit: b"ssh.service".as_slice().into(), action: Action::Enable },
            ]),
            tmpfiles: Box::new([Tmpfile { path: b"/var/log/journal".as_slice().into(), line: line(0o2755) }]),
            scripts: Box::new([script([1; 32])]),
        }),
    }]);
    Record::Spec(spec)
}
pub fn directives_change() -> Record {
    use crate::diff::{Change, ConfigChange, LineKeyed, PartChange, PresetKeyed, Rebuild, ScriptsChange, TargetChange};
    Record::Change(Change {
        target: b"x1-prod".as_slice().into(),
        change: TargetChange::Changed {
            rebuild: Rebuild::FromConfig,
            parts: Box::new([PartChange::Config(ConfigChange {
                tree: Box::new([]),
                groups: Box::new([]),
                users: Box::new([]),
                members: Box::new([]),
                presets: Box::new([PresetKeyed::Changed {
                    key: b"ssh.service".as_slice().into(),
                    before: Action::Enable,
                    after: Action::Disable,
                }]),
                tmpfiles: Box::new([LineKeyed::Changed {
                    key: b"/var/log/journal".as_slice().into(),
                    before: line(0o2755),
                    after: line(0o755),
                }]),
                scripts: Some(ScriptsChange {
                    before: Box::new([script([1; 32])]),
                    after: Box::new([script([2; 32])]),
                }),
            })]),
        },
    })
}
