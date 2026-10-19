//! Typed packages differences (domain/render.md, 6.1).
use super::{Apt, Box, Initramfs, List, Packages, Preseed, Sources, Variant};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub resolution: crate::diff::ResolutionChange,
    pub request: Names,
    pub seeds: SeedsChange,
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
        seeds: seeds(&before.seeds, &after.seeds),
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SeedsChange {
    pub users: Box<[SeedUserKeyed]>,
    pub groups: Box<[SeedGroupKeyed]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SeedIds {
    pub uid: u32,
    pub gid: Option<u32>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SeedUserKeyed {
    Added { key: Box<[u8]>, value: SeedIds },
    Removed { key: Box<[u8]>, value: SeedIds },
    Changed { key: Box<[u8]>, before: SeedIds, after: SeedIds },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SeedGroupKeyed {
    Added { key: Box<[u8]>, value: u32 },
    Removed { key: Box<[u8]>, value: u32 },
    Changed { key: Box<[u8]>, before: u32, after: u32 },
}

fn seeds(before: &super::Seeds, after: &super::Seeds) -> SeedsChange {
    SeedsChange {
        users: crate::parts::secrets::changed(
            &before.users,
            &after.users,
            |row| &row.name,
            |key, before, after| match (before, after) {
                (None, Some(value)) => SeedUserKeyed::Added { key, value: SeedIds { uid: value.uid, gid: value.gid } },
                (Some(value), None) => {
                    SeedUserKeyed::Removed { key, value: SeedIds { uid: value.uid, gid: value.gid } }
                }
                (Some(before), Some(after)) => SeedUserKeyed::Changed {
                    key,
                    before: SeedIds { uid: before.uid, gid: before.gid },
                    after: SeedIds { uid: after.uid, gid: after.gid },
                },
                (None, None) => unreachable!("union entry"),
            },
        ),
        groups: crate::parts::secrets::changed(
            &before.groups,
            &after.groups,
            |row| &row.name,
            |key, before, after| match (before, after) {
                (None, Some(value)) => SeedGroupKeyed::Added { key, value: value.gid },
                (Some(value), None) => SeedGroupKeyed::Removed { key, value: value.gid },
                (Some(before), Some(after)) => SeedGroupKeyed::Changed { key, before: before.gid, after: after.gid },
                (None, None) => unreachable!("union entry"),
            },
        ),
    }
}
