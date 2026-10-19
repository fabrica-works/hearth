//! Group declarations and bounded account ids (protocol/format.md, 5.6).
use super::secrets::fault;
use crate::{
    entries::{Concrete, Located},
    errors::{Error, Schema},
    revision::{Position, declaration::Name},
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Group {
    pub name: Box<[u8]>,
    pub task: Box<[u8]>,
    pub at: Position,
    pub gid: Option<u32>,
    pub package: bool,
}
pub(crate) fn entry(located: &Located<'_>) -> Result<Group, Box<Error>> {
    match checked(located) {
        Ok(group) => Ok(group),
        Err((name, schema)) => Err(Box::new(fault(located, Some(name), schema))),
    }
}
fn checked(located: &Located<'_>) -> Result<Group, (Name, Schema)> {
    Ok(Group {
        name: located.entry.key.clone(),
        task: copy_of(located.task),
        at: located.entry.at,
        package: package(located.entry)?,
        gid: id(located.entry, Name::Gid)?,
    })
}

pub(super) fn package(entry: &Concrete) -> Result<bool, (Name, Schema)> {
    match entry.text(Name::From).unwrap_or(b"sysusers") {
        b"sysusers" => Ok(false),
        b"package" => Ok(true),
        value => Err((
            Name::From,
            Schema::Enum { value: copy_of(value), listed: Box::new([copy_of(b"sysusers"), copy_of(b"package")]) },
        )),
    }
}
pub(super) fn bounded(value: i64, min: i64, max: i64) -> Result<u32, Schema> {
    if value < min {
        Err(Schema::Below { min })
    } else if value > max {
        Err(Schema::Above { max })
    } else {
        Ok(u32::try_from(value).expect("bounded id"))
    }
}
pub(super) fn id(entry: &Concrete, name: Name) -> Result<Option<u32>, (Name, Schema)> {
    let Some(value) = entry.int(name) else { return Ok(None) };
    match bounded(value, 1, 65533) {
        Ok(value) => Ok(Some(value)),
        Err(schema) => Err((name, schema)),
    }
}
