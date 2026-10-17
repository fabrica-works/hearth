use crate::{
    config::document as source,
    git_machine::listing,
    render::revision as target,
    translate::{documents, listing as entries},
};

const MODES: &[(listing::Mode, target::Mode)] = &[
    (listing::Mode::File, target::Mode::File),
    (listing::Mode::Executable, target::Mode::Executable),
    (listing::Mode::Link, target::Mode::Link),
    (listing::Mode::Submodule, target::Mode::Submodule),
];
fn mode_back(mode: target::Mode) -> listing::Mode {
    match mode {
        target::Mode::File => listing::Mode::File,
        target::Mode::Executable => listing::Mode::Executable,
        target::Mode::Link => listing::Mode::Link,
        target::Mode::Submodule => listing::Mode::Submodule,
    }
}
#[test]
fn each_mode_translates_and_back() {
    for (source, target) in MODES {
        assert_eq!(entries::mode_to_render(*source), *target, "mode {source:?}");
        assert_eq!(mode_back(*target), *source, "mode {target:?}");
    }
}
const KINDS: &[(source::Runs, target::Runs)] = &[
    (source::Runs::Metal, target::Runs::Metal),
    (source::Runs::Vm, target::Runs::Vm),
    (source::Runs::Container, target::Runs::Container),
];
fn kind_back(kind: target::Runs) -> source::Runs {
    match kind {
        target::Runs::Metal => source::Runs::Metal,
        target::Runs::Vm => source::Runs::Vm,
        target::Runs::Container => source::Runs::Container,
    }
}
#[test]
fn each_placement_kind_translates_and_back() {
    for (source, target) in KINDS {
        assert_eq!(documents::runs(*source), *target, "kind {source:?}");
        assert_eq!(kind_back(*target), *source, "kind {target:?}");
    }
}
const FIRMWARE: &[(source::Firmware, target::Firmware)] =
    &[(source::Firmware::Efi, target::Firmware::Efi), (source::Firmware::Bios, target::Firmware::Bios)];
fn firmware_back(firmware: target::Firmware) -> source::Firmware {
    match firmware {
        target::Firmware::Efi => source::Firmware::Efi,
        target::Firmware::Bios => source::Firmware::Bios,
    }
}
#[test]
fn each_firmware_translates_and_back() {
    for (source, target) in FIRMWARE {
        assert_eq!(documents::boot(*source), *target, "firmware {source:?}");
        assert_eq!(firmware_back(*target), *source, "firmware {target:?}");
    }
}

#[test]
fn a_containers_placement_keeps_its_host_position_and_mutability() {
    let input = source::Placement {
        at: crate::config::Position { offset: 7, line: 2, column: 4 },
        kind: source::Runs::Container,
        firmware: None,
        hardware: None,
        on: Some(source::On {
            target: skein_lib::bytes::copy_of(b"x1-prod"),
            at: crate::config::Position { offset: 17, line: 3, column: 8 },
        }),
        mutable: true,
    };
    assert_eq!(
        documents::placement_to_render(input),
        target::Placement {
            at: target::Position { line: 2, column: 4 },
            kind: target::Runs::Container,
            firmware: None,
            hardware: None,
            on: Some(target::On {
                target: skein_lib::bytes::copy_of(b"x1-prod"),
                at: target::Position { line: 3, column: 8 }
            }),
            mutable: true,
        },
        "container placement fields"
    );
}
#[test]
fn a_document_without_placement_keeps_its_position_and_absence() {
    let input = source::Document {
        members: None,
        at: crate::config::Position { offset: 7, line: 2, column: 4 },
        layer: source::Layer { tasks: Box::new([]), debian: None, placement: None },
    };
    assert_eq!(
        documents::document(input),
        target::Document {
            members: None,
            at: target::Position { line: 2, column: 4 },
            layer: target::Layer { debian: None, placement: None, tasks: Box::new([]) }
        },
        "no placement is introduced"
    );
}

