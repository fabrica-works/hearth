//! Records crossing the store boundary (domain/store.md, 6.6).
use crate::{
    builder::store::vocabulary::{Range, StoreRecord},
    tree,
};
use alloc::boxed::Box;
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
            progress::{Resolved, Resolving, Strays, Unresolved, Waiting},
            records::Record,
        },
    };
    let store = skein_lib::bytes::copy_of(store);
    match progress {
        Progress::Resolving { key } => Record::Resolving(Resolving { key: key.0 }),
        Progress::Resolved { key, took } => Record::Resolved(Resolved { key: key.0, took }),
        Progress::Unresolved { key } => Record::Unresolved(Unresolved { key: key.0 }),
        Progress::Waiting => Record::Waiting(Waiting { store }),
        Progress::Strays { count, first } => {
            let directory: &[u8] = match first.directory {
                Directory::Top => b".",
                Directory::Cache => b"cache",
                Directory::Resolutions => b"resolutions",
                Directory::Layers => b"layers",
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

/// A layer's record, as far as the store's index reads it (domain/store.md, 6.2 and 8.1).
#[must_use]
pub fn layer(record: tree::layer::Layer) -> crate::builder::store::vocabulary::Layer {
    use crate::builder::store::vocabulary::{Layer, LayerInputs};
    Layer {
        key: record.key,
        parent: record.parent,
        inputs: match record.inputs {
            tree::layer::Inputs::Empty => LayerInputs::Empty,
            tree::layer::Inputs::Packages { part, resolution } => LayerInputs::Packages { part, resolution },
            tree::layer::Inputs::Software { part } => LayerInputs::Software { part },
            tree::layer::Inputs::Config { part } => LayerInputs::Config { part },
        },
        tree: record.tree,
        uuid: record.subvolume.uuid,
        at: record.made.at,
    }
}

/// A sealed layer's record as it is written (domain/store.md, 6.2). `tools` and `costs` are empty
/// and a packages layer is a scratch build with no base, until the tasks that fill them; the scan's
/// rules, which only a walk with them can name, are the zero digest, until the scan (domain/build.md, 9.3).
#[must_use]
pub fn tree_layer(record: &crate::builder::store::vocabulary::Record) -> tree::layer::Layer {
    use crate::builder::store::vocabulary::LayerInputs;
    use tree::layer::{Counts, Inputs, LayerKind, Line, Made, Subvolume, TreeFile};
    let (kind, inputs) = match record.layer.inputs {
        LayerInputs::Empty => (LayerKind::Empty, Inputs::Empty),
        LayerInputs::Packages { part, resolution } => (LayerKind::Packages, Inputs::Packages { part, resolution }),
        LayerInputs::Software { part } => (LayerKind::Software, Inputs::Software { part }),
        LayerInputs::Config { part } => (LayerKind::Config, Inputs::Config { part }),
    };
    let at = record.layer.at;
    let counts = record.walk.counts;
    tree::layer::Layer {
        changes: record.walk.changes,
        costs: Box::new([]),
        counts: Counts {
            bytes: counts.bytes,
            deletes: counts.deletes,
            entries: counts.entries,
            files: counts.files,
            hashed: counts.hashed,
            puts: counts.puts,
        },
        inputs,
        key: record.layer.key,
        kind,
        made: Made {
            at,
            packages: (kind == LayerKind::Packages).then_some(Line {
                base: None,
                depth: 0,
                how: tree::layer::How::Scratch,
                line: at,
            }),
        },
        parent: record.layer.parent,
        placed: if kind == LayerKind::Software { Some(Box::new([])) } else { None },
        scan: if kind == LayerKind::Empty { None } else { Some(crate::render::vocabulary::Digest([0; 32])) },
        subvolume: Subvolume { parent_uuid: record.parent_uuid, uuid: record.layer.uuid },
        tools: Box::new([]),
        tree: record.walk.tree,
        tree_file: TreeFile { sha256: record.walk.tree_file.sha256, size: record.walk.tree_file.size },
    }
}
