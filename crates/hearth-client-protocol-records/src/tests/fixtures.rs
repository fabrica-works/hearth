use crate::{
    parts::{
        config::{Config, Node, TreeEntry},
        meta::{Firmware, Meta, Placement, PlacementKind},
    },
    records::{Part, PartRecord, Spec},
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
pub fn digest(prefix: [u8; 6]) -> [u8; 32] {
    let mut value = [0; 32];
    for (to, from) in value.iter_mut().zip(prefix) {
        *to = from;
    }
    value
}
pub fn x1_prod() -> Box<Spec> {
    Box::new(Spec {
        meta: Meta {
            target: copy_of(b"x1-prod"),
            base: copy_of(b"x1"),
            member: None,
            environment: copy_of(b"prod"),
            placement: Placement {
                kind: PlacementKind::Metal,
                firmware: Some(Firmware::Efi),
                hardware: Some(copy_of(b"7th-gen")),
                on: None,
            },
        },
        resolution: None,
        meta_digest: digest([0xc4, 0xd3, 0xa0, 0xb1, 0xe2, 0xf5]),
        commit: copy_of(b"1a2b3c4d00000000000000000000000000000000"),
        digest: digest([0x9a, 0x0c, 0x5e, 0x1f, 0x2b, 0x3d]),
        parts: Box::new([Part {
            digest: digest([0xe7, 0xc2, 0xa0, 0xd9, 0x5b, 0x13]),
            record: PartRecord::Config(Config {
                tree: Box::new([TreeEntry {
                    path: copy_of(b"/etc/hostname"),
                    node: Node::File {
                        content_digest: digest([0x3f, 0x1c, 0x2a, 0xb0, 0x7e, 0x44]),
                        mode: 0o644,
                        owner: copy_of(b"root"),
                        group: copy_of(b"root"),
                    },
                }]),
            }),
        }]),
    })
}
pub const X1_PROD_TEXT: &str = concat!(
    "target       x1-prod   base x1   environment prod\n",
    "placement    metal efi 7th-gen\n",
    "spec         9a0c5e1f2b3d\n",
    "config       part e7c2a0d95b13\n",
    "  /etc/hostname            file 0644 root:root 3f1c2ab07e44\n",
);

pub fn with_resolution(mut spec: Box<Spec>, resolution: crate::records::Resolution) -> Box<Spec> {
    spec.resolution = Some(resolution);
    spec
}
pub fn known() -> crate::records::Resolution {
    crate::records::Resolution::Known {
        key: digest([0x0c, 0x7d, 0x9e, 0x41, 0xc0, 0xd2]),
        keys: crate::records::Keys { packages: [0x11; 32], software: [0x22; 32], config: [0x33; 32] },
    }
}

pub fn wg0_changed() -> crate::diff::Change {
    use crate::diff::{
        Change, ConfigChange, EntryChange, HunkLine, Hunks, LineKind, PartChange, Rebuild, Sizes, TargetChange,
    };
    const LINES: [(LineKind, &[u8]); 7] = [
        (LineKind::Header, b"@@ -9,5 +9,5 @@ ListenPort=51820"),
        (LineKind::Context, b"[WireGuardPeer]"),
        (LineKind::Context, "PublicKey=m3NY8…=".as_bytes()),
        (LineKind::Context, b"AllowedIPs=10.0.0.1/32"),
        (LineKind::Removed, b"Endpoint=194.164.162.157:51820"),
        (LineKind::Added, b"Endpoint=194.164.162.158:51820"),
        (LineKind::Context, b"PersistentKeepalive=25"),
    ];
    let mut lines = skein_lib::List::with_capacity(7);
    for (kind, text) in LINES {
        lines.push(HunkLine { kind, text: copy_of(text) }).expect("seven hunk lines");
    }
    Change {
        target: copy_of(b"carbon-prod"),
        change: TargetChange::Changed {
            rebuild: Rebuild::FromConfig,
            parts: Box::new([PartChange::Config(ConfigChange {
                tree: Box::new([EntryChange::Changed {
                    path: copy_of(b"/etc/systemd/network/wg0.netdev"),
                    before: file([0x11; 32]),
                    after: file([0x22; 32]),
                    sizes: Some(Sizes { before: 268, after: 268 }),
                    hunks: Some(Hunks { lines: lines.into_boxed(), left_out: 0, binary: false }),
                }]),
            })]),
        },
    }
}
pub fn file(content_digest: [u8; 32]) -> Node {
    Node::File { content_digest, mode: 0o644, owner: copy_of(b"root"), group: copy_of(b"root") }
}
pub const WG0_TEXT: &str = concat!(
    "carbon-prod                                         rebuild from config\n",
    "  config   ~ /etc/systemd/network/wg0.netdev        0644 root:root\n",
    "               @@ -9,5 +9,5 @@ ListenPort=51820\n                [WireGuardPeer]\n",
    "                PublicKey=m3NY8…=\n                AllowedIPs=10.0.0.1/32\n",
    "               -Endpoint=194.164.162.157:51820\n               +Endpoint=194.164.162.158:51820\n                PersistentKeepalive=25\n",
);
pub fn tally(targets: u32, changed: u32, unchanged: u32) -> crate::diff::Tally {
    crate::diff::Tally { from: copy_of(b"main"), to: copy_of(b"HEAD"), targets, changed, unchanged }
}

pub fn modules_guard() -> crate::errors::Error {
    use crate::errors::{Class, Error, Guard, Kind, Place, Position};
    Error {
        side: None,
        kind: Kind::Guard(Guard {
            name: copy_of(b"modules"),
            message: copy_of(b"no initramfs modules for this hardware"),
        }),
        class: Class::Configuration,
        at: Some(Place { path: copy_of(b"tasks/boot/task.json"), position: Some(Position { line: 22, column: 7 }) }),
        also: None,
        target: Some(copy_of(b"carbon-lab")),
        task: Some(copy_of(b"boot")),
    }
}

pub fn duplicate() -> crate::errors::Error {
    use crate::errors::{Class, Decode, Error, Kind, Place, Position};
    Error {
        side: None,
        kind: Kind::Decode(Decode::Duplicate { key: copy_of(b"motd") }),
        class: Class::Configuration,
        at: Some(Place { path: copy_of(b"envs/lab.json"), position: Some(Position { line: 1, column: 14 }) }),
        also: Some(Place { path: copy_of(b"envs/lab.json"), position: Some(Position { line: 1, column: 2 }) }),
        target: None,
        task: None,
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SchemaInput {
    Task(&'static [u8]),
    Wrong(&'static crate::errors::TypeName),
    Enum { value: &'static [u8], listed: &'static [&'static [u8]] },
    Below(i64),
    Above(i64),
    Length(i64),
    Null,
    UndeclaredField { field: &'static [u8], declared: &'static [&'static [u8]] },
    MissingField(&'static [u8]),
    Unread { task: &'static [u8], setting: &'static [u8], declared: &'static [&'static [u8]] },
}
pub fn schema_fault(input: SchemaInput) -> crate::errors::Schema {
    use crate::errors::Schema;
    match input {
        SchemaInput::Wrong(expected) => Schema::Wrong { expected: expected.clone() },
        SchemaInput::Enum { value, listed } => Schema::Enum { value: copy_of(value), listed: schema_names(listed) },
        SchemaInput::Below(min) => Schema::Below { min },
        SchemaInput::Above(max) => Schema::Above { max },
        SchemaInput::Length(max_length) => Schema::Length { max_length },
        SchemaInput::Null => Schema::Null,
        SchemaInput::UndeclaredField { field, declared } => {
            Schema::UndeclaredField { field: copy_of(field), declared: schema_names(declared) }
        }
        SchemaInput::MissingField(field) => Schema::MissingField { field: copy_of(field) },
        SchemaInput::Task(task) => Schema::Task { task: copy_of(task) },
        SchemaInput::Unread { task, setting, declared } => {
            let mut names = skein_lib::List::with_capacity(u32::try_from(declared.len()).expect("fixture count"));
            for name in declared {
                names.push(copy_of(name)).expect("room");
            }
            Schema::Unread { task: copy_of(task), setting: copy_of(setting), declared: names.into_boxed() }
        }
    }
}

fn schema_names(names: &[&[u8]]) -> Box<[Box<[u8]>]> {
    let mut copied = skein_lib::List::with_capacity(u32::try_from(names.len()).expect("fixture count"));
    for name in names {
        copied.push(copy_of(name)).expect("room");
    }
    copied.into_boxed()
}

pub fn git_failure(command: crate::errors::Command, code: u8) -> crate::errors::Error {
    use crate::errors::{Class, Ended, Error, Kind, Tool};
    Error {
        side: None,
        kind: Kind::Tool(Tool {
            command,
            ended: Ended::Exit(code),
            tail: Box::new([copy_of(b"fatal: not a tree object")]),
        }),
        class: Class::Operational,
        at: None,
        also: None,
        target: None,
        task: None,
    }
}

#[must_use]
pub fn syntax_faults() -> [crate::errors::Syntax; 19] {
    use crate::errors::{Syntax as S, syntax::Expected};
    [
        S::TooLong,
        S::Character,
        S::Unterminated,
        S::Escape,
        S::Integer,
        S::TooDeep,
        S::Misplaced,
        S::UnknownFilter,
        S::UnknownTest,
        S::Utf8,
        S::Return,
        S::Unclosed,
        S::UnknownStatement,
        S::Unmatched,
        S::Unended,
        S::Shadows,
        S::Interpolation,
        S::Expected(Expected::Operand),
        S::Arity { filter: copy_of(b"join"), expected: 1, given: 2 },
    ]
}
pub fn syntax_error(syntax: crate::errors::Syntax) -> crate::errors::Error {
    let mut error = modules_guard();
    error.kind = crate::errors::Kind::Decode(crate::errors::Decode::Syntax(syntax));
    error.target = None;
    error.task = Some(copy_of(b"motd"));
    error.at = Some(crate::errors::Place {
        path: copy_of(b"tasks/motd/tree/etc/issue.j2"),
        position: Some(crate::errors::Position { line: 2, column: 1 }),
    });
    error
}
#[must_use]
pub fn meta_changes() -> [crate::diff::MetaChange; 2] {
    use crate::{
        diff::{MetaChange, Pair},
        parts::meta::{Firmware, Placement, PlacementKind},
    };
    [
        MetaChange {
            base: Some(Pair { before: b"x1".as_slice().into(), after: b"x2".as_slice().into() }),
            member: Some(Pair { before: None, after: Some(b"a".as_slice().into()) }),
            environment: Some(Pair { before: b"prod".as_slice().into(), after: b"lab".as_slice().into() }),
            placement: Some(Pair {
                before: Placement {
                    kind: PlacementKind::Metal,
                    firmware: Some(Firmware::Efi),
                    hardware: None,
                    on: None,
                },
                after: Placement {
                    kind: PlacementKind::Container,
                    firmware: None,
                    hardware: None,
                    on: Some(b"host-prod".as_slice().into()),
                },
            }),
        },
        MetaChange {
            base: None,
            member: Some(Pair { before: Some(b"a".as_slice().into()), after: None }),
            environment: None,
            placement: None,
        },
    ]
}

pub fn composition_error(why: crate::errors::Composition) -> crate::errors::Error {
    crate::errors::Error {
        side: None,
        kind: crate::errors::Kind::Composition(why),
        class: crate::errors::Class::Configuration,
        at: None,
        also: None,
        target: Some(copy_of(b"stoa-1-lab")),
        task: None,
    }
}