use crate::{
    records::{self, parts::meta as client_meta},
    render,
    translate::answers,
};
fn client_kind_back(kind: client_meta::PlacementKind) -> target::Runs {
    match kind {
        client_meta::PlacementKind::Metal => target::Runs::Metal,
        client_meta::PlacementKind::Vm => target::Runs::Vm,
        client_meta::PlacementKind::Container => target::Runs::Container,
    }
}
fn client_firmware_back(firmware: client_meta::Firmware) -> target::Firmware {
    match firmware {
        client_meta::Firmware::Efi => target::Firmware::Efi,
        client_meta::Firmware::Bios => target::Firmware::Bios,
    }
}
fn optional_client_firmware_back(firmware: Option<client_meta::Firmware>) -> Option<target::Firmware> {
    Some(client_firmware_back(firmware?))
}
fn spec_back(spec: records::records::Spec) -> Option<render::parts::Spec> {
    let records::records::Spec { meta, meta_digest, commit, digest, parts, resolution } = spec;
    let mut translated = skein_lib::List::with_capacity(u32::try_from(parts.len()).expect("bounded parts"));
    for records::records::Part { digest, record } in parts {
        let record = match record {
            records::records::PartRecord::Config(config) => {
                render::parts::PartRecord::Config(config::config_back(config))
            }
            records::records::PartRecord::Secrets(value) => render::parts::PartRecord::Secrets(secrets::back(value)),
            records::records::PartRecord::Packages(value) => packages::part_back(value),
            records::records::PartRecord::Live(value) => render::parts::PartRecord::Live(live::back(value)),
        };
        translated.push(render::parts::Part { digest: hearth_lib::encode::Digest(digest), record }).expect("part room");
    }
    match resolution {
        None => {}
        Some(
            records::records::Resolution::Needed { key: _ } | records::records::Resolution::Known { key: _, keys: _ },
        ) => return None, // hearth 24.1
    }
    let client_meta::Meta { target, base, member, environment, placement } = meta;
    let client_meta::Placement { kind, firmware, hardware, on } = placement;
    Some(render::parts::Spec {
        meta: render::parts::meta::Meta {
            target,
            base,
            member,
            environment,
            placement: render::parts::meta::Placement {
                kind: client_kind_back(kind),
                firmware: optional_client_firmware_back(firmware),
                hardware,
                on,
            },
        },
        meta_digest: hearth_lib::encode::Digest(meta_digest),
        commit: target::Oid(commit),
        digest: hearth_lib::encode::Digest(digest),
        parts: translated.into_boxed(),
    })
}
#[test]
fn a_spec_translates_to_the_records_and_back() {
    for (row, input) in super::fixtures::live_specs().into_iter().enumerate() {
        let back = spec_back(answers::spec(input.clone())).expect("spec");
        assert_eq!(back, input, "row {row}: spec");
    }
}

const CLIENT_KINDS: &[(target::Runs, client_meta::PlacementKind)] = &[
    (target::Runs::Metal, client_meta::PlacementKind::Metal),
    (target::Runs::Vm, client_meta::PlacementKind::Vm),
    (target::Runs::Container, client_meta::PlacementKind::Container),
];
#[test]
fn each_client_placement_kind_translates_and_back() {
    for (source, expected) in CLIENT_KINDS {
        assert_eq!(answers::runs(*source), *expected, "kind {source:?}");
        assert_eq!(client_kind_back(*expected), *source, "kind {expected:?}");
    }
}
const CLIENT_FIRMWARE: &[(target::Firmware, client_meta::Firmware)] =
    &[(target::Firmware::Efi, client_meta::Firmware::Efi), (target::Firmware::Bios, client_meta::Firmware::Bios)];
