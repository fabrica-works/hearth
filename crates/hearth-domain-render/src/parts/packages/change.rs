//! Typed packages differences (render.md, 6.1).
use super::{Apt, Box, Initramfs, List, Packages, Preseed, Sources, Variant};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub resolution: crate::diff::ResolutionChange,
    pub request: Names,
    pub sources: Option<SourcesPair>,
    pub variant: Option<VariantPair>,
    pub preseeds: Option<PreseedsPair>,
    pub initramfs: Option<InitramfsPair>,
    pub apt: Option<AptPair>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Names {
    pub added: Box<[Box<[u8]>]>,
    pub removed: Box<[Box<[u8]>]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SourcesPair {
    pub before: Sources,
    pub after: Sources,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct VariantPair {
    pub before: Variant,
    pub after: Variant,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PreseedsPair {
    pub before: Box<[Preseed]>,
    pub after: Box<[Preseed]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InitramfsPair {
    pub before: Option<Initramfs>,
    pub after: Option<Initramfs>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AptPair {
    pub before: Apt,
    pub after: Apt,
}

#[must_use]
pub fn change(before: &Packages, after: &Packages) -> Change {
    Change {
        resolution: crate::diff::resolution(
            crate::keys::resolution(&super::inputs(before)),
            crate::keys::resolution(&super::inputs(after)),
            None,
            None,
        ),
        request: Names {
            added: difference(&after.request, &before.request),
            removed: difference(&before.request, &after.request),
        },
        sources: if before.sources == after.sources {
            None
        } else {
            Some(SourcesPair { before: before.sources.clone(), after: after.sources.clone() })
        },
        variant: if before.variant == after.variant {
            None
        } else {
            Some(VariantPair { before: before.variant, after: after.variant })
        },
        preseeds: if before.preseeds == after.preseeds {
            None
        } else {
            Some(PreseedsPair { before: before.preseeds.clone(), after: after.preseeds.clone() })
        },
        initramfs: if before.initramfs == after.initramfs {
            None
        } else {
            Some(InitramfsPair { before: before.initramfs.clone(), after: after.initramfs.clone() })
        },
        apt: if before.apt == after.apt {
            None
        } else {
            Some(AptPair { before: before.apt.clone(), after: after.apt.clone() })
        },
    }
}
fn difference(names: &[Box<[u8]>], other: &[Box<[u8]>]) -> Box<[Box<[u8]>]> {
    let mut changed = List::with_capacity(u32::try_from(names.len()).expect("bounded request"));
    for name in names {
        if !other.contains(name) {
            changed.push(name.clone()).expect("request capacity");
        }
    }
    let mut changed = changed.into_boxed();
    changed.sort();
    changed
}
