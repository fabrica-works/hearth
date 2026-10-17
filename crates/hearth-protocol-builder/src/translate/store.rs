//! Records crossing the store boundary (store.md, 6.6).
use crate::{
    builder::store::vocabulary::{Range, StoreRecord},
    tree,
};
#[must_use]
pub fn store_record(record: tree::store::Store) -> StoreRecord {
    StoreRecord { layout: record.layout, made: record.made, range: store_range(record.range), uid: record.uid }
}
#[must_use]
pub fn tree_store(record: StoreRecord) -> tree::store::Store {
    tree::store::Store { layout: record.layout, made: record.made, range: tree_range(record.range), uid: record.uid }
}

fn store_range(range: Option<tree::store::Range>) -> Option<Range> {
    let range = range?;
    Some(Range { base: range.base, count: range.count })
}
fn tree_range(range: Option<Range>) -> Option<tree::store::Range> {
    let range = range?;
    Some(tree::store::Range { base: range.base, count: range.count })
}

#[must_use]
pub fn cache_file(record: crate::builder::store::vocabulary::CacheFile) -> tree::cache_file::CacheFile {
    use crate::builder::store::vocabulary::Origin;
    let crate::builder::store::vocabulary::CacheFile { at, origin, sha256, size, took } = record;
    let origin = match origin {
        Origin::Fetch { uri } => tree::cache_file::Origin::Fetch { uri },
        Origin::Index { resolution } => tree::cache_file::Origin::Index { resolution },
        Origin::Blob { oid } => tree::cache_file::Origin::Blob { oid },
        Origin::Rendered { path } => tree::cache_file::Origin::Rendered { path },
    };
    tree::cache_file::CacheFile { at, origin, sha256, size, took }
}
