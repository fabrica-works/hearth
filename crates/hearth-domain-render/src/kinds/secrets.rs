//! Secret names and places, never values (format.md, 9).
use crate::{
    entries::{Concrete, Located},
    errors::{Error, Format, Kind, Place, Schema},
    revision::declaration::Name,
    schema::forms,
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
pub(crate) fn text(entry: &Concrete, name: Name, default: &[u8]) -> Box<[u8]> {
    copy_of(entry.text(name).unwrap_or(default))
}
pub(crate) fn required(entry: &Concrete, name: Name) -> Result<&[u8], Schema> {
    match entry.text(name) {
        Some(value) => Ok(value),
        None => Err(Schema::MissingField { field: copy_of(name.key()) }),
    }
}
pub(crate) fn formed(value: &[u8], format: Format) -> Result<(), Schema> {
    if forms::check(format, value) { Ok(()) } else { Err(Schema::Malformed { format }) }
}
pub(crate) fn mode(entry: &Concrete) -> Result<u16, Schema> {
    let value = entry.text(Name::Mode).unwrap_or(b"0600");
    formed(value, Format::Mode)?;
    let mut mode = 0_u16;
    for byte in value {
        mode = mode
            .checked_mul(8)
            .expect("four octal digits")
            .checked_add(u16::from(byte.saturating_sub(b'0')))
            .expect("octal mode");
    }
    Ok(mode)
}
pub(crate) fn fault(located: &Located<'_>, name: Option<Name>, schema: Schema) -> Error {
    let mut at = located.entry.at;
    for field in &located.entry.fields {
        if Some(field.name) == name {
            at = field.at;
        }
    }
    let mut path = skein_lib::Writer::new(located.task.len().saturating_add(16));
    path.put(b"tasks/").expect("path room");
    path.put(located.task).expect("path room");
    path.put(b"/task.json").expect("path room");
    Error {
        side: None,
        kind: Kind::Schema(schema),
        class: crate::vocabulary::Class::Configuration,
        at: Some(Place { path: path.finish(), position: Some(at) }),
        also: None,
        target: None,
        task: Some(copy_of(located.task)),
    }
}
pub(crate) fn file(located: &Located<'_>) -> Result<crate::parts::secrets::SecretFile, Box<Error>> {
    let entry = located.entry;
    if let Err(schema) = formed(&entry.key, Format::Secret) {
        return Err(Box::new(fault(located, None, schema)));
    }
    let path = match required(entry, Name::Path) {
        Ok(path) => path,
        Err(schema) => return Err(Box::new(fault(located, None, schema))),
    };
    if let Err(schema) = formed(path, Format::Path) {
        return Err(Box::new(fault(located, Some(Name::Path), schema)));
    }
    let mode = match mode(entry) {
        Ok(mode) => mode,
        Err(schema) => return Err(Box::new(fault(located, Some(Name::Mode), schema))),
    };
    Ok(crate::parts::secrets::SecretFile {
        name: entry.key.clone(),
        path: copy_of(path),
        mode,
        owner: text(entry, Name::Owner, b"root"),
        group: text(entry, Name::Group, b"root"),
    })
}
