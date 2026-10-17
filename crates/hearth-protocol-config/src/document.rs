//! Configuration layers (format.md, 3.1 to 3.6).
use crate::{
    Error, Fault, Field, Position, Shape,
    json::{Entry, Kind, Value},
    names,
};
use alloc::boxed::Box;
use skein_lib::List;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Document {
    pub at: Position,
    pub layer: Layer,
    pub members: Option<Members>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Layer {
    pub tasks: Box<[Task]>,
    pub debian: Option<Value>,
    pub placement: Option<Placement>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Task {
    pub name: Box<[u8]>,
    pub at: Position,
    pub value: Value,
}
pub fn document(value: Value) -> Result<Document, Error> {
    let entries = match value.kind {
        Kind::Map(entries) => entries,
        Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::String(_) => {
            return Err(Error { at: value.at, fault: Fault::NotObject });
        }
    };
    let mut members = None;
    let mut layer = empty_layer();
    let mut tasks = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
    for entry in entries {
        if &*entry.key == b"members" {
            members = Some(read_members(entry.value)?);
        } else {
            read_field(entry, &mut layer, &mut tasks)?;
        }
    }
    layer.tasks = tasks.into_boxed();
    Ok(Document { at: value.at, layer, members })
}
fn empty_layer() -> Layer {
    Layer { tasks: Box::new([]), debian: None, placement: None }
}
fn read_layer(entries: Box<[Entry]>) -> Result<Layer, Error> {
    let mut layer = empty_layer();
    let mut tasks = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
    for entry in entries {
        read_field(entry, &mut layer, &mut tasks)?;
    }
    layer.tasks = tasks.into_boxed();
    Ok(layer)
}
fn read_field(entry: Entry, layer: &mut Layer, tasks: &mut List<Task>) -> Result<(), Error> {
    match &*entry.key {
        b"debian" => layer.debian = Some(entry.value),
        b"placement" => layer.placement = Some(read_placement(entry.value)?),
        b"members" => return Err(shape(entry.at, Shape::Member)),
        _ => {
            if !names::identifier(&entry.key) {
                return Err(shape(entry.at, Shape::Name));
            }
            match &entry.value.kind {
                Kind::Map(_) | Kind::Null => {}
                Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::String(_) => {
                    return Err(shape(entry.value.at, Shape::Settings { task: entry.key }));
                }
            }
            tasks.push(Task { name: entry.key, at: entry.at, value: entry.value }).expect("one task per entry");
        }
    }
    Ok(())
}
/// Members in name order, at the map's opening brace (format.md, 3.6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Members {
    pub at: Position,
    pub each: Box<[Member]>,
}
/// One member's layer, or a removal, at its name.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Member {
    pub name: Box<[u8]>,
    pub at: Position,
    pub layer: Option<Layer>,
}
fn read_members(value: Value) -> Result<Members, Error> {
    let Kind::Map(entries) = value.kind else {
        return Err(shape(value.at, Shape::Members));
    };
    let mut each = List::with_capacity(u32::try_from(entries.len()).expect("members bounded by u32"));
    for entry in entries {
        if !names::member(&entry.key) {
            return Err(shape(entry.at, Shape::Member));
        }
        let layer = match entry.value.kind {
            Kind::Null => None,
            Kind::Map(entries) => Some(read_layer(entries)?),
            Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::String(_) => {
                return Err(shape(entry.value.at, Shape::Members));
            }
        };
        each.push(Member { name: entry.key, at: entry.at, layer }).expect("one member per entry");
    }
    Ok(Members { at: value.at, each: each.into_boxed() })
}

/// A whole placement, replaced by a later layer (format.md, 3.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Placement {
    pub at: Position,
    pub kind: Runs,
    pub firmware: Option<Firmware>,
    pub hardware: Option<Box<[u8]>>,
    pub on: Option<On>,
    pub mutable: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Runs {
    Metal,
    Vm,
    Container,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Firmware {
    Efi,
    Bios,
}
/// A container's host, as written, at the target value.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct On {
    pub target: Box<[u8]>,
    pub at: Position,
}
fn shape(at: Position, shape: Shape) -> Error {
    Error { at, fault: Fault::Shape(shape) }
}
fn text(value: Value, field: Field) -> Result<Box<[u8]>, Error> {
    match value.kind {
        Kind::String(text) => Ok(text),
        Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::Map(_) => {
            Err(shape(value.at, Shape::Wrong { field }))
        }
    }
}
fn read_kind(value: &Value) -> Result<Runs, Error> {
    if let Kind::String(name) = &value.kind {
        match &**name {
            b"metal" => return Ok(Runs::Metal),
            b"vm" => return Ok(Runs::Vm),
            b"container" => return Ok(Runs::Container),
            _ => {}
        }
    }
    Err(shape(value.at, Shape::Wrong { field: Field::Kind }))
}
fn read_placement(value: Value) -> Result<Placement, Error> {
    let Kind::Map(entries) = value.kind else {
        return Err(shape(value.at, Shape::Placement));
    };
    let mut kind = None;
    for entry in &entries {
        if &*entry.key == b"kind" {
            kind = Some(read_kind(&entry.value)?);
        }
    }
    let kind = kind.ok_or(shape(value.at, Shape::Missing { field: Field::Kind }))?;
    let mut placement = Placement { at: value.at, kind, firmware: None, hardware: None, on: None, mutable: false };
    for entry in entries {
        let field = match &*entry.key {
            b"kind" => continue,
            b"firmware" => Field::Firmware,
            b"hardware" => Field::Hardware,
            b"on" => Field::On,
            b"mutable" if placement.kind != Runs::Container => Field::Mutable,
            _ => return Err(shape(entry.at, Shape::Unknown { field: entry.key })),
        };
        if (placement.kind == Runs::Container && (field == Field::Firmware || field == Field::Hardware))
            || (placement.kind != Runs::Container && field == Field::On)
        {
            return Err(shape(entry.at, Shape::Misplaced { field }));
        }
        match field {
            Field::Firmware => {
                let at = entry.value.at;
                placement.firmware = Some(match &*text(entry.value, field)? {
                    b"efi" => Firmware::Efi,
                    b"bios" => Firmware::Bios,
                    _ => return Err(shape(at, Shape::Wrong { field })),
                });
            }
            Field::Hardware => placement.hardware = Some(text(entry.value, field)?),
            Field::On => placement.on = Some(On { at: entry.value.at, target: text(entry.value, field)? }),
            Field::Mutable => {
                let Kind::Bool(mutable) = entry.value.kind else {
                    return Err(shape(entry.value.at, Shape::Wrong { field }));
                };
                placement.mutable = mutable;
            }
            Field::Kind => unreachable!("kind decoded before fields"),
        }
    }
    for (field, missing) in [
        (Field::Firmware, placement.kind != Runs::Container && placement.firmware.is_none()),
        (Field::Hardware, placement.kind != Runs::Container && placement.hardware.is_none()),
        (Field::On, placement.kind == Runs::Container && placement.on.is_none()),
    ] {
        if missing {
            return Err(shape(value.at, Shape::Missing { field }));
        }
    }
    Ok(placement)
}
