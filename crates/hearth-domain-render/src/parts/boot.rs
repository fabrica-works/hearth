//! Boot parameters and mutability (render.md, 4.3).
use crate::{
    entries::Expansion,
    errors::Error,
    revision::{self, declaration::Map},
};
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub before: Boot,
    pub after: Boot,
}
use hearth_lib::{encode::Digest, hash::H};
use skein_lib::{List, Map as Sorted};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Boot {
    pub params: Box<[Box<[u8]>]>,
    pub mutable: bool,
}
#[must_use]
pub fn digest(boot: &Boot) -> Digest {
    let mut hash = H::new(b"part:boot", 1);
    let record = hash.values();
    record.map(2);
    record.key(b"mutable");
    record.boolean(boot.mutable);
    record.key(b"params");
    record.list(u64::try_from(boot.params.len()).expect("bounded parameters"));
    for param in &boot.params {
        record.bytes(param);
    }
    hash.finish()
}
pub fn build(expansion: &Expansion, placement: &revision::Placement) -> Result<Boot, Box<[Error]>> {
    if placement.kind == revision::Runs::Container {
        return Ok(Boot { params: Box::new([]), mutable: placement.mutable });
    }
    let entries = expansion.of(Map::Cmdline);
    let capacity = u32::try_from(entries.len()).expect("bounded expansion");
    let mut names = Sorted::with_capacity(capacity);
    let mut params = Sorted::with_capacity(capacity);
    let mut faults = List::with_capacity(capacity);
    for located in &entries {
        match crate::kinds::cmdline::entry(located.entry) {
            Ok(param) => {
                // transitional: hearth 22
                if names.contains_key(&located.entry.key) {
                    continue;
                }
                names.insert(located.entry.key.clone(), ()).expect("entry count");
                params
                    .insert(
                        (param.order, located.entry.key.clone(), skein_lib::bytes::copy_of(located.task)),
                        param.param,
                    )
                    .expect("entry count");
            }
            Err((schema, at)) => {
                faults
                    .push(super::packages::build::error(schema, Some(located.task), Some(at), None))
                    .expect("entry count");
            }
        }
    }
    if !faults.is_empty() {
        return Err(faults.into_boxed());
    }
    let mut ordered = List::with_capacity(params.len());
    for (_, param) in &params {
        ordered.push(param.clone()).expect("entry count");
    }
    Ok(Boot { params: ordered.into_boxed(), mutable: placement.mutable })
}

#[must_use]
pub fn change(before: &Boot, after: &Boot) -> Change {
    Change { before: before.clone(), after: after.clone() }
}
