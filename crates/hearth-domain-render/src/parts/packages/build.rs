//! Read the fixed Debian schema and check every task's package declaration.
use super::{Apt, Component, Initramfs, Microcode, Packages, Sources, Variant};
use crate::{
    entries::Expansion,
    errors::{Error, Kind, Place, Schema},
    evaluate::Value,
    revision::{
        self,
        declaration::{Map, Name},
    },
    vocabulary::Class,
};
use alloc::boxed::Box;
use skein_lib::{List, Map as Names, Writer, bytes};
fn member<'a>(root: &'a Value, key: &[u8]) -> Option<&'a Value> {
    let Value::Map(entries) = root else { return None };
    for entry in entries {
        if entry.key.as_ref() == key {
            return entry.value.as_ref();
        }
    }
    None
}
fn text(root: &Value, key: &[u8]) -> Option<Box<[u8]>> {
    let Value::String(value) = member(root, key)? else { return None };
    Some(value.clone())
}
fn boolean(root: &Value, key: &[u8]) -> Option<bool> {
    let Value::Bool(value) = member(root, key)? else { return None };
    Some(*value)
}
#[must_use]
pub fn read(root: &Value) -> Option<(Sources, Apt)> {
    let enabled = member(root, b"components")?;
    let mut components = List::with_capacity(4);
    for component in Component::ALL {
        if boolean(enabled, component.name()) == Some(true) {
            components.push(component).expect("four components");
        }
    }
    let pockets = member(root, b"pockets")?;
    let snapshot = member(root, b"snapshot")?;
    let mut security = None;
    if boolean(pockets, b"security")? {
        security = Some(text(snapshot, b"security")?);
    }
    Some((
        Sources {
            suite: text(root, b"suite")?,
            architecture: text(root, b"architecture")?,
            components: components.into_boxed(),
            archive: text(snapshot, b"archive")?,
            updates: boolean(pockets, b"updates")?,
            security,
        },
        Apt { install_recommends: boolean(root, b"install_recommends")? },
    ))
}
pub fn build(
    root: &Value,
    placement: &revision::Placement,
    expansion: &Expansion,
    set: Option<&Place>,
) -> Result<Packages, Box<[Error]>> {
    let Some((sources, apt)) = read(root) else {
        return Err(Box::new([error(Schema::Missing { setting: bytes::copy_of(b"debian") }, None, None, None)]));
    };
    let entries = expansion.of(Map::Packages);
    let capacity = u32::try_from(entries.len()).expect("bounded expansion");
    let mut names = Names::with_capacity(capacity);
    let mut faults = List::with_capacity(capacity);
    for located in &entries {
        let entry = located.entry;
        let fault = match crate::kinds::packages::entry(entry) {
            Ok(component) => {
                if sources.components.contains(&component) {
                    names.insert(entry.key.clone(), ()).expect("entry count");
                    continue;
                }
                let mut at = entry.at;
                for field in &entry.fields {
                    if field.name == Name::Component {
                        at = field.at;
                    }
                }
                (Schema::Component { package: entry.key.clone(), component: bytes::copy_of(component.name()) }, at)
            }
            Err(fault) => fault,
        };
        let also = if let Schema::Component { .. } = &fault.0 { set.cloned() } else { None };
        faults.push(error(fault.0, Some(located.task), Some(fault.1), also)).expect("entry count");
    }
    if !faults.is_empty() {
        return Err(faults.into_boxed());
    }
    let mut request = List::with_capacity(names.len());
    for (name, ()) in &names {
        request.push(name.clone()).expect("name count");
    }
    let (variant, initramfs) = match placement.kind {
        revision::Runs::Metal => {
            (Variant::Apt, Some(Initramfs { modules: Box::new([]), microcode: Microcode::Generic }))
        }
        revision::Runs::Vm => (Variant::Apt, Some(Initramfs { modules: Box::new([]), microcode: Microcode::None })),
        revision::Runs::Container => (Variant::Container, None),
    };
    let (preseeds, modules) = super::declarations::build(expansion, initramfs.is_some())?;
    let mut initramfs = initramfs;
    if let Some(initramfs) = &mut initramfs {
        initramfs.modules = modules;
    }
    Ok(Packages { initramfs_file: None, sources, request: request.into_boxed(), variant, preseeds, initramfs, apt })
}
pub(in crate::parts) fn error(
    schema: Schema,
    task: Option<&[u8]>,
    position: Option<revision::Position>,
    also: Option<Place>,
) -> Error {
    let mut at = None;
    let mut name = None;
    if let Some(task) = task {
        let mut path = Writer::new(task.len().saturating_add(16));
        path.put(b"tasks/").expect("prefix");
        path.put(task).expect("task");
        path.put(b"/task.json").expect("suffix");
        at = Some(Place { path: path.finish(), position });
        name = Some(bytes::copy_of(task));
    }
    Error { side: None, kind: Kind::Schema(schema), class: Class::Configuration, at, also, target: None, task: name }
}