#[test]
fn each_client_firmware_translates_and_back() {
    for (source, expected) in CLIENT_FIRMWARE {
        assert_eq!(answers::boot(*source), *expected, "firmware {source:?}");
        assert_eq!(client_firmware_back(*expected), *source, "firmware {expected:?}");
    }
}
fn outcome_back(outcome: records::outcome::Outcome) -> render::vocabulary::Outcome {
    match outcome {
        records::outcome::Outcome::Done => render::vocabulary::Outcome::Done,
        records::outcome::Outcome::Failed => render::vocabulary::Outcome::Failed,
        records::outcome::Outcome::Usage => render::vocabulary::Outcome::Usage,
        records::outcome::Outcome::TryAgain => render::vocabulary::Outcome::TryAgain,
        records::outcome::Outcome::Negative => render::vocabulary::Outcome::Negative,
    }
}
const CLIENT_OUTCOMES: &[(render::vocabulary::Outcome, records::outcome::Outcome)] = &[
    (render::vocabulary::Outcome::Done, records::outcome::Outcome::Done),
    (render::vocabulary::Outcome::Failed, records::outcome::Outcome::Failed),
    (render::vocabulary::Outcome::Usage, records::outcome::Outcome::Usage),
    (render::vocabulary::Outcome::TryAgain, records::outcome::Outcome::TryAgain),
    (render::vocabulary::Outcome::Negative, records::outcome::Outcome::Negative),
];
#[test]
fn each_client_outcome_translates_and_back() {
    for (source, expected) in CLIENT_OUTCOMES {
        assert_eq!(answers::outcome(*source), *expected, "outcome {source:?}");
        assert_eq!(outcome_back(*expected), *source, "outcome {expected:?}");
    }
}

#[test]
fn a_config_part_translates_to_the_records_and_back() {
    let source = render::parts::Part {
        digest: hearth_lib::encode::Digest([7; 32]),
        record: render::parts::PartRecord::Config(render::parts::config::Config {
            tree: Box::new([render::parts::config::file(
                b"etc/motd",
                target::Mode::Executable,
                hearth_lib::encode::Digest([8; 32]),
            )]),
        }),
    };
    let records::records::Part { digest, record } = crate::translate::parts::part(source);
    assert_eq!(digest, [7; 32]);
    let records::records::PartRecord::Config(config) = record else { panic!("config part") };
    assert_eq!(
        config::config_back(config),
        render::parts::config::Config {
            tree: Box::new([render::parts::config::TreeEntry {
                path: skein_lib::bytes::copy_of(b"/etc/motd"),
                node: render::parts::config::Node::File {
                    content_digest: hearth_lib::encode::Digest([8; 32]),
                    mode: 0o755,
                    owner: skein_lib::bytes::copy_of(b"root"),
                    group: skein_lib::bytes::copy_of(b"root")
                }
            }])
        }
    );
}

const VALUES: &[(crate::config::json::Kind, target::Kind)] = &[
    (crate::config::json::Kind::Null, target::Kind::Null),
    (crate::config::json::Kind::Bool(true), target::Kind::Bool(true)),
    (crate::config::json::Kind::Int(42), target::Kind::Int(42)),
];
#[test]
fn scalar_configuration_values_keep_their_kind_and_position() {
    for (kind, expected) in VALUES {
        let value = documents::value(crate::config::json::Value {
            at: crate::config::Position { offset: 17, line: 3, column: 8 },
            kind: kind.clone(),
        });
        assert_eq!(value.kind, *expected);
        assert_eq!(value.at, target::Position { line: 3, column: 8 });
    }
}
#[test]
fn nested_configuration_values_keep_their_keys_lists_and_string_bytes() {
    let at = crate::config::Position { offset: 0, line: 1, column: 1 };
    let value = crate::config::json::Value {
        at,
        kind: crate::config::json::Kind::Map(Box::new([crate::config::json::Entry {
            key: skein_lib::bytes::copy_of(b"a"),
            at,
            value: crate::config::json::Value {
                at,
                kind: crate::config::json::Kind::List(Box::new([crate::config::json::Value {
                    at,
                    kind: crate::config::json::Kind::String(skein_lib::bytes::copy_of(b"hello")),
                }])),
            },
        }])),
    };
    assert_eq!(
        documents::value(value),
        target::Value {
            at: target::Position { line: 1, column: 1 },
            kind: target::Kind::Map(Box::new([target::Pair {
                key: skein_lib::bytes::copy_of(b"a"),
                at: target::Position { line: 1, column: 1 },
                value: target::Value {
                    at: target::Position { line: 1, column: 1 },
                    kind: target::Kind::List(Box::new([target::Value {
                        at: target::Position { line: 1, column: 1 },
                        kind: target::Kind::String(skein_lib::bytes::copy_of(b"hello"))
                    }]))
                },
            }])),
        }
    );
}

