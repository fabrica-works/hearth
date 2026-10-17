use crate::{records::parts::data as target, render::parts::data as source};
pub(super) fn back(data: target::Data) -> source::Data {
    let mut entries = skein_lib::List::with_capacity(u32::try_from(data.entries.len()).expect("entries"));
    for target::DataEntry { name, value } in data.entries {
        entries.push(source::DataEntry { name, value: value_back(value) }).expect("entries");
    }
    source::Data { entries: entries.into_boxed() }
}
fn value_back(value: target::DataValue) -> source::DataValue {
    source::DataValue {
        class: match value.class {
            target::DataClass::Core => source::DataClass::Core,
            target::DataClass::Data => source::DataClass::Data,
            target::DataClass::Cache => source::DataClass::Cache,
        },
        path: value.path,
    }
}
pub(super) fn change_back(change: target::DataChange) -> source::DataChange {
    let mut entries = skein_lib::List::with_capacity(u32::try_from(change.entries.len()).expect("entries"));
    for entry in change.entries {
        entries
            .push(match entry {
                target::DataKeyed::Added { key, value } => source::DataKeyed::Added { key, value: value_back(value) },
                target::DataKeyed::Removed { key, value } => {
                    source::DataKeyed::Removed { key, value: value_back(value) }
                }
                target::DataKeyed::Changed { key, before, after } => {
                    source::DataKeyed::Changed { key, before: value_back(before), after: value_back(after) }
                }
            })
            .expect("entries");
    }
    source::DataChange { entries: entries.into_boxed() }
}
#[test]
fn each_data_class_translates_to_the_records_and_back() {
    for (row, input) in crate::tests::fixtures::data::parts().into_iter().enumerate() {
        assert_eq!(back(crate::translate::parts::data::data(input.clone())), input, "row {row}: data part");
    }
}
#[test]
fn each_data_change_translates_to_the_records_and_back() {
    for (row, input) in crate::tests::fixtures::data::changes().into_iter().enumerate() {
        assert_eq!(change_back(crate::translate::parts::data::change(input.clone())), input, "row {row}: data change");
    }
}
