use crate::{
    entries::{Expansion, Fielded, Made},
    errors::Error,
    parts::live,
    revision::declaration::{Map, Name},
};
use skein_lib::bytes::copy_of;
pub fn live_entry(action: &[u8], unit: Option<&[u8]>, paths: &[&[u8]]) -> Expansion {
    let mut fields = skein_lib::List::with_capacity(2);
    fields.push((Name::Action, action)).expect("action");
    if let Some(unit) = unit {
        fields.push((Name::Unit, unit)).expect("unit");
    }
    let mut expansion = super::secret_entry(Map::Live, b"consumer", &fields.into_boxed());
    for task in &mut expansion.tasks {
        for group in &mut task.groups {
            for entry in &mut group.entries {
                let mut fields = skein_lib::List::with_capacity(3);
                for field in &entry.fields {
                    fields.push(field.clone()).expect("field");
                }
                let mut strings = skein_lib::List::with_capacity(u32::try_from(paths.len()).expect("path count"));
                for path in paths {
                    strings.push(copy_of(path)).expect("path");
                }
                fields
                    .push(Made {
                        name: Name::Paths,
                        at: super::FIELD_AT,
                        value: Fielded::Strings(strings.into_boxed()),
                    })
                    .expect("paths");
                entry.fields = fields.into_boxed();
            }
        }
    }
    expansion
}
pub fn live_fault(action: &[u8], unit: Option<&[u8]>, paths: &[&[u8]]) -> Error {
    super::only(&live::build(&live_entry(action, unit, paths)).expect_err("invalid live entry")).clone()
}
