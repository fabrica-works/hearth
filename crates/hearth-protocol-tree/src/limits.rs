//! The configured record bounds (domain/store.md, 6.1).

use crate::{
    decode,
    frame::Kind,
    layer::{Step, Tool},
    resolution::{Index, Line, Package},
    tree::{self, Kernel, Xattr},
};
use alloc::boxed::Box;
use hearth_lib::encode::{self, Digest, Item};
use skein_lib::List;

/// The records' configured limits, including the decoder's own bounds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub depth: u32,
    pub key: u32,
    pub store: StoreLimits,
    pub layer: LayerLimits,
    pub tree: TreeLimits,
    pub resolution: ResolutionLimits,
    pub cache_file: CacheFileLimits,
}
/// Kind 1's body.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StoreLimits {
    pub body: u32,
}
/// Kind 2's body and lists: `steps` counts the steps in `costs`, `name` bounds
/// a tool's name or version.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LayerLimits {
    pub body: u32,
    pub placed: u32,
    pub tools: u32,
    pub steps: u32,
    pub name: u32,
}
/// Kind 3's body, an item in it, the kernels in `boot`, and the longest map
/// key in an entry: an attribute's name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TreeLimits {
    pub body: u32,
    pub item: u32,
    pub kernels: u32,
    pub key: u32,
}
/// Kind 4's body and fields; `inputs` counts the four input values' items.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ResolutionLimits {
    pub body: u32,
    pub packages: u32,
    pub lines: u32,
    pub indices: u32,
    pub path: u32,
    pub name: u32,
    pub inputs: u32,
}
/// Kind 5's body, URI or path, and object id.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CacheFileLimits {
    pub body: u32,
    pub path: u32,
    pub oid: u32,
}
/// Store.md, 6.1's defaults and the plan's decoder bounds.
pub const SHIPPED: Limits = Limits {
    depth: 8,
    key: 64,
    store: StoreLimits { body: 4_096 },
    layer: LayerLimits { body: 1_048_576, placed: 4_096, tools: 256, steps: 64, name: 256 },
    tree: TreeLimits { body: 1_073_741_824, item: 4_718_592, kernels: 8, key: 255 },
    resolution: ResolutionLimits {
        body: 4_194_304,
        packages: 8_192,
        lines: 8,
        indices: 16,
        path: 4_096,
        name: 256,
        inputs: 8_256,
    },
    cache_file: CacheFileLimits { body: 8_192, path: 4_096, oid: 64 },
};

/// What a read holds beside its returned record, including a failed read's
/// partial record (programming-model.md, 6.3). Returns `None` past the
/// encoder's depth, or past a u64 for a resolution.
#[must_use]
pub fn worst_case(kind: Kind, limits: &Limits) -> Option<u64> {
    let own = decode::worst_case(&Limits { key: key(kind, limits), ..*limits })?;
    let inputs = match kind {
        Kind::Resolution => List::<Item>::worst_case(limits.resolution.inputs)?,
        Kind::Store | Kind::Layer | Kind::Tree | Kind::CacheFile => 0,
    };
    // A body is read whole, but a tree file's reader keeps the last path.
    let buffer = match kind {
        Kind::Tree => u64::from(tree::path::PATH),
        Kind::Store | Kind::Layer | Kind::Resolution | Kind::CacheFile => u64::from(kind.bound(limits)),
    };
    own.checked_add(inputs)?.checked_add(held(kind, limits)?)?.checked_add(buffer)
}

/// The longest map key a kind's decoder takes.
const fn key(kind: Kind, limits: &Limits) -> u32 {
    match kind {
        Kind::Tree => limits.tree.key,
        Kind::Store | Kind::Layer | Kind::Resolution | Kind::CacheFile => limits.key,
    }
}

/// The most heap a decoded record holds (domain/store.md, 6.1), for its owner's
/// worst case. Returns `None` past the encoder's depth, or past a u64 for
/// a resolution. Boxed bytes together fit within the body's bound; a tree
/// file's, within an item's.
#[must_use]
pub fn held(kind: Kind, limits: &Limits) -> Option<u64> {
    if limits.depth > encode::DEPTH {
        return None;
    }
    let l = limits.resolution;
    let lists = match kind {
        Kind::Store => return Some(0),
        Kind::CacheFile => 0,
        Kind::Layer => List::<Digest>::worst_case(limits.layer.placed)?
            .checked_add(List::<Tool>::worst_case(limits.layer.tools)?)?
            .checked_add(List::<Step>::worst_case(limits.layer.steps)?)?,
        Kind::Tree => List::<Xattr>::worst_case(tree::XATTRS)?
            .checked_add(List::<Kernel>::worst_case(limits.tree.kernels)?)?
            .checked_add(u64::from(limits.tree.item))?,
        Kind::Resolution => {
            let line = List::<Box<[u8]>>::worst_case(l.inputs)?.checked_add(List::<Index>::worst_case(l.indices)?)?;
            List::<Package>::worst_case(l.packages)?
                .checked_add(List::<Line>::worst_case(l.lines)?)?
                .checked_add(u64::from(l.lines).checked_mul(line)?)?
                .checked_add(List::<Item>::worst_case(l.inputs)?)?
        }
    };
    match kind {
        Kind::Tree => Some(lists),
        Kind::Store | Kind::Layer | Kind::Resolution | Kind::CacheFile => {
            lists.checked_add(u64::from(kind.bound(limits)))
        }
    }
}
