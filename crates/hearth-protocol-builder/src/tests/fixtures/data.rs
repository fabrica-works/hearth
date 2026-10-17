use crate::render::parts::data::{Data, DataChange, DataClass, DataEntry, DataKeyed, DataValue};
fn value(class: DataClass) -> DataValue {
    DataValue { class, path: b"/var/lib/app".as_slice().into() }
}
#[must_use]
pub fn parts() -> [Data; 3] {
    [DataClass::Core, DataClass::Data, DataClass::Cache]
        .map(|class| Data { entries: Box::new([DataEntry { name: b"app".as_slice().into(), value: value(class) }]) })
}
#[must_use]
pub fn changes() -> [DataChange; 3] {
    [
        DataKeyed::Added { key: b"app".as_slice().into(), value: value(DataClass::Core) },
        DataKeyed::Removed { key: b"app".as_slice().into(), value: value(DataClass::Data) },
        DataKeyed::Changed {
            key: b"app".as_slice().into(),
            before: value(DataClass::Core),
            after: value(DataClass::Cache),
        },
    ]
    .map(|entry| DataChange { entries: Box::new([entry]) })
}
