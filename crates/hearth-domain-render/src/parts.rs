//! Specs and part digests (render.md, 4.4, 4.5).
use crate::revision::Oid;
use alloc::boxed::Box;
use hearth_lib::{encode::Digest, hash::H};
pub mod config;
pub mod meta;
pub use meta::Meta;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Spec {
    pub meta: Meta,
    pub meta_digest: Digest,
    pub commit: Oid,
    pub digest: Digest,
    pub parts: Box<[Part]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Part {
    pub digest: Digest,
    pub record: PartRecord,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PartRecord {
    Config(config::Config),
}
#[must_use]
pub fn spec_digest(meta: &Digest, parts: &[Part]) -> Digest {
    let mut hash = H::new(b"spec", 1);
    let record = hash.values();
    record.map(u64::try_from(parts.len()).expect("bounded parts").saturating_add(1));
    for part in parts {
        match &part.record {
            PartRecord::Config(_) => record.key(b"config"),
        }
        record.digest(&part.digest);
    }
    record.key(b"meta");
    record.digest(meta);
    hash.finish()
}
