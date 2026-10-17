//! Live paths and their ordered consumers (render.md, 4.4).
use crate::{
    entries::Expansion,
    errors::Error,
    revision::declaration::{Map, Name},
};
use alloc::boxed::Box;
use hearth_lib::{encode::Digest, hash::H};
use skein_lib::{List, Map as Ordered};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Live {
    pub paths: Box<[Covered]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Covered {
    pub path: Box<[u8]>,
    pub consumers: Box<[Consumer]>,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Consumer {
    pub task: Box<[u8]>,
    pub unit: Option<Box<[u8]>>,
    pub action: Action,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Action {
    None,
    Reload,
    ReloadOrRestart,
    Restart,
    Start,
}
impl Action {
    #[must_use]
    pub fn name(self) -> &'static [u8] {
        match self {
            Self::Reload => b"reload",
            Self::Restart => b"restart",
            Self::ReloadOrRestart => b"reload-or-restart",
            Self::Start => b"start",
            Self::None => b"none",
        }
    }
}
pub fn build(expansion: &Expansion) -> Result<Live, Box<[Error]>> {
    let entries = expansion.of(Map::Live);
    let mut capacity = 0_u32;
    for located in &entries {
        capacity = capacity.saturating_add(
            u32::try_from(located.entry.strings(Name::Paths).unwrap_or(&[]).len()).expect("path count"),
        );
    }
    let count = u32::try_from(entries.len()).expect("entry count");
    let mut rows = Ordered::with_capacity(capacity);
    let mut faults = List::with_capacity(count);
    for located in &entries {
        match crate::kinds::live::entry(located) {
            Ok(consumer) => {
                for path in located.entry.strings(Name::Paths).expect("checked paths") {
                    if !rows.contains_key(path) {
                        rows.insert(path.clone(), Ordered::with_capacity(count)).expect("path count");
                    }
                    rows.get_mut(path).expect("covered path").insert(consumer.clone(), ()).expect("entry count");
                }
            }
            Err(error) => {
                faults.push(*error).expect("entry count");
            }
        }
    }
    if !faults.is_empty() {
        return Err(faults.into_boxed());
    }
    let mut paths = List::with_capacity(rows.len());
    for (path, entries) in &rows {
        let mut consumers = List::with_capacity(entries.len());
        for (consumer, ()) in entries {
            consumers.push(consumer.clone()).expect("consumer count");
        }
        paths.push(Covered { path: path.clone(), consumers: consumers.into_boxed() }).expect("path count");
    }
    Ok(Live { paths: paths.into_boxed() })
}
#[must_use]
pub fn digest(live: &Live) -> Digest {
    let mut hash = H::new(b"part:live", 1);
    let record = hash.values();
    record.map(u64::try_from(live.paths.len()).expect("path count"));
    for covered in &live.paths {
        record.key(&covered.path);
        record.list(u64::try_from(covered.consumers.len()).expect("consumer count"));
        for consumer in &covered.consumers {
            record.map(if consumer.unit.is_some() { 3 } else { 2 });
            record.key(b"action");
            record.bytes(consumer.action.name());
            record.key(b"task");
            record.bytes(&consumer.task);
            if let Some(unit) = &consumer.unit {
                record.key(b"unit");
                record.bytes(unit);
            }
        }
    }
    hash.finish()
}
