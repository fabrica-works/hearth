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
        target::Document { at: target::Position { line: 2, column: 4 }, layer: target::Layer { placement: None } },
        "no placement is introduced"
    );
}
