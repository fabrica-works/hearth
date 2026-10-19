//! Decoded values and compiled text charged by domain/render.md, 2.3.
pub(crate) mod declaration;
pub(crate) mod template;
use super::{Document, Kind, Layer, Value};
use crate::{evaluate::ELEMENT, limits::Limits};
use skein_lib::Stack;

pub(crate) fn document(document: &Document, limits: &Limits) -> u64 {
    let mut bytes = layer(&document.layer, limits);
    if let Some(members) = &document.members {
        bytes = bytes.saturating_add(ELEMENT).saturating_add(7);
        for member in &members.each {
            bytes = bytes.saturating_add(entry(&member.name));
            if let Some(member) = &member.layer {
                bytes = bytes.saturating_add(layer(member, limits));
            }
        }
    }
    bytes
}
fn layer(layer: &Layer, limits: &Limits) -> u64 {
    let mut bytes = 0_u64;
    if let Some(debian) = &layer.debian {
        bytes = bytes.saturating_add(ELEMENT).saturating_add(6).saturating_add(value(debian, limits));
    }
    if let Some(placement) = &layer.placement {
        bytes = bytes.saturating_add(ELEMENT).saturating_add(9);
        let kind: &[u8] = match placement.kind {
            super::Runs::Metal => b"metal",
            super::Runs::Vm => b"vm",
            super::Runs::Container => b"container",
        };
        bytes = bytes.saturating_add(entry(b"kind")).saturating_add(size(kind));
        if let Some(firmware) = placement.firmware {
            let name: &[u8] = match firmware {
                super::Firmware::Efi => b"efi",
                super::Firmware::Bios => b"bios",
            };
            bytes = bytes.saturating_add(entry(b"firmware")).saturating_add(size(name));
        }
        if let Some(hardware) = &placement.hardware {
            bytes = bytes.saturating_add(entry(b"hardware")).saturating_add(size(hardware));
        }
        if let Some(on) = &placement.on {
            bytes = bytes.saturating_add(entry(b"on")).saturating_add(size(&on.target));
        }
        if placement.mutable {
            bytes = bytes.saturating_add(entry(b"mutable"));
        }
    }
    for task in &layer.tasks {
        bytes = bytes.saturating_add(entry(&task.name)).saturating_add(value(&task.value, limits));
    }
    bytes
}
fn entry(key: &[u8]) -> u64 {
    ELEMENT.saturating_add(size(key))
}
fn size(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).expect("decoded bytes fit u64")
}
// A cursor per open container, rather than one pending reference per child.
pub(crate) fn value(root: &Value, limits: &Limits) -> u64 {
    let mut pending = Stack::with_capacity(limits.depth.saturating_add(1));
    pending.push((root, 0_usize)).expect("root cursor fits");
    let mut bytes = 0_u64;
    for _ in 0..u64::from(limits.document).saturating_mul(3).saturating_add(1) {
        let Some((node, index)) = pending.pop() else { return bytes };
        if index == 0 {
            bytes = bytes.saturating_add(shallow(node));
        }
        let child = next_child(node, index);
        let Some((child, next)) = child else { continue };
        if pending.push((node, next)).is_err() {
            return u64::MAX;
        }
        if pending.push((child, 0)).is_err() {
            return u64::MAX;
        }
    }
    u64::MAX
}
fn shallow(value: &Value) -> u64 {
    match &value.kind {
        Kind::Null | Kind::Bool(_) | Kind::Int(_) => 0,
        Kind::String(bytes) => size(bytes),
        Kind::List(items) => u64::try_from(items.len()).expect("bounded values").saturating_mul(ELEMENT),
        Kind::Map(pairs) => {
            let mut bytes = 0_u64;
            for pair in pairs {
                bytes = bytes.saturating_add(entry(&pair.key));
            }
            bytes
        }
    }
}

fn next_child(node: &Value, next: usize) -> Option<(&Value, usize)> {
    match &node.kind {
        Kind::List(items) => {
            for (index, item) in items.iter().enumerate().skip(next) {
                if owns(item) {
                    return Some((item, index.saturating_add(1)));
                }
            }
        }
        Kind::Map(pairs) => {
            for (index, pair) in pairs.iter().enumerate().skip(next) {
                if owns(&pair.value) {
                    return Some((&pair.value, index.saturating_add(1)));
                }
            }
        }
        Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::String(_) => {}
    }
    None
}
fn owns(value: &Value) -> bool {
    match value.kind {
        Kind::Null | Kind::Bool(_) | Kind::Int(_) => false,
        Kind::String(_) | Kind::List(_) | Kind::Map(_) => true,
    }
}
