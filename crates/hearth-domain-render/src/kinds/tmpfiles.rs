//! Typed tmpfiles lines; reserved paths and collisions belong to later passes.
use super::secrets::{fault, mode, required};
use crate::{
    entries::Located,
    errors::{Error, Schema},
    parts::config::{Line, LineKind, Tmpfile},
    revision::declaration::Name,
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
pub(crate) fn entry(located: &Located<'_>) -> Result<Tmpfile, Box<Error>> {
    let kind = match required(located.entry, Name::Type) {
        Ok(value) => match kind(value) {
            Some(kind) => kind,
            None => {
                let mut listed = List::with_capacity(u32::try_from(KINDS.len()).expect("fourteen types"));
                for kind in KINDS {
                    listed.push(copy_of(kind.name())).expect("fourteen types");
                }
                return Err(Box::new(fault(
                    located,
                    Some(Name::Type),
                    Schema::Enum { value: copy_of(value), listed: listed.into_boxed() },
                )));
            }
        },
        Err(schema) => return Err(Box::new(fault(located, None, schema))),
    };
    let mode = if located.entry.text(Name::Mode).is_some() {
        match mode(located.entry) {
            Ok(mode) => Some(mode),
            Err(schema) => return Err(Box::new(fault(located, Some(Name::Mode), schema))),
        }
    } else {
        None
    };
    Ok(Tmpfile {
        path: located.entry.key.clone(),
        line: Line {
            kind,
            mode,
            owner: optional(located, Name::Owner),
            group: optional(located, Name::Group),
            age: optional(located, Name::Age),
            argument: optional(located, Name::Argument),
        },
    })
}
fn optional(located: &Located<'_>, name: Name) -> Option<Box<[u8]>> {
    Some(copy_of(located.entry.text(name)?))
}
const KINDS: &[LineKind] = &[
    LineKind::D,
    LineKind::CapitalD,
    LineKind::E,
    LineKind::F,
    LineKind::FPlus,
    LineKind::W,
    LineKind::WPlus,
    LineKind::CapitalL,
    LineKind::CapitalLPlus,
    LineKind::CapitalC,
    LineKind::Z,
    LineKind::CapitalZ,
    LineKind::R,
    LineKind::CapitalR,
];
fn kind(value: &[u8]) -> Option<LineKind> {
    for kind in KINDS {
        if value == kind.name() {
            return Some(*kind);
        }
    }
    None
}