use crate::{records::diff as client_diff, render::diff, translate::changes};
const CHANGES: &[(diff::TargetChange, client_diff::TargetChange)] = &[
    (
        diff::TargetChange::Added { spec_digest: hearth_lib::encode::Digest([9; 32]) },
        client_diff::TargetChange::Added { spec_digest: [9; 32] },
    ),
    (diff::TargetChange::Removed, client_diff::TargetChange::Removed),
    (diff::TargetChange::Same, client_diff::TargetChange::Same),
    (
        diff::TargetChange::Failed { side: diff::Side::From },
        client_diff::TargetChange::Failed { side: client_diff::Side::From },
    ),
    (
        diff::TargetChange::Failed { side: diff::Side::To },
        client_diff::TargetChange::Failed { side: client_diff::Side::To },
    ),
    (
        diff::TargetChange::Failed { side: diff::Side::Both },
        client_diff::TargetChange::Failed { side: client_diff::Side::Both },
    ),
];
fn side_back(side: client_diff::Side) -> diff::Side {
    match side {
        client_diff::Side::From => diff::Side::From,
        client_diff::Side::To => diff::Side::To,
        client_diff::Side::Both => diff::Side::Both,
    }
}
fn needs_back(needs: client_diff::Needs) -> diff::Needs {
    match needs {
        client_diff::Needs::Known => diff::Needs::Known,
        client_diff::Needs::Needed => diff::Needs::Needed,
    }
}
fn rebuild_back(rebuild: client_diff::Rebuild) -> diff::Rebuild {
    match rebuild {
        client_diff::Rebuild::FromPackages { resolution } => {
            diff::Rebuild::FromPackages { resolution: needs_back(resolution) }
        }
        client_diff::Rebuild::FromSoftware => diff::Rebuild::FromSoftware,
        client_diff::Rebuild::FromConfig => diff::Rebuild::FromConfig,
        client_diff::Rebuild::SlotOnly => diff::Rebuild::SlotOnly,
        client_diff::Rebuild::Nothing => diff::Rebuild::Nothing,
    }
}
fn change_back(change: client_diff::TargetChange) -> diff::TargetChange {
    match change {
        client_diff::TargetChange::Added { spec_digest } => {
            diff::TargetChange::Added { spec_digest: hearth_lib::encode::Digest(spec_digest) }
        }
        client_diff::TargetChange::Removed => diff::TargetChange::Removed,
        client_diff::TargetChange::Same => diff::TargetChange::Same,
        client_diff::TargetChange::Failed { side } => diff::TargetChange::Failed { side: side_back(side) },
        client_diff::TargetChange::Changed { parts, rebuild } => {
            let mut translated = skein_lib::List::with_capacity(u32::try_from(parts.len()).expect("bounded parts"));
            for part in parts {
                match part {
                    client_diff::PartChange::Meta(meta) => {
                        translated.push(diff::PartChange::Meta(meta::back(meta))).expect("parts");
                    }
                    client_diff::PartChange::Config(client_diff::ConfigChange { tree }) => {
                        let mut entries =
                            skein_lib::List::with_capacity(u32::try_from(tree.len()).expect("bounded tree"));
                        for entry in tree {
                            entries.push(entry_back(entry)).expect("tree room");
                        }
                        translated
                            .push(diff::PartChange::Config(render::parts::config::Change {
                                tree: entries.into_boxed(),
                            }))
                            .expect("part room");
                    }
                }
            }
            diff::TargetChange::Changed { parts: translated.into_boxed(), rebuild: rebuild_back(rebuild) }
        }
    }
}
fn node_back(node: records::parts::config::Node) -> render::parts::config::Node {
    match node {
        records::parts::config::Node::Link { target, owner, group } => {
            render::parts::config::Node::Link { target, owner, group }
        }
        records::parts::config::Node::Directory { mode, owner, group } => {
            render::parts::config::Node::Directory { mode, owner, group }
        }
        records::parts::config::Node::File { content_digest, mode, owner, group } => {
            render::parts::config::Node::File {
                content_digest: hearth_lib::encode::Digest(content_digest),
                mode,
                owner,
                group,
            }
        }
    }
}
fn entry_back(entry: client_diff::EntryChange) -> render::parts::config::EntryChange {
    use render::parts::config::EntryChange;
    match entry {
        client_diff::EntryChange::Added { path, node } => EntryChange::Added { path, node: node_back(node) },
        client_diff::EntryChange::Removed { path, node } => EntryChange::Removed { path, node: node_back(node) },
        client_diff::EntryChange::Changed { path, before, after, sizes, hunks } => EntryChange::Changed {
            path,
            before: node_back(before),
            after: node_back(after),
            sizes: sizes_back(sizes),
            hunks: optional_hunks_back(hunks),
        },
    }
}
#[test]
fn each_change_rebuild_and_side_translates_to_the_records_and_back() {
    for (source, expected) in CHANGES {
        assert_eq!(changes::target_change(source.clone()), *expected, "change {source:?}");
        assert_eq!(change_back(expected.clone()), source.clone(), "change {expected:?}");
    }
}
const REBUILDS: &[(diff::Rebuild, client_diff::Rebuild)] = &[
    (
        diff::Rebuild::FromPackages { resolution: diff::Needs::Known },
        client_diff::Rebuild::FromPackages { resolution: client_diff::Needs::Known },
    ),
    (
        diff::Rebuild::FromPackages { resolution: diff::Needs::Needed },
        client_diff::Rebuild::FromPackages { resolution: client_diff::Needs::Needed },
    ),
    (diff::Rebuild::FromSoftware, client_diff::Rebuild::FromSoftware),
    (diff::Rebuild::FromConfig, client_diff::Rebuild::FromConfig),
    (diff::Rebuild::SlotOnly, client_diff::Rebuild::SlotOnly),
    (diff::Rebuild::Nothing, client_diff::Rebuild::Nothing),
];
#[test]
fn each_rebuild_translates_to_the_records_and_back() {
    for (source, expected) in REBUILDS {
        assert_eq!(changes::rebuild(*source), *expected, "rebuild {source:?}");
        assert_eq!(rebuild_back(*expected), *source, "rebuild {expected:?}");
    }
}
#[test]
fn the_tally_translates_to_the_records_and_back() {
    let tally = diff::Tally {
        from: target::Oid(skein_lib::bytes::copy_of(b"from")),
        to: target::Oid(skein_lib::bytes::copy_of(b"to")),
        targets: 4,
        changed: 2,
        unchanged: 1,
    };
    assert_eq!(
        changes::tally(tally),
        client_diff::Tally {
            from: skein_lib::bytes::copy_of(b"from"),
            to: skein_lib::bytes::copy_of(b"to"),
            targets: 4,
            changed: 2,
            unchanged: 1
        },
        "failed target only in total"
    );
}
#[test]
fn a_changed_config_entry_translates_its_sizes_and_hunks() {
    let change = super::fixtures::config_change();
    let translated = changes::change(change.clone());
    assert_eq!(translated.target.as_ref(), b"x1-prod");
    assert_eq!(
        change_back(translated.change),
        diff::TargetChange::Changed {
            parts: Box::new([diff::PartChange::Config(render::parts::config::Change {
                tree: Box::new([render::parts::config::EntryChange::Changed {
                    path: skein_lib::bytes::copy_of(b"/etc/motd"),
                    before: render::parts::config::Node::File {
                        content_digest: hearth_lib::encode::Digest([1; 32]),
                        mode: 0o644,
                        owner: skein_lib::bytes::copy_of(b"root"),
                        group: skein_lib::bytes::copy_of(b"root")
                    },
                    after: render::parts::config::Node::File {
                        content_digest: hearth_lib::encode::Digest([2; 32]),
                        mode: 0o755,
                        owner: skein_lib::bytes::copy_of(b"root"),
                        group: skein_lib::bytes::copy_of(b"root")
                    },
                    sizes: Some(render::parts::config::Sizes { before: 9, after: 22 }),
                    hunks: Some(diff::Hunks { lines: Box::new([]), left_out: 401, binary: false }),
                }])
            })]),
            rebuild: diff::Rebuild::FromConfig,
        },
        "config node fields and rebuild preserved"
    );
}

