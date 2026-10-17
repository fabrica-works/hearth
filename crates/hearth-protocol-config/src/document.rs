//! Configuration layers (format.md, 3.1 to 3.3).
use crate::{
    Error, Fault, Position, Shape,
    json::{Kind, Value},
    names,
};
use alloc::boxed::Box;
use skein_lib::List;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Document {
    pub at: Position,
    pub layer: Layer,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Layer {
    pub tasks: Box<[Task]>,
    pub debian: Option<Value>,
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
    let mut tasks = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
    let mut debian = None;
    for entry in entries {
        match &*entry.key {
            b"debian" => debian = Some(entry.value),
            b"placement" | b"members" => return Err(Error { at: entry.at, fault: Fault::Shape(Shape::Reserved) }),
            _ => {
                if !names::identifier(&entry.key) {
                    return Err(Error { at: entry.at, fault: Fault::Shape(Shape::Name) });
                }
                match &entry.value.kind {
                    Kind::Map(_) | Kind::Null => {}
                    Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::String(_) => {
                        return Err(Error {
                            at: entry.value.at,
                            fault: Fault::Shape(Shape::Settings { task: entry.key }),
                        });
                    }
                }
                tasks.push(Task { name: entry.key, at: entry.at, value: entry.value }).expect("one task per entry");
            }
        }
    }
    Ok(Document { at: value.at, layer: Layer { tasks: tasks.into_boxed(), debian } })
}
