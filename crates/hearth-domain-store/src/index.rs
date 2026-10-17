//! Recorded cache files, without reading their records (store.md, 8.1).
use hearth_lib::encode::Digest;
use skein_lib::Map;
pub(super) type Index = Map<Digest, bool>;
pub(super) fn unpaired(index: &Index, after: Option<Digest>) -> Option<Digest> {
    use core::ops::Bound::{Excluded, Unbounded};
    let start = match after {
        Some(digest) => Excluded(digest),
        None => Unbounded,
    };
    for (digest, paired) in index.range((start, Unbounded)) {
        if !paired {
            return Some(*digest);
        }
    }
    None
}
