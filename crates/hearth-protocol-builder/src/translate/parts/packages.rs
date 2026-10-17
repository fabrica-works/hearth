//! Packages cross into records without interpreting their fields.
use crate::{records::parts::packages as target, render::parts::packages as source};
use skein_lib::List;
#[must_use]
pub fn packages(packages: source::Packages) -> target::Packages {
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
