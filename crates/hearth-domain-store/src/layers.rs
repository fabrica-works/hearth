//! Published layers, indexed by key from their records (store.md, 6.2, 8.1 and 8.2).
use crate::{
    Store,
    scan::Scan,
    vocabulary::{Answer, Directory, Layer, LayerInputs, Output},
};
use hearth_lib::{encode::Digest, hash::H};
use skein_lib::Queue;
/// Answers whether a layer of this key is published, from the index alone.
pub(super) fn want(store: &Store, key: Digest, out: &mut Queue<Output>) {
    out.push(Output::Answer(if store.scan.layers.contains_key(&key) {
        Answer::Have { key }
    } else {
        Answer::Free { key }
    }));
}
/// The key a record's kind, parent and inputs make (store.md, 3.3), if they agree.
#[must_use]
pub fn key(record: &Layer) -> Option<Digest> {
    let empty = H::new(b"empty", 0).finish();
    let (tag, beneath, part) = match record.inputs {
        LayerInputs::Empty => return if record.parent.is_none() { Some(empty) } else { None },
        LayerInputs::Packages { part, resolution } => {
            if record.parent != Some(empty) {
                return None;
            }
            (b"packages".as_slice(), resolution, part)
        }
        LayerInputs::Software { part } => (b"software".as_slice(), record.parent?, part),
        LayerInputs::Config { part } => (b"config".as_slice(), record.parent?, part),
    };
    let mut hash = H::new(tag, 2);
    hash.values().digest(&beneath);
    hash.values().digest(&part);
    Some(hash.finish())
}
/// Indexes the record the scan loaded, or refuses it and reports it once among
/// the scan's strays (8.2, step 7); false when the index is full.
pub(super) fn index(scan: &mut Scan, record: Option<Layer>) -> bool {
    let (expected, name) = scan.layer.take().expect("a layer record load");
    match record {
        Some(record) if record.key == expected && key(&record) == Some(expected) => {
            // Matching the record to its subvolume (4.4, 4.5) needs skein's
            // btrfs calls, which come with the task that makes them: here.
            scan.layers.insert(expected, record).is_ok()
        }
        Some(_) | None => {
            scan.refuse(Directory::Layers, name);
            true
        }
    }
}