const STORE_RECORDS: &[(crate::tree::store::Store, crate::builder::store::vocabulary::StoreRecord)] = &[
    (
        crate::tree::store::Store { layout: 1, made: skein_lib::Wall::from_nanos(77), range: None, uid: 1000 },
        crate::builder::store::vocabulary::StoreRecord {
            layout: 1,
            made: skein_lib::Wall::from_nanos(77),
            range: None,
            uid: 1000,
        },
    ),
    (
        crate::tree::store::Store {
            layout: 2,
            made: skein_lib::Wall::from_nanos(99),
            range: Some(crate::tree::store::Range { base: 100_000, count: 65_536 }),
            uid: 1001,
        },
        crate::builder::store::vocabulary::StoreRecord {
            layout: 2,
            made: skein_lib::Wall::from_nanos(99),
            range: Some(crate::builder::store::vocabulary::Range { base: 100_000, count: 65_536 }),
            uid: 1001,
        },
    ),
];
#[test]
fn each_store_record_field_translates_and_back() {
    for &(record, translated) in STORE_RECORDS {
        assert_eq!(crate::translate::store::store_record(record), translated, "{record:?}");
        assert_eq!(crate::translate::store::tree_store(translated), record, "{translated:?}");
    }
}
mod errors;

mod back {
    use crate::render::evaluate as source;
    use hearth_protocol_template as target;
    use skein_lib::List;
    #[must_use]
    pub fn expression(expression: source::Expression) -> target::Expression {
        let source::Expression { operations, values } = expression;
        let mut translated = List::with_capacity(u32::try_from(operations.len()).expect("bounded operations"));
        for item in operations {
            translated.push(operation(item)).expect("operation room");
        }
        target::Expression { operations: translated.into_boxed(), values }
    }
    #[must_use]
    pub fn operation(operation: source::Operation) -> target::Operation {
        let source::Operation { at, op: value } = operation;
        target::Operation { at, op: op(value) }
    }
    #[must_use]
    pub fn op(op: source::Op) -> target::Op {
        match op {
            source::Op::String(v) => target::Op::String(v),
            source::Op::Integer(v) => target::Op::Integer(v),
            source::Op::Boolean(v) => target::Op::Boolean(v),
            source::Op::Name { name, missing: mark } => target::Op::Name { name, missing: missing(mark) },
            source::Op::Key { name, missing: mark } => target::Op::Key { name, missing: missing(mark) },
            source::Op::Item { missing: mark } => target::Op::Item { missing: missing(mark) },
            source::Op::List { length } => target::Op::List { length },
            source::Op::Filter(value) => target::Op::Filter(filter(value)),
            source::Op::Not => target::Op::Not,
            source::Op::Equal => target::Op::Equal,
            source::Op::NotEqual => target::Op::NotEqual,
            source::Op::Less => target::Op::Less,
            source::Op::LessOrEqual => target::Op::LessOrEqual,
            source::Op::Greater => target::Op::Greater,
            source::Op::GreaterOrEqual => target::Op::GreaterOrEqual,
            source::Op::In => target::Op::In,
            source::Op::NotIn => target::Op::NotIn,
            source::Op::Concat => target::Op::Concat,
            source::Op::Add => target::Op::Add,
            source::Op::Subtract => target::Op::Subtract,
            source::Op::Defined => target::Op::Defined,
            source::Op::NotDefined => target::Op::NotDefined,
            source::Op::JumpIfFalseKeeping { over } => target::Op::JumpIfFalseKeeping { over },
            source::Op::JumpIfTrueKeeping { over } => target::Op::JumpIfTrueKeeping { over },
            source::Op::JumpIfFalse { over } => target::Op::JumpIfFalse { over },
            source::Op::Jump { over } => target::Op::Jump { over },
        }
    }
    #[must_use]
    pub fn missing(missing: source::Missing) -> target::Missing {
        match missing {
            source::Missing::Error => target::Missing::Error,
            source::Missing::Undefined => target::Missing::Undefined,
        }
    }
    #[must_use]
    pub fn filter(filter: source::Filter) -> target::Filter {
        match filter {
            source::Filter::Default => target::Filter::Default,
            source::Filter::Join => target::Filter::Join,
            source::Filter::Split => target::Filter::Split,
            source::Filter::First => target::Filter::First,
            source::Filter::Last => target::Filter::Last,
            source::Filter::Length => target::Filter::Length,
            source::Filter::Replace => target::Filter::Replace,
            source::Filter::Items => target::Filter::Items,
            source::Filter::Keys => target::Filter::Keys,
            source::Filter::Values => target::Filter::Values,
            source::Filter::Sort => target::Filter::Sort,
            source::Filter::Tojson => target::Filter::Tojson,
            source::Filter::ShellQuote => target::Filter::ShellQuote,
            source::Filter::SystemdEscape => target::Filter::SystemdEscape,
            source::Filter::SystemdEscapePath => target::Filter::SystemdEscapePath,
            source::Filter::Basename => target::Filter::Basename,
            source::Filter::Dirname => target::Filter::Dirname,
        }
    }
}

