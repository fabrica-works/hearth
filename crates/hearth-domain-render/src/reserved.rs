//! Refuse reserved entries before kinds validate them (format.md, 5.11).
mod paths;
mod words;
use crate::{
    entries::{Expansion, Located},
    errors::{Collector, Error, Kind, Place, Reserved, Rule},
    parts::meta::Meta,
    revision::{Position, Runs, declaration::Map},
    vocabulary::Class,
};
use alloc::boxed::Box;
use skein_lib::{List, Writer, bytes::copy_of};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Spot {
    Declared(Position),
    Tree,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Refused {
    pub task: Box<[u8]>,
    pub spot: Spot,
    pub also: Option<Position>,
    pub reserved: Reserved,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Shape {
    File { empty: bool },
    Link,
    Directory,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Installed<'a> {
    pub task: &'a [u8],
    pub path: &'a [u8],
    pub at: Option<Position>,
    pub shape: Shape,
}
#[must_use]
pub fn tree(installed: &[Installed<'_>]) -> Box<[Refused]> {
    paths::tree(installed)
}
fn first(row: &Located<'_>, map: Map, container: bool) -> Option<Refused> {
    let words = words::entry(row, map, container);
    let paths = paths::entry(row, map);
    match (words, paths) {
        (Some(words), Some(paths)) => {
            if words.spot == Spot::Declared(row.entry.at) {
                return Some(words);
            }
            if paths.spot == Spot::Declared(row.entry.at) {
                return Some(paths);
            }
            for field in &row.entry.fields {
                if words.spot == Spot::Declared(field.at) {
                    return Some(words);
                }
                if paths.spot == Spot::Declared(field.at) {
                    return Some(paths);
                }
            }
            Some(words)
        }
        (Some(fault), None) | (None, Some(fault)) => Some(fault),
        (None, None) => None,
    }
}
fn refused(row: &Located<'_>, at: Position, rule: Rule, what: &[u8]) -> Refused {
    Refused {
        task: copy_of(row.task),
        spot: Spot::Declared(at),
        also: None,
        reserved: Reserved { rule, what: copy_of(what) },
    }
}
/// Every entry gives its first fault, then is hidden from the kind readers.
#[must_use]
pub fn entries(expansion: &mut Expansion, target: &[u8], container: bool) -> Box<[Refused]> {
    let mut count = 0_u32;
    for task in &expansion.tasks {
        for group in &task.groups {
            count = count.saturating_add(u32::try_from(group.entries.len()).expect("bounded entries"));
        }
    }
    let mut pending = List::with_capacity(count);
    for task in &expansion.tasks {
        for group in &task.groups {
            for entry in &group.entries {
                let row = Located { task: &task.task, entry };
                if let Some(fault) = first(&row, group.map, container) {
                    pending.push(Pending { map: group.map, key: entry.key.clone(), fault }).expect("entry count");
                }
            }
        }
    }
    let mut faults = List::with_capacity(count);
    for Pending { map, key, fault } in pending.into_boxed() {
        expansion.refuse(map, &fault.task, &key);
        faults.push(fault).expect("entry count");
    }
    words::length(expansion, target, &mut faults);
    faults.into_boxed()
}
#[derive(Debug)]
struct Pending {
    map: Map,
    key: Box<[u8]>,
    fault: Refused,
}
pub(crate) fn check(expansion: &mut Expansion, meta: &Meta, errors: &mut Collector) -> bool {
    let faults = entries(expansion, &meta.target, meta.placement.kind == Runs::Container);
    let valid = faults.is_empty();
    for fault in faults {
        errors.push(error(fault, meta));
    }
    valid
}
pub(crate) fn error(fault: Refused, meta: &Meta) -> Error {
    let tree = fault.spot == Spot::Tree;
    let suffix = if tree { b"/tree".as_slice() } else { b"/task.json" };
    let extra = if tree { fault.reserved.what.len() } else { 0 };
    let mut path = Writer::new(
        fault.task.len().saturating_add(b"tasks/".len()).saturating_add(suffix.len()).saturating_add(extra),
    );
    path.put(b"tasks/").expect("prefix");
    path.put(&fault.task).expect("task");
    path.put(suffix).expect("suffix");
    if tree {
        path.put(&fault.reserved.what).expect("tree path");
    }
    let path = path.finish();
    let position = match fault.spot {
        Spot::Declared(at) => Some(at),
        Spot::Tree => None,
    };
    Error {
        side: None,
        kind: Kind::Reserved(fault.reserved),
        class: Class::Configuration,
        at: Some(Place { path: path.clone(), position }),
        also: also(&path, fault.also),
        target: Some(meta.target.clone()),
        task: Some(fault.task),
    }
}

fn also(path: &[u8], position: Option<Position>) -> Option<Place> {
    Some(Place { path: copy_of(path), position: Some(position?) })
}
