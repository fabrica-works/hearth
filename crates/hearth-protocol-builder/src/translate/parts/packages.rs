//! Packages cross into records without interpreting their fields.
use crate::{records::parts::packages as target, render::parts::packages as source};
use alloc::boxed::Box;
use skein_lib::List;
#[must_use]
pub fn packages(packages: source::Packages) -> target::Packages {
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
pub fn change(value: source::Change) -> target::Change {
    let source::Change { resolution, request, sources, variant, preseeds, initramfs, apt } = value;
    let source::Names { added, removed } = request;
    target::Change {
        resolution: crate::translate::changes::resolution(resolution),
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

fn file(digest: Option<crate::render::vocabulary::Digest>) -> Option<[u8; 32]> {
    Some(digest?.0)
}