use crate::translate::operations;
use hearth_protocol_template as template;
#[test]
fn each_operation_translates_to_render_and_back() {
    for (row, op) in super::fixtures::every_op().into_iter().enumerate() {
        let back = back::op(operations::op(op.clone()));
        assert_eq!(back, op, "row {row}: hearth 05's Op through render's and back");
    }
}
const MISSING: &[template::Missing] = &[template::Missing::Error, template::Missing::Undefined];
#[test]
fn missing_translates_to_render_and_back() {
    for &missing in MISSING {
        assert_eq!(back::missing(operations::missing(missing)), missing, "{missing:?}");
    }
}
const FILTERS: &[template::Filter] = &[
    template::Filter::Default,
    template::Filter::Join,
    template::Filter::Split,
    template::Filter::First,
    template::Filter::Last,
    template::Filter::Length,
    template::Filter::Replace,
    template::Filter::Items,
    template::Filter::Keys,
    template::Filter::Values,
    template::Filter::Sort,
    template::Filter::Tojson,
    template::Filter::ShellQuote,
    template::Filter::SystemdEscape,
    template::Filter::SystemdEscapePath,
    template::Filter::Basename,
    template::Filter::Dirname,
];
#[test]
fn each_filter_translates_to_render_and_back() {
    for &filter in FILTERS {
        assert_eq!(back::filter(operations::filter(filter)), filter, "{filter:?}");
    }
}
#[test]
fn an_expressions_offsets_and_values_cross_unchanged() {
    let expression = template::Expression {
        values: 2,
        operations: Box::new([
            template::Operation { at: 7, op: template::Op::String(skein_lib::bytes::copy_of(b"text")) },
            template::Operation { at: 19, op: template::Op::Integer(-17) },
        ]),
    };
    assert_eq!(
        back::expression(operations::expression(expression)),
        template::Expression {
            values: 2,
            operations: Box::new([
                template::Operation { at: 7, op: template::Op::String(skein_lib::bytes::copy_of(b"text")) },
                template::Operation { at: 19, op: template::Op::Integer(-17) },
            ]),
        }
    );
}

