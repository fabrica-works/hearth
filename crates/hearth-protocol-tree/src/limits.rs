//! The configured record bounds (store.md, 6.1).

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
