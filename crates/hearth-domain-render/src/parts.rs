//! Specs and part digests (render.md, 4.4, 4.5).
use crate::revision::Oid;
use alloc::boxed::Box;
use hearth_lib::{encode::Digest, hash::H};
pub mod config;
pub mod meta;
pub mod packages;
pub mod secrets;
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
    Secrets(secrets::Secrets),
    Packages(packages::Packages),
}
#[must_use]
pub fn spec_digest(meta: &Digest, parts: &[Part]) -> Digest {
    let mut hash = H::new(b"spec", 1);
    let record = hash.values();
    record.map(u64::try_from(parts.len()).expect("bounded parts").saturating_add(1));
    let mut ordered = skein_lib::Map::with_capacity(u32::try_from(parts.len()).expect("part count").saturating_add(1));
    ordered.insert(skein_lib::bytes::copy_of(b"meta"), *meta).expect("part count");
    for part in parts {
        let name: &[u8] = match &part.record {
            PartRecord::Config(_) => b"config",
            PartRecord::Secrets(_) => b"secrets",
            PartRecord::Packages(_) => b"packages",
        };
        ordered.insert(skein_lib::bytes::copy_of(name), part.digest).expect("part count");
    }
    for (name, digest) in &ordered {
        record.key(name);
        record.digest(digest);
    }
    hash.finish()
}
