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
