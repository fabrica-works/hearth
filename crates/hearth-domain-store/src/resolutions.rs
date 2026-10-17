//! Canonical resolution and closure hashes (store.md, 6.4).
use crate::vocabulary::{Closed, Inputs};
use hearth_lib::{encode::Digest, hash::H};
#[must_use]
pub fn key(inputs: &Inputs) -> Digest {
    let mut hash = H::new(b"resolution", 4);
    for value in [&inputs.sources, &inputs.request, &inputs.variant, &inputs.apt] {
        for item in value {
            hash.values().item(item);
        }
    }
    hash.finish()
}
#[must_use]
pub fn digest(closure: &[Closed]) -> Digest {
    let mut hash = H::new(b"closure", 1);
    hash.values().list(u64::try_from(closure.len()).expect("bounded closure"));
    for package in closure {
        hash.values().list(4);
        hash.values().bytes(&package.name);
        hash.values().bytes(&package.architecture);
        hash.values().bytes(&package.version);
        hash.values().digest(&package.sha256);
    }
    hash.finish()
}
