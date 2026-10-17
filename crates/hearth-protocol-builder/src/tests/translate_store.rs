use super::super::fixtures;
use crate::{
    builder::store::vocabulary::{CacheFile, Origin},
    translate, tree,
};
fn back(record: tree::cache_file::CacheFile) -> CacheFile {
    let tree::cache_file::CacheFile { at, origin, sha256, size, took } = record;
    let origin = match origin {
        tree::cache_file::Origin::Fetch { uri } => Origin::Fetch { uri },
        tree::cache_file::Origin::Index { resolution } => Origin::Index { resolution },
        tree::cache_file::Origin::Blob { oid } => Origin::Blob { oid },
        tree::cache_file::Origin::Rendered { path } => Origin::Rendered { path },
    };
    CacheFile { at, origin, sha256, size, took }
}
#[test]
fn each_cache_file_field_and_origin_translates_and_back() {
    for (row, input) in fixtures::cache_records().into_iter().enumerate() {
        let record = back(translate::store::cache_file(input.clone()));
        assert_eq!(record, input, "row {row}: cache provenance and cost");
    }
}

#[test]
fn each_inputs_value_translates_and_back() {
    for (row, input) in fixtures::every_inputs().into_iter().enumerate() {
        let values = translate::store::inputs(input.clone());
        let back = tree::resolution::Inputs {
            apt: values.apt,
            request: values.request,
            sources: values.sources,
            variant: values.variant,
        };
        assert_eq!(back, input, "row {row}: each canonical value survives");
    }
}
