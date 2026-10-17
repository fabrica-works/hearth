//! H hashes the canonical encoding as it is written, without a copy (store.md, 3.2).

use crate::encode::{Digest, Encoder};

/// The encoding's version and hearth's constants', H's first value (store.md, 3.2).
pub const VERSION: &[u8] = b"hearth/1";

/// One digest being made: its list's head, version and tag hashed, its values to come.
#[derive(Debug)]
pub struct H {
    encoder: Encoder,
}

impl H {
    /// H(tag, v1, …, vn), n = `values`.
    #[must_use]
    pub fn new(tag: &[u8], values: u64) -> H {
        let count = values.checked_add(2).expect("H takes fewer than 2^64 - 2 values");
        let mut encoder = Encoder::hashing();
        encoder.list(count);
        encoder.bytes(VERSION);
        encoder.bytes(tag);
        H { encoder }
    }

    /// Where v1 … vn go, each one whole value.
    #[must_use]
    pub fn values(&mut self) -> &mut Encoder {
        &mut self.encoder
    }

    /// The digest, once each value is whole.
    #[must_use]
    pub fn finish(self) -> Digest {
        Digest(self.encoder.hashed())
    }
}
