//! Exhaustive data record translations.
use crate::{records::parts::data as target, render::parts::data as source};
use skein_lib::List;
#[must_use]
pub fn data(data: source::Data) -> target::Data {
    let mut entries = List::with_capacity(u32::try_from(data.entries.len()).expect("entry bound"));
    for source::DataEntry { name, value: row } in data.entries {
        entries.push(target::DataEntry { name, value: value(row) }).expect("entry bound");
    }
    target::Data { entries: entries.into_boxed() }
}
#[must_use]
pub fn value(value: source::DataValue) -> target::DataValue {
    target::DataValue {
        class: match value.class {
            source::DataClass::Core => target::DataClass::Core,
            source::DataClass::Data => target::DataClass::Data,
            source::DataClass::Cache => target::DataClass::Cache,
        },
        path: value.path,
    }
}
#[must_use]
pub fn change(change: source::DataChange) -> target::DataChange {
    let mut entries = List::with_capacity(u32::try_from(change.entries.len()).expect("entry bound"));
    for entry in change.entries {
        entries
            .push(match entry {
                source::DataKeyed::Added { key, value: row } => target::DataKeyed::Added { key, value: value(row) },
                source::DataKeyed::Removed { key, value: row } => target::DataKeyed::Removed { key, value: value(row) },
                source::DataKeyed::Changed { key, before, after } => {
                    target::DataKeyed::Changed { key, before: value(before), after: value(after) }
                }
            })
            .expect("entry bound");
    }
    target::DataChange { entries: entries.into_boxed() }
}
