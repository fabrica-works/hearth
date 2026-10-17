//! The configured record bounds (store.md, 6.1).

use crate::{
    decode,
    frame::Kind,
    resolution::{Index, Line, Package},
};
use alloc::boxed::Box;
use hearth_lib::encode::{self, Item};
use skein_lib::List;

/// The records' configured limits, including the decoder's own bounds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub depth: u32,
    pub key: u32,
    pub store: StoreLimits,
    pub resolution: ResolutionLimits,
    pub cache_file: CacheFileLimits,
}
/// Kind 1's body.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StoreLimits {
    pub body: u32,
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
    let own = decode::worst_case(limits)?;
    let inputs = match kind {
        Kind::Resolution => List::<Item>::worst_case(limits.resolution.inputs)?,
        Kind::Store | Kind::CacheFile => 0,
    };
    own.checked_add(inputs)?.checked_add(held(kind, limits)?)?.checked_add(u64::from(kind.bound(limits)))
}

/// The most heap a decoded record holds (store.md, 6.1), for its owner's
/// worst case. Returns `None` past the encoder's depth, or past a u64 for
/// a resolution. Boxed bytes together fit within the body's bound.
#[must_use]
pub fn held(kind: Kind, limits: &Limits) -> Option<u64> {
    if limits.depth > encode::DEPTH {
        return None;
    }
    let l = limits.resolution;
    let lists = match kind {
        Kind::Store => return Some(0),
        Kind::CacheFile => 0,
        Kind::Resolution => {
            let line = List::<Box<[u8]>>::worst_case(l.inputs)?.checked_add(List::<Index>::worst_case(l.indices)?)?;
            List::<Package>::worst_case(l.packages)?
                .checked_add(List::<Line>::worst_case(l.lines)?)?
                .checked_add(u64::from(l.lines).checked_mul(line)?)?
                .checked_add(List::<Item>::worst_case(l.inputs)?)?
        }
    };
    lists.checked_add(u64::from(kind.bound(limits)))
}
