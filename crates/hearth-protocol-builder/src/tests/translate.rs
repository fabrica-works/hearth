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
            at: target::Position { line: 2, column: 4 },
            layer: target::Layer { placement: None, tasks: Box::new([]) }
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
            records::records::PartRecord::Config(config) => render::parts::PartRecord::Config(config_back(config)),
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
    let actual = spec_back(answers::spec(super::fixtures::spec()));
    assert_eq!(
        actual,
        Some(render::parts::Spec {
            meta: render::parts::meta::Meta {
                target: skein_lib::bytes::copy_of(b"x1-prod"),
                base: skein_lib::bytes::copy_of(b"x1"),
                member: None,
                environment: skein_lib::bytes::copy_of(b"prod"),
                placement: render::parts::meta::Placement {
                    kind: target::Runs::Metal,
                    firmware: Some(target::Firmware::Efi),
                    hardware: Some(skein_lib::bytes::copy_of(b"7th-gen")),
                    on: None
                }
            },
            meta_digest: hearth_lib::encode::Digest([1; 32]),
            commit: target::Oid(skein_lib::bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567")),
            digest: hearth_lib::encode::Digest([2; 32]),
            parts: Box::new([])
        }),
        "a complete client spec translates back"
    );
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
fn outcome_back(outcome: records::outcome::Outcome) -> Option<render::vocabulary::Outcome> {
    match outcome {
        records::outcome::Outcome::Done => Some(render::vocabulary::Outcome::Done),
        records::outcome::Outcome::Failed => Some(render::vocabulary::Outcome::Failed),
        records::outcome::Outcome::Usage => Some(render::vocabulary::Outcome::Usage),
        records::outcome::Outcome::TryAgain => Some(render::vocabulary::Outcome::TryAgain),
        records::outcome::Outcome::Negative => None, // hearth 25
    }
}
const CLIENT_OUTCOMES: &[(render::vocabulary::Outcome, records::outcome::Outcome)] = &[
    (render::vocabulary::Outcome::Done, records::outcome::Outcome::Done),
    (render::vocabulary::Outcome::Failed, records::outcome::Outcome::Failed),
    (render::vocabulary::Outcome::Usage, records::outcome::Outcome::Usage),
    (render::vocabulary::Outcome::TryAgain, records::outcome::Outcome::TryAgain),
];
#[test]
fn each_client_outcome_translates_and_back() {
    for (source, expected) in CLIENT_OUTCOMES {
        assert_eq!(answers::outcome(*source), *expected, "outcome {source:?}");
        assert_eq!(outcome_back(*expected), Some(*source), "outcome {expected:?}");
    }
}

fn config_back(config: records::parts::config::Config) -> render::parts::config::Config {
    let mut tree = skein_lib::List::with_capacity(u32::try_from(config.tree.len()).expect("bounded tree"));
    for records::parts::config::TreeEntry { path, node } in config.tree {
        let node = match node {
            records::parts::config::Node::File { content_digest, mode, owner, group } => {
                render::parts::config::Node::File {
                    content_digest: hearth_lib::encode::Digest(content_digest),
                    mode,
                    owner,
                    group,
                }
            }
        };
        tree.push(render::parts::config::TreeEntry { path, node }).expect("tree room");
    }
    render::parts::config::Config { tree: tree.into_boxed() }
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
    let records::records::PartRecord::Config(config) = record;
    assert_eq!(
        config_back(config),
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
