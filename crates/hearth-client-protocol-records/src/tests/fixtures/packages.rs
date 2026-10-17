use crate::{
    parts::packages::*,
    records::{Part, PartRecord, Spec},
};
use skein_lib::bytes::copy_of;
pub fn packages_record() -> Packages {
    Packages {
        sources: Sources {
            suite: copy_of(b"forky"),
            architecture: copy_of(b"amd64"),
            components: Box::new(Component::ALL),
            archive: copy_of(b"20261001T030405Z"),
            updates: true,
            security: Some(copy_of(b"20261002T030405Z")),
        },
        request: Box::new([copy_of(b"curl"), copy_of(b"tzdata")]),
        variant: Variant::Apt,
        preseeds: Box::new([]),
        initramfs: Some(Initramfs { modules: Box::new([]), microcode: Microcode::Generic }),
        apt: Apt { install_recommends: false },
    }
}
pub fn packages_spec() -> Box<Spec> {
    let mut spec = super::x1_prod();
    spec.parts = Box::new([Part {
        digest: super::digest([0x4b, 0xe1, 0xc0, 0xd2, 0xa7, 0xf1]),
        record: PartRecord::Packages(packages_record()),
    }]);
    spec
}
pub fn packages_text(spec: Box<Spec>) -> Box<[u8]> {
    crate::form::print(crate::form::Form::Text, &crate::records::Record::Spec(spec), None, &crate::limits::SHIPPED)
        .expect("packages text")
}
pub fn preseeded_spec() -> Box<Spec> {
    let mut spec = packages_spec();
    for part in &mut spec.parts {
        if let PartRecord::Packages(packages) = &mut part.record {
            packages.preseeds = Box::new([
                Preseed {
                    question: copy_of(b"a/answer"),
                    owner: copy_of(b"a"),
                    kind: PreseedKind::String,
                    value: copy_of(b"word"),
                },
                Preseed {
                    question: copy_of(b"tzdata/Areas"),
                    owner: copy_of(b"other"),
                    kind: PreseedKind::Select,
                    value: copy_of(b"Europe"),
                },
            ]);
            packages.initramfs.as_mut().expect("metal").modules = Box::new([copy_of(b"btrfs"), copy_of(b"nvme")]);
        }
    }
    spec
}
pub fn boot_spec() -> Box<Spec> {
    let mut spec = super::x1_prod();
    spec.parts = Box::new([Part {
        digest: super::digest([0xbc, 0x6c, 0xab, 0x41, 0x72, 0x52]),
        record: PartRecord::Boot(crate::parts::boot::Boot {
            params: Box::new([copy_of(b"quiet"), copy_of(b"audit=0")]),
            mutable: true,
        }),
    }]);
    spec
}

#[must_use]
pub fn package_change(request: Names) -> crate::diff::PartChange {
    crate::diff::PartChange::Packages(Change {
        resolution: crate::diff::ResolutionChange::Same,
        request,
        sources: None,
        variant: None,
        preseeds: None,
        initramfs: None,
        apt: None,
    })
}
#[must_use]
pub fn change_record(part: crate::diff::PartChange) -> crate::records::Record {
    crate::records::Record::Change(crate::diff::Change {
        target: copy_of(b"ion-prod"),
        change: crate::diff::TargetChange::Changed { parts: Box::new([part]), rebuild: crate::diff::Rebuild::Nothing },
    })
}
#[must_use]
pub fn boot_change() -> crate::diff::PartChange {
    use crate::parts::boot::{Boot, Change};
    crate::diff::PartChange::Boot(Change {
        before: Boot { params: Box::new([copy_of(b"quiet")]), mutable: false },
        after: Boot { params: Box::new([]), mutable: true },
    })
}

#[must_use]
pub fn requested_packages_change() -> crate::diff::PartChange {
    package_change(Names { added: Box::new([copy_of(b"vim")]), removed: Box::new([copy_of(b"nano")]) })
}
#[must_use]
pub fn preseed_change() -> crate::diff::PartChange {
    let mut part = unchanged_packages_change();
    let crate::diff::PartChange::Packages(change) = &mut part else { panic!("packages") };
    let spec = preseeded_spec();
    for part in spec.parts {
        if let PartRecord::Packages(packages) = part.record {
            change.preseeds = Some(PreseedsPair { before: Box::new([]), after: packages.preseeds });
        }
    }
    part
}
#[must_use]
pub fn initramfs_change() -> crate::diff::PartChange {
    let mut part = unchanged_packages_change();
    let crate::diff::PartChange::Packages(change) = &mut part else { panic!("packages") };
    change.initramfs = Some(InitramfsPair {
        before: None,
        after: Some(Initramfs { modules: Box::new([copy_of(b"nvme")]), microcode: Microcode::Generic }),
    });
    part
}

#[must_use]
pub fn unchanged_packages_change() -> crate::diff::PartChange {
    package_change(Names { added: Box::new([]), removed: Box::new([]) })
}
