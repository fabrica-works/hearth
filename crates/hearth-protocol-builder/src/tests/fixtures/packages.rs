//! Packages cross into records without interpreting their fields.
use crate::{records::parts::packages as source, render::parts::packages as target};
use skein_lib::List;
#[must_use]
pub fn packages_back(packages: source::Packages) -> target::Packages {
    let source::Packages {
        sources: value,
        request,
        variant: kind,
        preseeds: answers,
        initramfs,
        initramfs_file,
        apt: options,
    } = packages;
    target::Packages {
        initramfs_file: file(initramfs_file),
        sources: sources(value),
        request,
        variant: variant(kind),
        preseeds: preseeds(answers),
        initramfs: optional_initramfs(initramfs),
        apt: apt(options),
    }
}
fn sources(sources: source::Sources) -> target::Sources {
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
    target::Sources { suite, architecture, components: translated.into_boxed(), archive, updates, security }
}
fn preseeds(preseeds: Box<[source::Preseed]>) -> Box<[target::Preseed]> {
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
    answers.into_boxed()
}
fn variant(value: source::Variant) -> target::Variant {
    match value {
        source::Variant::Apt => target::Variant::Apt,
        source::Variant::Container => target::Variant::Container,
    }
}
fn apt(value: source::Apt) -> target::Apt {
    let source::Apt { install_recommends } = value;
    target::Apt { install_recommends }
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
pub fn packages_change_back(value: source::Change) -> target::Change {
    let source::Change { resolution, request, sources, variant, preseeds, initramfs, apt } = value;
    let source::Names { added, removed } = request;
    target::Change {
        resolution: super::resolution_change_back(resolution),
        request: target::Names { added, removed },
        sources: sources_pair(sources),
        variant: variant_pair(variant),
        preseeds: preseeds_pair(preseeds),
        initramfs: initramfs_pair(initramfs),
        apt: apt_pair(apt),
    }
}
fn sources_pair(value: Option<source::SourcesPair>) -> Option<target::SourcesPair> {
    let source::SourcesPair { before, after } = value?;
    Some(target::SourcesPair { before: sources(before), after: sources(after) })
}
fn variant_pair(value: Option<source::VariantPair>) -> Option<target::VariantPair> {
    let source::VariantPair { before, after } = value?;
    Some(target::VariantPair { before: variant(before), after: variant(after) })
}
fn preseeds_pair(value: Option<source::PreseedsPair>) -> Option<target::PreseedsPair> {
    let source::PreseedsPair { before, after } = value?;
    Some(target::PreseedsPair { before: preseeds(before), after: preseeds(after) })
}
fn initramfs_pair(value: Option<source::InitramfsPair>) -> Option<target::InitramfsPair> {
    let source::InitramfsPair { before, after } = value?;
    Some(target::InitramfsPair { before: optional_initramfs(before), after: optional_initramfs(after) })
}
fn apt_pair(value: Option<source::AptPair>) -> Option<target::AptPair> {
    let source::AptPair { before, after } = value?;
    Some(target::AptPair { before: apt(before), after: apt(after) })
}

#[must_use]
pub fn package_records() -> [target::Packages; 2] {
    use skein_lib::bytes::copy_of;
    [target::Variant::Apt, target::Variant::Container].map(|variant| target::Packages {
        initramfs_file: Some(super::ABC),
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

#[must_use]
pub fn boot_records() -> [crate::render::parts::boot::Boot; 2] {
    use skein_lib::bytes::copy_of;
    [
        crate::render::parts::boot::Boot { params: Box::new([]), mutable: false },
        crate::render::parts::boot::Boot { params: Box::new([copy_of(b"quiet"), copy_of(b"audit=0")]), mutable: true },
    ]
}

#[must_use]
pub fn package_changes() -> [crate::render::parts::packages::Change; 2] {
    use crate::render::parts::packages::*;
    package_records().map(|packages| Change {
        resolution: crate::render::diff::ResolutionChange::NewKey,
        request: Names { added: packages.request.clone(), removed: Box::new([]) },
        sources: Some(SourcesPair { before: packages.sources.clone(), after: packages.sources }),
        variant: Some(VariantPair { before: Variant::Apt, after: packages.variant }),
        preseeds: Some(PreseedsPair { before: Box::new([]), after: packages.preseeds }),
        initramfs: Some(InitramfsPair { before: None, after: packages.initramfs }),
        apt: Some(AptPair { before: Apt { install_recommends: false }, after: packages.apt }),
    })
}

#[must_use]
pub fn boot_changes() -> [crate::render::parts::boot::Change; 2] {
    use crate::render::parts::boot::{Boot, Change};
    boot_records().map(|after| Change { before: Boot { params: Box::new([]), mutable: false }, after })
}
#[must_use]
pub fn boot_change_back(value: crate::records::parts::boot::Change) -> crate::render::parts::boot::Change {
    crate::render::parts::boot::Change {
        before: crate::render::parts::boot::Boot { params: value.before.params, mutable: value.before.mutable },
        after: crate::render::parts::boot::Boot { params: value.after.params, mutable: value.after.mutable },
    }
}

fn file(digest: Option<[u8; 32]>) -> Option<hearth_lib::encode::Digest> {
    Some(hearth_lib::encode::Digest(digest?))
}
