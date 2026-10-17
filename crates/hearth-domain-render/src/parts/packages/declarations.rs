//! Sorted task declarations for the packages record.
use super::Preseed;
use crate::{entries::Expansion, errors::Error, revision::declaration::Map};
use alloc::boxed::Box;
use skein_lib::{List, Map as Sorted};
type Declarations = (Box<[Preseed]>, Box<[Box<[u8]>]>);
pub(super) fn build(expansion: &Expansion, initramfs: bool) -> Result<Declarations, Box<[Error]>> {
    let entries = expansion.of(Map::Preseeds);
    let modules = expansion.of(Map::Initramfs);
    let capacity = u32::try_from(entries.len().saturating_add(modules.len())).expect("bounded expansion");
    let mut preseeds = Sorted::with_capacity(capacity);
    let mut names = Sorted::with_capacity(capacity);
    let mut faults = List::with_capacity(capacity);
    for located in &entries {
        match crate::kinds::preseeds::entry(located.entry) {
            Ok(preseed) => {
                // transitional: hearth 22
                if preseeds.get(&preseed.question).is_none() {
                    preseeds.insert(preseed.question.clone(), preseed).expect("entry count");
                }
            }
            Err((schema, at)) => {
                faults.push(super::build::error(schema, Some(located.task), Some(at), None)).expect("entry count");
            }
        }
    }
    if initramfs {
        for located in &modules {
            match crate::kinds::preseeds::module(located.entry) {
                Ok(()) => {
                    names.insert(located.entry.key.clone(), ()).expect("entry count");
                }
                Err((schema, at)) => {
                    faults.push(super::build::error(schema, Some(located.task), Some(at), None)).expect("entry count");
                }
            }
        }
    }
    if !faults.is_empty() {
        return Err(faults.into_boxed());
    }
    let mut answers = List::with_capacity(preseeds.len());
    for (_, preseed) in &preseeds {
        answers.push(preseed.clone()).expect("entry count");
    }
    let mut modules = List::with_capacity(names.len());
    for (name, ()) in &names {
        modules.push(name.clone()).expect("entry count");
    }
    Ok((answers.into_boxed(), modules.into_boxed()))
}