fn line_kind_back(kind: client_diff::LineKind) -> diff::LineKind {
    match kind {
        client_diff::LineKind::Header => diff::LineKind::Hunk,
        client_diff::LineKind::Context => diff::LineKind::Context,
        client_diff::LineKind::Removed => diff::LineKind::Removed,
        client_diff::LineKind::Added => diff::LineKind::Added,
        client_diff::LineKind::NoNewline => diff::LineKind::NoNewline,
    }
}
fn git_kind_back(kind: diff::LineKind) -> crate::git_machine::hunks::Kind {
    use crate::git_machine::hunks::Kind;
    match kind {
        diff::LineKind::Hunk => Kind::Hunk,
        diff::LineKind::Context => Kind::Context,
        diff::LineKind::Removed => Kind::Removed,
        diff::LineKind::Added => Kind::Added,
        diff::LineKind::NoNewline => Kind::NoNewline,
    }
}
fn sizes_back(value: Option<client_diff::Sizes>) -> Option<render::parts::config::Sizes> {
    let value = value?;
    Some(render::parts::config::Sizes { before: value.before, after: value.after })
}
fn optional_hunks_back(value: Option<client_diff::Hunks>) -> Option<diff::Hunks> {
    Some(hunks_back(value?))
}
fn hunks_back(hunks: client_diff::Hunks) -> diff::Hunks {
    let mut lines = skein_lib::List::with_capacity(u32::try_from(hunks.lines.len()).expect("bounded lines"));
    for line in hunks.lines {
        lines.push(diff::Line { kind: line_kind_back(line.kind), text: line.text }).expect("line room");
    }
    diff::Hunks { lines: lines.into_boxed(), left_out: hunks.left_out, binary: hunks.binary }
}
const HUNK_KINDS: &[(crate::git_machine::hunks::Kind, diff::LineKind, client_diff::LineKind)] = &[
    (crate::git_machine::hunks::Kind::Hunk, diff::LineKind::Hunk, client_diff::LineKind::Header),
    (crate::git_machine::hunks::Kind::Context, diff::LineKind::Context, client_diff::LineKind::Context),
    (crate::git_machine::hunks::Kind::Removed, diff::LineKind::Removed, client_diff::LineKind::Removed),
    (crate::git_machine::hunks::Kind::Added, diff::LineKind::Added, client_diff::LineKind::Added),
    (crate::git_machine::hunks::Kind::NoNewline, diff::LineKind::NoNewline, client_diff::LineKind::NoNewline),
];
#[test]
fn each_hunk_line_kind_translates_and_back() {
    for (row, (git, render, records)) in HUNK_KINDS.iter().enumerate() {
        let translated = crate::translate::hunks::kind(*git);
        assert_eq!(translated, *render, "row {row}: git kind");
        assert_eq!(git_kind_back(translated), *git, "row {row}: git kind and back");
        let translated = changes::line_kind(*render);
        assert_eq!(translated, *records, "row {row}: records kind");
        assert_eq!(line_kind_back(translated), *render, "row {row}: records kind and back");
    }
}
#[test]
fn retained_hunk_text_and_counts_translate_from_git() {
    let hunks = crate::git_machine::hunks::Hunks {
        lines: Box::new([crate::git_machine::hunks::Line {
            kind: crate::git_machine::hunks::Kind::NoNewline,
            text: skein_lib::bytes::copy_of(b" No newline at end of file"),
        }]),
        left_out: 401,
        binary: false,
    };
    assert_eq!(
        crate::translate::hunks::hunks(hunks),
        diff::Hunks {
            lines: Box::new([diff::Line {
                kind: diff::LineKind::NoNewline,
                text: skein_lib::bytes::copy_of(b" No newline at end of file")
            }]),
            left_out: 401,
            binary: false
        }
    );
}

#[test]
fn each_retained_hunk_line_translates_to_records_and_back() {
    for (row, input) in super::fixtures::render_hunks().into_iter().enumerate() {
        let back = hunks_back(changes::hunks(input.clone()));
        assert_eq!(back, input, "row {row}: retained text, kinds, counts and binary marker");
    }
}

mod composition;
mod declarations;
mod repository;
#[path = "translate_store.rs"]
mod store_arrivals;

mod evaluation;
mod instructions;
mod meta;

mod forms;

mod guards;

mod expansion;

mod debian;
mod fields;

mod config;
mod live;
mod secrets;

mod files;
mod packages;
