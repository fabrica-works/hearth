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

#[must_use]
pub fn resolution(record: tree::resolution::Resolution) -> crate::builder::store::vocabulary::Resolution {
    use crate::builder::store::vocabulary::{Closed, Costs, Index, Inputs, Line, Made, Resolution};
    let mut closure = skein_lib::List::with_capacity(u32::try_from(record.closure.len()).expect("closure bound"));
    for package in record.closure {
        closure
            .push(Closed {
                name: package.name,
                architecture: package.architecture,
                version: package.version,
                source: package.source,
                filename: package.filename,
                size: package.size,
                sha256: package.sha256,
            })
            .expect("closure bound");
    }
    let mut sources = skein_lib::List::with_capacity(u32::try_from(record.sources.len()).expect("sources bound"));
    for line in record.sources {
        let mut indices = skein_lib::List::with_capacity(u32::try_from(line.indices.len()).expect("indices bound"));
        for index in line.indices {
            indices.push(Index { path: index.path, size: index.size, sha256: index.sha256 }).expect("indices bound");
        }
        sources
            .push(Line { uri: line.uri, suite: line.suite, components: line.components, indices: indices.into_boxed() })
            .expect("sources bound");
    }
    Resolution {
        key: record.key,
        digest: record.digest,
        inputs: Inputs {
            apt: record.inputs.apt,
            request: record.inputs.request,
            sources: record.inputs.sources,
            variant: record.inputs.variant,
        },
        closure: closure.into_boxed(),
        sources: sources.into_boxed(),
        made: Made {
            at: record.made.at,
            apt: record.made.apt,
            mmdebstrap: record.made.mmdebstrap,
            costs: Costs { fetched: record.made.costs.fetched, took: record.made.costs.took },
        },
    }
}

#[must_use]
pub fn progress(progress: crate::builder::vocabulary::Progress, store: &[u8]) -> crate::records::records::Record {
    use crate::{
        builder::{store::vocabulary::Directory, vocabulary::Progress},
        records::{
            progress::{Strays, Waiting},
            records::Record,
        },
    };
    let store = skein_lib::bytes::copy_of(store);
    match progress {
        Progress::Waiting => Record::Waiting(Waiting { store }),
        Progress::Strays { count, first } => {
            let directory: &[u8] = match first.directory {
                Directory::Top => b".",
                Directory::Cache => b"cache",
                Directory::Resolutions => b"resolutions",
            };
            let mut path = skein_lib::Writer::new(directory.len().saturating_add(1).saturating_add(first.name.len()));
            path.put(directory).expect("directory room");
            path.put(b"/").expect("separator room");
            path.put(&first.name).expect("name room");
            Record::Strays(Strays { store, count, first: path.finish() })
        }
    }
}
#[must_use]
pub fn tree_resolution(record: crate::builder::store::vocabulary::Resolution) -> tree::resolution::Resolution {
    use tree::resolution::{Costs, Index, Inputs, Line, Made, Package, Resolution};
    let mut closure = skein_lib::List::with_capacity(u32::try_from(record.closure.len()).expect("closure bound"));
    for package in record.closure {
        closure
            .push(Package {
                name: package.name,
                architecture: package.architecture,
                version: package.version,
                source: package.source,
                filename: package.filename,
                size: package.size,
                sha256: package.sha256,
            })
            .expect("closure bound");
    }
    let mut sources = skein_lib::List::with_capacity(u32::try_from(record.sources.len()).expect("sources bound"));
    for line in record.sources {
        let mut indices = skein_lib::List::with_capacity(u32::try_from(line.indices.len()).expect("indices bound"));
        for index in line.indices {
            indices.push(Index { path: index.path, size: index.size, sha256: index.sha256 }).expect("indices bound");
        }
        sources
            .push(Line { uri: line.uri, suite: line.suite, components: line.components, indices: indices.into_boxed() })
            .expect("sources bound");
    }
    Resolution {
        key: record.key,
        digest: record.digest,
        inputs: Inputs {
            apt: record.inputs.apt,
            request: record.inputs.request,
            sources: record.inputs.sources,
            variant: record.inputs.variant,
        },
        closure: closure.into_boxed(),
        sources: sources.into_boxed(),
        made: Made {
            at: record.made.at,
            apt: record.made.apt,
            mmdebstrap: record.made.mmdebstrap,
            costs: Costs { fetched: record.made.costs.fetched, took: record.made.costs.took },
        },
    }
}
