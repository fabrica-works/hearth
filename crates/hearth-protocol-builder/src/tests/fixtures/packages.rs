//! Package record translation fixtures.
use crate::{records::parts::packages as source, render::parts::packages as target};
use skein_lib::List;
#[must_use]
pub fn packages_back(packages: source::Packages) -> target::Packages {
    let source::Packages { sources, request, variant, preseeds, initramfs, apt } = packages;
    let source::Sources { suite, architecture, components, archive, updates, security } = sources;
    let mut translated = List::with_capacity(u32::try_from(components.len()).expect("four components"));
    for component in components {
        translated
            .push(match component {
                source::Component::Main => target::Component::Main,
                source::Component::Contrib => target::Component::Contrib,
                source::Component::NonFree => target::Component::NonFree,
                source::Component::NonFreeFirmware => target::Component::NonFreeFirmware,
            })
            .expect("component count");
    }
    let mut answers = List::with_capacity(u32::try_from(preseeds.len()).expect("bounded preseeds"));
    for source::Preseed { question, owner, kind, value } in preseeds {
        answers
            .push(target::Preseed {
                question,
                owner,
                value,
                kind: match kind {
                    source::PreseedKind::Select => target::PreseedKind::Select,
                    source::PreseedKind::Multiselect => target::PreseedKind::Multiselect,
                    source::PreseedKind::String => target::PreseedKind::String,
                    source::PreseedKind::Boolean => target::PreseedKind::Boolean,
                },
            })
            .expect("preseed count");
    }
    let initramfs = optional_initramfs(initramfs);
    let source::Apt { install_recommends } = apt;
    target::Packages {
        sources: target::Sources {
            suite,
            architecture,
            components: translated.into_boxed(),
            archive,
            updates,
            security,
        },
        request,
        variant: match variant {
            source::Variant::Apt => target::Variant::Apt,
            source::Variant::Container => target::Variant::Container,
        },
        preseeds: answers.into_boxed(),
        initramfs,
        apt: target::Apt { install_recommends },
    }
}

#[must_use]
pub fn package_records() -> [target::Packages; 2] {
    use skein_lib::bytes::copy_of;
    [target::Variant::Apt, target::Variant::Container].map(|variant| target::Packages {
        sources: target::Sources {
            suite: copy_of(b"forky"),
            architecture: copy_of(b"arm64"),
            components: Box::new([
                target::Component::Main,
                target::Component::Contrib,
                target::Component::NonFree,
                target::Component::NonFreeFirmware,
            ]),
            archive: copy_of(b"20261001T030405Z"),
            updates: true,
            security: Some(copy_of(b"20261101T030405Z")),
        },
        request: Box::new([copy_of(b"curl"), copy_of(b"tzdata")]),
        variant,
        preseeds: Box::new(
            [
                target::PreseedKind::Select,
                target::PreseedKind::Multiselect,
                target::PreseedKind::String,
                target::PreseedKind::Boolean,
            ]
            .map(|kind| target::Preseed {
                question: copy_of(b"owner/question"),
                owner: copy_of(b"owner"),
                kind,
                value: copy_of(b"value"),
            }),
        ),
        initramfs: match variant {
            target::Variant::Apt => {
                Some(target::Initramfs { modules: Box::new([copy_of(b"nvme")]), microcode: target::Microcode::Generic })
            }
            target::Variant::Container => None,
        },
        apt: target::Apt { install_recommends: true },
    })
}

fn optional_initramfs(value: Option<source::Initramfs>) -> Option<target::Initramfs> {
    let source::Initramfs { modules, microcode } = value?;
    Some(target::Initramfs {
        modules,
        microcode: match microcode {
            source::Microcode::None => target::Microcode::None,
            source::Microcode::Generic => target::Microcode::Generic,
        },
    })
}
#[must_use]
pub fn boot_records() -> [crate::render::parts::boot::Boot; 2] {
    use skein_lib::bytes::copy_of;
    [
        crate::render::parts::boot::Boot { params: Box::new([]), mutable: false },
        crate::render::parts::boot::Boot { params: Box::new([copy_of(b"quiet"), copy_of(b"audit=0")]), mutable: true },
    ]
}
